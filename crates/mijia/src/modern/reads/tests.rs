use super::*;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::{collections::VecDeque, io::Write, sync::Mutex};
use tokio_util::sync::CancellationToken;

fn operation() -> Operation {
    Operation::new(10_000, CancellationToken::new()).unwrap()
}
fn session() -> ModernMijiaSession {
    ModernMijiaSession {
        identity: ClientIdentity::new().unwrap(),
        subject: PrivateString::new("12345".into()),
        encrypted_subject: PrivateString::new("fictional-c-user".into()),
        pass_token: PrivateString::new("fictional-pass-token".into()),
        token: PrivateString::new("fictional-service".into()),
        security: PrivateString::new("AAECAwQFBgcICQoLDA0ODw==".into()),
    }
}
fn entry(did: &str) -> Value {
    json!({"did":did,"name":"Fictional fan","model":"jdjz.fan.2072fc","isOnline":true})
}
fn page(entries: Vec<Value>, more: bool, cursor: Option<&str>) -> Value {
    json!({"code":0,"result":{"list":entries,"has_more":more,"next_start_did":cursor}})
}
fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder.write_all(bytes).unwrap();
    encoder.finish().unwrap()
}
struct Peer {
    replies: Mutex<VecDeque<Result<Value>>>,
    calls: Mutex<Vec<(ModernReadEndpoint, Value)>>,
    compressed: bool,
    stall: bool,
    properties: bool,
    cancel_on_reply: bool,
}
impl Peer {
    fn new(replies: Vec<Value>) -> Self {
        Self {
            replies: Mutex::new(replies.into_iter().map(Ok).collect()),
            calls: Mutex::new(vec![]),
            compressed: false,
            stall: false,
            properties: false,
            cancel_on_reply: false,
        }
    }
    fn count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }
}
#[async_trait]
impl ModernReadTransport for Peer {
    async fn send_modern_read(
        &self,
        request: &ModernReadRequest,
        operation: &Operation,
    ) -> Result<ModernReadResponse> {
        assert_eq!(format!("{request:?}"), "ModernReadRequest([REDACTED])");
        assert!(
            request
                .endpoint()
                .url()
                .starts_with("https://core.api.mijia.tech/app/")
        );
        assert_eq!(
            request.cookie(),
            "cUserId=fictional-c-user; serviceToken=fictional-service; yetAnotherServiceToken=fictional-service; countryCode=CN"
        );
        assert!(!request.cookie().contains("passToken"));
        let url = Url::parse(&format!(
            "https://example.invalid/?{}",
            std::str::from_utf8(request.body()).unwrap()
        ))
        .unwrap();
        let fields: BTreeMap<_, _> = url.query_pairs().into_owned().collect();
        let key = codec::signed_nonce(&fields["ssecurity"], &fields["_nonce"]).unwrap();
        // Decode a joined stream, independently of the reference request encoder.
        let mut joint = codec::unbase64(&fields["data"]).unwrap();
        let data_len = joint.len();
        joint.extend_from_slice(&codec::unbase64(&fields["rc4_hash__"]).unwrap());
        let decrypted = codec::decrypt(&key, codec::base64(&joint).as_bytes()).unwrap();
        let data = &decrypted.expose()[..data_len];
        let hash = &decrypted.expose()[data_len..];
        use sha1::Digest;
        let signature = format!(
            "POST&{}&data={}&{}",
            request.endpoint().path(),
            std::str::from_utf8(data).unwrap(),
            codec::base64(&key)
        );
        assert_eq!(
            hash,
            codec::base64(&sha1::Sha1::digest(signature.as_bytes())).as_bytes()
        );
        let data: Value = serde_json::from_slice(data).unwrap();
        self.calls
            .lock()
            .unwrap()
            .push((request.endpoint(), data.clone()));
        if self.stall {
            std::future::pending::<()>().await;
        }
        let value = if self.properties && request.endpoint() == ModernReadEndpoint::Properties {
            json!({"code":0,"result":data["params"].as_array().unwrap().iter().enumerate().map(|(i,p)| json!({"did":p["did"],"siid":p["siid"],"piid":p["piid"],"code":0,"value":i as u32})).collect::<Vec<_>>()})
        } else {
            self.replies
                .lock()
                .unwrap()
                .pop_front()
                .expect("No hidden request")?
        };
        let plain = serde_json::to_vec(&value).unwrap();
        let body = if self.compressed { gzip(&plain) } else { plain };
        if self.cancel_on_reply {
            operation.cancellation.cancel();
        }
        Ok(ModernReadResponse {
            body: SecretBytes::new(codec::encrypt(&key, &body).as_str().as_bytes().to_vec()),
            encoding: if self.compressed {
                ModernContentEncoding::Gzip
            } else {
                ModernContentEncoding::Plain
            },
        })
    }
}

