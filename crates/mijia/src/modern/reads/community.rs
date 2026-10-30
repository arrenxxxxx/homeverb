//! Shared bounded per-field community history profile, separate from reference reads.
use super::*;
use crate::MAX_RESPONSE_BYTES;

impl ModernMijiaSession {
    pub(super) async fn community_history_read(
        &self,
        transport: &dyn ModernReadTransport,
        endpoint: ModernReadEndpoint,
        data: &SecretBytes,
        operation: &Operation,
    ) -> Result<SecretBytes> {
        if !matches!(
            endpoint,
            ModernReadEndpoint::Statistics
                | ModernReadEndpoint::PropertyHistory
                | ModernReadEndpoint::EventHistory
        ) {
            return Err(invalid());
        }
        let (body, key) = codec::encode(
            endpoint.path(),
            std::str::from_utf8(data.expose()).map_err(|_| invalid())?,
            self.security.as_str(),
            &codec::nonce()?,
        )?;
        let request = ModernReadRequest {
            endpoint,
            body,
            agent: self.identity.agent.clone(),
            cookie: PrivateString::new(format!(
                concat!(
                    "userId={};cUserId={};yetAnotherServiceToken={};serviceToken={};",
                    "locale=zh_CN;timezone=GMT+08:00;is_daylight=0;dst_offset=0;",
                    "channel=MI_APP_STORE;PassportDeviceId={};"
                ),
                self.subject.as_str(),
                self.encrypted_subject.as_str(),
                self.token.as_str(),
                self.token.as_str(),
                self.identity.device.as_str()
            )),
        };
        let response = transport.send_modern_read(&request, operation).await?;
        decode_community_response(response, key.expose())
    }
}

fn decode_community_response(response: ModernReadResponse, key: &[u8]) -> Result<SecretBytes> {
    if response.body.expose().len() > MAX_RESPONSE_BYTES {
        return Err(invalid());
    }
    let decoded = if response.body.expose().trim_ascii_start().first() == Some(&b'{') {
        if response.encoding != ModernContentEncoding::Plain {
            return Err(invalid());
        }
        response.body
    } else {
        codec::decrypt(key, response.body.expose().trim_ascii())?
    };
    let bytes = if response.encoding == ModernContentEncoding::Gzip
        || decoded.expose().starts_with(&[31, 139])
    {
        use std::io::Read;
        let mut decoder = flate2::bufread::GzDecoder::new(decoded.expose());
        let mut output = zeroize::Zeroizing::new(Vec::new());
        decoder
            .by_ref()
            .take((MAX_RESPONSE_BYTES + 1) as u64)
            .read_to_end(&mut output)
            .map_err(|_| invalid())?;
        if output.len() > MAX_RESPONSE_BYTES || !decoder.into_inner().is_empty() {
            return Err(invalid());
        }
        SecretBytes::new(std::mem::take(&mut *output))
    } else {
        decoded
    };
    if bytes.expose().len() > MAX_RESPONSE_BYTES {
        return Err(invalid());
    }
    let status: Status = crate::decode(bytes.expose())?;
    check_status(status.code)?;
    Ok(bytes)
}
