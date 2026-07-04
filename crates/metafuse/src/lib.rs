pub mod fuse;

pub use fuse::{
    scrub_metadata, ExifTag, PresentedMetadata, ScrubPolicy, ScrubbedTimestamps, VaultMetadata,
    SCRUBBED_TIME_BASE_UTC_SECONDS,
};
