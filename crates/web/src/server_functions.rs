use leptos::*;
use loaa_core::{Database, KidRepository, TaskRepository, LedgerRepository, init_database, Uuid};
use loaa_core::models::*;
use std::sync::Arc;
use tokio::sync::OnceCell;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

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

// Dashboard data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KidSummary {
    pub kid: Kid,
    pub balance: Decimal,
    pub recent_entry: Option<LedgerEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub kid_summaries: Vec<KidSummary>,
    pub total_kids: usize,
    pub active_tasks: usize,
}

#[server]
pub async fn get_dashboard_data() -> Result<DashboardData, ServerFnError> {
    let db = get_db().await?;
    let kid_repo = KidRepository::new(db.client.clone());
    let task_repo = TaskRepository::new(db.client.clone());
    let ledger_repo = LedgerRepository::new(db.client.clone());

    let kids = kid_repo.list().await
        .map_err(|e| ServerFnError::new(format!("Failed to list kids: {}", e)))?;

    let tasks = task_repo.list().await
        .map_err(|e| ServerFnError::new(format!("Failed to list tasks: {}", e)))?;

    let mut kid_summaries = Vec::new();
    for kid in kids.iter() {
        let ledger = ledger_repo.get_ledger(kid.id).await
            .map_err(|e| ServerFnError::new(format!("Failed to get ledger: {}", e)))?;

        let recent_entry = ledger.entries.last().cloned();

        kid_summaries.push(KidSummary {
            kid: kid.clone(),
            balance: ledger.balance,
            recent_entry,
        });
    }

    Ok(DashboardData {
        kid_summaries,
        total_kids: kids.len(),
        active_tasks: tasks.len(),
    })
}

#[server]
pub async fn get_recent_activity(limit: usize) -> Result<Vec<LedgerEntry>, ServerFnError> {
    let db = get_db().await?;
    let kid_repo = KidRepository::new(db.client.clone());
    let ledger_repo = LedgerRepository::new(db.client.clone());

    let kids = kid_repo.list().await
        .map_err(|e| ServerFnError::new(format!("Failed to list kids: {}", e)))?;

    let mut all_entries = Vec::new();
    for kid in kids {
        let entries = ledger_repo.list_entries(kid.id).await
            .map_err(|e| ServerFnError::new(format!("Failed to get ledger entries: {}", e)))?;
        all_entries.extend(entries);
    }

    all_entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    all_entries.truncate(limit);

    Ok(all_entries)
}
