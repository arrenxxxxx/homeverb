//! Trusted startup-only exact light and purifier targets, independent of fan grants.
use homeverb_core::{Grant, Policy};
use homeverb_domain::{Error, Result, Transport};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MijiaSettingGrant {
    pub device_id: String,
    pub component_id: String,
    pub property: String,
    pub value: Value,
}

pub fn mijia_setting_policy(grants: &[MijiaSettingGrant]) -> Result<Policy> {
    // Includes all 3,801 exact color-temperature targets without rounding to presets.
    if grants.len() > 4096 {
        return Err(Error::invalid());
    }
    let mut policy = Policy::default();
    let mut seen = HashSet::new();
    for grant in grants {
        let id = Uuid::parse_str(&grant.device_id).map_err(|_| Error::invalid())?;
        if id.get_version_num() != 4
            || id.get_variant() != uuid::Variant::RFC4122
            || id.to_string() != grant.device_id
        {
            return Err(Error::invalid());
        }
        let option = homeverb_mijia::reference_setting_option(&grant.component_id, &grant.property)
            .ok_or_else(Error::invalid)?;
        let arguments = json!({grant.property.clone():grant.value});
        option
            .definition
            .validate_arguments(&arguments)
            .map_err(|_| Error::invalid())?;
        let normalized = grant
            .value
            .as_f64()
            .map_or_else(|| grant.value.clone(), |v| json!(v));
        if !seen.insert((
            &grant.device_id,
            &grant.component_id,
            &grant.property,
            normalized.to_string(),
        )) {
            return Err(Error::invalid());
        }
        policy.grants.push(Grant {
            device_id: grant.device_id.clone(),
            component_id: grant.component_id.clone(),
            capability: option.capability.clone(),
            action: option.action.clone(),
            transports: vec![Transport::Cloud],
            argument_schema: json!({"const":arguments}),
        });
    }
    Ok(policy)
}
