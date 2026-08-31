use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use adad_core::SessionIdentity;
use git_spoof::{commit, rewrite, CommitMetadata, FIXED_UTC_TIMESTAMP};

#[test]
fn commit_writes_stable_identity_and_fixed_utc_metadata_to_git() {
    let root = unique_temp_dir();
    fs::create_dir(&root).expect("create test repository");
    run_git(&root, &["init"]);
    run_git(&root, &["config", "commit.gpgsign", "false"]);
    fs::write(root.join("tracked.txt"), "safe test content\n").expect("write staged file");
    run_git(&root, &["add", "--", "tracked.txt"]);

    let identity = test_identity();
    let hash = commit(&root, "first private commit", &identity).expect("commit should succeed");
    assert!(!hash.is_empty());

    let metadata = run_git(
        &root,
        &["show", "-s", "--format=%an%n%ae%n%aI%n%cn%n%ce%n%cI"],
    );
    let fields = metadata.lines().collect::<Vec<_>>();
    assert_eq!(
        fields,
        [
            "Stable Persona",
            "stable@example.invalid",
            FIXED_UTC_TIMESTAMP,
            "Stable Persona",
            "stable@example.invalid",
            FIXED_UTC_TIMESTAMP,
        ]
    );

    fs::remove_dir_all(&root).expect("remove test repository");
}

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

fn unique_temp_dir() -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("adad-git-spoof-{}-{nanos}", std::process::id()))
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

fn run_git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(root)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", root.join("missing-global-config"))
        .args(args)
        .output()
        .expect("git must be installed for the integration test");
    assert!(
        output.status.success(),
        "git command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("git output should be UTF-8")
}
