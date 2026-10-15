//! Native credential storage. No secret bytes cross a public JSON host interface.

use async_trait::async_trait;
use homeverb_core::{CredentialRef, Operation, SecretBytes, SecretStore};
use homeverb_domain::{Error, Result};
use uuid::{Uuid, Variant};

#[cfg(target_os = "linux")]
mod linux;

#[cfg(any(target_os = "android", test))]
pub mod android;

/// Construction performs no I/O. The trusted host must persist its random profile namespace.
pub struct SystemSecretStore {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    profile: Uuid,
}

impl SystemSecretStore {
    pub fn new(profile: Uuid) -> Result<Self> {
        if profile.get_version_num() != 4 || profile.get_variant() != Variant::RFC4122 {
            return Err(Error::invalid());
        }
        Ok(Self {
            #[cfg(any(target_os = "linux", target_os = "android"))]
            profile,
        })
    }
}

#[async_trait]
impl SecretStore for SystemSecretStore {
    async fn get(
        &self,
        reference: &CredentialRef,
        operation: &Operation,
    ) -> Result<Option<SecretBytes>> {
        #[cfg(target_os = "linux")]
        {
            self.perform(reference, linux::Request::Get, operation)
                .await
        }
        #[cfg(target_os = "android")]
        {
            android::perform(self.profile, reference, android::Request::Get, operation).await
        }
        #[cfg(not(any(target_os = "linux", target_os = "android")))]
        {
            homeverb_core::UnavailableSecretStore
                .get(reference, operation)
                .await
        }
    }

    async fn put(
        &self,
        reference: &CredentialRef,
        secret: &SecretBytes,
        operation: &Operation,
    ) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            self.perform(reference, linux::Request::Put(secret), operation)
                .await
                .map(|_| ())
        }
        #[cfg(target_os = "android")]
        {
            android::perform(
                self.profile,
                reference,
                android::Request::Put(secret),
                operation,
            )
            .await
            .map(|_| ())
        }
        #[cfg(not(any(target_os = "linux", target_os = "android")))]
        {
            homeverb_core::UnavailableSecretStore
                .put(reference, secret, operation)
                .await
        }
    }

    async fn delete(&self, reference: &CredentialRef, operation: &Operation) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            self.perform(reference, linux::Request::Delete, operation)
                .await
                .map(|_| ())
        }
        #[cfg(target_os = "android")]
        {
            android::perform(self.profile, reference, android::Request::Delete, operation)
                .await
                .map(|_| ())
        }
        #[cfg(not(any(target_os = "linux", target_os = "android")))]
        {
            homeverb_core::UnavailableSecretStore
                .delete(reference, operation)
                .await
        }
    }
}
