//! Dedicated native verification prompts with protected binary output, never URLs in JSON.
use super::*;
use homeverb_host::{
    ContinueAccountLoginRequest, NativeVerificationLogin, NativeVerificationPrompt,
    VerificationAnswer, VerificationMethod,
};

const PROMPT_CAPACITY: usize = 131_072;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum AnswerKind {
    Captcha,
    Phone,
    Email,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Continue {
    schema_version: String,
    operation_id: String,
    timeout_ms: u64,
    challenge_id: u64,
    answer_kind: AnswerKind,
}

fn verification_output(
    result: homeverb_domain::Result<NativeVerificationLogin>,
    prompt: &mut [u8],
    discard: impl FnOnce(u64),
) -> Value {
    let value = result.and_then(|outcome| match outcome {
        NativeVerificationLogin::Connected(report) => {
            Ok(json!({"kind":"connected", "report":report}))
        }
        NativeVerificationLogin::Pending {
            challenge_id,
            prompt: protected,
            expires_in_ms,
        } => {
            let (bytes, mut metadata) = match protected {
                NativeVerificationPrompt::Captcha { image, media_type } => {
                    (image, json!({"kind":"captcha", "media_type":media_type}))
                }
                NativeVerificationPrompt::Verification {
                    provider_page,
                    methods,
                } => {
                    let methods: Vec<_> = methods
                        .iter()
                        .map(|method| match method {
                            VerificationMethod::Phone => "phone",
                            VerificationMethod::Email => "email",
                        })
                        .collect();
                    (
                        provider_page,
                        json!({"kind":"verification", "methods":methods}),
                    )
                }
            };
            if bytes.expose().is_empty() || bytes.expose().len() > prompt.len() {
                discard(challenge_id);
                return Err(Error::invalid());
            }
            prompt[..bytes.expose().len()].copy_from_slice(bytes.expose());
            metadata["prompt_length"] = json!(bytes.expose().len());
            metadata["challenge_id"] = json!(challenge_id);
            metadata["expires_in_ms"] = json!(expires_in_ms);
            Ok(metadata)
        }
    });
    json!(Query::from_result(value))
}

unsafe fn protected_prompt<'a>(value: *mut u8, capacity: usize) -> Option<&'a mut [u8]> {
    if value.is_null() || capacity != PROMPT_CAPACITY {
        return None;
    }
    let bytes = unsafe { std::slice::from_raw_parts_mut(value, capacity) };
    bytes.zeroize();
    Some(bytes)
}

/// Begin opt-in native verification; URLs/images use the protected output buffer.
/// # Safety
/// Metadata is NULL or valid NUL-terminated UTF-8. Login/password pointers are
/// NULL or exclusively owned writable buffers of their declared 1..256-byte size.
/// Prompt is NULL or exclusively owned writable memory of exactly 131072 bytes.
/// All inputs/output must be nonoverlapping. Valid inputs/output are cleared even
/// on admission failures. Invalid lengths are never dereferenced. The caller must
/// clear/free all buffers on every path and never log/cache/export prompt bytes.
/// Free the returned nonsecret JSON with homeverb_string_free.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn homeverb_begin_account_verification(
    handle: u64,
    metadata: *const c_char,
    login: *mut u8,
    login_length: usize,
    password: *mut u8,
    password_length: usize,
    prompt: *mut u8,
    prompt_capacity: usize,
) -> *mut c_char {
    output(|| {
        let login = unsafe { take_secret(login, login_length) };
        let password = unsafe { take_secret(password, password_length) };
        let prompt = unsafe { protected_prompt(prompt, prompt_capacity) };
        let (Some(login), Some(password), Some(prompt)) = (login, password, prompt) else {
            return error(
                "INVALID_ARGUMENT",
                "Provide bounded protected sign-in and prompt buffers.",
            );
        };
        let Some(host) = host(handle) else {
            return error("HOST_CLOSED", "The native host handle is closed.");
        };
        let Some(input) = (unsafe { input_json::<ConnectRequest>(metadata) }) else {
            return error(
                "INVALID_ARGUMENT",
                "Invalid protected verification metadata.",
            );
        };
        verification_output(
            host.runtime.block_on(
                host.session
                    .begin_account_verification(input, login, password),
            ),
            prompt,
            |id| {
                host.session.discard_account_verification(id);
            },
        )
    })
}

