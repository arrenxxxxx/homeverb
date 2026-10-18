use super::*;
use homeverb_host::ConfirmIdentityRecoveryRequest;

async fn prepare(f: &Fixture, id: Uuid) -> Value {
    f.session
        .prepare_native_identity_recovery(
            ConnectRequest {
                schema_version: "1.0".into(),
                operation_id: "prepare-recovery".into(),
                timeout_ms: 1000,
                label: "Kept account".into(),
                account_id: Some(id),
            },
            secret("owner@example.test"),
            secret("Synthetic password 42!"),
        )
        .await
}
async fn confirm(f: &Fixture, id: u64) -> Value {
    f.session
        .confirm_native_identity_recovery(ConfirmIdentityRecoveryRequest {
            schema_version: "1.0".into(),
            operation_id: "confirm-recovery".into(),
            timeout_ms: 1000,
            recovery_id: id,
        })
        .await
}
fn change_login(f: &Fixture) {
    *f.wire.subject.lock().unwrap() = "PRIVATE-ROTATED-LOGIN-ID".into();
    *f.wire.uid.lock().unwrap() = Some("PRIVATE-IOT-USER-ID".into());
}
fn saved(f: &Fixture) -> HashMap<String, Vec<u8>> {
    f.secrets.values.lock().unwrap().clone()
}

#[tokio::test(start_paused = true)]
async fn canonical_subject_reauthorizes_rotated_login_ids_and_refuses_other_uid() {
    let f = Fixture::new();
    *f.wire.uid.lock().unwrap() = Some("PRIVATE-IOT-USER-ID".into());
    assert_eq!(f.connect(None).await["data"]["authentication"], "succeeded");
    let id = f.store.accounts().unwrap()[0].id;
    let devices = f.store.devices().unwrap();
    change_login(&f);
    assert_eq!(
        f.connect(Some(id)).await["data"]["authentication"],
        "succeeded"
    );
    assert_eq!(
        serde_json::to_value(f.store.devices().unwrap()).unwrap(),
        serde_json::to_value(&devices).unwrap()
    );
    let before = saved(&f);
    *f.wire.uid.lock().unwrap() = Some("PRIVATE-OTHER-USER".into());
    assert_eq!(
        f.connect(Some(id)).await["error"]["code"],
        "ACCOUNT_IDENTITY_CONFLICT"
    );
    assert_eq!(saved(&f), before);
    *f.wire.uid.lock().unwrap() = None;
    assert_eq!(
        f.connect(Some(id)).await["error"]["code"],
        "ACCOUNT_IDENTITY_CONFLICT"
    );
    assert_eq!(saved(&f), before);
}

#[tokio::test(start_paused = true)]
async fn matching_legacy_alias_migrates_without_reconciliation() {
    let f = Fixture::new();
    f.connect(None).await;
    let id = f.store.accounts().unwrap()[0].id;
    *f.wire.uid.lock().unwrap() = Some("PRIVATE-IOT-USER-ID".into());
    assert_eq!(
        f.connect(Some(id)).await["data"]["authentication"],
        "succeeded"
    );
    change_login(&f);
    assert_eq!(
        f.connect(Some(id)).await["data"]["authentication"],
        "succeeded"
    );
    assert_eq!(f.store.accounts().unwrap().len(), 1);
}

#[tokio::test(start_paused = true)]
async fn explicit_recovery_keeps_bindings_and_requires_consent_then_seals_host() {
    let mut f = Fixture::new();
    f.connect(None).await;
    let id = f.store.accounts().unwrap()[0].id;
    let devices = f.store.devices().unwrap();
    let preferences = f.store.preferences().unwrap();
    let before = saved(&f);
    change_login(&f);
    assert_eq!(
        f.connect(Some(id)).await["error"]["code"],
        "ACCOUNT_IDENTITY_CONFLICT"
    );
    let preview = prepare(&f, id).await;
    assert!(preview["error"].is_null(), "{preview}");
    assert_eq!(preview["data"]["device_count"], devices.len());
    assert!(!preview.to_string().contains("PRIVATE"));
    assert_eq!(saved(&f), before);
    let recovery_id = preview["data"]["recovery_id"].as_u64().unwrap();
    assert_eq!(
        confirm(&f, recovery_id).await["data"]["authentication"],
        "succeeded"
    );
    assert_eq!(
        serde_json::to_value(f.store.devices().unwrap()).unwrap(),
        serde_json::to_value(&devices).unwrap()
    );
    assert_eq!(f.store.preferences().unwrap(), preferences);
    assert_eq!(
        f.call(json!({"method":"list_accounts"})).await["error"]["code"],
        "HOST_CLOSED"
    );
    assert!(confirm(&f, recovery_id).await["error"].is_object());
    f.reopen_with_power_grants(&[]);
    *f.wire.subject.lock().unwrap() = "PRIVATE-ANOTHER-LOGIN-ID".into();
    assert_eq!(
        f.connect(Some(id)).await["data"]["authentication"],
        "succeeded"
    );
    assert!(!f.calls().contains(&CloudEndpoint::ApplianceControl));
    assert_eq!(f.secrets.deletes.load(Ordering::Relaxed), 0);
}

