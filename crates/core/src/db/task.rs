use surrealdb::Surreal;
use surrealdb::engine::local::RocksDb;
use crate::models::Task;
use crate::error::{Error, Result};
use uuid::Uuid;
use std::sync::Arc;

pub struct TaskRepository {
    db: Arc<Surreal<RocksDb>>,
}

impl TaskRepository {
    pub fn new(db: Arc<Surreal<RocksDb>>) -> Self {
        Self { db }
    }

    pub async fn create(&self, task: Task) -> Result<Task> {
        let created: Vec<Task> = self.db
            .create(("task", task.id))
            .content(task)
            .await?;
        created.into_iter().next()
            .ok_or_else(|| Error::Database("Failed to create task".to_string()))
    }

    pub async fn get(&self, id: Uuid) -> Result<Task> {
        let task: Option<Task> = self.db
            .select(("task", id))
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
        let updated: Option<Task> = self.db
            .update(("task", task.id))
            .content(task)
            .await?;
        updated.ok_or_else(|| Error::NotFound(format!("Task with id {}", task.id)))
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let _deleted: Option<Task> = self.db
            .delete(("task", id))
            .await?;
        Ok(())
    }
}