/// Consume one explicit captcha/phone/email answer, with protected prompt output.
/// # Safety
/// Metadata is NULL or valid NUL-terminated UTF-8. Answer is NULL or exclusively
/// owned writable memory of its declared 1..2048-byte size. Prompt is NULL or
/// exclusively owned writable memory of exactly 131072 bytes. No buffers overlap.
/// Valid answer/prompt buffers are cleared even on admission failures. Invalid
/// lengths are not dereferenced; the caller clears/frees everything on every path.
/// Never log/persist prompt or answer material. Free JSON with homeverb_string_free.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn homeverb_continue_account_verification(
    handle: u64,
    metadata: *const c_char,
    answer: *mut u8,
    answer_length: usize,
    prompt: *mut u8,
    prompt_capacity: usize,
) -> *mut c_char {
    output(|| {
        let answer = if answer.is_null() || !(1..=2048).contains(&answer_length) {
            None
        } else {
            let bytes = unsafe { std::slice::from_raw_parts_mut(answer, answer_length) };
            let secret = SecretBytes::new(bytes.to_vec());
            bytes.zeroize();
            Some(secret)
        };
        let prompt = unsafe { protected_prompt(prompt, prompt_capacity) };
        let (Some(answer), Some(prompt)) = (answer, prompt) else {
            return error(
                "INVALID_ARGUMENT",
                "Provide bounded protected answer and prompt buffers.",
            );
        };
        let Some(host) = host(handle) else {
            return error("HOST_CLOSED", "The native host handle is closed.");
        };
        let Some(input) = (unsafe { input_json::<Continue>(metadata) }) else {
            return error(
                "INVALID_ARGUMENT",
                "Invalid protected verification continuation metadata.",
            );
        };
        let answer = match input.answer_kind {
            AnswerKind::Captcha => VerificationAnswer::Captcha(answer),
            AnswerKind::Phone => VerificationAnswer::Ticket {
                method: VerificationMethod::Phone,
                ticket: answer,
            },
            AnswerKind::Email => VerificationAnswer::Ticket {
                method: VerificationMethod::Email,
                ticket: answer,
            },
        };
        let input = ContinueAccountLoginRequest {
            schema_version: input.schema_version,
            operation_id: input.operation_id,
            timeout_ms: input.timeout_ms,
            challenge_id: input.challenge_id,
        };
        verification_output(
            host.runtime
                .block_on(host.session.continue_account_verification(input, answer)),
            prompt,
            |id| {
                host.session.discard_account_verification(id);
            },
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn homeverb_discard_account_verification(handle: u64, challenge_id: u64) -> bool {
    host(handle).is_some_and(|host| host.session.discard_account_verification(challenge_id))
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum CodeMethod {
    Phone,
    Email,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CodeRequest {
    schema_version: String,
    operation_id: String,
    timeout_ms: u64,
    challenge_id: u64,
    method: CodeMethod,
}

/// Explicitly request one verification code. No ordinary JSON tool equivalent.
/// # Safety
/// Metadata is NULL or valid NUL-terminated UTF-8. Prompt is NULL or exclusively
/// owned writable memory of exactly 131072 bytes, nonoverlapping with metadata.
/// Valid output is cleared before admission; invalid lengths are not dereferenced.
/// The caller clears/frees all output on every path, and frees nonsecret JSON with
/// homeverb_string_free. Never log/cache/serialize the protected prompt.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn homeverb_request_account_verification_code(
    handle: u64,
    metadata: *const c_char,
    prompt: *mut u8,
    prompt_capacity: usize,
) -> *mut c_char {
    output(|| {
        let Some(prompt) = (unsafe { protected_prompt(prompt, prompt_capacity) }) else {
            return error(
                "INVALID_ARGUMENT",
                "Provide the protected prompt output buffer.",
            );
        };
        let Some(host) = host(handle) else {
            return error("HOST_CLOSED", "The native host handle is closed.");
        };
        let Some(input) = (unsafe { input_json::<CodeRequest>(metadata) }) else {
            return error(
                "INVALID_ARGUMENT",
                "Invalid verification code request metadata.",
            );
        };
        let method = match input.method {
            CodeMethod::Phone => VerificationMethod::Phone,
            CodeMethod::Email => VerificationMethod::Email,
        };
        let input = ContinueAccountLoginRequest {
            schema_version: input.schema_version,
            operation_id: input.operation_id,
            timeout_ms: input.timeout_ms,
            challenge_id: input.challenge_id,
        };
        match host.runtime.block_on(
            host.session
                .request_account_verification_code(input, method),
        ) {
            Err(error) => verification_output(Err(error), prompt, |_| {}),
            Ok(result) => {
                let mut output = verification_output(Ok(result.login), prompt, |id| {
                    host.session.discard_account_verification(id);
                });
                if output["error"].is_null() {
                    output["data"]["code_requests"] = json!(result.requests);
                    output["data"]["cooldown_ms"] = json!(result.cooldown_ms);
                }
                output
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn protected_prompts_never_enter_json_and_match_exact_output_length() {
        for verification in [false, true] {
            let bytes = if verification {
                b"https://account.xiaomi.com/fe/service/identity/authStart?PRIVATE-FICTIONAL"
                    .to_vec()
            } else {
                b"PRIVATE-FICTIONAL-IMAGE".to_vec()
            };
            let prompt = if verification {
                NativeVerificationPrompt::Verification {
                    provider_page: SecretBytes::new(bytes.clone()),
                    methods: vec![VerificationMethod::Phone, VerificationMethod::Email],
                }
            } else {
                NativeVerificationPrompt::Captcha {
                    image: SecretBytes::new(bytes.clone()),
                    media_type: "image/png",
                }
            };
            let mut output = vec![0; PROMPT_CAPACITY];
            let result = verification_output(
                Ok(NativeVerificationLogin::Pending {
                    challenge_id: 7,
                    prompt,
                    expires_in_ms: 1000,
                }),
                &mut output,
                |_| panic!("Unexpected discard"),
            );
            assert!(!result.to_string().contains("PRIVATE"));
            assert!(!result.to_string().contains("https://"));
            assert_eq!(result["data"]["prompt_length"], bytes.len());
            assert_eq!(&output[..bytes.len()], bytes);
            assert!(output[bytes.len()..].iter().all(|b| *b == 0));
            assert_eq!(
                result["data"]["kind"],
                if verification {
                    "verification"
                } else {
                    "captcha"
                }
            );
        }
    }
    #[test]
    fn projection_failure_discards_instead_of_returning_a_truncated_prompt() {
        let mut output = [0; 1];
        let mut discarded = None;
        let result = verification_output(
            Ok(NativeVerificationLogin::Pending {
                challenge_id: 7,
                prompt: NativeVerificationPrompt::Captcha {
                    image: SecretBytes::new(vec![1, 2]),
                    media_type: "image/png",
                },
                expires_in_ms: 1000,
            }),
            &mut output,
            |id| discarded = Some(id),
        );
        assert_eq!(discarded, Some(7));
        assert_eq!(result["error"]["code"], "INVALID_ARGUMENT");
        assert_eq!(output, [0]);
    }
}
