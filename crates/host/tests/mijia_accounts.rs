use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD};
use homeverb_core::*;
use homeverb_domain::Result;
use homeverb_host::{ConnectRequest, HostRequest, Session};
use homeverb_mijia::{CloudEndpoint, CloudRequest, CloudResponse, CloudTransport, MijiaAdapter};
use homeverb_storage::SqliteStore;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

fn op() -> Operation {
    Operation::new(10000, CancellationToken::new()).unwrap()
}
fn secret(bytes: &[u8]) -> SecretBytes {
    SecretBytes::new(bytes.to_vec())
}
#[derive(Default)]
struct Secrets(Mutex<HashMap<String, Vec<u8>>>);
#[async_trait]
impl SecretStore for Secrets {
    async fn get(&self, key: &CredentialRef, op: &Operation) -> Result<Option<SecretBytes>> {
        op.run(async {
            Ok(self
                .0
                .lock()
                .unwrap()
                .get(&key.to_string())
                .map(|v| secret(v)))
        })
        .await
    }
    async fn put(&self, key: &CredentialRef, value: &SecretBytes, op: &Operation) -> Result<()> {
        op.run(async {
            self.0
                .lock()
                .unwrap()
                .insert(key.to_string(), value.expose().to_vec());
            Ok(())
        })
        .await
    }
    async fn delete(&self, key: &CredentialRef, op: &Operation) -> Result<()> {
        op.run(async {
            self.0.lock().unwrap().remove(&key.to_string());
            Ok(())
        })
        .await
    }
}
struct NoMeijuTraffic;
#[async_trait]
impl homeverb_midea::CloudTransport for NoMeijuTraffic {
    async fn send(&self, _: &homeverb_midea::SignedRequest, _: &Operation) -> Result<SecretBytes> {
        panic!("Mijia operations must not contact Meiju");
    }
}
// Fictional cloud peer implements the legacy stream cipher without invoking adapter internals.
fn stream(key: &[u8], bytes: &[u8]) -> Vec<u8> {
    let mut table: Vec<u8> = (0..=255).collect();
    let mut cursor = 0;
    for position in 0..256 {
        cursor =
            (cursor + usize::from(table[position]) + usize::from(key[position % key.len()])) % 256;
        table.swap(position, cursor);
    }
    let (mut left, mut right) = (0, 0);
    let mut output = Vec::new();
    for byte in std::iter::repeat_n(0, 1024).chain(bytes.iter().copied()) {
        left = (left + 1) % 256;
        right = (right + usize::from(table[left])) % 256;
        table.swap(left, right);
        output.push(byte ^ table[(usize::from(table[left]) + usize::from(table[right])) % 256]);
    }
    output.split_off(1024)
}
const KIND: &str = "urn:miot-spec-v2:device:fan:0000A005:fiction-native:1";
#[derive(Default)]
struct Cloud {
    calls: AtomicUsize,
    empty: AtomicBool,
    outlet: AtomicBool,
    auxiliary: AtomicUsize,
    writes: AtomicUsize,
    lose_write: AtomicBool,
    stall: AtomicBool,
    captcha: AtomicBool,
    started: tokio::sync::Notify,
}
#[async_trait]
impl CloudTransport for Cloud {
    async fn send_control(
        &self,
        request: &CloudRequest,
        progress: &WriteProgress,
        operation: &Operation,
    ) -> Result<CloudResponse> {
        operation
            .run(async {
                assert_eq!(request.endpoint(), CloudEndpoint::SetProperty);
                let fields: HashMap<_, _> = url::form_urlencoded::parse(request.body())
                    .into_owned()
                    .collect();
                let key = Sha256::digest(
                    [
                        STANDARD.decode(&fields["ssecurity"]).unwrap(),
                        STANDARD.decode(&fields["_nonce"]).unwrap(),
                    ]
                    .concat(),
                );
                let data: Value = serde_json::from_slice(&stream(
                    &key,
                    &STANDARD.decode(&fields["data"]).unwrap(),
                ))
                .unwrap();
                assert_eq!(data["params"].as_array().unwrap().len(), 1);
                let field = &data["params"][0];
                assert_eq!(
                    field,
                    &json!({"did":"PRIVATE-FICTIONAL-OUTLET","siid":2,"piid":1,"value":true})
                );
                self.writes.fetch_add(1, Ordering::SeqCst);
                progress.mark_sent();
                if self.lose_write.load(Ordering::SeqCst) {
                    return Err(homeverb_domain::Error::new(
                        "NETWORK_ERROR",
                        "Fictional lost acknowledgement.",
                    ));
                }
                let reply =
                    json!({"code":0,"result":[{"did":field["did"],"siid":2,"piid":1,"code":0}]});
                Ok(CloudResponse {
                    body: secret(
                        STANDARD
                            .encode(stream(&key, &serde_json::to_vec(&reply).unwrap()))
                            .as_bytes(),
                    ),
                    service_token: None,
                })
            })
            .await
    }

