//! Bounded numeric property logs, distinct from daily aggregates and live state.
use crate::{
    CloudDevice, CloudEndpoint, CloudTransport, MappedProperty, MijiaSession, ModelMapping,
    PropertyQuery, Region, invalid,
};
use chrono::{DateTime, Utc};
use homeverb_core::{Operation, SecretBytes};
use homeverb_domain::{Error, Result};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashSet};

pub(crate) const RECORD_LIMIT: usize = 31;

/// Positive half-open UTC range, bounded to 32 days; no retention claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropertyHistoryWindow {
    start: i64,
    end: i64,
}
impl PropertyHistoryWindow {
    pub fn new(start: i64, end: i64) -> Result<Self> {
        crate::DailyEnergyWindow::new(start, end)?;
        Ok(Self { start, end })
    }
    pub fn start(&self) -> i64 {
        self.start
    }
    pub fn end(&self) -> i64 {
        self.end
    }
}

/// A reported property-log value, not a verified physical observation.
#[derive(Debug, Clone, PartialEq)]
pub struct NumericPropertyRecord {
    pub reported_at: DateTime<Utc>,
    pub value: f64,
}
#[derive(Debug, Clone, PartialEq)]
pub struct NumericPropertyHistory {
    pub window: PropertyHistoryWindow,
    pub received_at: DateTime<Utc>,
    pub unit: Option<String>,
    pub records: Vec<NumericPropertyRecord>,
    pub rejected_records: usize,
    pub limit_reached: bool,
    pub completeness: crate::StatisticsCompleteness,
}

fn unsupported() -> Error {
    Error::new(
        "CAPABILITY_UNSUPPORTED",
        "This numeric property history has no reviewed mapping.",
    )
}
pub(crate) fn field(
    model: &str,
    specification: &str,
    query: &PropertyQuery,
) -> Result<MappedProperty> {
    let mapping = ModelMapping::pinned(model, Some(specification))?.ok_or_else(unsupported)?;
    let mut selected = mapping
        .properties
        .iter()
        .find(|p| {
            p.siid == query.siid
                && p.piid == query.piid
                && p.definition["readable"] == true
                && matches!(p.definition["type"].as_str(), Some("number" | "integer"))
        })
        .cloned()
        .ok_or_else(unsupported)?;
    selected.definition = mapping.numeric_history_definition(query.siid, query.piid)?;
    Ok(selected)
}
pub(crate) fn payload(
    did: &str,
    field: &MappedProperty,
    window: PropertyHistoryWindow,
) -> Result<SecretBytes> {
    if !crate::bounded_text(did, 128) || window.end > Utc::now().timestamp() {
        return Err(Error::invalid());
    }
    Ok(SecretBytes::new(
        serde_json::to_vec(&json!({
            "did":did, "key":format!("{}.{}",field.siid,field.piid), "type":"prop",
            "time_start":window.start, "time_end":window.end, "limit":RECORD_LIMIT,
        }))
        .map_err(|_| invalid())?,
    ))
}
impl MijiaSession {
    /// One exact pinned numeric field, with no raw-key selection or automatic paging.
    /// The adapter must independently validate protected account/device membership.
    pub async fn numeric_property_history(
        &self,
        transport: &dyn CloudTransport,
        device: &CloudDevice,
        query: &PropertyQuery,
        window: PropertyHistoryWindow,
        operation: &Operation,
    ) -> Result<NumericPropertyHistory> {
        operation
            .run(async {
                if self.region() != Region::Cn {
                    return Err(unsupported());
                }
                let field = field(
                    &device.model,
                    device.spec_type.as_deref().ok_or_else(unsupported)?,
                    query,
                )?;
                let data = payload(device.did.as_str(), &field, window)?;
                let bytes = self
                    .request(transport, CloudEndpoint::PropertyHistory, &data, operation)
                    .await?;
                let result = decode_history(
                    bytes.expose(),
                    device.did.as_str(),
                    &field,
                    window,
                    Utc::now(),
                )?;
                operation.run(async { Ok(result) }).await
            })
            .await
    }
}

#[derive(Deserialize)]
struct Envelope {
    code: i64,
    result: Vec<Value>,
}
fn timestamp(value: Option<&Value>) -> Option<i64> {
    let value = value?;
    if let Some(n) = value.as_i64() {
        return Some(n);
    }
    let text = value.as_str()?;
    if text.is_empty() || text.len() > 15 || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}
fn number(value: Option<&Value>, validator: &jsonschema::Validator) -> Option<f64> {
    let text = value?.as_str()?;
    if text.len() > 128 {
        return None;
    }
    let values: Vec<Value> = serde_json::from_str(text).ok()?;
    if values.len() != 1 || !values[0].is_number() || !validator.is_valid(&values[0]) {
        return None;
    }
    let number = values[0].as_f64()?;
    if !number.is_finite() {
        return None;
    }
    let coefficient = text
        .trim()
        .strip_prefix('[')?
        .strip_suffix(']')?
        .trim()
        .split(['e', 'E'])
        .next()?;
    if number == 0.0 && coefficient.bytes().any(|b| (b'1'..=b'9').contains(&b)) {
        return None;
    }
    Some(if number == 0.0 { 0.0 } else { number })
}
pub(crate) fn decode_history(
    bytes: &[u8],
    did: &str,
    field: &MappedProperty,
    window: PropertyHistoryWindow,
    received_at: DateTime<Utc>,
) -> Result<NumericPropertyHistory> {
    if bytes.len() > crate::MAX_RESPONSE_BYTES {
        return Err(invalid());
    }
    let envelope: Envelope = crate::decode(bytes)?;
    if envelope.code != 0 || envelope.result.len() > RECORD_LIMIT {
        return Err(invalid());
    }
    let limit_reached = envelope.result.len() == RECORD_LIMIT;
    let key = format!("{}.{}", field.siid, field.piid);
    let validator = jsonschema::validator_for(&field.definition).map_err(|_| invalid())?;
    let mut seen = HashSet::new();
    let mut records = BTreeMap::new();
    let mut rejected_records = 0;
    for value in envelope.result {
        let Some(record) = value.as_object() else {
            rejected_records += 1;
            continue;
        };
        if record.get("did").is_some_and(|v| v.as_str() != Some(did))
            || record
                .get("key")
                .is_some_and(|v| v.as_str() != Some(key.as_str()))
            || record
                .get("type")
                .is_some_and(|v| v.as_str() != Some("prop"))
        {
            return Err(invalid());
        }
        let Some(time) =
            timestamp(record.get("time")).filter(|t| *t >= window.start && *t < window.end)
        else {
            rejected_records += 1;
            continue;
        };
        if !seen.insert(time) {
            rejected_records += 1;
            if records.remove(&time).is_some() {
                rejected_records += 1;
            }
            continue;
        }
        let Some(value) = number(record.get("value"), &validator) else {
            rejected_records += 1;
            continue;
        };
        records.insert(
            time,
            NumericPropertyRecord {
                reported_at: DateTime::from_timestamp(time, 0).ok_or_else(invalid)?,
                value,
            },
        );
    }
    Ok(NumericPropertyHistory {
        window,
        received_at,
        unit: field.definition["unit"].as_str().map(str::to_owned),
        records: records.into_values().collect(),
        rejected_records,
        limit_reached,
        completeness: crate::StatisticsCompleteness::Unproven,
    })
}

#[cfg(test)]
mod tests;
