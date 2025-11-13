use crate::db::{TaskRepository, KidRepository, LedgerRepository};
use crate::models::LedgerEntry;
use crate::error::Result;
use uuid::Uuid;

/// Coordinates task completion workflow:
/// 1. Mark task as complete (create ledger entry)
/// 2. Reset task if it's a recurring task
pub struct TaskCompletionWorkflow {
    task_repo: TaskRepository,
    kid_repo: KidRepository,
    ledger_repo: LedgerRepository,
}

impl TaskCompletionWorkflow {
    pub fn new(
        task_repo: TaskRepository,
        kid_repo: KidRepository,
        ledger_repo: LedgerRepository,
    ) -> Self {
        Self {
            task_repo,
            kid_repo,
            ledger_repo,
        }
    }

    /// Complete a task for a kid
    /// - Creates a ledger entry with the task's value
    /// - Resets the task if it's a recurring cadence
    /// Returns the created ledger entry
    pub async fn complete_task(&self, task_id: Uuid, kid_id: Uuid) -> Result<LedgerEntry> {
        // 1. Verify the kid exists
        let _kid = self.kid_repo.get(kid_id).await?;

        // 2. Get the task
        let mut task = self.task_repo.get(task_id).await?;

        // 3. Create ledger entry for the earnings
        let description = format!("Completed: {}", task.name);
        let entry = LedgerEntry::earned(kid_id, task.value, description);
        let created_entry = self.ledger_repo.create_entry(entry).await?;

        // 4. Reset task if it needs resetting (recurring tasks)
        if task.needs_reset() {
            task.reset();
            self.task_repo.update(task).await?;
        }

        Ok(created_entry)
    }
}
