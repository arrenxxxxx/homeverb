//! Narrow startup-only permissions for experimental Meiju AC power controls.

use homeverb_core::{Grant, Policy};
use homeverb_domain::{Error, Result, Transport};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashSet;
use uuid::Uuid;

/// Trusted host configuration, never an ordinary request or model-supplied approval.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeijuPowerGrant {
    pub device_id: String,
    pub state: bool,
}

/// Validate the complete list before opening a profile or admitting any permission.
pub fn meiju_power_policy(grants: &[MeijuPowerGrant]) -> Result<Policy> {
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
            || !seen.insert((&grant.device_id, grant.state))
        {
            return Err(Error::invalid());
        }
        policy.grants.push(Grant {
            device_id: grant.device_id.clone(),
            component_id: "main".into(),
            capability: "power".into(),
            action: "set".into(),
            transports: vec![Transport::Cloud],
            argument_schema: json!({"const":{"state":grant.state}}),
        });
    }
    Ok(policy)
}
