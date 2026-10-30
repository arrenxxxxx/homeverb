//! Exact mapped scalar writes under the reference continuous-stream profile.
use super::*;
use crate::{ModernControlRequest, ModernControlTransport, PropertyWrite};
use homeverb_core::{AdapterOutcome, WriteProgress};
use homeverb_domain::Phase;

#[derive(Deserialize)]
struct AckEnvelope {
    code: i64,
    result: Vec<Ack>,
}
#[derive(Deserialize)]
struct Ack {
    did: PrivateString,
    siid: u32,
    piid: u32,
    code: i64,
}
fn unsupported() -> Error {
    Error::new(
        "CAPABILITY_UNSUPPORTED",
        "This Mijia device or action has no admitted mapping.",
    )
}

impl ModernMijiaSession {
    /// A low-level tracked primitive, not Core authorization. The adapter must
    /// independently validate grants, protected identity and trusted approval.
    pub async fn set_property(
        &self,
        transport: &dyn ModernControlTransport,
        entry: &ModernDirectoryEntry,
        write: &PropertyWrite,
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
                if !matches!(entry.kind, Some(DirectoryKind(0 | 6 | 16 | 18))) {
                    return Err(unsupported());
                }
                let mapping = entry.mapping.as_ref().ok_or_else(unsupported)?;
                let field = mapping
                    .properties
                    .iter()
                    .find(|p| p.siid == write.query.siid && p.piid == write.query.piid)
                    .ok_or_else(unsupported)?;
                let option = mapping.control_option(field).ok_or_else(unsupported)?;
                option
                    .definition
                    .validate_arguments(&json!({field.property.clone(): write.value}))?;
                if progress.phase() != Phase::NotSent
                    || !crate::bounded_text(entry.did.as_str(), 128)
                {
                    return Err(Error::invalid());
                }
                let data = PrivateString::new(
                    serde_json::to_string(&json!({"params": [{
                        "did": entry.did.as_str(),
                        "siid": write.query.siid,
                        "piid": write.query.piid,
                        "value": write.value,
                    }]}))
                    .map_err(|_| invalid())?,
                );
                let (body, key) = codec::encode_reference(
                    "/miotspec/prop/set",
                    data.as_str(),
                    self.security.as_str(),
                    &codec::nonce()?,
                )?;
                let request = ModernControlRequest {
                    endpoint: crate::ModernControlEndpoint::SetProperty,
                    body,
                    cookie: PrivateString::new(format!(
                        "cUserId={}; serviceToken={}; yetAnotherServiceToken={}; countryCode=CN",
                        self.encrypted_subject.as_str(),
                        self.token.as_str(),
                        self.token.as_str(),
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
                let reply: AckEnvelope = crate::decode(bytes.expose())?;
                operation.run(async { Ok(()) }).await?;
                if reply.code != 0
                    || reply.result.len() != 1
                    || progress.phase() != Phase::PossiblySent
                {
                    return Err(invalid());
                }
                let ack = &reply.result[0];
                if ack.did != entry.did
                    || ack.siid != write.query.siid
                    || ack.piid != write.query.piid
                {
                    return Err(invalid());
                }
                if ack.code != 0 {
                    return Ok(AdapterOutcome {
                        phase: Phase::Rejected,
                        error: Some(Error::new(
                            "MIJIA_REQUEST_REJECTED",
                            "Mijia rejected the selected property write.",
                        )),
                        completion: None,
                    });
                }
                progress.mark_accepted();
                Ok(AdapterOutcome {
                    phase: Phase::Accepted,
                    error: None,
                    completion: None,
                })
            })
            .await
    }
}
