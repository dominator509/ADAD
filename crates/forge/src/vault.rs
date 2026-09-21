use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use adad_core::{Config, Error, Provider};

pub const CURRENT_CONFIG_VERSION: u32 = 2;

const DEFAULT_IMAGE_SIZE_MIB: u32 = 64;
const CONFIG_RELATIVE_PATH: &str = "config/config.toml";
const REQUIRED_DIRECTORIES: &[&str] = &["config", "identity", "keys", "repos"];

static UNIQUE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct Vault;

pub struct Unsealed {
    image_path: PathBuf,
    loop_device: PathBuf,
    mapper_name: String,
    mount_dir: PathBuf,
    runtime: VaultRuntime,
    sealed: bool,
}

impl Vault {
    pub fn create(path: &Path, passphrase: &str) -> Result<(), Error> {
        validate_image_target(path, true)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(io_error)?;
        }
        validate_image_target(path, true)?;

        let runtime = VaultRuntime::new(passphrase);
        runtime.run(
            Command::new("truncate")
                .arg("--size")
                .arg(format!("{DEFAULT_IMAGE_SIZE_MIB}M"))
                .arg(path),
        )?;

        let loop_device = runtime.attach_loop_device(path)?;
        let mapper_name = mapper_name_for(path);
        let mapped_device = mapped_device_path(&mapper_name);
        let mount_dir = prepare_mount_dir(&mapper_name)?;

        if let Err(err) = runtime.luks_format(&loop_device) {
            let _ = runtime.detach_loop(&loop_device);
            return Err(err);
        }
        if let Err(err) = runtime.open_mapping(&loop_device, &mapper_name) {
            let _ = runtime.detach_loop(&loop_device);
            return Err(err);
        }
        if let Err(err) = runtime.make_filesystem(&mapped_device) {
            let _ = runtime.close_mapping(&mapper_name);
            let _ = runtime.detach_loop(&loop_device);
            return Err(err);
        }
        if let Err(err) = runtime.mount(&mapped_device, &mount_dir) {
            let _ = runtime.close_mapping(&mapper_name);
            let _ = runtime.detach_loop(&loop_device);
            return Err(err);
        }

        let vault = Unsealed {
            image_path: path.to_path_buf(),
            loop_device,
            mapper_name,
            mount_dir,
            runtime,
            sealed: false,
        };

        vault.ensure_layout()?;
        vault.write_config(&default_config())?;
        vault.seal()?;
        Ok(())
    }

    pub fn unlock(path: &Path, passphrase: &str) -> Result<Unsealed, Error> {
        validate_image_target(path, false)?;
        let runtime = VaultRuntime::new(passphrase);
        let loop_device = runtime.attach_loop_device(path)?;
        let mapper_name = mapper_name_for(path);
        let mapped_device = mapped_device_path(&mapper_name);
        let mount_dir = prepare_mount_dir(&mapper_name)?;

        if let Err(err) = runtime.open_mapping(&loop_device, &mapper_name) {
            let _ = runtime.detach_loop(&loop_device);
            return Err(err);
        }
        if let Err(err) = runtime.mount(&mapped_device, &mount_dir) {
            let _ = runtime.close_mapping(&mapper_name);
            let _ = runtime.detach_loop(&loop_device);
            return Err(err);
        }

        let vault = Unsealed {
            image_path: path.to_path_buf(),
            loop_device,
            mapper_name,
            mount_dir,
            runtime,
            sealed: false,
        };
        vault.ensure_layout()?;
        Ok(vault)
    }

    pub fn upgrade_in_place(path: &Path, passphrase: &str) -> Result<PathBuf, Error> {
        validate_image_target(path, false)?;
        let backup_path = backup_path_for(path);
        fs::copy(path, &backup_path).map_err(io_error)?;

        let vault = Self::unlock(path, passphrase)?;
        let mut config = vault.load_config()?;

        if config.config_version > CURRENT_CONFIG_VERSION {
            return Err(Error::VaultVersion);
        }

        if config.config_version < CURRENT_CONFIG_VERSION {
            config.config_version = CURRENT_CONFIG_VERSION;
            vault.write_config(&config)?;
        }

        vault.seal()?;
        Ok(backup_path)
    }

    #[must_use]
    pub fn backup_path_for(path: &Path) -> PathBuf {
        backup_path_for(path)
    }
}

