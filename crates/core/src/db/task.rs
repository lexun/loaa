use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use crate::models::Task;
use crate::error::{Error, Result};
use uuid::Uuid;
use std::sync::Arc;

pub struct TaskRepository {
    db: Arc<Surreal<Db>>,
}

impl TaskRepository {
    pub fn new(db: Arc<Surreal<Db>>) -> Self {
        Self { db }
    }

    pub async fn create(&self, task: Task) -> Result<Task> {
        let created: Option<Task> = self.db
            .create(("task", task.id.to_string()))
            .content(task)
            .await?;
        created.ok_or_else(|| Error::Database("Failed to create task".to_string()))
    }

    pub async fn get(&self, id: Uuid) -> Result<Task> {
        let task: Option<Task> = self.db
            .select(("task", id.to_string()))
            .await?;
        task.ok_or_else(|| Error::NotFound(format!("Task with id {}", id)))
    }

    pub async fn list(&self) -> Result<Vec<Task>> {
        let tasks: Vec<Task> = self.db
            .select("task")
            .await?;
        Ok(tasks)
    }

    pub async fn update(&self, task: Task) -> Result<Task> {
        let task_id = task.id;
        let updated: Option<Task> = self.db
            .update(("task", task_id.to_string()))
            .content(task)
            .await?;
        updated.ok_or_else(|| Error::NotFound(format!("Task with id {}", task_id)))
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let _deleted: Option<Task> = self.db
            .delete(("task", id.to_string()))
            .await?;
        Ok(())
    }
}

