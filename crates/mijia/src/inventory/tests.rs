use super::*;
use serde_json::json;

#[test]
fn inventory_retains_unsupported_devices_and_does_not_invent_power() {
    let result = decode_inventory(br#"{"code":0,"result":{"list":[{"did":"fictional-1","name":"Front Door","model":"fiction.lock.v1","isOnline":true},{"did":"fictional-2","name":"Desk Fan","model":"fiction.fan.v1"}]}}"#).unwrap();
    assert_eq!(result.devices.len(), 2);
    assert!(!result.complete);
    assert_eq!(result.devices[0].name, "Front Door");
    assert_eq!(result.devices[0].online_hint, Some(true));
    assert_eq!(result.devices[1].online_hint, None);
    assert!(!format!("{:?}", result.devices[0]).contains("fictional"));
}
#[test]
fn duplicate_malformed_and_truncated_inventory_fail() {
    let device = json!({"did":"fictional-1","name":"Doorbell","model":"fiction.doorbell.v1"});
    for value in [
        json!({"result":{"list":[device.clone(),device.clone()]}}),
        json!({"result":{"list":[device.clone()],"has_more":true}}),
        json!({"result":{"list":[{"did":"private","model":"fiction.v1"}]}}),
        json!({"result":{"list":[{"did":"","name":"Fan","model":"fiction.v1"}]}}),
        json!({"result":{"list":[{"did":"private","name":"Fan","model":"fiction.v1","isOnline":"true"}]}}),
    ] {
        assert!(decode_inventory(&serde_json::to_vec(&value).unwrap()).is_err());
    }
}

#[test]
fn property_reply_requires_exact_target_unique_requested_property_and_success_code() {
    let query = PropertyQuery { siid: 2, piid: 1 };
    let value = json!({"result":[{"did":"fictional-1","siid":2,"piid":1,"code":0,"value":false}]});
    let decoded = decode_properties(
        &serde_json::to_vec(&value).unwrap(),
        "fictional-1",
        &[query],
    )
    .unwrap();
    assert_eq!(decoded[0].value, false);
    for patch in [
        json!({"did":"other"}),
        json!({"siid":3}),
        json!({"piid":2}),
        json!({"value":{"secret":"private"}}),
    ] {
        let mut changed = value.clone();
        changed["result"][0]
            .as_object_mut()
            .unwrap()
            .extend(patch.as_object().unwrap().clone());
        assert!(
            decode_properties(
                &serde_json::to_vec(&changed).unwrap(),
                "fictional-1",
                &[query]
            )
            .is_err()
        );
    }
    for patch in [
        json!({"code":-704042011,"value":true}),
        json!({"value":null}),
    ] {
        let mut changed = value.clone();
        changed["result"][0]
            .as_object_mut()
            .unwrap()
            .extend(patch.as_object().unwrap().clone());
        assert!(
            decode_properties(
                &serde_json::to_vec(&changed).unwrap(),
                "fictional-1",
                &[query]
            )
            .unwrap()
            .is_empty()
        );
    }
    let duplicated = json!({"result":[value["result"][0],value["result"][0]]});
    assert!(
        decode_properties(
            &serde_json::to_vec(&duplicated).unwrap(),
            "fictional-1",
            &[query]
        )
        .is_err()
    );
}
