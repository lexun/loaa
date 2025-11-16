use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use surrealdb::engine::any::{self, Any};
use crate::config::{DatabaseConfig, DatabaseMode};
use crate::error::{Error, Result};
use std::sync::Arc;

pub struct Database {
    pub client: Arc<Surreal<Any>>,
}

impl Database {
    /// Initialize database with configuration
    pub async fn init_with_config(config: &DatabaseConfig) -> Result<Self> {
        let connection_str: String = match &config.mode {
            DatabaseMode::Memory => "memory".to_string(),
            DatabaseMode::Embedded => {
                let path = config.path.as_ref()
                    .ok_or_else(|| Error::Database("Embedded mode requires database path".to_string()))?;

                // Ensure parent directory exists
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| Error::Database(format!("Failed to create database directory: {}", e)))?;
                }

                // Return path as string for RocksDB
                format!("rocksdb://{}", path.display())
            }
            DatabaseMode::Remote => {
                let url = config.url.as_ref()
                    .ok_or_else(|| Error::Database("Remote mode requires database URL".to_string()))?;
                format!("ws://{}", url)
            }
        };

        let db = any::connect(connection_str.as_str())
            .await
            .map_err(|e| Error::Database(format!("Failed to connect to database: {}", e)))?;

        // Sign in as root user for remote databases only
        if matches!(config.mode, DatabaseMode::Remote) {
            db.signin(Root {
                username: "root",
                password: "root",
            })
            .await
            .map_err(|e| Error::Database(format!("Failed to authenticate: {}", e)))?;
        }

        // Set namespace and database for all modes
        db.use_ns("loaa").use_db("main").await
            .map_err(|e| Error::Database(format!("Failed to set namespace/database: {}", e)))?;

        Ok(Self { client: Arc::new(db) })
    }

    /// Legacy init method for backward compatibility (uses remote mode)
    pub async fn init(url: &str) -> Result<Self> {
        let config = DatabaseConfig {
            mode: DatabaseMode::Remote,
            url: Some(url.to_string()),
            path: None,
        };
        Self::init_with_config(&config).await
    }
}

/// Legacy function for backward compatibility
pub async fn init_database(url: &str) -> Result<Database> {
    Database::init(url).await
}

/// Initialize database with configuration
pub async fn init_database_with_config(config: &DatabaseConfig) -> Result<Database> {
    Database::init_with_config(config).await
}

