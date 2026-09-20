use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use adad_core::Error;

use crate::{InterfaceChange, Killswitch, KillswitchState, NetworkPosture};

pub const DROP_ALL_TARGET_LATENCY: Duration = Duration::from_millis(250);
const IP_COMMAND: &str = "/usr/sbin/ip";
const NFT_COMMAND: &str = "/usr/sbin/nft";
const DROP_RULESET_FILE: &str = "/etc/nftables.d/adad-killswitch-drop.nft";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NetlinkEvent {
    InterfaceDown {
        iface: String,
    },
    TunnelLost {
        iface: String,
    },
    Healthy {
        iface: String,
        posture: NetworkPosture,
    },
    Ambiguous {
        iface: String,
    },
}

impl NetlinkEvent {
    #[must_use]
    pub fn iface(&self) -> &str {
        match self {
            Self::InterfaceDown { iface }
            | Self::TunnelLost { iface }
            | Self::Healthy { iface, .. }
            | Self::Ambiguous { iface } => iface,
        }
    }

    fn into_interface_change(self) -> InterfaceChange {
        match self {
            Self::InterfaceDown { .. } => InterfaceChange::InterfaceDown,
            Self::TunnelLost { .. } => InterfaceChange::TunnelLost,
            Self::Healthy { posture, .. } => InterfaceChange::Healthy(posture),
            Self::Ambiguous { .. } => InterfaceChange::Ambiguous,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetlinkReaction {
    pub iface: String,
    pub state: KillswitchState,
    pub elapsed: Duration,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetlinkMonitor {
    target_latency: Duration,
}

impl NetlinkMonitor {
    #[must_use]
    pub fn new(target_latency: Duration) -> Self {
        Self { target_latency }
    }

    #[must_use]
    pub fn with_default_target() -> Self {
        Self::new(DROP_ALL_TARGET_LATENCY)
    }

    #[must_use]
    pub fn target_latency(&self) -> Duration {
        self.target_latency
    }

    pub fn handle_event(
        &self,
        killswitch: &mut Killswitch,
        event: NetlinkEvent,
    ) -> NetlinkReaction {
        let iface = event.iface().to_owned();
        let started = Instant::now();

        killswitch.on_interface_change(event.into_interface_change());

        NetlinkReaction {
            iface,
            state: killswitch.state(),
            elapsed: started.elapsed(),
        }
    }
}

impl Default for NetlinkMonitor {
    fn default() -> Self {
        Self::with_default_target()
    }
}

/// Run the Linux link monitor used by the shipped killswitch service.
///
/// The pure [`NetlinkMonitor`] above remains the deterministic policy model.
/// This adapter supplies its missing production event source and replaces the
/// active nftables ruleset with the fixed drop-only ruleset when a link is
/// deleted or reports `state DOWN`. A table flush alone would remove the
/// policy chains and could leave an accepting boundary. Losing the monitor
/// itself is an error so systemd can restart the service rather than leaving an
/// unobserved boundary.
pub fn run_system_monitor() -> Result<(), Error> {
    let mut child = Command::new(IP_COMMAND)
        .args(["monitor", "link"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| Error::Killswitch)?;

    let Some(stdout) = child.stdout.take() else {
        let _ = child.kill();
        let _ = child.wait();
        return Err(Error::Killswitch);
    };
    let result = (|| {
        for line in BufReader::new(stdout).lines() {
            let line = line.map_err(|_| Error::Killswitch)?;
            if is_fail_closed_event(&line) {
                install_drop_all_ruleset()?;
            }
        }
        Ok::<(), Error>(())
    })();

    let _ = child.kill();
    let _ = child.wait();
    result?;

    // A terminated event source is not a healthy monitor. Keep the service
    // fail-closed and let its supervisor restart it.
    Err(Error::Killswitch)
}

#[must_use]
pub fn is_fail_closed_event(line: &str) -> bool {
    let line = line.trim();
    !line.is_empty() && (line.starts_with("Deleted ") || line.contains("state DOWN"))
}

fn install_drop_all_ruleset() -> Result<(), Error> {
    let status = Command::new(NFT_COMMAND)
        .args(["-f", DROP_RULESET_FILE])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|_| Error::Killswitch)?;

    status.success().then_some(()).ok_or(Error::Killswitch)
}

#[cfg(test)]
mod tests {
    use super::{NetlinkEvent, NetlinkMonitor, DROP_ALL_TARGET_LATENCY};
    use crate::{Killswitch, KillswitchState, NetworkPosture};

    #[test]
    fn model_interface_down_maps_to_drop_all() {
        let monitor = NetlinkMonitor::default();
        let mut killswitch = Killswitch::new();
        killswitch.arm(NetworkPosture::tor_and_wireguard_active());

        let reaction = monitor.handle_event(
            &mut killswitch,
            NetlinkEvent::InterfaceDown {
                iface: "eth0".to_owned(),
            },
        );

        assert_eq!(reaction.iface, "eth0");
        assert_eq!(reaction.state, KillswitchState::DroppedAll);
        assert!(reaction.elapsed <= DROP_ALL_TARGET_LATENCY);
    }

    #[test]
    fn system_event_filter_only_reacts_to_down_or_deleted_links() {
        assert!(super::is_fail_closed_event(
            "3: wg0: <POINTOPOINT> state DOWN group default"
        ));
        assert!(super::is_fail_closed_event("Deleted 3: wg0: <POINTOPOINT>"));
        assert!(!super::is_fail_closed_event(
            "3: wg0: <POINTOPOINT> state UP group default"
        ));
        assert!(!super::is_fail_closed_event(""));
    }
}
