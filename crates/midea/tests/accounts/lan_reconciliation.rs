use super::*;
use homeverb_midea::lan::{DiscoveryCandidate, DiscoveryNetwork, DiscoveryReport, LanProtocol};
use std::net::{Ipv4Addr, SocketAddrV4};

const ID: &str = "90123456789";

fn network() -> DiscoveryNetwork {
    DiscoveryNetwork::new(Ipv4Addr::new(192, 168, 42, 2), 24).unwrap()
}

fn candidate(appliance: &Appliance) -> DiscoveryCandidate {
    DiscoveryCandidate {
        appliance_id: appliance.id.clone(),
        endpoint: SocketAddrV4::new(Ipv4Addr::new(192, 168, 42, 10), 6444),
        protocol: LanProtocol::V3,
        family: appliance.family,
    }
}

fn report(candidates: Vec<DiscoveryCandidate>) -> DiscoveryReport {
    DiscoveryReport {
        received_datagrams: candidates.len(),
        candidates,
        limit_reached: false,
    }
}

async fn setup(f: &Fixture, id: &str, family: &str) -> (MeijuAccount, DeviceBinding, Inventory) {
    let mut account = f.account().await;
    let listing = inventory(
        account.record().credential_ref,
        vec![appliance(id, family), appliance("22", "AC")],
    )
    .await;
    let bindings = account
        .bind_inventory(&listing, &operation())
        .await
        .unwrap();
    (account, bindings[0], listing)
}

#[tokio::test(start_paused = true)]
async fn matches_only_saved_identity_without_admitting_a_model_or_mutating_storage() {
    for family in ["AC", "E1", "CA"] {
        let f = Fixture::new();
        let (account, binding, listing) = setup(&f, "281474976710655", family).await;
        let reference = account.record().credential_ref;
        let original = f.bytes(reference);
        let puts = f.puts();
        let saved = descriptor(binding, account.record().id);
        f.store.save_device(&saved).unwrap();
        for protocol in [LanProtocol::V2, LanProtocol::V3] {
            let mut hint = candidate(&listing.appliances[0]);
            hint.protocol = protocol;
            let report = report(vec![hint]);
            let result = account
                .reconcile_lan_candidate(binding, &network(), &report, &operation())
                .await
                .unwrap();
            assert_eq!(
                format!("{result:?}"),
                "UniqueCandidate(DiscoveryCandidate([REDACTED]))"
            );
            let LanCandidateMatch::UniqueCandidate(matched) = result else {
                panic!("Expected one untrusted candidate");
            };
            assert!(std::ptr::eq(matched, &report.candidates[0]));
            assert_eq!(matched.protocol, protocol);
        }
        assert_eq!(f.bytes(reference), original);
        assert_eq!(f.puts(), puts);
        assert_eq!(
            serde_json::to_value(f.store.devices().unwrap()).unwrap(),
            serde_json::to_value(vec![saved]).unwrap()
        );
    }
}

#[tokio::test(start_paused = true)]
async fn exact_duplicates_and_unrelated_conflicts_never_choose_a_different_identity() {
    let f = Fixture::new();
    let (account, binding, listing) = setup(&f, ID, "AC").await;
    let mut unrelated = candidate(&listing.appliances[1]);
    unrelated.endpoint.set_ip(Ipv4Addr::new(192, 168, 42, 11));
    let mut conflicting_unrelated = candidate(&listing.appliances[1]);
    conflicting_unrelated
        .endpoint
        .set_ip(Ipv4Addr::new(192, 168, 42, 12));
    let mut report = report(vec![
        unrelated,
        candidate(&listing.appliances[0]),
        conflicting_unrelated,
        candidate(&listing.appliances[0]),
    ]);
    for _ in 0..4 {
        let result = account
            .reconcile_lan_candidate(binding, &network(), &report, &operation())
            .await
            .unwrap();
        let LanCandidateMatch::UniqueCandidate(hint) = result else {
            panic!("Expected identical selected-ID candidates");
        };
        assert_eq!(hint.appliance_id.expose(), ID);
        report.candidates.rotate_left(1);
    }
}

