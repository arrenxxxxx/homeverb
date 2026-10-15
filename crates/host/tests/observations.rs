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

#[derive(Clone, Copy, Debug)]
enum Fault {
    None,
    Version,
    Device,
    Component,
    Cached,
    Source,
    FutureReceipt,
    ReversedTime,
    InvalidExtra,
    EmptyWarning,
    WrongType,
    OutOfRange,
    WrongStep,
    WrongEnum,
    NullValue,
    Undeclared,
    IntegerTemperature,
    FloatTemperature,
    DifferentTemperature,
    StringTemperature,
    OldObservation,
    OldOtherObservation,
    AgeAfterRead,
    Empty,
}

struct ManualClock(Mutex<chrono::DateTime<chrono::Utc>>);
impl ManualClock {
    fn new() -> Self {
        Self(Mutex::new(chrono::Utc::now()))
    }
    fn advance(&self, duration: chrono::Duration) {
        *self.0.lock().unwrap() += duration;
    }
}
impl Clock for ManualClock {
    fn now(&self) -> chrono::DateTime<chrono::Utc> {
        *self.0.lock().unwrap()
    }
}

struct FaultAdapter {
    inner: Arc<MockAdapter>,
    fault: Mutex<Fault>,
    manual_clock: Option<Arc<ManualClock>>,
}

#[async_trait]
impl Adapter for FaultAdapter {
    fn id(&self) -> &str {
        self.inner.id()
    }
    async fn discover(&self, operation: &Operation) -> Result<Vec<Descriptor>> {
        self.inner.discover(operation).await
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
    async fn read(
        &self,
        device: &Device,
        component: &str,
        connection: &Connection,
        operation: &Operation,
    ) -> Result<Snapshot> {
        let mut snapshot = self
            .inner
            .read(device, component, connection, operation)
            .await?;
        let fault = *self.fault.lock().unwrap();
        match fault {
            Fault::None => (),
            Fault::Version => snapshot.schema_version = "9.0".into(),
            Fault::Device => snapshot.device_id = "different-device".into(),
            Fault::Component => snapshot.component_id = "different-component".into(),
            Fault::Cached => snapshot.from_cache = true,
            Fault::Source => {
                snapshot.observations.get_mut("power.state").unwrap().source = Transport::Cloud;
            }
            Fault::FutureReceipt => {
                snapshot
                    .observations
                    .get_mut("power.state")
                    .unwrap()
                    .received_at += chrono::Duration::minutes(1);
            }
            Fault::ReversedTime => {
                let observation = snapshot.observations.get_mut("power.state").unwrap();
                observation.observed_at =
                    Some(observation.received_at + chrono::Duration::seconds(1));
            }
            Fault::InvalidExtra => {
                let observation = snapshot.observations.get_mut("climate.mode").unwrap();
                observation.observed_at =
                    Some(observation.received_at + chrono::Duration::seconds(1));
            }
            Fault::EmptyWarning => snapshot.warnings.push(String::new()),
            Fault::WrongType => {
                snapshot.observations.get_mut("power.state").unwrap().value = json!("on")
            }
            Fault::OutOfRange | Fault::WrongStep => {
                snapshot
                    .observations
                    .get_mut("climate.target_temperature")
                    .unwrap()
                    .value = json!(if matches!(fault, Fault::OutOfRange) {
                    60.0
                } else {
                    24.25
                });
            }
            Fault::WrongEnum => {
                snapshot.observations.get_mut("climate.mode").unwrap().value = json!("unsupported")
            }
            Fault::NullValue => {
                snapshot.observations.get_mut("power.state").unwrap().value =
                    serde_json::Value::Null
            }
            Fault::Undeclared => {
                let observation = snapshot.observations["power.state"].clone();
                snapshot
                    .observations
                    .insert("power.undeclared".into(), observation);
            }
            Fault::IntegerTemperature
            | Fault::FloatTemperature
            | Fault::DifferentTemperature
            | Fault::StringTemperature => {
                snapshot
                    .observations
                    .get_mut("climate.target_temperature")
                    .unwrap()
                    .value = match fault {
                    Fault::IntegerTemperature => json!(25),
                    Fault::FloatTemperature => json!(25.0),
                    Fault::DifferentTemperature => json!(25.5),
                    _ => json!("25"),
                };
            }
            Fault::OldObservation | Fault::OldOtherObservation => {
                let key = if matches!(fault, Fault::OldObservation) {
                    "power.state"
                } else {
                    "climate.mode"
                };
                let observation = snapshot.observations.get_mut(key).unwrap();
                observation.observed_at =
                    Some(observation.received_at - chrono::Duration::minutes(1));
            }
            Fault::AgeAfterRead => self
                .manual_clock
                .as_ref()
                .expect("manual test clock")
                .advance(chrono::Duration::seconds(1)),
            Fault::Empty => snapshot.observations.clear(),
        }
        Ok(snapshot)
    }
}

struct Harness {
    path: PathBuf,
    core: Core,
    store: Arc<SqliteStore>,
    adapter: Arc<FaultAdapter>,
}

impl Harness {
    async fn new() -> Self {
        Self::with_manual_clock(None).await
    }