    async fn send_verification(
        &self,
        request: &homeverb_mijia::VerificationRequest,
        op: &Operation,
    ) -> Result<homeverb_mijia::VerificationResponse> {
        op.run(async {
            use homeverb_mijia::VerificationEndpoint as E;
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.stall.load(Ordering::SeqCst) { self.started.notify_one(); std::future::pending::<()>().await; }
            let (body, cookies): (Value, Vec<&str>) = match request.endpoint() {
                E::SigningKey if !request.cookie().contains("passToken=") => (json!({"_sign":"fictional"}), vec![]),
                E::Password if self.captcha.load(Ordering::SeqCst) && !request.body().windows(16).any(|b| b == b"captCode=correct") => (json!({"code":87001,"captchaUrl":"/pass/getCode?fictional"}), vec![]),
                E::Password => (json!({"code":0,"notificationUrl":"https://account.xiaomi.com/fe/service/identity/authStart?context=PRIVATE-FICTIONAL-CONTEXT"}), vec![]),
                E::Options => (json!({"code":0,"options":[4,8]}), vec!["identity_session=PRIVATE-FICTIONAL-SESSION; Path=/identity"]),
                E::PhoneInfo | E::EmailInfo => (json!({"code":0, "contentType":"160010"}), vec![]),
                E::Quota => (json!({"code":0,"result":0,"info":"3"}), vec![]),
                E::SendPhone | E::SendEmail => (json!({"code":0,"data":{"wt":60}}), vec![]),
                E::Phone | E::Email => (json!({"code":0,"location":"/identity/result/check?fictional"}), vec![]),
                E::Completion => (json!({}), vec!["passToken=PRIVATE-FICTIONAL-PASS; Path=/", "serviceToken=PRIVATE-VERIFICATION-TOKEN; Path=/", "userId=999001; Path=/"]),
                E::SigningKey => (json!({"code":0,"userId":"999001","ssecurity":"AAECAwQFBgcICQoLDA0ODw=="}), vec![]),
                _ => panic!("Unexpected verification request"),
            };
            homeverb_mijia::VerificationResponse::new(200, SecretBytes::new(serde_json::to_vec(&body).unwrap()), None, cookies.into_iter().map(|v| secret(v.as_bytes())).collect())
        }).await
    }

