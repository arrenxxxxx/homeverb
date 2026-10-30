use super::*;
use async_trait::async_trait;
use chrono::Utc;
use homeverb_core::*;
use homeverb_domain::*;
use homeverb_mock::{DEVICE_ID, MockAdapter, Scenario};
use homeverb_storage::SqliteStore;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use tokio_util::sync::CancellationToken;

pub(crate) struct EventAdapter {
    inner: MockAdapter,
    pub calls: AtomicUsize,
    pub active: Arc<AtomicUsize>,
    pub hang: AtomicBool,
    pub entered: tokio::sync::Notify,
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
        max_window_seconds: 2764800,
        max_records: 31,
    }
}
struct Active(Arc<AtomicUsize>);
impl Drop for Active {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}
#[async_trait]
impl Adapter for EventAdapter {
    fn id(&self) -> &str {
        self.inner.id()
    }
    async fn discover(&self, op: &Operation) -> Result<Vec<Descriptor>> {
        let mut descriptors = self.inner.discover(op).await?;
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
    async fn read(&self, d: &Device, c: &str, r: &Connection, op: &Operation) -> Result<Snapshot> {
        self.inner.read(d, c, r, op).await
    }
    async fn execute(
        &self,
        c: &ValidatedCommand,
        r: &Connection,
        p: WriteProgress,
        op: &Operation,
    ) -> Result<AdapterOutcome> {
        self.inner.execute(c, r, p, op).await
    }
    async fn event_history_options(
        &self,
        _: &Device,
        _: &Connection,
        _: &Operation,
    ) -> Result<Vec<EventHistoryOption>> {
        Ok(vec![option()])
    }
    async fn read_event_history(
        &self,
        d: &Device,
        r: &Connection,
        q: &EventHistoryRequest,
        _: &Operation,
    ) -> Result<EventHistoryResult> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.active.fetch_add(1, Ordering::SeqCst);
        let _active = Active(self.active.clone());
        self.entered.notify_one();
        if self.hang.load(Ordering::SeqCst) {
            return std::future::pending().await;
        }
        Ok(EventHistoryResult {
            schema_version: SCHEMA_VERSION.into(),
            device_id: d.id.clone(),
            connection_id: r.id.clone(),
            target: q.target.clone(),
            projection: q.projection,
            start: q.start,
            end: q.end,
            source: r.transport,
            received_at: Utc::now(),
            records: vec![
                EventHistoryRecord {
                    reported_at: q.start,
                },
                EventHistoryRecord {
                    reported_at: q.start,
                },
            ],
            rejected_records: 1,
            limit_reached: false,
            completeness: HistoryCompleteness::Unproven,
        })
    }
}
pub(crate) struct Fixture {
    pub directory: tempfile::TempDir,
    pub core: Arc<Core>,
    pub adapter: Arc<EventAdapter>,
    pub query: EventHistoryRequest,
}
pub(crate) async fn fixture() -> Fixture {
    let directory = tempfile::tempdir().unwrap();
    let clock = Arc::new(SystemClock);
    let store = Arc::new(SqliteStore::open(directory.path()).unwrap());
    let adapter = Arc::new(EventAdapter {
        inner: MockAdapter::open(
            &directory.path().join("appliance.sqlite3"),
            clock.clone(),
            Scenario::Normal,
        )
        .unwrap(),
        calls: AtomicUsize::new(0),
        active: Arc::new(AtomicUsize::new(0)),
        hang: AtomicBool::new(false),
        entered: tokio::sync::Notify::new(),
    });
    let core = Arc::new(Core::new(
        store.clone(),
        vec![adapter.clone()],
        clock,
        Policy::default(),
    ));
    core.discover(&operation()).await.unwrap();
    let descriptor = core.describe(DEVICE_ID).unwrap();
    let device = descriptor.device;
    let query = serde_json::from_value(json!({"schema_version":"1.0", "device_id":device.id,
        "connection_id":device.connections.iter().find(|v| v.transport == Transport::Cloud).unwrap().id,
        "target":target(), "projection":"occurrence_times", "start":"2026-10-01T00:00:00Z", "end":"2026-10-03T00:00:00Z"})).unwrap();
    Fixture {
        directory,
        core,
        adapter,
        query,
    }
}
fn operation() -> Operation {
    Operation::new(10000, CancellationToken::new()).unwrap()
}

