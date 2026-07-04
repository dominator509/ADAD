use std::time::{Duration, Instant};

use crate::{InterfaceChange, Killswitch, KillswitchState, NetworkPosture};

pub const DROP_ALL_TARGET_LATENCY: Duration = Duration::from_millis(250);

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

#[cfg(test)]
mod tests {
    use super::{NetlinkEvent, NetlinkMonitor, DROP_ALL_TARGET_LATENCY};
    use crate::{Killswitch, KillswitchState, NetworkPosture};

    #[test]
    fn simulated_interface_down_maps_to_drop_all() {
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
}
