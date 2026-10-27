//! Exact, bounded historical energy records; never current-state or calendar completeness.

use crate::{CloudDevice, CloudEndpoint, CloudTransport, MijiaSession, Region, decode, invalid};
use chrono::{DateTime, Utc};
use homeverb_core::{Operation, SecretBytes};
use homeverb_domain::{Error, Result};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashSet};

pub const DAILY_ENERGY_MODEL: &str = "cuco.plug.v3";
pub const DAILY_ENERGY_SPEC: &str = "urn:miot-spec-v2:device:outlet:0000A002:cuco-v3:2";
const RECORD_LIMIT: usize = 31;
const MAX_WINDOW_SECONDS: i64 = 32 * 86400;

/// Half-open absolute UTC range. It does not define the provider's daily timezone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DailyEnergyWindow {
    start: i64,
    end: i64,
}
impl DailyEnergyWindow {
    pub fn new(start: i64, end: i64) -> Result<Self> {
        if start <= 0
            || end <= start
            || end
                .checked_sub(start)
                .is_none_or(|span| span > MAX_WINDOW_SECONDS)
            || DateTime::from_timestamp(start, 0).is_none()
            || DateTime::from_timestamp(end, 0).is_none()
        {
            return Err(Error::invalid());
        }
        Ok(Self { start, end })
    }
    pub fn start(&self) -> i64 {
        self.start
    }
    pub fn end(&self) -> i64 {
        self.end
    }
}

/// A provider daily statistic with its original timestamp, not a verified day boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct DailyEnergyRecord {
    pub reported_at: DateTime<Utc>,
    pub kilowatt_hours: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatisticsCompleteness {
    /// Neither retention nor paging nor provider calendar completeness was established.
    Unproven,
}

/// Sanitized response. Missing/rejected dates are not zero and must not be interpolated.
#[derive(Debug, Clone, PartialEq)]
pub struct DailyEnergyHistory {
    pub window: DailyEnergyWindow,
    pub received_at: DateTime<Utc>,
    pub records: Vec<DailyEnergyRecord>,
    pub rejected_records: usize,
    pub limit_reached: bool,
    pub completeness: StatisticsCompleteness,
}

#[derive(Deserialize)]
struct Envelope {
    code: i64,
    result: Vec<Value>,
}

impl MijiaSession {
    /// Read this exact mainland community statistics profile once. No generic key/endpoint
    /// selection, polling, relogin or retry is available. Core must independently admit
    /// local identity, protected account/model bindings and historical-read permissions.
    pub async fn daily_energy_history(
        &self,
        transport: &dyn CloudTransport,
        device: &CloudDevice,
        window: DailyEnergyWindow,
        operation: &Operation,
    ) -> Result<DailyEnergyHistory> {
        operation
            .run(async {
                if self.region() != Region::Cn
                    || device.model != DAILY_ENERGY_MODEL
                    || device.spec_type.as_deref() != Some(DAILY_ENERGY_SPEC)
                {
                    return Err(Error::new(
                        "CAPABILITY_UNSUPPORTED",
                        "This exact historical energy profile is unavailable.",
                    ));
                }
                if !crate::bounded_text(device.did.as_str(), 128)
                    || window.end > Utc::now().timestamp()
                {
                    return Err(Error::invalid());
                }
                let data = SecretBytes::new(
                    serde_json::to_vec(&json!({
                        "did": device.did.as_str(), "key": "11.1", "data_type": "stat_day_v3",
                        "time_start": window.start, "time_end": window.end, "limit": RECORD_LIMIT,
                    }))
                    .map_err(|_| Error::invalid())?,
                );
                let response = self
                    .request(transport, CloudEndpoint::Statistics, &data, operation)
                    .await?;
                let result =
                    decode_history(response.expose(), device.did.as_str(), window, Utc::now())?;
                // Parsing must not publish a late result after lifecycle/deadline invalidation.
                operation.run(async { Ok(result) }).await
            })
            .await
    }
}

fn timestamp(value: Option<&Value>) -> Option<i64> {
    let value = value?;
    if let Some(number) = value.as_i64() {
        return Some(number);
    }
    let text = value.as_str()?;
    if text.is_empty() || text.len() > 15 || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

fn energy(value: Option<&Value>) -> Option<f64> {
    let encoded = value?.as_str()?;
    if encoded.len() > 128 {
        return None;
    }
    // Only one numeric channel is evidenced. Do not interpret additional payload fields.
    let values: Vec<f64> = serde_json::from_str(encoded).ok()?;
    if values.len() != 1 || !values[0].is_finite() || values[0] < 0.0 {
        return None;
    }
    // A positive or negative JSON coefficient must not underflow into an invented zero.
    let scalar = encoded.trim().strip_prefix('[')?.strip_suffix(']')?.trim();
    let coefficient = scalar.split(['e', 'E']).next()?;
    if values[0] == 0.0 && coefficient.bytes().any(|b| (b'1'..=b'9').contains(&b)) {
        return None;
    }
    let converted = values[0] * 0.01;
    if !converted.is_finite() || values[0] > 0.0 && converted == 0.0 {
        return None;
    }
    Some(if converted == 0.0 { 0.0 } else { converted })
}

fn decode_history(
    bytes: &[u8],
    did: &str,
    window: DailyEnergyWindow,
    received_at: DateTime<Utc>,
) -> Result<DailyEnergyHistory> {
    let envelope: Envelope = decode(bytes)?;
    if envelope.code != 0 || envelope.result.len() > RECORD_LIMIT {
        return Err(invalid());
    }
    let limit_reached = envelope.result.len() == RECORD_LIMIT;
    let mut seen = HashSet::new();
    let mut records = BTreeMap::new();
    let mut rejected = 0;
    for value in envelope.result {
        let Some(record) = value.as_object() else {
            rejected += 1;
            continue;
        };
        // Any supplied correlation must agree; never combine foreign-device records.
        if record.get("did").is_some_and(|v| v.as_str() != Some(did))
            || record
                .get("key")
                .is_some_and(|v| v.as_str() != Some("11.1"))
        {
            return Err(invalid());
        }
        let Some(time) = timestamp(record.get("time"))
            .filter(|time| *time >= window.start && *time < window.end)
        else {
            rejected += 1;
            continue;
        };
        if !seen.insert(time) {
            rejected += 1;
            if records.remove(&time).is_some() {
                rejected += 1;
            }
            continue;
        }
        let Some(kilowatt_hours) = energy(record.get("value")) else {
            rejected += 1;
            continue;
        };
        let reported_at = DateTime::from_timestamp(time, 0).ok_or_else(invalid)?;
        records.insert(
            time,
            DailyEnergyRecord {
                reported_at,
                kilowatt_hours,
            },
        );
    }
    Ok(DailyEnergyHistory {
        window,
        received_at,
        records: records.into_values().collect(),
        rejected_records: rejected,
        limit_reached,
        completeness: StatisticsCompleteness::Unproven,
    })
}

#[cfg(test)]
mod tests;
