//! Fictional topology evidence; never an account export or physical acceptance.
use super::*;

fn wire(id: &str, kind: Value, parent: Option<&str>) -> Value {
    let mut value = entry(id);
    value["name"] = json!(format!("Fictional {id}"));
    value["pid"] = kind;
    if let Some(parent) = parent {
        value["parent_id"] = json!(parent);
    }
    value
}
fn directory(entries: Vec<Value>) -> ModernDirectory {
    let session = session();
    ModernDirectory {
        entries: entries
            .into_iter()
            .map(|v| {
                session
                    .directory_entry(serde_json::from_value(v).unwrap())
                    .unwrap()
            })
            .collect(),
        complete: false,
        issues: vec![],
    }
}
fn topology() -> Vec<Value> {
    vec![
        wire("root", json!(0), None),
        wire("switch", json!(18), Some("root")),
        wire("switch-peer", json!("18"), Some("root")),
        wire("split-left", json!(21), Some("switch")),
        wire("split-right", json!("21"), Some("switch")),
    ]
}

#[tokio::test]
async fn gateway_children_remain_devices_while_split_children_share_one_group() {
    for reverse in [false, true] {
        let mut wires = topology();
        if reverse {
            wires.reverse();
        }
        let result = directory(wires).reconcile(&operation()).await.unwrap();
        assert_eq!(result.groups.len(), 3);
        assert!(result.unresolved.is_empty());
        assert!(result.issues.is_empty());
        assert!(!result.complete);
        let switch = result
            .groups
            .iter()
            .find(|g| g.entry().name() == "Fictional switch")
            .unwrap();
        assert_eq!(switch.logical_children().len(), 2);
        assert!(!switch.entry().readable_properties().is_empty());
        assert_eq!(
            result
                .groups
                .iter()
                .map(|g| 1 + g.logical_children().len())
                .sum::<usize>(),
            5
        );
        assert_eq!(format!("{result:?}"), "ModernDeviceInventory([REDACTED])");
        assert_eq!(format!("{switch:?}"), "ModernDeviceGroup([REDACTED])");
    }
}

#[tokio::test]
async fn exact_numeric_and_text_kinds_admit_independent_candidates_without_id_heuristics() {
    for kind in [0, 6, 16, 18] {
        for value in [json!(kind), json!(kind.to_string())] {
            let result = directory(vec![wire("root.s1", value, Some(""))])
                .reconcile(&operation())
                .await
                .unwrap();
            assert_eq!(result.groups.len(), 1);
            assert!(result.unresolved.is_empty());
            assert!(!result.complete);
        }
    }
}

#[tokio::test]
async fn unknown_and_missing_evidence_is_retained_without_creating_device_groups() {
    let mut wires = vec![
        wire("missing-kind", Value::Null, None),
        wire("unknown-kind", json!(999), None),
        wire("missing-parent", json!(21), None),
        wire("absent-parent", json!(21), Some("absent")),
        wire("unknown-parent", json!(21), Some("unknown-kind")),
        wire("nested-child", json!(21), Some("missing-parent")),
    ];
    wires[0].as_object_mut().unwrap().remove("pid");
    let result = directory(wires).reconcile(&operation()).await.unwrap();
    assert!(result.groups.is_empty());
    assert_eq!(result.unresolved.len(), 6);
    use ModernUnresolvedReason::*;
    assert_eq!(
        result
            .unresolved
            .iter()
            .map(|v| v.reason())
            .collect::<Vec<_>>(),
        [
            MissingKind,
            UnknownKind,
            MissingParent,
            MissingParent,
            UnsupportedParent,
            UnsupportedParent
        ]
    );
    assert_eq!(
        result.unresolved[0].entry().name(),
        "Fictional missing-kind"
    );
    assert_eq!(
        format!("{:?}", result.unresolved[0]),
        "ModernUnresolvedEntry([REDACTED])"
    );
    assert_eq!(result.issues.len(), 1);
    assert_eq!(result.issues[0].code, "MIJIA_INVENTORY_PARTIAL");
}

