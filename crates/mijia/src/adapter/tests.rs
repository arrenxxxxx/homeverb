use super::*;
use crate::{CloudEndpoint, CloudRequest, CloudResponse, codec};
use homeverb_core::{
    AccountStore, Core, CredentialRef, Policy, SecretBytes, SecretStore, Store, SystemClock,
};
use homeverb_domain::{PreferencesInput, Route};
use homeverb_storage::SqliteStore;
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{
        Mutex,
        atomic::{AtomicU8, AtomicUsize, Ordering},
    },
};
use tokio_util::sync::CancellationToken;

fn operation() -> Operation {
    Operation::new(20_000, CancellationToken::new()).unwrap()
}
fn credentials() -> LoginCredentials {
    LoginCredentials::new(
        SecretBytes::new(b"fictional@example.invalid".to_vec()),
        SecretBytes::new(b"fictional-password".to_vec()),
    )
    .unwrap()
}
fn fan_kind() -> &'static str {
    "urn:miot-spec-v2:device:fan:0000A005:fiction-one:1"
}
fn fan_spec() -> Value {
    json!({"type":fan_kind(),"services":[{"iid":2,"type":"urn:miot-spec-v2:service:fan:00007808:fiction-one:1","properties":[
        {"iid":1,"type":"urn:miot-spec-v2:property:on:00000006:fiction-one:1","format":"bool","access":["read","write"]},
        {"iid":2,"type":"urn:miot-spec-v2:property:fan-level:00000016:fiction-one:1","format":"uint8","access":["read","write"],"value-range":[1,4,1]}
    ]}]})
}
struct Directory(PathBuf);
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
#[derive(Default)]
struct Secrets {
    values: Mutex<HashMap<String, Vec<u8>>>,
    puts: AtomicUsize,
    mode: AtomicU8,
}
#[async_trait]
impl SecretStore for Secrets {
    async fn get(&self, key: &CredentialRef, op: &Operation) -> Result<Option<SecretBytes>> {
        op.run(async {
            if self.mode.load(Ordering::SeqCst) == 1 {
                return Err(Error::new(
                    "SECRET_STORE_UNAVAILABLE",
                    "Fictional unavailable store.",
                ));
            }
            Ok(self
                .values
                .lock()
                .unwrap()
                .get(&key.to_string())
                .map(|v| SecretBytes::new(v.clone())))
        })
        .await
    }
    async fn put(&self, key: &CredentialRef, value: &SecretBytes, op: &Operation) -> Result<()> {
        op.run(async {
            self.puts.fetch_add(1, Ordering::SeqCst);
            self.values
                .lock()
                .unwrap()
                .insert(key.to_string(), value.expose().to_vec());
            if self.mode.load(Ordering::SeqCst) == 2 {
                return Err(Error::new(
                    "SECRET_STORE_WRITE_UNKNOWN",
                    "Fictional uncertain store.",
                ));
            }
            Ok(())
        })
        .await
    }
    async fn delete(&self, key: &CredentialRef, op: &Operation) -> Result<()> {
        op.run(async {
            self.values.lock().unwrap().remove(&key.to_string());
            Ok(())
        })
        .await
    }
}
struct Cloud {
    calls: Mutex<Vec<CloudEndpoint>>,
    subject: Mutex<String>,
    devices: Mutex<Vec<Value>>,
    power: Mutex<Value>,
    spec_failure: AtomicU8,
    include_battery: AtomicU8,
    read_gate: tokio::sync::Notify,
    stall_read: AtomicU8,
    write_mode: AtomicU8,
}
impl Cloud {
    fn new() -> Self {
        Self {
            calls: Mutex::new(vec![]),
            subject: Mutex::new("999001".into()),
            devices: Mutex::new(vec![
                json!({"did":"PRIVATE-FAN-IDENTITY","name":"Desk Fan","model":"fiction.fan.one","spec_type":fan_kind(),"isOnline":true}),
                json!({"did":"PRIVATE-UNKNOWN-IDENTITY","name":"Other Appliance","model":"fiction.other.one"}),
            ]),
            power: Mutex::new(json!(true)),
            spec_failure: AtomicU8::new(0),
            include_battery: AtomicU8::new(0),
            read_gate: tokio::sync::Notify::new(),
            stall_read: AtomicU8::new(0),
            write_mode: AtomicU8::new(0),
        }
    }
    fn count(&self, endpoint: CloudEndpoint) -> usize {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .filter(|e| **e == endpoint)
            .count()
    }
}
#[async_trait]
impl CloudTransport for Cloud {
    async fn send_control(
        &self,
        request: &CloudRequest,
        progress: &WriteProgress,
        op: &Operation,
    ) -> Result<CloudResponse> {
        op.run(async {
            assert_eq!(request.endpoint(), CloudEndpoint::SetProperty);
            let url = reqwest::Url::parse(&format!("https://localhost/?{}", std::str::from_utf8(request.body()).unwrap())).unwrap();
            let fields: HashMap<_,_> = url.query_pairs().into_owned().collect();
            let key = codec::signed_nonce(&fields["ssecurity"], &fields["_nonce"])?;
            let data: Value = serde_json::from_slice(codec::decrypt(&key,fields["data"].as_bytes())?.expose()).unwrap();
            assert_eq!(data["params"].as_array().unwrap().len(), 1);
            let field = &data["params"][0];
            assert_eq!(field["did"], "PRIVATE-FAN-IDENTITY");
            assert_eq!(field["siid"],2);
            self.calls.lock().unwrap().push(CloudEndpoint::SetProperty);
            progress.mark_sent();
            let mode = self.write_mode.load(Ordering::SeqCst);
            if mode == 1 { return Err(crate::failed()); }
            if mode == 3 { self.read_gate.notify_one(); std::future::pending::<()>().await; }
            let result = json!({"code":0,"result":[{"did":field["did"],"siid":field["siid"],"piid":field["piid"],"code":if mode == 2 {-704220043} else {0}}]});
            let encrypted = codec::encrypt(&key, &serde_json::to_vec(&result).unwrap());
            Ok(CloudResponse { body: SecretBytes::new(encrypted.as_str().as_bytes().to_vec()), service_token: None })
        }).await
    }
    async fn send(&self, request: &CloudRequest, op: &Operation) -> Result<CloudResponse> {
        op.run(async {
            self.calls.lock().unwrap().push(request.endpoint());
            let raw=match request.endpoint() {
                CloudEndpoint::LoginSign=>Some(json!({"_sign":"fictional-sign"})),
                CloudEndpoint::LoginPassword=>Some(json!({"code":0,"userId":self.subject.lock().unwrap().clone(),"ssecurity":"AAECAwQFBgcICQoLDA0ODw==","location":"https://sts.api.io.mi.com/sts?ticket=fictional"})),
                CloudEndpoint::LoginExchange=>return Ok(CloudResponse{body:SecretBytes::new(vec![]),service_token:Some(SecretBytes::new(b"PRIVATE-SERVICE-TOKEN".to_vec()))}),
                CloudEndpoint::SpecIndex=>{
                    if self.spec_failure.load(Ordering::SeqCst)!=0 {return Err(crate::failed());}
                    Some(json!({"instances":[{"model":"fiction.fan.one","type":fan_kind(),"status":"released","version":1}]}))
                },
                CloudEndpoint::SpecInstance=>{
                    let mut spec = fan_spec();
                    if self.include_battery.load(Ordering::SeqCst) != 0 {
                        spec["services"].as_array_mut().unwrap().insert(0, json!({"iid":1,"type":"urn:miot-spec-v2:service:battery:00007805:fiction-one:1","properties":[{"iid":1,"type":"urn:miot-spec-v2:property:battery-level:00000014:fiction-one:1","format":"uint8","access":["read"],"value-range":[0,100,1],"unit":"percentage"}]}));
                    }
                    Some(spec)
                },
                _=>None,
            };
            if let Some(raw)=raw {return Ok(CloudResponse{body:SecretBytes::new(serde_json::to_vec(&raw).unwrap()),service_token:None});}
            let url=reqwest::Url::parse(&format!("https://localhost/?{}",std::str::from_utf8(request.body()).unwrap())).unwrap();
            let fields:HashMap<_,_>=url.query_pairs().into_owned().collect();
            let key=codec::signed_nonce(&fields["ssecurity"],&fields["_nonce"])?;
            let data:Value=serde_json::from_slice(codec::decrypt(&key,fields["data"].as_bytes())?.expose()).unwrap();
            let result=match request.endpoint() {
                CloudEndpoint::Devices=>json!({"list":self.devices.lock().unwrap().clone()}),
                CloudEndpoint::Homes=>json!({"homelist":[],"has_more":false}),
                CloudEndpoint::Properties=>{
                    if self.stall_read.load(Ordering::SeqCst)!=0 {self.read_gate.notify_one();std::future::pending::<()>().await;}
                    let values:Vec<_>=data["params"].as_array().unwrap().iter().map(|p|json!({"did":p["did"],"siid":p["siid"],"piid":p["piid"],"code":0,"value":if p["piid"]==1 {self.power.lock().unwrap().clone()}else{json!(3)}})).collect();
                    json!(values)
                },
                _=>panic!("unexpected endpoint"),
            };
            let encrypted=codec::encrypt(&key,&serde_json::to_vec(&json!({"code":0,"result":result})).unwrap());
            Ok(CloudResponse{body:SecretBytes::new(encrypted.as_str().as_bytes().to_vec()),service_token:None})
        }).await
    }
}
struct Fixture {
    directory: Directory,
    store: Arc<SqliteStore>,
    secrets: Arc<Secrets>,
    cloud: Arc<Cloud>,
    adapter: Arc<MijiaAdapter>,
    core: Core,
    vault: Arc<AccountVault>,
}
impl Fixture {
    fn new() -> Self {
        let directory =
            Directory(std::env::temp_dir().join(format!("homeverb-mijia-{}", Uuid::new_v4())));
        let store = Arc::new(SqliteStore::open(&directory.0).unwrap());
        let secrets = Arc::new(Secrets::default());
        let cloud = Arc::new(Cloud::new());
        let vault = Arc::new(AccountVault::new(store.clone(), secrets.clone()));
        let clock = Arc::new(SystemClock);
        let adapter = Arc::new(MijiaAdapter::new(
            vault.clone(),
            cloud.clone(),
            clock.clone(),
        ));
        let core = Core::new(
            store.clone(),
            vec![adapter.clone()],
            clock,
            Policy::default(),
        );
        Self {
            directory,
            store,
            secrets,
            cloud,
            adapter,
            core,
            vault,
        }
    }
    async fn connect(&self) -> AccountDiscovery {
        self.adapter
            .connect_devices(credentials(), None, "Home", &operation())
            .await
            .unwrap()
    }
    async fn publish(&self, report: &AccountDiscovery) {
        self.core
            .publish_discovery(PROVIDER, report.descriptors.as_ref().unwrap(), &operation())
            .await
            .unwrap();
    }
    fn fan(&self) -> Device {
        self.core
            .list()
            .unwrap()
            .into_iter()
            .find(|d| d.model == "fiction.fan.one")
            .unwrap()
    }
}

