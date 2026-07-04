use adad_core::{log_event, Component, InMemoryLogSink, LogEvent, LogField, LogLevel, LogOutcome};

#[test]
fn redaction_secret_bearing_events_emit_no_secret_material() {
    let event = LogEvent::new(
        "tor+456",
        LogLevel::Warn,
        Component::Wallet,
        "wallet_status",
        LogOutcome::Error,
    )
    .with_field(LogField::sensitive(
        "passphrase",
        "correct horse battery staple",
    ))
    .with_field(LogField::sensitive("api_key", "sk-live-secret"))
    .with_field(LogField::sensitive("wireguard_key", "wg-private-key"))
    .with_field(LogField::sensitive(
        "onion",
        "abcdefghijklmnopqrstuvwxyzabcdefghijklmnop.onion",
    ))
    .with_field(LogField::sensitive("xmr_address", "88xmraddress"))
    .with_field(LogField::sensitive("identity", "Real Person"));
    let mut sink = InMemoryLogSink::new();

    log_event(&mut sink, &event);

    let rendered = sink.lines().join("\n");
    assert!(rendered.contains("component=wallet"));
    assert!(rendered.contains("outcome=error"));
    for secret in [
        "correct horse battery staple",
        "sk-live-secret",
        "wg-private-key",
        "abcdefghijklmnopqrstuvwxyzabcdefghijklmnop.onion",
        "88xmraddress",
        "Real Person",
    ] {
        assert!(!rendered.contains(secret));
    }
}

#[test]
fn redaction_logger_is_in_memory_only() {
    let mut sink = InMemoryLogSink::new();
    log_event(
        &mut sink,
        &LogEvent::new(
            "tor+789",
            LogLevel::Info,
            Component::Agent,
            "status_poll",
            LogOutcome::Ok,
        ),
    );

    assert_eq!(sink.lines().len(), 1);
    assert!(!sink.lines()[0].contains('\\'));
    assert!(!sink.lines()[0].contains('/'));
}
