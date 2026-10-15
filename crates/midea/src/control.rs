//! Conservative cloud acknowledgement handling. No reply establishes physical completion.

use crate::{CloudId, CloudTransport, MAX_RESPONSE_BYTES, MeijuSession};
use homeverb_core::{AdapterOutcome, Operation, SecretBytes, WriteProgress};
use homeverb_domain::{Error, Phase, Result};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(untagged)]
enum Code {
    Number(i64),
    Text(String),
}

#[derive(Deserialize)]
struct Acknowledgement {
    code: Code,
}

fn reply(body: &SecretBytes) -> Option<i64> {
    if body.expose().len() > MAX_RESPONSE_BYTES {
        return None;
    }
    // A typed envelope also rejects duplicate codes. Ignore data/messages without
    // retaining private provider text or treating echoed state as verification.
    let envelope: Acknowledgement = serde_json::from_slice(body.expose()).ok()?;
    match envelope.code {
        Code::Number(code) => Some(code),
        Code::Text(code) => match code.as_str() {
            "0" => Some(0),
            "7400" => Some(7400),
            "40002" => Some(40002),
            _ => None,
        },
    }
}

fn authorization_required() -> Error {
    Error::new(
        "AUTH_REQUIRED",
        "The platform account requires user-side authorization.",
    )
}

pub(crate) async fn set_power(
    session: &MeijuSession,
    transport: &dyn CloudTransport,
    appliance: &CloudId,
    state: bool,
    progress: &WriteProgress,
    operation: &Operation,
) -> Result<AdapterOutcome> {
    operation
        .run(async {
            let request = session.ac_power_request(appliance, state)?;
            let response = transport.send_control(&request, progress, operation).await;
            let (phase, error) = match response {
                Ok(body) => match reply(&body) {
                    Some(0) => {
                        progress.mark_accepted();
                        (Phase::Accepted, None)
                    }
                    Some(7400 | 40002) => (Phase::Rejected, Some(authorization_required())),
                    // Other codes can describe downstream timeouts after dispatch. The
                    // inspected protocol provides no general proof of non-execution.
                    _ => (Phase::PossiblySent, Some(unknown())),
                },
                Err(error) if progress.phase() == Phase::NotSent => {
                    let safe = match error.code.as_str() {
                        "AUTH_REQUIRED" => authorization_required(),
                        "CAPABILITY_UNSUPPORTED" => unsupported(),
                        "CANCELLED" => Error::new(
                            "CANCELLED",
                            "The cloud control was cancelled before transmission.",
                        ),
                        "DEADLINE_EXCEEDED" => Error::new(
                            "DEADLINE_EXCEEDED",
                            "The cloud control budget expired before transmission.",
                        ),
                        _ => Error::new(
                            "MIDEA_REQUEST_FAILED",
                            "The cloud control could not be prepared.",
                        ),
                    };
                    (Phase::NotSent, Some(safe))
                }
                Err(error) if error.code == "AUTH_REQUIRED" => {
                    (Phase::Rejected, Some(authorization_required()))
                }
                Err(_) => (Phase::PossiblySent, Some(unknown())),
            };
            Ok(AdapterOutcome {
                phase,
                error,
                completion: None,
            })
        })
        .await
}

pub(crate) fn unsupported() -> Error {
    Error::new(
        "CAPABILITY_UNSUPPORTED",
        "This model or host has no admitted cloud power control.",
    )
}

fn unknown() -> Error {
    Error::new(
        "WRITE_OUTCOME_UNKNOWN",
        "The cloud control may have executed; inspect state without replaying it.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acknowledgement_requires_an_unambiguous_protocol_code() {
        for (body, code) in [
            (r#"{"code":0}"#, Some(0)),
            (r#"{"code":"0","data":null}"#, Some(0)),
            (r#"{"code":7400,"msg":"PRIVATE"}"#, Some(7400)),
            (r#"{"code":"40002"}"#, Some(40002)),
            (r#"{"code":1306}"#, Some(1306)),
            (r#"{"code":0,"code":7400}"#, None),
            (r#"{"code":true}"#, None),
            (r#"{"code":0.0}"#, None),
            (r#"{"code":"+0"}"#, None),
            (r#"{"data":{"power":"off"}}"#, None),
            (r#"{"code":0} trailing"#, None),
        ] {
            assert_eq!(reply(&SecretBytes::new(body.as_bytes().to_vec())), code);
        }
        assert_eq!(
            reply(&SecretBytes::new(vec![b' '; MAX_RESPONSE_BYTES + 1])),
            None
        );
    }
}
