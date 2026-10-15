//! C ABI for foreground hosts. Calls run on a host worker thread, never the UI thread.

use homeverb_core::SecretBytes;
use homeverb_domain::{Error, Query};
use homeverb_host::{ConnectRequest, HostRequest, MeijuFanGrant, MeijuPowerGrant, Session};
use homeverb_mock::Scenario;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    ffi::{CStr, CString, c_char},
    path::PathBuf,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};
use zeroize::Zeroize;

#[cfg(target_os = "android")]
mod android;

struct Host {
    session: Session,
    runtime: tokio::runtime::Runtime,
}
static HOSTS: OnceLock<Mutex<HashMap<u64, Arc<Host>>>> = OnceLock::new();
static NEXT_ID: AtomicU64 = AtomicU64::new(1);
fn hosts() -> &'static Mutex<HashMap<u64, Arc<Host>>> {
    HOSTS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    schema_version: String,
    data_dir: PathBuf,
    #[serde(default)]
    adapter: AdapterMode,
    allow_mock_writes: bool,
    #[serde(default)]
    meiju_power_grants: Vec<MeijuPowerGrant>,
    #[serde(default)]
    meiju_fan_grants: Vec<MeijuFanGrant>,
    #[serde(default)]
    scenario: Scenario,
}

#[derive(Default, Deserialize, PartialEq, Eq)]
enum AdapterMode {
    #[default]
    #[serde(rename = "homeverb.mock")]
    Mock,
    #[serde(rename = "midea.meiju")]
    Meiju,
}

fn error(code: &str, message: &str) -> Value {
    json!(Query::<Value>::from_result(Err(Error::new(code, message))))
}
fn host(id: u64) -> Option<Arc<Host>> {
    hosts().lock().ok()?.get(&id).cloned()
}

fn output(work: impl FnOnce() -> Value) -> *mut c_char {
    let value = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)).unwrap_or_else(|_| {
        error(
            "INTERNAL_ERROR",
            "The native host could not complete the request.",
        )
    });
    CString::new(value.to_string())
        .expect("JSON escapes embedded NUL bytes")
        .into_raw()
}

/// # Safety
/// `input` must point to a valid NUL-terminated UTF-8 string for this call's lifetime.
unsafe fn input_json<T: serde::de::DeserializeOwned>(input: *const c_char) -> Option<T> {
    if input.is_null() {
        return None;
    }
    // The caller owns the input and keeps it alive until this function returns.
    let bytes = unsafe { CStr::from_ptr(input) }.to_bytes();
    if bytes.len() > 1_048_576 {
        return None;
    }
    serde_json::from_slice(bytes).ok()
}

/// Open an explicitly selected host. Configuration comes from trusted application startup.
/// Omitted adapter selection preserves the existing Mock default.
/// # Safety
/// `config` must be NULL or a valid NUL-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn homeverb_open(config: *const c_char) -> *mut c_char {
    output(|| {
        let Some(config) = (unsafe { input_json::<Config>(config) }) else {
            return error("INVALID_ARGUMENT", "Invalid host configuration.");
        };
        if config.schema_version != "1.0" || !config.data_dir.is_absolute() {
            return error(
                "INVALID_ARGUMENT",
                "Use schema version 1.0 and an absolute private data directory.",
            );
        }
        if config.adapter == AdapterMode::Meiju
            && (config.allow_mock_writes || config.scenario != Scenario::Normal)
        {
            return error(
                "INVALID_ARGUMENT",
                "Meiju startup does not accept Mock scenarios or grants.",
            );
        }
        if config.adapter != AdapterMode::Meiju
            && (!config.meiju_power_grants.is_empty() || !config.meiju_fan_grants.is_empty())
        {
            return error(
                "INVALID_ARGUMENT",
                "Meiju control grants require the Meiju adapter.",
            );
        }
        let (adapter, session) = match config.adapter {
            AdapterMode::Mock => (
                "homeverb.mock",
                homeverb_host::open_mock(
                    &config.data_dir,
                    config.scenario,
                    config.allow_mock_writes,
                )
                .map(Session::new),
            ),
            AdapterMode::Meiju => (
                "midea.meiju",
                homeverb_host::open_meiju_session_with_control_grants(
                    &config.data_dir,
                    &config.meiju_power_grants,
                    &config.meiju_fan_grants,
                ),
            ),
        };
        let session = match session {
            Ok(session) => session,
            Err(problem) => return json!(Query::<Value>::from_result(Err(problem))),
        };
        let runtime = match tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime,
            Err(_) => return error("INTERNAL_ERROR", "The native runtime could not start."),
        };
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        match hosts().lock() {
            Ok(mut hosts) => {
                hosts.insert(id, Arc::new(Host { session, runtime }));
            }
            Err(_) => return error("INTERNAL_ERROR", "The host registry is unavailable."),
        }
        json!(Query::from_result(Ok(
            json!({"handle": id, "adapter": adapter})
        )))
    })
}

