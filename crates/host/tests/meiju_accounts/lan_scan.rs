use super::*;
use homeverb_domain::Descriptor;
use homeverb_host::PendingMeijuLanScan;
use homeverb_midea::lan::{
    DiscoveryCandidate, DiscoveryNetwork, DiscoveryReport, LanDiscovery, LanProtocol,
};
use std::time::Duration;
use tokio::sync::Notify;

#[path = "lan_scan/native.rs"]
mod native;

#[derive(Default)]
struct Discovery {
    reply: Mutex<Option<DiscoveryReport>>,
    calls: AtomicUsize,
    stall: AtomicBool,
    fail: AtomicBool,
    started: Notify,
    resume: Notify,
}

#[async_trait]
impl LanDiscovery for Discovery {
    async fn discover(&self, network: &DiscoveryNetwork, _: &Operation) -> Result<DiscoveryReport> {
        assert_eq!(network.local_address().to_string(), "192.168.42.2");
        assert_eq!(network.prefix_length(), 24);
        self.calls.fetch_add(1, Ordering::Relaxed);
        if self.fail.load(Ordering::Relaxed) {
            return Err(Error::new(
                "MIDEA_REQUEST_FAILED",
                "Synthetic discovery failure.",
            ));
        }
        if self.stall.load(Ordering::Relaxed) {
            self.started.notify_one();
            self.resume.notified().await;
        }
        Ok(self.reply.lock().unwrap().take().unwrap())
    }
}

fn network() -> DiscoveryNetwork {
    DiscoveryNetwork::new("192.168.42.2".parse().unwrap(), 24).unwrap()
}

fn session(f: &Fixture, discovery: Arc<Discovery>) -> Arc<Session> {
    let adapter = Arc::new(
        MeijuAdapter::new(f.vault.clone(), f.wire.clone(), Arc::new(SystemClock))
            .with_lan_discovery(discovery),
    );
    Arc::new(Session::with_accounts(
        Core::new(
            f.store.clone(),
            vec![adapter.clone()],
            Arc::new(SystemClock),
            Policy::default(),
        ),
        adapter,
    ))
}

async fn setup(family: &str, model: &str) -> (Fixture, Arc<Discovery>, Descriptor, CloudId) {
    let mut f = Fixture::new();
    *f.wire.product_fields.lock().unwrap() = Some(json!({
        "applianceCode":"123456789", "type":family, "productModel":model
    }));
    assert_eq!(
        f.connect(None).await["data"]["retrieval"]["status"],
        "complete"
    );
    let descriptor = f
        .store
        .devices()
        .unwrap()
        .into_iter()
        .find(|d| d.device.model == model)
        .unwrap();
    let account_id = descriptor.device.connections[0]
        .account_id
        .as_ref()
        .unwrap()
        .parse()
        .unwrap();
    let account = MeijuAccount::open(
        f.vault.lease(account_id, &operation()).await.unwrap(),
        &operation(),
    )
    .await
    .unwrap();
    let id = account
        .resolve_device(descriptor.device.id.parse().unwrap(), &operation())
        .await
        .unwrap();
    drop(account);
    let discovery = Arc::new(Discovery::default());
    f.session = session(&f, discovery.clone());
    (f, discovery, descriptor, id)
}

async fn preview(f: &Fixture, descriptor: &Descriptor) -> Result<PendingMeijuLanScan> {
    f.session
        .prepare_meiju_lan_scan(
            "preview".into(),
            &descriptor.device.id,
            &descriptor.device.connections[0].id,
            network(),
            1000,
        )
        .await
}

fn hint(id: &CloudId, family: ApplianceFamily, host: u8) -> DiscoveryCandidate {
    DiscoveryCandidate {
        appliance_id: id.clone(),
        endpoint: format!("192.168.42.{host}:6444").parse().unwrap(),
        protocol: LanProtocol::V3,
        family,
    }
}

fn report(candidates: Vec<DiscoveryCandidate>) -> DiscoveryReport {
    DiscoveryReport {
        received_datagrams: candidates.len(),
        candidates,
        limit_reached: false,
    }
}

