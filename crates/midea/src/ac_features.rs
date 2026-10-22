//! Reviewed scalar settings from the pinned public AC mapping. No source code is executed.

use crate::{StateQuery, control::CloudControl};
use homeverb_domain::{Action, Idempotency, Result, Risk, Transport, VerificationMethod};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::OnceLock};

#[derive(Deserialize)]
pub(crate) struct FeatureOption {
    value: Value,
    control: BTreeMap<String, Value>,
}

#[derive(Deserialize)]
pub(crate) struct AcFeature {
    pub name: String,
    wire: String,
    read_field: String,
    parameter: Value,
    options: Vec<FeatureOption>,
}

#[derive(Deserialize)]
pub(crate) struct AcFeatureProfile {
    selector: String,
    codes: Vec<String>,
    #[serde(default)]
    models: Vec<String>,
    queries: Vec<String>,
    preserve: Vec<String>,
    pub features: Vec<AcFeature>,
}

fn catalog() -> &'static [AcFeatureProfile] {
    static PROFILES: OnceLock<Vec<AcFeatureProfile>> = OnceLock::new();
    PROFILES.get_or_init(|| {
        let mut profiles: Vec<AcFeatureProfile> =
            serde_json::from_str(include_str!("ac_features.json")).expect("reviewed AC catalog");
        profiles.extend(
            serde_json::from_str::<Vec<AcFeatureProfile>>(include_str!(
                "ac_reference_features.json"
            ))
            .expect("reviewed exact AC reference catalog"),
        );
        profiles
    })
}

pub(crate) fn select(
    subtype: Option<&str>,
    code: Option<&str>,
    category: Option<&str>,
    model: Option<&str>,
) -> Option<&'static AcFeatureProfile> {
    if !model.is_some_and(crate::adapter::reviewed_wall_ac) {
        return None;
    }
    // Exact official product evidence takes precedence over the broad community
    // default. Contradictory model/subtype evidence must not regain that default.
    if let Some(profile) = catalog()
        .iter()
        .find(|p| !p.models.is_empty() && code.is_some_and(|c| p.codes.iter().any(|v| v == c)))
    {
        return (subtype != Some("524")
            && model.is_some_and(|m| profile.models.iter().any(|v| v == m))
            && category.is_some_and(|c| c.replace('-', "_") == "wall_air_conditioner"))
        .then_some(profile);
    }
    if subtype == Some("524") {
        return catalog().iter().find(|p| p.selector == "portable");
    }
    let code = code.filter(|c| !matches!(*c, "00000000" | "00000Q1F"))?;
    if subtype.is_none() && category.is_none_or(|c| c.replace('-', "_") != "wall_air_conditioner") {
        return None;
    }
    // An explicit override never inherits optional functions from the category default.
    if matches!(
        code,
        "22012895"
            | "21293193"
            | "17497071"
            | "10693145"
            | "106J6363"
            | "26093137"
            | "26093139"
            | "26096947"
            | "22012227"
            | "22013167"
            | "22013285"
            | "22251749"
    ) {
        return None;
    }
    catalog()
        .iter()
        .find(|p| p.codes.iter().any(|c| c == code))
        .or_else(|| {
            category
                .filter(|c| c.replace('-', "_") == "wall_air_conditioner")
                .and_then(|_| catalog().iter().find(|p| p.selector == "wall"))
        })
}

impl AcFeature {
    pub fn action_name(&self) -> String {
        format!("set_{}", self.name)
    }
    pub fn action(&self) -> Action {
        Action {
            parameters: json!({"type":"object","properties":{self.name.clone():self.parameter},"required":[self.name],"additionalProperties":false}),
            transports: vec![Transport::Cloud],
            idempotency: Idempotency::SetValue,
            risk: Risk::Low,
            verification: VerificationMethod::Unavailable,
            atomic: false,
            independent_fields: false,
        }
    }
    pub fn property(&self, writable: bool) -> Value {
        // Observation values are not narrowed to the writable option list.
        json!({"type":self.parameter["type"],"readable":true,"writable":writable,"control_requires_observation":true})
    }
    pub fn observed(&self, raw: &BTreeMap<String, Value>) -> Result<Option<Value>> {
        let Some(value) = raw.get(&self.read_field).or_else(|| raw.get(&self.wire)) else {
            return Ok(None);
        };
        // Display firmware can report either the numeric pair or on/off strings.
        if self.name == "display"
            && let Some(value) = value.as_str()
        {
            match value {
                "off" => return Ok(Some(json!(false))),
                "on" => return Ok(Some(json!(true))),
                _ => (),
            };
        }
        if self.options.is_empty() {
            return numeric_value(value).map(Some).ok_or_else(crate::invalid);
        }
        let found = self.options.iter().find(|o| {
            o.control
                .get(&self.wire)
                .is_some_and(|expected| wire_equal(expected, value))
        });
        if let Some(found) = found {
            return Ok(Some(found.value.clone()));
        }
        // Unknown numeric/enum codes remain observable; they never enter a command.
        if self.parameter["type"] == "integer" {
            return numeric_value(value).map(Some).ok_or_else(crate::invalid);
        }
        if self.parameter["type"] == "string" && value.is_string() {
            return Ok(Some(value.clone()));
        }
        Err(crate::invalid())
    }
    pub fn query(&self, profile: &AcFeatureProfile) -> StateQuery {
        profile
            .queries
            .iter()
            .find(|q| *q == &self.wire || *q == &self.read_field)
            .map_or(StateQuery::Default, |q| {
                query_name(q).map_or(StateQuery::Default, StateQuery::AcOptional)
            })
    }
}

