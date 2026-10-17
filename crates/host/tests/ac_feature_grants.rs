use homeverb_core::Policy;
use homeverb_domain::{Action, Command, Transport};
use homeverb_host::{MeijuAcFeatureGrant, meiju_ac_feature_policy};
use serde_json::{Value, json};
use uuid::Uuid;
fn grant(id: &str, feature: &str, value: Value) -> MeijuAcFeatureGrant {
    MeijuAcFeatureGrant {
        device_id: id.into(),
        feature: feature.into(),
        value,
    }
}
#[test]
fn startup_optional_permissions_are_exact_independent_and_cloud_only() {
    let id = Uuid::new_v4().to_string();
    let policy = meiju_ac_feature_policy(&[grant(&id, "eco", json!(true))]).unwrap();
    let action:Action=serde_json::from_value(json!({"parameters":{"type":"object"},"transports":["cloud","lan"],"idempotency":"set_value","risk":"low","atomic":false,"verification":"unavailable"})).unwrap();
    let allowed:Command=serde_json::from_value(json!({"schema_version":"1.0","request_id":"eco","device_id":id,"component_id":"main","capability":"vendor.midea.ac","action":"set_eco","arguments":{"eco":true},"connection":"auto","timeout_ms":1000,"wait_for":"accepted"})).unwrap();
    assert_eq!(
        policy.authorize(&allowed, &action).unwrap(),
        vec![Transport::Cloud]
    );
    assert!(Policy::default().authorize(&allowed, &action).is_err());
    for change in [
        "device",
        "component",
        "action",
        "value",
        "extra",
        "route",
        "capability",
    ] {
        let mut denied = allowed.clone();
        match change {
            "device" => denied.device_id = Uuid::new_v4().to_string(),
            "component" => denied.component_id = "other".into(),
            "action" => {
                denied.action = "set_boost".into();
                denied.arguments = json!({"boost":true});
            }
            "value" => denied.arguments = json!({"eco":false}),
            "extra" => denied.arguments["confirmed"] = json!(true),
            "route" => denied.connection = homeverb_domain::Route::Lan,
            _ => denied.capability = "power".into(),
        };
        assert!(policy.authorize(&denied, &action).is_err());
    }
}
#[test]
fn startup_rejects_unsupported_values_duplicate_numeric_forms_and_unbounded_authority() {
    let id = "abcdef00-1234-4567-89ab-123456abcdef";
    assert!(meiju_ac_feature_policy(&[]).unwrap().grants.is_empty());
    for (f, v) in [
        ("eco", json!("on")),
        ("eco", json!(1)),
        ("breezeless", json!(4)),
        ("horizontal_direction", json!(30)),
        ("self_clean", json!(true)),
        ("display", Value::Null),
    ] {
        assert!(meiju_ac_feature_policy(&[grant(id, f, v)]).is_err());
    }
    assert!(
        meiju_ac_feature_policy(&[
            grant(id, "breezeless", json!(2)),
            grant(id, "breezeless", json!(2.0))
        ])
        .is_err()
    );
    assert_eq!(
        meiju_ac_feature_policy(&[grant(id, "breezeless", json!(2.0))])
            .unwrap()
            .grants[0]
            .argument_schema,
        json!({"const":{"breezeless":2}})
    );
    for invalid in [
        "*".into(),
        Uuid::nil().to_string(),
        id.to_uppercase(),
        id.replace('-', ""),
    ] {
        assert!(meiju_ac_feature_policy(&[grant(&invalid, "eco", json!(true))]).is_err());
    }
    let mut grants: Vec<_> = (0..1024)
        .map(|_| grant(&Uuid::new_v4().to_string(), "eco", json!(true)))
        .collect();
    assert_eq!(meiju_ac_feature_policy(&grants).unwrap().grants.len(), 1024);
    grants.push(grant(&Uuid::new_v4().to_string(), "eco", json!(true)));
    assert!(meiju_ac_feature_policy(&grants).is_err());
    for value in [
        json!({"device_id":id,"feature":"eco"}),
        json!({"device_id":id,"feature":"eco","value":true,"confirmed":true}),
    ] {
        assert!(serde_json::from_value::<MeijuAcFeatureGrant>(value).is_err());
    }
}
