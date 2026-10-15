use crate::*;
use homeverb_core::{
    AccountLock, AccountRecord, AccountRemoval, AccountStore, CredentialRef, CredentialState,
    account_removal_pending, account_scope_changed, removal_device_ids, valid_account_id,
};
use uuid::Uuid;

struct FileAccountLock {
    _file: File,
}

impl AccountLock for FileAccountLock {}

fn missing_account() -> Error {
    Error::new(
        "ACCOUNT_NOT_FOUND",
        "The local account reference does not exist.",
    )
}

fn row_account(row: &rusqlite::Row<'_>) -> rusqlite::Result<[String; 6]> {
    Ok([
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
    ])
}

fn account_record(
    [id, provider, region, label, reference, state]: [String; 6],
) -> Result<AccountRecord> {
    let record = AccountRecord {
        id: Uuid::parse_str(&id).map_err(storage_error)?,
        provider,
        region,
        label,
        credential_ref: reference.parse::<CredentialRef>().map_err(storage_error)?,
        credential_state: match state.as_str() {
            "pending" => CredentialState::Pending,
            "stored" => CredentialState::Stored,
            "removing" => CredentialState::Removing,
            _ => return Err(storage_error(())),
        },
    };
    record.validate().map_err(storage_error)?;
    Ok(record)
}

impl AccountStore for SqliteStore {
    fn profile_namespace(&self) -> Result<Uuid> {
        let value: String = self
            .connection
            .lock()
            .map_err(storage_error)?
            .query_row(
                "SELECT namespace FROM profile_namespace WHERE singleton=1",
                [],
                |row| row.get(0),
            )
            .map_err(storage_error)?;
        let id = Uuid::parse_str(&value).map_err(storage_error)?;
        valid_account_id(id).map_err(storage_error)?;
        Ok(id)
    }

    fn accounts(&self) -> Result<Vec<AccountRecord>> {
        let connection = self.connection.lock().map_err(storage_error)?;
        let mut query = connection.prepare("SELECT id, provider, region, label, credential_ref, credential_state FROM accounts ORDER BY rowid").map_err(storage_error)?;
        query
            .query_map([], row_account)
            .map_err(storage_error)?
            .map(|row| account_record(row.map_err(storage_error)?))
            .collect()
    }

    fn account(&self, id: Uuid) -> Result<AccountRecord> {
        valid_account_id(id)?;
        let row = self.connection.lock().map_err(storage_error)?.query_row(
            "SELECT id, provider, region, label, credential_ref, credential_state FROM accounts WHERE id=?1",
            [id.to_string()], row_account
        ).optional().map_err(storage_error)?.ok_or_else(missing_account)?;
        account_record(row)
    }

    fn insert_account(&self, account: &AccountRecord) -> Result<()> {
        account.validate()?;
        if account.credential_state != CredentialState::Pending {
            return Err(Error::invalid());
        }
        let mut connection = self.connection.lock().map_err(storage_error)?;
        let transaction = connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        if let Some(removal) = removal(&transaction, account.id)? {
            return Err(inactive_account(&removal));
        }
        transaction
            .execute(
                "INSERT INTO accounts VALUES (?1, ?2, ?3, ?4, ?5, 'pending')",
                params![
                    account.id.to_string(),
                    account.provider,
                    account.region,
                    account.label,
                    account.credential_ref.to_string()
                ],
            )
            .map_err(storage_error)?;
        transaction.commit().map_err(storage_error)
    }

