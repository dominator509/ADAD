#![allow(dead_code)]

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
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
