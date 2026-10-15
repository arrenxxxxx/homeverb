use homeverb_core::{Core, Operation, SecretBytes};
use homeverb_domain::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostRequest {
    pub schema_version: String,
    pub operation_id: String,
    pub timeout_ms: u64,
    pub request: Request,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    GetApiInfo {},
    ListAccounts {},
    GetAccount {
        account_id: uuid::Uuid,
    },
    InspectAccountInventory {
        account_id: uuid::Uuid,
    },
    RenameAccount {
        account_id: uuid::Uuid,
        label: String,
    },
    RemoveAccount {
        account_id: uuid::Uuid,
        expected_device_ids: Vec<String>,
    },
    RefreshAccount {
        account_id: uuid::Uuid,
    },
    RecoverAccount {
        account_id: uuid::Uuid,
    },
    ListDevices {},
    Discover {},
    DescribeDevice {
        device: String,
    },
    ReadState {
        device: String,
        component: String,
        connection: Route,
        cached: bool,
    },
    Plan {
        command: Command,
    },
    Execute {
        command: Command,
    },
    PlanBatch {
        batch: BatchCommand,
    },
    ExecuteBatch {
        batch: BatchCommand,
    },
    RecentExecutions {},
    ClearCachedState {},
    ClearExecutionHistory {},
    GetExecution {
        request_id: String,
    },
    Diagnostics {},
    DeviceExecutions {
        device_id: String,
    },
    ListRooms {},
    CreateRoom {
        name: String,
    },
    RenameRoom {
        room_id: String,
        name: String,
    },
    DeleteRoom {
        room_id: String,
    },
    ReorderRooms {
        room_ids: Vec<String>,
    },
    ListDeviceCards {},
    SetDevicePreferences {
        device_id: String,
        preferences: PreferencesInput,
    },
    ReorderDevices {
        device_ids: Vec<String>,
    },
    AssignDevicesToRoom {
        assignment: RoomAssignment,
    },
}

struct State {
    foreground: bool,
    closed: bool,
    operations: HashMap<String, CancellationToken>,
}

pub struct Session {
    core: Core,
    accounts: Option<Arc<homeverb_midea::MeijuAdapter>>,
    state: Mutex<State>,
}

struct Registration<'a> {
    state: &'a Mutex<State>,
    id: String,
}
impl Drop for Registration<'_> {
    fn drop(&mut self) {
        if let Ok(mut state) = self.state.lock() {
            state.operations.remove(&self.id);
        }
    }
}

impl Session {
    pub fn new(core: Core) -> Self {
        Self {
            core,
            accounts: None,
            state: Mutex::new(State {
                foreground: true,
                closed: false,
                operations: HashMap::new(),
            }),
        }
    }

    pub fn with_accounts(core: Core, accounts: Arc<homeverb_midea::MeijuAdapter>) -> Self {
        let mut session = Self::new(core);
        session.accounts = Some(accounts);
        session
    }

    /// Trusted native secret inputs only. The general JSON dispatcher has no login method.
    pub async fn connect_account(
        &self,
        input: crate::ConnectRequest,
        login: SecretBytes,
        password: SecretBytes,
    ) -> Value {
        let result = async {
            let (operation, _registration) =
                self.register(&input.schema_version, input.operation_id, input.timeout_ms)?;
            let adapter = self
                .accounts
                .as_ref()
                .ok_or_else(crate::accounts::unsupported)?;
            let credentials = homeverb_midea::LoginCredentials::new(login, password)?;
            let report = adapter
                .connect_devices(credentials, input.account_id, &input.label, &operation)
                .await?;
            Ok(crate::accounts::publish(&self.core, report, "succeeded", &operation).await)
        }
        .await;
        json!(Query::from_result(result))
    }

    pub fn set_foreground(&self, foreground: bool) {
        if let Ok(mut state) = self.state.lock() {
            state.foreground = foreground;
            if !foreground {
                for token in state.operations.values() {
                    token.cancel();
                }
            }
        }
    }

    pub fn cancel(&self, operation_id: &str) -> bool {
        self.state
            .lock()
            .ok()
            .and_then(|state| state.operations.get(operation_id).cloned())
            .is_some_and(|token| {
                token.cancel();
                true
            })
    }