#[tokio::test]
async fn graph_cycles_and_self_parenting_fail_before_group_publication() {
    for entries in [
        vec![wire("a", json!(0), Some("a"))],
        vec![
            wire("a", json!(18), Some("b")),
            wire("b", json!(18), Some("a")),
        ],
        vec![
            wire("a", json!(21), Some("b")),
            wire("b", json!(21), Some("c")),
            wire("c", json!(21), Some("a")),
        ],
    ] {
        assert_eq!(
            directory(entries)
                .reconcile(&operation())
                .await
                .unwrap_err()
                .code,
            "DEVICE_IDENTITY_CONFLICT"
        );
    }
}

#[tokio::test]
async fn mixed_subjects_and_duplicates_are_never_reconciled_by_equal_names_or_models() {
    for encrypted in [false, true] {
        let mut value = directory(vec![wire("a", json!(0), None), wire("b", json!(0), None)]);
        if encrypted {
            value.entries[1].encrypted_subject = PrivateString::new("foreign-c-user".into());
        } else {
            value.entries[1].subject = PrivateString::new("54321".into());
        }
        assert_eq!(
            value.reconcile(&operation()).await.unwrap_err().code,
            "DEVICE_IDENTITY_CONFLICT"
        );
    }
    let duplicate = vec![wire("same", json!(0), None), wire("same", json!(0), None)];
    assert_eq!(
        directory(duplicate)
            .reconcile(&operation())
            .await
            .unwrap_err()
            .code,
        "DEVICE_IDENTITY_CONFLICT"
    );
}

#[tokio::test]
async fn contradictory_split_model_or_spec_is_not_silently_hidden() {
    for model in [true, false] {
        let mut wires = topology();
        if model {
            wires[3]["model"] = json!("fiction.different.v1");
        } else {
            wires[1]["spec_type"] = json!("urn:miot-spec-v2:device:fan:0000A005:fiction:1");
            wires[3]["spec_type"] = json!("urn:miot-spec-v2:device:fan:0000A005:fiction:2");
        }
        assert_eq!(
            directory(wires)
                .reconcile(&operation())
                .await
                .unwrap_err()
                .code,
            "DEVICE_IDENTITY_CONFLICT"
        );
    }
}

#[test]
fn malformed_kind_and_parent_metadata_fail_decode_without_exposing_the_values() {
    for kind in [
        json!(-1),
        json!(1.5),
        json!(true),
        json!("021"),
        json!("+21"),
        json!(" 21"),
        json!(""),
        json!(4294967296_u64),
    ] {
        assert!(serde_json::from_value::<EntryWire>(wire("a", kind, None)).is_err());
    }
    for parent in [json!(123), json!({}), json!([])] {
        let mut value = wire("a", json!(0), None);
        value["parent_id"] = parent;
        assert!(serde_json::from_value::<EntryWire>(value).is_err());
    }
    for parent in ["\n", "a\0b", &"a".repeat(129)] {
        let wire = serde_json::from_value(wire("a", json!(21), Some(parent))).unwrap();
        assert_eq!(
            session().directory_entry(wire).unwrap_err().code,
            "MIJIA_INVALID_RESPONSE"
        );
    }
}

#[tokio::test]
async fn bounded_partition_preserves_original_issues_and_handles_worst_case_graph() {
    let wires = (0..4096)
        .map(|i| {
            wire(
                &format!("node-{i}"),
                json!(18),
                (i > 0).then(|| format!("node-{}", i - 1)).as_deref(),
            )
        })
        .collect();
    let mut value = directory(wires);
    value.issues = vec![Error::new(
        "MIJIA_REQUEST_FAILED",
        "Fictional page failure.",
    )];
    let result = value.reconcile(&operation()).await.unwrap();
    assert_eq!(result.groups.len(), 4096);
    assert!(result.unresolved.is_empty());
    assert_eq!(result.issues[0].code, "MIJIA_REQUEST_FAILED");
    let too_many = directory(
        (0..4097)
            .map(|i| wire(&format!("node-{i}"), json!(0), None))
            .collect(),
    );
    assert_eq!(
        too_many.reconcile(&operation()).await.unwrap_err().code,
        "MIJIA_INVALID_RESPONSE"
    );
}