#[tokio::test]
async fn protected_connection_publishes_stable_cards_and_core_reads_without_identity_leaks() {
    let f = Fixture::new();
    assert!(f.core.list().unwrap().is_empty());
    assert!(f.cloud.calls.lock().unwrap().is_empty());
    let report = f.connect().await;
    assert_eq!(report.account.credential_state, CredentialState::Stored);
    assert!(!report.complete);
    assert!(report.account.last_complete_inventory_at.is_none());
    f.publish(&report).await;
    let fan = f.fan();
    let devices = f.core.list().unwrap();
    assert_eq!(devices.len(), 2);
    assert_eq!(
        devices
            .iter()
            .find(|d| d.model == "fiction.other.one")
            .unwrap()
            .support_status,
        "unsupported"
    );
    let state = f
        .core
        .read(&fan.id, "fan-2", Route::Cloud, false, &operation())
        .await
        .unwrap();
    assert_eq!(state.observations["power.state"].value, true);
    assert!(
        state
            .observations
            .values()
            .all(|o| o.stale && o.observed_at.is_none())
    );
    let public = serde_json::to_string(&devices).unwrap() + &serde_json::to_string(&state).unwrap();
    for private in [
        "PRIVATE-FAN-IDENTITY",
        "PRIVATE-UNKNOWN-IDENTITY",
        "PRIVATE-SERVICE-TOKEN",
        "999001",
    ] {
        assert!(!public.contains(private));
    }
    assert!(f.secrets.values.lock().unwrap().values().any(|v| {
        v.windows(b"PRIVATE-FAN-IDENTITY".len())
            .any(|w| w == b"PRIVATE-FAN-IDENTITY")
    }));
    for entry in std::fs::read_dir(&f.directory.0).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            let bytes = std::fs::read(path).unwrap();
            for private in [
                b"PRIVATE-FAN-IDENTITY".as_slice(),
                b"PRIVATE-SERVICE-TOKEN".as_slice(),
            ] {
                assert!(!bytes.windows(private.len()).any(|w| w == private));
            }
        }
    }
}

