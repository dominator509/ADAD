//! Shared pure-domain surface for ADAD crates.

mod config;
mod egress;
mod epoch;
mod error;
mod identity;
mod provider;

pub use config::{Config, SecretString};
pub use egress::EgressSnapshot;
pub use epoch::ZeroClockEpoch;
pub use error::{ConfigField, Error};
pub use identity::SessionIdentity;
pub use provider::Provider;

/// Current ADAD workspace version.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
