//! Reported event occurrence times; opaque values never become public payloads.
use crate::{CloudDevice, CloudEndpoint, CloudTransport, MijiaSession, Region, invalid};
use chrono::{DateTime, Utc};
use homeverb_core::{Operation, SecretBytes};
use homeverb_domain::{Error, Result};
use serde::Deserialize;
use serde_json::{Value, json};

pub(crate) const RECORD_LIMIT: usize = 31;
/// Closed selector into the pinned event catalog, never a raw provider key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventHistoryQuery {
    pub siid: u32,
    pub eiid: u32,
}
/// Positive half-open whole-second UTC window of at most 32 days.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventHistoryWindow(crate::PropertyHistoryWindow);
impl EventHistoryWindow {
    pub fn new(start: i64, end: i64) -> Result<Self> {
        Ok(Self(crate::PropertyHistoryWindow::new(start, end)?))
    }
    pub fn start(&self) -> i64 {
        self.0.start()
    }
    pub fn end(&self) -> i64 {
        self.0.end()
    }
}
/// One provider row. Equal timestamps do not establish equal or distinct events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventOccurrence {
    pub reported_at: DateTime<Utc>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ReportedEventHistory {
    pub window: EventHistoryWindow,
    pub received_at: DateTime<Utc>,
    pub records: Vec<EventOccurrence>,
    pub rejected_records: usize,
    pub limit_reached: bool,
    pub completeness: crate::StatisticsCompleteness,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Profile {
    pub model: String,
    pub spec_type: String,
    pub service_type: String,
    pub event_type: String,
    pub siid: u32,
    pub eiid: u32,
    pub arguments: Vec<u32>,
}
pub(crate) fn profiles() -> Result<Vec<Profile>> {
    crate::decode(include_bytes!("event_profiles.json"))
}
pub(crate) fn unsupported() -> Error {
    Error::new(
        "CAPABILITY_UNSUPPORTED",
        "This event history has no reviewed exact mapping.",
    )
}
pub(crate) fn admit(model: &str, specification: &str, query: &EventHistoryQuery) -> Result<()> {
    let profiles = profiles()?;
    let mut matching = profiles.iter().filter(|p| {
        p.model == model
            && p.spec_type == specification
            && p.siid == query.siid
            && p.eiid == query.eiid
    });
    let p = matching.next().ok_or_else(unsupported)?;
    if matching.next().is_some() || p.siid == 0 || p.eiid == 0 || p.arguments.contains(&0) {
        return Err(unsupported());
    }
    crate::spec::validate_device_type(&p.spec_type)?;
    validate_event_urn(&p.service_type, "service")?;
    validate_event_urn(&p.event_type, "event")?;
    Ok(())
}
// Only pinned event namespaces are admitted. Normalize a temporary syntax-check
// input; exact catalog identity and stored URNs retain their original namespace.
fn validate_event_urn(value: &str, category: &str) -> Result<()> {
    if value.len() > 512 || !matches!(category, "service" | "event") {
        return Err(invalid());
    }
    let (namespace, suffix) = value
        .strip_prefix("urn:")
        .and_then(|v| v.split_once(':'))
        .ok_or_else(invalid)?;
    if !matches!(
        namespace,
        "miot-spec-v2" | "cuco-spec" | "madv-spec" | "xiaomi-spec"
    ) {
        return Err(invalid());
    }
    crate::spec::urn(&format!("urn:miot-spec-v2:{suffix}"), category).map(|_| ())
}
pub(crate) fn payload(
    did: &str,
    query: &EventHistoryQuery,
    window: EventHistoryWindow,
) -> Result<SecretBytes> {
    if !crate::bounded_text(did, 128) || window.end() > Utc::now().timestamp() {
        return Err(Error::invalid());
    }
    Ok(SecretBytes::new(
        serde_json::to_vec(&json!({
            "did":did,"key":format!("{}.{}",query.siid,query.eiid),"type":"event",
            "time_start":window.start(),"time_end":window.end(),"limit":RECORD_LIMIT,
        }))
        .map_err(|_| invalid())?,
    ))
}
impl MijiaSession {
    /// One explicit historical page. Protected membership belongs to the adapter.
    pub async fn reported_event_history(
        &self,
        transport: &dyn CloudTransport,
        device: &CloudDevice,
        query: &EventHistoryQuery,
        window: EventHistoryWindow,
        operation: &Operation,
    ) -> Result<ReportedEventHistory> {
        operation
            .run(async {
                if self.region() != Region::Cn {
                    return Err(unsupported());
                }
                admit(
                    &device.model,
                    device.spec_type.as_deref().ok_or_else(unsupported)?,
                    query,
                )?;
                let data = payload(device.did.as_str(), query, window)?;
                let bytes = self
                    .request(transport, CloudEndpoint::EventHistory, &data, operation)
                    .await?;
                let result = decode_history(
                    bytes.expose(),
                    device.did.as_str(),
                    query,
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
pub(crate) fn decode_history(
    bytes: &[u8],
    did: &str,
    query: &EventHistoryQuery,
    window: EventHistoryWindow,
    received_at: DateTime<Utc>,
) -> Result<ReportedEventHistory> {
    if bytes.len() > crate::MAX_RESPONSE_BYTES {
        return Err(invalid());
    }
    let envelope: Envelope = crate::decode(bytes)?;
    if envelope.code != 0 || envelope.result.len() > RECORD_LIMIT {
        return Err(invalid());
    }
    let key = format!("{}.{}", query.siid, query.eiid);
    let limit_reached = envelope.result.len() == RECORD_LIMIT;
    let mut records = Vec::new();
    let mut rejected_records = 0;
    for row in envelope.result {
        let Some(row) = row.as_object() else {
            rejected_records += 1;
            continue;
        };
        if row.get("did").and_then(Value::as_str) != Some(did)
            || row.get("key").and_then(Value::as_str) != Some(key.as_str())
            || row.get("type").and_then(Value::as_str) != Some("event")
        {
            return Err(invalid());
        }
        let Some(time) =
            timestamp(row.get("time")).filter(|t| *t >= window.start() && *t < window.end())
        else {
            rejected_records += 1;
            continue;
        };
        if !row.contains_key("value") {
            rejected_records += 1;
            continue;
        }
        records.push(EventOccurrence {
            reported_at: DateTime::from_timestamp(time, 0).ok_or_else(invalid)?,
        });
    }
    records.sort_by_key(|r| r.reported_at);
    Ok(ReportedEventHistory {
        window,
        received_at,
        records,
        rejected_records,
        limit_reached,
        completeness: crate::StatisticsCompleteness::Unproven,
    })
}
#[cfg(test)]
mod tests;
