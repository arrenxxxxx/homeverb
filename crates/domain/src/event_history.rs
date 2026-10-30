//! Explicit reported occurrence times, separate from numeric history and live state.
use crate::history::whole_utc;
use crate::{HistoryCompleteness, Transport};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventHistoryProjection {
    OccurrenceTimes,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventHistoryTarget {
    pub component_id: String,
    pub capability: String,
    pub event: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventHistoryOption {
    pub target: EventHistoryTarget,
    pub projection: EventHistoryProjection,
    pub max_window_seconds: u32,
    pub max_records: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventHistoryOptions {
    pub schema_version: String,
    pub device_id: String,
    pub connection_id: String,
    pub options: Vec<EventHistoryOption>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventHistoryRequest {
    pub schema_version: String,
    pub device_id: String,
    pub connection_id: String,
    pub target: EventHistoryTarget,
    pub projection: EventHistoryProjection,
    #[serde(deserialize_with = "whole_utc")]
    pub start: DateTime<Utc>,
    #[serde(deserialize_with = "whole_utc")]
    pub end: DateTime<Utc>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventHistoryRecord {
    #[serde(deserialize_with = "whole_utc")]
    pub reported_at: DateTime<Utc>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventHistoryResult {
    pub schema_version: String,
    pub device_id: String,
    pub connection_id: String,
    pub target: EventHistoryTarget,
    pub projection: EventHistoryProjection,
    #[serde(deserialize_with = "whole_utc")]
    pub start: DateTime<Utc>,
    #[serde(deserialize_with = "whole_utc")]
    pub end: DateTime<Utc>,
    pub source: Transport,
    pub received_at: DateTime<Utc>,
    pub records: Vec<EventHistoryRecord>,
    pub rejected_records: u32,
    pub limit_reached: bool,
    pub completeness: HistoryCompleteness,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    fn fixture(name: &str) -> Value {
        serde_json::from_str(match name {
            "options" => include_str!("../../../examples/event-history.options.json"),
            "request" => include_str!("../../../examples/event-history.request.json"),
            "result" => include_str!("../../../examples/event-history.result.json"),
            _ => unreachable!(),
        })
        .unwrap()
    }
    fn valid(name: &str, value: Value) -> bool {
        match name {
            "options" => serde_json::from_value::<EventHistoryOptions>(value).is_ok(),
            "request" => serde_json::from_value::<EventHistoryRequest>(value).is_ok(),
            "result" => serde_json::from_value::<EventHistoryResult>(value).is_ok(),
            _ => unreachable!(),
        }
    }
    #[test]
    fn event_history_examples_keep_equal_timestamps_without_numeric_values() {
        let options: EventHistoryOptions = serde_json::from_value(fixture("options")).unwrap();
        let request: EventHistoryRequest = serde_json::from_value(fixture("request")).unwrap();
        let result: EventHistoryResult = serde_json::from_value(fixture("result")).unwrap();
        assert_eq!(result.target, request.target);
        assert_eq!(options.options[0].target, request.target);
        assert_eq!(result.records[0], result.records[1]);
        assert_eq!(serde_json::to_value(result).unwrap(), fixture("result"));
        assert_eq!(serde_json::to_value(options).unwrap(), fixture("options"));
        assert_eq!(serde_json::to_value(request).unwrap(), fixture("request"));
        assert!(serde_json::from_value::<crate::HistoryResult>(fixture("result")).is_err());
    }
    #[test]
    fn event_history_rejects_remote_authority_and_payload_injection() {
        for name in ["options", "request", "result"] {
            for field in [
                "provider",
                "did",
                "key",
                "uid",
                "token",
                "endpoint",
                "principal",
                "confirmed",
                "limit",
                "cursor",
                "payload",
            ] {
                let mut value = fixture(name);
                value[field] = json!("FORBIDDEN");
                assert!(!valid(name, value), "{name}/{field}");
            }
        }
        for field in ["value", "payload", "actor", "event_id", "url", "verified"] {
            let mut value = fixture("result");
            value["records"][0][field] = json!("FORBIDDEN");
            assert!(!valid("result", value), "record/{field}");
        }
        for name in ["request", "result"] {
            let mut value = fixture(name);
            value["target"]["property"] = json!("state");
            assert!(!valid(name, value));
            let mut value = fixture(name);
            value["projection"] = json!("raw_payload");
            assert!(!valid(name, value));
        }
    }
    #[test]
    fn event_history_refuses_noncanonical_and_leap_second_query_times() {
        for time in [
            "2026-10-01T00:00:00+00:00",
            "2026-10-01T00:00:00.000Z",
            "2026-10-01T00:00:00.1Z",
            "2026-10-01T00:00:60Z",
            "2026-02-30T00:00:00Z",
            "invalid",
        ] {
            for name in ["request", "result"] {
                for field in ["start", "end"] {
                    let mut value = fixture(name);
                    value[field] = json!(time);
                    assert!(!valid(name, value), "{name}/{field}/{time}");
                }
            }
            let mut value = fixture("result");
            value["records"][0]["reported_at"] = json!(time);
            assert!(!valid("result", value));
        }
    }
}
