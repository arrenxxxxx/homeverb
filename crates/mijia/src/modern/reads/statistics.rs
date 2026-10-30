//! Exact modern statistics profile from the pinned MIT community protocol.
use super::*;
use crate::statistics::{DAILY_ENERGY_MODEL, DAILY_ENERGY_SPEC, decode_history};
use crate::{DailyEnergyHistory, DailyEnergyWindow, MAX_RESPONSE_BYTES};
use chrono::Utc;

impl ModernMijiaSession {
    /// One exact historical query. This is not the reference continuous-stream route,
    /// a current observation, a permission grant or a generic statistics endpoint.
    pub async fn daily_energy_history(
        &self,
        transport: &dyn ModernReadTransport,
        entry: &ModernDirectoryEntry,
        window: DailyEnergyWindow,
        operation: &Operation,
    ) -> Result<DailyEnergyHistory> {
        operation
            .run(async {
                self.validate()?;
                if entry.subject != self.subject
                    || entry.encrypted_subject != self.encrypted_subject
                {
                    return Err(crate::protected::conflict());
                }
                if !matches!(entry.kind, Some(DirectoryKind(0 | 6 | 16 | 18)))
                    || entry.model != DAILY_ENERGY_MODEL
                    || entry.mapping.as_ref().is_none_or(|mapping| {
                        mapping.spec_type != DAILY_ENERGY_SPEC
                            || !mapping
                                .properties
                                .iter()
                                .any(|p| p.siid == 11 && p.piid == 1)
                    })
                {
                    return Err(Error::new(
                        "CAPABILITY_UNSUPPORTED",
                        "This exact historical energy profile is unavailable.",
                    ));
                }
                if !crate::bounded_text(entry.did.as_str(), 128)
                    || window.end() > Utc::now().timestamp()
                {
                    return Err(Error::invalid());
                }
                let data = PrivateString::new(
                    serde_json::to_string(&json!({
                        "did": entry.did.as_str(), "key": "11.1", "data_type": "stat_day_v3",
                        "time_start": window.start(), "time_end": window.end(), "limit": 31
                    }))
                    .map_err(|_| invalid())?,
                );
                let endpoint = ModernReadEndpoint::Statistics;
                // This source resets the stream for each field. Never retry with the
                // reference codec or forward a legacy SID token to this origin.
                let (body, key) = codec::encode(
                    endpoint.path(),
                    data.as_str(),
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
                let bytes = decode_statistics_response(response, key.expose())?;
                let history =
                    decode_history(bytes.expose(), entry.did.as_str(), window, Utc::now())?;
                operation.run(async { Ok(history) }).await
            })
            .await
    }
}

fn decode_statistics_response(response: ModernReadResponse, key: &[u8]) -> Result<SecretBytes> {
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
