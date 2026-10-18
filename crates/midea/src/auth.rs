//! Explicit native Meiju password login. No automatic renewal or challenge bypass.
//! Request shapes are modified from the source attributed in third-party/midea-auto-cloud/NOTICE.md.

use crate::{
    CloudEndpoint, CloudTransport, MAX_RESPONSE_BYTES, ReadEndpoint, ReadTransport, SignedRequest,
    codec, invalid, material,
};
use async_trait::async_trait;
use chrono::{Local, Utc};
use homeverb_core::{CredentialRef, Operation, SecretBytes};
use homeverb_domain::{Error, Result};
use serde::{Deserialize, Deserializer, Serialize, de::DeserializeOwned};
use std::{fmt, str};
use uuid::Uuid;
use zeroize::Zeroizing;

mod lan;
pub use lan::LanUdpIdMethod;

/// Trusted native credentials cannot enter JSON requests.
/// ```compile_fail
/// fn export(value: &homeverb_midea::LoginCredentials) { let _ = serde_json::to_string(value); }
/// ```
pub struct LoginCredentials {
    account: SecretBytes,
    password: SecretBytes,
}

impl fmt::Debug for LoginCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LoginCredentials([REDACTED])")
    }
}

fn ascii(value: &[u8], limit: usize, spaces: bool) -> bool {
    !value.is_empty()
        && value.len() <= limit
        && value
            .iter()
            .all(|b| b.is_ascii_graphic() || (spaces && *b == b' '))
}

impl LoginCredentials {
    /// Only reviewed ASCII input is accepted. Consume protected native input, never public JSON.
    pub fn new(account: SecretBytes, password: SecretBytes) -> Result<Self> {
        if !ascii(account.expose(), 256, false) || !ascii(password.expose(), 256, true) {
            return Err(Error::new(
                "MIDEA_LOGIN_INPUT_UNSUPPORTED",
                "The reviewed login flow requires bounded nonempty ASCII credentials.",
            ));
        }
        Ok(Self { account, password })
    }
}

struct PrivateString(Zeroizing<String>);
impl<'de> Deserialize<'de> for PrivateString {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        String::deserialize(deserializer).map(|value| Self(Zeroizing::new(value)))
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Code {
    Number(i64),
    Text(String),
}

#[derive(Deserialize)]
struct ResponseCode {
    code: Code,
}

#[derive(Deserialize)]
struct ResponseData<T> {
    data: Option<T>,
}

fn decode<T: DeserializeOwned>(body: &SecretBytes) -> Result<T> {
    if body.expose().len() > MAX_RESPONSE_BYTES
        || body.expose().trim_ascii_start().first() != Some(&b'{')
    {
        return Err(invalid());
    }
    // Rejections may carry challenge data instead of the successful endpoint schema.
    // Ignore that data without materializing its private strings before checking the code.
    let envelope: ResponseCode = serde_json::from_slice(body.expose()).map_err(|_| invalid())?;
    let code = match envelope.code {
        Code::Number(code) => code,
        Code::Text(code) => code.parse().map_err(|_| invalid())?,
    };
    match code {
        0 => {
            let response: ResponseData<T> =
                serde_json::from_slice(body.expose()).map_err(|_| invalid())?;
            response.data.ok_or_else(invalid)
        }
        7400 | 40002 => Err(Error::new(
            "AUTH_REQUIRED",
            "The platform account requires user-side authorization.",
        )),
        _ => Err(Error::new(
            "MIDEA_REQUEST_REJECTED",
            "The provider rejected the login request; additional challenges are not supported.",
        )),
    }
}

#[derive(Serialize)]
struct Request<T> {
    #[serde(flatten)]
    data: T,
    #[serde(rename = "reqId")]
    request_id: String,
    stamp: String,
}

pub(crate) async fn request(
    transport: &dyn CloudTransport,
    endpoint: CloudEndpoint,
    data: impl Serialize,
    token: Option<&SecretBytes>,
    operation: &Operation,
) -> Result<SecretBytes> {
    request_at(
        transport,
        endpoint,
        data,
        token,
        &Local::now().format("%Y%m%d%H%M%S").to_string(),
        operation,
    )
    .await
}

async fn request_at(
    transport: &dyn CloudTransport,
    endpoint: CloudEndpoint,
    data: impl Serialize,
    token: Option<&SecretBytes>,
    stamp: &str,
    operation: &Operation,
) -> Result<SecretBytes> {
    operation
        .run(async {
            let payload = Request {
                data,
                request_id: Uuid::new_v4().simple().to_string(),
                stamp: stamp.to_owned(),
            };
            let request = SignedRequest::new(
                endpoint,
                &payload,
                token,
                Utc::now().timestamp().to_string(),
            )?;
            transport
                .send(&request, operation)
                .await
                .map_err(crate::sanitized)
        })
        .await
}

#[derive(Serialize)]
struct LoginIdRequest<'a> {
    #[serde(rename = "loginAccount")]
    login_account: &'a str,
    #[serde(rename = "type")]
    kind: &'a str,
}

