//! Account-bound Core reads and local card identity. Construction performs no I/O.

mod history;

use crate::{
    CloudTransport, Inventory, LoginCredentials, ModelMapping, PROVIDER, PropertyQuery, Region,
    SpecIndex,
    protected::{Binding, LoadedAccount, Payload, StoredSession, conflict, unconfirmed},
};
use async_trait::async_trait;
use homeverb_core::{
    AccountRecord, AccountVault, Adapter, AdapterOutcome, Clock, CredentialState, Operation,
    ValidatedCommand, WriteProgress,
};
use homeverb_domain::{
    Capability, Component, Connection, Descriptor, Device, Error, Observation, Result, Snapshot,
    Transport,
};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::Arc,
    time::Duration,
};
use uuid::Uuid;

/// Public projections must omit credential references and private payloads.
/// ```compile_fail
/// fn export(value: &homeverb_mijia::AccountDiscovery) {
///     let _ = serde_json::to_string(value);
/// }
/// ```
pub struct AccountDiscovery {
    pub account: AccountRecord,
    pub descriptors: Result<Vec<Descriptor>>,
    pub issues: Vec<Error>,
    pub complete: bool,
}
pub struct DeviceInventoryMembership {
    pub device_id: Uuid,
    pub connection_id: Uuid,
    pub listed: bool,
}
pub struct AccountInventory {
    pub account: AccountRecord,
    pub devices: Vec<DeviceInventoryMembership>,
}

mod login;
pub use login::{
    AccountLogin, AccountLoginChallenge, ModernAccountDiscovery, ModernAccountEntry,
    PendingMijiaLogin, PendingModernMijiaLogin, PendingVerificationConnection,
    VerificationAccountLogin, VerificationAnswer,
};

