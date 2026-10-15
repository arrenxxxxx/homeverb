//! Private account/session storage and stable cloud-device bindings.

use crate::{
    Appliance, ApplianceFamily, CloudId, CloudTransport, Inventory, LoginCredentials,
    MAX_RESPONSE_BYTES, MeijuSession, ReadTransport, StateQuery, login, read_cloud_state,
    retrieve_inventory,
};
use homeverb_core::{
    AccountLease, AccountRecord, AccountVault, Clock, CredentialRef, Operation, SecretBytes,
    valid_account_id,
};
use homeverb_domain::{Connection, Device, Error, Result, Snapshot, Transport};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt};
use uuid::Uuid;
use zeroize::Zeroize;

const PROVIDER: &str = "midea.meiju";
const MAX_BINDINGS: usize = 4096;

/// Random local identifiers only. A binding grants neither capabilities nor permissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceBinding {
    pub device_id: Uuid,
    pub connection_id: Uuid,
    pub identity_ref: Uuid,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredBinding {
    appliance_id: String,
    family: u16,
    model: Option<String>,
    local: DeviceBinding,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Payload {
    version: u8,
    account_id: Uuid,
    credential_ref: CredentialRef,
    provider: String,
    region: String,
    subject: Vec<u8>,
    session: Vec<u8>,
    bindings: Vec<StoredBinding>,
}

impl Drop for Payload {
    fn drop(&mut self) {
        self.subject.zeroize();
        self.session.zeroize();
        for binding in &mut self.bindings {
            binding.appliance_id.zeroize();
        }
    }
}

/// Holds the account lease, not an authenticated connection or a session cache.
/// ```compile_fail
/// fn export(account: &homeverb_midea::MeijuAccount) {
///     let _ = serde_json::to_string(account);
/// }
/// ```
pub struct MeijuAccount {
    lease: AccountLease,
}

impl fmt::Debug for MeijuAccount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("MeijuAccount([REDACTED])")
    }
}

fn unconfirmed() -> Error {
    Error::new(
        "ACCOUNT_SESSION_UNCONFIRMED",
        "The protected account state requires trusted validation and recovery.",
    )
}

fn identity_conflict() -> Error {
    Error::new(
        "DEVICE_IDENTITY_CONFLICT",
        "The account inventory conflicts with an established appliance binding.",
    )
}

fn valid_material(subject: &[u8], session: &[u8]) -> bool {
    !subject.is_empty() && subject.len() <= 1024 && !session.is_empty() && session.len() <= 65536
}

fn valid_model(model: &Option<String>) -> bool {
    model.as_ref().is_none_or(|model| {
        !model.is_empty() && model.len() <= 256 && !model.chars().any(char::is_control)
    })
}

fn family_code(family: ApplianceFamily) -> u16 {
    match family {
        ApplianceFamily::AirConditioner => 0xac,
        ApplianceFamily::Dishwasher => 0xe1,
        ApplianceFamily::Refrigerator => 0xca,
        ApplianceFamily::Other(code) => code,
    }
}

impl Payload {
    fn validate(&self, record: &AccountRecord) -> Result<()> {
        if self.version != 1
            || self.account_id != record.id
            || self.credential_ref != record.credential_ref
            || self.provider != PROVIDER
            || self.provider != record.provider
            || self.region != record.region
            || !valid_material(&self.subject, &self.session)
            || self.bindings.len() > MAX_BINDINGS
        {
            return Err(unconfirmed());
        }
        let mut appliances = HashSet::new();
        let mut locals = HashSet::new();
        for binding in &self.bindings {
            let id = CloudId::parse(&serde_json::Value::String(binding.appliance_id.clone()))
                .map_err(|_| unconfirmed())?;
            if id.expose() != binding.appliance_id
                || !appliances.insert(id)
                || !valid_model(&binding.model)
            {
                return Err(unconfirmed());
            }
            for id in [
                binding.local.device_id,
                binding.local.connection_id,
                binding.local.identity_ref,
            ] {
                valid_account_id(id).map_err(|_| unconfirmed())?;
                if !locals.insert(id) {
                    return Err(unconfirmed());
                }
            }
        }
        Ok(())
    }
}

