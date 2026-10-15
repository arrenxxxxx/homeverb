//! Exact startup authority for the combined AC power-on and mode action.

use homeverb_core::{Grant, Policy};
use homeverb_domain::{Error, Result, Transport};
use homeverb_midea::AcMode;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeijuModeGrant {
    pub device_id: String,
    pub mode: AcMode,
}

pub fn meiju_mode_policy(grants: &[MeijuModeGrant]) -> Result<Policy> {
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
            || !seen.insert((&grant.device_id, grant.mode))
        {
            return Err(Error::invalid());
        }
        policy.grants.push(Grant {
            device_id: grant.device_id.clone(),
            component_id: "main".into(),
            capability: "vendor.midea.ac".into(),
            action: "activate_mode".into(),
            transports: vec![Transport::Cloud],
            argument_schema: json!({"const":{"mode":grant.mode}}),
        });
    }
    Ok(policy)
}
