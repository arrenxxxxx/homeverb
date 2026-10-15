use async_trait::async_trait;
use homeverb_core::*;
use homeverb_domain::{Command, Device, Error, PreferencesInput, Result, Route, Snapshot};
use homeverb_midea::*;
use homeverb_storage::SqliteStore;
use serde_json::{Value, json};
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

const MODELS: &[(&str, &str)] = &[
    ("KFR-35G/N8HE1\u{2161}Pro", "AC"),
    ("KFR-35G/WF2", "AC"),
    ("KFR-72G/N8HA1Pro", "AC"),
    ("KFR-26G/N8VJC3", "AC"),
    ("BCD-600WUFIPZM(E)", "CA"),
    ("GX1200Max \u{5c0a}\u{4eab}\u{7248}", "E1"),
];

fn operation() -> Operation {
    Operation::new(1000, CancellationToken::new()).unwrap()
}
fn bytes(value: Value) -> SecretBytes {
    SecretBytes::new(serde_json::to_vec(&value).unwrap())
}
fn appliances() -> Vec<Value> {
    MODELS
        .iter()
        .enumerate()
        .map(|(i, (model, family))| {
            json!({
                "applianceCode": format!("88000000000000{i}"), "type":family,
                "productModel": model, "name":"Synthetic appliance"
            })
        })
        .collect()
}

#[derive(Default)]
struct Secrets {
    values: Mutex<HashMap<String, Vec<u8>>>,
    unavailable: AtomicBool,
}
#[async_trait]
impl SecretStore for Secrets {
    async fn get(
        &self,
        reference: &CredentialRef,
        operation: &Operation,
    ) -> Result<Option<SecretBytes>> {
        operation
            .run(async {
                if self.unavailable.load(Ordering::Relaxed) {
                    return Err(Error::new(
                        "SECRET_STORE_UNAVAILABLE",
                        "Synthetic unavailable store.",
                    ));
                }
                Ok(self
                    .values
                    .lock()
                    .unwrap()
                    .get(&reference.to_string())
                    .map(|v| SecretBytes::new(v.clone())))
            })
            .await
    }
    async fn put(
        &self,
        reference: &CredentialRef,
        secret: &SecretBytes,
        operation: &Operation,
    ) -> Result<()> {
        operation
            .run(async {
                self.values
                    .lock()
                    .unwrap()
                    .insert(reference.to_string(), secret.expose().to_vec());
                Ok(())
            })
            .await
    }
    async fn delete(&self, _: &CredentialRef, _: &Operation) -> Result<()> {
        panic!("Read-only discovery must not delete credentials.")
    }
}

#[derive(Default)]
struct Wire {
    inventories: Mutex<HashMap<String, Vec<Value>>>,
    state: Mutex<Option<Value>>,
    calls: AtomicUsize,
    partial: AtomicBool,
    delay: AtomicBool,
    started: tokio::sync::Notify,
}
#[async_trait]
impl CloudTransport for Wire {
    async fn send(&self, request: &SignedRequest, _: &Operation) -> Result<SecretBytes> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        self.started.notify_one();
        if self.delay.load(Ordering::Relaxed) {
            std::future::pending::<()>().await;
        }
        let token = std::str::from_utf8(request.token().unwrap()).unwrap();
        let data = match request.endpoint() {
            CloudEndpoint::Homes => json!({"homeList":[{"homegroupId":"1"}]}),
            CloudEndpoint::HomeAppliances => {
                if self.partial.load(Ordering::Relaxed) {
                    return Ok(bytes(json!({"code":999,"msg":"PRIVATE-PROVIDER-MESSAGE"})));
                }
                json!({"homeList":[{"roomList":[{"applianceList":self.inventories.lock().unwrap()[token]}]}]})
            }
            CloudEndpoint::ApplianceState => {
                let body: Value = serde_json::from_slice(request.body()).unwrap();
                assert_eq!(body["command"], json!({"query":{}}));
                if let Some(state) = self.state.lock().unwrap().as_ref() {
                    state.clone()
                } else {
                    match body["applianceCode"].as_str().unwrap() {
                        "880000000000004" => {
                            json!({"storage_temperature":4,"refrigeration_real_temperature":5,"freezing_temperature":-18,"freezing_real_temperature":-17})
                        }
                        "880000000000005" => {
                            json!({"mode":"fast_wash","work_status":"power_off","doorswitch":1,"water_lack":0,"softwater_lack":1,"bright_lack":1,"temperature":23,"left_time":0})
                        }
                        _ => {
                            json!({"power":"on","temperature":25,"small_temperature":0.5,"indoor_temperature":27.2,"mode":"cool","wind_speed":102})
                        }
                    }
                }
            }
            _ => panic!("A saved-account read must never log in implicitly."),
        };
        Ok(bytes(json!({"code":0,"data":data})))
    }
}

