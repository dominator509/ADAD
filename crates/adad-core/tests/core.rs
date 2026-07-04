use adad_core::{Provider, SessionIdentity, ZeroClockEpoch};

#[test]
fn zero_clock_epoch_is_deterministic_from_seed() {
    let a = ZeroClockEpoch::from_seed(b"seed");
    let b = ZeroClockEpoch::from_seed(b"seed");
    let c = ZeroClockEpoch::from_seed(b"seed-2");

    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn zero_clock_epoch_source_avoids_host_clock_calls() {
    let source = include_str!("../src/epoch.rs");

    assert!(!source.contains("SystemTime"));
    assert!(!source.contains("Instant"));
    assert!(!source.contains("chrono"));
}

#[test]
fn session_identity_debug_is_redacted() {
    let identity = SessionIdentity::new(
        "adad-user",
        "Stable Persona",
        "stable@example.invalid",
        Some("abcdefghijklmnop.onion".to_owned()),
    )
    .expect("identity should be valid");

    let rendered = format!("{identity:?}");
    assert!(!rendered.contains("adad-user"));
    assert!(!rendered.contains("Stable Persona"));
    assert!(!rendered.contains("stable@example.invalid"));
    assert!(!rendered.contains("abcdefghijklmnop.onion"));
    assert!(rendered.contains("[REDACTED]"));
}

#[test]
fn session_identity_rejects_invalid_email() {
    let error = SessionIdentity::new("adad-user", "Stable Persona", "not-an-email", None)
        .expect_err("invalid email should fail");

    assert_eq!(error.to_string(), "Session identity error");
}

#[test]
fn provider_strings_are_stable() {
    assert_eq!(Provider::Local.as_str(), "local");
    assert_eq!(Provider::OpenAi.as_str(), "openai");
    assert_eq!(Provider::Venice.as_str(), "venice");
}
