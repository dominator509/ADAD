pub mod fuse;

#[cfg(target_os = "linux")]
mod runtime;

pub use fuse::{
    scrub_metadata, ExifTag, PresentedMetadata, ScrubPolicy, ScrubbedTimestamps, VaultMetadata,
    SCRUBBED_TIME_BASE_UTC_SECONDS,
};

#[cfg(target_os = "linux")]
pub use runtime::mount_read_only;

#[cfg(not(target_os = "linux"))]
pub fn mount_read_only(
    _source: impl AsRef<std::path::Path>,
    _mountpoint: impl AsRef<std::path::Path>,
    _policy: ScrubPolicy,
) -> Result<(), adad_core::Error> {
    Err(adad_core::Error::Metafuse)
}
