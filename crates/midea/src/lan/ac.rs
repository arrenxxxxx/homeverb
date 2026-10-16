//! Standard AC query and C0 candidates; exact-model admission and freshness remain separate.

use homeverb_core::SecretBytes;
use homeverb_domain::{Error, Result};
use std::fmt;

fn invalid() -> Error {
    Error::new(
        "MIDEA_RESPONSE_INVALID",
        "The AC LAN message is malformed or does not match the expected context.",
    )
}

fn unsupported() -> Error {
    Error::new(
        "CAPABILITY_UNSUPPORTED",
        "The AC LAN message type or temperature unit is outside the reviewed C0 scope.",
    )
}

fn crc8(bytes: &[u8]) -> u8 {
    let mut crc = 0;
    for byte in bytes {
        crc ^= byte;
        for _ in 0..8 {
            crc = (crc >> 1) ^ if crc & 1 != 0 { 0x8c } else { 0 };
        }
    }
    crc
}

/// Encode one standard AC read query. Explicit sequence/version inputs grant no access.
pub fn encode_ac_status_query(protocol_version: u8, sequence: u8) -> Result<SecretBytes> {
    if !(1..=253).contains(&sequence) {
        return Err(Error::invalid());
    }
    let mut bytes = vec![0; 33];
    bytes[..3].copy_from_slice(&[0xaa, 32, 0xac]);
    bytes[8] = protocol_version;
    bytes[9] = 3;
    bytes[10..14].copy_from_slice(&[0x41, 0x81, 0, 0xff]);
    bytes[30] = sequence;
    bytes[31] = crc8(&bytes[10..31]);
    bytes[32] = bytes[1..32]
        .iter()
        .fold(0_u8, |sum, byte| sum.wrapping_sub(*byte));
    Ok(SecretBytes::new(bytes))
}

/// Native candidates, not admitted properties, fresh observations or write evidence.
/// ```compile_fail
/// fn export(value: &homeverb_midea::lan::AcC0Status) {
///     let _ = serde_json::to_string(value);
/// }
/// ```
pub struct AcC0Status {
    pub power: bool,
    pub mode_code: u8,
    pub fan_speed_code: u8,
    pub target_celsius: f64,
    pub indoor_celsius: Option<f64>,
    pub outdoor_celsius: Option<f64>,
}

impl fmt::Debug for AcC0Status {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AcC0Status([REDACTED])")
    }
}

fn ambient(integer: u8, decimal: u8) -> Result<Option<f64>> {
    if integer == 0xff {
        return Ok(None);
    }
    if decimal > 9 {
        return Err(invalid());
    }
    let half = (f64::from(integer) - 50.0) / 2.0;
    Ok(Some(if decimal == 0 {
        half
    } else {
        half.trunc() + if half < 0.0 { -1.0 } else { 1.0 } * f64::from(decimal) / 10.0
    }))
}

/// Decode one C0 query candidate after protected identity and outer-packet validation.
/// This neither validates a model profile nor proves reply freshness/correlation.
pub fn decode_ac_c0_status(
    expected_protocol_version: u8,
    message: &SecretBytes,
) -> Result<AcC0Status> {
    let bytes = message.expose();
    if !(12..=256).contains(&bytes.len())
        || bytes[0] != 0xaa
        || usize::from(bytes[1]) + 1 != bytes.len()
        || bytes[2] != 0xac
        || bytes[8] != expected_protocol_version
        || bytes[1..]
            .iter()
            .fold(0_u8, |sum, byte| sum.wrapping_add(*byte))
            != 0
    {
        return Err(invalid());
    }
    let body = &bytes[10..bytes.len() - 1];
    if bytes[9] != 3 || body[0] != 0xc0 {
        return Err(unsupported());
    }
    if body.len() < 15 {
        return Err(invalid());
    }
    if body[10] & 4 != 0 {
        return Err(unsupported());
    }
    let decimal = if body.len() > 20 { body[15] } else { 0 };
    Ok(AcC0Status {
        power: body[1] & 1 != 0,
        mode_code: body[2] >> 5,
        fan_speed_code: body[3] & 0x7f,
        target_celsius: f64::from(body[2] & 15) + 16.0 + if body[2] & 16 != 0 { 0.5 } else { 0.0 },
        indoor_celsius: ambient(body[11], decimal & 15)?,
        outdoor_celsius: ambient(body[12], decimal >> 4)?,
    })
}

#[cfg(test)]
mod tests;
