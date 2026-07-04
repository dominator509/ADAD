use metafuse::{scrub_metadata, ExifTag, ScrubPolicy, VaultMetadata};

#[test]
fn presented_metadata_strips_exif_and_real_owner_fields() {
    let metadata = vault_metadata();
    let policy = ScrubPolicy::new(65_534, 65_533, [1; 32]).expect("policy");

    let presented = scrub_metadata(&metadata, &policy).expect("scrubbed metadata");
    let rendered = presented.rendered_public_fields();

    assert!(presented.exif.is_empty());
    assert!(!rendered.contains("CameraSerial"));
    assert!(!rendered.contains("real-camera-serial"));
    assert!(!rendered.contains("1000"));
    assert_eq!(presented.uid, 65_534);
    assert_eq!(presented.gid, 65_533);
}

#[test]
fn presented_metadata_randomizes_all_real_timestamps() {
    let metadata = vault_metadata();
    let policy = ScrubPolicy::new(65_534, 65_533, [2; 32]).expect("policy");

    let presented = scrub_metadata(&metadata, &policy).expect("scrubbed metadata");
    let private_fields = metadata.rendered_private_fields();
    let public_fields = presented.rendered_public_fields();

    assert_ne!(presented.timestamps.created_at, metadata.created_at);
    assert_ne!(presented.timestamps.modified_at, metadata.modified_at);
    assert_ne!(presented.timestamps.accessed_at, metadata.accessed_at);
    assert!(!public_fields.contains(&metadata.created_at.to_string()));
    assert!(!public_fields.contains(&metadata.modified_at.to_string()));
    assert!(!public_fields.contains(&metadata.accessed_at.to_string()));
    assert!(private_fields.contains(&metadata.created_at.to_string()));
}

#[test]
fn scrubbed_timestamps_are_stable_for_one_session_and_change_across_sessions() {
    let metadata = vault_metadata();
    let first_policy = ScrubPolicy::new(65_534, 65_533, [3; 32]).expect("first policy");
    let second_policy = ScrubPolicy::new(65_534, 65_533, [4; 32]).expect("second policy");

    let first = scrub_metadata(&metadata, &first_policy).expect("first scrub");
    let repeated = scrub_metadata(&metadata, &first_policy).expect("repeated scrub");
    let second = scrub_metadata(&metadata, &second_policy).expect("second scrub");

    assert_eq!(first.timestamps, repeated.timestamps);
    assert_ne!(first.timestamps, second.timestamps);
}

#[test]
fn invalid_scrub_inputs_are_rejected() {
    let metadata = VaultMetadata::new("", 1000, 1000, 1, 2, 3, Vec::new());
    let policy = ScrubPolicy::new(65_534, 65_533, [5; 32]).expect("policy");

    assert!(scrub_metadata(&metadata, &policy).is_err());
    assert!(ScrubPolicy::new(0, 65_533, [5; 32]).is_err());
    assert!(policy.with_timestamp_window_seconds(0).is_err());
}

fn vault_metadata() -> VaultMetadata {
    VaultMetadata::new(
        "/vault/src/photo.jpg",
        1000,
        1000,
        1_783_123_200,
        1_783_126_800,
        1_783_130_400,
        vec![
            ExifTag::new("CameraSerial", "real-camera-serial"),
            ExifTag::new("GpsLatitude", "47.6205"),
        ],
    )
}
