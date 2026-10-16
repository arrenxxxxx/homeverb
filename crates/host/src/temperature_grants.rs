//! Exact Celsius targets; authority never supplies a missing device constraint.

use homeverb_core::{Grant, Policy};
use homeverb_domain::{Error, Result, Transport};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeijuTemperatureGrant {
    pub device_id: String,
    pub target_temperature: f64,
}

pub fn meiju_temperature_policy(grants: &[MeijuTemperatureGrant]) -> Result<Policy> {
    if grants.len() > 256 {
        return Err(Error::invalid());
    }
    let mut seen = HashSet::new();
    let mut policy = Policy::default();
    for grant in grants {
        let id = Uuid::parse_str(&grant.device_id).map_err(|_| Error::invalid())?;
        let target = grant.target_temperature;
        // This union of reviewed setpoints is not a universal device range.
        if id.get_version_num() != 4
            || id.get_variant() != uuid::Variant::RFC4122
            || id.to_string() != grant.device_id
            || !target.is_finite()
            || !(16.0..=30.0).contains(&target)
            || target % 0.5 != 0.0
            || !seen.insert((&grant.device_id, (target * 2.0) as u8))
        {
            return Err(Error::invalid());
        }
        policy.grants.push(Grant {
            device_id: grant.device_id.clone(),
            component_id: "main".into(),
            capability: "climate".into(),
            action: "set".into(),
            transports: vec![Transport::Cloud],
            argument_schema: json!({"const":{"target_temperature":target}}),
        });
    }
    Ok(policy)
}
