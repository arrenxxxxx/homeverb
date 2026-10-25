use super::*;
use crate::CloudResponse;
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio_util::sync::CancellationToken;

fn kind(device: &str, code: &str) -> String {
    format!("urn:miot-spec-v2:device:{device}:{code}:fiction-one:1")
}
fn property(iid: u32, name: &str, code: &str, format: &str, access: Value) -> Value {
    json!({"iid":iid,"type":format!("urn:miot-spec-v2:property:{name}:{code}:fiction-one:1"),"format":format,"access":access})
}
fn service(iid: u32, name: &str, code: &str, properties: Value) -> Value {
    json!({"iid":iid,"type":format!("urn:miot-spec-v2:service:{name}:{code}:fiction-one:1"),"properties":properties})
}
fn fan() -> Value {
    let mut level = property(
        2,
        "fan-level",
        "00000016",
        "uint8",
        json!(["read", "write"]),
    );
    level["value-range"] = json!([1, 4, 1]);
    json!({"type":kind("fan","0000A005"),"services":[
        service(1,"device-information","00007801",json!([property(3,"serial-number","00000003","string",json!(["read"]))])),
        service(2,"fan","00007808",json!([
            property(1,"on","00000006","bool",json!(["read","write","notify"])), level,
            property(3,"horizontal-swing","00000017","bool",json!(["read","write"]))
        ]))
    ]})
}
fn parse(value: &Value) -> Result<ModelMapping> {
    ModelMapping::decode(
        "fiction.fan.one",
        &kind("fan", "0000A005"),
        &serde_json::to_vec(value).unwrap(),
    )
}

#[test]
fn fan_power_requires_exact_service_property_type_and_read_permission() {
    let mapped = parse(&fan()).unwrap();
    assert_eq!(mapped.properties.len(), 3);
    assert_eq!(mapped.properties[0].capability, "power");
    assert_eq!(mapped.properties[0].property, "state");
    assert_eq!(mapped.properties[0].component, "fan-2");
    assert_eq!(
        mapped.properties[0].definition,
        json!({"type":"boolean","readable":true,"writable":false})
    );
    assert_eq!(
        mapped.properties[1].write_domain.as_ref().unwrap()["maximum"],
        4.0
    );
    assert_eq!(mapped.properties[1].definition["maximum"], 255);
    assert!(mapped.capabilities().iter().all(|c| c.actions.is_empty()));
    for change in [
        json!({"format":"uint8"}),
        json!({"access":["notify"]}),
        json!({"type":"urn:miot-spec-v2:property:on:DEADBEEF:fiction-one:1"}),
    ] {
        let mut spec = fan();
        spec["services"][1]["properties"][0]
            .as_object_mut()
            .unwrap()
            .extend(change.as_object().unwrap().clone());
        assert_eq!(parse(&spec).unwrap().properties.len(), 2);
    }
}

#[test]
fn identity_and_event_only_properties_are_not_live_state() {
    let mut spec = fan();
    spec["services"][1]["properties"][0]["access"] = json!([]);
    let mapped = parse(&spec).unwrap();
    assert!(
        mapped
            .properties
            .iter()
            .all(|p| p.siid != 1 && p.property != "state")
    );
}

#[test]
fn locks_preserve_raw_status_and_never_map_switch_or_writes() {
    let k = kind("lock", "0000A038");
    let mut status = property(
        5,
        "status",
        "00000007",
        "uint8",
        json!(["read", "write", "notify"]),
    );
    status["value-list"] =
        json!([{"value":0,"description":"Open"},{"value":255,"description":"Others"}]);
    let spec = json!({"type":k,"services":[service(2,"lock","00007855",json!([
        status,property(7,"on","00000006","bool",json!(["read","write"])),property(8,"operation-id","00000097","uint32",json!(["read"]))
    ])),service(4,"battery","00007805",json!([property(1,"battery-level","00000014","uint8",json!(["read"]))]))]});
    let mapped =
        ModelMapping::decode("fiction.lock.one", &k, &serde_json::to_vec(&spec).unwrap()).unwrap();
    assert_eq!(mapped.properties.len(), 2);
    assert_eq!(mapped.properties[0].property, "status");
    assert_eq!(mapped.properties[0].definition["type"], "integer");
    assert!(
        !mapped.properties[0]
            .definition
            .as_object()
            .unwrap()
            .contains_key("enum")
    );
    assert!(mapped.properties.iter().all(|p| p.write_domain.is_none()));
}

