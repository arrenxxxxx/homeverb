//! Private account/session storage and stable cloud-device bindings.

use crate::{
    Appliance, ApplianceFamily, CloudId, CloudTransport, Inventory, LoginCredentials,
    MAX_RESPONSE_BYTES, MeijuSession, ProductProfile, ReadTransport, StateQuery, login,
    read_cloud_state, retrieve_inventory,
};
use homeverb_core::{
    AccountLease, AccountRecord, AccountVault, Clock, CredentialRef, CredentialState, Operation,
    SecretBytes, valid_account_id,
};
use homeverb_domain::{Connection, Device, Error, Result, Snapshot, Transport};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt,
};
use uuid::Uuid;
use zeroize::Zeroize;

mod conflicts;
pub(crate) mod identity_recovery;
mod lan;
pub use identity_recovery::PendingIdentityRecovery;
pub use lan::LanCandidateMatch;

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

/// Saved membership evidence only; no provider identifiers or operational authority.
pub struct DeviceInventoryMembership {
    pub device_id: Uuid,
    pub connection_id: Uuid,
    pub listed: Option<bool>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredBinding {
    appliance_id: String,
    family: u16,
    model: Option<String>,
    local: DeviceBinding,
    #[serde(default)]
    listed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    product_profile: Option<crate::product::StoredProfile>,
}

impl StoredBinding {
    fn require_listed(&self) -> Result<()> {
        match self.listed {
            Some(true) => Ok(()),
            Some(false) => Err(Error::new(
                "MIDEA_DEVICE_NOT_LISTED",
                "The device was absent from the last complete account inventory. Refresh the account before trying again.",
            )),
            None => Err(Error::new(
                "MIDEA_INVENTORY_REQUIRED",
                "Refresh the account to establish complete device inventory before live access.",
            )),
        }
    }
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

pub(crate) fn identity_conflict() -> Error {
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
    fn decode(bytes: &SecretBytes, record: &AccountRecord) -> Result<Self> {
        if bytes.expose().len() > MAX_RESPONSE_BYTES {
            return Err(unconfirmed());
        }
        let mut payload: Self =
            serde_json::from_slice(bytes.expose()).map_err(|_| unconfirmed())?;
        payload.validate(record)?;
        // Decode legacy identities without writing; the next explicit update uses version 3.
        payload.version = 3;
        Ok(payload)
    }

    fn cloud_session(&self) -> Result<MeijuSession> {
        let session = MeijuSession::restore(&SecretBytes::new(self.session.clone()))?;
        if session.subject().expose() != self.subject {
            return Err(unconfirmed());
        }
        Ok(session)
    }

    fn validate(&self, record: &AccountRecord) -> Result<()> {
        if !matches!(self.version, 1..=3)
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
                || (self.version == 1 && binding.listed.is_some())
                || (self.version < 3 && binding.product_profile.is_some())
            {
                return Err(unconfirmed());
            }
            if let Some(profile) = &binding.product_profile {
                profile.validate().map_err(|_| unconfirmed())?;
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
        self.load(operation).await?.cloud_session()
    }

    /// Explicit recovery validates saved credentials with reads, never a login or control.
    pub(crate) async fn recover(
        lease: AccountLease,
        transport: &dyn CloudTransport,
        operation: &Operation,
    ) -> ConnectReport {
        let mut account = Self { lease };
        let retrieval = account.recover_inventory(transport, operation).await;
        ConnectReport {
            account: account.record().clone(),
            retrieval,
        }
    }

    async fn recover_inventory(
        &mut self,
        transport: &dyn CloudTransport,
        operation: &Operation,
    ) -> Result<BoundInventory> {
        if self.record().provider != PROVIDER || self.record().region != "CN" {
            return Err(unconfirmed());
        }
        if self.record().credential_state == CredentialState::Stored {
            return self.retrieve(transport, operation).await;
        }
        let bytes = self
            .lease
            .inspect_session(operation)
            .await?
            .ok_or_else(|| {
                Error::new(
                    "AUTH_REQUIRED",
                    "The account has no locally stored session.",
                )
            })?;
        let session = operation
            .run(async { Payload::decode(&bytes, self.record())?.cloud_session() })
            .await?;
        let reference = self.record().credential_ref;
        let inventory =
            retrieve_inventory(&session.reads(transport, reference), reference, operation).await?;
        if !inventory.is_complete() {
            if let Some(failure) = inventory
                .failures
                .iter()
                .find(|failure| failure.error.code == "AUTH_REQUIRED")
            {
                return Err(failure.error.clone());
            }
            return Err(Error::new(
                "MIDEA_INVENTORY_PARTIAL",
                "Recovery requires complete cloud retrieval; the account remains unconfirmed.",
            ));
        }
        self.lease.confirm_session(&bytes, operation).await?;
        let bindings = self.bind_inventory(&inventory, operation).await?;
        self.lease.record_complete_inventory(operation).await?;
        Ok(BoundInventory {
            inventory,
            bindings,
        })
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
        if inventory.is_complete() {
            self.lease.record_complete_inventory(operation).await?;
        }
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
            version: 3,
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

    pub(crate) async fn inspect_inventory(
        &self,
        operation: &Operation,
    ) -> Result<Vec<DeviceInventoryMembership>> {
        let payload = self.load(operation).await?;
        operation
            .run(async {
                let mut devices: Vec<_> = payload
                    .bindings
                    .iter()
                    .map(|binding| DeviceInventoryMembership {
                        device_id: binding.local.device_id,
                        connection_id: binding.local.connection_id,
                        listed: binding.listed,
                    })
                    .collect();
                devices.sort_by_key(|entry| entry.device_id);
                Ok(devices)
            })
            .await
    }

    pub async fn session(&self, operation: &Operation) -> Result<SecretBytes> {
        let mut payload = self.load(operation).await?;
        Ok(SecretBytes::new(std::mem::take(&mut payload.session)))
    }

    /// Include historical bindings when checking cross-account ambiguity before publication.
    pub(crate) async fn bound_appliances(&self, operation: &Operation) -> Result<Vec<CloudId>> {
        let payload = self.load(operation).await?;
        payload
            .bindings
            .iter()
            .map(|binding| CloudId::parse(&serde_json::Value::String(binding.appliance_id.clone())))
            .collect()
    }

    /// Retained product evidence for explicitly opted-in descriptor construction.
    pub(crate) async fn product_profiles(
        &self,
        operation: &Operation,
    ) -> Result<HashMap<Uuid, ProductProfile>> {
        let payload = self.load(operation).await?;
        Ok(payload
            .bindings
            .iter()
            .map(|binding| {
                (
                    binding.local.device_id,
                    ProductProfile::from_stored(&binding.product_profile),
                )
            })
            .collect())
    }

    /// Model admission must agree with protected evidence, not only public device metadata.
    pub(crate) async fn validate_read_model(
        &self,
        device: &Device,
        family: ApplianceFamily,
        operation: &Operation,
    ) -> Result<()> {
        let payload = self.load(operation).await?;
        if !payload.bindings.iter().any(|binding| {
            binding.local.device_id.to_string() == device.id
                && binding.model.as_deref() == Some(device.model.as_str())
                && binding.family == family_code(family)
        }) {
            return Err(identity_conflict());
        }
        Ok(())
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

    /// Inspect identity for an explicit login without declaring the old session usable.
    async fn reauthorization_material(&self, operation: &Operation) -> Result<SecretBytes> {
        let bytes = self
            .lease
            .inspect_session(operation)
            .await?
            .ok_or_else(unconfirmed)?;
        operation
            .run(async {
                Payload::decode(&bytes, self.record())?.cloud_session()?;
                Ok(bytes)
            })
            .await
    }

    /// Replace only unchanged protected bindings after a fresh same-subject login.
    async fn reauthorize(
        &mut self,
        original: &SecretBytes,
        session: &MeijuSession,
        operation: &Operation,
    ) -> Result<()> {
        let mut payload = operation
            .run(async {
                let payload = Payload::decode(original, self.record())?;
                if crate::auth_trace::enabled() {
                    session.trace_reauthorization(&payload.cloud_session()?);
                }
                if !session.matches_saved_subject(&payload.subject) {
                    return Err(Error::new(
                        "ACCOUNT_IDENTITY_CONFLICT",
                        "The authorized provider identity differs from this local account.",
                    ));
                }
                Ok(payload)
            })
            .await?;
        let current = self.lease.inspect_session(operation).await?;
        if current
            .as_ref()
            .is_none_or(|bytes| bytes.expose() != original.expose())
        {
            return Err(unconfirmed());
        }
        payload.session.zeroize();
        payload.session = session.encode()?.expose().to_vec();
        payload.subject.zeroize();
        payload.subject = session.subject().expose().to_vec();
        self.save(&payload, operation).await
    }

    /// Return bindings in inventory order only after all new identity evidence is persisted.
    /// Complete inventory updates membership; missing entries never remove older identities.
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
                    if !seen.insert(appliance.id.expose()) || !valid_model(&appliance.model) {
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
                        let mut profile = ProductProfile::from_stored(&binding.product_profile);
                        if profile.merge(&appliance.product_profile)? {
                            binding.product_profile = profile.stored();
                            changed = true;
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
                            listed: None,
                            product_profile: appliance.product_profile.stored(),
                        });
                        result.push(local);
                        changed = true;
                    }
                }
                if inventory.is_complete() {
                    for binding in &mut payload.bindings {
                        let listed = Some(seen.contains(binding.appliance_id.as_str()));
                        if binding.listed != listed {
                            binding.listed = listed;
                            changed = true;
                        }
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

    /// Resolve historical identity only. This does not admit a live appliance request.
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

    /// Resolve a control only from protected evidence, never a public provider identifier.
    pub(crate) async fn control_appliance(
        &self,
        device_id: &str,
        connection: &Connection,
        operation: &Operation,
    ) -> Result<Appliance> {
        let appliance = self
            .bound_appliance(device_id, connection, operation)
            .await?;
        if !matches!(
            appliance.family,
            ApplianceFamily::AirConditioner | ApplianceFamily::Dishwasher
        ) {
            return Err(crate::control::unsupported());
        }
        Ok(appliance)
    }

    /// Resolve saved identity and listed membership without granting control authority.
    pub(crate) async fn bound_appliance(
        &self,
        device_id: &str,
        connection: &Connection,
        operation: &Operation,
    ) -> Result<Appliance> {
        let payload = self.load(operation).await?;
        let binding = payload
            .bindings
            .iter()
            .find(|binding| binding.local.device_id.to_string() == device_id)
            .ok_or_else(identity_conflict)?;
        let expected = Connection {
            id: binding.local.connection_id.to_string(),
            adapter: PROVIDER.into(),
            transport: Transport::Cloud,
            account_id: Some(self.record().id.to_string()),
            identity_ref: binding.local.identity_ref.to_string(),
        };
        if *connection != expected {
            return Err(identity_conflict());
        }
        binding.require_listed()?;
        let family = match binding.family {
            0xac => ApplianceFamily::AirConditioner,
            0xe1 => ApplianceFamily::Dishwasher,
            0xca => ApplianceFamily::Refrigerator,
            value => ApplianceFamily::Other(value),
        };
        Ok(Appliance {
            id: CloudId::parse(&serde_json::Value::String(binding.appliance_id.clone()))?,
            family,
            name: None,
            model: binding.model.clone(),
            online_hint: None,
            product_profile: ProductProfile::from_stored(&binding.product_profile),
        })
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
        self.read_bound_state_with_features(
            transport, device, component, connection, clock, operation, false,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn read_bound_state_with_features(
        &self,
        transport: &dyn ReadTransport,
        device: &Device,
        component: &str,
        connection: &Connection,
        clock: &dyn Clock,
        operation: &Operation,
        include_features: bool,
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
                binding.require_listed()?;
                let family = match binding.family {
                    0xac if component == "main" => ApplianceFamily::AirConditioner,
                    0xe1 if component == "main" => ApplianceFamily::Dishwasher,
                    0xca if matches!(component, "fridge" | "freezer")
                        || (component == "variable"
                            && ProductProfile::from_stored(&binding.product_profile)
                                .three_zone_refrigerator(binding.model.as_deref())) =>
                    {
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
                    product_profile: ProductProfile::from_stored(&binding.product_profile),
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
        let mut snapshot = state.snapshot(&device.id, component)?;
        if !device.components.iter().any(|part| {
            part.id == component
                && part
                    .capabilities
                    .iter()
                    .any(|id| id == crate::refrigerator::DETAILS)
        }) {
            snapshot
                .observations
                .retain(|name, _| !name.starts_with("vendor.midea.refrigerator.details."));
        }
        if !device.components.iter().any(|part| {
            part.id == component
                && part
                    .capabilities
                    .iter()
                    .any(|id| id == "vendor.midea.refrigerator")
        }) {
            // Saved descriptors are upgraded only by explicit inventory refresh.
            snapshot
                .observations
                .retain(|name, _| !name.starts_with("vendor.midea.refrigerator."));
        }
        if include_features
            && let Some(profile) = appliance
                .product_profile
                .ac_features(appliance.model.as_deref())
        {
            let mut raw = state.ac_fields;
            for query in profile.queries() {
                if !profile.needs_query(query, &raw) {
                    continue;
                }
                match read_cloud_state(
                    transport,
                    self.record().credential_ref,
                    &appliance,
                    query,
                    clock,
                    operation,
                )
                .await
                {
                    Ok(state) => {
                        snapshot
                            .observations
                            .extend(state.snapshot(&device.id, component)?.observations);
                        raw.extend(state.ac_fields);
                    }
                    Err(error)
                        if matches!(
                            error.code.as_str(),
                            "CANCELLED" | "DEADLINE_EXCEEDED" | "AUTH_REQUIRED"
                        ) =>
                    {
                        return Err(error);
                    }
                    Err(_) => {
                        if !snapshot
                            .warnings
                            .iter()
                            .any(|w| w == "An optional AC state query is unavailable.")
                        {
                            snapshot
                                .warnings
                                .push("An optional AC state query is unavailable.".into());
                        }
                    }
                }
            }
        }
        if !include_features
            && let Some(profile) = appliance
                .product_profile
                .ac_features(appliance.model.as_deref())
        {
            // Legacy saved descriptors can still contain only the basic AC properties.
            // Optional observations are admitted with the host's selected-device opt-in.
            for feature in &profile.features {
                snapshot
                    .observations
                    .remove(&format!("vendor.midea.ac.{}", feature.name));
            }
        }
        operation.run(async { Ok(snapshot) }).await
    }

    async fn load(&self, operation: &Operation) -> Result<Payload> {
        let bytes = self.lease.read_session(operation).await?;
        let payload = operation
            .run(async { Payload::decode(&bytes, self.record()) })
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
        crate::auth_trace::emit(crate::auth_trace::Event::SelectedAccount);
        let account = MeijuAccount {
            lease: vault.lease(id, operation).await?,
        };
        if account.record().provider != PROVIDER || account.record().region != "CN" {
            return Err(Error::new(
                "AUTH_FLOW_UNSUPPORTED",
                "Only mainland Meiju is admitted.",
            ));
        }
        let original = account.reauthorization_material(operation).await?;
        crate::auth_trace::emit(crate::auth_trace::Event::ProtectedBindingValidated);
        Some((account, original))
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
    crate::auth_trace::emit(crate::auth_trace::Event::LoginSucceeded);
    if selected.is_none() {
        let matches: Vec<_> = subjects
            .iter()
            .filter(|(_, subject)| session.matches_saved_subject(subject.expose()))
            .map(|(id, _)| *id)
            .collect();
        if matches.len() > 1 {
            return Err(Error::new(
                "ACCOUNT_IDENTITY_CONFLICT",
                "Multiple local accounts require explicit identity reconciliation.",
            ));
        }
        if let Some(id) = matches.first() {
            let account = MeijuAccount::open(vault.lease(*id, operation).await?, operation).await?;
            let original = account.reauthorization_material(operation).await?;
            selected = Some((account, original));
        }
    }
    let mut account = match selected {
        Some((mut account, original)) => {
            if session.is_canonical() {
                identity_recovery::ensure_unique_subject(
                    vault,
                    account.record().id,
                    &session,
                    operation,
                )
                .await?;
            }
            account.reauthorize(&original, &session, operation).await?;
            account
        }
        None => {
            let encoded = session.encode()?;
            MeijuAccount::create(vault, "CN", &label, session.subject(), &encoded, operation)
                .await?
        }
    };
    crate::auth_trace::emit(crate::auth_trace::Event::SessionSaved);
    let retrieval = account.retrieve(transport, operation).await;
    crate::auth_trace::emit(if retrieval.is_ok() {
        crate::auth_trace::Event::InventorySucceeded
    } else {
        crate::auth_trace::Event::InventoryFailed
    });
    Ok(ConnectReport {
        account: account.record().clone(),
        retrieval,
    })
}
