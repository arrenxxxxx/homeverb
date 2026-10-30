//! Actual Host/Core publication with fictional encrypted directory/property replies.
use super::*;

async fn connect(f: &ModernFixture) -> Value {
    let qr = f.begin().await;
    serde_json::to_value(
        f.session
            .continue_modern_device_login(answer_metadata(qr.challenge_id))
            .await
            .unwrap(),
    )
    .unwrap()
}
fn id(report: &Value) -> Uuid {
    Uuid::parse_str(report["account"]["id"].as_str().unwrap()).unwrap()
}
async fn refresh(f: &ModernFixture, account: Uuid, recover: bool) -> Value {
    let result = if recover {
        f.session
            .recover_modern_account_devices(account_request(account))
            .await
    } else {
        f.session
            .refresh_modern_account_devices(account_request(account))
            .await
    };
    serde_json::to_value(result.unwrap()).unwrap()
}
async fn cards(f: &ModernFixture) -> Vec<Value> {
    f.call(json!({"method":"list_devices"})).await["data"]
        .as_array()
        .unwrap()
        .clone()
}
fn validate_report(report: &Value, method: &str) {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../schemas");
    let mut options = jsonschema::options();
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        let Some(name) = path
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| n.strip_suffix(".schema.json"))
        else {
            continue;
        };
        options = options.with_resource(
            format!("urn:homeverb:schema:{name}"),
            jsonschema::Resource::from_contents(
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap(),
            ),
        );
    }
    options
        .build(&json!({"$ref":format!("urn:homeverb:schema:native-response#/$defs/{method}")}))
        .unwrap()
        .validate(&json!({"schema_version":"1.0","data":report,"warnings":[],"error":null}))
        .unwrap();
    let (definition, kind) = if method == "connect_account" {
        ("continue_result", "connected_devices")
    } else {
        ("session_result", "devices")
    };
    validate_modern_result(
        definition,
        &json!({"schema_version":"1.0","data":{"kind":kind,"report":report},"warnings":[],"error":null}),
    );
}

#[tokio::test]
async fn qr_publishes_grouped_cards_and_core_reads_with_one_directory_and_no_legacy_traffic() {
    let f = ModernFixture::new();
    *f.peer.directory.lock().unwrap() = Some(json!({"list":[
        {"did":"PRIVATE-MODERN-DID","name":"Modern Fan","model":"jdjz.fan.2072fc","pid":0},
        {"did":"PRIVATE-CHILD","parent_id":"PRIVATE-MODERN-DID","name":"Logical Fan","model":"jdjz.fan.2072fc","pid":21},
        {"did":"PRIVATE-UNKNOWN","name":"Unknown Appliance","model":"fiction.unknown.v1","pid":0},
        {"did":"PRIVATE-ORPHAN","name":"Unresolved Child","model":"fiction.unknown.v1","pid":21}
    ],"has_more":false}));
    let report = connect(&f).await;
    validate_report(&report, "connect_account");
    assert_eq!(report["authentication"], "succeeded");
    assert_eq!(report["retrieval"]["status"], "partial");
    assert_eq!(
        report["retrieval"]["device_ids"].as_array().unwrap().len(),
        2
    );
    assert!(report["account"]["last_complete_inventory_at"].is_null());
    assert_eq!(f.peer.reads.load(Ordering::SeqCst), 1);
    assert_eq!(f.peer.auth_count(), 4);
    let cards = cards(&f).await;
    assert_eq!(cards.len(), 2);
    let fan = cards.iter().find(|d| d["alias"] == "Modern Fan").unwrap();
    let unknown = cards
        .iter()
        .find(|d| d["alias"] == "Unknown Appliance")
        .unwrap();
    assert_eq!(unknown["support_status"], "unsupported");
    let state = f.call(json!({"method":"read_state","device":fan["id"],"component":"fan-2","connection":"cloud","cached":false})).await;
    assert!(state["error"].is_null(), "{state}");
    let power = &state["data"]["observations"]["power.state"];
    assert_eq!(power["value"], false);
    assert_eq!(power["stale"], true);
    assert!(power["observed_at"].is_null());
    assert!(power["received_at"].is_string());
    assert_eq!(f.peer.properties.load(Ordering::SeqCst), 1);
    assert_eq!(f.peer.reads.load(Ordering::SeqCst), 1);
    assert_eq!(f.cloud.calls.load(Ordering::SeqCst), 0);
    let descriptor = f
        .call(json!({"method":"describe_device","device":fan["id"]}))
        .await;
    assert!(
        descriptor["data"]["capabilities"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["actions"].as_object().unwrap().is_empty())
    );
    let public = format!("{report}{state}{descriptor}{}", json!(cards));
    for marker in ["PRIVATE-", "999001", "credential_ref", "ssecurity"] {
        assert!(!public.contains(marker));
        // Credential references are permitted local metadata, unlike secret values.
        if marker == "credential_ref" {
            continue;
        }
        for file in std::fs::read_dir(&f.directory).unwrap() {
            let path = file.unwrap().path();
            if path.is_file() {
                assert!(
                    !std::fs::read(path)
                        .unwrap()
                        .windows(marker.len())
                        .any(|v| v == marker.as_bytes())
                );
            }
        }
    }
}

