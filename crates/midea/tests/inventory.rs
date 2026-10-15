use async_trait::async_trait;
use homeverb_core::{CredentialRef, Operation, SecretBytes};
use homeverb_domain::{Error, Result};
use homeverb_midea::*;
use serde_json::{Value, json};
use std::{collections::VecDeque, sync::Mutex, time::Duration};
use tokio_util::sync::CancellationToken;

struct Transport {
    account: CredentialRef,
    replies: Mutex<VecDeque<Result<SecretBytes>>>,
    calls: Mutex<Vec<String>>,
    slow_call: Option<usize>,
}

impl Transport {
    fn new(replies: Vec<Result<SecretBytes>>) -> Self {
        Self {
            account: CredentialRef::new(),
            replies: Mutex::new(replies.into()),
            calls: Mutex::new(Vec::new()),
            slow_call: None,
        }
    }
}

#[async_trait]
impl ReadTransport for Transport {
    async fn request(
        &self,
        account: &CredentialRef,
        endpoint: ReadEndpoint<'_>,
        _: &Operation,
    ) -> Result<SecretBytes> {
        assert_eq!(*account, self.account);
        let key = match endpoint {
            ReadEndpoint::Homes => "homes".to_owned(),
            ReadEndpoint::HomeAppliances(id) => id.expose().to_owned(),
            ReadEndpoint::ApplianceState { .. } => panic!("Inventory must not query state."),
        };
        let count = {
            let mut calls = self.calls.lock().unwrap();
            calls.push(key);
            calls.len()
        };
        if self.slow_call == Some(count) {
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
        self.replies
            .lock()
            .unwrap()
            .pop_front()
            .expect("unexpected extra request")
    }
}

fn body(value: Value) -> Result<SecretBytes> {
    Ok(SecretBytes::new(serde_json::to_vec(&value).unwrap()))
}

fn homes(ids: &[u64]) -> Result<SecretBytes> {
    body(
        json!({"code": 0, "data": {"homeList": ids.iter().map(|id| json!({"homegroupId": id})).collect::<Vec<_>>()}}),
    )
}

fn appliance(id: Value, family: &str) -> Value {
    json!({"applianceCode": id, "type": family, "name": "PRIVATE-APPLIANCE-NAME", "productModel": "Model-A", "onlineStatus": "1", "sn": "PRIVATE-SERIAL-IGNORED"})
}

fn devices(home: u64, entries: Vec<Value>) -> Result<SecretBytes> {
    body(
        json!({"code": "0", "data": {"homeList": [{"homegroupId": home, "roomList": [{"applianceList": entries}]}]}}),
    )
}

fn operation() -> Operation {
    Operation::new(1000, CancellationToken::new()).unwrap()
}

#[tokio::test]
async fn traverses_bound_devices_and_keeps_unrecognized_families_without_lan_requests() {
    let transport = Transport::new(vec![
        homes(&[1, 2, 1]),
        devices(
            1,
            vec![appliance(json!(100), "0xAC"), appliance(json!(200), "E1")],
        ),
        devices(
            2,
            vec![
                appliance(json!("000100"), "ac"),
                appliance(json!(300), "0XCA"),
                appliance(json!("9007199254740993"), "34"),
            ],
        ),
    ]);
    let inventory = retrieve_inventory(&transport, transport.account, &operation())
        .await
        .unwrap();
    assert!(inventory.is_complete());
    assert_eq!(inventory.account_ref, transport.account);
    assert_eq!(inventory.appliances.len(), 4);
    assert_eq!(
        inventory.appliances[0].family,
        ApplianceFamily::AirConditioner
    );
    assert_eq!(inventory.appliances[1].family, ApplianceFamily::Dishwasher);
    assert_eq!(
        inventory.appliances[2].family,
        ApplianceFamily::Refrigerator
    );
    assert_eq!(inventory.appliances[3].family, ApplianceFamily::Other(0x34));
    assert_eq!(inventory.appliances[3].id.expose(), "9007199254740993");
    assert_eq!(*transport.calls.lock().unwrap(), ["homes", "1", "2"]);
    let debug = format!("{inventory:?}");
    assert!(!debug.contains("PRIVATE"));
    assert!(!debug.contains("Model-A"));
    assert!(!debug.contains("9007199254740993"));
}

#[tokio::test]
async fn empty_inventory_requires_a_successful_explicit_empty_list() {
    let transport = Transport::new(vec![homes(&[])]);
    let inventory = retrieve_inventory(&transport, transport.account, &operation())
        .await
        .unwrap();
    assert!(inventory.is_complete());
    assert!(inventory.appliances.is_empty());
    assert_eq!(transport.calls.lock().unwrap().len(), 1);
    for value in [
        json!({}),
        json!({"code": 0}),
        json!({"code": 0, "data": {}}),
        json!({"code": 0, "data": {"homeList": null}}),
        json!({"code": true, "data": {"homeList": []}}),
        json!({"code": 0, "data": {"homeList": [{"homegroupId": 0}]}}),
        json!({"code": 0, "data": {"homeList": [{"homegroupId": 1.5}]}}),
        json!({"code": 0, "data": {"homeList": [{"homegroupId": "secret-account-name"}]}}),
    ] {
        let transport = Transport::new(vec![body(value)]);
        let error = retrieve_inventory(&transport, transport.account, &operation())
            .await
            .unwrap_err();
        assert_eq!(error.code, "MIDEA_RESPONSE_INVALID");
        assert!(!error.to_string().contains("secret-account-name"));
    }
}

#[tokio::test]
async fn reports_failed_homes_without_retry_or_empty_account_success() {
    let transport = Transport::new(vec![
        homes(&[1, 2]),
        Err(Error::new("NETWORK_DETAIL", "PRIVATE-RAW-URL-AND-TOKEN")),
        devices(2, vec![appliance(json!(10), "AC")]),
    ]);
    let inventory = retrieve_inventory(&transport, transport.account, &operation())
        .await
        .unwrap();
    assert!(!inventory.is_complete());
    assert_eq!(inventory.appliances.len(), 1);
    assert_eq!(inventory.failures[0].error.code, "MIDEA_REQUEST_FAILED");
    assert_eq!(inventory.failures[0].home_id.expose(), "1");
    assert!(!format!("{inventory:?}").contains("PRIVATE"));
    assert_eq!(transport.calls.lock().unwrap().len(), 3);
}

#[tokio::test]
async fn stops_remaining_requests_when_authorization_expires() {
    for code in [json!(40002), json!("7400")] {
        let transport = Transport::new(vec![
            homes(&[1, 2, 3]),
            devices(1, vec![appliance(json!(10), "AC")]),
            body(json!({"code": code, "msg": "PRIVATE-TOKEN"})),
        ]);
        let inventory = retrieve_inventory(&transport, transport.account, &operation())
            .await
            .unwrap();
        assert!(!inventory.is_complete());
        assert_eq!(inventory.appliances.len(), 1);
        assert_eq!(inventory.failures[0].error.code, "AUTH_REQUIRED");
        assert_eq!(transport.calls.lock().unwrap().len(), 3);
        assert!(!format!("{inventory:?}").contains("PRIVATE"));
    }
    let transport = Transport::new(vec![body(json!({"code": 40002}))]);
    assert_eq!(
        retrieve_inventory(&transport, transport.account, &operation())
            .await
            .unwrap_err()
            .code,
        "AUTH_REQUIRED"
    );
}

#[tokio::test]
async fn refuses_conflicting_identity_instead_of_overwriting_it() {
    for (family, model) in [("CA", "Model-A"), ("AC", "Different-Model")] {
        let mut conflicting = appliance(json!(10), family);
        conflicting["productModel"] = json!(model);
        let transport = Transport::new(vec![
            homes(&[1, 2]),
            devices(1, vec![appliance(json!(10), "AC")]),
            devices(2, vec![conflicting]),
        ]);
        assert_eq!(
            retrieve_inventory(&transport, transport.account, &operation())
                .await
                .unwrap_err()
                .code,
            "DEVICE_IDENTITY_CONFLICT"
        );
    }
}

#[tokio::test]
async fn malformed_or_wrong_home_listing_cannot_publish_its_devices() {
    for response in [
        json!({"code": 0, "data": {"homeList": [{"homegroupId": 9, "roomList": []}]}}),
        json!({"code": 0, "data": {"homeList": [{}]}}),
        json!({"code": 0, "data": {"homeList": [{"roomList": [{}]}]}}),
        json!({"code": 0, "data": {"homeList": [{"roomList": [{"applianceList": [appliance(json!(10), "bad-family")]}]}]}}),
        json!({"code": 1306, "msg": "PRIVATE-PROVIDER-MESSAGE"}),
    ] {
        let transport = Transport::new(vec![homes(&[1]), body(response)]);
        let inventory = retrieve_inventory(&transport, transport.account, &operation())
            .await
            .unwrap();
        assert!(!inventory.is_complete());
        assert!(inventory.appliances.is_empty());
        assert_eq!(inventory.failures.len(), 1);
        assert!(!format!("{inventory:?}").contains("PRIVATE"));
    }
}

#[tokio::test]
async fn missing_or_conflicting_online_hints_remain_unknown() {
    let mut missing = appliance(json!(10), "AC");
    missing.as_object_mut().unwrap().remove("onlineStatus");
    missing.as_object_mut().unwrap().remove("productModel");
    missing.as_object_mut().unwrap().remove("name");
    let mut unknown = appliance(json!(20), "AC");
    unknown["onlineStatus"] = json!(2);
    let transport = Transport::new(vec![
        homes(&[1, 2]),
        devices(1, vec![missing, unknown]),
        devices(2, vec![appliance(json!(10), "AC")]),
    ]);
    let inventory = retrieve_inventory(&transport, transport.account, &operation())
        .await
        .unwrap();
    assert!(
        inventory
            .appliances
            .iter()
            .all(|item| item.online_hint.is_none())
    );
    assert_eq!(inventory.appliances[0].model.as_deref(), Some("Model-A"));
    assert!(inventory.appliances[0].name.is_none());
}

#[tokio::test]
async fn cancellation_prevents_requests_and_drops_inflight_work() {
    let mut transport = Transport::new(vec![homes(&[1])]);
    let operation = operation();
    operation.cancellation.cancel();
    assert_eq!(
        retrieve_inventory(&transport, transport.account, &operation)
            .await
            .unwrap_err()
            .code,
        "CANCELLED"
    );
    assert!(transport.calls.lock().unwrap().is_empty());
    transport.slow_call = Some(2);
    let operation = Operation::new(1000, CancellationToken::new()).unwrap();
    let cancel = async {
        while transport.calls.lock().unwrap().len() < 2 {
            tokio::task::yield_now().await;
        }
        operation.cancellation.cancel();
    };
    let (result, ()) = tokio::time::timeout(Duration::from_secs(2), async {
        tokio::join!(
            retrieve_inventory(&transport, transport.account, &operation),
            cancel
        )
    })
    .await
    .expect("inventory cancellation did not finish");
    assert_eq!(result.unwrap_err().code, "CANCELLED");
    assert_eq!(transport.calls.lock().unwrap().len(), 2);
}

#[tokio::test(start_paused = true)]
async fn all_home_requests_share_one_deadline() {
    let mut transport = Transport::new(vec![homes(&[1])]);
    transport.slow_call = Some(2);
    let operation = Operation::new(5, CancellationToken::new()).unwrap();
    assert_eq!(
        retrieve_inventory(&transport, transport.account, &operation)
            .await
            .unwrap_err()
            .code,
        "DEADLINE_EXCEEDED"
    );
    assert_eq!(transport.calls.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn oversized_or_nonjson_responses_are_redacted_failures() {
    for bytes in [
        vec![b'x'; MAX_RESPONSE_BYTES + 1],
        b"PRIVATE-NONJSON-ERROR".to_vec(),
    ] {
        let transport = Transport::new(vec![Ok(SecretBytes::new(bytes))]);
        let error = retrieve_inventory(&transport, transport.account, &operation())
            .await
            .unwrap_err();
        assert_eq!(error.code, "MIDEA_RESPONSE_INVALID");
        assert!(!error.to_string().contains("PRIVATE"));
    }
}

#[tokio::test]
async fn equal_vendor_ids_in_different_accounts_stay_separate() {
    let a = Transport::new(vec![
        homes(&[1]),
        devices(1, vec![appliance(json!(10), "AC")]),
    ]);
    let b = Transport::new(vec![
        homes(&[1]),
        devices(1, vec![appliance(json!(10), "AC")]),
    ]);
    let first = retrieve_inventory(&a, a.account, &operation())
        .await
        .unwrap();
    let second = retrieve_inventory(&b, b.account, &operation())
        .await
        .unwrap();
    assert_ne!(first.account_ref, second.account_ref);
    assert_eq!(first.appliances.len(), 1);
    assert_eq!(second.appliances.len(), 1);
}
