//! Read-only GX1200Max plugin semantics from the pinned October 22 reference.
//! These fields never select a command profile, firmware branch or TSL endpoint.

use homeverb_domain::{Capability, Result};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(crate) const ID: &str = "vendor.midea.dishwasher.details";
pub(crate) const PROGRAMS: &[&str] = &[
    "keep",
    "crystal_flame_disinfect",
    "fruit",
    "rapid",
    "strong",
    "normal",
    "wash_dry",
    "self_clean",
    "eco",
    "germ",
    "quiet_night",
    "glass",
    "hotpot",
    "seafood",
];
pub(crate) const STATES: &[&str] = &[
    "offline",
    "error",
    "dry",
    "keep",
    "work",
    "work_pause",
    "order",
    "power_on",
    "power_off",
    "order_pause",
    "dry_pause",
    "keep_pause",
    "ota",
    "drain",
    "airing",
];

pub(crate) fn capability() -> Capability {
    let mut properties = BTreeMap::new();
    for (name, kind, unit) in [
        ("program", "string", None),
        ("cycle_state", "string", None),
        ("door_state", "string", None),
        ("temperature", "number", Some("celsius")),
        ("remaining_minutes", "integer", Some("minutes")),
        ("estimated_minutes", "integer", Some("minutes")),
        ("storage_enabled", "boolean", None),
        ("cycle_storage_enabled", "boolean", None),
        ("smart_dose", "boolean", None),
        ("lower_zone_boost", "boolean", None),
        ("child_lock", "boolean", None),
    ] {
        let mut property = json!({"type":kind,"readable":true,"writable":false});
        if let Some(unit) = unit {
            property["unit"] = json!(unit);
        }
        properties.insert(name.into(), property);
    }
    properties.get_mut("program").unwrap()["enum"] =
        json!(PROGRAMS.iter().copied().chain(["none"]).collect::<Vec<_>>());
    properties.get_mut("cycle_state").unwrap()["enum"] = json!(STATES);
    properties.get_mut("door_state").unwrap()["enum"] = json!(["open", "closed"]);
    for key in ["remaining_minutes", "estimated_minutes"] {
        properties.get_mut(key).unwrap()["minimum"] = json!(0);
    }
    Capability {
        schema_version: "1.0".into(),
        id: ID.into(),
        component_id: Some("main".into()),
        revision: 1,
        properties,
        actions: BTreeMap::new(),
        events: BTreeMap::new(),
    }
}

fn present<'a>(data: &'a Value, name: &str) -> Option<&'a Value> {
    data.get(name).filter(|value| !value.is_null())
}

fn number(value: &Value) -> Result<f64> {
    match value {
        Value::Number(n) => n.as_f64(),
        Value::String(s) if s.len() <= 64 => s.parse().ok(),
        _ => None,
    }
    .filter(|n| n.is_finite())
    .ok_or_else(crate::invalid)
}

fn flag(value: &Value, unsupported_sentinel: bool) -> Result<Option<bool>> {
    if let Some(value) = value.as_bool() {
        return Ok(Some(value));
    }
    match number(value)? {
        0.0 => Ok(Some(false)),
        1.0 => Ok(Some(true)),
        3.0 if unsupported_sentinel => Ok(None),
        _ => Err(crate::invalid()),
    }
}

fn token<'a>(data: &'a Value, name: &str) -> Result<Option<&'a str>> {
    present(data, name)
        .map(|value| {
            value
                .as_str()
                .filter(|s| {
                    !s.is_empty()
                        && s.len() <= 64
                        && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
                })
                .ok_or_else(crate::invalid)
        })
        .transpose()
}

