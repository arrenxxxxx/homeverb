//! Ephemeral origin/path-bound cookies; service tokens are captured but never sent.
use super::*;
use tokio::time::Instant;

struct Cookie {
    name: String,
    value: PrivateString,
    path: String,
    expires: Option<Instant>,
}
#[derive(Default)]
pub(in crate::auth::challenge) struct Cookies {
    account: Vec<Cookie>,
    token: Option<Cookie>,
    subject: Option<PrivateString>,
}
impl Cookies {
    pub(in crate::auth::challenge) fn new(base: &str) -> Result<Self> {
        let mut result = Self::default();
        for field in base.split("; ") {
            let (name, value) = field.split_once('=').ok_or_else(invalid)?;
            if !matches!(name, "sdkVersion" | "deviceId") || !cookie_value(value) {
                return Err(invalid());
            }
            result.account.push(Cookie {
                name: name.into(),
                value: PrivateString::new(value.into()),
                path: "/".into(),
                expires: None,
            });
        }
        Ok(result)
    }
    pub(in crate::auth::challenge) fn absorb(
        &mut self,
        url: &reqwest::Url,
        headers: Vec<SecretBytes>,
    ) -> Result<()> {
        let mut seen = std::collections::HashSet::new();
        for header in headers {
            let text = std::str::from_utf8(header.expose()).map_err(|_| invalid())?;
            if !crate::bounded_text(text, 8192) {
                return Err(invalid());
            }
            let mut parts = text.split(';');
            let (name, value) = parts
                .next()
                .unwrap()
                .trim()
                .split_once('=')
                .ok_or_else(invalid)?;
            if name.is_empty()
                || name.len() > 64
                || !name
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
                || (!value.is_empty() && !cookie_value(value))
                || !seen.insert(name.to_owned())
            {
                return Err(invalid());
            }
            let mut path = url
                .path()
                .rsplit_once('/')
                .map(|(parent, _)| if parent.is_empty() { "/" } else { parent })
                .unwrap_or("/")
                .to_owned();
            let mut expiry = None;
            let mut max_age = None;
            let mut attrs = std::collections::HashSet::new();
            for part in parts {
                let (key, value) = part.trim().split_once('=').unwrap_or((part.trim(), ""));
                let key = key.to_ascii_lowercase();
                if !attrs.insert(key.clone()) {
                    return Err(invalid());
                }
                match key.as_str() {
                    "domain" => {
                        let domain = value.trim_start_matches('.').to_ascii_lowercase();
                        let host = url.host_str().ok_or_else(invalid)?;
                        let allowed = if host == "account.xiaomi.com" {
                            ["account.xiaomi.com", "xiaomi.com"].contains(&domain.as_str())
                        } else {
                            ["sts.api.io.mi.com", "api.io.mi.com", "io.mi.com", "mi.com"]
                                .contains(&domain.as_str())
                        };
                        if !allowed {
                            return Err(invalid());
                        }
                    }
                    "path" => {
                        if !value.starts_with('/') || !crate::bounded_text(value, 256) {
                            return Err(invalid());
                        }
                        path = value.into();
                    }
                    "max-age" => {
                        max_age = Some(value.parse::<i64>().map_err(|_| invalid())?);
                    }
                    "expires" => {
                        let time = chrono::DateTime::parse_from_rfc2822(value)
                            .map_err(|_| invalid())?
                            .timestamp();
                        let seconds = time
                            .saturating_sub(chrono::Utc::now().timestamp())
                            .clamp(0, 180);
                        expiry = Some(Instant::now() + Duration::from_secs(seconds as u64));
                    }
                    _ => (),
                }
            }
            if let Some(age) = max_age {
                expiry = Some(Instant::now() + Duration::from_secs(age.clamp(0, 180) as u64));
            }
            let expired = value.is_empty() || expiry.is_some_and(|e| e <= Instant::now());
            if name == "userId" && !expired {
                numeric_subject(value)?;
                if self.subject.as_ref().is_some_and(|v| v.as_str() != value) {
                    return Err(invalid());
                }
                self.subject = Some(PrivateString::new(value.into()));
            }
            let cookie = Cookie {
                name: name.into(),
                value: PrivateString::new(value.into()),
                path,
                expires: expiry,
            };
            if name == "serviceToken" {
                self.token = (!expired).then_some(cookie);
                continue;
            }
            if url.host_str() != Some("account.xiaomi.com") {
                continue;
            }
            if self
                .account
                .iter()
                .any(|c| c.name == name && c.path != cookie.path)
            {
                return Err(invalid());
            }
            self.account.retain(|c| c.name != name);
            if !expired {
                self.account.push(cookie);
            }
            if self.account.len() > 16
                || self
                    .account
                    .iter()
                    .map(|c| c.name.len() + c.value.as_str().len() + 3)
                    .sum::<usize>()
                    > 16_384
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
    pub(in crate::auth::challenge) fn header(&self, url: &reqwest::Url) -> PrivateString {
        if url.host_str() != Some("account.xiaomi.com") {
            return PrivateString::new(String::new());
        }
        PrivateString::new(
            self.account
                .iter()
                .filter(|c| {
                    c.expires.is_none_or(|e| e > Instant::now())
                        && (url.path() == c.path
                            || url
                                .path()
                                .strip_prefix(&c.path)
                                .is_some_and(|tail| c.path.ends_with('/') || tail.starts_with('/')))
                })
                .map(|c| format!("{}={}", c.name, c.value.as_str()))
                .collect::<Vec<_>>()
                .join("; "),
        )
    }
    pub(in crate::auth::challenge) fn has_identity_session(&self, url: &reqwest::Url) -> bool {
        self.header(url)
            .as_str()
            .split("; ")
            .any(|v| v.starts_with("identity_session="))
    }
    pub(in crate::auth::challenge) fn subject(&self) -> Option<&str> {
        self.subject.as_ref().map(PrivateString::as_str)
    }
    pub(in crate::auth::challenge) fn token(&self) -> Option<&str> {
        self.token
            .as_ref()
            .filter(|c| c.expires.is_none_or(|e| e > Instant::now()))
            .map(|c| c.value.as_str())
    }
}
