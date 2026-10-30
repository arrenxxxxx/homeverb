//! Explicit reported numeric property logs through the community history profile.
use super::*;
use crate::{NumericPropertyHistory, PropertyHistoryWindow};
use chrono::Utc;

impl ModernMijiaSession {
    /// Read one pinned numeric root property; never interpret it as a daily statistic.
    pub async fn numeric_property_history(
        &self,
        transport: &dyn ModernReadTransport,
        entry: &ModernDirectoryEntry,
        query: &PropertyQuery,
        window: PropertyHistoryWindow,
        operation: &Operation,
    ) -> Result<NumericPropertyHistory> {
        operation
            .run(async {
                self.validate()?;
                if entry.subject != self.subject
                    || entry.encrypted_subject != self.encrypted_subject
                {
                    return Err(crate::protected::conflict());
                }
                if !matches!(entry.kind, Some(DirectoryKind(0 | 6 | 16 | 18))) {
                    return Err(Error::new(
                        "CAPABILITY_UNSUPPORTED",
                        "This logical entry cannot supply property history.",
                    ));
                }
                let mapping = entry.mapping.as_ref().ok_or_else(invalid)?;
                mapping.validate_saved()?;
                let field =
                    crate::property_history::field(&entry.model, &mapping.spec_type, query)?;
                if !mapping
                    .properties
                    .iter()
                    .any(|p| p.siid == field.siid && p.piid == field.piid)
                {
                    return Err(crate::protected::conflict());
                }
                let data = crate::property_history::payload(entry.did.as_str(), &field, window)?;
                let bytes = self
                    .community_history_read(
                        transport,
                        ModernReadEndpoint::PropertyHistory,
                        &data,
                        operation,
                    )
                    .await?;
                let result = crate::property_history::decode_history(
                    bytes.expose(),
                    entry.did.as_str(),
                    &field,
                    window,
                    Utc::now(),
                )?;
                operation.run(async { Ok(result) }).await
            })
            .await
    }
}