impl MeijuAccount {
    /// Load an admitted session and confirm its subject against the protected account binding.
    pub async fn cloud_session(&self, operation: &Operation) -> Result<MeijuSession> {
        if self.record().region != "CN" {
            return Err(Error::new(
                "AUTH_FLOW_UNSUPPORTED",
                "Only mainland Meiju is admitted.",
            ));
        }
        let payload = self.load(operation).await?;
        let session = MeijuSession::restore(&SecretBytes::new(payload.session.clone()))?;
        if session.subject().expose() != payload.subject {
            return Err(unconfirmed());
        }
        Ok(session)
    }

    /// Explicit retrieval only. Keep partial failures and preserve existing bindings.
    pub async fn retrieve(
        &mut self,
        transport: &dyn CloudTransport,
        operation: &Operation,
    ) -> Result<BoundInventory> {
        let session = self.cloud_session(operation).await?;
        let reference = self.record().credential_ref;
        let inventory =
            retrieve_inventory(&session.reads(transport, reference), reference, operation).await?;
        let bindings = self.bind_inventory(&inventory, operation).await?;
        Ok(BoundInventory {
            inventory,
            bindings,
        })
    }

    /// Native authentication must validate both inputs and their association before this call.
    /// Never use a display name, mutable login spelling, or unverified UI input as the subject.
    pub async fn create(
        vault: &AccountVault,
        region: &str,
        label: &str,
        subject: &SecretBytes,
        session: &SecretBytes,
        operation: &Operation,
    ) -> Result<Self> {
        operation
            .run(async {
                if !valid_material(subject.expose(), session.expose()) {
                    return Err(Error::invalid());
                }
                Ok(())
            })
            .await?;
        let lease = vault.prepare(PROVIDER, region, label, operation).await?;
        let mut account = Self { lease };
        if account.lease.inspect_session(operation).await?.is_some() {
            return Err(unconfirmed());
        }
        let payload = Payload {
            version: 1,
            account_id: account.record().id,
            credential_ref: account.record().credential_ref,
            provider: account.record().provider.clone(),
            region: account.record().region.clone(),
            subject: subject.expose().to_vec(),
            session: session.expose().to_vec(),
            bindings: Vec::new(),
        };
        account.save(&payload, operation).await?;
        Ok(account)
    }

    /// Local validation only; successful opening does not establish provider authorization.
    pub async fn open(lease: AccountLease, operation: &Operation) -> Result<Self> {
        let account = Self { lease };
        account.load(operation).await?;
        Ok(account)
    }

    pub fn record(&self) -> &AccountRecord {
        self.lease.record()
    }

    pub async fn session(&self, operation: &Operation) -> Result<SecretBytes> {
        let mut payload = self.load(operation).await?;
        Ok(SecretBytes::new(std::mem::take(&mut payload.session)))
    }

    /// Replace only after native provider validation; keep every established device binding.
    pub async fn replace_session(
        &mut self,
        subject: &SecretBytes,
        session: &SecretBytes,
        operation: &Operation,
    ) -> Result<()> {
        let mut payload = self.load(operation).await?;
        if payload.subject != subject.expose() {
            return Err(Error::new(
                "ACCOUNT_IDENTITY_CONFLICT",
                "The authorized provider identity differs from this local account.",
            ));
        }
        if !valid_material(subject.expose(), session.expose()) {
            return Err(Error::invalid());
        }
        payload.session.zeroize();
        payload.session = session.expose().to_vec();
        self.save(&payload, operation).await
    }

