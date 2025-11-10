use leptos::*;
use loaa_core::{Database, KidRepository, TaskRepository, LedgerRepository, init_database};
use loaa_core::models::*;
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::OnceCell;

static DB_PATH: &str = ".data/db";

// Helper to get database connection
async fn get_db() -> Result<Arc<Database>, ServerFnError> {
    static DB: OnceCell<Arc<Database>> = OnceCell::const_new();
    DB.get_or_try_init(|| async {
        init_database(DB_PATH).await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))
            .map(Arc::new)
    })
    .await
    .cloned()
}

#[server]
pub async fn get_kids() -> Result<Vec<Kid>, ServerFnError> {
    let db = get_db().await?;
    let kid_repo = KidRepository::new(db.client.clone());
    kid_repo.list().await
        .map_err(|e| ServerFnError::new(format!("Failed to list kids: {}", e)))
}

#[server]
pub async fn create_kid(name: String) -> Result<Kid, ServerFnError> {
    let kid = Kid::new(name)
        .map_err(|e| ServerFnError::new(format!("Validation error: {}", e)))?;
    let db = get_db().await?;
    let kid_repo = KidRepository::new(db.client.clone());
    kid_repo.create(kid).await
        .map_err(|e| ServerFnError::new(format!("Failed to create kid: {}", e)))
}

#[server]
pub async fn get_tasks() -> Result<Vec<Task>, ServerFnError> {
    let db = get_db().await?;
    let task_repo = TaskRepository::new(db.client.clone());
    task_repo.list().await
        .map_err(|e| ServerFnError::new(format!("Failed to list tasks: {}", e)))
}

#[server]
pub async fn create_task(
    name: String,
    description: String,
    value: rust_decimal::Decimal,
    cadence: Cadence,
) -> Result<Task, ServerFnError> {
    let task = Task::new(name, description, value, cadence)
        .map_err(|e| ServerFnError::new(format!("Validation error: {}", e)))?;
    let db = get_db().await?;
    let task_repo = TaskRepository::new(db.client.clone());
    task_repo.create(task).await
        .map_err(|e| ServerFnError::new(format!("Failed to create task: {}", e)))
}

#[server]
pub async fn complete_task(kid_id: Uuid, task_id: Uuid) -> Result<(), ServerFnError> {
    let db = get_db().await?;

    let task_repo = TaskRepository::new(db.client.clone());
    let task = task_repo.get(task_id).await
        .map_err(|e| ServerFnError::new(format!("Task not found: {}", e)))?;

    let ledger_repo = LedgerRepository::new(db.client.clone());
    let entry = LedgerEntry::earned(
        kid_id,
        task.value,
        format!("Completed: {}", task.name),
    );
    ledger_repo.create_entry(entry).await
        .map_err(|e| ServerFnError::new(format!("Failed to create ledger entry: {}", e)))?;

    Ok(())
}

#[server]
pub async fn get_ledger(kid_id: Uuid) -> Result<Ledger, ServerFnError> {
    let db = get_db().await?;
    let ledger_repo = LedgerRepository::new(db.client.clone());
    ledger_repo.get_ledger(kid_id).await
        .map_err(|e| ServerFnError::new(format!("Failed to get ledger: {}", e)))
}
