//! Private, closed payload for system secret storage; never an ordinary host schema.
use super::*;
use crate::protected::{serialize_private, unconfirmed};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
enum Profile {
    #[serde(rename = "mijia_reference_v1")]
    Reference,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StoredModernSession {
    profile: Profile,
    #[serde(serialize_with = "serialize_private")]
    subject: PrivateString,
    #[serde(serialize_with = "serialize_private")]
    encrypted_subject: PrivateString,
    #[serde(serialize_with = "serialize_private")]
    security: PrivateString,
    #[serde(serialize_with = "serialize_private")]
    token: PrivateString,
    #[serde(serialize_with = "serialize_private")]
    pass_token: PrivateString,
    #[serde(serialize_with = "serialize_private")]
    device: PrivateString,
    #[serde(serialize_with = "serialize_private")]
    pass_o: PrivateString,
    #[serde(serialize_with = "serialize_private")]
    agent: PrivateString,
}
impl StoredModernSession {
    pub fn from_session(session: ModernMijiaSession) -> Self {
        Self {
            profile: Profile::Reference,
            subject: session.subject,
            encrypted_subject: session.encrypted_subject,
            security: session.security,
            token: session.token,
            pass_token: session.pass_token,
            device: session.identity.device,
            pass_o: session.identity.pass_o,
            agent: session.identity.agent,
        }
    }
    pub fn subject(&self) -> &str {
        self.subject.as_str()
    }
    pub fn session(&self) -> Result<ModernMijiaSession> {
        if !valid_client(self) {
            return Err(unconfirmed());
        }
        let session = ModernMijiaSession {
            identity: ClientIdentity {
                device: self.device.clone(),
                pass_o: self.pass_o.clone(),
                agent: self.agent.clone(),
            },
            subject: self.subject.clone(),
            encrypted_subject: self.encrypted_subject.clone(),
            security: self.security.clone(),
            token: self.token.clone(),
            pass_token: self.pass_token.clone(),
        };
        session.validate().map_err(|_| unconfirmed())?;
        Ok(session)
    }
}
fn upper_hex(text: &str, length: usize) -> bool {
    text.len() == length
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'A'..=b'F').contains(&b))
}
fn valid_client(stored: &StoredModernSession) -> bool {
    if !upper_hex(stored.device.as_str(), 16)
        || stored.pass_o.as_str().len() != 16
        || !stored
            .pass_o
            .as_str()
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return false;
    }
    let Some(agent) = stored
        .agent
        .as_str()
        .strip_prefix("Android-15-11.0.701-Xiaomi-23046RP50C-OS2.0.212.0.VMYCNXM-")
    else {
        return false;
    };
    let pieces: Vec<_> = agent.split('-').collect();
    if pieces.len() != 8
        || !upper_hex(pieces[0], 40)
        || pieces[1] != "CN"
        || !upper_hex(pieces[2], 32)
        || !upper_hex(pieces[3], 32)
        || pieces[4] != "SmartHome"
        || pieces[5] != "MI_APP_STORE"
        || pieces[7] != "64"
    {
        return false;
    }
    let tail: Vec<_> = pieces[6].split('|').collect();
    tail.len() == 3
        && tail[0] == pieces[0]
        && upper_hex(tail[1], 40)
        && tail[2] == stored.pass_o.as_str()
}