fn numeric_value(value: &Value) -> Option<Value> {
    let number = value
        .as_i64()
        .or_else(|| {
            value
                .as_f64()
                .filter(|v| v.is_finite() && v.fract() == 0.0 && v.abs() <= 9_007_199_254_740_991.0)
                .map(|v| v as i64)
        })
        .or_else(|| {
            value
                .as_str()
                .filter(|v| v.len() <= 16)?
                .parse::<i64>()
                .ok()
        })?;
    Some(json!(number))
}

fn wire_equal(expected: &Value, actual: &Value) -> bool {
    expected == actual || (expected.is_number() && numeric_value(actual).as_ref() == Some(expected))
}

pub(crate) fn query_allowed(query: &str) -> bool {
    matches!(
        query,
        "run_status"
            | "no_wind_sense"
            | "prevent_straight_wind"
            | "prevent_super_cool"
            | "wind_swing_lr_angle"
            | "wind_swing_ud_angle"
            | "group_data_four"
            | "indoor_humidity"
            | "indoor_temperature"
            | "outdoor_temperature"
            | "fresh_air"
            | "module_30"
            | "module_31"
            | "module_32"
    )
}

// This closed catalog cannot be used to submit arbitrary protocol query strings.
fn query_name(query: &str) -> Option<&'static str> {
    Some(match query {
        "run_status" => "run_status",
        "no_wind_sense" => "no_wind_sense",
        "prevent_straight_wind" => "prevent_straight_wind",
        "prevent_super_cool" => "prevent_super_cool",
        "wind_swing_lr_angle" => "wind_swing_lr_angle",
        "wind_swing_ud_angle" => "wind_swing_ud_angle",
        "group_data_four" => "group_data_four",
        "indoor_humidity" => "indoor_humidity",
        "indoor_temperature" => "indoor_temperature",
        "outdoor_temperature" => "outdoor_temperature",
        "fresh_air" => "fresh_air",
        "module_30" => "module_30",
        "module_31" => "module_31",
        "module_32" => "module_32",
        _ => return None,
    })
}

