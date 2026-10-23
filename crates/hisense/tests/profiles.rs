use chrono::{TimeZone, Utc};
use homeverb_hisense::WasherProfile;
use serde_json::json;

fn profile(product: &str) -> &'static WasherProfile {
    let category = if product == "WV20W-1" {
        "connector.device.type.washer.multiple"
    } else {
        "connector.device.type.washer"
    };
    WasherProfile::select(product, category, "1001").unwrap()
}

#[test]
fn exact_product_category_and_partner_are_required() {
    for (product, category, partner) in [
        ("WV20W-1", "connector.device.type.washer", "1001"),
        (
            "WH120E5S-1",
            "connector.device.type.washer.multiple",
            "1001",
        ),
        ("virtualWholeFamily", "connector.device.type.washer", "1001"),
        ("WH120E5S-1", "connector.device.type.washer", "other"),
        ("wh120e5s-1", "connector.device.type.washer", "1001"),
    ] {
        assert_eq!(
            WasherProfile::select(product, category, partner)
                .unwrap_err()
                .code,
            "CAPABILITY_UNSUPPORTED"
        );
    }
    assert_eq!(profile("WV20W-1").components(), ["main", "left", "right"]);
    assert_eq!(profile("WH120E5S-1").components(), ["main"]);
}

#[test]
fn programs_keep_product_and_tub_specific_values() {
    let upper = profile("WV20W-1");
    let lower = profile("WH120E5S-1");
    assert_eq!(upper.programs("left").unwrap().count(), 27);
    assert_eq!(upper.programs("right").unwrap().count(), 27);
    assert_eq!(upper.programs("main").unwrap().count(), 0);
    assert_eq!(lower.programs("main").unwrap().count(), 37);
    let baby = |p: &'static WasherProfile, component: &'static str| {
        p.programs(component)
            .unwrap()
            .find(|p| p.code.starts_with("Baby_care_wash"))
            .unwrap()
    };
    assert_eq!(baby(upper, "left").value, "12");
    assert_eq!(baby(upper, "right").value, "12");
    assert_eq!(baby(lower, "main").value, "19");
    assert!(lower.programs("left").is_err());
    assert_eq!(upper.missing_program_function_codes().len(), 22);
    assert_eq!(lower.missing_program_function_codes().len(), 15);
    assert!(!baby(lower, "main").function_codes.is_empty());
}

#[test]
fn cached_tubs_remain_independent_and_no_extra_device_is_created() {
    let input = json!({
        "powerStatus":"0", "onlineStatus":"1", "remoteControlFlag":"0",
        "childLock":"1", "runStatus.multiple":"2", "runStatus.left":"4",
        "runStatus.right":"8", "remainRunTime.left":"17", "remainRunTime.right":"29",
        "workMode.left":"12", "workMode.right":"3", "remainRunTime":"999",
        "deviceId":"fictional-private-device", "unreviewed":"ignored"
    });
    let time = Utc.with_ymd_and_hms(2026, 10, 23, 0, 0, 0).unwrap();
    let state = profile("WV20W-1")
        .decode_cached_state(&serde_json::to_vec(&input).unwrap(), time)
        .unwrap();
    let components = state.components();
    assert_eq!(components.len(), 3);
    assert_eq!(components["main"]["power.state"], false);
    assert_eq!(components["main"]["vendor.hisense.washer.child_lock"], true);
    assert_eq!(
        components["main"]["vendor.hisense.washer.remote_permission"],
        "UnSupported"
    );
    assert_eq!(
        components["left"]["vendor.hisense.washer.remaining_minutes"],
        17
    );
    assert_eq!(
        components["right"]["vendor.hisense.washer.remaining_minutes"],
        29
    );
    assert_eq!(
        components["left"]["vendor.hisense.washer.program"],
        "Baby_care_wash.left"
    );
    assert_eq!(
        components["right"]["vendor.hisense.washer.program"],
        "Strong.right"
    );
    assert!(!format!("{state:?}").contains("fictional-private-device"));
    assert!(state.is_stale());
    assert_eq!(state.captured_at(), time);
}

