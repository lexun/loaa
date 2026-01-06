//! Migration script to add account_id to existing users
//!
//! This script finds all users without an account_id (nil UUID) and creates
//! an Account for them, then updates the user with the new account_id.

use loaa_core::{
    init_database_with_config, Config, UserRepository, Account, AccountRepository,
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Migrating users to add account_id...\n");

    // Initialize database using config
    let config = Config::from_env();
    config.validate()?;
    let db = init_database_with_config(&config.database).await?;

    let user_repo = UserRepository::new(db.client.clone());
    let account_repo = AccountRepository::new(db.client.clone());

    // Get all users
    let users = user_repo.list().await?;
    let nil_uuid = Uuid::nil();

    let mut migrated_count = 0;
    let mut skipped_count = 0;

    for user in users {
        if user.account_id == nil_uuid {
            println!("👤 Migrating user: {} (ID: {})", user.username, user.id);

            // Create a new account for this user
            let account = Account::new(format!("{}'s Household", user.username))?;
            let created_account = account_repo.create(account).await?;
            println!("   ✓ Created account: {} (ID: {})", created_account.name, created_account.id);

            // Update the user with the new account_id
            let mut updated_user = user.clone();
            updated_user.account_id = created_account.id;
            user_repo.update(updated_user).await?;
            println!("   ✓ Updated user with account_id: {}", created_account.id);

            migrated_count += 1;
        } else {
            println!("⏭️  Skipping user: {} (already has account_id: {})", user.username, user.account_id);
            skipped_count += 1;
        }
    }

    println!("\n✅ Migration complete!");
    println!("   Migrated: {} users", migrated_count);
    println!("   Skipped:  {} users", skipped_count);

    Ok(())
}
