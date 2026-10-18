use homeverb_host::{MijiaFanGrant, mijia_fan_policy};
use serde_json::{Value, json};

fn grant() -> Value {
    json!({"device_id":"40000000-0000-4000-8000-000000000001","component_id":"fan-2","property":"level","value":3})
}
fn parse(value: Value) -> MijiaFanGrant {
    serde_json::from_value(value).unwrap()
}

#[test]
fn exact_device_component_property_value_and_cloud_scope() {
    let policy = mijia_fan_policy(&[parse(grant())]).unwrap();
    let rule = &policy.grants[0];
    assert_eq!(rule.component_id, "fan-2");
    assert_eq!(rule.capability, "vendor.xiaomi.mijia.fan");
    assert_eq!(rule.action, "set_level");
    assert_eq!(rule.argument_schema, json!({"const":{"level":3.0}}));
    assert_eq!(rule.transports, vec![homeverb_domain::Transport::Cloud]);
    for property in ["state", "horizontal_swing"] {
        let mut value = grant();
        value["property"] = json!(property);
        value["value"] = json!(false);
        let policy = mijia_fan_policy(&[parse(value)]).unwrap();
        assert_eq!(
            policy.grants[0].capability,
            if property == "state" {
                "power"
            } else {
                "vendor.xiaomi.mijia.fan"
            }
        );
    }
}

#[test]
fn invalid_scope_types_duplicates_and_bounds_refuse() {
    for (field, value) in [
        ("device_id", json!("00000000-0000-0000-0000-000000000000")),
        ("component_id", json!("fan-0")),
        ("component_id", json!("fan-02")),
        ("component_id", json!("fan-4294967296")),
        ("component_id", json!("lock-2")),
        ("property", json!("unlock")),
        ("value", json!(true)),
        ("value", Value::Null),
    ] {
        let mut value0 = grant();
        value0[field] = value;
        assert!(mijia_fan_policy(&[parse(value0)]).is_err());
    }
    let mut integral = grant();
    integral["value"] = json!(3.0);
    assert!(mijia_fan_policy(&[parse(grant()), parse(integral)]).is_err());
    assert!(mijia_fan_policy(&vec![parse(grant()); 1025]).is_err());
    let mut extra = grant();
    extra["confirmed"] = json!(true);
    assert!(serde_json::from_value::<MijiaFanGrant>(extra).is_err());
}

#[test]
fn malformed_grants_fail_before_profile_creation() {
    let directory =
        std::env::temp_dir().join(format!("homeverb-invalid-mijia-{}", uuid::Uuid::new_v4()));
    let mut value = grant();
    value["component_id"] = json!("lock-2");
    assert!(
        homeverb_host::open_mijia_session_with_grants(&directory, &[parse(value.clone())]).is_err()
    );
    assert!(
        homeverb_host::open_platform_session_with_grants(
            &directory,
            Default::default(),
            &[parse(value)]
        )
        .is_err()
    );
    assert!(!directory.exists());
}
