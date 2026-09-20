use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::time::Duration;

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

use adad_core::Error;

const LUKS_MAGIC: [u8; 6] = *b"LUKS\xBA\xBE";
const LUKS2_VERSION: u16 = 2;
const IO_CHUNK_BYTES: usize = 8192;

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

/// A real LUKS2 header target restricted to a regular image file.
///
/// The regular-file restriction is deliberate: this backend cannot open a
/// block device, so tests and the image-only DMS path cannot accidentally
/// operate on a host disk.
pub struct LuksHeaderFile {
    file: File,
    header_len: u64,
}

impl std::fmt::Debug for LuksHeaderFile {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LuksHeaderFile")
            .field("header_len", &self.header_len)
            .finish()
    }
}

impl LuksHeaderFile {
    pub fn open(path: &Path, header_len: u64) -> Result<Self, Error> {
        if header_len < 8 {
            return Err(Error::VaultUnlock);
        }

        let metadata = fs::symlink_metadata(path).map_err(|_| Error::VaultUnlock)?;
        if !metadata.file_type().is_file() || metadata.len() < header_len {
            return Err(Error::VaultUnlock);
        }

        let mut options = OpenOptions::new();
        options.read(true).write(true);
        #[cfg(unix)]
        options.custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
        let mut file = options.open(path).map_err(|_| Error::VaultUnlock)?;

        let mut prefix = [0_u8; 8];
        file.read_exact(&mut prefix)
            .map_err(|_| Error::VaultUnlock)?;
        if prefix[..6] != LUKS_MAGIC || u16::from_be_bytes([prefix[6], prefix[7]]) != LUKS2_VERSION
        {
            return Err(Error::VaultUnlock);
        }

        file.seek(SeekFrom::Start(0)).map_err(|_| Error::Io)?;
        Ok(Self { file, header_len })
    }

    #[must_use]
    pub fn header_len(&self) -> u64 {
        self.header_len
    }

    /// Zero and verify the selected LUKS2 header bytes, then flush them.
    pub fn wipe_header(&mut self) -> Result<bool, Error> {
        self.file.seek(SeekFrom::Start(0)).map_err(|_| Error::Io)?;
        let zeros = [0_u8; IO_CHUNK_BYTES];
        let mut remaining = self.header_len;
        while remaining > 0 {
            let count = usize::try_from(remaining)
                .unwrap_or(IO_CHUNK_BYTES)
                .min(IO_CHUNK_BYTES);
            self.file
                .write_all(&zeros[..count])
                .map_err(|_| Error::Io)?;
            remaining -= u64::try_from(count).expect("chunk size fits u64");
        }
        self.file.sync_all().map_err(|_| Error::Io)?;

        self.file.seek(SeekFrom::Start(0)).map_err(|_| Error::Io)?;
        let mut buffer = [0_u8; IO_CHUNK_BYTES];
        let mut remaining = self.header_len;
        while remaining > 0 {
            let count = usize::try_from(remaining)
                .unwrap_or(IO_CHUNK_BYTES)
                .min(IO_CHUNK_BYTES);
            self.file
                .read_exact(&mut buffer[..count])
                .map_err(|_| Error::Io)?;
            if buffer[..count].iter().any(|byte| *byte != 0) {
                return Err(Error::VaultUnlock);
            }
            remaining -= u64::try_from(count).expect("chunk size fits u64");
        }
        Ok(true)
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
        for byte in &mut self.bytes {
            // SAFETY: each pointer is derived from a unique mutable slice
            // reference and remains valid for this loop iteration.
            unsafe { std::ptr::write_volatile(byte, 0) };
        }
        std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
    }
}

impl Drop for RamSecret {
    fn drop(&mut self) {
        self.wipe();
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
        match self.remaining(tor_ntp) {
            Some(remaining) => Ok(DmsOutcome::Armed { remaining }),
            None => {
                image.wipe_header();
                Ok(DmsOutcome::Expired {
                    header_wiped: image.header_wiped(),
                })
            }
        }
    }

    /// Evaluate against a real, disposable LUKS2 image file.
    pub fn evaluate_file(
        &mut self,
        tor_ntp: TorNtpTime,
        _local_clock: LocalClockTime,
        image: &mut LuksHeaderFile,
    ) -> Result<DmsOutcome, Error> {
        match self.remaining(tor_ntp) {
            Some(remaining) => Ok(DmsOutcome::Armed { remaining }),
            None => Ok(DmsOutcome::Expired {
                header_wiped: image.wipe_header()?,
            }),
        }
    }

    fn remaining(&mut self, tor_ntp: TorNtpTime) -> Option<Duration> {
        if self.state == DmsState::Expired {
            return None;
        }

        let Some(elapsed) = tor_ntp.duration_since(self.last_vault_access) else {
            self.state = DmsState::Expired;
            return None;
        };

        if elapsed >= self.window {
            self.state = DmsState::Expired;
            None
        } else {
            Some(self.window - elapsed)
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

/// Wipe RAM secrets and a disposable LUKS2 image target together.
pub fn panic_wipe_file(
    secrets: &mut [RamSecret],
    image: &mut LuksHeaderFile,
) -> Result<PanicWipeReport, Error> {
    for secret in secrets.iter_mut() {
        secret.wipe();
    }

    let header_wiped = image.wipe_header()?;
    Ok(PanicWipeReport {
        ram_wiped_count: secrets.iter().filter(|secret| secret.is_wiped()).count(),
        header_wiped,
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