impl Unsealed {
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.mount_dir
    }

    #[must_use]
    pub fn image_path(&self) -> &Path {
        &self.image_path
    }

    #[must_use]
    pub fn mapper_name(&self) -> &str {
        &self.mapper_name
    }

    pub fn write_bytes(&self, relative_path: &Path, bytes: &[u8]) -> Result<(), Error> {
        let path = self.resolve_relative_path(relative_path)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(io_error)?;
        }
        fs::write(path, bytes).map_err(io_error)
    }

    pub fn read_bytes(&self, relative_path: &Path) -> Result<Vec<u8>, Error> {
        let path = self.resolve_relative_path(relative_path)?;
        fs::read(path).map_err(io_error)
    }

    pub fn write_config_text(&self, text: &str) -> Result<(), Error> {
        self.write_bytes(Path::new(CONFIG_RELATIVE_PATH), text.as_bytes())
    }

    pub fn load_config(&self) -> Result<Config, Error> {
        let config = Config::from_bytes(&self.read_bytes(Path::new(CONFIG_RELATIVE_PATH))?)?;
        if config.config_version > CURRENT_CONFIG_VERSION {
            return Err(Error::VaultVersion);
        }
        Ok(config)
    }

    pub fn write_config(&self, config: &Config) -> Result<(), Error> {
        self.write_config_text(&render_config(config))
    }

    pub fn seal(mut self) -> Result<(), Error> {
        // Only tear down a mount point we still own: if the directory was
        // swapped for a symlink, unmounting and recursively deleting through
        // it could destroy an unrelated tree. The mapping and loop device are
        // still released below via their own names.
        if mount_dir_is_plain_dir(&self.mount_dir) {
            self.runtime.unmount(&self.mount_dir)?;
            fs::remove_dir_all(&self.mount_dir).map_err(io_error)?;
        }
        self.runtime.close_mapping(&self.mapper_name)?;
        self.runtime.detach_loop(&self.loop_device)?;
        self.sealed = true;
        Ok(())
    }

    fn ensure_layout(&self) -> Result<(), Error> {
        for directory in REQUIRED_DIRECTORIES {
            fs::create_dir_all(self.mount_dir.join(directory)).map_err(io_error)?;
        }
        Ok(())
    }

    fn resolve_relative_path(&self, relative_path: &Path) -> Result<PathBuf, Error> {
        if relative_path.is_absolute()
            || relative_path.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err(Error::Io);
        }

        Ok(self.mount_dir.join(relative_path))
    }
}

impl Drop for Unsealed {
    fn drop(&mut self) {
        if !self.sealed {
            if mount_dir_is_plain_dir(&self.mount_dir) {
                let _ = self.runtime.unmount(&self.mount_dir);
                let _ = fs::remove_dir_all(&self.mount_dir);
            }
            let _ = self.runtime.close_mapping(&self.mapper_name);
            let _ = self.runtime.detach_loop(&self.loop_device);
        }
    }
}

struct VaultRuntime {
    passphrase: SensitiveBytes,
}

impl VaultRuntime {
    fn new(passphrase: &str) -> Self {
        Self {
            passphrase: SensitiveBytes::new(passphrase.as_bytes()),
        }
    }

    fn attach_loop_device(&self, image_path: &Path) -> Result<PathBuf, Error> {
        let output = self.output(
            Command::new("losetup")
                .arg("--find")
                .arg("--show")
                .arg(image_path),
        )?;
        let loop_device = String::from_utf8(output.stdout).map_err(|_| Error::Io)?;
        Ok(PathBuf::from(loop_device.trim()))
    }

    fn detach_loop(&self, loop_device: &Path) -> Result<(), Error> {
        self.run(Command::new("losetup").arg("--detach").arg(loop_device))
    }

    fn luks_format(&self, loop_device: &Path) -> Result<(), Error> {
        self.run_with_passphrase(
            Command::new("cryptsetup")
                .arg("luksFormat")
                .arg("--type")
                .arg("luks2")
                .arg("--pbkdf")
                .arg("argon2id")
                .arg("--batch-mode")
                .arg("--key-file")
                .arg("-")
                .arg(loop_device),
        )
    }