#[tokio::test(start_paused = true)]
async fn recovery_lifecycle_expiry_and_byte_changes_cannot_replace_the_binding() {
    for action in [
        "discard",
        "background",
        "cancel",
        "close",
        "expire",
        "bytes",
    ] {
        let f = Fixture::new();
        f.connect(None).await;
        let account = f.store.accounts().unwrap()[0].clone();
        change_login(&f);
        let preview = prepare(&f, account.id).await;
        let id = preview["data"]["recovery_id"].as_u64().expect("preview");
        match action {
            "discard" => {
                assert!(f.session.discard_native_identity_recovery(id));
            }
            "background" => {
                f.session.set_foreground(false);
                f.session.set_foreground(true);
            }
            "cancel" => {
                f.session.cancel("unrelated-operation");
            }
            "close" => f.session.close(),
            "expire" => tokio::time::advance(std::time::Duration::from_secs(61)).await,
            "bytes" => {
                let mut values = f.secrets.values.lock().unwrap();
                let bytes = values.get_mut(&account.credential_ref.to_string()).unwrap();
                bytes.push(b' ');
            }
            _ => unreachable!(),
        }
        let before = saved(&f);
        let puts = f.secrets.puts.load(Ordering::Relaxed);
        assert!(confirm(&f, id).await["error"].is_object(), "{action}");
        assert_eq!(saved(&f), before, "{action}");
        assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
    }
}

#[tokio::test(start_paused = true)]
async fn recovery_refuses_incomplete_changed_or_missing_inventory_and_uid() {
    for change in ["extra", "empty", "partial", "missing_uid"] {
        let f = Fixture::new();
        f.connect(None).await;
        let id = f.store.accounts().unwrap()[0].id;
        let before = saved(&f);
        change_login(&f);
        match change {
            "extra" => f.wire.extra_device.store(true, Ordering::Relaxed),
            "empty" => f.wire.empty_inventory.store(true, Ordering::Relaxed),
            "partial" => f.wire.inventory_code.store(7400, Ordering::Relaxed),
            "missing_uid" => *f.wire.uid.lock().unwrap() = None,
            _ => unreachable!(),
        }
        assert!(prepare(&f, id).await["error"].is_object(), "{change}");
        assert_eq!(saved(&f), before);
    }
}

#[tokio::test(start_paused = true)]
async fn uncertain_recovery_stays_pending_and_never_replays() {
    let f = Fixture::new();
    f.connect(None).await;
    let account = f.store.accounts().unwrap()[0].clone();
    change_login(&f);
    let preview = prepare(&f, account.id).await;
    let id = preview["data"]["recovery_id"].as_u64().unwrap();
    f.secrets.uncertain.store(true, Ordering::Relaxed);
    assert_eq!(
        confirm(&f, id).await["error"]["code"],
        "SECRET_STORE_WRITE_UNKNOWN"
    );
    assert_eq!(
        f.store.accounts().unwrap()[0].credential_state,
        CredentialState::Pending
    );
    let puts = f.secrets.puts.load(Ordering::Relaxed);
    assert!(confirm(&f, id).await["error"].is_object());
    assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
}

#[tokio::test(start_paused = true)]
async fn recovery_refuses_different_login_input_and_existing_canonical_account() {
    let f = Fixture::new();
    f.connect(None).await;
    let id = f.store.accounts().unwrap()[0].id;
    change_login(&f);
    let before = saved(&f);
    let result = f
        .session
        .prepare_native_identity_recovery(
            ConnectRequest {
                schema_version: "1.0".into(),
                operation_id: "other-input".into(),
                timeout_ms: 1000,
                label: "Account".into(),
                account_id: Some(id),
            },
            secret("other@example.test"),
            secret("Synthetic password 42!"),
        )
        .await;
    assert_eq!(result["error"]["code"], "ACCOUNT_IDENTITY_CONFLICT");
    assert_eq!(saved(&f), before);
    f.connect(None).await;
    assert_eq!(f.store.accounts().unwrap().len(), 2);
    let before_alias = saved(&f);
    *f.wire.subject.lock().unwrap() = "synthetic-login-001".into();
    assert_eq!(
        f.connect(Some(id)).await["error"]["code"],
        "ACCOUNT_IDENTITY_CONFLICT"
    );
    assert_eq!(saved(&f), before_alias);
    change_login(&f);
    let preview = prepare(&f, id).await;
    let recovery = preview["data"]["recovery_id"].as_u64().unwrap();
    let before = saved(&f);
    assert_eq!(
        confirm(&f, recovery).await["error"]["code"],
        "ACCOUNT_IDENTITY_CONFLICT"
    );
    assert_eq!(saved(&f), before);
}
