use homeverb_domain::{Action, Command};
use homeverb_host::{MeijuDishwasherGrant, meiju_dishwasher_policy};
use serde_json::json;
use uuid::Uuid;

fn grant(id: &str, arguments: serde_json::Value) -> MeijuDishwasherGrant {
    MeijuDishwasherGrant {
        device_id: id.into(),
        arguments,
    }
}

#[test]
fn dishwasher_grants_bind_exact_arguments_and_still_require_approval() {
    let id = Uuid::new_v4().to_string();
    let policy =
        meiju_dishwasher_policy(&[grant(&id, json!({"program":"eco_wash","additional":0}))])
            .unwrap();
    let mut command: Command = serde_json::from_str(include_str!(
        "../../../examples/command.meiju-dishwasher.json"
    ))
    .unwrap();
    command.device_id = id;
    let capability: homeverb_domain::Capability = serde_json::from_str(include_str!(
        "../../../examples/capability.meiju-dishwasher.json"
    ))
    .unwrap();
    let action = &capability.actions["start_cycle"];
    assert_eq!(
        policy.authorize(&command, action).unwrap_err().code,
        "POLICY_DENIED"
    );
    // The low-risk copy checks grant scope only; the production action stays elevated.
    let ordinary = Action {
        risk: homeverb_domain::Risk::Low,
        ..action.clone()
    };
    assert_eq!(
        policy.authorize(&command, &ordinary).unwrap(),
        vec![homeverb_domain::Transport::Cloud]
    );
    for field in [
        "device",
        "component",
        "capability",
        "action",
        "route",
        "program",
        "additional",
        "omitted",
        "extra",
    ] {
        let mut bad = command.clone();
        match field {
            "device" => bad.device_id = Uuid::new_v4().to_string(),
            "component" => bad.component_id = "freezer".into(),
            "capability" => bad.capability = "power".into(),
            "action" => bad.action = "pause".into(),
            "route" => bad.connection = homeverb_domain::Route::Lan,
            "program" => bad.arguments["program"] = json!("strong_wash"),
            "additional" => bad.arguments["additional"] = json!(9),
            "omitted" => {
                bad.arguments.as_object_mut().unwrap().remove("additional");
            }
            _ => bad.arguments["confirmed"] = json!(true),
        };
        assert!(policy.authorize(&bad, &ordinary).is_err(), "{field}");
    }
}

#[test]
fn dishwasher_grants_validate_before_profile_creation_and_reject_implicit_defaults() {
    let id = "abcdef00-1234-4567-89ab-123456abcdef";
    assert!(meiju_dishwasher_policy(&[]).unwrap().grants.is_empty());
    for program in homeverb_midea::DISHWASHER_PROGRAMS {
        assert!(meiju_dishwasher_policy(&[grant(id, json!({"program":program}))]).is_ok());
    }
    for args in [
        json!(null),
        json!({}),
        json!({"program":"neutral_gear"}),
        json!({"program":true}),
        json!({"program":"eco_wash","additional":null}),
        json!({"program":"eco_wash","additional":1}),
        json!({"program":"eco_wash","additional":9.5}),
        json!({"program":"eco_wash","additional":"0"}),
        json!({"program":"eco_wash","confirmed":true}),
    ] {
        let dir =
            std::env::temp_dir().join(format!("homeverb-dishwasher-grant-{}", Uuid::new_v4()));
        let grants = [grant(id, args)];
        assert!(
            homeverb_host::open_meiju_with_grants(
                &dir,
                homeverb_host::MeijuControlGrants {
                    dishwasher: &grants,
                    ..Default::default()
                }
            )
            .is_err()
        );
        assert!(!dir.exists());
    }
    for bad in [
        id.to_uppercase(),
        id.replace('-', ""),
        Uuid::nil().to_string(),
        "*".into(),
    ] {
        assert!(meiju_dishwasher_policy(&[grant(&bad, json!({"program":"eco_wash"}))]).is_err());
    }
    let duplicate = [
        grant(id, json!({"program":"eco_wash","additional":0})),
        grant(id, json!({"additional":0.0,"program":"eco_wash"})),
    ];
    assert!(meiju_dishwasher_policy(&duplicate).is_err());
    let mut grants: Vec<_> = (0..64)
        .map(|_| grant(&Uuid::new_v4().to_string(), json!({"program":"eco_wash"})))
        .collect();
    assert_eq!(meiju_dishwasher_policy(&grants).unwrap().grants.len(), 64);
    grants.push(grant(id, json!({"program":"eco_wash"})));
    assert!(meiju_dishwasher_policy(&grants).is_err());
    assert!(
        serde_json::from_value::<MeijuDishwasherGrant>(
            json!({"device_id":id,"arguments":{"program":"eco_wash"},"approved":true})
        )
        .is_err()
    );
}
