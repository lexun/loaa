use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use rust_decimal::Decimal;
use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub value: Decimal,
    pub cadence: Cadence,
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
    pub fn new(name: String, description: String, value: Decimal, cadence: Cadence) -> Result<Self> {
        let now = Utc::now();
        let task = Self {
            id: Uuid::new_v4(),
            name: name.trim().to_string(),
            description: description.trim().to_string(),
            value,
            cadence,
            last_reset: now,
            created_at: now,
            updated_at: now,
        };
        task.validate()?;
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "Take out trash".to_string(),
            "Empty all trash bins".to_string(),
            dec!(1.50),
            Cadence::Daily,
        ).unwrap();
        assert_eq!(task.name, "Take out trash");
        assert_eq!(task.value, dec!(1.50));
    }

    #[test]
    fn test_task_validation_empty_name() {
        let result = Task::new(
            "   ".to_string(),
            "".to_string(),
            dec!(1.0),
            Cadence::Daily,
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
        ).unwrap();
        assert!(!task.needs_reset());
    }
}

