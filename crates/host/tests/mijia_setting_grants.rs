use homeverb_host::{MijiaSettingGrant, mijia_setting_policy};
use serde_json::{Value, json};
fn grant(component: &str, property: &str, value: Value) -> MijiaSettingGrant {
    MijiaSettingGrant {
        device_id: "40000000-0000-4000-8000-000000000001".into(),
        component_id: component.into(),
        property: property.into(),
        value,
    }
}
#[test]
fn exact_settings_keep_domains_closed_and_accept_full_color_temperature_range() {
    let grants: Vec<_> = (2700..=6500)
        .map(|v| grant("light-2", "color_temperature_3", json!(v)))
        .collect();
    let policy = mijia_setting_policy(&grants).unwrap();
    assert_eq!(policy.grants.len(), 3801);
    assert_eq!(
        policy.grants[1621].argument_schema,
        json!({"const":{"color_temperature_3":4321}})
    );
    for (component, property, value) in [
        ("light-2", "color_temperature_3", json!(2699)),
        ("light-2", "color_temperature_3", json!(6501)),
        ("light-2", "color_temperature_3", json!(4321.5)),
        ("light-2", "brightness_2", json!(0)),
        ("light-2", "brightness_2", json!(101)),
        ("light-2", "mode_7", json!(1)),
        ("air_purifier-2", "fan_level_2", json!(3)),
        ("air_purifier_favorite-8", "favorite_fan_level_1", json!(17)),
        ("fan-2", "state", json!(true)),
        ("switch-2", "state", json!(true)),
        ("light-3", "state", json!(true)),
        ("light-2", "state", json!(1)),
        ("light-2", "favorite_fan_level_1", json!(1)),
    ] {
        assert!(mijia_setting_policy(&[grant(component, property, value)]).is_err());
    }
    assert!(
        mijia_setting_policy(&[
            grant("light-2", "brightness_2", json!(50)),
            grant("light-2", "brightness_2", json!(50.0))
        ])
        .is_err()
    );
    let mut excessive = grants;
    excessive.resize_with(4097, || grant("light-2", "brightness_2", json!(50)));
    assert!(mijia_setting_policy(&excessive).is_err());
    let mut malformed = grant("light-2", "state", json!(true));
    malformed.device_id = "not-an-id".into();
    assert!(mijia_setting_policy(&[malformed]).is_err());
}

#[test]
fn auxiliary_settings_keep_exact_codes_and_do_not_admit_fan_timers_or_sibling_lights() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../examples/native-open.mijia-auxiliary.json"
    ))
    .unwrap();
    let grants: Vec<MijiaSettingGrant> =
        serde_json::from_value(fixture["mijia_setting_grants"].clone()).unwrap();
    let policy = mijia_setting_policy(&grants).unwrap();
    assert_eq!(policy.grants.len(), 7);
    for (input, output) in grants.iter().zip(&policy.grants) {
        assert_eq!(
            output.argument_schema,
            json!({"const":{input.property.clone():input.value}})
        );
        assert_eq!(output.transports, vec![homeverb_domain::Transport::Cloud]);
        for value in [Value::Null, json!("true"), json!(-1), json!(3), json!(1.5)] {
            let mut bad = input.clone();
            bad.value = value;
            assert!(mijia_setting_policy(&[bad]).is_err());
        }
    }
    for (component, property, value) in [
        ("function-3", "timing_1", json!(1)),
        ("function-3", "reserve_2", json!(1)),
        ("fan-2", "state", json!(true)),
        ("fan-2", "level", json!(1)),
        ("light-7", "brightness_2", json!(1)),
        ("light-6", "brightness_2", json!(50)),
        ("light-6", "state", json!(1)),
        ("function-3", "quiet_5", json!(1)),
    ] {
        assert!(mijia_setting_policy(&[grant(component, property, value)]).is_err());
    }
    let mut float = grant("light-6", "brightness_2", json!(1.0));
    assert!(
        mijia_setting_policy(&[float.clone(), grant("light-6", "brightness_2", json!(1))]).is_err()
    );
    float.component_id = "hood-2".into();
    assert!(mijia_setting_policy(&[float]).is_err());
}

#[test]
fn hood_manual_grants_retain_exact_codes_without_clock_or_companion_authority() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../examples/native-open.mijia-hood.json"
    ))
    .unwrap();
    let grants: Vec<MijiaSettingGrant> =
        serde_json::from_value(fixture["mijia_setting_grants"].clone()).unwrap();
    let policy = mijia_setting_policy(&grants).unwrap();
    assert_eq!(policy.grants.len(), 10);
    for (input, output) in grants.iter().zip(&policy.grants) {
        assert_eq!(
            output.argument_schema,
            json!({"const":{input.property.clone():input.value}})
        );
        assert_eq!(output.component_id, input.component_id);
        assert_eq!(output.action, format!("set_{}", input.property));
    }
    for (component, property, value) in [
        ("hood-2", "on_1", json!(1)),
        ("hood-2", "state", json!(true)),
        ("fan_control-3", "fan_level_1", json!(0)),
        ("fan_control-3", "fan_level_1", json!(5)),
        ("fan_control-3", "fan_level_1", json!(1.5)),
        ("screen-13", "display_state_1", json!(2)),
        ("screen-13", "auto_screen_off_2", json!(1)),
        ("screen-13", "display_state_6", json!(1)),
        ("screen-13", "auto_screen_off_time_3", json!(5)),
        ("screen-13", "auto_screen_off_time_5", json!(1260)),
        ("hood-2", "ch4_out_auto_ventilation_24", json!(false)),
    ] {
        assert!(mijia_setting_policy(&[grant(component, property, value)]).is_err());
    }
}

#[test]
fn hood_preferences_admit_only_exact_independent_values() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../examples/native-open.mijia-hood-preferences.json"
    ))
    .unwrap();
    let grants: Vec<MijiaSettingGrant> =
        serde_json::from_value(fixture["mijia_setting_grants"].clone()).unwrap();
    let policy = mijia_setting_policy(&grants).unwrap();
    assert_eq!(policy.grants.len(), 12);
    for (input, output) in grants.iter().zip(&policy.grants) {
        assert_eq!(
            output.argument_schema,
            json!({"const":{input.property.clone():input.value}})
        );
        assert_eq!(output.capability, "vendor.xiaomi.mijia.hood");
        assert_eq!(output.component_id, "hood-2");
        assert_eq!(output.action, format!("set_{}", input.property));
        let mut foreign = input.clone();
        foreign.component_id = "hood-3".into();
        assert!(mijia_setting_policy(&[foreign]).is_err());
    }
    for value in [
        json!(0),
        json!(10),
        json!(19),
        json!(21),
        json!(25),
        json!(49),
        json!(51),
        json!(60),
        json!(20.5),
        json!(true),
        json!("20"),
    ] {
        assert!(mijia_setting_policy(&[grant("hood-2", "clean_remind_time_12", value)]).is_err());
    }
    for name in [
        "power_on_with_light_9",
        "power_off_with_light_10",
        "clean_remind_on_11",
        "gestures_18",
    ] {
        assert!(mijia_setting_policy(&[grant("hood-2", name, json!(1))]).is_err());
    }
}
