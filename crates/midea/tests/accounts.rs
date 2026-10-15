use async_trait::async_trait;
use homeverb_core::*;
use homeverb_domain::{Connection, Descriptor, Error, PreferencesInput, Result, Transport};
use homeverb_midea::*;
use homeverb_storage::SqliteStore;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU8, AtomicUsize, Ordering},
    },
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

struct Directory(PathBuf);
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[derive(Default)]
struct Secrets {
    values: Mutex<HashMap<String, Vec<u8>>>,
    puts: AtomicUsize,
    mode: AtomicU8,
}

#[async_trait]
impl SecretStore for Secrets {
    async fn get(
        &self,
        reference: &CredentialRef,
        operation: &Operation,
    ) -> Result<Option<SecretBytes>> {
        operation
            .run(async {
                Ok(self
                    .values
                    .lock()
                    .unwrap()
                    .get(&reference.to_string())
                    .map(|value| SecretBytes::new(value.clone())))
            })
            .await
    }
    async fn put(
        &self,
        reference: &CredentialRef,
        secret: &SecretBytes,
        operation: &Operation,
    ) -> Result<()> {
        operation
            .run(async {
                self.puts.fetch_add(1, Ordering::Relaxed);
                let mode = self.mode.load(Ordering::Relaxed);
                if mode == 1 {
                    return Err(Error::new(
                        "SECRET_STORE_UNAVAILABLE",
                        "Synthetic unavailable store.",
                    ));
                }
                self.values
                    .lock()
                    .unwrap()
                    .insert(reference.to_string(), secret.expose().to_vec());
                if mode == 3 {
                    operation.cancellation.cancel();
                }
                if mode >= 2 {
                    return Err(Error::new(
                        "SECRET_STORE_WRITE_UNKNOWN",
                        "Synthetic uncertain store.",
                    ));
                }
                Ok(())
            })
            .await
    }
    async fn delete(&self, _: &CredentialRef, _: &Operation) -> Result<()> {
        panic!("Identity storage must never delete secrets implicitly.")
    }
}

struct Fixture {
    vault: AccountVault,
    store: Arc<SqliteStore>,
    secrets: Arc<Secrets>,
    directory: Directory,
}
impl Fixture {
    fn new() -> Self {
        let directory = Directory(
            std::env::temp_dir().join(format!("homeverb-meiju-account-{}", Uuid::new_v4())),
        );
        let store = Arc::new(SqliteStore::open(&directory.0).unwrap());
        let secrets = Arc::new(Secrets::default());
        let vault = AccountVault::new(store.clone(), secrets.clone());
        Self {
            vault,
            store,
            secrets,
            directory,
        }
    }
    async fn account(&self) -> MeijuAccount {
        MeijuAccount::create(
            &self.vault,
            "CN",
            "Home",
            &secret("PRIVATE-SUBJECT"),
            &secret("PRIVATE-SESSION"),
            &operation(),
        )
        .await
        .unwrap()
    }
    async fn reopen(&self, id: Uuid) -> Result<MeijuAccount> {
        MeijuAccount::open(self.vault.lease(id, &operation()).await?, &operation()).await
    }
    fn puts(&self) -> usize {
        self.secrets.puts.load(Ordering::Relaxed)
    }
    fn bytes(&self, reference: CredentialRef) -> Vec<u8> {
        self.secrets.values.lock().unwrap()[&reference.to_string()].clone()
    }
    fn set_bytes(&self, reference: CredentialRef, bytes: Vec<u8>) {
        self.secrets
            .values
            .lock()
            .unwrap()
            .insert(reference.to_string(), bytes);
    }
}

fn operation() -> Operation {
    Operation::new(1000, CancellationToken::new()).unwrap()
}
fn secret(value: &str) -> SecretBytes {
    SecretBytes::new(value.as_bytes().to_vec())
}

struct InventoryTransport(Vec<Value>);
#[async_trait]
impl ReadTransport for InventoryTransport {
    async fn request(
        &self,
        _: &CredentialRef,
        endpoint: ReadEndpoint<'_>,
        _: &Operation,
    ) -> Result<SecretBytes> {
        let data = match endpoint {
            ReadEndpoint::Homes => json!({"homeList": [{"homegroupId": "19"}]}),
            ReadEndpoint::HomeAppliances(_) => {
                json!({"homeList": [{"roomList": [{"applianceList": self.0}]}]})
            }
            ReadEndpoint::ApplianceState { .. } => {
                panic!("Binding must not request appliance state.")
            }
        };
        Ok(SecretBytes::new(
            serde_json::to_vec(&json!({"code": 0, "data": data})).unwrap(),
        ))
    }
}
fn appliance(id: &str, family: &str) -> Value {
    json!({"applianceCode": id, "type": family, "productModel": "Model-A", "name": "PRIVATE-NAME"})
}
async fn inventory(reference: CredentialRef, entries: Vec<Value>) -> Inventory {
    retrieve_inventory(&InventoryTransport(entries), reference, &operation())
        .await
        .unwrap()
}
fn descriptor(binding: DeviceBinding, account_id: Uuid) -> Descriptor {
    let mut device: homeverb_domain::Device =
        serde_json::from_str(include_str!("../../../examples/device.json")).unwrap();
    device.id = binding.device_id.to_string();
    device.connections = vec![Connection {
        id: binding.connection_id.to_string(),
        adapter: "midea.meiju".into(),
        transport: Transport::Cloud,
        account_id: Some(account_id.to_string()),
        identity_ref: binding.identity_ref.to_string(),
    }];
    Descriptor {
        device,
        capabilities: Vec::new(),
    }
}

