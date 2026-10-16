//! One operation-owned V3 exchange, with separate command transmission evidence.

use super::{DiscoveryNetwork, V3Handshake, V3Response, v3_frame_len};
use homeverb_core::{Operation, SecretBytes, WriteProgress};
use homeverb_domain::{Error, Phase, Result};
use std::net::SocketAddrV4;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpSocket, TcpStream},
};
use zeroize::Zeroizing;

fn failed() -> Error {
    Error::new(
        "MIDEA_REQUEST_FAILED",
        "The V3 LAN exchange could not be completed.",
    )
}

fn invalid() -> Error {
    Error::new(
        "MIDEA_RESPONSE_INVALID",
        "The V3 LAN frame is malformed or unsupported.",
    )
}

async fn read_frame(
    stream: &mut TcpStream,
    kind: u8,
    operation: &Operation,
) -> Result<SecretBytes> {
    operation
        .run(async {
            let mut header = [0; 6];
            stream.read_exact(&mut header).await.map_err(|_| failed())?;
            let size = v3_frame_len(&header)?.ok_or_else(invalid)?;
            if header[5] & 15 != kind || (kind == 1 && size != 13 && size != 72) {
                return Err(invalid());
            }
            let mut bytes = Zeroizing::new(vec![0; size]);
            bytes[..6].copy_from_slice(&header);
            stream
                .read_exact(&mut bytes[6..])
                .await
                .map_err(|_| failed())?;
            Ok(SecretBytes::new(std::mem::take(&mut *bytes)))
        })
        .await
}

/// Low-level native transport, not device identity, capability or write authorization.
/// The caller must admit the protected device/endpoint and payload before invoking it.
/// A decoded response remains unverified; this function never marks acceptance.
pub async fn exchange_v3(
    network: &DiscoveryNetwork,
    endpoint: SocketAddrV4,
    handshake: V3Handshake,
    payload: &SecretBytes,
    progress: &WriteProgress,
    operation: &Operation,
) -> Result<V3Response> {
    operation
        .run(async {
            if !network.admits(*endpoint.ip())
                || *endpoint.ip() == network.local_address()
                || endpoint.port() == 0
                || progress.phase() != Phase::NotSent
            {
                return Err(Error::invalid());
            }
            super::v3::padding_for(payload.expose().len())?;
            let socket = TcpSocket::new_v4().map_err(|_| failed())?;
            socket
                .bind(SocketAddrV4::new(network.local_address(), 0).into())
                .map_err(|_| failed())?;
            let mut stream = socket
                .connect(endpoint.into())
                .await
                .map_err(|_| failed())?;
            operation
                .run(async {
                    stream
                        .write_all(handshake.request().expose())
                        .await
                        .map_err(|_| failed())
                })
                .await?;
            let reply = read_frame(&mut stream, 1, operation).await?;
            let mut session = handshake.finish(reply.expose())?;
            let request = session.encode_request(payload)?;
            operation.run(async { Ok(()) }).await?;
            // Handshake traffic is not an appliance command. From this point, partial
            // writes, cancellation or a lost response cannot prove non-transmission.
            progress.mark_sent();
            stream
                .write_all(request.expose())
                .await
                .map_err(|_| failed())?;
            let response = read_frame(&mut stream, 3, operation).await?;
            operation
                .run(async { session.decode_response(response.expose()) })
                .await
        })
        .await
}

#[cfg(test)]
mod tests;
