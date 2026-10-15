use homeverb_core::Policy;
use homeverb_domain::{Action, Command, Transport};
use homeverb_host::{MeijuFanGrant, meiju_fan_policy};
use homeverb_midea::AC_FAN_SPEEDS;
use serde_json::json;
use uuid::Uuid;

fn grant(id: &str, wind_speed: u8) -> MeijuFanGrant {
    MeijuFanGrant {
        device_id: id.into(),
        wind_speed,
    }
}

#[test]
fn fan_permissions_are_exact_and_independent_of_power_and_universal_fan_actions() {
    let id = Uuid::new_v4().to_string();
    let policy = meiju_fan_policy(&[grant(&id, 40)]).unwrap();
    let action: Action = serde_json::from_value(json!({"parameters":{"type":"object"},
        "transports":["cloud","lan"],"idempotency":"set_value","risk":"low",
        "atomic":false,"verification":"unavailable"}))
    .unwrap();
    let allowed: Command = serde_json::from_value(json!({"schema_version":"1.0","request_id":"fan",
        "device_id":id,"component_id":"main","capability":"vendor.midea.ac","action":"set_fan_speed",
        "arguments":{"wind_speed":40},"connection":"auto","timeout_ms":1000,"wait_for":"accepted"})).unwrap();
    assert_eq!(
        policy.authorize(&allowed, &action).unwrap(),
        vec![Transport::Cloud]
    );
    assert!(Policy::default().authorize(&allowed, &action).is_err());
    for change in [
        "device",
        "component",
        "power",
        "fan",
        "speed",
        "extra",
        "lan",
    ] {
        let mut denied = allowed.clone();
        match change {
            "device" => denied.device_id = Uuid::new_v4().to_string(),
            "component" => denied.component_id = "freezer".into(),
            "power" => {
                denied.capability = "power".into();
                denied.action = "set".into();
                denied.arguments = json!({"state":true});
            }
            "fan" => denied.capability = "fan".into(),
            "speed" => denied.arguments = json!({"wind_speed":60}),
            "extra" => denied.arguments["confirmed"] = json!(true),
            _ => denied.connection = homeverb_domain::Route::Lan,
        }
        assert_eq!(
            policy.authorize(&denied, &action).unwrap_err().code,
            "POLICY_DENIED"
        );
    }
    let mut elevated = action;
    elevated.risk = homeverb_domain::Risk::Elevated;
    assert!(policy.authorize(&allowed, &elevated).is_err());
}

#[test]
fn fan_startup_rejects_unknown_codes_duplicates_and_forged_authority() {
    let id = "abcdef00-1234-4567-89ab-123456abcdef";
    assert!(meiju_fan_policy(&[]).unwrap().grants.is_empty());
    for speed in AC_FAN_SPEEDS {
        assert_eq!(
            meiju_fan_policy(&[grant(id, speed)]).unwrap().grants.len(),
            1
        );
    }
    let integral =
        serde_json::from_value::<MeijuFanGrant>(json!({"device_id":id,"wind_speed":20.0})).unwrap();
    assert_eq!(integral.wind_speed, 20);
    for speed in [0, 1, 19, 21, 101, 103, 255] {
        assert!(meiju_fan_policy(&[grant(id, speed)]).is_err());
    }
    for invalid in [
        "*".into(),
        Uuid::nil().to_string(),
        id.to_uppercase(),
        id.replace('-', ""),
    ] {
        assert!(meiju_fan_policy(&[grant(&invalid, 20)]).is_err());
    }
    assert!(meiju_fan_policy(&[grant(id, 20), grant(id, 20)]).is_err());
    let mut grants: Vec<_> = (0..64)
        .map(|_| grant(&Uuid::new_v4().to_string(), 20))
        .collect();
    assert_eq!(meiju_fan_policy(&grants).unwrap().grants.len(), 64);
    grants.push(grant(&Uuid::new_v4().to_string(), 20));
    assert!(meiju_fan_policy(&grants).is_err());
    for value in [
        json!({"device_id":id}),
        json!({"device_id":id,"wind_speed":"20"}),
        json!({"device_id":id,"wind_speed":20.5}),
        json!({"device_id":id,"wind_speed":true}),
        json!({"device_id":id,"wind_speed":20,"confirmed":true}),
    ] {
        assert!(serde_json::from_value::<MeijuFanGrant>(value).is_err());
    }
}
