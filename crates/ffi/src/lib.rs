//! C ABI for foreground hosts. Calls run on a host worker thread, never the UI thread.

use homeverb_domain::{Error, Query};
use homeverb_host::{HostRequest, Session};
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
    allow_mock_writes: bool,
    #[serde(default)]
    scenario: Scenario,
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

/// Open an explicit Mock host. Configuration comes from trusted application startup.
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
        let core = match homeverb_host::open_mock(
            &config.data_dir,
            config.scenario,
            config.allow_mock_writes,
        ) {
            Ok(core) => core,
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
                hosts.insert(
                    id,
                    Arc::new(Host {
                        session: Session::new(core),
                        runtime,
                    }),
                );
            }
            Err(_) => return error("INTERNAL_ERROR", "The host registry is unavailable."),
        }
        json!(Query::from_result(Ok(
            json!({"handle": id, "adapter": "homeverb.mock"})
        )))
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
