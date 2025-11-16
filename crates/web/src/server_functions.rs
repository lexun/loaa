use leptos::*;
use crate::dto::*;

#[cfg(feature = "ssr")]
use loaa_core::{Database, KidRepository, TaskRepository, LedgerRepository, init_database, Uuid};
#[cfg(feature = "ssr")]
use loaa_core::models::*;
#[cfg(feature = "ssr")]
use loaa_core::workflows::TaskCompletionWorkflow;
#[cfg(feature = "ssr")]
use std::sync::Arc;
#[cfg(feature = "ssr")]
use tokio::sync::OnceCell;
#[cfg(feature = "ssr")]
use std::str::FromStr;

#[cfg(feature = "ssr")]
static DB_URL: &str = "127.0.0.1:8000";

// Helper to get database connection
#[cfg(feature = "ssr")]
async fn get_db() -> Result<Arc<Database>, ServerFnError> {
    static DB: OnceCell<Arc<Database>> = OnceCell::const_new();
    DB.get_or_try_init(|| async {
        init_database(DB_URL).await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))
            .map(Arc::new)
    })
    .await
    .cloned()
}

#[server]
pub async fn get_kids() -> Result<Vec<KidDto>, ServerFnError> {
    let db = get_db().await?;
    let kid_repo = KidRepository::new(db.client.clone());
    let kids = kid_repo.list().await
        .map_err(|e| ServerFnError::new(format!("Failed to list kids: {}", e)))?;
    Ok(kids.into_iter().map(Into::into).collect())
}

#[server]
pub async fn create_kid(name: String) -> Result<KidDto, ServerFnError> {
    let kid = Kid::new(name)
        .map_err(|e| ServerFnError::new(format!("Validation error: {}", e)))?;
    let db = get_db().await?;
    let kid_repo = KidRepository::new(db.client.clone());
    let created = kid_repo.create(kid).await
        .map_err(|e| ServerFnError::new(format!("Failed to create kid: {}", e)))?;
    Ok(created.into())
}

#[server]
pub async fn get_tasks() -> Result<Vec<TaskDto>, ServerFnError> {
    let db = get_db().await?;
    let task_repo = TaskRepository::new(db.client.clone());
    let tasks = task_repo.list().await
        .map_err(|e| ServerFnError::new(format!("Failed to list tasks: {}", e)))?;
    Ok(tasks.into_iter().map(Into::into).collect())
}

#[server]
pub async fn create_task(
    name: String,
    description: String,
    value: rust_decimal::Decimal,
    cadence: CadenceDto,
) -> Result<TaskDto, ServerFnError> {
    let task = Task::new(name, description, value, cadence.into())
        .map_err(|e| ServerFnError::new(format!("Validation error: {}", e)))?;
    let db = get_db().await?;
    let task_repo = TaskRepository::new(db.client.clone());
    let created = task_repo.create(task).await
        .map_err(|e| ServerFnError::new(format!("Failed to create task: {}", e)))?;
    Ok(created.into())
}

#[server]
pub async fn complete_task(kid_id: UuidDto, task_id: UuidDto) -> Result<(), ServerFnError> {
    let db = get_db().await?;

    let kid_uuid = Uuid::from_str(&kid_id)
        .map_err(|e| ServerFnError::new(format!("Invalid kid ID: {}", e)))?;
    let task_uuid = Uuid::from_str(&task_id)
        .map_err(|e| ServerFnError::new(format!("Invalid task ID: {}", e)))?;

    // Use the TaskCompletionWorkflow to handle task completion
    // This ensures recurring tasks are properly reset
    let task_repo = TaskRepository::new(db.client.clone());
    let kid_repo = KidRepository::new(db.client.clone());
    let ledger_repo = LedgerRepository::new(db.client.clone());

    let workflow = TaskCompletionWorkflow::new(task_repo, kid_repo, ledger_repo);
    workflow.complete_task(task_uuid, kid_uuid).await
        .map_err(|e| ServerFnError::new(format!("Failed to complete task: {}", e)))?;

    Ok(())
}

#[server]
pub async fn get_ledger(kid_id: UuidDto) -> Result<LedgerDto, ServerFnError> {
    let db = get_db().await?;
    let kid_uuid = Uuid::from_str(&kid_id)
        .map_err(|e| ServerFnError::new(format!("Invalid kid ID: {}", e)))?;
    let ledger_repo = LedgerRepository::new(db.client.clone());
    let ledger = ledger_repo.get_ledger(kid_uuid).await
        .map_err(|e| ServerFnError::new(format!("Failed to get ledger: {}", e)))?;
    Ok(ledger.into())
}

#[server]
pub async fn get_dashboard_data() -> Result<DashboardDataDto, ServerFnError> {
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

        let recent_entry = ledger.entries.last().cloned().map(Into::into);

        kid_summaries.push(KidSummaryDto {
            kid: kid.clone().into(),
            balance: ledger.balance,
            recent_entry,
        });
    }

    Ok(DashboardDataDto {
        kid_summaries,
        total_kids: kids.len(),
        active_tasks: tasks.len(),
    })
}

#[server]
pub async fn get_recent_activity(limit: usize) -> Result<Vec<LedgerEntryDto>, ServerFnError> {
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

    Ok(all_entries.into_iter().map(Into::into).collect())
}