    fn lock_account(&self, id: Uuid) -> Result<Box<dyn AccountLock>> {
        valid_account_id(id)?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.locks.join(format!("account-{id}")))
            .map_err(storage_error)?;
        file.try_lock().map_err(|_| {
            Error::new(
                "RESOURCE_BUSY",
                "Another process is using this account session.",
            )
        })?;
        Ok(Box::new(FileAccountLock { _file: file }))
    }

    fn set_credential_state(&self, id: Uuid, state: CredentialState) -> Result<()> {
        valid_account_id(id)?;
        let state = match state {
            CredentialState::Pending => "pending",
            CredentialState::Stored => "stored",
            CredentialState::Removing => return Err(account_removal_pending()),
        };
        let changed = self
            .connection
            .lock()
            .map_err(storage_error)?
            .execute(
                "UPDATE accounts SET credential_state=?2 WHERE id=?1 AND credential_state!='removing'",
                params![id.to_string(), state],
            )
            .map_err(storage_error)?;
        if changed != 1 {
            if self.account(id)?.credential_state == CredentialState::Removing {
                return Err(account_removal_pending());
            }
            return Err(missing_account());
        }
        Ok(())
    }

    fn rename_account(&self, id: Uuid, label: &str) -> Result<()> {
        valid_account_id(id)?;
        let label = local_name(label)?;
        let changed = self
            .connection
            .lock()
            .map_err(storage_error)?
            .execute(
                "UPDATE accounts SET label=?2 WHERE id=?1",
                params![id.to_string(), label],
            )
            .map_err(storage_error)?;
        if changed != 1 {
            return Err(missing_account());
        }
        Ok(())
    }

    fn account_removal(&self, id: Uuid) -> Result<Option<AccountRemoval>> {
        valid_account_id(id)?;
        removal(&*self.connection.lock().map_err(storage_error)?, id)
    }

    fn begin_account_removal(
        &self,
        id: Uuid,
        expected_devices: &[String],
    ) -> Result<AccountRemoval> {
        valid_account_id(id)?;
        let expected = removal_device_ids(expected_devices)?;
        let mut connection = self.connection.lock().map_err(storage_error)?;
        let transaction = connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let row = transaction.query_row(
            "SELECT id, provider, region, label, credential_ref, credential_state FROM accounts WHERE id=?1",
            [id.to_string()], row_account,
        ).optional().map_err(storage_error)?.ok_or_else(missing_account)?;
        let account = account_record(row)?;
        let device_ids = linked_devices(&transaction, &account)?;
        if device_ids != expected {
            return Err(account_scope_changed());
        }
        let record = match removal(&transaction, id)? {
            Some(previous) => {
                if previous.completed || previous.device_ids != expected {
                    return Err(account_scope_changed());
                }
                previous
            }
            None => AccountRemoval {
                account_id: id,
                provider: account.provider,
                region: account.region,
                device_ids,
                completed: false,
            },
        };
        transaction
            .execute(
                "INSERT INTO account_removals VALUES (?1, ?2) ON CONFLICT(id) DO NOTHING",
                params![id.to_string(), encode(&record)?],
            )
            .map_err(storage_error)?;
        transaction
            .execute(
                "UPDATE accounts SET credential_state='removing' WHERE id=?1",
                [id.to_string()],
            )
            .map_err(storage_error)?;
        transaction.commit().map_err(storage_error)?;
        Ok(record)
    }

    fn finish_account_removal(&self, id: Uuid) -> Result<()> {
        valid_account_id(id)?;
        let mut connection = self.connection.lock().map_err(storage_error)?;
        let transaction = connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let mut record = removal(&transaction, id)?.ok_or_else(missing_account)?;
        if record.completed {
            return Ok(());
        }
        let row = transaction.query_row(
            "SELECT id, provider, region, label, credential_ref, credential_state FROM accounts WHERE id=?1",
            [id.to_string()], row_account,
        ).optional().map_err(storage_error)?.ok_or_else(missing_account)?;
        let account = account_record(row)?;
        if account.credential_state != CredentialState::Removing
            || linked_devices(&transaction, &account)? != record.device_ids
        {
            return Err(account_scope_changed());
        }
        for device in &record.device_ids {
            transaction
                .execute("DELETE FROM snapshots WHERE device=?1", [device])
                .map_err(storage_error)?;
            transaction
                .execute("DELETE FROM devices WHERE id=?1", [device])
                .map_err(storage_error)?;
        }
        transaction
            .execute("DELETE FROM accounts WHERE id=?1", [id.to_string()])
            .map_err(storage_error)?;
        record.completed = true;
        transaction
            .execute(
                "UPDATE account_removals SET record=?2 WHERE id=?1",
                params![id.to_string(), encode(&record)?],
            )
            .map_err(storage_error)?;
        transaction.commit().map_err(storage_error)
    }
}

fn removal(connection: &Connection, id: Uuid) -> Result<Option<AccountRemoval>> {
    let document: Option<String> = connection
        .query_row(
            "SELECT record FROM account_removals WHERE id=?1",
            [id.to_string()],
            |row| row.get(0),
        )
        .optional()
        .map_err(storage_error)?;
    document
        .map(|document| {
            let record: AccountRemoval = decode(document)?;
            if record.account_id != id
                || removal_device_ids(&record.device_ids).map_err(storage_error)?
                    != record.device_ids
            {
                return Err(storage_error(()));
            }
            Ok(record)
        })
        .transpose()
}

fn linked_devices(connection: &Connection, account: &AccountRecord) -> Result<Vec<String>> {
    let id = account.id.to_string();
    let mut query = connection
        .prepare("SELECT id, descriptor FROM devices ORDER BY id")
        .map_err(storage_error)?;
    let rows = query
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(storage_error)?;
    let mut ids = vec![];
    for row in rows {
        let (key, document) = row.map_err(storage_error)?;
        let descriptor: Descriptor = decode(document)?;
        if descriptor
            .device
            .connections
            .iter()
            .any(|c| c.account_id.as_deref() == Some(&id))
        {
            if descriptor.device.id != key {
                return Err(storage_error(()));
            }
            if descriptor
                .device
                .connections
                .iter()
                .any(|c| c.account_id.as_deref() != Some(&id) || c.adapter != account.provider)
            {
                return Err(Error::new(
                    "ACCOUNT_REMOVAL_UNSUPPORTED",
                    "A linked device has other routes; local account removal cannot detach them safely.",
                ));
            }
            ids.push(key);
        }
    }
    Ok(ids)
}

pub(crate) fn admit_connections(connection: &Connection, device: &Device) -> Result<()> {
    for route in &device.connections {
        if let Some(account) = &route.account_id {
            let document: Option<String> = connection
                .query_row(
                    "SELECT record FROM account_removals WHERE id=?1",
                    [account],
                    |row| row.get(0),
                )
                .optional()
                .map_err(storage_error)?;
            if let Some(document) = document {
                let removal: AccountRemoval = decode(document)?;
                return Err(inactive_account(&removal));
            }
        }
    }
    Ok(())
}

fn inactive_account(removal: &AccountRemoval) -> Error {
    if removal.completed {
        missing_account()
    } else {
        account_removal_pending()
    }
}
