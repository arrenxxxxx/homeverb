//! Exact modern statistics profile from the pinned MIT community protocol.
use super::*;
use crate::statistics::{DAILY_ENERGY_MODEL, DAILY_ENERGY_SPEC, decode_history};
use crate::{DailyEnergyHistory, DailyEnergyWindow};
use chrono::Utc;

impl ModernMijiaSession {
    /// One exact historical query. This is not the reference continuous-stream route,
    /// a current observation, a permission grant or a generic statistics endpoint.
    pub async fn daily_energy_history(
        &self,
        transport: &dyn ModernReadTransport,
        entry: &ModernDirectoryEntry,
        window: DailyEnergyWindow,
        operation: &Operation,
    ) -> Result<DailyEnergyHistory> {
        operation
            .run(async {
                self.validate()?;
                if entry.subject != self.subject
                    || entry.encrypted_subject != self.encrypted_subject
                {
                    return Err(crate::protected::conflict());
                }
                if !matches!(entry.kind, Some(DirectoryKind(0 | 6 | 16 | 18)))
                    || entry.model != DAILY_ENERGY_MODEL
                    || entry.mapping.as_ref().is_none_or(|mapping| {
                        mapping.spec_type != DAILY_ENERGY_SPEC
                            || !mapping
                                .properties
                                .iter()
                                .any(|p| p.siid == 11 && p.piid == 1)
                    })
                {
                    return Err(Error::new(
                        "CAPABILITY_UNSUPPORTED",
                        "This exact historical energy profile is unavailable.",
                    ));
                }
                if !crate::bounded_text(entry.did.as_str(), 128)
                    || window.end() > Utc::now().timestamp()
                {
                    return Err(Error::invalid());
                }
                let data = PrivateString::new(
                    serde_json::to_string(&json!({
                        "did": entry.did.as_str(), "key": "11.1", "data_type": "stat_day_v3",
                        "time_start": window.start(), "time_end": window.end(), "limit": 31
                    }))
                    .map_err(|_| invalid())?,
                );
                let data = SecretBytes::new(data.as_str().as_bytes().to_vec());
                let bytes = self
                    .community_history_read(
                        transport,
                        ModernReadEndpoint::Statistics,
                        &data,
                        operation,
                    )
                    .await?;
                let history =
                    decode_history(bytes.expose(), entry.did.as_str(), window, Utc::now())?;
                operation.run(async { Ok(history) }).await
            })
            .await
    }
}
