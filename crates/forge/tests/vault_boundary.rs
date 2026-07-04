#[path = "support/loop_image.rs"]
mod loop_image;

use std::path::Path;

use forge::Vault;
use loop_image::LoopImageHarness;

#[test]
fn vault_write_api_rejects_paths_outside_the_mount_root() {
    if let Some(reason) = runtime_skip_reason() {
        eprintln!("vault_boundary skipped: {reason}");
        return;
    }

    let harness = LoopImageHarness::new();
    Vault::create(harness.image_path(), "boundary-passphrase").expect("vault image is created");

    let unsealed =
        Vault::unlock(harness.image_path(), "boundary-passphrase").expect("vault unlocks");
    assert_eq!(
        unsealed.write_bytes(Path::new("../escape.txt"), b"nope"),
        Err(adad_core::Error::Io)
    );
    assert_eq!(
        unsealed.write_bytes(Path::new("/tmp/escape.txt"), b"nope"),
        Err(adad_core::Error::Io)
    );

    unsealed
        .write_bytes(Path::new("identity/session.txt"), b"allowed")
        .expect("in-vault write succeeds");
    assert_eq!(
        unsealed
            .read_bytes(Path::new("identity/session.txt"))
            .expect("read allowed path"),
        b"allowed"
    );
    unsealed.seal().expect("vault seals");

    std::fs::remove_dir_all(harness.root_dir()).ok();
}

fn runtime_skip_reason() -> Option<String> {
    if !cfg!(target_os = "linux") {
        return Some("host OS is not Linux".to_string());
    }

    let harness = LoopImageHarness::new();
    let missing = harness.missing_tools();
    if missing.is_empty() {
        None
    } else {
        Some(format!("missing host tools: {}", missing.join(", ")))
    }
}
