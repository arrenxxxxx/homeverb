use async_trait::async_trait;
use homeverb_core::*;
use homeverb_domain::{Error, Result};
use homeverb_host::{ConnectRequest, HostRequest, MeijuPowerGrant, Session, meiju_power_policy};
use homeverb_midea::*;
use homeverb_storage::SqliteStore;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering},
    },
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

fn operation() -> Operation {
    Operation::new(1000, CancellationToken::new()).unwrap()
}
fn bytes(value: Value) -> SecretBytes {
    SecretBytes::new(serde_json::to_vec(&value).unwrap())
}
fn secret(value: &str) -> SecretBytes {
    SecretBytes::new(value.as_bytes().to_vec())
}

fn validate_response(value: &Value, definition: &str) {
    let root: Value =
        serde_json::from_str(include_str!("../../../schemas/native-response.schema.json")).unwrap();
    fn expand(value: &Value, root: &Value) -> Value {
        if let Some(reference) = value.get("$ref").and_then(Value::as_str) {
            let name = reference
                .strip_prefix("urn:homeverb:schema:native-response#/$defs/")
                .expect("Account responses use only local account definitions.");
            return expand(&root["$defs"][name], root);
        }
        match value {
            Value::Object(fields) => Value::Object(
                fields
                    .iter()
                    .map(|(key, value)| (key.clone(), expand(value, root)))
                    .collect(),
            ),
            Value::Array(values) => {
                Value::Array(values.iter().map(|value| expand(value, root)).collect())
            }
            _ => value.clone(),
        }
    }
    let schema = expand(&root["$defs"][definition], &root);
    jsonschema::validator_for(&schema)
        .unwrap()
        .validate(value)
        .unwrap();
}

#[derive(Default)]
struct Secrets {
    values: Mutex<HashMap<String, Vec<u8>>>,
    unavailable: AtomicBool,
    uncertain: AtomicBool,
    reads: AtomicUsize,
    deletes: AtomicUsize,
    deletion: AtomicI32,
    delete_started: tokio::sync::Notify,
}
#[async_trait]
impl SecretStore for Secrets {
    async fn get(&self, key: &CredentialRef, op: &Operation) -> Result<Option<SecretBytes>> {
        self.reads.fetch_add(1, Ordering::Relaxed);
        op.run(async {
            if self.unavailable.load(Ordering::Relaxed) {
                return Err(Error::new(
                    "SECRET_STORE_UNAVAILABLE",
                    "Synthetic unavailable store.",
                ));
            }
            Ok(self
                .values
                .lock()
                .unwrap()
                .get(&key.to_string())
                .map(|v| SecretBytes::new(v.clone())))
        })
        .await
    }
    async fn put(&self, key: &CredentialRef, value: &SecretBytes, op: &Operation) -> Result<()> {
        op.run(async {
            self.values
                .lock()
                .unwrap()
                .insert(key.to_string(), value.expose().to_vec());
            if self.uncertain.load(Ordering::Relaxed) {
                return Err(Error::new(
                    "SECRET_STORE_WRITE_UNKNOWN",
                    "Synthetic uncertain storage mutation.",
                ));
            }
            Ok(())
        })
        .await
    }
    async fn delete(&self, key: &CredentialRef, op: &Operation) -> Result<()> {
        let mode = self.deletion.load(Ordering::Relaxed);
        assert_ne!(mode, 0, "Connection must not delete credentials.");
        op.run(async { Ok(()) }).await?;
        self.deletes.fetch_add(1, Ordering::Relaxed);
        if mode == 3 {
            return Err(Error::new(
                "SECRET_STORE_UNAVAILABLE",
                "Synthetic unavailable store.",
            ));
        }
        self.values.lock().unwrap().remove(&key.to_string());
        if mode == 4 {
            self.delete_started.notify_one();
            let _: Result<()> = op.run(std::future::pending()).await;
        }
        if mode == 2 || mode == 4 {
            return Err(Error::new(
                "SECRET_STORE_WRITE_UNKNOWN",
                "Synthetic unconfirmed deletion.",
            ));
        }
        Ok(())
    }
}