fn edit_binding(f: &Fixture, device: &str, edit: impl FnOnce(&mut Value)) {
    let mut values = f.secrets.values.lock().unwrap();
    let bytes = values.values_mut().next().unwrap();
    let mut payload: Value = serde_json::from_slice(bytes).unwrap();
    let binding = payload["bindings"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|b| b["local"]["device_id"] == device)
        .unwrap();
    edit(binding);
    *bytes = serde_json::to_vec(&payload).unwrap();
}

#[tokio::test(start_paused = true)]
async fn consent_scans_once_for_each_family_without_cloud_or_storage_mutation() {
    for (code, model, family) in [
        ("AC", "KFR-35G/WF2", ApplianceFamily::AirConditioner),
        (
            "E1",
            "GX1200Max \u{5c0a}\u{4eab}\u{7248}",
            ApplianceFamily::Dishwasher,
        ),
        ("CA", "BCD-600WUFIPZM(E)", ApplianceFamily::Refrigerator),
    ] {
        let (f, discovery, descriptor, id) = setup(code, model).await;
        *discovery.reply.lock().unwrap() = Some(report(vec![hint(&id, family, 3)]));
        let calls = f.calls();
        let secrets = f.secrets.values.lock().unwrap().clone();
        let cards = json!(f.store.devices().unwrap());
        let puts = f.secrets.puts.load(Ordering::Relaxed);
        let pending = preview(&f, &descriptor).await.unwrap();
        assert_eq!(pending.descriptor().device.id, descriptor.device.id);
        assert_eq!(pending.connection(), &descriptor.device.connections[0]);
        assert_eq!(pending.network().local_address(), network().local_address());
        assert_eq!(pending.network().prefix_length(), 24);
        assert_eq!(pending.remaining(), Duration::from_secs(60));
        assert!(!pending.is_expired());
        assert_eq!(format!("{pending:?}"), "PendingMeijuLanScan([REDACTED])");
        assert_eq!(discovery.calls.load(Ordering::Relaxed), 0);
        assert_eq!(f.calls(), calls);
        let outcome = f
            .session
            .scan_meiju_lan_candidate("scan".into(), pending, 1000)
            .await
            .unwrap();
        assert_eq!(
            format!("{outcome:?}"),
            "UniqueCandidate(DiscoveryCandidate([REDACTED]))"
        );
        let LanScanOutcome::UniqueCandidate(candidate) = outcome else {
            panic!("Expected candidate")
        };
        assert_eq!(candidate.appliance_id, id);
        assert_eq!(candidate.family, family);
        assert_eq!(candidate.endpoint.to_string(), "192.168.42.3:6444");
        assert_eq!(discovery.calls.load(Ordering::Relaxed), 1);
        assert_eq!(f.calls(), calls);
        assert_eq!(*f.secrets.values.lock().unwrap(), secrets);
        assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
        assert_eq!(f.secrets.deletes.load(Ordering::Relaxed), 0);
        assert_eq!(json!(f.store.devices().unwrap()), cards);
        assert!(!f.session.cancel("scan"));
        drop(preview(&f, &descriptor).await.unwrap());
        assert_eq!(discovery.calls.load(Ordering::Relaxed), 1);
    }
}

#[tokio::test(start_paused = true)]
async fn partial_conflicting_and_invalid_reports_never_choose_an_endpoint() {
    for mode in [
        "empty",
        "limited",
        "conflict",
        "family",
        "off_network",
        "oversized",
        "invalid_count",
    ] {
        let (f, discovery, descriptor, id) = setup("AC", "KFR-35G/WF2").await;
        let mut reply = report(vec![hint(&id, ApplianceFamily::AirConditioner, 3)]);
        match mode {
            "empty" => reply = report(vec![]),
            "limited" => reply.limit_reached = true,
            "conflict" => {
                reply
                    .candidates
                    .push(hint(&id, ApplianceFamily::AirConditioner, 4));
                reply.received_datagrams = 2;
            }
            "family" => reply.candidates[0].family = ApplianceFamily::Refrigerator,
            "off_network" => reply.candidates[0].endpoint = "192.168.43.3:6444".parse().unwrap(),
            "oversized" => reply.received_datagrams = 257,
            _ => reply.received_datagrams = 0,
        }
        *discovery.reply.lock().unwrap() = Some(reply);
        let pending = preview(&f, &descriptor).await.unwrap();
        let result = f
            .session
            .scan_meiju_lan_candidate("scan".into(), pending, 1000)
            .await;
        match mode {
            "empty" => assert!(matches!(result.unwrap(), LanScanOutcome::NotObserved)),
            "limited" => assert!(matches!(result.unwrap(), LanScanOutcome::Incomplete)),
            "conflict" | "family" => assert!(matches!(result.unwrap(), LanScanOutcome::Ambiguous)),
            _ => assert_eq!(result.unwrap_err().code, "INVALID_ARGUMENT"),
        }
        assert_eq!(discovery.calls.load(Ordering::Relaxed), 1);
    }
}

