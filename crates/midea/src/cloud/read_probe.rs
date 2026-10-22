//! Explicitly compiled, one-shot compatibility reads using the existing session.
//! Results never enter inventory, state caches, permissions or account admission.

use super::{CloudEndpoint, ORIGIN, SignedRequest, client_builder, send_once};
use homeverb_core::{Operation, SecretBytes};
use serde::{
    Deserialize,
    de::{IgnoredAny, MapAccess, SeqAccess, Visitor},
};
use std::{
    ffi::CStr,
    sync::atomic::{AtomicBool, Ordering},
};

static INVENTORY: AtomicBool = AtomicBool::new(false);
static STATE: AtomicBool = AtomicBool::new(false);

fn claim(once: &AtomicBool) -> bool {
    !once.swap(true, Ordering::SeqCst)
}

fn enabled() -> bool {
    option_env!("HOMEVERB_MEIJU_READ_PROBE") == Some("1")
}

#[derive(Clone, Copy)]
enum Kind {
    Inventory,
    State,
}

fn kind(request: &SignedRequest) -> Option<Kind> {
    request.token()?;
    match request.endpoint() {
        CloudEndpoint::Homes => Some(Kind::Inventory),
        CloudEndpoint::ApplianceState => Some(Kind::State),
        _ => None,
    }
}

fn prepare(kind: Kind, original: &SignedRequest) -> homeverb_domain::Result<SignedRequest> {
    let data = match kind {
        Kind::Inventory => serde_json::json!({"cardType":[
            {"type":"appliance", "query":{}, "extra":["uiTemplate","newUiTemplate","compose","permission"]},
            {"type":"lightGroup", "query":{}, "extra":["permission","newUiTemplate"]}
        ]}),
        Kind::State => {
            #[derive(Deserialize)]
            struct Target<'a> {
                #[serde(rename = "applianceCode", borrow)]
                id: &'a str,
            }
            let target: Target<'_> =
                serde_json::from_slice(original.body()).map_err(|_| crate::invalid())?;
            if target.id.is_empty()
                || target.id.len() > 64
                || !target.id.bytes().all(|b| b.is_ascii_digit())
            {
                return Err(crate::invalid());
            }
            serde_json::json!({"applianceCode":target.id, "command":{}, "caller":"meiju-plugin-tradition"})
        }
    };
    let mut data = data;
    data["reqId"] = serde_json::json!(uuid::Uuid::new_v4().simple().to_string());
    data["stamp"] = serde_json::json!(chrono::Local::now().format("%Y%m%d%H%M%S").to_string());
    SignedRequest::new(
        original.endpoint(),
        &data,
        original.token.as_ref(),
        chrono::Utc::now().timestamp_millis().to_string(),
    )
}

#[derive(Debug, PartialEq, Eq)]
enum Shape {
    Object,
    Array,
    Other,
}
impl<'de> Deserialize<'de> for Shape {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Shape;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a JSON value")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Shape, A::Error> {
                while a.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
                Ok(Shape::Object)
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Shape, A::Error> {
                while a.next_element::<IgnoredAny>()?.is_some() {}
                Ok(Shape::Array)
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Shape, E> {
                Ok(Shape::Other)
            }
            fn visit_bool<E: serde::de::Error>(self, _: bool) -> Result<Shape, E> {
                Ok(Shape::Other)
            }
            fn visit_i64<E: serde::de::Error>(self, _: i64) -> Result<Shape, E> {
                Ok(Shape::Other)
            }
            fn visit_u64<E: serde::de::Error>(self, _: u64) -> Result<Shape, E> {
                Ok(Shape::Other)
            }
            fn visit_f64<E: serde::de::Error>(self, _: f64) -> Result<Shape, E> {
                Ok(Shape::Other)
            }
            fn visit_str<E: serde::de::Error>(self, _: &str) -> Result<Shape, E> {
                Ok(Shape::Other)
            }
        }
        d.deserialize_any(V)
    }
}

fn outcome(body: &SecretBytes) -> &'static CStr {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Code {
        Number(i64),
        Text(String),
    }
    #[derive(Deserialize)]
    struct Envelope {
        code: Code,
        data: Option<Shape>,
    }
    let Ok(value) = serde_json::from_slice::<Envelope>(body.expose()) else {
        return c"response_invalid";
    };
    let code = match value.code {
        Code::Number(n) => Some(n),
        Code::Text(t) => t.parse::<i64>().ok(),
    };
    match code {
        Some(0) => match value.data {
            Some(Shape::Object) => c"business_success_object",
            Some(Shape::Array) => c"business_success_array",
            _ => c"business_success_without_structured_data",
        },
        Some(7400 | 40002) => c"authentication_required",
        Some(_) => c"business_rejected",
        None => c"response_invalid",
    }
}

fn emit(kind: Kind, marker: &'static CStr) {
    #[cfg(target_os = "android")]
    {
        #[link(name = "log")]
        unsafe extern "C" {
            fn __android_log_write(
                priority: std::ffi::c_int,
                tag: *const std::ffi::c_char,
                text: *const std::ffi::c_char,
            ) -> std::ffi::c_int;
        }
        // Callers pass only static closed markers, never provider strings or values.
        unsafe {
            let tag = match kind {
                Kind::Inventory => c"HomeVerbReadProbeInventory",
                Kind::State => c"HomeVerbReadProbeLua",
            };
            __android_log_write(4, tag.as_ptr(), marker.as_ptr());
        }
    }
    #[cfg(not(target_os = "android"))]
    let _ = (kind, marker);
}

