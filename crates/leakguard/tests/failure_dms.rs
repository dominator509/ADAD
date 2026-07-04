use std::time::Duration;

use leakguard::{Dms, DmsOutcome, LocalClockTime, LuksHeaderImage, TorNtpTime};

#[test]
fn failure_zero_dms_window_is_rejected() {
    assert!(Dms::new(Duration::ZERO, TorNtpTime::from_unix_seconds(1_000)).is_err());
}

#[test]
fn failure_tor_ntp_regression_fails_closed_by_wiping_header() {
    let mut dms = Dms::new(
        Duration::from_secs(60),
        TorNtpTime::from_unix_seconds(1_000),
    )
    .expect("DMS");
    let mut image = LuksHeaderImage::new(vec![0xAA; 128], 32).expect("image target");

    let outcome = dms
        .evaluate(
            TorNtpTime::from_unix_seconds(999),
            LocalClockTime::from_unix_seconds(1_001),
            &mut image,
        )
        .expect("DMS evaluation");

    assert_eq!(outcome, DmsOutcome::Expired { header_wiped: true });
    assert!(image.header_wiped());
}
