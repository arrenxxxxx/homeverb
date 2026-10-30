//! Protected exact-profile historical admission, independent of public descriptor claims.
use super::*;
use homeverb_domain::{
    HistoryAggregation, HistoryCompleteness, HistoryOption, HistoryRecord, HistoryRequest,
    HistoryResult, HistoryTarget, SCHEMA_VERSION,
};

fn option(binding: &Binding) -> Option<HistoryOption> {
    let mapping = binding.mapping.as_ref()?;
    if !binding.listed
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
        unit: Some("kWh".into()),
        max_window_seconds: 32 * 86400,
        max_records: 31,
        bucket_timezone: None,
    })
}
fn options(binding: &Binding) -> Result<Vec<HistoryOption>> {
    let mut options: Vec<_> = option(binding).into_iter().collect();
    if !binding.listed {
        return Ok(options);
    }
    let Some(mapping) = &binding.mapping else {
        return Ok(options);
    };
    let Some(pinned) = ModelMapping::pinned(&binding.model, Some(&mapping.spec_type))? else {
        return Ok(options);
    };
    for property in &mapping.properties {
        if !matches!(
            property.definition["type"].as_str(),
            Some("number" | "integer")
        ) {
            continue;
        }
        let Some(exact) = pinned
            .properties
            .iter()
            .find(|p| p.siid == property.siid && p.piid == property.piid)
        else {
            continue;
        };
        if exact.component != property.component
            || exact.capability != property.capability
            || exact.property != property.property
            || exact.definition != property.definition
        {
            return Err(conflict());
        }
        let definition = pinned.numeric_history_definition(property.siid, property.piid)?;
        options.push(HistoryOption {
            target: HistoryTarget {
                component_id: property.component.clone(),
                capability: property.capability.clone(),
                property: property.property.clone(),
            },
            aggregation: HistoryAggregation::ReportedSamples,
            unit: definition["unit"].as_str().map(str::to_owned),
            max_window_seconds: 32 * 86400,
            max_records: 31,
            bucket_timezone: None,
        });
    }
    Ok(options)
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
                if binding.is_modern() && self.modern_reads.is_none() {
                    return Ok(Vec::new());
                }
                options(binding)
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
                let option = options(binding)?
                    .into_iter()
                    .find(|option| {
                        option.target == request.target && option.aggregation == request.aggregation
                    })
                    .ok_or_else(unsupported)?;
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
                let mapping = binding.mapping.as_ref().ok_or_else(unsupported)?;
                let field = mapping
                    .properties
                    .iter()
                    .find(|field| {
                        field.component == option.target.component_id
                            && field.capability == option.target.capability
                            && field.property == option.target.property
                    })
                    .ok_or_else(unsupported)?;
                let mut exact_device = binding.device();
                exact_device.spec_type = Some(mapping.spec_type.clone());
                let query = PropertyQuery {
                    siid: field.siid,
                    piid: field.piid,
                };
                let (records, rejected_records, limit_reached) = match option.aggregation {
                    HistoryAggregation::ReportedSamples => {
                        let window = crate::PropertyHistoryWindow::new(
                            request.start.timestamp(),
                            request.end.timestamp(),
                        )?;
                        let history = if binding.is_modern() {
                            let session = account.payload.modern_session()?;
                            let transport = self.modern_reads.as_deref().ok_or_else(unsupported)?;
                            let mut entry = session.protected_entry(&exact_device)?;
                            entry.kind = Some(crate::modern::reads::DirectoryKind(
                                binding.modern.as_ref().ok_or_else(conflict)?.kind,
                            ));
                            session
                                .numeric_property_history(
                                    transport, &entry, &query, window, operation,
                                )
                                .await?
                        } else {
                            account
                                .payload
                                .legacy_session()?
                                .numeric_property_history(
                                    self.transport.as_ref(),
                                    &exact_device,
                                    &query,
                                    window,
                                    operation,
                                )
                                .await?
                        };
                        if history.unit != option.unit {
                            return Err(crate::invalid());
                        }
                        (
                            history
                                .records
                                .into_iter()
                                .map(|r| HistoryRecord {
                                    reported_at: r.reported_at,
                                    value: r.value,
                                })
                                .collect(),
                            history.rejected_records,
                            history.limit_reached,
                        )
                    }
                    HistoryAggregation::ReportedDaily => {
                        let window = crate::DailyEnergyWindow::new(
                            request.start.timestamp(),
                            request.end.timestamp(),
                        )?;
                        let history = if binding.is_modern() {
                            let session = account.payload.modern_session()?;
                            let transport = self.modern_reads.as_deref().ok_or_else(unsupported)?;
                            let mut entry = session.protected_entry(&exact_device)?;
                            entry.kind = Some(crate::modern::reads::DirectoryKind(
                                binding.modern.as_ref().ok_or_else(conflict)?.kind,
                            ));
                            session
                                .daily_energy_history(transport, &entry, window, operation)
                                .await?
                        } else {
                            account
                                .payload
                                .legacy_session()?
                                .daily_energy_history(
                                    self.transport.as_ref(),
                                    &exact_device,
                                    window,
                                    operation,
                                )
                                .await?
                        };
                        (
                            history
                                .records
                                .into_iter()
                                .map(|r| HistoryRecord {
                                    reported_at: r.reported_at,
                                    value: r.kilowatt_hours,
                                })
                                .collect(),
                            history.rejected_records,
                            history.limit_reached,
                        )
                    }
                };
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
                    records,
                    rejected_records: u32::try_from(rejected_records)
                        .map_err(|_| crate::invalid())?,
                    limit_reached,
                    completeness: HistoryCompleteness::Unproven,
                    bucket_timezone: None,
                };
                operation.run(async { Ok(result) }).await
            })
            .await
    }
}
