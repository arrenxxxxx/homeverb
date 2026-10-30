//! Protected exact-profile historical admission, independent of public descriptor claims.
use super::*;
use homeverb_domain::{
    HistoryAggregation, HistoryCompleteness, HistoryOption, HistoryRecord, HistoryRequest,
    HistoryResult, HistoryTarget, SCHEMA_VERSION,
};

fn option(binding: &Binding) -> Option<HistoryOption> {
    let mapping = binding.mapping.as_ref()?;
    if binding.is_modern()
        || !binding.listed
        || binding.model != crate::statistics::DAILY_ENERGY_MODEL
        || mapping.spec_type != crate::statistics::DAILY_ENERGY_SPEC
    {
        return None;
    }
    let property = mapping
        .properties
        .iter()
        .find(|p| p.siid == 11 && p.piid == 1)?;
    Some(HistoryOption {
        target: HistoryTarget {
            component_id: property.component.clone(),
            capability: property.capability.clone(),
            property: property.property.clone(),
        },
        aggregation: HistoryAggregation::ReportedDaily,
        unit: "kWh".into(),
        max_window_seconds: 32 * 86400,
        max_records: 31,
        bucket_timezone: None,
    })
}
impl MijiaAdapter {
    pub(super) async fn numeric_history_options(
        &self,
        device: &Device,
        connection: &Connection,
        operation: &Operation,
    ) -> Result<Vec<HistoryOption>> {
        operation
            .run(async {
                let id = connection
                    .account_id
                    .as_deref()
                    .and_then(|id| Uuid::parse_str(id).ok())
                    .ok_or_else(conflict)?;
                let account =
                    LoadedAccount::load(self.vault.lease(id, operation).await?, false, operation)
                        .await?;
                let binding = account
                    .payload
                    .bindings
                    .iter()
                    .find(|b| b.device_id.to_string() == device.id)
                    .ok_or_else(conflict)?;
                validate_device(binding, id, device, connection)?;
                Ok(option(binding).into_iter().collect())
            })
            .await
    }
    pub(super) async fn numeric_history_read(
        &self,
        device: &Device,
        connection: &Connection,
        request: &HistoryRequest,
        operation: &Operation,
    ) -> Result<HistoryResult> {
        operation
            .run(async {
                let id = connection
                    .account_id
                    .as_deref()
                    .and_then(|id| Uuid::parse_str(id).ok())
                    .ok_or_else(conflict)?;
                let account =
                    LoadedAccount::load(self.vault.lease(id, operation).await?, false, operation)
                        .await?;
                let binding = account
                    .payload
                    .bindings
                    .iter()
                    .find(|b| b.device_id.to_string() == device.id)
                    .ok_or_else(conflict)?;
                validate_device(binding, id, device, connection)?;
                let option = option(binding).ok_or_else(unsupported)?;
                if request.schema_version != SCHEMA_VERSION
                    || request.device_id != device.id
                    || request.connection_id != connection.id
                    || request.target != option.target
                    || request.aggregation != option.aggregation
                    || request.start.timestamp_subsec_nanos() != 0
                    || request.end.timestamp_subsec_nanos() != 0
                    || request.end > self.clock.now()
                {
                    return Err(Error::invalid());
                }
                let window = crate::DailyEnergyWindow::new(
                    request.start.timestamp(),
                    request.end.timestamp(),
                )?;
                let session = account.payload.legacy_session()?;
                let mut exact_device = binding.device();
                // A protected pinned reference derivation is evidence even when inventory omitted the URN.
                exact_device.spec_type = Some(crate::statistics::DAILY_ENERGY_SPEC.into());
                let history = session
                    .daily_energy_history(self.transport.as_ref(), &exact_device, window, operation)
                    .await?;
                let result = HistoryResult {
                    schema_version: SCHEMA_VERSION.into(),
                    device_id: device.id.clone(),
                    connection_id: connection.id.clone(),
                    target: option.target,
                    aggregation: option.aggregation,
                    start: request.start,
                    end: request.end,
                    unit: option.unit,
                    source: Transport::Cloud,
                    received_at: self.clock.now(),
                    records: history
                        .records
                        .into_iter()
                        .map(|r| HistoryRecord {
                            reported_at: r.reported_at,
                            value: r.kilowatt_hours,
                        })
                        .collect(),
                    rejected_records: u32::try_from(history.rejected_records)
                        .map_err(|_| crate::invalid())?,
                    limit_reached: history.limit_reached,
                    completeness: HistoryCompleteness::Unproven,
                    bucket_timezone: None,
                };
                operation.run(async { Ok(result) }).await
            })
            .await
    }
}