    fn open_mapping(&self, loop_device: &Path, mapper_name: &str) -> Result<(), Error> {
        self.run_with_passphrase(
            Command::new("cryptsetup")
                .arg("open")
                .arg("--type")
                .arg("luks2")
                .arg("--key-file")
                .arg("-")
                .arg(loop_device)
                .arg(mapper_name),
        )
    }

    fn close_mapping(&self, mapper_name: &str) -> Result<(), Error> {
        self.run(Command::new("cryptsetup").arg("close").arg(mapper_name))
    }

    fn make_filesystem(&self, mapped_device: &Path) -> Result<(), Error> {
        self.run(Command::new("mkfs.ext4").arg("-F").arg(mapped_device))
    }

    fn mount(&self, mapped_device: &Path, mount_dir: &Path) -> Result<(), Error> {
        let mut command = Command::new("mount");
        command.args(mount_args(mapped_device, mount_dir));
        self.run(&mut command)
    }

    fn unmount(&self, mount_dir: &Path) -> Result<(), Error> {
        self.run(Command::new("umount").arg(mount_dir))
    }

    fn run(&self, command: &mut Command) -> Result<(), Error> {
        let status = command.status().map_err(io_error)?;
        if status.success() {
            Ok(())
        } else {
            Err(Error::Io)
        }
    }

    fn run_with_passphrase(&self, command: &mut Command) -> Result<(), Error> {
        let mut child = command.stdin(Stdio::piped()).spawn().map_err(io_error)?;
        let mut stdin = child.stdin.take().ok_or(Error::Io)?;
        if stdin.write_all(&self.passphrase.0).is_err() {
            let _ = child.kill();
            let _ = child.wait();
            return Err(Error::Io);
        }
        drop(stdin);

        let status = child.wait().map_err(io_error)?;
        if status.success() {
            Ok(())
        } else {
            Err(Error::Io)
        }
    }

    fn output(&self, command: &mut Command) -> Result<std::process::Output, Error> {
        let output = command.output().map_err(io_error)?;
        if output.status.success() {
            Ok(output)
        } else {
            Err(Error::Io)
        }
    }
}

impl Drop for VaultRuntime {
    fn drop(&mut self) {
        self.passphrase.zeroize();
    }
}

struct SensitiveBytes(Vec<u8>);

impl SensitiveBytes {
    fn new(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }

    fn zeroize(&mut self) {
        for byte in &mut self.0 {
            *byte = 0;
        }
    }
}

impl Drop for SensitiveBytes {
    fn drop(&mut self) {
        self.zeroize();
    }
}

fn default_config() -> Config {
    Config {
        config_version: CURRENT_CONFIG_VERSION,
        provider: Provider::Local,
        local_base_url: None,
        openai_base_url: None,
        venice_base_url: None,
        venice_allow_anonymized: false,
        openai_api_key: None,
        venice_api_key: None,
        model: None,
        dms_window_hours: None,
        wg_conf: None,
        monero_rpc_url: None,
    }
}

fn render_config(config: &Config) -> String {
    let mut lines = vec![
        format!("config_version = {}", config.config_version),
        format!("provider = \"{}\"", config.provider.as_str()),
    ];

    push_optional_string(
        &mut lines,
        "local_base_url",
        config.local_base_url.as_deref(),
    );
    push_optional_string(
        &mut lines,
        "openai_base_url",
        config.openai_base_url.as_deref(),
    );
    push_optional_string(
        &mut lines,
        "venice_base_url",
        config.venice_base_url.as_deref(),
    );
    if config.venice_allow_anonymized {
        lines.push("venice_allow_anonymized = true".to_string());
    }
    push_optional_secret(&mut lines, "openai_api_key", config.openai_api_key.as_ref());
    push_optional_secret(&mut lines, "venice_api_key", config.venice_api_key.as_ref());
    push_optional_string(&mut lines, "model", config.model.as_deref());
    if let Some(hours) = config.dms_window_hours {
        lines.push(format!("dms_window_hours = {hours}"));
    }
    push_optional_secret(&mut lines, "wg_conf", config.wg_conf.as_ref());
    push_optional_string(
        &mut lines,
        "monero_rpc_url",
        config.monero_rpc_url.as_deref(),
    );
    lines.join("\n") + "\n"
}