#[test]
fn reference_continuous_stream_matches_independent_python_vector_and_differs_from_legacy() {
    let data = r#"{"get_split_device":true}"#;
    let (encoded, key) = codec::encode_reference(
        "/v2/home/device_list_page",
        data,
        "AAECAwQFBgcICQoLDA0ODw==",
        "AAECAwQFBgcICQoL",
    )
    .unwrap();
    assert_eq!(
        std::str::from_utf8(encoded.expose()).unwrap(),
        "data=Gjc%2B3t3uXCiVSODgtnuVD5Q9c74jny%2BObw%3D%3D&rc4_hash__=jkEYHNZjG2NWi7jCv9HUaW1R1osqJIa2jIDUZw%3D%3D&signature=NgFsskNIaIP1LqpyJ2Xs7oojXtI%3D&_nonce=AAECAwQFBgcICQoL&ssecurity=AAECAwQFBgcICQoLDA0ODw%3D%3D"
    );
    let (legacy, legacy_key) = codec::encode(
        "/v2/home/device_list_page",
        data,
        "AAECAwQFBgcICQoLDA0ODw==",
        "AAECAwQFBgcICQoL",
    )
    .unwrap();
    assert_eq!(key.expose(), legacy_key.expose());
    assert_ne!(encoded.expose(), legacy.expose());
    let fields = |bytes: &[u8]| {
        Url::parse(&format!(
            "https://example.invalid/?{}",
            std::str::from_utf8(bytes).unwrap()
        ))
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect::<BTreeMap<_, _>>()
    };
    assert_eq!(
        fields(encoded.expose())["data"],
        fields(legacy.expose())["data"]
    );
    assert_ne!(
        fields(encoded.expose())["rc4_hash__"],
        fields(legacy.expose())["rc4_hash__"]
    );
}

#[tokio::test]
async fn paged_directory_preserves_logical_entries_without_claiming_physical_completeness() {
    for compressed in [false, true] {
        let mut peer = Peer::new(vec![
            page(
                vec![entry("fictional"), entry("fictional.s1")],
                true,
                Some("fictional-cursor"),
            ),
            page(vec![entry("fictional.s2")], false, None),
        ]);
        peer.compressed = compressed;
        let result = session().directory(&peer, &operation()).await.unwrap();
        assert_eq!(result.entries.len(), 3);
        assert!(!result.complete);
        assert!(result.issues.is_empty());
        assert_eq!(result.entries[1].name(), "Fictional fan");
        assert_eq!(result.entries[1].online_hint(), Some(true));
        assert!(!result.entries[1].readable_properties().is_empty());
        assert_eq!(
            format!("{:?}", result.entries[1]),
            "ModernDirectoryEntry([REDACTED])"
        );
        let calls = peer.calls.lock().unwrap();
        assert_eq!(calls[0].1["get_split_device"], true);
        assert!(calls[0].1.get("start_did").is_none());
        assert_eq!(calls[0].1.as_object().unwrap().len(), 8);
        assert_eq!(calls[1].1["start_did"], "fictional-cursor");
    }
}

#[tokio::test]
async fn every_reference_scalar_is_queryable_only_through_its_protected_entry() {
    let profiles: Value =
        serde_json::from_str(include_str!("../../reference_profiles.json")).unwrap();
    let entries:Vec<_>=profiles.as_array().unwrap().iter().enumerate().map(|(i,p)|json!({"did":format!("fictional-{i}"),"name":"Fictional reference entry","model":p["model"],"spec_type":p["spec_type"]})).collect();
    let mut peer = Peer::new(vec![page(entries, false, None)]);
    peer.properties = true;
    let session = session();
    let directory = session.directory(&peer, &operation()).await.unwrap();
    let mut total = 0;
    for entry in &directory.entries {
        let queries = entry.readable_properties();
        assert_eq!(queries.is_empty(), entry.model() == "njcuk.fitting.ad1204");
        total += queries.len();
        for part in queries.chunks(90) {
            let values = session
                .properties(&peer, entry, part, &operation())
                .await
                .unwrap();
            assert_eq!(values.len(), part.len());
            assert_eq!(values[0].value, 0);
        }
    }
    assert_eq!(directory.entries.len(), 17);
    assert_eq!(total, 264);
}

