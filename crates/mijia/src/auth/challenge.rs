//! Explicit user answers continue one protected login, never an automatic retry.
use super::*;
use crate::{CaptchaRequest, CaptchaResponse};
use std::time::Duration;
use zeroize::Zeroizing;

const MAX_ANSWERS: u8 = 3;

/// Native-only outcome; neither sessions nor challenges enter general JSON.
#[derive(Debug)]
pub enum LoginOutcome {
    Connected(MijiaSession),
    Captcha(CaptchaChallenge),
}

mod verification;
pub use verification::{VerificationChallenge, VerificationMethod};

/// Opt-in native protocol outcome. Existing account/native entry points remain captcha-only.
#[derive(Debug)]
pub enum VerificationLoginOutcome {
    Connected(MijiaSession),
    Captcha(CaptchaChallenge),
    Verification(VerificationChallenge),
}
impl VerificationLoginOutcome {
    fn captcha_only(self) -> Result<LoginOutcome> {
        match self {
            Self::Connected(v) => Ok(LoginOutcome::Connected(v)),
            Self::Captcha(v) => Ok(LoginOutcome::Captcha(v)),
            Self::Verification(_) => Err(required()),
        }
    }
}

struct Context {
    account: SecretBytes,
    hash: PrivateString,
    sign: PrivateString,
    cookie: PrivateString,
    region: Region,
    lifetime: Operation,
    remaining: u8,
    verification_cookies: Option<Box<verification::Cookies>>,
}
impl Context {
    fn password_request(&self, answer: Option<(&str, &str)>) -> Result<CloudRequest> {
        let account = std::str::from_utf8(self.account.expose()).map_err(|_| Error::invalid())?;
        let mut fields = vec![
            ("sid", "xiaomiio"),
            ("hash", self.hash.as_str()),
            ("callback", "https://sts.api.io.mi.com/sts"),
            ("qs", "%3Fsid%3Dxiaomiio%26_json%3Dtrue"),
            ("user", account),
            ("_json", "true"),
            ("_sign", self.sign.as_str()),
        ];
        let cookie = if let Some((answer, cookie)) = answer {
            fields.push(("captCode", answer));
            PrivateString::new(format!("{}; ick={cookie}", self.cookie.as_str()))
        } else {
            self.cookie.clone()
        };
        Ok(CloudRequest {
            endpoint: CloudEndpoint::LoginPassword,
            region: self.region,
            cookie,
            location: None,
            body: codec::form(&fields),
        })
    }
}

/// Owned, single-use, expiring native challenge. Drop to discard its protected state.
/// ```compile_fail
/// fn export(value: &homeverb_mijia::CaptchaChallenge) {
///     let _ = serde_json::to_string(value);
/// }
/// ```
pub struct CaptchaChallenge {
    context: Context,
    response: CaptchaResponse,
}
impl fmt::Debug for CaptchaChallenge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CaptchaChallenge([REDACTED])")
    }
}
impl CaptchaChallenge {
    /// Protected image bytes for a trusted native image view, not logging/export.
    pub fn image(&self) -> &[u8] {
        self.response.image()
    }
    pub fn media_type(&self) -> &'static str {
        self.response.media_type()
    }
    pub fn expires_at(&self) -> tokio::time::Instant {
        self.context.lifetime.deadline
    }

    pub async fn answer(
        self,
        transport: &dyn CloudTransport,
        answer: SecretBytes,
        operation: &Operation,
    ) -> Result<LoginOutcome> {
        self.answer_extended(transport, answer, operation, false)
            .await?
            .captcha_only()
    }

    pub async fn answer_with_verification(
        self,
        transport: &dyn CloudTransport,
        answer: SecretBytes,
        operation: &Operation,
    ) -> Result<VerificationLoginOutcome> {
        self.answer_extended(transport, answer, operation, true)
            .await
    }

    async fn answer_extended(
        self,
        transport: &dyn CloudTransport,
        answer: SecretBytes,
        operation: &Operation,
        verification: bool,
    ) -> Result<VerificationLoginOutcome> {
        let lifetime = self.context.lifetime.clone();
        lifetime
            .run(operation.run(async {
                let text = std::str::from_utf8(answer.expose()).map_err(|_| Error::invalid())?;
                if text.is_empty()
                    || text.len() > 32
                    || !text.bytes().all(|b| b.is_ascii_alphanumeric())
                {
                    return Err(Error::invalid());
                }
                let mut context = self.context;
                if context.remaining == 0 {
                    return Err(required());
                }
                context.remaining -= 1;
                let request =
                    context.password_request(Some((text, self.response.cookie.as_str())))?;
                drop(answer);
                let response = verification::send_login(
                    transport,
                    &request,
                    &mut context.verification_cookies,
                    operation,
                )
                .await?;
                finish(transport, context, response, operation, true, verification).await
            }))
            .await
    }
}

pub(super) fn required() -> Error {
    Error::new(
        "MIJIA_AUTH_CHALLENGE_REQUIRED",
        "Mijia requires interactive account verification; this login has not completed.",
    )
}

pub async fn begin_login(
    transport: &dyn CloudTransport,
    credentials: LoginCredentials,
    region: Region,
    operation: &Operation,
) -> Result<LoginOutcome> {
    start(transport, credentials, region, operation, true).await
}

/// Opt-in protocol entry point. Only trusted callers may handle provider-page material.
pub async fn begin_verification_login(
    transport: &dyn CloudTransport,
    credentials: LoginCredentials,
    region: Region,
    operation: &Operation,
) -> Result<VerificationLoginOutcome> {
    start_extended(transport, credentials, region, operation, true, true).await
}

