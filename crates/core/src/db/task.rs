use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::sql::Thing;
use crate::models::Task;
use crate::error::{Error, Result};
use uuid::Uuid;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use base64::Engine;

// Helper struct to handle SurrealDB record with id
#[derive(Debug, Serialize, Deserialize)]
struct TaskRecord {
    id: Thing,
    #[serde(flatten)]
    task: Task,
}

impl TaskRecord {
    fn into_task(self) -> Task {
        let mut task = self.task;
        // Extract UUID from SurrealDB Thing
        // SurrealDB wraps the ID in angle brackets: ⟨uuid⟩
        let id_str = self.id.id.to_string();
        let clean_id = id_str.trim_start_matches('⟨').trim_end_matches('⟩');
        task.id = Uuid::parse_str(clean_id)
            .unwrap_or_else(|_| Uuid::nil());
        task
    }
}

pub struct TaskRepository {
    db: Arc<Surreal<Any>>,
}

impl TaskRepository {
    pub fn new(db: Arc<Surreal<Any>>) -> Self {
        Self { db }
    }

    pub async fn create(&self, task: Task) -> Result<Task> {
        let task_id = task.id.to_string();
        let created: Option<TaskRecord> = self.db
            .create(("task", &task_id))
            .content(task)
            .await?;

        created
            .map(|rec| rec.into_task())
            .ok_or_else(|| Error::Database("Failed to create task".to_string()))
    }

    pub async fn get(&self, id: Uuid) -> Result<Task> {
        let record: Option<TaskRecord> = self.db
            .select(("task", id.to_string()))
            .await?;

        record
            .map(|rec| rec.into_task())
            .ok_or_else(|| Error::NotFound(format!("Task with id {}", id)))
    }

    pub async fn list(&self) -> Result<Vec<Task>> {
        let records: Vec<TaskRecord> = self.db
            .select("task")
            .await?;

        Ok(records.into_iter().map(|rec| rec.into_task()).collect())
    }

    pub async fn list_by_owner(&self, owner_id: &str) -> Result<Vec<Task>> {
        let records: Vec<TaskRecord> = self.db
            .query("SELECT * FROM task WHERE owner_id = $owner_id")
            .bind(("owner_id", owner_id.to_string()))
            .await?
            .take(0)?;

        Ok(records.into_iter().map(|rec| rec.into_task()).collect())
    }

    pub async fn list_by_account(&self, account_id: Uuid) -> Result<Vec<Task>> {
        // Include tasks with matching account_id OR nil account_id (backward compatibility)
        // Handle native UUID, string-stored UUID, and bytes-stored UUID formats in SurrealDB
        // Historical data may have account_id stored in different formats due to driver changes
        let nil_uuid = Uuid::nil();
        let account_id_str = account_id.to_string();
        let nil_uuid_str = nil_uuid.to_string();
        // For bytes comparison, use base64 encoding of UUID bytes (no padding to match SurrealDB format)
        let account_id_b64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode(account_id.as_bytes());
        let nil_uuid_b64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode(nil_uuid.as_bytes());
        let records: Vec<TaskRecord> = self.db
            .query("SELECT * FROM task WHERE
                account_id = type::uuid($account_id) OR
                (type::is::string(account_id) AND account_id = $account_id) OR
                (account_id IS NOT NONE AND encoding::base64::encode(account_id) = $account_id_b64) OR
                account_id = type::uuid($nil_uuid) OR
                (type::is::string(account_id) AND account_id = $nil_uuid) OR
                (account_id IS NOT NONE AND encoding::base64::encode(account_id) = $nil_uuid_b64) OR
                account_id IS NONE")
            .bind(("account_id", account_id_str))
            .bind(("account_id_b64", account_id_b64))
            .bind(("nil_uuid", nil_uuid_str))
            .bind(("nil_uuid_b64", nil_uuid_b64))
            .await?
            .take(0)?;

        Ok(records.into_iter().map(|rec| rec.into_task()).collect())
    }

    pub async fn update(&self, task: Task) -> Result<Task> {
        let task_id = task.id;

        // First check if the task exists
        let _existing: Task = self.get(task_id).await?;

        // If it exists, update it
        let updated: Option<TaskRecord> = self.db
            .update(("task", task_id.to_string()))
            .content(task)
            .await?;

        updated
            .map(|rec| rec.into_task())
            .ok_or_else(|| Error::NotFound(format!("Task with id {}", task_id)))
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let _deleted: Option<TaskRecord> = self.db
            .delete(("task", id.to_string()))
            .await?;
        Ok(())
    }
}