#[tokio::test]
async fn properties_refuse_foreign_accounts_unreviewed_fields_and_mismatched_specs_before_send() {
    let mut unknown = entry("fictional-unknown");
    unknown["model"] = json!("fiction.unknown.v1");
    let mut mismatch = entry("fictional-mismatch");
    mismatch["spec_type"] = json!("urn:miot-spec-v2:device:fan:0000A005:wrong-spec:1");
    let peer = Peer::new(vec![page(
        vec![entry("fictional"), unknown, mismatch],
        false,
        None,
    )]);
    let session = session();
    let directory = session.directory(&peer, &operation()).await.unwrap();
    let entry = &directory.entries[0];
    let valid = entry.readable_properties()[0];
    for queries in [
        vec![],
        vec![valid, valid],
        vec![valid; 91],
        vec![PropertyQuery { siid: 1, piid: 1 }],
        vec![PropertyQuery {
            siid: 999,
            piid: 999,
        }],
    ] {
        assert!(
            session
                .properties(&peer, entry, &queries, &operation())
                .await
                .is_err()
        );
    }
    for entry in &directory.entries[1..] {
        assert!(entry.readable_properties().is_empty());
        assert_eq!(
            session
                .properties(&peer, entry, &[valid], &operation())
                .await
                .unwrap_err()
                .code,
            "CAPABILITY_UNSUPPORTED"
        );
    }
    let mut foreign = super::tests::session();
    foreign.subject = PrivateString::new("54321".into());
    assert_eq!(
        foreign
            .properties(&peer, entry, &[valid], &operation())
            .await
            .unwrap_err()
            .code,
        "DEVICE_IDENTITY_CONFLICT"
    );
    foreign.subject = session.subject.clone();
    foreign.encrypted_subject = PrivateString::new("different-c-user".into());
    assert!(
        foreign
            .properties(&peer, entry, &[valid], &operation())
            .await
            .is_err()
    );
    assert_eq!(peer.count(), 1);
}

#[tokio::test]
async fn failed_missing_and_false_property_values_remain_distinct_and_correlated() {
    let session = session();
    for mode in 0..6 {
        let mut peer = Peer::new(vec![page(vec![entry("fictional")], false, None)]);
        let directory = session.directory(&peer, &operation()).await.unwrap();
        let entry = &directory.entries[0];
        let queries = &entry.readable_properties()[..3];
        let mut rows: Vec<_> = queries
            .iter()
            .map(|q| json!({"did":"fictional","siid":q.siid,"piid":q.piid,"code":0,"value":false}))
            .collect();
        rows[1]["code"] = json!(-1);
        rows[2].as_object_mut().unwrap().remove("value");
        match mode {
            1 => rows[0]["did"] = json!("foreign"),
            2 => rows.push(rows[0].clone()),
            3 => rows[0]["siid"] = json!(999),
            4 => rows[0]["value"] = json!({"private":"opaque"}),
            5 => rows[0]["value"] = json!(null),
            _ => {}
        }
        peer.replies
            .get_mut()
            .unwrap()
            .push_back(Ok(json!({"code":0,"result":rows})));
        let result = session
            .properties(&peer, entry, queries, &operation())
            .await;
        if mode == 0 {
            let result = result.unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].value, false);
        } else if mode == 5 {
            assert!(result.unwrap().is_empty());
        } else {
            assert_eq!(result.unwrap_err().code, "MIJIA_INVALID_RESPONSE");
        }
        assert_eq!(peer.count(), 2);
    }
}

