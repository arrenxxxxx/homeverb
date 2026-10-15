//! Explicit Meiju v2 token lookup, derived from the attributed midea-local source.

use super::{MeijuSession, PrivateString, decode, request_at};
use crate::{CloudEndpoint, CloudId, CloudTransport, invalid, lan::V3Handshake};
use homeverb_core::{Operation, SecretBytes};
use homeverb_domain::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

/// Select one reviewed UDP-ID encoding explicitly; never try alternate keys automatically.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanUdpIdMethod {
    BigEndian,
    LittleEndian,
}

impl LanUdpIdMethod {
    pub(crate) fn udp_id(self, appliance: &str) -> Result<Zeroizing<String>> {
        let id: u64 = appliance.parse().map_err(|_| unsupported())?;
        if id == 0 || id >= 1 << 48 {
            return Err(unsupported());
        }
        let bytes = match self {
            Self::BigEndian => id.to_be_bytes()[2..].to_vec(),
            Self::LittleEndian => id.to_le_bytes()[..6].to_vec(),
        };
        let digest = Sha256::digest(&bytes);
        Ok(Zeroizing::new(
            (0..16)
                .map(|i| format!("{:02x}", digest[i] ^ digest[i + 16]))
                .collect(),
        ))
    }
}

fn unsupported() -> Error {
    Error::new(
        "CAPABILITY_UNSUPPORTED",
        "The appliance identifier is outside the reviewed V3 credential scope.",
    )
}

#[derive(Serialize)]
struct TokenRequest<'a> {
    #[serde(rename = "homegroupId")]
    home: &'a str,
    #[serde(rename = "applianceCodes")]
    appliances: [&'a str; 1],
    udpid: &'a str,
    src: &'a str,
    format: &'a str,
    #[serde(rename = "platformId")]
    platform_id: &'a str,
    #[serde(rename = "deviceId")]
    device_id: &'a str,
    uid: Option<&'a str>,
    #[serde(rename = "clientType")]
    client_type: &'a str,
    #[serde(rename = "appId")]
    app_id: &'a str,
    language: &'a str,
}

#[derive(Deserialize)]
struct TokenList {
    tokenlist: Vec<TokenEntry>,
}

#[derive(Deserialize)]
struct TokenEntry {
    #[serde(rename = "udpId")]
    udp_id: PrivateString,
    token: PrivateString,
    key: PrivateString,
}

fn hex_secret(value: &str, size: usize) -> Result<SecretBytes> {
    if value.len() != size * 2 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(invalid());
    }
    let mut bytes = Zeroizing::new(Vec::with_capacity(size));
    for offset in (0..value.len()).step_by(2) {
        bytes.push(u8::from_str_radix(&value[offset..offset + 2], 16).map_err(|_| invalid())?);
    }
    Ok(SecretBytes::new(std::mem::take(&mut *bytes)))
}

fn handshake(body: &SecretBytes, udp_id: &str) -> Result<V3Handshake> {
    let entries: TokenList = decode(body).map_err(crate::sanitized)?;
    if entries.tokenlist.len() > 64 {
        return Err(invalid());
    }
    let mut matching = entries
        .tokenlist
        .iter()
        .filter(|entry| entry.udp_id.0.eq_ignore_ascii_case(udp_id));
    let entry = matching.next().ok_or_else(|| {
        Error::new(
            "AUTH_REQUIRED",
            "The provider did not return credentials for the selected appliance and UDP method.",
        )
    })?;
    if matching.next().is_some() {
        return Err(invalid());
    }
    V3Handshake::new(
        hex_secret(&entry.token.0, 64)?,
        hex_secret(&entry.key.0, 32)?,
    )
}

impl MeijuSession {
    pub(crate) fn lan_client_id(&self) -> Result<&str> {
        self.client_device_id
            .as_ref()
            .and_then(|id| std::str::from_utf8(id.expose()).ok())
            .ok_or_else(|| Error::new(
                "AUTH_REQUIRED",
                "Explicit account reauthorization is required to retain the LAN lookup client identifier.",
            ))
    }

    pub(crate) async fn lan_handshake(
        &self,
        transport: &dyn CloudTransport,
        appliance: &CloudId,
        home: &CloudId,
        udp_id: &str,
        operation: &Operation,
    ) -> Result<V3Handshake> {
        let body = request_at(
            transport,
            CloudEndpoint::LanCredentials,
            TokenRequest {
                home: home.expose(),
                appliances: [appliance.expose()],
                udpid: udp_id,
                src: "900",
                format: "2",
                platform_id: "1",
                device_id: self.lan_client_id()?,
                uid: None,
                client_type: "1",
                app_id: "900",
                language: "en_US",
            },
            Some(&self.token),
            &chrono::Utc::now().format("%Y%m%d%H%M%S").to_string(),
            operation,
        )
        .await?;
        operation.run(async { handshake(&body, udp_id) }).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn udp_ids_match_independent_upstream_vectors_and_reject_wide_ids() {
        for (method, expected) in [
            (
                LanUdpIdMethod::BigEndian,
                "505407553cbc909df7d36b82967ace2e",
            ),
            (
                LanUdpIdMethod::LittleEndian,
                "8011a3aa5116c19f17161815770eb6e4",
            ),
        ] {
            assert_eq!(method.udp_id("123456789").unwrap().as_str(), expected);
            assert!(method.udp_id("281474976710655").is_ok());
            for id in ["0", "281474976710656", "18446744073709551616", "invalid"] {
                assert_eq!(
                    method.udp_id(id).unwrap_err().code,
                    "CAPABILITY_UNSUPPORTED"
                );
            }
        }
    }
}
