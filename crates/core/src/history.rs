//! Bounded, explicit historical reads. No cache, receipt or verification mutation.
use crate::*;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::{collections::HashSet, sync::OnceLock};

fn unsupported() -> Error {
    Error::new(
        "CAPABILITY_UNSUPPORTED",
        "This historical series is unavailable.",
    )
}
fn invalid_history() -> Error {
    Error::new(
        "INVALID_HISTORY",
        "The historical result does not satisfy its admitted contract.",
    )
}
fn shape(value: &impl Serialize) -> bool {
    static VALIDATOR: OnceLock<jsonschema::Validator> = OnceLock::new();
    let validator = VALIDATOR.get_or_init(|| {
        let schema: serde_json::Value =
            serde_json::from_str(include_str!("../../../schemas/history.schema.json"))
                .expect("checked history schema");
        jsonschema::validator_for(&schema).expect("checked history schema")
    });
    serde_json::to_value(value).is_ok_and(|value| validator.is_valid(&value))
}
fn validate_options(options: &HistoryOptions, descriptor: &Descriptor) -> Result<()> {
    if options.options.len() > 64 || !shape(options) {
        return Err(unsupported());
    }
    let mut targets = HashSet::new();
    for option in &options.options {
        if !targets.insert((&option.target, option.aggregation)) {
            return Err(unsupported());
        }
        let property = descriptor
            .capability(&option.target.component_id, &option.target.capability)
            .and_then(|capability| capability.properties.get(&option.target.property))
            .ok_or_else(unsupported)?;
        if property.get("readable") != Some(&serde_json::Value::Bool(true))
            || !matches!(
                property.get("type").and_then(|v| v.as_str()),
                Some("number" | "integer")
            )
            || option.unit.chars().any(char::is_control)
        {
            return Err(unsupported());
        }
    }
    Ok(())
}
fn validate_result(
    result: &HistoryResult,
    request: &HistoryRequest,
    option: &HistoryOption,
    connection: &Connection,
    started: DateTime<Utc>,
    finished: DateTime<Utc>,
) -> Result<()> {
    let count = result
        .records
        .len()
        .checked_add(result.rejected_records as usize)
        .ok_or_else(invalid_history)?;
    if result.records.len() > option.max_records as usize
        || count > option.max_records as usize
        || !shape(result)
        || result.device_id != request.device_id
        || result.connection_id != request.connection_id
        || result.target != request.target
        || result.aggregation != request.aggregation
        || result.start != request.start
        || result.end != request.end
        || result.unit != option.unit
        || result.bucket_timezone != option.bucket_timezone
        || result.source != connection.transport
        || result.received_at < started
        || result.received_at > finished
        || result.limit_reached != (count == option.max_records as usize)
    {
        return Err(invalid_history());
    }
    let mut previous = None;
    for record in &result.records {
        if !record.value.is_finite()
            || record.value < 0.0
            || record.reported_at < request.start
            || record.reported_at >= request.end
            || previous.is_some_and(|time| record.reported_at <= time)
        {
            return Err(invalid_history());
        }
        previous = Some(record.reported_at);
    }
    Ok(())
}
impl Core {
    fn history_connection(&self, device: &Device, id: &str) -> Result<Connection> {
        self.routes(device, Route::Auto, None)
            .into_iter()
            .find(|c| c.id == id)
            .ok_or_else(unsupported)
    }
    async fn history_options_for(
        &self,
        descriptor: &Descriptor,
        connection: &Connection,
        operation: &Operation,
    ) -> Result<HistoryOptions> {
        let adapter = self
            .adapters
            .iter()
            .find(|a| a.id() == connection.adapter)
            .ok_or_else(unsupported)?;
        let options = operation
            .run(adapter.history_options(&descriptor.device, connection, operation))
            .await?;
        let result = HistoryOptions {
            schema_version: SCHEMA_VERSION.into(),
            device_id: descriptor.device.id.clone(),
            connection_id: connection.id.clone(),
            options,
        };
        validate_options(&result, descriptor)?;
        operation.run(async { Ok(result) }).await
    }
    /// Local protected inspection only. No networking or changes to authority/metadata.
    pub async fn inspect_history_options(
        &self,
        device_id: &str,
        connection_id: &str,
        operation: &Operation,
    ) -> Result<HistoryOptions> {
        operation
            .run(async {
                let descriptor = self.describe(device_id)?;
                if descriptor.device.id != device_id {
                    return Err(Error::invalid());
                }
                let connection = self.history_connection(&descriptor.device, connection_id)?;
                self.history_options_for(&descriptor, &connection, operation)
                    .await
            })
            .await
    }
    /// One selected connection, no automatic retries or fallback and no state/receipt writes.
    pub async fn read_history(
        &self,
        request: &HistoryRequest,
        operation: &Operation,
    ) -> Result<HistoryResult> {
        operation
            .run(async {
                let started = self.clock.now();
                if !shape(request)
                    || request.start.timestamp() <= 0
                    || request.end <= request.start
                    || request.end > started
                    || request
                        .end
                        .signed_duration_since(request.start)
                        .num_seconds()
                        > 32 * 86400
                {
                    return Err(Error::invalid());
                }
                let _lock = operation
                    .run(async { self.store.lock_device(&request.device_id) })
                    .await?;
                let descriptor = self.describe(&request.device_id)?;
                if descriptor.device.id != request.device_id {
                    return Err(Error::invalid());
                }
                let connection =
                    self.history_connection(&descriptor.device, &request.connection_id)?;
                let options = self
                    .history_options_for(&descriptor, &connection, operation)
                    .await?;
                let option = options
                    .options
                    .iter()
                    .find(|option| {
                        option.target == request.target && option.aggregation == request.aggregation
                    })
                    .ok_or_else(unsupported)?;
                if request
                    .end
                    .signed_duration_since(request.start)
                    .num_seconds()
                    > i64::from(option.max_window_seconds)
                {
                    return Err(Error::invalid());
                }
                let adapter = self
                    .adapters
                    .iter()
                    .find(|a| a.id() == connection.adapter)
                    .ok_or_else(unsupported)?;
                let result = operation
                    .run(adapter.read_history(&descriptor.device, &connection, request, operation))
                    .await?;
                validate_result(
                    &result,
                    request,
                    option,
                    &connection,
                    started,
                    self.clock.now(),
                )?;
                // Repeat local descriptor admission after I/O; changed bindings cannot publish history.
                let current = self.describe(&request.device_id)?;
                if serde_json::to_value(&current).map_err(|_| invalid_history())?
                    != serde_json::to_value(&descriptor).map_err(|_| invalid_history())?
                {
                    return Err(Error::new(
                        "DEVICE_IDENTITY_CONFLICT",
                        "Device metadata changed during the historical read.",
                    ));
                }
                operation.run(async { Ok(result) }).await
            })
            .await
    }
}
