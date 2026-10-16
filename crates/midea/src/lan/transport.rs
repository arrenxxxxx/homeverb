//! Operation-owned V3 exchanges, with separate command transmission evidence.

use super::{
    AcC0Status, DiscoveryNetwork, V3Handshake, V3Response, V3Session, decode_ac_c0_status,
    decode_appliance_protocol, decode_lan_packet, encode_ac_status_query, encode_appliance_query,
    encode_lan_packet, v3_frame_len,
};
use crate::{ApplianceFamily, CloudId};
use homeverb_core::{Clock, Operation, SecretBytes, WriteProgress};
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

async fn connect_v3(
    network: &DiscoveryNetwork,
    endpoint: SocketAddrV4,
    handshake: V3Handshake,
    operation: &Operation,
) -> Result<(TcpStream, V3Session)> {
    operation
        .run(async {
            if !network.admits(*endpoint.ip())
                || *endpoint.ip() == network.local_address()
                || endpoint.port() == 0
            {
                return Err(Error::invalid());
            }
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
            Ok((stream, handshake.finish(reply.expose())?))
        })
        .await
}

async fn exchange_session(
    stream: &mut TcpStream,
    session: &mut V3Session,
    payload: &SecretBytes,
    progress: &WriteProgress,
    operation: &Operation,
) -> Result<V3Response> {
    operation
        .run(async {
            let request = session.encode_request(payload)?;
            operation.run(async { Ok(()) }).await?;
            // Handshake traffic is not an appliance command. From this point, partial
            // writes, cancellation or a lost response cannot prove non-transmission.
            progress.mark_sent();
            stream
                .write_all(request.expose())
                .await
                .map_err(|_| failed())?;
            let response = read_frame(stream, 3, operation).await?;
            operation
                .run(async { session.decode_response(response.expose()) })
                .await
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
            if progress.phase() != Phase::NotSent {
                return Err(Error::invalid());
            }
            super::v3::padding_for(payload.expose().len())?;
            let (mut stream, mut session) =
                connect_v3(network, endpoint, handshake, operation).await?;
            exchange_session(&mut stream, &mut session, payload, progress, operation).await
        })
        .await
}

/// Query the message version, then one standard AC status on the same bounded connection.
/// The trusted caller must separately admit the protected endpoint/model and network access.
/// Returns only C0 candidates, never a Snapshot, fresh state or write outcome.
pub async fn query_ac_v3(
    network: &DiscoveryNetwork,
    endpoint: SocketAddrV4,
    handshake: V3Handshake,
    target: &CloudId,
    clock: &dyn Clock,
    sequence: u8,
    operation: &Operation,
) -> Result<AcC0Status> {
    operation
        .run(async {
            // Reject the explicit sequence before opening a socket, without assuming
            // that the initial message version is usable for a real status query.
            encode_ac_status_query(0, sequence)?;
            let appliance = encode_appliance_query(ApplianceFamily::AirConditioner)?;
            let packet = encode_lan_packet(target, &appliance, clock.now())?;
            let (mut stream, mut session) =
                connect_v3(network, endpoint, handshake, operation).await?;
            let response = exchange_session(
                &mut stream,
                &mut session,
                &packet,
                &WriteProgress::default(),
                operation,
            )
            .await?;
            let message = decode_lan_packet(target, response.payload.expose())?;
            let version = decode_appliance_protocol(ApplianceFamily::AirConditioner, &message)?;
            let query = encode_ac_status_query(version, sequence)?;
            let packet = encode_lan_packet(target, &query, clock.now())?;
            let response = exchange_session(
                &mut stream,
                &mut session,
                &packet,
                &WriteProgress::default(),
                operation,
            )
            .await?;
            let message = decode_lan_packet(target, response.payload.expose())?;
            operation
                .run(async { decode_ac_c0_status(version, &message) })
                .await
        })
        .await
}

#[cfg(test)]
mod query_tests;
#[cfg(test)]
mod tests;
