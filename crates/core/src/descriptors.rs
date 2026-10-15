use homeverb_domain::{Descriptor, Error, Result};
use serde_json::Value;
use std::{collections::HashSet, sync::OnceLock};

pub(crate) fn invalid() -> Error {
    Error::new(
        "CAPABILITY_UNSUPPORTED",
        "The device metadata does not satisfy the supported descriptor contract.",
    )
}

pub(crate) fn validate(descriptor: &Descriptor) -> Result<()> {
    static SCHEMAS: OnceLock<(jsonschema::Validator, jsonschema::Validator)> = OnceLock::new();
    let (device_schema, capability_schema) = SCHEMAS.get_or_init(|| {
        let compile = |text| {
            let schema: Value = serde_json::from_str(text).expect("checked descriptor schema");
            jsonschema::validator_for(&schema).expect("checked descriptor schema")
        };
        (
            compile(include_str!("../../../schemas/device.schema.json")),
            compile(include_str!("../../../schemas/capability.schema.json")),
        )
    });
    let device = &descriptor.device;
    if !device_schema.is_valid(&serde_json::to_value(device).map_err(|_| invalid())?) {
        return Err(invalid());
    }
    let mut capability_ids = HashSet::new();
    for capability in &descriptor.capabilities {
        if !capability_ids.insert((capability.component_id.as_deref(), capability.id.as_str()))
            || !capability_schema
                .is_valid(&serde_json::to_value(capability).map_err(|_| invalid())?)
        {
            return Err(invalid());
        }
        if capability.component_id.as_ref().is_some_and(|scope| {
            !device.components.iter().any(|component| {
                &component.id == scope && component.capabilities.contains(&capability.id)
            })
        }) {
            return Err(invalid());
        }
        for action in capability.actions.values() {
            // Compile only locally; external schema retrieval is disabled by the dependency.
            jsonschema::validator_for(&action.parameters).map_err(|_| invalid())?;
        }
    }
    if descriptor.capabilities.iter().any(|capability| {
        capability.component_id.is_some()
            && capability_ids.contains(&(None, capability.id.as_str()))
    }) {
        return Err(invalid());
    }
    let mut component_ids = HashSet::new();
    for component in &device.components {
        let mut references = HashSet::new();
        let mut property_names = HashSet::new();
        if !component_ids.insert(&component.id) {
            return Err(invalid());
        }
        for id in &component.capabilities {
            if !references.insert(id) {
                return Err(invalid());
            }
            let capability = descriptor
                .capability(&component.id, id)
                .ok_or_else(invalid)?;
            for name in capability.properties.keys() {
                if !property_names.insert(format!("{id}.{name}")) {
                    return Err(invalid());
                }
            }
        }
    }
    let mut connection_ids = HashSet::new();
    if device
        .connections
        .iter()
        .any(|connection| !connection_ids.insert(&connection.id))
    {
        return Err(invalid());
    }
    Ok(())
}
