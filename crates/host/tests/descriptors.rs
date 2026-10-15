use async_trait::async_trait;
use homeverb_core::*;
use homeverb_domain::*;
use homeverb_mock::{DEVICE_ID, MockAdapter, Scenario};
use homeverb_storage::SqliteStore;
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, atomic::Ordering},
};
use tokio_util::sync::CancellationToken;

struct DiscoveryAdapter {
    id: &'static str,
    inner: Arc<MockAdapter>,
    descriptors: Mutex<Vec<Descriptor>>,
}

#[async_trait]
impl Adapter for DiscoveryAdapter {
    fn id(&self) -> &str {
        self.id
    }
    async fn discover(&self, _: &Operation) -> Result<Vec<Descriptor>> {
        Ok(self.descriptors.lock().unwrap().clone())
    }
    async fn read(
        &self,
        device: &Device,
        component: &str,
        connection: &Connection,
        operation: &Operation,
    ) -> Result<Snapshot> {
        self.inner
            .read(device, component, connection, operation)
            .await
    }
    async fn execute(
        &self,
        command: &ValidatedCommand,
        connection: &Connection,
        progress: WriteProgress,
        operation: &Operation,
    ) -> Result<AdapterOutcome> {
        self.inner
            .execute(command, connection, progress, operation)
            .await
    }
}

struct Fixture {
    directory: PathBuf,
    core: Core,
    adapter: Arc<DiscoveryAdapter>,
    store: Arc<SqliteStore>,
}

impl Fixture {
    async fn new() -> Self {
        let fixture = Self::empty();
        fixture.core.discover(&operation()).await.unwrap();
        fixture
    }

