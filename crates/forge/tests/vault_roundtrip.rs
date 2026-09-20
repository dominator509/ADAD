#[path = "support/loop_image.rs"]
mod loop_image;

use std::path::Path;

use forge::Vault;
use loop_image::LoopImageHarness;

#[test]
fn loop_image_harness_exposes_distinct_paths_and_mapper_name() {
    let harness = LoopImageHarness::new();

    assert!(harness
        .root_dir()
        .to_string_lossy()
        .contains("adad-forge-loop-image-"));
    assert!(harness.image_path().ends_with(Path::new("vault.img")));
    assert!(harness.mount_dir().ends_with(Path::new("mnt")));
    assert!(harness.mapper_name().starts_with("adad-vault-"));
    assert_eq!(
        harness.mapped_device_path(),
        Path::new("/dev/mapper").join(harness.mapper_name())
    );
}

#[test]
fn command_shapes_match_expected_loopback_luks_flow() {
    let harness = LoopImageHarness::new();
    let loop_device = Path::new("/dev/loop7");

    let truncate = harness.create_sparse_image_command(64);
    let losetup = harness.attach_loop_command();
    let luks_format = harness.luks_format_command(loop_device);
    let unlock = harness.unlock_command(loop_device);
    let lock = harness.lock_command();
    let make_filesystem = harness.make_filesystem_command();
    let mount = harness.mount_command();
    let unmount = harness.unmount_command();
    let detach = harness.detach_loop_command(Path::new("/dev/loop7"));

    assert_eq!(truncate.get_program().to_string_lossy(), "truncate");
    assert_eq!(
        truncate
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        vec![
            "--size".to_string(),
            "64M".to_string(),
            harness.image_path().display().to_string(),
        ]
    );

    assert_eq!(losetup.get_program().to_string_lossy(), "losetup");
    assert_eq!(
        losetup
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        vec![
            "--find".to_string(),
            "--show".to_string(),
            harness.image_path().display().to_string(),
        ]
    );

    assert_eq!(luks_format.get_program().to_string_lossy(), "cryptsetup");
    assert_eq!(
        luks_format
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        vec![
            "luksFormat".to_string(),
            "--type".to_string(),
            "luks2".to_string(),
            "--pbkdf".to_string(),
            "argon2id".to_string(),
            "--batch-mode".to_string(),
            "--key-file".to_string(),
            "-".to_string(),
            loop_device.display().to_string(),
        ]
    );

    assert_eq!(unlock.get_program().to_string_lossy(), "cryptsetup");
    assert_eq!(
        unlock
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        vec![
            "open".to_string(),
            "--type".to_string(),
            "luks2".to_string(),
            "--key-file".to_string(),
            "-".to_string(),
            loop_device.display().to_string(),
            harness.mapper_name().to_string(),
        ]
    );

    assert_eq!(lock.get_program().to_string_lossy(), "cryptsetup");
    assert_eq!(
        lock.get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        vec!["close".to_string(), harness.mapper_name().to_string()]
    );

    assert_eq!(make_filesystem.get_program().to_string_lossy(), "mkfs.ext4");
    assert_eq!(
        make_filesystem
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        vec![
            "-F".to_string(),
            harness.mapped_device_path().display().to_string(),
        ]
    );

    assert_eq!(mount.get_program().to_string_lossy(), "mount");
    assert_eq!(
        mount
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        vec![
            "-t".to_string(),
            "ext4".to_string(),
            harness.mapped_device_path().display().to_string(),
            harness.mount_dir().display().to_string(),
        ]
    );

    assert_eq!(unmount.get_program().to_string_lossy(), "umount");
    assert_eq!(
        unmount
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        vec![harness.mount_dir().display().to_string()]
    );

    assert_eq!(detach.get_program().to_string_lossy(), "losetup");
    assert_eq!(
        detach
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        vec!["--detach".to_string(), "/dev/loop7".to_string()]
    );
}

#[test]
fn helper_reports_missing_host_tools_without_touching_real_devices() {
    let harness = LoopImageHarness::new();
    let missing = harness.missing_tools();

    for tool in missing {
        assert!(LoopImageHarness::required_tools().contains(&tool));
    }
}

#[test]
fn vault_roundtrip_runs_when_linux_host_tools_are_available() {
    if let Some(reason) = runtime_skip_reason() {
        if std::env::var("ADAD_REQUIRE_VAULT").as_deref() == Ok("1") {
            panic!("vault integration is required but unavailable: {reason}");
        }
        eprintln!("vault_roundtrip skipped: {reason}");
        return;
    }

    let harness = LoopImageHarness::new();
    let payload_path = Path::new("repos/demo/notes.txt");
    let passphrase = "correct horse battery staple";

    Vault::create(harness.image_path(), passphrase).expect("vault image is created");

    let unsealed = Vault::unlock(harness.image_path(), passphrase).expect("vault unlocks");
    unsealed
        .write_bytes(payload_path, b"persistent payload")
        .expect("payload writes");
    unsealed.seal().expect("vault seals");

    let reopened = Vault::unlock(harness.image_path(), passphrase).expect("vault re-unlocks");
    assert_eq!(
        reopened.read_bytes(payload_path).expect("payload reads"),
        b"persistent payload"
    );
    reopened.seal().expect("vault reseals");

    std::fs::remove_dir_all(harness.root_dir()).ok();
}

#[test]
fn wrong_passphrase_fails_cleanly_when_runtime_is_available() {
    if let Some(reason) = runtime_skip_reason() {
        if std::env::var("ADAD_REQUIRE_VAULT").as_deref() == Ok("1") {
            panic!("vault integration is required but unavailable: {reason}");
        }
        eprintln!("vault_roundtrip skipped: {reason}");
        return;
    }

    let harness = LoopImageHarness::new();
    Vault::create(harness.image_path(), "sunrise passphrase").expect("vault image is created");

    assert!(matches!(
        Vault::unlock(harness.image_path(), "incorrect passphrase"),
        Err(adad_core::Error::Io)
    ));

    std::fs::remove_dir_all(harness.root_dir()).ok();
}

fn runtime_skip_reason() -> Option<String> {
    LoopImageHarness::new().runtime_skip_reason()
}
