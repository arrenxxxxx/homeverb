//! Native Meiju accounts and model-scoped reads, with reviewed experimental AC/dishwasher controls.

mod auth;
mod cloud;
mod codec;
mod control;
pub use control::{AC_FAN_SPEEDS, AC_MODES, AcMode};
mod dishwasher;
mod material;
mod product;
mod temperature;
pub use auth::{LoginCredentials, MeijuSession, SessionReads, login};
pub use cloud::{CloudEndpoint, CloudTransport, MeijuHttps, SignedRequest};
pub use dishwasher::{DISHWASHER_ADDITIONAL, DISHWASHER_PROGRAMS};
pub use product::ProductProfile;

mod account;
pub use account::{
    BoundInventory, ConnectReport, DeviceBinding, DeviceInventoryMembership, MeijuAccount, connect,
};
mod adapter;
pub use adapter::{AccountDiscovery, AccountInventory, MeijuAdapter};
mod state;
pub use state::{CloudState, StateQuery, read_cloud_state};

use async_trait::async_trait;
use homeverb_core::{CredentialRef, Operation, SecretBytes};
use homeverb_domain::{Error, Result};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, fmt};

pub const MAX_RESPONSE_BYTES: usize = 1_048_576;

/// A vendor identifier stays inside trusted native code, never in public DTOs or logs.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CloudId(String);

impl CloudId {
    pub fn expose(&self) -> &str {
        &self.0
    }

    fn parse(value: &Value) -> Result<Self> {
        let text = match value {
            Value::String(value) => value.clone(),
            Value::Number(value) if value.is_u64() => value.to_string(),
            _ => return Err(invalid()),
        };
        if text.is_empty()
            || text.len() > 64
            || !text.bytes().all(|byte| byte.is_ascii_digit())
            || text.bytes().all(|byte| byte == b'0')
        {
            return Err(invalid());
        }
        // The upstream client treats IDs as integers; canonicalize without float conversion.
        Ok(Self(text.trim_start_matches('0').to_owned()))
    }
}

impl fmt::Debug for CloudId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CloudId([REDACTED])")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplianceFamily {
    AirConditioner,
    Dishwasher,
    Refrigerator,
    Other(u16),
}

/// Account-scoped source inventory; not a Device descriptor or support claim.
pub struct Appliance {
    pub id: CloudId,
    pub family: ApplianceFamily,
    pub name: Option<String>,
    pub model: Option<String>,
    pub online_hint: Option<bool>,
    pub product_profile: ProductProfile,
}

impl fmt::Debug for Appliance {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Appliance([REDACTED])")
    }
}

#[derive(Debug)]
pub struct HomeFailure {
    pub home_id: CloudId,
    pub error: Error,
}

/// Inventory is private adapter data, not a serializable UI response.
/// ```compile_fail
/// fn export(inventory: &homeverb_midea::Inventory) {
///     let _ = serde_json::to_string(inventory);
/// }
/// ```
#[derive(Debug)]
pub struct Inventory {
    pub account_ref: CredentialRef,
    pub appliances: Vec<Appliance>,
    pub failures: Vec<HomeFailure>,
}

impl Inventory {
    pub fn is_complete(&self) -> bool {
        self.failures.is_empty()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ReadEndpoint<'a> {
    Homes,
    HomeAppliances(&'a CloudId),
    ApplianceState {
        appliance: &'a CloudId,
        query: StateQuery,
    },
}

impl ReadEndpoint<'_> {
    pub fn path(&self) -> &'static str {
        match self {
            Self::Homes => "/v1/homegroup/list/get",
            Self::HomeAppliances(_) => "/v1/appliance/home/list/get",
            Self::ApplianceState { .. } => "/mjl/v1/device/status/lua/get",
        }
    }
}

#[async_trait]
pub trait ReadTransport: Send + Sync {
    /// Return a bounded, successfully received HTTP response body, including code/data.
    /// Resolve this exact account securely; never reauthenticate or retry implicitly.
    async fn request(
        &self,
        account_ref: &CredentialRef,
        endpoint: ReadEndpoint<'_>,
        operation: &Operation,
    ) -> Result<SecretBytes>;
}