fn push_optional_string(lines: &mut Vec<String>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        lines.push(format!("{key} = \"{}\"", escape_toml_string(value)));
    }
}

fn push_optional_secret(
    lines: &mut Vec<String>,
    key: &str,
    value: Option<&adad_core::SecretString>,
) {
    if let Some(value) = value {
        lines.push(format!(
            "{key} = \"{}\"",
            escape_toml_string(value.expose())
        ));
    }
}

fn escape_toml_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\u{0008}', "\\b")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\u{000C}', "\\f")
        .replace('\r', "\\r")
}

fn mapped_device_path(mapper_name: &str) -> PathBuf {
    Path::new("/dev/mapper").join(mapper_name)
}

fn mapper_name_for(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or("vault")
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>();

    format!("adad-{stem}-{}", std::process::id())
}

fn unique_mount_dir(mapper_name: &str) -> PathBuf {
    env::temp_dir().join(format!("{mapper_name}-mount-{}", unique_suffix()))
}

/// Create the vault mount point exclusively with owner-only permissions.
///
/// `create_dir` (not `create_dir_all`) fails if the path already exists, so a
/// pre-planted symlink in the world-writable temp dir cannot redirect the
/// mount or the later teardown. The canonicalized path is returned so later
/// containment checks compare against the real location.
fn prepare_mount_dir(mapper_name: &str) -> Result<PathBuf, Error> {
    let mount_dir = unique_mount_dir(mapper_name);
    fs::create_dir(&mount_dir).map_err(|_| Error::Io)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&mount_dir, fs::Permissions::from_mode(0o700))
            .map_err(|_| Error::Io)?;
    }
    fs::canonicalize(&mount_dir).map_err(|_| Error::Io)
}

/// Mount arguments for the vault filesystem. `nosuid,nodev` keeps a crafted
/// vault image from smuggling setuid binaries or device nodes onto the host.
fn mount_args(mapped_device: &Path, mount_dir: &Path) -> Vec<std::ffi::OsString> {
    vec![
        "-t".into(),
        "ext4".into(),
        "-o".into(),
        "nosuid,nodev".into(),
        mapped_device.into(),
        mount_dir.into(),
    ]
}

/// True only when `path` is a real directory, not a symlink. Teardown must
/// never follow a swapped-in symlink with a recursive delete.
fn mount_dir_is_plain_dir(path: &Path) -> bool {
    matches!(
        fs::symlink_metadata(path),
        Ok(metadata) if metadata.file_type().is_dir()
    )
}

fn backup_path_for(path: &Path) -> PathBuf {
    let mut backup_name = path
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("vault.img")
        .to_string();
    backup_name.push_str(".bak");
    path.with_file_name(backup_name)
}

fn unique_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let counter = UNIQUE_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{nanos}-{counter}")
}

fn io_error(_: std::io::Error) -> Error {
    Error::Io
}

