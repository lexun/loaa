use loaa_core::{
    init_database_with_config, Config, Kid, KidRepository, Task, TaskRepository,
    Cadence, LedgerRepository, LedgerEntry, User, UserRepository, hash_password,
};
use rust_decimal_macros::dec;
use chrono::{Duration, Utc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if we should also create test transactions
    let create_transactions = std::env::args().any(|arg| arg == "--with-transactions");

    println!("🌱 Seeding Loa'a database...\n");

    // Initialize database using config
    let config = Config::from_env();
    config.validate()?;
    let db = init_database_with_config(&config.database).await?;

    let user_repo = UserRepository::new(db.client.clone());
    let kid_repo = KidRepository::new(db.client.clone());
    let task_repo = TaskRepository::new(db.client.clone());

    // Create a test user account that owns the test data
    println!("👤 Creating test user account...");
    let mut test_user = User::new("testuser".to_string())?;
    test_user.password_hash = hash_password("testuser")?;
    let created_user = user_repo.create(test_user).await?;
    let owner_id = created_user.id.to_string();
    println!("  ✓ Created: testuser (password: testuser)\n");

    // Create kids (owned by testuser)
    println!("👦 Creating kids...");
    let kids = vec![
        Kid::new("Jack".to_string(), owner_id.clone())?,
        Kid::new("Emma".to_string(), owner_id.clone())?,
    ];

    for kid in kids {
        let created = kid_repo.create(kid.clone()).await?;
        println!("  ✓ Created: {} (ID: {})", created.name, created.id);
    }

    // Create tasks (owned by testuser)
    // Tasks based on production data
    println!("\n📋 Creating tasks...");
    let tasks = vec![
        // Daily tasks - $0.50
        Task::new(
            "Clean litter box".to_string(),
            "Clean the litter box".to_string(),
            dec!(0.50),
            Cadence::Daily,
            owner_id.clone(),
        )?,
        Task::new(
            "Take out trash".to_string(),
            "Take out the trash".to_string(),
            dec!(0.50),
            Cadence::Daily,
            owner_id.clone(),
        )?,
        Task::new(
            "Wipe down surfaces".to_string(),
            "Wipe down surfaces".to_string(),
            dec!(0.50),
            Cadence::Daily,
            owner_id.clone(),
        )?,
        Task::new(
            "Sweep floor".to_string(),
            "Sweep the floor".to_string(),
            dec!(0.50),
            Cadence::Daily,
            owner_id.clone(),
        )?,
        Task::new(
            "Clean bathroom sink".to_string(),
            "Clean the bathroom sink".to_string(),
            dec!(0.50),
            Cadence::Daily,
            owner_id.clone(),
        )?,
        // Daily tasks - $1.00
        Task::new(
            "Wash dishes".to_string(),
            "Wash a full load of dishes".to_string(),
            dec!(1.00),
            Cadence::Daily,
            owner_id.clone(),
        )?,
        Task::new(
            "Vacuum carpets".to_string(),
            "Vacuum the carpets".to_string(),
            dec!(1.00),
            Cadence::Daily,
            owner_id.clone(),
        )?,
        Task::new(
            "Mop floor".to_string(),
            "Mop the floor".to_string(),
            dec!(1.00),
            Cadence::Daily,
            owner_id.clone(),
        )?,
        // Weekly tasks
        Task::new(
            "Clean bathroom mirror".to_string(),
            "Clean the bathroom mirror".to_string(),
            dec!(0.50),
            Cadence::Weekly,
            owner_id.clone(),
        )?,
        Task::new(
            "Clean toilet".to_string(),
            "Clean the toilet".to_string(),
            dec!(1.00),
            Cadence::Weekly,
            owner_id.clone(),
        )?,
    ];

    // Set last_reset to 2 days ago so tasks are available to complete
    let two_days_ago = Utc::now() - Duration::days(2);
    for mut task in tasks {
        task.last_reset = two_days_ago;
        let created = task_repo.create(task.clone()).await?;
        println!("  ✓ Created: {} - ${} ({:?})", created.name, created.value, created.cadence);
    }

    println!("\n✅ Database seeded successfully!");

    // Create test transactions if requested
    if create_transactions {
        println!("\n🎯 Creating test transactions...\n");
        let ledger_repo = LedgerRepository::new(db.client.clone());

        // Get all kids and tasks
        let kids = kid_repo.list().await?;
        let tasks = task_repo.list().await?;

        println!("Found {} kids and {} tasks\n", kids.len(), tasks.len());

        // Create some transactions
        // Jack completes Wash dishes
        let jack = &kids[0];
        let wash_dishes = tasks.iter().find(|t| t.name == "Wash dishes").unwrap();
        let entry = LedgerEntry::earned(
            jack.id,
            wash_dishes.value,
            format!("Completed: {}", wash_dishes.name),
        );
        ledger_repo.create_entry(entry).await?;
        println!("✓ {} completed {} (+${})", jack.name, wash_dishes.name, wash_dishes.value);

        // Jack completes Clean litter box
        let litter_box = tasks.iter().find(|t| t.name == "Clean litter box").unwrap();
        let entry = LedgerEntry::earned(
            jack.id,
            litter_box.value,
            format!("Completed: {}", litter_box.name),
        );
        ledger_repo.create_entry(entry).await?;
        println!("✓ {} completed {} (+${})", jack.name, litter_box.name, litter_box.value);

        // Emma completes Sweep floor
        let emma = &kids[1];
        let sweep = tasks.iter().find(|t| t.name == "Sweep floor").unwrap();
        let entry = LedgerEntry::earned(
            emma.id,
            sweep.value,
            format!("Completed: {}", sweep.name),
        );
        ledger_repo.create_entry(entry).await?;
        println!("✓ {} completed {} (+${})", emma.name, sweep.name, sweep.value);

        // Emma completes Take out trash
        let trash = tasks.iter().find(|t| t.name == "Take out trash").unwrap();
        let entry = LedgerEntry::earned(
            emma.id,
            trash.value,
            format!("Completed: {}", trash.name),
        );
        ledger_repo.create_entry(entry).await?;
        println!("✓ {} completed {} (+${})", emma.name, trash.name, trash.value);

        // Emma completes Vacuum carpets
        let vacuum = tasks.iter().find(|t| t.name == "Vacuum carpets").unwrap();
        let entry = LedgerEntry::earned(
            emma.id,
            vacuum.value,
            format!("Completed: {}", vacuum.name),
        );
        ledger_repo.create_entry(entry).await?;
        println!("✓ {} completed {} (+${})", emma.name, vacuum.name, vacuum.value);

        // Print final balances
        println!("\n💰 Final Balances:");
        for kid in &kids {
            let ledger = ledger_repo.get_ledger(kid.id).await?;
            println!("  {} - ${}", kid.name, ledger.balance);
        }

        println!("\n✅ Transactions created successfully!");
    }

    println!("🎯 Ready to track chores!\n");
    println!("Visit http://127.0.0.1:3000 to see the data");

    Ok(())
}
