use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use rust_decimal::Decimal;
use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    #[serde(skip)]
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub value: Decimal,
    pub cadence: Cadence,
    /// Account this task belongs to
    pub account_id: Uuid,
    /// Owner of this task (user_id as string, or "admin" for admin-created)
    #[serde(default)]
    pub owner_id: String,
    /// Whether multiple kids can complete this task and each earn full credit
    #[serde(default)]
    pub collaborative: bool,
    /// Kids who have completed this task in the current period (cleared on reset)
    #[serde(default)]
    pub completed_by: Vec<Uuid>,
    pub last_reset: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Cadence {
    Daily,
    Weekly,
    OneTime,
}

impl Task {
    pub fn new(name: String, description: String, value: Decimal, cadence: Cadence, account_id: Uuid, owner_id: String) -> Result<Self> {
        let now = Utc::now();
        let task = Self {
            id: Uuid::new_v4(),
            name: name.trim().to_string(),
            description: description.trim().to_string(),
            value,
            cadence,
            account_id,
            owner_id,
            collaborative: false,
            completed_by: Vec::new(),
            last_reset: now,
            created_at: now,
            updated_at: now,
        };
        task.validate()?;
        Ok(task)
    }

    /// Create a new collaborative task where multiple kids can each earn full credit
    pub fn new_collaborative(name: String, description: String, value: Decimal, cadence: Cadence, account_id: Uuid, owner_id: String) -> Result<Self> {
        let mut task = Self::new(name, description, value, cadence, account_id, owner_id)?;
        task.collaborative = true;
        Ok(task)
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(crate::error::Error::Validation("Task name cannot be empty".to_string()));
        }
        if self.value <= Decimal::ZERO {
            return Err(crate::error::Error::Validation("Task value must be positive".to_string()));
        }
        Ok(())
    }

    pub fn needs_reset(&self) -> bool {
        match self.cadence {
            Cadence::OneTime => false,
            Cadence::Daily => {
                let next_reset = self.last_reset + Duration::days(1);
                Utc::now() >= next_reset
            }
            Cadence::Weekly => {
                let next_reset = self.last_reset + Duration::weeks(1);
                Utc::now() >= next_reset
            }
        }
    }

    pub fn reset(&mut self) {
        self.last_reset = Utc::now();
        self.updated_at = Utc::now();
        self.completed_by.clear();
    }

    /// Check if a kid can complete this task
    ///
    /// Returns Ok(()) if allowed, or Err with reason if not
    pub fn can_complete(&self, kid_id: Uuid) -> std::result::Result<(), &'static str> {
        // Check if this kid already completed it
        if self.completed_by.contains(&kid_id) {
            return Err("You already completed this task");
        }

        // For non-collaborative tasks, check if anyone has completed it
        if !self.collaborative && !self.completed_by.is_empty() {
            return Err("Already completed by another kid");
        }

        Ok(())
    }

    /// Mark this task as completed by a kid
    pub fn mark_completed(&mut self, kid_id: Uuid) {
        if !self.completed_by.contains(&kid_id) {
            self.completed_by.push(kid_id);
            self.updated_at = Utc::now();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn test_account_id() -> Uuid {
        Uuid::new_v4()
    }

    #[test]
    fn test_task_creation() {
        let account_id = test_account_id();
        let task = Task::new(
            "Take out trash".to_string(),
            "Empty all trash bins".to_string(),
            dec!(1.50),
            Cadence::Daily,
            account_id,
            "test-owner".to_string(),
        ).unwrap();
        assert_eq!(task.name, "Take out trash");
        assert_eq!(task.value, dec!(1.50));
        assert_eq!(task.account_id, account_id);
        assert_eq!(task.owner_id, "test-owner");
    }

    #[test]
    fn test_task_validation_empty_name() {
        let result = Task::new(
            "   ".to_string(),
            "".to_string(),
            dec!(1.0),
            Cadence::Daily,
            test_account_id(),
            "test-owner".to_string(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_task_validation_zero_value() {
        let result = Task::new(
            "Test".to_string(),
            "".to_string(),
            dec!(0),
            Cadence::Daily,
            test_account_id(),
            "test-owner".to_string(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_needs_reset_daily() {
        let mut task = Task::new(
            "Test".to_string(),
            "".to_string(),
            dec!(1.0),
            Cadence::Daily,
            test_account_id(),
            "test-owner".to_string(),
        ).unwrap();
        task.last_reset = Utc::now() - Duration::days(2);
        assert!(task.needs_reset());
    }

    #[test]
    fn test_needs_reset_onetime() {
        let task = Task::new(
            "Test".to_string(),
            "".to_string(),
            dec!(1.0),
            Cadence::OneTime,
            test_account_id(),
            "test-owner".to_string(),
        ).unwrap();
        assert!(!task.needs_reset());
    }

    #[test]
    fn test_non_collaborative_task_blocks_second_kid() {
        let mut task = Task::new(
            "Test".to_string(),
            "".to_string(),
            dec!(1.0),
            Cadence::Daily,
            test_account_id(),
            "test-owner".to_string(),
        ).unwrap();

        let kid1 = Uuid::new_v4();
        let kid2 = Uuid::new_v4();

        // First kid can complete
        assert!(task.can_complete(kid1).is_ok());
        task.mark_completed(kid1);

        // Second kid cannot complete non-collaborative task
        assert!(task.can_complete(kid2).is_err());
        assert_eq!(task.can_complete(kid2).unwrap_err(), "Already completed by another kid");

        // First kid also cannot complete again
        assert!(task.can_complete(kid1).is_err());
        assert_eq!(task.can_complete(kid1).unwrap_err(), "You already completed this task");
    }

    #[test]
    fn test_collaborative_task_allows_multiple_kids() {
        let mut task = Task::new_collaborative(
            "Test".to_string(),
            "".to_string(),
            dec!(1.0),
            Cadence::Daily,
            test_account_id(),
            "test-owner".to_string(),
        ).unwrap();

        let kid1 = Uuid::new_v4();
        let kid2 = Uuid::new_v4();

        // First kid can complete
        assert!(task.can_complete(kid1).is_ok());
        task.mark_completed(kid1);

        // Second kid can also complete collaborative task
        assert!(task.can_complete(kid2).is_ok());
        task.mark_completed(kid2);

        // But neither can complete twice
        assert!(task.can_complete(kid1).is_err());
        assert!(task.can_complete(kid2).is_err());
    }

    #[test]
    fn test_reset_clears_completed_by() {
        let mut task = Task::new(
            "Test".to_string(),
            "".to_string(),
            dec!(1.0),
            Cadence::Daily,
            test_account_id(),
            "test-owner".to_string(),
        ).unwrap();

        let kid = Uuid::new_v4();
        task.mark_completed(kid);
        assert!(task.can_complete(kid).is_err());

        task.reset();
        assert!(task.can_complete(kid).is_ok());
        assert!(task.completed_by.is_empty());
    }
}

