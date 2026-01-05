use crate::error::{Error, Result};
use crate::models::Account;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::engine::any::Any;
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct AccountRecord {
    id: Thing,
    #[serde(flatten)]
    account: Account,
}

impl AccountRecord {
    fn into_account(self) -> Account {
        let mut account = self.account;
        let id_str = self.id.id.to_string();
        let clean_id = id_str.trim_start_matches('⟨').trim_end_matches('⟩');
        account.id = Uuid::parse_str(clean_id).unwrap_or_else(|_| Uuid::nil());
        account
    }
}

pub struct AccountRepository {
    db: Arc<Surreal<Any>>,
}

impl AccountRepository {
    pub fn new(db: Arc<Surreal<Any>>) -> Self {
        Self { db }
    }

    pub async fn create(&self, account: Account) -> Result<Account> {
        let account_id = account.id.to_string();
        let created: Option<AccountRecord> = self
            .db
            .create(("account", &account_id))
            .content(account)
            .await?;

        created
            .map(|rec| rec.into_account())
            .ok_or_else(|| Error::Database("Failed to create account".to_string()))
    }

    pub async fn get(&self, id: Uuid) -> Result<Account> {
        let record: Option<AccountRecord> = self.db.select(("account", id.to_string())).await?;

        record
            .map(|rec| rec.into_account())
            .ok_or_else(|| Error::NotFound(format!("Account with id {}", id)))
    }

    pub async fn get_by_name(&self, name: &str) -> Result<Account> {
        let records: Vec<AccountRecord> = self
            .db
            .query("SELECT * FROM account WHERE name = $name LIMIT 1")
            .bind(("name", name.to_string()))
            .await?
            .take(0)?;

        records
            .into_iter()
            .next()
            .map(|rec| rec.into_account())
            .ok_or_else(|| Error::NotFound(format!("Account with name {}", name)))
    }

    pub async fn list(&self) -> Result<Vec<Account>> {
        let records: Vec<AccountRecord> = self.db.select("account").await?;

        Ok(records.into_iter().map(|rec| rec.into_account()).collect())
    }

    pub async fn update(&self, account: Account) -> Result<Account> {
        let account_id = account.id;

        // First check if the account exists
        let _existing: Account = self.get(account_id).await?;

        let updated: Option<AccountRecord> = self
            .db
            .update(("account", account_id.to_string()))
            .content(account)
            .await?;

        updated
            .map(|rec| rec.into_account())
            .ok_or_else(|| Error::NotFound(format!("Account with id {}", account_id)))
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let _deleted: Option<AccountRecord> = self.db.delete(("account", id.to_string())).await?;
        Ok(())
    }
}
