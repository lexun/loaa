use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::sql::Thing;
use crate::models::User;
use crate::error::{Error, Result};
use uuid::Uuid;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

// Helper struct to handle SurrealDB record with id
#[derive(Debug, Serialize, Deserialize)]
struct UserRecord {
    id: Thing,
    #[serde(flatten)]
    user: User,
}

impl UserRecord {
    fn into_user(self) -> User {
        let mut user = self.user;
        // Extract UUID from SurrealDB Thing
        // SurrealDB wraps the ID in angle brackets: ⟨uuid⟩
        let id_str = self.id.id.to_string();
        let clean_id = id_str.trim_start_matches('⟨').trim_end_matches('⟩');
        user.id = Uuid::parse_str(clean_id)
            .unwrap_or_else(|_| Uuid::nil());
        user
    }
}

pub struct UserRepository {
    db: Arc<Surreal<Any>>,
}

impl UserRepository {
    pub fn new(db: Arc<Surreal<Any>>) -> Self {
        Self { db }
    }

    pub async fn create(&self, user: User) -> Result<User> {
        let user_id = user.id.to_string();
        let created: Option<UserRecord> = self.db
            .create(("user", &user_id))
            .content(user)
            .await?;

        created
            .map(|rec| rec.into_user())
            .ok_or_else(|| Error::Database("Failed to create user".to_string()))
    }

    pub async fn get(&self, id: Uuid) -> Result<User> {
        let record: Option<UserRecord> = self.db
            .select(("user", id.to_string()))
            .await?;

        record
            .map(|rec| rec.into_user())
            .ok_or_else(|| Error::NotFound(format!("User with id {}", id)))
    }

    pub async fn get_by_username(&self, username: &str) -> Result<User> {
        let query = "SELECT * FROM user WHERE username = $username LIMIT 1";
        let username_string = username.to_string();
        let mut result = self.db
            .query(query)
            .bind(("username", username_string))
            .await?;

        let users: Vec<UserRecord> = result.take(0)?;

        users
            .into_iter()
            .next()
            .map(|rec| rec.into_user())
            .ok_or_else(|| Error::NotFound(format!("User with username '{}'", username)))
    }

    pub async fn list(&self) -> Result<Vec<User>> {
        let records: Vec<UserRecord> = self.db
            .select("user")
            .await?;

        Ok(records.into_iter().map(|rec| rec.into_user()).collect())
    }

    pub async fn update(&self, user: User) -> Result<User> {
        let user_id = user.id;

        // First check if the user exists
        let _existing: User = self.get(user_id).await?;

        // If it exists, update it
        let updated: Option<UserRecord> = self.db
            .update(("user", user_id.to_string()))
            .content(user)
            .await?;

        updated
            .map(|rec| rec.into_user())
            .ok_or_else(|| Error::NotFound(format!("User with id {}", user_id)))
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let _deleted: Option<UserRecord> = self.db
            .delete(("user", id.to_string()))
            .await?;
        Ok(())
    }
}
