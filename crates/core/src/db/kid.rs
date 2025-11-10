use surrealdb::Surreal;
use surrealdb::engine::local::RocksDb;
use crate::models::Kid;
use crate::error::{Error, Result};
use uuid::Uuid;
use std::sync::Arc;

pub struct KidRepository {
    db: Arc<Surreal<RocksDb>>,
}

impl KidRepository {
    pub fn new(db: Arc<Surreal<RocksDb>>) -> Self {
        Self { db }
    }

    pub async fn create(&self, kid: Kid) -> Result<Kid> {
        let created: Vec<Kid> = self.db
            .create(("kid", kid.id))
            .content(kid)
            .await?;
        created.into_iter().next()
            .ok_or_else(|| Error::Database("Failed to create kid".to_string()))
    }

    pub async fn get(&self, id: Uuid) -> Result<Kid> {
        let kid: Option<Kid> = self.db
            .select(("kid", id))
            .await?;
        kid.ok_or_else(|| Error::NotFound(format!("Kid with id {}", id)))
    }

    pub async fn list(&self) -> Result<Vec<Kid>> {
        let kids: Vec<Kid> = self.db
            .select("kid")
            .await?;
        Ok(kids)
    }

    pub async fn update(&self, kid: Kid) -> Result<Kid> {
        let updated: Option<Kid> = self.db
            .update(("kid", kid.id))
            .content(kid)
            .await?;
        updated.ok_or_else(|| Error::NotFound(format!("Kid with id {}", kid.id)))
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let _deleted: Option<Kid> = self.db
            .delete(("kid", id))
            .await?;
        Ok(())
    }
}