#[test]
fn doorbell_does_not_invent_power_from_volume() {
    let k = kind("doorbell", "0000A03B");
    let spec = json!({"type":k,"services":[service(2,"doorbell","00007857",json!([property(2,"volume","00000013","uint8",json!(["read","write"]))]))]});
    let mapped = ModelMapping::decode(
        "fiction.doorbell.one",
        &k,
        &serde_json::to_vec(&spec).unwrap(),
    )
    .unwrap();
    assert_eq!(mapped.properties.len(), 1);
    assert_eq!(mapped.properties[0].property, "volume");
    assert!(mapped.properties[0].write_domain.is_none());
}

#[test]
fn mismatches_duplicate_wire_keys_and_duplicate_semantics_fail() {
    let mut mismatch = fan();
    mismatch["type"] = json!(kind("lock", "0000A038"));
    assert!(parse(&mismatch).is_err());
    let mut duplicate = fan();
    let service = duplicate["services"][1].clone();
    duplicate["services"].as_array_mut().unwrap().push(service);
    assert!(parse(&duplicate).is_err());
    let mut duplicate = fan();
    let mut prop = duplicate["services"][1]["properties"][0].clone();
    prop["iid"] = json!(9);
    duplicate["services"][1]["properties"]
        .as_array_mut()
        .unwrap()
        .push(prop);
    assert!(parse(&duplicate).is_err());
    for range in [json!([0, 1, 0]), json!([5, 1, 1]), json!([1, 4])] {
        let mut spec = fan();
        spec["services"][1]["properties"][1]["value-range"] = range;
        assert!(parse(&spec).is_err());
    }
}

#[test]
fn model_selection_requires_exact_released_model_and_unambiguous_version() {
    let k = kind("fan", "0000A005");
    let entry = json!({"model":"fiction.fan.one","status":"released","version":1,"type":k});
    let index =
        SpecIndex::decode(&serde_json::to_vec(&json!({"instances":[entry.clone()]})).unwrap())
            .unwrap();
    assert_eq!(
        index.resolve("fiction.fan.one", None).unwrap(),
        Some(k.clone())
    );
    assert!(index.resolve("fiction.fan.two", None).unwrap().is_none());
    assert!(
        index
            .resolve(
                "fiction.fan.one",
                Some("urn:miot-spec-v2:device:fan:0000A005:fiction-other:1")
            )
            .unwrap()
            .is_none()
    );
    let mut debug = entry.clone();
    debug["status"] = json!("debug");
    let index =
        SpecIndex::decode(&serde_json::to_vec(&json!({"instances":[debug]})).unwrap()).unwrap();
    assert!(index.resolve("fiction.fan.one", None).unwrap().is_none());
    let mut other = entry.clone();
    other["type"] = json!(kind("fan", "0000FFFF"));
    let index =
        SpecIndex::decode(&serde_json::to_vec(&json!({"instances":[entry,other]})).unwrap())
            .unwrap();
    assert!(index.resolve("fiction.fan.one", None).is_err());
}

