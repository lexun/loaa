use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::sql::Thing;
use crate::models::Kid;
use crate::error::{Error, Result};
use uuid::Uuid;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use base64::Engine;

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
    db: Arc<Surreal<Any>>,
}

impl KidRepository {
    pub fn new(db: Arc<Surreal<Any>>) -> Self {
        Self { db }
    }

    pub async fn create(&self, kid: Kid) -> Result<Kid> {
        let kid_id = kid.id.to_string();
        let created: Option<KidRecord> = self.db
            .create(("kid", &kid_id))
            .content(kid)
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

    pub async fn list_by_owner(&self, owner_id: &str) -> Result<Vec<Kid>> {
        let records: Vec<KidRecord> = self.db
            .query("SELECT * FROM kid WHERE owner_id = $owner_id")
            .bind(("owner_id", owner_id.to_string()))
            .await?
            .take(0)?;

        Ok(records.into_iter().map(|rec| rec.into_kid()).collect())
    }

    pub async fn list_by_account(&self, account_id: Uuid) -> Result<Vec<Kid>> {
        // Include kids with matching account_id OR nil account_id (backward compatibility)
        // Handle native UUID, string-stored UUID, and bytes-stored UUID formats in SurrealDB
        // Historical data may have account_id stored in different formats due to driver changes
        let nil_uuid = Uuid::nil();
        let account_id_str = account_id.to_string();
        let nil_uuid_str = nil_uuid.to_string();
        // For bytes comparison, use base64 encoding of UUID bytes (no padding to match SurrealDB format)
        let account_id_b64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode(account_id.as_bytes());
        let nil_uuid_b64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode(nil_uuid.as_bytes());
        let records: Vec<KidRecord> = self.db
            .query("SELECT * FROM kid WHERE
                account_id = type::uuid($account_id) OR
                (type::is::string(account_id) AND account_id = $account_id) OR
                (type::is::bytes(account_id) AND encoding::base64::encode(account_id) = $account_id_b64) OR
                account_id = type::uuid($nil_uuid) OR
                (type::is::string(account_id) AND account_id = $nil_uuid) OR
                (type::is::bytes(account_id) AND encoding::base64::encode(account_id) = $nil_uuid_b64) OR
                account_id IS NONE")
            .bind(("account_id", account_id_str))
            .bind(("account_id_b64", account_id_b64))
            .bind(("nil_uuid", nil_uuid_str))
            .bind(("nil_uuid_b64", nil_uuid_b64))
            .await?
            .take(0)?;

        Ok(records.into_iter().map(|rec| rec.into_kid()).collect())
    }

    pub async fn update(&self, kid: Kid) -> Result<Kid> {
        let kid_id = kid.id;

        // First check if the kid exists
        let _existing: Kid = self.get(kid_id).await?;

        // If it exists, update it
        let updated: Option<KidRecord> = self.db
            .update(("kid", kid_id.to_string()))
            .content(kid)
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