#[tokio::test]
async fn refresh_and_same_subject_reconnect_preserve_preferences_bindings_and_unchanged_secret() {
    let f = Fixture::new();
    let report = f.connect().await;
    f.publish(&report).await;
    let before = f.fan();
    f.store
        .set_preferences(
            &before.id,
            &PreferencesInput {
                alias: Some("My Fan".into()),
                room_id: None,
                favorite: true,
            },
        )
        .unwrap();
    let puts = f.secrets.puts.load(Ordering::SeqCst);
    let refresh = f
        .adapter
        .refresh_devices(report.account.id, &operation())
        .await
        .unwrap();
    f.publish(&refresh).await;
    assert_eq!(f.secrets.puts.load(Ordering::SeqCst), puts);
    let reconnect = f
        .adapter
        .connect_devices(credentials(), None, "Unused New Label", &operation())
        .await
        .unwrap();
    f.publish(&reconnect).await;
    assert_eq!(reconnect.account.id, report.account.id);
    assert_eq!(reconnect.account.label, "Home");
    assert_eq!(f.fan().id, before.id);
    assert_eq!(f.fan().connections, before.connections);
    assert_eq!(f.core.list().unwrap().len(), 2);
    assert_eq!(
        f.store
            .preferences()
            .unwrap()
            .iter()
            .find(|p| p.device_id == before.id)
            .unwrap()
            .alias
            .as_deref(),
        Some("My Fan")
    );
    assert_eq!(f.cloud.count(CloudEndpoint::SpecInstance), 1);
}

