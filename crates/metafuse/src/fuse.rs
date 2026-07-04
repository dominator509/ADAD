use adad_core::Error;

pub const SCRUBBED_TIME_BASE_UTC_SECONDS: i64 = 946_684_800;
const DEFAULT_TIMESTAMP_WINDOW_SECONDS: u32 = 30 * 24 * 60 * 60;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScrubbedTimestamps {
    pub created_at: i64,
    pub modified_at: i64,
    pub accessed_at: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExifTag {
    pub name: String,
    pub value: String,
}

impl ExifTag {
    #[must_use]
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultMetadata {
    pub vault_path: String,
    pub uid: u32,
    pub gid: u32,
    pub created_at: i64,
    pub modified_at: i64,
    pub accessed_at: i64,
    pub exif: Vec<ExifTag>,
}

impl VaultMetadata {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        vault_path: impl Into<String>,
        uid: u32,
        gid: u32,
        created_at: i64,
        modified_at: i64,
        accessed_at: i64,
        exif: Vec<ExifTag>,
    ) -> Self {
        Self {
            vault_path: vault_path.into(),
            uid,
            gid,
            created_at,
            modified_at,
            accessed_at,
            exif,
        }
    }

    #[must_use]
    pub fn rendered_private_fields(&self) -> String {
        let exif = self
            .exif
            .iter()
            .map(|tag| format!("{}={}", tag.name, tag.value))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            self.uid, self.gid, self.created_at, self.modified_at, self.accessed_at, exif
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScrubPolicy {
    pub fake_uid: u32,
    pub fake_gid: u32,
    pub timestamp_seed: [u8; 32],
    pub timestamp_window_seconds: u32,
}

impl ScrubPolicy {
    pub fn new(fake_uid: u32, fake_gid: u32, timestamp_seed: [u8; 32]) -> Result<Self, Error> {
        if fake_uid == 0 || fake_gid == 0 {
            return Err(Error::Metafuse);
        }

        Ok(Self {
            fake_uid,
            fake_gid,
            timestamp_seed,
            timestamp_window_seconds: DEFAULT_TIMESTAMP_WINDOW_SECONDS,
        })
    }

    pub fn with_timestamp_window_seconds(mut self, seconds: u32) -> Result<Self, Error> {
        if seconds == 0 {
            return Err(Error::Metafuse);
        }

        self.timestamp_window_seconds = seconds;
        Ok(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentedMetadata {
    pub vault_path: String,
    pub uid: u32,
    pub gid: u32,
    pub timestamps: ScrubbedTimestamps,
    pub exif: Vec<ExifTag>,
}

impl PresentedMetadata {
    #[must_use]
    pub fn rendered_public_fields(&self) -> String {
        let exif = self
            .exif
            .iter()
            .map(|tag| format!("{}={}", tag.name, tag.value))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            self.uid,
            self.gid,
            self.timestamps.created_at,
            self.timestamps.modified_at,
            self.timestamps.accessed_at,
            exif
        )
    }
}

pub fn scrub_metadata(
    metadata: &VaultMetadata,
    policy: &ScrubPolicy,
) -> Result<PresentedMetadata, Error> {
    if metadata.vault_path.trim().is_empty()
        || policy.fake_uid == 0
        || policy.fake_gid == 0
        || policy.timestamp_window_seconds == 0
    {
        return Err(Error::Metafuse);
    }

    Ok(PresentedMetadata {
        vault_path: metadata.vault_path.clone(),
        uid: policy.fake_uid,
        gid: policy.fake_gid,
        timestamps: ScrubbedTimestamps {
            created_at: scrubbed_timestamp(metadata, policy, b"created"),
            modified_at: scrubbed_timestamp(metadata, policy, b"modified"),
            accessed_at: scrubbed_timestamp(metadata, policy, b"accessed"),
        },
        exif: Vec::new(),
    })
}

fn scrubbed_timestamp(metadata: &VaultMetadata, policy: &ScrubPolicy, label: &[u8]) -> i64 {
    let mut state = 0xcbf2_9ce4_8422_2325_u64;
    for byte in policy
        .timestamp_seed
        .into_iter()
        .chain(metadata.vault_path.as_bytes().iter().copied())
        .chain(label.iter().copied())
    {
        state ^= u64::from(byte);
        state = state.wrapping_mul(0x0000_0100_0000_01b3);
        state ^= state.rotate_left(17);
    }

    let offset = state % u64::from(policy.timestamp_window_seconds);
    SCRUBBED_TIME_BASE_UTC_SECONDS + i64::try_from(offset).expect("u32 window fits i64")
}

#[cfg(test)]
mod tests {
    use super::{scrub_metadata, ExifTag, ScrubPolicy, VaultMetadata};

    #[test]
    fn scrubbed_metadata_uses_fake_owner_and_hides_real_timestamps() {
        let metadata = vault_metadata();
        let policy = ScrubPolicy::new(65_534, 65_533, [7; 32]).expect("policy");

        let presented = scrub_metadata(&metadata, &policy).expect("scrubbed metadata");

        assert_eq!(presented.uid, 65_534);
        assert_eq!(presented.gid, 65_533);
        assert_ne!(presented.timestamps.created_at, metadata.created_at);
        assert_ne!(presented.timestamps.modified_at, metadata.modified_at);
        assert_ne!(presented.timestamps.accessed_at, metadata.accessed_at);
    }

    fn vault_metadata() -> VaultMetadata {
        VaultMetadata::new(
            "/vault/src/photo.jpg",
            1000,
            1000,
            1_783_123_200,
            1_783_126_800,
            1_783_130_400,
            vec![ExifTag::new("CameraSerial", "real-camera-serial")],
        )
    }
}
