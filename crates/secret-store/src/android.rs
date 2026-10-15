//! Trusted Java/JNI backend. No detached workers or public credential JSON.

use homeverb_core::{CredentialRef, Operation, SecretBytes};
use homeverb_domain::{Error, Result};
use jni::{
    JNIEnv, JavaVM,
    objects::{GlobalRef, JByteArray, JObject, JValue},
};
use std::sync::OnceLock;
use uuid::Uuid;

const MAX_BYTES: usize = 1_048_576;
const METHOD: &str = "(ILjava/lang/String;Ljava/lang/String;[B)Lorg/homeverb/CredentialResult;";

struct Backend {
    vm: JavaVM,
    object: GlobalRef,
}
static BACKEND: OnceLock<Backend> = OnceLock::new();

/// Install once from trusted Java startup. Initialization never reads a credential.
pub fn install(env: &mut JNIEnv<'_>, object: JObject<'_>) -> bool {
    let result = (|| -> jni::errors::Result<bool> {
        if object.is_null() || BACKEND.get().is_some() {
            return Ok(false);
        }
        let class = env.get_object_class(&object)?;
        env.get_method_id(class, "operate", METHOD)?;
        let backend = Backend {
            vm: env.get_java_vm()?,
            object: env.new_global_ref(object)?,
        };
        Ok(BACKEND.set(backend).is_ok())
    })();
    if result.is_err() {
        let _ = env.exception_clear();
    }
    result.unwrap_or(false)
}

pub(crate) enum Request<'a> {
    Get,
    Put(&'a SecretBytes),
    Delete,
}

fn unavailable() -> Error {
    Error::new(
        "SECRET_STORE_UNAVAILABLE",
        "The Android credential store is unavailable.",
    )
}

fn unknown() -> Error {
    Error::new(
        "SECRET_STORE_WRITE_UNKNOWN",
        "The credential change may have completed; inspect the reference before recovery.",
    )
}

fn decode(status: i32, value: Option<SecretBytes>, mutation: bool) -> Result<Option<SecretBytes>> {
    match (status, mutation) {
        (0, false) => value.map(Some).ok_or_else(unavailable),
        (0, true) if value.is_none() => Ok(None),
        (1, false) if value.is_none() => Ok(None),
        (2, _) => Err(unavailable()),
        (3, _) => Err(Error::new(
            "SECRET_STORE_LOCKED",
            "Unlock the device before trying again.",
        )),
        (4, _) => Err(Error::new(
            "RESOURCE_BUSY",
            "Another credential operation owns this profile.",
        )),
        (5, true) => Err(unknown()),
        (_, true) => Err(unknown()),
        _ => Err(unavailable()),
    }
}

fn invoke(
    backend: &Backend,
    profile: Uuid,
    reference: &CredentialRef,
    request: &Request<'_>,
) -> Result<Option<SecretBytes>> {
    let mutation = !matches!(request, Request::Get);
    let failed = || if mutation { unknown() } else { unavailable() };
    let mut env = backend.vm.attach_current_thread().map_err(|_| failed())?;
    let result = env.with_local_frame(16, |env| -> jni::errors::Result<_> {
        let profile = JObject::from(env.new_string(profile.to_string())?);
        let reference = JObject::from(env.new_string(reference.to_string())?);
        let (action, input) = match request {
            Request::Get => (0, JByteArray::default()),
            Request::Put(secret) => (1, env.byte_array_from_slice(secret.expose())?),
            Request::Delete => (2, JByteArray::default()),
        };
        let result = env.call_method(
            backend.object.as_obj(),
            "operate",
            METHOD,
            &[
                JValue::Int(action),
                JValue::Object(&profile),
                JValue::Object(&reference),
                JValue::Object(&input),
            ],
        );
        // Clear an exception before calling JNI again; never obtain or print its message.
        if result.is_err() {
            env.exception_clear()?;
        }
        if let Request::Put(secret) = request {
            env.set_byte_array_region(&input, 0, &vec![0; secret.expose().len()])?;
        }
        let output = result?.l()?;
        let status = env.get_field(&output, "status", "I")?.i()?;
        let bytes = JByteArray::from(env.get_field(&output, "value", "[B")?.l()?);
        let value = if bytes.is_null() {
            None
        } else {
            let length = env.get_array_length(&bytes)? as usize;
            if length > MAX_BYTES {
                return Ok(Err(failed()));
            }
            let secret = SecretBytes::new(env.convert_byte_array(&bytes)?);
            env.set_byte_array_region(&bytes, 0, &vec![0; length])?;
            Some(secret)
        };
        Ok(decode(status, value, mutation))
    });
    if result.is_err() {
        let _ = env.exception_clear();
    }
    result.map_err(|_| failed())?
}

pub(crate) async fn perform(
    profile: Uuid,
    reference: &CredentialRef,
    request: Request<'_>,
    operation: &Operation,
) -> Result<Option<SecretBytes>> {
    operation.run(async { Ok(()) }).await?;
    let backend = BACKEND.get().ok_or_else(unavailable)?;
    if matches!(&request, Request::Put(secret) if secret.expose().len() > MAX_BYTES) {
        return Err(Error::invalid());
    }
    let mutation = !matches!(request, Request::Get);
    // Keep the caller's lease and thread until the OS call returns. No late detached write.
    let result = invoke(backend, profile, reference, &request);
    operation.run(async { result }).await.map_err(|error| {
        if mutation && matches!(error.code.as_str(), "CANCELLED" | "DEADLINE_EXCEEDED") {
            unknown()
        } else {
            error
        }
    })
}

#[cfg(test)]
mod tests;