struct Fixture {
    core: Core,
    adapter: Arc<MeijuAdapter>,
    vault: Arc<AccountVault>,
    store: Arc<SqliteStore>,
    secrets: Arc<Secrets>,
    wire: Arc<Wire>,
    path: PathBuf,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
impl Fixture {
    async fn new(inventory: Vec<Value>) -> Self {
        let path = std::env::temp_dir().join(format!("homeverb-meiju-adapter-{}", Uuid::new_v4()));
        let store = Arc::new(SqliteStore::open(&path).unwrap());
        let secrets = Arc::new(Secrets::default());
        let vault = Arc::new(AccountVault::new(store.clone(), secrets.clone()));
        let wire = Arc::new(Wire::default());
        let adapter = Arc::new(MeijuAdapter::new(
            vault.clone(),
            wire.clone(),
            Arc::new(SystemClock),
        ));
        let core = Core::new(
            store.clone(),
            vec![adapter.clone()],
            Arc::new(SystemClock),
            Policy::default(),
        );
        let fixture = Self {
            core,
            adapter,
            vault,
            store,
            secrets,
            wire,
            path,
        };
        fixture.account("FIRST", inventory).await;
        fixture
    }
    async fn account(&self, token: &str, inventory: Vec<Value>) -> Uuid {
        let session = MeijuSession::restore(&bytes(json!({"version":1,"subject":format!("meiju.login-id.v1:{token}"),"token":token,"key":"synthetic-key-01"}))).unwrap();
        let account = MeijuAccount::create(
            &self.vault,
            "CN",
            "Synthetic",
            session.subject(),
            &session.encode().unwrap(),
            &operation(),
        )
        .await
        .unwrap();
        self.wire
            .inventories
            .lock()
            .unwrap()
            .insert(token.into(), inventory);
        account.record().id
    }
    async fn discover(&self) -> Vec<Device> {
        self.core.discover(&operation()).await.unwrap()
    }
    async fn read(&self, device: &Device, component: &str, cached: bool) -> Result<Snapshot> {
        self.core
            .read(&device.id, component, Route::Cloud, cached, &operation())
            .await
    }
    fn calls(&self) -> usize {
        self.wire.calls.load(Ordering::Relaxed)
    }
}

#[tokio::test]
async fn six_models_publish_read_only_scopes_and_pass_core_validation() {
    let fixture = Fixture::new(appliances()).await;
    assert_eq!(fixture.calls(), 0);
    assert!(fixture.core.list().unwrap().is_empty());
    let devices = fixture.discover().await;
    assert_eq!(devices.len(), 6);
    assert_eq!(fixture.calls(), 2);
    let mut snapshots = 0;
    for device in &devices {
        assert_eq!(device.support_status, "experimental");
        assert_eq!(device.brand, "Unknown");
        assert_eq!(
            device.components.len(),
            if device.model == MODELS[4].0 { 2 } else { 1 }
        );
        let descriptor = fixture.core.describe(&device.id).unwrap();
        for cap in descriptor.capabilities {
            assert!(cap.actions.is_empty() && cap.events.is_empty());
            assert!(cap.properties.values().all(|p| p["readable"] == true
                && p["writable"] == false
                && p.get("enum").is_none()
                && p.get("minimum").is_none()));
        }
        for component in &device.components {
            let snapshot = fixture.read(device, &component.id, false).await.unwrap();
            snapshots += 1;
            assert!(!snapshot.observations.is_empty());
            assert!(
                snapshot
                    .observations
                    .values()
                    .all(|v| v.stale && v.observed_at.is_none())
            );
            if device.model == MODELS[0].0 {
                assert_eq!(
                    snapshot.observations["vendor.midea.ac.wind_speed"].value,
                    json!(102)
                );
            }
            let cached = fixture.read(device, &component.id, true).await.unwrap();
            assert!(cached.from_cache);
            for (key, value) in &cached.observations {
                assert_eq!(value.received_at, snapshot.observations[key].received_at);
            }
        }
    }
    assert_eq!(snapshots, 7);
    assert_eq!(fixture.calls(), 9);
    let public = serde_json::to_string(&fixture.core.list_device_cards().unwrap()).unwrap();
    for forbidden in [
        "88000000000000",
        "FIRST",
        "synthetic-key-01",
        "meiju.login-id",
    ] {
        assert!(!public.contains(forbidden));
    }
    let device = devices.iter().find(|d| d.model == MODELS[0].0).unwrap();
    let mut command: Command =
        serde_json::from_str(include_str!("../../../examples/command.json")).unwrap();
    command.device_id = device.id.clone();
    assert_eq!(
        fixture.core.plan(&command).unwrap_err().code,
        "CAPABILITY_UNSUPPORTED"
    );
    assert_eq!(fixture.calls(), 9);
    assert!(fixture.core.recent_executions().unwrap().is_empty());
}

#[tokio::test]
async fn unknown_missing_and_wrong_family_models_are_visible_without_read_admission() {
    let fixture = Fixture::new(vec![
        json!({"applianceCode":"1","type":"AC","productModel":"Unreviewed"}),
        json!({"applianceCode":"2","type":"AC"}),
        json!({"applianceCode":"3","type":"E1","productModel":MODELS[0].0}),
        json!({"applianceCode":"4","type":"AC","productModel":format!("{} extra", MODELS[0].0)}),
    ])
    .await;
    let devices = fixture.discover().await;
    assert_eq!(devices.len(), 4);
    for device in &devices {
        assert_eq!(device.support_status, "unsupported");
        let descriptor = fixture.core.describe(&device.id).unwrap();
        assert!(descriptor.capabilities[0].properties.is_empty());
        assert_eq!(
            fixture.read(device, "main", false).await.unwrap_err().code,
            "CAPABILITY_UNSUPPORTED"
        );
        let mut forged = device.clone();
        forged.model = MODELS[0].0.into();
        forged.support_status = "experimental".into();
        assert_eq!(
            fixture
                .adapter
                .read(&forged, "main", &forged.connections[0], &operation())
                .await
                .unwrap_err()
                .code,
            "DEVICE_IDENTITY_CONFLICT"
        );
    }
    assert_eq!(fixture.calls(), 2);
}

#[tokio::test]
async fn repeated_partial_and_empty_discovery_preserve_cards_preferences_and_ids() {
    let fixture = Fixture::new(appliances()).await;
    let devices = fixture.discover().await;
    let room = fixture.core.create_room("Kitchen").unwrap();
    fixture
        .core
        .set_device_preferences(
            &devices[0].id,
            PreferencesInput {
                alias: Some("My appliance".into()),
                room_id: Some(room.id),
                favorite: true,
            },
        )
        .unwrap();
    let baseline = serde_json::to_value(fixture.core.list_device_cards().unwrap()).unwrap();
    fixture.discover().await;
    assert_eq!(
        serde_json::to_value(fixture.core.list_device_cards().unwrap()).unwrap(),
        baseline
    );
    fixture.wire.partial.store(true, Ordering::Relaxed);
    assert_eq!(
        fixture.core.discover(&operation()).await.unwrap_err().code,
        "MIDEA_INVENTORY_PARTIAL"
    );
    assert_eq!(
        serde_json::to_value(fixture.core.list_device_cards().unwrap()).unwrap(),
        baseline
    );
    fixture.wire.partial.store(false, Ordering::Relaxed);
    fixture
        .wire
        .inventories
        .lock()
        .unwrap()
        .insert("FIRST".into(), vec![]);
    fixture.discover().await;
    assert_eq!(
        serde_json::to_value(fixture.core.list_device_cards().unwrap()).unwrap(),
        baseline
    );
}

#[tokio::test]
async fn historical_cross_account_conflicts_never_publish_duplicate_devices() {
    for hide_previous in [false, true] {
        let fixture = Fixture::new(appliances()).await;
        fixture.discover().await;
        let baseline = serde_json::to_value(fixture.core.list().unwrap()).unwrap();
        fixture
            .account("SECOND", vec![appliances()[0].clone()])
            .await;
        if hide_previous {
            fixture
                .wire
                .inventories
                .lock()
                .unwrap()
                .insert("FIRST".into(), vec![]);
        }
        assert_eq!(
            fixture.core.discover(&operation()).await.unwrap_err().code,
            "DEVICE_IDENTITY_CONFLICT"
        );
        assert_eq!(
            serde_json::to_value(fixture.core.list().unwrap()).unwrap(),
            baseline
        );
        for account in fixture.vault.list(&operation()).await.unwrap() {
            drop(fixture.vault.lease(account.id, &operation()).await.unwrap());
        }
    }
}

#[tokio::test]
async fn invalid_cloud_reads_preserve_cache_and_secret_unavailability_does_not_break_local_queries()
{
    let fixture = Fixture::new(appliances()).await;
    let device = fixture
        .discover()
        .await
        .into_iter()
        .find(|d| d.model == MODELS[0].0)
        .unwrap();
    fixture.read(&device, "main", false).await.unwrap();
    let baseline =
        serde_json::to_value(fixture.store.snapshot(&device.id, "main").unwrap()).unwrap();
    for invalid in [
        json!({"power":"off","wind_speed":"malformed"}),
        json!({"power":"off","applianceCode":"999"}),
    ] {
        *fixture.wire.state.lock().unwrap() = Some(invalid);
        assert_eq!(
            fixture.read(&device, "main", false).await.unwrap_err().code,
            "MIDEA_RESPONSE_INVALID"
        );
        assert_eq!(
            serde_json::to_value(fixture.store.snapshot(&device.id, "main").unwrap()).unwrap(),
            baseline
        );
    }
    fixture.secrets.unavailable.store(true, Ordering::Relaxed);
    assert!(
        fixture
            .read(&device, "main", true)
            .await
            .unwrap()
            .from_cache
    );
    assert_eq!(fixture.core.list().unwrap().len(), 6);
    assert_eq!(fixture.core.list_device_cards().unwrap().len(), 6);
    fixture.core.describe(&device.id).unwrap();
    assert_eq!(
        fixture.read(&device, "main", false).await.unwrap_err().code,
        "SECRET_STORE_UNAVAILABLE"
    );
    assert_eq!(fixture.calls(), 5);
}

#[tokio::test(start_paused = true)]
async fn cancellation_deadline_and_busy_leases_stop_discovery_without_publication() {
    let fixture = Fixture::new(appliances()).await;
    let cancelled = operation();
    cancelled.cancellation.cancel();
    assert_eq!(
        fixture.core.discover(&cancelled).await.unwrap_err().code,
        "CANCELLED"
    );
    let expired = operation();
    tokio::time::advance(std::time::Duration::from_secs(2)).await;
    assert_eq!(
        fixture.core.discover(&expired).await.unwrap_err().code,
        "DEADLINE_EXCEEDED"
    );
    assert_eq!(fixture.calls(), 0);
    let id = fixture.vault.list(&operation()).await.unwrap()[0].id;
    let lease = fixture.vault.lease(id, &operation()).await.unwrap();
    assert_eq!(
        fixture.core.discover(&operation()).await.unwrap_err().code,
        "RESOURCE_BUSY"
    );
    assert_eq!(fixture.calls(), 0);
    drop(lease);
    fixture.wire.delay.store(true, Ordering::Relaxed);
    let active = operation();
    {
        let work = fixture.core.discover(&active);
        tokio::pin!(work);
        tokio::select! {
            result = &mut work => panic!("Unexpected completion: {result:?}"),
            _ = fixture.wire.started.notified() => (),
        }
        assert_eq!(
            fixture
                .vault
                .lease(id, &operation())
                .await
                .err()
                .unwrap()
                .code,
            "RESOURCE_BUSY"
        );
        active.cancellation.cancel();
        assert_eq!(work.await.unwrap_err().code, "CANCELLED");
    }
    drop(fixture.vault.lease(id, &operation()).await.unwrap());
    let short = Operation::new(10, CancellationToken::new()).unwrap();
    assert_eq!(
        fixture.core.discover(&short).await.unwrap_err().code,
        "DEADLINE_EXCEEDED"
    );
    assert!(fixture.core.list().unwrap().is_empty());
    assert_eq!(fixture.calls(), 2);
    fixture.wire.delay.store(false, Ordering::Relaxed);
    assert_eq!(fixture.discover().await.len(), 6);
}
