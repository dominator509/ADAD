use std::time::Duration;

use adad_core::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TorNtpTime {
    seconds: u64,
}

impl TorNtpTime {
    #[must_use]
    pub fn from_unix_seconds(seconds: u64) -> Self {
        Self { seconds }
    }

    #[must_use]
    pub fn seconds(self) -> u64 {
        self.seconds
    }

    fn duration_since(self, earlier: Self) -> Option<Duration> {
        self.seconds
            .checked_sub(earlier.seconds)
            .map(Duration::from_secs)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocalClockTime {
    seconds: u64,
}

impl LocalClockTime {
    #[must_use]
    pub fn from_unix_seconds(seconds: u64) -> Self {
        Self { seconds }
    }

    #[must_use]
    pub fn seconds(self) -> u64 {
        self.seconds
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LuksHeaderImage {
    bytes: Vec<u8>,
    header_len: usize,
}

impl LuksHeaderImage {
    pub fn new(bytes: Vec<u8>, header_len: usize) -> Result<Self, Error> {
        if header_len == 0 || bytes.len() < header_len {
            return Err(Error::VaultUnlock);
        }

        Ok(Self { bytes, header_len })
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub fn header_len(&self) -> usize {
        self.header_len
    }

    #[must_use]
    pub fn header_wiped(&self) -> bool {
        self.bytes[..self.header_len].iter().all(|byte| *byte == 0)
    }

    #[must_use]
    pub fn payload_preserved(&self) -> bool {
        self.bytes[self.header_len..].iter().all(|byte| *byte != 0)
    }

    fn wipe_header(&mut self) {
        self.bytes[..self.header_len].fill(0);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RamSecret {
    bytes: Vec<u8>,
}

impl RamSecret {
    #[must_use]
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    #[must_use]
    pub fn is_wiped(&self) -> bool {
        self.bytes.iter().all(|byte| *byte == 0)
    }

    fn wipe(&mut self) {
        self.bytes.fill(0);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DmsState {
    Armed,
    Expired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DmsOutcome {
    Armed { remaining: Duration },
    Expired { header_wiped: bool },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dms {
    window: Duration,
    last_vault_access: TorNtpTime,
    state: DmsState,
}

impl Dms {
    pub fn new(window: Duration, tor_ntp: TorNtpTime) -> Result<Self, Error> {
        if window.is_zero() {
            return Err(Error::VaultUnlock);
        }

        Ok(Self {
            window,
            last_vault_access: tor_ntp,
            state: DmsState::Armed,
        })
    }

    pub fn record_vault_access(&mut self, tor_ntp: TorNtpTime) -> Result<(), Error> {
        if self.state == DmsState::Expired {
            return Err(Error::VaultUnlock);
        }

        self.last_vault_access = tor_ntp;
        Ok(())
    }

    #[must_use]
    pub fn state(&self) -> DmsState {
        self.state
    }

    #[must_use]
    pub fn last_vault_access(&self) -> TorNtpTime {
        self.last_vault_access
    }

    pub fn evaluate(
        &mut self,
        tor_ntp: TorNtpTime,
        _local_clock: LocalClockTime,
        image: &mut LuksHeaderImage,
    ) -> Result<DmsOutcome, Error> {
        if self.state == DmsState::Expired {
            image.wipe_header();
            return Ok(DmsOutcome::Expired {
                header_wiped: image.header_wiped(),
            });
        }

        let Some(elapsed) = tor_ntp.duration_since(self.last_vault_access) else {
            image.wipe_header();
            self.state = DmsState::Expired;
            return Ok(DmsOutcome::Expired {
                header_wiped: image.header_wiped(),
            });
        };

        if elapsed >= self.window {
            image.wipe_header();
            self.state = DmsState::Expired;
            Ok(DmsOutcome::Expired {
                header_wiped: image.header_wiped(),
            })
        } else {
            Ok(DmsOutcome::Armed {
                remaining: self.window - elapsed,
            })
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PanicWipeReport {
    pub ram_wiped_count: usize,
    pub header_wiped: bool,
    pub image_only: bool,
}

pub fn panic_wipe(
    secrets: &mut [RamSecret],
    image: &mut LuksHeaderImage,
) -> Result<PanicWipeReport, Error> {
    for secret in secrets.iter_mut() {
        secret.wipe();
    }
    image.wipe_header();

    Ok(PanicWipeReport {
        ram_wiped_count: secrets.iter().filter(|secret| secret.is_wiped()).count(),
        header_wiped: image.header_wiped(),
        image_only: true,
    })
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{Dms, DmsOutcome, LocalClockTime, LuksHeaderImage, TorNtpTime};

    #[test]
    fn expiry_wipes_header_on_image() {
        let mut dms = Dms::new(
            Duration::from_secs(60),
            TorNtpTime::from_unix_seconds(1_000),
        )
        .expect("DMS");
        let mut image = test_image();

        let outcome = dms
            .evaluate(
                TorNtpTime::from_unix_seconds(1_061),
                LocalClockTime::from_unix_seconds(1_000),
                &mut image,
            )
            .expect("DMS evaluation");

        assert_eq!(outcome, DmsOutcome::Expired { header_wiped: true });
        assert!(image.header_wiped());
        assert!(image.payload_preserved());
    }

    fn test_image() -> LuksHeaderImage {
        LuksHeaderImage::new(vec![0xAA; 128], 32).expect("test image")
    }
}