/// Enumerate existing bound devices without LAN discovery or device-control requests.
pub async fn retrieve_inventory(
    transport: &dyn ReadTransport,
    account_ref: CredentialRef,
    operation: &Operation,
) -> Result<Inventory> {
    let body = operation
        .run(transport.request(&account_ref, ReadEndpoint::Homes, operation))
        .await
        .map_err(sanitized)?;
    let homes = operation.run(async { parse_homes(&body) }).await?;
    let mut inventory = Inventory {
        account_ref,
        appliances: Vec::new(),
        failures: Vec::new(),
    };
    let mut positions = HashMap::<CloudId, usize>::new();
    for home_id in homes {
        let listing = operation
            .run(async {
                let body = transport
                    .request(
                        &account_ref,
                        ReadEndpoint::HomeAppliances(&home_id),
                        operation,
                    )
                    .await?;
                parse_appliances(&body, &home_id)
            })
            .await;
        match listing {
            Ok(appliances) => {
                for appliance in appliances {
                    if let Some(position) = positions.get(&appliance.id) {
                        let known = &mut inventory.appliances[*position];
                        if known.family != appliance.family
                            || matches!((&known.model, &appliance.model), (Some(a), Some(b)) if a != b)
                        {
                            return Err(Error::new(
                                "DEVICE_IDENTITY_CONFLICT",
                                "The account inventory contains conflicting appliance identity evidence.",
                            ));
                        }
                        known.product_profile.merge(&appliance.product_profile)?;
                        // Conflicting cloud hints are unknown, not a newer physical observation.
                        if known.online_hint != appliance.online_hint {
                            known.online_hint = None;
                        }
                        if known.model.is_none() {
                            known.model = appliance.model;
                        }
                    } else {
                        positions.insert(appliance.id.clone(), inventory.appliances.len());
                        inventory.appliances.push(appliance);
                    }
                }
            }
            Err(error) => {
                let error = sanitized(error);
                if matches!(error.code.as_str(), "CANCELLED" | "DEADLINE_EXCEEDED") {
                    return Err(error);
                }
                let authentication_required = error.code == "AUTH_REQUIRED";
                inventory.failures.push(HomeFailure { home_id, error });
                if authentication_required {
                    break;
                }
            }
        }
    }
    operation.run(async { Ok(inventory) }).await
}

fn invalid() -> Error {
    Error::new(
        "MIDEA_RESPONSE_INVALID",
        "The provider response does not match the supported read protocol.",
    )
}

fn sanitized(error: Error) -> Error {
    let message = match error.code.as_str() {
        "AUTH_REQUIRED" => "The platform account requires user-side authorization.",
        "CANCELLED" => "The cloud read was cancelled.",
        "DEADLINE_EXCEEDED" => "The cloud read exceeded its deadline.",
        "MIDEA_RESPONSE_INVALID" => return invalid(),
        "MIDEA_REQUEST_REJECTED" => "The provider rejected the read request.",
        _ => {
            return Error::new(
                "MIDEA_REQUEST_FAILED",
                "The cloud read could not be completed.",
            );
        }
    };
    Error::new(&error.code, message)
}

fn response_data(body: &SecretBytes) -> Result<Value> {
    if body.expose().len() > MAX_RESPONSE_BYTES {
        return Err(invalid());
    }
    let envelope: Value = serde_json::from_slice(body.expose()).map_err(|_| invalid())?;
    let code = match envelope.get("code") {
        Some(Value::Number(value)) => value.as_i64(),
        Some(Value::String(value)) => value.parse::<i64>().ok(),
        _ => None,
    }
    .ok_or_else(invalid)?;
    match code {
        0 => envelope
            .get("data")
            .filter(|value| value.is_object())
            .cloned()
            .ok_or_else(invalid),
        7400 | 40002 => Err(sanitized(Error::new("AUTH_REQUIRED", ""))),
        _ => Err(sanitized(Error::new("MIDEA_REQUEST_REJECTED", ""))),
    }
}

fn array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    value.get(key).and_then(Value::as_array).ok_or_else(invalid)
}

fn parse_homes(body: &SecretBytes) -> Result<Vec<CloudId>> {
    let data = response_data(body)?;
    let mut homes = Vec::new();
    for home in array(&data, "homeList")? {
        let id = CloudId::parse(home.get("homegroupId").ok_or_else(invalid)?)?;
        if !homes.contains(&id) {
            homes.push(id);
        }
    }
    Ok(homes)
}

#[derive(Deserialize)]
struct DeviceFields {
    #[serde(rename = "applianceCode")]
    id: Value,
    #[serde(rename = "type")]
    family: String,
    name: Option<String>,
    #[serde(rename = "productModel")]
    model: Option<String>,
    #[serde(rename = "onlineStatus")]
    online: Option<Value>,
}

fn parse_appliances(body: &SecretBytes, requested_home: &CloudId) -> Result<Vec<Appliance>> {
    let data = response_data(body)?;
    let mut appliances = Vec::new();
    for home in array(&data, "homeList")? {
        if let Some(id) = home.get("homegroupId")
            && CloudId::parse(id)? != *requested_home
        {
            return Err(invalid());
        }
        for room in array(home, "roomList")? {
            for entry in array(room, "applianceList")? {
                let fields: DeviceFields =
                    serde_json::from_value(entry.clone()).map_err(|_| invalid())?;
                let code = fields
                    .family
                    .strip_prefix("0x")
                    .or_else(|| fields.family.strip_prefix("0X"))
                    .unwrap_or(&fields.family);
                let family = match u16::from_str_radix(code, 16).map_err(|_| invalid())? {
                    0xac => ApplianceFamily::AirConditioner,
                    0xe1 => ApplianceFamily::Dishwasher,
                    0xca => ApplianceFamily::Refrigerator,
                    other => ApplianceFamily::Other(other),
                };
                let online_hint = match fields.online.as_ref().and_then(Value::as_str) {
                    Some("1") => Some(true),
                    Some("0") => Some(false),
                    _ => None,
                };
                appliances.push(Appliance {
                    id: CloudId::parse(&fields.id)?,
                    family,
                    name: fields.name.filter(|value| !value.is_empty()),
                    model: fields.model.filter(|value| !value.is_empty()),
                    online_hint,
                    product_profile: ProductProfile::parse(entry)?,
                });
            }
        }
    }
    Ok(appliances)
}
