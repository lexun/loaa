use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kid {
    #[serde(skip)]
    pub id: Uuid,
    pub name: String,
    /// Owner of this kid record (user_id as string, or "admin" for admin-created)
    #[serde(default)]
    pub owner_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Kid {
    pub fn new(name: String, owner_id: String) -> Result<Self> {
        let kid = Self {
            id: Uuid::new_v4(),
            name: name.trim().to_string(),
            owner_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        kid.validate()?;
        Ok(kid)
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(crate::error::Error::Validation("Kid name cannot be empty".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kid_creation() {
        let kid = Kid::new("Alice".to_string(), "test-owner".to_string()).unwrap();
        assert_eq!(kid.name, "Alice");
        assert_eq!(kid.owner_id, "test-owner");
        assert!(!kid.id.is_nil());
    }

    #[test]
    fn test_kid_validation_empty_name() {
        let result = Kid::new("   ".to_string(), "test-owner".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_kid_validation_trimmed_name() {
        let kid = Kid::new("  Bob  ".to_string(), "test-owner".to_string()).unwrap();
        assert_eq!(kid.name, "Bob");
    }
}

