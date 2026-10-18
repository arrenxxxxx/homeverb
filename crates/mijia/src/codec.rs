//! Legacy wire compatibility from MIT micloud; see third-party/micloud/NOTICE.md.

use crate::{PrivateString, invalid};
use base64::{Engine, engine::general_purpose::STANDARD};
use homeverb_core::SecretBytes;
use homeverb_domain::Result;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

pub(crate) fn base64(bytes: &[u8]) -> String {
    STANDARD.encode(bytes)
}
pub(crate) fn unbase64(text: &str) -> Result<Zeroizing<Vec<u8>>> {
    STANDARD
        .decode(text)
        .map(Zeroizing::new)
        .map_err(|_| invalid())
}

pub(crate) fn signed_nonce(security: &str, nonce: &str) -> Result<Zeroizing<Vec<u8>>> {
    let security = unbase64(security)?;
    let nonce = unbase64(nonce)?;
    if security.len() != 16 || nonce.len() != 12 {
        return Err(invalid());
    }
    let mut hash = Sha256::new();
    hash.update(&*security);
    hash.update(&*nonce);
    Ok(Zeroizing::new(hash.finalize().to_vec()))
}

fn signature(path: &str, fields: &[(&str, &str)], key: &[u8]) -> PrivateString {
    use sha1::Digest;
    let mut input = Zeroizing::new(format!("POST&{path}"));
    for (name, value) in fields {
        input.push_str(&format!("&{name}={value}"));
    }
    input.push('&');
    input.push_str(&base64(key));
    PrivateString::new(base64(&sha1::Sha1::digest(input.as_bytes())))
}

/// RC4 is required by this vendor protocol, always inside authenticated HTTPS.
fn rc4(key: &[u8], input: &[u8]) -> SecretBytes {
    let mut state = Zeroizing::new([0u8; 256]);
    for (i, byte) in state.iter_mut().enumerate() {
        *byte = i as u8;
    }
    let mut j = 0usize;
    for i in 0..256 {
        j = (j + state[i] as usize + key[i % key.len()] as usize) & 255;
        state.swap(i, j);
    }
    let (mut i, mut j) = (0usize, 0usize);
    let mut out = Vec::with_capacity(input.len());
    for index in 0..(1024 + input.len()) {
        i = (i + 1) & 255;
        j = (j + state[i] as usize) & 255;
        state.swap(i, j);
        let byte = state[(state[i] as usize + state[j] as usize) & 255];
        if index >= 1024 {
            out.push(input[index - 1024] ^ byte);
        }
    }
    SecretBytes::new(out)
}

pub(crate) fn encrypt(key: &[u8], input: &[u8]) -> PrivateString {
    PrivateString::new(base64(rc4(key, input).expose()))
}
pub(crate) fn decrypt(key: &[u8], input: &[u8]) -> Result<SecretBytes> {
    let input = std::str::from_utf8(input).map_err(|_| invalid())?;
    Ok(rc4(key, &unbase64(input)?))
}

pub(crate) fn form(fields: &[(&str, &str)]) -> SecretBytes {
    // The serializer may hold transient copies; its final buffer is zeroized on drop.
    let text = reqwest::Url::parse_with_params("https://localhost/", fields).expect("fixed URL");
    SecretBytes::new(text.query().unwrap_or("").as_bytes().to_vec())
}

pub(crate) fn encode(
    path: &str,
    data: &str,
    security: &str,
    nonce: &str,
) -> Result<(SecretBytes, SecretBytes)> {
    let key = signed_nonce(security, nonce)?;
    let hash = signature(path, &[("data", data)], &key);
    let encrypted = encrypt(&key, data.as_bytes());
    let hash = encrypt(&key, hash.as_str().as_bytes());
    let sign = signature(
        path,
        &[("data", encrypted.as_str()), ("rc4_hash__", hash.as_str())],
        &key,
    );
    let body = form(&[
        ("data", encrypted.as_str()),
        ("rc4_hash__", hash.as_str()),
        ("signature", sign.as_str()),
        ("ssecurity", security),
        ("_nonce", nonce),
    ]);
    Ok((body, SecretBytes::new(key.to_vec())))
}

pub(crate) fn nonce() -> Result<String> {
    let mut bytes = [0u8; 12];
    getrandom::fill(&mut bytes[..8]).map_err(|_| crate::failed())?;
    let minutes =
        u32::try_from(chrono::Utc::now().timestamp() / 60).map_err(|_| crate::failed())?;
    bytes[8..].copy_from_slice(&minutes.to_be_bytes());
    Ok(base64(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    // Independently calculated with Python hashlib/base64 and a separate RC4 fixture.
    #[test]
    fn independent_known_answer() {
        let (body, key) = encode(
            "/home/device_list",
            r#"{"getVirtualModel":true}"#,
            "AAECAwQFBgcICQoLDA0ODw==",
            "AAECAwQFBgcICQoL",
        )
        .unwrap();
        assert_eq!(
            base64(key.expose()),
            "Nz6YMO70n75ITJeXTWpiyXXLp6IBZ0yT0/hEJB0FDOg="
        );
        assert_eq!(
            std::str::from_utf8(body.expose()).unwrap(),
            "data=Gjc%2B3t3nRiqNVPXTn3GHA5t6a%2FAlmD%2BW&rc4_hash__=LkMQ%2BJDXGy%2BXYv7zpUuaTcU3CcMNlzWOddlUQA%3D%3D&signature=Kc1WMAMNFJbk8qk3Ro0k9WQRJ14%3D&ssecurity=AAECAwQFBgcICQoLDA0ODw%3D%3D&_nonce=AAECAwQFBgcICQoL"
        );
        assert_eq!(
            decrypt(
                key.expose(),
                b"Gjc61M3UDWLJDbbNt22WCoN6a/91gTOYZssNJgn9bA=="
            )
            .unwrap()
            .expose(),
            br#"{"code":0,"result":{"list":[]}}"#
        );
    }
    #[test]
    fn malformed_keys_and_nonces_fail_closed() {
        for (key, nonce) in [
            ("!", "AAECAwQFBgcICQoL"),
            ("AA==", "AAECAwQFBgcICQoL"),
            ("AAECAwQFBgcICQoLDA0ODw==", "AA=="),
        ] {
            assert!(encode("/home/device_list", "{}", key, nonce).is_err());
        }
        assert!(decrypt(&[0u8; 32], b"not-base64").is_err());
    }
    #[test]
    fn nonce_has_random_prefix_and_current_minutes() {
        let first = nonce().unwrap();
        let second = nonce().unwrap();
        assert_ne!(first, second);
        let bytes = unbase64(&first).unwrap();
        assert_eq!(bytes.len(), 12);
        assert_eq!(
            u32::from_be_bytes(bytes[8..].try_into().unwrap()),
            (chrono::Utc::now().timestamp() / 60) as u32
        );
    }
}
