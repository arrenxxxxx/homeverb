//! Exact startup-only permissions for reviewed optional AC scalar settings.

use homeverb_core::{Grant, Policy};
use homeverb_domain::{Error, Result, Transport};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeijuAcFeatureGrant {
    pub device_id: String,
    pub feature: String,
    pub value: Value,
}

pub fn meiju_ac_feature_policy(grants: &[MeijuAcFeatureGrant]) -> Result<Policy> {
    if grants.len() > 1024 {
        return Err(Error::invalid());
    }
    let mut seen = HashSet::new();
    let mut policy = Policy::default();
    for grant in grants {
        homeverb_midea::validate_ac_feature_value(&grant.feature, &grant.value)
            .map_err(|_| Error::invalid())?;
        // JSON Schema accepts integral floats. Canonicalize before duplicate and policy checks.
        let value = grant
            .value
            .as_f64()
            .map_or_else(|| grant.value.clone(), |v| json!(v as i64));
        let id = Uuid::parse_str(&grant.device_id).map_err(|_| Error::invalid())?;
        if id.get_version_num() != 4
            || id.get_variant() != uuid::Variant::RFC4122
            || id.to_string() != grant.device_id
            || !seen.insert((&grant.device_id, &grant.feature, value.to_string()))
        {
            return Err(Error::invalid());
        }
        policy.grants.push(Grant {
            device_id: grant.device_id.clone(),
            component_id: "main".into(),
            capability: "vendor.midea.ac".into(),
            action: format!("set_{}", grant.feature),
            transports: vec![Transport::Cloud],
            argument_schema: json!({"const":{grant.feature.clone():value}}),
        });
    }
    Ok(policy)
}
