use adad_core::{EgressSnapshot, Error};

use crate::{EgressClass, FirewallAction, FirewallPosture};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrafficClass {
    General,
    Api,
    Dns,
    Ipv6,
    Mdns,
    Ssdp,
    Netbios,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RouteTarget {
    Tor,
    WireGuard,
    Blocked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoutingPosture {
    general: RouteTarget,
    api: RouteTarget,
    dns: RouteTarget,
    ipv6_enabled: bool,
    mdns: RouteTarget,
    ssdp: RouteTarget,
    netbios: RouteTarget,
}

impl RoutingPosture {
    #[must_use]
    pub fn leak_free() -> Self {
        Self {
            general: RouteTarget::Tor,
            api: RouteTarget::WireGuard,
            dns: RouteTarget::Tor,
            ipv6_enabled: false,
            mdns: RouteTarget::Blocked,
            ssdp: RouteTarget::Blocked,
            netbios: RouteTarget::Blocked,
        }
    }

    #[must_use]
    pub fn route_for(&self, class: TrafficClass) -> RouteTarget {
        match class {
            TrafficClass::General => self.general,
            TrafficClass::Api => self.api,
            TrafficClass::Dns => self.dns,
            TrafficClass::Ipv6 => {
                if self.ipv6_enabled {
                    RouteTarget::Tor
                } else {
                    RouteTarget::Blocked
                }
            }
            TrafficClass::Mdns => self.mdns,
            TrafficClass::Ssdp => self.ssdp,
            TrafficClass::Netbios => self.netbios,
        }
    }

    #[must_use]
    pub fn ipv6_enabled(&self) -> bool {
        self.ipv6_enabled
    }

    #[must_use]
    pub fn firewall_posture(&self) -> FirewallPosture {
        FirewallPosture::default_drop(true, true)
    }

    #[must_use]
    pub fn egress_snapshot(&self, firewall: FirewallPosture) -> EgressSnapshot {
        EgressSnapshot::new(
            self.route_for(TrafficClass::Api) == RouteTarget::WireGuard
                && firewall.permits(EgressClass::WireGuard),
            firewall.default_action() == FirewallAction::Drop,
            self.route_for(TrafficClass::Dns) == RouteTarget::Tor
                && !firewall.permits(EgressClass::DirectDns),
            self.route_for(TrafficClass::Ipv6) == RouteTarget::Blocked
                && !self.ipv6_enabled()
                && !firewall.permits(EgressClass::Ipv6),
            self.route_for(TrafficClass::Mdns) == RouteTarget::Blocked
                && self.route_for(TrafficClass::Ssdp) == RouteTarget::Blocked
                && self.route_for(TrafficClass::Netbios) == RouteTarget::Blocked
                && !firewall.permits(EgressClass::Mdns)
                && !firewall.permits(EgressClass::Ssdp)
                && !firewall.permits(EgressClass::Netbios),
        )
    }

    pub fn validate_leak_free(&self) -> Result<(), Error> {
        let safe = self.route_for(TrafficClass::General) == RouteTarget::Tor
            && self.route_for(TrafficClass::Api) == RouteTarget::WireGuard
            && self.route_for(TrafficClass::Dns) == RouteTarget::Tor
            && self.route_for(TrafficClass::Ipv6) == RouteTarget::Blocked
            && self.route_for(TrafficClass::Mdns) == RouteTarget::Blocked
            && self.route_for(TrafficClass::Ssdp) == RouteTarget::Blocked
            && self.route_for(TrafficClass::Netbios) == RouteTarget::Blocked
            && !self.ipv6_enabled()
            && !self.firewall_posture().permits(EgressClass::DirectDns)
            && !self.firewall_posture().permits(EgressClass::Ipv6)
            && !self.firewall_posture().permits(EgressClass::Mdns)
            && !self.firewall_posture().permits(EgressClass::Ssdp)
            && !self.firewall_posture().permits(EgressClass::Netbios);

        if safe {
            Ok(())
        } else {
            Err(Error::Killswitch)
        }
    }
}

impl Default for RoutingPosture {
    fn default() -> Self {
        Self::leak_free()
    }
}

#[cfg(test)]
mod tests {
    use super::{RouteTarget, RoutingPosture, TrafficClass};

    #[test]
    fn leak_free_posture_routes_only_through_tor_or_wireguard() {
        let posture = RoutingPosture::leak_free();

        assert_eq!(posture.route_for(TrafficClass::General), RouteTarget::Tor);
        assert_eq!(posture.route_for(TrafficClass::Api), RouteTarget::WireGuard);
        assert_eq!(posture.route_for(TrafficClass::Dns), RouteTarget::Tor);
        assert_eq!(posture.route_for(TrafficClass::Ipv6), RouteTarget::Blocked);
        assert_eq!(posture.route_for(TrafficClass::Mdns), RouteTarget::Blocked);
        assert_eq!(posture.route_for(TrafficClass::Ssdp), RouteTarget::Blocked);
        assert_eq!(
            posture.route_for(TrafficClass::Netbios),
            RouteTarget::Blocked
        );
        assert!(posture.validate_leak_free().is_ok());
    }
}
