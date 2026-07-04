#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EgressSnapshot {
    pub wireguard_tunnel_active: bool,
    pub default_drop: bool,
    pub direct_dns_blocked: bool,
    pub ipv6_blocked: bool,
    pub discovery_blocked: bool,
}

impl EgressSnapshot {
    #[must_use]
    pub fn new(
        wireguard_tunnel_active: bool,
        default_drop: bool,
        direct_dns_blocked: bool,
        ipv6_blocked: bool,
        discovery_blocked: bool,
    ) -> Self {
        Self {
            wireguard_tunnel_active,
            default_drop,
            direct_dns_blocked,
            ipv6_blocked,
            discovery_blocked,
        }
    }

    #[must_use]
    pub fn leak_free_fallback_ready(self) -> bool {
        self.wireguard_tunnel_active
            && self.default_drop
            && self.direct_dns_blocked
            && self.ipv6_blocked
            && self.discovery_blocked
    }
}

#[cfg(test)]
mod tests {
    use super::EgressSnapshot;

    #[test]
    fn fallback_ready_requires_every_leak_guard() {
        let ready = EgressSnapshot::new(true, true, true, true, true);
        let missing_wireguard = EgressSnapshot::new(false, true, true, true, true);
        let permissive_default = EgressSnapshot::new(true, false, true, true, true);

        assert!(ready.leak_free_fallback_ready());
        assert!(!missing_wireguard.leak_free_fallback_ready());
        assert!(!permissive_default.leak_free_fallback_ready());
    }
}