struct Wire {
    subject: Mutex<String>,
    calls: Mutex<Vec<CloudEndpoint>>,
    inventory_code: AtomicI32,
    control_code: AtomicI32,
    homes_code: AtomicI32,
    extra_device: AtomicBool,
    delay: Mutex<Option<CloudEndpoint>>,
    started: tokio::sync::Notify,
}
impl Default for Wire {
    fn default() -> Self {
        Self {
            subject: Mutex::new("synthetic-login-001".into()),
            calls: Mutex::new(vec![]),
            inventory_code: AtomicI32::new(0),
            control_code: AtomicI32::new(0),
            homes_code: AtomicI32::new(0),
            extra_device: AtomicBool::new(false),
            delay: Mutex::new(None),
            started: tokio::sync::Notify::new(),
        }
    }
}
#[async_trait]
impl CloudTransport for Wire {
    async fn send_control(
        &self,
        request: &SignedRequest,
        progress: &WriteProgress,
        _: &Operation,
    ) -> Result<SecretBytes> {
        assert_eq!(request.endpoint(), CloudEndpoint::ApplianceControl);
        progress.mark_sent();
        self.calls.lock().unwrap().push(request.endpoint());
        if *self.delay.lock().unwrap() == Some(CloudEndpoint::ApplianceControl) {
            self.started.notify_one();
            std::future::pending::<()>().await;
        }
        Ok(bytes(
            json!({"code":self.control_code.load(Ordering::Relaxed)}),
        ))
    }

    async fn send(&self, request: &SignedRequest, _: &Operation) -> Result<SecretBytes> {
        self.calls.lock().unwrap().push(request.endpoint());
        let delay = *self.delay.lock().unwrap() == Some(request.endpoint());
        if delay {
            self.started.notify_one();
            std::future::pending::<()>().await;
        }
        let (code, mut data) = match request.endpoint() {
            CloudEndpoint::LoginId => (0, json!({"loginId":*self.subject.lock().unwrap()})),
            CloudEndpoint::Login => {
                let vector: Value =
                    serde_json::from_str(include_str!("../../midea/tests/fixtures/login.json"))
                        .unwrap();
                (
                    0,
                    json!({"mdata":{"accessToken":"SYNTHETIC-TOKEN"},"key":vector["wrapped_key"]}),
                )
            }
            CloudEndpoint::Homes => (
                self.homes_code.load(Ordering::Relaxed),
                json!({"homeList":[{"homegroupId":"1"}]}),
            ),
            CloudEndpoint::HomeAppliances => (
                self.inventory_code.load(Ordering::Relaxed),
                json!({"homeList":[{"roomList":[{"applianceList":[
                    {"applianceCode":"880000000000001","type":"AC","productModel":"KFR-35G/WF2","name":"Synthetic AC"},
                    {"applianceCode":"880000000000002","type":"AC","productModel":"Unknown model","name":"Synthetic unknown"}
                ]}]}]}),
            ),
            _ => panic!("Account connection must not request state or controls."),
        };
        if request.endpoint() == CloudEndpoint::HomeAppliances
            && self.extra_device.load(Ordering::Relaxed)
        {
            data["homeList"][0]["roomList"][0]["applianceList"].as_array_mut().unwrap().push(json!({"applianceCode":"880000000000003","type":"AC","productModel":"KFR-35G/WF2"}));
        }
        Ok(bytes(
            json!({"code":code,"data":data,"msg":"PRIVATE-PROVIDER-MESSAGE"}),
        ))
    }
}

