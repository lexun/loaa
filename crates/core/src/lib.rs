pub mod models;
pub mod db;
pub mod error;
pub mod workflows;
pub mod config;
pub mod auth;

pub use models::*;
pub use error::{Error, Result};
pub use db::{Database, init_database, init_database_with_config, KidRepository, TaskRepository, LedgerRepository, UserRepository};
pub use workflows::TaskCompletionWorkflow;
pub use config::{Config, DatabaseConfig, DatabaseMode, ServerConfig};
pub use auth::{hash_password, verify_password};

// Re-export uuid for convenience
pub use uuid::Uuid;

