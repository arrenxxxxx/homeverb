//! Standard E1/CA query candidates; model admission and observation evidence are separate.

use homeverb_core::SecretBytes;
use homeverb_domain::{Error, Result};
use std::fmt;

fn invalid() -> Error {
    Error::new(
        "MIDEA_RESPONSE_INVALID",
        "The appliance LAN message is malformed or does not match the expected context.",
    )
}

fn unsupported() -> Error {
    Error::new(
        "CAPABILITY_UNSUPPORTED",
        "The appliance LAN message type or unit is outside the reviewed standard query scope.",
    )
}

fn encode_query(family: u8, version: u8) -> SecretBytes {
    let mut bytes = vec![0xaa, 11, family, 0, 0, 0, 0, 0, version, 3, 0, 0];
    bytes[11] = bytes[1..11]
        .iter()
        .fold(0_u8, |sum, byte| sum.wrapping_sub(*byte));
    SecretBytes::new(bytes)
}

/// Encode the standard E1 query. The message version grants no device access.
pub fn encode_dishwasher_status_query(protocol_version: u8) -> SecretBytes {
    encode_query(0xe1, protocol_version)
}

/// Encode the standard AA/CA query, not the alternative Toshiba protocol.
pub fn encode_refrigerator_status_query(protocol_version: u8) -> SecretBytes {
    encode_query(0xca, protocol_version)
}

fn body(family: u8, version: u8, message: &SecretBytes, minimum: usize) -> Result<&[u8]> {
    let bytes = message.expose();
    if !(12..=256).contains(&bytes.len())
        || bytes[0] != 0xaa
        || usize::from(bytes[1]) + 1 != bytes.len()
        || bytes[2] != family
        || bytes[8] != version
        || bytes[1..]
            .iter()
            .fold(0_u8, |sum, byte| sum.wrapping_add(*byte))
            != 0
    {
        return Err(invalid());
    }
    if bytes[9] != 3 || bytes[10] != 0 {
        return Err(unsupported());
    }
    let body = &bytes[10..bytes.len() - 1];
    if body.len() < minimum {
        return Err(invalid());
    }
    Ok(body)
}

/// Native E1 candidates, not model capabilities, fresh observations or cycle completion.
/// ```compile_fail
/// fn export(value: &homeverb_midea::lan::DishwasherStatus) {
///     let _ = serde_json::to_string(value);
/// }
/// ```
pub struct DishwasherStatus {
    pub status_code: u8,
    pub mode_code: u8,
    pub additional_code: u8,
    pub temperature_code: u8,
    pub door_open: bool,
    pub rinse_aid_low: bool,
    pub salt_low: bool,
    pub water_low: bool,
    pub remaining_minutes: u16,
}

impl fmt::Debug for DishwasherStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("DishwasherStatus([REDACTED])")
    }
}

/// Decode the standard E1 query response after separate outer-packet and identity checks.
pub fn decode_dishwasher_status(
    expected_protocol_version: u8,
    message: &SecretBytes,
) -> Result<DishwasherStatus> {
    let body = body(0xe1, expected_protocol_version, message, 17)?;
    let high = if body.len() >= 34 { body[32] } else { 0 };
    Ok(DishwasherStatus {
        status_code: body[1],
        mode_code: body[2],
        additional_code: body[3],
        temperature_code: body[11],
        door_open: body[5] & 1 == 0,
        rinse_aid_low: body[5] & 2 != 0,
        salt_low: body[5] & 4 != 0,
        water_low: body[5] & 0x80 != 0,
        remaining_minutes: u16::from_le_bytes([body[6], high]),
    })
}

/// Native main-compartment candidates, without model, freshness or write authority.
/// ```compile_fail
/// fn export(value: &homeverb_midea::lan::RefrigeratorStatus) {
///     let _ = serde_json::to_string(value);
/// }
/// ```
pub struct RefrigeratorStatus {
    pub mode_flags: u8,
    pub fridge_power: bool,
    pub freezer_power: bool,
    pub fridge_target_celsius: u8,
    pub freezer_target_celsius: i8,
    pub fridge_current_celsius: f64,
    pub freezer_current_celsius: f64,
}

impl fmt::Debug for RefrigeratorStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RefrigeratorStatus([REDACTED])")
    }
}

/// Decode the standard AA/CA Celsius query response; other layouts remain unsupported.
pub fn decode_refrigerator_status(
    expected_protocol_version: u8,
    message: &SecretBytes,
) -> Result<RefrigeratorStatus> {
    let body = body(0xca, expected_protocol_version, message, 25)?;
    if body[7] & 8 != 0 {
        return Err(unsupported());
    }
    Ok(RefrigeratorStatus {
        mode_flags: body[1],
        fridge_power: body[6] & 1 == 0,
        freezer_power: body[6] & 0x10 == 0,
        fridge_target_celsius: body[2] & 15,
        freezer_target_celsius: -12 - (body[2] >> 4) as i8,
        fridge_current_celsius: (f64::from(body[17]) - 100.0) / 2.0,
        freezer_current_celsius: (f64::from(body[18]) - 100.0) / 2.0,
    })
}

#[cfg(test)]
mod tests;