pub struct MijiaAdapter {
    login_owner: Uuid,
    vault: Arc<AccountVault>,
    transport: Arc<dyn CloudTransport>,
    modern_auth: Option<Arc<dyn crate::ModernAuthTransport>>,
    modern_reads: Option<Arc<dyn crate::ModernReadTransport>>,
    clock: Arc<dyn Clock>,
    fan_controls: bool,
    reference_controls: bool,
    outlet_controls: bool,
    control_policies: Option<(homeverb_core::Policy, homeverb_core::Policy)>,
}
impl MijiaAdapter {
    pub fn new(
        vault: Arc<AccountVault>,
        transport: Arc<dyn CloudTransport>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            login_owner: Uuid::new_v4(),
            vault,
            transport,
            modern_auth: None,
            modern_reads: None,
            clock,
            fan_controls: false,
            reference_controls: false,
            outlet_controls: false,
            control_policies: None,
        }
    }

    /// Explicit typed modern composition. This does not change existing device routes.
    pub fn with_modern_protocol(
        mut self,
        auth: Arc<dyn crate::ModernAuthTransport>,
        reads: Arc<dyn crate::ModernReadTransport>,
    ) -> Self {
        self.modern_auth = Some(auth);
        self.modern_reads = Some(reads);
        self
    }

    /// Trusted composition only. Core must independently grant exact arguments.
    pub fn with_fan_controls(mut self) -> Self {
        self.fan_controls = true;
        self
    }

    /// Trusted opt-in for the exact reference light and purifier settings.
    /// Fan controls and Core argument grants remain independent.
    pub fn with_reference_controls(mut self) -> Self {
        self.reference_controls = true;
        self
    }

    /// Separate trusted composition for the exact outlet. Core grants and actual
    /// trusted approval for elevated targets remain mandatory at every execution.
    pub fn with_outlet_controls(mut self) -> Self {
        self.outlet_controls = true;
        self
    }

    /// Keep startup grant families separate even where capability identifiers overlap.
    /// This narrows adapter admission; it never replaces Core policy or approval.
    pub fn with_control_policies(
        mut self,
        reference: homeverb_core::Policy,
        outlet: homeverb_core::Policy,
    ) -> Self {
        self.control_policies = Some((reference, outlet));
        self
    }

    /// Inspect protected model evidence locally; this never enables a control.
    pub async fn inspect_control_options(
        &self,
        device: &Device,
        connection: &Connection,
        operation: &Operation,
    ) -> Result<Vec<crate::ControlOption>> {
        self.inspect_options(device, connection, operation, false)
            .await
    }

    /// Native inspection including outlet options and local upgrades from valid
    /// older protected derivations. This never saves mappings or grants authority.
    pub async fn inspect_control_options_with_outlets(
        &self,
        device: &Device,
        connection: &Connection,
        operation: &Operation,
    ) -> Result<Vec<crate::ControlOption>> {
        self.inspect_options(device, connection, operation, true)
            .await
    }

    async fn inspect_options(
        &self,
        device: &Device,
        connection: &Connection,
        operation: &Operation,
        include_outlets: bool,
    ) -> Result<Vec<crate::ControlOption>> {
        let id = connection
            .account_id
            .as_deref()
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(conflict)?;
        let account =
            LoadedAccount::load(self.vault.lease(id, operation).await?, false, operation).await?;
        let binding = account
            .payload
            .bindings
            .iter()
            .find(|b| b.device_id.to_string() == device.id)
            .ok_or_else(conflict)?;
        validate_device(binding, id, device, connection)?;
        if !binding.listed {
            return Err(unsupported());
        }
        match binding.mapping.as_ref() {
            Some(mapping) if mapping.is_outlet() && include_outlets => {
                Ok(mapping.upgraded()?.control_options())
            }
            Some(mapping) if mapping.is_outlet() => Ok(Vec::new()),
            Some(mapping) if include_outlets => Ok(mapping.upgraded()?.control_options()),
            Some(mapping) => Ok(mapping.control_options()),
            None => Ok(Vec::new()),
        }
    }

    pub async fn accounts(&self, operation: &Operation) -> Result<Vec<AccountRecord>> {
        Ok(self
            .vault
            .list(operation)
            .await?
            .into_iter()
            .filter(|r| r.provider == PROVIDER && r.region == "CN")
            .collect())
    }
    pub async fn account(&self, id: Uuid, operation: &Operation) -> Result<AccountRecord> {
        homeverb_core::valid_account_id(id)?;
        self.accounts(operation)
            .await?
            .into_iter()
            .find(|r| r.id == id)
            .ok_or_else(|| {
                Error::new(
                    "ACCOUNT_NOT_FOUND",
                    "The selected Mijia account does not exist.",
                )
            })
    }
    pub async fn rename_account(
        &self,
        id: Uuid,
        label: &str,
        operation: &Operation,
    ) -> Result<AccountRecord> {
        self.account(id, operation).await?;
        let mut lease = self.vault.lease(id, operation).await?;
        if lease.record().provider != PROVIDER || lease.record().region != "CN" {
            return Err(unconfirmed());
        }
        lease.rename(label, operation).await?;
        Ok(lease.record().clone())
    }
    pub async fn remove_account(
        &self,
        id: Uuid,
        expected_devices: &[String],
        operation: &Operation,
    ) -> Result<homeverb_core::AccountRemovalResult> {
        self.vault
            .remove_account(id, PROVIDER, "CN", expected_devices, operation)
            .await
    }
    pub async fn inspect_inventory(
        &self,
        id: Uuid,
        operation: &Operation,
    ) -> Result<AccountInventory> {
        let account =
            LoadedAccount::load(self.vault.lease(id, operation).await?, false, operation).await?;
        Ok(AccountInventory {
            account: account.lease.record().clone(),
            devices: account
                .payload
                .bindings
                .iter()
                .map(|b| DeviceInventoryMembership {
                    device_id: b.device_id,
                    connection_id: b.connection_id,
                    listed: b.listed,
                })
                .collect(),
        })
    }

    async fn load_accounts(
        &self,
        pending: Option<Uuid>,
        operation: &Operation,
    ) -> Result<Vec<LoadedAccount>> {
        let records = self.accounts(operation).await?;
        if records.len() > 64 {
            return Err(Error::new(
                "ACCOUNT_SESSION_UNCONFIRMED",
                "The local Mijia account limit requires explicit cleanup.",
            ));
        }
        let mut accounts = vec![];
        for record in records {
            accounts.push(
                LoadedAccount::load(
                    self.vault.lease(record.id, operation).await?,
                    pending == Some(record.id),
                    operation,
                )
                .await?,
            );
        }
        Ok(accounts)
    }

    /// Password buffers are consumed; only provider-validated session material is saved.
    pub async fn connect_devices(
        &self,
        credentials: LoginCredentials,
        existing: Option<Uuid>,
        label: &str,
        operation: &Operation,
    ) -> Result<AccountDiscovery> {
        let label = homeverb_domain::local_name(label)?;
        let _registration = self.vault.lock_registration(operation).await?;
        self.vault.check_secure_storage(operation).await?;
        if let Some(id) = existing {
            self.account(id, operation).await?;
        }
        let accounts = self.load_accounts(existing, operation).await?;
        let session =
            crate::login(self.transport.as_ref(), credentials, Region::Cn, operation).await?;
        self.save_authenticated(session, existing, &label, accounts, operation)
            .await
    }

    async fn save_authenticated(
        &self,
        session: crate::MijiaSession,
        existing: Option<Uuid>,
        label: &str,
        mut accounts: Vec<LoadedAccount>,
        operation: &Operation,
    ) -> Result<AccountDiscovery> {
        if accounts.iter().any(|account| {
            account.payload.modern_session().is_ok()
                && account.payload.subject() != session.user_id.as_str()
                && account.payload.subject() == session.user_id.as_str().trim_start_matches('0')
        }) {
            return Err(Error::new(
                "ACCOUNT_IDENTITY_CONFLICT",
                "Legacy sign-in is ambiguous with a protected modern Mijia identity.",
            ));
        }
        let matches: Vec<_> = accounts
            .iter()
            .enumerate()
            .filter(|(_, a)| a.payload.subject() == session.user_id.as_str())
            .map(|(i, _)| i)
            .collect();
        if matches.len() > 1 {
            return Err(Error::new(
                "ACCOUNT_IDENTITY_CONFLICT",
                "Multiple saved Mijia accounts match the authenticated identity.",
            ));
        }
        let selected = if let Some(id) = existing {
            let index = accounts
                .iter()
                .position(|a| a.lease.record().id == id)
                .ok_or_else(unconfirmed)?;
            if matches.first() != Some(&index) {
                return Err(Error::new(
                    "ACCOUNT_IDENTITY_CONFLICT",
                    "Sign-in did not match the selected protected Mijia identity.",
                ));
            }
            Some(index)
        } else {
            matches.first().copied()
        };
        let mut account = if let Some(index) = selected {
            let mut account = accounts.remove(index);
            let current = account
                .lease
                .inspect_session(operation)
                .await?
                .ok_or_else(unconfirmed)?;
            if current.expose() != account.bytes.expose() {
                return Err(unconfirmed());
            }
            account.payload.session = Some(StoredSession::from_session(session));
            // Explicit reauthorization must confirm the new login even if its bytes match.
            let bytes = account.payload.encode(account.lease.record())?;
            if bytes.expose() == account.bytes.expose() {
                account.lease.confirm_session(&bytes, operation).await?;
            } else {
                account.lease.store_session(&bytes, operation).await?;
                account.bytes = bytes;
            }
            account
        } else {
            if accounts.len() >= 64 {
                return Err(unconfirmed());
            }
            let mut lease = self.vault.prepare(PROVIDER, "CN", label, operation).await?;
            if lease.inspect_session(operation).await?.is_some() {
                return Err(unconfirmed());
            }
            let payload = Payload::new(lease.record(), session);
            let bytes = payload.encode(lease.record())?;
            lease.store_session(&bytes, operation).await?;
            LoadedAccount {
                lease,
                payload,
                bytes,
            }
        };
        let result = self.retrieve(&mut account, &accounts, operation).await;
        Ok(discovery(account.lease.record().clone(), result))
    }

    pub async fn refresh_devices(
        &self,
        id: Uuid,
        operation: &Operation,
    ) -> Result<AccountDiscovery> {
        self.reload(id, false, operation).await
    }
    pub async fn recover_devices(
        &self,
        id: Uuid,
        operation: &Operation,
    ) -> Result<AccountDiscovery> {
        self.reload(id, true, operation).await
    }
    async fn reload(
        &self,
        id: Uuid,
        recover: bool,
        operation: &Operation,
    ) -> Result<AccountDiscovery> {
        self.account(id, operation).await?;
        let _registration = self.vault.lock_registration(operation).await?;
        let mut accounts = self.load_accounts(recover.then_some(id), operation).await?;
        let index = accounts
            .iter()
            .position(|a| a.lease.record().id == id)
            .ok_or_else(unconfirmed)?;
        let mut account = accounts.remove(index);
        // Recovery validates the exact inspected session with one bounded inventory read;
        // it neither logs in again nor treats local bytes as current authorization.
        let result =
            if recover && account.lease.record().credential_state == CredentialState::Pending {
                async {
                    let inventory = account
                        .payload
                        .legacy_session()?
                        .enumerate(self.transport.as_ref(), operation)
                        .await?;
                    account
                        .lease
                        .confirm_session(&account.bytes, operation)
                        .await?;
                    self.bind_inventory(&mut account, &accounts, inventory, operation)
                        .await
                }
                .await
            } else {
                self.retrieve(&mut account, &accounts, operation).await
            };
        Ok(discovery(account.lease.record().clone(), result))
    }

    async fn retrieve(
        &self,
        account: &mut LoadedAccount,
        others: &[LoadedAccount],
        operation: &Operation,
    ) -> Result<BoundDiscovery> {
        let inventory = account
            .payload
            .legacy_session()?
            .enumerate(self.transport.as_ref(), operation)
            .await?;
        self.bind_inventory(account, others, inventory, operation)
            .await
    }

    async fn bind_inventory(
        &self,
        account: &mut LoadedAccount,
        others: &[LoadedAccount],
        inventory: Inventory,
        operation: &Operation,
    ) -> Result<BoundDiscovery> {
        let mut issues = inventory.issues;
        let other_ids: HashSet<_> = others
            .iter()
            .flat_map(|a| a.payload.bindings.iter().map(|b| &b.did))
            .collect();
        for device in &inventory.devices {
            if other_ids.contains(&device.did) {
                return Err(conflict());
            }
            if account
                .payload
                .bindings
                .iter()
                .any(|b| b.did == device.did && b.model != device.model)
            {
                return Err(conflict());
            }
        }
        // Reserve part of the inherited budget for protected persistence and publication.
        // Failed optional public metadata keeps device cards rather than losing inventory.
        let now = tokio::time::Instant::now();
        let metadata = Operation {
            cancellation: operation.cancellation.clone(),
            deadline: now
                + operation
                    .deadline
                    .saturating_duration_since(now)
                    .div_f64(2.0)
                    .min(Duration::from_secs(5)),
        };
        let needs_mapping = inventory.devices.iter().any(|d| {
            ModelMapping::pinned(&d.model, d.spec_type.as_deref())
                .ok()
                .flatten()
                .is_none()
                && account
                    .payload
                    .bindings
                    .iter()
                    .find(|b| b.did == d.did)
                    .is_none_or(|b| b.mapping.is_none() || b.reported_spec != d.spec_type)
        });
        let index = if needs_mapping && !inventory.devices.is_empty() {
            match SpecIndex::fetch(self.transport.as_ref(), &metadata).await {
                Ok(index) => Some(index),
                Err(_) => {
                    operation.run(async { Ok(()) }).await?;
                    issues.push(spec_unavailable());
                    None
                }
            }
        } else {
            None
        };
        let mut model_mappings: HashMap<(String, Option<String>), Option<ModelMapping>> =
            HashMap::new();
        let mut selected = vec![];
        for device in &inventory.devices {
            let position = match account
                .payload
                .bindings
                .iter()
                .position(|b| b.did == device.did)
            {
                Some(index) => index,
                None => {
                    if account.payload.bindings.len() >= 4096 {
                        return Err(unconfirmed());
                    }
                    account.payload.bindings.push(Binding::new(device));
                    account.payload.bindings.len() - 1
                }
            };
            let binding = &mut account.payload.bindings[position];
            binding.name = device.name.clone();
            binding.listed = true;
            if let Some(mapping) = &binding.mapping {
                binding.mapping = Some(mapping.upgraded()?);
            }
            if binding.reported_spec != device.spec_type {
                binding.mapping = None;
                binding.reported_spec = device.spec_type.clone();
            }
            if binding.mapping.is_none() {
                binding.mapping = ModelMapping::pinned(&device.model, device.spec_type.as_deref())?;
            }
            if binding.mapping.is_none()
                && let Some(index) = &index
            {
                let key = (device.model.clone(), device.spec_type.clone());
                if let Some(mapping) = model_mappings.get(&key) {
                    binding.mapping = mapping.clone();
                } else {
                    let mapping = match index
                        .mapping(
                            &device.model,
                            device.spec_type.as_deref(),
                            self.transport.as_ref(),
                            &metadata,
                        )
                        .await
                    {
                        Ok(mapping) => mapping,
                        Err(_) => {
                            operation.run(async { Ok(()) }).await?;
                            issues.push(spec_unavailable());
                            None
                        }
                    };
                    model_mappings.insert(key, mapping.clone());
                    binding.mapping = mapping;
                }
            }
            // Default/native API 1.33 retrieval preserves the previous read-only
            // outlet derivation. Only the explicit typed outlet opt-in upgrades it.
            if !self.outlet_controls
                && let Some(mapping) = &mut binding.mapping
                && mapping.is_outlet()
            {
                for field in &mut mapping.properties {
                    field.write_domain = None;
                }
            }
            selected.push(position);
        }
        // This provider's current enumerator never proves complete membership. Missing
        // historical bindings are retained without inventing exclusion or online evidence.
        if inventory.complete {
            return Err(crate::invalid());
        }
        account.save(operation).await?;
        let descriptors = selected
            .into_iter()
            .map(|i| {
                descriptor(
                    &account.payload.bindings[i],
                    account.lease.record().id,
                    self.fan_controls,
                    self.reference_controls,
                    self.outlet_controls,
                )
            })
            .collect();
        issues.truncate(64);
        operation
            .run(async {
                Ok(BoundDiscovery {
                    descriptors,
                    issues,
                    complete: false,
                })
            })
            .await
    }
}

