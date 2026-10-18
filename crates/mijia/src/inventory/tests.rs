use super::*;
use serde_json::json;

use crate::{CloudRequest, CloudResponse, Region, codec};
use async_trait::async_trait;
use std::{collections::VecDeque, sync::Mutex};
use tokio_util::sync::CancellationToken;

struct Pages {
    replies: Mutex<VecDeque<(CloudEndpoint, Value)>>,
}
#[async_trait]
impl CloudTransport for Pages {
    async fn send(&self, request: &CloudRequest, _: &Operation) -> Result<CloudResponse> {
        let (endpoint, value) = self
            .replies
            .lock()
            .unwrap()
            .pop_front()
            .expect("unexpected additional request");
        assert_eq!(request.endpoint(), endpoint);
        let url = reqwest::Url::parse(&format!(
            "https://localhost/?{}",
            std::str::from_utf8(request.body()).unwrap()
        ))
        .unwrap();
        let fields: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        let key = codec::signed_nonce(&fields["ssecurity"], &fields["_nonce"])?;
        let decoded = codec::decrypt(&key, fields["data"].as_bytes())?;
        let data: Value = serde_json::from_slice(decoded.expose()).unwrap();
        if endpoint == CloudEndpoint::HomeDevices {
            assert_eq!(data["home_id"], 1);
            assert_eq!(data["home_owner"], 2);
            assert_eq!(data["limit"], 300);
        }
        let body = codec::encrypt(&key, &serde_json::to_vec(&value).unwrap());
        Ok(CloudResponse {
            body: SecretBytes::new(body.as_str().as_bytes().to_vec()),
            service_token: None,
        })
    }
}
fn pages(replies: Vec<(CloudEndpoint, Value)>) -> Pages {
    Pages {
        replies: Mutex::new(replies.into()),
    }
}
fn session() -> MijiaSession {
    MijiaSession {
        region: Region::Cn,
        user_id: PrivateString::new("2".into()),
        security: PrivateString::new("AAECAwQFBgcICQoLDA0ODw==".into()),
        token: PrivateString::new("fictional-token".into()),
    }
}
fn operation() -> Operation {
    Operation::new(10_000, CancellationToken::new()).unwrap()
}
fn device(did: &str) -> Value {
    json!({"did":did,"name":"Fictional device","model":"fiction.fan.one"})
}

#[tokio::test]
async fn bounded_home_pages_union_devices_and_preserve_partial_scope() {
    let transport = pages(vec![
        (
            CloudEndpoint::Devices,
            json!({"code":0,"result":{"list":[device("fictional-a")]}}),
        ),
        (
            CloudEndpoint::Homes,
            json!({"code":0,"result":{"homelist":[{"id":"1","uid":2}],"has_more":false}}),
        ),
        (
            CloudEndpoint::HomeDevices,
            json!({"code":0,"result":{"device_info":[device("fictional-a"),device("fictional-b")],"has_more":true,"max_did":"fictional-b"}}),
        ),
        (
            CloudEndpoint::HomeDevices,
            json!({"code":0,"result":{"device_info":[device("fictional-c")],"has_more":false}}),
        ),
    ]);
    let inventory = session().enumerate(&transport, &operation()).await.unwrap();
    assert_eq!(inventory.devices.len(), 3);
    assert!(!inventory.complete);
    assert!(inventory.issues.is_empty());
    assert!(transport.replies.lock().unwrap().is_empty());
}

#[tokio::test]
async fn partial_home_error_keeps_prior_devices_but_auth_error_stops() {
    for (code, auth) in [(123, false), (-3, true)] {
        let transport = pages(vec![
            (
                CloudEndpoint::Devices,
                json!({"code":0,"result":{"list":[device("fictional-a")]}}),
            ),
            (
                CloudEndpoint::Homes,
                json!({"code":code,"message":"private-provider-detail"}),
            ),
        ]);
        let result = session().enumerate(&transport, &operation()).await;
        if auth {
            assert_eq!(result.unwrap_err().code, "AUTH_REQUIRED");
        } else {
            let result = result.unwrap();
            assert_eq!(result.devices.len(), 1);
            assert_eq!(result.issues.len(), 1);
            assert!(!format!("{:?}", result.issues).contains("private-provider"));
        }
    }
}

#[tokio::test]
async fn home_model_conflicts_and_cursor_cycles_refuse() {
    let mut conflicting = device("fictional-a");
    conflicting["model"] = json!("fiction.lock.one");
    let transport = pages(vec![
        (
            CloudEndpoint::Devices,
            json!({"code":0,"result":{"list":[device("fictional-a")]}}),
        ),
        (
            CloudEndpoint::Homes,
            json!({"code":0,"result":{"homelist":[{"id":1,"uid":2}],"has_more":false}}),
        ),
        (
            CloudEndpoint::HomeDevices,
            json!({"code":0,"result":{"device_info":[conflicting],"has_more":false}}),
        ),
    ]);
    assert_eq!(
        session()
            .enumerate(&transport, &operation())
            .await
            .unwrap_err()
            .code,
        "DEVICE_IDENTITY_CONFLICT"
    );
    let page =
        json!({"code":0,"result":{"device_info":[],"has_more":true,"max_did":"fictional-cursor"}});
    let transport = pages(vec![
        (
            CloudEndpoint::Devices,
            json!({"code":0,"result":{"list":[]}}),
        ),
        (
            CloudEndpoint::Homes,
            json!({"code":0,"result":{"homelist":[{"id":1,"uid":2}],"has_more":false}}),
        ),
        (CloudEndpoint::HomeDevices, page.clone()),
        (CloudEndpoint::HomeDevices, page),
    ]);
    assert_eq!(
        session()
            .enumerate(&transport, &operation())
            .await
            .unwrap_err()
            .code,
        "MIJIA_INVALID_RESPONSE"
    );
    assert!(transport.replies.lock().unwrap().is_empty());
}

#[tokio::test]
async fn enumeration_checks_cancellation_before_transport() {
    let transport = pages(vec![]);
    let operation = operation();
    operation.cancellation.cancel();
    assert_eq!(
        session()
            .enumerate(&transport, &operation)
            .await
            .unwrap_err()
            .code,
        "CANCELLED"
    );
}

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