    /// Return bindings in inventory order only after all new identity evidence is persisted.
    /// Missing entries never remove older bindings, even for a complete inventory.
    pub async fn bind_inventory(
        &mut self,
        inventory: &Inventory,
        operation: &Operation,
    ) -> Result<Vec<DeviceBinding>> {
        let mut payload = self.load(operation).await?;
        let (bindings, changed) = operation
            .run(async {
                if inventory.account_ref != self.record().credential_ref {
                    return Err(identity_conflict());
                }
                if inventory.appliances.len() > MAX_BINDINGS {
                    return Err(Error::invalid());
                }
                let mut seen = HashSet::new();
                let mut result = Vec::new();
                let mut changed = false;
                for appliance in &inventory.appliances {
                    if !seen.insert(&appliance.id) || !valid_model(&appliance.model) {
                        return Err(identity_conflict());
                    }
                    let family = family_code(appliance.family);
                    if let Some(binding) = payload
                        .bindings
                        .iter_mut()
                        .find(|entry| entry.appliance_id == appliance.id.expose())
                    {
                        let model_conflicts = matches!(
                            (&binding.model, &appliance.model), (Some(a), Some(b)) if a != b
                        );
                        if binding.family != family || model_conflicts {
                            return Err(identity_conflict());
                        }
                        if binding.model.is_none() && appliance.model.is_some() {
                            binding.model = appliance.model.clone();
                            changed = true;
                        }
                        result.push(binding.local);
                    } else {
                        if payload.bindings.len() >= MAX_BINDINGS {
                            return Err(Error::invalid());
                        }
                        let local = DeviceBinding {
                            device_id: Uuid::new_v4(),
                            connection_id: Uuid::new_v4(),
                            identity_ref: Uuid::new_v4(),
                        };
                        payload.bindings.push(StoredBinding {
                            appliance_id: appliance.id.expose().into(),
                            family,
                            model: appliance.model.clone(),
                            local,
                        });
                        result.push(local);
                        changed = true;
                    }
                }
                Ok((result, changed))
            })
            .await?;
        if changed {
            self.save(&payload, operation).await?;
        }
        operation.run(async { Ok(bindings) }).await
    }

    /// Resolve within this leased account only, before issuing an appliance request.
    pub async fn resolve_device(&self, device_id: Uuid, operation: &Operation) -> Result<CloudId> {
        let payload = self.load(operation).await?;
        let binding = payload
            .bindings
            .iter()
            .find(|binding| binding.local.device_id == device_id)
            .ok_or_else(|| {
                Error::new(
                    "DEVICE_NOT_FOUND",
                    "The device has no binding in this account.",
                )
            })?;
        CloudId::parse(&serde_json::Value::String(binding.appliance_id.clone()))
    }

    /// Verify the selected local binding before one default cloud query. Core must still
    /// admit the descriptor and validate every returned property before caching the result.
    pub async fn read_bound_state(
        &self,
        transport: &dyn ReadTransport,
        device: &Device,
        component: &str,
        connection: &Connection,
        clock: &dyn Clock,
        operation: &Operation,
    ) -> Result<Snapshot> {
        let payload = self.load(operation).await?;
        let appliance = operation
            .run(async {
                let binding = payload
                    .bindings
                    .iter()
                    .find(|binding| binding.local.device_id.to_string() == device.id)
                    .ok_or_else(identity_conflict)?;
                let expected = Connection {
                    id: binding.local.connection_id.to_string(),
                    adapter: PROVIDER.into(),
                    transport: Transport::Cloud,
                    account_id: Some(self.record().id.to_string()),
                    identity_ref: binding.local.identity_ref.to_string(),
                };
                if *connection != expected
                    || !device.connections.contains(connection)
                    || binding
                        .model
                        .as_ref()
                        .is_some_and(|model| model != &device.model)
                {
                    return Err(identity_conflict());
                }
                let family = match binding.family {
                    0xac if component == "main" => ApplianceFamily::AirConditioner,
                    0xe1 if component == "main" => ApplianceFamily::Dishwasher,
                    0xca if matches!(component, "fridge" | "freezer") => {
                        ApplianceFamily::Refrigerator
                    }
                    _ => return Err(crate::state::unsupported()),
                };
                if !device.components.iter().any(|entry| entry.id == component) {
                    return Err(crate::state::unsupported());
                }
                Ok(Appliance {
                    id: CloudId::parse(&serde_json::Value::String(binding.appliance_id.clone()))?,
                    family,
                    name: None,
                    model: binding.model.clone(),
                    online_hint: None,
                })
            })
            .await?;
        drop(payload);
        let state = read_cloud_state(
            transport,
            self.record().credential_ref,
            &appliance,
            StateQuery::Default,
            clock,
            operation,
        )
        .await?;
        operation
            .run(async { state.snapshot(&device.id, component) })
            .await
    }

