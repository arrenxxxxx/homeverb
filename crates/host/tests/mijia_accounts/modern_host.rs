//! Fictional Host lifecycle checks; no vendor traffic or real QR scanning.
use super::*;
use homeverb_host::{ContinueAccountLoginRequest, ModernAccountRequest, NativeModernQrLogin};
use homeverb_mijia::{
    ModernAuthEndpoint as Auth, ModernAuthRequest, ModernAuthResponse, ModernAuthTransport,
    ModernContentEncoding, ModernReadEndpoint, ModernReadRequest, ModernReadResponse,
    ModernReadTransport,
};

#[derive(Default)]
struct Peer {
    auth: Mutex<Vec<Auth>>,
    reads: AtomicUsize,
    pause: AtomicUsize,
    started: tokio::sync::Notify,
    renewal: AtomicBool,
    on_reply: Mutex<Option<Box<dyn FnOnce() + Send>>>,
}
impl Peer {
    async fn gate(&self, phase: usize) {
        if self.pause.load(Ordering::SeqCst) == phase {
            self.started.notify_one();
            std::future::pending::<()>().await;
        }
    }
    fn hook(&self) {
        if let Some(action) = self.on_reply.lock().unwrap().take() {
            action();
        }
    }
    fn auth_count(&self) -> usize {
        self.auth.lock().unwrap().len()
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
        let renewal = request.cookie().contains("passToken=");
        self.renewal.store(
            renewal || self.renewal.load(Ordering::SeqCst),
            Ordering::SeqCst,
        );
        let body = match request.endpoint() {
            Auth::Bootstrap if !renewal => {
                self.gate(1).await;
                let location = url::Url::parse_with_params(
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
                self.hook();
                json!({"code":0,"loginUrl":"https://c3.account.xiaomi.com/longPolling/login?sid=mijia&ticket=PRIVATE-QR&dc=1&ts=2","lp":"https://c3.lp.account.xiaomi.com/lp/s?k=PRIVATE-POLL"})
            }
            Auth::Poll | Auth::Bootstrap => {
                self.gate(if renewal { 4 } else { 2 }).await;
                json!({"code":0,"userId":"999001","cUserId":"PRIVATE-MODERN-USER","passToken":"PRIVATE-MODERN-PASS",
                    "ssecurity":if renewal {"EBESExQVFhcYGRobHB0eHw=="} else {"AAECAwQFBgcICQoLDA0ODw=="},"location":"https://sts.api.mijia.tech/mijia/sts?ticket=PRIVATE-STS"})
            }
            Auth::Exchange => {
                if self.renewal.load(Ordering::SeqCst) {
                    self.hook();
                }
                json!({})
            }
        };
        Ok(ModernAuthResponse {
            body: secret(&serde_json::to_vec(&body).unwrap()),
            user_id: None,
            c_user_id: None,
            service_token: (request.endpoint() == Auth::Exchange).then(|| {
                secret(if self.renewal.load(Ordering::SeqCst) {
                    b"PRIVATE-RENEWED"
                } else {
                    b"PRIVATE-MODERN"
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
        assert_eq!(request.endpoint(), ModernReadEndpoint::Directory);
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.gate(3).await;
        self.hook();
        let fields: HashMap<_, _> = url::form_urlencoded::parse(request.body())
            .into_owned()
            .collect();
        let key = Sha256::digest(
            [
                STANDARD.decode(&fields["ssecurity"]).unwrap(),
                STANDARD.decode(&fields["_nonce"]).unwrap(),
            ]
            .concat(),
        );
        let response = json!({"code":0,"result":{"list":[{"did":"PRIVATE-MODERN-DID","name":"Modern Fan","model":"jdjz.fan.2072fc"}],"has_more":false}});
        Ok(ModernReadResponse {
            body: secret(
                STANDARD
                    .encode(stream(&key, &serde_json::to_vec(&response).unwrap()))
                    .as_bytes(),
            ),
            encoding: ModernContentEncoding::Plain,
        })
    }
}
#[derive(Default)]
struct ProtectedStore {
    inner: Secrets,
    uncertain: AtomicBool,
    puts: AtomicUsize,
}
#[async_trait]
impl SecretStore for ProtectedStore {
    async fn get(&self, key: &CredentialRef, op: &Operation) -> Result<Option<SecretBytes>> {
        self.inner.get(key, op).await
    }
    async fn put(&self, key: &CredentialRef, value: &SecretBytes, op: &Operation) -> Result<()> {
        self.puts.fetch_add(1, Ordering::SeqCst);
        self.inner.put(key, value, op).await?;
        if self.uncertain.load(Ordering::SeqCst) {
            Err(homeverb_domain::Error::new(
                "SECRET_STORE_WRITE_UNKNOWN",
                "Fictional uncertain store.",
            ))
        } else {
            Ok(())
        }
    }
    async fn delete(&self, key: &CredentialRef, op: &Operation) -> Result<()> {
        self.inner.delete(key, op).await
    }
}
struct ModernFixture {
    directory: PathBuf,
    session: Arc<Session>,
    peer: Arc<Peer>,
    cloud: Arc<Cloud>,
    vault: Arc<AccountVault>,
    secrets: Arc<ProtectedStore>,
}
impl Drop for ModernFixture {
    fn drop(&mut self) {
        self.session.close();
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}
impl ModernFixture {
    fn new() -> Self {
        let directory =
            std::env::temp_dir().join(format!("homeverb-modern-host-{}", Uuid::new_v4()));
        let store = Arc::new(SqliteStore::open(&directory).unwrap());
        let secrets = Arc::new(ProtectedStore::default());
        let vault = Arc::new(AccountVault::new(store.clone(), secrets.clone()));
        let clock = Arc::new(SystemClock);
        let peer = Arc::new(Peer::default());
        let cloud = Arc::new(Cloud::default());
        let adapter = Arc::new(
            MijiaAdapter::new(vault.clone(), cloud.clone(), clock.clone())
                .with_modern_protocol(peer.clone(), peer.clone()),
        );
        let meiju = Arc::new(homeverb_midea::MeijuAdapter::new(
            vault.clone(),
            Arc::new(NoMeijuTraffic),
            clock.clone(),
        ));
        let core = Core::new(
            store,
            vec![adapter.clone(), meiju.clone()],
            clock,
            Policy::default(),
        );
        let session = Arc::new(Session::with_platform_accounts(
            core,
            Some(meiju),
            Some(adapter),
        ));
        Self {
            directory,
            session,
            peer,
            cloud,
            vault,
            secrets,
        }
    }
    async fn begin(&self) -> NativeModernQrLogin {
        self.session
            .begin_modern_account_login(begin_metadata())
            .await
            .unwrap()
    }
    async fn connect(&self) -> homeverb_mijia::ModernAccountDiscovery {
        let qr = self.begin().await;
        self.session
            .continue_modern_account_login(answer_metadata(qr.challenge_id))
            .await
            .unwrap()
    }
    async fn call(&self, request: Value) -> Value {
        let input: HostRequest = serde_json::from_value(json!({"schema_version":"1.0","operation_id":"public","timeout_ms":10000,"request":request})).unwrap();
        self.session.call(input).await
    }
}
fn account_request(id: Uuid) -> ModernAccountRequest {
    serde_json::from_value(json!({"schema_version":"1.0","operation_id":"modern-account","timeout_ms":10000,"account_id":id})).unwrap()
}

#[tokio::test]
async fn protected_qr_completion_and_saved_session_work_are_explicit_and_do_not_publish_cards() {
    let f = ModernFixture::new();
    assert_eq!(f.peer.auth_count(), 0);
    assert_eq!(f.peer.reads.load(Ordering::SeqCst), 0);
    let qr = f.begin().await;
    assert_eq!(format!("{qr:?}"), "NativeModernQrLogin([REDACTED])");
    assert!(
        qr.scan_payload
            .expose()
            .windows(10)
            .any(|v| v == b"PRIVATE-QR")
    );
    assert!((1..=10000).contains(&qr.expires_in_ms));
    assert_eq!(
        *f.peer.auth.lock().unwrap(),
        vec![Auth::Bootstrap, Auth::Qr]
    );
    let result = f
        .session
        .continue_modern_account_login(answer_metadata(qr.challenge_id))
        .await
        .unwrap();
    assert_eq!(result.account.credential_state, CredentialState::Stored);
    assert!(!result.complete);
    assert_eq!(result.entries.unwrap().len(), 1);
    assert!(
        f.call(json!({"method":"list_devices"})).await["data"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let public = f.call(json!({"method":"list_accounts"})).await;
    for marker in [
        "PRIVATE-",
        "999001",
        "credential_ref",
        "ssecurity",
        "scan_payload",
    ] {
        assert!(!public.to_string().contains(marker));
    }
    assert_eq!(f.peer.reads.load(Ordering::SeqCst), 1);
    f.session
        .refresh_modern_account(account_request(result.account.id))
        .await
        .unwrap()
        .entries
        .unwrap();
    let count = f.peer.reads.load(Ordering::SeqCst);
    let renewed = f
        .session
        .renew_modern_account(account_request(result.account.id))
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(renewed).unwrap()["credential_state"],
        "stored"
    );
    assert_eq!(f.peer.reads.load(Ordering::SeqCst), count);
    assert_eq!(f.cloud.calls.load(Ordering::SeqCst), 0);
    assert!(!f.session.discard_modern_account_login(qr.challenge_id));
    assert!(
        f.session
            .continue_modern_account_login(answer_metadata(qr.challenge_id))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn same_subject_host_adoption_preserves_existing_legacy_cards() {
    let f = ModernFixture::new();
    let connected = f
        .session
        .connect_account(
            begin_metadata(),
            secret(b"fiction@example.invalid"),
            secret(b"fictional-password"),
        )
        .await;
    assert!(connected["error"].is_null());
    let before = f.call(json!({"method":"list_devices"})).await;
    let report = f.connect().await;
    assert_eq!(json!(report.account.id), connected["data"]["account"]["id"]);
    assert_eq!(f.call(json!({"method":"list_devices"})).await, before);
    assert_eq!(
        f.call(json!({"method":"list_accounts"})).await["data"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn qr_handles_are_foreign_session_safe_and_bad_metadata_consumes_them() {
    let f = ModernFixture::new();
    let other = ModernFixture::new();
    let qr = f.begin().await;
    assert!(
        other
            .session
            .continue_modern_account_login(answer_metadata(qr.challenge_id))
            .await
            .is_err()
    );
    assert_eq!(other.peer.auth_count(), 0);
    assert!(f.session.discard_modern_account_login(qr.challenge_id));
    assert!(!f.session.discard_modern_account_login(qr.challenge_id));
    for bad in [0, 1, 2] {
        let qr = f.begin().await;
        let mut input = answer_metadata(qr.challenge_id);
        match bad {
            0 => input.schema_version = "future".into(),
            1 => input.timeout_ms = 0,
            _ => input.operation_id.clear(),
        }
        assert!(
            f.session
                .continue_modern_account_login(input)
                .await
                .is_err()
        );
        assert!(
            f.session
                .continue_modern_account_login(answer_metadata(qr.challenge_id))
                .await
                .is_err()
        );
    }
    assert!(!f.peer.auth.lock().unwrap().contains(&Auth::Poll));
}

#[tokio::test]
async fn qr_and_both_legacy_families_share_the_pending_limit() {
    let f = ModernFixture::new();
    let first = f.begin().await;
    f.begin().await;
    f.cloud.captcha.store(true, Ordering::SeqCst);
    assert!(matches!(
        f.session
            .begin_account_login(
                begin_metadata(),
                secret(b"fiction@example.invalid"),
                secret(b"fictional-password")
            )
            .await
            .unwrap(),
        homeverb_host::NativeAccountLogin::Captcha { .. }
    ));
    f.cloud.captcha.store(false, Ordering::SeqCst);
    assert!(matches!(
        f.session
            .begin_account_verification(
                begin_metadata(),
                secret(b"fiction@example.invalid"),
                secret(b"fictional-password")
            )
            .await
            .unwrap(),
        homeverb_host::NativeVerificationLogin::Pending { .. }
    ));
    let calls = f.peer.auth_count();
    let old = f.cloud.calls.load(Ordering::SeqCst);
    assert_eq!(
        f.session
            .begin_modern_account_login(begin_metadata())
            .await
            .unwrap_err()
            .code,
        "RESOURCE_BUSY"
    );
    assert!(
        f.session
            .begin_account_login(
                begin_metadata(),
                secret(b"fiction@example.invalid"),
                secret(b"fictional-password")
            )
            .await
            .is_err()
    );
    assert!(
        f.session
            .begin_account_verification(
                begin_metadata(),
                secret(b"fiction@example.invalid"),
                secret(b"fictional-password")
            )
            .await
            .is_err()
    );
    assert_eq!(f.peer.auth_count(), calls);
    assert_eq!(f.cloud.calls.load(Ordering::SeqCst), old);
    assert!(f.session.discard_modern_account_login(first.challenge_id));
    f.begin().await;
}

#[tokio::test]
async fn expiry_is_pruned_and_never_extended_by_continuation() {
    let f = ModernFixture::new();
    let mut input = begin_metadata();
    input.timeout_ms = 100;
    let qr = f.session.begin_modern_account_login(input).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(110)).await;
    assert_eq!(
        f.session
            .continue_modern_account_login(answer_metadata(qr.challenge_id))
            .await
            .unwrap_err()
            .code,
        "DEADLINE_EXCEEDED"
    );
    assert!(!f.peer.auth.lock().unwrap().contains(&Auth::Poll));
    let mut input = begin_metadata();
    input.timeout_ms = 100;
    let old = f.session.begin_modern_account_login(input).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(110)).await;
    for _ in 0..4 {
        f.begin().await;
    }
    assert!(!f.session.discard_modern_account_login(old.challenge_id));
}

#[tokio::test]
async fn lifecycle_and_discard_clear_pending_qr_without_polling() {
    for mode in 0..4 {
        let f = ModernFixture::new();
        let qr = f.begin().await;
        match mode {
            0 => {
                f.session.set_foreground(false);
                f.session.set_foreground(true);
            }
            1 => {
                f.session.cancel("unregistered");
            }
            2 => f.session.close(),
            _ => {
                assert!(f.session.discard_modern_account_login(qr.challenge_id));
            }
        }
        assert!(
            f.session
                .continue_modern_account_login(answer_metadata(qr.challenge_id))
                .await
                .is_err()
        );
        assert_eq!(f.peer.auth_count(), 2);
    }
}

#[tokio::test]
async fn foreground_close_and_cancel_abort_inflight_begin_and_poll_without_replay() {
    for phase in [1, 2] {
        for mode in 0..3 {
            let f = ModernFixture::new();
            let qr = if phase == 2 {
                Some(f.begin().await)
            } else {
                None
            };
            f.peer.pause.store(phase, Ordering::SeqCst);
            let work = async {
                if let Some(qr) = qr {
                    f.session
                        .continue_modern_account_login(answer_metadata(qr.challenge_id))
                        .await
                        .map(|_| ())
                } else {
                    f.session
                        .begin_modern_account_login(begin_metadata())
                        .await
                        .map(|_| ())
                }
            };
            tokio::pin!(work);
            tokio::select! { _ = f.peer.started.notified() => {}, _ = &mut work => panic!("Must wait for the peer") }
            match mode {
                0 => f.session.set_foreground(false),
                1 => f.session.close(),
                _ => {
                    assert!(f.session.cancel(if phase == 1 {
                        "begin-captcha"
                    } else {
                        "answer-captcha"
                    }));
                }
            }
            assert_eq!(work.await.unwrap_err().code, "CANCELLED");
            assert_eq!(f.secrets.puts.load(Ordering::SeqCst), 0);
            assert!(f.vault.list(&op()).await.unwrap().is_empty());
            assert_eq!(f.peer.auth_count(), if phase == 1 { 1 } else { 3 });
        }
    }
}

#[tokio::test]
async fn late_ready_qr_cannot_resurrect_a_cancelled_lifecycle_generation() {
    let f = ModernFixture::new();
    let session = Arc::downgrade(&f.session);
    *f.peer.on_reply.lock().unwrap() = Some(Box::new(move || {
        let s = session.upgrade().unwrap();
        s.set_foreground(false);
        s.set_foreground(true);
    }));
    assert!(
        f.session
            .begin_modern_account_login(begin_metadata())
            .await
            .is_err()
    );
    assert_eq!(f.peer.auth_count(), 2);
    for _ in 0..4 {
        f.begin().await;
    }
}

#[tokio::test]
async fn cancelled_refresh_and_renewal_release_the_account_lease_and_operation_id() {
    for phase in [3, 4] {
        let f = ModernFixture::new();
        let report = f.connect().await;
        f.peer.pause.store(phase, Ordering::SeqCst);
        let work = async {
            if phase == 3 {
                f.session
                    .refresh_modern_account(account_request(report.account.id))
                    .await
                    .map(|_| ())
            } else {
                f.session
                    .renew_modern_account(account_request(report.account.id))
                    .await
                    .map(|_| ())
            }
        };
        tokio::pin!(work);
        tokio::select! { _ = f.peer.started.notified() => {}, _ = &mut work => panic!("Must wait for the peer") }
        assert!(f.session.cancel("modern-account"));
        assert_eq!(work.await.unwrap_err().code, "CANCELLED");
        let lease = f.vault.lease(report.account.id, &op()).await.unwrap();
        drop(lease);
        f.peer.pause.store(0, Ordering::SeqCst);
        f.session
            .refresh_modern_account(account_request(report.account.id))
            .await
            .unwrap()
            .entries
            .unwrap();
    }
}

#[tokio::test]
async fn dropped_begin_poll_and_account_futures_release_operation_registration() {
    for phase in [1, 2, 3, 4] {
        let f = ModernFixture::new();
        let account = if phase >= 3 {
            Some(f.connect().await.account.id)
        } else {
            None
        };
        let qr = if phase == 2 {
            Some(f.begin().await)
        } else {
            None
        };
        f.peer.pause.store(phase, Ordering::SeqCst);
        {
            let work = async {
                match phase {
                    1 => f
                        .session
                        .begin_modern_account_login(begin_metadata())
                        .await
                        .map(|_| ()),
                    2 => f
                        .session
                        .continue_modern_account_login(answer_metadata(qr.unwrap().challenge_id))
                        .await
                        .map(|_| ()),
                    3 => f
                        .session
                        .refresh_modern_account(account_request(account.unwrap()))
                        .await
                        .map(|_| ()),
                    _ => f
                        .session
                        .renew_modern_account(account_request(account.unwrap()))
                        .await
                        .map(|_| ()),
                }
            };
            tokio::pin!(work);
            tokio::select! { _ = f.peer.started.notified() => {}, _ = &mut work => panic!("Must wait") }
        }
        f.peer.pause.store(0, Ordering::SeqCst);
        if let Some(id) = account {
            f.session
                .refresh_modern_account(account_request(id))
                .await
                .unwrap()
                .entries
                .unwrap();
        } else {
            f.connect().await;
        }
    }
}

#[tokio::test]
async fn pending_saved_session_requires_explicit_recovery_without_authentication_or_rewrite() {
    let f = ModernFixture::new();
    let qr = f.begin().await;
    f.secrets.uncertain.store(true, Ordering::SeqCst);
    assert_eq!(
        f.session
            .continue_modern_account_login(answer_metadata(qr.challenge_id))
            .await
            .unwrap_err()
            .code,
        "SECRET_STORE_WRITE_UNKNOWN"
    );
    let account = f.vault.list(&op()).await.unwrap().remove(0);
    assert_eq!(account.credential_state, CredentialState::Pending);
    assert!(
        f.session
            .refresh_modern_account(account_request(account.id))
            .await
            .is_err()
    );
    assert!(
        f.session
            .renew_modern_account(account_request(account.id))
            .await
            .is_err()
    );
    let auth = f.peer.auth_count();
    let puts = f.secrets.puts.load(Ordering::SeqCst);
    let report = f
        .session
        .recover_modern_account(account_request(account.id))
        .await
        .unwrap();
    report.entries.unwrap();
    assert_eq!(report.account.credential_state, CredentialState::Stored);
    assert_eq!(f.peer.auth_count(), auth);
    assert_eq!(f.secrets.puts.load(Ordering::SeqCst), puts);
}

#[tokio::test]
async fn provider_metadata_and_ordinary_dispatch_refuse_modern_entry_points() {
    let f = ModernFixture::new();
    for provider in [None, Some(homeverb_host::AccountProvider::Meiju)] {
        let mut input = begin_metadata();
        input.provider = provider;
        assert!(f.session.begin_modern_account_login(input).await.is_err());
    }
    assert_eq!(f.peer.auth_count(), 0);
    for method in [
        "begin_modern_account_login",
        "continue_modern_account_login",
        "discard_modern_account_login",
        "refresh_modern_account",
        "recover_modern_account",
        "renew_modern_account",
    ] {
        assert!(serde_json::from_value::<HostRequest>(json!({"schema_version":"1.0","operation_id":"forbidden","timeout_ms":10000,"request":{"method":method}})).is_err());
    }
    assert!(serde_json::from_value::<ModernAccountRequest>(json!({"schema_version":"1.0","operation_id":"forbidden","timeout_ms":10000,"account_id":Uuid::new_v4(),"origin":"https://example.invalid"})).is_err());
    assert!(serde_json::from_value::<ContinueAccountLoginRequest>(json!({"schema_version":"1.0","operation_id":"forbidden","timeout_ms":10000,"challenge_id":1,"scan_payload":"private"})).is_err());
    let legacy = Fixture::new(false);
    assert_eq!(
        legacy
            .session
            .begin_modern_account_login(begin_metadata())
            .await
            .unwrap_err()
            .code,
        "CAPABILITY_UNSUPPORTED"
    );
}

#[tokio::test]
async fn ready_results_cannot_cross_an_invalidated_generation_even_if_the_operation_was_not_cancelled()
 {
    for renewal in [false, true] {
        let f = ModernFixture::new();
        let account = f.connect().await.account.id;
        let session = Arc::downgrade(&f.session);
        *f.peer.on_reply.lock().unwrap() = Some(Box::new(move || {
            // Cancelling an unrelated ID invalidates pending presentation without cancelling this token.
            assert!(!session.upgrade().unwrap().cancel("not-active"));
        }));
        let error = if renewal {
            f.session
                .renew_modern_account(account_request(account))
                .await
                .err()
                .unwrap()
        } else {
            f.session
                .refresh_modern_account(account_request(account))
                .await
                .unwrap_err()
        };
        assert_eq!(error.code, "MIJIA_AUTH_CHALLENGE_REQUIRED");
        // Completed protected writes are not rolled back by a presentation lifecycle change.
        assert_eq!(
            f.vault.list(&op()).await.unwrap()[0].credential_state,
            CredentialState::Stored
        );
        f.session
            .refresh_modern_account(account_request(account))
            .await
            .unwrap()
            .entries
            .unwrap();
    }
}

#[tokio::test]
async fn duplicate_active_operation_consumes_the_second_handle_without_affecting_the_first() {
    let f = ModernFixture::new();
    let first = f.begin().await;
    let second = f.begin().await;
    f.peer.pause.store(2, Ordering::SeqCst);
    let work = f
        .session
        .continue_modern_account_login(answer_metadata(first.challenge_id));
    tokio::pin!(work);
    tokio::select! { _ = f.peer.started.notified() => {}, _ = &mut work => panic!("Poll must wait") }
    assert_eq!(
        f.session
            .continue_modern_account_login(answer_metadata(second.challenge_id))
            .await
            .unwrap_err()
            .code,
        "RESOURCE_BUSY"
    );
    assert!(!f.session.discard_modern_account_login(second.challenge_id));
    assert_eq!(
        f.peer
            .auth
            .lock()
            .unwrap()
            .iter()
            .filter(|e| **e == Auth::Poll)
            .count(),
        1
    );
    f.session.cancel("answer-captcha");
    assert_eq!(work.await.unwrap_err().code, "CANCELLED");
    f.peer.pause.store(0, Ordering::SeqCst);
    f.connect().await;
}

fn validate_modern_result(definition: &str, value: &Value) {
    let mut options = jsonschema::options();
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../schemas");
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        let Some(name) = path
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| n.strip_suffix(".schema.json"))
        else {
            continue;
        };
        let document = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        options = options.with_resource(
            format!("urn:homeverb:schema:{name}"),
            jsonschema::Resource::from_contents(document),
        );
    }
    options.build(&json!({"$ref":format!("urn:homeverb:schema:native-modern-account#/$defs/{definition}")})).unwrap().validate(value).unwrap();
}

#[tokio::test]
async fn native_modern_reports_match_closed_contracts_without_private_membership_or_complete_claims()
 {
    let f = ModernFixture::new();
    let mut report = f.connect().await;
    let account = report.account.id;
    // Even an internal future completeness flag cannot imply a complete physical inventory.
    report.complete = true;
    let report = homeverb_host::ModernAccountReport::connected(report);
    let result = json!({"schema_version":"1.0","data":{"kind":"connected","report":report},"warnings":[],"error":null});
    validate_modern_result("continue_result", &result);
    assert_eq!(
        result["data"]["report"]["directory"]["logical_entry_count"],
        1
    );
    assert_eq!(result["data"]["report"]["directory"]["complete"], false);
    for private in [
        "PRIVATE",
        "Modern Fan",
        "jdjz.fan",
        "credential_ref",
        "999001",
        "device_ids",
        "serviceToken",
    ] {
        assert!(!result.to_string().contains(private));
    }
    for failed in [false, true] {
        let mut report = f
            .session
            .refresh_modern_account(account_request(account))
            .await
            .unwrap();
        if failed {
            report.entries = Err(homeverb_domain::Error::new(
                "MIJIA_REQUEST_FAILED",
                "Fictional directory failure.",
            ));
        } else {
            report.entries = Ok(vec![]);
        }
        let result = json!({"schema_version":"1.0","data":{"kind":"directory","report":homeverb_host::ModernAccountReport::inspected(report)},"warnings":[],"error":null});
        validate_modern_result("session_result", &result);
        assert_eq!(
            result["data"]["report"]["directory"]["logical_entry_count"],
            0
        );
        assert_eq!(
            result["data"]["report"]["directory"]["status"],
            if failed { "failed" } else { "partial" }
        );
    }
    let renewed = f
        .session
        .renew_modern_account(account_request(account))
        .await
        .unwrap();
    validate_modern_result(
        "session_result",
        &json!({"schema_version":"1.0","data":{"kind":"renewed","account":renewed},"warnings":[],"error":null}),
    );
}
