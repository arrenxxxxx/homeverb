use async_trait::async_trait;
use homeverb_core::*;
use homeverb_domain::*;
use homeverb_midea::*;
use homeverb_storage::SqliteStore;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

const IDS: [&str; 3] = ["9007199254740993", "9007199254740994", "9007199254740995"];

fn operation() -> Operation {
    Operation::new(1000, CancellationToken::new()).unwrap()
}
fn bytes(value: Value) -> SecretBytes {
    SecretBytes::new(serde_json::to_vec(&value).unwrap())
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
                    .map(|value| SecretBytes::new(value.clone())))
            })
            .await
    }
    async fn put(
        &self,
        reference: &CredentialRef,
        value: &SecretBytes,
        operation: &Operation,
    ) -> Result<()> {
        operation
            .run(async {
                self.values
                    .lock()
                    .unwrap()
                    .insert(reference.to_string(), value.expose().to_vec());
                Ok(())
            })
            .await
    }
    async fn delete(&self, _: &CredentialRef, _: &Operation) -> Result<()> {
        panic!("Reading must never delete a credential.")
    }
}

struct Wire {
    reference: CredentialRef,
    state: Mutex<HashMap<String, Value>>,
    calls: AtomicUsize,
    delay: AtomicBool,
    started: tokio::sync::Notify,
}

#[async_trait]
impl ReadTransport for Wire {
    async fn request(
        &self,
        reference: &CredentialRef,
        endpoint: ReadEndpoint<'_>,
        _: &Operation,
    ) -> Result<SecretBytes> {
        assert_eq!(*reference, self.reference);
        let data = match endpoint {
            ReadEndpoint::Homes => json!({"homeList": [{"homegroupId": 1}]}),
            ReadEndpoint::HomeAppliances(_) => json!({"homeList": [{"roomList": [{"applianceList":
                IDS.iter().zip(["AC", "E1", "CA"]).map(|(id, family)| json!({"applianceCode": id, "type": family, "productModel": "Synthetic-Model"})).collect::<Vec<_>>()
            }]}]}),
            ReadEndpoint::ApplianceState { appliance, query } => {
                assert_eq!(query, StateQuery::Default);
                self.calls.fetch_add(1, Ordering::Relaxed);
                self.started.notify_one();
                if self.delay.load(Ordering::Relaxed) {
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                    panic!("The bounded read must drop this delayed request.");
                }
                self.state.lock().unwrap()[appliance.expose()].clone()
            }
        };
        Ok(bytes(json!({"code": 0, "data": data})))
    }
}

/// Test-only registration: reviewed model admission and live host registration remain pending.
struct ReadProbe {
    vault: Arc<AccountVault>,
    account_id: Uuid,
    wire: Arc<Wire>,
    descriptors: Vec<Descriptor>,
}
#[async_trait]
impl Adapter for ReadProbe {
    fn id(&self) -> &str {
        "midea.meiju"
    }
    async fn discover(&self, _: &Operation) -> Result<Vec<Descriptor>> {
        Ok(self.descriptors.clone())
    }
    async fn read(
        &self,
        device: &Device,
        component: &str,
        connection: &Connection,
        operation: &Operation,
    ) -> Result<Snapshot> {
        let lease = self.vault.lease(self.account_id, operation).await?;
        let account = MeijuAccount::open(lease, operation).await?;
        account
            .read_bound_state(
                self.wire.as_ref(),
                device,
                component,
                connection,
                &SystemClock,
                operation,
            )
            .await
    }
    async fn execute(
        &self,
        _: &ValidatedCommand,
        _: &Connection,
        _: WriteProgress,
        _: &Operation,
    ) -> Result<AdapterOutcome> {
        panic!("Read-only test descriptors must never permit a send.")
    }
}

fn capability(component: &str, id: &str, fields: &[(&str, &str, Option<&str>)]) -> Capability {
    Capability {
        schema_version: "1.0".into(),
        id: id.into(),
        component_id: Some(component.into()),
        revision: 1,
        properties: fields
            .iter()
            .map(|(name, kind, unit)| {
                let mut property = json!({"type": kind, "readable": true, "writable": false});
                if let Some(unit) = unit {
                    property["unit"] = json!(unit);
                }
                (name.to_string(), property)
            })
            .collect(),
        actions: BTreeMap::new(),
        events: BTreeMap::new(),
    }
}

