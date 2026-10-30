//! Exact no-argument MIoT actions. This primitive does not grant Core authority.
use crate::{
    CloudDevice, CloudEndpoint, CloudTransport, MijiaSession, PrivateString, Region, codec, invalid,
};
use homeverb_core::{AdapterOutcome, Operation, SecretBytes, WriteProgress};
use homeverb_domain::{Error, Phase, Result};
use serde::{Deserialize, de::IgnoredAny};
use serde_json::json;

/// A selector into the pinned no-input/no-output catalog, never arbitrary RPC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionQuery {
    pub siid: u32,
    pub aiid: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Profile {
    model: String,
    spec_type: String,
    service_type: String,
    action_type: String,
    siid: u32,
    aiid: u32,
    #[serde(rename = "in")]
    input: Vec<u32>,
    out: Vec<u32>,
}

pub(crate) fn unsupported() -> Error {
    Error::new(
        "CAPABILITY_UNSUPPORTED",
        "This Mijia action has no reviewed no-argument mapping.",
    )
}
pub(crate) fn admit(model: &str, specification: &str, query: &ActionQuery) -> Result<()> {
    let profiles: Vec<Profile> = crate::decode(include_bytes!("action_profiles.json"))?;
    let mut matches = profiles.iter().filter(|p| {
        p.model == model
            && p.spec_type == specification
            && p.siid == query.siid
            && p.aiid == query.aiid
    });
    let profile = matches.next().ok_or_else(unsupported)?;
    if matches.next().is_some()
        || !profile.input.is_empty()
        || !profile.out.is_empty()
        || profile.siid == 0
        || profile.aiid == 0
    {
        return Err(unsupported());
    }
    crate::spec::validate_device_type(&profile.spec_type)?;
    crate::spec::urn(&profile.service_type, "service")?;
    crate::spec::urn(&profile.action_type, "action")?;
    Ok(())
}
pub(crate) fn payload(
    did: &str,
    query: &ActionQuery,
    progress: &WriteProgress,
) -> Result<SecretBytes> {
    if !crate::bounded_text(did, 128) || progress.phase() != Phase::NotSent {
        return Err(Error::invalid());
    }
    Ok(SecretBytes::new(
        serde_json::to_vec(&json!({"params":{
            "did":did,"siid":query.siid,"aiid":query.aiid,"in":[]
        }}))
        .map_err(|_| invalid())?,
    ))
}
#[derive(Deserialize)]
struct Envelope {
    code: i64,
    result: Option<Acknowledgement>,
}
#[derive(Deserialize)]
struct Acknowledgement {
    did: PrivateString,
    siid: u32,
    aiid: u32,
    code: i64,
    #[serde(default)]
    out: Vec<IgnoredAny>,
}
fn pre_execution_rejection(code: i64) -> bool {
    matches!(code, -4003 | -4006 | -4007)
        || ((-709_999_999..=-700_000_000).contains(&code)
            && matches!((-code) % 1000, 5 | 8 | 25 | 35))
}
pub(crate) async fn acknowledge(
    bytes: &[u8],
    did: &str,
    query: &ActionQuery,
    progress: &WriteProgress,
    operation: &Operation,
) -> Result<AdapterOutcome> {
    let envelope: Envelope = crate::decode(bytes)?;
    operation
        .run(async {
            if progress.phase() != Phase::PossiblySent {
                return Err(invalid());
            }
            if matches!(
                envelope.code,
                3 | -3 | -6 | -10001 | -10002 | -10003 | -704010000
            ) {
                return Err(crate::auth_required());
            }
            if envelope.code != 0 {
                return Err(invalid());
            }
            let ack = envelope.result.ok_or_else(invalid)?;
            if ack.did.as_str() != did
                || ack.siid != query.siid
                || ack.aiid != query.aiid
                || !ack.out.is_empty()
            {
                return Err(invalid());
            }
            if ack.code == 0 {
                progress.mark_accepted();
                Ok(AdapterOutcome {
                    phase: Phase::Accepted,
                    error: None,
                    completion: None,
                })
            } else if pre_execution_rejection(ack.code) {
                Ok(AdapterOutcome {
                    phase: Phase::Rejected,
                    error: Some(Error::new(
                        "MIJIA_REQUEST_REJECTED",
                        "Mijia rejected the selected action before execution.",
                    )),
                    completion: None,
                })
            } else {
                // Execution errors and timeouts do not prove that a non-idempotent action did nothing.
                Err(crate::failed())
            }
        })
        .await
}
impl MijiaSession {
    /// One non-idempotent action attempt; requires independent protected membership
    /// and authorization by its caller. No Core/native/App action is added here.
    pub async fn invoke_action(
        &self,
        transport: &dyn CloudTransport,
        device: &CloudDevice,
        query: &ActionQuery,
        progress: &WriteProgress,
        operation: &Operation,
    ) -> Result<AdapterOutcome> {
        operation
            .run(async {
                if self.region() != Region::Cn {
                    return Err(unsupported());
                }
                admit(
                    &device.model,
                    device.spec_type.as_deref().ok_or_else(unsupported)?,
                    query,
                )?;
                let data = payload(device.did.as_str(), query, progress)?;
                let (request, key) = self.prepare_request(CloudEndpoint::Action, &data)?;
                let response = transport
                    .send_control(&request, progress, operation)
                    .await?;
                if response.body.expose().len() > crate::MAX_RESPONSE_BYTES {
                    return Err(invalid());
                }
                let bytes = codec::decrypt(key.expose(), response.body.expose())?;
                acknowledge(
                    bytes.expose(),
                    device.did.as_str(),
                    query,
                    progress,
                    operation,
                )
                .await
            })
            .await
    }
}
#[cfg(test)]
pub(crate) mod tests;
