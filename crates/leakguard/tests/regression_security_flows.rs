use std::time::Duration;

use leakguard::{
    Dms, DmsOutcome, EgressClass, InterfaceChange, Killswitch, KillswitchState, LocalClockTime,
    LuksHeaderImage, NetworkPosture, RoutingPosture, TorNtpTime,
};

#[test]
fn regression_tunnel_loss_forces_drop_all_and_blocks_fallback_snapshot() {
    let routing = RoutingPosture::leak_free();
    let mut killswitch = Killswitch::new();
    killswitch.arm(NetworkPosture::tor_and_wireguard_active());
    assert!(routing
        .egress_snapshot(killswitch.firewall())
        .leak_free_fallback_ready());

    killswitch.on_interface_change(InterfaceChange::TunnelLost);

    assert_eq!(killswitch.state(), KillswitchState::DroppedAll);
    assert!(!killswitch.firewall().permits(EgressClass::Tor));
    assert!(!killswitch.firewall().permits(EgressClass::WireGuard));
    assert!(!routing
        .egress_snapshot(killswitch.firewall())
        .leak_free_fallback_ready());
}

#[test]
fn regression_dms_ignores_frozen_local_clock_and_wipes_image_header() {
    let mut dms = Dms::new(
        Duration::from_secs(90),
        TorNtpTime::from_unix_seconds(1_000),
    )
    .expect("DMS");
    let frozen_local_clock = LocalClockTime::from_unix_seconds(1_000);
    let mut image = LuksHeaderImage::new(vec![0xAA; 256], 64).expect("image target");

    let outcome = dms
        .evaluate(
            TorNtpTime::from_unix_seconds(1_091),
            frozen_local_clock,
            &mut image,
        )
        .expect("DMS evaluation");

    assert_eq!(outcome, DmsOutcome::Expired { header_wiped: true });
    assert!(image.header_wiped());
    assert!(image.payload_preserved());
}