#[tokio::test]
async fn explicit_refresh_reconnect_and_partial_empty_directory_preserve_local_identity() {
    let f = ModernFixture::new();
    let first = connect(&f).await;
    let device = first["retrieval"]["device_ids"][0].clone();
    let room = f.call(json!({"method":"create_room","name":"Study"})).await;
    let preference = f
        .call(json!({"method":"set_device_preferences","device_id":device,
        "preferences":{"alias":"Study Fan","room_id":room["data"]["id"],"favorite":true}}))
        .await;
    assert!(preference["error"].is_null(), "{preference}");
    let before = cards(&f).await;
    let organized = f.call(json!({"method":"list_device_cards"})).await;
    let puts = f.secrets.puts.load(Ordering::SeqCst);
    let next = refresh(&f, id(&first), false).await;
    validate_report(&next, "refresh_account");
    assert_eq!(next["authentication"], "not_attempted");
    assert_eq!(first["retrieval"], next["retrieval"]);
    assert_eq!(f.peer.auth_count(), 4);
    assert_eq!(f.secrets.puts.load(Ordering::SeqCst), puts);
    let reconnected = connect(&f).await;
    assert_eq!(first, reconnected);
    assert_eq!(cards(&f).await, before);
    assert_eq!(
        f.call(json!({"method":"list_device_cards"})).await,
        organized
    );
    // Reauthentication can rotate protected client material; unchanged refresh cannot.
    let puts = f.secrets.puts.load(Ordering::SeqCst);
    *f.peer.directory.lock().unwrap() = Some(json!({"list":[],"has_more":false}));
    let empty = refresh(&f, id(&first), false).await;
    validate_report(&empty, "refresh_account");
    assert_eq!(empty["retrieval"]["status"], "partial");
    assert_eq!(empty["retrieval"]["device_ids"], json!([]));
    assert_eq!(cards(&f).await, before);
    assert_eq!(f.secrets.puts.load(Ordering::SeqCst), puts);
}

