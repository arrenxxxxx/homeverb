//! API 1.37 protected QR/device publication; API 1.36 logical calls remain compatible.
use super::*;
use homeverb_host::{
    ContinueAccountLoginRequest, ModernAccountReport, ModernAccountRequest, NativeModernQrLogin,
};

const SCAN_CAPACITY: usize = 8192;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum SessionOperation {
    Refresh,
    Recover,
    Renew,
    RefreshDevices,
    RecoverDevices,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SessionInput {
    schema_version: String,
    operation_id: String,
    timeout_ms: u64,
    account_id: String,
    operation: SessionOperation,
}
impl SessionInput {
    fn request(&self) -> homeverb_domain::Result<ModernAccountRequest> {
        Ok(ModernAccountRequest {
            schema_version: self.schema_version.clone(),
            operation_id: self.operation_id.clone(),
            timeout_ms: self.timeout_ms,
            account_id: self.account_id.parse().map_err(|_| Error::invalid())?,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContinueInput {
    schema_version: String,
    operation_id: String,
    timeout_ms: u64,
    challenge_id: u64,
    #[serde(default)]
    publish_devices: bool,
}
impl ContinueInput {
    fn request(self) -> ContinueAccountLoginRequest {
        ContinueAccountLoginRequest {
            schema_version: self.schema_version,
            operation_id: self.operation_id,
            timeout_ms: self.timeout_ms,
            challenge_id: self.challenge_id,
        }
    }
}

fn qr_output(
    result: homeverb_domain::Result<NativeModernQrLogin>,
    buffer: &mut [u8],
    discard: impl FnOnce(u64),
) -> Value {
    let result = result.and_then(|qr| {
        let bytes = qr.scan_payload.expose();
        if bytes.is_empty()
            || bytes.len() > buffer.len()
            || qr.challenge_id == 0
            || !(1..=180_000).contains(&qr.expires_in_ms)
        {
            discard(qr.challenge_id);
            return Err(Error::invalid());
        }
        buffer[..bytes.len()].copy_from_slice(bytes);
        Ok(json!({"kind":"qr","challenge_id":qr.challenge_id,
            "scan_length":bytes.len(),"expires_in_ms":qr.expires_in_ms}))
    });
    json!(Query::from_result(result))
}

unsafe fn scan_buffer<'a>(buffer: *mut u8, capacity: usize) -> Option<&'a mut [u8]> {
    if buffer.is_null() || capacity != SCAN_CAPACITY {
        return None;
    }
    let bytes = unsafe { std::slice::from_raw_parts_mut(buffer, capacity) };
    bytes.zeroize();
    Some(bytes)
}

/// Begin one explicit modern QR login and return scan data in protected memory only.
/// # Safety
/// Metadata is NULL or valid NUL-terminated UTF-8. Scan is NULL or exclusively
/// owned writable memory of exactly 8192 bytes, disjoint from metadata. Valid
/// output is cleared even on host/metadata failure; invalid lengths are never
/// dereferenced. The caller clears/frees it on every path and must not log,
/// persist, navigate to or export scan data. Free JSON with homeverb_string_free.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn homeverb_begin_modern_account_login(
    handle: u64,
    metadata: *const c_char,
    scan: *mut u8,
    scan_capacity: usize,
) -> *mut c_char {
    output(|| {
        let Some(buffer) = (unsafe { scan_buffer(scan, scan_capacity) }) else {
            return error(
                "INVALID_ARGUMENT",
                "Provide the bounded protected QR output buffer.",
            );
        };
        let Some(host) = host(handle) else {
            return error("HOST_CLOSED", "The native host handle is closed.");
        };
        let Some(input) = (unsafe { input_json::<ConnectRequest>(metadata) }) else {
            return error("INVALID_ARGUMENT", "Invalid modern sign-in metadata.");
        };
        qr_output(
            host.runtime
                .block_on(host.session.begin_modern_account_login(input)),
            buffer,
            |id| {
                host.session.discard_modern_account_login(id);
            },
        )
    })
}

/// Consume one modern QR continuation. No scan data, password or credential input.
/// # Safety
/// Metadata is NULL or valid NUL-terminated UTF-8 for the duration of the call.
/// Free the returned nonsecret JSON using homeverb_string_free.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn homeverb_continue_modern_account_login(
    handle: u64,
    metadata: *const c_char,
) -> *mut c_char {
    output(|| {
        let Some(host) = host(handle) else {
            return error("HOST_CLOSED", "The native host handle is closed.");
        };
        let Some(input) = (unsafe { input_json::<ContinueInput>(metadata) }) else {
            return error("INVALID_ARGUMENT", "Invalid modern continuation metadata.");
        };
        let publish_devices = input.publish_devices;
        let request = input.request();
        let result = host.runtime.block_on(async {
            if publish_devices {
                host.session.continue_modern_device_login(request).await
                    .map(|report| json!({"kind":"connected_devices","report":report}))
            } else {
                host.session.continue_modern_account_login(request).await
                    .map(|report| json!({"kind":"connected","report":ModernAccountReport::connected(report)}))
            }
        });
        json!(Query::from_result(result))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn homeverb_discard_modern_account_login(handle: u64, challenge_id: u64) -> bool {
    host(handle).is_some_and(|host| host.session.discard_modern_account_login(challenge_id))
}

/// One explicit refresh, pending-session recovery or credential renewal; no retry.
/// # Safety
/// Metadata is NULL or valid NUL-terminated UTF-8 for the duration of the call.
/// Free the returned nonsecret JSON using homeverb_string_free.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn homeverb_modern_account_session(
    handle: u64,
    metadata: *const c_char,
) -> *mut c_char {
    output(|| {
        let Some(host) = host(handle) else {
            return error("HOST_CLOSED", "The native host handle is closed.");
        };
        let Some(input) = (unsafe { input_json::<SessionInput>(metadata) }) else {
            return error("INVALID_ARGUMENT", "Invalid modern session metadata.");
        };
        let result = host.runtime.block_on(async {
            let request = input.request()?;
            match input.operation {
                SessionOperation::Refresh => host.session.refresh_modern_account(request).await
                    .map(|report| json!({"kind":"directory","report":ModernAccountReport::inspected(report)})),
                SessionOperation::Recover => host.session.recover_modern_account(request).await
                    .map(|report| json!({"kind":"directory","report":ModernAccountReport::inspected(report)})),
                SessionOperation::RefreshDevices => host.session.refresh_modern_account_devices(request).await
                    .map(|report| json!({"kind":"devices","report":report})),
                SessionOperation::RecoverDevices => host.session.recover_modern_account_devices(request).await
                    .map(|report| json!({"kind":"devices","report":report})),
                SessionOperation::Renew => host.session.renew_modern_account(request).await
                    .map(|account| json!({"kind":"renewed","account":account})),
            }
        });
        json!(Query::from_result(result))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn qr_projection_uses_only_protected_bytes_and_exact_nonsecret_metadata() {
        let mut bytes = vec![0; SCAN_CAPACITY];
        let value = qr_output(
            Ok(NativeModernQrLogin {
                challenge_id: 7,
                scan_payload: SecretBytes::new(b"PRIVATE-FICTIONAL-QR".to_vec()),
                expires_in_ms: 1000,
            }),
            &mut bytes,
            |_| panic!("Valid projection must keep its handle"),
        );
        assert_eq!(
            value["data"],
            json!({"kind":"qr","challenge_id":7,"scan_length":20,"expires_in_ms":1000})
        );
        assert_eq!(&bytes[..20], b"PRIVATE-FICTIONAL-QR");
        assert!(bytes[20..].iter().all(|b| *b == 0));
        assert!(!value.to_string().contains("PRIVATE"));
    }
    #[test]
    fn exhausted_or_invalid_qr_lifetime_discards_before_copying_protected_data() {
        for (id, expiry) in [(7, 0), (7, 180_001), (0, 1000)] {
            let mut bytes = vec![0; SCAN_CAPACITY];
            let mut discarded = None;
            let value = qr_output(
                Ok(NativeModernQrLogin {
                    challenge_id: id,
                    scan_payload: SecretBytes::new(b"PRIVATE-FICTIONAL-QR".to_vec()),
                    expires_in_ms: expiry,
                }),
                &mut bytes,
                |id| discarded = Some(id),
            );
            assert_eq!(discarded, Some(id));
            assert_eq!(value["error"]["code"], "INVALID_ARGUMENT");
            assert!(bytes.iter().all(|b| *b == 0));
        }
    }

    #[test]
    fn invalid_qr_projection_discards_without_returning_truncated_scan_data() {
        for length in [0, SCAN_CAPACITY + 1] {
            let mut bytes = vec![0; SCAN_CAPACITY];
            let mut discarded = None;
            let value = qr_output(
                Ok(NativeModernQrLogin {
                    challenge_id: 7,
                    scan_payload: SecretBytes::new(vec![1; length]),
                    expires_in_ms: 1000,
                }),
                &mut bytes,
                |id| discarded = Some(id),
            );
            assert_eq!(discarded, Some(7));
            assert_eq!(value["error"]["code"], "INVALID_ARGUMENT");
            assert!(bytes.iter().all(|b| *b == 0));
        }
    }
}
