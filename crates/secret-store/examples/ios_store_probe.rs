//! Test-only Simulator helper; never linked into the production native framework.
#![cfg(all(target_os = "ios", target_env = "sim"))]

use homeverb_core::{CredentialRef, Operation, SecretBytes, SecretStore};
use homeverb_domain::{Error, Result};
use homeverb_secrets::SystemSecretStore;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[repr(C)]
pub struct ProbeResult {
    code: u32,
    checks: u32,
}

fn operation() -> Operation {
    Operation::new(10_000, CancellationToken::new()).expect("Valid test budget")
}

fn check(checks: &mut u32, value: bool) -> Result<()> {
    if !value {
        return Err(Error::new(
            "TEST_ASSERTION",
            "The fictional store check failed.",
        ));
    }
    *checks += 1;
    Ok(())
}

async fn probe(phase: u32, profile: Uuid, checks: &mut u32) -> Result<()> {
    let store = SystemSecretStore::new(profile)?;
    // The UUID belongs only to this disposable run, never to a saved account.
    let reference: CredentialRef = profile.to_string().parse()?;
    let original = SecretBytes::new(vec![17, 29, 0, 255]);
    let replacement = SecretBytes::new(vec![31, 41, 0, 128]);
    if phase == 1 {
        check(checks, store.get(&reference, &operation()).await?.is_none())?;
        store.put(&reference, &original, &operation()).await?;
        check(checks, true)?;
        let reopened = SystemSecretStore::new(profile)?;
        check(
            checks,
            reopened
                .get(&reference, &operation())
                .await?
                .is_some_and(|v| v.expose() == original.expose()),
        )?;
        let other = SystemSecretStore::new(Uuid::new_v4())?;
        check(checks, other.get(&reference, &operation()).await?.is_none())?;
        let absent = CredentialRef::new();
        check(checks, store.get(&absent, &operation()).await?.is_none())?;
        store.put(&reference, &replacement, &operation()).await?;
        check(checks, true)?;
        let cancelled = operation();
        cancelled.cancellation.cancel();
        check(
            checks,
            store
                .get(&reference, &cancelled)
                .await
                .is_err_and(|e| e.code == "CANCELLED"),
        )?;
        check(
            checks,
            store
                .put(&reference, &original, &cancelled)
                .await
                .is_err_and(|e| e.code == "CANCELLED"),
        )?;
        check(
            checks,
            store
                .delete(&reference, &cancelled)
                .await
                .is_err_and(|e| e.code == "CANCELLED"),
        )?;
        check(
            checks,
            reopened
                .get(&reference, &operation())
                .await?
                .is_some_and(|v| v.expose() == replacement.expose()),
        )?;
    } else if phase == 2 {
        check(
            checks,
            store
                .get(&reference, &operation())
                .await?
                .is_some_and(|v| v.expose() == replacement.expose()),
        )?;
        store.delete(&reference, &operation()).await?;
        check(checks, true)?;
        check(checks, store.get(&reference, &operation()).await?.is_none())?;
        store.delete(&reference, &operation()).await?;
        check(checks, true)?;
    } else {
        return Err(Error::invalid());
    }
    Ok(())
}

/// Only the isolated test app calls this symbol with its fresh run UUID.
#[unsafe(no_mangle)]
pub extern "C" fn homeverb_test_ios_store(phase: u32, high: u64, low: u64) -> ProbeResult {
    let mut checks = 0;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| Error::new("TEST_RUNTIME", "The test runtime could not start."))?;
        runtime.block_on(probe(
            phase,
            Uuid::from_u128((u128::from(high) << 64) | u128::from(low)),
            &mut checks,
        ))
    }));
    let code = match result {
        Ok(Ok(())) => 0,
        Ok(Err(error)) => match error.code.as_str() {
            "SECRET_STORE_LOCKED" => 2,
            "SECRET_STORE_UNAVAILABLE" => 3,
            "SECRET_STORE_WRITE_UNKNOWN" => 4,
            "SECRET_STORE_CONFLICT" => 5,
            "CANCELLED" => 6,
            "DEADLINE_EXCEEDED" => 7,
            _ => 1,
        },
        Err(_) => 1,
    };
    ProbeResult { code, checks }
}
