use super::*;
use homeverb_domain::{Phase, Risk, Status, Transport};

const CAPABILITY: &str = "vendor.midea.dishwasher";
pub(super) fn inventory(profile: Value) -> Vec<Value> {
    let mut entry = appliances().pop().unwrap();
    entry
        .as_object_mut()
        .unwrap()
        .extend(profile.as_object().unwrap().clone());
    vec![entry]
}
fn core(f: &Fixture, device: &Device, enabled: bool) -> Core {
    let mut adapter = MeijuAdapter::new(f.vault.clone(), f.wire.clone(), Arc::new(SystemClock));
    if enabled {
        adapter = adapter.with_dishwasher_control();
    }
    Core::new(
        f.store.clone(),
        vec![Arc::new(adapter)],
        Arc::new(SystemClock),
        Policy {
            grants: vec![Grant {
                device_id: device.id.clone(),
                component_id: "main".into(),
                capability: CAPABILITY.into(),
                action: "start_cycle".into(),
                transports: vec![Transport::Cloud],
                argument_schema: json!({"type":"object"}),
            }],
        },
    )
}
fn command(device: &Device, arguments: Value) -> Command {
    let mut value = power_command(device, false);
    value.capability = CAPABILITY.into();
    value.action = "start_cycle".into();
    value.arguments = arguments;
    value
}
async fn approved(core: &Core, command: Command) -> homeverb_domain::Execution {
    let preview = core.prepare_user_approval(command).unwrap();
    core.execute_user_approved(preview, &operation()).await
}

#[tokio::test]
async fn reviewed_programs_encode_exact_fields_only_after_approval_without_cache_or_read_changes() {
    for product in ["7600020L", "76006481", "7600649Q", "7600V1E7", "760Y0026"] {
        let f = Fixture::new(inventory(json!({"modelNumber":1,"sn8":product}))).await;
        let device = f.discover().await.remove(0);
        let before = f.read(&device, "main", false).await.unwrap();
        let enabled = core(&f, &device, true);
        enabled.discover(&operation()).await.unwrap();
        let descriptor = enabled.describe(&device.id).unwrap();
        let capability = descriptor.capability("main", CAPABILITY).unwrap();
        let action = &capability.actions["start_cycle"];
        assert_eq!(action.risk, Risk::Elevated);
        assert_eq!(
            action.idempotency,
            homeverb_domain::Idempotency::NonIdempotent
        );
        assert!(!action.atomic && !action.independent_fields);
        assert!(
            capability
                .properties
                .values()
                .all(|p| p["writable"] == false)
        );
        let programs = action.parameters["properties"]["program"]["enum"]
            .as_array()
            .unwrap();
        assert_eq!(
            programs.len(),
            match product {
                "7600V1E7" => 9,
                "760Y0026" => 12,
                _ => 13,
            }
        );
        for program in programs {
            let additions: &[Option<u8>] = if product == "760Y0026" {
                &[Some(0), Some(9), Some(10), Some(13), Some(14)]
            } else {
                &[None]
            };
            for additional in additions {
                let mut args = json!({"program":program});
                if let Some(value) = additional {
                    args["additional"] = json!(value);
                }
                let c = command(&device, args);
                assert_eq!(enabled.plan(&c).unwrap_err().code, "POLICY_DENIED");
                assert_eq!(
                    enabled
                        .execute(c.clone(), CancellationToken::new())
                        .await
                        .error
                        .unwrap()
                        .code,
                    "POLICY_DENIED"
                );
                let reads = f.calls();
                let sends = f.wire.controls.lock().unwrap().len();
                let preview = enabled.prepare_user_approval(c.clone()).unwrap();
                let duplicate = enabled.prepare_user_approval(c.clone()).unwrap();
                let result = enabled.execute_user_approved(preview, &operation()).await;
                assert_eq!(result.status, Status::Accepted);
                assert_eq!(result.error.as_ref().unwrap().code, "VERIFICATION_PENDING");
                assert!(result.verification.is_none());
                let expected_mode = match (product, program.as_str().unwrap()) {
                    ("7600020L", "auto_wash") => "auto",
                    ("7600020L", "strong_wash") => "intensive",
                    ("7600020L", "standard_wash") => "normal",
                    ("7600020L", "eco_wash") => "eco",
                    ("7600020L", "glass_wash") => "glass",
                    ("7600020L", "90min_wash") => "90min",
                    ("7600020L", "fast_wash") => "rapid",
                    ("7600020L", "soak_wash") => "soak",
                    ("7600020L", "hour_wash") => "1hour",
                    ("7600020L", "quietnight_wash") => "quiet",
                    ("7600020L", "germ") => "hygiene",
                    ("7600020L", "fruit_wash") => "fruit",
                    ("7600V1E7", "soft_wash") => "glass_wash",
                    (_, value) => value,
                };
                let mut expected = json!({"work_status":"work","mode":expected_mode});
                if let Some(value) = additional {
                    expected["additional"] = json!(value);
                }
                assert_eq!(
                    f.wire.controls.lock().unwrap().last().unwrap()["command"],
                    json!({"control":expected})
                );
                assert_eq!(f.calls(), reads);
                assert_eq!(
                    json!(f.store.snapshot(&device.id, "main").unwrap().unwrap()),
                    json!(before)
                );
                assert_eq!(
                    json!(enabled.execute_user_approved(duplicate, &operation()).await),
                    json!(result)
                );
                assert_eq!(
                    json!(
                        enabled
                            .execution(&c.request_id, &operation())
                            .await
                            .unwrap()
                    ),
                    json!(result)
                );
                assert_eq!(f.wire.controls.lock().unwrap().len(), sends + 1);
            }
        }
        if product == "760Y0026" {
            let fixture: homeverb_domain::Capability = serde_json::from_str(include_str!(
                "../../../../examples/capability.meiju-dishwasher.json"
            ))
            .unwrap();
            assert_eq!(json!(capability), json!(fixture));
        }
        // Omitted later inventory fields cannot erase retained protected mappings.
        *f.wire.inventories.lock().unwrap().get_mut("FIRST").unwrap() = inventory(json!({}));
        enabled.discover(&operation()).await.unwrap();
        assert_eq!(
            json!(enabled.describe(&device.id).unwrap()),
            json!(descriptor)
        );
        core(&f, &device, false)
            .discover(&operation())
            .await
            .unwrap();
        assert!(
            f.core
                .describe(&device.id)
                .unwrap()
                .capability("main", CAPABILITY)
                .unwrap()
                .actions
                .is_empty()
        );
    }
}

