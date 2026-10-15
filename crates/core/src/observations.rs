use chrono::{DateTime, Duration, Utc};
use homeverb_domain::{Descriptor, Error, Result, SCHEMA_VERSION, Snapshot, Transport};
use serde_json::Value;
use std::{collections::BTreeMap, sync::OnceLock};

fn invalid() -> Error {
    Error::new(
        "INVALID_OBSERVATION",
        "The observation does not satisfy the supported snapshot contract.",
    )
}

pub(crate) fn matches_target(observed: &Value, target: &Value) -> bool {
    let (Value::Number(left), Value::Number(right)) = (observed, target) else {
        return observed == target;
    };
    if left.is_f64() == right.is_f64() {
        return left == right;
    }
    let (integer, float) = if left.is_f64() {
        (right, left.as_f64().expect("floating number"))
    } else {
        (left, right.as_f64().expect("floating number"))
    };
    if float.fract() != 0.0 {
        return false;
    }
    // Do not convert integers to f64: that would merge distinct values above 2^53.
    // Check exclusive upper bounds before Rust's saturating float-to-integer casts.
    if let Some(integer) = integer.as_i64() {
        float >= i64::MIN as f64 && float < -(i64::MIN as f64) && float as i64 == integer
    } else if let Some(integer) = integer.as_u64() {
        float >= 0.0 && float < u64::MAX as f64 && float as u64 == integer
    } else {
        false
    }
}

fn validate(
    snapshot: &Snapshot,
    descriptor: &Descriptor,
    component: &str,
) -> Result<BTreeMap<String, Duration>> {
    if snapshot.schema_version != SCHEMA_VERSION
        || snapshot.device_id != descriptor.device.id
        || snapshot.component_id != component
        || snapshot.device_id.is_empty()
        || snapshot.component_id.is_empty()
        || snapshot.warnings.iter().any(String::is_empty)
        || snapshot.observations.values().any(|observation| {
            observation
                .observed_at
                .is_some_and(|observed| observed > observation.received_at)
        })
    {
        return Err(invalid());
    }
    validate_values(snapshot, descriptor, component)
}

fn validate_values(
    snapshot: &Snapshot,
    descriptor: &Descriptor,
    component: &str,
) -> Result<BTreeMap<String, Duration>> {
    let mut components = descriptor
        .device
        .components
        .iter()
        .filter(|c| c.id == component);
    let component = components.next().ok_or_else(invalid)?;
    if components.next().is_some() {
        return Err(invalid());
    }
    let mut properties = BTreeMap::new();
    for id in &component.capabilities {
        let capability = descriptor
            .capability(&component.id, id)
            .ok_or_else(invalid)?;
        if capability.schema_version != SCHEMA_VERSION || capability.revision == 0 {
            return Err(invalid());
        }
        for (name, property) in &capability.properties {
            // Match the full name so vendor namespaces containing dots remain intact.
            if properties
                .insert(format!("{id}.{name}"), property)
                .is_some()
            {
                return Err(invalid());
            }
        }
    }
    static PROPERTY_DESCRIPTOR: OnceLock<jsonschema::Validator> = OnceLock::new();
    let descriptor_validator = PROPERTY_DESCRIPTOR.get_or_init(|| {
        let schema: Value =
            serde_json::from_str(include_str!("../../../schemas/capability.schema.json"))
                .expect("checked capability schema");
        jsonschema::validator_for(&schema["properties"]["properties"]["additionalProperties"])
            .expect("checked property descriptor schema")
    });
    let mut max_ages = BTreeMap::new();
    for (name, observation) in &snapshot.observations {
        let property = properties.get(name).ok_or_else(invalid)?;
        if !descriptor_validator.is_valid(property) || property["readable"] != true {
            return Err(invalid());
        }
        // The property descriptor uses JSON Schema value constraints. Metadata keys
        // such as readable, writable, and unit do not affect value validation.
        let validator = jsonschema::validator_for(property).map_err(|_| invalid())?;
        if !validator.is_valid(&observation.value) {
            return Err(invalid());
        }
        // The descriptor bounds this integral number to JSON's exact integer range.
        // Reading it as f64 also accepts integer-valued JSON numbers such as 1000.0.
        let max_age_ms = match property.get("max_age_ms") {
            Some(value) => value.as_f64().ok_or_else(invalid)? as i64,
            None => 30_000,
        };
        max_ages.insert(name.clone(), Duration::milliseconds(max_age_ms));
    }
    Ok(max_ages)
}

