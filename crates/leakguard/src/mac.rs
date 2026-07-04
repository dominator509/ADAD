use adad_core::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionSeed([u8; 32]);

impl SessionSeed {
    #[must_use]
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    fn bytes(self) -> [u8; 32] {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    #[must_use]
    pub fn octets(self) -> [u8; 6] {
        self.0
    }

    #[must_use]
    pub fn is_locally_administered(self) -> bool {
        self.0[0] & 0b0000_0010 != 0
    }

    #[must_use]
    pub fn is_multicast(self) -> bool {
        self.0[0] & 0b0000_0001 != 0
    }

    #[must_use]
    pub fn vendor_oui(self) -> [u8; 3] {
        [self.0[0], self.0[1], self.0[2]]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MacAssignment {
    pub iface: String,
    pub address: MacAddress,
}

pub fn randomize(iface: &str, session_seed: SessionSeed) -> Result<MacAssignment, Error> {
    if iface.trim().is_empty() {
        return Err(Error::Killswitch);
    }

    let mut state = 0xcbf2_9ce4_8422_2325_u64;
    for byte in session_seed
        .bytes()
        .into_iter()
        .chain(iface.as_bytes().iter().copied())
    {
        state ^= u64::from(byte);
        state = state.wrapping_mul(0x0000_0100_0000_01b3);
        state ^= state.rotate_left(17);
    }

    let mut octets = state.to_be_bytes();
    let mut address = [0_u8; 6];
    address.copy_from_slice(&octets[0..6]);
    address[0] &= 0b1111_1110;
    address[0] |= 0b0000_0010;
    octets.fill(0);

    Ok(MacAssignment {
        iface: iface.to_owned(),
        address: MacAddress(address),
    })
}

#[cfg(test)]
mod tests {
    use super::{randomize, SessionSeed};

    #[test]
    fn randomized_mac_sets_local_bit_and_clears_multicast_bit() {
        let assignment = randomize("wlan0", SessionSeed::new([7; 32])).expect("MAC assignment");

        assert!(assignment.address.is_locally_administered());
        assert!(!assignment.address.is_multicast());
    }
}
