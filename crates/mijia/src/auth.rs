//! Explicit native password login; credentials never enter general host JSON.

use crate::{
    CloudEndpoint, CloudRequest, CloudTransport, PrivateString, Region, auth_required, codec,
    cookie_value, decode, invalid,
};
use homeverb_core::{Operation, SecretBytes};
use homeverb_domain::{Error, Result};
use serde::Deserialize;
use std::fmt;
use zeroize::Zeroizing;

pub struct LoginCredentials {
    account: SecretBytes,
    password: SecretBytes,
}
impl fmt::Debug for LoginCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LoginCredentials([REDACTED])")
    }
}
impl LoginCredentials {
    pub fn new(account: SecretBytes, password: SecretBytes) -> Result<Self> {
        for value in [&account, &password] {
            let text = std::str::from_utf8(value.expose()).map_err(|_| Error::invalid())?;
            if !crate::bounded_text(text, 256) {
                return Err(Error::invalid());
            }
        }
        Ok(Self { account, password })
    }
}

/// User-scoped protected session; not serializable through ordinary host APIs.
pub struct MijiaSession {
    pub(crate) region: Region,
    pub(crate) user_id: PrivateString,
    pub(crate) security: PrivateString,
    pub(crate) token: PrivateString,
}
impl fmt::Debug for MijiaSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MijiaSession([REDACTED])")
    }
}
impl MijiaSession {
    pub fn region(&self) -> Region {
        self.region
    }
    pub(crate) async fn request(
        &self,
        transport: &dyn CloudTransport,
        endpoint: CloudEndpoint,
        data: &SecretBytes,
        operation: &Operation,
    ) -> Result<SecretBytes> {
        operation.run(async {
            if !matches!(endpoint, CloudEndpoint::Devices | CloudEndpoint::Properties | CloudEndpoint::Homes | CloudEndpoint::HomeDevices) { return Err(Error::invalid()); }
            if data.expose().len() > crate::MAX_RESPONSE_BYTES { return Err(Error::invalid()); }
            let nonce = codec::nonce()?;
            let data = std::str::from_utf8(data.expose()).map_err(|_| Error::invalid())?;
            let (body, key) = codec::encode(endpoint.path(), data, self.security.as_str(), &nonce)?;
            let request = CloudRequest {
                endpoint, region: self.region, location: None,
                cookie: PrivateString::new(format!("userId={}; serviceToken={}; yetAnotherServiceToken={}; locale=en_US; timezone=GMT%2B08%3A00; is_daylight=0; dst_offset=0; channel=MI_APP_STORE", self.user_id.as_str(), self.token.as_str(), self.token.as_str())),
                body,
            };
            let response = transport.send(&request, operation).await?;
            // Unencrypted provider errors are not successful property/inventory evidence.
            if response.body.expose().trim_ascii_start().first() == Some(&b'{') {
                let status: Status = decode(response.body.expose())?;
                check_code(status.code)?;
                return Err(invalid());
            }
            let decoded = codec::decrypt(key.expose(), response.body.expose())?;
            let status: Status = decode(decoded.expose())?;
            check_code(status.code)?;
            Ok(decoded)
        }).await
    }
}

#[derive(Deserialize)]
struct Status {
    code: i64,
}
fn check_code(code: i64) -> Result<()> {
    match code {
        0 => Ok(()),
        3 | -3 | -6 | -10001 | -10002 | -10003 | -704010000 => Err(auth_required()),
        _ => Err(Error::new(
            "MIJIA_REQUEST_REJECTED",
            "Mijia rejected the request.",
        )),
    }
}
#[derive(Deserialize)]
struct Sign {
    #[serde(rename = "_sign")]
    sign: PrivateString,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum Subject {
    Text(PrivateString),
    Number(u64),
}
#[derive(Deserialize)]
struct LoginResult {
    #[serde(rename = "userId")]
    user_id: Subject,
    ssecurity: PrivateString,
    location: PrivateString,
}
#[derive(Deserialize)]
struct LoginStatus {
    code: i64,
    #[serde(rename = "notificationUrl")]
    notification: Option<PrivateString>,
    #[serde(rename = "captchaUrl")]
    captcha: Option<PrivateString>,
}

pub async fn login(
    transport: &dyn CloudTransport,
    credentials: LoginCredentials,
    region: Region,
    operation: &Operation,
) -> Result<MijiaSession> {
    operation.run(async {
        let device = uuid::Uuid::new_v4().simple().to_string();
        let cookie = PrivateString::new(format!("sdkVersion=3.8.6; deviceId={device}"));
        let sign_request = CloudRequest { endpoint: CloudEndpoint::LoginSign, region, cookie: cookie.clone(), location: None, body: SecretBytes::new(vec![]) };
        let response = transport.send(&sign_request, operation).await?;
        let sign: Sign = decode(response.body.expose())?;
        if !crate::bounded_text(sign.sign.as_str(), 4096) { return Err(invalid()); }
        use md5::Digest;
        let hash = Zeroizing::new(md5::Md5::digest(credentials.password.expose()).iter().map(|byte| format!("{byte:02X}")).collect::<String>());
        let account = std::str::from_utf8(credentials.account.expose()).map_err(|_| Error::invalid())?;
        let request = CloudRequest {
            endpoint: CloudEndpoint::LoginPassword, region, cookie: cookie.clone(), location: None,
            body: codec::form(&[("sid", "xiaomiio"), ("hash", &hash), ("callback", "https://sts.api.io.mi.com/sts"),
                ("qs", "%3Fsid%3Dxiaomiio%26_json%3Dtrue"), ("user", account), ("_json", "true"), ("_sign", sign.sign.as_str())]),
        };
        let response = transport.send(&request, operation).await?;
        drop(credentials);
        let status: LoginStatus = decode(response.body.expose())?;
        if status.notification.is_some() || status.captcha.is_some() {
            return Err(Error::new("MIJIA_AUTH_CHALLENGE_REQUIRED", "Mijia requires interactive account verification; this login has not completed."));
        }
        check_code(status.code)?;
        let result: LoginResult = decode(response.body.expose())?;
        crate::transport::exchange_url(result.location.as_str())?;
        let user_id = match result.user_id { Subject::Text(s) => s, Subject::Number(n) => PrivateString::new(n.to_string()) };
        if user_id.as_str().len() > 32 || user_id.as_str().is_empty()
            || !user_id.as_str().bytes().all(|b| b.is_ascii_digit())
            || user_id.as_str().bytes().all(|b| b == b'0')
            || codec::unbase64(result.ssecurity.as_str())?.len() != 16 { return Err(invalid()); }
        let exchange = CloudRequest { endpoint: CloudEndpoint::LoginExchange, region, location: Some(result.location), cookie, body: SecretBytes::new(vec![]) };
        let response = transport.send(&exchange, operation).await?;
        let token = response.service_token.ok_or_else(auth_required)?;
        let token = std::str::from_utf8(token.expose()).map_err(|_| invalid())?;
        if !cookie_value(token) { return Err(invalid()); }
        Ok(MijiaSession { region, user_id, security: result.ssecurity, token: PrivateString::new(token.to_owned()) })
    }).await
}

#[cfg(test)]
mod tests;