struct BoundDiscovery {
    descriptors: Vec<Descriptor>,
    issues: Vec<Error>,
    complete: bool,
}
fn discovery(account: AccountRecord, result: Result<BoundDiscovery>) -> AccountDiscovery {
    match result {
        Ok(result) => AccountDiscovery {
            account,
            descriptors: Ok(result.descriptors),
            issues: result.issues,
            complete: result.complete,
        },
        Err(error) => AccountDiscovery {
            account,
            descriptors: Err(error),
            issues: vec![],
            complete: false,
        },
    }
}
fn spec_unavailable() -> Error {
    Error::new(
        "MIJIA_SPEC_UNAVAILABLE",
        "Some device capabilities could not be loaded; their device cards remain available.",
    )
}
fn unsupported() -> Error {
    Error::new(
        "CAPABILITY_UNSUPPORTED",
        "This Mijia device or action has no admitted mapping.",
    )
}

fn descriptor(
    binding: &Binding,
    account: Uuid,
    fan: bool,
    settings: bool,
    outlet: bool,
) -> Descriptor {
    let mut capabilities = binding
        .mapping
        .as_ref()
        .map(|m| m.controlled_capabilities(fan, settings, outlet))
        .unwrap_or_default();
    let supported = !capabilities.is_empty();
    if capabilities.is_empty() {
        capabilities.push(Capability {
            schema_version: "1.0".into(),
            id: "vendor.xiaomi.mijia.inventory".into(),
            component_id: Some("main".into()),
            revision: 1,
            properties: BTreeMap::new(),
            actions: BTreeMap::new(),
            events: BTreeMap::new(),
        });
    }
    let mut components: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for capability in &capabilities {
        components
            .entry(capability.component_id.clone().unwrap())
            .or_default()
            .push(capability.id.clone());
    }
    let mut components: Vec<_> = components
        .into_iter()
        .map(|(id, capabilities)| Component { id, capabilities })
        .collect();
    // Primary appliance services precede auxiliary battery readings for host summaries.
    let family = binding
        .mapping
        .as_ref()
        .and_then(|m| m.spec_type.split(':').nth(3))
        .unwrap_or("")
        .replace('-', "_");
    let primary = match family.as_str() {
        "outlet" => "switch",
        "camera" => "camera_control",
        "lock" => "door",
        other => other,
    };
    components.sort_by_key(|component| {
        (
            !component.id.starts_with(&format!("{primary}-")),
            component.id.starts_with("battery-"),
            component.id.clone(),
        )
    });
    Descriptor {
        device: Device {
            schema_version: "1.0".into(),
            id: binding.device_id.to_string(),
            alias: homeverb_domain::local_name(&binding.name.chars().take(80).collect::<String>())
                .unwrap_or_else(|_| "Mijia device".into()),
            brand: "Unknown".into(),
            model: binding.model.clone(),
            components,
            support_status: if supported {
                "experimental"
            } else {
                "unsupported"
            }
            .into(),
            connections: vec![Connection {
                id: binding.connection_id.to_string(),
                adapter: PROVIDER.into(),
                transport: Transport::Cloud,
                account_id: Some(account.to_string()),
                identity_ref: binding.identity_ref.to_string(),
            }],
        },
        capabilities,
    }
}