#[derive(Deserialize)]
struct LoginId {
    #[serde(rename = "loginId")]
    id: PrivateString,
}
#[derive(Deserialize)]
struct Token {
    #[serde(rename = "accessToken")]
    token: PrivateString,
}
#[derive(Deserialize)]
struct LoginData {
    mdata: Token,
    key: PrivateString,
    #[serde(default, rename = "iotUserId")]
    iot_user_id: Option<PrivateString>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct IotLogin<'a> {
    client_type: u8,
    device_id: &'a str,
    iampwd: &'a str,
    iot_app_id: &'a str,
    login_account: &'a str,
    password: &'a str,
    req_id: String,
    stamp: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppLogin<'a> {
    app_key: &'a str,
    device_id: &'a str,
    platform: u8,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LoginRequest<'a> {
    iot_data: IotLogin<'a>,
    data: AppLogin<'a>,
    timestamp: &'a str,
}

/// Explicitly connect the mainland Meiju account. Success is not physical-device verification.
pub async fn login(
    transport: &dyn CloudTransport,
    credentials: &LoginCredentials,
    operation: &Operation,
) -> Result<MeijuSession> {
    // Input was validated before construction; no lossy conversion or account normalization.
    let account = str::from_utf8(credentials.account.expose()).map_err(|_| invalid())?;
    let password = str::from_utf8(credentials.password.expose()).map_err(|_| invalid())?;
    let body = request(
        transport,
        CloudEndpoint::LoginId,
        LoginIdRequest {
            login_account: account,
            kind: "1",
        },
        None,
        operation,
    )
    .await?;
    let id: LoginId = decode(&body)?;
    if !ascii(id.id.0.as_bytes(), 256, false) {
        return Err(invalid());
    }
    let (password, iam) = codec::passwords(&id.id.0, password);
    let device_id = codec::device_id(account);
    let stamp = Local::now().format("%Y%m%d%H%M%S").to_string();
    let data = LoginRequest {
        iot_data: IotLogin {
            client_type: 1,
            device_id: &device_id,
            iampwd: &iam,
            iot_app_id: "900",
            login_account: account,
            password: &password,
            req_id: Uuid::new_v4().simple().to_string(),
            stamp: &stamp,
        },
        data: AppLogin {
            app_key: material::APP_KEY,
            device_id: &device_id,
            platform: 2,
        },
        timestamp: &stamp,
    };
    let response = request_at(
        transport,
        CloudEndpoint::Login,
        data,
        None,
        &stamp,
        operation,
    )
    .await?;
    crate::auth_trace::login_identity_shape(response.expose());
    let result: LoginData = decode(&response)?;
    if !ascii(result.mdata.token.0.as_bytes(), 8192, false) {
        return Err(invalid());
    }
    let key = codec::session_key(&result.key.0)?;
    let mut legacy = b"meiju.login-id.v1:".to_vec();
    legacy.extend_from_slice(id.id.0.as_bytes());
    let (subject, legacy_subject) = match result.iot_user_id {
        Some(uid) => {
            if !ascii(uid.0.as_bytes(), 256, false) {
                return Err(invalid());
            }
            let mut subject = b"meiju.iot-user-id.v1:".to_vec();
            subject.extend_from_slice(uid.0.as_bytes());
            (subject, Some(SecretBytes::new(legacy)))
        }
        None => (legacy, None),
    };
    operation
        .run(async {
            Ok(MeijuSession {
                subject: SecretBytes::new(subject),
                token: SecretBytes::new(result.mdata.token.0.as_bytes().to_vec()),
                key,
                client_device_id: Some(SecretBytes::new(device_id.as_bytes().to_vec())),
                legacy_subject,
            })
        })
        .await
}

/// Protected native session. Its persistence encoding must only enter the system secret store.
/// ```compile_fail
/// fn export(value: &homeverb_midea::MeijuSession) { let _ = serde_json::to_string(value); }
/// ```
pub struct MeijuSession {
    subject: SecretBytes,
    token: SecretBytes,
    key: SecretBytes,
    client_device_id: Option<SecretBytes>,
    legacy_subject: Option<SecretBytes>,
}

