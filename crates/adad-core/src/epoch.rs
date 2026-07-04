/// Host-independent timestamp base used by forge-rs for sterile writes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ZeroClockEpoch {
    unix_seconds: u64,
}

impl ZeroClockEpoch {
    #[must_use]
    pub fn from_seed(seed: &[u8]) -> Self {
        let hash = seed.iter().fold(0xcbf2_9ce4_8422_2325_u64, |acc, byte| {
            (acc ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        });

        // Keep the epoch in a bounded synthetic window so it never needs the host clock.
        let unix_seconds = hash % (50_u64 * 365 * 24 * 60 * 60);
        Self { unix_seconds }
    }

    #[must_use]
    pub fn as_unix_seconds(self) -> u64 {
        self.unix_seconds
    }
}
