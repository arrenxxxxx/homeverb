//! Exact dishwasher action authority, independently gated by trusted UI approval.

use homeverb_core::{Grant, Policy};
use homeverb_domain::{Error, Result, Transport};
use homeverb_midea::{DISHWASHER_ADDITIONAL, DISHWASHER_PROGRAMS, DishwasherCycleAction};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeijuDishwasherGrant {
    pub device_id: String,
    pub arguments: Value,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeijuDishwasherCycleGrant {
    pub device_id: String,
    pub action: DishwasherCycleAction,
    #[serde(default = "empty_arguments")]
    pub arguments: Value,
}

fn empty_arguments() -> Value {
    json!({})
}

pub fn meiju_dishwasher_cycle_policy(grants: &[MeijuDishwasherCycleGrant]) -> Result<Policy> {
    if grants.len() > 64 {
        return Err(Error::invalid());
    }
    let mut seen = HashSet::new();
    let mut policy = Policy::default();
    for grant in grants {
        let id = Uuid::parse_str(&grant.device_id).map_err(|_| Error::invalid())?;
        let arguments = grant.arguments.as_object().ok_or_else(Error::invalid)?;
        let additional = arguments
            .get("additional")
            .map(|value| {
                value
                    .as_f64()
                    .filter(|value| {
                        DISHWASHER_ADDITIONAL
                            .iter()
                            .any(|code| f64::from(*code) == *value)
                    })
                    .map(|value| value as u8)
                    .ok_or_else(Error::invalid)
            })
            .transpose()?;
        if id.get_version_num() != 4
            || id.get_variant() != uuid::Variant::RFC4122
            || id.to_string() != grant.device_id
            || arguments.keys().any(|key| key != "additional")
            || !seen.insert((&grant.device_id, grant.action, additional))
        {
            return Err(Error::invalid());
        }
        policy.grants.push(Grant {
            device_id: grant.device_id.clone(),
            component_id: "main".into(),
            capability: "vendor.midea.dishwasher".into(),
            action: grant.action.action_name().into(),
            transports: vec![Transport::Cloud],
            argument_schema: json!({"const":grant.arguments}),
        });
    }
    Ok(policy)
}

pub fn meiju_dishwasher_policy(grants: &[MeijuDishwasherGrant]) -> Result<Policy> {
    if grants.len() > 64 {
        return Err(Error::invalid());
    }
    let mut seen = HashSet::new();
    let mut policy = Policy::default();
    for grant in grants {
        let id = Uuid::parse_str(&grant.device_id).map_err(|_| Error::invalid())?;
        let arguments = grant.arguments.as_object().ok_or_else(Error::invalid)?;
        let program = arguments
            .get("program")
            .and_then(Value::as_str)
            .ok_or_else(Error::invalid)?;
        let additional = arguments
            .get("additional")
            .map(|value| {
                value
                    .as_f64()
                    .filter(|value| {
                        DISHWASHER_ADDITIONAL
                            .iter()
                            .any(|code| f64::from(*code) == *value)
                    })
                    .map(|value| value as u8)
                    .ok_or_else(Error::invalid)
            })
            .transpose()?;
        if id.get_version_num() != 4
            || id.get_variant() != uuid::Variant::RFC4122
            || id.to_string() != grant.device_id
            || arguments
                .keys()
                .any(|key| key != "program" && key != "additional")
            || !DISHWASHER_PROGRAMS.contains(&program)
            || !seen.insert((&grant.device_id, program, additional))
        {
            return Err(Error::invalid());
        }
        policy.grants.push(Grant {
            device_id: grant.device_id.clone(),
            component_id: "main".into(),
            capability: "vendor.midea.dishwasher".into(),
            action: "start_cycle".into(),
            transports: vec![Transport::Cloud],
            argument_schema: json!({"const":grant.arguments}),
        });
    }
    Ok(policy)
}
