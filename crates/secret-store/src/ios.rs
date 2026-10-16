//! Operation-owned iOS Keychain storage. No detached work or public secret DTOs.

use homeverb_core::{CredentialRef, Operation, SecretBytes};
use homeverb_domain::{Error, Result};
use uuid::Uuid;

#[cfg(any(target_os = "ios", all(test, target_os = "macos")))]
mod keychain;
#[cfg(test)]
mod tests;

const MAX_BYTES: usize = 1_048_576;
const MAX_REFERENCE: usize = 4096;

pub(crate) enum Request<'a> {
    Get,
    Put(&'a SecretBytes),
    Delete,
}

fn unavailable() -> Error {
    Error::new(
        "SECRET_STORE_UNAVAILABLE",
        "The iOS credential store is unavailable.",
    )
}

fn unknown() -> Error {
    Error::new(
        "SECRET_STORE_WRITE_UNKNOWN",
        "The credential change may have completed; inspect the reference before recovery.",
    )
}

fn conflict() -> Error {
    Error::new(
        "SECRET_STORE_CONFLICT",
        "Multiple entries match this credential reference.",
    )
}

fn status_error(status: i32) -> Error {
    match status {
        // errSecInteractionNotAllowed and errSecAuthFailed; never expose OS text.
        -25308 | -25293 => Error::new(
            "SECRET_STORE_LOCKED",
            "Unlock the device and ensure a device passcode is set before trying again.",
        ),
        -25299 => conflict(),
        _ => unavailable(),
    }
}

struct Scope {
    service: String,
    account: String,
    access_group: String,
}

impl Scope {
    fn new(profile: Uuid, reference: &CredentialRef, group: Option<&str>) -> Result<Self> {
        let group = group.ok_or_else(unavailable)?;
        if group.len() > 255
            || !group.contains('.')
            || group.split('.').any(str::is_empty)
            || !group
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-'))
        {
            return Err(unavailable());
        }
        Ok(Self {
            service: format!("org.homeverb.credentials.v1.{profile}"),
            account: reference.to_string(),
            access_group: group.to_owned(),
        })
    }
}

struct Entry {
    matches_scope: bool,
    passcode_only: bool,
    synchronizable: bool,
    unsupported_access_control: bool,
    persistent_ref: SecretBytes,
}

impl Entry {
    fn validate(&self) -> Result<()> {
        if !self.matches_scope
            || !self.passcode_only
            || self.synchronizable
            || self.unsupported_access_control
            || self.persistent_ref.expose().is_empty()
            || self.persistent_ref.expose().len() > MAX_REFERENCE
        {
            return Err(unavailable());
        }
        Ok(())
    }
}

enum Change<'a> {
    Add(&'a SecretBytes),
    Update(&'a Entry, &'a SecretBytes),
    Delete(&'a Entry),
}

trait Backend {
    fn find(&mut self, scope: &Scope) -> Result<Vec<Entry>>;
    fn read(&mut self, scope: &Scope, entry: &Entry) -> Result<Option<SecretBytes>>;
    /// Set dispatched immediately before the OS mutation, after local preparation.
    fn change(
        &mut self,
        scope: &Scope,
        change: Change<'_>,
        operation: &Operation,
        dispatched: &mut bool,
    ) -> Result<()>;
}

fn before_dispatch(operation: &Operation) -> Result<()> {
    if operation.cancellation.is_cancelled() {
        return Err(Error::new("CANCELLED", "The operation was cancelled."));
    }
    if tokio::time::Instant::now() >= operation.deadline {
        return Err(Error::new(
            "DEADLINE_EXCEEDED",
            "The total operation budget expired.",
        ));
    }
    Ok(())
}

async fn operate(
    backend: &mut impl Backend,
    scope: &Scope,
    request: Request<'_>,
    operation: &Operation,
) -> Result<Option<SecretBytes>> {
    operation.run(async { Ok(()) }).await?;
    if matches!(&request, Request::Put(secret) if secret.expose().len() > MAX_BYTES) {
        return Err(Error::invalid());
    }
    // Synchronous OS calls retain this future and the caller's lease until return.
    let matches = backend.find(scope);
    let mut matches = operation.run(async { matches }).await?;
    if matches.len() > 1 {
        return Err(conflict());
    }
    let entry = matches.pop();
    if let Some(entry) = &entry {
        entry.validate()?;
    }
    let change = match (request, entry.as_ref()) {
        (Request::Get, Some(entry)) => {
            operation.run(async { Ok(()) }).await?;
            let result = backend.read(scope, entry);
            let result = operation.run(async { result }).await?;
            if result
                .as_ref()
                .is_some_and(|value| value.expose().len() > MAX_BYTES)
            {
                return Err(unavailable());
            }
            return Ok(result);
        }
        (Request::Get | Request::Delete, None) => return Ok(None),
        (Request::Put(secret), None) => Change::Add(secret),
        (Request::Put(secret), Some(entry)) => Change::Update(entry, secret),
        (Request::Delete, Some(entry)) => Change::Delete(entry),
    };
    operation.run(async { Ok(()) }).await?;
    let mut dispatched = false;
    let result = backend.change(scope, change, operation, &mut dispatched);
    let result = operation.run(async { result }).await;
    if dispatched && result.is_err() {
        Err(unknown())
    } else {
        result.map(|_| None)
    }
}

#[cfg(target_os = "ios")]
pub(crate) async fn perform(
    profile: Uuid,
    reference: &CredentialRef,
    request: Request<'_>,
    operation: &Operation,
) -> Result<Option<SecretBytes>> {
    operation.run(async { Ok(()) }).await?;
    let scope = Scope::new(
        profile,
        reference,
        option_env!("HOMEVERB_IOS_KEYCHAIN_ACCESS_GROUP"),
    )?;
    operate(&mut keychain::Keychain, &scope, request, operation).await
}
