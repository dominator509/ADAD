#![allow(dead_code)]

use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static UNIQUE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
pub struct LoopImageHarness {
    root_dir: PathBuf,
    image_path: PathBuf,
    mount_dir: PathBuf,
    mapper_name: String,
}

impl LoopImageHarness {
    pub fn new() -> Self {
        let unique = unique_suffix();
        let root = env::temp_dir().join(format!("adad-forge-loop-image-{unique}"));

        Self {
            root_dir: root.clone(),
            image_path: root.join("vault.img"),
            mount_dir: root.join("mnt"),
            mapper_name: format!("adad-vault-{unique}"),
        }
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    pub fn image_path(&self) -> &Path {
        &self.image_path
    }

    pub fn mount_dir(&self) -> &Path {
        &self.mount_dir
    }

    pub fn mapper_name(&self) -> &str {
        &self.mapper_name
    }

    pub fn mapped_device_path(&self) -> PathBuf {
        Path::new("/dev/mapper").join(self.mapper_name())
    }

    pub fn required_tools() -> &'static [&'static str] {
        &[
            "truncate",
            "losetup",
            "cryptsetup",
            "mkfs.ext4",
            "mount",
            "umount",
        ]
    }

    pub fn missing_tools(&self) -> Vec<&'static str> {
        Self::required_tools()
            .iter()
            .copied()
            .filter(|tool| !command_is_on_path(tool))
            .collect()
    }

    pub fn runtime_skip_reason(&self) -> Option<String> {
        if !cfg!(target_os = "linux") {
            return Some("host OS is not Linux".to_string());
        }

        let missing = self.missing_tools();
        if !missing.is_empty() {
            return Some(format!("missing host tools: {}", missing.join(", ")));
        }

        self.probe_runtime()
            .err()
            .map(|stage| format!("vault runtime unavailable at {stage}"))
    }

    fn probe_runtime(&self) -> Result<(), &'static str> {
        fs::create_dir_all(self.root_dir()).map_err(|_| "temporary directory creation")?;
        if !command_succeeded(&mut self.create_sparse_image_command(64)) {
            let _ = fs::remove_dir_all(self.root_dir());
            return Err("sparse image creation");
        }

        let output = match self.attach_loop_command().output() {
            Ok(output) if output.status.success() => output,
            Ok(_) => {
                let _ = fs::remove_dir_all(self.root_dir());
                return Err("loop-device attachment");
            }
            Err(_) => {
                let _ = fs::remove_dir_all(self.root_dir());
                return Err("loop-device attachment");
            }
        };
        let loop_device_text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if loop_device_text.is_empty() {
            let _ = fs::remove_dir_all(self.root_dir());
            return Err("loop-device attachment output");
        }
        let loop_device = PathBuf::from(loop_device_text);
        let mut mapped = false;
        let mut mounted = false;
        let operation = (|| {
            if !command_with_stdin_succeeded(
                &mut self.luks_format_command(&loop_device),
                b"adad-ci-probe-passphrase",
            ) {
                return Err("LUKS formatting");
            }
            if !command_with_stdin_succeeded(
                &mut self.unlock_command(&loop_device),
                b"adad-ci-probe-passphrase",
            ) {
                return Err("device-mapper unlock");
            }
            mapped = true;
            if !command_succeeded(&mut self.make_filesystem_command()) {
                return Err("filesystem creation");
            }
            fs::create_dir_all(self.mount_dir()).map_err(|_| "mount directory creation")?;
            if !command_succeeded(&mut self.mount_command()) {
                return Err("filesystem mount");
            }
            mounted = true;
            if !command_succeeded(&mut self.unmount_command()) {
                return Err("filesystem unmount");
            }
            mounted = false;
            if !command_succeeded(&mut self.lock_command()) {
                return Err("device-mapper close");
            }
            mapped = false;
            Ok(())
        })();

        if mounted {
            let _ = command_succeeded(&mut self.unmount_command());
        }
        if mapped {
            let _ = command_succeeded(&mut self.lock_command());
        }
        let detached = command_succeeded(&mut self.detach_loop_command(&loop_device));
        let cleanup = fs::remove_dir_all(self.root_dir());

        match operation {
            Err(stage) => Err(stage),
            Ok(()) if !detached => Err("loop-device detachment"),
            Ok(()) if cleanup.is_err() => Err("temporary probe cleanup"),
            Ok(()) => Ok(()),
        }
    }

    pub fn create_sparse_image_command(&self, size_mebibytes: u32) -> Command {
        let mut command = Command::new("truncate");
        command
            .arg("--size")
            .arg(format!("{size_mebibytes}M"))
            .arg(self.image_path());
        command
    }

    pub fn attach_loop_command(&self) -> Command {
        let mut command = Command::new("losetup");
        command.arg("--find").arg("--show").arg(self.image_path());
        command
    }

    pub fn detach_loop_command(&self, loop_device: &Path) -> Command {
        let mut command = Command::new("losetup");
        command.arg("--detach").arg(loop_device);
        command
    }

    pub fn luks_format_command(&self, loop_device: &Path) -> Command {
        let mut command = Command::new("cryptsetup");
        command
            .arg("luksFormat")
            .arg("--type")
            .arg("luks2")
            .arg("--pbkdf")
            .arg("argon2id")
            .arg("--batch-mode")
            .arg("--key-file")
            .arg("-")
            .arg(loop_device);
        command
    }

    pub fn unlock_command(&self, loop_device: &Path) -> Command {
        let mut command = Command::new("cryptsetup");
        command
            .arg("open")
            .arg("--type")
            .arg("luks2")
            .arg("--key-file")
            .arg("-")
            .arg(loop_device)
            .arg(self.mapper_name());
        command
    }

    pub fn lock_command(&self) -> Command {
        let mut command = Command::new("cryptsetup");
        command.arg("close").arg(self.mapper_name());
        command
    }

    pub fn make_filesystem_command(&self) -> Command {
        let mut command = Command::new("mkfs.ext4");
        command.arg("-F").arg(self.mapped_device_path());
        command
    }

    pub fn mount_command(&self) -> Command {
        let mut command = Command::new("mount");
        command
            .arg("-t")
            .arg("ext4")
            .arg(self.mapped_device_path())
            .arg(self.mount_dir());
        command
    }

    pub fn unmount_command(&self) -> Command {
        let mut command = Command::new("umount");
        command.arg(self.mount_dir());
        command
    }
}

fn command_succeeded(command: &mut Command) -> bool {
    command
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn command_with_stdin_succeeded(command: &mut Command, input: &[u8]) -> bool {
    let Ok(mut child) = command
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    else {
        return false;
    };

    let Some(mut stdin) = child.stdin.take() else {
        let _ = child.kill();
        let _ = child.wait();
        return false;
    };
    if stdin.write_all(input).is_err() {
        let _ = child.kill();
        let _ = child.wait();
        return false;
    }
    drop(stdin);

    child.wait().map(|status| status.success()).unwrap_or(false)
}

fn unique_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let counter = UNIQUE_COUNTER.fetch_add(1, Ordering::Relaxed);

    format!("{nanos}-{counter}")
}

fn command_is_on_path(command_name: &str) -> bool {
    let Some(path) = env::var_os("PATH") else {
        return false;
    };

    env::split_paths(&path).any(|entry| {
        let candidate = entry.join(command_name);
        if candidate.is_file() {
            return true;
        }

        if cfg!(windows) {
            [".exe", ".bat", ".cmd"]
                .iter()
                .any(|suffix| entry.join(format!("{command_name}{suffix}")).is_file())
        } else {
            false
        }
    })
}