#[tokio::test(start_paused = true)]
async fn changed_foreign_and_expired_previews_stop_before_discovery() {
    for mode in [
        "foreign",
        "expired",
        "background",
        "close",
        "alias",
        "model",
        "membership",
        "removed",
    ] {
        let (f, discovery, descriptor, _) = setup("AC", "KFR-35G/WF2").await;
        let pending = preview(&f, &descriptor).await.unwrap();
        let mut target = f.session.clone();
        match mode {
            "foreign" => target = session(&f, discovery.clone()),
            "expired" => {
                tokio::time::advance(Duration::from_secs(60)).await;
                assert!(pending.is_expired());
                assert_eq!(pending.remaining(), Duration::ZERO);
            }
            "background" => {
                target.set_foreground(false);
                target.set_foreground(true);
            }
            "close" => target.close(),
            "alias" => {
                f.call(
                    json!({"method":"set_device_preferences", "device_id":descriptor.device.id,
                "preferences":{"alias":"Changed alias","room_id":null,"favorite":false}}),
                )
                .await;
            }
            "model" => {
                let mut changed = descriptor.clone();
                changed.device.model = "Changed model".into();
                f.store.save_device(&changed).unwrap();
            }
            "membership" => edit_binding(&f, &descriptor.device.id, |b| b["listed"] = json!(false)),
            _ => {
                f.store
                    .begin_account_removal(
                        f.store.accounts().unwrap()[0].id,
                        &f.store
                            .devices()
                            .unwrap()
                            .into_iter()
                            .map(|d| d.device.id)
                            .collect::<Vec<_>>(),
                    )
                    .unwrap();
            }
        }
        let error = target
            .scan_meiju_lan_candidate("scan".into(), pending, 1000)
            .await
            .unwrap_err();
        assert_eq!(
            error.code,
            match mode {
                "expired" => "LAN_SCAN_EXPIRED",
                "close" => "HOST_CLOSED",
                "membership" => "MIDEA_DEVICE_NOT_LISTED",
                "removed" => "ACCOUNT_REMOVAL_PENDING",
                _ => "LAN_SCAN_INVALID",
            },
            "{mode}"
        );
        assert_eq!(discovery.calls.load(Ordering::Relaxed), 0);
        assert!(!target.cancel("scan"));
    }
}

#[tokio::test(start_paused = true)]
async fn invalid_selection_and_protected_binding_fail_without_network() {
    for mode in [
        "alias",
        "connection",
        "identity",
        "model",
        "membership",
        "family",
        "wide_id",
        "secret",
        "pending",
    ] {
        let (f, discovery, mut descriptor, _) = setup("AC", "KFR-35G/WF2").await;
        let calls = f.calls();
        let expected = match mode {
            "alias" => {
                descriptor.device.id = descriptor.device.alias.clone();
                "INVALID_ARGUMENT"
            }
            "connection" => {
                descriptor.device.connections[0].id = Uuid::new_v4().to_string();
                "LAN_SCAN_INVALID"
            }
            "identity" => {
                edit_binding(&f, &descriptor.device.id, |b| {
                    b["local"]["identity_ref"] = json!(Uuid::new_v4())
                });
                "DEVICE_IDENTITY_CONFLICT"
            }
            "model" => {
                edit_binding(&f, &descriptor.device.id, |b| {
                    b["model"] = json!("Changed model")
                });
                "DEVICE_IDENTITY_CONFLICT"
            }
            "membership" => {
                edit_binding(&f, &descriptor.device.id, |b| b["listed"] = json!(false));
                "MIDEA_DEVICE_NOT_LISTED"
            }
            "family" => {
                edit_binding(&f, &descriptor.device.id, |b| b["family"] = json!(0xff));
                "CAPABILITY_UNSUPPORTED"
            }
            "wide_id" => {
                edit_binding(&f, &descriptor.device.id, |b| {
                    b["appliance_id"] = json!("880000000000099")
                });
                "CAPABILITY_UNSUPPORTED"
            }
            "secret" => {
                f.secrets.unavailable.store(true, Ordering::Relaxed);
                "SECRET_STORE_UNAVAILABLE"
            }
            _ => {
                f.store
                    .set_credential_state(
                        f.store.accounts().unwrap()[0].id,
                        CredentialState::Pending,
                    )
                    .unwrap();
                "ACCOUNT_SESSION_UNCONFIRMED"
            }
        };
        assert_eq!(
            preview(&f, &descriptor).await.unwrap_err().code,
            expected,
            "{mode}"
        );
        assert_eq!(discovery.calls.load(Ordering::Relaxed), 0);
        assert_eq!(f.calls(), calls);
        assert!(!f.session.cancel("preview"));
    }
}

