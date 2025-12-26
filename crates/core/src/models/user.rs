use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AccountType {
    /// Admin account - can only manage other accounts
    Admin,
    /// Regular user account - sees dashboard with their own data
    #[default]
    User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(skip)]
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    #[serde(default)]
    pub account_type: AccountType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Create a new regular user (without password hash - that must be set separately)
    pub fn new(username: String) -> Result<Self> {
        Self::new_with_type(username, AccountType::User)
    }

    /// Create a new user with a specific account type
    pub fn new_with_type(username: String, account_type: AccountType) -> Result<Self> {
        let user = Self {
            id: Uuid::new_v4(),
            username,
            password_hash: String::new(),  // Must be set before saving
            account_type,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        user.validate()?;
        Ok(user)
    }

    /// Check if this is an admin account
    pub fn is_admin(&self) -> bool {
        self.account_type == AccountType::Admin
    }

    /// Validate user data
    pub fn validate(&self) -> Result<()> {
        if self.username.trim().is_empty() {
            return Err(Error::Validation(
                "Username cannot be empty".to_string()
            ));
        }

        if self.username.len() < 3 {
            return Err(Error::Validation(
                "Username must be at least 3 characters".to_string()
            ));
        }

        if self.username.len() > 50 {
            return Err(Error::Validation(
                "Username cannot exceed 50 characters".to_string()
            ));
        }

        // Username should only contain alphanumeric, underscore, and hyphen
        if !self.username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(Error::Validation(
                "Username can only contain letters, numbers, underscores, and hyphens".to_string()
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_user() {
        let user = User::new("testuser".to_string()).unwrap();
        assert_eq!(user.username, "testuser");
        assert!(user.password_hash.is_empty());
    }

    #[test]
    fn test_username_too_short() {
        let result = User::new("ab".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_username_too_long() {
        let long_name = "a".repeat(51);
        let result = User::new(long_name);
        assert!(result.is_err());
    }

    #[test]
    fn test_username_invalid_chars() {
        let result = User::new("test@user".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_username_with_underscore_and_hyphen() {
        let user = User::new("test_user-123".to_string()).unwrap();
        assert_eq!(user.username, "test_user-123");
    }
}
