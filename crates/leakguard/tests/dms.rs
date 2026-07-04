use std::time::Duration;

use leakguard::{
    panic_wipe, Dms, DmsOutcome, DmsState, LocalClockTime, LuksHeaderImage, PanicWipeReport,
    RamSecret, TorNtpTime,
};

#[test]
fn dms_expiry_wipes_luks_header_on_image_only() {
    let mut dms = Dms::new(
        Duration::from_secs(300),
        TorNtpTime::from_unix_seconds(10_000),
    )
    .expect("DMS");
    let mut image = test_image();

    let outcome = dms
        .evaluate(
            TorNtpTime::from_unix_seconds(10_301),
            LocalClockTime::from_unix_seconds(10_301),
            &mut image,
        )
        .expect("DMS evaluation");

    assert_eq!(outcome, DmsOutcome::Expired { header_wiped: true });
    assert_eq!(dms.state(), DmsState::Expired);
    assert!(image.header_wiped());
    assert!(image.payload_preserved());
}

#[test]
fn frozen_local_clock_does_not_extend_tor_ntp_window() {
    let mut dms = Dms::new(
        Duration::from_secs(120),
        TorNtpTime::from_unix_seconds(20_000),
    )
    .expect("DMS");
    let frozen_local_clock = LocalClockTime::from_unix_seconds(20_000);
    let mut image = test_image();

    let still_armed = dms
        .evaluate(
            TorNtpTime::from_unix_seconds(20_060),
            frozen_local_clock,
            &mut image,
        )
        .expect("first DMS evaluation");

    assert!(matches!(still_armed, DmsOutcome::Armed { .. }));
    assert!(!image.header_wiped());

    let expired = dms
        .evaluate(
            TorNtpTime::from_unix_seconds(20_121),
            frozen_local_clock,
            &mut image,
        )
        .expect("second DMS evaluation");

    assert_eq!(expired, DmsOutcome::Expired { header_wiped: true });
    assert!(image.header_wiped());
}

#[test]
fn tor_ntp_regression_fails_closed_by_wiping_header() {
    let mut dms = Dms::new(
        Duration::from_secs(120),
        TorNtpTime::from_unix_seconds(30_000),
    )
    .expect("DMS");
    let mut image = test_image();

    let outcome = dms
        .evaluate(
            TorNtpTime::from_unix_seconds(29_999),
            LocalClockTime::from_unix_seconds(30_001),
            &mut image,
        )
        .expect("DMS evaluation");

    assert_eq!(outcome, DmsOutcome::Expired { header_wiped: true });
    assert!(image.header_wiped());
}

#[test]
fn vault_access_refreshes_tor_ntp_deadline() {
    let mut dms = Dms::new(
        Duration::from_secs(120),
        TorNtpTime::from_unix_seconds(40_000),
    )
    .expect("DMS");
    let mut image = test_image();

    dms.record_vault_access(TorNtpTime::from_unix_seconds(40_100))
        .expect("vault access refresh");
    let outcome = dms
        .evaluate(
            TorNtpTime::from_unix_seconds(40_190),
            LocalClockTime::from_unix_seconds(40_500),
            &mut image,
        )
        .expect("DMS evaluation");

    assert!(matches!(outcome, DmsOutcome::Armed { .. }));
    assert!(!image.header_wiped());
    assert_eq!(
        dms.last_vault_access(),
        TorNtpTime::from_unix_seconds(40_100)
    );
}

#[test]
fn panic_path_wipes_ram_and_image_header_in_sandbox() {
    let mut image = test_image();
    let mut secrets = vec![
        RamSecret::new(vec![1, 2, 3, 4]),
        RamSecret::new(vec![5, 6, 7, 8]),
    ];

    let report = panic_wipe(&mut secrets, &mut image).expect("panic wipe");

    assert_eq!(
        report,
        PanicWipeReport {
            ram_wiped_count: 2,
            header_wiped: true,
            image_only: true,
        }
    );
    assert!(secrets.iter().all(RamSecret::is_wiped));
    assert!(image.header_wiped());
    assert!(image.payload_preserved());
}

fn test_image() -> LuksHeaderImage {
    let mut bytes = vec![0xAA; 128];
    bytes.extend([0xBB; 128]);
    LuksHeaderImage::new(bytes, 128).expect("test image")
}
