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
    ///
    /// - Checks if the task is available for this kid
    /// - Creates a ledger entry with the task's value
    /// - Marks the kid as having completed the task
    /// - Resets the task if it's a recurring cadence that needs resetting
    ///
    /// Returns the created ledger entry
    pub async fn complete_task(&self, task_id: Uuid, kid_id: Uuid) -> Result<LedgerEntry> {
        // 1. Verify the kid exists
        let _kid = self.kid_repo.get(kid_id).await?;

        // 2. Get the task
        let mut task = self.task_repo.get(task_id).await?;

        // 3. Reset task first if it needs resetting (makes it available again)
        if task.needs_reset() {
            task.reset();
        }

        // 4. Check if this kid can complete the task
        if let Err(reason) = task.can_complete(kid_id) {
            return Err(crate::error::Error::TaskNotAvailable(reason.to_string()));
        }

        // 5. Create ledger entry for the earnings
        let description = format!("Completed: {}", task.name);
        let entry = LedgerEntry::earned(kid_id, task.value, description);
        let created_entry = self.ledger_repo.create_entry(entry).await?;

        // 6. Mark this kid as having completed the task
        task.mark_completed(kid_id);
        self.task_repo.update(task).await?;

        Ok(created_entry)
    }
}