#[tokio::test(start_paused = true)]
async fn selected_identity_and_endpoint_conflicts_remain_ambiguous_in_both_orders() {
    let f = Fixture::new();
    let (account, binding, listing) = setup(&f, ID, "AC").await;
    for case in [
        "address",
        "port",
        "protocol",
        "family",
        "foreign-id",
        "family-only",
        "family-alias",
    ] {
        let mut second = candidate(&listing.appliances[0]);
        match case {
            "address" => second.endpoint.set_ip(Ipv4Addr::new(192, 168, 42, 11)),
            "port" => second.endpoint.set_port(6445),
            "protocol" => second.protocol = LanProtocol::V2,
            "family" | "family-only" => second.family = ApplianceFamily::Refrigerator,
            "family-alias" => second.family = ApplianceFamily::Other(0xac),
            "foreign-id" => second.appliance_id = listing.appliances[1].id.clone(),
            _ => unreachable!(),
        }
        let mut hints = vec![second];
        if !matches!(case, "family-only" | "family-alias") {
            hints.push(candidate(&listing.appliances[0]));
        }
        let mut report = report(hints);
        for _ in 0..2 {
            assert!(
                matches!(
                    account
                        .reconcile_lan_candidate(binding, &network(), &report, &operation())
                        .await
                        .unwrap(),
                    LanCandidateMatch::Ambiguous
                ),
                "{case}"
            );
            report.candidates.reverse();
        }
    }
}

#[tokio::test(start_paused = true)]
async fn missing_and_limited_reports_do_not_establish_absence_or_unique_peers() {
    let f = Fixture::new();
    let (account, binding, listing) = setup(&f, ID, "AC").await;
    for hints in [vec![], vec![candidate(&listing.appliances[1])]] {
        let report = report(hints);
        assert!(matches!(
            account
                .reconcile_lan_candidate(binding, &network(), &report, &operation())
                .await
                .unwrap(),
            LanCandidateMatch::NotObserved
        ));
    }
    for hints in [
        vec![],
        vec![candidate(&listing.appliances[0])],
        (0..64).map(|_| candidate(&listing.appliances[0])).collect(),
    ] {
        let mut report = report(hints);
        report.received_datagrams = 256;
        report.limit_reached = true;
        assert!(matches!(
            account
                .reconcile_lan_candidate(binding, &network(), &report, &operation())
                .await
                .unwrap(),
            LanCandidateMatch::Incomplete
        ));
    }
}

#[tokio::test(start_paused = true)]
async fn malformed_native_reports_fail_before_candidate_selection() {
    let f = Fixture::new();
    let (account, binding, listing) = setup(&f, ID, "AC").await;
    let wide = inventory(
        account.record().credential_ref,
        vec![appliance("281474976710656", "AC")],
    )
    .await;
    for case in [
        "subnet",
        "self",
        "broadcast",
        "network",
        "public",
        "port",
        "count",
        "datagrams",
        "candidates",
        "unmarked-datagrams",
        "unmarked-candidates",
        "wide-id",
    ] {
        let mut report = report(vec![candidate(&listing.appliances[0])]);
        let hint = &mut report.candidates[0];
        match case {
            "subnet" => hint.endpoint.set_ip(Ipv4Addr::new(192, 168, 43, 10)),
            "self" => hint.endpoint.set_ip(Ipv4Addr::new(192, 168, 42, 2)),
            "broadcast" => hint.endpoint.set_ip(Ipv4Addr::new(192, 168, 42, 255)),
            "network" => hint.endpoint.set_ip(Ipv4Addr::new(192, 168, 42, 0)),
            "public" => hint.endpoint.set_ip(Ipv4Addr::new(203, 0, 113, 10)),
            "port" => hint.endpoint.set_port(0),
            "count" => report.received_datagrams = 0,
            "datagrams" => report.received_datagrams = 257,
            "unmarked-datagrams" => report.received_datagrams = 256,
            "candidates" | "unmarked-candidates" => {
                let count = if case == "candidates" { 65 } else { 64 };
                report.candidates = (0..count)
                    .map(|_| candidate(&listing.appliances[0]))
                    .collect();
                report.received_datagrams = count;
            }
            "wide-id" => hint.appliance_id = wide.appliances[0].id.clone(),
            _ => unreachable!(),
        }
        let error = account
            .reconcile_lan_candidate(binding, &network(), &report, &operation())
            .await
            .unwrap_err();
        assert_eq!(error.code, "INVALID_ARGUMENT", "{case}");
        assert!(!error.message.contains(ID));
        assert!(!error.message.contains("192.168"));
    }
}