#[tokio::test]
async fn directory_errors_cycles_identity_conflicts_and_limits_are_bounded() {
    for mode in 0..7 {
        let first = page(vec![entry("fictional")], true, Some("cursor"));
        let mut next = page(vec![entry("fictional-next")], false, None);
        match mode {
            0 => next["result"]["list"] = json!([entry("fictional")]),
            1 => {
                next["result"]["list"] = json!([entry("fictional")]);
                next["result"]["list"][0]["model"] = json!("different.model");
            }
            2 => {
                next["result"]["has_more"] = json!(true);
                next["result"]["next_start_did"] = json!("cursor");
            }
            3 => {
                next["result"]["has_more"] = json!(true);
                next["result"]
                    .as_object_mut()
                    .unwrap()
                    .remove("next_start_did");
            }
            4 => {
                next["result"]["has_more"] = json!(true);
                next["result"]["list"] = json!([]);
            }
            5 => next["result"]
                .as_object_mut()
                .unwrap()
                .remove("has_more")
                .map(|_| ())
                .unwrap(),
            _ => next["result"]["list"][0]["did"] = json!(""),
        }
        let peer = Peer::new(vec![first, next]);
        let error = session().directory(&peer, &operation()).await.unwrap_err();
        assert_eq!(
            error.code,
            if mode == 1 {
                "DEVICE_IDENTITY_CONFLICT"
            } else {
                "MIJIA_INVALID_RESPONSE"
            }
        );
        assert_eq!(peer.count(), 2);
    }
    let peer = Peer::new(
        (0..16)
            .map(|i| {
                page(
                    vec![entry(&format!("fictional-{i}"))],
                    true,
                    Some(&format!("cursor-{i}")),
                )
            })
            .collect(),
    );
    let result = session().directory(&peer, &operation()).await.unwrap();
    assert_eq!(peer.count(), 16);
    assert_eq!(result.entries.len(), 16);
    assert_eq!(result.issues[0].code, "MIJIA_INVENTORY_PARTIAL");
    let peer = Peer::new(vec![page(
        (0..4097)
            .map(|i| entry(&format!("fictional-{i}")))
            .collect(),
        false,
        None,
    )]);
    assert_eq!(
        session()
            .directory(&peer, &operation())
            .await
            .unwrap_err()
            .code,
        "MIJIA_INVALID_RESPONSE"
    );
    assert_eq!(peer.count(), 1);
}

#[tokio::test]
async fn later_failures_keep_positive_entries_but_auth_and_cancellation_abort() {
    for code in [
        "MIJIA_REQUEST_FAILED",
        "MIJIA_REQUEST_REJECTED",
        "MIJIA_INVALID_RESPONSE",
        "AUTH_REQUIRED",
        "CANCELLED",
        "DEADLINE_EXCEEDED",
    ] {
        let mut peer = Peer::new(vec![page(vec![entry("fictional")], true, Some("cursor"))]);
        peer.replies
            .get_mut()
            .unwrap()
            .push_back(Err(Error::new(code, "Fictional failure.")));
        let result = session().directory(&peer, &operation()).await;
        if matches!(code, "MIJIA_REQUEST_FAILED" | "MIJIA_REQUEST_REJECTED") {
            let result = result.unwrap();
            assert_eq!(result.entries.len(), 1);
            assert_eq!(result.issues[0].code, code);
            assert!(!result.complete);
        } else {
            assert_eq!(result.unwrap_err().code, code);
        }
        assert_eq!(peer.count(), 2);
    }
    let peer = Peer::new(vec![json!({"code":3})]);
    assert_eq!(
        session()
            .directory(&peer, &operation())
            .await
            .unwrap_err()
            .code,
        "AUTH_REQUIRED"
    );
    assert_eq!(peer.count(), 1);
}

#[tokio::test]
async fn homes_validate_private_identity_and_keep_membership_uncertainty() {
    for has_more in [Some(false), Some(true), None] {
        let peer = Peer::new(vec![
            json!({"code":0,"result":{"homelist":[{"id":"123","uid":456,"name":"Fictional home"}],"has_more":has_more}}),
        ]);
        let result = session().homes(&peer, &operation()).await.unwrap();
        assert_eq!(result.homes[0].name(), Some("Fictional home"));
        assert!(!result.complete);
        assert_eq!(result.issues.is_empty(), has_more == Some(false));
        assert_eq!(format!("{:?}", result.homes[0]), "ModernHome([REDACTED])");
        assert_eq!(peer.calls.lock().unwrap()[0].1["app_ver"], 12);
    }
    for homes in [
        json!([{"id":"001","uid":456}]),
        json!([{"id":0,"uid":456}]),
        json!([{"id":1,"uid":2},{"id":1,"uid":2}]),
        json!([{"id":1,"uid":2,"name":""}]),
    ] {
        let peer = Peer::new(vec![json!({"code":0,"result":{"homelist":homes}})]);
        assert_eq!(
            session().homes(&peer, &operation()).await.unwrap_err().code,
            "MIJIA_INVALID_RESPONSE"
        );
    }
}