pub(crate) fn decode(data: &Value, readings: &mut BTreeMap<String, Value>) -> Result<()> {
    let mut add = |name: &str, value: Value| {
        readings.insert(format!("{ID}.{name}"), value);
    };
    if let Some(mode) = token(data, "mode")?
        && (PROGRAMS.contains(&mode) || mode == "none")
    {
        add("program", json!(mode));
    }
    let work = token(data, "work_status")?;
    let base = token(data, "base_status")?;
    if matches!((work, base), (Some(a), Some(b)) if a != b) {
        return Err(crate::invalid());
    }
    let state = base.or(work);
    if let Some(state) = state.filter(|s| STATES.contains(s)) {
        add("cycle_state", json!(state));
    }
    // base_door_on is the plugin's open-door flag. It is deliberately not
    // inferred from the older doorswitch code, whose polarity is model-specific.
    if let Some(value) = present(data, "base_door_on") {
        add(
            "door_state",
            json!(if flag(value, false)?.unwrap() {
                "open"
            } else {
                "closed"
            }),
        );
    }
    if let Some(value) = present(data, "base_cur_temp") {
        add("temperature", json!(number(value)?));
    }
    if let Some(value) = present(data, "base_mode_time") {
        let minutes = number(value)?;
        if minutes < 0.0 || minutes.fract() != 0.0 || minutes > u32::MAX as f64 {
            return Err(crate::invalid());
        }
        // Standby exposes estimated duration, not remaining running time.
        let name = match state {
            Some("power_on") => Some("estimated_minutes"),
            Some(
                "work" | "work_pause" | "dry" | "dry_pause" | "keep" | "keep_pause" | "order"
                | "order_pause" | "airing",
            ) => Some("remaining_minutes"),
            _ => None,
        };
        if let Some(name) = name {
            add(name, json!(minutes as u32));
        }
    }
    for (wire, name, sentinel) in [
        ("more_keep_device", "storage_enabled", false),
        ("more_cycle_keep_device", "cycle_storage_enabled", false),
        ("more_auto_throw", "smart_dose", true),
        ("more_region_reinforce", "lower_zone_boost", true),
        ("lock_on", "child_lock", false),
    ] {
        if let Some(value) = present(data, wire)
            && let Some(value) = flag(value, sentinel)?
        {
            add(name, json!(value));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(data: Value) -> Result<BTreeMap<String, Value>> {
        let mut readings = BTreeMap::new();
        decode(&data, &mut readings)?;
        Ok(readings)
    }

    #[test]
    fn reported_programs_and_states_are_not_action_definitions() {
        for program in PROGRAMS.iter().copied().chain(["none"]) {
            assert_eq!(
                read(json!({"mode":program})).unwrap()[&format!("{ID}.program")],
                program
            );
        }
        for state in STATES {
            assert_eq!(
                read(json!({"base_status":state})).unwrap()[&format!("{ID}.cycle_state")],
                *state
            );
        }
        assert!(
            read(json!({"mode":"future_program","work_status":"future_state"}))
                .unwrap()
                .is_empty()
        );
        assert!(capability().actions.is_empty());
    }

    #[test]
    fn absence_and_unsupported_options_never_become_off_or_safe_door_readings() {
        assert!(
            read(
                json!({"base_door_on":null,"doorswitch":1,"more_auto_throw":3,
            "more_region_reinforce":"3","mode":null})
            )
            .unwrap()
            .is_empty()
        );
        for input in [json!(0), json!("0"), json!(false)] {
            let values = read(json!({"base_door_on":input,"more_auto_throw":input})).unwrap();
            assert_eq!(values[&format!("{ID}.door_state")], "closed");
            assert_eq!(values[&format!("{ID}.smart_dose")], false);
        }
        for input in [json!(1), json!("1"), json!(true)] {
            assert_eq!(
                read(json!({"base_door_on":input})).unwrap()[&format!("{ID}.door_state")],
                "open"
            );
        }
    }

    #[test]
    fn durations_follow_reported_phase_without_guessing_a_running_cycle() {
        for state in STATES {
            let values = read(json!({"base_status":state,"base_mode_time":"42"})).unwrap();
            let remaining = values.get(&format!("{ID}.remaining_minutes"));
            let estimate = values.get(&format!("{ID}.estimated_minutes"));
            if *state == "power_on" {
                assert_eq!(estimate, Some(&json!(42)));
                assert!(remaining.is_none());
            } else if [
                "work",
                "work_pause",
                "dry",
                "dry_pause",
                "keep",
                "keep_pause",
                "order",
                "order_pause",
                "airing",
            ]
            .contains(state)
            {
                assert_eq!(remaining, Some(&json!(42)));
                assert!(estimate.is_none());
            } else {
                assert!(remaining.is_none() && estimate.is_none());
            }
        }
        assert!(read(json!({"base_mode_time":42})).unwrap().is_empty());
    }

    #[test]
    fn malformed_fields_and_conflicting_state_refuse_the_snapshot() {
        for data in [
            json!({"base_status":"work","work_status":"power_off"}),
            json!({"base_door_on":2}),
            json!({"more_auto_throw":2}),
            json!({"lock_on":3}),
            json!({"base_mode_time":-1}),
            json!({"base_mode_time":0.5}),
            json!({"base_cur_temp":"NaN"}),
            json!({"base_cur_temp":true}),
            json!({"mode":[]}),
            json!({"base_status":""}),
        ] {
            assert_eq!(read(data).unwrap_err().code, "MIDEA_RESPONSE_INVALID");
        }
    }
}