struct PublicMetadata {
    calls: AtomicUsize,
}
#[async_trait]
impl CloudTransport for PublicMetadata {
    async fn send(&self, request: &CloudRequest, _: &Operation) -> Result<CloudResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert!(request.cookie().is_empty());
        assert!(request.body().is_empty());
        let value = if request.endpoint() == CloudEndpoint::SpecIndex {
            json!({"instances":[{"model":"fiction.fan.one","status":"released","version":1,"type":kind("fan","0000A005")}]})
        } else {
            assert_eq!(request.endpoint(), CloudEndpoint::SpecInstance);
            fan()
        };
        Ok(CloudResponse {
            body: SecretBytes::new(serde_json::to_vec(&value).unwrap()),
            service_token: None,
        })
    }
}
#[tokio::test]
async fn explicit_metadata_fetch_uses_no_account_and_honors_cancellation() {
    let transport = PublicMetadata {
        calls: AtomicUsize::new(0),
    };
    let operation = Operation::new(1000, CancellationToken::new()).unwrap();
    let index = SpecIndex::fetch(&transport, &operation).await.unwrap();
    let mapping = index
        .mapping("fiction.fan.one", None, &transport, &operation)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(mapping.properties.len(), 3);
    assert_eq!(transport.calls.load(Ordering::SeqCst), 2);
    operation.cancellation.cancel();
    assert!(
        index
            .mapping("fiction.fan.one", None, &transport, &operation)
            .await
            .is_err()
    );
    assert!(SpecIndex::fetch(&transport, &operation).await.is_err());
    assert_eq!(transport.calls.load(Ordering::SeqCst), 2);
}

#[test]
fn exact_reference_catalog_maps_only_reviewed_scalar_reads() {
    let mut count = 0;
    assert_eq!(references().len(), 17);
    for profile in references() {
        let mapping = ModelMapping::pinned(&profile.model, Some(&profile.spec_type))
            .unwrap()
            .unwrap();
        mapping.validate_saved().unwrap();
        count += mapping.properties.len();
        assert!(
            mapping
                .properties
                .iter()
                .all(|p| p.definition["type"] != "string")
        );
        assert!(
            mapping
                .capabilities()
                .iter()
                .all(|c| c.actions.is_empty() && c.events.is_empty())
        );
        assert!(
            mapping
                .control_options()
                .iter()
                .all(|o| o.component_id.starts_with("fan-")
                    && matches!(
                        o.action.as_str(),
                        "set"
                            | "set_level"
                            | "set_mode"
                            | "set_horizontal_swing"
                            | "set_horizontal_angle"
                            | "set_horizontal_swing_included_angle_6"
                            | "set_vertical_swing_7"
                            | "set_vertical_swing_included_angle_8"
                            | "set_anion_9"
                    ))
        );
        assert!(
            ModelMapping::pinned(
                &format!("{}.other", profile.model),
                Some(&profile.spec_type)
            )
            .unwrap()
            .is_none()
        );
        assert!(
            ModelMapping::pinned(&profile.model, Some(&format!("{}0", profile.spec_type)))
                .unwrap()
                .is_none()
        );
    }
    assert!(
        count > 250,
        "catalog unexpectedly lost read mappings: {count}"
    );
    let fan = ModelMapping::pinned("jdjz.fan.2072fc", None)
        .unwrap()
        .unwrap();
    assert_eq!(fan.properties.len(), 12);
    assert!(
        fan.properties
            .iter()
            .any(|p| p.property == "vertical_swing_7"
                && p.write_domain == Some(json!({"type":"boolean"})))
    );
    let hub = ModelMapping::pinned("xiaomi.gateway.hub1", None)
        .unwrap()
        .unwrap();
    assert_eq!(hub.properties.len(), 2);
    assert!(
        hub.properties
            .iter()
            .all(|p| !p.property.contains("mac") && !p.property.contains("address"))
    );
    assert!(
        ModelMapping::pinned("miwear.watch.n66tc", None)
            .unwrap()
            .is_none()
    );
}