#[test]
fn bounded_decode_refuses_plaintext_success_bad_gzip_and_ambiguous_encodings() {
    let key = [7u8; 32];
    let decode = |bytes: &[u8], encoding| {
        decode_response(
            ModernReadResponse {
                body: SecretBytes::new(codec::encrypt(&key, bytes).as_str().as_bytes().to_vec()),
                encoding,
            },
            &key,
        )
    };
    let good = br#"{"code":0,"result":[]}"#;
    assert_eq!(
        decode(good, ModernContentEncoding::Plain).unwrap().expose(),
        good
    );
    assert_eq!(
        decode(&gzip(good), ModernContentEncoding::Gzip)
            .unwrap()
            .expose(),
        good
    );
    let mut corrupt = gzip(good);
    let n = corrupt.len();
    corrupt[n - 5] ^= 1;
    let mut trailing = gzip(good);
    trailing.push(0);
    let mut concat = gzip(good);
    concat.extend(gzip(good));
    for bad in [
        corrupt,
        trailing,
        concat,
        gzip(good)[..10].to_vec(),
        good.to_vec(),
        gzip(&vec![b'x'; MAX_READ_BYTES + 1]),
    ] {
        assert_eq!(
            decode(&bad, ModernContentEncoding::Gzip).unwrap_err().code,
            "MIJIA_INVALID_RESPONSE"
        );
    }
    assert!(decode(&gzip(good), ModernContentEncoding::Plain).is_err());
    for (body, expected) in [
        (good.as_slice(), "MIJIA_INVALID_RESPONSE"),
        (
            br#"{"code":3,"message":"private-marker"}"#.as_slice(),
            "AUTH_REQUIRED",
        ),
        (br#"{"code":99}"#.as_slice(), "MIJIA_REQUEST_REJECTED"),
    ] {
        let error = decode_response(
            ModernReadResponse {
                body: SecretBytes::new(body.to_vec()),
                encoding: ModernContentEncoding::Plain,
            },
            &key,
        )
        .unwrap_err();
        assert_eq!(error.code, expected);
        assert!(!format!("{error:?}").contains("private-marker"));
    }
    assert!(
        decode_response(
            ModernReadResponse {
                body: SecretBytes::new(vec![b'x'; MAX_READ_BYTES + 1]),
                encoding: ModernContentEncoding::Plain
            },
            &key
        )
        .is_err()
    );
}

#[tokio::test(start_paused = true)]
async fn cancellation_deadline_and_dropped_reads_never_retry() {
    let session = session();
    for mode in 0..4 {
        let mut peer = Peer::new(vec![]);
        peer.stall = true;
        let op = operation();
        if mode == 0 {
            op.cancellation.cancel();
        }
        let mut pending = Box::pin(session.directory(&peer, &op));
        if mode == 0 {
            assert_eq!(pending.await.unwrap_err().code, "CANCELLED");
            assert_eq!(peer.count(), 0);
            continue;
        }
        tokio::select! {r=&mut pending=>panic!("Unexpected {r:?}"),_=tokio::task::yield_now()=>{}}
        assert_eq!(peer.count(), 1);
        if mode == 1 {
            op.cancellation.cancel();
            assert_eq!(pending.await.unwrap_err().code, "CANCELLED");
        } else if mode == 2 {
            assert_eq!(pending.await.unwrap_err().code, "DEADLINE_EXCEEDED");
        } else {
            drop(pending);
        }
        assert_eq!(peer.count(), 1);
    }
}

#[tokio::test]
async fn cancelled_delivery_cannot_publish_a_synchronously_ready_response() {
    let mut peer = Peer::new(vec![page(vec![entry("fictional")], false, None)]);
    peer.cancel_on_reply = true;
    assert_eq!(
        session()
            .directory(&peer, &operation())
            .await
            .unwrap_err()
            .code,
        "CANCELLED"
    );
    assert_eq!(peer.count(), 1);
}
