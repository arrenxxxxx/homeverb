use async_trait::async_trait;
use homeverb_core::*;
use homeverb_domain::*;
use homeverb_storage::SqliteStore;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio_util::sync::CancellationToken;

const DEVICE: &str = "dev-example-fridge";

fn descriptor() -> Descriptor {
    Descriptor {
        device: serde_json::from_str(include_str!("../../../examples/device.refrigerator.json"))
            .unwrap(),
        capabilities: vec![
            serde_json::from_str(include_str!("../../../examples/capability.fridge.json")).unwrap(),
            serde_json::from_str(include_str!("../../../examples/capability.freezer.json"))
                .unwrap(),
        ],
    }
}

struct CompartmentAdapter {
    descriptor: Mutex<Descriptor>,
    values: Mutex<BTreeMap<String, Value>>,
    reads: AtomicUsize,
    sends: AtomicUsize,
}

#[async_trait]
impl Adapter for CompartmentAdapter {
    fn id(&self) -> &str {
        "homeverb.mock"
    }
    async fn discover(&self, _: &Operation) -> Result<Vec<Descriptor>> {
        Ok(vec![self.descriptor.lock().unwrap().clone()])
    }
    async fn read(
        &self,
        device: &Device,
        component: &str,
        connection: &Connection,
        _: &Operation,
    ) -> Result<Snapshot> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        Ok(Snapshot {
            schema_version: SCHEMA_VERSION.into(),
            device_id: device.id.clone(),
            component_id: component.into(),
            from_cache: false,
            warnings: vec![],
            observations: BTreeMap::from([(
                "climate.target_temperature".into(),
                Observation {
                    value: self.values.lock().unwrap()[component].clone(),
                    source: connection.transport,
                    observed_at: None,
                    received_at: chrono::Utc::now(),
                    stale: false,
                },
            )]),
        })
    }
    async fn execute(
        &self,
        command: &ValidatedCommand,
        _: &Connection,
        progress: WriteProgress,
        _: &Operation,
    ) -> Result<AdapterOutcome> {
        progress.mark_sent();
        self.sends.fetch_add(1, Ordering::SeqCst);
        let command = command.command();
        self.values.lock().unwrap().insert(
            command.component_id.clone(),
            command.arguments["target_temperature"].clone(),
        );
        progress.mark_accepted();
        Ok(AdapterOutcome {
            phase: Phase::Accepted,
            error: None,
            completion: None,
        })
    }
}

