//! Bounded V3 wire compatibility, not HomeVerb storage cryptography or device admission.

use aes::{
    Aes256,
    cipher::{BlockModeDecrypt, BlockModeEncrypt, KeyIvInit, block_padding::NoPadding},
};
use homeverb_core::SecretBytes;
use homeverb_domain::{Error, Result};
use sha2::{Digest, Sha256};
use std::fmt;
use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

const MAX_FRAME: usize = 4096;

fn invalid() -> Error {
    Error::new(
        "MIDEA_RESPONSE_INVALID",
        "The V3 LAN frame is malformed or unsupported.",
    )
}

fn authentication_required() -> Error {
    Error::new(
        "AUTH_REQUIRED",
        "The V3 LAN credentials could not be confirmed.",
    )
}

/// Inspect one frame header without retaining a buffer. A length is not validity.
/// The transport must bound buffering and split coalesced frames before decoding.
pub fn v3_frame_len(prefix: &[u8]) -> Result<Option<usize>> {
    if prefix.len() < 6 {
        return Ok(None);
    }
    let size = usize::from(u16::from_be_bytes([prefix[2], prefix[3]])) + 8;
    let padding = usize::from(prefix[5] >> 4);
    if prefix[..2] != [0x83, 0x70] || prefix[4] != 0x20 || size > MAX_FRAME {
        return Err(invalid());
    }
    match prefix[5] & 15 {
        0 | 1 if padding == 0 => {}
        3 | 6 if size >= 54 && (size - 38).is_multiple_of(16) && size - 38 >= padding + 2 => {}
        _ => return Err(invalid()),
    }
    Ok(Some(size))
}

fn decrypt(bytes: &[u8], key: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
    let mut plain = Zeroizing::new(bytes.to_vec());
    cbc::Decryptor::<Aes256>::new_from_slices(key, &[0; 16])
        .map_err(|_| invalid())?
        .decrypt_padded::<NoPadding>(&mut plain)
        .map_err(|_| invalid())?;
    Ok(plain)
}

/// One handshake using explicitly supplied protected credentials, with no defaults.
/// ```compile_fail
/// fn export(handshake: &homeverb_midea::lan::V3Handshake) {
///     let _ = serde_json::to_string(handshake);
/// }
/// ```
pub struct V3Handshake {
    key: SecretBytes,
    request: SecretBytes,
}

impl fmt::Debug for V3Handshake {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("V3Handshake([REDACTED])")
    }
}

impl V3Handshake {
    pub fn new(token: SecretBytes, key: SecretBytes) -> Result<Self> {
        if token.expose().len() != 64 || key.expose().len() != 32 {
            return Err(Error::invalid());
        }
        let mut request = vec![0x83, 0x70, 0, 64, 0x20, 0, 0, 0];
        request.extend_from_slice(token.expose());
        Ok(Self {
            key,
            request: SecretBytes::new(request),
        })
    }

    pub fn request(&self) -> &SecretBytes {
        &self.request
    }

    /// Consumes the exchange; successful key confirmation does not bind an account/device.
    pub fn finish(self, frame: &[u8]) -> Result<V3Session> {
        if v3_frame_len(frame)? != Some(frame.len()) || frame[5] != 1 {
            return Err(invalid());
        }
        let response = &frame[8..];
        if response == b"ERROR" {
            return Err(authentication_required());
        }
        if response.len() != 64 {
            return Err(invalid());
        }
        let mut key = decrypt(&response[..32], self.key.expose())?;
        if !bool::from(Sha256::digest(&*key).as_slice().ct_eq(&response[32..])) {
            return Err(authentication_required());
        }
        for (byte, secret) in key.iter_mut().zip(self.key.expose()) {
            *byte ^= secret;
        }
        Ok(V3Session {
            key: SecretBytes::new(std::mem::take(&mut *key)),
            next_request: 0,
        })
    }
}

/// An opaque session key with a bounded request counter; no replay/freshness proof.
/// ```compile_fail
/// fn export(session: &homeverb_midea::lan::V3Session) {
///     let _ = serde_json::to_string(session);
/// }
/// ```
pub struct V3Session {
    key: SecretBytes,
    next_request: u16,
}

impl fmt::Debug for V3Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("V3Session([REDACTED])")
    }
}

#[derive(Debug)]
pub struct V3Response {
    pub sequence: u16,
    pub payload: SecretBytes,
}

pub(super) fn padding_for(size: usize) -> Result<usize> {
    if size == 0 || size > MAX_FRAME {
        return Err(Error::invalid());
    }
    let padding = (16 - (size + 2) % 16) % 16;
    if size + padding + 40 > MAX_FRAME {
        return Err(Error::invalid());
    }
    Ok(padding)
}