    async fn load(&self, operation: &Operation) -> Result<Payload> {
        let bytes = self.lease.read_session(operation).await?;
        let payload = operation
            .run(async {
                if bytes.expose().len() > MAX_RESPONSE_BYTES {
                    return Err(unconfirmed());
                }
                let payload: Payload =
                    serde_json::from_slice(bytes.expose()).map_err(|_| unconfirmed())?;
                payload.validate(self.record())?;
                Ok(payload)
            })
            .await?;
        operation.run(async { Ok(payload) }).await
    }

    async fn save(&mut self, payload: &Payload, operation: &Operation) -> Result<()> {
        let bytes = operation
            .run(async {
                payload.validate(self.record())?;
                let bytes =
                    SecretBytes::new(serde_json::to_vec(payload).map_err(|_| unconfirmed())?);
                if bytes.expose().len() > MAX_RESPONSE_BYTES {
                    return Err(Error::invalid());
                }
                Ok(bytes)
            })
            .await?;
        self.lease.store_session(&bytes, operation).await
    }
}

/// Private protocol inventory paired with stable local identifiers; not capability admission.
#[derive(Debug)]
pub struct BoundInventory {
    pub inventory: Inventory,
    pub bindings: Vec<DeviceBinding>,
}

/// Session persistence and retrieval are separate outcomes. No lease escapes this operation.
#[derive(Debug)]
pub struct ConnectReport {
    pub account: AccountRecord,
    pub retrieval: Result<BoundInventory>,
}

/// Explicit mainland Meiju connection. Reuse an exact subject match; never merge by login spelling.
/// Consumes credentials so they are dropped before persistence and inventory work.
pub async fn connect(
    vault: &AccountVault,
    transport: &dyn CloudTransport,
    credentials: LoginCredentials,
    existing: Option<Uuid>,
    label: &str,
    operation: &Operation,
) -> Result<ConnectReport> {
    let label = homeverb_domain::local_name(label)?;
    let _registration = vault.lock_registration(operation).await?;
    vault.check_secure_storage(operation).await?;
    let mut selected = if let Some(id) = existing {
        let account = MeijuAccount::open(vault.lease(id, operation).await?, operation).await?;
        if account.record().region != "CN" {
            return Err(Error::new(
                "AUTH_FLOW_UNSUPPORTED",
                "Only mainland Meiju is admitted.",
            ));
        }
        Some(account)
    } else {
        None
    };
    let mut subjects = Vec::new();
    if selected.is_none() {
        for record in vault.list(operation).await? {
            if record.provider != PROVIDER || record.region != "CN" {
                continue;
            }
            let account =
                MeijuAccount::open(vault.lease(record.id, operation).await?, operation).await?;
            let mut payload = account.load(operation).await?;
            subjects.push((
                record.id,
                SecretBytes::new(std::mem::take(&mut payload.subject)),
            ));
        }
    }
    let session = login(transport, &credentials, operation).await?;
    drop(credentials);
    if selected.is_none() {
        let matches: Vec<_> = subjects
            .iter()
            .filter(|(_, subject)| subject.expose() == session.subject().expose())
            .map(|(id, _)| *id)
            .collect();
        if matches.len() > 1 {
            return Err(Error::new(
                "ACCOUNT_IDENTITY_CONFLICT",
                "Multiple local accounts require explicit identity reconciliation.",
            ));
        }
        if let Some(id) = matches.first() {
            selected =
                Some(MeijuAccount::open(vault.lease(*id, operation).await?, operation).await?);
        }
    }
    let encoded = session.encode()?;
    let mut account = match selected {
        Some(mut account) => {
            account
                .replace_session(session.subject(), &encoded, operation)
                .await?;
            account
        }
        None => {
            MeijuAccount::create(vault, "CN", &label, session.subject(), &encoded, operation)
                .await?
        }
    };
    let retrieval = account.retrieve(transport, operation).await;
    Ok(ConnectReport {
        account: account.record().clone(),
        retrieval,
    })
}