struct Harness {
    path: PathBuf,
    core: Core,
    store: Arc<SqliteStore>,
    adapter: Arc<CompartmentAdapter>,
}
impl Harness {
    fn empty() -> Self {
        let path =
            std::env::temp_dir().join(format!("homeverb-components-{}", uuid::Uuid::new_v4()));
        let store = Arc::new(SqliteStore::open(&path).unwrap());
        let adapter = Arc::new(CompartmentAdapter {
            descriptor: Mutex::new(descriptor()),
            values: Mutex::new(BTreeMap::from([
                ("fridge".into(), json!(4)),
                ("freezer".into(), json!(-18)),
            ])),
            reads: AtomicUsize::new(0),
            sends: AtomicUsize::new(0),
        });
        let core = Core::new(
            store.clone(),
            vec![adapter.clone()],
            Arc::new(SystemClock),
            Policy {
                grants: ["fridge", "freezer"].map(grant).into(),
            },
        );
        Self {
            path,
            core,
            store,
            adapter,
        }
    }
    async fn new() -> Self {
        let h = Self::empty();
        h.core.discover(&operation()).await.unwrap();
        h
    }
    async fn read(&self, component: &str, cached: bool) -> Result<Snapshot> {
        self.core
            .read(DEVICE, component, Route::Lan, cached, &operation())
            .await
    }
}
impl Drop for Harness {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
fn operation() -> Operation {
    Operation::new(10_000, CancellationToken::new()).unwrap()
}
fn grant(component: &str) -> Grant {
    Grant {
        device_id: DEVICE.into(),
        component_id: component.into(),
        capability: "climate".into(),
        action: "set".into(),
        transports: vec![Transport::Lan],
        argument_schema: json!({"type":"object"}),
    }
}
fn command(component: &str, temperature: Value) -> Command {
    Command {
        schema_version: SCHEMA_VERSION.into(),
        request_id: uuid::Uuid::new_v4().to_string(),
        device_id: DEVICE.into(),
        component_id: component.into(),
        capability: "climate".into(),
        action: "set".into(),
        arguments: json!({"target_temperature":temperature}),
        connection: Route::Lan,
        timeout_ms: 10_000,
        wait_for: WaitFor::Verified,
    }
}

#[tokio::test]
async fn one_device_exposes_distinct_limits_and_verifies_each_component_independently() {
    let h = Harness::new().await;
    assert_eq!(h.core.list().unwrap().len(), 1);
    for (component, temperature) in [("fridge", json!(4.5)), ("freezer", json!(-20))] {
        let command = command(component, temperature.clone());
        assert!(h.core.plan(&command).is_ok());
        let result = h.core.execute(command, CancellationToken::new()).await;
        assert_eq!(result.status, Status::Verified, "{result:?}");
        assert_eq!(result.device_id, DEVICE);
        assert_eq!(
            h.read(component, true).await.unwrap().observations["climate.target_temperature"].value,
            temperature
        );
    }
    assert_eq!(h.adapter.sends.load(Ordering::SeqCst), 2);
    for (component, temperature) in [
        ("fridge", json!(-20)),
        ("freezer", json!(4.5)),
        ("freezer", json!(-18.5)),
    ] {
        let result = h
            .core
            .execute(command(component, temperature), CancellationToken::new())
            .await;
        assert_eq!(result.error.unwrap().code, "INVALID_ARGUMENT");
        assert!(result.attempts.is_empty());
    }
    assert_eq!(h.adapter.sends.load(Ordering::SeqCst), 2);
    let reopened = SqliteStore::open(&h.path).unwrap();
    assert_eq!(
        reopened.devices().unwrap()[0].capabilities[0]
            .component_id
            .as_deref(),
        Some("fridge")
    );
    assert_eq!(
        reopened
            .snapshot(DEVICE, "freezer")
            .unwrap()
            .unwrap()
            .observations["climate.target_temperature"]
            .value,
        -20
    );
}

#[tokio::test]
async fn a_shared_power_definition_can_coexist_with_scoped_climate_definitions() {
    let h = Harness::empty();
    {
        let mut descriptor = h.adapter.descriptor.lock().unwrap();
        descriptor.capabilities.push(
            homeverb_mock::MockAdapter::descriptor()
                .capabilities
                .remove(0),
        );
        for component in &mut descriptor.device.components {
            component.capabilities.push("power".into());
        }
    }
    h.core.discover(&operation()).await.unwrap();
    let descriptor = h.core.describe(DEVICE).unwrap();
    assert!(std::ptr::eq(
        descriptor.capability("fridge", "power").unwrap(),
        descriptor.capability("freezer", "power").unwrap()
    ));
    assert_ne!(
        descriptor
            .capability("fridge", "climate")
            .unwrap()
            .component_id,
        descriptor
            .capability("freezer", "climate")
            .unwrap()
            .component_id
    );
    assert!(h.read("freezer", false).await.is_ok());
    let mut request = command("freezer", json!(-18));
    request.capability = "power".into();
    request.arguments = json!({"state":true});
    assert_eq!(h.core.plan(&request).unwrap_err().code, "POLICY_DENIED");
    assert_eq!(h.adapter.sends.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn component_grants_routes_and_action_risk_remain_independent() {
    let h = Harness::new().await;
    let limited = Core::new(
        h.store.clone(),
        vec![h.adapter.clone()],
        Arc::new(SystemClock),
        Policy {
            grants: vec![grant("fridge")],
        },
    );
    assert_eq!(
        limited
            .execute(command("freezer", json!(-18)), CancellationToken::new())
            .await
            .error
            .unwrap()
            .code,
        "POLICY_DENIED"
    );
    assert_eq!(h.adapter.sends.load(Ordering::SeqCst), 0);
    let mut descriptor = descriptor();
    descriptor.capabilities[1]
        .actions
        .get_mut("set")
        .unwrap()
        .risk = Risk::High;
    h.store.save_device(&descriptor).unwrap();
    assert_eq!(
        h.core
            .execute(command("freezer", json!(-18)), CancellationToken::new())
            .await
            .error
            .unwrap()
            .code,
        "POLICY_DENIED"
    );
    assert_eq!(
        h.core
            .execute(command("fridge", json!(5)), CancellationToken::new())
            .await
            .status,
        Status::Verified
    );
    let mut request = command("fridge", json!(5));
    request.connection = Route::Cloud;
    assert_eq!(
        h.core
            .execute(request, CancellationToken::new())
            .await
            .error
            .unwrap()
            .code,
        "POLICY_DENIED"
    );
    assert_eq!(h.adapter.sends.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn live_and_cached_observations_use_the_selected_component_constraints_and_freshness() {
    let h = Harness::new().await;
    for (component, stale) in [("fridge", true), ("freezer", false)] {
        let mut snapshot = h.read(component, false).await.unwrap();
        snapshot
            .observations
            .get_mut("climate.target_temperature")
            .unwrap()
            .received_at -= chrono::Duration::seconds(30);
        h.store.save_snapshot(&snapshot).unwrap();
        assert_eq!(
            h.read(component, true).await.unwrap().observations["climate.target_temperature"].stale,
            stale
        );
        assert!(
            !h.store
                .snapshot(DEVICE, component)
                .unwrap()
                .unwrap()
                .observations["climate.target_temperature"]
                .stale
        );
    }
    h.adapter
        .values
        .lock()
        .unwrap()
        .insert("fridge".into(), json!(-18));
    assert_eq!(
        h.read("fridge", false).await.unwrap_err().code,
        "INVALID_OBSERVATION"
    );
    assert_eq!(
        h.store
            .snapshot(DEVICE, "fridge")
            .unwrap()
            .unwrap()
            .observations["climate.target_temperature"]
            .value,
        4
    );
    let mut snapshot = h.store.snapshot(DEVICE, "freezer").unwrap().unwrap();
    snapshot
        .observations
        .get_mut("climate.target_temperature")
        .unwrap()
        .value = json!(4);
    h.store.save_snapshot(&snapshot).unwrap();
    assert_eq!(
        h.read("freezer", true).await.unwrap_err().code,
        "INVALID_OBSERVATION"
    );
}

#[tokio::test]
async fn invalid_scopes_and_ambiguous_definitions_never_replace_valid_discovery_metadata() {
    let h = Harness::new().await;
    let before = serde_json::to_value(h.store.devices().unwrap()).unwrap();
    let mutations: &[fn(&mut Descriptor)] = &[
        |d| d.capabilities[1].component_id = None,
        |d| d.capabilities.push(d.capabilities[0].clone()),
        |d| {
            d.capabilities.pop();
        },
        |d| d.capabilities[1].component_id = Some("missing".into()),
        |d| d.capabilities[1].component_id = Some("fridge".into()),
        |d| d.capabilities[1].id = "vendor.example.unreferenced".into(),
        |d| d.capabilities[1].component_id = Some(String::new()),
    ];
    for mutate in mutations {
        let mut descriptor = descriptor();
        mutate(&mut descriptor);
        *h.adapter.descriptor.lock().unwrap() = descriptor;
        assert_eq!(
            h.core.discover(&operation()).await.unwrap_err().code,
            "CAPABILITY_UNSUPPORTED"
        );
        assert_eq!(
            serde_json::to_value(h.store.devices().unwrap()).unwrap(),
            before
        );
    }
    assert_eq!(h.adapter.sends.load(Ordering::SeqCst), 0);
    assert_eq!(h.adapter.reads.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn invalid_stored_scope_blocks_operations_until_valid_rediscovery() {
    let h = Harness::new().await;
    let mut invalid = descriptor();
    invalid.capabilities[1].component_id = None;
    h.store.save_device(&invalid).unwrap();
    assert_eq!(
        h.core.describe(DEVICE).unwrap_err().code,
        "CAPABILITY_UNSUPPORTED"
    );
    assert_eq!(
        h.read("fridge", false).await.unwrap_err().code,
        "CAPABILITY_UNSUPPORTED"
    );
    assert_eq!(
        h.core
            .execute(command("fridge", json!(4)), CancellationToken::new())
            .await
            .error
            .unwrap()
            .code,
        "CAPABILITY_UNSUPPORTED"
    );
    assert_eq!(h.adapter.reads.load(Ordering::SeqCst), 0);
    assert_eq!(h.adapter.sends.load(Ordering::SeqCst), 0);
    assert!(h.store.recent_executions().unwrap().is_empty());
    h.core.discover(&operation()).await.unwrap();
    assert!(h.read("fridge", false).await.is_ok());
}

#[test]
fn resolver_preserves_shared_legacy_definitions_and_refuses_ambiguous_scope() {
    let mut descriptor = descriptor();
    assert_eq!(
        descriptor
            .capability("freezer", "climate")
            .unwrap()
            .properties["target_temperature"]["maximum"],
        -16
    );
    assert!(descriptor.capability("missing", "climate").is_none());
    assert!(descriptor.capability("fridge", "power").is_none());
    descriptor.capabilities[0].component_id = None;
    assert!(descriptor.capability("fridge", "climate").is_none());
    assert!(descriptor.capability("freezer", "climate").is_none());
    descriptor.capabilities.pop();
    let shared = descriptor.capability("fridge", "climate").unwrap();
    assert!(std::ptr::eq(
        shared,
        descriptor.capability("freezer", "climate").unwrap()
    ));
    assert!(
        serde_json::to_value(shared)
            .unwrap()
            .get("component_id")
            .is_none()
    );
    descriptor
        .capabilities
        .push(descriptor.capabilities[0].clone());
    assert!(descriptor.capability("fridge", "climate").is_none());
}

#[test]
fn explicit_null_or_invalid_scope_types_cannot_deserialize_as_shared_definitions() {
    for scope in [json!(null), json!(false), json!(1), json!([])] {
        let mut value = serde_json::to_value(&descriptor().capabilities[0]).unwrap();
        value["component_id"] = scope;
        assert!(serde_json::from_value::<Capability>(value).is_err());
    }
}
