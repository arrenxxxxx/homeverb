//! Compile-time, model-scoped cloud mappings. Controls require explicit host admission.

use crate::{
    Appliance, ApplianceFamily, BoundInventory, CloudTransport, DeviceBinding, LoginCredentials,
    MeijuAccount,
};
use async_trait::async_trait;
use homeverb_core::{
    AccountRecord, AccountVault, Adapter, AdapterOutcome, Clock, Operation, ValidatedCommand,
    WriteProgress,
};
use homeverb_domain::{
    Action, Capability, Component, Connection, Descriptor, Device, Error, Idempotency, Result,
    Risk, Snapshot, Transport, VerificationMethod,
};
use serde_json::json;
use std::{
    collections::{BTreeMap, HashSet},
    sync::Arc,
};
use uuid::Uuid;

const PROVIDER: &str = "midea.meiju";

// Exact model strings from the read-only cloud probe; not physical support certification.
const READ_MODELS: &[(&str, ApplianceFamily)] = &[
    ("KFR-35G/N8HE1\u{2161}Pro", ApplianceFamily::AirConditioner),
    ("KFR-35G/WF2", ApplianceFamily::AirConditioner),
    ("KFR-72G/N8HA1Pro", ApplianceFamily::AirConditioner),
    ("KFR-26G/N8VJC3", ApplianceFamily::AirConditioner),
    ("BCD-600WUFIPZM(E)", ApplianceFamily::Refrigerator),
    (
        "GX1200Max \u{5c0a}\u{4eab}\u{7248}",
        ApplianceFamily::Dishwasher,
    ),
];

fn model_family(model: &str) -> Option<ApplianceFamily> {
    READ_MODELS
        .iter()
        .find(|(name, _)| *name == model)
        .map(|(_, family)| *family)
}

fn capability(component: &str, id: &str, fields: &[(&str, &str, Option<&str>)]) -> Capability {
    Capability {
        schema_version: "1.0".into(),
        id: id.into(),
        component_id: Some(component.into()),
        revision: 1,
        properties: fields
            .iter()
            .map(|(name, kind, unit)| {
                let mut value = json!({"type":kind,"readable":true,"writable":false});
                if let Some(unit) = unit {
                    value["unit"] = json!(unit);
                }
                (name.to_string(), value)
            })
            .collect(),
        actions: BTreeMap::new(),
        events: BTreeMap::new(),
    }
}