    async fn fetch_captcha(
        &self,
        _: &homeverb_mijia::CaptchaRequest,
        _: &Operation,
    ) -> Result<homeverb_mijia::CaptchaResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let image = STANDARD.decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jP1sAAAAASUVORK5CYII=").unwrap();
        homeverb_mijia::CaptchaResponse::new(SecretBytes::new(image), secret(b"fictional-ick"))
    }

    async fn send(&self, request: &CloudRequest, op: &Operation) -> Result<CloudResponse> {
        op.run(async {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.stall.load(Ordering::SeqCst) {
                self.started.notify_one(); std::future::pending::<()>().await;
            }
            let raw = match request.endpoint() {
                CloudEndpoint::LoginSign => Some(json!({"_sign":"fictional"})),
                CloudEndpoint::LoginPassword if self.captcha.load(Ordering::SeqCst) && !request.body().windows(16).any(|b| b == b"captCode=correct") => Some(json!({"code":87001,"captchaUrl":"/pass/getCode?fictional"})),
                CloudEndpoint::LoginPassword => Some(json!({"code":0,"userId":"999001","ssecurity":"AAECAwQFBgcICQoLDA0ODw==","location":"https://sts.api.io.mi.com/sts?ticket=fictional"})),
                CloudEndpoint::LoginExchange => return Ok(CloudResponse { body: secret(b""), service_token: Some(secret(b"PRIVATE-NATIVE-TOKEN")) }),
                CloudEndpoint::SpecIndex => Some(json!({"instances":[{"model":"fiction.fan.native","type":KIND,"status":"released","version":1}]})),
                CloudEndpoint::SpecInstance => Some(json!({"type":KIND,"services":[{"iid":2,"type":"urn:miot-spec-v2:service:fan:00007808:fiction-native:1","properties":[{"iid":1,"type":"urn:miot-spec-v2:property:on:00000006:fiction-native:1","format":"bool","access":["read","write"]}]}]})),
                _ => None,
            };
            if let Some(raw) = raw { return Ok(CloudResponse { body: SecretBytes::new(serde_json::to_vec(&raw).unwrap()), service_token: None }); }
            let fields: HashMap<_, _> = url::form_urlencoded::parse(request.body()).into_owned().collect();
            let key = Sha256::digest([STANDARD.decode(&fields["ssecurity"]).unwrap(), STANDARD.decode(&fields["_nonce"]).unwrap()].concat());
            let data: Value = serde_json::from_slice(&stream(&key, &STANDARD.decode(&fields["data"]).unwrap())).unwrap();
            let result = match request.endpoint() {
                CloudEndpoint::Devices if self.auxiliary.load(Ordering::SeqCst) != 0 => json!({"list":[{"did":"PRIVATE-FICTIONAL-AUXILIARY","name":"Fictional auxiliary","model":if self.auxiliary.load(Ordering::SeqCst)==1 {"xiaomi.hood.cyp10"} else {"jdjz.fan.2072fc"}}]}),
                CloudEndpoint::Devices if self.outlet.load(Ordering::SeqCst) => json!({"list":[{"did":"PRIVATE-FICTIONAL-OUTLET","name":"Fictional outlet","model":"cuco.plug.v3"}]}),
                CloudEndpoint::Devices => json!({"list":if self.empty.load(Ordering::SeqCst) {vec![]} else {vec![json!({"did":"PRIVATE-FAN-ID","name":"Desk Fan","model":"fiction.fan.native","spec_type":KIND,"isOnline":true}), json!({"did":"PRIVATE-OTHER-ID","name":"Hall Appliance","model":"fiction.unknown.native","isOnline":true})]}}),
                CloudEndpoint::Homes => json!({"homelist":[],"has_more":false}),
                CloudEndpoint::Properties => json!(data["params"].as_array().unwrap().iter().map(|p| json!({"did":p["did"],"siid":p["siid"],"piid":p["piid"],"code":0,"value":false})).collect::<Vec<_>>()),
                _ => panic!("Unexpected fictional endpoint"),
            };
            Ok(CloudResponse { body: secret(STANDARD.encode(stream(&key, &serde_json::to_vec(&json!({"code":0,"result":result})).unwrap())).as_bytes()), service_token: None })
        }).await
    }
}
struct Fixture {
    dir: PathBuf,
    session: Session,
    cloud: Arc<Cloud>,
    vault: Arc<AccountVault>,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}
impl Fixture {
    fn new(combined: bool) -> Self {
        let dir = std::env::temp_dir().join(format!("homeverb-native-mijia-{}", Uuid::new_v4()));
        let store = Arc::new(SqliteStore::open(&dir).unwrap());
        let vault = Arc::new(AccountVault::new(
            store.clone(),
            Arc::new(Secrets::default()),
        ));
        let cloud = Arc::new(Cloud::default());
        let clock = Arc::new(SystemClock);
        let mijia = Arc::new(MijiaAdapter::new(
            vault.clone(),
            cloud.clone(),
            clock.clone(),
        ));
        let meiju = combined.then(|| {
            Arc::new(homeverb_midea::MeijuAdapter::new(
                vault.clone(),
                Arc::new(NoMeijuTraffic),
                clock.clone(),
            ))
        });
        let mut adapters: Vec<Arc<dyn Adapter>> = vec![mijia.clone()];
        if let Some(a) = &meiju {
            adapters.push(a.clone());
        }
        let core = Core::new(store, adapters, clock, Policy::default());
        Self {
            dir,
            session: Session::with_platform_accounts(core, meiju, Some(mijia)),
            cloud,
            vault,
        }
    }
    async fn connect(&self, provider: Option<&str>, id: Option<Uuid>) -> Value {
        let metadata: ConnectRequest = serde_json::from_value(json!({"schema_version":"1.0","operation_id":"connect","timeout_ms":10000,"label":"Mijia Home","account_id":id,"provider":provider})).unwrap();
        let result = self
            .session
            .connect_account(
                metadata,
                secret(b"fiction@example.invalid"),
                secret(b"fictional-password"),
            )
            .await;
        validate(&result, "connect_account");
        result
    }
    async fn call(&self, request: Value) -> Value {
        let method = request["method"].as_str().unwrap().to_string();
        let input: HostRequest = serde_json::from_value(json!({"schema_version":"1.0","operation_id":"query","timeout_ms":10000,"request":request})).unwrap();
        let result = self.session.call(input).await;
        validate(&result, &method);
        result
    }
}
fn validate(value: &Value, definition: &str) {
    fn expand(value: &Value, root: &Value) -> Value {
        if let Some(reference) = value.get("$ref").and_then(Value::as_str) {
            if let Some(pointer) = reference.strip_prefix('#') {
                return expand(root.pointer(pointer).unwrap(), root);
            }
            let name = reference.strip_prefix("urn:homeverb:schema:").unwrap();
            let (name, pointer) = name.split_once('#').unwrap_or((name, ""));
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../schemas")
                .join(format!("{name}.schema.json"));
            let referenced: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
            return expand(referenced.pointer(pointer).unwrap(), &referenced);
        }
        match value {
            Value::Object(map) => Value::Object(
                map.iter()
                    .map(|(k, v)| (k.clone(), expand(v, root)))
                    .collect(),
            ),
            Value::Array(items) => Value::Array(items.iter().map(|v| expand(v, root)).collect()),
            _ => value.clone(),
        }
    }
    let root: Value =
        serde_json::from_str(include_str!("../../../schemas/native-response.schema.json")).unwrap();
    jsonschema::validator_for(&expand(&root["$defs"][definition], &root))
        .unwrap()
        .validate(value)
        .unwrap();
}