#[tokio::test]
async fn changed_subject_and_public_route_or_model_tampering_fail_before_read() {
    let f = Fixture::new();
    let report = f.connect().await;
    f.publish(&report).await;
    let saved = f.secrets.values.lock().unwrap().clone();
    *f.cloud.subject.lock().unwrap() = "999002".into();
    let error = f
        .adapter
        .connect_devices(credentials(), Some(report.account.id), "Home", &operation())
        .await
        .err()
        .unwrap();
    assert_eq!(error.code, "ACCOUNT_IDENTITY_CONFLICT");
    assert_eq!(*f.secrets.values.lock().unwrap(), saved);
    let original = f.fan();
    for mode in 0..3 {
        let mut fan = original.clone();
        if mode == 0 {
            fan.model = "fiction.other.one".into();
        } else if mode == 1 {
            fan.connections[0].identity_ref = Uuid::new_v4().to_string();
        } else {
            fan.components[0].id = "foreign".into();
        }
        assert_eq!(
            f.adapter
                .read(&fan, "fan-2", &fan.connections[0], &operation())
                .await
                .unwrap_err()
                .code,
            "DEVICE_IDENTITY_CONFLICT"
        );
    }
    assert_eq!(f.cloud.count(CloudEndpoint::Properties), 0);
}

