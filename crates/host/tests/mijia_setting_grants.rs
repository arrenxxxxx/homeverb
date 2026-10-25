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
