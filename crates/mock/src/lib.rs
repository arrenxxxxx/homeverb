//! Fictional appliance with persistent state and deterministic fault injection.
//! This adapter never opens a device network connection.

use async_trait::async_trait;
use homeverb_core::*;
use homeverb_domain::*;
use rusqlite::{Connection as Database, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

pub const DEVICE_ID: &str = "dev-mock-ac";

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Scenario {
    #[default]
    Normal,
    Reject,
    LostAck,
    AcceptOnly,
    LanOffline,
    Offline,
    SlowBeforeSend,
    SlowAfterSend,
    StaleRead,
    Partial,
    CorrelatedCompletion,
    ApprovalRequired,
}

pub struct MockAdapter {
    database: Mutex<Database>,
    clock: Arc<dyn Clock>,
    scenario: Scenario,
    pub sends: AtomicUsize,
    pub reads: AtomicUsize,
}

fn failure<T>(_: T) -> Error {
    Error::new(
        "MOCK_STORAGE_ERROR",
        "The fictional appliance state is unavailable.",
    )
}

impl MockAdapter {
    pub fn open(path: &Path, clock: Arc<dyn Clock>, scenario: Scenario) -> Result<Self> {
        let database = Database::open(path).map_err(failure)?;
        database
            .busy_timeout(Duration::from_millis(250))
            .map_err(failure)?;
        database.execute_batch("PRAGMA synchronous=FULL; CREATE TABLE IF NOT EXISTS appliance (id INTEGER PRIMARY KEY CHECK(id=1), state TEXT NOT NULL);")
            .map_err(failure)?;
        database.execute("INSERT OR IGNORE INTO appliance VALUES (1, ?1)", [json!({"power.state": false, "climate.target_temperature": 26.0, "climate.mode": "cool"}).to_string()]).map_err(failure)?;
        Ok(Self {
            database: Mutex::new(database),
            clock,
            scenario,
            sends: AtomicUsize::new(0),
            reads: AtomicUsize::new(0),
        })
    }
    pub fn descriptor() -> Descriptor {
        let mut device: Device =
            serde_json::from_str(include_str!("../../../examples/device.json"))
                .expect("checked fixture");
        device.id = DEVICE_ID.into();
        device.alias = "mock-ac".into();
        device.support_status = "mock-tested".into();
        for connection in &mut device.connections {
            connection.adapter = "homeverb.mock".into();
            connection.account_id = None;
        }
        let climate: Capability =
            serde_json::from_str(include_str!("../../../examples/capability.climate.json"))
                .expect("checked fixture");
        let power: Capability = serde_json::from_value(json!({
            "schema_version": "1.0", "id": "power", "revision": 1,
            "properties": {"state": {"type": "boolean", "readable": true, "writable": true}},
            "actions": {"set": {"parameters": {"type": "object", "properties": {"state": {"type": "boolean"}},
                "required": ["state"], "additionalProperties": false}, "transports": ["lan", "cloud"],
                "idempotency": "set_value", "risk": "low", "atomic": true, "verification": "state_match"}}, "events": {}
        })).expect("static power descriptor");
        Descriptor {
            device,
            capabilities: vec![power, climate],
        }
    }
    fn reachable(&self, connection: &Connection) -> Result<()> {
        if self.scenario == Scenario::Offline
            || (self.scenario == Scenario::LanOffline && connection.transport == Transport::Lan)
        {
            return Err(Error::new(
                if connection.transport == Transport::Lan {
                    "LAN_UNREACHABLE"
                } else {
                    "CLOUD_UNREACHABLE"
                },
                "The simulated route is unavailable before transmission.",
            ));
        }
        Ok(())
    }
}

#[async_trait]
impl Adapter for MockAdapter {
    fn id(&self) -> &str {
        "homeverb.mock"
    }
    async fn discover(&self, _operation: &Operation) -> Result<Vec<Descriptor>> {
        let mut descriptor = Self::descriptor();
        if self.scenario == Scenario::CorrelatedCompletion {
            for capability in &mut descriptor.capabilities {
                capability.revision += 1;
                for action in capability.actions.values_mut() {
                    action.verification = VerificationMethod::CorrelatedCompletion;
                }
            }
        }
        if self.scenario == Scenario::ApprovalRequired {
            descriptor.device.components[0]
                .capabilities
                .push("vendor.example.cycle".into());
            descriptor.capabilities.push(
                serde_json::from_str(include_str!(
                    "../../../examples/capability.approved-cycle.json"
                ))
                .expect("checked approval fixture"),
            );
        }
        Ok(vec![descriptor])
    }
    async fn read(
        &self,
        device: &Device,
        component: &str,
        connection: &Connection,
        _operation: &Operation,
    ) -> Result<Snapshot> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.reachable(connection)?;
        let text: String = self
            .database
            .lock()
            .map_err(failure)?
            .query_row("SELECT state FROM appliance WHERE id=1", [], |r| r.get(0))
            .map_err(failure)?;
        let values: BTreeMap<String, Value> = serde_json::from_str(&text).map_err(failure)?;
        let stale = self.scenario == Scenario::StaleRead;
        let now = self.clock.now();
        Ok(Snapshot {
            schema_version: SCHEMA_VERSION.into(),
            device_id: device.id.clone(),
            component_id: component.into(),
            from_cache: false,
            observations: values
                .into_iter()
                .map(|(key, value)| {
                    (
                        key,
                        Observation {
                            value,
                            source: connection.transport,
                            observed_at: None,
                            received_at: if stale {
                                now - chrono::Duration::seconds(60)
                            } else {
                                now
                            },
                            stale,
                        },
                    )
                })
                .collect(),
            warnings: vec!["Fictional Mock appliance; no physical device was contacted.".into()],
        })
    }
    async fn execute(
        &self,
        validated: &ValidatedCommand,
        connection: &Connection,
        progress: WriteProgress,
        _operation: &Operation,
    ) -> Result<AdapterOutcome> {
        let command = validated.command();
        if command.capability == "vendor.example.cycle"
            && (self.scenario != Scenario::ApprovalRequired
                || !validated.has_user_approval()
                || command.component_id != "main"
                || command.action != "start"
                || command.arguments != json!({"program":"normal"})
                || connection.transport != Transport::Cloud)
        {
            return Err(Error::new(
                "POLICY_DENIED",
                "The fictional cycle requires its exact trusted approval.",
            ));
        }
        self.reachable(connection)?;
        if self.scenario == Scenario::SlowBeforeSend {
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
        progress.mark_sent();
        let number = self.sends.fetch_add(1, Ordering::SeqCst);
        if self.scenario == Scenario::Reject || (self.scenario == Scenario::Partial && number > 0) {
            return Ok(AdapterOutcome {
                phase: Phase::Rejected,
                error: Some(Error::new(
                    "DEVICE_REJECTED",
                    "The simulated appliance rejected this write.",
                )),
                completion: None,
            });
        }
        let command = validated.command();
        if self.scenario != Scenario::AcceptOnly && command.capability != "vendor.example.cycle" {
            let mut database = self.database.lock().map_err(failure)?;
            let transaction = database.transaction().map_err(failure)?;
            let text: String = transaction
                .query_row("SELECT state FROM appliance WHERE id=1", [], |r| r.get(0))
                .map_err(failure)?;
            let mut state: BTreeMap<String, Value> =
                serde_json::from_str(&text).map_err(failure)?;
            for (key, value) in command.arguments.as_object().expect("validated arguments") {
                state.insert(format!("{}.{key}", command.capability), value.clone());
            }
            transaction
                .execute(
                    "UPDATE appliance SET state=?1 WHERE id=1",
                    params![serde_json::to_string(&state).map_err(failure)?],
                )
                .map_err(failure)?;
            transaction.commit().map_err(failure)?;
        }
        if self.scenario == Scenario::SlowAfterSend {
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
        if self.scenario == Scenario::LostAck {
            return Ok(AdapterOutcome {
                phase: Phase::PossiblySent,
                error: None,
                completion: None,
            });
        }
        progress.mark_accepted();
        Ok(AdapterOutcome {
            phase: Phase::Accepted,
            error: None,
            completion: (self.scenario == Scenario::CorrelatedCompletion
                && command.wait_for == WaitFor::Verified)
                .then(|| CorrelatedCompletion {
                    request_id: command.request_id.clone(),
                    device_id: command.device_id.clone(),
                    component_id: command.component_id.clone(),
                    connection_id: connection.id.clone(),
                    completed_at: None,
                    received_at: self.clock.now(),
                }),
        })
    }
}