#[tokio::test]
async fn unreadable_secure_store_refuses_before_login_and_rename_stays_local() {
    let f = Fixture::new();
    f.secrets.mode.store(1, Ordering::SeqCst);
    assert_eq!(
        f.adapter
            .connect_devices(credentials(), None, "Home", &operation())
            .await
            .err()
            .unwrap()
            .code,
        "SECRET_STORE_UNAVAILABLE"
    );
    assert!(f.cloud.calls.lock().unwrap().is_empty());
    assert!(f.store.accounts().unwrap().is_empty());
    f.secrets.mode.store(0, Ordering::SeqCst);
    let report = f.connect().await;
    let calls = f.cloud.calls.lock().unwrap().len();
    f.secrets.mode.store(1, Ordering::SeqCst);
    let renamed = f
        .adapter
        .rename_account(report.account.id, "New Name", &operation())
        .await
        .unwrap();
    assert_eq!(renamed.label, "New Name");
    assert_eq!(calls, f.cloud.calls.lock().unwrap().len());
}

#[tokio::test]
async fn spec_failure_keeps_named_placeholder_and_explicit_refresh_can_upgrade_it() {
    let f = Fixture::new();
    f.cloud.spec_failure.store(1, Ordering::SeqCst);
    let report = f.connect().await;
    f.publish(&report).await;
    assert_eq!(f.fan().support_status, "unsupported");
    assert_eq!(report.issues[0].code, "MIJIA_SPEC_UNAVAILABLE");
    let identity = f.fan().id;
    f.cloud.spec_failure.store(0, Ordering::SeqCst);
    let report = f
        .adapter
        .refresh_devices(report.account.id, &operation())
        .await
        .unwrap();
    f.publish(&report).await;
    assert_eq!(f.fan().id, identity);
    assert_eq!(f.fan().support_status, "experimental");
}

#[tokio::test]
async fn uncertain_secure_save_requires_explicit_same_bytes_recovery_without_login() {
    let f = Fixture::new();
    f.secrets.mode.store(2, Ordering::SeqCst);
    assert_eq!(
        f.adapter
            .connect_devices(credentials(), None, "Home", &operation())
            .await
            .err()
            .unwrap()
            .code,
        "SECRET_STORE_WRITE_UNKNOWN"
    );
    let record = f.store.accounts().unwrap().pop().unwrap();
    assert_eq!(record.credential_state, CredentialState::Pending);
    assert_eq!(
        f.adapter
            .refresh_devices(record.id, &operation())
            .await
            .err()
            .unwrap()
            .code,
        "ACCOUNT_SESSION_UNCONFIRMED"
    );
    f.secrets.mode.store(0, Ordering::SeqCst);
    let login_calls = f.cloud.count(CloudEndpoint::LoginPassword);
    let report = f
        .adapter
        .recover_devices(record.id, &operation())
        .await
        .unwrap();
    f.publish(&report).await;
    assert_eq!(report.account.credential_state, CredentialState::Stored);
    assert_eq!(f.cloud.count(CloudEndpoint::LoginPassword), login_calls);
    assert_eq!(f.core.list().unwrap().len(), 2);
}

