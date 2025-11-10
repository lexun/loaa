use surrealdb::{Surreal, engine::local::RocksDb};
use crate::error::{Error, Result};
use std::path::PathBuf;
use std::sync::Arc;

pub struct Database {
    pub client: Arc<Surreal<RocksDb>>,
}

impl Database {
    pub async fn init(path: &str) -> Result<Self> {
        let db_path = PathBuf::from(path);
        std::fs::create_dir_all(&db_path)
            .map_err(|e| Error::Database(format!("Failed to create database directory: {}", e)))?;

        let db = Surreal::new::<RocksDb>(db_path)
            .await
            .map_err(|e| Error::Database(format!("Failed to initialize database: {}", e)))?;

        db.use_ns("loaa").use_db("main").await
            .map_err(|e| Error::Database(format!("Failed to set namespace/database: {}", e)))?;

        Ok(Self { client: Arc::new(db) })
    }
}

pub async fn init_database(path: &str) -> Result<Database> {
    Database::init(path).await
}

