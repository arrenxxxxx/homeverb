//! Credential bytes stay inside trusted native code, outside serializable DTOs.

use crate::Operation;
use async_trait::async_trait;
use homeverb_domain::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use uuid::{Uuid, Variant};
use zeroize::Zeroizing;

/// An opaque, nonsecret reference. Never derive one from account information.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CredentialRef(Uuid);

impl CredentialRef {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for CredentialRef {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CredentialRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl FromStr for CredentialRef {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        let id = Uuid::parse_str(value).map_err(|_| Error::invalid())?;
        if id.get_version_num() != 4 || id.get_variant() != Variant::RFC4122 {
            return Err(Error::invalid());
        }
        Ok(Self(id))
    }
}

impl TryFrom<String> for CredentialRef {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        value.parse()
    }
}

impl From<CredentialRef> for String {
    fn from(reference: CredentialRef) -> Self {
        reference.to_string()
    }
}

/// Zeroizes its owned buffer on drop. Upstream and OS copies are not controlled here.
///
/// Credential bytes cannot be passed to JSON serialization:
/// ```compile_fail
/// use homeverb_core::SecretBytes;
/// let secret = SecretBytes::new(vec![0]);
/// let _ = serde_json::to_string(&secret);
/// ```
pub struct SecretBytes(Zeroizing<Vec<u8>>);

impl SecretBytes {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(Zeroizing::new(bytes))
    }

    pub fn expose(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for SecretBytes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretBytes([REDACTED])")
    }
}

#[async_trait]
pub trait SecretStore: Send + Sync {
    async fn get(
        &self,
        reference: &CredentialRef,
        operation: &Operation,
    ) -> Result<Option<SecretBytes>>;
    /// Replaces this exact reference. An uncertain mutation must not be retried automatically.
    async fn put(
        &self,
        reference: &CredentialRef,
        secret: &SecretBytes,
        operation: &Operation,
    ) -> Result<()>;
    /// Deletes the exact local entry without reading its secret; confirmed absence succeeds.
    /// Reject OS-reported unavailable/locked or ambiguous storage. This is not a lock-state probe
    /// and does not revoke provider access.
    async fn delete(&self, reference: &CredentialRef, operation: &Operation) -> Result<()>;
}

pub struct UnavailableSecretStore;

impl UnavailableSecretStore {
    async fn unavailable<T>(operation: &Operation) -> Result<T> {
        operation
            .run(async {
                Err(Error::new(
                    "SECRET_STORE_UNAVAILABLE",
                    "No secure credential provider is configured.",
                ))
            })
            .await
    }
}

#[async_trait]
impl SecretStore for UnavailableSecretStore {
    async fn get(&self, _: &CredentialRef, operation: &Operation) -> Result<Option<SecretBytes>> {
        Self::unavailable(operation).await
    }

    async fn put(&self, _: &CredentialRef, _: &SecretBytes, operation: &Operation) -> Result<()> {
        Self::unavailable(operation).await
    }

    async fn delete(&self, _: &CredentialRef, operation: &Operation) -> Result<()> {
        Self::unavailable(operation).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_util::sync::CancellationToken;

    #[test]
    fn references_round_trip_but_reject_account_strings_and_nonrandom_ids() {
        let reference = CredentialRef::new();
        let json = serde_json::to_string(&reference).unwrap();
        assert_eq!(
            serde_json::from_str::<CredentialRef>(&json).unwrap(),
            reference
        );
        for invalid in [
            "account@example.test",
            "00000000-0000-0000-0000-000000000000",
            "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
        ] {
            assert!(invalid.parse::<CredentialRef>().is_err());
            assert!(serde_json::from_value::<CredentialRef>(serde_json::json!(invalid)).is_err());
        }
    }

    #[test]
    fn debug_never_contains_secret_bytes_or_length() {
        for bytes in [vec![], vec![7; 64]] {
            assert_eq!(
                format!("{:?}", SecretBytes::new(bytes)),
                "SecretBytes([REDACTED])"
            );
        }
    }

    #[tokio::test]
    async fn unavailable_store_has_no_fallback_and_honors_cancellation() {
        let operation = Operation::new(1000, CancellationToken::new()).unwrap();
        let reference = CredentialRef::new();
        let secret = SecretBytes::new(vec![7]);
        assert_eq!(
            UnavailableSecretStore
                .get(&reference, &operation)
                .await
                .unwrap_err()
                .code,
            "SECRET_STORE_UNAVAILABLE"
        );
        assert_eq!(
            UnavailableSecretStore
                .put(&reference, &secret, &operation)
                .await
                .unwrap_err()
                .code,
            "SECRET_STORE_UNAVAILABLE"
        );
        assert_eq!(
            UnavailableSecretStore
                .delete(&reference, &operation)
                .await
                .unwrap_err()
                .code,
            "SECRET_STORE_UNAVAILABLE"
        );
        operation.cancellation.cancel();
        assert_eq!(
            UnavailableSecretStore
                .put(&reference, &secret, &operation)
                .await
                .unwrap_err()
                .code,
            "CANCELLED"
        );
    }
}