#[tokio::test]
async fn cli_event_history_preserves_records_and_never_writes_cache_or_receipts() {
    let f = fixture().await;
    let options = run(
        &f.core,
        EventHistoryCommand::Options {
            device_id: f.query.device_id.clone(),
            connection_id: f.query.connection_id.clone(),
        },
        &operation(),
    )
    .await
    .unwrap();
    assert_eq!(options["options"][0]["projection"], "occurrence_times");
    assert_eq!(f.adapter.calls.load(Ordering::SeqCst), 0);
    let path = f.directory.path().join("history.json");
    std::fs::write(&path, serde_json::to_vec(&f.query).unwrap()).unwrap();
    let result = run(
        &f.core,
        EventHistoryCommand::Read { file: path },
        &operation(),
    )
    .await
    .unwrap();
    assert_eq!(result["records"][0], result["records"][1]);
    assert_eq!(result["records"][0].as_object().unwrap().len(), 1);
    assert_eq!(result["rejected_records"], 1);
    assert_eq!(result["completeness"], "unproven");
    assert_eq!(result["connection_id"], f.query.connection_id);
    assert_eq!(f.adapter.calls.load(Ordering::SeqCst), 1);
    assert!(f.core.recent_executions().unwrap().is_empty());
    assert!(
        f.core
            .read(DEVICE_ID, "main", Route::Cloud, true, &operation())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn malformed_event_history_files_fail_before_adapter_read() {
    let f = fixture().await;
    let path = f.directory.path().join("history.json");
    for field in [
        "principal",
        "confirmed",
        "provider",
        "url",
        "did",
        "limit",
        "payload",
        "cursor",
        "mijia_event_history",
    ] {
        let mut query = json!(f.query);
        query[field] = json!("forged");
        std::fs::write(&path, serde_json::to_vec(&query).unwrap()).unwrap();
        assert_eq!(
            run(
                &f.core,
                EventHistoryCommand::Read { file: path.clone() },
                &operation()
            )
            .await
            .unwrap_err()
            .code,
            "INVALID_ARGUMENT"
        );
    }
    for content in [b"{".to_vec(), vec![b' '; 16385], vec![0xff; 32]] {
        std::fs::write(&path, content).unwrap();
        assert_eq!(request(&path).unwrap_err().code, "INVALID_ARGUMENT");
    }
    for time in [
        "2026-10-01T00:00:00+00:00",
        "2026-10-01T00:00:00.000Z",
        "2026-10-01T00:00:60Z",
    ] {
        let mut query = json!(f.query);
        query["start"] = json!(time);
        std::fs::write(&path, serde_json::to_vec(&query).unwrap()).unwrap();
        assert_eq!(request(&path).unwrap_err().code, "INVALID_ARGUMENT");
    }
    assert!(request(f.directory.path()).is_err());
    assert_eq!(f.adapter.calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn cli_event_history_deadline_and_cancellation_drop_the_read_without_retry() {
    for cancel in [false, true] {
        let f = fixture().await;
        f.adapter.hang.store(true, Ordering::SeqCst);
        let path = f.directory.path().join("history.json");
        std::fs::write(&path, serde_json::to_vec(&f.query).unwrap()).unwrap();
        let token = CancellationToken::new();
        let op = Operation::new(if cancel { 10000 } else { 1000 }, token.clone()).unwrap();
        let result = run(&f.core, EventHistoryCommand::Read { file: path }, &op);
        tokio::pin!(result);
        tokio::select! {
            _ = f.adapter.entered.notified() => {},
            _ = &mut result => panic!("Read must enter the adapter"),
        }
        if cancel {
            token.cancel();
        }
        assert_eq!(
            result.await.unwrap_err().code,
            if cancel {
                "CANCELLED"
            } else {
                "DEADLINE_EXCEEDED"
            }
        );
        assert_eq!(f.adapter.calls.load(Ordering::SeqCst), 1);
        assert_eq!(f.adapter.active.load(Ordering::SeqCst), 0);
    }
}

#[cfg(unix)]
#[test]
fn fifo_is_not_a_history_file() {
    use std::os::unix::ffi::OsStrExt;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("fifo");
    let name = std::ffi::CString::new(path.as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
    assert_eq!(request(&path).unwrap_err().code, "INVALID_ARGUMENT");
}
