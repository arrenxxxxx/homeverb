//! Appliance message-version discovery, not model or status admission.

use crate::ApplianceFamily;
use homeverb_core::SecretBytes;
use homeverb_domain::{Error, Result};

fn family_code(family: ApplianceFamily) -> Result<u8> {
    match family {
        ApplianceFamily::AirConditioner => Ok(0xac),
        ApplianceFamily::Dishwasher => Ok(0xe1),
        ApplianceFamily::Refrigerator => Ok(0xca),
        ApplianceFamily::Other(_) => Err(Error::new(
            "CAPABILITY_UNSUPPORTED",
            "The appliance family is outside the reviewed LAN query scope.",
        )),
    }
}

/// Encode the initial appliance query before choosing any message protocol version.
pub fn encode_appliance_query(family: ApplianceFamily) -> Result<SecretBytes> {
    let mut bytes = vec![0; 30];
    bytes[..3].copy_from_slice(&[0xaa, 29, family_code(family)?]);
    bytes[9] = 0xa0;
    bytes[29] = bytes[1..29]
        .iter()
        .fold(0_u8, |sum, byte| sum.wrapping_sub(*byte));
    Ok(SecretBytes::new(bytes))
}

/// Extract the message-version hint after separate outer identity validation.
/// No body/model/freshness semantics or query correlation are inferred.
pub fn decode_appliance_protocol(family: ApplianceFamily, message: &SecretBytes) -> Result<u8> {
    let family = family_code(family)?;
    let bytes = message.expose();
    if !(12..=256).contains(&bytes.len())
        || bytes[0] != 0xaa
        || usize::from(bytes[1]) + 1 != bytes.len()
        || bytes[2] != family
        || bytes[1..]
            .iter()
            .fold(0_u8, |sum, byte| sum.wrapping_add(*byte))
            != 0
    {
        return Err(Error::new(
            "MIDEA_RESPONSE_INVALID",
            "The LAN appliance reply is malformed or does not match the expected family.",
        ));
    }
    if bytes[9] != 0xa0 {
        return Err(Error::new(
            "CAPABILITY_UNSUPPORTED",
            "The LAN message is not an appliance-query reply.",
        ));
    }
    Ok(bytes[8])
}

#[cfg(test)]
mod tests;