pub(super) async fn start(
    transport: &dyn CloudTransport,
    credentials: LoginCredentials,
    region: Region,
    operation: &Operation,
    interactive: bool,
) -> Result<LoginOutcome> {
    start_extended(
        transport,
        credentials,
        region,
        operation,
        interactive,
        false,
    )
    .await?
    .captcha_only()
}

async fn start_extended(
    transport: &dyn CloudTransport,
    credentials: LoginCredentials,
    region: Region,
    operation: &Operation,
    interactive: bool,
    verification: bool,
) -> Result<VerificationLoginOutcome> {
    operation
        .run(async {
            let device = uuid::Uuid::new_v4().simple().to_string();
            let cookie = PrivateString::new(format!("sdkVersion=3.8.6; deviceId={device}"));
            let sign_request = CloudRequest {
                endpoint: CloudEndpoint::LoginSign,
                region,
                cookie: cookie.clone(),
                location: None,
                body: SecretBytes::new(vec![]),
            };
            let mut verification_cookies = verification
                .then(|| verification::Cookies::new(cookie.as_str()))
                .transpose()?
                .map(Box::new);
            let response = verification::send_login(
                transport,
                &sign_request,
                &mut verification_cookies,
                operation,
            )
            .await?;
            let sign: Sign = decode(response.body.expose())?;
            if !crate::bounded_text(sign.sign.as_str(), 4096) {
                return Err(invalid());
            }
            use md5::Digest;
            let hash = Zeroizing::new(
                md5::Md5::digest(credentials.password.expose())
                    .iter()
                    .map(|byte| format!("{byte:02X}"))
                    .collect::<String>(),
            );
            let LoginCredentials { account, password } = credentials;
            drop(password);
            let mut lifetime = operation.clone();
            lifetime.deadline = tokio::time::Instant::now() + Duration::from_secs(180);
            let mut context = Context {
                account,
                hash: PrivateString(hash),
                sign: sign.sign,
                cookie,
                region,
                lifetime,
                remaining: MAX_ANSWERS,
                verification_cookies,
            };
            let request = context.password_request(None)?;
            let response = verification::send_login(
                transport,
                &request,
                &mut context.verification_cookies,
                operation,
            )
            .await?;
            finish(
                transport,
                context,
                response,
                operation,
                interactive,
                verification,
            )
            .await
        })
        .await
}

async fn finish(
    transport: &dyn CloudTransport,
    context: Context,
    response: crate::CloudResponse,
    operation: &Operation,
    interactive: bool,
    verification: bool,
) -> Result<VerificationLoginOutcome> {
    let status: LoginStatus = decode(response.body.expose())?;
    if let Some(page) = status.notification {
        if !interactive || !verification {
            return Err(required());
        }
        return VerificationChallenge::begin(transport, context, page, operation)
            .await
            .map(VerificationLoginOutcome::Verification);
    }
    if let Some(location) = status.captcha {
        if !interactive || context.remaining == 0 {
            return Err(required());
        }
        let cookie = if let Some(jar) = &context.verification_cookies {
            jar.header(&crate::transport::verification::account_url(
                location.as_str(),
            )?)
        } else {
            context.cookie.clone()
        };
        let request = CaptchaRequest::new(location, cookie)?;
        let response = transport.fetch_captcha(&request, operation).await?;
        return Ok(VerificationLoginOutcome::Captcha(CaptchaChallenge {
            context,
            response,
        }));
    }
    check_code(status.code)?;
    let result: LoginResult = decode(response.body.expose())?;
    crate::transport::exchange_url(result.location.as_str())?;
    let user_id = match result.user_id {
        Subject::Text(s) => s,
        Subject::Number(n) => PrivateString::new(n.to_string()),
    };
    if user_id.as_str().len() > 32
        || user_id.as_str().is_empty()
        || !user_id.as_str().bytes().all(|b| b.is_ascii_digit())
        || user_id.as_str().bytes().all(|b| b == b'0')
        || codec::unbase64(result.ssecurity.as_str())?.len() != 16
    {
        return Err(invalid());
    }
    if let Some(mut cookies) = context.verification_cookies {
        if cookies.subject().is_some_and(|v| v != user_id.as_str()) {
            return Err(invalid());
        }
        let request = crate::VerificationRequest {
            endpoint: crate::VerificationEndpoint::Exchange,
            location: result.location,
            cookie: PrivateString::new(String::new()),
            body: SecretBytes::new(vec![]),
        };
        let url = request.url()?;
        let response = transport.send_verification(&request, operation).await?;
        if response.status != 200 || response.location.is_some() {
            return Err(invalid());
        }
        cookies.absorb(&url, response.cookies)?;
        if cookies.subject().is_some_and(|v| v != user_id.as_str()) {
            return Err(invalid());
        }
        let token = cookies.token().ok_or_else(auth_required)?;
        return Ok(VerificationLoginOutcome::Connected(MijiaSession {
            region: context.region,
            user_id,
            security: result.ssecurity,
            token: PrivateString::new(token.to_owned()),
        }));
    }
    let exchange = CloudRequest {
        endpoint: CloudEndpoint::LoginExchange,
        region: context.region,
        location: Some(result.location),
        cookie: context.cookie,
        body: SecretBytes::new(vec![]),
    };
    let response = transport.send(&exchange, operation).await?;
    let token = response.service_token.ok_or_else(auth_required)?;
    let token = std::str::from_utf8(token.expose()).map_err(|_| invalid())?;
    if !cookie_value(token) {
        return Err(invalid());
    }
    Ok(VerificationLoginOutcome::Connected(MijiaSession {
        region: context.region,
        user_id,
        security: result.ssecurity,
        token: PrivateString::new(token.to_owned()),
    }))
}

#[cfg(test)]
mod tests;
