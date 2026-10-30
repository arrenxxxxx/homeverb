//! Fictional modern account lifecycle with real SQLite and an in-memory secret store.
use super::*;
use crate::{
    ModernAccountDiscovery, ModernAuthEndpoint as Auth, ModernAuthRequest, ModernAuthResponse,
    ModernAuthTransport, ModernContentEncoding, ModernReadEndpoint as Read, ModernReadRequest,
    ModernReadResponse, ModernReadTransport,
};

struct Peer {
    auth: Mutex<Vec<Auth>>,
    devices: Mutex<Vec<Value>>,
    property_requests: Mutex<Vec<Value>>,
    reads: Mutex<Vec<Read>>,
    subject: Mutex<String>,
    renewed: AtomicU8,
    fail_read: AtomicU8,
    pause: AtomicU8,
    started: tokio::sync::Notify,
    resume: tokio::sync::Notify,
    hook: Mutex<Option<Box<dyn FnOnce() + Send>>>,
}
impl Peer {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            auth: Mutex::new(vec![]),
            devices: Mutex::new(vec![
                json!({"did":"PRIVATE-MODERN-DEVICE","name":"Modern Fan","model":"jdjz.fan.2072fc","pid":0}),
            ]),
            property_requests: Mutex::new(vec![]),
            reads: Mutex::new(vec![]),
            subject: Mutex::new("999001".into()),
            renewed: AtomicU8::new(0),
            fail_read: AtomicU8::new(0),
            pause: AtomicU8::new(0),
            started: tokio::sync::Notify::new(),
            resume: tokio::sync::Notify::new(),
            hook: Mutex::new(None),
        })
    }
    async fn gate(&self, phase: u8) {
        if self.pause.load(Ordering::SeqCst) == phase {
            self.started.notify_one();
            self.resume.notified().await;
        }
    }
    fn invoke_hook(&self) {
        if let Some(hook) = self.hook.lock().unwrap().take() {
            hook();
        }
    }
}
#[async_trait]
impl ModernAuthTransport for Peer {
    async fn send_modern_auth(
        &self,
        request: &ModernAuthRequest,
        _: &Operation,
    ) -> Result<ModernAuthResponse> {
        self.auth.lock().unwrap().push(request.endpoint());
        let renewing = request.cookie().contains("passToken=");
        if renewing {
            self.renewed.store(1, Ordering::SeqCst);
        }
        let body = match request.endpoint() {
            Auth::Bootstrap if !renewing => {
                let location = reqwest::Url::parse_with_params(
                    "https://account.xiaomi.com/fe/service/login",
                    &[
                        ("sid", "mijia"),
                        ("callback", "https://sts.api.mijia.tech/mijia/sts"),
                        ("_sign", "fictional-sign"),
                        ("qs", "%3Fsid%3Dmijia"),
                    ],
                )
                .unwrap();
                json!({"code":70016,"location":location.as_str()})
            }
            Auth::Qr => {
                json!({"code":0,"loginUrl":"https://c3.account.xiaomi.com/longPolling/login?sid=mijia&ticket=fictional-qr&dc=1&ts=2","lp":"https://c3.lp.account.xiaomi.com/lp/s?k=fictional-poll"})
            }
            Auth::Poll | Auth::Bootstrap => {
                self.gate(1).await;
                json!({"code":0,"userId":*self.subject.lock().unwrap(),"cUserId":"PRIVATE-MODERN-USER","passToken":"PRIVATE-MODERN-PASS",
                    "ssecurity":if renewing {"EBESExQVFhcYGRobHB0eHw=="} else {"AAECAwQFBgcICQoLDA0ODw=="},"location":"https://sts.api.mijia.tech/mijia/sts?ticket=fictional-sts"})
            }
            Auth::Exchange => {
                self.invoke_hook();
                json!({})
            }
        };
        Ok(ModernAuthResponse {
            body: SecretBytes::new(serde_json::to_vec(&body).unwrap()),
            user_id: None,
            c_user_id: None,
            service_token: (request.endpoint() == Auth::Exchange).then(|| {
                SecretBytes::new(if self.renewed.load(Ordering::SeqCst) == 0 {
                    b"PRIVATE-MODERN-TOKEN".to_vec()
                } else {
                    b"PRIVATE-RENEWED-TOKEN".to_vec()
                })
            }),
        })
    }
}
#[async_trait]
impl ModernReadTransport for Peer {
    async fn send_modern_read(
        &self,
        request: &ModernReadRequest,
        _: &Operation,
    ) -> Result<ModernReadResponse> {
        self.reads.lock().unwrap().push(request.endpoint());
        assert!(!request.cookie().contains("PRIVATE-SERVICE-TOKEN"));
        assert!(!request.cookie().contains("passToken"));
        self.gate(2).await;
        self.invoke_hook();
        if self.fail_read.load(Ordering::SeqCst) != 0 {
            return Err(crate::failed());
        }
        let url = reqwest::Url::parse(&format!(
            "https://example.invalid/?{}",
            std::str::from_utf8(request.body()).unwrap()
        ))
        .unwrap();
        let fields: HashMap<_, _> = url.query_pairs().into_owned().collect();
        let key = codec::signed_nonce(&fields["ssecurity"], &fields["_nonce"])?;
        let data: Value =
            serde_json::from_slice(codec::decrypt(&key, fields["data"].as_bytes())?.expose())
                .unwrap();
        let result = match request.endpoint() {
            Read::Directory => {
                json!({"list":self.devices.lock().unwrap().clone(),"has_more":false})
            }
            Read::Properties => {
                let params = data["params"].as_array().unwrap();
                self.property_requests
                    .lock()
                    .unwrap()
                    .extend(params.clone());
                json!(params.iter().map(|p| {
                    let devices = self.devices.lock().unwrap();
                    let device = devices.iter().find(|d| d["did"] == p["did"]).unwrap();
                    let mapping = ModelMapping::pinned(device["model"].as_str().unwrap(), None).unwrap().unwrap();
                    let property = mapping.properties.iter().find(|v| json!(v.siid) == p["siid"] && json!(v.piid) == p["piid"]).unwrap();
                    let value = match property.definition["type"].as_str().unwrap() {
                        "boolean" => json!(false),
                        _ => property.definition["enum"].as_array().and_then(|v| v.first()).cloned()
                            .or_else(|| property.definition.get("minimum").cloned()).unwrap_or(json!(0)),
                    };
                    json!({"did":p["did"],"siid":p["siid"],"piid":p["piid"],"code":0,"value":value})
                }).collect::<Vec<_>>())
            }
            _ => panic!("No hidden home request"),
        };
        Ok(ModernReadResponse {
            body: SecretBytes::new(
                codec::encrypt(
                    &key,
                    &serde_json::to_vec(&json!({"code":0,"result":result})).unwrap(),
                )
                .as_str()
                .as_bytes()
                .to_vec(),
            ),
            encoding: ModernContentEncoding::Plain,
        })
    }
}
fn adapter(f: &Fixture, peer: &Arc<Peer>) -> MijiaAdapter {
    MijiaAdapter::new(f.vault.clone(), f.cloud.clone(), Arc::new(SystemClock))
        .with_modern_protocol(peer.clone(), peer.clone())
}
async fn connect(adapter: &MijiaAdapter) -> ModernAccountDiscovery {
    let pending = adapter
        .begin_modern_connection(None, "Modern Home", &operation())
        .await
        .unwrap();
    adapter
        .continue_modern_connection(pending, &operation())
        .await
        .unwrap()
}
fn bytes(f: &Fixture, account: &AccountRecord) -> Vec<u8> {
    f.secrets.values.lock().unwrap()[&account.credential_ref.to_string()].clone()
}
fn payload(f: &Fixture, account: &AccountRecord) -> Value {
    serde_json::from_slice(&bytes(f, account)).unwrap()
}
fn puts(f: &Fixture) -> usize {
    f.secrets.puts.load(Ordering::SeqCst)
}

