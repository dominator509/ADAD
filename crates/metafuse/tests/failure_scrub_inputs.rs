use adad_core::Error;
use metafuse::{scrub_metadata, ScrubPolicy, VaultMetadata};

#[test]
fn failure_blank_vault_path_is_refused() {
    let metadata = VaultMetadata::new("", 1000, 1000, 1, 2, 3, Vec::new());
    let policy = ScrubPolicy::new(65_534, 65_533, [1; 32]).expect("policy");

    let error = scrub_metadata(&metadata, &policy).expect_err("blank path should fail");

    assert_eq!(error, Error::Metafuse);
}

#[test]
fn failure_root_fake_owner_is_refused() {
    assert_eq!(
        ScrubPolicy::new(0, 65_533, [1; 32]).expect_err("root fake uid should fail"),
        Error::Metafuse
    );
}
