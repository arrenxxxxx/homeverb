use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use homeverb_core::*;
use homeverb_domain::*;
use homeverb_mock::{DEVICE_ID, MockAdapter, Scenario};
use homeverb_storage::SqliteStore;
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio_util::sync::CancellationToken;

fn op() -> Operation {
    Operation::new(10000, CancellationToken::new()).unwrap()
}
fn target() -> EventHistoryTarget {
    EventHistoryTarget {
        component_id: "main".into(),
        capability: "climate".into(),
        event: "reported_activity".into(),
    }
}
fn option() -> EventHistoryOption {
    EventHistoryOption {
        target: target(),
        projection: EventHistoryProjection::OccurrenceTimes,
        max_window_seconds: 32 * 86400,
        max_records: 31,
    }
}
struct EventAdapter {
    inner: Arc<MockAdapter>,
    options: Mutex<Vec<EventHistoryOption>>,
    fault: Mutex<String>,
    calls: AtomicUsize,
    entered: tokio::sync::Notify,
    store: Arc<SqliteStore>,
}
#[async_trait]
impl Adapter for EventAdapter {
    fn id(&self) -> &str {
        self.inner.id()
    }
    async fn discover(&self, operation: &Operation) -> Result<Vec<Descriptor>> {
        let mut descriptors = self.inner.discover(operation).await?;
        for descriptor in &mut descriptors {
            descriptor
                .capabilities
                .iter_mut()
                .find(|c| c.id == "climate")
                .unwrap()
                .events
                .insert(
                    "reported_activity".into(),
                    json!({"history":{"projection":"occurrence_times"}}),
                );
        }
        Ok(descriptors)
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
        c: &ValidatedCommand,
        r: &Connection,
        p: WriteProgress,
        o: &Operation,
    ) -> Result<AdapterOutcome> {
        self.inner.execute(c, r, p, o).await
    }
    async fn event_history_options(
        &self,
        _: &Device,
        _: &Connection,
        _: &Operation,
    ) -> Result<Vec<EventHistoryOption>> {
        Ok(self.options.lock().unwrap().clone())
    }
    async fn read_event_history(
        &self,
        device: &Device,
        connection: &Connection,
        request: &EventHistoryRequest,
        operation: &Operation,
    ) -> Result<EventHistoryResult> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let fault = self.fault.lock().unwrap().clone();
        if fault == "hang" {
            self.entered.notify_one();
            return std::future::pending().await;
        }
        if fault == "error" {
            return Err(Error::new(
                "CLOUD_UNREACHABLE",
                "Fictional unavailable history.",
            ));
        }
        if fault == "cancel" {
            operation.cancellation.cancel();
        }
        if fault == "changed" {
            let mut descriptor = self
                .store
                .devices()?
                .into_iter()
                .find(|d| d.device.id == device.id)
                .unwrap();
            descriptor.device.alias = "Changed during read".into();
            self.store.save_device(&descriptor)?;
        }
        let mut result = EventHistoryResult {
            schema_version: SCHEMA_VERSION.into(),
            device_id: device.id.clone(),
            connection_id: connection.id.clone(),
            target: request.target.clone(),
            projection: request.projection,
            start: request.start,
            end: request.end,
            source: connection.transport,
            received_at: Utc::now(),
            records: vec![
                EventHistoryRecord {
                    reported_at: request.start,
                },
                EventHistoryRecord {
                    reported_at: request.start + Duration::seconds(1),
                },
            ],
            rejected_records: 1,
            limit_reached: false,
            completeness: HistoryCompleteness::Unproven,
        };
        match fault.as_str() {
            "version" => result.schema_version = "9.0".into(),
            "device" => result.device_id = "foreign".into(),
            "connection" => result.connection_id = "foreign".into(),
            "target" => result.target.event = "mode".into(),
            "source" => {
                result.source = if connection.transport == Transport::Cloud {
                    Transport::Lan
                } else {
                    Transport::Cloud
                }
            }
            "start" => result.start -= Duration::seconds(1),
            "end" => result.end += Duration::seconds(1),
            "old_receipt" => result.received_at -= Duration::days(1),
            "future_receipt" => result.received_at += Duration::days(1),
            "duplicate" => result.records[1].reported_at = result.records[0].reported_at,
            "order" => result.records.reverse(),
            "before" => result.records[0].reported_at = request.start - Duration::seconds(1),
            "after" => result.records[1].reported_at = request.end,
            "fractional" => result.records[0].reported_at += Duration::milliseconds(1),
            "records" => result.records = vec![result.records[0].clone(); 32],
            "count" => result.rejected_records = 30,
            "limit" => result.limit_reached = true,
            "full" => {
                result.records = vec![result.records[0].clone(); 31];
                result.rejected_records = 0;
                result.limit_reached = true;
            }
            "all_rejected" => {
                result.records.clear();
                result.rejected_records = 31;
                result.limit_reached = true;
            }
            "full_mismatch" => {
                result.records.clear();
                result.rejected_records = 31;
                result.limit_reached = false;
            }
            "empty" => {
                result.records.clear();
                result.rejected_records = 0;
            }
            _ => (),
        }
        Ok(result)
    }
}
struct Fixture {
    path: PathBuf,
    store: Arc<SqliteStore>,
    adapter: Arc<EventAdapter>,
    core: Core,
    request: EventHistoryRequest,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
impl Fixture {
    async fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("homeverb-event-history-{}", uuid::Uuid::new_v4()));
        let store = Arc::new(SqliteStore::open(&path).unwrap());
        let clock = Arc::new(SystemClock);
        let inner = Arc::new(
            MockAdapter::open(
                &path.join("appliance.sqlite3"),
                clock.clone(),
                Scenario::Normal,
            )
            .unwrap(),
        );
        let adapter = Arc::new(EventAdapter {
            inner,
            options: Mutex::new(vec![option()]),
            fault: Mutex::new(String::new()),
            calls: AtomicUsize::new(0),
            entered: tokio::sync::Notify::new(),
            store: store.clone(),
        });
        let core = Core::new(
            store.clone(),
            vec![adapter.clone()],
            clock,
            Policy::default(),
        );
        core.discover(&op()).await.unwrap();
        let device = core.describe(DEVICE_ID).unwrap().device;
        let request = EventHistoryRequest {
            schema_version: SCHEMA_VERSION.into(),
            device_id: device.id,
            connection_id: device
                .connections
                .iter()
                .find(|c| c.transport == Transport::Cloud)
                .unwrap()
                .id
                .clone(),
            target: target(),
            projection: EventHistoryProjection::OccurrenceTimes,
            start: DateTime::from_timestamp(1700000000, 0).unwrap(),
            end: DateTime::from_timestamp(1700086400, 0).unwrap(),
        };
        Self {
            path,
            store,
            adapter,
            core,
            request,
        }
    }
}
#[tokio::test]
async fn historical_read_uses_explicit_connection_and_never_changes_snapshot_or_receipts() {
    let f = Fixture::new().await;
    f.core
        .read(DEVICE_ID, "main", Route::Lan, false, &op())
        .await
        .unwrap();
    let before = serde_json::to_value(f.store.snapshot(DEVICE_ID, "main").unwrap()).unwrap();
    let options = f
        .core
        .inspect_event_history_options(DEVICE_ID, &f.request.connection_id, &op())
        .await
        .unwrap();
    assert_eq!(options.options, vec![option()]);
    assert_eq!(f.adapter.calls.load(Ordering::SeqCst), 0);
    let result = f.core.read_event_history(&f.request, &op()).await.unwrap();
    assert_eq!(result.records[0].reported_at, f.request.start);
    assert_eq!(result.source, Transport::Cloud);
    assert_eq!(result.rejected_records, 1);
    assert_eq!(
        before,
        serde_json::to_value(f.store.snapshot(DEVICE_ID, "main").unwrap()).unwrap()
    );
    assert!(f.core.recent_executions().unwrap().is_empty());
    assert_eq!(f.adapter.inner.sends.load(Ordering::SeqCst), 0);
    *f.adapter.fault.lock().unwrap() = "empty".into();
    assert!(
        f.core
            .read_event_history(&f.request, &op())
            .await
            .unwrap()
            .records
            .is_empty()
    );
    *f.adapter.fault.lock().unwrap() = "error".into();
    let before = f.adapter.calls.load(Ordering::SeqCst);
    assert_eq!(
        f.core
            .read_event_history(&f.request, &op())
            .await
            .unwrap_err()
            .code,
        "CLOUD_UNREACHABLE"
    );
    assert_eq!(f.adapter.calls.load(Ordering::SeqCst), before + 1);
}
#[tokio::test]
async fn malformed_adapter_histories_are_rejected_as_a_whole() {
    let f = Fixture::new().await;
    for fault in [
        "version",
        "device",
        "connection",
        "target",
        "full_mismatch",
        "source",
        "start",
        "end",
        "old_receipt",
        "future_receipt",
        "order",
        "before",
        "after",
        "fractional",
        "records",
        "count",
        "limit",
    ] {
        *f.adapter.fault.lock().unwrap() = fault.into();
        assert_eq!(
            f.core
                .read_event_history(&f.request, &op())
                .await
                .unwrap_err()
                .code,
            "INVALID_HISTORY",
            "{fault}"
        );
        assert!(f.store.snapshot(DEVICE_ID, "main").unwrap().is_none());
        assert!(f.core.recent_executions().unwrap().is_empty());
    }
}
#[tokio::test]
async fn invalid_requests_and_public_options_never_dispatch() {
    let f = Fixture::new().await;
    for fault in [
        "version",
        "device",
        "connection",
        "target",
        "reverse",
        "future",
        "span",
        "fractional",
        "zero",
    ] {
        let mut r = f.request.clone();
        match fault {
            "version" => r.schema_version = "2.0".into(),
            "device" => r.device_id = "not-present".into(),
            "connection" => r.connection_id = "wrong".into(),
            "target" => r.target.capability = "unreviewed".into(),
            "reverse" => r.end = r.start,
            "future" => {
                r.start = Utc::now();
                r.end = r.start + Duration::seconds(2);
            }
            "span" => r.end = r.start + Duration::days(33),
            "fractional" => r.start += Duration::milliseconds(1),
            _ => r.start = DateTime::from_timestamp(0, 0).unwrap(),
        }
        assert!(
            f.core.read_event_history(&r, &op()).await.is_err(),
            "{fault}"
        );
    }
    for fault in [
        "duplicate",
        "count",
        "target",
        "boolean",
        "window",
        "records",
        "control",
    ] {
        let mut options = vec![option()];
        match fault {
            "duplicate" => {
                let mut second = option();
                second.max_records = 30;
                options.push(second);
            }
            "count" => options = vec![option(); 65],
            "target" => options[0].target.component_id = "missing".into(),
            "boolean" => {
                options[0].target.capability = "power".into();
                options[0].target.event = "state".into();
            }
            "window" => options[0].max_window_seconds = 0,
            "records" => options[0].max_records = 32,
            _ => options[0].target.event = "missing".into(),
        }
        *f.adapter.options.lock().unwrap() = options;
        assert!(
            f.core
                .inspect_event_history_options(DEVICE_ID, &f.request.connection_id, &op())
                .await
                .is_err(),
            "{fault}"
        );
        assert!(
            f.core.read_event_history(&f.request, &op()).await.is_err(),
            "{fault}"
        );
    }
    assert_eq!(f.adapter.calls.load(Ordering::SeqCst), 0);
    *f.adapter.options.lock().unwrap() = vec![];
    assert!(
        f.core
            .inspect_event_history_options(DEVICE_ID, &f.request.connection_id, &op())
            .await
            .unwrap()
            .options
            .is_empty()
    );
    assert!(f.core.read_event_history(&f.request, &op()).await.is_err());
    let mut narrower = option();
    narrower.max_window_seconds = 60;
    *f.adapter.options.lock().unwrap() = vec![narrower];
    assert_eq!(
        f.core
            .read_event_history(&f.request, &op())
            .await
            .unwrap_err()
            .code,
        "INVALID_ARGUMENT"
    );
    assert_eq!(f.adapter.calls.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn binding_changes_and_late_cancellation_never_publish_history() {
    for fault in ["changed", "cancel"] {
        let f = Fixture::new().await;
        *f.adapter.fault.lock().unwrap() = fault.into();
        assert_eq!(
            f.core
                .read_event_history(&f.request, &op())
                .await
                .unwrap_err()
                .code,
            if fault == "changed" {
                "DEVICE_IDENTITY_CONFLICT"
            } else {
                "CANCELLED"
            }
        );
    }
}
#[tokio::test]
async fn pending_history_holds_device_lock_and_releases_it_on_cancel_deadline_or_drop() {
    for mode in ["cancel", "deadline", "drop"] {
        let f = Fixture::new().await;
        *f.adapter.fault.lock().unwrap() = "hang".into();
        let mut operation = op();
        if mode == "deadline" {
            operation.deadline =
                tokio::time::Instant::now() + std::time::Duration::from_millis(100);
        }
        let mut work = Box::pin(f.core.read_event_history(&f.request, &operation));
        tokio::select! {result=&mut work=>panic!("Unexpected result: {result:?}"),_=f.adapter.entered.notified()=>{}}
        assert!(f.store.lock_device(DEVICE_ID).is_err());
        match mode {
            "cancel" => {
                operation.cancellation.cancel();
                assert_eq!(work.await.unwrap_err().code, "CANCELLED");
            }
            "deadline" => assert_eq!(work.await.unwrap_err().code, "DEADLINE_EXCEEDED"),
            _ => drop(work),
        }
        assert!(f.store.lock_device(DEVICE_ID).is_ok());
        assert_eq!(f.adapter.calls.load(Ordering::SeqCst), 1);
    }
}
#[tokio::test]
async fn default_adapter_history_refuses_without_traffic() {
    let f = Fixture::new().await;
    let d = f.core.describe(DEVICE_ID).unwrap().device;
    assert!(
        f.adapter
            .inner
            .event_history_options(&d, &d.connections[0], &op())
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        f.adapter
            .inner
            .read_event_history(&d, &d.connections[0], &f.request, &op())
            .await
            .unwrap_err()
            .code,
        "CAPABILITY_UNSUPPORTED"
    );
}
#[tokio::test]
async fn locked_devices_and_nonreadable_or_malformed_descriptors_prevent_dispatch() {
    let f = Fixture::new().await;
    let lock = f.store.lock_device(DEVICE_ID).unwrap();
    assert!(f.core.read_event_history(&f.request, &op()).await.is_err());
    drop(lock);
    let original = f.core.describe(DEVICE_ID).unwrap();
    let mut descriptor = original.clone();
    let capability = descriptor
        .capabilities
        .iter_mut()
        .find(|c| c.id == "climate")
        .unwrap();
    capability.events.get_mut("reported_activity").unwrap()["history"]["projection"] =
        json!("raw_payload");
    f.store.save_device(&descriptor).unwrap();
    assert!(
        f.core
            .inspect_event_history_options(DEVICE_ID, &f.request.connection_id, &op())
            .await
            .is_err()
    );
    assert!(f.core.read_event_history(&f.request, &op()).await.is_err());
    let mut descriptor = original;
    descriptor.capabilities[0].revision = 0;
    f.store.save_device(&descriptor).unwrap();
    assert!(f.core.read_event_history(&f.request, &op()).await.is_err());
    assert_eq!(f.adapter.calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn event_history_preserves_same_second_rows_and_validates_full_rejected_pages() {
    let f = Fixture::new().await;
    *f.adapter.fault.lock().unwrap() = "duplicate".into();
    let result = f.core.read_event_history(&f.request, &op()).await.unwrap();
    assert_eq!(result.records[0], result.records[1]);
    for fault in ["full", "all_rejected"] {
        *f.adapter.fault.lock().unwrap() = fault.into();
        let result = f.core.read_event_history(&f.request, &op()).await.unwrap();
        assert!(result.limit_reached);
        assert_eq!(result.records.len() + result.rejected_records as usize, 31);
        assert_eq!(result.completeness, HistoryCompleteness::Unproven);
    }
    *f.adapter.fault.lock().unwrap() = "duplicate".into();
    let mut option = option();
    option.max_records = 2;
    *f.adapter.options.lock().unwrap() = vec![option];
    assert_eq!(
        f.core
            .read_event_history(&f.request, &op())
            .await
            .unwrap_err()
            .code,
        "INVALID_HISTORY"
    );
}

#[tokio::test]
async fn event_history_rejects_aliases_undeclared_events_and_expired_inspection_before_dispatch() {
    let f = Fixture::new().await;
    let original = f.core.describe(DEVICE_ID).unwrap();
    let mut request = f.request.clone();
    request.device_id = original.device.alias.clone();
    assert!(f.core.read_event_history(&request, &op()).await.is_err());
    for definition in [
        json!(null),
        json!({}),
        json!({"history":true}),
        json!({"history":{"projection":"payload"}}),
    ] {
        let mut descriptor = original.clone();
        descriptor
            .capabilities
            .iter_mut()
            .find(|c| c.id == "climate")
            .unwrap()
            .events
            .insert("reported_activity".into(), definition);
        f.store.save_device(&descriptor).unwrap();
        assert!(f.core.read_event_history(&f.request, &op()).await.is_err());
    }
    f.store.save_device(&original).unwrap();
    let cancelled = op();
    cancelled.cancellation.cancel();
    assert_eq!(
        f.core
            .inspect_event_history_options(DEVICE_ID, &f.request.connection_id, &cancelled)
            .await
            .unwrap_err()
            .code,
        "CANCELLED"
    );
    let mut expired = op();
    expired.deadline = tokio::time::Instant::now() - std::time::Duration::from_secs(1);
    assert_eq!(
        f.core
            .read_event_history(&f.request, &expired)
            .await
            .unwrap_err()
            .code,
        "DEADLINE_EXCEEDED"
    );
    assert_eq!(f.adapter.calls.load(Ordering::SeqCst), 0);
}
