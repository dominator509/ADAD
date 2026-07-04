use agent_coding::{run_status_headless, DaemonHealth, StatusAlert, StatusEvent, StatusSnapshot};

#[test]
fn alerts_render_all_security_banners_with_text_labels() {
    let status = StatusSnapshot {
        tor: DaemonHealth::Ready,
        wireguard: DaemonHealth::Down,
        llama_server: DaemonHealth::Ready,
        monero: DaemonHealth::Ready,
        git: DaemonHealth::Ready,
        killswitch: DaemonHealth::Down,
        dms_hours_remaining: Some(2),
        vault_lock_minutes_remaining: Some(10),
        provider: "local".to_owned(),
        model: "qwen2.5-coder".to_owned(),
    };

    let log = run_status_headless(&[StatusEvent::SetStatus(status.clone())])
        .expect("status alerts should render");
    let frame = &log.frames[0];

    for alert in status.alerts() {
        assert!(frame.contains(alert.label()));
    }
    assert!(frame.contains("Alert: KILLSWITCH FIRED"));
    assert!(frame.contains("Alert: TUNNEL DOWN"));
    assert!(frame.contains("Alert: DMS NEAR EXPIRY"));
    assert!(frame.contains("Alert: VAULT LOCK IMMINENT"));
}

#[test]
fn alerts_do_not_render_when_conditions_are_safe() {
    let status = StatusSnapshot {
        tor: DaemonHealth::Ready,
        wireguard: DaemonHealth::Ready,
        llama_server: DaemonHealth::Ready,
        monero: DaemonHealth::Ready,
        git: DaemonHealth::Ready,
        killswitch: DaemonHealth::Ready,
        dms_hours_remaining: Some(24),
        vault_lock_minutes_remaining: Some(60),
        provider: "local".to_owned(),
        model: "qwen2.5-coder".to_owned(),
    };

    let log = run_status_headless(&[StatusEvent::SetStatus(status)])
        .expect("status alerts should render");

    assert!(!log.frames[0].contains("Alert: KILLSWITCH FIRED"));
    assert!(!log.frames[0].contains("Alert: TUNNEL DOWN"));
    assert!(!log.frames[0].contains("Alert: DMS NEAR EXPIRY"));
    assert!(!log.frames[0].contains("Alert: VAULT LOCK IMMINENT"));
}

#[test]
fn alert_labels_are_stable_for_all_conditions() {
    assert_eq!(
        StatusAlert::KillswitchFired.label(),
        "Alert: KILLSWITCH FIRED - all egress dropped"
    );
    assert_eq!(
        StatusAlert::TunnelDown.label(),
        "Alert: TUNNEL DOWN - fallback API egress blocked"
    );
    assert_eq!(
        StatusAlert::DmsNearExpiry.label(),
        "Alert: DMS NEAR EXPIRY - access vault or prepare wipe"
    );
    assert_eq!(
        StatusAlert::VaultLockImminent.label(),
        "Alert: VAULT LOCK IMMINENT - save and seal soon"
    );
}
