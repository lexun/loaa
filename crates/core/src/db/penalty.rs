use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::sql::Thing;
use crate::models::Penalty;
use crate::error::{Error, Result};
use uuid::Uuid;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

// Helper struct to handle SurrealDB record with id
#[derive(Debug, Serialize, Deserialize)]
struct PenaltyRecord {
    id: Thing,
    #[serde(flatten)]
    penalty: Penalty,
}

impl PenaltyRecord {
    fn into_penalty(self) -> Penalty {
        let mut penalty = self.penalty;
        // Extract UUID from SurrealDB Thing
        let id_str = self.id.id.to_string();
        let clean_id = id_str.trim_start_matches('⟨').trim_end_matches('⟩');
        penalty.id = Uuid::parse_str(clean_id)
            .unwrap_or_else(|_| Uuid::nil());
        penalty
    }
}

pub struct PenaltyRepository {
    db: Arc<Surreal<Any>>,
}

impl PenaltyRepository {
    pub fn new(db: Arc<Surreal<Any>>) -> Self {
        Self { db }
    }

    pub async fn create(&self, penalty: Penalty) -> Result<Penalty> {
        let penalty_id = penalty.id.to_string();
        let created: Option<PenaltyRecord> = self.db
            .create(("penalty", &penalty_id))
            .content(penalty)
            .await?;

        created
            .map(|rec| rec.into_penalty())
            .ok_or_else(|| Error::Database("Failed to create penalty".to_string()))
    }

    pub async fn get(&self, id: Uuid) -> Result<Penalty> {
        let record: Option<PenaltyRecord> = self.db
            .select(("penalty", id.to_string()))
            .await?;

        record
            .map(|rec| rec.into_penalty())
            .ok_or_else(|| Error::NotFound(format!("Penalty with id {}", id)))
    }

    pub async fn list(&self) -> Result<Vec<Penalty>> {
        let records: Vec<PenaltyRecord> = self.db
            .select("penalty")
            .await?;

        Ok(records.into_iter().map(|rec| rec.into_penalty()).collect())
    }

    pub async fn list_by_account(&self, account_id: Uuid) -> Result<Vec<Penalty>> {
        // Since penalties are new, we only need to handle string-stored UUIDs
        let account_id_str = account_id.to_string();
        let records: Vec<PenaltyRecord> = self.db
            .query("SELECT * FROM penalty WHERE account_id = type::uuid($account_id)")
            .bind(("account_id", account_id_str))
            .await?
            .take(0)?;

        Ok(records.into_iter().map(|rec| rec.into_penalty()).collect())
    }

    pub async fn update(&self, penalty: Penalty) -> Result<Penalty> {
        let penalty_id = penalty.id;

        // First check if the penalty exists
        let _existing: Penalty = self.get(penalty_id).await?;

        // If it exists, update it
        let updated: Option<PenaltyRecord> = self.db
            .update(("penalty", penalty_id.to_string()))
            .content(penalty)
            .await?;

        updated
            .map(|rec| rec.into_penalty())
            .ok_or_else(|| Error::NotFound(format!("Penalty with id {}", penalty_id)))
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let _deleted: Option<PenaltyRecord> = self.db
            .delete(("penalty", id.to_string()))
            .await?;
        Ok(())
    }
}
