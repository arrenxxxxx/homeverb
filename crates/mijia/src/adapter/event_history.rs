//! Protected event-only metadata and one selected-protocol historical request.
use super::*;
use crate::{EventHistoryQuery, EventHistoryWindow};
use homeverb_domain::{
    EventHistoryOption, EventHistoryProjection, EventHistoryRecord, EventHistoryRequest,
    EventHistoryResult, EventHistoryTarget, HistoryCompleteness, SCHEMA_VERSION,
};
use serde_json::json;

fn entries(binding: &Binding) -> Result<Vec<(EventHistoryQuery, EventHistoryOption)>> {
    if binding.modern.as_ref().is_some_and(|m| m.parent.is_some()) {
        return Ok(vec![]);
    }
    let Some(mapping) = &binding.mapping else {
        return Ok(vec![]);
    };
    mapping.validate_saved()?;
    if mapping.model != binding.model {
        return Err(conflict());
    }
    let mut entries = Vec::new();
    for profile in crate::event_history::profiles()?
        .into_iter()
        .filter(|p| p.model == binding.model && p.spec_type == mapping.spec_type)
    {
        let query = EventHistoryQuery {
            siid: profile.siid,
            eiid: profile.eiid,
        };
        crate::event_history::admit(&binding.model, &mapping.spec_type, &query)?;
        let service = profile
            .service_type
            .split(':')
            .nth(3)
            .ok_or_else(unsupported)?
            .replace('-', "_");
        let namespace = profile
            .service_type
            .split(':')
            .nth(1)
            .ok_or_else(unsupported)?
            .replace('-', "_");
        let event = profile
            .event_type
            .split(':')
            .nth(3)
            .ok_or_else(unsupported)?
            .replace('-', "_");
        entries.push((
            query,
            EventHistoryOption {
                target: EventHistoryTarget {
                    component_id: format!("{service}-{}", profile.siid),
                    capability: format!("vendor.xiaomi.mijia.event.{namespace}.{service}"),
                    event: format!("{event}_{}", profile.eiid),
                },
                projection: EventHistoryProjection::OccurrenceTimes,
                max_window_seconds: 32 * 86400,
                max_records: 31,
            },
        ));
    }
    Ok(entries)
}
fn augment(binding: &Binding, mut descriptor: Descriptor) -> Result<Descriptor> {
    for (_, option) in entries(binding)? {
        let target = option.target;
        if let Some(component) = descriptor
            .device
            .components
            .iter_mut()
            .find(|c| c.id == target.component_id)
        {
            if !component.capabilities.contains(&target.capability) {
                component.capabilities.push(target.capability.clone());
            }
        } else {
            descriptor.device.components.push(Component {
                id: target.component_id.clone(),
                capabilities: vec![target.capability.clone()],
            });
        }
        let position = match descriptor.capabilities.iter().position(|c| {
            c.component_id.as_deref() == Some(&target.component_id) && c.id == target.capability
        }) {
            Some(position) => position,
            None => {
                descriptor.capabilities.push(Capability {
                    schema_version: SCHEMA_VERSION.into(),
                    id: target.capability.clone(),
                    component_id: Some(target.component_id),
                    revision: 1,
                    properties: BTreeMap::new(),
                    actions: BTreeMap::new(),
                    events: BTreeMap::new(),
                });
                descriptor.capabilities.len() - 1
            }
        };
        if descriptor.capabilities[position]
            .events
            .insert(
                target.event,
                json!({"history":{"projection":"occurrence_times"}}),
            )
            .is_some()
        {
            return Err(conflict());
        }
    }
    Ok(descriptor)
}
pub(super) fn matches_components(
    binding: &Binding,
    original: &Descriptor,
    device: &Device,
) -> Result<bool> {
    let actual = serde_json::to_value(&device.components).map_err(|_| conflict())?;
    if actual == serde_json::to_value(&original.device.components).map_err(|_| conflict())? {
        return Ok(true);
    }
    Ok(actual
        == serde_json::to_value(&augment(binding, original.clone())?.device.components)
            .map_err(|_| conflict())?)
}
impl MijiaAdapter {
    pub(super) fn event_descriptor(
        &self,
        binding: &Binding,
        descriptor: Descriptor,
    ) -> Result<Descriptor> {
        if self.event_history && (!binding.is_modern() || self.modern_reads.is_some()) {
            augment(binding, descriptor)
        } else {
            Ok(descriptor)
        }
    }
    fn admitted_events(
        &self,
        binding: &Binding,
        account: Uuid,
        device: &Device,
        connection: &Connection,
    ) -> Result<Vec<(EventHistoryQuery, EventHistoryOption)>> {
        validate_device(binding, account, device, connection)?;
        if !self.event_history
            || !binding.listed
            || (binding.is_modern() && self.modern_reads.is_none())
        {
            return Ok(vec![]);
        }
        let expected = augment(binding, descriptor(binding, account, false, false, false))?;
        // Old saved descriptors keep ordinary reads but cannot publish event history.
        if serde_json::to_value(&device.components).map_err(|_| conflict())?
            != serde_json::to_value(&expected.device.components).map_err(|_| conflict())?
        {
            return Ok(vec![]);
        }
        entries(binding)
    }
    pub(super) async fn protected_event_options(
        &self,
        device: &Device,
        connection: &Connection,
        operation: &Operation,
    ) -> Result<Vec<EventHistoryOption>> {
        operation
            .run(async {
                if !self.event_history {
                    return Ok(vec![]);
                }
                let id = connection
                    .account_id
                    .as_deref()
                    .and_then(|s| Uuid::parse_str(s).ok())
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
                let result = self
                    .admitted_events(binding, id, device, connection)?
                    .into_iter()
                    .map(|(_, o)| o)
                    .collect();
                operation.run(async { Ok(result) }).await
            })
            .await
    }
    pub(super) async fn protected_event_read(
        &self,
        device: &Device,
        connection: &Connection,
        request: &EventHistoryRequest,
        operation: &Operation,
    ) -> Result<EventHistoryResult> {
        operation
            .run(async {
                if !self.event_history {
                    return Err(unsupported());
                }
                let id = connection
                    .account_id
                    .as_deref()
                    .and_then(|s| Uuid::parse_str(s).ok())
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
                let (query, option) = self
                    .admitted_events(binding, id, device, connection)?
                    .into_iter()
                    .find(|(_, o)| o.target == request.target && o.projection == request.projection)
                    .ok_or_else(unsupported)?;
                if request.schema_version != SCHEMA_VERSION
                    || request.device_id != device.id
                    || request.connection_id != connection.id
                    || request.start.timestamp_subsec_nanos() != 0
                    || request.end.timestamp_subsec_nanos() != 0
                    || request.end > self.clock.now()
                {
                    return Err(Error::invalid());
                }
                let window =
                    EventHistoryWindow::new(request.start.timestamp(), request.end.timestamp())?;
                let mut exact = binding.device();
                exact.spec_type = Some(
                    binding
                        .mapping
                        .as_ref()
                        .ok_or_else(unsupported)?
                        .spec_type
                        .clone(),
                );
                let history = if binding.is_modern() {
                    let session = account.payload.modern_session()?;
                    let mut entry = session.protected_entry(&exact)?;
                    entry.kind = Some(crate::modern::reads::DirectoryKind(
                        binding.modern.as_ref().ok_or_else(conflict)?.kind,
                    ));
                    session
                        .reported_event_history(
                            self.modern_reads.as_deref().ok_or_else(unsupported)?,
                            &entry,
                            &query,
                            window,
                            operation,
                        )
                        .await?
                } else {
                    account
                        .payload
                        .legacy_session()?
                        .reported_event_history(
                            self.transport.as_ref(),
                            &exact,
                            &query,
                            window,
                            operation,
                        )
                        .await?
                };
                let result = EventHistoryResult {
                    schema_version: SCHEMA_VERSION.into(),
                    device_id: device.id.clone(),
                    connection_id: connection.id.clone(),
                    target: option.target,
                    projection: option.projection,
                    start: request.start,
                    end: request.end,
                    source: Transport::Cloud,
                    received_at: self.clock.now(),
                    records: history
                        .records
                        .into_iter()
                        .map(|r| EventHistoryRecord {
                            reported_at: r.reported_at,
                        })
                        .collect(),
                    rejected_records: u32::try_from(history.rejected_records)
                        .map_err(|_| crate::invalid())?,
                    limit_reached: history.limit_reached,
                    completeness: HistoryCompleteness::Unproven,
                };
                operation.run(async { Ok(result) }).await
            })
            .await
    }
}