struct Fixture {
    core: Core,
    store: Arc<SqliteStore>,
    probe: Arc<ReadProbe>,
    secrets: Arc<Secrets>,
    path: PathBuf,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
impl Fixture {
    async fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("homeverb-meiju-bound-read-{}", Uuid::new_v4()));
        let store = Arc::new(SqliteStore::open(&path).unwrap());
        let secrets = Arc::new(Secrets::default());
        let vault = Arc::new(AccountVault::new(store.clone(), secrets.clone()));
        let mut account = MeijuAccount::create(
            &vault,
            "CN",
            "Synthetic",
            &SecretBytes::new(b"SUBJECT".to_vec()),
            &SecretBytes::new(b"SESSION".to_vec()),
            &operation(),
        )
        .await
        .unwrap();
        let record = account.record().clone();
        let wire = Arc::new(Wire {
            reference: record.credential_ref,
            state: Mutex::new(IDS.into_iter().zip([
                json!({"power": "on", "temperature": 25, "small_temperature": 0.5, "indoor_temperature": 27.2}),
                json!({"doorswitch": 1, "temperature": 48, "mode": "intensive", "work_status": "run", "left_time": 13}),
                json!({"storage_temperature": 4, "refrigeration_real_temperature": 5, "freezing_temperature": -18, "freezing_real_temperature": -17}),
            ]).map(|(id, state)| (id.to_owned(), state)).collect()),
            calls: AtomicUsize::new(0), delay: AtomicBool::new(false), started: tokio::sync::Notify::new(),
        });
        let inventory = retrieve_inventory(wire.as_ref(), record.credential_ref, &operation())
            .await
            .unwrap();
        let bindings = account
            .bind_inventory(&inventory, &operation())
            .await
            .unwrap();
        let temperatures = [
            ("current_temperature", "number", Some("celsius")),
            ("target_temperature", "number", Some("celsius")),
        ];
        let definitions = [
            vec![
                capability("main", "power", &[("state", "boolean", None)]),
                capability("main", "climate", &temperatures),
            ],
            vec![capability(
                "main",
                "vendor.midea.dishwasher",
                &[
                    ("doorswitch", "integer", None),
                    ("temperature", "number", Some("celsius")),
                    ("mode", "string", None),
                    ("work_status", "string", None),
                    ("remaining_minutes", "integer", Some("minutes")),
                ],
            )],
            vec![
                capability("fridge", "climate", &temperatures),
                capability("freezer", "climate", &temperatures),
            ],
        ];
        let descriptors = bindings
            .into_iter()
            .zip(definitions)
            .map(|(binding, capabilities)| {
                let mut components = BTreeMap::<String, Vec<String>>::new();
                for cap in &capabilities {
                    components
                        .entry(cap.component_id.clone().unwrap())
                        .or_default()
                        .push(cap.id.clone());
                }
                Descriptor {
                    device: Device {
                        schema_version: "1.0".into(),
                        id: binding.device_id.to_string(),
                        alias: format!("Synthetic {}", binding.device_id),
                        brand: "Fictional".into(),
                        model: "Synthetic-Model".into(),
                        support_status: "mock-tested".into(),
                        components: components
                            .into_iter()
                            .map(|(id, capabilities)| Component { id, capabilities })
                            .collect(),
                        connections: vec![Connection {
                            id: binding.connection_id.to_string(),
                            adapter: "midea.meiju".into(),
                            transport: Transport::Cloud,
                            account_id: Some(record.id.to_string()),
                            identity_ref: binding.identity_ref.to_string(),
                        }],
                    },
                    capabilities,
                }
            })
            .collect();
        drop(account);
        let probe = Arc::new(ReadProbe {
            vault,
            account_id: record.id,
            wire,
            descriptors,
        });
        let core = Core::new(
            store.clone(),
            vec![probe.clone()],
            Arc::new(SystemClock),
            Policy::default(),
        );
        core.discover(&operation()).await.unwrap();
        Self {
            core,
            store,
            probe,
            secrets,
            path,
        }
    }
    async fn read(&self, index: usize, component: &str, cached: bool) -> Result<Snapshot> {
        self.core
            .read(
                &self.probe.descriptors[index].device.id,
                component,
                Route::Cloud,
                cached,
                &operation(),
            )
            .await
    }
    fn calls(&self) -> usize {
        self.probe.wire.calls.load(Ordering::Relaxed)
    }
}