#[test]
fn power_off_with_remaining_time_is_preserved_without_inventing_running_state() {
    let time = Utc::now();
    let state = profile("WH120E5S-1")
        .decode_cached_state(br#"{"powerStatus":"0","remainRunTime":"43","runStatus":"0","remoteControlFlag":"3","doorLockStatus":null}"#, time)
        .unwrap();
    let main = &state.components()["main"];
    assert_eq!(main["power.state"], false);
    assert_eq!(main["vendor.hisense.washer.remaining_minutes"], 43);
    assert_eq!(main["vendor.hisense.washer.run_status"], "StandingBy");
    assert_eq!(
        main["vendor.hisense.washer.remote_permission"],
        "LongAuthorized"
    );
    assert!(!main.contains_key("vendor.hisense.washer.door_locked"));
    assert!(!main.contains_key("appliance_cycle.running"));
    assert!(state.is_stale());
}

#[test]
fn malformed_known_values_reject_the_entire_projection() {
    for bad in [
        json!({"powerStatus": "2"}),
        json!({"powerStatus": true}),
        json!({"runStatus": "999"}),
        json!({"workMode": "999"}),
        json!({"remoteControlFlag": "4"}),
        json!({"remainRunTime": "-1"}),
        json!({"remainRunTime": "+1"}),
        json!({"remainRunTime": "1.5"}),
        json!({"remainRunTime": "1921"}),
        json!({"remainRunTime": "999999999999999999999999999"}),
        json!({"remainRunTime": 43}),
        json!({"doorLockStatus": ""}),
        json!([]),
    ] {
        let result = profile("WH120E5S-1")
            .decode_cached_state(&serde_json::to_vec(&bad).unwrap(), Utc::now());
        assert_eq!(result.unwrap_err().code, "HISENSE_INVALID_CACHE");
    }
    assert!(
        profile("WH120E5S-1")
            .decode_cached_state(&vec![b' '; 65537], Utc::now())
            .is_err()
    );
}

#[test]
fn absent_fields_have_no_defaults_and_every_capability_is_read_only() {
    let schema = jsonschema::validator_for(
        &serde_json::from_str(include_str!("../../../schemas/capability.schema.json")).unwrap(),
    )
    .unwrap();
    for product in ["WV20W-1", "WH120E5S-1"] {
        let profile = profile(product);
        let empty = profile.decode_cached_state(b"{}", Utc::now()).unwrap();
        assert!(empty.components().values().all(|fields| fields.is_empty()));
        for capability in profile.capabilities() {
            assert!(schema.is_valid(&serde_json::to_value(&capability).unwrap()));
            assert!(capability.actions.is_empty());
            assert!(capability.events.is_empty());
            assert!(capability.component_id.is_some());
            for property in capability.properties.values() {
                assert_eq!(property["readable"], true);
                assert_eq!(property["writable"], false);
            }
        }
    }
}

#[test]
fn cached_projection_satisfies_the_generated_property_domains() {
    for product in ["WV20W-1", "WH120E5S-1"] {
        let profile = profile(product);
        let input = if product == "WV20W-1" {
            json!({"powerStatus":"1", "workMode.left":"12", "remainRunTime.right":"1920"})
        } else {
            json!({"powerStatus":"0", "workMode":"19", "doorStatus":"1", "doorLockStatus":"0"})
        };
        let state = profile
            .decode_cached_state(&serde_json::to_vec(&input).unwrap(), Utc::now())
            .unwrap();
        for (component, readings) in state.components() {
            for (name, value) in readings {
                let definitions = profile.capabilities();
                let property = definitions
                    .iter()
                    .filter(|cap| cap.component_id.as_ref() == Some(component))
                    .find_map(|cap| {
                        name.strip_prefix(&format!("{}.", cap.id))
                            .and_then(|p| cap.properties.get(p))
                    })
                    .unwrap();
                if let Some(values) = property["enum"].as_array() {
                    assert!(values.contains(value));
                }
                let mut schema = property.clone();
                for key in ["readable", "writable", "unit"] {
                    schema.as_object_mut().unwrap().remove(key);
                }
                assert!(jsonschema::validator_for(&schema).unwrap().is_valid(value));
            }
        }
    }
}

#[test]
fn stage_and_pause_codes_are_never_shared_across_products() {
    let upper = profile("WV20W-1")
        .decode_cached_state(
            br#"{"runStatus.left":"15","startPauseStatus.left":"1"}"#,
            Utc::now(),
        )
        .unwrap();
    let lower = profile("WH120E5S-1")
        .decode_cached_state(br#"{"runStatus":"15","startPauseStatus":"1"}"#, Utc::now())
        .unwrap();
    assert_eq!(
        upper.components()["left"]["vendor.hisense.washer.run_status"],
        "WaterDetection"
    );
    assert_eq!(
        lower.components()["main"]["vendor.hisense.washer.run_status"],
        "AICheck"
    );
    assert_eq!(
        upper.components()["left"]["vendor.hisense.washer.start_pause_status"],
        "StandingBy"
    );
    assert_eq!(
        lower.components()["main"]["vendor.hisense.washer.start_pause_status"],
        "Paused"
    );
}