#[tokio::test]
async fn cancellation_expiry_and_invented_completeness_cannot_publish_groups() {
    let op = operation();
    op.cancellation.cancel();
    assert_eq!(
        directory(topology()).reconcile(&op).await.unwrap_err().code,
        "CANCELLED"
    );
    let op = Operation {
        cancellation: CancellationToken::new(),
        deadline: tokio::time::Instant::now(),
    };
    assert_eq!(
        directory(topology()).reconcile(&op).await.unwrap_err().code,
        "DEADLINE_EXCEEDED"
    );
    let mut value = directory(topology());
    value.complete = true;
    assert_eq!(
        value.reconcile(&operation()).await.unwrap_err().code,
        "MIJIA_INVALID_RESPONSE"
    );
}

#[tokio::test]
async fn actual_paged_response_keeps_relationships_and_reconciliation_sends_nothing() {
    let mut entries = topology();
    let later = entries.split_off(3);
    let peer = Peer::new(vec![
        page(entries, true, Some("cursor")),
        page(later, false, None),
    ]);
    let directory = session().directory(&peer, &operation()).await.unwrap();
    assert_eq!(peer.count(), 2);
    let result = directory.reconcile(&operation()).await.unwrap();
    assert_eq!(result.groups.len(), 3);
    assert_eq!(peer.count(), 2);
}

#[tokio::test]
async fn cancellation_after_reconciliation_has_started_interrupts_local_work() {
    let value = directory(
        (0..512)
            .map(|i| wire(&format!("node-{i}"), json!(0), None))
            .collect(),
    );
    let op = operation();
    let token = op.cancellation.clone();
    let task = tokio::spawn(async move {
        tokio::task::yield_now().await;
        token.cancel();
    });
    assert_eq!(value.reconcile(&op).await.unwrap_err().code, "CANCELLED");
    task.await.unwrap();
}

#[tokio::test]
async fn missing_gateway_does_not_turn_an_independent_device_into_a_logical_split() {
    let result = directory(vec![wire("switch", json!(18), Some("absent-gateway"))])
        .reconcile(&operation())
        .await
        .unwrap();
    assert_eq!(result.groups.len(), 1);
    assert!(result.unresolved.is_empty());
    assert!(result.groups[0].logical_children().is_empty());
}

#[tokio::test]
async fn conflicting_kind_or_parent_on_duplicate_pages_is_identity_conflict() {
    for changed_kind in [false, true] {
        let first = wire("same", json!(18), Some("root-a"));
        let mut second = first.clone();
        if changed_kind {
            second["pid"] = json!(21);
        } else {
            second["parent_id"] = json!("root-b");
        }
        let peer = Peer::new(vec![
            page(vec![first], true, Some("cursor")),
            page(vec![second], false, None),
        ]);
        assert_eq!(
            session()
                .directory(&peer, &operation())
                .await
                .unwrap_err()
                .code,
            "DEVICE_IDENTITY_CONFLICT"
        );
    }
}

#[tokio::test]
async fn empty_inventory_and_bounded_existing_issues_do_not_invent_completeness() {
    let empty = directory(vec![]).reconcile(&operation()).await.unwrap();
    assert!(empty.groups.is_empty());
    assert!(empty.unresolved.is_empty());
    assert!(empty.issues.is_empty());
    assert!(!empty.complete);
    let mut value = directory(vec![wire("a", json!(21), None)]);
    value.issues = (0..64)
        .map(|_| Error::new("MIJIA_REQUEST_FAILED", "Fictional page failure."))
        .collect();
    let result = value.reconcile(&operation()).await.unwrap();
    assert_eq!(result.issues.len(), 64);
    assert!(
        result
            .issues
            .iter()
            .all(|e| e.code == "MIJIA_REQUEST_FAILED")
    );
    assert_eq!(result.unresolved.len(), 1);
}
