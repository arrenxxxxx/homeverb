//! Read-only field mapping from the pinned community protocol; no capability admission.

use crate::{
    Appliance, ApplianceFamily, CloudId, ReadEndpoint, ReadTransport, invalid, response_data,
    sanitized,
};
use chrono::{DateTime, Utc};
use homeverb_core::{Clock, CredentialRef, Operation};
use homeverb_domain::{Error, Observation, Result, Snapshot, Transport};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateQuery {
    Default,
    IndoorTemperature,
}

impl StateQuery {
    /// The reviewed query object for a future signed transport; contains no private identity.
    pub fn payload(self) -> Value {
        match self {
            Self::Default => json!({}),
            Self::IndoorTemperature => json!({"query_type": "indoor_temperature"}),
        }
    }
}

/// Private source state with no serialization or raw Debug output.
/// ```compile_fail
/// fn export(state: &homeverb_midea::CloudState) {
///     let _ = serde_json::to_string(state);
/// }
/// ```
pub struct CloudState {
    pub account_ref: CredentialRef,
    pub appliance_id: CloudId,
    received_at: DateTime<Utc>,
    components: BTreeMap<&'static str, BTreeMap<String, Value>>,
}

impl fmt::Debug for CloudState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CloudState([REDACTED])")
    }
}

impl CloudState {
    /// Protocol component slots, not a claim that a model supplied every reading.
    pub fn component_ids(&self) -> impl Iterator<Item = &str> {
        self.components.keys().copied()
    }

    /// The adapter must first validate the local identity binding and component descriptor.
    /// Decoder component slots do not prove that a model supports those components.
    pub fn snapshot(&self, local_device_id: &str, component: &str) -> Result<Snapshot> {
        if local_device_id.trim().is_empty()
            || local_device_id.len() > 256
            || local_device_id.chars().any(char::is_control)
        {
            return Err(Error::invalid());
        }
        let readings = self.components.get(component).ok_or_else(unsupported)?;
        Ok(Snapshot {
            schema_version: "1.0".into(),
            device_id: local_device_id.into(),
            component_id: component.into(),
            from_cache: false,
            observations: readings
                .iter()
                .map(|(key, value)| {
                    (
                        key.clone(),
                        Observation {
                            value: value.clone(),
                            source: Transport::Cloud,
                            observed_at: None,
                            received_at: self.received_at,
                            stale: true,
                        },
                    )
                })
                .collect(),
            warnings: vec!["Cloud observation time is unavailable; freshness is unproven.".into()],
        })
    }
}

pub async fn read_cloud_state(
    transport: &dyn ReadTransport,
    account_ref: CredentialRef,
    appliance: &Appliance,
    query: StateQuery,
    clock: &dyn Clock,
    operation: &Operation,
) -> Result<CloudState> {
    operation
        .run(async {
            if matches!(appliance.family, ApplianceFamily::Other(_))
                || (query == StateQuery::IndoorTemperature
                    && appliance.family != ApplianceFamily::AirConditioner)
            {
                return Err(unsupported());
            }
            Ok(())
        })
        .await?;
    let body = operation
        .run(transport.request(
            &account_ref,
            ReadEndpoint::ApplianceState {
                appliance: &appliance.id,
                query,
            },
            operation,
        ))
        .await
        .map_err(sanitized)?;
    let received_at = clock.now();
    let state = operation
        .run(async {
            let data = response_data(&body)?;
            if let Some(id) = data.get("applianceCode")
                && CloudId::parse(id)? != appliance.id
            {
                return Err(invalid());
            }
            if let Some(unit) = present(&data, "temperature_unit")
                && unit.as_str() != Some("celsius")
            {
                return Err(unsupported());
            }
            Ok(CloudState {
                account_ref,
                appliance_id: appliance.id.clone(),
                received_at,
                components: decode(&data, appliance)?,
            })
        })
        .await?;
    operation.run(async { Ok(state) }).await
}

pub(crate) fn unsupported() -> Error {
    Error::new(
        "CAPABILITY_UNSUPPORTED",
        "The cloud state family, query, component, or temperature unit is unsupported.",
    )
}

fn present<'a>(data: &'a Value, field: &str) -> Option<&'a Value> {
    data.get(field).filter(|value| !value.is_null())
}

fn number(value: &Value) -> Result<f64> {
    let number = match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) if text.len() <= 64 => text.parse::<f64>().ok(),
        _ => None,
    }
    .filter(|number| number.is_finite())
    .ok_or_else(invalid)?;
    Ok(number)
}