#[tokio::test]
async fn old_logical_login_stays_card_free_until_explicit_device_refresh() {
    let f = ModernFixture::new();
    let old = f.connect().await;
    assert!(cards(&f).await.is_empty());
    let auth = f.peer.auth_count();
    let report = refresh(&f, old.account.id, false).await;
    validate_report(&report, "refresh_account");
    assert_eq!(cards(&f).await.len(), 1);
    assert_eq!(f.peer.auth_count(), auth);
    assert_eq!(f.peer.reads.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn directory_failure_preserves_successful_authentication_and_explicit_retry_never_logs_in() {
    let f = ModernFixture::new();
    f.peer.fail_read.store(true, Ordering::SeqCst);
    let report = connect(&f).await;
    validate_report(&report, "connect_account");
    assert_eq!(report["authentication"], "succeeded");
    assert_eq!(report["account"]["credential_state"], "stored");
    assert_eq!(report["retrieval"]["status"], "failed");
    assert_eq!(report["retrieval"]["device_ids"], json!([]));
    assert!(cards(&f).await.is_empty());
    assert_eq!(f.peer.reads.load(Ordering::SeqCst), 1);
    f.peer.fail_read.store(false, Ordering::SeqCst);
    let recovered = refresh(&f, id(&report), false).await;
    assert_eq!(recovered["retrieval"]["status"], "partial");
    assert_eq!(f.peer.auth_count(), 4);
    assert_eq!(cards(&f).await.len(), 1);
}

#[tokio::test]
async fn uncertain_binding_save_publishes_nothing_and_recovery_confirms_without_rewrite() {
    let f = ModernFixture::new();
    f.secrets.uncertain_at.store(2, Ordering::SeqCst);
    let report = connect(&f).await;
    validate_report(&report, "connect_account");
    assert_eq!(report["authentication"], "succeeded");
    assert_eq!(report["account"]["credential_state"], "pending");
    assert_eq!(report["retrieval"]["status"], "failed");
    assert_eq!(
        report["retrieval"]["error"]["code"],
        "SECRET_STORE_WRITE_UNKNOWN"
    );
    assert_eq!(report["retrieval"]["device_ids"], json!([]));
    assert!(cards(&f).await.is_empty());
    assert!(
        f.session
            .refresh_modern_account_devices(account_request(id(&report)))
            .await
            .is_err()
    );
    let recovered = refresh(&f, id(&report), true).await;
    validate_report(&recovered, "refresh_account");
    assert_eq!(recovered["account"]["credential_state"], "stored");
    assert_eq!(cards(&f).await.len(), 1);
    assert_eq!(f.secrets.puts.load(Ordering::SeqCst), 2);
    assert_eq!(f.peer.auth_count(), 4);
}

#[tokio::test]
async fn device_continuation_consumes_handles_before_metadata_admission_and_rejects_foreign_handles()
 {
    let f = ModernFixture::new();
    let foreign = ModernFixture::new();
    let qr = f.begin().await;
    assert!(
        foreign
            .session
            .continue_modern_device_login(answer_metadata(qr.challenge_id))
            .await
            .is_err()
    );
    let mut input = answer_metadata(qr.challenge_id);
    input.timeout_ms = 0;
    assert!(f.session.continue_modern_device_login(input).await.is_err());
    assert!(
        f.session
            .continue_modern_device_login(answer_metadata(qr.challenge_id))
            .await
            .is_err()
    );
    assert!(!f.session.discard_modern_account_login(qr.challenge_id));
    assert_eq!(f.peer.auth_count(), 2);
    assert_eq!(foreign.peer.auth_count(), 0);
}

#[tokio::test]
async fn original_qr_deadline_bounds_directory_and_publication() {
    let f = ModernFixture::new();
    let mut input = begin_metadata();
    input.timeout_ms = 100;
    let qr = f.session.begin_modern_account_login(input).await.unwrap();
    f.peer.pause.store(3, Ordering::SeqCst);
    let started = tokio::time::Instant::now();
    let error = f
        .session
        .continue_modern_device_login(answer_metadata(qr.challenge_id))
        .await
        .err()
        .unwrap();
    assert_eq!(error.code, "DEADLINE_EXCEEDED");
    assert!(started.elapsed() < std::time::Duration::from_secs(2));
    assert!(cards(&f).await.is_empty());
    let account = f.vault.list(&op()).await.unwrap().remove(0);
    assert_eq!(account.credential_state, CredentialState::Stored);
    drop(f.vault.lease(account.id, &op()).await.unwrap());
}

#[tokio::test]
async fn cancellation_background_close_and_drop_release_publication_work_and_leases() {
    for mode in 0..4 {
        let f = ModernFixture::new();
        let qr = f.begin().await;
        f.peer.pause.store(3, Ordering::SeqCst);
        {
            let work = f
                .session
                .continue_modern_device_login(answer_metadata(qr.challenge_id));
            tokio::pin!(work);
            tokio::select! { _ = f.peer.started.notified() => {}, _ = &mut work => panic!("Directory must wait") }
            match mode {
                0 => {
                    assert!(f.session.cancel("answer-captcha"));
                }
                1 => f.session.set_foreground(false),
                2 => f.session.close(),
                _ => {}
            }
            if mode != 3 {
                assert_eq!(work.await.err().unwrap().code, "CANCELLED");
            }
        }
        let account = f.vault.list(&op()).await.unwrap().remove(0);
        drop(f.vault.lease(account.id, &op()).await.unwrap());
        assert_eq!(f.secrets.puts.load(Ordering::SeqCst), 1);
        assert_eq!(f.peer.reads.load(Ordering::SeqCst), 1);
        if mode != 2 {
            f.session.set_foreground(true);
            assert!(cards(&f).await.is_empty());
            f.peer.pause.store(0, Ordering::SeqCst);
            assert_eq!(
                refresh(&f, account.id, false).await["retrieval"]["status"],
                "partial"
            );
        }
    }
}

#[tokio::test]
async fn late_lifecycle_generation_refuses_publication_without_claiming_storage_rollback() {
    let f = ModernFixture::new();
    let qr = f.begin().await;
    let session = Arc::downgrade(&f.session);
    *f.peer.on_reply.lock().unwrap() = Some(Box::new(move || {
        assert!(!session.upgrade().unwrap().cancel("not-active"));
    }));
    let error = f
        .session
        .continue_modern_device_login(answer_metadata(qr.challenge_id))
        .await
        .err()
        .unwrap();
    assert_eq!(error.code, "MIJIA_AUTH_CHALLENGE_REQUIRED");
    assert!(cards(&f).await.is_empty());
    let account = f.vault.list(&op()).await.unwrap().remove(0);
    assert_eq!(account.credential_state, CredentialState::Stored);
    let report = refresh(&f, account.id, false).await;
    assert_eq!(report["retrieval"]["status"], "partial");
    assert_eq!(f.peer.auth_count(), 4);
}

#[tokio::test]
async fn modern_refresh_cancellation_and_drop_release_operation_id_for_explicit_retry() {
    for recover in [false, true] {
        for cancel in [false, true] {
            let f = ModernFixture::new();
            let report = connect(&f).await;
            f.peer.pause.store(3, Ordering::SeqCst);
            {
                let work = async {
                    if recover {
                        f.session
                            .recover_modern_account_devices(account_request(id(&report)))
                            .await
                    } else {
                        f.session
                            .refresh_modern_account_devices(account_request(id(&report)))
                            .await
                    }
                };
                tokio::pin!(work);
                tokio::select! { _ = f.peer.started.notified() => {}, _ = &mut work => panic!("Directory must wait") }
                if cancel {
                    assert!(f.session.cancel("modern-account"));
                    assert_eq!(work.await.err().unwrap().code, "CANCELLED");
                }
            }
            drop(f.vault.lease(id(&report), &op()).await.unwrap());
            f.peer.pause.store(0, Ordering::SeqCst);
            assert_eq!(
                refresh(&f, id(&report), recover).await["retrieval"]["status"],
                "partial"
            );
            assert_eq!(f.peer.auth_count(), 4);
        }
    }
}

#[tokio::test]
async fn failed_core_identity_publication_keeps_existing_card_and_returns_no_result_ids() {
    let f = ModernFixture::new();
    let first = connect(&f).await;
    let store = SqliteStore::open(&f.directory).unwrap();
    let original = store.devices().unwrap().remove(0);
    let mut changed = original.clone();
    changed.device.connections[0].identity_ref = Uuid::new_v4().to_string();
    assert_eq!(
        store.save_device(&changed).unwrap_err().code,
        "DEVICE_IDENTITY_CONFLICT"
    );
    // Inject corrupted persisted metadata below the normal store identity guard.
    let database = rusqlite::Connection::open(f.directory.join("homeverb.sqlite3")).unwrap();
    database
        .execute(
            "UPDATE devices SET descriptor=?1 WHERE id=?2",
            rusqlite::params![serde_json::to_string(&changed).unwrap(), changed.device.id],
        )
        .unwrap();
    let report = refresh(&f, id(&first), false).await;
    validate_report(&report, "refresh_account");
    assert_eq!(report["retrieval"]["status"], "failed");
    assert_eq!(
        report["retrieval"]["error"]["code"],
        "DEVICE_IDENTITY_CONFLICT"
    );
    assert_eq!(report["retrieval"]["device_ids"], json!([]));
    assert_eq!(
        store.devices().unwrap()[0].device.connections,
        changed.device.connections
    );
    assert_eq!(f.peer.auth_count(), 4);
    database
        .execute(
            "UPDATE devices SET descriptor=?1 WHERE id=?2",
            rusqlite::params![
                serde_json::to_string(&original).unwrap(),
                original.device.id
            ],
        )
        .unwrap();
    assert_eq!(
        refresh(&f, id(&first), false).await["retrieval"]["status"],
        "partial"
    );
}

#[tokio::test]
async fn uncertain_authentication_save_requires_explicit_recovery_before_any_card() {
    let f = ModernFixture::new();
    f.secrets.uncertain_at.store(1, Ordering::SeqCst);
    let qr = f.begin().await;
    let error = f
        .session
        .continue_modern_device_login(answer_metadata(qr.challenge_id))
        .await
        .err()
        .unwrap();
    assert_eq!(error.code, "SECRET_STORE_WRITE_UNKNOWN");
    assert!(cards(&f).await.is_empty());
    assert_eq!(f.peer.reads.load(Ordering::SeqCst), 0);
    let account = f.vault.list(&op()).await.unwrap().remove(0);
    assert_eq!(account.credential_state, CredentialState::Pending);
    let report = refresh(&f, account.id, true).await;
    assert_eq!(report["account"]["credential_state"], "stored");
    assert_eq!(report["retrieval"]["status"], "partial");
    assert_eq!(f.peer.auth_count(), 4);
    assert_eq!(f.peer.reads.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn ordinary_refresh_adopts_modern_only_accounts_with_no_authentication_or_legacy_requests() {
    for publish in [false, true] {
        let f = ModernFixture::new();
        let account = if publish {
            id(&connect(&f).await)
        } else {
            f.connect().await.account.id
        };
        let result = f
            .call(json!({"method":"refresh_account","account_id":account}))
            .await;
        assert!(result["error"].is_null(), "{result}");
        validate_report(&result["data"], "refresh_account");
        assert_eq!(result["data"]["retrieval"]["status"], "partial");
        assert_eq!(cards(&f).await.len(), 1);
        assert_eq!(f.peer.reads.load(Ordering::SeqCst), 2);
        assert_eq!(f.peer.auth_count(), 4);
        assert_eq!(f.cloud.calls.load(Ordering::SeqCst), 0);
    }
}

#[tokio::test]
async fn ordinary_mixed_account_refresh_preserves_legacy_until_modern_device_adoption_and_never_falls_back()
 {
    let f = ModernFixture::new();
    let legacy = f
        .session
        .connect_account(
            begin_metadata(),
            secret(b"fiction@example.invalid"),
            secret(b"fictional-password"),
        )
        .await;
    assert!(legacy["error"].is_null(), "{legacy}");
    let original_cards = cards(&f).await;
    let modern = f.connect().await;
    let account = modern.account.id;
    assert_eq!(json!(account), legacy["data"]["account"]["id"]);
    let cloud = f.cloud.calls.load(Ordering::SeqCst);
    let result = f
        .call(json!({"method":"refresh_account","account_id":account}))
        .await;
    assert!(result["error"].is_null());
    assert_eq!(result["data"]["retrieval"]["status"], "partial");
    assert!(f.cloud.calls.load(Ordering::SeqCst) > cloud);
    assert_eq!(f.peer.reads.load(Ordering::SeqCst), 1);
    assert_eq!(cards(&f).await, original_cards);
    refresh(&f, account, false).await;
    let cloud = f.cloud.calls.load(Ordering::SeqCst);
    let result = f
        .call(json!({"method":"refresh_account","account_id":account}))
        .await;
    assert_eq!(result["data"]["retrieval"]["status"], "partial");
    assert_eq!(f.peer.reads.load(Ordering::SeqCst), 3);
    assert_eq!(f.cloud.calls.load(Ordering::SeqCst), cloud);
    let before = cards(&f).await;
    assert_eq!(before.len(), 3);
    for device in original_cards {
        assert!(before.contains(&device));
    }
    f.peer.fail_read.store(true, Ordering::SeqCst);
    let result = f
        .call(json!({"method":"refresh_account","account_id":account}))
        .await;
    assert!(result["error"].is_null());
    assert_eq!(result["data"]["retrieval"]["status"], "failed");
    assert_eq!(result["data"]["retrieval"]["device_ids"], json!([]));
    assert_eq!(f.peer.reads.load(Ordering::SeqCst), 4);
    assert_eq!(f.peer.auth_count(), 4);
    assert_eq!(f.cloud.calls.load(Ordering::SeqCst), cloud);
    assert_eq!(cards(&f).await, before);
}

#[tokio::test]
async fn ordinary_recovery_uses_pending_protected_modern_evidence_without_rewriting_or_authentication()
 {
    let f = ModernFixture::new();
    f.secrets.uncertain_at.store(2, Ordering::SeqCst);
    let first = connect(&f).await;
    assert_eq!(first["account"]["credential_state"], "pending");
    let rejected = f
        .call(json!({"method":"refresh_account","account_id":id(&first)}))
        .await;
    assert!(!rejected["error"].is_null());
    let result = f
        .call(json!({"method":"recover_account","account_id":id(&first)}))
        .await;
    assert!(result["error"].is_null(), "{result}");
    validate_report(&result["data"], "refresh_account");
    assert_eq!(result["data"]["account"]["credential_state"], "stored");
    assert_eq!(cards(&f).await.len(), 1);
    assert_eq!(f.secrets.puts.load(Ordering::SeqCst), 2);
    assert_eq!(f.peer.auth_count(), 4);
    assert_eq!(f.peer.reads.load(Ordering::SeqCst), 2);
    assert_eq!(f.cloud.calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn selected_modern_protocol_without_registered_transport_refuses_before_legacy_io() {
    let f = ModernFixture::new();
    let first = connect(&f).await;
    let adapter = Arc::new(MijiaAdapter::new(
        f.vault.clone(),
        f.cloud.clone(),
        Arc::new(SystemClock),
    ));
    let core = Core::new(
        Arc::new(SqliteStore::open(&f.directory).unwrap()),
        vec![adapter.clone()],
        Arc::new(SystemClock),
        Policy::default(),
    );
    let session = Session::with_platform_accounts(core, None, Some(adapter));
    let request: HostRequest = serde_json::from_value(
        json!({"schema_version":"1.0","operation_id":"no-modern-transport","timeout_ms":10000,
        "request":{"method":"refresh_account","account_id":id(&first)}}),
    )
    .unwrap();
    let result = session.call(request).await;
    assert!(result["error"].is_null(), "{result}");
    assert_eq!(result["data"]["retrieval"]["status"], "failed");
    assert_eq!(
        result["data"]["retrieval"]["error"]["code"],
        "CAPABILITY_UNSUPPORTED"
    );
    assert_eq!(result["data"]["retrieval"]["device_ids"], json!([]));
    assert_eq!(cards(&f).await.len(), 1);
    assert_eq!(f.peer.auth_count(), 4);
    assert_eq!(f.peer.reads.load(Ordering::SeqCst), 1);
    assert_eq!(f.cloud.calls.load(Ordering::SeqCst), 0);
    session.close();
}

#[tokio::test]
async fn qr_outlet_uses_ordinary_native_history_without_login_or_inventory_replay() {
    let f = ModernFixture::new();
    *f.peer.directory.lock().unwrap() = Some(
        json!({"list":[{"did":"PRIVATE-MODERN-DID","name":"Modern outlet","model":"cuco.plug.v3","pid":0}],"has_more":false}),
    );
    connect(&f).await;
    let card = cards(&f).await.remove(0);
    let inspect = json!({"method":"inspect_history_options","device_id":card["id"],"connection_id":card["connections"][0]["id"]});
    let options = f.call(inspect.clone()).await;
    assert!(options["error"].is_null(), "{options}");
    assert_eq!(options["data"]["options"].as_array().unwrap().len(), 1);
    assert_eq!(f.peer.statistics.load(Ordering::SeqCst), 0);
    let read = json!({"method":"read_history","history":{"schema_version":"1.0","device_id":card["id"],"connection_id":card["connections"][0]["id"],"target":options["data"]["options"][0]["target"],"aggregation":"reported_daily","start":"2023-11-14T22:13:20Z","end":"2023-11-15T22:13:20Z"}});
    let auth = f.peer.auth_count();
    let puts = f.secrets.puts.load(Ordering::SeqCst);
    let history = f.call(read.clone()).await;
    assert!(history["error"].is_null(), "{history}");
    let parsed: homeverb_domain::HistoryResult =
        serde_json::from_value(history["data"].clone()).unwrap();
    assert_eq!(parsed.records[0].value, 1.25);
    assert_eq!(
        parsed.completeness,
        homeverb_domain::HistoryCompleteness::Unproven
    );
    assert_eq!(f.peer.statistics.load(Ordering::SeqCst), 1);
    assert_eq!(f.peer.reads.load(Ordering::SeqCst), 1);
    assert_eq!(f.peer.auth_count(), auth);
    assert_eq!(f.secrets.puts.load(Ordering::SeqCst), puts);
    assert_eq!(f.cloud.calls.load(Ordering::SeqCst), 0);
    f.peer.fail_read.store(true, Ordering::SeqCst);
    assert_eq!(
        f.call(read.clone()).await["error"]["code"],
        "MIJIA_REQUEST_FAILED"
    );
    assert_eq!(f.peer.statistics.load(Ordering::SeqCst), 2);
    f.session.set_foreground(false);
    assert_eq!(f.call(read).await["error"]["code"], "HOST_BACKGROUND");
    assert_eq!(f.peer.statistics.load(Ordering::SeqCst), 2);
    assert_eq!(f.peer.auth_count(), auth);
    assert_eq!(f.cloud.calls.load(Ordering::SeqCst), 0);
}
