//! Historical aggregates are separate from current observations and execution evidence.
use crate::Transport;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub(crate) fn whole_utc<'de, D: serde::Deserializer<'de>>(d: D) -> Result<DateTime<Utc>, D::Error> {
    let text = String::deserialize(d)?;
    let value = DateTime::parse_from_rfc3339(&text).map_err(serde::de::Error::custom)?;
    if text.len() != 20 || !text.ends_with('Z') || value.timestamp_subsec_nanos() != 0 {
        return Err(serde::de::Error::custom(
            "A whole-second UTC timestamp is required.",
        ));
    }
    Ok(value.with_timezone(&Utc))
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryTarget {
    pub component_id: String,
    pub capability: String,
    pub property: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HistoryAggregation {
    ReportedDaily,
    ReportedSamples,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HistoryCompleteness {
    Unproven,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryOption {
    pub target: HistoryTarget,
    pub aggregation: HistoryAggregation,
    pub unit: Option<String>,
    pub max_window_seconds: u32,
    pub max_records: u32,
    /// Unknown for the initial contract; reported timestamps do not establish a timezone.
    pub bucket_timezone: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryOptions {
    pub schema_version: String,
    pub device_id: String,
    pub connection_id: String,
    pub options: Vec<HistoryOption>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryRequest {
    pub schema_version: String,
    pub device_id: String,
    pub connection_id: String,
    pub target: HistoryTarget,
    pub aggregation: HistoryAggregation,
    #[serde(deserialize_with = "whole_utc")]
    pub start: DateTime<Utc>,
    #[serde(deserialize_with = "whole_utc")]
    pub end: DateTime<Utc>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryRecord {
    #[serde(deserialize_with = "whole_utc")]
    pub reported_at: DateTime<Utc>,
    pub value: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryResult {
    pub schema_version: String,
    pub device_id: String,
    pub connection_id: String,
    pub target: HistoryTarget,
    pub aggregation: HistoryAggregation,
    #[serde(deserialize_with = "whole_utc")]
    pub start: DateTime<Utc>,
    #[serde(deserialize_with = "whole_utc")]
    pub end: DateTime<Utc>,
    pub unit: Option<String>,
    pub source: Transport,
    pub received_at: DateTime<Utc>,
    pub records: Vec<HistoryRecord>,
    pub rejected_records: u32,
    pub limit_reached: bool,
    pub completeness: HistoryCompleteness,
    pub bucket_timezone: Option<String>,
}