#[tokio::test]
async fn bindings_survive_reopen_reauthorization_and_rediscovery_with_local_preferences() {
    let fixture = Fixture::new();
    let mut account = fixture.account().await;
    let record = account.record().clone();
    let listing = inventory(
        record.credential_ref,
        vec![
            appliance("90123456789", "AC"),
            appliance("22", "E1"),
            appliance("33", "CA"),
            appliance("44", "34"),
        ],
    )
    .await;
    let bindings = account
        .bind_inventory(&listing, &operation())
        .await
        .unwrap();
    assert_eq!(bindings.len(), 4);
    let original = descriptor(bindings[0], record.id);
    fixture.store.save_device(&original).unwrap();
    let room = fixture.store.create_room("room-test", "Kitchen").unwrap();
    let preferences = PreferencesInput {
        alias: Some("My AC".into()),
        room_id: Some(room.id),
        favorite: true,
    };
    fixture
        .store
        .set_preferences(&original.device.id, &preferences)
        .unwrap();
    let saved_preferences = fixture.store.preferences().unwrap();
    drop(account);

    let store = Arc::new(SqliteStore::open(&fixture.directory.0).unwrap());
    let vault = AccountVault::new(store.clone(), fixture.secrets.clone());
    let mut account = MeijuAccount::open(
        vault.lease(record.id, &operation()).await.unwrap(),
        &operation(),
    )
    .await
    .unwrap();
    account
        .replace_session(
            &secret("PRIVATE-SUBJECT"),
            &secret("ROTATED-SESSION"),
            &operation(),
        )
        .await
        .unwrap();
    assert_eq!(
        account.session(&operation()).await.unwrap().expose(),
        b"ROTATED-SESSION"
    );
    assert_eq!(
        account
            .bind_inventory(&listing, &operation())
            .await
            .unwrap(),
        bindings
    );
    assert_eq!(fixture.puts(), 3);
    let mut updated = descriptor(bindings[0], record.id);
    updated.device.alias = "Renamed upstream".into();
    store.save_device(&updated).unwrap();
    assert_eq!(store.preferences().unwrap(), saved_preferences);
    assert_eq!(store.devices().unwrap().len(), 1);
    assert_eq!(
        account
            .resolve_device(bindings[0].device_id, &operation())
            .await
            .unwrap()
            .expose(),
        "90123456789"
    );
    assert_eq!(format!("{account:?}"), "MeijuAccount([REDACTED])");
    for filename in ["homeverb.sqlite3", "homeverb.sqlite3-wal"] {
        if let Ok(bytes) = std::fs::read(fixture.directory.0.join(filename)) {
            for marker in [
                "90123456789".to_owned(),
                "PRIVATE".to_owned(),
                serde_json::to_string(&b"PRIVATE-SUBJECT".to_vec()).unwrap(),
                serde_json::to_string(&b"ROTATED-SESSION".to_vec()).unwrap(),
            ] {
                assert!(
                    !bytes
                        .windows(marker.len())
                        .any(|part| part == marker.as_bytes())
                );
            }
        }
    }
}

#[tokio::test]
async fn partial_and_empty_lists_keep_older_bindings_and_ignore_names_and_online_hints() {
    let fixture = Fixture::new();
    let mut account = fixture.account().await;
    let reference = account.record().credential_ref;
    let initial = inventory(
        reference,
        vec![appliance("11", "AC"), appliance("22", "CA")],
    )
    .await;
    let bindings = account
        .bind_inventory(&initial, &operation())
        .await
        .unwrap();
    let mut partial = inventory(reference, vec![appliance("00022", "CA")]).await;
    partial.failures.push(HomeFailure {
        home_id: initial.appliances[0].id.clone(),
        error: Error::new("MIDEA_REQUEST_FAILED", "Synthetic missing home."),
    });
    partial.appliances[0].name = Some("Changed name".into());
    partial.appliances[0].online_hint = Some(false);
    partial.appliances[0].model = None;
    assert_eq!(
        account
            .bind_inventory(&partial, &operation())
            .await
            .unwrap(),
        vec![bindings[1]]
    );
    let empty = inventory(reference, vec![]).await;
    assert!(
        account
            .bind_inventory(&empty, &operation())
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        account
            .resolve_device(bindings[0].device_id, &operation())
            .await
            .unwrap(),
        initial.appliances[0].id
    );
    assert_eq!(
        account
            .bind_inventory(&initial, &operation())
            .await
            .unwrap(),
        bindings
    );
    assert_eq!(fixture.puts(), 4);
}