#[tokio::test]
async fn missing_unreviewed_or_wrong_family_evidence_never_supplies_a_dishwasher_action() {
    for profile in [
        json!({}),
        json!({"sn8":"76006481"}),
        json!({"modelNumber":1}),
        json!({"modelNumber":1,"sn8":"TEST1234"}),
        json!({"modelNumber":1,"sn8":"7600016L"}),
        json!({"modelNumber":1,"sn8":"76006481","productModel":"Unreviewed dishwasher"}),
        json!({"modelNumber":1,"sn8":"76006481","type":"AC"}),
    ] {
        let f = Fixture::new(inventory(profile)).await;
        let device = f.discover().await.remove(0);
        let enabled = core(&f, &device, true);
        enabled.discover(&operation()).await.unwrap();
        assert!(
            enabled
                .describe(&device.id)
                .unwrap()
                .capabilities
                .iter()
                .all(|cap| !cap.actions.contains_key("start_cycle"))
        );
        assert!(
            enabled
                .prepare_user_approval(command(&device, json!({"program":"eco_wash"})))
                .is_err()
        );
        assert!(f.wire.controls.lock().unwrap().is_empty());
    }
}

#[tokio::test]
async fn profile_constraints_and_private_approval_marker_survive_public_metadata_tampering() {
    for (product, bad) in [
        ("760Y0026", json!({"program":"eco_wash"})),
        ("760Y0026", json!({"program":"neutral_gear","additional":0})),
        ("760Y0026", json!({"program":"eco_wash","additional":1})),
        ("760Y0026", json!({"program":"eco_wash","additional":null})),
        ("76006481", json!({"program":"eco_wash","additional":0})),
        ("7600V1E7", json!({"program":"90min_wash"})),
        ("7600020L", json!({"program":"eco"})),
        (
            "76006481",
            json!({"program":"eco_wash","work_status":"work"}),
        ),
    ] {
        let f = Fixture::new(inventory(json!({"modelNumber":1,"sn8":product}))).await;
        let device = f.discover().await.remove(0);
        let enabled = core(&f, &device, true);
        enabled.discover(&operation()).await.unwrap();
        let c = command(&device, bad);
        assert!(enabled.prepare_user_approval(c.clone()).is_err());
        let mut descriptor = enabled.describe(&device.id).unwrap();
        descriptor.capabilities[0]
            .actions
            .get_mut("start_cycle")
            .unwrap()
            .parameters = json!({"type":"object"});
        f.store.save_device(&descriptor).unwrap();
        let result = approved(&enabled, c).await;
        assert_eq!(result.status, Status::Failed);
        assert_eq!(result.attempts[0].phase, Phase::NotSent);
        assert!(f.wire.controls.lock().unwrap().is_empty());
    }
    let f = Fixture::new(inventory(json!({"modelNumber":1,"sn8":"76006481"}))).await;
    let device = f.discover().await.remove(0);
    let enabled = core(&f, &device, true);
    enabled.discover(&operation()).await.unwrap();
    let mut descriptor = enabled.describe(&device.id).unwrap();
    descriptor.capabilities[0]
        .actions
        .get_mut("start_cycle")
        .unwrap()
        .risk = Risk::Low;
    f.store.save_device(&descriptor).unwrap();
    f.secrets.unavailable.store(true, Ordering::Relaxed);
    let result = enabled
        .execute(
            command(&device, json!({"program":"eco_wash"})),
            CancellationToken::new(),
        )
        .await;
    assert_eq!(result.error.unwrap().code, "POLICY_DENIED");
    assert_eq!(result.attempts[0].phase, Phase::NotSent);
    assert!(f.wire.controls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn approved_cycle_preserves_uncertainty_rejection_and_pre_send_refusal() {
    for (fault, status, phase) in [
        (0, Status::Accepted, Phase::Accepted),
        (1, Status::Failed, Phase::NotSent),
        (2, Status::Unknown, Phase::PossiblySent),
        (4, Status::Failed, Phase::NotSent),
        (5, Status::Failed, Phase::Rejected),
    ] {
        let f = Fixture::new(inventory(json!({"modelNumber":1,"sn8":"76006481"}))).await;
        let device = f.discover().await.remove(0);
        let enabled = core(&f, &device, true);
        enabled.discover(&operation()).await.unwrap();
        f.wire.control_fault.store(fault, Ordering::Relaxed);
        let c = command(&device, json!({"program":"eco_wash"}));
        let result = approved(&enabled, c.clone()).await;
        assert_eq!(result.status, status);
        assert_eq!(result.attempts[0].phase, phase);
        assert!(!serde_json::to_string(&result).unwrap().contains("PRIVATE"));
        assert_eq!(
            json!(
                enabled
                    .execution(&c.request_id, &operation())
                    .await
                    .unwrap()
            ),
            json!(result)
        );
    }
}

#[tokio::test]
async fn cancelled_possible_cycle_send_is_unknown_and_membership_refusal_sends_nothing() {
    let f = Fixture::new(inventory(json!({"modelNumber":1,"sn8":"76006481"}))).await;
    let device = f.discover().await.remove(0);
    let enabled = core(&f, &device, true);
    enabled.discover(&operation()).await.unwrap();
    f.wire.control_fault.store(3, Ordering::Relaxed);
    let c = command(&device, json!({"program":"eco_wash"}));
    let preview = enabled.prepare_user_approval(c.clone()).unwrap();
    let op = operation();
    let work = enabled.execute_user_approved(preview, &op);
    tokio::pin!(work);
    tokio::select! { _=f.wire.control_started.notified()=>(), result=&mut work=>panic!("Expected pending: {result:?}") }
    op.cancellation.cancel();
    let result = work.await;
    assert_eq!(result.status, Status::Unknown);
    assert_eq!(result.attempts[0].phase, Phase::PossiblySent);
    let receipt = enabled
        .execution(&c.request_id, &operation())
        .await
        .unwrap();
    assert_eq!(json!(receipt), json!(result));
    let next = command(&device, json!({"program":"eco_wash"}));
    let preview = enabled.prepare_user_approval(next).unwrap();
    f.wire
        .inventories
        .lock()
        .unwrap()
        .get_mut("FIRST")
        .unwrap()
        .clear();
    enabled.discover(&operation()).await.unwrap();
    let result = enabled.execute_user_approved(preview, &operation()).await;
    assert_eq!(result.error.unwrap().code, "MIDEA_DEVICE_NOT_LISTED");
    assert_eq!(result.attempts[0].phase, Phase::NotSent);
    assert_eq!(f.wire.controls.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn forged_public_actions_cannot_replace_protected_product_model_family_or_host_admission() {
    for (profile, enabled) in [
        (json!({"modelNumber":1,"sn8":"TEST1234"}), true),
        (json!({"modelNumber":1,"sn8":"7600016L"}), true),
        (
            json!({"modelNumber":1,"sn8":"760Y0026","productModel":"Unreviewed"}),
            true,
        ),
        (json!({"modelNumber":1,"sn8":"760Y0026","type":"AC"}), true),
        (json!({"modelNumber":1,"sn8":"760Y0026"}), false),
    ] {
        let f = Fixture::new(inventory(profile)).await;
        let device = f.discover().await.remove(0);
        let enabled = core(&f, &device, enabled);
        let mut descriptor = enabled.describe(&device.id).unwrap();
        descriptor.device.model = MODELS[5].0.into();
        descriptor.device.support_status = "experimental".into();
        descriptor.device.components = vec![homeverb_domain::Component {
            id: "main".into(),
            capabilities: vec![CAPABILITY.into()],
        }];
        descriptor.capabilities = vec![
            serde_json::from_str(include_str!(
                "../../../../examples/capability.meiju-dishwasher.json"
            ))
            .unwrap(),
        ];
        f.store.save_device(&descriptor).unwrap();
        let result = approved(
            &enabled,
            command(&device, json!({"program":"eco_wash","additional":0})),
        )
        .await;
        assert_eq!(result.status, Status::Failed);
        assert_eq!(result.error.unwrap().code, "CAPABILITY_UNSUPPORTED");
        assert_eq!(result.attempts[0].phase, Phase::NotSent);
        assert!(f.wire.controls.lock().unwrap().is_empty());
    }
}
