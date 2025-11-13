use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Client;
use surrealdb::sql::Thing;
use crate::models::{LedgerEntry, Ledger};
use crate::error::{Error, Result};
use uuid::Uuid;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

// Helper struct to handle SurrealDB record with id
#[derive(Debug, Serialize, Deserialize)]
struct LedgerEntryRecord {
    id: Thing,
    #[serde(flatten)]
    entry: LedgerEntry,
}

impl LedgerEntryRecord {
    fn into_entry(self) -> LedgerEntry {
        let mut entry = self.entry;
        // Extract UUID from SurrealDB Thing
        // SurrealDB wraps the ID in angle brackets: ⟨uuid⟩
        let id_str = self.id.id.to_string();
        let clean_id = id_str.trim_start_matches('⟨').trim_end_matches('⟩');
        entry.id = Uuid::parse_str(clean_id)
            .unwrap_or_else(|_| Uuid::nil());
        entry
    }
}

pub struct LedgerRepository {
    db: Arc<Surreal<Client>>,
}

impl LedgerRepository {
    pub fn new(db: Arc<Surreal<Client>>) -> Self {
        Self { db }
    }

    pub async fn create_entry(&self, entry: LedgerEntry) -> Result<LedgerEntry> {
        let entry_id = entry.id.to_string();
        let created: Option<LedgerEntryRecord> = self.db
            .create(("ledger_entry", &entry_id))
            .content(&entry)
            .await?;

        created
            .map(|rec| rec.into_entry())
            .ok_or_else(|| Error::Database("Failed to create ledger entry".to_string()))
    }

    pub async fn get_ledger(&self, kid_id: Uuid) -> Result<Ledger> {
        let mut response = self.db
            .query("SELECT * FROM ledger_entry WHERE string::lowercase(kid_id) = string::lowercase($kid_id) ORDER BY created_at ASC")
            .bind(("kid_id", kid_id.to_string()))
            .await?;

        let records: Vec<LedgerEntryRecord> = response.take(0)?;
        let entries: Vec<LedgerEntry> = records.into_iter().map(|rec| rec.into_entry()).collect();
        Ok(Ledger::new(kid_id, entries))
    }

    pub async fn list_entries(&self, kid_id: Uuid) -> Result<Vec<LedgerEntry>> {
        let mut response = self.db
            .query("SELECT * FROM ledger_entry WHERE string::lowercase(kid_id) = string::lowercase($kid_id) ORDER BY created_at DESC")
            .bind(("kid_id", kid_id.to_string()))
            .await?;

        let records: Vec<LedgerEntryRecord> = response.take(0)?;
        Ok(records.into_iter().map(|rec| rec.into_entry()).collect())
    }
}
