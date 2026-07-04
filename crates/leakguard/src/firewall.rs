#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirewallAction {
    Allow,
    Drop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EgressClass {
    Tor,
    WireGuard,
    DirectDns,
    Ipv6,
    Mdns,
    Ssdp,
    Netbios,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirewallPosture {
    default_action: FirewallAction,
    tor: FirewallAction,
    wireguard: FirewallAction,
}

impl FirewallPosture {
    #[must_use]
    pub fn drop_all() -> Self {
        Self {
            default_action: FirewallAction::Drop,
            tor: FirewallAction::Drop,
            wireguard: FirewallAction::Drop,
        }
    }

    #[must_use]
    pub fn default_drop(allow_tor: bool, allow_wireguard: bool) -> Self {
        Self {
            default_action: FirewallAction::Drop,
            tor: allow_action(allow_tor),
            wireguard: allow_action(allow_wireguard),
        }
    }

    #[must_use]
    pub fn permits(&self, class: EgressClass) -> bool {
        match class {
            EgressClass::Tor => self.tor == FirewallAction::Allow,
            EgressClass::WireGuard => self.wireguard == FirewallAction::Allow,
            EgressClass::DirectDns
            | EgressClass::Ipv6
            | EgressClass::Mdns
            | EgressClass::Ssdp
            | EgressClass::Netbios
            | EgressClass::Other => self.default_action == FirewallAction::Allow,
        }
    }

    #[must_use]
    pub fn default_action(&self) -> FirewallAction {
        self.default_action
    }
}

fn allow_action(allowed: bool) -> FirewallAction {
    if allowed {
        FirewallAction::Allow
    } else {
        FirewallAction::Drop
    }
}

#[cfg(test)]
mod tests {
    use super::{EgressClass, FirewallAction, FirewallPosture};

    #[test]
    fn drop_all_permits_no_egress_classes() {
        let posture = FirewallPosture::drop_all();

        assert_eq!(posture.default_action(), FirewallAction::Drop);
        assert!(!posture.permits(EgressClass::Tor));
        assert!(!posture.permits(EgressClass::WireGuard));
        assert!(!posture.permits(EgressClass::DirectDns));
        assert!(!posture.permits(EgressClass::Ipv6));
        assert!(!posture.permits(EgressClass::Mdns));
        assert!(!posture.permits(EgressClass::Ssdp));
        assert!(!posture.permits(EgressClass::Netbios));
        assert!(!posture.permits(EgressClass::Other));
    }

    #[test]
    fn default_drop_only_allows_requested_tunnel_classes() {
        let posture = FirewallPosture::default_drop(true, true);

        assert_eq!(posture.default_action(), FirewallAction::Drop);
        assert!(posture.permits(EgressClass::Tor));
        assert!(posture.permits(EgressClass::WireGuard));
        assert!(!posture.permits(EgressClass::DirectDns));
        assert!(!posture.permits(EgressClass::Ipv6));
        assert!(!posture.permits(EgressClass::Mdns));
        assert!(!posture.permits(EgressClass::Ssdp));
        assert!(!posture.permits(EgressClass::Netbios));
        assert!(!posture.permits(EgressClass::Other));
    }
}
