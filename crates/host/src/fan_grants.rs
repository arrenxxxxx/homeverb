//! Independent startup permissions for reviewed vendor AC fan presets.

use homeverb_core::{Grant, Policy};
use homeverb_domain::{Error, Result, Transport};
use homeverb_midea::AC_FAN_SPEEDS;
use serde::{Deserialize, Deserializer};
use serde_json::json;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeijuFanGrant {
    pub device_id: String,
    #[serde(deserialize_with = "deserialize_speed")]
    pub wind_speed: u8,
}

fn deserialize_speed<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<u8, D::Error> {
    // JSON Schema accepts integral numbers such as 20.0; normalize only reviewed codes.
    let value = serde_json::Value::deserialize(deserializer)?;
    AC_FAN_SPEEDS
        .into_iter()
        .find(|speed| value.as_f64() == Some(f64::from(*speed)))
        .ok_or_else(|| serde::de::Error::custom("Unsupported AC fan preset."))
}

pub fn meiju_fan_policy(grants: &[MeijuFanGrant]) -> Result<Policy> {
    if grants.len() > 64 {
        return Err(Error::invalid());
    }
    let mut seen = HashSet::new();
    let mut policy = Policy::default();
    for grant in grants {
        let id = Uuid::parse_str(&grant.device_id).map_err(|_| Error::invalid())?;
        if id.get_version_num() != 4
            || id.get_variant() != uuid::Variant::RFC4122
            || id.to_string() != grant.device_id
            || !AC_FAN_SPEEDS.contains(&grant.wind_speed)
            || !seen.insert((&grant.device_id, grant.wind_speed))
        {
            return Err(Error::invalid());
        }
        policy.grants.push(Grant {
            device_id: grant.device_id.clone(),
            component_id: "main".into(),
            capability: "vendor.midea.ac".into(),
            action: "set_fan_speed".into(),
            transports: vec![Transport::Cloud],
            argument_schema: json!({"const":{"wind_speed":grant.wind_speed}}),
        });
    }
    Ok(policy)
}
