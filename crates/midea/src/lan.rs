//! Bounded LAN discovery decoding, not device authentication or a registered route.
//! Derived from the MIT source recorded in third-party/midea-local/NOTICE.md.

use crate::{ApplianceFamily, CloudId};
use aes::{
    Aes128,
    cipher::{BlockCipherDecrypt, KeyInit},
};
use homeverb_domain::{Error, Result};
use md5::{Digest, Md5};
use std::{
    fmt,
    net::{Ipv4Addr, SocketAddrV4},
};
use zeroize::Zeroizing;

const MAX_DATAGRAM: usize = 512;
// Public wire compatibility material; neither constant establishes peer identity.
const KEY: [u8; 16] = [
    0x6a, 0x92, 0xef, 0x40, 0x6b, 0xad, 0x2f, 0x03, 0x59, 0xba, 0xad, 0x99, 0x41, 0x71, 0xea, 0x6d,
];
const SALT: &[u8] = b"xhdiwjnchekd4d512chdjx5d8e4c394D2D7S";

/// Fixed protocol probe, with no account or appliance credentials.
pub const DISCOVERY_REQUEST: [u8; 72] = [
    0x5a, 0x5a, 0x01, 0x11, 0x48, 0x00, 0x92, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x7f, 0x75, 0xbd, 0x6b, 0x3e, 0x4f, 0x8b, 0x76,
    0x2e, 0x84, 0x9c, 0x6e, 0x57, 0x8d, 0x65, 0x90, 0x03, 0x6e, 0x9d, 0x43, 0x42, 0xa5, 0x0f, 0x1f,
    0x56, 0x9e, 0xb8, 0xec, 0x91, 0x8e, 0x92, 0xe5,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanProtocol {
    V2,
    V3,
}

/// Unauthenticated hints for later reconciliation, never a Device or permission.
/// ```compile_fail
/// fn export(candidate: &homeverb_midea::lan::DiscoveryCandidate) {
///     let _ = serde_json::to_string(candidate);
/// }
/// ```
pub struct DiscoveryCandidate {
    pub appliance_id: CloudId,
    pub endpoint: SocketAddrV4,
    pub protocol: LanProtocol,
    pub family: ApplianceFamily,
}

impl fmt::Debug for DiscoveryCandidate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("DiscoveryCandidate([REDACTED])")
    }
}

fn invalid() -> Error {
    Error::new(
        "MIDEA_RESPONSE_INVALID",
        "The LAN discovery response is malformed or unsupported.",
    )
}

/// Decode one complete UDP datagram without I/O, persistence or device admission.
/// The caller supplies the actual peer address; advertised addresses are ignored.
pub fn decode_discovery(datagram: &[u8], source: Ipv4Addr) -> Result<DiscoveryCandidate> {
    if !(104..=MAX_DATAGRAM).contains(&datagram.len())
        || source.is_unspecified()
        || source.is_multicast()
        || source.is_broadcast()
    {
        return Err(invalid());
    }
    let (packet, protocol) = match &datagram[..2] {
        [0x5a, 0x5a] => (datagram, LanProtocol::V2),
        [0x83, 0x70] => (&datagram[8..datagram.len() - 16], LanProtocol::V3),
        _ => return Err(invalid()),
    };
    if packet.len() < 104
        || packet[..2] != [0x5a, 0x5a]
        || usize::from(u16::from_le_bytes([packet[4], packet[5]])) != packet.len()
    {
        return Err(invalid());
    }
    let end = packet.len() - 16;
    if !(end - 40).is_multiple_of(16) {
        return Err(invalid());
    }
    let mut checksum = Md5::new();
    checksum.update(&packet[..end]);
    checksum.update(SALT);
    if packet[end..] != checksum.finalize()[..] {
        return Err(invalid());
    }
    // Discovery uses six ID bytes; the following reserved bytes are not identity.
    let mut id_bytes = [0; 8];
    id_bytes[..6].copy_from_slice(&packet[20..26]);
    let id = u64::from_le_bytes(id_bytes);
    if id == 0 {
        return Err(invalid());
    }
    let cipher = Aes128::new((&KEY).into());
    let mut body = Zeroizing::new(packet[40..end].to_vec());
    for block in body.chunks_exact_mut(16) {
        cipher.decrypt_block(block.try_into().unwrap());
    }
    let padding = usize::from(*body.last().unwrap());
    if !(1..=16).contains(&padding)
        || !body[body.len() - padding..]
            .iter()
            .all(|byte| usize::from(*byte) == padding)
    {
        return Err(invalid());
    }
    let body = &body[..body.len() - padding];
    if body.len() < 41 {
        return Err(invalid());
    }
    let port =
        u16::try_from(u32::from_le_bytes(body[4..8].try_into().unwrap())).map_err(|_| invalid())?;
    let ssid_len = usize::from(body[40]);
    if port == 0 || ssid_len == 0 || ssid_len > 32 || 41 + ssid_len > body.len() {
        return Err(invalid());
    }
    let ssid = &body[41..41 + ssid_len];
    if !ssid.is_ascii() {
        return Err(invalid());
    }
    let family = std::str::from_utf8(ssid)
        .map_err(|_| invalid())?
        .split('_')
        .nth(1)
        .ok_or_else(invalid)?;
    if family.is_empty() || family.len() > 4 || !family.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(invalid());
    }
    let family = match u16::from_str_radix(family, 16).map_err(|_| invalid())? {
        0xac => ApplianceFamily::AirConditioner,
        0xe1 => ApplianceFamily::Dishwasher,
        0xca => ApplianceFamily::Refrigerator,
        other => ApplianceFamily::Other(other),
    };
    Ok(DiscoveryCandidate {
        appliance_id: CloudId(id.to_string()),
        endpoint: SocketAddrV4::new(source, port),
        protocol,
        family,
    })
}
