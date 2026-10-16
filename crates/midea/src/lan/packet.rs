//! Ordinary LAN envelope compatibility, without model semantics or peer authentication.

use super::{KEY, SALT};
use crate::CloudId;
use aes::{
    Aes128,
    cipher::{BlockCipherDecrypt, BlockCipherEncrypt, KeyInit},
};
use chrono::{DateTime, Datelike, Timelike, Utc};
use homeverb_core::SecretBytes;
use homeverb_domain::{Error, Result};
use md5::{Digest, Md5};
use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

// Largest 56 + N*16 envelope that also fits the bounded V3 transport frame.
const MAX_PACKET: usize = 4040;
const MAX_PLAINTEXT: usize = MAX_PACKET - 56 - 1;

fn invalid() -> Error {
    Error::new(
        "MIDEA_RESPONSE_INVALID",
        "The LAN packet envelope is malformed or unsupported.",
    )
}

fn appliance_id(target: &CloudId) -> Result<u64> {
    let id: u64 = target.expose().parse().map_err(|_| Error::invalid())?;
    if id == 0 || id >= 1 << 48 {
        return Err(Error::invalid());
    }
    Ok(id)
}

/// Report a bounded ordinary envelope length; this is not packet validity.
pub fn lan_packet_len(prefix: &[u8]) -> Result<Option<usize>> {
    if prefix.len() < 6 {
        return Ok(None);
    }
    let size = usize::from(u16::from_le_bytes([prefix[4], prefix[5]]));
    if prefix[..4] != [0x5a, 0x5a, 1, 0x11]
        || !(72..=MAX_PACKET).contains(&size)
        || !(size - 56).is_multiple_of(16)
    {
        return Err(invalid());
    }
    Ok(Some(size))
}

/// Encode one opaque inner message for an explicitly supplied target and UTC time.
/// The trusted caller must validate the model, permissions and message before sending.
pub fn encode_lan_packet(
    target: &CloudId,
    message: &SecretBytes,
    timestamp: DateTime<Utc>,
) -> Result<SecretBytes> {
    let id = appliance_id(target)?;
    let plain = message.expose();
    if plain.is_empty()
        || plain.len() > MAX_PLAINTEXT
        || !(1..=9999).contains(&timestamp.year())
        || timestamp.nanosecond() >= 1_000_000_000
    {
        return Err(Error::invalid());
    }
    let padding = 16 - plain.len() % 16;
    let size = 56 + plain.len() + padding;
    let mut packet = Zeroizing::new(vec![0; size]);
    packet[..4].copy_from_slice(&[0x5a, 0x5a, 1, 0x11]);
    packet[4..6].copy_from_slice(&(size as u16).to_le_bytes());
    packet[6] = 0x20;
    packet[12..20].copy_from_slice(&[
        (timestamp.nanosecond() / 10_000_000) as u8,
        timestamp.second() as u8,
        timestamp.minute() as u8,
        timestamp.hour() as u8,
        timestamp.day() as u8,
        timestamp.month() as u8,
        (timestamp.year() % 100) as u8,
        (timestamp.year() / 100) as u8,
    ]);
    packet[20..28].copy_from_slice(&id.to_le_bytes());
    packet[40..40 + plain.len()].copy_from_slice(plain);
    packet[40 + plain.len()..size - 16].fill(padding as u8);
    let cipher = Aes128::new((&KEY).into());
    for block in packet[40..size - 16].chunks_exact_mut(16) {
        cipher.encrypt_block(block.try_into().unwrap());
    }
    let mut checksum = Md5::new();
    checksum.update(&packet[..size - 16]);
    checksum.update(SALT);
    packet[size - 16..].copy_from_slice(&checksum.finalize());
    Ok(SecretBytes::new(std::mem::take(&mut *packet)))
}

/// Decode exactly one complete envelope for the expected target, never a write outcome.
/// Matching the forgeable ID/checksum does not authenticate the sending endpoint.
pub fn decode_lan_packet(target: &CloudId, packet: &[u8]) -> Result<SecretBytes> {
    let expected = appliance_id(target)?;
    if lan_packet_len(packet)? != Some(packet.len()) {
        return Err(invalid());
    }
    let end = packet.len() - 16;
    let mut checksum = Md5::new();
    checksum.update(&packet[..end]);
    checksum.update(SALT);
    if !bool::from(checksum.finalize().as_slice().ct_eq(&packet[end..])) {
        return Err(invalid());
    }
    let mut id = [0; 8];
    id[..6].copy_from_slice(&packet[20..26]);
    if u64::from_le_bytes(id) != expected {
        return Err(Error::new(
            "DEVICE_IDENTITY_CONFLICT",
            "The LAN packet does not match the expected appliance identity.",
        ));
    }
    let cipher = Aes128::new((&KEY).into());
    let mut message = Zeroizing::new(packet[40..end].to_vec());
    for block in message.chunks_exact_mut(16) {
        cipher.decrypt_block(block.try_into().unwrap());
    }
    let padding = usize::from(*message.last().unwrap());
    if !(1..=16).contains(&padding)
        || !message[message.len() - padding..]
            .iter()
            .all(|byte| usize::from(*byte) == padding)
        || padding == message.len()
    {
        return Err(invalid());
    }
    let length = message.len() - padding;
    message.truncate(length);
    Ok(SecretBytes::new(std::mem::take(&mut *message)))
}

#[cfg(test)]
mod tests;
