use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::sql::Thing;
use crate::models::{LedgerEntry, Ledger, TransactionStatus};
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
    db: Arc<Surreal<Any>>,
}

impl LedgerRepository {
    pub fn new(db: Arc<Surreal<Any>>) -> Self {
        Self { db }
    }

    pub async fn create_entry(&self, entry: LedgerEntry) -> Result<LedgerEntry> {
        let entry_id = entry.id.to_string();
        let created: Option<LedgerEntryRecord> = self.db
            .create(("ledger_entry", &entry_id))
            .content(entry)
            .await?;

        created
            .map(|rec| rec.into_entry())
            .ok_or_else(|| Error::Database("Failed to create ledger entry".to_string()))
    }

    pub async fn get_ledger(&self, kid_id: Uuid) -> Result<Ledger> {
        // Sort by effective date: completed_at if backdated, otherwise created_at
        let mut response = self.db
            .query("SELECT * FROM ledger_entry WHERE string::lowercase(kid_id) = string::lowercase($kid_id) ORDER BY completed_at ?? created_at DESC")
            .bind(("kid_id", kid_id.to_string()))
            .await?;

        let records: Vec<LedgerEntryRecord> = response.take(0)?;
        let entries: Vec<LedgerEntry> = records.into_iter().map(|rec| rec.into_entry()).collect();
        Ok(Ledger::new(kid_id, entries))
    }

    pub async fn list_entries(&self, kid_id: Uuid) -> Result<Vec<LedgerEntry>> {
        // Sort by effective date: completed_at if backdated, otherwise created_at
        let mut response = self.db
            .query("SELECT * FROM ledger_entry WHERE string::lowercase(kid_id) = string::lowercase($kid_id) ORDER BY completed_at ?? created_at DESC")
            .bind(("kid_id", kid_id.to_string()))
            .await?;

        let records: Vec<LedgerEntryRecord> = response.take(0)?;
        Ok(records.into_iter().map(|rec| rec.into_entry()).collect())
    }

    /// Get a single ledger entry by ID
    pub async fn get_entry(&self, entry_id: Uuid) -> Result<Option<LedgerEntry>> {
        let record: Option<LedgerEntryRecord> = self.db
            .select(("ledger_entry", entry_id.to_string()))
            .await?;

        Ok(record.map(|rec| rec.into_entry()))
    }

    /// List pending transactions for all kids in an account
    pub async fn list_pending_for_kids(&self, kid_ids: &[Uuid]) -> Result<Vec<LedgerEntry>> {
        if kid_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Convert UUIDs to lowercase strings for query
        let kid_id_strs: Vec<String> = kid_ids.iter().map(|id| id.to_string().to_lowercase()).collect();

        let mut response = self.db
            .query("SELECT * FROM ledger_entry WHERE string::lowercase(kid_id) IN $kid_ids AND status = 'Pending' ORDER BY completed_at ?? created_at DESC")
            .bind(("kid_ids", kid_id_strs))
            .await?;

        let records: Vec<LedgerEntryRecord> = response.take(0)?;
        Ok(records.into_iter().map(|rec| rec.into_entry()).collect())
    }

    /// Update the status of a ledger entry (for approval workflow)
    pub async fn update_status(&self, entry_id: Uuid, status: TransactionStatus) -> Result<LedgerEntry> {
        let mut response = self.db
            .query("UPDATE ledger_entry SET status = $status WHERE id = type::thing('ledger_entry', $id)")
            .bind(("id", entry_id.to_string()))
            .bind(("status", status))
            .await?;

        let records: Vec<LedgerEntryRecord> = response.take(0)?;
        records
            .into_iter()
            .next()
            .map(|rec| rec.into_entry())
            .ok_or_else(|| Error::NotFound(format!("Ledger entry {} not found", entry_id)))
    }

    /// Delete a ledger entry (for rejection workflow)
    pub async fn delete(&self, entry_id: Uuid) -> Result<()> {
        let _: Option<LedgerEntryRecord> = self.db
            .delete(("ledger_entry", entry_id.to_string()))
            .await?;
        Ok(())
    }
}
