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
    stall: AtomicBool,
    started: tokio::sync::Notify,
}
#[async_trait]
impl CloudTransport for Cloud {
    async fn send(&self, request: &CloudRequest, op: &Operation) -> Result<CloudResponse> {
        op.run(async {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.stall.load(Ordering::SeqCst) {
                self.started.notify_one(); std::future::pending::<()>().await;
            }
            let raw = match request.endpoint() {
                CloudEndpoint::LoginSign => Some(json!({"_sign":"fictional"})),
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
    assert_eq!(f.call(json!({"method":"inspect_control_options","device_id":fan["id"],"connection_id":fan["connections"][0]["id"]})).await["error"]["code"], "CAPABILITY_UNSUPPORTED");
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
