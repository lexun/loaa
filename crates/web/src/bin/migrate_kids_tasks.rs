//! Migration script to add account_id to existing kids and tasks
//!
//! This script finds all kids/tasks without an account_id (nil UUID) and
//! assigns them the account_id from their owner's User record.

use loaa_core::{
    init_database_with_config, Config, UserRepository, KidRepository, TaskRepository,
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Migrating kids and tasks to add account_id...\n");

    // Initialize database using config
    let config = Config::from_env();
    config.validate()?;
    let db = init_database_with_config(&config.database).await?;

    let user_repo = UserRepository::new(db.client.clone());
    let kid_repo = KidRepository::new(db.client.clone());
    let task_repo = TaskRepository::new(db.client.clone());

    let nil_uuid = Uuid::nil();

    // Migrate kids
    println!("👦 Migrating kids...");
    let kids = kid_repo.list().await?;
    let mut kids_migrated = 0;
    let mut kids_skipped = 0;

    for kid in kids {
        if kid.account_id == nil_uuid {
            // Try to find the owner's account_id
            if let Ok(owner_id) = Uuid::parse_str(&kid.owner_id) {
                if let Ok(owner) = user_repo.get(owner_id).await {
                    if owner.account_id != nil_uuid {
                        let mut updated_kid = kid.clone();
                        updated_kid.account_id = owner.account_id;
                        kid_repo.update(updated_kid).await?;
                        println!("   ✓ {} -> account_id: {}", kid.name, owner.account_id);
                        kids_migrated += 1;
                    } else {
                        println!("   ⚠️  {} - owner has nil account_id", kid.name);
                        kids_skipped += 1;
                    }
                } else {
                    println!("   ⚠️  {} - owner not found: {}", kid.name, kid.owner_id);
                    kids_skipped += 1;
                }
            } else {
                println!("   ⚠️  {} - invalid owner_id: {}", kid.name, kid.owner_id);
                kids_skipped += 1;
            }
        } else {
            println!("   ⏭️  {} (already has account_id)", kid.name);
            kids_skipped += 1;
        }
    }

    // Migrate tasks
    println!("\n📋 Migrating tasks...");
    let tasks = task_repo.list().await?;
    let mut tasks_migrated = 0;
    let mut tasks_skipped = 0;

    for task in tasks {
        if task.account_id == nil_uuid {
            // Try to find the owner's account_id
            if let Ok(owner_id) = Uuid::parse_str(&task.owner_id) {
                if let Ok(owner) = user_repo.get(owner_id).await {
                    if owner.account_id != nil_uuid {
                        let mut updated_task = task.clone();
                        updated_task.account_id = owner.account_id;
                        task_repo.update(updated_task).await?;
                        println!("   ✓ {} -> account_id: {}", task.name, owner.account_id);
                        tasks_migrated += 1;
                    } else {
                        println!("   ⚠️  {} - owner has nil account_id", task.name);
                        tasks_skipped += 1;
                    }
                } else {
                    println!("   ⚠️  {} - owner not found: {}", task.name, task.owner_id);
                    tasks_skipped += 1;
                }
            } else {
                println!("   ⚠️  {} - invalid owner_id: {}", task.name, task.owner_id);
                tasks_skipped += 1;
            }
        } else {
            println!("   ⏭️  {} (already has account_id)", task.name);
            tasks_skipped += 1;
        }
    }

    println!("\n✅ Migration complete!");
    println!("   Kids:  {} migrated, {} skipped", kids_migrated, kids_skipped);
    println!("   Tasks: {} migrated, {} skipped", tasks_migrated, tasks_skipped);

    Ok(())
}
