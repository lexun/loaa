use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Client;
use surrealdb::sql::Thing;
use crate::models::Kid;
use crate::error::{Error, Result};
use uuid::Uuid;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

// Helper struct to handle SurrealDB record with id
#[derive(Debug, Serialize, Deserialize)]
struct KidRecord {
    id: Thing,
    #[serde(flatten)]
    kid: Kid,
}

impl KidRecord {
    fn into_kid(self) -> Kid {
        let mut kid = self.kid;
        // Extract UUID from SurrealDB Thing
        // SurrealDB wraps the ID in angle brackets: ⟨uuid⟩
        let id_str = self.id.id.to_string();
        let clean_id = id_str.trim_start_matches('⟨').trim_end_matches('⟩');
        kid.id = Uuid::parse_str(clean_id)
            .unwrap_or_else(|_| Uuid::nil());
        kid
    }
}

pub struct KidRepository {
    db: Arc<Surreal<Client>>,
}

impl KidRepository {
    pub fn new(db: Arc<Surreal<Client>>) -> Self {
        Self { db }
    }

    pub async fn create(&self, kid: Kid) -> Result<Kid> {
        let kid_id = kid.id.to_string();
        let created: Option<KidRecord> = self.db
            .create(("kid", &kid_id))
            .content(&kid)
            .await?;

        created
            .map(|rec| rec.into_kid())
            .ok_or_else(|| Error::Database("Failed to create kid".to_string()))
    }

    pub async fn get(&self, id: Uuid) -> Result<Kid> {
        let record: Option<KidRecord> = self.db
            .select(("kid", id.to_string()))
            .await?;

        record
            .map(|rec| rec.into_kid())
            .ok_or_else(|| Error::NotFound(format!("Kid with id {}", id)))
    }

    pub async fn list(&self) -> Result<Vec<Kid>> {
        let records: Vec<KidRecord> = self.db
            .select("kid")
            .await?;

        Ok(records.into_iter().map(|rec| rec.into_kid()).collect())
    }

    pub async fn update(&self, kid: Kid) -> Result<Kid> {
        let kid_id = kid.id;

        // First check if the kid exists
        let _existing: Kid = self.get(kid_id).await?;

        // If it exists, update it
        let updated: Option<KidRecord> = self.db
            .update(("kid", kid_id.to_string()))
            .content(&kid)
            .await?;

        updated
            .map(|rec| rec.into_kid())
            .ok_or_else(|| Error::NotFound(format!("Kid with id {}", kid_id)))
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let _deleted: Option<KidRecord> = self.db
            .delete(("kid", id.to_string()))
            .await?;
        Ok(())
    }
}