#[tokio::test]
async fn identity_conflicts_reject_the_whole_update_before_any_write() {
    let fixture = Fixture::new();
    let mut account = fixture.account().await;
    let reference = account.record().credential_ref;
    let listing = inventory(reference, vec![appliance("11", "AC")]).await;
    let binding = account
        .bind_inventory(&listing, &operation())
        .await
        .unwrap();
    let original = fixture.bytes(reference);
    for kind in 0..4 {
        let mut conflicting = inventory(
            reference,
            vec![appliance("99", "E1"), appliance("11", "AC")],
        )
        .await;
        match kind {
            0 => conflicting.appliances[1].model = Some("Other model".into()),
            1 => conflicting.appliances[1].family = ApplianceFamily::Refrigerator,
            2 => conflicting.account_ref = CredentialRef::new(),
            _ => conflicting.appliances[1].id = conflicting.appliances[0].id.clone(),
        }
        let error = account
            .bind_inventory(&conflicting, &operation())
            .await
            .unwrap_err();
        assert_eq!(error.code, "DEVICE_IDENTITY_CONFLICT");
        assert!(!error.message.contains("Other model"));
        assert_eq!(fixture.bytes(reference), original);
    }
    assert_eq!(
        account
            .replace_session(
                &secret("OTHER-SUBJECT"),
                &secret("OTHER-SESSION"),
                &operation()
            )
            .await
            .unwrap_err()
            .code,
        "ACCOUNT_IDENTITY_CONFLICT"
    );
    assert_eq!(fixture.puts(), 2);
    assert_eq!(fixture.bytes(reference), original);
    assert_eq!(
        account
            .bind_inventory(&listing, &operation())
            .await
            .unwrap(),
        binding
    );
}

