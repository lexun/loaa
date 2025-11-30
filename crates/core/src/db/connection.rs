use surrealdb::opt::auth::{Root, Namespace};
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
                // Support both ws:// and wss:// prefixes, or add ws:// if missing
                if url.starts_with("ws://") || url.starts_with("wss://") {
                    url.clone()
                } else {
                    format!("ws://{}", url)
                }
            }
        };

        let db = any::connect(connection_str.as_str())
            .await
            .map_err(|e| Error::Database(format!("Failed to connect to database: {}", e)))?;

        // Sign in for remote databases only
        if matches!(config.mode, DatabaseMode::Remote) {
            // Use token authentication if provided (for SurrealDB Cloud)
            if let Some(token) = &config.token {
                db.authenticate(token.clone())
                    .await
                    .map_err(|e| Error::Database(format!("Failed to authenticate with token: {}", e)))?;
            } else {
                // Namespace-level authentication
                if let Some(namespace) = &config.namespace {
                    let username = config.username.as_deref().unwrap_or("root");
                    let password = config.password.as_deref().unwrap_or("root");

                    db.signin(Namespace {
                        namespace,
                        username,
                        password,
                    })
                    .await
                    .map_err(|e| Error::Database(format!("Failed to authenticate as namespace user: {}", e)))?;
                } else {
                    // Root-level authentication
                    let username = config.username.as_deref().unwrap_or("root");
                    let password = config.password.as_deref().unwrap_or("root");

                    db.signin(Root {
                        username,
                        password,
                    })
                    .await
                    .map_err(|e| Error::Database(format!("Failed to authenticate as root: {}", e)))?;
                }
            }
        }

        // Set namespace and database for all modes
        let namespace = config.namespace.as_deref().unwrap_or("loaa");
        let database = config.database.as_deref().unwrap_or("main");

        db.use_ns(namespace).use_db(database).await
            .map_err(|e| Error::Database(format!("Failed to set namespace/database: {}", e)))?;

        Ok(Self { client: Arc::new(db) })
    }

    /// Legacy init method for backward compatibility (uses remote mode)
    pub async fn init(url: &str) -> Result<Self> {
        let config = DatabaseConfig {
            mode: DatabaseMode::Remote,
            url: Some(url.to_string()),
            path: None,
            namespace: None,
            database: None,
            username: None,
            password: None,
            token: None,
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