fn warning(snapshot: &mut Snapshot, message: &str) {
    if !snapshot.warnings.iter().any(|existing| existing == message) {
        snapshot.warnings.push(message.into());
    }
}

fn mark_stale(snapshot: &mut Snapshot, max_ages: &BTreeMap<String, Duration>, now: DateTime<Utc>) {
    let mut previously_stale = false;
    let mut expired = false;
    let mut clock_changed = false;
    for (name, observation) in &mut snapshot.observations {
        let observed = observation.observed_at.unwrap_or(observation.received_at);
        let old = now.signed_duration_since(observed) >= max_ages[name];
        let future = observation.received_at > now;
        previously_stale |= observation.stale;
        expired |= old;
        clock_changed |= future;
        observation.stale |= old || future;
    }
    if previously_stale {
        warning(
            snapshot,
            "One or more observations were already marked stale.",
        );
    }
    if expired {
        warning(
            snapshot,
            "One or more observations reached their property freshness limit.",
        );
    }
    if clock_changed {
        warning(
            snapshot,
            "One or more receipt timestamps are ahead of the local clock.",
        );
    }
}

pub(crate) fn prepare_live(
    snapshot: &mut Snapshot,
    descriptor: &Descriptor,
    component: &str,
    source: Transport,
    now: DateTime<Utc>,
) -> Result<()> {
    let max_ages = validate(snapshot, descriptor, component)?;
    if snapshot.from_cache
        || snapshot
            .observations
            .values()
            .any(|observation| observation.source != source || observation.received_at > now)
    {
        return Err(invalid());
    }
    mark_stale(snapshot, &max_ages, now);
    Ok(())
}

pub(crate) fn prepare_cached(
    snapshot: &mut Snapshot,
    descriptor: &Descriptor,
    component: &str,
    now: DateTime<Utc>,
) -> Result<()> {
    let max_ages = validate(snapshot, descriptor, component)?;
    snapshot.from_cache = true;
    mark_stale(snapshot, &max_ages, now);
    warning(snapshot, "Cached observation; no device request was made.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::matches_target;
    use serde_json::json;

    #[test]
    fn numeric_target_equality_is_symmetric_and_does_not_lose_integer_precision() {
        for (left, right, expected) in [
            (json!(25), json!(25.0), true),
            (json!(-25), json!(-25.0), true),
            (json!(0), json!(-0.0), true),
            (json!(25.25), json!(25.25), true),
            (json!(25.25), json!(25.250000000000004), false),
            (json!(25), json!(25.5), false),
            (json!(25), json!("25"), false),
            (json!(1), json!(true), false),
            (json!(false), json!(0.0), false),
            (json!(null), json!(0), false),
            (
                json!(9_007_199_254_740_992_u64),
                json!(9_007_199_254_740_992.0),
                true,
            ),
            (
                json!(9_007_199_254_740_993_u64),
                json!(9_007_199_254_740_992.0),
                false,
            ),
            (json!(i64::MIN), json!(i64::MIN as f64), true),
            (json!(i64::MIN + 1), json!(i64::MIN as f64), false),
            (json!(i64::MAX), json!(i64::MAX as f64), false),
            (json!((i64::MAX as u64) + 1), json!(i64::MAX as f64), true),
            (json!(u64::MAX), json!(u64::MAX as f64), false),
            (
                json!(u64::MAX - 2047),
                json!((u64::MAX - 2047) as f64),
                true,
            ),
            (json!(u64::MAX), json!(u64::MAX), true),
            (json!(u64::MAX), json!(-1), false),
        ] {
            assert_eq!(matches_target(&left, &right), expected, "{left} vs {right}");
            assert_eq!(matches_target(&right, &left), expected, "{right} vs {left}");
        }
    }
}
