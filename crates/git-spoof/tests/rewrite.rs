use adad_core::SessionIdentity;
use git_spoof::{rewrite, CommitMetadata, FIXED_UTC_TIMESTAMP};

#[test]
fn rewrite_outputs_stable_pseudonym_and_normalized_timestamp() {
    let identity = test_identity();
    let raw = raw_commit("Real User", "real@example.com", "-0700", "first");

    let rewritten = rewrite(&raw, &identity).expect("rewrite should succeed");

    assert_eq!(rewritten.author_name, "Stable Persona");
    assert_eq!(rewritten.author_email, "stable@example.invalid");
    assert_eq!(rewritten.committer_name, "Stable Persona");
    assert_eq!(rewritten.committer_email, "stable@example.invalid");
    assert_eq!(rewritten.author_timestamp, FIXED_UTC_TIMESTAMP);
    assert_eq!(rewritten.committer_timestamp, FIXED_UTC_TIMESTAMP);
}

#[test]
fn rewrite_strips_real_name_email_and_local_timezone() {
    let identity = test_identity();
    let raw = raw_commit("Real User", "real@example.com", "-0700", "first");

    let rendered = rewrite(&raw, &identity)
        .expect("rewrite should succeed")
        .rendered_identity_fields();

    assert!(!rendered.contains("Real User"));
    assert!(!rendered.contains("real@example.com"));
    assert!(!rendered.contains("-0700"));
}

#[test]
fn rewrite_keeps_identity_stable_across_commits_without_rotation() {
    let identity = test_identity();
    let first = rewrite(
        &raw_commit("Real User", "real@example.com", "-0700", "first"),
        &identity,
    )
    .expect("first rewrite");
    let second = rewrite(
        &raw_commit("Other Real User", "other@example.com", "+0200", "second"),
        &identity,
    )
    .expect("second rewrite");

    assert_eq!(first.author_name, second.author_name);
    assert_eq!(first.author_email, second.author_email);
    assert_eq!(first.committer_name, second.committer_name);
    assert_eq!(first.committer_email, second.committer_email);
    assert_eq!(first.author_timestamp, second.author_timestamp);
}

fn test_identity() -> SessionIdentity {
    SessionIdentity::new(
        "adad-user",
        "Stable Persona",
        "stable@example.invalid",
        None,
    )
    .expect("identity should be valid")
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