#[tokio::test]
async fn partial_inventory_keeps_membership_and_new_bindings_unconfirmed() {
    let fixture = Fixture::new();
    let mut account = fixture.account().await;
    let reference = account.record().credential_ref;
    let initial = inventory(
        reference,
        vec![appliance("11", "AC"), appliance("22", "CA")],
    )
    .await;
    account
        .bind_inventory(&initial, &operation())
        .await
        .unwrap();
    let subset = inventory(reference, vec![appliance("11", "AC")]).await;
    account.bind_inventory(&subset, &operation()).await.unwrap();
    let mut partial = inventory(
        reference,
        vec![appliance("22", "CA"), appliance("33", "E1")],
    )
    .await;
    partial.failures.push(HomeFailure {
        home_id: initial.appliances[0].id.clone(),
        error: Error::new("MIDEA_REQUEST_FAILED", "Synthetic missing home."),
    });
    account
        .bind_inventory(&partial, &operation())
        .await
        .unwrap();
    let payload: Value = serde_json::from_slice(&fixture.bytes(reference)).unwrap();
    let membership: Vec<_> = payload["bindings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["listed"].clone())
        .collect();
    assert_eq!(membership, vec![json!(true), json!(false), Value::Null]);
    let before = fixture.bytes(reference);
    account
        .replace_session(&secret("PRIVATE-SUBJECT"), &secret("ROTATED"), &operation())
        .await
        .unwrap();
    let updated: Value = serde_json::from_slice(&fixture.bytes(reference)).unwrap();
    assert_eq!(updated["bindings"], payload["bindings"]);
    assert_ne!(fixture.bytes(reference), before);
}

#[tokio::test]
async fn uncertain_membership_removal_blocks_account_reuse_without_losing_identity() {
    let fixture = Fixture::new();
    let mut account = fixture.account().await;
    let record = account.record().clone();
    let initial = inventory(record.credential_ref, vec![appliance("11", "AC")]).await;
    let ids = account
        .bind_inventory(&initial, &operation())
        .await
        .unwrap();
    let empty = inventory(record.credential_ref, vec![]).await;
    fixture.secrets.mode.store(2, Ordering::Relaxed);
    assert_eq!(
        account
            .bind_inventory(&empty, &operation())
            .await
            .unwrap_err()
            .code,
        "SECRET_STORE_WRITE_UNKNOWN"
    );
    assert_eq!(account.record().credential_state, CredentialState::Pending);
    drop(account);
    assert_eq!(
        fixture.reopen(record.id).await.unwrap_err().code,
        "ACCOUNT_SESSION_UNCONFIRMED"
    );
    let payload: Value = serde_json::from_slice(&fixture.bytes(record.credential_ref)).unwrap();
    assert_eq!(payload["bindings"][0]["listed"], false);
    assert_eq!(payload["bindings"][0]["local"], json!(ids[0]));
}

#[tokio::test]
async fn missing_model_can_be_learned_but_never_replaced_silently() {
    let fixture = Fixture::new();
    let mut account = fixture.account().await;
    let mut listing = inventory(account.record().credential_ref, vec![appliance("11", "AC")]).await;
    listing.appliances[0].model = None;
    let original = account
        .bind_inventory(&listing, &operation())
        .await
        .unwrap();
    listing.appliances[0].model = Some("Known model".into());
    assert_eq!(
        account
            .bind_inventory(&listing, &operation())
            .await
            .unwrap(),
        original
    );
    assert_eq!(fixture.puts(), 3);
    listing.appliances[0].model = Some("Replacement model".into());
    assert_eq!(
        account
            .bind_inventory(&listing, &operation())
            .await
            .unwrap_err()
            .code,
        "DEVICE_IDENTITY_CONFLICT"
    );
    assert_eq!(fixture.puts(), 3);
}

#[tokio::test]
async fn accounts_are_isolated_and_copied_payloads_cannot_rebind_metadata() {
    let fixture = Fixture::new();
    let mut first = fixture.account().await;
    let mut second = MeijuAccount::create(
        &fixture.vault,
        "OTHER",
        "Other",
        &secret("OTHER-SUBJECT"),
        &secret("OTHER-SESSION"),
        &operation(),
    )
    .await
    .unwrap();
    let first_reference = first.record().credential_ref;
    let second_record = second.record().clone();
    let first_listing = inventory(first_reference, vec![appliance("11", "AC")]).await;
    let second_listing = inventory(second_record.credential_ref, vec![appliance("11", "AC")]).await;
    let a = first
        .bind_inventory(&first_listing, &operation())
        .await
        .unwrap()[0];
    let b = second
        .bind_inventory(&second_listing, &operation())
        .await
        .unwrap()[0];
    assert_ne!(a, b);
    assert_eq!(
        second
            .resolve_device(a.device_id, &operation())
            .await
            .unwrap_err()
            .code,
        "DEVICE_NOT_FOUND"
    );
    assert_eq!(
        fixture
            .vault
            .lease(first.record().id, &operation())
            .await
            .err()
            .unwrap()
            .code,
        "RESOURCE_BUSY"
    );
    fixture.set_bytes(second_record.credential_ref, fixture.bytes(first_reference));
    drop(second);
    assert_eq!(
        fixture.reopen(second_record.id).await.unwrap_err().code,
        "ACCOUNT_SESSION_UNCONFIRMED"
    );
}

#[tokio::test]
async fn malformed_or_missing_protected_state_never_creates_a_replacement_map() {
    let fixture = Fixture::new();
    let mut account = fixture.account().await;
    let record = account.record().clone();
    let listing = inventory(
        record.credential_ref,
        vec![appliance("11", "AC"), appliance("22", "CA")],
    )
    .await;
    account
        .bind_inventory(&listing, &operation())
        .await
        .unwrap();
    let original = fixture.bytes(record.credential_ref);
    drop(account);
    for kind in 0..20 {
        let mut data: Value = serde_json::from_slice(&original).unwrap();
        match kind {
            0 => data["version"] = json!(3),
            1 => data["account_id"] = json!(Uuid::new_v4()),
            2 => data["credential_ref"] = json!(CredentialRef::new()),
            3 => data["provider"] = json!("another.provider"),
            4 => data["region"] = json!("OTHER"),
            5 => data["subject"] = json!([]),
            6 => data["session"] = json!([]),
            7 => data["bindings"][1]["appliance_id"] = data["bindings"][0]["appliance_id"].clone(),
            8 => data["bindings"][1]["local"] = data["bindings"][0]["local"].clone(),
            9 => data["bindings"][0]["local"]["device_id"] = json!(Uuid::nil()),
            10 => data["bindings"][0]["appliance_id"] = json!("00011"),
            11 => data["extra"] = json!("PRIVATE-UNEXPECTED"),
            12 => data["subject"] = json!(vec![1; 1025]),
            13 => data["session"] = json!(vec![1; 65537]),
            14 => data["bindings"][0]["model"] = json!("x".repeat(257)),
            15 => data["bindings"][0]["model"] = json!("Invalid\nModel"),
            16 => data["bindings"][0]["listed"] = json!("true"),
            17 => data["bindings"][0]["listed"] = json!(1),
            18 => data["version"] = json!(1),
            _ => {
                data["version"] = json!(1);
                data["bindings"][0]["listed"] = json!(false);
            }
        }
        fixture.set_bytes(record.credential_ref, serde_json::to_vec(&data).unwrap());
        let error = fixture.reopen(record.id).await.unwrap_err();
        assert_eq!(error.code, "ACCOUNT_SESSION_UNCONFIRMED");
        assert!(!error.message.contains("PRIVATE"));
    }
    for bytes in [
        b"not-json PRIVATE".to_vec(),
        vec![b' '; MAX_RESPONSE_BYTES + 1],
    ] {
        fixture.set_bytes(record.credential_ref, bytes);
        assert_eq!(
            fixture.reopen(record.id).await.unwrap_err().code,
            "ACCOUNT_SESSION_UNCONFIRMED"
        );
    }
    fixture.secrets.values.lock().unwrap().clear();
    assert_eq!(
        fixture.reopen(record.id).await.unwrap_err().code,
        "AUTH_REQUIRED"
    );
    assert_eq!(fixture.puts(), 2);
    assert_eq!(fixture.store.accounts().unwrap().len(), 1);
    fixture.set_bytes(record.credential_ref, original);
    fixture.reopen(record.id).await.unwrap();
}

#[tokio::test]
async fn uncertain_mapping_updates_return_no_ids_and_recover_exact_saved_bindings_without_rewrite()
{
    for mode in [2, 3] {
        let fixture = Fixture::new();
        let mut account = fixture.account().await;
        let record = account.record().clone();
        let listing = inventory(record.credential_ref, vec![appliance("11", "AC")]).await;
        fixture.secrets.mode.store(mode, Ordering::Relaxed);
        assert_eq!(
            account
                .bind_inventory(&listing, &operation())
                .await
                .unwrap_err()
                .code,
            "SECRET_STORE_WRITE_UNKNOWN"
        );
        assert_eq!(
            account.session(&operation()).await.unwrap_err().code,
            "ACCOUNT_SESSION_UNCONFIRMED"
        );
        drop(account);
        assert_eq!(
            fixture.reopen(record.id).await.unwrap_err().code,
            "ACCOUNT_SESSION_UNCONFIRMED"
        );
        let mut lease = fixture.vault.lease(record.id, &operation()).await.unwrap();
        let bytes = lease.inspect_session(&operation()).await.unwrap().unwrap();
        let payload: Value = serde_json::from_slice(bytes.expose()).unwrap();
        let persisted: DeviceBinding =
            serde_json::from_value(payload["bindings"][0]["local"].clone()).unwrap();
        // This synthetic test stands in for future trusted validation of the exact stored payload.
        lease.confirm_session(&bytes, &operation()).await.unwrap();
        let mut account = MeijuAccount::open(lease, &operation()).await.unwrap();
        assert_eq!(
            account
                .bind_inventory(&listing, &operation())
                .await
                .unwrap(),
            vec![persisted]
        );
        assert_eq!(fixture.puts(), 2);
    }
}

#[tokio::test]
async fn unavailable_storage_leaves_pending_state_without_plaintext_fallback_or_generated_results()
{
    let fixture = Fixture::new();
    let mut account = fixture.account().await;
    let record = account.record().clone();
    let original = fixture.bytes(record.credential_ref);
    let listing = inventory(record.credential_ref, vec![appliance("11", "AC")]).await;
    fixture.secrets.mode.store(1, Ordering::Relaxed);
    assert_eq!(
        account
            .bind_inventory(&listing, &operation())
            .await
            .unwrap_err()
            .code,
        "SECRET_STORE_UNAVAILABLE"
    );
    assert_eq!(fixture.bytes(record.credential_ref), original);
    assert_eq!(
        fixture.store.account(record.id).unwrap().credential_state,
        CredentialState::Pending
    );
    assert_eq!(
        account
            .bind_inventory(&listing, &operation())
            .await
            .unwrap_err()
            .code,
        "ACCOUNT_SESSION_UNCONFIRMED"
    );
    assert!(fixture.store.devices().unwrap().is_empty());
    assert_eq!(fixture.puts(), 2);
    let unavailable = AccountVault::new(fixture.store.clone(), Arc::new(UnavailableSecretStore));
    assert_eq!(
        MeijuAccount::create(
            &unavailable,
            "CN",
            "Unavailable",
            &secret("SUBJECT"),
            &secret("SESSION"),
            &operation()
        )
        .await
        .unwrap_err()
        .code,
        "SECRET_STORE_UNAVAILABLE"
    );
}

#[tokio::test(start_paused = true)]
async fn cancelled_expired_and_oversized_inputs_never_start_a_mutation() {
    let fixture = Fixture::new();
    let mut account = fixture.account().await;
    let listing = inventory(account.record().credential_ref, vec![appliance("11", "AC")]).await;
    let cancelled = operation();
    cancelled.cancellation.cancel();
    assert_eq!(
        account
            .bind_inventory(&listing, &cancelled)
            .await
            .unwrap_err()
            .code,
        "CANCELLED"
    );
    let expired = operation();
    tokio::time::advance(std::time::Duration::from_secs(2)).await;
    assert_eq!(
        account
            .bind_inventory(&listing, &expired)
            .await
            .unwrap_err()
            .code,
        "DEADLINE_EXCEEDED"
    );
    assert_eq!(
        MeijuAccount::create(
            &fixture.vault,
            "CN",
            "Cancelled",
            &secret("SUBJECT"),
            &secret("SESSION"),
            &cancelled
        )
        .await
        .unwrap_err()
        .code,
        "CANCELLED"
    );
    for (subject, session) in [
        (secret(""), secret("SESSION")),
        (SecretBytes::new(vec![1; 1025]), secret("SESSION")),
        (secret("SUBJECT"), SecretBytes::new(vec![1; 65537])),
    ] {
        assert_eq!(
            MeijuAccount::create(
                &fixture.vault,
                "CN",
                "Invalid",
                &subject,
                &session,
                &operation()
            )
            .await
            .unwrap_err()
            .code,
            "INVALID_ARGUMENT"
        );
    }
    assert_eq!(
        account
            .replace_session(&secret("PRIVATE-SUBJECT"), &secret(""), &operation())
            .await
            .unwrap_err()
            .code,
        "INVALID_ARGUMENT"
    );
    assert_eq!(fixture.puts(), 1);
    assert_eq!(fixture.store.accounts().unwrap().len(), 1);
}

struct LoginTransport {
    replies: Mutex<std::collections::VecDeque<Value>>,
    calls: AtomicUsize,
}
impl LoginTransport {
    fn new(subject: &str, homes: Value) -> Self {
        let vector: Value = serde_json::from_str(include_str!("fixtures/login.json")).unwrap();
        Self {
            replies: Mutex::new(vec![
                json!({"code":0,"data":{"loginId":subject}}),
                json!({"code":0,"data":{"mdata":{"accessToken":"PRIVATE-LOGIN-TOKEN"},"key":vector["wrapped_key"]}}),
                homes,
                json!({"code":0,"data":{"homeList":[{"roomList":[{"applianceList":[appliance("90123456789","AC"),appliance("22","CA")]}]}]}}),
            ].into()),
            calls: AtomicUsize::new(0),
        }
    }
    fn complete(subject: &str) -> Self {
        Self::new(
            subject,
            json!({"code":0,"data":{"homeList":[{"homegroupId":"19"}]}}),
        )
    }
}
#[async_trait]
impl CloudTransport for LoginTransport {
    async fn send(&self, _: &SignedRequest, _: &Operation) -> Result<SecretBytes> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(SecretBytes::new(
            serde_json::to_vec(
                &self
                    .replies
                    .lock()
                    .unwrap()
                    .pop_front()
                    .expect("No extra cloud requests are allowed."),
            )
            .unwrap(),
        ))
    }
}
fn login_credentials() -> LoginCredentials {
    LoginCredentials::new(secret("fictional@example.test"), secret("PRIVATE-PASSWORD")).unwrap()
}