    async fn with_manual_clock(manual_clock: Option<Arc<ManualClock>>) -> Self {
        let path =
            std::env::temp_dir().join(format!("homeverb-observations-{}", uuid::Uuid::new_v4()));
        let store = Arc::new(SqliteStore::open(&path).unwrap());
        let clock: Arc<dyn Clock> = match &manual_clock {
            Some(clock) => clock.clone(),
            None => Arc::new(SystemClock),
        };
        let inner = Arc::new(
            MockAdapter::open(&path.join("mock.sqlite3"), clock.clone(), Scenario::Normal).unwrap(),
        );
        let adapter = Arc::new(FaultAdapter {
            inner,
            fault: Mutex::new(Fault::None),
            manual_clock,
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
        core.discover(&operation()).await.unwrap();
        Self {
            path,
            core,
            store,
            adapter,
        }
    }
    async fn read(&self, cached: bool) -> Result<Snapshot> {
        self.core
            .read(DEVICE_ID, "main", Route::Auto, cached, &operation())
            .await
    }
    fn set_fault(&self, fault: Fault) {
        *self.adapter.fault.lock().unwrap() = fault;
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
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

const INVALID: [Fault; 15] = [
    Fault::Version,
    Fault::Device,
    Fault::Component,
    Fault::Cached,
    Fault::Source,
    Fault::FutureReceipt,
    Fault::ReversedTime,
    Fault::InvalidExtra,
    Fault::EmptyWarning,
    Fault::WrongType,
    Fault::OutOfRange,
    Fault::WrongStep,
    Fault::WrongEnum,
    Fault::NullValue,
    Fault::Undeclared,
];

#[tokio::test]
async fn cached_freshness_uses_each_property_limit_and_preserves_stored_observations() {
    let clock = Arc::new(ManualClock::new());
    let h = Harness::with_manual_clock(Some(clock.clone())).await;
    let mut descriptor = h.core.describe(DEVICE_ID).unwrap();
    descriptor.capabilities[0]
        .properties
        .get_mut("state")
        .unwrap()["max_age_ms"] = json!(1000.0);
    descriptor.capabilities[1]
        .properties
        .get_mut("mode")
        .unwrap()["max_age_ms"] = json!(120000);
    h.store.save_device(&descriptor).unwrap();
    let original = h.read(false).await;
    assert!(
        original.is_ok(),
        "Valid property-specific freshness must be admitted: {original:?}"
    );
    let original = original.unwrap();
    clock.advance(chrono::Duration::milliseconds(999));
    assert!(!h.read(true).await.unwrap().observations["power.state"].stale);
    clock.advance(chrono::Duration::milliseconds(1));
    let boundary = h.read(true).await.unwrap();
    assert!(boundary.observations["power.state"].stale);
    assert!(!boundary.observations["climate.target_temperature"].stale);
    assert!(!boundary.observations["climate.mode"].stale);
    clock.advance(chrono::Duration::milliseconds(28999));
    assert!(!h.read(true).await.unwrap().observations["climate.target_temperature"].stale);
    clock.advance(chrono::Duration::milliseconds(1));
    assert!(h.read(true).await.unwrap().observations["climate.target_temperature"].stale);
    clock.advance(chrono::Duration::seconds(30));
    let mixed = h.read(true).await.unwrap();
    assert!(mixed.observations["climate.target_temperature"].stale);
    assert!(!mixed.observations["climate.mode"].stale);
    clock.advance(chrono::Duration::seconds(60));
    assert!(h.read(true).await.unwrap().observations["climate.mode"].stale);
    assert_eq!(
        serde_json::to_value(h.store.snapshot(DEVICE_ID, "main").unwrap().unwrap()).unwrap(),
        serde_json::to_value(original).unwrap()
    );
    assert_eq!(h.adapter.inner.reads.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn increasing_a_freshness_limit_never_clears_a_stored_stale_flag() {
    let h = Harness::new().await;
    let mut descriptor = h.core.describe(DEVICE_ID).unwrap();
    descriptor.capabilities[0]
        .properties
        .get_mut("state")
        .unwrap()["max_age_ms"] = json!(9007199254740991_i64);
    h.store.save_device(&descriptor).unwrap();
    let mut snapshot = h.read(false).await.unwrap();
    assert!(!snapshot.observations["power.state"].stale);
    snapshot.observations.get_mut("power.state").unwrap().stale = true;
    h.store.save_snapshot(&snapshot).unwrap();
    let cached = h.read(true).await.unwrap();
    assert!(cached.observations["power.state"].stale);
    assert!(
        cached
            .warnings
            .iter()
            .any(|warning| warning.contains("already marked stale"))
    );
    assert_eq!(h.adapter.inner.reads.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn verification_uses_declared_freshness_at_the_exact_age_boundary() {
    for (max_age_ms, expected) in [(1000, Status::Accepted), (1001, Status::Verified)] {
        let clock = Arc::new(ManualClock::new());
        let h = Harness::with_manual_clock(Some(clock)).await;
        let mut descriptor = h.core.describe(DEVICE_ID).unwrap();
        descriptor.capabilities[0]
            .properties
            .get_mut("state")
            .unwrap()["max_age_ms"] = json!(max_age_ms);
        h.store.save_device(&descriptor).unwrap();
        h.set_fault(Fault::AgeAfterRead);
        let result = h.core.execute(command(), CancellationToken::new()).await;
        assert_eq!(result.status, expected, "{result:?}");
        assert_eq!(h.adapter.inner.sends.load(Ordering::SeqCst), 1);
        assert_eq!(h.adapter.inner.reads.load(Ordering::SeqCst), 1);
        if expected == Status::Accepted {
            assert_eq!(result.error.unwrap().code, "VERIFICATION_PENDING");
            assert!(result.verification.is_none());
            assert!(h.store.snapshot(DEVICE_ID, "main").unwrap().is_none());
        } else {
            assert!(result.verification.unwrap().matches_target);
        }
    }
}

#[tokio::test]
async fn relaxed_freshness_never_proves_a_write_from_a_pre_send_observation() {
    let h = Harness::new().await;
    let mut descriptor = h.core.describe(DEVICE_ID).unwrap();
    descriptor.capabilities[0]
        .properties
        .get_mut("state")
        .unwrap()["max_age_ms"] = json!(120000);
    h.store.save_device(&descriptor).unwrap();
    h.set_fault(Fault::OldObservation);
    assert!(!h.read(false).await.unwrap().observations["power.state"].stale);
    let result = h.core.execute(command(), CancellationToken::new()).await;
    assert_eq!(result.status, Status::Accepted);
    assert_eq!(result.error.unwrap().code, "VERIFICATION_PENDING");
    assert!(result.verification.is_none());
    assert_eq!(h.adapter.inner.sends.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn invalid_live_snapshots_never_replace_cache_or_try_another_route() {
    for fault in INVALID {
        let h = Harness::new().await;
        h.read(false).await.unwrap();
        let before = serde_json::to_value(h.store.snapshot(DEVICE_ID, "main").unwrap()).unwrap();
        h.set_fault(fault);
        assert_eq!(
            h.read(false).await.unwrap_err().code,
            "INVALID_OBSERVATION",
            "{fault:?}"
        );
        assert_eq!(
            serde_json::to_value(h.store.snapshot(DEVICE_ID, "main").unwrap()).unwrap(),
            before
        );
        assert_eq!(h.adapter.inner.reads.load(Ordering::SeqCst), 2);
        assert_eq!(h.adapter.inner.sends.load(Ordering::SeqCst), 0);
    }
}

#[tokio::test]
async fn invalid_verification_snapshots_preserve_acceptance_without_replay_or_cache_poisoning() {
    for fault in INVALID {
        let h = Harness::new().await;
        h.set_fault(fault);
        let result = h.core.execute(command(), CancellationToken::new()).await;
        assert_eq!(result.status, Status::Accepted, "{fault:?}");
        assert_eq!(result.error.unwrap().code, "VERIFICATION_PENDING");
        assert!(result.verification.is_none());
        assert!(h.store.snapshot(DEVICE_ID, "main").unwrap().is_none());
        assert_eq!(h.adapter.inner.sends.load(Ordering::SeqCst), 1);
        assert_eq!(h.adapter.inner.reads.load(Ordering::SeqCst), 1);
        assert_eq!(result.attempts.len(), 1);
    }
}

#[tokio::test]
async fn observation_age_survives_new_receipt_times_and_cache_reads() {
    let h = Harness::new().await;
    h.set_fault(Fault::OldObservation);
    let live = h.read(false).await.unwrap();
    assert!(live.observations["power.state"].stale);
    assert!(!live.warnings.is_empty());
    let cached = h.read(true).await.unwrap();
    assert!(cached.observations["power.state"].stale);
    assert!(cached.from_cache);
    assert_eq!(h.adapter.inner.reads.load(Ordering::SeqCst), 1);
    assert_eq!(
        h.core
            .execute(command(), CancellationToken::new())
            .await
            .status,
        Status::Accepted
    );
}

#[tokio::test]
async fn stale_other_properties_and_empty_snapshots_do_not_invent_state_or_block_valid_evidence() {
    let h = Harness::new().await;
    h.set_fault(Fault::OldOtherObservation);
    let result = h.core.execute(command(), CancellationToken::new()).await;
    assert_eq!(result.status, Status::Verified);
    assert!(
        h.store
            .snapshot(DEVICE_ID, "main")
            .unwrap()
            .unwrap()
            .observations["climate.mode"]
            .stale
    );
    h.set_fault(Fault::Empty);
    assert!(h.read(false).await.unwrap().observations.is_empty());
    assert_eq!(
        h.core
            .execute(command(), CancellationToken::new())
            .await
            .status,
        Status::Accepted
    );
}

#[tokio::test]
async fn cached_observations_validate_version_and_chronology_without_refreshing() {
    let h = Harness::new().await;
    let original = h.read(false).await.unwrap();
    for wrong_version in [false, true] {
        let mut snapshot = original.clone();
        if wrong_version {
            snapshot.schema_version = "9.0".into();
        } else {
            let observation = snapshot.observations.get_mut("power.state").unwrap();
            observation.observed_at = Some(observation.received_at + chrono::Duration::seconds(1));
        }
        h.store.save_snapshot(&snapshot).unwrap();
        let stored = serde_json::to_value(&snapshot).unwrap();
        assert_eq!(h.read(true).await.unwrap_err().code, "INVALID_OBSERVATION");
        assert_eq!(
            serde_json::to_value(h.store.snapshot(DEVICE_ID, "main").unwrap().unwrap()).unwrap(),
            stored
        );
    }
    assert_eq!(h.adapter.inner.reads.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn cached_values_must_match_the_current_component_without_refresh_or_rewrite() {
    let h = Harness::new().await;
    let original = h.read(false).await.unwrap();
    for (name, value) in [
        ("power.state", json!("on")),
        ("power.state", json!(null)),
        ("climate.target_temperature", json!(60)),
        ("climate.target_temperature", json!(24.25)),
        ("climate.mode", json!("unsupported")),
        ("unknown.value", json!(false)),
    ] {
        let mut snapshot = original.clone();
        let mut observation = snapshot.observations["power.state"].clone();
        observation.value = value;
        snapshot.observations.insert(name.into(), observation);
        h.store.save_snapshot(&snapshot).unwrap();
        assert_eq!(h.read(true).await.unwrap_err().code, "INVALID_OBSERVATION");
        assert_eq!(
            serde_json::to_value(h.store.snapshot(DEVICE_ID, "main").unwrap().unwrap()).unwrap(),
            serde_json::to_value(snapshot).unwrap()
        );
    }
    assert_eq!(h.adapter.inner.reads.load(Ordering::SeqCst), 1);
    assert_eq!(h.adapter.inner.sends.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn invalid_or_ambiguous_property_metadata_cannot_authorize_observations() {
    for mutation in [
        "unreadable",
        "missing-capability",
        "foreign-component",
        "duplicate-capability",
        "invalid-type",
        "unknown-field",
        "zero-step",
        "old-version",
        "zero-revision",
        "duplicate-component",
    ] {
        let h = Harness::new().await;
        let original = h.read(false).await.unwrap();
        let mut descriptor = h.core.describe(DEVICE_ID).unwrap();
        let climate = descriptor
            .capabilities
            .iter_mut()
            .find(|c| c.id == "climate")
            .unwrap();
        match mutation {
            "unreadable" => climate.properties.get_mut("mode").unwrap()["readable"] = json!(false),
            "invalid-type" => {
                climate.properties.get_mut("mode").unwrap()["type"] = json!("anything")
            }
            "unknown-field" => {
                climate.properties.get_mut("mode").unwrap()["$ref"] =
                    json!("https://example.invalid/schema")
            }
            "zero-step" => {
                climate.properties.get_mut("target_temperature").unwrap()["multipleOf"] = json!(0)
            }
            "old-version" => climate.schema_version = "9.0".into(),
            "zero-revision" => climate.revision = 0,
            "missing-capability" => descriptor.capabilities.retain(|c| c.id != "climate"),
            "foreign-component" => {
                descriptor.device.components[0]
                    .capabilities
                    .retain(|id| id != "climate");
                descriptor.device.components.push(Component {
                    id: "other".into(),
                    capabilities: vec!["climate".into()],
                });
            }
            "duplicate-capability" => {
                let duplicate = climate.clone();
                descriptor.capabilities.push(duplicate);
            }
            "duplicate-component" => descriptor
                .device
                .components
                .push(descriptor.device.components[0].clone()),
            _ => unreachable!(),
        }
        h.store.save_device(&descriptor).unwrap();
        let valid_metadata = matches!(mutation, "unreadable" | "foreign-component");
        for cached in [true, false] {
            assert_eq!(
                h.read(cached).await.unwrap_err().code,
                if valid_metadata {
                    "INVALID_OBSERVATION"
                } else {
                    "CAPABILITY_UNSUPPORTED"
                },
                "{mutation}"
            );
        }
        let result = h.core.execute(command(), CancellationToken::new()).await;
        if valid_metadata {
            assert_eq!(result.status, Status::Accepted, "{mutation}");
            assert_eq!(result.error.unwrap().code, "VERIFICATION_PENDING");
            assert_eq!(h.adapter.inner.sends.load(Ordering::SeqCst), 1);
            assert_eq!(h.adapter.inner.reads.load(Ordering::SeqCst), 3);
        } else {
            assert_eq!(result.status, Status::Failed, "{mutation}");
            assert_eq!(result.error.unwrap().code, "CAPABILITY_UNSUPPORTED");
            assert!(result.attempts.is_empty());
            assert_eq!(h.adapter.inner.sends.load(Ordering::SeqCst), 0);
            assert_eq!(h.adapter.inner.reads.load(Ordering::SeqCst), 1);
        }
        assert_eq!(
            serde_json::to_value(h.store.snapshot(DEVICE_ID, "main").unwrap().unwrap()).unwrap(),
            serde_json::to_value(original).unwrap()
        );
    }
}

#[tokio::test]
async fn readable_vendor_properties_use_their_own_types_limits_and_steps() {
    let h = Harness::new().await;
    let original = h.read(false).await.unwrap();
    let mut descriptor = h.core.describe(DEVICE_ID).unwrap();
    descriptor.device.components[0]
        .capabilities
        .push("vendor.example.sensor".into());
    descriptor.capabilities.push(Capability {
        component_id: None,
        schema_version: SCHEMA_VERSION.into(), id: "vendor.example.sensor".into(), revision: 1,
        properties: serde_json::from_value(json!({
            "temperature": {"type": "number", "readable": true, "writable": false, "minimum": -10, "maximum": 100, "multipleOf": 0.25},
            "count": {"type": "integer", "readable": true, "writable": false, "minimum": 0}
        })).unwrap(),
        actions: Default::default(), events: Default::default(),
    });
    h.store.save_device(&descriptor).unwrap();
    for (property, value, valid) in [
        ("temperature", json!(-10), true),
        ("temperature", json!(40.25), true),
        ("temperature", json!(100), true),
        ("temperature", json!(100.25), false),
        ("count", json!(0), true),
        ("count", json!(2.0), true),
        ("count", json!(2.5), false),
        ("count", json!(false), false),
    ] {
        let key = format!("vendor.example.sensor.{property}");
        let mut snapshot = original.clone();
        let mut observation = snapshot.observations["power.state"].clone();
        observation.value = value.clone();
        snapshot.observations.insert(key.clone(), observation);
        h.store.save_snapshot(&snapshot).unwrap();
        let result = h.read(true).await;
        if valid {
            assert!(result.is_ok(), "{key}={value}: {result:?}");
            assert_eq!(result.unwrap().observations[&key].value, value);
        } else {
            assert_eq!(result.unwrap_err().code, "INVALID_OBSERVATION");
        }
    }
    assert_eq!(h.adapter.inner.reads.load(Ordering::SeqCst), 1);
    assert_eq!(h.adapter.inner.sends.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn negative_fractional_temperatures_obey_device_and_grant_bounds() {
    let mut h = Harness::new().await;
    let mut descriptor = h.core.describe(DEVICE_ID).unwrap();
    let climate = descriptor
        .capabilities
        .iter_mut()
        .find(|c| c.id == "climate")
        .unwrap();
    let property = climate.properties.get_mut("target_temperature").unwrap();
    property["minimum"] = json!(-30);
    property["maximum"] = json!(10);
    property["multipleOf"] = json!(0.25);
    climate.actions.get_mut("set").unwrap().parameters = json!({
        "type": "object", "properties": {"target_temperature": {
            "type": "number", "minimum": -30, "maximum": 10, "multipleOf": 0.25
        }}, "required": ["target_temperature"], "additionalProperties": false
    });
    h.store.save_device(&descriptor).unwrap();
    h.core = Core::new(
        h.store.clone(),
        vec![h.adapter.clone()],
        Arc::new(SystemClock),
        Policy {
            grants: vec![Grant {
                device_id: DEVICE_ID.into(),
                component_id: "main".into(),
                capability: "climate".into(),
                action: "set".into(),
                transports: vec![Transport::Lan],
                argument_schema: json!({
                    "type": "object", "properties": {"target_temperature": {
                        "type": "number", "minimum": -20, "maximum": 0, "multipleOf": 0.25
                    }}, "required": ["target_temperature"], "additionalProperties": false
                }),
            }],
        },
    );
    for (temperature, error) in [
        (-10.25, None),
        (-10.125, Some("INVALID_ARGUMENT")),
        (-25.0, Some("POLICY_DENIED")),
    ] {
        let mut request = command();
        request.capability = "climate".into();
        request.arguments = json!({"target_temperature": temperature});
        let result = h.core.execute(request, CancellationToken::new()).await;
        if let Some(error) = error {
            assert_eq!(result.status, Status::Failed);
            assert_eq!(result.error.unwrap().code, error);
            assert!(result.attempts.is_empty());
        } else {
            assert_eq!(result.status, Status::Verified);
        }
    }
    assert_eq!(
        h.read(false).await.unwrap().observations["climate.target_temperature"].value,
        json!(-10.25)
    );
    assert_eq!(
        h.read(true).await.unwrap().observations["climate.target_temperature"].value,
        json!(-10.25)
    );
    assert_eq!(h.adapter.inner.sends.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn verification_matches_numeric_values_without_coercing_types_or_rounding() {
    for (target, fault, expected) in [
        (json!(25), Fault::FloatTemperature, Status::Verified),
        (json!(25.0), Fault::IntegerTemperature, Status::Verified),
        (json!(25), Fault::DifferentTemperature, Status::Accepted),
        (json!(25), Fault::StringTemperature, Status::Accepted),
    ] {
        let mut h = Harness::new().await;
        h.core = Core::new(
            h.store.clone(),
            vec![h.adapter.clone()],
            Arc::new(SystemClock),
            Policy {
                grants: vec![Grant {
                    device_id: DEVICE_ID.into(),
                    component_id: "main".into(),
                    capability: "climate".into(),
                    action: "set".into(),
                    transports: vec![Transport::Lan],
                    argument_schema: json!({"type": "object"}),
                }],
            },
        );
        h.set_fault(fault);
        let mut request = command();
        request.capability = "climate".into();
        request.arguments = json!({"target_temperature": target});
        let result = h
            .core
            .execute(request.clone(), CancellationToken::new())
            .await;
        assert_eq!(result.status, expected, "{fault:?}");
        assert_eq!(h.adapter.inner.sends.load(Ordering::SeqCst), 1);
        assert_eq!(h.adapter.inner.reads.load(Ordering::SeqCst), 1);
        if expected == Status::Verified {
            assert!(result.verification.unwrap().matches_target);
            let cached = h.read(true).await.unwrap();
            let observed = &cached.observations["climate.target_temperature"].value;
            assert_eq!(observed.is_f64(), matches!(fault, Fault::FloatTemperature));
            assert_eq!(result.data.unwrap(), request.arguments);
        } else {
            assert_eq!(result.error.unwrap().code, "VERIFICATION_PENDING");
            assert!(h.store.snapshot(DEVICE_ID, "main").unwrap().is_none());
        }
    }
}

#[tokio::test]
async fn cached_clock_regression_is_stale_and_cancellation_never_returns_cached_state() {
    let h = Harness::new().await;
    let mut original = h.read(false).await.unwrap();
    for observation in original.observations.values_mut() {
        observation.received_at += chrono::Duration::minutes(1);
        observation.observed_at = Some(observation.received_at);
    }
    h.store.save_snapshot(&original).unwrap();
    let cached = h.read(true).await.unwrap();
    assert!(cached.observations.values().all(|value| value.stale));
    assert!(
        cached
            .warnings
            .iter()
            .any(|warning| warning.contains("local clock"))
    );
    assert_eq!(
        serde_json::to_value(h.store.snapshot(DEVICE_ID, "main").unwrap().unwrap()).unwrap(),
        serde_json::to_value(original).unwrap()
    );
    let cancelled = operation();
    cancelled.cancellation.cancel();
    assert_eq!(
        h.core
            .read(DEVICE_ID, "main", Route::Auto, true, &cancelled)
            .await
            .unwrap_err()
            .code,
        "CANCELLED"
    );
    let mut expired = operation();
    expired.deadline = tokio::time::Instant::now() - std::time::Duration::from_secs(1);
    assert_eq!(
        h.core
            .read(DEVICE_ID, "main", Route::Auto, true, &expired)
            .await
            .unwrap_err()
            .code,
        "DEADLINE_EXCEEDED"
    );
    assert_eq!(h.adapter.inner.reads.load(Ordering::SeqCst), 1);
}
