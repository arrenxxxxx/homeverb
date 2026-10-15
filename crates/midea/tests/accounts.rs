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
    assert_eq!(fixture.puts(), 2);
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
    for kind in 0..16 {
        let mut data: Value = serde_json::from_slice(&original).unwrap();
        match kind {
            0 => data["version"] = json!(2),
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
            _ => data["bindings"][0]["model"] = json!("Invalid\nModel"),
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