/// Copy a bounded secret and erase the supplied buffer before any operation starts.
/// # Safety
/// A non-NULL pointer with length 1..=256 must be valid writable memory for this call.
unsafe fn take_secret(value: *mut u8, length: usize) -> Option<SecretBytes> {
    if value.is_null() || !(1..=256).contains(&length) {
        return None;
    }
    let bytes = unsafe { std::slice::from_raw_parts_mut(value, length) };
    let secret = SecretBytes::new(bytes.to_vec());
    bytes.zeroize();
    Some(secret)
}

/// Connect the selected account provider using trusted native input, outside general JSON.
/// Free the returned JSON with homeverb_string_free. The caller owns and clears/frees inputs.
/// # Safety
/// Metadata must be NULL or a valid NUL-terminated UTF-8 string. Each non-NULL secret pointer
/// with length 1..=256 must be valid writable memory for that length. Buffers and metadata
/// must not overlap.
/// No other thread may access these buffers during the call. Valid 1..=256 byte buffers
/// are copied and cleared even when metadata/host admission fails. Oversized inputs are
/// not dereferenced; caller cleanup is mandatory on every path.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn homeverb_connect_account(
    handle: u64,
    metadata: *const c_char,
    login: *mut u8,
    login_length: usize,
    password: *mut u8,
    password_length: usize,
) -> *mut c_char {
    output(|| {
        let login = unsafe { take_secret(login, login_length) };
        let password = unsafe { take_secret(password, password_length) };
        let (Some(login), Some(password)) = (login, password) else {
            return error(
                "MIDEA_LOGIN_INPUT_UNSUPPORTED",
                "Provide bounded native account and password buffers.",
            );
        };
        let Some(host) = host(handle) else {
            return error("HOST_CLOSED", "The native host handle is closed.");
        };
        let Some(request) = (unsafe { input_json::<ConnectRequest>(metadata) }) else {
            return error("INVALID_ARGUMENT", "Invalid account connection metadata.");
        };
        host.runtime
            .block_on(host.session.connect_account(request, login, password))
    })
}

/// Execute a request synchronously on a host worker thread. Free the returned string once.
/// # Safety
/// `request` must be NULL or a valid NUL-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn homeverb_call(handle: u64, request: *const c_char) -> *mut c_char {
    output(|| {
        let Some(host) = host(handle) else {
            return error("HOST_CLOSED", "The native host handle is closed.");
        };
        let Some(request) = (unsafe { input_json::<HostRequest>(request) }) else {
            return error("INVALID_ARGUMENT", "Invalid host request.");
        };
        host.runtime.block_on(host.session.call(request))
    })
}

/// # Safety
/// `operation_id` must be NULL or a valid NUL-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn homeverb_cancel(handle: u64, operation_id: *const c_char) -> bool {
    if operation_id.is_null() {
        return false;
    }
    let Ok(operation_id) = (unsafe { CStr::from_ptr(operation_id) }).to_str() else {
        return false;
    };
    host(handle).is_some_and(|host| host.session.cancel(operation_id))
}

#[unsafe(no_mangle)]
pub extern "C" fn homeverb_set_foreground(handle: u64, foreground: bool) {
    if let Some(host) = host(handle) {
        host.session.set_foreground(foreground);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn homeverb_close(handle: u64) {
    if let Ok(mut hosts) = hosts().lock()
        && let Some(host) = hosts.remove(&handle)
    {
        host.session.close();
    }
}

/// # Safety
/// `value` must be NULL or an unfreed pointer returned by homeverb_open/homeverb_call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn homeverb_string_free(value: *mut c_char) {
    if !value.is_null() {
        drop(unsafe { CString::from_raw(value) });
    }
}
