//! Trusted startup-only exact empty-argument actions, independent of setters.
use homeverb_core::{Grant, Policy};
use homeverb_domain::{Error, Result, Transport};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MijiaActionGrant {
    pub device_id: String,
    pub component_id: String,
    pub capability: String,
    pub action: String,
}

pub fn mijia_action_policy(grants: &[MijiaActionGrant]) -> Result<Policy> {
    if grants.len() > 64 {
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
        let option = homeverb_mijia::no_argument_action_option(
            &grant.component_id,
            &grant.capability,
            &grant.action,
        )
        .ok_or_else(Error::invalid)?;
        if !seen.insert((
            &grant.device_id,
            &grant.component_id,
            &grant.capability,
            &grant.action,
        )) {
            return Err(Error::invalid());
        }
        policy.grants.push(Grant {
            device_id: grant.device_id.clone(),
            component_id: option.component_id.clone(),
            capability: option.capability.clone(),
            action: option.action.clone(),
            transports: vec![Transport::Cloud],
            argument_schema: json!({"const":{}}),
        });
    }
    Ok(policy)
}
