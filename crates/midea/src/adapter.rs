//! Compile-time, model-scoped cloud reads. No appliance-control endpoint is registered.

use crate::{Appliance, ApplianceFamily, CloudTransport, DeviceBinding, MeijuAccount};
use async_trait::async_trait;
use homeverb_core::{
    AccountVault, Adapter, AdapterOutcome, Clock, Operation, ValidatedCommand, WriteProgress,
};
use homeverb_domain::{
    Capability, Component, Connection, Descriptor, Device, Error, Result, Snapshot, Transport,
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

fn descriptors(appliance: &Appliance, binding: DeviceBinding, account: Uuid) -> Descriptor {
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
            capabilities.push(capability("main", "power", &[("state", "boolean", None)]));
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

/// Foreground on-demand adapter. Construction and local Core queries perform no account work.
pub struct MeijuAdapter {
    vault: Arc<AccountVault>,
    transport: Arc<dyn CloudTransport>,
    clock: Arc<dyn Clock>,
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
        }
    }
}

#[async_trait]
impl Adapter for MeijuAdapter {
    fn id(&self) -> &str {
        PROVIDER
    }

    async fn discover(&self, operation: &Operation) -> Result<Vec<Descriptor>> {
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
        for account in &mut accounts {
            let bound = account.retrieve(self.transport.as_ref(), operation).await?;
            if !bound.inventory.is_complete() {
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
            for (appliance, binding) in bound.inventory.appliances.iter().zip(bound.bindings) {
                output.push(descriptors(appliance, binding, account.record().id));
            }
        }
        operation.run(async { Ok(output) }).await
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
        _: &ValidatedCommand,
        _: &Connection,
        _: WriteProgress,
        _: &Operation,
    ) -> Result<AdapterOutcome> {
        Err(Error::new(
            "CAPABILITY_UNSUPPORTED",
            "This adapter exposes read-only model mappings; appliance controls are unavailable.",
        ))
    }
}
