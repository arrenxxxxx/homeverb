//! Dedicated captcha retrieval; provider URLs never become generic request targets.
use super::*;

pub(crate) const MAX_IMAGE_BYTES: usize = 131_072;

pub struct CaptchaRequest {
    pub(super) location: PrivateString,
    cookie: PrivateString,
}
impl fmt::Debug for CaptchaRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CaptchaRequest([REDACTED])")
    }
}
impl CaptchaRequest {
    pub(crate) fn new(location: PrivateString, cookie: PrivateString) -> Result<Self> {
        captcha_url(location.as_str())?;
        Ok(Self { location, cookie })
    }
}

pub struct CaptchaResponse {
    image: SecretBytes,
    pub(crate) cookie: PrivateString,
    media_type: &'static str,
}
impl fmt::Debug for CaptchaResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CaptchaResponse([REDACTED])")
    }
}
impl CaptchaResponse {
    pub fn new(image: SecretBytes, cookie: SecretBytes) -> Result<Self> {
        let value = std::str::from_utf8(cookie.expose()).map_err(|_| invalid())?;
        if !crate::cookie_value(value) {
            return Err(invalid());
        }
        let media_type = image_type(image.expose())?;
        Ok(Self {
            image,
            cookie: PrivateString::new(value.to_owned()),
            media_type,
        })
    }
    pub fn image(&self) -> &[u8] {
        self.image.expose()
    }
    pub fn media_type(&self) -> &'static str {
        self.media_type
    }
}

pub(super) fn captcha_url(value: &str) -> Result<Url> {
    if !crate::bounded_text(value, 8192) || value.contains('\\') || value.starts_with("//") {
        return Err(invalid());
    }
    let value = if value.starts_with('/') {
        format!("https://account.xiaomi.com{value}")
    } else {
        value.to_owned()
    };
    let url = Url::parse(&value).map_err(|_| invalid())?;
    if url.scheme() != "https"
        || url.host_str() != Some("account.xiaomi.com")
        || url.port().is_some_and(|p| p != 443)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || url.path() == "/"
    {
        return Err(invalid());
    }
    Ok(url)
}

fn dimensions(width: u32, height: u32) -> Result<()> {
    if width == 0 || height == 0 || width > 2048 || height > 2048 || width * height > 1_048_576 {
        return Err(invalid());
    }
    Ok(())
}

// This admission checks format and allocation bounds, not full image integrity.
// Native image decoders still reject corrupt pixels without executing active content.
fn image_type(bytes: &[u8]) -> Result<&'static str> {
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err(invalid());
    }
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n")
        && bytes.len() >= 33
        && bytes[8..12] == 13u32.to_be_bytes()
        && &bytes[12..16] == b"IHDR"
    {
        dimensions(
            u32::from_be_bytes(bytes[16..20].try_into().unwrap()),
            u32::from_be_bytes(bytes[20..24].try_into().unwrap()),
        )?;
        return Ok("image/png");
    }
    if bytes.starts_with(&[0xff, 0xd8]) {
        let mut pos = 2;
        while pos + 4 <= bytes.len() {
            if bytes[pos] != 0xff {
                return Err(invalid());
            }
            let marker = bytes[pos + 1];
            if marker == 0xff {
                pos += 1;
                continue;
            }
            let size = usize::from(u16::from_be_bytes([bytes[pos + 2], bytes[pos + 3]]));
            if size < 2 || pos + 2 + size > bytes.len() {
                return Err(invalid());
            }
            if matches!(marker, 0xc0..=0xc3 | 0xc5..=0xc7 | 0xc9..=0xcb | 0xcd..=0xcf) {
                if size < 8 {
                    return Err(invalid());
                }
                dimensions(
                    u32::from(u16::from_be_bytes([bytes[pos + 7], bytes[pos + 8]])),
                    u32::from(u16::from_be_bytes([bytes[pos + 5], bytes[pos + 6]])),
                )?;
                return Ok("image/jpeg");
            }
            pos += 2 + size;
        }
    }
    Err(invalid())
}

pub(super) async fn transmit_captcha(
    client: Client,
    url: Url,
    request: &CaptchaRequest,
    operation: &Operation,
) -> Result<CaptchaResponse> {
    operation
        .run(async {
            let mut cookie =
                HeaderValue::from_str(request.cookie.as_str()).map_err(|_| invalid())?;
            cookie.set_sensitive(true);
            let mut response = client
                .get(url)
                .header("cookie", cookie)
                .header("accept", "image/png,image/jpeg")
                .header("accept-encoding", "identity")
                .send()
                .await
                .map_err(|_| failed())?;
            if response.status() != reqwest::StatusCode::OK {
                return Err(failed());
            }
            if response
                .content_length()
                .is_some_and(|v| v > MAX_IMAGE_BYTES as u64)
            {
                return Err(invalid());
            }
            let media_type = response
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.split(';').next())
                .unwrap_or("")
                .trim()
                .to_ascii_lowercase();
            if !matches!(media_type.as_str(), "image/png" | "image/jpeg") {
                return Err(invalid());
            }
            let mut ick = None;
            for cookie in response.headers().get_all("set-cookie") {
                let cookie = cookie.to_str().map_err(|_| invalid())?;
                if let Some(value) = cookie
                    .split(';')
                    .next()
                    .and_then(|c| c.strip_prefix("ick="))
                {
                    if ick.is_some() || !crate::cookie_value(value) {
                        return Err(invalid());
                    }
                    ick = Some(SecretBytes::new(value.as_bytes().to_vec()));
                }
            }
            let ick = ick.ok_or_else(invalid)?;
            let mut body = zeroize::Zeroizing::new(Vec::new());
            while let Some(chunk) = response.chunk().await.map_err(|_| failed())? {
                if body.len() + chunk.len() > MAX_IMAGE_BYTES {
                    return Err(invalid());
                }
                body.extend_from_slice(&chunk);
            }
            let response = CaptchaResponse::new(SecretBytes::new(std::mem::take(&mut *body)), ick)?;
            if response.media_type() != media_type {
                return Err(invalid());
            }
            Ok(response)
        })
        .await
}