#[tokio::test]
async fn connect_automatically_retrieves_and_reuses_account_device_ids_after_reopening() {
    let f = Fixture::new();
    let report = connect(
        &f.vault,
        &LoginTransport::complete("PRIVATE-LOGIN-ID"),
        login_credentials(),
        None,
        "Home",
        &operation(),
    )
    .await
    .unwrap();
    let first = report.retrieval.unwrap();
    assert_eq!(first.bindings.len(), 2);
    assert!(first.inventory.is_complete());
    assert_eq!(report.account.credential_state, CredentialState::Stored);
    let descriptor = descriptor(first.bindings[0], report.account.id);
    f.store.save_device(&descriptor).unwrap();
    f.store
        .set_preferences(
            &descriptor.device.id,
            &PreferencesInput {
                alias: Some("My AC".into()),
                room_id: None,
                favorite: true,
            },
        )
        .unwrap();
    let preferences = f.store.preferences().unwrap();
    let vault = AccountVault::new(
        Arc::new(SqliteStore::open(&f.directory.0).unwrap()),
        f.secrets.clone(),
    );
    let second = connect(
        &vault,
        &LoginTransport::complete("PRIVATE-LOGIN-ID"),
        login_credentials(),
        None,
        "Ignored reconnect label",
        &operation(),
    )
    .await
    .unwrap();
    assert_eq!(second.account.id, report.account.id);
    assert_eq!(second.account.credential_ref, report.account.credential_ref);
    assert_eq!(second.account.label, "Home");
    assert_eq!(second.retrieval.unwrap().bindings, first.bindings);
    assert_eq!(f.store.accounts().unwrap().len(), 1);
    assert_eq!(f.store.preferences().unwrap(), preferences);
    let account = f.reopen(report.account.id).await.unwrap();
    assert!(account.cloud_session(&operation()).await.is_ok());
    let payload: Value = serde_json::from_slice(&f.bytes(report.account.credential_ref)).unwrap();
    let bytes: Vec<u8> = serde_json::from_value(payload["session"].clone()).unwrap();
    assert!(!String::from_utf8_lossy(&bytes).contains("PRIVATE-PASSWORD"));
    assert!(!String::from_utf8_lossy(&bytes).contains("fictional@example.test"));
    for filename in ["homeverb.sqlite3", "homeverb.sqlite3-wal"] {
        if let Ok(bytes) = std::fs::read(f.directory.0.join(filename)) {
            for marker in ["PRIVATE-LOGIN", "PRIVATE-PASSWORD", "90123456789"] {
                assert!(
                    !bytes
                        .windows(marker.len())
                        .any(|part| part == marker.as_bytes())
                );
            }
        }
    }
}

