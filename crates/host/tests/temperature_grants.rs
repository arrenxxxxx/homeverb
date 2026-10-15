use homeverb_domain::{Action, Command, Transport};
use homeverb_host::{MeijuTemperatureGrant, meiju_temperature_policy};
use serde_json::json;
use uuid::Uuid;

#[test]
fn temperature_authority_is_exact_independent_and_never_supplies_missing_device_limits() {
    let id = Uuid::new_v4().to_string();
    let policy = meiju_temperature_policy(&[MeijuTemperatureGrant {
        device_id: id.clone(),
        target_temperature: 26.5,
    }])
    .unwrap();
    let action:Action=serde_json::from_value(json!({"parameters":{"type":"object"},"transports":["cloud","lan"],"idempotency":"set_value","risk":"low","atomic":false,"verification":"unavailable"})).unwrap();
    let command:Command=serde_json::from_value(json!({"schema_version":"1.0","request_id":"setpoint","device_id":id,"component_id":"main","capability":"climate","action":"set","arguments":{"target_temperature":26.5},"connection":"auto","timeout_ms":1000,"wait_for":"accepted"})).unwrap();
    assert_eq!(
        policy.authorize(&command, &action).unwrap(),
        vec![Transport::Cloud]
    );
    for (capability, name, arguments) in [
        ("climate", "set", json!({"target_temperature":26})),
        (
            "climate",
            "set",
            json!({"target_temperature":26.5,"mode":"cool"}),
        ),
        ("power", "set", json!({"state":true})),
        ("vendor.midea.ac", "activate_mode", json!({"mode":"cool"})),
        ("vendor.midea.ac", "set_fan_speed", json!({"wind_speed":40})),
    ] {
        let mut denied = command.clone();
        denied.capability = capability.into();
        denied.action = name.into();
        denied.arguments = arguments;
        assert_eq!(
            policy.authorize(&denied, &action).unwrap_err().code,
            "POLICY_DENIED"
        );
    }
    for field in ["device", "component", "route"] {
        let mut denied = command.clone();
        match field {
            "device" => denied.device_id = Uuid::new_v4().to_string(),
            "component" => denied.component_id = "freezer".into(),
            _ => denied.connection = homeverb_domain::Route::Lan,
        };
        assert!(policy.authorize(&denied, &action).is_err());
    }
    let mut elevated = action;
    elevated.risk = homeverb_domain::Risk::Elevated;
    assert!(policy.authorize(&command, &elevated).is_err());
}

#[test]
fn four_air_conditioners_can_retain_every_reviewed_exact_temperature_grant() {
    let grants: Vec<_> = (0..4)
        .flat_map(|_| {
            let device_id = Uuid::new_v4().to_string();
            (32..=60).map(move |half| MeijuTemperatureGrant {
                device_id: device_id.clone(),
                target_temperature: f64::from(half) / 2.0,
            })
        })
        .collect();
    assert_eq!(grants.len(), 116);
    assert_eq!(meiju_temperature_policy(&grants).unwrap().grants.len(), 116);
}

#[test]
fn temperature_grants_refuse_invalid_targets_ids_duplicate_numeric_values_and_excess_entries() {
    let id = "abcdef00-1234-4567-89ab-123456abcdef";
    let grant = |device: &str, target| MeijuTemperatureGrant {
        device_id: device.into(),
        target_temperature: target,
    };
    assert!(meiju_temperature_policy(&[]).unwrap().grants.is_empty());
    for half in 32..=60 {
        assert!(meiju_temperature_policy(&[grant(id, f64::from(half) / 2.0)]).is_ok());
    }
    for value in [15.5, 30.5, 26.25, f64::NAN, f64::INFINITY] {
        assert!(meiju_temperature_policy(&[grant(id, value)]).is_err());
    }
    for invalid in [
        id.to_uppercase(),
        id.replace('-', ""),
        "*".into(),
        Uuid::nil().to_string(),
    ] {
        assert!(meiju_temperature_policy(&[grant(&invalid, 26.0)]).is_err());
    }
    let duplicates:Vec<MeijuTemperatureGrant>=serde_json::from_value(json!([{"device_id":id,"target_temperature":26},{"device_id":id,"target_temperature":26.0}])).unwrap();
    assert!(meiju_temperature_policy(&duplicates).is_err());
    let mut grants: Vec<_> = (0..256)
        .map(|_| grant(&Uuid::new_v4().to_string(), 26.0))
        .collect();
    assert_eq!(meiju_temperature_policy(&grants).unwrap().grants.len(), 256);
    grants.push(grant(&Uuid::new_v4().to_string(), 26.0));
    assert!(meiju_temperature_policy(&grants).is_err());
    for bad in [
        json!({"device_id":id}),
        json!({"device_id":id,"target_temperature":true}),
        json!({"device_id":id,"target_temperature":"26"}),
        json!({"device_id":id,"target_temperature":26,"confirmed":true}),
    ] {
        assert!(serde_json::from_value::<MeijuTemperatureGrant>(bad).is_err());
    }
}