#[tokio::test(start_paused = true)]
async fn confirmation_retains_lease_and_rechecks_metadata_and_protected_membership() {
    for mode in ["complete", "alias", "membership", "identity"] {
        let (f, discovery, descriptor, id) = setup("AC", "KFR-35G/WF2").await;
        let pending = preview(&f, &descriptor).await.unwrap();
        *discovery.reply.lock().unwrap() =
            Some(report(vec![hint(&id, ApplianceFamily::AirConditioner, 3)]));
        discovery.stall.store(true, Ordering::Relaxed);
        let scan = f
            .session
            .scan_meiju_lan_candidate("scan".into(), pending, 1000);
        tokio::pin!(scan);
        tokio::select! {
            _ = discovery.started.notified() => (),
            result = &mut scan => panic!("Expected pending discovery: {result:?}"),
        }
        let account_id = f.store.accounts().unwrap()[0].id;
        assert_eq!(
            f.vault
                .lease(account_id, &operation())
                .await
                .err()
                .unwrap()
                .code,
            "RESOURCE_BUSY"
        );
        assert_eq!(
            f.call_id("scan", json!({"method":"list_accounts"})).await["error"]["code"],
            "RESOURCE_BUSY"
        );
        match mode {
            "alias" => {
                f.call(
                    json!({"method":"set_device_preferences","device_id":descriptor.device.id,
                "preferences":{"alias":"New alias","room_id":null,"favorite":false}}),
                )
                .await;
            }
            "membership" => edit_binding(&f, &descriptor.device.id, |b| b["listed"] = json!(false)),
            "identity" => edit_binding(&f, &descriptor.device.id, |b| {
                b["local"]["identity_ref"] = json!(Uuid::new_v4())
            }),
            _ => (),
        }
        discovery.resume.notify_one();
        let result = scan.await;
        match mode {
            "complete" => assert!(matches!(
                result.unwrap(),
                LanScanOutcome::UniqueCandidate(_)
            )),
            "alias" => assert_eq!(result.unwrap_err().code, "LAN_SCAN_INVALID"),
            "membership" => assert_eq!(result.unwrap_err().code, "MIDEA_DEVICE_NOT_LISTED"),
            _ => assert_eq!(result.unwrap_err().code, "DEVICE_IDENTITY_CONFLICT"),
        }
        drop(f.vault.lease(account_id, &operation()).await.unwrap());
        assert!(!f.session.cancel("scan"));
        assert_eq!(discovery.calls.load(Ordering::Relaxed), 1);
    }
}