impl AcFeatureProfile {
    pub fn queries(&self) -> impl Iterator<Item = StateQuery> + '_ {
        self.queries
            .iter()
            .filter_map(|q| query_name(q).map(StateQuery::AcOptional))
    }
    pub fn queries_for(&self, feature: &AcFeature) -> impl Iterator<Item = StateQuery> + '_ {
        let direct = feature.query(self);
        self.queries().filter(move |query| {
            *query == direct
                || (direct == StateQuery::Default
                    && matches!(
                        query,
                        StateQuery::AcOptional(
                            "run_status" | "fresh_air" | "module_30" | "module_31" | "module_32"
                        )
                    ))
        })
    }
    pub fn needs_query(&self, query: StateQuery, raw: &BTreeMap<String, Value>) -> bool {
        self.features.iter().any(|feature| {
            if raw.contains_key(&feature.read_field) || raw.contains_key(&feature.wire) {
                return false;
            }
            let direct = feature.query(self);
            direct == query
                || (direct == StateQuery::Default
                    && matches!(
                        query,
                        StateQuery::AcOptional(
                            "run_status" | "fresh_air" | "module_30" | "module_31" | "module_32"
                        )
                    ))
        })
    }
    pub fn feature(&self, name: &str) -> Option<&AcFeature> {
        self.features.iter().find(|f| f.name == name)
    }
    pub fn fields(&self, data: &Value) -> BTreeMap<String, Value> {
        let mut fields = BTreeMap::new();
        for feature in &self.features {
            for key in [&feature.wire, &feature.read_field]
                .into_iter()
                .chain(feature.options.iter().flat_map(|o| o.control.keys()))
            {
                if let Some(value) = data.get(key).filter(|v| !v.is_null()) {
                    fields.insert(key.clone(), value.clone());
                }
            }
        }
        for key in &self.preserve {
            if let Some(value) = data.get(key).filter(|v| !v.is_null()) {
                fields.insert(key.clone(), value.clone());
            }
        }
        fields
    }
    pub fn control(
        &self,
        feature: &AcFeature,
        arguments: &Value,
        raw: &BTreeMap<String, Value>,
    ) -> Result<CloudControl> {
        feature.action().validate_arguments(arguments)?;
        feature
            .observed(raw)?
            .ok_or_else(crate::control::unsupported)?;
        let value = &arguments[&feature.name];
        let mut control = if feature.options.is_empty() {
            BTreeMap::from([(
                feature.wire.clone(),
                numeric_value(value).ok_or_else(crate::control::unsupported)?,
            )])
        } else {
            feature
                .options
                .iter()
                .find(|o| wire_equal(&o.value, value))
                .ok_or_else(crate::control::unsupported)?
                .control
                .clone()
        };
        if feature.name == "display"
            && raw
                .get(&feature.wire)
                .is_some_and(|v| matches!(v.as_str(), Some("on" | "off")))
        {
            control.insert(
                feature.wire.clone(),
                json!(if value.as_bool() == Some(true) {
                    "on"
                } else {
                    "off"
                }),
            );
        }
        for key in &self.preserve {
            if control.contains_key(key) {
                continue;
            }
            if key != "buzzer" {
                return Err(crate::control::unsupported());
            }
            // Upstream carries a client-side buzzer preference, including a source
            // default. It is not a required device state: basic native AC commands
            // already omit it. Preserve a report, but never invent that preference.
            if let Some(value) = raw.get(key) {
                if !matches!(value.as_str(), Some("on" | "off")) {
                    return Err(crate::control::unsupported());
                }
                control.insert(key.clone(), value.clone());
            }
        }
        // The source swing presets include all axes in one request. Preserve the others
        // using prerequisite state rather than assuming an omitted axis is unchanged.
        if feature.name.ends_with("_swing") {
            for axis in self.features.iter().filter(|f| f.name.ends_with("_swing")) {
                if control.contains_key(&axis.wire) {
                    continue;
                }
                let value = raw
                    .get(&axis.wire)
                    .ok_or_else(crate::control::unsupported)?;
                if !matches!(value.as_str(), Some("on" | "off")) {
                    return Err(crate::control::unsupported());
                }
                control.insert(axis.wire.clone(), value.clone());
            }
        }
        Ok(CloudControl::AcFeature(control))
    }
}

