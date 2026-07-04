use std::collections::BTreeMap;

use adad_core::Error;
use agent_coding::{
    check_all, run_status_headless, Daemon, DaemonHealth, DaemonProbe, HealthReport, StatusEvent,
    StatusSnapshot,
};

#[test]
fn status_accuracy_reflects_health_report_including_unknown() {
    let report = HealthReport {
        tor: DaemonHealth::Ready,
        wireguard: DaemonHealth::Down,
        llama_server: DaemonHealth::Ready,
        monero: DaemonHealth::Unknown,
        git: DaemonHealth::Ready,
        killswitch: DaemonHealth::Ready,
        dms_hours_remaining: Some(12),
    };

    let log = run_status_headless(&[StatusEvent::SetHealth(report)])
        .expect("status monitor should render health report");

    assert!(log.frames[0].contains("Tor: ready"));
    assert!(log.frames[0].contains("WireGuard: down"));
    assert!(log.frames[0].contains("Monero: unknown"));
    assert!(log.frames[0].contains("DMS: 12h remaining"));
}

#[test]
fn status_accuracy_query_failure_becomes_unknown_not_stale_ok() {
    let stale_ok = StatusSnapshot {
        tor: DaemonHealth::Ready,
        wireguard: DaemonHealth::Ready,
        llama_server: DaemonHealth::Ready,
        monero: DaemonHealth::Ready,
        git: DaemonHealth::Ready,
        killswitch: DaemonHealth::Ready,
        dms_hours_remaining: Some(72),
        vault_lock_minutes_remaining: None,
        provider: "local".to_owned(),
        model: "qwen2.5-coder".to_owned(),
    };
    let mut probe = MapProbe::new([
        (Daemon::Tor, Err(Error::Io)),
        (Daemon::WireGuard, Ok(DaemonHealth::Ready)),
        (Daemon::LlamaServer, Ok(DaemonHealth::Ready)),
        (Daemon::Monero, Ok(DaemonHealth::Down)),
        (Daemon::Git, Ok(DaemonHealth::Ready)),
        (Daemon::Killswitch, Ok(DaemonHealth::Ready)),
    ]);
    let report = check_all(&mut probe);

    let log = run_status_headless(&[
        StatusEvent::SetStatus(stale_ok),
        StatusEvent::SetHealth(report),
    ])
    .expect("status monitor should render updated health");

    let latest = log.frames.last().expect("latest frame exists");
    assert!(latest.contains("Tor: unknown"));
    assert!(latest.contains("Monero: down"));
    assert!(!latest.contains("Tor: ready"));
}

struct MapProbe {
    states: BTreeMap<Daemon, Result<DaemonHealth, Error>>,
}

impl MapProbe {
    fn new<const N: usize>(entries: [(Daemon, Result<DaemonHealth, Error>); N]) -> Self {
        Self {
            states: entries.into_iter().collect(),
        }
    }
}

impl DaemonProbe for MapProbe {
    fn check(&mut self, daemon: Daemon) -> Result<DaemonHealth, Error> {
        self.states.get(&daemon).cloned().unwrap_or(Err(Error::Io))
    }
}
