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
