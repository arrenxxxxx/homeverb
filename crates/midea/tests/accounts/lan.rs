use super::*;
use homeverb_midea::lan::LanUdpIdMethod;
use std::collections::VecDeque;

const ID: &str = "90123456789";
// Independent Python hashlib over six-byte big/little-endian fictional IDs.
const BIG: &str = "3173c5bf997f38a6ae430ac51d5e3666";
const LITTLE: &str = "70de887c01ed6832d8ed0f4056730542";
const CLIENT_ID: &str = "02f54511dbe6cb5d";

enum Reply {
    Body(Value),
    Error,
    Wait,
}

struct Script {
    replies: Mutex<VecDeque<(CloudEndpoint, Reply)>>,
    calls: Mutex<Vec<(CloudEndpoint, Value)>>,
}

impl Script {
    fn new(replies: Vec<(CloudEndpoint, Value)>) -> Self {
        Self {
            replies: Mutex::new(
                replies
                    .into_iter()
                    .map(|(e, v)| (e, Reply::Body(v)))
                    .collect(),
            ),
            calls: Mutex::default(),
        }
    }

    fn lookup(response: Value) -> Self {
        Self::new(vec![
            (CloudEndpoint::Homes, homes(&["19"])),
            (
                CloudEndpoint::HomeAppliances,
                listing("19", vec![appliance(ID, "AC")]),
            ),
            (CloudEndpoint::LanCredentials, response),
        ])
    }

    fn count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }
}

#[async_trait]
impl CloudTransport for Script {
    async fn send(&self, request: &SignedRequest, _: &Operation) -> Result<SecretBytes> {
        assert_eq!(request.token(), Some(b"PRIVATE-LOGIN-TOKEN".as_slice()));
        assert_eq!(format!("{request:?}"), "SignedRequest([REDACTED])");
        self.calls.lock().unwrap().push((
            request.endpoint(),
            serde_json::from_slice(request.body()).unwrap(),
        ));
        let (expected, reply) = self
            .replies
            .lock()
            .unwrap()
            .pop_front()
            .expect("No extra request");
        assert_eq!(request.endpoint(), expected);
        match reply {
            Reply::Body(value) => Ok(SecretBytes::new(serde_json::to_vec(&value).unwrap())),
            Reply::Error => Err(Error::new(
                "TRANSPORT_PRIVATE",
                "PRIVATE-TOKEN-AND-ENDPOINT",
            )),
            Reply::Wait => std::future::pending().await,
        }
    }
}

fn response(data: Value) -> Value {
    json!({"code":0, "data":data})
}

fn homes(ids: &[&str]) -> Value {
    response(json!({"homeList":ids.iter().map(|id| json!({"homegroupId":id})).collect::<Vec<_>>()}))
}

fn listing(home: &str, appliances: Vec<Value>) -> Value {
    response(json!({"homeList":[{"homegroupId":home,"roomList":[{"applianceList":appliances}]}]}))
}

fn vector() -> Value {
    serde_json::from_str(include_str!("../fixtures/lan-v3.json")).unwrap()
}

fn token(udp_id: &str) -> Value {
    let v = vector();
    json!({"udpId":udp_id,"token":v["token"],"key":v["key"]})
}

fn keys(udp_id: &str) -> Value {
    response(json!({"tokenlist":[token(udp_id)]}))
}

