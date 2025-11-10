use surrealdb::Surreal;
use surrealdb::engine::local::RocksDb;
use crate::models::{LedgerEntry, Ledger};
use crate::error::{Error, Result};
use uuid::Uuid;
use std::sync::Arc;

pub struct LedgerRepository {
    db: Arc<Surreal<RocksDb>>,
}

impl LedgerRepository {
    pub fn new(db: Arc<Surreal<RocksDb>>) -> Self {
        Self { db }
    }

    pub async fn create_entry(&self, entry: LedgerEntry) -> Result<LedgerEntry> {
        let created: Vec<LedgerEntry> = self.db
            .create(("ledger_entry", entry.id))
            .content(entry)
            .await?;
        created.into_iter().next()
            .ok_or_else(|| Error::Database("Failed to create ledger entry".to_string()))
    }

    pub async fn get_ledger(&self, kid_id: Uuid) -> Result<Ledger> {
        let mut response = self.db
            .query("SELECT * FROM ledger_entry WHERE kid_id = $kid_id ORDER BY created_at ASC")
            .bind(("kid_id", kid_id))
            .await?;

        let entries: Vec<LedgerEntry> = response.take(0)?;
        Ok(Ledger::new(kid_id, entries))
    }

    pub async fn list_entries(&self, kid_id: Uuid) -> Result<Vec<LedgerEntry>> {
        let mut response = self.db
            .query("SELECT * FROM ledger_entry WHERE kid_id = $kid_id ORDER BY created_at DESC")
            .bind(("kid_id", kid_id))
            .await?;

        let entries: Vec<LedgerEntry> = response.take(0)?;
        Ok(entries)
    }
}
