use crate::error::{Error, Result};
use crate::models::AccountMembership;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::engine::any::Any;
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct MembershipRecord {
    id: Thing,
    #[serde(flatten)]
    membership: AccountMembership,
}

impl MembershipRecord {
    fn into_membership(self) -> AccountMembership {
        let mut membership = self.membership;
        let id_str = self.id.id.to_string();
        let clean_id = id_str.trim_start_matches('⟨').trim_end_matches('⟩');
        membership.id = Uuid::parse_str(clean_id).unwrap_or_else(|_| Uuid::nil());
        membership
    }
}

pub struct AccountMembershipRepository {
    db: Arc<Surreal<Any>>,
}

impl AccountMembershipRepository {
    pub fn new(db: Arc<Surreal<Any>>) -> Self {
        Self { db }
    }

    pub async fn create(&self, membership: AccountMembership) -> Result<AccountMembership> {
        let membership_id = membership.id.to_string();
        let created: Option<MembershipRecord> = self
            .db
            .create(("account_membership", &membership_id))
            .content(membership)
            .await?;

        created
            .map(|rec| rec.into_membership())
            .ok_or_else(|| Error::Database("Failed to create account membership".to_string()))
    }

    pub async fn get(&self, id: Uuid) -> Result<AccountMembership> {
        let record: Option<MembershipRecord> = self
            .db
            .select(("account_membership", id.to_string()))
            .await?;

        record
            .map(|rec| rec.into_membership())
            .ok_or_else(|| Error::NotFound(format!("AccountMembership with id {}", id)))
    }

    /// Get membership by user and account
    pub async fn get_by_user_and_account(
        &self,
        user_id: Uuid,
        account_id: Uuid,
    ) -> Result<AccountMembership> {
        let records: Vec<MembershipRecord> = self
            .db
            .query("SELECT * FROM account_membership WHERE user_id = $user_id AND account_id = $account_id LIMIT 1")
            .bind(("user_id", user_id))
            .bind(("account_id", account_id))
            .await?
            .take(0)?;

        records
            .into_iter()
            .next()
            .map(|rec| rec.into_membership())
            .ok_or_else(|| {
                Error::NotFound(format!(
                    "Membership for user {} in account {}",
                    user_id, account_id
                ))
            })
    }

    /// List all memberships for a user (typically one, but could be multiple)
    pub async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<AccountMembership>> {
        let records: Vec<MembershipRecord> = self
            .db
            .query("SELECT * FROM account_membership WHERE user_id = $user_id")
            .bind(("user_id", user_id))
            .await?
            .take(0)?;

        Ok(records
            .into_iter()
            .map(|rec| rec.into_membership())
            .collect())
    }

    /// List all members of an account
    pub async fn list_by_account(&self, account_id: Uuid) -> Result<Vec<AccountMembership>> {
        let records: Vec<MembershipRecord> = self
            .db
            .query("SELECT * FROM account_membership WHERE account_id = $account_id")
            .bind(("account_id", account_id))
            .await?
            .take(0)?;

        Ok(records
            .into_iter()
            .map(|rec| rec.into_membership())
            .collect())
    }

    /// Get the first/primary membership for a user (convenience method)
    pub async fn get_primary_for_user(&self, user_id: Uuid) -> Result<AccountMembership> {
        let memberships = self.list_by_user(user_id).await?;
        memberships.into_iter().next().ok_or_else(|| {
            Error::NotFound(format!("No account membership found for user {}", user_id))
        })
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let _deleted: Option<MembershipRecord> = self
            .db
            .delete(("account_membership", id.to_string()))
            .await?;
        Ok(())
    }
}