#[tokio::test]
async fn successful_login_and_failed_or_partial_inventory_are_separate_outcomes() {
    let f = Fixture::new();
    let cloud = LoginTransport::new("PRIVATE-LOGIN-ID", json!({"code":40002}));
    let report = connect(
        &f.vault,
        &cloud,
        login_credentials(),
        None,
        "Home",
        &operation(),
    )
    .await
    .unwrap();
    assert_eq!(report.account.credential_state, CredentialState::Stored);
    assert_eq!(report.retrieval.unwrap_err().code, "AUTH_REQUIRED");
    assert_eq!(cloud.calls.load(Ordering::Relaxed), 3);
    let mut account = f.reopen(report.account.id).await.unwrap();
    let cloud = LoginTransport::complete("unused");
    cloud.replies.lock().unwrap().drain(..2);
    let retry = account.retrieve(&cloud, &operation()).await.unwrap();
    assert_eq!(retry.bindings.len(), 2);
    assert_eq!(cloud.calls.load(Ordering::Relaxed), 2);
    drop(account);
    let partial = LoginTransport::complete("PRIVATE-LOGIN-ID");
    partial.replies.lock().unwrap()[3] = json!({"code":123});
    let report = connect(
        &f.vault,
        &partial,
        login_credentials(),
        Some(report.account.id),
        "Home",
        &operation(),
    )
    .await
    .unwrap();
    let listing = report.retrieval.unwrap();
    assert!(!listing.inventory.is_complete());
    assert!(listing.bindings.is_empty());
    let account = f.reopen(report.account.id).await.unwrap();
    assert_eq!(
        account
            .resolve_device(retry.bindings[0].device_id, &operation())
            .await
            .unwrap()
            .expose(),
        "90123456789"
    );
}

