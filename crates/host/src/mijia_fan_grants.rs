//! Trusted startup-only exact Mijia fan permissions; protected specs remain authoritative.

use homeverb_core::{Grant, Policy};
use homeverb_domain::{Error, Result, Transport};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MijiaFanGrant {
    pub device_id: String,
    pub component_id: String,
    pub property: String,
    pub value: Value,
}

pub fn mijia_fan_policy(grants: &[MijiaFanGrant]) -> Result<Policy> {
    if grants.len() > 1024 {
        return Err(Error::invalid());
    }
    let mut policy = Policy::default();
    let mut seen = HashSet::new();
    for grant in grants {
        let id = Uuid::parse_str(&grant.device_id).map_err(|_| Error::invalid())?;
        let service = grant
            .component_id
            .strip_prefix("fan-")
            .and_then(|s| s.parse::<u32>().ok())
            .filter(|v| *v > 0)
            .ok_or_else(Error::invalid)?;
        if id.get_version_num() != 4
            || id.get_variant() != uuid::Variant::RFC4122
            || id.to_string() != grant.device_id
            || grant.component_id != format!("fan-{service}")
        {
            return Err(Error::invalid());
        }
        let capability = match grant.property.as_str() {
            "state" if grant.value.is_boolean() => "power",
            "horizontal_swing" if grant.value.is_boolean() => "vendor.xiaomi.mijia.fan",
            "level" | "mode" | "horizontal_angle" if grant.value.is_number() => {
                "vendor.xiaomi.mijia.fan"
            }
            _ => return Err(Error::invalid()),
        };
        // Numeric spellings such as 3 and 3.0 must not evade duplicate bounds.
        let duplicate_key = grant
            .value
            .as_f64()
            .map_or_else(|| grant.value.clone(), |v| json!(v));
        if !seen.insert((
            &grant.device_id,
            &grant.component_id,
            &grant.property,
            duplicate_key.to_string(),
        )) {
            return Err(Error::invalid());
        }
        policy.grants.push(Grant {
            device_id: grant.device_id.clone(),
            component_id: grant.component_id.clone(),
            capability: capability.into(),
            action: if capability == "power" {
                "set".into()
            } else {
                format!("set_{}", grant.property)
            },
            transports: vec![Transport::Cloud],
            argument_schema: json!({"const":{grant.property.clone():grant.value.clone()}}),
        });
    }
    Ok(policy)
}
