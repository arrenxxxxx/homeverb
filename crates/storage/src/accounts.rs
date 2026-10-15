use crate::*;
use homeverb_core::{
    AccountLock, AccountRecord, AccountStore, CredentialRef, CredentialState, valid_account_id,
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
        self.connection
            .lock()
            .map_err(storage_error)?
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
        Ok(())
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
        };
        let changed = self
            .connection
            .lock()
            .map_err(storage_error)?
            .execute(
                "UPDATE accounts SET credential_state=?2 WHERE id=?1",
                params![id.to_string(), state],
            )
            .map_err(storage_error)?;
        if changed != 1 {
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
}