#[tokio::test]
async fn all_three_families_pass_core_validation_without_inventing_freshness_or_write_support() {
    let fixture = Fixture::new().await;
    let ac = fixture.read(0, "main", false).await.unwrap();
    assert_eq!(ac.observations["power.state"].value, json!(true));
    assert_eq!(
        ac.observations["climate.target_temperature"].value,
        json!(25.5)
    );
    let dishwasher = fixture.read(1, "main", false).await.unwrap();
    assert_eq!(
        dishwasher.observations["vendor.midea.dishwasher.doorswitch"].value,
        json!(1)
    );
    assert!(
        !dishwasher
            .observations
            .contains_key("appliance_cycle.running")
    );
    let fridge = fixture.read(2, "fridge", false).await.unwrap();
    let freezer = fixture.read(2, "freezer", false).await.unwrap();
    assert_eq!(fridge.device_id, freezer.device_id);
    assert_eq!(
        fridge.observations["climate.target_temperature"].value,
        json!(4.0)
    );
    assert_eq!(
        freezer.observations["climate.target_temperature"].value,
        json!(-18.0)
    );
    for snapshot in [&ac, &dishwasher, &fridge, &freezer] {
        assert!(!snapshot.from_cache);
        assert!(!snapshot.warnings.is_empty());
        assert!(
            snapshot
                .observations
                .values()
                .all(|observation| observation.stale
                    && observation.observed_at.is_none()
                    && observation.source == Transport::Cloud)
        );
        let serialized = serde_json::to_string(snapshot).unwrap();
        for id in IDS {
            assert!(!serialized.contains(id));
        }
    }
    fixture.secrets.unavailable.store(true, Ordering::Relaxed);
    let cached = fixture.read(0, "main", true).await.unwrap();
    assert!(cached.from_cache);
    assert_eq!(
        cached.observations["power.state"].received_at,
        ac.observations["power.state"].received_at
    );
    assert_eq!(fixture.calls(), 4);
    assert_eq!(
        fixture.read(0, "main", false).await.unwrap_err().code,
        "SECRET_STORE_UNAVAILABLE"
    );
    assert_eq!(fixture.calls(), 4);
    let mut command: Command =
        serde_json::from_str(include_str!("../../../examples/command.json")).unwrap();
    command.device_id = ac.device_id;
    assert_eq!(
        fixture.core.plan(&command).unwrap_err().code,
        "CAPABILITY_UNSUPPORTED"
    );
    assert!(fixture.core.recent_executions().unwrap().is_empty());
}

#[tokio::test]
async fn binding_mismatch_and_invalid_components_never_start_a_cloud_request() {
    let fixture = Fixture::new().await;
    let lease = fixture
        .probe
        .vault
        .lease(fixture.probe.account_id, &operation())
        .await
        .unwrap();
    let account = MeijuAccount::open(lease, &operation()).await.unwrap();
    let original = &fixture.probe.descriptors[0].device;
    for kind in 0..9 {
        let mut device = original.clone();
        let mut connection = device.connections[0].clone();
        match kind {
            0 => device.id = Uuid::new_v4().to_string(),
            1 => connection.id = Uuid::new_v4().to_string(),
            2 => connection.identity_ref = Uuid::new_v4().to_string(),
            3 => connection.account_id = Some(Uuid::new_v4().to_string()),
            4 => connection.account_id = None,
            5 => connection.transport = Transport::Lan,
            6 => connection.adapter = "another.provider".into(),
            7 => device.connections.clear(),
            _ => device.model = "Other model".into(),
        }
        assert_eq!(
            account
                .read_bound_state(
                    fixture.probe.wire.as_ref(),
                    &device,
                    "main",
                    &connection,
                    &SystemClock,
                    &operation()
                )
                .await
                .unwrap_err()
                .code,
            "DEVICE_IDENTITY_CONFLICT"
        );
    }
    for component in ["freezer", "unknown"] {
        assert_eq!(
            account
                .read_bound_state(
                    fixture.probe.wire.as_ref(),
                    original,
                    component,
                    &original.connections[0],
                    &SystemClock,
                    &operation()
                )
                .await
                .unwrap_err()
                .code,
            "CAPABILITY_UNSUPPORTED"
        );
    }
    let mut device = original.clone();
    device.components.clear();
    assert_eq!(
        account
            .read_bound_state(
                fixture.probe.wire.as_ref(),
                &device,
                "main",
                &device.connections[0],
                &SystemClock,
                &operation()
            )
            .await
            .unwrap_err()
            .code,
        "CAPABILITY_UNSUPPORTED"
    );
    assert_eq!(fixture.calls(), 0);
}