#[tokio::test(start_paused = true)]
async fn exact_account_binding_membership_and_scope_are_required() {
    for case in [
        "device",
        "connection",
        "identity",
        "unlisted",
        "unknown",
        "family",
        "wide",
    ] {
        let f = Fixture::new();
        let (account, mut binding, listing) = setup(&f, ID, "AC").await;
        let reference = account.record().credential_ref;
        let mut payload: Value = serde_json::from_slice(&f.bytes(reference)).unwrap();
        let expected = match case {
            "device" => {
                binding.device_id = Uuid::new_v4();
                "DEVICE_IDENTITY_CONFLICT"
            }
            "connection" => {
                binding.connection_id = Uuid::new_v4();
                "DEVICE_IDENTITY_CONFLICT"
            }
            "identity" => {
                binding.identity_ref = Uuid::new_v4();
                "DEVICE_IDENTITY_CONFLICT"
            }
            "unlisted" => {
                payload["bindings"][0]["listed"] = json!(false);
                "MIDEA_DEVICE_NOT_LISTED"
            }
            "unknown" => {
                payload["bindings"][0]["listed"] = Value::Null;
                "MIDEA_INVENTORY_REQUIRED"
            }
            "family" => {
                payload["bindings"][0]["family"] = json!(0xff);
                "CAPABILITY_UNSUPPORTED"
            }
            "wide" => {
                payload["bindings"][0]["appliance_id"] = json!("281474976710656");
                "CAPABILITY_UNSUPPORTED"
            }
            _ => unreachable!(),
        };
        f.set_bytes(reference, serde_json::to_vec(&payload).unwrap());
        let original = f.bytes(reference);
        let puts = f.puts();
        let report = report(vec![candidate(&listing.appliances[0])]);
        assert_eq!(
            account
                .reconcile_lan_candidate(binding, &network(), &report, &operation())
                .await
                .unwrap_err()
                .code,
            expected,
            "{case}"
        );
        assert_eq!(f.bytes(reference), original);
        assert_eq!(f.puts(), puts);
    }
}

#[tokio::test(start_paused = true)]
async fn foreign_accounts_and_lifecycle_failures_do_not_reuse_historical_hints() {
    let f = Fixture::new();
    let (account, binding, listing) = setup(&f, ID, "AC").await;
    let reference = account.record().credential_ref;
    let report = report(vec![candidate(&listing.appliances[0])]);
    let (other, other_binding, _) = setup(&f, ID, "AC").await;
    assert_ne!(binding, other_binding);
    assert_eq!(
        other
            .reconcile_lan_candidate(binding, &network(), &report, &operation())
            .await
            .unwrap_err()
            .code,
        "DEVICE_IDENTITY_CONFLICT"
    );
    let foreign = MeijuAccount::create(
        &f.vault,
        "EU",
        "Other",
        &secret("subject"),
        &secret("session"),
        &operation(),
    )
    .await
    .unwrap();
    assert_eq!(
        foreign
            .reconcile_lan_candidate(binding, &network(), &report, &operation())
            .await
            .unwrap_err()
            .code,
        "AUTH_FLOW_UNSUPPORTED"
    );
    let cancelled = operation();
    cancelled.cancellation.cancel();
    assert_eq!(
        account
            .reconcile_lan_candidate(binding, &network(), &report, &cancelled)
            .await
            .unwrap_err()
            .code,
        "CANCELLED"
    );
    let expired = operation();
    tokio::time::advance(std::time::Duration::from_secs(1)).await;
    assert_eq!(
        account
            .reconcile_lan_candidate(binding, &network(), &report, &expired)
            .await
            .unwrap_err()
            .code,
        "DEADLINE_EXCEEDED"
    );
    f.secrets
        .values
        .lock()
        .unwrap()
        .remove(&reference.to_string());
    assert!(
        account
            .reconcile_lan_candidate(binding, &network(), &report, &operation())
            .await
            .is_err()
    );
}
