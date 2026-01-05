use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{Error, Result};

/// Role within an account
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MembershipRole {
    /// Parent role - full access, can approve tasks, manage account
    #[default]
    Parent,
    /// Kid role - limited access, task completions require approval
    Kid,
}

/// Links a user to an account with a specific role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountMembership {
    #[serde(skip)]
    pub id: Uuid,
    pub account_id: Uuid,
    pub user_id: Uuid,
    pub role: MembershipRole,
    /// For Kid role: optionally links to a specific Kid record
    /// If None and role is Kid, user has shared access to all kids
    pub kid_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AccountMembership {
    /// Create a new parent membership
    pub fn new_parent(account_id: Uuid, user_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            account_id,
            user_id,
            role: MembershipRole::Parent,
            kid_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Create a new kid membership linked to a specific kid record
    pub fn new_kid(account_id: Uuid, user_id: Uuid, kid_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            account_id,
            user_id,
            role: MembershipRole::Kid,
            kid_id: Some(kid_id),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Create a new kid membership with shared access (no specific kid)
    pub fn new_kid_shared(account_id: Uuid, user_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            account_id,
            user_id,
            role: MembershipRole::Kid,
            kid_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.account_id.is_nil() {
            return Err(Error::Validation(
                "Account ID cannot be nil".to_string(),
            ));
        }

        if self.user_id.is_nil() {
            return Err(Error::Validation(
                "User ID cannot be nil".to_string(),
            ));
        }

        Ok(())
    }

    pub fn is_parent(&self) -> bool {
        self.role == MembershipRole::Parent
    }

    pub fn is_kid(&self) -> bool {
        self.role == MembershipRole::Kid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parent_membership() {
        let account_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let membership = AccountMembership::new_parent(account_id, user_id);

        assert_eq!(membership.account_id, account_id);
        assert_eq!(membership.user_id, user_id);
        assert_eq!(membership.role, MembershipRole::Parent);
        assert!(membership.kid_id.is_none());
        assert!(membership.is_parent());
        assert!(!membership.is_kid());
    }

    #[test]
    fn test_kid_membership() {
        let account_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let kid_id = Uuid::new_v4();
        let membership = AccountMembership::new_kid(account_id, user_id, kid_id);

        assert_eq!(membership.role, MembershipRole::Kid);
        assert_eq!(membership.kid_id, Some(kid_id));
        assert!(membership.is_kid());
    }

    #[test]
    fn test_kid_shared_membership() {
        let account_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let membership = AccountMembership::new_kid_shared(account_id, user_id);

        assert_eq!(membership.role, MembershipRole::Kid);
        assert!(membership.kid_id.is_none());
    }
}
