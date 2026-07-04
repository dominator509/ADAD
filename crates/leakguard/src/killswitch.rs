use crate::firewall::FirewallPosture;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TunnelHealth {
    Active,
    Inactive,
    Unknown,
}

impl TunnelHealth {
    fn is_active(self) -> bool {
        self == Self::Active
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetworkPosture {
    pub tor: TunnelHealth,
    pub wireguard: TunnelHealth,
}

impl NetworkPosture {
    #[must_use]
    pub fn new(tor: TunnelHealth, wireguard: TunnelHealth) -> Self {
        Self { tor, wireguard }
    }

    #[must_use]
    pub fn tor_and_wireguard_active() -> Self {
        Self::new(TunnelHealth::Active, TunnelHealth::Active)
    }

    fn is_fail_closed_safe(self) -> bool {
        self.tor.is_active() && self.wireguard != TunnelHealth::Unknown
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterfaceChange {
    Healthy(NetworkPosture),
    InterfaceDown,
    TunnelLost,
    Ambiguous,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KillswitchState {
    Disarmed,
    Armed,
    DroppedAll,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Killswitch {
    state: KillswitchState,
    firewall: FirewallPosture,
}

impl Killswitch {
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: KillswitchState::Disarmed,
            firewall: FirewallPosture::drop_all(),
        }
    }

    pub fn arm(&mut self, posture: NetworkPosture) {
        if posture.is_fail_closed_safe() {
            self.state = KillswitchState::Armed;
            self.firewall = FirewallPosture::default_drop(true, posture.wireguard.is_active());
        } else {
            self.drop_all();
        }
    }

    pub fn on_interface_change(&mut self, change: InterfaceChange) {
        match change {
            InterfaceChange::Healthy(posture) if posture.is_fail_closed_safe() => {
                self.state = KillswitchState::Armed;
                self.firewall = FirewallPosture::default_drop(true, posture.wireguard.is_active());
            }
            InterfaceChange::Healthy(_)
            | InterfaceChange::InterfaceDown
            | InterfaceChange::TunnelLost
            | InterfaceChange::Ambiguous => self.drop_all(),
        }
    }

    #[must_use]
    pub fn state(&self) -> KillswitchState {
        self.state
    }

    #[must_use]
    pub fn firewall(&self) -> FirewallPosture {
        self.firewall
    }

    fn drop_all(&mut self) {
        self.state = KillswitchState::DroppedAll;
        self.firewall = FirewallPosture::drop_all();
    }
}

impl Default for Killswitch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{InterfaceChange, Killswitch, KillswitchState, NetworkPosture, TunnelHealth};
    use crate::{EgressClass, FirewallAction};

    #[test]
    fn new_killswitch_starts_fail_closed() {
        let killswitch = Killswitch::new();

        assert_eq!(killswitch.state(), KillswitchState::Disarmed);
        assert_eq!(killswitch.firewall().default_action(), FirewallAction::Drop);
        assert!(!killswitch.firewall().permits(EgressClass::Tor));
        assert!(!killswitch.firewall().permits(EgressClass::WireGuard));
        assert!(!killswitch.firewall().permits(EgressClass::Other));
    }

    #[test]
    fn arm_installs_default_drop_with_tor_and_wireguard_allow_list() {
        let mut killswitch = Killswitch::new();

        killswitch.arm(NetworkPosture::tor_and_wireguard_active());

        assert_eq!(killswitch.state(), KillswitchState::Armed);
        assert_eq!(killswitch.firewall().default_action(), FirewallAction::Drop);
        assert!(killswitch.firewall().permits(EgressClass::Tor));
        assert!(killswitch.firewall().permits(EgressClass::WireGuard));
        assert!(!killswitch.firewall().permits(EgressClass::Other));
    }

    #[test]
    fn arm_without_tor_fails_closed() {
        let mut killswitch = Killswitch::new();

        killswitch.arm(NetworkPosture::new(
            TunnelHealth::Inactive,
            TunnelHealth::Active,
        ));

        assert_eq!(killswitch.state(), KillswitchState::DroppedAll);
        assert!(!killswitch.firewall().permits(EgressClass::Tor));
        assert!(!killswitch.firewall().permits(EgressClass::WireGuard));
    }

    #[test]
    fn interface_down_or_tunnel_loss_drops_all() {
        for change in [InterfaceChange::InterfaceDown, InterfaceChange::TunnelLost] {
            let mut killswitch = armed_killswitch();

            killswitch.on_interface_change(change);

            assert_eq!(killswitch.state(), KillswitchState::DroppedAll);
            assert!(!killswitch.firewall().permits(EgressClass::Tor));
            assert!(!killswitch.firewall().permits(EgressClass::WireGuard));
            assert!(!killswitch.firewall().permits(EgressClass::Other));
        }
    }

    #[test]
    fn ambiguity_drops_all_instead_of_guessing_safe_posture() {
        let mut killswitch = armed_killswitch();

        killswitch.on_interface_change(InterfaceChange::Ambiguous);

        assert_eq!(killswitch.state(), KillswitchState::DroppedAll);
        assert!(!killswitch.firewall().permits(EgressClass::Tor));
        assert!(!killswitch.firewall().permits(EgressClass::WireGuard));
    }

    #[test]
    fn healthy_change_with_wireguard_down_keeps_tor_only_default_drop() {
        let mut killswitch = armed_killswitch();

        killswitch.on_interface_change(InterfaceChange::Healthy(NetworkPosture::new(
            TunnelHealth::Active,
            TunnelHealth::Inactive,
        )));

        assert_eq!(killswitch.state(), KillswitchState::Armed);
        assert!(killswitch.firewall().permits(EgressClass::Tor));
        assert!(!killswitch.firewall().permits(EgressClass::WireGuard));
        assert!(!killswitch.firewall().permits(EgressClass::Other));
    }

    #[test]
    fn unknown_tunnel_state_is_not_allowed_to_pass() {
        let mut killswitch = armed_killswitch();

        killswitch.on_interface_change(InterfaceChange::Healthy(NetworkPosture::new(
            TunnelHealth::Active,
            TunnelHealth::Unknown,
        )));

        assert_eq!(killswitch.state(), KillswitchState::DroppedAll);
        assert!(!killswitch.firewall().permits(EgressClass::Tor));
        assert!(!killswitch.firewall().permits(EgressClass::WireGuard));
    }

    fn armed_killswitch() -> Killswitch {
        let mut killswitch = Killswitch::new();
        killswitch.arm(NetworkPosture::tor_and_wireguard_active());
        killswitch
    }
}
