#[path = "support/loop_image.rs"]
mod loop_image;

use std::path::Path;

use forge::{Vault, CURRENT_CONFIG_VERSION};
use loop_image::LoopImageHarness;

#[test]
fn vault_upgrade_makes_a_backup_and_preserves_data() {
    if let Some(reason) = runtime_skip_reason() {
        if std::env::var("ADAD_REQUIRE_VAULT").as_deref() == Ok("1") {
            panic!("vault integration is required but unavailable: {reason}");
        }
        eprintln!("vault_upgrade skipped: {reason}");
        return;
    }

    let harness = LoopImageHarness::new();
    let payload_path = Path::new("repos/demo/upgrade.txt");
    let passphrase = "upgrade-passphrase";

    Vault::create(harness.image_path(), passphrase).expect("vault image is created");
    let unsealed = Vault::unlock(harness.image_path(), passphrase).expect("vault unlocks");
    unsealed
        .write_config_text("config_version = 1\nprovider = \"local\"\n")
        .expect("older config version writes");
    unsealed
        .write_bytes(payload_path, b"upgrade payload")
        .expect("payload writes");
    unsealed.seal().expect("vault seals");

    let backup_path =
        Vault::upgrade_in_place(harness.image_path(), passphrase).expect("upgrade succeeds");
    assert!(backup_path.exists());

    let upgraded = Vault::unlock(harness.image_path(), passphrase).expect("upgraded vault unlocks");
    let config = upgraded.load_config().expect("config loads after upgrade");
    assert_eq!(config.config_version, CURRENT_CONFIG_VERSION);
    assert_eq!(
        upgraded.read_bytes(payload_path).expect("payload reads"),
        b"upgrade payload"
    );
    upgraded.seal().expect("vault reseals");

    std::fs::remove_dir_all(harness.root_dir()).ok();
}

fn runtime_skip_reason() -> Option<String> {
    LoopImageHarness::new().runtime_skip_reason()
}