fn bytes(value: &Value) -> Vec<u8> {
    let hex = value.as_str().unwrap();
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

async fn connected(f: &Fixture) -> (MeijuAccount, DeviceBinding) {
    let report = connect(
        &f.vault,
        &LoginTransport::complete("PRIVATE-LOGIN-ID"),
        login_credentials(),
        None,
        "Home",
        &operation(),
    )
    .await
    .unwrap();
    let binding = report.retrieval.unwrap().bindings[0];
    (f.reopen(report.account.id).await.unwrap(), binding)
}

#[tokio::test(start_paused = true)]
async fn selected_binding_resolves_unique_home_and_one_method_without_persisting_keys() {
    for (method, udp) in [
        (LanUdpIdMethod::BigEndian, BIG),
        (LanUdpIdMethod::LittleEndian, LITTLE),
    ] {
        let f = Fixture::new();
        let (account, binding) = connected(&f).await;
        let original = f.bytes(account.record().credential_ref);
        let puts = f.puts();
        let cloud = Script::new(vec![
            (CloudEndpoint::Homes, homes(&["19", "20"])),
            (
                CloudEndpoint::HomeAppliances,
                listing("19", vec![appliance("22", "CA")]),
            ),
            (
                CloudEndpoint::HomeAppliances,
                listing("20", vec![appliance(ID, "AC")]),
            ),
            (
                CloudEndpoint::LanCredentials,
                response(json!({"tokenlist":[token("foreign"), token(&udp.to_uppercase())]})),
            ),
        ]);
        let handshake = account
            .lan_v3_handshake(&cloud, binding, method, &operation())
            .await
            .unwrap();
        assert_eq!(format!("{handshake:?}"), "V3Handshake([REDACTED])");
        assert_eq!(handshake.request().expose(), bytes(&vector()["request"]));
        handshake.finish(&bytes(&vector()["response"])).unwrap();
        let calls = cloud.calls.lock().unwrap();
        assert_eq!(calls.len(), 4);
        assert_eq!(calls[1].1["homegroupId"], 19);
        assert_eq!(calls[2].1["homegroupId"], 20);
        let request = &calls[3].1;
        assert_eq!(request["homegroupId"], "20");
        assert_eq!(request["applianceCodes"], json!([ID]));
        assert_eq!(request["udpid"], udp);
        assert_eq!(request["deviceId"], CLIENT_ID);
        for (field, value) in [
            ("src", "900"),
            ("appId", "900"),
            ("format", "2"),
            ("platformId", "1"),
            ("clientType", "1"),
            ("language", "en_US"),
        ] {
            assert_eq!(request[field], value);
        }
        assert!(request["uid"].is_null());
        assert_eq!(request.as_object().unwrap().len(), 13);
        assert_eq!(
            CloudEndpoint::LanCredentials.path(),
            "/v2/iot/secure/getToken"
        );
        assert_eq!(f.bytes(account.record().credential_ref), original);
        assert_eq!(f.puts(), puts);
    }
}

#[tokio::test(start_paused = true)]
async fn legacy_cloud_sessions_remain_usable_and_explicit_reauthorization_preserves_bindings() {
    let f = Fixture::new();
    let (mut account, binding) = connected(&f).await;
    let record = account.record().clone();
    let mut payload: Value = serde_json::from_slice(&f.bytes(record.credential_ref)).unwrap();
    let raw: Vec<u8> = serde_json::from_value(payload["session"].clone()).unwrap();
    let mut session: Value = serde_json::from_slice(&raw).unwrap();
    session["version"] = json!(1);
    session.as_object_mut().unwrap().remove("client_device_id");
    payload["session"] = json!(serde_json::to_vec(&session).unwrap());
    f.set_bytes(record.credential_ref, serde_json::to_vec(&payload).unwrap());
    let original = f.bytes(record.credential_ref);
    let puts = f.puts();
    let cloud = Script::new(vec![]);
    assert_eq!(
        account
            .lan_v3_handshake(&cloud, binding, LanUdpIdMethod::BigEndian, &operation())
            .await
            .unwrap_err()
            .code,
        "AUTH_REQUIRED"
    );
    assert_eq!(cloud.count(), 0);
    let cloud = Script::new(vec![
        (CloudEndpoint::Homes, homes(&["19"])),
        (
            CloudEndpoint::HomeAppliances,
            listing("19", vec![appliance(ID, "AC"), appliance("22", "CA")]),
        ),
    ]);
    let inventory = account.retrieve(&cloud, &operation()).await.unwrap();
    assert!(inventory.inventory.is_complete());
    assert_eq!(inventory.bindings[0], binding);
    assert_eq!(f.bytes(record.credential_ref), original);
    assert_eq!(f.puts(), puts);
    drop(account);
    let report = connect(
        &f.vault,
        &LoginTransport::complete("PRIVATE-LOGIN-ID"),
        login_credentials(),
        Some(record.id),
        "Home",
        &operation(),
    )
    .await
    .unwrap();
    assert_eq!(report.account.id, record.id);
    assert_eq!(report.retrieval.unwrap().bindings[0], binding);
    let updated: Value = serde_json::from_slice(&f.bytes(record.credential_ref)).unwrap();
    assert_eq!(updated["bindings"], payload["bindings"]);
    let raw: Vec<u8> = serde_json::from_value(updated["session"].clone()).unwrap();
    let updated_session: Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(updated_session["version"], 2);
    assert_eq!(updated_session["client_device_id"], CLIENT_ID);
    for entry in std::fs::read_dir(&f.directory.0).unwrap().flatten() {
        if entry.path().is_file() {
            let bytes = std::fs::read(entry.path()).unwrap();
            assert!(
                !bytes
                    .windows(CLIENT_ID.len())
                    .any(|value| value == CLIENT_ID.as_bytes())
            );
        }
    }
    assert!(
        !serde_json::to_string(&report.account)
            .unwrap()
            .contains(CLIENT_ID)
    );
}

#[tokio::test(start_paused = true)]
async fn protected_binding_membership_session_and_scope_fail_before_cloud_access() {
    for case in [
        "device",
        "connection",
        "identity",
        "unlisted",
        "legacy",
        "subject",
        "session",
        "family",
        "wide",
    ] {
        let f = Fixture::new();
        let (account, mut binding) = connected(&f).await;
        let reference = account.record().credential_ref;
        let mut payload: Value = serde_json::from_slice(&f.bytes(reference)).unwrap();
        let expected = match case {
            "device" => {
                binding.device_id = Uuid::new_v4();
                "DEVICE_IDENTITY_CONFLICT"
            }
            "connection" => {
                binding.connection_id = Uuid::new_v4();
                "DEVICE_IDENTITY_CONFLICT"
            }
            "identity" => {
                binding.identity_ref = Uuid::new_v4();
                "DEVICE_IDENTITY_CONFLICT"
            }
            "unlisted" => {
                payload["bindings"][0]["listed"] = json!(false);
                "MIDEA_DEVICE_NOT_LISTED"
            }
            "legacy" => {
                payload["bindings"][0]["listed"] = Value::Null;
                "MIDEA_INVENTORY_REQUIRED"
            }
            "subject" => {
                payload["subject"] = json!(b"foreign-subject".to_vec());
                "ACCOUNT_SESSION_UNCONFIRMED"
            }
            "session" => {
                payload["session"] = json!(b"invalid".to_vec());
                "ACCOUNT_SESSION_UNCONFIRMED"
            }
            "family" => {
                payload["bindings"][0]["family"] = json!(0xff);
                "CAPABILITY_UNSUPPORTED"
            }
            "wide" => {
                payload["bindings"][0]["appliance_id"] = json!("281474976710656");
                "CAPABILITY_UNSUPPORTED"
            }
            _ => unreachable!(),
        };
        f.set_bytes(reference, serde_json::to_vec(&payload).unwrap());
        let cloud = Script::new(vec![]);
        let original = f.bytes(reference);
        let error = account
            .lan_v3_handshake(&cloud, binding, LanUdpIdMethod::BigEndian, &operation())
            .await
            .unwrap_err();
        assert_eq!(error.code, expected, "{case}");
        assert_eq!(cloud.count(), 0);
        assert_eq!(f.bytes(reference), original);
    }
}

#[tokio::test(start_paused = true)]
async fn foreign_account_region_and_unavailable_secret_never_request_keys() {
    let f = Fixture::new();
    let (account, binding) = connected(&f).await;
    let reference = account.record().credential_ref;
    let other = f.account().await;
    let cloud = Script::new(vec![]);
    assert_eq!(
        other
            .lan_v3_handshake(&cloud, binding, LanUdpIdMethod::BigEndian, &operation())
            .await
            .unwrap_err()
            .code,
        "DEVICE_IDENTITY_CONFLICT"
    );
    let foreign = MeijuAccount::create(
        &f.vault,
        "EU",
        "Other",
        &secret("subject"),
        &secret("session"),
        &operation(),
    )
    .await
    .unwrap();
    assert_eq!(
        foreign
            .lan_v3_handshake(&cloud, binding, LanUdpIdMethod::BigEndian, &operation())
            .await
            .unwrap_err()
            .code,
        "AUTH_FLOW_UNSUPPORTED"
    );
    f.secrets
        .values
        .lock()
        .unwrap()
        .remove(&reference.to_string());
    assert!(
        account
            .lan_v3_handshake(&cloud, binding, LanUdpIdMethod::BigEndian, &operation())
            .await
            .is_err()
    );
    assert_eq!(cloud.count(), 0);
}

#[tokio::test(start_paused = true)]
async fn fresh_inventory_requires_unique_consistent_target_and_complete_home_reads() {
    for case in [
        "absent",
        "duplicate",
        "two-homes",
        "model",
        "missing-model",
        "family",
        "profile",
        "wrong-home",
        "partial",
        "expired",
        "empty",
        "too-many",
    ] {
        let f = Fixture::new();
        let (account, binding) = connected(&f).await;
        let reference = account.record().credential_ref;
        let mut target = appliance(ID, "AC");
        let mut entries = vec![target.clone()];
        let mut expected = "DEVICE_IDENTITY_CONFLICT";
        match case {
            "absent" => {
                entries.clear();
                expected = "MIDEA_DEVICE_NOT_LISTED";
            }
            "duplicate" => entries.push(target.clone()),
            "model" => {
                target["productModel"] = json!("Changed");
                entries = vec![target];
            }
            "missing-model" => {
                target.as_object_mut().unwrap().remove("productModel");
                entries = vec![target];
            }
            "family" => {
                target["type"] = json!("E1");
                entries = vec![target];
            }
            "profile" => {
                let mut payload: Value = serde_json::from_slice(&f.bytes(reference)).unwrap();
                payload["bindings"][0]["product_profile"] =
                    json!({"sn8":"OLD12345","subtype":null,"category":null});
                f.set_bytes(reference, serde_json::to_vec(&payload).unwrap());
                target["sn8"] = json!("NEW12345");
                entries = vec![target];
            }
            "wrong-home" | "too-many" => expected = "MIDEA_RESPONSE_INVALID",
            "partial" => expected = "MIDEA_REQUEST_REJECTED",
            "expired" => expected = "AUTH_REQUIRED",
            "empty" => expected = "MIDEA_DEVICE_NOT_LISTED",
            _ => {}
        }
        let mut steps = vec![
            (CloudEndpoint::Homes, homes(&["19", "20"])),
            (CloudEndpoint::HomeAppliances, listing("19", entries)),
        ];
        match case {
            "wrong-home" => steps[1].1 = listing("20", vec![appliance(ID, "AC")]),
            "partial" | "expired" => steps.push((CloudEndpoint::HomeAppliances, json!({"code":if case == "expired" {7400} else {999},"msg":"PRIVATE-PROVIDER-MESSAGE"}))),
            "empty" => steps = vec![(CloudEndpoint::Homes, homes(&[]))],
            "too-many" => steps = vec![(CloudEndpoint::Homes, response(json!({"homeList":(1..=33).map(|id| json!({"homegroupId":id})).collect::<Vec<_>>()})))],
            _ => steps.push((CloudEndpoint::HomeAppliances, listing("20", if case == "two-homes" { vec![appliance(ID, "AC")] } else { vec![] }))),
        }
        let cloud = Script::new(steps);
        let original = f.bytes(reference);
        let error = account
            .lan_v3_handshake(&cloud, binding, LanUdpIdMethod::BigEndian, &operation())
            .await
            .unwrap_err();
        assert_eq!(error.code, expected, "{case}");
        assert!(!error.message.contains("PRIVATE"));
        assert!(
            cloud
                .calls
                .lock()
                .unwrap()
                .iter()
                .all(|(e, _)| *e != CloudEndpoint::LanCredentials)
        );
        assert_eq!(f.bytes(reference), original);
    }
}

#[tokio::test(start_paused = true)]
async fn credentials_reject_missing_ambiguous_malformed_and_oversized_payloads_without_fallback() {
    for case in [
        "empty",
        "foreign",
        "duplicate",
        "short-token",
        "long-token",
        "short-key",
        "hex",
        "unicode",
        "missing",
        "too-many",
        "oversize",
        "expired",
        "retired",
        "rejected",
    ] {
        let f = Fixture::new();
        let (account, binding) = connected(&f).await;
        let mut body = keys(BIG);
        let mut expected = "MIDEA_RESPONSE_INVALID";
        match case {
            "empty" => {
                body["data"]["tokenlist"] = json!([]);
                expected = "AUTH_REQUIRED";
            }
            "foreign" => {
                body = keys(LITTLE);
                expected = "AUTH_REQUIRED";
            }
            "duplicate" => body["data"]["tokenlist"] = json!([token(BIG), token(BIG)]),
            "short-token" => body["data"]["tokenlist"][0]["token"] = json!("ab".repeat(63)),
            "long-token" => body["data"]["tokenlist"][0]["token"] = json!("ab".repeat(65)),
            "short-key" => body["data"]["tokenlist"][0]["key"] = json!("ab".repeat(31)),
            "hex" => body["data"]["tokenlist"][0]["key"] = json!("zz".repeat(32)),
            "unicode" => body["data"]["tokenlist"][0]["key"] = json!("\u{e9}".repeat(32)),
            "missing" => body["data"] = json!({}),
            "too-many" => body["data"]["tokenlist"] = json!(vec![token(BIG); 65]),
            "oversize" => body["extra"] = json!("x".repeat(MAX_RESPONSE_BYTES)),
            "expired" => {
                body = json!({"code":"40002","msg":"PRIVATE"});
                expected = "AUTH_REQUIRED";
            }
            "retired" | "rejected" => {
                body = json!({"code":if case == "retired" {40404} else {1002},"msg":"PRIVATE"});
                expected = "MIDEA_REQUEST_REJECTED";
            }
            _ => unreachable!(),
        }
        let cloud = Script::lookup(body);
        let original = f.bytes(account.record().credential_ref);
        let error = account
            .lan_v3_handshake(&cloud, binding, LanUdpIdMethod::BigEndian, &operation())
            .await
            .unwrap_err();
        assert_eq!(error.code, expected, "{case}");
        assert!(!error.message.contains("PRIVATE"));
        assert_eq!(cloud.count(), 3);
        assert_eq!(f.bytes(account.record().credential_ref), original);
    }
}

#[tokio::test(start_paused = true)]
async fn transport_failure_is_sanitized_at_every_step_without_extra_requests() {
    let f = Fixture::new();
    let (account, binding) = connected(&f).await;
    for step in 0..3 {
        let cloud = Script::lookup(keys(BIG));
        cloud.replies.lock().unwrap()[step].1 = Reply::Error;
        let error = account
            .lan_v3_handshake(&cloud, binding, LanUdpIdMethod::BigEndian, &operation())
            .await
            .unwrap_err();
        assert_eq!(error.code, "MIDEA_REQUEST_FAILED");
        assert!(!error.message.contains("PRIVATE"));
        assert_eq!(cloud.count(), step + 1);
    }
}

#[tokio::test(start_paused = true)]
async fn cancellation_deadline_and_future_drop_stop_the_shared_operation() {
    let f = Fixture::new();
    let (account, binding) = connected(&f).await;
    let original = f.bytes(account.record().credential_ref);
    for step in 0..3 {
        for stop in ["cancel", "deadline", "drop"] {
            let cloud = Script::lookup(keys(BIG));
            cloud.replies.lock().unwrap()[step].1 = Reply::Wait;
            let op = operation();
            let future = account.lan_v3_handshake(&cloud, binding, LanUdpIdMethod::BigEndian, &op);
            let mut future = Box::pin(future);
            let ready = async {
                while cloud.count() <= step {
                    tokio::task::yield_now().await;
                }
            };
            tokio::select! { _ = &mut future => panic!("Script must wait"), () = ready => {} }
            if stop == "drop" {
                drop(future);
            } else {
                if stop == "cancel" {
                    op.cancellation.cancel();
                } else {
                    tokio::time::advance(std::time::Duration::from_secs(2)).await;
                }
                assert_eq!(
                    future.await.unwrap_err().code,
                    if stop == "cancel" {
                        "CANCELLED"
                    } else {
                        "DEADLINE_EXCEEDED"
                    }
                );
            }
            assert_eq!(cloud.count(), step + 1);
            assert_eq!(f.bytes(account.record().credential_ref), original);
        }
    }
    for deadline in [false, true] {
        let op = operation();
        if deadline {
            tokio::time::advance(std::time::Duration::from_secs(2)).await;
        } else {
            op.cancellation.cancel();
        }
        let cloud = Script::new(vec![]);
        assert_eq!(
            account
                .lan_v3_handshake(&cloud, binding, LanUdpIdMethod::BigEndian, &op)
                .await
                .unwrap_err()
                .code,
            if deadline {
                "DEADLINE_EXCEEDED"
            } else {
                "CANCELLED"
            }
        );
        assert_eq!(cloud.count(), 0);
    }
    let id = account.record().id;
    drop(account);
    assert!(f.vault.lease(id, &operation()).await.is_ok());
}
