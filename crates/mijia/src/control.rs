//! One-property MIoT wire primitive. Account/identity/domain admission belongs to the adapter.

use crate::{
    CloudDevice, CloudEndpoint, CloudTransport, MijiaSession, PrivateString, PropertyQuery, codec,
    decode, invalid,
};
use homeverb_core::{AdapterOutcome, Operation, SecretBytes, WriteProgress};
use homeverb_domain::{Error, Phase, Result};
use serde::Deserialize;
use serde_json::{Value, json};

/// Exact wire property and scalar target, not a permission or host request.
pub struct PropertyWrite {
    pub query: PropertyQuery,
    pub value: Value,
}

#[derive(Deserialize)]
struct Envelope {
    code: i64,
    result: Vec<Acknowledgement>,
}

#[derive(Deserialize)]
struct Acknowledgement {
    did: PrivateString,
    siid: u32,
    piid: u32,
    code: i64,
}

impl MijiaSession {
    /// Low-level protocol primitive; this does not admit a Core action or grant authority.
    /// Callers must independently validate the protected binding, writable property and
    /// exact value domain. The production Core adapter still denies all writes.
    /// A correlated code-zero reply means accepted, never physically verified.
    pub async fn set_property(
        &self,
        transport: &dyn CloudTransport,
        device: &CloudDevice,
        write: &PropertyWrite,
        progress: &WriteProgress,
        operation: &Operation,
    ) -> Result<AdapterOutcome> {
        operation
            .run(async {
                let PropertyWrite { query, value } = write;
                if query.siid == 0
                    || query.piid == 0
                    || !crate::bounded_text(device.did.as_str(), 128)
                    || !(value.is_boolean() || value.is_number())
                    || progress.phase() != Phase::NotSent
                {
                    return Err(Error::invalid());
                }
                let data = SecretBytes::new(serde_json::to_vec(&json!({"params": [{
                "did":device.did.as_str(), "siid":query.siid, "piid":query.piid, "value":value
            }]})).map_err(|_| Error::invalid())?);
                let (request, key) = self.prepare_request(CloudEndpoint::SetProperty, &data)?;
                let response = transport
                    .send_control(&request, progress, operation)
                    .await?;
                // Neither HTTP status nor a plaintext/global code correlates this property.
                let decoded = codec::decrypt(key.expose(), response.body.expose())?;
                let reply: Envelope = decode(decoded.expose())?;
                if reply.code != 0 || reply.result.len() != 1 {
                    return Err(invalid());
                }
                let ack = &reply.result[0];
                if ack.did != device.did || ack.siid != query.siid || ack.piid != query.piid {
                    return Err(invalid());
                }
                if progress.phase() != Phase::PossiblySent {
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

#[cfg(test)]
mod tests;
