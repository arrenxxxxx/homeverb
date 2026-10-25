//! Dedicated trusted UI bridge for user-driven Mijia authentication.
use super::*;
use base64::{Engine, engine::general_purpose::STANDARD};
use homeverb_host::{ContinueAccountLoginRequest, NativeAccountLogin};

fn login_output(result: homeverb_domain::Result<NativeAccountLogin>) -> Value {
    let value = result.map(|outcome| match outcome {
        NativeAccountLogin::Connected(report) => json!({"kind":"connected", "report":report}),
        NativeAccountLogin::Captcha {
            challenge_id,
            image,
            media_type,
            expires_in_ms,
        } => json!({
            "kind":"captcha", "challenge_id":challenge_id,
            "image_base64":STANDARD.encode(image.expose()),
            "media_type":media_type, "expires_in_ms":expires_in_ms,
        }),
    });
    json!(Query::from_result(value))
}

/// Begin protected foreground Mijia sign-in. No general JSON tool equivalent.
/// # Safety
/// Metadata is NULL or a valid NUL-terminated UTF-8 string. Each non-NULL secret
/// buffer of length 1..=256 is writable for that length, nonoverlapping with other
/// inputs, and exclusively owned for this call. Valid buffers are copied and
/// cleared even on metadata/host failure. Oversized buffers are not dereferenced;
/// the caller must clear/free all original inputs on every path. Free the returned
/// string using homeverb_string_free; do not log or persist challenge image data.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn homeverb_begin_account_login(
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
                "INVALID_ARGUMENT",
                "Provide bounded protected sign-in buffers.",
            );
        };
        let Some(host) = host(handle) else {
            return error("HOST_CLOSED", "The native host handle is closed.");
        };
        let Some(input) = (unsafe { input_json::<ConnectRequest>(metadata) }) else {
            return error("INVALID_ARGUMENT", "Invalid protected sign-in metadata.");
        };
        login_output(
            host.runtime
                .block_on(host.session.begin_account_login(input, login, password)),
        )
    })
}

/// Consume one captcha continuation with the user's protected answer.
/// # Safety
/// Metadata is NULL or a valid NUL-terminated UTF-8 string. The answer pointer is
/// NULL or exclusively owned writable memory for its declared length 1..=256;
/// it must not overlap metadata. Valid buffers are copied and cleared on every
/// admission path. Oversized buffers are not dereferenced; the caller clears and
/// frees inputs on every path. Answers must contain 1..=32 ASCII letters/digits.
/// Free returned strings using homeverb_string_free. Never log/persist challenges.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn homeverb_continue_account_login(
    handle: u64,
    metadata: *const c_char,
    answer: *mut u8,
    answer_length: usize,
) -> *mut c_char {
    output(|| {
        let Some(answer) = (unsafe { take_secret(answer, answer_length) }) else {
            return error(
                "INVALID_ARGUMENT",
                "Provide a bounded protected captcha answer.",
            );
        };
        let Some(host) = host(handle) else {
            return error("HOST_CLOSED", "The native host handle is closed.");
        };
        let Some(input) = (unsafe { input_json::<ContinueAccountLoginRequest>(metadata) }) else {
            return error("INVALID_ARGUMENT", "Invalid login continuation metadata.");
        };
        login_output(
            host.runtime
                .block_on(host.session.continue_account_login(input, answer)),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn homeverb_discard_account_login(handle: u64, challenge_id: u64) -> bool {
    host(handle).is_some_and(|host| host.session.discard_account_login(challenge_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dedicated_captcha_projection_has_only_ui_fields() {
        let data = login_output(Ok(NativeAccountLogin::Captcha {
            challenge_id: 7,
            image: SecretBytes::new(b"fictional-image".to_vec()),
            media_type: "image/png",
            expires_in_ms: 1000,
        }));
        assert_eq!(
            data["data"],
            json!({"kind":"captcha", "challenge_id":7,
            "image_base64":STANDARD.encode(b"fictional-image"), "media_type":"image/png", "expires_in_ms":1000})
        );
        assert!(data["error"].is_null());
    }
}
