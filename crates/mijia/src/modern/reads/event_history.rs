//! Exact historical event occurrence times on the separate community read profile.
use super::*;
use crate::{EventHistoryQuery, EventHistoryWindow, ReportedEventHistory};
use chrono::Utc;
impl ModernMijiaSession {
    pub async fn reported_event_history(
        &self,
        transport: &dyn ModernReadTransport,
        entry: &ModernDirectoryEntry,
        query: &EventHistoryQuery,
        window: EventHistoryWindow,
        operation: &Operation,
    ) -> Result<ReportedEventHistory> {
        operation
            .run(async {
                self.validate()?;
                if entry.subject != self.subject
                    || entry.encrypted_subject != self.encrypted_subject
                {
                    return Err(crate::protected::conflict());
                }
                if !matches!(entry.kind, Some(DirectoryKind(0 | 6 | 16 | 18))) {
                    return Err(crate::event_history::unsupported());
                }
                let mapping = entry
                    .mapping
                    .as_ref()
                    .ok_or_else(crate::event_history::unsupported)?;
                mapping.validate_saved()?;
                if mapping.model != entry.model {
                    return Err(crate::protected::conflict());
                }
                crate::event_history::admit(&entry.model, &mapping.spec_type, query)?;
                let data = crate::event_history::payload(entry.did.as_str(), query, window)?;
                let bytes = self
                    .community_history_read(
                        transport,
                        ModernReadEndpoint::EventHistory,
                        &data,
                        operation,
                    )
                    .await?;
                let result = crate::event_history::decode_history(
                    bytes.expose(),
                    entry.did.as_str(),
                    query,
                    window,
                    Utc::now(),
                )?;
                operation.run(async { Ok(result) }).await
            })
            .await
    }
}