/// Validate a startup union only. Execution separately selects the protected profile.
pub fn validate_ac_feature_value(name: &str, value: &Value) -> Result<()> {
    for profile in catalog() {
        if let Some(feature) = profile.feature(name)
            && feature
                .action()
                .validate_arguments(&json!({name:value}))
                .is_ok()
        {
            return Ok(());
        }
    }
    Err(crate::control::unsupported())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_reference_profiles_do_not_inherit_other_models_or_generic_features() {
        for (code, model, horizontal, breezeless, strong) in [
            ("22012559", "KFR-26G/N8VJC3", false, false, false),
            ("22040089", "KFR-35G/N8HE1\u{2161}Pro", true, false, true),
            ("22013331", "KFR-35G/WF2", true, true, false),
            ("220F4057", "KFR-72G/N8HA1Pro", true, false, false),
        ] {
            let profile =
                select(None, Some(code), Some("wall_air_conditioner"), Some(model)).unwrap();
            assert_eq!(profile.feature("horizontal_swing").is_some(), horizontal);
            assert_eq!(profile.feature("breezeless").is_some(), breezeless);
            assert_eq!(profile.feature("strong_cooling").is_some(), strong);
            assert!(profile.feature("comfort_energy_saving").is_none());
            assert!(profile.feature("follow_airflow").is_none());
            assert!(profile.feature("eco").is_none());
            assert!(profile.feature("auxiliary_heat").is_none());
            assert!(profile.feature("vertical_swing").is_some());
            assert!(
                select(
                    Some("524"),
                    Some(code),
                    Some("wall_air_conditioner"),
                    Some(model)
                )
                .is_none()
            );
            assert!(select(None, Some(code), None, Some(model)).is_none());
            let wrong_model = if model == MODEL {
                "KFR-26G/N8VJC3"
            } else {
                MODEL
            };
            assert!(
                select(
                    None,
                    Some(code),
                    Some("wall_air_conditioner"),
                    Some(wrong_model)
                )
                .is_none()
            );
        }
    }

    #[test]
    fn official_independent_scalars_require_reports_and_keep_exact_encoding() {
        let p = select(
            None,
            Some("22040089"),
            Some("wall_air_conditioner"),
            Some("KFR-35G/N8HE1\u{2161}Pro"),
        )
        .unwrap();
        for (name, wire, on) in [
            ("cool_energy_saving", "cool_power_saving", 1),
            ("strong_cooling", "strong_cool", 1),
            ("light_sensing", "light_sensitive", 3),
        ] {
            let feature = p.feature(name).unwrap();
            for value in [false, true] {
                let control = p
                    .control(
                        feature,
                        &json!({name:value}),
                        &BTreeMap::from([(wire.into(), json!(0)), ("buzzer".into(), json!("off"))]),
                    )
                    .unwrap();
                let CloudControl::AcFeature(control) = control else {
                    panic!("Wrong control kind")
                };
                assert_eq!(
                    json!(control),
                    json!({wire:if value {on} else {0}, "buzzer":"off"})
                );
            }
            for raw in [
                BTreeMap::new(),
                BTreeMap::from([(wire.into(), json!(2))]),
                BTreeMap::from([(wire.into(), json!(false))]),
            ] {
                assert!(p.control(feature, &json!({name:true}), &raw).is_err());
            }
        }
    }

    const MODEL: &str = "KFR-35G/WF2";
    fn wall() -> &'static AcFeatureProfile {
        select(
            None,
            Some("TEST1234"),
            Some("wall_air_conditioner"),
            Some(MODEL),
        )
        .unwrap()
    }
    fn raw(value: Value) -> BTreeMap<String, Value> {
        serde_json::from_value(value).unwrap()
    }
    fn packet(profile: &AcFeatureProfile, name: &str, value: Value, state: Value) -> Value {
        serde_json::to_value(
            profile
                .control(
                    profile.feature(name).unwrap(),
                    &json!({name:value}),
                    &raw(state),
                )
                .unwrap(),
        )
        .unwrap()
    }
    #[test]
    fn protected_selector_precedence_and_exclusions_never_inherit_wall_features() {
        for (subtype, code, category, model) in [
            (None, None, Some("wall_air_conditioner"), Some(MODEL)),
            (None, Some("TEST1234"), None, Some(MODEL)),
            (
                None,
                Some("00000Q1F"),
                Some("wall_air_conditioner"),
                Some(MODEL),
            ),
            (
                Some("1"),
                Some("22012895"),
                Some("wall_air_conditioner"),
                Some(MODEL),
            ),
            (
                Some("1"),
                Some("TEST1234"),
                Some("wall_air_conditioner"),
                Some("Unknown"),
            ),
        ] {
            assert!(select(subtype, code, category, model).is_none());
        }
        let portable = select(
            Some("524"),
            Some("22251077"),
            Some("wall_air_conditioner"),
            Some(MODEL),
        )
        .unwrap();
        assert!(portable.feature("vertical_swing").is_some());
        for absent in [
            "horizontal_swing",
            "breezeless",
            "horizontal_direction",
            "button_sound",
        ] {
            assert!(portable.feature(absent).is_none());
        }
        let product = select(
            None,
            Some("22040023"),
            Some("wall-air-conditioner"),
            Some(MODEL),
        )
        .unwrap();
        assert!(product.feature("eco").is_some());
        assert!(product.feature("breezeless").is_none());
        assert!(product.feature("boost").is_none());
    }
    #[test]
    fn reviewed_catalog_has_closed_queries_valid_schemas_and_scalar_only_commands() {
        for profile in catalog() {
            let mut names = std::collections::HashSet::new();
            for q in &profile.queries {
                assert!(query_allowed(q));
                assert!(query_name(q).is_some());
            }
            for f in &profile.features {
                assert!(names.insert(&f.name));
                assert!(!matches!(
                    f.name.as_str(),
                    "self_clean" | "disinfect" | "prepare_food" | "quick_fry"
                ));
                for o in &f.options {
                    f.action()
                        .validate_arguments(&json!({&f.name:&o.value}))
                        .unwrap();
                    assert!(!o.control.keys().any(|key| matches!(
                        key.as_str(),
                        "power" | "temperature" | "mode" | "self_clean" | "disinfect"
                    )));
                }
            }
        }
        assert!(query_name("arbitrary").is_none());
    }
    #[test]
    fn eco_breezeless_and_directions_use_exact_codes_and_reported_buzzer() {
        let p = wall();
        assert_eq!(
            packet(p, "eco", json!(true), json!({"eco":"off","buzzer":"off"})),
            json!({"eco":"on","cool_power_saving":1,"buzzer":"off"})
        );
        assert_eq!(
            packet(p, "eco", json!(false), json!({"eco":"on","buzzer":"on"})),
            json!({"eco":"off","cool_power_saving":0,"buzzer":"on"})
        );
        assert_eq!(
            packet(
                p,
                "breezeless",
                json!(2.0),
                json!({"no_wind_sense":"0","buzzer":"off"})
            ),
            json!({"no_wind_sense":2,"buzzer":"off"})
        );
        assert_eq!(
            packet(
                p,
                "horizontal_direction",
                json!(25),
                json!({"wind_swing_lr_angle":50.0,"buzzer":"on"})
            ),
            json!({"wind_swing_lr_angle":25,"buzzer":"on"})
        );
        for (name, value) in [("breezeless", 4), ("horizontal_direction", 30)] {
            assert!(
                p.control(
                    p.feature(name).unwrap(),
                    &json!({name:value}),
                    &raw(json!({"buzzer":"off"}))
                )
                .is_err()
            );
        }
        assert_eq!(
            p.feature("breezeless")
                .unwrap()
                .observed(&raw(json!({"no_wind_sense":7})))
                .unwrap(),
            Some(json!(7))
        );
        assert_eq!(
            packet(p, "eco", json!(true), json!({"eco":"off"})),
            json!({"eco":"on","cool_power_saving":1})
        );
        assert!(
            p.control(
                p.feature("eco").unwrap(),
                &json!({"eco":true}),
                &raw(json!({"buzzer":"off"}))
            )
            .is_err()
        );
    }
    #[test]
    fn display_wire_formats_anti_draft_pairs_and_follow_enable_are_profile_specific() {
        let p = wall();
        assert_eq!(
            packet(
                p,
                "display",
                json!(false),
                json!({"screen_display_now":"100","screen_display":"on","buzzer":"on"})
            ),
            json!({"screen_display":"off","buzzer":"on"})
        );
        assert_eq!(
            packet(
                p,
                "display",
                json!(true),
                json!({"screen_display_now":0,"screen_display":0,"buzzer":"off"})
            ),
            json!({"screen_display":100,"buzzer":"off"})
        );
        assert_eq!(
            packet(
                p,
                "avoid_direct_airflow",
                json!(false),
                json!({"prevent_straight_wind":2,"buzzer":"off"})
            ),
            json!({"prevent_straight_wind":1,"buzzer":"off"})
        );
        let other = select(Some("1"), Some("22259015"), None, Some(MODEL)).unwrap();
        assert_eq!(
            packet(
                other,
                "avoid_direct_airflow",
                json!(true),
                json!({"prevent_straight_wind":0})
            ),
            json!({"prevent_straight_wind":1})
        );
        assert_eq!(
            packet(
                p,
                "follow_airflow",
                json!(true),
                json!({"follow_body_sense":"off","buzzer":"off"})
            ),
            json!({"follow_body_sense":"on","follow_body_sense_enable":1,"buzzer":"off"})
        );
    }
    #[test]
    fn swing_preserves_every_other_axis_and_missing_axes_refuse() {
        let p = select(Some("1"), Some("22251077"), None, Some(MODEL)).unwrap();
        assert_eq!(
            packet(
                p,
                "horizontal_swing",
                json!(true),
                json!({"wind_swing_lr":"off","wind_swing_ud":"on","wind_swing_lr_under":"off","buzzer":"off"})
            ),
            json!({"wind_swing_lr":"on","wind_swing_ud":"on","wind_swing_lr_under":"off","buzzer":"off"})
        );
        assert!(
            p.control(
                p.feature("horizontal_swing").unwrap(),
                &json!({"horizontal_swing":true}),
                &raw(json!({"wind_swing_lr":"off","wind_swing_ud":"on","buzzer":"off"}))
            )
            .is_err()
        );
    }
    #[test]
    fn extra_queries_only_enrich_missing_relevant_fields() {
        let p = wall();
        let all: BTreeMap<_, _> = p
            .features
            .iter()
            .map(|f| (f.read_field.clone(), json!(0)))
            .collect();
        assert!(p.queries().all(|q| !p.needs_query(q, &all)));
        let none = BTreeMap::new();
        assert_eq!(p.queries().filter(|q| p.needs_query(*q, &none)).count(), 5);
        assert!(!p.needs_query(StateQuery::AcOptional("group_data_four"), &none));
        assert!(
            p.feature("display")
                .unwrap()
                .observed(&raw(json!({"screen_display_now":999})))
                .is_err()
        );
    }
}
