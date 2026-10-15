use homeverb_domain::{Action, Command, Transport};
use homeverb_host::{MeijuModeGrant, meiju_mode_policy};
use homeverb_midea::{AC_MODES, AcMode};
use serde_json::json;
use uuid::Uuid;

#[test]
fn activation_grants_cover_only_one_exact_combined_action_and_cloud_mode() {
    let id = Uuid::new_v4().to_string();
    let policy = meiju_mode_policy(&[MeijuModeGrant {
        device_id: id.clone(),
        mode: AcMode::Cool,
    }])
    .unwrap();
    let action: Action = serde_json::from_value(
        json!({"parameters":{"type":"object"},"transports":["lan","cloud"],
        "idempotency":"set_value","risk":"low","atomic":false,"verification":"unavailable"}),
    )
    .unwrap();
    let command:Command=serde_json::from_value(json!({"schema_version":"1.0","request_id":"mode","device_id":id,
        "component_id":"main","capability":"vendor.midea.ac","action":"activate_mode","arguments":{"mode":"cool"},
        "connection":"auto","timeout_ms":1000,"wait_for":"accepted"})).unwrap();
    assert_eq!(
        policy.authorize(&command, &action).unwrap(),
        vec![Transport::Cloud]
    );
    for change in [
        "device",
        "component",
        "power",
        "fan",
        "mode",
        "extra",
        "lan",
    ] {
        let mut denied = command.clone();
        match change {
            "device" => denied.device_id = Uuid::new_v4().to_string(),
            "component" => denied.component_id = "freezer".into(),
            "power" => {
                denied.capability = "power".into();
                denied.action = "set".into();
                denied.arguments = json!({"state":true});
            }
            "fan" => {
                denied.action = "set_fan_speed".into();
                denied.arguments = json!({"wind_speed":40});
            }
            "mode" => denied.arguments = json!({"mode":"heat"}),
            "extra" => denied.arguments["power"] = json!("on"),
            _ => denied.connection = homeverb_domain::Route::Lan,
        }
        assert_eq!(
            policy.authorize(&denied, &action).unwrap_err().code,
            "POLICY_DENIED"
        );
    }
    let mut elevated = action;
    elevated.risk = homeverb_domain::Risk::Elevated;
    assert!(policy.authorize(&command, &elevated).is_err());
}

#[test]
fn startup_rejects_unknown_modes_duplicates_noncanonical_ids_and_extra_authority() {
    let id = "abcdef00-1234-4567-89ab-123456abcdef";
    let grant = |device: &str, mode| MeijuModeGrant {
        device_id: device.into(),
        mode,
    };
    assert!(meiju_mode_policy(&[]).unwrap().grants.is_empty());
    for mode in AC_MODES {
        assert_eq!(
            meiju_mode_policy(&[grant(id, mode)]).unwrap().grants.len(),
            1
        );
    }
    for invalid in [
        "*".into(),
        Uuid::nil().to_string(),
        id.to_uppercase(),
        id.replace('-', ""),
    ] {
        assert!(meiju_mode_policy(&[grant(&invalid, AcMode::Cool)]).is_err());
    }
    assert!(meiju_mode_policy(&[grant(id, AcMode::Cool), grant(id, AcMode::Cool)]).is_err());
    let mut grants: Vec<_> = (0..64)
        .map(|_| grant(&Uuid::new_v4().to_string(), AcMode::Cool))
        .collect();
    assert_eq!(meiju_mode_policy(&grants).unwrap().grants.len(), 64);
    grants.push(grant(&Uuid::new_v4().to_string(), AcMode::Cool));
    assert!(meiju_mode_policy(&grants).is_err());
    for value in [
        json!({"device_id":id}),
        json!({"device_id":id,"mode":"off"}),
        json!({"device_id":id,"mode":1}),
        json!({"device_id":id,"mode":"COOL"}),
        json!({"device_id":id,"mode":"cool","confirmed":true}),
    ] {
        assert!(serde_json::from_value::<MeijuModeGrant>(value).is_err());
    }
}
