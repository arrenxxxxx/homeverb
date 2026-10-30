//! Exact no-input actions on the protected modern reference profile.
use super::*;
use crate::{
    ActionQuery, ModernControlEndpoint, ModernControlRequest, ModernControlTransport, action,
};
use homeverb_core::{AdapterOutcome, WriteProgress};

impl ModernMijiaSession {
    /// One tracked non-idempotent attempt, not a Core grant or host command.
    pub async fn invoke_action(
        &self,
        transport: &dyn ModernControlTransport,
        entry: &ModernDirectoryEntry,
        query: &ActionQuery,
        progress: &WriteProgress,
        operation: &Operation,
    ) -> Result<AdapterOutcome> {
        operation
            .run(async {
                self.validate()?;
                if entry.subject != self.subject
                    || entry.encrypted_subject != self.encrypted_subject
                {
                    return Err(crate::protected::conflict());
                }
                if entry.parent.is_some()
                    || !matches!(entry.kind, Some(DirectoryKind(0 | 6 | 16 | 18)))
                {
                    return Err(action::unsupported());
                }
                let mapping = entry.mapping.as_ref().ok_or_else(action::unsupported)?;
                if mapping.model != entry.model
                    || entry
                        .spec_type
                        .as_ref()
                        .is_some_and(|s| s != &mapping.spec_type)
                {
                    return Err(action::unsupported());
                }
                action::admit(&entry.model, &mapping.spec_type, query)?;
                let data = action::payload(entry.did.as_str(), query, progress)?;
                let endpoint = ModernControlEndpoint::Action;
                let (body, key) = codec::encode_reference(
                    endpoint.path(),
                    std::str::from_utf8(data.expose()).map_err(|_| invalid())?,
                    self.security.as_str(),
                    &codec::nonce()?,
                )?;
                let request = ModernControlRequest {
                    endpoint,
                    body,
                    cookie: PrivateString::new(format!(
                        "cUserId={}; serviceToken={}; yetAnotherServiceToken={}; countryCode=CN",
                        self.encrypted_subject.as_str(),
                        self.token.as_str(),
                        self.token.as_str()
                    )),
                };
                let response = transport
                    .send_modern_control(&request, progress, operation)
                    .await?;
                let bytes = decode_reference_response(
                    ModernReadResponse {
                        body: response.body,
                        encoding: response.encoding,
                    },
                    key.expose(),
                    crate::MAX_RESPONSE_BYTES,
                )?;
                action::acknowledge(
                    bytes.expose(),
                    entry.did.as_str(),
                    query,
                    progress,
                    operation,
                )
                .await
            })
            .await
    }
}