impl fmt::Debug for MeijuSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MeijuSession([REDACTED])")
    }
}

#[derive(Serialize)]
struct StoredSession<'a> {
    version: u8,
    subject: &'a str,
    token: &'a str,
    key: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_device_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    legacy_subject: Option<&'a str>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RestoredSession {
    version: u8,
    subject: PrivateString,
    token: PrivateString,
    key: PrivateString,
    #[serde(default)]
    client_device_id: Option<PrivateString>,
    #[serde(default)]
    legacy_subject: Option<PrivateString>,
}

impl MeijuSession {
    pub(crate) fn matches_saved_subject(&self, subject: &[u8]) -> bool {
        self.subject.expose() == subject
            || self
                .legacy_subject
                .as_ref()
                .is_some_and(|id| id.expose() == subject)
    }

    pub(crate) fn is_canonical(&self) -> bool {
        self.legacy_subject.is_some()
    }

    pub(crate) fn same_login_input(&self, previous: &Self) -> bool {
        matches!((&self.client_device_id, &previous.client_device_id),
            (Some(a), Some(b)) if a.expose() == b.expose())
    }

    /// Diagnostic comparisons never participate in account admission or persistence.
    pub(crate) fn trace_reauthorization(&self, previous: &Self) {
        use crate::auth_trace::{Event, emit};
        emit(if self.subject.expose() == previous.subject.expose() {
            Event::SubjectMatches
        } else if self
            .subject
            .expose()
            .eq_ignore_ascii_case(previous.subject.expose())
        {
            Event::SubjectDiffersOnlyInAsciiCase
        } else {
            Event::SubjectDiffers
        });
        emit(match (&previous.client_device_id, &self.client_device_id) {
            (Some(old), Some(new)) if old.expose() == new.expose() => {
                Event::ClientIdentifierMatches
            }
            (Some(_), Some(_)) => Event::ClientIdentifierDiffers,
            _ => Event::LegacyClientIdentifierAbsent,
        });
    }

    pub(crate) fn control_request(
        &self,
        appliance: &crate::CloudId,
        control: crate::control::CloudControl,
    ) -> Result<SignedRequest> {
        #[derive(Serialize)]
        struct Control {
            control: crate::control::CloudControl,
        }
        #[derive(Serialize)]
        struct Target<'a> {
            #[serde(rename = "applianceCode")]
            appliance: &'a str,
            command: Control,
        }
        SignedRequest::new(
            CloudEndpoint::ApplianceControl,
            &Request {
                data: Target {
                    appliance: appliance.expose(),
                    command: Control { control },
                },
                request_id: Uuid::new_v4().simple().to_string(),
                stamp: Local::now().format("%Y%m%d%H%M%S").to_string(),
            },
            Some(&self.token),
            Utc::now().timestamp().to_string(),
        )
    }

    pub fn subject(&self) -> &SecretBytes {
        &self.subject
    }

    pub fn encode(&self) -> Result<SecretBytes> {
        fn text(value: &SecretBytes) -> Result<&str> {
            str::from_utf8(value.expose()).map_err(|_| invalid())
        }
        Ok(SecretBytes::new(
            serde_json::to_vec(&StoredSession {
                version: if self.is_canonical() {
                    3
                } else if self.client_device_id.is_some() {
                    2
                } else {
                    1
                },
                subject: text(&self.subject)?,
                token: text(&self.token)?,
                key: text(&self.key)?,
                client_device_id: self.client_device_id.as_ref().map(text).transpose()?,
                legacy_subject: self.legacy_subject.as_ref().map(text).transpose()?,
            })
            .map_err(|_| invalid())?,
        ))
    }

    pub fn restore(bytes: &SecretBytes) -> Result<Self> {
        let invalid_session = || {
            Error::new(
                "ACCOUNT_SESSION_UNCONFIRMED",
                "The protected Meiju session requires validation and recovery.",
            )
        };
        if bytes.expose().len() > 65536 {
            return Err(invalid_session());
        }
        let stored: RestoredSession =
            serde_json::from_slice(bytes.expose()).map_err(|_| invalid_session())?;
        let subject = stored.subject.0.as_bytes();
        let valid_version = match (stored.version, &stored.client_device_id) {
            (1, None) => true,
            (2 | 3, Some(id)) => {
                id.0.len() == 16
                    && id
                        .0
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            }
            _ => false,
        };
        let valid_subject = match (stored.version, &stored.legacy_subject) {
            (1 | 2, None) => subject
                .strip_prefix(b"meiju.login-id.v1:")
                .is_some_and(|id| ascii(id, 256, false)),
            (3, Some(legacy)) => {
                subject
                    .strip_prefix(b"meiju.iot-user-id.v1:")
                    .is_some_and(|id| ascii(id, 256, false))
                    && legacy
                        .0
                        .as_bytes()
                        .strip_prefix(b"meiju.login-id.v1:")
                        .is_some_and(|id| ascii(id, 256, false))
            }
            _ => false,
        };
        if !valid_version
            || !valid_subject
            || !ascii(stored.token.0.as_bytes(), 8192, false)
            || !matches!(stored.key.0.len(), 16 | 24 | 32)
            || !ascii(stored.key.0.as_bytes(), 32, false)
        {
            return Err(invalid_session());
        }
        Ok(Self {
            subject: SecretBytes::new(subject.to_vec()),
            token: SecretBytes::new(stored.token.0.as_bytes().to_vec()),
            key: SecretBytes::new(stored.key.0.as_bytes().to_vec()),
            client_device_id: stored
                .client_device_id
                .as_ref()
                .map(|id| SecretBytes::new(id.0.as_bytes().to_vec())),
            legacy_subject: stored
                .legacy_subject
                .as_ref()
                .map(|id| SecretBytes::new(id.0.as_bytes().to_vec())),
        })
    }

    /// Bind this session to the exact account reference while its native account lease is held.
    pub fn reads<'a>(
        &'a self,
        transport: &'a dyn CloudTransport,
        reference: CredentialRef,
    ) -> SessionReads<'a> {
        SessionReads {
            session: self,
            transport,
            reference,
        }
    }
}

