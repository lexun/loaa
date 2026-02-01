use crate::db::{TaskRepository, KidRepository, LedgerRepository};
use crate::models::{LedgerEntry, TransactionStatus, Cadence};
use crate::error::Result;
use uuid::Uuid;
use chrono::{DateTime, Utc};

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
        self.complete_task_with_status(task_id, kid_id, TransactionStatus::Confirmed, None, None).await
    }

    /// Complete a task for a kid with specified status and reporter
    ///
    /// - status: Confirmed for parent completions, Pending for kid completions
    /// - reported_by: user_id of who reported the completion (for audit trail)
    /// - completed_at: optional date when the task was actually completed (for backdating)
    ///
    /// When backdating (completed_at is in a previous period):
    /// - Creates ledger entry with the backdated completion time
    /// - Does NOT mark the kid in completed_by (since the period has passed)
    /// - This allows the task to still be available for today
    pub async fn complete_task_with_status(
        &self,
        task_id: Uuid,
        kid_id: Uuid,
        status: TransactionStatus,
        reported_by: Option<String>,
        completed_at: Option<DateTime<Utc>>,
    ) -> Result<LedgerEntry> {
        // 1. Verify the kid exists
        let _kid = self.kid_repo.get(kid_id).await?;

        // 2. Get the task
        let mut task = self.task_repo.get(task_id).await?;

        // 3. Reset task first if it needs resetting (makes it available again)
        if task.needs_reset() {
            task.reset();
        }

        // 4. Determine if this is a backdated completion (from a previous period)
        let is_backdated = completed_at.map_or(false, |date| {
            Self::is_from_previous_period(date, task.last_reset, task.cadence)
        });

        // 5. Check if this kid can complete the task (only for non-backdated)
        // For backdated completions, we don't check availability since it's a past period
        if !is_backdated {
            if let Err(reason) = task.can_complete(kid_id) {
                return Err(crate::error::Error::TaskNotAvailable(reason.to_string()));
            }
        }

        // 6. Create ledger entry for the earnings with metadata
        let description = format!("Completed: {}", task.name);
        let entry = LedgerEntry::earned_with_metadata(
            kid_id,
            task.value,
            description,
            status,
            Some(task_id),
            reported_by,
            completed_at,
        );
        let created_entry = self.ledger_repo.create_entry(entry).await?;

        // 7. Mark this kid as having completed the task (only for current period)
        // For backdated completions, don't mark as completed since it's a past period
        if !is_backdated {
            task.mark_completed(kid_id);
            self.task_repo.update(task).await?;
        }

        Ok(created_entry)
    }

    /// Check if a completion date is from a previous period relative to the task's last reset
    fn is_from_previous_period(completed_at: DateTime<Utc>, last_reset: DateTime<Utc>, cadence: Cadence) -> bool {
        match cadence {
            // OneTime tasks don't have periods, so never consider backdated
            Cadence::OneTime => false,
            // For daily tasks, check if completed_at is before the current period started
            Cadence::Daily => completed_at < last_reset,
            // For weekly tasks, check if completed_at is before the current period started
            Cadence::Weekly => completed_at < last_reset,
        }
    }
}