fn validate_device(
    binding: &Binding,
    id: Uuid,
    device: &Device,
    connection: &Connection,
) -> Result<()> {
    let expected = descriptor(binding, id, false, false, false);
    if device.model != expected.device.model
        || device.support_status != expected.device.support_status
        || serde_json::to_value(&device.components).map_err(|_| conflict())?
            != serde_json::to_value(&expected.device.components).map_err(|_| conflict())?
        || expected.device.connections.as_slice() != device.connections.as_slice()
        || expected.device.connections.first() != Some(connection)
    {
        return Err(conflict());
    }
    Ok(())
}

#[async_trait]
impl Adapter for MijiaAdapter {
    async fn history_options(
        &self,
        device: &Device,
        connection: &Connection,
        operation: &Operation,
    ) -> Result<Vec<homeverb_domain::HistoryOption>> {
        self.numeric_history_options(device, connection, operation)
            .await
    }
    async fn read_history(
        &self,
        device: &Device,
        connection: &Connection,
        request: &homeverb_domain::HistoryRequest,
        operation: &Operation,
    ) -> Result<homeverb_domain::HistoryResult> {
        self.numeric_history_read(device, connection, request, operation)
            .await
    }
    fn id(&self) -> &str {
        PROVIDER
    }
    async fn discover(&self, operation: &Operation) -> Result<Vec<Descriptor>> {
        // Generic discovery has no partial-result envelope. Refuse before credentials
        // or networking instead of hiding partial coverage behind a successful list.
        if !self.accounts(operation).await?.is_empty() {
            return Err(Error::new(
                "MIJIA_INVENTORY_PARTIAL",
                "Use explicit account refresh for the partial Mijia inventory.",
            ));
        }
        Ok(vec![])
    }
    async fn read(
        &self,
        device: &Device,
        component: &str,
        connection: &Connection,
        operation: &Operation,
    ) -> Result<Snapshot> {
        operation
            .run(async {
                let id = connection
                    .account_id
                    .as_deref()
                    .and_then(|s| Uuid::parse_str(s).ok())
                    .ok_or_else(conflict)?;
                let account =
                    LoadedAccount::load(self.vault.lease(id, operation).await?, false, operation)
                        .await?;
                let binding = account
                    .payload
                    .bindings
                    .iter()
                    .find(|b| b.device_id.to_string() == device.id)
                    .ok_or_else(conflict)?;
                validate_device(binding, id, device, connection)?;
                if !binding.listed {
                    return Err(Error::new(
                        "MIJIA_DEVICE_NOT_LISTED",
                        "The protected device has no admitted account inventory membership.",
                    ));
                }
                let mapping = binding.mapping.as_ref().ok_or_else(unsupported)?;
                let fields: Vec<_> = mapping
                    .properties
                    .iter()
                    .filter(|p| p.component == component)
                    .collect();
                if fields.is_empty() {
                    return Err(unsupported());
                }
                let queries: Vec<_> = fields
                    .iter()
                    .map(|p| PropertyQuery {
                        siid: p.siid,
                        piid: p.piid,
                    })
                    .collect();
                let session = account.payload.legacy_session()?;
                let readings = session
                    .properties(
                        self.transport.as_ref(),
                        &binding.device(),
                        &queries,
                        operation,
                    )
                    .await?;
                let received_at = self.clock.now();
                let mut values = BTreeMap::new();
                for reading in readings {
                    let field = fields
                        .iter()
                        .find(|p| p.siid == reading.query.siid && p.piid == reading.query.piid)
                        .ok_or_else(crate::invalid)?;
                    values.insert(
                        format!("{}.{}", field.capability, field.property),
                        Observation {
                            value: reading.value,
                            source: Transport::Cloud,
                            observed_at: None,
                            received_at,
                            stale: true,
                        },
                    );
                }
                Ok(Snapshot {
                    schema_version: "1.0".into(),
                    device_id: device.id.clone(),
                    component_id: component.into(),
                    from_cache: false,
                    observations: values,
                    warnings: vec![
                        "Mijia cloud readings have no trustworthy device observation timestamp."
                            .into(),
                    ],
                })
            })
            .await
    }
    async fn execute(
        &self,
        validated: &ValidatedCommand,
        connection: &Connection,
        progress: WriteProgress,
        operation: &Operation,
    ) -> Result<AdapterOutcome> {
        if !self.fan_controls && !self.reference_controls && !self.outlet_controls {
            return Err(unsupported());
        }
        operation
            .run(async {
                let command = validated.command();
                let id = connection
                    .account_id
                    .as_deref()
                    .and_then(|s| Uuid::parse_str(s).ok())
                    .ok_or_else(conflict)?;
                let account =
                    LoadedAccount::load(self.vault.lease(id, operation).await?, false, operation)
                        .await?;
                let binding = account
                    .payload
                    .bindings
                    .iter()
                    .find(|b| b.device_id.to_string() == command.device_id)
                    .ok_or_else(conflict)?;
                if descriptor(binding, id, false, false, false)
                    .device
                    .connections
                    .first()
                    != Some(connection)
                {
                    return Err(conflict());
                }
                if !binding.listed {
                    return Err(unsupported());
                }
                let mapping = binding.mapping.as_ref().ok_or_else(unsupported)?;
                // Re-derive action semantics from protected model/spec evidence, never from
                // the public descriptor's risk, parameters, wire IDs or writable flag.
                let (field, option) = mapping
                    .properties
                    .iter()
                    .filter_map(|p| mapping.control_option(p).map(|o| (p, o)))
                    .find(|(_, o)| {
                        o.component_id == command.component_id
                            && o.capability == command.capability
                            && o.action == command.action
                    })
                    .ok_or_else(unsupported)?;
                if !mapping.control_family_enabled(
                    field,
                    self.fan_controls,
                    self.reference_controls,
                    self.outlet_controls,
                ) {
                    return Err(unsupported());
                }
                if option.definition.risk != homeverb_domain::Risk::Low
                    && !validated.has_user_approval()
                {
                    return Err(Error::new(
                        "POLICY_DENIED",
                        "This outlet setting requires trusted user approval.",
                    ));
                }
                if let Some((reference, outlet)) = &self.control_policies {
                    let family = if mapping.is_outlet() {
                        Some(outlet)
                    } else if crate::reference_setting_option(
                        &option.component_id,
                        option.definition.parameters["required"][0]
                            .as_str()
                            .unwrap_or(""),
                    )
                    .is_some()
                    {
                        Some(reference)
                    } else {
                        None
                    };
                    if let Some(policy) = family {
                        // Risk is independently checked above. This check admits only
                        // exact targets from the correct trusted startup family.
                        let mut scope = option.definition.clone();
                        scope.risk = homeverb_domain::Risk::Low;
                        if !policy
                            .authorize(command, &scope)?
                            .contains(&Transport::Cloud)
                        {
                            return Err(Error::new(
                                "POLICY_DENIED",
                                "The control family has no exact grant.",
                            ));
                        }
                    }
                }
                option.definition.validate_arguments(&command.arguments)?;
                account
                    .payload
                    .legacy_session()?
                    .set_property(
                        self.transport.as_ref(),
                        &binding.device(),
                        &crate::PropertyWrite {
                            query: PropertyQuery {
                                siid: field.siid,
                                piid: field.piid,
                            },
                            value: command.arguments[&field.property].clone(),
                        },
                        &progress,
                        operation,
                    )
                    .await
            })
            .await
    }
}

#[cfg(test)]
mod tests;
