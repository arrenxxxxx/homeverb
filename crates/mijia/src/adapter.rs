//! Account-bound Core reads and local card identity. Construction performs no I/O.

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

pub struct MijiaAdapter {
    vault: Arc<AccountVault>,
    transport: Arc<dyn CloudTransport>,
    clock: Arc<dyn Clock>,
}
impl MijiaAdapter {
    pub fn new(
        vault: Arc<AccountVault>,
        transport: Arc<dyn CloudTransport>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            vault,
            transport,
            clock,
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
        let mut accounts = self.load_accounts(existing, operation).await?;
        let session =
            crate::login(self.transport.as_ref(), credentials, Region::Cn, operation).await?;
        let matches: Vec<_> = accounts
            .iter()
            .enumerate()
            .filter(|(_, a)| a.payload.session.subject() == session.user_id.as_str())
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
            account.payload.session = StoredSession::from_session(session);
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
            let mut lease = self
                .vault
                .prepare(PROVIDER, "CN", &label, operation)
                .await?;
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
                        .session
                        .session()?
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
            .session
            .session()?
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
            account
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
            if binding.reported_spec != device.spec_type {
                binding.mapping = None;
                binding.reported_spec = device.spec_type.clone();
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
            .map(|i| descriptor(&account.payload.bindings[i], account.lease.record().id))
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

fn descriptor(binding: &Binding, account: Uuid) -> Descriptor {
    let mut capabilities = binding
        .mapping
        .as_ref()
        .map(ModelMapping::capabilities)
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
    components
        .sort_by_key(|component| (component.id.starts_with("battery-"), component.id.clone()));
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

#[async_trait]
impl Adapter for MijiaAdapter {
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
                let expected = descriptor(binding, id);
                if device.model != expected.device.model
                    || device.support_status != expected.device.support_status
                    || serde_json::to_value(&device.components).map_err(|_| conflict())?
                        != serde_json::to_value(&expected.device.components)
                            .map_err(|_| conflict())?
                    || expected.device.connections.as_slice() != device.connections.as_slice()
                    || expected.device.connections.first() != Some(connection)
                {
                    return Err(conflict());
                }
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
                let session = account.payload.session.session()?;
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
        _: &ValidatedCommand,
        _: &Connection,
        _: WriteProgress,
        _: &Operation,
    ) -> Result<AdapterOutcome> {
        Err(unsupported())
    }
}

#[cfg(test)]
mod tests;