#[tokio::test]
async fn invalid_cloud_evidence_preserves_core_cache_without_retry_or_partial_publication() {
    let fixture = Fixture::new().await;
    fixture.read(0, "main", false).await.unwrap();
    let id = &fixture.probe.descriptors[0].device.id;
    let original = serde_json::to_value(fixture.store.snapshot(id, "main").unwrap()).unwrap();
    for (state, error) in [
        (
            json!({"power": "off", "mode": "cool"}),
            "INVALID_OBSERVATION",
        ),
        (
            json!({"power": "off", "wind_speed": "malformed"}),
            "MIDEA_RESPONSE_INVALID",
        ),
        (
            json!({"power": "off", "applianceCode": "999"}),
            "MIDEA_RESPONSE_INVALID",
        ),
    ] {
        fixture
            .probe
            .wire
            .state
            .lock()
            .unwrap()
            .insert(IDS[0].into(), state);
        assert_eq!(
            fixture.read(0, "main", false).await.unwrap_err().code,
            error
        );
        assert_eq!(
            serde_json::to_value(fixture.store.snapshot(id, "main").unwrap()).unwrap(),
            original
        );
    }
    assert_eq!(fixture.calls(), 4);
    assert!(fixture.core.recent_executions().unwrap().is_empty());
}

#[tokio::test(start_paused = true)]
async fn cancellation_expiry_and_pending_credentials_stop_reads_and_release_the_account_lease() {
    let fixture = Fixture::new().await;
    let id = &fixture.probe.descriptors[0].device.id;
    let cancelled = operation();
    cancelled.cancellation.cancel();
    assert_eq!(
        fixture
            .core
            .read(id, "main", Route::Cloud, false, &cancelled)
            .await
            .unwrap_err()
            .code,
        "CANCELLED"
    );
    let expired = operation();
    tokio::time::advance(std::time::Duration::from_secs(2)).await;
    assert_eq!(
        fixture
            .core
            .read(id, "main", Route::Cloud, false, &expired)
            .await
            .unwrap_err()
            .code,
        "DEADLINE_EXCEEDED"
    );
    assert_eq!(fixture.calls(), 0);
    fixture.probe.wire.delay.store(true, Ordering::Relaxed);
    let active = operation();
    {
        let read = fixture.core.read(id, "main", Route::Cloud, false, &active);
        tokio::pin!(read);
        tokio::select! {
            result = &mut read => panic!("Unexpected early completion: {result:?}"),
            _ = fixture.probe.wire.started.notified() => (),
        }
        assert_eq!(
            fixture
                .probe
                .vault
                .lease(fixture.probe.account_id, &operation())
                .await
                .err()
                .unwrap()
                .code,
            "RESOURCE_BUSY"
        );
        active.cancellation.cancel();
        assert_eq!(read.await.unwrap_err().code, "CANCELLED");
    }
    let lease = fixture
        .probe
        .vault
        .lease(fixture.probe.account_id, &operation())
        .await
        .unwrap();
    drop(lease);
    assert!(fixture.store.snapshot(id, "main").unwrap().is_none());
    let during_request = Operation::new(10, CancellationToken::new()).unwrap();
    assert_eq!(
        fixture
            .core
            .read(id, "main", Route::Cloud, false, &during_request)
            .await
            .unwrap_err()
            .code,
        "DEADLINE_EXCEEDED"
    );
    drop(
        fixture
            .probe
            .vault
            .lease(fixture.probe.account_id, &operation())
            .await
            .unwrap(),
    );
    fixture.probe.wire.delay.store(false, Ordering::Relaxed);
    fixture
        .store
        .set_credential_state(fixture.probe.account_id, CredentialState::Pending)
        .unwrap();
    assert_eq!(
        fixture.read(0, "main", false).await.unwrap_err().code,
        "ACCOUNT_SESSION_UNCONFIRMED"
    );
    assert_eq!(fixture.calls(), 2);
}