#[tokio::test(start_paused = true)]
async fn preparation_and_scan_release_operations_and_leases_on_every_interruption() {
    for preparing in [true, false] {
        for stop in ["cancel", "background", "close", "deadline", "drop"] {
            let (f, discovery, descriptor, _) = setup("AC", "KFR-35G/WF2").await;
            let pending = preview(&f, &descriptor).await.unwrap();
            let calls = f.calls();
            f.secrets.read_delay.store(preparing, Ordering::Relaxed);
            discovery.stall.store(!preparing, Ordering::Relaxed);
            let operation_id = if preparing { "preview" } else { "scan" };
            {
                let work = async {
                    if preparing {
                        preview(&f, &descriptor).await.map(|_| ())
                    } else {
                        f.session
                            .scan_meiju_lan_candidate("scan".into(), pending, 1000)
                            .await
                            .map(|_| ())
                    }
                };
                tokio::pin!(work);
                tokio::select! {
                    _ = async {
                        if preparing { f.secrets.read_started.notified().await; }
                        else { discovery.started.notified().await; }
                    } => (),
                    result = &mut work => panic!("Expected pending operation: {result:?}"),
                }
                match stop {
                    "cancel" => assert!(f.session.cancel(operation_id)),
                    "background" => f.session.set_foreground(false),
                    "close" => f.session.close(),
                    "deadline" => tokio::time::advance(Duration::from_secs(2)).await,
                    _ => (),
                }
                if stop != "drop" {
                    assert_eq!(
                        work.await.unwrap_err().code,
                        if stop == "deadline" {
                            "DEADLINE_EXCEEDED"
                        } else {
                            "CANCELLED"
                        }
                    );
                }
            }
            f.secrets.read_delay.store(false, Ordering::Relaxed);
            drop(
                f.vault
                    .lease(f.store.accounts().unwrap()[0].id, &operation())
                    .await
                    .unwrap(),
            );
            assert!(!f.session.cancel(operation_id));
            assert_eq!(
                discovery.calls.load(Ordering::Relaxed),
                usize::from(!preparing)
            );
            assert_eq!(f.calls(), calls);
        }
    }
}

#[tokio::test(start_paused = true)]
async fn remaining_preview_lifetime_caps_scan_and_json_cannot_confirm() {
    let (f, discovery, descriptor, _) = setup("AC", "KFR-35G/WF2").await;
    let pending = preview(&f, &descriptor).await.unwrap();
    tokio::time::advance(Duration::from_secs(59)).await;
    discovery.stall.store(true, Ordering::Relaxed);
    let scan = f
        .session
        .scan_meiju_lan_candidate("scan".into(), pending, 30_000);
    tokio::pin!(scan);
    tokio::select! {
        _ = discovery.started.notified() => (),
        result = &mut scan => panic!("Expected pending discovery: {result:?}"),
    }
    tokio::time::advance(Duration::from_secs(1)).await;
    assert_eq!(scan.await.unwrap_err().code, "DEADLINE_EXCEEDED");
    for method in ["prepare_meiju_lan_scan", "scan_meiju_lan_candidate"] {
        assert!(
            serde_json::from_value::<HostRequest>(json!({
                "schema_version":"1.0","operation_id":"scan","timeout_ms":1000,
                "request":{"method":method,"device_id":descriptor.device.id,"confirmed":true}
            }))
            .is_err()
        );
    }
    let no_accounts = Session::new(Core::new(
        f.store.clone(),
        vec![],
        Arc::new(SystemClock),
        Policy::default(),
    ));
    assert_eq!(
        no_accounts
            .prepare_meiju_lan_scan(
                "preview".into(),
                &descriptor.device.id,
                &descriptor.device.connections[0].id,
                network(),
                1000
            )
            .await
            .unwrap_err()
            .code,
        "AUTH_FLOW_UNSUPPORTED"
    );
}

#[tokio::test(start_paused = true)]
async fn discovery_failure_never_retries_or_falls_back_to_cloud() {
    let (f, discovery, descriptor, _) = setup("AC", "KFR-35G/WF2").await;
    let calls = f.calls();
    let pending = preview(&f, &descriptor).await.unwrap();
    discovery.fail.store(true, Ordering::Relaxed);
    assert_eq!(
        f.session
            .scan_meiju_lan_candidate("scan".into(), pending, 1000)
            .await
            .unwrap_err()
            .code,
        "MIDEA_REQUEST_FAILED"
    );
    assert_eq!(discovery.calls.load(Ordering::Relaxed), 1);
    assert_eq!(f.calls(), calls);
    drop(
        f.vault
            .lease(f.store.accounts().unwrap()[0].id, &operation())
            .await
            .unwrap(),
    );
    assert!(!f.session.cancel("scan"));
}
