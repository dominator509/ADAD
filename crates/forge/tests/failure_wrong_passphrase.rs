#[path = "support/loop_image.rs"]
mod loop_image;

use forge::Vault;
use loop_image::LoopImageHarness;

#[test]
fn failure_wrong_passphrase_leaves_vault_locked() {
    if let Some(reason) = runtime_skip_reason() {
        if std::env::var("ADAD_REQUIRE_VAULT").as_deref() == Ok("1") {
            panic!("vault integration is required but unavailable: {reason}");
        }
        eprintln!("failure_wrong_passphrase skipped: {reason}");
        return;
    }

    let harness = LoopImageHarness::new();
    Vault::create(harness.image_path(), "correct passphrase").expect("vault image is created");

    match Vault::unlock(harness.image_path(), "wrong passphrase") {
        Ok(unsealed) => {
            let _ = unsealed.seal();
            panic!("wrong passphrase should not unlock");
        }
        Err(error) => assert_eq!(error, adad_core::Error::Io),
    }
    std::fs::remove_dir_all(harness.root_dir()).ok();
}

fn runtime_skip_reason() -> Option<String> {
    LoopImageHarness::new().runtime_skip_reason()
}
