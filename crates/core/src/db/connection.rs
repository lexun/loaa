use surrealdb::opt::auth::Root;
use surrealdb::{Surreal, engine::remote::ws::{Client, Ws}};
use crate::error::{Error, Result};
use std::sync::Arc;

pub struct Database {
    pub client: Arc<Surreal<Client>>,
}

impl Database {
    pub async fn init(url: &str) -> Result<Self> {
        let db = Surreal::new::<Ws>(url)
            .await
            .map_err(|e| Error::Database(format!("Failed to connect to database: {}", e)))?;

        // Sign in as root user
        db.signin(Root {
            username: "root",
            password: "root",
        })
        .await
        .map_err(|e| Error::Database(format!("Failed to authenticate: {}", e)))?;

        db.use_ns("loaa").use_db("main").await
            .map_err(|e| Error::Database(format!("Failed to set namespace/database: {}", e)))?;

        Ok(Self { client: Arc::new(db) })
    }
}

pub async fn init_database(url: &str) -> Result<Database> {
    Database::init(url).await
}

