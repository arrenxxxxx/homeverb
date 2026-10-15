use crate::{Device, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub position: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DevicePreferences {
    pub device_id: String,
    pub alias: Option<String>,
    pub room_id: Option<String>,
    pub favorite: bool,
    pub position: u32,
}

/// Replace all editable preferences. A null alias restores the adapter's local label.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreferencesInput {
    #[serde(deserialize_with = "required_optional")]
    pub alias: Option<String>,
    #[serde(deserialize_with = "required_optional")]
    pub room_id: Option<String>,
    pub favorite: bool,
}

/// Move a selection atomically without replacing other presentation preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoomAssignment {
    pub device_ids: Vec<String>,
    #[serde(deserialize_with = "required_optional")]
    pub room_id: Option<String>,
}

impl RoomAssignment {
    pub fn validate(&self) -> Result<()> {
        let valid_id = |id: &str| !id.trim().is_empty() && id.len() <= 256;
        if self.device_ids.is_empty()
            || self.device_ids.len() > 256
            || self.device_ids.iter().any(|id| !valid_id(id))
            || self.device_ids.iter().collect::<HashSet<_>>().len() != self.device_ids.len()
            || self.room_id.as_deref().is_some_and(|id| !valid_id(id))
        {
            return Err(Error::invalid());
        }
        Ok(())
    }
}

fn required_optional<'de, D, T>(deserializer: D) -> std::result::Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::deserialize(deserializer)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceCard {
    pub device: Device,
    pub preferences: DevicePreferences,
}

pub fn local_name(value: &str) -> Result<String> {
    let name = value.trim();
    if name.is_empty() || name.chars().count() > 80 || name.chars().any(char::is_control) {
        return Err(Error::new(
            "INVALID_ARGUMENT",
            "A local name must contain 1-80 characters without control characters.",
        ));
    }
    Ok(name.to_owned())
}