#[tokio::test]
async fn partial_empty_inventory_retains_cards_bindings_and_model_conflict_preserves_bytes() {
    let f = Fixture::new();
    let report = f.connect().await;
    f.publish(&report).await;
    let devices = f.cloud.devices.lock().unwrap().clone();
    let before = f.secrets.values.lock().unwrap().clone();
    f.cloud.devices.lock().unwrap().clear();
    let refresh = f
        .adapter
        .refresh_devices(report.account.id, &operation())
        .await
        .unwrap();
    f.publish(&refresh).await;
    assert!(refresh.descriptors.unwrap().is_empty());
    assert_eq!(f.core.list().unwrap().len(), 2);
    assert_eq!(*f.secrets.values.lock().unwrap(), before);
    *f.cloud.devices.lock().unwrap() = devices;
    f.cloud.devices.lock().unwrap()[0]["model"] = json!("fiction.changed.one");
    let refresh = f
        .adapter
        .refresh_devices(report.account.id, &operation())
        .await
        .unwrap();
    assert_eq!(
        refresh.descriptors.unwrap_err().code,
        "DEVICE_IDENTITY_CONFLICT"
    );
    assert_eq!(*f.secrets.values.lock().unwrap(), before);
}

#[tokio::test]
async fn invalid_live_value_preserves_cache_and_cancelled_read_releases_account_lease() {
    let f = Fixture::new();
    let report = f.connect().await;
    f.publish(&report).await;
    let fan = f.fan();
    f.core
        .read(&fan.id, "fan-2", Route::Cloud, false, &operation())
        .await
        .unwrap();
    *f.cloud.power.lock().unwrap() = json!("not-a-boolean");
    assert_eq!(
        f.core
            .read(&fan.id, "fan-2", Route::Cloud, false, &operation())
            .await
            .unwrap_err()
            .code,
        "INVALID_OBSERVATION"
    );
    assert_eq!(
        f.core
            .read(&fan.id, "fan-2", Route::Cloud, true, &operation())
            .await
            .unwrap()
            .observations["power.state"]
            .value,
        true
    );
    f.cloud.stall_read.store(1, Ordering::SeqCst);
    let op = operation();
    let read = f.core.read(&fan.id, "fan-2", Route::Cloud, false, &op);
    tokio::pin!(read);
    tokio::select! {result=&mut read=>panic!("Unexpected result: {result:?}"),_=f.cloud.read_gate.notified()=>{}}
    assert!(
        f.vault
            .lease(report.account.id, &operation())
            .await
            .is_err()
    );
    op.cancellation.cancel();
    assert_eq!(read.await.unwrap_err().code, "CANCELLED");
    assert!(f.vault.lease(report.account.id, &operation()).await.is_ok());
}