#[tokio::test]
async fn reconnect_refuses_changed_subject_without_overwriting_session_or_fetching_devices() {
    let f = Fixture::new();
    let first = connect(
        &f.vault,
        &LoginTransport::complete("PRIVATE-LOGIN-ID"),
        login_credentials(),
        None,
        "Home",
        &operation(),
    )
    .await
    .unwrap();
    let original = f.bytes(first.account.credential_ref);
    let puts = f.puts();
    let changed = LoginTransport::complete("OTHER-LOGIN-ID");
    assert_eq!(
        connect(
            &f.vault,
            &changed,
            login_credentials(),
            Some(first.account.id),
            "Home",
            &operation()
        )
        .await
        .unwrap_err()
        .code,
        "ACCOUNT_IDENTITY_CONFLICT"
    );
    assert_eq!(f.puts(), puts);
    assert_eq!(f.bytes(first.account.credential_ref), original);
    assert_eq!(changed.calls.load(Ordering::Relaxed), 2);
}

#[tokio::test]
async fn unavailable_storage_pending_state_and_registration_lock_prevent_login() {
    let f = Fixture::new();
    let cloud = LoginTransport::complete("PRIVATE-LOGIN-ID");
    let unavailable = AccountVault::new(f.store.clone(), Arc::new(UnavailableSecretStore));
    assert_eq!(
        connect(
            &unavailable,
            &cloud,
            login_credentials(),
            None,
            "Home",
            &operation()
        )
        .await
        .unwrap_err()
        .code,
        "SECRET_STORE_UNAVAILABLE"
    );
    assert!(f.store.accounts().unwrap().is_empty());
    let guard = f.vault.lock_registration(&operation()).await.unwrap();
    assert_eq!(
        connect(
            &f.vault,
            &cloud,
            login_credentials(),
            None,
            "Home",
            &operation()
        )
        .await
        .unwrap_err()
        .code,
        "RESOURCE_BUSY"
    );
    drop(guard);
    let lease = f
        .vault
        .prepare("midea.meiju", "CN", "Pending", &operation())
        .await
        .unwrap();
    drop(lease);
    assert_eq!(
        connect(
            &f.vault,
            &cloud,
            login_credentials(),
            None,
            "Home",
            &operation()
        )
        .await
        .unwrap_err()
        .code,
        "ACCOUNT_SESSION_UNCONFIRMED"
    );
    assert_eq!(cloud.calls.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn interrupted_session_persistence_prevents_inventory_and_leaves_recoverable_account() {
    let f = Fixture::new();
    f.secrets.mode.store(2, Ordering::Relaxed);
    let cloud = LoginTransport::complete("PRIVATE-LOGIN-ID");
    assert_eq!(
        connect(
            &f.vault,
            &cloud,
            login_credentials(),
            None,
            "Home",
            &operation()
        )
        .await
        .unwrap_err()
        .code,
        "SECRET_STORE_WRITE_UNKNOWN"
    );
    assert_eq!(cloud.calls.load(Ordering::Relaxed), 2);
    let records = f.store.accounts().unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].credential_state, CredentialState::Pending);
    assert_eq!(f.puts(), 1);
    let next = LoginTransport::complete("PRIVATE-LOGIN-ID");
    assert_eq!(
        connect(
            &f.vault,
            &next,
            login_credentials(),
            None,
            "Home",
            &operation()
        )
        .await
        .unwrap_err()
        .code,
        "ACCOUNT_SESSION_UNCONFIRMED"
    );
    assert_eq!(next.calls.load(Ordering::Relaxed), 0);
    assert!(f.vault.lock_registration(&operation()).await.is_ok());
}

