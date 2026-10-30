use super::*;
use crate::action::tests::{Peer as ActionPeer, *};
use homeverb_core::WriteProgress;
use homeverb_domain::Phase;
use std::sync::atomic::Ordering;

fn target(s: &ModernMijiaSession, model: &str, spec: &str) -> ModernDirectoryEntry {
    s.directory_entry(EntryWire {
        did: PrivateString::new(DID.into()),
        name: "Fictional action target".into(),
        model: model.into(),
        spec_type: Some(spec.into()),
        online: None,
        pid: Some(DirectoryKind(0)),
        parent_id: None,
    })
    .unwrap()
}
#[tokio::test]
async fn every_modern_action_correlates_exact_acknowledgement_without_replay() {
    let s = session();
    for (model, spec, query) in profiles() {
        let entry = target(&s, &model, &spec);
        for (code, expected) in outcomes() {
            let peer = ActionPeer::new(query, ack(query, code));
            let p = WriteProgress::default();
            assert_outcome(
                s.invoke_action(&peer, &entry, &query, &p, &super::operation())
                    .await,
                &p,
                expected,
            );
            assert!(
                s.invoke_action(&peer, &entry, &query, &p, &super::operation())
                    .await
                    .is_err()
            );
            assert_eq!(peer.count(), 1);
        }
        for kind in [0, 6, 16, 18] {
            let mut entry = target(&s, &model, &spec);
            entry.kind = Some(DirectoryKind(kind));
            let mut reply = ack(query, 0);
            reply["result"].as_object_mut().unwrap().remove("out");
            let mut peer = ActionPeer::new(query, reply);
            peer.reply = super::gzip(&peer.reply);
            peer.encoding = ModernContentEncoding::Gzip;
            let p = WriteProgress::default();
            assert_outcome(
                s.invoke_action(&peer, &entry, &query, &p, &super::operation())
                    .await,
                &p,
                Some(Phase::Accepted),
            );
        }
    }
}
#[tokio::test]
async fn modern_action_refuses_foreign_malformed_late_and_unbounded_replies() {
    let s = session();
    let (model, spec, query) = profiles().remove(0);
    let entry = target(&s, &model, &spec);
    for reply in bad_replies(query) {
        let peer = ActionPeer::raw(query, reply);
        let p = WriteProgress::default();
        assert_outcome(
            s.invoke_action(&peer, &entry, &query, &p, &super::operation())
                .await,
            &p,
            None,
        );
        assert_eq!(peer.count(), 1);
    }
    for bytes in [
        vec![31, 139, 0],
        super::gzip(&vec![b'x'; crate::MAX_RESPONSE_BYTES + 1]),
        [super::gzip(b"{}"), super::gzip(b"{}")].concat(),
    ] {
        let mut peer = ActionPeer::raw(query, bytes);
        peer.encoding = ModernContentEncoding::Gzip;
        let p = WriteProgress::default();
        assert_outcome(
            s.invoke_action(&peer, &entry, &query, &p, &super::operation())
                .await,
            &p,
            None,
        );
        assert_eq!(peer.count(), 1);
    }
    for fault in [
        Fault::Plaintext,
        Fault::NoMarker,
        Fault::BeforeSend,
        Fault::AfterSend,
        Fault::CancelReply,
        Fault::DeadlineReply,
    ] {
        let mut peer = ActionPeer::new(query, ack(query, 0));
        peer.fault = fault;
        let p = WriteProgress::default();
        assert!(
            s.invoke_action(
                &peer,
                &entry,
                &query,
                &p,
                &Operation::new(20, CancellationToken::new()).unwrap()
            )
            .await
            .is_err()
        );
        assert_eq!(
            p.phase(),
            if matches!(fault, Fault::NoMarker | Fault::BeforeSend) {
                Phase::NotSent
            } else {
                Phase::PossiblySent
            }
        );
        assert_eq!(peer.count(), 1);
    }
}
#[tokio::test]
async fn modern_action_requires_exact_subject_kind_root_model_spec_and_unused_progress() {
    let s = session();
    let (model, spec, query) = profiles().remove(0);
    let peer = ActionPeer::new(query, ack(query, 0));
    for mode in 0..15 {
        let mut entry = target(&s, &model, &spec);
        let mut q = query;
        let p = WriteProgress::default();
        let mut op = super::operation();
        match mode {
            0 => entry.subject = PrivateString::new("foreign".into()),
            1 => entry.encrypted_subject = PrivateString::new("foreign".into()),
            2 => entry.kind = Some(DirectoryKind(21)),
            3 => entry.kind = None,
            4 => entry.parent = Some(PrivateString::new("fictional-parent".into())),
            5 => entry.model = "foreign.light".into(),
            6 => entry.spec_type = Some("foreign-spec".into()),
            7 => entry.mapping = None,
            8 => q.aiid = 99,
            9 => q.siid = 99,
            10 => entry.did = PrivateString::new("".into()),
            11 => p.mark_sent(),
            12 => p.mark_accepted(),
            13 => op.cancellation.cancel(),
            _ => op.deadline = tokio::time::Instant::now(),
        }
        let before = p.phase();
        assert!(s.invoke_action(&peer, &entry, &q, &p, &op).await.is_err());
        assert_eq!(p.phase(), before);
    }
    assert_eq!(peer.count(), 0);
}
#[tokio::test]
async fn modern_action_pending_cancel_deadline_and_drop_preserve_uncertainty() {
    let s = session();
    let (model, spec, query) = profiles().remove(0);
    let entry = target(&s, &model, &spec);
    for mode in 0..3 {
        let mut peer = ActionPeer::new(query, ack(query, 0));
        peer.fault = Fault::Stall;
        let p = WriteProgress::default();
        let op =
            Operation::new(if mode == 1 { 40 } else { 5000 }, CancellationToken::new()).unwrap();
        let mut work = Box::pin(s.invoke_action(&peer, &entry, &query, &p, &op));
        tokio::select! { _=peer.started.notified()=>{},_=&mut work=>panic!("Expected pending action") }
        assert_eq!(peer.active.load(Ordering::SeqCst), 1);
        if mode == 0 {
            op.cancellation.cancel();
            assert_eq!(work.await.err().unwrap().code, "CANCELLED");
        } else if mode == 1 {
            assert_eq!(work.await.err().unwrap().code, "DEADLINE_EXCEEDED");
        } else {
            drop(work);
        }
        assert_eq!(peer.active.load(Ordering::SeqCst), 0);
        assert_eq!(p.phase(), Phase::PossiblySent);
        assert!(
            s.invoke_action(&peer, &entry, &query, &p, &super::operation())
                .await
                .is_err()
        );
        assert_eq!(peer.count(), 1);
    }
}
