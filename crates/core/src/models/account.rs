use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{Error, Result};

/// An account represents a family or household.
/// All kids, tasks, and ledger entries belong to an account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    #[serde(skip)]
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Account {
    pub fn new(name: String) -> Result<Self> {
        let account = Self {
            id: Uuid::new_v4(),
            name: name.trim().to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        account.validate()?;
        Ok(account)
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::Validation(
                "Account name cannot be empty".to_string(),
            ));
        }

        if self.name.len() > 100 {
            return Err(Error::Validation(
                "Account name cannot exceed 100 characters".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation() {
        let account = Account::new("Smith Family".to_string()).unwrap();
        assert_eq!(account.name, "Smith Family");
        assert!(!account.id.is_nil());
    }

    #[test]
    fn test_account_validation_empty_name() {
        let result = Account::new("   ".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_account_validation_trimmed_name() {
        let account = Account::new("  Jones Family  ".to_string()).unwrap();
        assert_eq!(account.name, "Jones Family");
    }

    #[test]
    fn test_account_validation_name_too_long() {
        let long_name = "a".repeat(101);
        let result = Account::new(long_name);
        assert!(result.is_err());
    }
}