pub struct SessionReads<'a> {
    session: &'a MeijuSession,
    transport: &'a dyn CloudTransport,
    reference: CredentialRef,
}

#[async_trait]
impl ReadTransport for SessionReads<'_> {
    async fn request(
        &self,
        reference: &CredentialRef,
        endpoint: ReadEndpoint<'_>,
        operation: &Operation,
    ) -> Result<SecretBytes> {
        if *reference != self.reference {
            return Err(Error::new(
                "ACCOUNT_IDENTITY_CONFLICT",
                "The cloud session belongs to a different local account reference.",
            ));
        }
        #[derive(Serialize)]
        struct Home {
            #[serde(rename = "homegroupId")]
            id: u64,
        }
        #[derive(Serialize)]
        struct Query {
            query: serde_json::Value,
        }
        #[derive(Serialize)]
        struct State<'a> {
            #[serde(rename = "applianceCode")]
            id: &'a str,
            command: Query,
        }
        let token = Some(&self.session.token);
        match endpoint {
            ReadEndpoint::Homes => {
                request(
                    self.transport,
                    CloudEndpoint::Homes,
                    serde_json::json!({}),
                    token,
                    operation,
                )
                .await
            }
            ReadEndpoint::HomeAppliances(home) => {
                request(
                    self.transport,
                    CloudEndpoint::HomeAppliances,
                    Home {
                        id: home.expose().parse().map_err(|_| invalid())?,
                    },
                    token,
                    operation,
                )
                .await
            }
            ReadEndpoint::ApplianceState { appliance, query } => {
                request(
                    self.transport,
                    CloudEndpoint::ApplianceState,
                    State {
                        id: appliance.expose(),
                        command: Query {
                            query: query.payload(),
                        },
                    },
                    token,
                    operation,
                )
                .await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_envelopes_require_bounded_json_objects_with_unambiguous_codes() {
        for raw in [
            "null",
            "[]",
            "[7400]",
            "[0,{\"loginId\":\"synthetic\"}]",
            "{\"data\":{}}",
            "{\"code\":true}",
            "{\"code\":0.0}",
            "{\"code\":\"invalid\"}",
            "{\"code\":0,\"code\":7400,\"data\":{}}",
            "{\"code\":7400,\"code\":0,\"data\":{}}",
            "{\"code\":0,\"data\":{},\"data\":{}}",
            "{\"code\":7400,\"data\":",
            "{\"code\":7400} trailing",
        ] {
            let body = SecretBytes::new(raw.as_bytes().to_vec());
            assert_eq!(
                decode::<LoginId>(&body).err().unwrap().code,
                "MIDEA_RESPONSE_INVALID"
            );
        }
        let oversized = format!(
            "{{\"code\":7400,\"data\":\"{}\"}}",
            "x".repeat(MAX_RESPONSE_BYTES)
        );
        assert_eq!(
            decode::<LoginId>(&SecretBytes::new(oversized.into_bytes()))
                .err()
                .unwrap()
                .code,
            "MIDEA_RESPONSE_INVALID"
        );
    }
}
