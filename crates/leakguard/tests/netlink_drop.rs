use leakguard::{
    EgressClass, Killswitch, KillswitchState, NetlinkEvent, NetlinkMonitor, NetworkPosture,
    DROP_ALL_TARGET_LATENCY,
};

#[test]
fn simulated_netlink_interface_drop_forces_drop_all_within_target_latency() {
    let monitor = NetlinkMonitor::with_default_target();
    let mut killswitch = Killswitch::new();
    killswitch.arm(NetworkPosture::tor_and_wireguard_active());

    let reaction = monitor.handle_event(
        &mut killswitch,
        NetlinkEvent::InterfaceDown {
            iface: "wg0".to_owned(),
        },
    );

    assert_eq!(reaction.iface, "wg0");
    assert_eq!(reaction.state, KillswitchState::DroppedAll);
    assert!(reaction.elapsed <= monitor.target_latency());
    assert!(reaction.elapsed <= DROP_ALL_TARGET_LATENCY);
    assert!(!killswitch.firewall().permits(EgressClass::Tor));
    assert!(!killswitch.firewall().permits(EgressClass::WireGuard));
    assert!(!killswitch.firewall().permits(EgressClass::Other));
}

#[test]
fn simulated_netlink_tunnel_loss_forces_drop_all_within_target_latency() {
    let monitor = NetlinkMonitor::with_default_target();
    let mut killswitch = Killswitch::new();
    killswitch.arm(NetworkPosture::tor_and_wireguard_active());

    let reaction = monitor.handle_event(
        &mut killswitch,
        NetlinkEvent::TunnelLost {
            iface: "wg0".to_owned(),
        },
    );

    assert_eq!(reaction.state, KillswitchState::DroppedAll);
    assert!(reaction.elapsed <= monitor.target_latency());
}