    fn empty() -> Self {
        let directory =
            std::env::temp_dir().join(format!("homeverb-descriptors-{}", uuid::Uuid::new_v4()));
        let store = Arc::new(SqliteStore::open(&directory).unwrap());
        let clock = Arc::new(SystemClock);
        let inner = Arc::new(
            MockAdapter::open(
                &directory.join("mock.sqlite3"),
                clock.clone(),
                Scenario::Normal,
            )
            .unwrap(),
        );
        let adapter = Arc::new(DiscoveryAdapter {
            id: "homeverb.mock",
            inner,
            descriptors: Mutex::new(vec![MockAdapter::descriptor()]),
        });
        let core = Core::new(
            store.clone(),
            vec![adapter.clone()],
            clock,
            Policy {
                grants: vec![Grant {
                    device_id: DEVICE_ID.into(),
                    component_id: "main".into(),
                    capability: "power".into(),
                    action: "set".into(),
                    transports: vec![Transport::Lan, Transport::Cloud],
                    argument_schema: json!({"type": "object"}),
                }],
            },
        );
        Self {
            directory,
            core,
            adapter,
            store,
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

fn operation() -> Operation {
    Operation::new(10000, CancellationToken::new()).unwrap()
}
fn command() -> Command {
    Command {
        schema_version: SCHEMA_VERSION.into(),
        request_id: uuid::Uuid::new_v4().to_string(),
        device_id: DEVICE_ID.into(),
        component_id: "main".into(),
        capability: "power".into(),
        action: "set".into(),
        arguments: json!({"state": true}),
        connection: Route::Auto,
        timeout_ms: 10000,
        wait_for: WaitFor::Verified,
    }
}

fn invalid_descriptors() -> Vec<(&'static str, Descriptor)> {
    type Mutation = (&'static str, fn(&mut Descriptor));
    let mutations: &[Mutation] = &[
        ("device version", |d| d.device.schema_version = "9.0".into()),
        ("empty identity", |d| d.device.id.clear()),
        ("empty connections", |d| d.device.connections.clear()),
        ("duplicate connection", |d| {
            d.device.connections.push(d.device.connections[0].clone())
        }),
        ("duplicate component", |d| {
            d.device.components.push(d.device.components[0].clone())
        }),
        ("missing capability", |d| {
            d.capabilities.pop();
        }),
        ("duplicate capability", |d| {
            d.capabilities.push(d.capabilities[0].clone())
        }),
        ("duplicate reference", |d| {
            d.device.components[0].capabilities.push("power".into())
        }),
        ("capability version", |d| {
            d.capabilities[0].schema_version = "9.0".into()
        }),
        ("zero revision", |d| d.capabilities[0].revision = 0),
        ("malformed property", |d| {
            d.capabilities[0].properties.get_mut("state").unwrap()["type"] = json!("unknown")
        }),
        ("zero freshness", |d| {
            d.capabilities[0].properties.get_mut("state").unwrap()["max_age_ms"] = json!(0)
        }),
        ("negative freshness", |d| {
            d.capabilities[0].properties.get_mut("state").unwrap()["max_age_ms"] = json!(-1)
        }),
        ("fractional freshness", |d| {
            d.capabilities[0].properties.get_mut("state").unwrap()["max_age_ms"] = json!(0.5)
        }),
        ("oversized freshness", |d| {
            d.capabilities[0].properties.get_mut("state").unwrap()["max_age_ms"] =
                json!(9007199254740992_i64)
        }),
        ("string freshness", |d| {
            d.capabilities[0].properties.get_mut("state").unwrap()["max_age_ms"] = json!("1000")
        }),
        ("null freshness", |d| {
            d.capabilities[0].properties.get_mut("state").unwrap()["max_age_ms"] = json!(null)
        }),
        ("boolean freshness", |d| {
            d.capabilities[0].properties.get_mut("state").unwrap()["max_age_ms"] = json!(true)
        }),
        ("malformed action schema", |d| {
            d.capabilities[0].actions.get_mut("set").unwrap().parameters =
                json!({"type": "unknown"})
        }),
        ("external action schema", |d| {
            d.capabilities[0].actions.get_mut("set").unwrap().parameters =
                json!({"$ref": "https://example.invalid/schema"})
        }),
        ("duplicate transport", |d| {
            d.capabilities[0]
                .actions
                .get_mut("set")
                .unwrap()
                .transports
                .push(Transport::Lan)
        }),
        ("empty action routes", |d| {
            d.capabilities[0]
                .actions
                .get_mut("set")
                .unwrap()
                .transports
                .clear()
        }),
        ("atomic action with independent fields", |d| {
            d.capabilities[0]
                .actions
                .get_mut("set")
                .unwrap()
                .independent_fields = true
        }),
        ("non-idempotent action with independent fields", |d| {
            d.capabilities[1]
                .actions
                .get_mut("set")
                .unwrap()
                .idempotency = Idempotency::NonIdempotent
        }),
        ("unknown action with independent fields", |d| {
            d.capabilities[1]
                .actions
                .get_mut("set")
                .unwrap()
                .idempotency = Idempotency::Unknown
        }),
        ("read-only action with independent fields", |d| {
            d.capabilities[1]
                .actions
                .get_mut("set")
                .unwrap()
                .idempotency = Idempotency::ReadOnly
        }),
    ];
    mutations
        .iter()
        .map(|(name, mutate)| {
            let mut descriptor = MockAdapter::descriptor();
            mutate(&mut descriptor);
            (*name, descriptor)
        })
        .collect()
}

#[tokio::test]
async fn rediscovery_rejects_identity_changes_before_saving_any_batch_member() {
    let h = Fixture::new().await;
    assert_eq!(
        h.core
            .execute(command(), CancellationToken::new())
            .await
            .status,
        Status::Verified
    );
    h.core
        .set_device_preferences(
            DEVICE_ID,
            PreferencesInput {
                alias: Some("Preserved device".into()),
                room_id: None,
                favorite: true,
            },
        )
        .unwrap();
    let devices = serde_json::to_value(h.store.devices().unwrap()).unwrap();
    let preferences = h.store.preferences().unwrap();
    let snapshot = serde_json::to_value(h.store.snapshot(DEVICE_ID, "main").unwrap()).unwrap();
    let history = serde_json::to_value(h.store.recent_executions().unwrap()).unwrap();
    type Mutation = (&'static str, fn(&mut Descriptor));
    let mutations: &[Mutation] = &[
        ("identity reference", |d| {
            d.device.connections[0].identity_ref = "identity:another-appliance".into()
        }),
        ("account", |d| {
            d.device.connections[1].account_id = Some("another-account".into())
        }),
        ("transport", |d| {
            d.device.connections[0].transport = Transport::Cloud
        }),
        ("connection ID", |d| {
            d.device.connections[0].id = "another-connection".into()
        }),
        ("removed connection", |d| {
            d.device.connections.pop();
        }),
        ("added connection", |d| {
            let mut extra = d.device.connections[0].clone();
            extra.id = "extra-connection".into();
            d.device.connections.push(extra);
        }),
    ];
    for (name, mutate) in mutations {
        let mut conflicting = MockAdapter::descriptor();
        mutate(&mut conflicting);
        let mut new = MockAdapter::descriptor();
        new.device.id = "new-device".into();
        *h.adapter.descriptors.lock().unwrap() = vec![new, conflicting];
        let result = h.core.discover(&operation()).await;
        assert!(result.is_err(), "Rediscovery replaced identity: {name}");
        assert_eq!(
            result.unwrap_err().code,
            "DEVICE_IDENTITY_CONFLICT",
            "{name}"
        );
        assert_eq!(
            serde_json::to_value(h.store.devices().unwrap()).unwrap(),
            devices,
            "{name}"
        );
        assert_eq!(h.store.preferences().unwrap(), preferences);
        assert_eq!(
            serde_json::to_value(h.store.snapshot(DEVICE_ID, "main").unwrap()).unwrap(),
            snapshot
        );
        assert_eq!(
            serde_json::to_value(h.store.recent_executions().unwrap()).unwrap(),
            history
        );
    }
    assert_eq!(h.adapter.inner.sends.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn another_adapter_cannot_take_over_an_existing_device_id() {
    let h = Fixture::new().await;
    let original = serde_json::to_value(h.store.devices().unwrap()).unwrap();
    let mut descriptor = MockAdapter::descriptor();
    for connection in &mut descriptor.device.connections {
        connection.adapter = "another.adapter".into();
    }
    let adapter = Arc::new(DiscoveryAdapter {
        id: "another.adapter",
        inner: h.adapter.inner.clone(),
        descriptors: Mutex::new(vec![descriptor]),
    });
    let other = Core::new(
        h.store.clone(),
        vec![adapter],
        Arc::new(SystemClock),
        Policy::default(),
    );
    let result = other.discover(&operation()).await;
    assert!(
        result.is_err(),
        "Another adapter replaced the granted device identity"
    );
    assert_eq!(result.unwrap_err().code, "DEVICE_IDENTITY_CONFLICT");
    assert_eq!(
        serde_json::to_value(h.store.devices().unwrap()).unwrap(),
        original
    );
    assert_eq!(
        h.core
            .execute(command(), CancellationToken::new())
            .await
            .status,
        Status::Verified
    );
}

#[tokio::test]
async fn rediscovery_allows_reordered_connections_and_updated_capabilities() {
    let h = Fixture::new().await;
    let mut descriptor = MockAdapter::descriptor();
    descriptor.device.alias = "Updated label".into();
    descriptor.device.connections.reverse();
    descriptor.capabilities[0].revision += 1;
    *h.adapter.descriptors.lock().unwrap() = vec![descriptor];
    h.core.discover(&operation()).await.unwrap();
    let stored = h.core.describe(DEVICE_ID).unwrap();
    assert_eq!(stored.device.alias, "Updated label");
    assert_eq!(stored.device.connections[0].transport, Transport::Cloud);
    assert_eq!(stored.capabilities[0].revision, 2);
    assert_eq!(
        h.core
            .execute(command(), CancellationToken::new())
            .await
            .status,
        Status::Verified
    );
}

#[tokio::test]
async fn discovery_validates_the_whole_adapter_batch_before_saving_any_descriptor() {
    let h = Fixture::new().await;
    let original = serde_json::to_value(h.store.devices().unwrap()).unwrap();
    for (name, invalid) in invalid_descriptors() {
        let mut valid = MockAdapter::descriptor();
        valid.device.id = "new-device".into();
        *h.adapter.descriptors.lock().unwrap() = vec![valid, invalid];
        let result = h.core.discover(&operation()).await;
        assert!(result.is_err(), "{name}");
        assert_eq!(result.unwrap_err().code, "CAPABILITY_UNSUPPORTED", "{name}");
        assert_eq!(
            serde_json::to_value(h.store.devices().unwrap()).unwrap(),
            original,
            "{name}"
        );
    }
    assert_eq!(h.adapter.inner.sends.load(Ordering::SeqCst), 0);
    assert_eq!(h.adapter.inner.reads.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn stored_invalid_metadata_blocks_planning_reads_and_writes_before_adapter_calls() {
    for (name, mut descriptor) in invalid_descriptors() {
        // Keep the store lookup identity stable; discovery separately checks an empty ID.
        descriptor.device.id = DEVICE_ID.into();
        if name == "empty identity" {
            continue;
        }
        // Seed malformed persisted data before an identity has been established.
        let h = Fixture::empty();
        h.store.save_device(&descriptor).unwrap();
        let result = h.core.execute(command(), CancellationToken::new()).await;
        assert_eq!(result.status, Status::Failed, "{name}");
        assert_eq!(
            result.error.unwrap().code,
            "CAPABILITY_UNSUPPORTED",
            "{name}"
        );
        assert!(result.attempts.is_empty());
        assert!(h.core.describe(DEVICE_ID).is_err(), "{name}");
        assert_eq!(
            h.core.plan(&command()).unwrap_err().code,
            "CAPABILITY_UNSUPPORTED",
            "{name}"
        );
        for cached in [false, true] {
            assert_eq!(
                h.core
                    .read(DEVICE_ID, "main", Route::Auto, cached, &operation())
                    .await
                    .unwrap_err()
                    .code,
                "CAPABILITY_UNSUPPORTED",
                "{name}"
            );
        }
        assert!(h.store.recent_executions().unwrap().is_empty());
        assert!(h.core.device_executions(DEVICE_ID).unwrap().is_empty());
        assert_eq!(h.core.resolve_device_id("mock-ac").unwrap(), DEVICE_ID);
        assert_eq!(h.adapter.inner.sends.load(Ordering::SeqCst), 0);
        assert_eq!(h.adapter.inner.reads.load(Ordering::SeqCst), 0);
    }
}

#[tokio::test]
async fn duplicate_discovery_ids_and_foreign_connections_preserve_the_stored_device() {
    let h = Fixture::new().await;
    let original = serde_json::to_value(h.store.devices().unwrap()).unwrap();
    for foreign in [false, true] {
        let mut descriptor = MockAdapter::descriptor();
        let batch = if foreign {
            descriptor.device.connections[0].adapter = "foreign.adapter".into();
            vec![descriptor]
        } else {
            let mut duplicate = descriptor.clone();
            duplicate.device.alias = "Conflicting label".into();
            vec![descriptor, duplicate]
        };
        *h.adapter.descriptors.lock().unwrap() = batch;
        assert_eq!(
            h.core.discover(&operation()).await.unwrap_err().code,
            "CAPABILITY_UNSUPPORTED"
        );
        assert_eq!(
            serde_json::to_value(h.store.devices().unwrap()).unwrap(),
            original
        );
    }
}

#[tokio::test]
async fn valid_read_only_vendor_metadata_and_multiple_components_remain_supported() {
    let h = Fixture::new().await;
    let mut descriptor = MockAdapter::descriptor();
    descriptor.device.components.push(Component {
        id: "auxiliary".into(),
        capabilities: vec!["vendor.example.sensor".into()],
    });
    descriptor.capabilities.push(Capability {
        component_id: None,
        schema_version: SCHEMA_VERSION.into(),
        id: "vendor.example.sensor".into(),
        revision: 1,
        properties: serde_json::from_value(
            json!({"reading": {"type": "number", "readable": true, "writable": false}}),
        )
        .unwrap(),
        actions: Default::default(),
        events: Default::default(),
    });
    *h.adapter.descriptors.lock().unwrap() = vec![descriptor];
    h.core.discover(&operation()).await.unwrap();
    assert_eq!(
        h.core.describe(DEVICE_ID).unwrap().device.components.len(),
        2
    );
    assert_eq!(
        h.core
            .execute(command(), CancellationToken::new())
            .await
            .status,
        Status::Verified
    );
    assert_eq!(h.adapter.inner.sends.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn colliding_full_property_names_are_rejected_without_rewriting_metadata() {
    let h = Fixture::new().await;
    let mut descriptor = MockAdapter::descriptor();
    descriptor.capabilities[0].properties.insert(
        "extra.value".into(),
        json!({"type": "boolean", "readable": true, "writable": false}),
    );
    descriptor.device.components[0]
        .capabilities
        .push("power.extra".into());
    descriptor.capabilities.push(Capability {
        component_id: None,
        schema_version: SCHEMA_VERSION.into(),
        id: "power.extra".into(),
        revision: 1,
        properties: serde_json::from_value(
            json!({"value": {"type": "boolean", "readable": true, "writable": false}}),
        )
        .unwrap(),
        actions: Default::default(),
        events: Default::default(),
    });
    *h.adapter.descriptors.lock().unwrap() = vec![descriptor];
    assert_eq!(
        h.core.discover(&operation()).await.unwrap_err().code,
        "CAPABILITY_UNSUPPORTED"
    );
    assert_eq!(h.core.describe(DEVICE_ID).unwrap().capabilities.len(), 2);
}