#[tokio::test]
async fn modern_only_reopens_reads_and_renews_without_publishing_or_legacy_requests() {
    let f = Fixture::new();
    let p = Peer::new();
    let a = adapter(&f, &p);
    let report = connect(&a).await;
    let record = report.account.clone();
    assert!(!report.complete);
    assert!(report.issues.is_empty());
    assert_eq!(record.credential_state, CredentialState::Stored);
    assert_eq!(payload(&f, &record)["version"], 2);
    assert!(payload(&f, &record)["session"].is_null());
    assert!(
        payload(&f, &record)["bindings"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(f.core.list().unwrap().is_empty());
    let entries = report.entries.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name(), "Modern Fan");
    let reopened = adapter(&f, &p);
    let legacy_attempt = reopened
        .refresh_devices(record.id, &operation())
        .await
        .unwrap();
    assert_eq!(
        legacy_attempt.descriptors.unwrap_err().code,
        "CAPABILITY_UNSUPPORTED"
    );
    let q = entries[0].readable_properties();
    let values = reopened
        .read_modern_entry(&entries[0], &q[..1], &operation())
        .await
        .unwrap();
    assert_eq!(values.len(), 1);
    let old = payload(&f, &record);
    let count = p.reads.lock().unwrap().len();
    reopened
        .renew_modern_account(record.id, &operation())
        .await
        .unwrap();
    let new = payload(&f, &record);
    assert_eq!(old["modern"]["device"], new["modern"]["device"]);
    assert_eq!(old["modern"]["pass_token"], new["modern"]["pass_token"]);
    assert_ne!(old["modern"]["security"], new["modern"]["security"]);
    assert_ne!(old["modern"]["token"], new["modern"]["token"]);
    assert_eq!(p.reads.lock().unwrap().len(), count);
    reopened
        .refresh_modern_directory(record.id, &operation())
        .await
        .unwrap()
        .entries
        .unwrap();
    assert!(f.cloud.calls.lock().unwrap().is_empty());
    assert_eq!(
        format!("{:?}", entries[0]),
        "ModernAccountEntry([REDACTED])"
    );
    for file in std::fs::read_dir(&f.directory.0).unwrap() {
        let path = file.unwrap().path();
        if path.is_file() {
            let content = std::fs::read(path).unwrap();
            for secret in [
                "PRIVATE-MODERN",
                "PRIVATE-RENEWED",
                "AAECAwQFBgcICQoLDA0ODw==",
            ] {
                assert!(
                    !content
                        .windows(secret.len())
                        .any(|v| v == secret.as_bytes())
                );
            }
        }
    }
}

#[tokio::test]
async fn same_subject_adopts_legacy_account_and_preserves_bindings_labels_and_old_reads() {
    let f = Fixture::new();
    let legacy = f.connect().await;
    f.publish(&legacy).await;
    let before = payload(&f, &legacy.account);
    let p = Peer::new();
    let a = adapter(&f, &p);
    let modern = connect(&a).await;
    assert_eq!(modern.account.id, legacy.account.id);
    assert_eq!(modern.account.label, "Home");
    assert_eq!(a.accounts(&operation()).await.unwrap().len(), 1);
    let after = payload(&f, &legacy.account);
    assert_eq!(before["session"], after["session"]);
    assert_eq!(before["bindings"], after["bindings"]);
    a.renew_modern_account(legacy.account.id, &operation())
        .await
        .unwrap();
    let renewed = payload(&f, &legacy.account);
    assert_eq!(before["session"], renewed["session"]);
    assert_eq!(before["bindings"], renewed["bindings"]);
    let refreshed = a
        .refresh_devices(legacy.account.id, &operation())
        .await
        .unwrap();
    assert_eq!(refreshed.descriptors.unwrap().len(), 2);
    assert_eq!(renewed["modern"], payload(&f, &legacy.account)["modern"]);
}

#[tokio::test]
async fn legacy_login_adopts_modern_account_without_erasing_modern_slot() {
    let f = Fixture::new();
    let p = Peer::new();
    let a = adapter(&f, &p);
    let modern = connect(&a).await;
    let before = payload(&f, &modern.account);
    let legacy = f.connect().await;
    assert_eq!(legacy.account.id, modern.account.id);
    assert_eq!(legacy.account.label, "Modern Home");
    let after = payload(&f, &modern.account);
    assert_eq!(after["version"], 2);
    assert_eq!(before["modern"], after["modern"]);
    assert!(!after["bindings"].as_array().unwrap().is_empty());
    a.refresh_modern_directory(modern.account.id, &operation())
        .await
        .unwrap()
        .entries
        .unwrap();
}

#[tokio::test]
async fn payload_v1_bytes_are_stable_and_v2_refuses_tampered_protocol_identity() {
    let f = Fixture::new();
    let legacy = f.connect().await;
    let raw = bytes(&f, &legacy.account);
    let decoded = Payload::decode(&SecretBytes::new(raw.clone()), &legacy.account).unwrap();
    assert_eq!(decoded.encode(&legacy.account).unwrap().expose(), raw);
    let old = payload(&f, &legacy.account);
    assert_eq!(old["version"], 1);
    assert!(old.get("modern").is_none());
    let p = Peer::new();
    let a = adapter(&f, &p);
    let modern = connect(&a).await;
    let baseline = payload(&f, &modern.account);
    for (pointer, value) in [
        ("/version", json!(1)),
        ("/modern/profile", json!("alternate")),
        ("/modern/subject", json!("999002")),
        ("/modern/subject", json!("0999001")),
        ("/modern/device", json!("untrusted")),
        ("/modern/agent", json!("arbitrary agent")),
        ("/modern/pass_o", json!("1234567890ABCDEF")),
        ("/modern/security", json!("invalid")),
        ("/modern/token", json!("unsafe; cookie")),
        ("/modern", Value::Null),
        ("/session", Value::Null),
    ] {
        let mut bad = baseline.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(
            Payload::decode(
                &SecretBytes::new(serde_json::to_vec(&bad).unwrap()),
                &modern.account
            )
            .is_err(),
            "{pointer}"
        );
    }
    let mut bad = baseline.clone();
    bad["modern"]["origin"] = json!("https://example.invalid");
    assert!(
        Payload::decode(
            &SecretBytes::new(serde_json::to_vec(&bad).unwrap()),
            &modern.account
        )
        .is_err()
    );
    let mut copied = modern.account.clone();
    copied.id = Uuid::new_v4();
    assert!(Payload::decode(&SecretBytes::new(bytes(&f, &modern.account)), &copied).is_err());
}

#[tokio::test]
async fn missing_protocol_and_unavailable_storage_refuse_before_network() {
    let f = Fixture::new();
    assert_eq!(
        f.adapter
            .begin_modern_connection(None, "Home", &operation())
            .await
            .unwrap_err()
            .code,
        "CAPABILITY_UNSUPPORTED"
    );
    let p = Peer::new();
    let a = adapter(&f, &p);
    f.secrets.mode.store(1, Ordering::SeqCst);
    assert_eq!(
        a.begin_modern_connection(None, "Home", &operation())
            .await
            .unwrap_err()
            .code,
        "SECRET_STORE_UNAVAILABLE"
    );
    assert!(p.auth.lock().unwrap().is_empty());
}

#[tokio::test]
async fn uncertain_save_requires_exact_recovery_without_new_auth_or_secret_write() {
    let f = Fixture::new();
    let p = Peer::new();
    let a = adapter(&f, &p);
    let pending = a
        .begin_modern_connection(None, "Home", &operation())
        .await
        .unwrap();
    f.secrets.mode.store(2, Ordering::SeqCst);
    assert_eq!(
        a.continue_modern_connection(pending, &operation())
            .await
            .unwrap_err()
            .code,
        "SECRET_STORE_WRITE_UNKNOWN"
    );
    let record = a.accounts(&operation()).await.unwrap().remove(0);
    assert_eq!(record.credential_state, CredentialState::Pending);
    let saved = bytes(&f, &record);
    let count = puts(&f);
    let auth = p.auth.lock().unwrap().len();
    assert!(
        a.refresh_modern_directory(record.id, &operation())
            .await
            .is_err()
    );
    assert!(
        a.renew_modern_account(record.id, &operation())
            .await
            .is_err()
    );
    assert!(p.reads.lock().unwrap().is_empty());
    f.secrets.mode.store(0, Ordering::SeqCst);
    p.fail_read.store(1, Ordering::SeqCst);
    let fail = a
        .recover_modern_directory(record.id, &operation())
        .await
        .unwrap();
    assert!(fail.entries.is_err());
    assert_eq!(fail.account.credential_state, CredentialState::Pending);
    p.fail_read.store(0, Ordering::SeqCst);
    let good = a
        .recover_modern_directory(record.id, &operation())
        .await
        .unwrap();
    good.entries.unwrap();
    assert_eq!(good.account.credential_state, CredentialState::Stored);
    assert_eq!(bytes(&f, &record), saved);
    assert_eq!(puts(&f), count);
    assert_eq!(p.auth.lock().unwrap().len(), auth);
}

#[tokio::test]
async fn changed_secret_during_recovery_stays_pending() {
    let f = Fixture::new();
    let p = Peer::new();
    let a = adapter(&f, &p);
    let pending = a
        .begin_modern_connection(None, "Home", &operation())
        .await
        .unwrap();
    f.secrets.mode.store(2, Ordering::SeqCst);
    a.continue_modern_connection(pending, &operation())
        .await
        .unwrap_err();
    f.secrets.mode.store(0, Ordering::SeqCst);
    let record = a.accounts(&operation()).await.unwrap().remove(0);
    let secrets = f.secrets.clone();
    let key = record.credential_ref.to_string();
    *p.hook.lock().unwrap() = Some(Box::new(move || {
        secrets
            .values
            .lock()
            .unwrap()
            .get_mut(&key)
            .unwrap()
            .push(b' ');
    }));
    let report = a
        .recover_modern_directory(record.id, &operation())
        .await
        .unwrap();
    assert_eq!(
        report.entries.unwrap_err().code,
        "ACCOUNT_SESSION_UNCONFIRMED"
    );
    assert_eq!(report.account.credential_state, CredentialState::Pending);
}

#[tokio::test]
async fn qr_owner_snapshot_and_subject_conflicts_cannot_overwrite_accounts() {
    let f = Fixture::new();
    let legacy = f.connect().await;
    let p = Peer::new();
    let a = adapter(&f, &p);
    let pending = a
        .begin_modern_connection(None, "Home", &operation())
        .await
        .unwrap();
    assert_eq!(
        adapter(&f, &p)
            .continue_modern_connection(pending, &operation())
            .await
            .unwrap_err()
            .code,
        "ACCOUNT_SESSION_UNCONFIRMED"
    );
    assert_eq!(p.auth.lock().unwrap().len(), 2);
    let pending = a
        .begin_modern_connection(None, "Home", &operation())
        .await
        .unwrap();
    // No lease survives while the user scans; even an external saved-byte change is detected.
    let lease = f
        .vault
        .lease(legacy.account.id, &operation())
        .await
        .unwrap();
    drop(lease);
    f.secrets
        .values
        .lock()
        .unwrap()
        .get_mut(&legacy.account.credential_ref.to_string())
        .unwrap()
        .push(b' ');
    assert_eq!(
        a.continue_modern_connection(pending, &operation())
            .await
            .unwrap_err()
            .code,
        "ACCOUNT_SESSION_UNCONFIRMED"
    );
    assert!(!p.auth.lock().unwrap().contains(&Auth::Poll));
    let pending = a
        .begin_modern_connection(Some(legacy.account.id), "Home", &operation())
        .await
        .unwrap();
    *p.subject.lock().unwrap() = "999002".into();
    let saved = bytes(&f, &legacy.account);
    let count = puts(&f);
    assert_eq!(
        a.continue_modern_connection(pending, &operation())
            .await
            .unwrap_err()
            .code,
        "ACCOUNT_IDENTITY_CONFLICT"
    );
    assert_eq!(bytes(&f, &legacy.account), saved);
    assert_eq!(puts(&f), count);
}

#[tokio::test]
async fn qr_snapshot_is_checked_again_after_provider_exchange() {
    let f = Fixture::new();
    let legacy = f.connect().await;
    let p = Peer::new();
    let a = adapter(&f, &p);
    let pending = a
        .begin_modern_connection(None, "Home", &operation())
        .await
        .unwrap();
    let secrets = f.secrets.clone();
    let key = legacy.account.credential_ref.to_string();
    let count = puts(&f);
    *p.hook.lock().unwrap() = Some(Box::new(move || {
        secrets
            .values
            .lock()
            .unwrap()
            .get_mut(&key)
            .unwrap()
            .push(b' ');
    }));
    assert_eq!(
        a.continue_modern_connection(pending, &operation())
            .await
            .unwrap_err()
            .code,
        "ACCOUNT_SESSION_UNCONFIRMED"
    );
    assert_eq!(puts(&f), count);
    assert!(p.reads.lock().unwrap().is_empty());
}

#[tokio::test]
async fn original_and_continuation_cancellation_prevent_persistence() {
    for original_cancel in [true, false] {
        let f = Fixture::new();
        let p = Peer::new();
        let a = adapter(&f, &p);
        let original = operation();
        let next = operation();
        let pending = a
            .begin_modern_connection(None, "Home", &original)
            .await
            .unwrap();
        let token = if original_cancel {
            original.cancellation.clone()
        } else {
            next.cancellation.clone()
        };
        *p.hook.lock().unwrap() = Some(Box::new(move || token.cancel()));
        assert_eq!(
            a.continue_modern_connection(pending, &next)
                .await
                .unwrap_err()
                .code,
            "CANCELLED"
        );
        assert_eq!(puts(&f), 0);
        assert!(a.accounts(&operation()).await.unwrap().is_empty());
    }
}

#[tokio::test]
async fn cancelled_or_dropped_directory_releases_lease_and_blocks_concurrent_renewal() {
    let f = Fixture::new();
    let p = Peer::new();
    let a = adapter(&f, &p);
    let report = connect(&a).await;
    p.pause.store(2, Ordering::SeqCst);
    let op = operation();
    let read = a.refresh_modern_directory(report.account.id, &op);
    tokio::pin!(read);
    tokio::select! { _ = p.started.notified() => {}, _ = &mut read => panic!("Read must wait") }
    let short = Operation::new(1, CancellationToken::new()).unwrap();
    assert!(
        a.renew_modern_account(report.account.id, &short)
            .await
            .is_err()
    );
    op.cancellation.cancel();
    assert_eq!(read.await.unwrap().entries.unwrap_err().code, "CANCELLED");
    let lease = f
        .vault
        .lease(report.account.id, &operation())
        .await
        .unwrap();
    drop(lease);
    let op = operation();
    {
        let read = a.refresh_modern_directory(report.account.id, &op);
        tokio::pin!(read);
        tokio::select! { _ = p.started.notified() => {}, _ = &mut read => panic!("Read must wait") }
    }
    let lease = f
        .vault
        .lease(report.account.id, &operation())
        .await
        .unwrap();
    drop(lease);
}

#[tokio::test]
async fn leading_zero_subjects_cannot_create_cross_protocol_duplicates() {
    for modern_first in [true, false] {
        let f = Fixture::new();
        let p = Peer::new();
        let a = adapter(&f, &p);
        if modern_first {
            connect(&a).await;
            *f.cloud.subject.lock().unwrap() = "0999001".into();
            assert_eq!(
                f.adapter
                    .connect_devices(credentials(), None, "Duplicate", &operation())
                    .await
                    .err()
                    .unwrap()
                    .code,
                "ACCOUNT_IDENTITY_CONFLICT"
            );
        } else {
            *f.cloud.subject.lock().unwrap() = "0999001".into();
            f.connect().await;
            let pending = a
                .begin_modern_connection(None, "Duplicate", &operation())
                .await
                .unwrap();
            assert_eq!(
                a.continue_modern_connection(pending, &operation())
                    .await
                    .unwrap_err()
                    .code,
                "ACCOUNT_IDENTITY_CONFLICT"
            );
        }
        assert_eq!(a.accounts(&operation()).await.unwrap().len(), 1);
    }
}

#[tokio::test]
async fn failed_renewal_preserves_old_slots_and_uncertain_renewal_requires_recovery() {
    let f = Fixture::new();
    let legacy = f.connect().await;
    let p = Peer::new();
    let a = adapter(&f, &p);
    let modern = connect(&a).await;
    let saved = bytes(&f, &modern.account);
    let count = puts(&f);
    *p.subject.lock().unwrap() = "999002".into();
    assert!(
        a.renew_modern_account(modern.account.id, &operation())
            .await
            .is_err()
    );
    assert_eq!(bytes(&f, &modern.account), saved);
    assert_eq!(puts(&f), count);
    *p.subject.lock().unwrap() = "999001".into();
    f.secrets.mode.store(2, Ordering::SeqCst);
    assert_eq!(
        a.renew_modern_account(modern.account.id, &operation())
            .await
            .unwrap_err()
            .code,
        "SECRET_STORE_WRITE_UNKNOWN"
    );
    assert_eq!(
        a.account(modern.account.id, &operation())
            .await
            .unwrap()
            .credential_state,
        CredentialState::Pending
    );
    let now = payload(&f, &legacy.account);
    let before: Value = serde_json::from_slice(&saved).unwrap();
    assert_eq!(before["session"], now["session"]);
    assert_eq!(before["bindings"], now["bindings"]);
    assert_ne!(before["modern"]["token"], now["modern"]["token"]);
    f.secrets.mode.store(0, Ordering::SeqCst);
    let auth = p.auth.lock().unwrap().len();
    let count = puts(&f);
    let report = a
        .recover_modern_directory(modern.account.id, &operation())
        .await
        .unwrap();
    report.entries.unwrap();
    assert_eq!(report.account.credential_state, CredentialState::Stored);
    assert_eq!(p.auth.lock().unwrap().len(), auth);
    assert_eq!(puts(&f), count);
}

#[tokio::test]
async fn entry_cannot_cross_a_changed_protected_credential_reference_or_subject() {
    let f = Fixture::new();
    let p = Peer::new();
    let a = adapter(&f, &p);
    let report = connect(&a).await;
    let entries = report.entries.unwrap();
    let queries = entries[0].readable_properties();
    let count = p.reads.lock().unwrap().len();
    let mut record = report.account.clone();
    record.credential_ref = CredentialRef::new();
    let raw = bytes(&f, &report.account);
    assert!(Payload::decode(&SecretBytes::new(raw.clone()), &record).is_err());
    let mut changed: Value = serde_json::from_slice(&raw).unwrap();
    changed["modern"]["subject"] = json!("999002");
    f.secrets.values.lock().unwrap().insert(
        report.account.credential_ref.to_string(),
        serde_json::to_vec(&changed).unwrap(),
    );
    assert!(
        a.read_modern_entry(&entries[0], &queries[..1], &operation())
            .await
            .is_err()
    );
    assert_eq!(p.reads.lock().unwrap().len(), count);
}

#[tokio::test]
async fn deadline_and_removed_account_refuse_delayed_qr_completion() {
    let f = Fixture::new();
    let p = Peer::new();
    let a = adapter(&f, &p);
    let original = Operation::new(20, CancellationToken::new()).unwrap();
    let pending = a
        .begin_modern_connection(None, "Home", &original)
        .await
        .unwrap();
    tokio::time::sleep_until(original.deadline).await;
    assert_eq!(
        a.continue_modern_connection(pending, &operation())
            .await
            .unwrap_err()
            .code,
        "DEADLINE_EXCEEDED"
    );
    assert_eq!(puts(&f), 0);
    assert_eq!(p.auth.lock().unwrap().len(), 2);
    let report = connect(&a).await;
    let pending = a
        .begin_modern_connection(None, "Home", &operation())
        .await
        .unwrap();
    a.remove_account(report.account.id, &[], &operation())
        .await
        .unwrap();
    let auth = p.auth.lock().unwrap().len();
    assert_eq!(
        a.continue_modern_connection(pending, &operation())
            .await
            .unwrap_err()
            .code,
        "ACCOUNT_SESSION_UNCONFIRMED"
    );
    assert_eq!(p.auth.lock().unwrap().len(), auth);
    assert!(a.accounts(&operation()).await.unwrap().is_empty());
}

#[tokio::test]
async fn original_cancellation_during_poll_releases_registration_and_account_locks() {
    let f = Fixture::new();
    let p = Peer::new();
    let a = adapter(&f, &p);
    let saved = connect(&a).await;
    let original = operation();
    let pending = a
        .begin_modern_connection(None, "Home", &original)
        .await
        .unwrap();
    p.pause.store(1, Ordering::SeqCst);
    let next = operation();
    let continuation = a.continue_modern_connection(pending, &next);
    tokio::pin!(continuation);
    tokio::select! { _ = p.started.notified() => {}, _ = &mut continuation => panic!("Poll must wait") }
    let registration = f.vault.lock_registration(&operation()).await.unwrap();
    let count = puts(&f);
    original.cancellation.cancel();
    assert_eq!(continuation.await.unwrap_err().code, "CANCELLED");
    drop(registration);
    assert_eq!(puts(&f), count);
    let registration = f.vault.lock_registration(&operation()).await.unwrap();
    drop(registration);
    let lease = f.vault.lease(saved.account.id, &operation()).await.unwrap();
    drop(lease);
}

struct CancelAfterPut {
    inner: Arc<Secrets>,
    cancellation: CancellationToken,
}
#[async_trait]
impl SecretStore for CancelAfterPut {
    async fn get(&self, key: &CredentialRef, op: &Operation) -> Result<Option<SecretBytes>> {
        self.inner.get(key, op).await
    }
    async fn put(&self, key: &CredentialRef, value: &SecretBytes, op: &Operation) -> Result<()> {
        self.inner.put(key, value, op).await?;
        self.cancellation.cancel();
        Ok(())
    }
    async fn delete(&self, key: &CredentialRef, op: &Operation) -> Result<()> {
        self.inner.delete(key, op).await
    }
}

#[tokio::test]
async fn original_cancellation_during_secret_write_keeps_pending_bytes_for_explicit_recovery() {
    let f = Fixture::new();
    let p = Peer::new();
    let original = operation();
    let secrets = Arc::new(CancelAfterPut {
        inner: f.secrets.clone(),
        cancellation: original.cancellation.clone(),
    });
    let vault = Arc::new(AccountVault::new(f.store.clone(), secrets));
    let a = MijiaAdapter::new(vault, f.cloud.clone(), Arc::new(SystemClock))
        .with_modern_protocol(p.clone(), p.clone());
    let pending = a
        .begin_modern_connection(None, "Home", &original)
        .await
        .unwrap();
    assert_eq!(
        a.continue_modern_connection(pending, &operation())
            .await
            .unwrap_err()
            .code,
        "CANCELLED"
    );
    let record = a.accounts(&operation()).await.unwrap().remove(0);
    assert_eq!(record.credential_state, CredentialState::Pending);
    assert!(p.reads.lock().unwrap().is_empty());
    let before = bytes(&f, &record);
    let count = puts(&f);
    let recovered = a
        .recover_modern_directory(record.id, &operation())
        .await
        .unwrap();
    recovered.entries.unwrap();
    assert_eq!(recovered.account.credential_state, CredentialState::Stored);
    assert_eq!(bytes(&f, &record), before);
    assert_eq!(puts(&f), count);
}

#[path = "modern_accounts/devices.rs"]
mod devices;
