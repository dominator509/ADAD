use adad_core::SessionIdentity;
use git_spoof::{rewrite, CommitMetadata, FIXED_UTC_TIMESTAMP};

#[test]
fn regression_git_identity_is_stable_and_real_metadata_stays_stripped() {
    let identity = SessionIdentity::new(
        "adad-user",
        "Stable Persona",
        "stable@example.invalid",
        None,
    )
    .expect("identity");
    let first = rewrite(
        &raw_commit("Real User", "real@example.com", "-0700", "first"),
        &identity,
    )
    .expect("first rewrite");
    let second = rewrite(
        &raw_commit("Other User", "other@example.com", "+0200", "second"),
        &identity,
    )
    .expect("second rewrite");

    assert_eq!(first.author_name, second.author_name);
    assert_eq!(first.author_email, second.author_email);
    assert_eq!(first.author_timestamp, FIXED_UTC_TIMESTAMP);
    assert_eq!(second.committer_timestamp, FIXED_UTC_TIMESTAMP);
    let rendered = format!(
        "{}\n{}",
        first.rendered_identity_fields(),
        second.rendered_identity_fields()
    );
    assert!(!rendered.contains("Real User"));
    assert!(!rendered.contains("Other User"));
    assert!(!rendered.contains("real@example.com"));
    assert!(!rendered.contains("other@example.com"));
    assert!(!rendered.contains("-0700"));
    assert!(!rendered.contains("+0200"));
}

fn raw_commit(name: &str, email: &str, tz: &str, message: &str) -> CommitMetadata {
    CommitMetadata::new(
        name,
        email,
        format!("2026-07-03T21:00:00{tz}"),
        name,
        email,
        format!("2026-07-03T21:00:00{tz}"),
        message,
    )
}
