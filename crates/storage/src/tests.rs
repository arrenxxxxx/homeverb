use super::*;

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("homeverb-storage-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn database(&self) -> PathBuf {
        self.0.join("homeverb.sqlite3")
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn old_database(directory: &Directory) -> Connection {
    let connection = Connection::open(directory.database()).unwrap();
    connection.execute_batch("PRAGMA journal_mode=WAL;
        CREATE TABLE devices (id TEXT PRIMARY KEY, descriptor TEXT NOT NULL);
        CREATE TABLE snapshots (device TEXT NOT NULL, component TEXT NOT NULL, snapshot TEXT NOT NULL, PRIMARY KEY(device, component));
        CREATE TABLE executions (request_id TEXT PRIMARY KEY, command TEXT NOT NULL, result TEXT NOT NULL);
        PRAGMA user_version=1;").unwrap();
    connection
}

fn command(id: &str, device: &str) -> (Command, Execution) {
    let mut command: Command =
        serde_json::from_str(include_str!("../../../examples/command.json")).unwrap();
    command.request_id = id.into();
    command.device_id = device.into();
    let mut execution: Execution = serde_json::from_str(include_str!(
        "../../../examples/command-result.verified.json"
    ))
    .unwrap();
    execution.request_id = id.into();
    execution.device_id = device.into();
    (command, execution)
}

#[test]
fn competing_discovery_inserts_cannot_replace_the_winning_identity() {
    let directory = Directory::new();
    let first = SqliteStore::open(&directory.0).unwrap();
    let second = SqliteStore::open(&directory.0).unwrap();
    let descriptor = Descriptor {
        device: serde_json::from_str(include_str!("../../../examples/device.json")).unwrap(),
        capabilities: vec![],
    };
    let mut competing = descriptor.clone();
    competing.device.connections[0].identity_ref = "identity:competing-appliance".into();
    let barrier = std::sync::Barrier::new(2);
    let (left, right) = std::thread::scope(|scope| {
        let left = scope.spawn(|| {
            assert!(first.devices().unwrap().is_empty());
            barrier.wait();
            first.save_device(&descriptor)
        });
        let right = scope.spawn(|| {
            assert!(second.devices().unwrap().is_empty());
            barrier.wait();
            second.save_device(&competing)
        });
        (left.join().unwrap(), right.join().unwrap())
    });
    let (winner, loser, error) = match (left, right) {
        (Ok(()), Err(error)) => (descriptor, competing, error),
        (Err(error), Ok(())) => (competing, descriptor, error),
        results => panic!("Exactly one identity must win: {results:?}"),
    };
    assert_eq!(error.code, "DEVICE_IDENTITY_CONFLICT");
    assert!(!error.message.contains("competing-appliance"));
    assert_eq!(
        first.devices().unwrap()[0].device.connections,
        winner.device.connections
    );
    assert_eq!(
        second.save_device(&loser).unwrap_err().code,
        "DEVICE_IDENTITY_CONFLICT"
    );
    let preferences = first.preferences().unwrap();
    let mut updated = winner;
    updated.device.connections.reverse();
    updated.device.alias = "Updated appliance label".into();
    second.save_device(&updated).unwrap();
    assert_eq!(
        first.devices().unwrap()[0].device.alias,
        "Updated appliance label"
    );
    assert_eq!(first.preferences().unwrap(), preferences);
}

#[test]
fn lock_names_preserve_legacy_paths_and_bound_long_utf8_identifiers() {
    let directory = Directory::new();
    let store = SqliteStore::open(&directory.0).unwrap();
    for (id, filename) in [
        ("a".repeat(127), "61".repeat(127)),
        (
            "a".repeat(128),
            "sha256-6836cf13bac400e9105071cd6af47084dfacad4e5e302c94bfed24e013afb73e".into(),
        ),
        (
            "\u{00e9}".repeat(128),
            "sha256-e42dd264fd5cf1bc947505b995dceb9ae0a2d2a4c99b4ce5ea02f36526819280".into(),
        ),
    ] {
        let lock = store.lock_device(&id).unwrap();
        let file = File::options()
            .read(true)
            .write(true)
            .open(directory.0.join("locks").join(filename))
            .unwrap();
        assert!(file.try_lock().is_err());
        drop(lock);
        file.try_lock().unwrap();
    }
}

#[test]
fn migration_preserves_records_and_creates_a_consistent_version_one_backup() {
    let directory = Directory::new();
    let old = old_database(&directory);
    let descriptor = Descriptor {
        device: serde_json::from_str(include_str!("../../../examples/device.json")).unwrap(),
        capabilities: vec![
            serde_json::from_str(include_str!("../../../examples/capability.climate.json"))
                .unwrap(),
        ],
    };
    let (command, execution) = command("previous", &descriptor.device.id);
    old.execute(
        "INSERT INTO devices VALUES (?1, ?2)",
        params![descriptor.device.id, encode(&descriptor).unwrap()],
    )
    .unwrap();
    old.execute(
        "INSERT INTO executions VALUES (?1, ?2, ?3)",
        params![
            command.request_id,
            encode(&command).unwrap(),
            encode(&execution).unwrap()
        ],
    )
    .unwrap();
    let snapshot: Snapshot =
        serde_json::from_str(include_str!("../../../examples/state.json")).unwrap();
    old.execute(
        "INSERT INTO snapshots VALUES (?1, ?2, ?3)",
        params![
            snapshot.device_id,
            snapshot.component_id,
            encode(&snapshot).unwrap()
        ],
    )
    .unwrap();
    // Keep an ordinary SQLite connection open to exercise backup of committed WAL contents.
    let store = SqliteStore::open(&directory.0).unwrap();
    assert_eq!(
        migration::version(&store.connection.lock().unwrap()).unwrap(),
        4
    );
    assert_eq!(store.devices().unwrap()[0].device.id, descriptor.device.id);
    assert_eq!(
        store.preferences().unwrap()[0].device_id,
        descriptor.device.id
    );
    assert_eq!(
        store.execution("previous").unwrap().unwrap().1.status,
        Status::Verified
    );
    assert!(
        store
            .snapshot(&snapshot.device_id, "main")
            .unwrap()
            .is_some()
    );
    let backups: Vec<_> = std::fs::read_dir(directory.0.join("backups"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(backups.len(), 1);
    let backup = Connection::open(&backups[0]).unwrap();
    assert_eq!(migration::version(&backup).unwrap(), 1);
    assert_eq!(
        backup
            .query_row("SELECT COUNT(*) FROM executions", [], |r| r
                .get::<_, usize>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        backup
            .query_row("SELECT COUNT(*) FROM snapshots", [], |r| r
                .get::<_, usize>(0))
            .unwrap(),
        1
    );
    drop(old);
}

#[test]
fn failed_migration_rolls_back_and_preserves_its_backup() {
    let directory = Directory::new();
    let old = old_database(&directory);
    old.execute_batch("CREATE TABLE rooms (legacy TEXT); INSERT INTO rooms VALUES ('preserved');")
        .unwrap();
    assert!(SqliteStore::open(&directory.0).is_err());
    assert_eq!(migration::version(&old).unwrap(), 1);
    assert_eq!(
        old.query_row("SELECT legacy FROM rooms", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "preserved"
    );
    assert_eq!(
        old.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE name='device_preferences'",
            [],
            |r| r.get::<_, u32>(0)
        )
        .unwrap(),
        0
    );
    assert_eq!(
        std::fs::read_dir(directory.0.join("backups"))
            .unwrap()
            .count(),
        1
    );
}

#[test]
fn schema_two_upgrade_preserves_organization_and_persists_a_random_profile_namespace() {
    use homeverb_core::AccountStore;
    let directory = Directory::new();
    let store = SqliteStore::open(&directory.0).unwrap();
    store.create_room("room-example", "Bedroom").unwrap();
    store
        .connection
        .lock()
        .unwrap()
        .execute_batch("DROP TABLE accounts; DROP TABLE profile_namespace; DROP TABLE account_removals; PRAGMA user_version=2;")
        .unwrap();
    drop(store);
    let upgraded = SqliteStore::open(&directory.0).unwrap();
    assert_eq!(upgraded.rooms().unwrap()[0].name, "Bedroom");
    let namespace = upgraded.profile_namespace().unwrap();
    assert_eq!(namespace.get_version_num(), 4);
    assert!(upgraded.accounts().unwrap().is_empty());
    let backups: Vec<_> = std::fs::read_dir(directory.0.join("backups"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    assert_eq!(backups.len(), 1);
    let backup = Connection::open(&backups[0]).unwrap();
    assert_eq!(migration::version(&backup).unwrap(), 2);
    assert_eq!(
        backup
            .query_row("SELECT name FROM rooms", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "Bedroom"
    );
    drop(upgraded);
    assert_eq!(
        SqliteStore::open(&directory.0)
            .unwrap()
            .profile_namespace()
            .unwrap(),
        namespace
    );
}

#[test]
fn account_storage_rejects_rebinding_and_corrupt_references() {
    use homeverb_core::{AccountRecord, AccountStore, CredentialRef, CredentialState};
    let directory = Directory::new();
    let store = SqliteStore::open(&directory.0).unwrap();
    let record = AccountRecord {
        id: uuid::Uuid::new_v4(),
        provider: "midea.meiju".into(),
        region: "CN".into(),
        label: "Home".into(),
        credential_ref: CredentialRef::new(),
        credential_state: CredentialState::Pending,
    };
    store.insert_account(&record).unwrap();
    let mut rebound = record.clone();
    rebound.provider = "other.provider".into();
    assert!(store.insert_account(&rebound).is_err());
    assert_eq!(store.account(record.id).unwrap(), record);
    store
        .connection
        .lock()
        .unwrap()
        .execute(
            "UPDATE accounts SET credential_ref='not-a-random-id' WHERE id=?1",
            [record.id.to_string()],
        )
        .unwrap();
    assert_eq!(store.account(record.id).unwrap_err().code, "STORAGE_ERROR");
    assert_eq!(store.accounts().unwrap_err().code, "STORAGE_ERROR");
}

#[test]
fn newer_or_foreign_databases_are_rejected_without_schema_changes() {
    for version in [0, 5] {
        let directory = Directory::new();
        let connection = Connection::open(directory.database()).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE sentinel (value TEXT); INSERT INTO sentinel VALUES ('preserved');",
            )
            .unwrap();
        connection
            .pragma_update(None, "user_version", version)
            .unwrap();
        assert!(
            matches!(SqliteStore::open(&directory.0), Err(error) if error.code == "STORAGE_VERSION_UNSUPPORTED")
        );
        assert_eq!(migration::version(&connection).unwrap(), version);
        assert_eq!(
            connection
                .query_row("SELECT value FROM sentinel", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "preserved"
        );
    }
}

#[test]
fn migration_requires_exclusive_profile_access() {
    let directory = Directory::new();
    let store = SqliteStore::open(&directory.0).unwrap();
    // Model a caller retaining an older schema lease while a new process starts.
    store
        .connection
        .lock()
        .unwrap()
        .pragma_update(None, "user_version", 1)
        .unwrap();
    assert!(matches!(SqliteStore::open(&directory.0), Err(error) if error.code == "RESOURCE_BUSY"));
    assert_eq!(
        migration::version(&store.connection.lock().unwrap()).unwrap(),
        1
    );
}

#[test]
fn retirement_bounds_receipts_keeps_replay_ids_and_preserves_active_writes() {
    let directory = Directory::new();
    let store = SqliteStore::open(&directory.0).unwrap();
    let active_lock = store.lock_device("active-device").unwrap();
    let (pending_command, mut pending) = command("pending", "active-device");
    pending.status = Status::Unknown;
    pending.verification = None;
    pending.error = Some(Error::new(
        "WRITE_OUTCOME_UNKNOWN",
        "The earlier write is unconfirmed.",
    ));
    store.save_execution(&pending_command, &pending).unwrap();
    for index in 0..RECEIPT_LIMIT + 5 {
        let (command, execution) = command(&format!("completed-{index}"), "another-device");
        store.finish_execution(&command, &execution).unwrap();
    }
    let connection = store.connection.lock().unwrap();
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM executions WHERE finalized=1",
                [],
                |r| r.get::<_, usize>(0)
            )
            .unwrap(),
        RECEIPT_LIMIT
    );
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM retired_requests", [], |r| r
                .get::<_, usize>(0))
            .unwrap(),
        5
    );
    drop(connection);
    assert_eq!(
        store.execution("completed-0").unwrap_err().code,
        "REQUEST_RETIRED"
    );
    assert_eq!(
        store.execution("pending").unwrap().unwrap().1.status,
        Status::Unknown
    );
    let other = SqliteStore::open(&directory.0).unwrap();
    assert!(other.execution("pending").unwrap().is_some());
    assert!(
        !other
            .connection
            .lock()
            .unwrap()
            .query_row(
                "SELECT finalized FROM executions WHERE request_id='pending'",
                [],
                |r| r.get::<_, bool>(0)
            )
            .unwrap()
    );
    drop(active_lock);
    drop(other);
    let reopened = SqliteStore::open(&directory.0).unwrap();
    assert_eq!(
        reopened.execution("pending").unwrap_err().code,
        "REQUEST_RETIRED"
    );
    assert_eq!(reopened.recent_executions().unwrap().len(), RECEIPT_LIMIT);
    let (old_command, old_result) = command("completed-0", "another-device");
    assert_eq!(
        reopened
            .save_execution(&old_command, &old_result)
            .unwrap_err()
            .code,
        "REQUEST_RETIRED"
    );
}

#[test]
fn schema_three_migration_preserves_account_references_and_enables_durable_removal() {
    use homeverb_core::{AccountRecord, AccountStore, CredentialRef, CredentialState};
    let directory = Directory::new();
    let store = SqliteStore::open(&directory.0).unwrap();
    let namespace = store.profile_namespace().unwrap();
    let mut account = AccountRecord {
        id: uuid::Uuid::new_v4(),
        provider: "midea.meiju".into(),
        region: "CN".into(),
        label: "Existing account".into(),
        credential_ref: CredentialRef::new(),
        credential_state: CredentialState::Pending,
    };
    store.insert_account(&account).unwrap();
    store
        .set_credential_state(account.id, CredentialState::Stored)
        .unwrap();
    account.credential_state = CredentialState::Stored;
    store.create_room("kept-room", "Bedroom").unwrap();
    drop(store);
    let old = Connection::open(directory.database()).unwrap();
    old.execute_batch(
        "ALTER TABLE accounts RENAME TO newer_accounts;
        CREATE TABLE accounts (id TEXT PRIMARY KEY, provider TEXT NOT NULL, region TEXT NOT NULL,
        label TEXT NOT NULL, credential_ref TEXT NOT NULL UNIQUE,
        credential_state TEXT NOT NULL CHECK(credential_state IN ('pending','stored')));
        INSERT INTO accounts SELECT * FROM newer_accounts; DROP TABLE newer_accounts;
        DROP TABLE account_removals; PRAGMA user_version=3;",
    )
    .unwrap();
    let upgraded = SqliteStore::open(&directory.0).unwrap();
    assert_eq!(upgraded.account(account.id).unwrap(), account);
    assert_eq!(upgraded.profile_namespace().unwrap(), namespace);
    assert_eq!(upgraded.rooms().unwrap().len(), 1);
    assert!(upgraded.account_removal(account.id).unwrap().is_none());
    upgraded.begin_account_removal(account.id, &[]).unwrap();
    assert_eq!(
        upgraded.account(account.id).unwrap().credential_state,
        CredentialState::Removing
    );
    upgraded.finish_account_removal(account.id).unwrap();
    assert!(
        upgraded
            .account_removal(account.id)
            .unwrap()
            .unwrap()
            .completed
    );
    account.credential_state = CredentialState::Pending;
    assert_eq!(
        upgraded.insert_account(&account).unwrap_err().code,
        "ACCOUNT_NOT_FOUND"
    );
    let backup_path = std::fs::read_dir(directory.0.join("backups"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let backup = Connection::open(backup_path).unwrap();
    assert_eq!(migration::version(&backup).unwrap(), 3);
    assert_eq!(
        backup
            .query_row("SELECT credential_state FROM accounts", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "stored"
    );
    assert_eq!(
        backup
            .query_row("SELECT credential_ref FROM accounts", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        account.credential_ref.to_string()
    );
    assert_eq!(
        backup
            .query_row("SELECT namespace FROM profile_namespace", [], |row| row
                .get::<_, String>(
                0
            ))
            .unwrap(),
        namespace.to_string()
    );
}