fn descriptors(
    appliance: &Appliance,
    binding: DeviceBinding,
    account: Uuid,
    ac_power: bool,
) -> Descriptor {
    let family = appliance
        .model
        .as_deref()
        .and_then(model_family)
        .filter(|family| *family == appliance.family);
    let mut capabilities = Vec::new();
    let temperatures = [
        ("current_temperature", "number", Some("celsius")),
        ("target_temperature", "number", Some("celsius")),
    ];
    match family {
        Some(ApplianceFamily::AirConditioner) => {
            let mut power = capability("main", "power", &[("state", "boolean", None)]);
            if ac_power {
                power.revision = 2;
                power.properties.get_mut("state").unwrap()["writable"] = json!(true);
                power.actions.insert("set".into(), power_action());
            }
            capabilities.push(power);
            capabilities.push(capability("main", "climate", &temperatures));
            capabilities.push(capability(
                "main",
                "vendor.midea.ac",
                &[("mode", "string", None), ("wind_speed", "integer", None)],
            ));
        }
        Some(ApplianceFamily::Refrigerator) => {
            for component in ["fridge", "freezer"] {
                capabilities.push(capability(
                    component,
                    "power",
                    &[("state", "boolean", None)],
                ));
                capabilities.push(capability(component, "climate", &temperatures));
            }
        }
        Some(ApplianceFamily::Dishwasher) => capabilities.push(capability(
            "main",
            "vendor.midea.dishwasher",
            &[
                ("mode", "string", None),
                ("work_status", "string", None),
                ("doorswitch", "integer", None),
                ("water_lack", "integer", None),
                ("softwater_lack", "integer", None),
                ("bright_lack", "integer", None),
                ("temperature", "number", Some("celsius")),
                ("remaining_minutes", "integer", Some("minutes")),
            ],
        )),
        _ => capabilities.push(capability("main", "vendor.midea.inventory", &[])),
    }
    let mut components = BTreeMap::<String, Vec<String>>::new();
    for cap in &capabilities {
        components
            .entry(cap.component_id.clone().unwrap())
            .or_default()
            .push(cap.id.clone());
    }
    let fallback = match appliance.family {
        ApplianceFamily::AirConditioner => "Air conditioner",
        ApplianceFamily::Dishwasher => "Dishwasher",
        ApplianceFamily::Refrigerator => "Refrigerator",
        _ => "Unsupported appliance",
    };
    Descriptor {
        device: Device {
            schema_version: "1.0".into(),
            id: binding.device_id.to_string(),
            alias: appliance
                .name
                .as_deref()
                .and_then(|name| homeverb_domain::local_name(name).ok())
                .unwrap_or_else(|| fallback.into()),
            brand: "Unknown".into(),
            model: appliance
                .model
                .clone()
                .unwrap_or_else(|| "Unknown model".into()),
            support_status: if family.is_some() {
                "experimental"
            } else {
                "unsupported"
            }
            .into(),
            components: components
                .into_iter()
                .map(|(id, capabilities)| Component { id, capabilities })
                .collect(),
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

/// Native account result before Core publication; never serialize the private account record.
pub struct AccountDiscovery {
    pub account: AccountRecord,
    pub descriptors: Result<Vec<Descriptor>>,
}

/// Foreground on-demand adapter. Construction and local Core queries perform no account work.
pub struct MeijuAdapter {
    vault: Arc<AccountVault>,
    transport: Arc<dyn CloudTransport>,
    clock: Arc<dyn Clock>,
    ac_power: bool,
}

impl MeijuAdapter {
    pub fn new(
        vault: Arc<AccountVault>,
        transport: Arc<dyn CloudTransport>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            vault,
            transport,
            clock,
            ac_power: false,
        }
    }

    /// Trusted host opt-in to experimental descriptors; this grants no write authority.
    /// Core still requires an exact device/component/argument/cloud grant.
    pub fn with_ac_power_control(mut self) -> Self {
        self.ac_power = true;
        self
    }
    /// Local metadata only; a stored credential is not proof of current authorization.
    pub async fn accounts(&self, operation: &Operation) -> Result<Vec<AccountRecord>> {
        Ok(self
            .vault
            .list(operation)
            .await?
            .into_iter()
            .filter(|record| record.provider == PROVIDER && record.region == "CN")
            .collect())
    }

    pub async fn account(&self, id: Uuid, operation: &Operation) -> Result<AccountRecord> {
        homeverb_core::valid_account_id(id)?;
        self.accounts(operation)
            .await?
            .into_iter()
            .find(|record| record.id == id)
            .ok_or_else(|| {
                Error::new(
                    "ACCOUNT_NOT_FOUND",
                    "The selected Meiju account does not exist.",
                )
            })
    }

    /// Change only local presentation metadata, even when secure storage is unavailable.
    pub async fn rename_account(
        &self,
        id: Uuid,
        label: &str,
        operation: &Operation,
    ) -> Result<AccountRecord> {
        let mut lease = self.vault.lease(id, operation).await?;
        if lease.record().provider != PROVIDER || lease.record().region != "CN" {
            return Err(Error::new(
                "ACCOUNT_NOT_FOUND",
                "The selected Meiju account does not exist.",
            ));
        }
        lease.rename(label, operation).await?;
        Ok(lease.record().clone())
    }

    pub async fn remove_account(
        &self,
        id: Uuid,
        expected_device_ids: &[String],
        operation: &Operation,
    ) -> Result<homeverb_core::AccountRemovalResult> {
        self.vault
            .remove_account(id, PROVIDER, "CN", expected_device_ids, operation)
            .await
    }

    pub async fn connect_devices(
        &self,
        credentials: LoginCredentials,
        existing: Option<Uuid>,
        label: &str,
        operation: &Operation,
    ) -> Result<AccountDiscovery> {
        let report = crate::connect(
            &self.vault,
            self.transport.as_ref(),
            credentials,
            existing,
            label,
            operation,
        )
        .await?;
        let descriptors = match report.retrieval {
            Ok(bound) => {
                self.discover_inner(Some(report.account.id), Some(bound), operation)
                    .await
            }
            Err(error) => Err(error),
        };
        Ok(AccountDiscovery {
            account: report.account,
            descriptors,
        })
    }

    pub async fn refresh_devices(
        &self,
        id: Uuid,
        operation: &Operation,
    ) -> Result<AccountDiscovery> {
        self.accounts(operation)
            .await?
            .into_iter()
            .find(|record| record.id == id)
            .ok_or_else(|| {
                Error::new(
                    "ACCOUNT_NOT_FOUND",
                    "The selected Meiju account does not exist.",
                )
            })?;
        let (account, retrieval) = {
            let _registration = self.vault.lock_registration(operation).await?;
            let lease = self.vault.lease(id, operation).await?;
            let record = lease.record().clone();
            match MeijuAccount::open(lease, operation).await {
                Ok(mut account) => {
                    let retrieval = account.retrieve(self.transport.as_ref(), operation).await;
                    (account.record().clone(), retrieval)
                }
                Err(error) => (record, Err(error)),
            }
        };
        let descriptors = match retrieval {
            Ok(bound) => self.discover_inner(Some(id), Some(bound), operation).await,
            Err(error) => Err(error),
        };
        Ok(AccountDiscovery {
            account,
            descriptors,
        })
    }

    async fn discover_inner(
        &self,
        target: Option<Uuid>,
        mut prefetched: Option<BoundInventory>,
        operation: &Operation,
    ) -> Result<Vec<Descriptor>> {
        let _registration = self.vault.lock_registration(operation).await?;
        let mut output = Vec::new();
        let mut physical_ids = HashSet::new();
        let mut accounts = Vec::new();
        for record in self.vault.list(operation).await? {
            if record.provider != PROVIDER || record.region != "CN" {
                continue;
            }
            let account =
                MeijuAccount::open(self.vault.lease(record.id, operation).await?, operation)
                    .await?;
            accounts.push(account);
        }
        // Hold every account lease through the identity check so concurrent retrieval cannot
        // change the protected map between accounts. No descriptors escape on any failure.
        if let Some(id) = target
            && !accounts.iter().any(|account| account.record().id == id)
        {
            return Err(Error::new(
                "ACCOUNT_NOT_FOUND",
                "The selected Meiju account does not exist.",
            ));
        }
        for account in &mut accounts {
            let selected = target.is_none_or(|id| account.record().id == id);
            let bound = if selected {
                Some(match prefetched.take() {
                    Some(bound) => bound,
                    None => account.retrieve(self.transport.as_ref(), operation).await?,
                })
            } else {
                None
            };
            if bound
                .as_ref()
                .is_some_and(|bound| !bound.inventory.is_complete())
            {
                if bound.as_ref().is_some_and(|bound| {
                    bound
                        .inventory
                        .failures
                        .iter()
                        .any(|failure| failure.error.code == "AUTH_REQUIRED")
                }) {
                    return Err(Error::new(
                        "AUTH_REQUIRED",
                        "The platform account requires user-side authorization.",
                    ));
                }
                return Err(Error::new(
                    "MIDEA_INVENTORY_PARTIAL",
                    "Cloud retrieval was incomplete; existing device cards were preserved.",
                ));
            }
            for id in account.bound_appliances(operation).await? {
                if !physical_ids.insert(id) {
                    return Err(Error::new(
                        "DEVICE_IDENTITY_CONFLICT",
                        "An appliance is visible through multiple accounts and requires explicit reconciliation.",
                    ));
                }
            }
            if let Some(bound) = bound {
                for (appliance, binding) in bound.inventory.appliances.iter().zip(bound.bindings) {
                    output.push(descriptors(
                        appliance,
                        binding,
                        account.record().id,
                        self.ac_power,
                    ));
                }
            }
        }
        operation.run(async { Ok(output) }).await
    }
}

#[async_trait]
impl Adapter for MeijuAdapter {
    fn id(&self) -> &str {
        PROVIDER
    }

    async fn discover(&self, operation: &Operation) -> Result<Vec<Descriptor>> {
        self.discover_inner(None, None, operation).await
    }

    async fn read(
        &self,
        device: &Device,
        component: &str,
        connection: &Connection,
        operation: &Operation,
    ) -> Result<Snapshot> {
        let family = model_family(&device.model)
            .filter(|_| device.support_status == "experimental")
            .ok_or_else(|| {
                Error::new(
                    "CAPABILITY_UNSUPPORTED",
                    "This exact model has no admitted cloud read mapping.",
                )
            })?;
        let id = connection
            .account_id
            .as_deref()
            .and_then(|id| Uuid::parse_str(id).ok())
            .ok_or_else(|| {
                Error::new(
                    "DEVICE_IDENTITY_CONFLICT",
                    "The cloud connection has no valid local account binding.",
                )
            })?;
        let account = MeijuAccount::open(self.vault.lease(id, operation).await?, operation).await?;
        account
            .validate_read_model(device, family, operation)
            .await?;
        let session = account.cloud_session(operation).await?;
        let reads = session.reads(self.transport.as_ref(), account.record().credential_ref);
        account
            .read_bound_state(
                &reads,
                device,
                component,
                connection,
                self.clock.as_ref(),
                operation,
            )
            .await
    }

    async fn execute(
        &self,
        validated: &ValidatedCommand,
        connection: &Connection,
        progress: WriteProgress,
        operation: &Operation,
    ) -> Result<AdapterOutcome> {
        let command = validated.command();
        if !self.ac_power
            || command.component_id != "main"
            || command.capability != "power"
            || command.action != "set"
        {
            return Err(crate::control::unsupported());
        }
        // Repeat the fixed adapter contract even if public stored metadata was modified.
        power_action().validate_arguments(&command.arguments)?;
        let id = connection
            .account_id
            .as_deref()
            .and_then(|id| Uuid::parse_str(id).ok())
            .ok_or_else(|| {
                Error::new(
                    "DEVICE_IDENTITY_CONFLICT",
                    "The cloud connection has no valid local account binding.",
                )
            })?;
        let account = MeijuAccount::open(self.vault.lease(id, operation).await?, operation).await?;
        let appliance = account
            .control_appliance(&command.device_id, connection, operation)
            .await?;
        if appliance.model.as_deref().and_then(model_family)
            != Some(ApplianceFamily::AirConditioner)
        {
            return Err(crate::control::unsupported());
        }
        let session = account.cloud_session(operation).await?;
        crate::control::set_power(
            &session,
            self.transport.as_ref(),
            &appliance.id,
            command.arguments["state"]
                .as_bool()
                .expect("Validated boolean state."),
            &progress,
            operation,
        )
        .await
    }
}

fn power_action() -> Action {
    Action {
        parameters: json!({"type":"object","properties":{"state":{"type":"boolean"}},"required":["state"],"additionalProperties":false}),
        transports: vec![Transport::Cloud],
        idempotency: Idempotency::SetValue,
        risk: Risk::Low,
        atomic: false,
        independent_fields: false,
        // Cloud state has no trustworthy observation time or completion correlation.
        verification: VerificationMethod::Unavailable,
    }
}