fn numeric(value: &Value) -> Result<Value> {
    Ok(json!(number(value)?))
}

fn on_off(value: &Value) -> Result<Value> {
    match value.as_str() {
        Some("on") => Ok(json!(true)),
        Some("off") => Ok(json!(false)),
        _ => Err(invalid()),
    }
}

fn vendor_code(value: &Value) -> Result<Value> {
    let code = number(value)?;
    if code < 0.0 || code.fract() != 0.0 || code > u32::MAX as f64 {
        return Err(invalid());
    }
    Ok(json!(code as u32))
}

fn vendor_token(value: &Value) -> Result<Value> {
    let token = value.as_str().ok_or_else(invalid)?;
    if token.is_empty()
        || token.len() > 64
        || !token
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err(invalid());
    }
    Ok(json!(token))
}

fn field(
    output: &mut BTreeMap<String, Value>,
    data: &Value,
    wire: &str,
    property: &str,
    convert: fn(&Value) -> Result<Value>,
) -> Result<()> {
    if let Some(value) = present(data, wire) {
        output.insert(property.into(), convert(value)?);
    }
    Ok(())
}

fn decode(
    data: &Value,
    appliance: &Appliance,
) -> Result<BTreeMap<&'static str, BTreeMap<String, Value>>> {
    let mut output = BTreeMap::new();
    match appliance.family {
        ApplianceFamily::AirConditioner => {
            let readings = output.entry("main").or_default();
            field(readings, data, "power", "power.state", on_off)?;
            field(readings, data, "mode", "vendor.midea.ac.mode", vendor_token)?;
            field(
                readings,
                data,
                "wind_speed",
                "vendor.midea.ac.wind_speed",
                vendor_code,
            )?;
            field(
                readings,
                data,
                "indoor_temperature",
                "climate.current_temperature",
                numeric,
            )?;
            if appliance
                .product_profile
                .temperature()
                .is_some_and(|profile| !profile.fractional_field)
            {
                // Read the selected wire shape without applying writable bounds or steps.
                field(
                    readings,
                    data,
                    "temperature",
                    "climate.target_temperature",
                    numeric,
                )?;
            } else {
                let whole = present(data, "temperature").map(number).transpose()?;
                let fraction = present(data, "small_temperature").map(number).transpose()?;
                if whole.is_some_and(|value| value.fract() != 0.0)
                    || fraction.is_some_and(|value| !(0.0..1.0).contains(&value))
                {
                    return Err(invalid());
                }
                if let (Some(whole), Some(fraction)) = (whole, fraction) {
                    let target = whole + fraction;
                    if !target.is_finite() {
                        return Err(invalid());
                    }
                    readings.insert("climate.target_temperature".into(), json!(target));
                }
            }
        }
        ApplianceFamily::Dishwasher => {
            let readings = output.entry("main").or_default();
            for wire in ["mode", "work_status", "operator"] {
                field(
                    readings,
                    data,
                    wire,
                    &format!("vendor.midea.dishwasher.{wire}"),
                    vendor_token,
                )?;
            }
            for wire in ["doorswitch", "water_lack", "softwater_lack", "bright_lack"] {
                field(
                    readings,
                    data,
                    wire,
                    &format!("vendor.midea.dishwasher.{wire}"),
                    vendor_code,
                )?;
            }
            field(
                readings,
                data,
                "temperature",
                "vendor.midea.dishwasher.temperature",
                numeric,
            )?;
            field(
                readings,
                data,
                "left_time",
                "vendor.midea.dishwasher.remaining_minutes",
                vendor_code,
            )?;
        }
        ApplianceFamily::Refrigerator => {
            for (component, power, target, current) in [
                (
                    "fridge",
                    "storage_power",
                    "storage_temperature",
                    "refrigeration_real_temperature",
                ),
                (
                    "freezer",
                    "freezing_power",
                    "freezing_temperature",
                    "freezing_real_temperature",
                ),
            ] {
                let readings = output.entry(component).or_default();
                field(readings, data, power, "power.state", on_off)?;
                field(
                    readings,
                    data,
                    target,
                    "climate.target_temperature",
                    numeric,
                )?;
                field(
                    readings,
                    data,
                    current,
                    "climate.current_temperature",
                    numeric,
                )?;
            }
        }
        ApplianceFamily::Other(_) => return Err(unsupported()),
    }
    Ok(output)
}