pub(super) async fn run(original: &SignedRequest, operation: &Operation) {
    if !enabled() {
        return;
    }
    let Some(kind) = kind(original) else {
        return;
    };
    let once = match kind {
        Kind::Inventory => &INVENTORY,
        Kind::State => &STATE,
    };
    if !claim(once) {
        return;
    }
    let result = operation
        .run(async {
            let request = prepare(kind, original)?;
            let client = client_builder()
                .default_headers(reqwest::header::HeaderMap::from_iter([
                    (
                        reqwest::header::HeaderName::from_static("appid"),
                        reqwest::header::HeaderValue::from_static("900"),
                    ),
                    (
                        reqwest::header::HeaderName::from_static("terminalid"),
                        reqwest::header::HeaderValue::from_static("900-android"),
                    ),
                    (
                        reqwest::header::HeaderName::from_static("version"),
                        reqwest::header::HeaderValue::from_static("11.7.10"),
                    ),
                    (
                        reqwest::header::HeaderName::from_static("platform"),
                        reqwest::header::HeaderValue::from_static("0"),
                    ),
                ]))
                .build()
                .map_err(|_| super::failed())?;
            let route = match kind {
                Kind::Inventory => "/v1/appliance/home/list/aggregate",
                Kind::State => "/v1/device/status/lua/get",
            };
            emit(
                kind,
                match kind {
                    Kind::Inventory => c"inventory_started",
                    Kind::State => c"lua_state_started",
                },
            );
            send_once(client, &format!("{ORIGIN}{route}"), &request, operation).await
        })
        .await;
    emit(
        kind,
        match &result {
            Ok(body) => outcome(body),
            Err(error) if error.code == "AUTH_REQUIRED" => c"authentication_required",
            Err(_) => c"request_failed_or_cancelled",
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request(endpoint: CloudEndpoint, token: bool) -> SignedRequest {
        SignedRequest::new(
            endpoint,
            &serde_json::json!({"applianceCode":"12345","command":{"query":{"anything":true}}}),
            token
                .then(|| SecretBytes::new(b"fictional-token".to_vec()))
                .as_ref(),
            "1".into(),
        )
        .unwrap()
    }
    #[test]
    fn only_two_authenticated_read_triggers_are_admitted() {
        assert!(kind(&request(CloudEndpoint::Homes, true)).is_some());
        assert!(kind(&request(CloudEndpoint::ApplianceState, true)).is_some());
        for endpoint in [
            CloudEndpoint::LoginId,
            CloudEndpoint::Login,
            CloudEndpoint::HomeAppliances,
            CloudEndpoint::ApplianceControl,
            CloudEndpoint::LanCredentials,
        ] {
            assert!(kind(&request(endpoint, true)).is_none());
        }
        assert!(kind(&request(CloudEndpoint::Homes, false)).is_none());
    }
    #[test]
    fn a_claim_is_consumed_once_even_when_the_request_fails() {
        let once = AtomicBool::new(false);
        std::thread::scope(|scope| {
            let handles: Vec<_> = (0..16).map(|_| scope.spawn(|| claim(&once))).collect();
            assert_eq!(
                handles
                    .into_iter()
                    .map(|h| h.join().unwrap())
                    .filter(|won| *won)
                    .count(),
                1
            );
        });
        assert!(!claim(&once));
    }
    #[test]
    fn aggregate_probe_has_only_the_reviewed_read_card_types() {
        let prepared = prepare(Kind::Inventory, &request(CloudEndpoint::Homes, true)).unwrap();
        let value: serde_json::Value = serde_json::from_slice(prepared.body()).unwrap();
        assert_eq!(value.as_object().unwrap().len(), 3);
        assert_eq!(value["cardType"].as_array().unwrap().len(), 2);
        assert_eq!(value["cardType"][0]["type"], "appliance");
        assert_eq!(value["cardType"][1]["type"], "lightGroup");
        assert_eq!(value["cardType"][0]["query"], serde_json::json!({}));
    }
    #[test]
    fn state_probe_uses_only_empty_query_and_preserves_protected_token() {
        let original = request(CloudEndpoint::ApplianceState, true);
        let prepared = prepare(Kind::State, &original).unwrap();
        let value: serde_json::Value = serde_json::from_slice(prepared.body()).unwrap();
        assert_eq!(value["command"], serde_json::json!({}));
        assert_eq!(value["caller"], "meiju-plugin-tradition");
        assert_eq!(value["applianceCode"], "12345");
        assert_eq!(prepared.token(), original.token());
        assert_eq!(value.as_object().unwrap().len(), 5);
        assert!(!format!("{prepared:?}").contains("fictional-token"));
    }
    #[test]
    fn provider_data_is_reduced_to_closed_outcomes_without_accepting_http_alone() {
        for (body, expected) in [
            (
                r#"{"code":0,"data":{"private":"ignored"}}"#,
                c"business_success_object",
            ),
            (r#"{"code":"0","data":[]}"#, c"business_success_array"),
            (
                r#"{"code":0,"data":null}"#,
                c"business_success_without_structured_data",
            ),
            (r#"{"code":7400,"data":{}}"#, c"authentication_required"),
            (r#"{"code":123,"message":"private"}"#, c"business_rejected"),
            (r#"{"data":{}}"#, c"response_invalid"),
            (r#"{"code":0,"code":7400,"data":{}}"#, c"response_invalid"),
        ] {
            assert_eq!(
                outcome(&SecretBytes::new(body.as_bytes().to_vec())),
                expected
            );
        }
    }
}