#[tokio::test]
async fn protected_connect_publishes_partial_named_cards_and_reads_actual_power() {
    let f = Fixture::new(false);
    let connected = f.connect(None, None).await;
    assert!(connected["error"].is_null(), "{connected}");
    let account = connected["data"]["account"]["id"].clone();
    assert_eq!(connected["data"]["account"]["provider"], "xiaomi.mijia");
    assert_eq!(connected["data"]["retrieval"]["status"], "partial");
    assert_eq!(
        connected["data"]["retrieval"]["device_ids"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let cards = f.call(json!({"method":"list_devices"})).await;
    let fan = cards["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["alias"] == "Desk Fan")
        .unwrap();
    let other = cards["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["alias"] == "Hall Appliance")
        .unwrap();
    assert_eq!(other["support_status"], "unsupported");
    let placeholder = f
        .call(json!({"method":"describe_device","device":other["id"]}))
        .await;
    assert!(
        placeholder["data"]["capabilities"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["actions"].as_object().unwrap().is_empty()
                && c["properties"].as_object().unwrap().is_empty())
    );
    let component = fan["components"][0]["id"].clone();
    let state = f.call(json!({"method":"read_state","device":fan["id"],"component":component,"connection":"cloud","cached":false})).await;
    assert!(state["error"].is_null(), "{state}");
    assert_eq!(state["data"]["observations"]["power.state"]["value"], false);
    let calls = f.cloud.calls.load(Ordering::SeqCst);
    assert_eq!(
        f.call(json!({"method":"discover"})).await["error"]["code"],
        "MIJIA_INVENTORY_PARTIAL"
    );
    assert_eq!(f.cloud.calls.load(Ordering::SeqCst), calls);
    let rename = f
        .call(json!({"method":"rename_account","account_id":account,"label":"Family Mijia"}))
        .await;
    assert_eq!(rename["data"]["label"], "Family Mijia");
    f.cloud.empty.store(true, Ordering::SeqCst);
    let refreshed = f
        .call(json!({"method":"refresh_account","account_id":account}))
        .await;
    assert_eq!(refreshed["data"]["retrieval"]["status"], "partial");
    assert_eq!(
        f.call(json!({"method":"list_devices"})).await["data"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let inventory = f
        .call(json!({"method":"inspect_account_inventory","account_id":account}))
        .await;
    assert!(
        inventory["data"]["devices"]
            .as_array()
            .unwrap()
            .iter()
            .all(|d| d["membership"] == "unconfirmed")
    );
    let serialized = format!("{connected}{cards}{state}{refreshed}{inventory}");
    for marker in ["PRIVATE-", "999001", "ssecurity", "credential_ref"] {
        assert!(!serialized.contains(marker));
    }
}

#[tokio::test]
async fn combined_host_routes_by_saved_provider_and_replays_local_removal() {
    let f = Fixture::new(true);
    let meiju = f
        .vault
        .prepare("midea.meiju", "CN", "Meiju Home", &op())
        .await
        .unwrap()
        .record()
        .id;
    let foreign = f
        .vault
        .prepare("other.provider", "CN", "Foreign", &op())
        .await
        .unwrap()
        .record()
        .id;
    let connected = f.connect(Some("xiaomi.mijia"), None).await;
    assert!(connected["error"].is_null(), "{connected}");
    let account = connected["data"]["account"]["id"].clone();
    let accounts = f.call(json!({"method":"list_accounts"})).await;
    assert_eq!(accounts["data"].as_array().unwrap().len(), 2);
    assert_eq!(
        f.call(json!({"method":"get_account","account_id":foreign}))
            .await["error"]["code"],
        "ACCOUNT_NOT_FOUND"
    );
    assert_eq!(
        f.call(json!({"method":"get_account","account_id":meiju}))
            .await["data"]["account"]["provider"],
        "midea.meiju"
    );
    let calls = f.cloud.calls.load(Ordering::SeqCst);
    assert_eq!(
        f.connect(Some("xiaomi.mijia"), Some(meiju)).await["error"]["code"],
        "ACCOUNT_NOT_FOUND"
    );
    assert_eq!(f.cloud.calls.load(Ordering::SeqCst), calls);
    assert_eq!(f.call(json!({"method":"inspect_device_conflicts","account_id":account,"device_id":Uuid::new_v4()})).await["error"]["code"],"CAPABILITY_UNSUPPORTED");
    let devices = f.call(json!({"method":"list_devices"})).await;
    let fan = devices["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["alias"] == "Desk Fan")
        .unwrap();
    let inspected = f.call(json!({"method":"inspect_control_options","device_id":fan["id"],"connection_id":fan["connections"][0]["id"]})).await;
    assert!(inspected["error"].is_null());
    assert_eq!(inspected["data"]["authorization"], "not_checked");
    assert_eq!(inspected["data"]["source"], "saved_inventory");
    assert_eq!(inspected["data"]["options"][0]["component_id"], "fan-2");
    assert_eq!(inspected["data"]["options"][0]["action"], "set");
    assert_eq!(f.cloud.calls.load(Ordering::SeqCst), calls);
    let removal = json!({"method":"remove_account","account_id":account,"expected_device_ids":connected["data"]["retrieval"]["device_ids"]});
    assert_eq!(f.call(removal.clone()).await["data"]["status"], "removed");
    assert_eq!(f.call(removal).await["data"]["status"], "removed");
    assert_eq!(
        f.call(json!({"method":"list_accounts"})).await["data"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(f.cloud.calls.load(Ordering::SeqCst), calls);
}

#[tokio::test]
async fn provider_admission_and_lifecycle_cancel_without_replaying_login() {
    let f = Fixture::new(false);
    assert_eq!(
        f.connect(Some("midea.meiju"), None).await["error"]["code"],
        "AUTH_FLOW_UNSUPPORTED"
    );
    assert_eq!(f.cloud.calls.load(Ordering::SeqCst), 0);
    f.cloud.stall.store(true, Ordering::SeqCst);
    let connect = f.connect(None, None);
    tokio::pin!(connect);
    tokio::select! { _ = &mut connect => panic!("The fictional request must wait"), _ = f.cloud.started.notified() => {} }
    f.session.set_foreground(false);
    assert_eq!(connect.await["error"]["code"], "CANCELLED");
    assert_eq!(
        f.connect(None, None).await["error"]["code"],
        "HOST_BACKGROUND"
    );
    assert_eq!(f.cloud.calls.load(Ordering::SeqCst), 1);
    f.session.set_foreground(true);
    f.cloud.stall.store(false, Ordering::SeqCst);
    assert!(f.connect(None, None).await["error"].is_null());
}

fn begin_metadata() -> ConnectRequest {
    serde_json::from_value(json!({"schema_version":"1.0","operation_id":"begin-captcha","timeout_ms":10000,"provider":"xiaomi.mijia","label":"Mijia Home"})).unwrap()
}
fn answer_metadata(id: u64) -> homeverb_host::ContinueAccountLoginRequest {
    serde_json::from_value(json!({"schema_version":"1.0","operation_id":"answer-captcha","timeout_ms":10000,"challenge_id":id})).unwrap()
}
async fn pending_login(f: &Fixture) -> u64 {
    let result = f
        .session
        .begin_account_login(
            begin_metadata(),
            secret(b"fiction@example.invalid"),
            secret(b"fictional-password"),
        )
        .await
        .unwrap();
    assert_eq!(format!("{result:?}"), "NativeAccountLogin([REDACTED])");
    let homeverb_host::NativeAccountLogin::Captcha {
        challenge_id,
        image,
        media_type,
        expires_in_ms,
    } = result
    else {
        panic!("Expected captcha")
    };
    assert_eq!(media_type, "image/png");
    assert!(image.expose().starts_with(b"\x89PNG"));
    assert!((1..=180000).contains(&expires_in_ms));
    challenge_id
}

#[tokio::test]
async fn interactive_host_replaces_challenges_and_publishes_partial_cards_after_success() {
    let f = Fixture::new(true);
    f.cloud.captcha.store(true, Ordering::SeqCst);
    let first = pending_login(&f).await;
    assert!(
        f.call(json!({"method":"list_accounts"})).await["data"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let homeverb_host::NativeAccountLogin::Captcha {
        challenge_id: second,
        ..
    } = f
        .session
        .continue_account_login(answer_metadata(first), secret(b"wrong"))
        .await
        .unwrap()
    else {
        panic!("Expected new captcha")
    };
    assert_ne!(first, second);
    let calls = f.cloud.calls.load(Ordering::SeqCst);
    assert!(
        f.session
            .continue_account_login(answer_metadata(first), secret(b"correct"))
            .await
            .is_err()
    );
    assert_eq!(f.cloud.calls.load(Ordering::SeqCst), calls);
    let homeverb_host::NativeAccountLogin::Connected(report) = f
        .session
        .continue_account_login(answer_metadata(second), secret(b"correct"))
        .await
        .unwrap()
    else {
        panic!("Expected connection")
    };
    let report = serde_json::to_value(report).unwrap();
    assert_eq!(report["retrieval"]["status"], "partial");
    assert_eq!(
        report["retrieval"]["device_ids"].as_array().unwrap().len(),
        2
    );
    assert!(!serde_json::to_string(&report).unwrap().contains("PRIVATE"));
    assert!(!f.session.discard_account_login(second));
}

#[tokio::test]
async fn native_challenges_are_owned_bounded_and_cleared_by_lifecycle() {
    for mode in 0..4 {
        let f = Fixture::new(false);
        f.cloud.captcha.store(true, Ordering::SeqCst);
        let id = pending_login(&f).await;
        let other = Fixture::new(false);
        assert!(
            other
                .session
                .continue_account_login(answer_metadata(id), secret(b"correct"))
                .await
                .is_err()
        );
        let calls = f.cloud.calls.load(Ordering::SeqCst);
        match mode {
            0 => {
                f.session.set_foreground(false);
                f.session.set_foreground(true);
            }
            1 => {
                f.session.cancel("begin-captcha");
            }
            2 => f.session.close(),
            _ => {
                assert!(f.session.discard_account_login(id));
            }
        }
        assert!(
            f.session
                .continue_account_login(answer_metadata(id), secret(b"correct"))
                .await
                .is_err()
        );
        assert_eq!(f.cloud.calls.load(Ordering::SeqCst), calls);
    }
    let f = Fixture::new(false);
    f.cloud.captcha.store(true, Ordering::SeqCst);
    let mut ids = vec![];
    for _ in 0..4 {
        ids.push(pending_login(&f).await);
    }
    let calls = f.cloud.calls.load(Ordering::SeqCst);
    assert_eq!(
        f.session
            .begin_account_login(
                begin_metadata(),
                secret(b"fiction@example.invalid"),
                secret(b"fictional-password")
            )
            .await
            .unwrap_err()
            .code,
        "RESOURCE_BUSY"
    );
    assert_eq!(f.cloud.calls.load(Ordering::SeqCst), calls);
    assert!(f.session.discard_account_login(ids[0]));
    pending_login(&f).await;
}

#[tokio::test]
async fn late_captcha_cannot_restore_cancelled_or_backgrounded_host_state() {
    for background in [false, true] {
        let f = Fixture::new(false);
        f.cloud.captcha.store(true, Ordering::SeqCst);
        f.cloud.stall.store(true, Ordering::SeqCst);
        let started = f.cloud.started.notified();
        let begin = f.session.begin_account_login(
            begin_metadata(),
            secret(b"fiction@example.invalid"),
            secret(b"fictional-password"),
        );
        tokio::pin!(begin);
        tokio::select! { _ = started => {}, value = &mut begin => panic!("Unexpected result: {value:?}") }
        if background {
            f.session.set_foreground(false);
        } else {
            f.session.cancel("begin-captcha");
        }
        assert_eq!(begin.await.unwrap_err().code, "CANCELLED");
        f.session.set_foreground(true);
        assert!(
            f.call(json!({"method":"list_accounts"})).await["data"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }
}

#[tokio::test(start_paused = true)]
async fn expired_challenge_and_invalid_continuation_metadata_never_replay() {
    for expired in [true, false] {
        let f = Fixture::new(false);
        f.cloud.captcha.store(true, Ordering::SeqCst);
        let id = pending_login(&f).await;
        let calls = f.cloud.calls.load(Ordering::SeqCst);
        let mut metadata = answer_metadata(id);
        if expired {
            tokio::time::advance(std::time::Duration::from_secs(181)).await;
        } else {
            metadata.schema_version = "invalid".into();
        }
        assert!(
            f.session
                .continue_account_login(metadata, secret(b"correct"))
                .await
                .is_err()
        );
        assert!(
            f.session
                .continue_account_login(answer_metadata(id), secret(b"correct"))
                .await
                .is_err()
        );
        assert_eq!(f.cloud.calls.load(Ordering::SeqCst), calls);
    }
}

#[tokio::test]
async fn protected_login_requires_registered_provider_and_cannot_enter_json_dispatch() {
    let f = Fixture::new(true);
    let mut input = begin_metadata();
    input.provider = Some(homeverb_host::AccountProvider::Meiju);
    assert_eq!(
        f.session
            .begin_account_login(
                input,
                secret(b"fiction@example.invalid"),
                secret(b"fictional-password")
            )
            .await
            .unwrap_err()
            .code,
        "AUTH_FLOW_UNSUPPORTED"
    );
    assert_eq!(f.cloud.calls.load(Ordering::SeqCst), 0);
    for method in [
        "begin_account_login",
        "continue_account_login",
        "discard_account_login",
    ] {
        assert!(serde_json::from_value::<HostRequest>(json!({"schema_version":"1.0","operation_id":"test","timeout_ms":1000,"request":{"method":method}})).is_err());
    }
}

#[path = "mijia_accounts/mijia_verification.rs"]
mod mijia_verification;

#[tokio::test]
async fn outlet_inspection_before_grants_is_local_and_contract_valid() {
    for combined in [false, true] {
        let f = Fixture::new(combined);
        f.cloud.outlet.store(true, Ordering::SeqCst);
        let connected = f.connect(Some("xiaomi.mijia"), None).await;
        assert!(connected["error"].is_null(), "{connected}");
        let devices = f.call(json!({"method":"list_devices"})).await;
        let device = &devices["data"][0];
        let before = f.cloud.calls.load(Ordering::SeqCst);
        let options = f.call(json!({"method":"inspect_control_options","device_id":device["id"],"connection_id":device["connections"][0]["id"]})).await;
        assert!(options["error"].is_null(), "{options}");
        let options = options["data"]["options"].as_array().unwrap();
        assert_eq!(options.len(), 4);
        assert_eq!(
            options
                .iter()
                .filter(|o| o["definition"]["risk"] == "elevated")
                .count(),
            3
        );
        let grants = f.call(json!({"method":"inspect_session_grants","device_id":device["id"],"connection_id":device["connections"][0]["id"]})).await;
        assert_eq!(grants["data"]["grants"], json!([]));
        assert_eq!(f.cloud.calls.load(Ordering::SeqCst), before);
    }
}

#[tokio::test]
async fn native_outlet_approval_is_foreground_single_use_and_preserves_uncertainty() {
    for lost in [false, true] {
        let f = Fixture::new(false);
        f.cloud.outlet.store(true, Ordering::SeqCst);
        let connected = f.connect(Some("xiaomi.mijia"), None).await;
        let devices = f.call(json!({"method":"list_devices"})).await;
        let device = devices["data"][0]["id"].as_str().unwrap();
        let policy = homeverb_host::mijia_outlet_policy(&[homeverb_host::MijiaOutletGrant {
            device_id: device.into(),
            component_id: "switch-2".into(),
            property: "state".into(),
            value: json!(true),
        }])
        .unwrap();
        let store = Arc::new(SqliteStore::open(&f.dir).unwrap());
        let adapter = Arc::new(
            MijiaAdapter::new(f.vault.clone(), f.cloud.clone(), Arc::new(SystemClock))
                .with_outlet_controls()
                .with_control_policies(Policy::default(), policy.clone()),
        );
        let core = Core::new(store, vec![adapter.clone()], Arc::new(SystemClock), policy);
        let account: Uuid =
            serde_json::from_value(connected["data"]["account"]["id"].clone()).unwrap();
        let report = adapter.refresh_devices(account, &op()).await.unwrap();
        core.publish_discovery(
            homeverb_mijia::PROVIDER,
            &report.descriptors.unwrap(),
            &op(),
        )
        .await
        .unwrap();
        let session = Session::with_platform_accounts(core, None, Some(adapter));
        let mut command = json!({"schema_version":"1.0","request_id":"outlet-native","device_id":device,"component_id":"switch-2","capability":"power","action":"set","arguments":{"state":true},"connection":"cloud","timeout_ms":5000,"wait_for":"accepted"});
        let prepare = |command: Value| {
            serde_json::from_value(
                json!({"schema_version":"1.0","operation_id":"preview","command":command}),
            )
            .unwrap()
        };
        let execute = |id: Value| {
            serde_json::from_value(
                json!({"schema_version":"1.0","operation_id":"consent","approval_id":id}),
            )
            .unwrap()
        };
        let preview = session
            .prepare_native_approval(prepare(command.clone()))
            .await;
        assert!(preview["error"].is_null(), "{preview}");
        assert_eq!(preview["data"]["action"]["risk"], "elevated");
        assert_eq!(f.cloud.writes.load(Ordering::SeqCst), 0);
        session.set_foreground(false);
        session.set_foreground(true);
        let stale = session
            .execute_native_approved(execute(preview["data"]["approval_id"].clone()))
            .await;
        assert_eq!(stale["error"]["code"], "APPROVAL_INVALID");
        assert_eq!(f.cloud.writes.load(Ordering::SeqCst), 0);
        command["request_id"] = json!("outlet-confirmed");
        let preview = session.prepare_native_approval(prepare(command)).await;
        assert!(preview["error"].is_null(), "{preview}");
        f.cloud.lose_write.store(lost, Ordering::SeqCst);
        let result = session
            .execute_native_approved(execute(preview["data"]["approval_id"].clone()))
            .await;
        assert_eq!(
            result["status"],
            if lost { "unknown" } else { "accepted" },
            "{result}"
        );
        assert_eq!(result["request_id"], "outlet-confirmed");
        let repeated = session
            .execute_native_approved(execute(preview["data"]["approval_id"].clone()))
            .await;
        assert_eq!(repeated["error"]["code"], "APPROVAL_INVALID");
        assert_eq!(f.cloud.writes.load(Ordering::SeqCst), 1);
    }
}

#[tokio::test]
async fn auxiliary_options_are_local_before_grants_and_match_native_scopes() {
    for (mode, component, count) in [
        (1, "light-6", 2),
        (1, "hood-2", 1),
        (1, "fan_control-3", 1),
        (1, "screen-13", 2),
        (2, "function-3", 1),
    ] {
        let f = Fixture::new(true);
        f.cloud.auxiliary.store(mode, Ordering::SeqCst);
        let connected = f.connect(Some("xiaomi.mijia"), None).await;
        assert!(connected["error"].is_null(), "{connected}");
        let devices = f.call(json!({"method":"list_devices"})).await;
        let device = &devices["data"][0];
        let before = f.cloud.calls.load(Ordering::SeqCst);
        let options=f.call(json!({"method":"inspect_control_options","device_id":device["id"],"connection_id":device["connections"][0]["id"]})).await;
        assert!(options["error"].is_null(), "{options}");
        let options: Vec<_> = options["data"]["options"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|o| o["component_id"] == component)
            .collect();
        assert_eq!(options.len(), count);
        assert!(options.iter().all(|o| o["definition"]["risk"] == "low"));
        let grants=f.call(json!({"method":"inspect_session_grants","device_id":device["id"],"connection_id":device["connections"][0]["id"]})).await;
        assert_eq!(grants["data"]["grants"], json!([]));
        assert_eq!(f.cloud.calls.load(Ordering::SeqCst), before);
        assert_eq!(f.cloud.writes.load(Ordering::SeqCst), 0);
    }
}

#[path = "mijia_accounts/modern_host.rs"]
mod modern_host;
