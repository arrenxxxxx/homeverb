use super::*;
use tokio_util::sync::CancellationToken;

const GROUP: &str = "SYNTHETIC.org.homeverb.test";
const REFERENCE: &[u8] = b"fictional persistent reference";

#[test]
fn os_status_mapping_uses_fixed_public_errors() {
    for (status, code) in [
        (-25308, "SECRET_STORE_LOCKED"),
        (-25293, "SECRET_STORE_LOCKED"),
        (-25299, "SECRET_STORE_CONFLICT"),
        (-25295, "SECRET_STORE_UNAVAILABLE"),
        (-34018, "SECRET_STORE_UNAVAILABLE"),
        (-1, "SECRET_STORE_UNAVAILABLE"),
    ] {
        let error = status_error(status);
        assert_eq!(error.code, code);
        assert!(!error.retryable);
        assert!(!error.message.contains(&status.to_string()));
    }
}

fn scope() -> Scope {
    Scope::new(Uuid::new_v4(), &CredentialRef::new(), Some(GROUP)).unwrap()
}

fn operation() -> Operation {
    Operation::new(10_000, CancellationToken::new()).unwrap()
}

fn entry() -> Entry {
    Entry {
        matches_scope: true,
        passcode_only: true,
        synchronizable: false,
        unsupported_access_control: false,
        persistent_ref: SecretBytes::new(REFERENCE.to_vec()),
    }
}

type Hook = Box<dyn FnMut(&str) -> Result<()> + Send>;

struct Fake {
    entries: Vec<Entry>,
    secret: Option<Vec<u8>>,
    calls: Vec<&'static str>,
    hook: Hook,
    expected_scope: Scope,
}

impl Fake {
    fn new(scope: &Scope, present: bool) -> Self {
        Self {
            entries: if present { vec![entry()] } else { Vec::new() },
            secret: present.then(|| vec![7, 0, 255]),
            calls: Vec::new(),
            hook: Box::new(|_| Ok(())),
            expected_scope: Scope {
                service: scope.service.clone(),
                account: scope.account.clone(),
                access_group: scope.access_group.clone(),
            },
        }
    }

    fn visit(&mut self, call: &'static str) -> Result<()> {
        self.calls.push(call);
        (self.hook)(call)
    }

    fn check_scope(&self, scope: &Scope) {
        assert_eq!(scope.service, self.expected_scope.service);
        assert_eq!(scope.account, self.expected_scope.account);
        assert_eq!(scope.access_group, self.expected_scope.access_group);
    }
}

impl Backend for Fake {
    fn find(&mut self, scope: &Scope) -> Result<Vec<Entry>> {
        self.check_scope(scope);
        self.visit("find")?;
        Ok(std::mem::take(&mut self.entries))
    }

    fn read(&mut self, scope: &Scope, entry: &Entry) -> Result<Option<SecretBytes>> {
        self.check_scope(scope);
        assert_eq!(entry.persistent_ref.expose(), REFERENCE);
        self.visit("read")?;
        Ok(self.secret.clone().map(SecretBytes::new))
    }

    fn change(
        &mut self,
        scope: &Scope,
        change: Change<'_>,
        operation: &Operation,
        dispatched: &mut bool,
    ) -> Result<()> {
        self.check_scope(scope);
        self.visit("prepare")?;
        before_dispatch(operation)?;
        *dispatched = true;
        match change {
            Change::Add(secret) => {
                self.visit("add")?;
                assert!(self.secret.is_none());
                self.secret = Some(secret.expose().to_vec());
            }
            Change::Update(entry, secret) => {
                assert_eq!(entry.persistent_ref.expose(), REFERENCE);
                self.visit("update")?;
                self.secret = Some(secret.expose().to_vec());
            }
            Change::Delete(entry) => {
                assert_eq!(entry.persistent_ref.expose(), REFERENCE);
                self.visit("delete")?;
                self.secret = None;
            }
        }
        self.visit("return")
    }
}