struct Fixture {
    session: Arc<Session>,
    store: Arc<SqliteStore>,
    vault: Arc<AccountVault>,
    wire: Arc<Wire>,
    secrets: Arc<Secrets>,
    path: PathBuf,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("homeverb-account-host-{}", Uuid::new_v4()));
        let store = Arc::new(SqliteStore::open(&path).unwrap());
        let secrets = Arc::new(Secrets::default());
        let vault = Arc::new(AccountVault::new(store.clone(), secrets.clone()));
        let wire = Arc::new(Wire::default());
        let clock = Arc::new(SystemClock);
        let adapter = Arc::new(MeijuAdapter::new(
            vault.clone(),
            wire.clone(),
            clock.clone(),
        ));
        let core = Core::new(
            store.clone(),
            vec![adapter.clone()],
            clock,
            Policy::default(),
        );
        let session = Arc::new(Session::with_accounts(core, adapter));
        Self {
            session,
            store,
            vault,
            wire,
            secrets,
            path,
        }
    }
    fn reopen_with_power_grants(&mut self, grants: &[MeijuPowerGrant]) {
        self.session.close();
        let policy = meiju_power_policy(grants).unwrap();
        let mut adapter =
            MeijuAdapter::new(self.vault.clone(), self.wire.clone(), Arc::new(SystemClock));
        if !policy.grants.is_empty() {
            adapter = adapter.with_ac_power_control();
        }
        let adapter = Arc::new(adapter);
        let core = Core::new(
            self.store.clone(),
            vec![adapter.clone()],
            Arc::new(SystemClock),
            policy,
        );
        self.session = Arc::new(Session::with_accounts(core, adapter));
    }

    async fn call(&self, request: Value) -> Value {
        self.call_id("query", request).await
    }
    async fn call_id(&self, id: &str, request: Value) -> Value {
        let method = request["method"].as_str().unwrap().to_owned();
        let result = self.session.call(serde_json::from_value::<HostRequest>(json!({"schema_version":"1.0","operation_id":id,"timeout_ms":1000,"request":request})).unwrap()).await;
        if matches!(
            method.as_str(),
            "list_accounts"
                | "refresh_account"
                | "get_account"
                | "rename_account"
                | "remove_account"
        ) {
            validate_response(&result, &method);
        }
        result
    }
    async fn connect(&self, existing: Option<Uuid>) -> Value {
        let result = self
            .session
            .connect_account(
                ConnectRequest {
                    schema_version: "1.0".into(),
                    operation_id: "connect".into(),
                    timeout_ms: 1000,
                    label: "Test account".into(),
                    account_id: existing,
                },
                secret("owner@example.test"),
                secret("Synthetic password 42!"),
            )
            .await;
        validate_response(&result, "connect_account");
        result
    }
    async fn refresh(&self, id: &str) -> Value {
        self.call(json!({"method":"refresh_account","account_id":id}))
            .await
    }
    fn calls(&self) -> Vec<CloudEndpoint> {
        self.wire.calls.lock().unwrap().clone()
    }
}

