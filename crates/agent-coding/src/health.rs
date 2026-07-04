use adad_core::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Daemon {
    Tor,
    WireGuard,
    LlamaServer,
    Monero,
    Git,
    Killswitch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DaemonHealth {
    Ready,
    Down,
    Unknown,
}

impl DaemonHealth {
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Down => "down",
            Self::Unknown => "unknown",
        }
    }
}

pub trait DaemonProbe {
    fn check(&mut self, daemon: Daemon) -> Result<DaemonHealth, Error>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HealthReport {
    pub tor: DaemonHealth,
    pub wireguard: DaemonHealth,
    pub llama_server: DaemonHealth,
    pub monero: DaemonHealth,
    pub git: DaemonHealth,
    pub killswitch: DaemonHealth,
    pub dms_hours_remaining: Option<u32>,
}

impl Default for HealthReport {
    fn default() -> Self {
        Self {
            tor: DaemonHealth::Unknown,
            wireguard: DaemonHealth::Unknown,
            llama_server: DaemonHealth::Unknown,
            monero: DaemonHealth::Unknown,
            git: DaemonHealth::Unknown,
            killswitch: DaemonHealth::Unknown,
            dms_hours_remaining: None,
        }
    }
}

pub fn check_all(probe: &mut impl DaemonProbe) -> HealthReport {
    HealthReport {
        tor: checked(probe, Daemon::Tor),
        wireguard: checked(probe, Daemon::WireGuard),
        llama_server: checked(probe, Daemon::LlamaServer),
        monero: checked(probe, Daemon::Monero),
        git: checked(probe, Daemon::Git),
        killswitch: checked(probe, Daemon::Killswitch),
        dms_hours_remaining: None,
    }
}

fn checked(probe: &mut impl DaemonProbe, daemon: Daemon) -> DaemonHealth {
    probe.check(daemon).unwrap_or(DaemonHealth::Unknown)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use adad_core::Error;

    use super::{check_all, Daemon, DaemonHealth, DaemonProbe};

    #[test]
    fn failed_probe_query_maps_to_unknown() {
        let mut probe = MapProbe::new([(Daemon::Tor, Ok(DaemonHealth::Ready))]);

        let report = check_all(&mut probe);

        assert_eq!(report.tor, DaemonHealth::Ready);
        assert_eq!(report.wireguard, DaemonHealth::Unknown);
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
}
