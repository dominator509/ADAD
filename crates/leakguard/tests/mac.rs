use leakguard::{randomize, SessionSeed};

#[test]
fn randomized_mac_is_locally_administered_unicast() {
    let assignment = randomize("wlan0", SessionSeed::new([1; 32])).expect("MAC assignment");

    assert_eq!(assignment.iface, "wlan0");
    assert!(assignment.address.is_locally_administered());
    assert!(!assignment.address.is_multicast());
}

#[test]
fn randomized_mac_changes_across_sessions() {
    let first = randomize("wlan0", SessionSeed::new([1; 32])).expect("first MAC");
    let second = randomize("wlan0", SessionSeed::new([2; 32])).expect("second MAC");

    assert_ne!(first.address, second.address);
}

#[test]
fn randomized_mac_is_stable_within_a_session_for_one_interface() {
    let first = randomize("wlan0", SessionSeed::new([3; 32])).expect("first MAC");
    let second = randomize("wlan0", SessionSeed::new([3; 32])).expect("second MAC");

    assert_eq!(first.address, second.address);
}

#[test]
fn randomized_mac_does_not_use_a_real_vendor_oui_blend() {
    let assignment = randomize("eth0", SessionSeed::new([4; 32])).expect("MAC assignment");
    let oui = assignment.address.vendor_oui();

    assert_ne!(oui, [0x00, 0x1a, 0x2b]);
    assert_ne!(oui, [0x3c, 0x5a, 0xb4]);
    assert!(assignment.address.is_locally_administered());
}
