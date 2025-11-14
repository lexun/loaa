use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Client;
use surrealdb::sql::Thing;
use crate::models::Task;
use crate::error::{Error, Result};
use uuid::Uuid;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

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
    db: Arc<Surreal<Client>>,
}

impl TaskRepository {
    pub fn new(db: Arc<Surreal<Client>>) -> Self {
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

