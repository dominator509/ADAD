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
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(io_error)?;
        }

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
        let mount_dir = unique_mount_dir(&mapper_name);

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
        fs::create_dir_all(&mount_dir).map_err(io_error)?;
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
        let runtime = VaultRuntime::new(passphrase);
        let loop_device = runtime.attach_loop_device(path)?;
        let mapper_name = mapper_name_for(path);
        let mapped_device = mapped_device_path(&mapper_name);
        let mount_dir = unique_mount_dir(&mapper_name);

        if let Err(err) = runtime.open_mapping(&loop_device, &mapper_name) {
            let _ = runtime.detach_loop(&loop_device);
            return Err(err);
        }
        fs::create_dir_all(&mount_dir).map_err(io_error)?;
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
        if self.mount_dir.exists() {
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
        if !self.sealed && self.mount_dir.exists() {
            let _ = self.runtime.unmount(&self.mount_dir);
            let _ = self.runtime.close_mapping(&self.mapper_name);
            let _ = self.runtime.detach_loop(&self.loop_device);
            let _ = fs::remove_dir_all(&self.mount_dir);
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
        self.run(
            Command::new("mount")
                .arg("-t")
                .arg("ext4")
                .arg(mapped_device)
                .arg(mount_dir),
        )
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{backup_path_for, default_config, render_config, Vault, CURRENT_CONFIG_VERSION};

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
    fn backup_path_is_publicly_exposed_through_vault() {
        assert_eq!(
            Vault::backup_path_for(Path::new("/tmp/vault.img")),
            Path::new("/tmp/vault.img.bak")
        );
    }
}