#[tokio::test]
async fn local_account_details_and_rename_preserve_bindings_without_secret_access() {
    let f = Fixture::new();
    let connected = f.connect(None).await;
    let id = connected["data"]["account"]["id"].as_str().unwrap();
    let account_id = Uuid::parse_str(id).unwrap();
    let before_record = f.store.account(account_id).unwrap();
    let before_secrets = f.secrets.values.lock().unwrap().clone();
    let before_calls = f.calls();
    let cards = f.call(json!({"method":"list_device_cards"})).await;
    f.secrets.unavailable.store(true, Ordering::Relaxed);

    let details = f
        .call(json!({"method":"get_account","account_id":id}))
        .await;
    assert_eq!(details["data"]["account"], connected["data"]["account"]);
    let mut expected_ids = connected["data"]["retrieval"]["device_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect::<Vec<_>>();
    expected_ids.sort();
    assert_eq!(details["data"]["device_ids"], json!(expected_ids));
    assert_eq!(
        expected_ids.len(),
        2,
        "Unsupported cards belong in local account details."
    );

    let renamed = f
        .call(json!({"method":"rename_account","account_id":id,"label":"  Family appliances  "}))
        .await;
    assert_eq!(renamed["error"], Value::Null);
    assert_eq!(renamed["data"]["label"], "Family appliances");
    assert_eq!(renamed["data"]["authorization"], "not_checked");
    let mut expected_record = before_record;
    expected_record.label = "Family appliances".into();
    let reopened = SqliteStore::open(&f.path).unwrap();
    assert_eq!(reopened.account(account_id).unwrap(), expected_record);
    assert_eq!(
        f.call(json!({"method":"list_accounts"})).await["data"][0],
        renamed["data"]
    );
    assert_eq!(f.call(json!({"method":"list_device_cards"})).await, cards);
    assert_eq!(*f.secrets.values.lock().unwrap(), before_secrets);
    assert_eq!(f.calls(), before_calls);
    for marker in [
        "credential_ref",
        "SYNTHETIC-TOKEN",
        "88000000000000",
        "synthetic-login-001",
    ] {
        assert!(!format!("{details}{renamed}").contains(marker));
    }
}

#[tokio::test]
async fn local_account_edits_reject_conflicts_and_leave_pending_sessions_pending() {
    let f = Fixture::new();
    let lease = f
        .vault
        .prepare("midea.meiju", "CN", "Pending account", &operation())
        .await
        .unwrap();
    let id = lease.record().id;
    let query = json!({"method":"get_account","account_id":id});
    let edit = json!({"method":"rename_account","account_id":id,"label":"Renamed"});
    assert_eq!(f.call(query.clone()).await["data"]["device_ids"], json!([]));
    assert_eq!(f.call(edit.clone()).await["error"]["code"], "RESOURCE_BUSY");
    drop(lease);
    assert_eq!(
        f.call(edit.clone()).await["data"]["credential_state"],
        "pending"
    );
    for label in [" ".to_string(), "a\nb".to_string(), "a".repeat(81)] {
        let result = f
            .call(json!({"method":"rename_account","account_id":id,"label":label}))
            .await;
        assert_eq!(result["error"]["code"], "INVALID_ARGUMENT");
        assert_eq!(f.store.account(id).unwrap().label, "Renamed");
    }
    for (provider, region) in [("other.provider", "CN"), ("midea.meiju", "US")] {
        let other = f
            .vault
            .prepare(provider, region, "Other", &operation())
            .await
            .unwrap();
        let other_id = other.record().id;
        drop(other);
        for method in ["get_account", "rename_account"] {
            let mut request = json!({"method":method,"account_id":other_id});
            if method == "rename_account" {
                request["label"] = json!("Denied");
            }
            assert_eq!(f.call(request).await["error"]["code"], "ACCOUNT_NOT_FOUND");
        }
        assert_eq!(f.store.account(other_id).unwrap().label, "Other");
    }
    for method in ["get_account", "rename_account"] {
        let mut request = json!({"method":method,"account_id":Uuid::new_v4()});
        if method == "rename_account" {
            request["label"] = json!("Missing");
        }
        assert_eq!(f.call(request).await["error"]["code"], "ACCOUNT_NOT_FOUND");
    }
    f.session.set_foreground(false);
    assert_eq!(
        f.call(edit.clone()).await["error"]["code"],
        "HOST_BACKGROUND"
    );
    f.session.close();
    assert_eq!(f.call(edit).await["error"]["code"], "HOST_CLOSED");
    assert_eq!(f.store.account(id).unwrap().label, "Renamed");
    assert!(f.calls().is_empty());
    assert!(f.secrets.values.lock().unwrap().is_empty());
}

#[tokio::test]
async fn connect_publishes_cards_once_and_reconnect_keeps_identity_and_preferences() {
    let f = Fixture::new();
    assert_eq!(
        f.call(json!({"method":"list_accounts"})).await["data"],
        json!([])
    );
    assert!(f.calls().is_empty());
    let connected = f.connect(None).await;
    assert_eq!(connected["error"], Value::Null);
    assert_eq!(connected["data"]["authentication"], "succeeded");
    assert_eq!(connected["data"]["retrieval"]["status"], "complete");
    let id = connected["data"]["account"]["id"].as_str().unwrap();
    assert_eq!(
        connected["data"]["retrieval"]["device_ids"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        f.calls(),
        vec![
            CloudEndpoint::LoginId,
            CloudEndpoint::Login,
            CloudEndpoint::Homes,
            CloudEndpoint::HomeAppliances
        ]
    );
    let cards = f.call(json!({"method":"list_device_cards"})).await;
    assert_eq!(cards["data"].as_array().unwrap().len(), 2);
    assert!(
        cards["data"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["device"]["support_status"] == "unsupported")
    );
    let device = &cards["data"][0]["device"]["id"];
    let room = f
        .call(json!({"method":"create_room","name":"Bedroom"}))
        .await;
    f.call(json!({"method":"set_device_preferences","device_id":device,"preferences":{"alias":"My AC","room_id":room["data"]["id"],"favorite":true}})).await;
    let baseline = f.call(json!({"method":"list_device_cards"})).await;
    let repeated = f.connect(Some(Uuid::parse_str(id).unwrap())).await;
    assert_eq!(repeated["data"]["account"]["id"], id);
    assert_eq!(
        repeated["data"]["retrieval"]["device_ids"],
        connected["data"]["retrieval"]["device_ids"]
    );
    assert_eq!(
        f.call(json!({"method":"list_device_cards"})).await,
        baseline
    );
    let refreshed = f.refresh(id).await;
    assert_eq!(refreshed["data"]["authentication"], "not_attempted");
    assert_eq!(refreshed["data"]["retrieval"]["status"], "complete");
    assert_eq!(f.calls().len(), 10);
    let before = f.calls();
    f.secrets.unavailable.store(true, Ordering::Relaxed);
    let listed = f.call(json!({"method":"list_accounts"})).await;
    assert_eq!(listed["data"][0]["authorization"], "not_checked");
    assert_eq!(listed["data"][0]["credential_state"], "stored");
    assert_eq!(f.calls(), before);
    for marker in [
        "credential_ref",
        "subject",
        "SYNTHETIC-TOKEN",
        "88000000000000",
        "owner@example.test",
        "Synthetic password",
    ] {
        assert!(!format!("{connected}{listed}{baseline}").contains(marker));
    }
}

#[tokio::test]
async fn failed_or_partial_retrieval_keeps_stored_authentication_and_retry_never_logs_in() {
    for (code, expected) in [(999, "partial"), (7400, "failed")] {
        let f = Fixture::new();
        f.wire.inventory_code.store(code, Ordering::Relaxed);
        let result = f.connect(None).await;
        assert_eq!(result["data"]["authentication"], "succeeded");
        assert_eq!(result["data"]["account"]["credential_state"], "stored");
        assert_eq!(result["data"]["retrieval"]["status"], expected);
        assert_eq!(result["data"]["retrieval"]["device_ids"], json!([]));
        assert!(f.store.devices().unwrap().is_empty());
        let id = result["data"]["account"]["id"].as_str().unwrap();
        f.wire.inventory_code.store(0, Ordering::Relaxed);
        assert_eq!(
            f.refresh(id).await["data"]["retrieval"]["status"],
            "complete"
        );
        assert_eq!(f.calls().len(), 6);
        f.wire.homes_code.store(7400, Ordering::Relaxed);
        let expired = f.refresh(id).await;
        assert_eq!(expired["data"]["retrieval"]["status"], "failed");
        assert_eq!(
            expired["data"]["retrieval"]["error"]["code"],
            "AUTH_REQUIRED"
        );
        assert_eq!(f.store.devices().unwrap().len(), 2);
    }
}

#[tokio::test]
async fn wrong_subject_and_cross_account_duplicates_never_replace_existing_cards() {
    let f = Fixture::new();
    let connected = f.connect(None).await;
    let id = connected["data"]["account"]["id"].as_str().unwrap();
    let baseline = f.call(json!({"method":"list_device_cards"})).await;
    *f.wire.subject.lock().unwrap() = "different-subject".into();
    assert_eq!(
        f.connect(Some(Uuid::parse_str(id).unwrap())).await["error"]["code"],
        "ACCOUNT_IDENTITY_CONFLICT"
    );
    assert_eq!(
        f.call(json!({"method":"list_accounts"})).await["data"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let duplicate = f.connect(None).await;
    assert_eq!(duplicate["data"]["authentication"], "succeeded");
    assert_eq!(duplicate["data"]["retrieval"]["status"], "failed");
    assert_eq!(
        duplicate["data"]["retrieval"]["error"]["code"],
        "DEVICE_IDENTITY_CONFLICT"
    );
    assert_eq!(
        f.call(json!({"method":"list_device_cards"})).await,
        baseline
    );
}

#[tokio::test]
async fn uncertain_inventory_storage_returns_current_pending_state_without_resetting_bindings() {
    let f = Fixture::new();
    let connected = f.connect(None).await;
    let id = connected["data"]["account"]["id"].as_str().unwrap();
    f.wire.extra_device.store(true, Ordering::Relaxed);
    f.secrets.uncertain.store(true, Ordering::Relaxed);
    let refreshed = f.refresh(id).await;
    assert_eq!(refreshed["data"]["account"]["credential_state"], "pending");
    assert_eq!(
        refreshed["data"]["retrieval"]["error"]["code"],
        "SECRET_STORE_WRITE_UNKNOWN"
    );
    assert_eq!(f.store.devices().unwrap().len(), 2);
    let calls = f.calls().len();
    assert_eq!(
        f.refresh(id).await["data"]["retrieval"]["error"]["code"],
        "ACCOUNT_SESSION_UNCONFIRMED"
    );
    assert_eq!(f.calls().len(), calls);
    assert_eq!(f.store.accounts().unwrap().len(), 1);
}

#[tokio::test]
async fn unavailable_credentials_and_invalid_metadata_stop_before_login() {
    let f = Fixture::new();
    f.secrets.unavailable.store(true, Ordering::Relaxed);
    assert_eq!(
        f.connect(None).await["error"]["code"],
        "SECRET_STORE_UNAVAILABLE"
    );
    assert!(f.calls().is_empty());
    assert!(f.store.accounts().unwrap().is_empty());
    f.secrets.unavailable.store(false, Ordering::Relaxed);
    assert_eq!(
        f.refresh(&Uuid::new_v4().to_string()).await["error"]["code"],
        "ACCOUNT_NOT_FOUND"
    );
    assert!(f.calls().is_empty());
    let metadata = json!({"schema_version":"1.0","operation_id":"connect","timeout_ms":1000,"label":"Test","password":"REFUSED"});
    assert!(serde_json::from_value::<ConnectRequest>(metadata).is_err());
    assert!(serde_json::from_value::<HostRequest>(json!({"schema_version":"1.0","operation_id":"a","timeout_ms":1000,"request":{"method":"connect_account","password":"REFUSED"}})).is_err());
}

#[tokio::test(start_paused = true)]
async fn connection_shares_foreground_operation_ids_deadlines_and_lease_cleanup() {
    for endpoint in [CloudEndpoint::Login, CloudEndpoint::HomeAppliances] {
        for terminate in ["cancel", "background", "close", "deadline"] {
            let f = Fixture::new();
            *f.wire.delay.lock().unwrap() = Some(endpoint);
            let pending = f.connect(None);
            tokio::pin!(pending);
            tokio::select! {
                value = &mut pending => panic!("Unexpected completion: {value}"),
                _ = f.wire.started.notified() => (),
            }
            assert_eq!(
                f.call_id("connect", json!({"method":"list_accounts"}))
                    .await["error"]["code"],
                "RESOURCE_BUSY"
            );
            assert_eq!(
                f.vault
                    .lock_registration(&operation())
                    .await
                    .err()
                    .unwrap()
                    .code,
                "RESOURCE_BUSY"
            );
            match terminate {
                "cancel" => assert!(f.session.cancel("connect")),
                "background" => f.session.set_foreground(false),
                "close" => f.session.close(),
                _ => tokio::time::advance(std::time::Duration::from_secs(2)).await,
            }
            let result = pending.await;
            let error = if endpoint == CloudEndpoint::Login {
                &result["error"]
            } else {
                assert_eq!(result["data"]["authentication"], "succeeded");
                &result["data"]["retrieval"]["error"]
            };
            assert_eq!(
                error["code"],
                if terminate == "deadline" {
                    "DEADLINE_EXCEEDED"
                } else {
                    "CANCELLED"
                }
            );
            drop(f.vault.lock_registration(&operation()).await.unwrap());
            for record in f.vault.list(&operation()).await.unwrap() {
                drop(f.vault.lease(record.id, &operation()).await.unwrap());
            }
            assert!(f.store.devices().unwrap().is_empty());
            let calls = f.calls().len();
            if terminate == "background" || terminate == "close" {
                assert_eq!(
                    f.connect(None).await["error"]["code"],
                    if terminate == "close" {
                        "HOST_CLOSED"
                    } else {
                        "HOST_BACKGROUND"
                    }
                );
                assert_eq!(f.calls().len(), calls);
            }
        }
    }
}

#[tokio::test]
async fn publication_refuses_unregistered_foreign_or_invalid_descriptor_batches() {
    let f = Fixture::new();
    let core = Core::new(
        f.store.clone(),
        vec![],
        Arc::new(SystemClock),
        Policy::default(),
    );
    assert_eq!(
        core.publish_discovery("midea.meiju", &[], &operation())
            .await
            .unwrap_err()
            .code,
        "CAPABILITY_UNSUPPORTED"
    );
    assert!(f.store.devices().unwrap().is_empty());
    assert_eq!(
        f.connect(None).await["data"]["retrieval"]["status"],
        "complete"
    );
    let baseline = serde_json::to_value(f.store.devices().unwrap()).unwrap();
    let adapter = Arc::new(MeijuAdapter::new(
        f.vault.clone(),
        f.wire.clone(),
        Arc::new(SystemClock),
    ));
    let core = Core::new(
        f.store.clone(),
        vec![adapter],
        Arc::new(SystemClock),
        Policy::default(),
    );
    for kind in ["foreign", "invalid", "duplicate"] {
        let mut descriptors = f.store.devices().unwrap();
        descriptors[0].device.alias = "Must not publish".into();
        match kind {
            "foreign" => descriptors[1].device.connections[0].adapter = "other.provider".into(),
            "invalid" => descriptors[1].device.schema_version = "99.0".into(),
            _ => descriptors[1].device.id = descriptors[0].device.id.clone(),
        }
        assert_eq!(
            core.publish_discovery("midea.meiju", &descriptors, &operation())
                .await
                .unwrap_err()
                .code,
            "CAPABILITY_UNSUPPORTED"
        );
        assert_eq!(
            serde_json::to_value(f.store.devices().unwrap()).unwrap(),
            baseline
        );
    }
}

fn power_command(device: &str, state: bool) -> Value {
    json!({"schema_version":"1.0","request_id":Uuid::new_v4().to_string(),
        "device_id":device,"component_id":"main","capability":"power","action":"set",
        "arguments":{"state":state},"connection":"cloud","timeout_ms":1000,"wait_for":"verified"})
}

#[tokio::test]
async fn native_power_grants_scope_actual_session_plans_outcomes_and_reopening() {
    let mut f = Fixture::new();
    let connected = f.connect(None).await;
    let account = connected["data"]["account"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let device = f
        .store
        .devices()
        .unwrap()
        .into_iter()
        .find(|d| d.device.model == "KFR-35G/WF2")
        .unwrap()
        .device
        .id;
    f.reopen_with_power_grants(&[MeijuPowerGrant {
        device_id: device.clone(),
        state: false,
    }]);
    f.refresh(&account).await;
    let before = f.calls();
    let command = power_command(&device, false);
    assert!(f.call(json!({"method":"plan","command":command})).await["error"].is_null());
    assert_eq!(f.calls(), before);
    assert_eq!(
        f.call(json!({"method":"plan","command":power_command(&device,true)}))
            .await["error"]["code"],
        "POLICY_DENIED"
    );
    let mut lan = power_command(&device, false);
    lan["connection"] = json!("lan");
    assert_eq!(
        f.call(json!({"method":"plan","command":lan})).await["error"]["code"],
        "POLICY_DENIED"
    );
    let result = f.call(json!({"method":"execute","command":command})).await;
    assert_eq!(result["status"], "accepted");
    assert_eq!(result["error"]["code"], "VERIFICATION_PENDING");
    assert!(result["verification"].is_null());
    let schema: Value =
        serde_json::from_str(include_str!("../../../schemas/command-result.schema.json")).unwrap();
    jsonschema::validator_for(&schema)
        .unwrap()
        .validate(&result)
        .unwrap();
    assert_eq!(f.calls().len(), before.len() + 1);
    assert_eq!(
        f.call(json!({"method":"execute","command":command})).await,
        result
    );
    assert_eq!(f.calls().len(), before.len() + 1);
    f.wire.control_code.store(1306, Ordering::Relaxed);
    let unknown = f
        .call(json!({"method":"execute","command":power_command(&device,false)}))
        .await;
    assert_eq!(unknown["status"], "unknown");
    jsonschema::validator_for(&schema)
        .unwrap()
        .validate(&unknown)
        .unwrap();
    let before = f.calls();
    f.reopen_with_power_grants(&[]);
    assert_eq!(
        f.call(json!({"method":"execute","command":power_command(&device,false)}))
            .await["error"]["code"],
        "POLICY_DENIED"
    );
    let receipt = f
        .call(json!({"method":"get_execution","request_id":command["request_id"]}))
        .await;
    assert_eq!(receipt["data"], result);
    assert_eq!(f.calls(), before);
}

#[tokio::test]
async fn native_power_background_cancel_and_close_preserve_unknown_and_release_account() {
    let mut f = Fixture::new();
    let connected = f.connect(None).await;
    let account = connected["data"]["account"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let device = f
        .store
        .devices()
        .unwrap()
        .into_iter()
        .find(|d| d.device.model == "KFR-35G/WF2")
        .unwrap()
        .device
        .id;
    let grants = [MeijuPowerGrant {
        device_id: device.clone(),
        state: false,
    }];
    for stop in ["background", "cancel", "close"] {
        f.reopen_with_power_grants(&grants);
        *f.wire.delay.lock().unwrap() = None;
        f.refresh(&account).await;
        *f.wire.delay.lock().unwrap() = Some(CloudEndpoint::ApplianceControl);
        let command = power_command(&device, false);
        {
            let work = f.call_id("power", json!({"method":"execute","command":command}));
            tokio::pin!(work);
            tokio::select! {
                _ = f.wire.started.notified() => (),
                _ = &mut work => panic!("Expected an in-flight control."),
            }
            match stop {
                "background" => f.session.set_foreground(false),
                "cancel" => {
                    assert!(f.session.cancel("power"));
                }
                _ => f.session.close(),
            }
            let result = work.await;
            assert_eq!(result["status"], "unknown");
            assert_eq!(result["attempts"][0]["phase"], "possibly_sent");
        }
        let before = f.calls();
        if stop != "cancel" {
            let result = f
                .call(json!({"method":"execute","command":power_command(&device,false)}))
                .await;
            assert_eq!(
                result["error"]["code"],
                if stop == "close" {
                    "HOST_CLOSED"
                } else {
                    "HOST_BACKGROUND"
                }
            );
        }
        assert_eq!(f.calls(), before);
        drop(
            f.vault
                .lease(Uuid::parse_str(&account).unwrap(), &operation())
                .await
                .unwrap(),
        );
    }
}

#[path = "meiju_accounts/removal.rs"]
mod removal;
