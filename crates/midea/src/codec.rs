//! Minimal Meiju compatibility codec; not cryptography for HomeVerb storage.
//! Modified from the pinned Apache-2.0 source in third-party/midea-auto-cloud/NOTICE.md.

use crate::{invalid, material};
use aes::{
    Aes128,
    cipher::{BlockCipherDecrypt, KeyInit},
};
use hmac::{Hmac, Mac};
use homeverb_core::SecretBytes;
use homeverb_domain::Result;
use md5::Md5;
use sha2::{Digest, Sha256};
use zeroize::{Zeroize, Zeroizing};

fn hex(bytes: &[u8]) -> Zeroizing<String> {
    const DIGITS: &[u8] = b"0123456789abcdef";
    let mut value = Zeroizing::new(String::with_capacity(bytes.len() * 2));
    for byte in bytes {
        value.push(DIGITS[(byte >> 4) as usize] as char);
        value.push(DIGITS[(byte & 15) as usize] as char);
    }
    value
}

pub(crate) fn sign(body: &[u8], random: &str) -> Zeroizing<String> {
    let mut mac = <Hmac<Sha256> as KeyInit>::new_from_slice(material::HMAC_KEY.as_bytes())
        .expect("The pinned HMAC key is valid.");
    mac.update(material::IOT_KEY.as_bytes());
    mac.update(body);
    mac.update(random.as_bytes());
    hex(&mac.finalize().into_bytes())
}

pub(crate) fn passwords(login_id: &str, password: &str) -> (Zeroizing<String>, Zeroizing<String>) {
    let inner = hex(&Sha256::digest(password.as_bytes()));
    let mut sha = Sha256::new();
    sha.update(login_id.as_bytes());
    sha.update(inner.as_bytes());
    sha.update(material::LOGIN_KEY.as_bytes());
    let iot = hex(&sha.finalize());
    let md = hex(&Md5::digest(password.as_bytes()));
    (iot, hex(&Md5::digest(md.as_bytes())))
}

pub(crate) fn device_id(account: &str) -> Zeroizing<String> {
    let mut md = Md5::new();
    md.update(b"Hello, ");
    md.update(account.as_bytes());
    md.update(b"!");
    let mut value = hex(&md.finalize());
    value.truncate(16);
    value
}

pub(crate) fn session_key(encoded: &str) -> Result<SecretBytes> {
    // The reviewed flow returns a hex AES-128-ECB/PKCS#7 wrapped ASCII session key.
    if !matches!(encoded.len(), 64 | 96) || !encoded.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(invalid());
    }
    let mut bytes = Zeroizing::new(Vec::with_capacity(encoded.len() / 2));
    for index in (0..encoded.len()).step_by(2) {
        bytes.push(u8::from_str_radix(&encoded[index..index + 2], 16).map_err(|_| invalid())?);
    }
    let cipher = Aes128::new(material::SESSION_KEY.into());
    for block in bytes.chunks_exact_mut(16) {
        cipher.decrypt_block(block.try_into().map_err(|_| invalid())?);
    }
    let padding = *bytes.last().ok_or_else(invalid)? as usize;
    if !(1..=16).contains(&padding)
        || !bytes[bytes.len() - padding..]
            .iter()
            .all(|b| *b as usize == padding)
    {
        return Err(invalid());
    }
    let size = bytes.len() - padding;
    if !matches!(size, 16 | 24 | 32) || !bytes[..size].iter().all(|b| b.is_ascii_graphic()) {
        return Err(invalid());
    }
    bytes[size..].zeroize();
    bytes.truncate(size);
    Ok(SecretBytes::new(std::mem::take(&mut *bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_independent_fictional_known_answers() {
        let vector: serde_json::Value =
            serde_json::from_str(include_str!("../tests/fixtures/login.json")).unwrap();
        let get = |key: &str| vector[key].as_str().unwrap();
        assert_eq!(&*device_id(get("account")), get("device_id"));
        let (iot, iam) = passwords(get("login_id"), get("password"));
        assert_eq!(&*iot, get("iot_password"));
        assert_eq!(&*iam, get("iam_password"));
        assert_eq!(
            &*sign(get("body").as_bytes(), get("random")),
            get("signature")
        );
        assert_eq!(
            session_key(get("wrapped_key")).unwrap().expose(),
            get("session_key").as_bytes()
        );
        assert_ne!(&*sign(b"different bytes", get("random")), get("signature"));
        for bad in [
            "",
            "00",
            &"0".repeat(64),
            &"g".repeat(64),
            &"a".repeat(128),
            &"a".repeat(65),
        ] {
            assert_eq!(session_key(bad).unwrap_err().code, "MIDEA_RESPONSE_INVALID");
        }
    }
}
