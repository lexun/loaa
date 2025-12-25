pub mod models;
pub mod db;
pub mod error;
pub mod workflows;
pub mod config;
pub mod auth;
pub mod events;

pub use models::*;
pub use error::{Error, Result};
pub use db::{Database, init_database, init_database_with_config, KidRepository, TaskRepository, LedgerRepository, UserRepository};
pub use workflows::TaskCompletionWorkflow;
pub use config::{Config, DatabaseConfig, DatabaseMode, ServerConfig};
pub use auth::{hash_password, verify_password};
pub use events::{DataEvent, EventSender, EventReceiver, create_event_channel, broadcast_event};

// Re-export uuid for convenience
pub use uuid::Uuid;