#[test]
fn namespace_is_explicit_and_isolates_profiles_and_references() {
    let profile = Uuid::new_v4();
    let reference = CredentialRef::new();
    let first = Scope::new(profile, &reference, Some(GROUP)).unwrap();
    let same = Scope::new(profile, &reference, Some(GROUP)).unwrap();
    assert_eq!(first.service, same.service);
    assert_eq!(first.account, same.account);
    assert_eq!(first.access_group, GROUP);
    assert_ne!(
        first.service,
        Scope::new(Uuid::new_v4(), &reference, Some(GROUP))
            .unwrap()
            .service
    );
    assert_ne!(
        first.account,
        Scope::new(profile, &CredentialRef::new(), Some(GROUP))
            .unwrap()
            .account
    );
    for group in [
        None,
        Some(""),
        Some("group"),
        Some(".bad"),
        Some("bad."),
        Some("a..b"),
        Some("a.*"),
        Some("a. b"),
        Some("a.\0b"),
    ] {
        assert_eq!(
            Scope::new(profile, &reference, group).err().unwrap().code,
            "SECRET_STORE_UNAVAILABLE"
        );
    }
    assert!(Scope::new(profile, &reference, Some(&format!("a.{}", "b".repeat(254)))).is_err());
}

#[tokio::test]
async fn missing_reads_and_deletes_never_dispatch_and_add_is_single_shot() {
    let scope = scope();
    for request in [Request::Get, Request::Delete] {
        let mut backend = Fake::new(&scope, false);
        assert!(
            operate(&mut backend, &scope, request, &operation())
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(backend.calls, ["find"]);
    }
    let mut backend = Fake::new(&scope, false);
    let secret = SecretBytes::new(vec![1, 2, 0, 255]);
    operate(&mut backend, &scope, Request::Put(&secret), &operation())
        .await
        .unwrap();
    assert_eq!(backend.secret.as_deref(), Some(secret.expose()));
    assert_eq!(backend.calls, ["find", "prepare", "add", "return"]);
}

#[tokio::test]
async fn reads_replacements_and_deletes_use_the_exact_reference_without_recreation() {
    let scope = scope();
    let mut backend = Fake::new(&scope, true);
    let read = operate(&mut backend, &scope, Request::Get, &operation())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(read.expose(), &[7, 0, 255]);
    assert_eq!(backend.calls, ["find", "read"]);
    let replacement = SecretBytes::new(vec![31, 41]);
    let mut backend = Fake::new(&scope, true);
    operate(
        &mut backend,
        &scope,
        Request::Put(&replacement),
        &operation(),
    )
    .await
    .unwrap();
    assert_eq!(backend.secret.as_deref(), Some(replacement.expose()));
    assert_eq!(backend.calls, ["find", "prepare", "update", "return"]);
    let mut backend = Fake::new(&scope, true);
    backend.secret = None;
    operate(&mut backend, &scope, Request::Delete, &operation())
        .await
        .unwrap();
    assert_eq!(backend.calls, ["find", "prepare", "delete", "return"]);
}

#[tokio::test]
async fn ambiguous_foreign_or_weakened_entries_never_read_or_mutate() {
    let scope = scope();
    let secret = SecretBytes::new(vec![3]);
    for variant in 0..7 {
        for request in [Request::Get, Request::Put(&secret), Request::Delete] {
            let mut backend = Fake::new(&scope, true);
            let item = &mut backend.entries[0];
            match variant {
                0 => backend.entries.push(entry()),
                1 => item.matches_scope = false,
                2 => item.passcode_only = false,
                3 => item.synchronizable = true,
                4 => item.unsupported_access_control = true,
                5 => item.persistent_ref = SecretBytes::new(Vec::new()),
                _ => item.persistent_ref = SecretBytes::new(vec![7; MAX_REFERENCE + 1]),
            }
            let error = operate(&mut backend, &scope, request, &operation())
                .await
                .unwrap_err();
            assert_eq!(
                error.code,
                if variant == 0 {
                    "SECRET_STORE_CONFLICT"
                } else {
                    "SECRET_STORE_UNAVAILABLE"
                }
            );
            assert_eq!(backend.calls, ["find"]);
        }
    }
}

#[tokio::test(start_paused = true)]
async fn cancellation_and_expiry_before_access_make_no_os_calls() {
    let scope = scope();
    let secret = SecretBytes::new(vec![3]);
    for expired in [false, true] {
        let op = operation();
        if expired {
            tokio::time::advance(std::time::Duration::from_secs(11)).await;
        } else {
            op.cancellation.cancel();
        }
        for request in [Request::Get, Request::Put(&secret), Request::Delete] {
            let mut backend = Fake::new(&scope, true);
            let error = operate(&mut backend, &scope, request, &op)
                .await
                .unwrap_err();
            assert_eq!(
                error.code,
                if expired {
                    "DEADLINE_EXCEEDED"
                } else {
                    "CANCELLED"
                }
            );
            assert!(backend.calls.is_empty());
        }
    }
}

#[tokio::test]
async fn cancellation_during_lookup_or_preparation_prevents_mutation() {
    let scope = scope();
    let secret = SecretBytes::new(vec![3]);
    for point in ["find", "prepare"] {
        let op = operation();
        let token = op.cancellation.clone();
        let mut backend = Fake::new(&scope, true);
        backend.hook = Box::new(move |call| {
            if call == point {
                token.cancel();
            }
            Ok(())
        });
        let error = operate(&mut backend, &scope, Request::Put(&secret), &op)
            .await
            .unwrap_err();
        assert_eq!(error.code, "CANCELLED");
        assert_eq!(backend.secret, Some(vec![7, 0, 255]));
        assert!(!backend.calls.contains(&"update"));
    }
}

#[tokio::test]
async fn interrupted_completed_mutations_remain_unknown_without_retry() {
    let scope = scope();
    let secret = SecretBytes::new(vec![42]);
    for request in [Request::Put(&secret), Request::Delete] {
        let op = operation();
        let token = op.cancellation.clone();
        let mut backend = Fake::new(&scope, true);
        backend.hook = Box::new(move |call| {
            if call == "return" {
                token.cancel();
            }
            Ok(())
        });
        let error = operate(&mut backend, &scope, request, &op)
            .await
            .unwrap_err();
        assert_eq!(error.code, "SECRET_STORE_WRITE_UNKNOWN");
        assert!(!error.retryable);
        assert_eq!(backend.calls.len(), 4);
        assert_ne!(backend.secret, Some(vec![7, 0, 255]));
    }
}

#[tokio::test]
async fn deadline_during_os_call_distinguishes_lookup_from_dispatched_write() {
    let scope = scope();
    let secret = SecretBytes::new(vec![42]);
    for point in ["find", "return"] {
        let op = Operation::new(100, CancellationToken::new()).unwrap();
        let mut backend = Fake::new(&scope, true);
        backend.hook = Box::new(move |call| {
            if call == point {
                std::thread::sleep(std::time::Duration::from_millis(120));
            }
            Ok(())
        });
        let error = operate(&mut backend, &scope, Request::Put(&secret), &op)
            .await
            .unwrap_err();
        assert_eq!(
            error.code,
            if point == "find" {
                "DEADLINE_EXCEEDED"
            } else {
                "SECRET_STORE_WRITE_UNKNOWN"
            }
        );
        assert_eq!(backend.calls.len(), if point == "find" { 1 } else { 4 });
    }
}

#[tokio::test]
async fn lookup_and_dispatch_errors_are_redacted_and_never_retried() {
    let scope = scope();
    let secret = SecretBytes::new(vec![42]);
    for code in [-25308, -25293, -25295, -25299, -34018, -1] {
        for point in ["find", "prepare", "update", "return"] {
            let mut backend = Fake::new(&scope, true);
            backend.hook = Box::new(move |call| {
                if call == point {
                    Err(status_error(code))
                } else {
                    Ok(())
                }
            });
            let error = operate(&mut backend, &scope, Request::Put(&secret), &operation())
                .await
                .unwrap_err();
            let mapped = status_error(code);
            assert_eq!(
                error.code,
                if matches!(point, "update" | "return") {
                    "SECRET_STORE_WRITE_UNKNOWN"
                } else {
                    mapped.code.as_str()
                }
            );
            assert!(!error.retryable);
            assert!(!backend.calls.contains(&"add"));
            assert!(!backend.calls.contains(&"delete"));
        }
    }
}

#[tokio::test]
async fn duplicate_add_race_is_uncertain_without_an_update_fallback() {
    let scope = scope();
    let secret = SecretBytes::new(vec![42]);
    let mut backend = Fake::new(&scope, false);
    backend.hook = Box::new(|call| {
        if call == "add" {
            Err(status_error(-25299))
        } else {
            Ok(())
        }
    });
    assert_eq!(
        operate(&mut backend, &scope, Request::Put(&secret), &operation())
            .await
            .unwrap_err()
            .code,
        "SECRET_STORE_WRITE_UNKNOWN"
    );
    assert_eq!(backend.calls, ["find", "prepare", "add"]);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancelled_blocking_call_retains_caller_lifetime_until_os_return() {
    use std::sync::{
        Arc, Barrier,
        atomic::{AtomicBool, Ordering},
    };

    struct CallerGuard(Arc<AtomicBool>);
    impl Drop for CallerGuard {
        fn drop(&mut self) {
            self.0.store(false, Ordering::SeqCst);
        }
    }

    let held = Arc::new(AtomicBool::new(true));
    let release = Arc::new(Barrier::new(2));
    let entered = Arc::new(tokio::sync::Notify::new());
    let op = operation();
    let cancellation = op.cancellation.clone();
    let guard = CallerGuard(held.clone());
    let native_release = release.clone();
    let native_entered = entered.clone();
    let task = tokio::spawn(async move {
        let _guard = guard;
        let scope = scope();
        let mut backend = Fake::new(&scope, true);
        backend.hook = Box::new(move |call| {
            if call == "update" {
                native_entered.notify_one();
                native_release.wait();
            }
            Ok(())
        });
        let secret = SecretBytes::new(vec![42]);
        let result = operate(&mut backend, &scope, Request::Put(&secret), &op).await;
        (result.unwrap_err().code, backend.secret)
    });
    entered.notified().await;
    cancellation.cancel();
    assert!(held.load(Ordering::SeqCst));
    assert!(!task.is_finished());
    release.wait();
    let (code, value) = task.await.unwrap();
    assert_eq!(code, "SECRET_STORE_WRITE_UNKNOWN");
    assert_eq!(value, Some(vec![42]));
    assert!(!held.load(Ordering::SeqCst));
}

#[tokio::test]
async fn interrupted_or_oversized_reads_return_no_secret_and_oversized_puts_never_lookup() {
    let scope = scope();
    let op = operation();
    let token = op.cancellation.clone();
    let mut backend = Fake::new(&scope, true);
    backend.hook = Box::new(move |call| {
        if call == "read" {
            token.cancel();
        }
        Ok(())
    });
    assert_eq!(
        operate(&mut backend, &scope, Request::Get, &op)
            .await
            .unwrap_err()
            .code,
        "CANCELLED"
    );
    let mut backend = Fake::new(&scope, true);
    backend.secret = Some(vec![7; MAX_BYTES + 1]);
    assert_eq!(
        operate(&mut backend, &scope, Request::Get, &operation())
            .await
            .unwrap_err()
            .code,
        "SECRET_STORE_UNAVAILABLE"
    );
    let mut backend = Fake::new(&scope, true);
    let secret = SecretBytes::new(vec![7; MAX_BYTES + 1]);
    assert_eq!(
        operate(&mut backend, &scope, Request::Put(&secret), &operation())
            .await
            .unwrap_err()
            .code,
        "INVALID_ARGUMENT"
    );
    assert!(backend.calls.is_empty());
}
