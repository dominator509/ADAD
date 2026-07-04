use leakguard::{
    EgressClass, InterfaceChange, Killswitch, NetworkPosture, RouteTarget, RoutingPosture,
    TrafficClass,
};

#[test]
fn routing_posture_sends_general_to_tor_and_api_to_wireguard() {
    let posture = RoutingPosture::leak_free();

    assert_eq!(posture.route_for(TrafficClass::General), RouteTarget::Tor);
    assert_eq!(posture.route_for(TrafficClass::Api), RouteTarget::WireGuard);
    assert_eq!(posture.route_for(TrafficClass::Dns), RouteTarget::Tor);
}

#[test]
fn routing_posture_blocks_ipv6_and_local_discovery_classes() {
    let posture = RoutingPosture::leak_free();

    assert!(!posture.ipv6_enabled());
    assert_eq!(posture.route_for(TrafficClass::Ipv6), RouteTarget::Blocked);
    assert_eq!(posture.route_for(TrafficClass::Mdns), RouteTarget::Blocked);
    assert_eq!(posture.route_for(TrafficClass::Ssdp), RouteTarget::Blocked);
    assert_eq!(
        posture.route_for(TrafficClass::Netbios),
        RouteTarget::Blocked
    );
}

#[test]
fn firewall_model_blocks_direct_dns_ipv6_and_discovery_egress() {
    let firewall = RoutingPosture::leak_free().firewall_posture();

    assert!(firewall.permits(EgressClass::Tor));
    assert!(firewall.permits(EgressClass::WireGuard));
    assert!(!firewall.permits(EgressClass::DirectDns));
    assert!(!firewall.permits(EgressClass::Ipv6));
    assert!(!firewall.permits(EgressClass::Mdns));
    assert!(!firewall.permits(EgressClass::Ssdp));
    assert!(!firewall.permits(EgressClass::Netbios));
    assert!(!firewall.permits(EgressClass::Other));
}

#[test]
fn routing_posture_validates_as_leak_free() {
    let posture = RoutingPosture::leak_free();

    posture
        .validate_leak_free()
        .expect("model posture should not allow clearnet leaks");
}

#[test]
fn egress_snapshot_requires_wireguard_and_every_leak_block() {
    let routing = RoutingPosture::leak_free();
    let mut killswitch = Killswitch::new();
    killswitch.arm(NetworkPosture::tor_and_wireguard_active());

    let ready = routing.egress_snapshot(killswitch.firewall());
    assert!(ready.leak_free_fallback_ready());

    killswitch.on_interface_change(InterfaceChange::TunnelLost);
    let dropped = routing.egress_snapshot(killswitch.firewall());
    assert!(!dropped.leak_free_fallback_ready());
}