#[tokio::test]
async fn mismatched_encoded_session_subject_never_reaches_cloud() {
    let f = Fixture::new();
    let report = connect(
        &f.vault,
        &LoginTransport::complete("PRIVATE-LOGIN-ID"),
        login_credentials(),
        None,
        "Home",
        &operation(),
    )
    .await
    .unwrap();
    let mut payload: Value =
        serde_json::from_slice(&f.bytes(report.account.credential_ref)).unwrap();
    payload["subject"] = json!(b"another-subject".to_vec());
    f.set_bytes(
        report.account.credential_ref,
        serde_json::to_vec(&payload).unwrap(),
    );
    let mut account = f.reopen(report.account.id).await.unwrap();
    let cloud = LoginTransport::complete("unused");
    assert_eq!(
        account
            .retrieve(&cloud, &operation())
            .await
            .unwrap_err()
            .code,
        "ACCOUNT_SESSION_UNCONFIRMED"
    );
    assert_eq!(cloud.calls.load(Ordering::Relaxed), 0);
}

struct WaitingCloud(AtomicUsize);
#[async_trait]
impl CloudTransport for WaitingCloud {
    async fn send(&self, _: &SignedRequest, _: &Operation) -> Result<SecretBytes> {
        self.0.fetch_add(1, Ordering::Relaxed);
        std::future::pending().await
    }
}
#[tokio::test]
async fn cancelled_connect_releases_registration_without_creating_an_account() {
    let f = Fixture::new();
    let cloud = WaitingCloud(AtomicUsize::new(0));
    let op = operation();
    let cancel = async {
        while cloud.0.load(Ordering::Relaxed) == 0 {
            tokio::task::yield_now().await;
        }
        op.cancellation.cancel();
    };
    let (result, ()) = tokio::join!(
        connect(&f.vault, &cloud, login_credentials(), None, "Home", &op),
        cancel
    );
    assert_eq!(result.unwrap_err().code, "CANCELLED");
    assert!(f.store.accounts().unwrap().is_empty());
    assert!(f.vault.lock_registration(&operation()).await.is_ok());
}

#[tokio::test]
async fn foreign_region_never_uses_the_mainland_endpoint() {
    let f = Fixture::new();
    let payload = json!({"version":1,"subject":"meiju.login-id.v1:test-id","token":"FICTIONAL","key":"synthetic-key-01"});
    let mut account = MeijuAccount::create(
        &f.vault,
        "EU",
        "Other",
        &secret("meiju.login-id.v1:test-id"),
        &SecretBytes::new(serde_json::to_vec(&payload).unwrap()),
        &operation(),
    )
    .await
    .unwrap();
    let cloud = LoginTransport::complete("test-id");
    assert_eq!(
        account
            .retrieve(&cloud, &operation())
            .await
            .unwrap_err()
            .code,
        "AUTH_FLOW_UNSUPPORTED"
    );
    assert_eq!(cloud.calls.load(Ordering::Relaxed), 0);
}
