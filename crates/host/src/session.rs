use homeverb_core::{Core, Operation, PendingApproval, PendingConnectionLink, SecretBytes};
use homeverb_domain::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};
use tokio_util::sync::CancellationToken;

mod lan_scan;
pub use lan_scan::{ConfirmLanScanRequest, PendingMeijuLanScan, PrepareLanScanRequest};
mod connection_link;
pub use connection_link::{ConfirmConnectionLinkRequest, PrepareConnectionLinkRequest};

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
    InspectDeviceConflicts {
        account_id: uuid::Uuid,
        device_id: uuid::Uuid,
    },
    InspectAccountInventory {
        account_id: uuid::Uuid,
    },
    InspectControlOptions {
        device_id: String,
        connection_id: String,
    },
    InspectSessionGrants {
        device_id: String,
        connection_id: String,
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

static NEXT_PREVIEW_ID: AtomicU64 = AtomicU64::new(1);

struct State {
    foreground: bool,
    closed: bool,
    operations: HashMap<String, CancellationToken>,
    approvals: HashMap<u64, PendingApproval>,
    lan_scans: HashMap<u64, PendingMeijuLanScan>,
    connection_links: HashMap<u64, PendingConnectionLink>,
    approval_generation: u64,
}

pub struct Session {
    core: Core,
    accounts: Option<Arc<homeverb_midea::MeijuAdapter>>,
    mijia_accounts: Option<Arc<homeverb_mijia::MijiaAdapter>>,
    state: Arc<Mutex<State>>,
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
            mijia_accounts: None,
            state: Arc::new(Mutex::new(State {
                foreground: true,
                closed: false,
                operations: HashMap::new(),
                approvals: HashMap::new(),
                lan_scans: HashMap::new(),
                connection_links: HashMap::new(),
                approval_generation: 0,
            })),
        }
    }

    pub fn with_accounts(core: Core, accounts: Arc<homeverb_midea::MeijuAdapter>) -> Self {
        let mut session = Self::new(core);
        session.accounts = Some(accounts);
        session
    }

    /// Trusted compile-time adapter composition; no credentials or networking.
    pub fn with_platform_accounts(
        core: Core,
        meiju: Option<Arc<homeverb_midea::MeijuAdapter>>,
        mijia: Option<Arc<homeverb_mijia::MijiaAdapter>>,
    ) -> Self {
        let mut session = Self::new(core);
        session.accounts = meiju;
        session.mijia_accounts = mijia;
        session
    }

    async fn account_adapter(
        &self,
        id: uuid::Uuid,
        op: &Operation,
    ) -> Result<crate::accounts::AccountAdapter<'_>> {
        homeverb_core::valid_account_id(id)?;
        if let Some(adapter) = &self.accounts
            && adapter.accounts(op).await?.iter().any(|a| a.id == id)
        {
            return Ok(crate::accounts::AccountAdapter::Meiju(adapter));
        }
        if let Some(adapter) = &self.mijia_accounts
            && adapter.accounts(op).await?.iter().any(|a| a.id == id)
        {
            return Ok(crate::accounts::AccountAdapter::Mijia(adapter));
        }
        if self.accounts.is_none() && self.mijia_accounts.is_none() {
            Err(crate::accounts::unsupported())
        } else {
            Err(Error::new(
                "ACCOUNT_NOT_FOUND",
                "The selected account does not exist in this host.",
            ))
        }
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
            use crate::accounts::AccountProvider;
            let provider = input.provider.unwrap_or(
                if self.accounts.is_none() && self.mijia_accounts.is_some() {
                    AccountProvider::Mijia
                } else {
                    AccountProvider::Meiju
                },
            );
            match provider {
                AccountProvider::Meiju => {
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
                AccountProvider::Mijia => {
                    let adapter = self
                        .mijia_accounts
                        .as_ref()
                        .ok_or_else(crate::accounts::unsupported)?;
                    let credentials = homeverb_mijia::LoginCredentials::new(login, password)?;
                    let report = adapter
                        .connect_devices(credentials, input.account_id, &input.label, &operation)
                        .await?;
                    Ok(
                        crate::accounts::publish_mijia(&self.core, report, "succeeded", &operation)
                            .await,
                    )
                }
            }
        }
        .await;
        json!(Query::from_result(result))
    }

    pub fn set_foreground(&self, foreground: bool) {
        if let Ok(mut state) = self.state.lock() {
            state.foreground = foreground;
            if !foreground {
                self.core.invalidate_user_approvals();
                state.approvals.clear();
                state.lan_scans.clear();
                state.connection_links.clear();
                state.approval_generation += 1;
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
            self.core.invalidate_user_approvals();
            state.approvals.clear();
            state.lan_scans.clear();
            state.connection_links.clear();
            state.approval_generation += 1;
            for token in state.operations.values() {
                token.cancel();
            }
        }
    }

    /// Trusted in-process UI only; this is intentionally absent from ordinary JSON.
    pub async fn prepare_connection_link(
        &self,
        operation_id: String,
        device_id: &str,
        anchor_id: &str,
        connection: Connection,
        timeout_ms: u64,
    ) -> Result<PendingConnectionLink> {
        let (operation, _registration) = self.register(SCHEMA_VERSION, operation_id, timeout_ms)?;
        self.core
            .prepare_connection_link(device_id, anchor_id, connection, &operation)
            .await
    }

    /// Consume only after trusted UI consent; never expose through ordinary JSON.
    pub async fn confirm_connection_link(
        &self,
        operation_id: String,
        preview: PendingConnectionLink,
        timeout_ms: u64,
    ) -> Result<Descriptor> {
        let (operation, _registration) = self.register(SCHEMA_VERSION, operation_id, timeout_ms)?;
        self.core.confirm_connection_link(preview, &operation).await
    }

    /// Trusted in-process UI only; this is intentionally absent from ordinary JSON.
    pub async fn prepare_user_approval(
        &self,
        operation_id: String,
        command: Command,
    ) -> Result<PendingApproval> {
        let (operation, _registration) =
            self.register(SCHEMA_VERSION, operation_id, command.timeout_ms)?;
        let preview = operation
            .run(async { self.core.prepare_user_approval(command) })
            .await?;
        let state = self
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
        Ok(preview)
    }

    /// Call only after trusted UI consent to this immutable preview; consumes it once.
    pub async fn execute_user_approved(
        &self,
        operation_id: String,
        approval: PendingApproval,
    ) -> Result<Execution> {
        let (operation, _registration) =
            self.register(SCHEMA_VERSION, operation_id, approval.command().timeout_ms)?;
        Ok(self.core.execute_user_approved(approval, &operation).await)
    }

    /// Separate trusted C bridge. A preview is not user consent or execution authority.
    pub async fn prepare_native_approval(&self, input: crate::PrepareApprovalRequest) -> Value {
        let result = async {
            if input.schema_version != SCHEMA_VERSION {
                return Err(Error::invalid());
            }
            let generation = self
                .state
                .lock()
                .map_err(|_| Error::new("HOST_CLOSED", "The host session is unavailable."))?
                .approval_generation;
            let preview = self
                .prepare_user_approval(input.operation_id, input.command)
                .await?;
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
            if state.approval_generation != generation {
                return Err(Error::new(
                    "APPROVAL_INVALID",
                    "The host lifecycle changed during approval preparation.",
                ));
            }
            state.approvals.retain(|_, value| !value.is_expired());
            if state.approvals.len() >= 8 {
                return Err(Error::new(
                    "RESOURCE_BUSY",
                    "Dismiss a pending approval before preparing another.",
                ));
            }
            let id = NEXT_PREVIEW_ID.fetch_add(1, Ordering::Relaxed);
            let value = json!({
                "approval_id": id,
                "command": preview.command(),
                "descriptor": preview.descriptor(),
                "action": preview.action(),
                "connection": preview.connection(),
                "expires_at": preview.expires_at()
            });
            state.approvals.insert(id, preview);
            Ok(value)
        }
        .await;
        json!(Query::from_result(result))
    }

    /// Only the trusted UI may call this after confirming the returned immutable preview.
    pub async fn execute_native_approved(&self, input: crate::ExecuteApprovalRequest) -> Value {
        let result = async {
            if input.approval_id == 0 {
                return Err(Error::invalid());
            }
            let approval = {
                let mut state = self
                    .state
                    .lock()
                    .map_err(|_| Error::new("HOST_CLOSED", "The host session is unavailable."))?;
                let approval = state.approvals.remove(&input.approval_id);
                if state.closed {
                    return Err(Error::new("HOST_CLOSED", "The host session is closed."));
                }
                if !state.foreground {
                    return Err(Error::new(
                        "HOST_BACKGROUND",
                        "New operations require a foreground host.",
                    ));
                }
                approval.ok_or_else(|| {
                    Error::new(
                        "APPROVAL_INVALID",
                        "The approval is unavailable for this host; review the command again.",
                    )
                })?
            };
            if input.schema_version != SCHEMA_VERSION {
                return Err(Error::invalid());
            }
            self.execute_user_approved(input.operation_id, approval)
                .await
        }
        .await;
        match result {
            Ok(receipt) => json!(receipt),
            Err(error) => json!(Query::<Value>::from_result(Err(error))),
        }
    }

    /// Dismissal is local-only and cannot revoke an already consumed/sent command.
    pub fn discard_native_approval(&self, approval_id: u64) -> bool {
        self.state
            .lock()
            .ok()
            .is_some_and(|mut state| state.approvals.remove(&approval_id).is_some())
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
                let mut accounts = match &self.accounts {
                    Some(adapter) => adapter.accounts(&operation).await?,
                    None => vec![],
                };
                if let Some(adapter) = &self.mijia_accounts {
                    accounts.extend(adapter.accounts(&operation).await?);
                }
                accounts.sort_by_key(|a| a.id);
                Ok(json!(
                    accounts
                        .into_iter()
                        .map(crate::AccountSummary::from)
                        .collect::<Vec<_>>()
                ))
            }
            Request::GetAccount { account_id } => {
                let adapter = self.account_adapter(account_id, &operation).await?;
                let account = adapter.account(account_id, &operation).await?;
                crate::accounts::details(&self.core, account, &operation).await
            }
            Request::InspectDeviceConflicts {
                account_id,
                device_id,
            } => {
                let adapter = match self.account_adapter(account_id, &operation).await? {
                    crate::accounts::AccountAdapter::Meiju(adapter) => adapter,
                    crate::accounts::AccountAdapter::Mijia(_) => {
                        return Err(Error::new(
                            "CAPABILITY_UNSUPPORTED",
                            "Saved device conflict inspection is unavailable for this provider.",
                        ));
                    }
                };
                let inspection = adapter
                    .inspect_device_conflicts(account_id, device_id, &operation)
                    .await?;
                operation
                    .run(async { Ok(crate::accounts::conflicts(inspection)) })
                    .await
            }
            Request::InspectAccountInventory { account_id } => {
                let adapter = self.account_adapter(account_id, &operation).await?;
                adapter.inventory(account_id, &operation).await
            }
            Request::InspectControlOptions {
                device_id,
                connection_id,
            } => {
                let adapter = self
                    .accounts
                    .as_ref()
                    .ok_or_else(crate::accounts::unsupported)?;
                let descriptor = self.core.describe(&device_id)?;
                if descriptor.device.id != device_id {
                    return Err(Error::invalid());
                }
                let connection = descriptor
                    .device
                    .connections
                    .iter()
                    .find(|connection| connection.id == connection_id)
                    .ok_or_else(Error::invalid)?;
                let options = adapter
                    .inspect_control_options(&descriptor.device, connection, &operation)
                    .await?;
                operation.run(async {
                    if json!(self.core.describe(&device_id)?) != json!(descriptor) {
                        return Err(Error::new("DEVICE_IDENTITY_CONFLICT", "The device metadata changed during inspection."));
                    }
                    Ok(json!({"device_id":device_id,"connection_id":connection_id,
                        "source":"saved_inventory","authorization":"not_checked","options":options}))
                }).await
            }
            Request::InspectSessionGrants {
                device_id,
                connection_id,
            } => {
                self.core
                    .inspect_session_grants(&device_id, &connection_id, &operation)
                    .await
            }
            Request::RenameAccount { account_id, label } => {
                let adapter = self.account_adapter(account_id, &operation).await?;
                Ok(json!(crate::AccountSummary::from(
                    adapter.rename(account_id, &label, &operation).await?
                )))
            }
            Request::RemoveAccount {
                account_id,
                expected_device_ids,
            } => {
                // Provider-scoped removal checks its durable ledger before mutation.
                // An already removed account is absent from the ordinary account list.
                let mut result = Err(crate::accounts::unsupported());
                if let Some(adapter) = &self.accounts {
                    result = adapter
                        .remove_account(account_id, &expected_device_ids, &operation)
                        .await;
                }
                if (self.accounts.is_none()
                    || matches!(&result, Err(e) if e.code == "ACCOUNT_NOT_FOUND"))
                    && let Some(adapter) = &self.mijia_accounts
                {
                    result = adapter
                        .remove_account(account_id, &expected_device_ids, &operation)
                        .await;
                }
                result.map(|value| json!(value))
            }
            Request::RefreshAccount { account_id } => {
                let adapter = self.account_adapter(account_id, &operation).await?;
                Ok(json!(
                    adapter
                        .retrieve(&self.core, account_id, false, &operation)
                        .await?
                ))
            }
            Request::RecoverAccount { account_id } => {
                let adapter = self.account_adapter(account_id, &operation).await?;
                Ok(json!(
                    adapter
                        .retrieve(&self.core, account_id, true, &operation)
                        .await?
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