#[test]
fn extended_urns_and_reference_fields_keep_exact_identity_and_types() {
    let profile = reference(
        "jdjz.fan.2072fc",
        "urn:miot-spec-v2:device:fan:0000A005:jdjz-2072fc:1:0000D062",
    )
    .unwrap();
    assert!(validate_device_type(&profile.spec_type).is_ok());
    for suffix in [":", ":0000D062:1", ":0000D06Z", ":123"] {
        assert!(
            validate_device_type(&format!(
                "urn:miot-spec-v2:device:fan:0000A005:fiction:1{suffix}"
            ))
            .is_err()
        );
    }
    let original = ModelMapping::pinned(&profile.model, None).unwrap().unwrap();
    let mut forged = original.clone();
    forged.properties[0].piid += 100;
    assert!(forged.validate_saved().is_err());
    let mut source = profile.source.clone();
    let services = source["services"].as_array_mut().unwrap();
    let custom = services.iter_mut().find(|s| s["iid"] == 3).unwrap();
    custom["properties"][0]["format"] = json!("bool");
    let mapped = ModelMapping::decode(
        &profile.model,
        &profile.spec_type,
        &serde_json::to_vec(&source).unwrap(),
    )
    .unwrap();
    assert!(!mapped.properties.iter().any(|p| p.siid == 3 && p.piid == 1));
    let legacy = ModelMapping::decode_inner(
        &profile.model,
        &profile.spec_type,
        &serde_json::to_vec(&profile.source).unwrap(),
        false,
    )
    .unwrap();
    assert!(legacy.properties.len() < original.properties.len());
    legacy.validate_saved().unwrap();
    assert_eq!(legacy.upgraded().unwrap(), original);
}

#[test]
fn reference_fan_controls_preserve_fields_and_accept_only_exact_source_domains() {
    let profile = reference(REFERENCE_FAN_MODEL, REFERENCE_FAN_TYPE).unwrap();
    let current = ModelMapping::pinned(REFERENCE_FAN_MODEL, None)
        .unwrap()
        .unwrap();
    let options = current.control_options();
    for (name, domain) in [
        (
            "horizontal_swing_included_angle_6",
            json!({"type":"integer","minimum":0,"maximum":255,"enum":[60,90,120]}),
        ),
        ("vertical_swing_7", json!({"type":"boolean"})),
        (
            "vertical_swing_included_angle_8",
            json!({"type":"integer","minimum":0,"maximum":255,"enum":[60,105,120]}),
        ),
        ("anion_9", json!({"type":"boolean"})),
    ] {
        let option = options
            .iter()
            .find(|o| o.action == format!("set_{name}"))
            .unwrap();
        assert_eq!(option.component_id, "fan-2");
        assert_eq!(option.definition.parameters["properties"][name], domain);
    }
    assert_eq!(options.len(), 8);
    for change in [
        "model", "service", "property", "unit", "format", "read", "write", "domain",
    ] {
        let mut source = profile.source.clone();
        let field = &mut source["services"][0]["properties"][5];
        assert_eq!(field["iid"], 6);
        match change {
            "service" => {
                source["services"][0]["type"] =
                    json!("urn:miot-spec-v2:service:fan:00007808:other:1")
            }
            "property" => {
                field["type"] = json!("urn:miot-spec-v2:property:horizontal-angle:00000019:other:1")
            }
            "unit" => field["unit"] = json!("percentage"),
            "format" => field["format"] = json!("uint16"),
            "read" => field["access"] = json!(["write"]),
            "write" => field["access"] = json!(["read"]),
            "domain" => field["value-list"] = json!([{"value":60},{"value":180}]),
            _ => {}
        }
        let mapped = ModelMapping::decode(
            if change == "model" {
                "fiction.fan.other"
            } else {
                REFERENCE_FAN_MODEL
            },
            REFERENCE_FAN_TYPE,
            &serde_json::to_vec(&source).unwrap(),
        )
        .unwrap();
        assert!(
            !mapped
                .control_options()
                .iter()
                .any(|o| o.action == "set_horizontal_swing_included_angle_6"),
            "{change}"
        );
    }
    // The entire previous derivation remains accepted; edited partial derivations do not.
    let mut old = current.clone();
    for property in &mut old.properties {
        if property.siid == 2 && matches!(property.piid, 6..=9) {
            property.write_domain = None;
        }
    }
    old.validate_saved().unwrap();
    assert_eq!(old.upgraded().unwrap(), current);
    assert_eq!(old.control_options().len(), 4);
    let mut forged = old.clone();
    forged
        .properties
        .iter_mut()
        .find(|p| p.piid == 6 && p.siid == 2)
        .unwrap()
        .write_domain = Some(json!({"type":"integer","enum":[180]}));
    assert!(forged.validate_saved().is_err());
    assert_eq!(
        serde_json::to_value(old.capabilities()).unwrap(),
        serde_json::to_value(current.capabilities()).unwrap()
    );
}
