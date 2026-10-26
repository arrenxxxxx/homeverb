//! Explicit user-driven code delivery; no automatic send, resend or risk challenge retry.
use super::*;
use tokio::time::Instant;

#[derive(Default)]
pub(super) struct Delivery {
    attempts: [u8; 2],
    next_allowed: Option<Instant>,
}
impl VerificationMethod {
    fn delivery_index(self) -> usize {
        match self {
            Self::Phone => 0,
            Self::Email => 1,
        }
    }
    fn info_endpoint(self) -> Endpoint {
        match self {
            Self::Phone => Endpoint::PhoneInfo,
            Self::Email => Endpoint::EmailInfo,
        }
    }
    fn send_endpoint(self) -> Endpoint {
        match self {
            Self::Phone => Endpoint::SendPhone,
            Self::Email => Endpoint::SendEmail,
        }
    }
    fn address_type(self) -> &'static str {
        match self {
            Self::Phone => "PH",
            Self::Email => "EM",
        }
    }
}

impl VerificationChallenge {
    pub fn code_requests(&self) -> u8 {
        self.delivery.attempts.iter().sum()
    }
    pub fn code_request_cooldown(&self) -> Duration {
        self.delivery
            .next_allowed
            .map(|time| time.saturating_duration_since(Instant::now()))
            .unwrap_or_default()
    }

    /// Consume one explicitly requested send attempt. Failures never retain a replayable send.
    /// A successful provider response does not prove SMS/email delivery to the user.
    pub async fn request_code(
        mut self,
        transport: &dyn CloudTransport,
        method: VerificationMethod,
        operation: &Operation,
    ) -> Result<Self> {
        let lifetime = self.lifetime.clone();
        lifetime
            .run(operation.run(async {
                if !self.methods.contains(&method) {
                    return Err(Error::invalid());
                }
                if self.code_requests() >= 3 || !self.code_request_cooldown().is_zero() {
                    return Err(required());
                }
                let info_url = account_url(&format!(
                    "{}?_flag={}&_json=true",
                    method.path(),
                    method.flag()
                ))?;
                if !self.cookies.has_identity_session(&info_url) {
                    return Err(required());
                }
                let response = send(
                    transport,
                    method.info_endpoint(),
                    info_url.as_str(),
                    &self.cookies,
                    SecretBytes::new(vec![]),
                    operation,
                )
                .await?;
                check_response(&response)?;
                let info: MethodInfo = decode(response.body.expose())?;
                if info.code != 0 {
                    return Err(required());
                }
                self.cookies.absorb(&info_url, response.cookies)?;
                let content_type = info
                    .content_type
                    .as_ref()
                    .map(PrivateString::as_str)
                    .unwrap_or("160010");
                if content_type.is_empty()
                    || content_type.len() > 16
                    || !content_type.bytes().all(|b| b.is_ascii_digit())
                {
                    return Err(invalid());
                }
                let quota_url = account_url("/identity/pass/sms/userQuota")?;
                if !self.cookies.has_identity_session(&quota_url) {
                    return Err(required());
                }
                let body = codec::form(&[
                    ("addressType", method.address_type()),
                    ("contentType", content_type),
                    ("_json", "true"),
                ]);
                let response = send(
                    transport,
                    Endpoint::Quota,
                    quota_url.as_str(),
                    &self.cookies,
                    body,
                    operation,
                )
                .await?;
                check_response(&response)?;
                let quota: Quota = decode(response.body.expose())?;
                if quota.code != 0 || quota.result != 0 {
                    return Err(required());
                }
                let available = quota.info.value()?;
                if available == 0 {
                    return Err(required());
                }
                if available > 1_000_000 {
                    return Err(invalid());
                }
                self.cookies.absorb(&quota_url, response.cookies)?;
                let send_url = match method {
                    VerificationMethod::Phone => account_url("/identity/auth/sendPhoneTicket")?,
                    VerificationMethod::Email => account_url(&format!(
                        "/identity/auth/sendEmailTicket?_dc={}",
                        chrono::Utc::now().timestamp_millis()
                    ))?,
                };
                if !self.cookies.has_identity_session(&send_url) {
                    return Err(required());
                }
                let retry = self.delivery.attempts[method.delivery_index()].to_string();
                let body = codec::form(&[("retry", &retry), ("icode", ""), ("_json", "true")]);
                let response = send(
                    transport,
                    method.send_endpoint(),
                    send_url.as_str(),
                    &self.cookies,
                    body,
                    operation,
                )
                .await?;
                check_response(&response)?;
                let result: Sent = decode(response.body.expose())?;
                if result.code != 0 {
                    return Err(required());
                }
                let wait = result
                    .data
                    .and_then(|data| data.wait)
                    .map(|wait| wait.value())
                    .transpose()?
                    .unwrap_or(60)
                    .max(60);
                if wait > 86_400 {
                    return Err(invalid());
                }
                self.cookies.absorb(&send_url, response.cookies)?;
                self.delivery.attempts[method.delivery_index()] += 1;
                self.delivery.next_allowed = Some(Instant::now() + Duration::from_secs(wait));
                Ok(self)
            }))
            .await
    }
}
fn check_response(response: &VerificationResponse) -> Result<()> {
    if response.status != 200 || response.location.is_some() {
        return Err(invalid());
    }
    Ok(())
}
#[derive(Deserialize)]
struct MethodInfo {
    code: i64,
    #[serde(rename = "contentType")]
    content_type: Option<PrivateString>,
}
#[derive(Deserialize)]
struct Quota {
    code: i64,
    result: i64,
    info: Number,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum Number {
    Integer(u64),
    Text(PrivateString),
}
impl Number {
    fn value(self) -> Result<u64> {
        match self {
            Self::Integer(v) => Ok(v),
            Self::Text(v)
                if !v.as_str().is_empty()
                    && v.as_str().len() <= 16
                    && v.as_str().bytes().all(|b| b.is_ascii_digit()) =>
            {
                v.as_str().parse().map_err(|_| invalid())
            }
            _ => Err(invalid()),
        }
    }
}
#[derive(Deserialize)]
struct Sent {
    code: i64,
    data: Option<SentData>,
}
#[derive(Deserialize)]
struct SentData {
    #[serde(rename = "wt")]
    wait: Option<Number>,
}