fn validate_image_target(path: &Path, allow_missing: bool) -> Result<(), Error> {
    if path.as_os_str().is_empty() {
        return Err(Error::Io);
    }

    let target_result = match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => Ok(()),
        Ok(_) => Err(Error::Io),
        Err(error) if allow_missing && error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(Error::Io),
    };
    target_result?;

    let mut ancestor = path.parent();
    while let Some(directory) = ancestor {
        match fs::symlink_metadata(directory) {
            Ok(metadata) if metadata.file_type().is_dir() => {}
            Ok(_) => return Err(Error::Io),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(Error::Io),
        }
        ancestor = directory.parent();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::{
        backup_path_for, default_config, render_config, validate_image_target, Vault,
        CURRENT_CONFIG_VERSION,
    };

    #[test]
    fn backup_paths_keep_the_original_name_and_add_bak_suffix() {
        assert_eq!(
            backup_path_for(Path::new("/tmp/vault.img")),
            Path::new("/tmp/vault.img.bak")
        );
    }

    #[test]
    fn config_renderer_round_trips_the_default_shape() {
        let config = default_config();
        let rendered = render_config(&config);
        let parsed = adad_core::Config::from_toml_str(&rendered).expect("rendered config parses");

        assert_eq!(parsed.config_version, CURRENT_CONFIG_VERSION);
        assert_eq!(parsed.provider, config.provider);
    }

    #[test]
    fn config_renderer_round_trips_escaped_values() {
        let mut config = default_config();
        config.model = Some("line\nquote\"slash\\".to_owned());

        let rendered = render_config(&config);
        let parsed = adad_core::Config::from_toml_str(&rendered).expect("escaped config parses");

        assert_eq!(parsed.model, config.model);
    }

    #[test]
    fn mount_args_disable_setuid_and_device_nodes() {
        use super::mount_args;

        let args = mount_args(Path::new("/dev/mapper/adad-x"), Path::new("/tmp/mnt"));
        let rendered: Vec<String> = args
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();

        let options_index = rendered
            .iter()
            .position(|arg| arg == "-o")
            .expect("mount carries -o");
        assert_eq!(rendered[options_index + 1], "nosuid,nodev");
        assert!(rendered.contains(&"/dev/mapper/adad-x".to_owned()));
    }

    #[test]
    fn prepared_mount_dir_is_exclusive_and_owner_only() {
        use std::os::unix::fs::PermissionsExt;

        use super::{mount_dir_is_plain_dir, prepare_mount_dir};

        let first = prepare_mount_dir("adad-test-mount").expect("mount dir prepares");
        let metadata = fs::symlink_metadata(&first).expect("mount dir exists");
        assert!(metadata.file_type().is_dir());
        assert_eq!(metadata.permissions().mode() & 0o777, 0o700);
        assert!(mount_dir_is_plain_dir(&first));

        fs::remove_dir(&first).expect("fixture cleanup");
    }

    #[test]
    fn mount_dir_guard_rejects_symlinks() {
        use std::os::unix::fs::symlink;

        use super::mount_dir_is_plain_dir;

        let root = std::env::temp_dir().join(format!(
            "adad-forge-guard-{}",
            super::unique_suffix()
        ));
        fs::create_dir(&root).expect("fixture root creates");
        let target = root.join("real");
        fs::create_dir(&target).expect("target creates");
        let link = root.join("link");
        symlink(&target, &link).expect("symlink creates");

        assert!(mount_dir_is_plain_dir(&target));
        assert!(!mount_dir_is_plain_dir(&link));
        assert!(!mount_dir_is_plain_dir(&root.join("missing")));

        fs::remove_dir_all(&root).expect("fixture cleanup");
    }

    #[test]
    fn backup_path_is_publicly_exposed_through_vault() {
        assert_eq!(
            Vault::backup_path_for(Path::new("/tmp/vault.img")),
            Path::new("/tmp/vault.img.bak")
        );
    }

    #[test]
    fn image_target_validation_accepts_missing_create_target() {
        let path =
            std::env::temp_dir().join(format!("adad-forge-missing-{}", super::unique_suffix()));

        assert_eq!(validate_image_target(&path, true), Ok(()));
    }

    #[test]
    fn image_target_validation_rejects_directories_and_missing_unlock_targets() {
        let root =
            std::env::temp_dir().join(format!("adad-forge-target-{}", super::unique_suffix()));
        fs::create_dir(&root).expect("test directory creates");

        assert!(validate_image_target(&root, true).is_err());
        assert!(validate_image_target(&root.join("missing.img"), false).is_err());

        fs::remove_dir(&root).expect("test directory removes");
    }

    #[cfg(unix)]
    #[test]
    fn image_target_validation_rejects_symlinks() {
        use std::os::unix::fs::symlink;

        let root =
            std::env::temp_dir().join(format!("adad-forge-symlink-{}", super::unique_suffix()));
        fs::create_dir(&root).expect("test directory creates");
        let target = root.join("target.img");
        fs::write(&target, b"not a vault").expect("target file writes");
        let link = root.join("link.img");
        symlink(&target, &link).expect("test symlink creates");

        let real_parent = root.join("real-parent");
        fs::create_dir(&real_parent).expect("real parent creates");
        let linked_parent = root.join("linked-parent");
        symlink(&real_parent, &linked_parent).expect("parent symlink creates");
        let nested_target = linked_parent.join("nested.img");

        assert!(validate_image_target(&link, true).is_err());
        assert!(validate_image_target(&link, false).is_err());
        assert!(validate_image_target(&nested_target, true).is_err());

        fs::remove_dir_all(&root).expect("test directory removes");
    }
}
