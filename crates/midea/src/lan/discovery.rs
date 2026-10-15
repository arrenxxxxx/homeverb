//! Explicit, operation-owned UDP discovery within one selected local network.

use super::{DISCOVERY_REQUEST, DiscoveryCandidate, MAX_DATAGRAM, decode_discovery};
use homeverb_core::Operation;
use homeverb_domain::{Error, Result};
use std::{fmt, net::Ipv4Addr, net::SocketAddrV4, time::Duration};
use tokio::{net::UdpSocket, time::Instant};
use zeroize::Zeroizing;

const WINDOW: Duration = Duration::from_secs(5);
const MAX_CANDIDATES: usize = 64;
const MAX_RESPONSES: usize = 256;

/// A trusted host must select an assigned interface and obtain network consent.
/// Construction validates scope, not interface ownership or user authorization.
pub struct DiscoveryNetwork {
    address: Ipv4Addr,
    mask: u32,
}

impl fmt::Debug for DiscoveryNetwork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DiscoveryNetwork([REDACTED])")
    }
}

impl DiscoveryNetwork {
    /// Restrict discovery to an RFC 1918 or IPv4 link-local subnet, with host space.
    pub fn new(address: Ipv4Addr, prefix: u8) -> Result<Self> {
        if !(8..=30).contains(&prefix) {
            return Err(Error::invalid());
        }
        let mask = u32::MAX << (32 - prefix);
        let first = Ipv4Addr::from(u32::from(address) & mask);
        let last = Ipv4Addr::from(u32::from(first) | !mask);
        let local = |ip: Ipv4Addr| ip.is_private() || ip.is_link_local();
        if !local(first) || !local(last) || address == first || address == last {
            return Err(Error::invalid());
        }
        Ok(Self { address, mask })
    }

    fn broadcast(&self) -> Ipv4Addr {
        Ipv4Addr::from(u32::from(self.address) | !self.mask)
    }

    pub(super) fn local_address(&self) -> Ipv4Addr {
        self.address
    }

    #[cfg(test)]
    pub(super) fn loopback() -> Self {
        Self {
            address: Ipv4Addr::LOCALHOST,
            mask: 0xffff_ff00,
        }
    }

    pub(super) fn admits(&self, source: Ipv4Addr) -> bool {
        let network = u32::from(self.address) & self.mask;
        u32::from(source) & self.mask == network
            && u32::from(source) != network
            && source != self.broadcast()
    }
}

/// Unauthenticated observations, never a complete account inventory or device list.
/// ```compile_fail
/// fn export(report: &homeverb_midea::lan::DiscoveryReport) {
///     let _ = serde_json::to_string(report);
/// }
/// ```
#[derive(Debug)]
pub struct DiscoveryReport {
    pub candidates: Vec<DiscoveryCandidate>,
    pub received_datagrams: usize,
    pub limit_reached: bool,
}

fn failed() -> Error {
    Error::new(
        "MIDEA_REQUEST_FAILED",
        "The LAN discovery operation could not be completed.",
    )
}

/// Send one probe per known port on the selected network, with no retry or worker.
/// Call only for an explicitly authorized foreground operation, never on startup.
pub async fn discover(
    network: &DiscoveryNetwork,
    operation: &Operation,
) -> Result<DiscoveryReport> {
    scan(
        network,
        [6445, 20086].map(|port| SocketAddrV4::new(network.broadcast(), port)),
        operation,
    )
    .await
}

async fn scan(
    network: &DiscoveryNetwork,
    targets: [SocketAddrV4; 2],
    operation: &Operation,
) -> Result<DiscoveryReport> {
    operation
        .run(async {
            let end = Instant::now() + WINDOW;
            let setup = Operation {
                cancellation: operation.cancellation.clone(),
                deadline: operation.deadline.min(end),
            };
            let socket = setup
                .run(async {
                    let socket = UdpSocket::bind(SocketAddrV4::new(network.address, 0))
                        .await
                        .map_err(|_| failed())?;
                    socket.set_broadcast(true).map_err(|_| failed())?;
                    socket.set_ttl(1).map_err(|_| failed())?;
                    for target in targets {
                        setup
                            .run(async {
                                let sent = socket
                                    .send_to(&DISCOVERY_REQUEST, target)
                                    .await
                                    .map_err(|_| failed())?;
                                if sent != DISCOVERY_REQUEST.len() {
                                    return Err(failed());
                                }
                                Ok(())
                            })
                            .await?;
                    }
                    Ok(socket)
                })
                .await?;
            let mut report = DiscoveryReport {
                candidates: Vec::new(),
                received_datagrams: 0,
                limit_reached: false,
            };
            // The extra byte rejects truncation of an oversized UDP datagram.
            let mut buffer = Zeroizing::new(vec![0; MAX_DATAGRAM + 1]);
            loop {
                operation.run(async { Ok(()) }).await?;
                if Instant::now() >= end {
                    return Ok(report);
                }
                let (size, source) = tokio::select! {
                    biased;
                    _ = tokio::time::sleep_until(end) => return Ok(report),
                    received = socket.recv_from(&mut buffer) => received.map_err(|_| failed())?,
                };
                report.received_datagrams += 1;
                if let std::net::SocketAddr::V4(source) = source
                    && network.admits(*source.ip())
                    && let Ok(candidate) = decode_discovery(&buffer[..size], *source.ip())
                    && !report.candidates.iter().any(|existing| {
                        existing.appliance_id == candidate.appliance_id
                            && existing.endpoint == candidate.endpoint
                            && existing.protocol == candidate.protocol
                            && existing.family == candidate.family
                    })
                {
                    report.candidates.push(candidate);
                }
                if report.candidates.len() == MAX_CANDIDATES
                    || report.received_datagrams == MAX_RESPONSES
                {
                    report.limit_reached = true;
                    return Ok(report);
                }
            }
        })
        .await
}

#[cfg(test)]
mod tests;
