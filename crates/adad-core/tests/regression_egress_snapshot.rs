use adad_core::EgressSnapshot;

#[test]
fn regression_fallback_requires_all_leak_guards() {
    let ready = EgressSnapshot::new(true, true, true, true, true);
    let cases = [
        EgressSnapshot::new(false, true, true, true, true),
        EgressSnapshot::new(true, false, true, true, true),
        EgressSnapshot::new(true, true, false, true, true),
        EgressSnapshot::new(true, true, true, false, true),
        EgressSnapshot::new(true, true, true, true, false),
    ];

    assert!(ready.leak_free_fallback_ready());
    for snapshot in cases {
        assert!(!snapshot.leak_free_fallback_ready());
    }
}
