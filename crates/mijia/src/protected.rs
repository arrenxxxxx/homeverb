//! Versioned, account-bound payloads for system secret storage only.

use crate::{CloudDevice, MijiaSession, ModelMapping, PROVIDER, PrivateString, Region};
use homeverb_core::{
    AccountLease, AccountRecord, CredentialRef, Operation, SecretBytes, valid_account_id,
};
use homeverb_domain::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

pub(crate) fn unconfirmed() -> Error {
    Error::new(
        "ACCOUNT_SESSION_UNCONFIRMED",
        "The protected Mijia account requires explicit validation or recovery.",
    )
}
pub(crate) fn conflict() -> Error {
    Error::new(
        "DEVICE_IDENTITY_CONFLICT",
        "Mijia device identity conflicts with an established protected binding.",
    )
}
pub(crate) fn serialize_private<S: serde::Serializer>(
    value: &PrivateString,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    serializer.serialize_str(value.as_str())
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StoredSession {
    region: Region,
    #[serde(serialize_with = "serialize_private")]
    subject: PrivateString,
    #[serde(serialize_with = "serialize_private")]
    security: PrivateString,
    #[serde(serialize_with = "serialize_private")]
    token: PrivateString,
}
impl StoredSession {
    pub fn from_session(session: MijiaSession) -> Self {
        Self {
            region: session.region,
            subject: session.user_id,
            security: session.security,
            token: session.token,
        }
    }
    pub fn subject(&self) -> &str {
        self.subject.as_str()
    }
    pub fn session(&self) -> Result<MijiaSession> {
        if self.region != Region::Cn
            || self.subject.as_str().is_empty()
            || self.subject.as_str().len() > 32
            || !self.subject.as_str().bytes().all(|b| b.is_ascii_digit())
            || self.subject.as_str().bytes().all(|b| b == b'0')
            || !crate::cookie_value(self.token.as_str())
            || crate::codec::unbase64(self.security.as_str())
                .map_err(|_| unconfirmed())?
                .len()
                != 16
        {
            return Err(unconfirmed());
        }
        Ok(MijiaSession {
            region: self.region,
            user_id: self.subject.clone(),
            security: self.security.clone(),
            token: self.token.clone(),
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Binding {
    #[serde(serialize_with = "serialize_private")]
    pub did: PrivateString,
    pub device_id: Uuid,
    pub connection_id: Uuid,
    pub identity_ref: Uuid,
    pub name: String,
    pub model: String,
    pub reported_spec: Option<String>,
    pub mapping: Option<ModelMapping>,
    pub listed: bool,
}
impl Binding {
    pub fn new(device: &CloudDevice) -> Self {
        Self {
            did: device.did.clone(),
            device_id: Uuid::new_v4(),
            connection_id: Uuid::new_v4(),
            identity_ref: Uuid::new_v4(),
            name: device.name.clone(),
            model: device.model.clone(),
            reported_spec: device.spec_type.clone(),
            mapping: None,
            listed: true,
        }
    }
    pub fn device(&self) -> CloudDevice {
        CloudDevice {
            did: self.did.clone(),
            name: self.name.clone(),
            model: self.model.clone(),
            spec_type: self.reported_spec.clone(),
            online_hint: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Payload {
    version: u8,
    account_id: Uuid,
    credential_ref: CredentialRef,
    provider: String,
    region: String,
    pub session: StoredSession,
    pub bindings: Vec<Binding>,
}
impl Payload {
    pub fn new(record: &AccountRecord, session: MijiaSession) -> Self {
        Self {
            version: 1,
            account_id: record.id,
            credential_ref: record.credential_ref,
            provider: PROVIDER.into(),
            region: "CN".into(),
            session: StoredSession::from_session(session),
            bindings: vec![],
        }
    }
    pub fn decode(bytes: &SecretBytes, record: &AccountRecord) -> Result<Self> {
        if bytes.expose().len() > crate::MAX_RESPONSE_BYTES {
            return Err(unconfirmed());
        }
        let payload: Self = serde_json::from_slice(bytes.expose()).map_err(|_| unconfirmed())?;
        payload.validate(record)?;
        Ok(payload)
    }
    pub fn encode(&self, record: &AccountRecord) -> Result<SecretBytes> {
        self.validate(record)?;
        let bytes = SecretBytes::new(serde_json::to_vec(self).map_err(|_| unconfirmed())?);
        if bytes.expose().len() > crate::MAX_RESPONSE_BYTES {
            return Err(unconfirmed());
        }
        Ok(bytes)
    }
    fn validate(&self, record: &AccountRecord) -> Result<()> {
        if self.version != 1
            || self.account_id != record.id
            || self.credential_ref != record.credential_ref
            || self.provider != PROVIDER
            || record.provider != PROVIDER
            || self.region != "CN"
            || record.region != "CN"
            || self.bindings.len() > 4096
        {
            return Err(unconfirmed());
        }
        self.session.session()?;
        let mut provider_ids = HashSet::new();
        let mut local_ids = HashSet::new();
        for binding in &self.bindings {
            if !crate::bounded_text(binding.did.as_str(), 128)
                || !provider_ids.insert(&binding.did)
                || !crate::bounded_text(&binding.name, 256)
                || !crate::bounded_text(&binding.model, 256)
                || binding
                    .reported_spec
                    .as_ref()
                    .is_some_and(|s| !crate::bounded_text(s, 512))
            {
                return Err(unconfirmed());
            }
            for id in [
                binding.device_id,
                binding.connection_id,
                binding.identity_ref,
            ] {
                valid_account_id(id).map_err(|_| unconfirmed())?;
                if !local_ids.insert(id) {
                    return Err(unconfirmed());
                }
            }
            if let Some(mapping) = &binding.mapping {
                if mapping.model != binding.model
                    || binding
                        .reported_spec
                        .as_ref()
                        .is_some_and(|s| s != &mapping.spec_type)
                {
                    return Err(unconfirmed());
                }
                mapping.validate_saved().map_err(|_| unconfirmed())?;
            }
        }
        Ok(())
    }
}

pub(crate) struct LoadedAccount {
    pub lease: AccountLease,
    pub bytes: SecretBytes,
    pub payload: Payload,
}
impl LoadedAccount {
    pub async fn load(lease: AccountLease, pending: bool, operation: &Operation) -> Result<Self> {
        if lease.record().provider != PROVIDER || lease.record().region != "CN" {
            return Err(Error::new(
                "ACCOUNT_NOT_FOUND",
                "The selected Mijia account does not exist.",
            ));
        }
        let bytes = if pending {
            lease
                .inspect_session(operation)
                .await?
                .ok_or_else(unconfirmed)?
        } else {
            lease.read_session(operation).await?
        };
        let payload = operation
            .run(async { Payload::decode(&bytes, lease.record()) })
            .await?;
        Ok(Self {
            lease,
            bytes,
            payload,
        })
    }
    pub async fn save(&mut self, operation: &Operation) -> Result<()> {
        let bytes = operation
            .run(async { self.payload.encode(self.lease.record()) })
            .await?;
        if bytes.expose() != self.bytes.expose() {
            self.lease.store_session(&bytes, operation).await?;
            self.bytes = bytes;
        }
        Ok(())
    }
}