#[tokio::test]
async fn removal_erases_only_local_account_and_rejects_reuse() {
    let f = Fixture::new();
    let report = f.connect().await;
    f.publish(&report).await;
    let calls = f.cloud.calls.lock().unwrap().len();
    let ids = f
        .core
        .list()
        .unwrap()
        .iter()
        .map(|d| d.id.clone())
        .collect::<Vec<_>>();
    let removed = f
        .adapter
        .remove_account(report.account.id, &ids, &operation())
        .await
        .unwrap();
    assert_eq!(removed.status, "removed");
    assert_eq!(removed.remote_revocation, "not_attempted");
    assert!(f.core.list().unwrap().is_empty());
    assert!(f.secrets.values.lock().unwrap().is_empty());
    assert_eq!(f.cloud.calls.lock().unwrap().len(), calls);
    assert!(
        f.core
            .publish_discovery(PROVIDER, report.descriptors.as_ref().unwrap(), &operation())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn corrupted_saved_mapping_cannot_select_a_different_wire_property() {
    let f = Fixture::new();
    let report = f.connect().await;
    f.publish(&report).await;
    let fan = f.fan();
    let key = report.account.credential_ref.to_string();
    {
        let mut values = f.secrets.values.lock().unwrap();
        let mut payload: Value = serde_json::from_slice(&values[&key]).unwrap();
        payload["bindings"][0]["mapping"]["properties"][0]["piid"] = json!(99);
        values.insert(key, serde_json::to_vec(&payload).unwrap());
    }
    assert_eq!(
        f.core
            .read(&fan.id, "fan-2", Route::Cloud, false, &operation())
            .await
            .unwrap_err()
            .code,
        "ACCOUNT_SESSION_UNCONFIRMED"
    );
    assert_eq!(f.cloud.count(CloudEndpoint::Properties), 0);
}

#[tokio::test]
async fn cross_account_inventory_cannot_duplicate_or_transfer_existing_device_identity() {
    let f = Fixture::new();
    let first = f.connect().await;
    f.publish(&first).await;
    let fan = f.fan();
    *f.cloud.subject.lock().unwrap() = "999002".into();
    let second = f.connect().await;
    assert_ne!(first.account.id, second.account.id);
    assert_eq!(
        second.descriptors.unwrap_err().code,
        "DEVICE_IDENTITY_CONFLICT"
    );
    assert_eq!(f.core.list().unwrap().len(), 2);
    assert_eq!(f.fan().connections, fan.connections);
    let second_inventory = f
        .adapter
        .inspect_inventory(second.account.id, &operation())
        .await
        .unwrap();
    assert!(second_inventory.devices.is_empty());
}

#[tokio::test]
async fn reopening_uses_saved_binding_and_never_logs_in_implicitly() {
    let f = Fixture::new();
    let report = f.connect().await;
    f.publish(&report).await;
    let fan = f.fan();
    let calls = f.cloud.calls.lock().unwrap().len();
    let store = Arc::new(SqliteStore::open(&f.directory.0).unwrap());
    let vault = Arc::new(AccountVault::new(store.clone(), f.secrets.clone()));
    let adapter = Arc::new(MijiaAdapter::new(
        vault,
        f.cloud.clone(),
        Arc::new(SystemClock),
    ));
    let core = Core::new(
        store,
        vec![adapter],
        Arc::new(SystemClock),
        Policy::default(),
    );
    assert_eq!(core.list().unwrap().len(), 2);
    assert_eq!(f.cloud.calls.lock().unwrap().len(), calls);
    let state = core
        .read(&fan.id, "fan-2", Route::Cloud, false, &operation())
        .await
        .unwrap();
    assert_eq!(state.observations["power.state"].value, true);
    assert_eq!(f.cloud.count(CloudEndpoint::LoginPassword), 1);
}

#[tokio::test]
async fn foreign_protected_envelopes_are_refused_without_cloud_access() {
    let f = Fixture::new();
    let report = f.connect().await;
    f.publish(&report).await;
    let fan = f.fan();
    let key = report.account.credential_ref.to_string();
    let original = f.secrets.values.lock().unwrap()[&key].clone();
    for (field, value) in [
        ("account_id", json!(Uuid::new_v4())),
        ("credential_ref", json!(Uuid::new_v4())),
        ("provider", json!("midea.meiju")),
        ("region", json!("US")),
        ("version", json!(2)),
    ] {
        let mut payload: Value = serde_json::from_slice(&original).unwrap();
        payload[field] = value;
        f.secrets
            .values
            .lock()
            .unwrap()
            .insert(key.clone(), serde_json::to_vec(&payload).unwrap());
        assert_eq!(
            f.core
                .read(&fan.id, "fan-2", Route::Cloud, false, &operation())
                .await
                .unwrap_err()
                .code,
            "ACCOUNT_SESSION_UNCONFIRMED"
        );
    }
    assert_eq!(f.cloud.count(CloudEndpoint::Properties), 0);
}

#[tokio::test]
async fn dropped_read_releases_lease_and_generic_discovery_refuses_before_network() {
    let f = Fixture::new();
    let report = f.connect().await;
    f.publish(&report).await;
    let fan = f.fan();
    let calls = f.cloud.calls.lock().unwrap().len();
    assert_eq!(
        f.core.discover(&operation()).await.unwrap_err().code,
        "MIJIA_INVENTORY_PARTIAL"
    );
    assert_eq!(f.cloud.calls.lock().unwrap().len(), calls);
    f.cloud.stall_read.store(1, Ordering::SeqCst);
    let op = operation();
    let mut read = Box::pin(f.core.read(&fan.id, "fan-2", Route::Cloud, false, &op));
    tokio::select! {result=&mut read=>panic!("Unexpected result: {result:?}"),_=f.cloud.read_gate.notified()=>{}}
    assert!(
        f.vault
            .lease(report.account.id, &operation())
            .await
            .is_err()
    );
    drop(read);
    assert!(f.vault.lease(report.account.id, &operation()).await.is_ok());
}

#[tokio::test]
async fn read_only_adapter_denies_a_forged_public_action_even_with_a_core_grant() {
    use homeverb_core::Grant;
    use homeverb_domain::{
        Action, Command, Idempotency, Risk, Status, VerificationMethod, WaitFor,
    };
    let f = Fixture::new();
    let report = f.connect().await;
    f.publish(&report).await;
    let fan = f.fan();
    let calls = f.cloud.calls.lock().unwrap().len();
    let mut descriptor = f.core.describe(&fan.id).unwrap();
    let cap = descriptor
        .capabilities
        .iter_mut()
        .find(|c| c.id == "power")
        .unwrap();
    cap.properties.get_mut("state").unwrap()["writable"] = json!(true);
    let parameters = json!({"type":"object","properties":{"state":{"type":"boolean"}},"required":["state"],"additionalProperties":false});
    cap.actions.insert(
        "set".into(),
        Action {
            parameters: parameters.clone(),
            transports: vec![Transport::Cloud],
            idempotency: Idempotency::SetValue,
            risk: Risk::Low,
            atomic: false,
            independent_fields: false,
            verification: VerificationMethod::Unavailable,
        },
    );
    f.store.save_device(&descriptor).unwrap();
    let policy = Policy {
        grants: vec![Grant {
            device_id: fan.id.clone(),
            component_id: "fan-2".into(),
            capability: "power".into(),
            action: "set".into(),
            transports: vec![Transport::Cloud],
            argument_schema: parameters,
        }],
    };
    let core = Core::new(
        f.store.clone(),
        vec![f.adapter.clone()],
        Arc::new(SystemClock),
        policy,
    );
    let command = Command {
        schema_version: "1.0".into(),
        request_id: "fictional-write-denied".into(),
        device_id: fan.id,
        component_id: "fan-2".into(),
        capability: "power".into(),
        action: "set".into(),
        arguments: json!({"state":false}),
        connection: Route::Cloud,
        timeout_ms: 10000,
        wait_for: WaitFor::Accepted,
    };
    let receipt = core.execute(command, CancellationToken::new()).await;
    assert_eq!(receipt.status, Status::Failed);
    assert_eq!(
        receipt.error.as_ref().unwrap().code,
        "CAPABILITY_UNSUPPORTED"
    );
    assert_eq!(receipt.attempts.len(), 1);
    assert_eq!(receipt.attempts[0].phase, homeverb_domain::Phase::NotSent);
    assert_eq!(f.cloud.calls.lock().unwrap().len(), calls);
}

#[tokio::test]
async fn primary_appliance_component_precedes_auxiliary_battery_for_card_reads() {
    let f = Fixture::new();
    f.cloud.include_battery.store(1, Ordering::SeqCst);
    let report = f.connect().await;
    f.publish(&report).await;
    let fan = f.fan();
    assert_eq!(
        fan.components
            .iter()
            .map(|c| c.id.as_str())
            .collect::<Vec<_>>(),
        vec!["fan-2", "battery-1"]
    );
    assert!(fan.components[0].capabilities.contains(&"power".into()));
    let state = f
        .core
        .read(
            &fan.id,
            &fan.components[0].id,
            Route::Cloud,
            false,
            &operation(),
        )
        .await
        .unwrap();
    assert_eq!(state.observations["power.state"].value, true);
}

#[path = "control_tests.rs"]
mod control_tests;