impl V3Session {
    pub fn encode_request(&mut self, payload: &SecretBytes) -> Result<SecretBytes> {
        let mut padding = Zeroizing::new(vec![0; padding_for(payload.expose().len())?]);
        getrandom::fill(&mut padding).map_err(|_| {
            Error::new(
                "MIDEA_REQUEST_FAILED",
                "The V3 LAN request could not be prepared.",
            )
        })?;
        self.encode(payload, &padding)
    }

    fn encode(&mut self, payload: &SecretBytes, padding: &[u8]) -> Result<SecretBytes> {
        let payload = payload.expose();
        if padding_for(payload.len())? != padding.len() {
            return Err(Error::invalid());
        }
        if self.next_request == u16::MAX {
            return Err(Error::new(
                "RESOURCE_BUSY",
                "The V3 session request limit was reached.",
            ));
        }
        // Always include the digest, including when the plaintext needs no padding.
        let size = (payload.len() + padding.len() + 32) as u16;
        let mut frame = vec![
            0x83,
            0x70,
            (size >> 8) as u8,
            size as u8,
            0x20,
            (padding.len() as u8) << 4 | 6,
        ];
        let mut plain = Zeroizing::new(Vec::with_capacity(2 + payload.len() + padding.len()));
        plain.extend_from_slice(&self.next_request.to_be_bytes());
        plain.extend_from_slice(payload);
        plain.extend_from_slice(padding);
        let mut digest = Sha256::new();
        digest.update(&frame);
        digest.update(&*plain);
        let length = plain.len();
        cbc::Encryptor::<Aes256>::new_from_slices(self.key.expose(), &[0; 16])
            .map_err(|_| invalid())?
            .encrypt_padded::<NoPadding>(&mut plain, length)
            .map_err(|_| invalid())?;
        frame.extend_from_slice(&plain);
        frame.extend_from_slice(&digest.finalize());
        self.next_request += 1;
        Ok(SecretBytes::new(frame))
    }

    /// Decode exactly one response; callers must separately validate inner identity/state.
    pub fn decode_response(&self, frame: &[u8]) -> Result<V3Response> {
        if v3_frame_len(frame)? != Some(frame.len()) || frame[5] & 15 != 3 {
            return Err(invalid());
        }
        let end = frame.len() - 32;
        let plain = decrypt(&frame[6..end], self.key.expose())?;
        let mut digest = Sha256::new();
        digest.update(&frame[..6]);
        digest.update(&*plain);
        if !bool::from(digest.finalize().as_slice().ct_eq(&frame[end..])) {
            return Err(invalid());
        }
        let payload_end = plain.len() - usize::from(frame[5] >> 4);
        Ok(V3Response {
            sequence: u16::from_be_bytes([plain[0], plain[1]]),
            payload: SecretBytes::new(plain[2..payload_end].to_vec()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn bytes(value: &Value) -> Vec<u8> {
        let value = value.as_str().unwrap();
        (0..value.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&value[i..i + 2], 16).unwrap())
            .collect()
    }

    fn fixture() -> Value {
        serde_json::from_str(include_str!("../../tests/fixtures/lan-v3.json")).unwrap()
    }

    fn session(fixture: &Value) -> V3Session {
        V3Handshake::new(
            SecretBytes::new(bytes(&fixture["token"])),
            SecretBytes::new(bytes(&fixture["key"])),
        )
        .unwrap()
        .finish(&bytes(&fixture["response"]))
        .unwrap()
    }

    #[test]
    fn independent_request_vectors_include_digest_for_aligned_payloads() {
        let fixture = fixture();
        for case in fixture["frames"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| v["type"] == 6)
        {
            let mut session = session(&fixture);
            let payload = SecretBytes::new(bytes(&case["payload"]));
            let frame = session.encode(&payload, &bytes(&case["padding"])).unwrap();
            assert_eq!(frame.expose(), bytes(&case["frame"]));
            assert_eq!(session.next_request, 1);
        }
    }

    #[test]
    fn request_bounds_and_counter_exhaustion_fail_without_wrapping() {
        let fixture = fixture();
        let mut session = session(&fixture);
        for size in [0, 4047, 4096, 4097] {
            assert_eq!(
                session
                    .encode_request(&SecretBytes::new(vec![0; size]))
                    .unwrap_err()
                    .code,
                "INVALID_ARGUMENT"
            );
            assert_eq!(session.next_request, 0);
        }
        let frame = session
            .encode_request(&SecretBytes::new(vec![0; 4046]))
            .unwrap();
        assert_eq!(frame.expose().len(), 4086);
        assert_eq!(v3_frame_len(frame.expose()).unwrap(), Some(4086));
        let payload = SecretBytes::new(vec![0; 120]);
        session.encode_request(&payload).unwrap();
        assert_eq!(session.next_request, 2);
        session.next_request = u16::MAX - 1;
        session.encode_request(&payload).unwrap();
        assert_eq!(session.next_request, u16::MAX);
        assert_eq!(
            session.encode_request(&payload).unwrap_err().code,
            "RESOURCE_BUSY"
        );
        assert_eq!(session.next_request, u16::MAX);
    }
}
