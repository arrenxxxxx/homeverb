//! Bounded native Mijia cloud protocol. No background tasks or implicit controls.

mod adapter;
mod auth;
mod codec;
mod inventory;
mod protected;
mod spec;
mod transport;
pub use adapter::{AccountDiscovery, AccountInventory, DeviceInventoryMembership, MijiaAdapter};
pub use auth::{LoginCredentials, MijiaSession, login};
pub use inventory::{CloudDevice, Inventory, PropertyQuery, PropertyReading};
pub use spec::{MappedProperty, ModelMapping, SpecIndex};
pub use transport::{CloudEndpoint, CloudRequest, CloudResponse, CloudTransport, MijiaHttps};

use homeverb_domain::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::Zeroizing;

pub const PROVIDER: &str = "xiaomi.mijia";
pub const MAX_RESPONSE_BYTES: usize = 1_048_576;

/// Exact supported cloud origin selection, never an arbitrary URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Region {
    Cn,
    De,
    Us,
    Sg,
    Ru,
    In,
    I2,
}

impl Region {
    pub fn code(self) -> &'static str {
        match self {
            Self::Cn => "cn",
            Self::De => "de",
            Self::Us => "us",
            Self::Sg => "sg",
            Self::Ru => "ru",
            Self::In => "in",
            Self::I2 => "i2",
        }
    }
    pub(crate) fn origin(self) -> String {
        let prefix = if self == Self::Cn {
            String::new()
        } else {
            format!("{}.", self.code())
        };
        format!("https://{prefix}api.io.mi.com")
    }
}

/// Strings with provider identity/credential content never implement public serialization.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PrivateString(pub Zeroizing<String>);

impl std::hash::Hash for PrivateString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(self.as_str(), state);
    }
}

impl PrivateString {
    pub fn new(value: String) -> Self {
        Self(Zeroizing::new(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl fmt::Debug for PrivateString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}
impl<'de> Deserialize<'de> for PrivateString {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        String::deserialize(d).map(Self::new)
    }
}

pub(crate) fn invalid() -> Error {
    Error::new(
        "MIJIA_INVALID_RESPONSE",
        "The Mijia response did not match the reviewed protocol.",
    )
}
pub(crate) fn failed() -> Error {
    Error::new(
        "MIJIA_REQUEST_FAILED",
        "The Mijia request could not be completed.",
    )
}
pub(crate) fn auth_required() -> Error {
    Error::new(
        "AUTH_REQUIRED",
        "The Mijia account requires user-side authorization.",
    )
}
pub(crate) fn bounded_text(value: &str, max: usize) -> bool {
    !value.is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}
pub(crate) fn cookie_value(value: &str) -> bool {
    bounded_text(value, 4096)
        && value
            .bytes()
            .all(|b| b.is_ascii_graphic() && !b";,\"\\".contains(&b))
}
pub(crate) fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    if bytes.len() > MAX_RESPONSE_BYTES {
        return Err(invalid());
    }
    let bytes = bytes.strip_prefix(b"&&&START&&&").unwrap_or(bytes);
    serde_json::from_slice(bytes).map_err(|_| invalid())
}