    pub fn close(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.closed = true;
            for token in state.operations.values() {
                token.cancel();
            }
        }
    }

    pub async fn call(&self, input: HostRequest) -> Value {
        match self.call_inner(input).await {
            Ok(value) => value,
            Err(error) => json!(Query::<Value>::from_result(Err(error))),
        }
    }

    fn register(
        &self,
        schema_version: &str,
        operation_id: String,
        timeout_ms: u64,
    ) -> Result<(Operation, Registration<'_>)> {
        if schema_version != SCHEMA_VERSION || operation_id.is_empty() || operation_id.len() > 256 {
            return Err(Error::invalid());
        }
        let token = CancellationToken::new();
        let operation = Operation::new(timeout_ms, token.clone())?;
        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| Error::new("HOST_CLOSED", "The host session is unavailable."))?;
            if state.closed {
                return Err(Error::new("HOST_CLOSED", "The host session is closed."));
            }
            if !state.foreground {
                return Err(Error::new(
                    "HOST_BACKGROUND",
                    "New operations require a foreground host.",
                ));
            }
            if state.operations.contains_key(&operation_id) || state.operations.len() >= 8 {
                return Err(Error::new(
                    "RESOURCE_BUSY",
                    "The operation ID is active or the session has reached its operation limit.",
                ));
            }
            state.operations.insert(operation_id.clone(), token.clone());
        }
        let registration = Registration {
            state: &self.state,
            id: operation_id,
        };
        Ok((operation, registration))
    }

    async fn call_inner(&self, input: HostRequest) -> Result<Value> {
        let (operation, _registration) =
            self.register(&input.schema_version, input.operation_id, input.timeout_ms)?;
        let result = match input.request {
            Request::ListAccounts {} => {
                let accounts = match &self.accounts {
                    Some(adapter) => adapter.accounts(&operation).await?,
                    None => vec![],
                };
                Ok(json!(
                    accounts
                        .into_iter()
                        .map(crate::AccountSummary::from)
                        .collect::<Vec<_>>()
                ))
            }
            Request::GetAccount { account_id } => {
                let adapter = self
                    .accounts
                    .as_ref()
                    .ok_or_else(crate::accounts::unsupported)?;
                let account = adapter.account(account_id, &operation).await?;
                crate::accounts::details(&self.core, account, &operation).await
            }
            Request::InspectAccountInventory { account_id } => {
                let adapter = self
                    .accounts
                    .as_ref()
                    .ok_or_else(crate::accounts::unsupported)?;
                let inspection = adapter.inspect_inventory(account_id, &operation).await?;
                operation
                    .run(async { Ok(crate::accounts::inventory(inspection)) })
                    .await
            }
            Request::RenameAccount { account_id, label } => {
                let adapter = self
                    .accounts
                    .as_ref()
                    .ok_or_else(crate::accounts::unsupported)?;
                let account = adapter
                    .rename_account(account_id, &label, &operation)
                    .await?;
                Ok(json!(crate::AccountSummary::from(account)))
            }
            Request::RemoveAccount {
                account_id,
                expected_device_ids,
            } => {
                let adapter = self
                    .accounts
                    .as_ref()
                    .ok_or_else(crate::accounts::unsupported)?;
                Ok(json!(
                    adapter
                        .remove_account(account_id, &expected_device_ids, &operation)
                        .await?
                ))
            }
            Request::RefreshAccount { account_id } => {
                let adapter = self
                    .accounts
                    .as_ref()
                    .ok_or_else(crate::accounts::unsupported)?;
                let report = adapter.refresh_devices(account_id, &operation).await?;
                Ok(json!(
                    crate::accounts::publish(&self.core, report, "not_attempted", &operation).await
                ))
            }
            Request::RecoverAccount { account_id } => {
                let adapter = self
                    .accounts
                    .as_ref()
                    .ok_or_else(crate::accounts::unsupported)?;
                let report = adapter.recover_devices(account_id, &operation).await?;
                Ok(json!(
                    crate::accounts::publish(&self.core, report, "not_attempted", &operation).await
                ))
            }
            Request::GetApiInfo {} => Ok(serde_json::from_str(include_str!(
                "../../../packages/native-contract/api.json"
            ))
            .expect("the compiled native API catalog is valid JSON")),
            Request::ListDevices {} => self.core.list().map(|v| json!(v)),
            Request::Discover {} => self.core.discover(&operation).await.map(|v| json!(v)),
            Request::DescribeDevice { device } => self.core.describe(&device).map(|v| json!(v)),
            Request::ReadState {
                device,
                component,
                connection,
                cached,
            } => self
                .core
                .read(&device, &component, connection, cached, &operation)
                .await
                .map(|v| json!(v)),
            Request::Plan { command } => self.core.plan(&command),
            Request::PlanBatch { batch } => self.core.plan_batch(&batch, &operation).await,
            Request::ExecuteBatch { batch } => {
                return self
                    .core
                    .execute_batch(batch, &operation)
                    .await
                    .map(|v| json!(v));
            }
            Request::Execute { command } => {
                command.validate()?;
                // One command budget, not two independently reset deadlines.
                if command.timeout_ms != input.timeout_ms {
                    return Err(Error::invalid());
                }
                return Ok(json!(
                    self.core.execute_with_operation(command, &operation).await
                ));
            }
            Request::RecentExecutions {} => self.core.recent_executions().map(|v| json!(v)),
            Request::ClearCachedState {} => self
                .core
                .clear_cached_state(&operation)
                .await
                .map(|count| json!({"removed_snapshots": count})),
            Request::ClearExecutionHistory {} => self
                .core
                .clear_execution_history(&operation)
                .await
                .map(|count| json!({"removed_receipts": count})),
            Request::GetExecution { request_id } => self
                .core
                .execution(&request_id, &operation)
                .await
                .map(|v| json!(v)),
            Request::Diagnostics {} => self.core.diagnostics(&operation).await.map(|v| json!(v)),
            Request::DeviceExecutions { device_id } => {
                self.core.device_executions(&device_id).map(|v| json!(v))
            }
            Request::ListRooms {} => self.core.list_rooms().map(|v| json!(v)),
            Request::CreateRoom { name } => self.core.create_room(&name).map(|v| json!(v)),
            Request::RenameRoom { room_id, name } => {
                self.core.rename_room(&room_id, &name).map(|v| json!(v))
            }
            Request::DeleteRoom { room_id } => self.core.delete_room(&room_id).map(|_| Value::Null),
            Request::ReorderRooms { room_ids } => {
                self.core.reorder_rooms(&room_ids).map(|v| json!(v))
            }
            Request::ListDeviceCards {} => self.core.list_device_cards().map(|v| json!(v)),
            Request::SetDevicePreferences {
                device_id,
                preferences,
            } => self
                .core
                .set_device_preferences(&device_id, preferences)
                .map(|v| json!(v)),
            Request::ReorderDevices { device_ids } => {
                self.core.reorder_devices(&device_ids).map(|v| json!(v))
            }
            Request::AssignDevicesToRoom { assignment } => self
                .core
                .assign_devices_to_room(assignment)
                .map(|v| json!(v)),
        };
        Ok(json!(Query::from_result(result)))
    }
}
