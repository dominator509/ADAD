use std::process::Command;

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

/// Probe the Linux services and interfaces that back the status monitor.
///
/// A missing command or an unrecognised host environment is deliberately
/// reported as `Unknown`; the status surface must never turn an unavailable
/// observation into a false `Ready` state. The probe is also harmless on
/// development hosts that do not run systemd or the ADAD daemons.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SystemDaemonProbe;

impl SystemDaemonProbe {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    fn systemd_unit(unit: &str) -> DaemonHealth {
        match Command::new("systemctl")
            .args(["is-active", "--quiet", unit])
            .output()
        {
            Ok(output) if output.status.success() => DaemonHealth::Ready,
            Ok(_) => DaemonHealth::Down,
            Err(_) => DaemonHealth::Unknown,
        }
    }

    fn process(name: &str) -> DaemonHealth {
        match Command::new("pidof").arg(name).output() {
            Ok(output) if output.status.success() && !output.stdout.is_empty() => {
                DaemonHealth::Ready
            }
            Ok(_) => DaemonHealth::Down,
            Err(_) => DaemonHealth::Unknown,
        }
    }

    fn interface_and_tool(interface: &str) -> DaemonHealth {
        let interface_present =
            Self::command_succeeds("ip", &["link", "show", "dev", interface, "up"]);
        let tool_present = Self::command_succeeds("wg", &["show", interface]);
        match (interface_present, tool_present) {
            (Some(true), Some(true)) => DaemonHealth::Ready,
            (Some(_), Some(_)) => DaemonHealth::Down,
            _ => DaemonHealth::Unknown,
        }
    }

    fn command_succeeds(program: &str, args: &[&str]) -> Option<bool> {
        Command::new(program)
            .args(args)
            .output()
            .ok()
            .map(|output| output.status.success())
    }

    fn killswitch() -> DaemonHealth {
        match Command::new("nft")
            .args(["list", "table", "inet", "adad_killswitch"])
            .output()
        {
            Ok(output) if output.status.success() => {
                let rules = String::from_utf8_lossy(&output.stdout);
                if rules.contains("policy drop") {
                    DaemonHealth::Ready
                } else {
                    DaemonHealth::Down
                }
            }
            Ok(_) => DaemonHealth::Down,
            Err(_) => DaemonHealth::Unknown,
        }
    }
}

impl DaemonProbe for SystemDaemonProbe {
    fn check(&mut self, daemon: Daemon) -> Result<DaemonHealth, Error> {
        let health = match daemon {
            Daemon::Tor => Self::systemd_unit("tor.service"),
            Daemon::WireGuard => Self::interface_and_tool("wg0"),
            Daemon::LlamaServer => Self::process("llama-server"),
            Daemon::Monero => {
                match (Self::process("monerod"), Self::process("monero-wallet-rpc")) {
                    (DaemonHealth::Ready, _) | (_, DaemonHealth::Ready) => DaemonHealth::Ready,
                    (DaemonHealth::Down, DaemonHealth::Down) => DaemonHealth::Down,
                    _ => DaemonHealth::Unknown,
                }
            }
            Daemon::Git => Self::systemd_unit("git-daemon.service"),
            Daemon::Killswitch => Self::killswitch(),
        };
        Ok(health)
    }
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

    use super::{check_all, Daemon, DaemonHealth, DaemonProbe, SystemDaemonProbe};

    #[test]
    fn failed_probe_query_maps_to_unknown() {
        let mut probe = MapProbe::new([(Daemon::Tor, Ok(DaemonHealth::Ready))]);

        let report = check_all(&mut probe);

        assert_eq!(report.tor, DaemonHealth::Ready);
        assert_eq!(report.wireguard, DaemonHealth::Unknown);
    }

    #[test]
    fn system_probe_is_constructible_and_never_panics_on_unknown_hosts() {
        let mut probe = SystemDaemonProbe::new();
        let _ = probe
            .check(Daemon::Tor)
            .expect("system probe returns typed state");
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
