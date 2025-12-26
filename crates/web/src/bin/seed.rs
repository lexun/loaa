use loaa_core::{
    init_database_with_config, Config, Kid, KidRepository, Task, TaskRepository,
    Cadence, LedgerRepository, LedgerEntry, User, UserRepository, hash_password,
    models::AccountType
};
use rust_decimal_macros::dec;

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

    // Create admin user
    println!("👤 Creating admin user...");
    let mut admin_user = User::new("admin".to_string())?;
    admin_user.password_hash = hash_password("admin123")?;
    admin_user.account_type = AccountType::Admin;
    let created_user = user_repo.create(admin_user).await?;
    let owner_id = created_user.id.to_string();
    println!("  ✓ Created user: {} (Default password: admin123)", created_user.username);
    println!("  ⚠️  IMPORTANT: Change this password after first login!\n");

    // Create kids (owned by admin for seed data)
    println!("👦 Creating kids...");
    let kids = vec![
        Kid::new("Auri".to_string(), owner_id.clone())?,
        Kid::new("Zevi".to_string(), owner_id.clone())?,
        Kid::new("Yasu".to_string(), owner_id.clone())?,
    ];

    for kid in kids {
        let created = kid_repo.create(kid.clone()).await?;
        println!("  ✓ Created: {} (ID: {})", created.name, created.id);
    }

    // Create tasks (owned by admin for seed data)
    println!("\n📋 Creating tasks...");
    let tasks = vec![
        Task::new(
            "Math Lesson".to_string(),
            "Complete daily math lesson".to_string(),
            dec!(2.00),
            Cadence::Daily,
            owner_id.clone(),
        )?,
        Task::new(
            "Feed Pets".to_string(),
            "Feed and water all pets".to_string(),
            dec!(1.00),
            Cadence::Daily,
            owner_id.clone(),
        )?,
        Task::new(
            "Typing Practice".to_string(),
            "Practice typing for 10 minutes".to_string(),
            dec!(1.50),
            Cadence::Daily,
            owner_id.clone(),
        )?,
        Task::new(
            "Math Practice".to_string(),
            "Extra math practice problems".to_string(),
            dec!(1.50),
            Cadence::Daily,
            owner_id.clone(),
        )?,
        Task::new(
            "Dusting Surfaces".to_string(),
            "Dust all surfaces in common areas".to_string(),
            dec!(2.50),
            Cadence::Weekly,
            owner_id.clone(),
        )?,
        Task::new(
            "Clean Floors".to_string(),
            "Vacuum and mop floors".to_string(),
            dec!(3.00),
            Cadence::Weekly,
            owner_id.clone(),
        )?,
        Task::new(
            "Wash Dishes".to_string(),
            "Wash, dry, and put away dishes".to_string(),
            dec!(2.00),
            Cadence::Daily,
            owner_id.clone(),
        )?,
        Task::new(
            "Clean Room".to_string(),
            "Clean and organize bedroom".to_string(),
            dec!(2.50),
            Cadence::Weekly,
            owner_id.clone(),
        )?,
    ];

    for task in tasks {
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
        // Auri completes Math Lesson
        let auri = &kids[0];
        let math_lesson = tasks.iter().find(|t| t.name == "Math Lesson").unwrap();
        let entry = LedgerEntry::earned(
            auri.id,
            math_lesson.value,
            format!("Completed: {}", math_lesson.name),
        );
        ledger_repo.create_entry(entry).await?;
        println!("✓ {} completed {} (+${})", auri.name, math_lesson.name, math_lesson.value);

        // Auri completes Feed Pets
        let feed_pets = tasks.iter().find(|t| t.name == "Feed Pets").unwrap();
        let entry = LedgerEntry::earned(
            auri.id,
            feed_pets.value,
            format!("Completed: {}", feed_pets.name),
        );
        ledger_repo.create_entry(entry).await?;
        println!("✓ {} completed {} (+${})", auri.name, feed_pets.name, feed_pets.value);

        // Zevi completes Typing Practice
        let zevi = &kids[1];
        let typing = tasks.iter().find(|t| t.name == "Typing Practice").unwrap();
        let entry = LedgerEntry::earned(
            zevi.id,
            typing.value,
            format!("Completed: {}", typing.name),
        );
        ledger_repo.create_entry(entry).await?;
        println!("✓ {} completed {} (+${})", zevi.name, typing.name, typing.value);

        // Zevi completes Wash Dishes
        let dishes = tasks.iter().find(|t| t.name == "Wash Dishes").unwrap();
        let entry = LedgerEntry::earned(
            zevi.id,
            dishes.value,
            format!("Completed: {}", dishes.name),
        );
        ledger_repo.create_entry(entry).await?;
        println!("✓ {} completed {} (+${})", zevi.name, dishes.name, dishes.value);

        // Yasu completes Clean Room
        let yasu = &kids[2];
        let clean_room = tasks.iter().find(|t| t.name == "Clean Room").unwrap();
        let entry = LedgerEntry::earned(
            yasu.id,
            clean_room.value,
            format!("Completed: {}", clean_room.name),
        );
        ledger_repo.create_entry(entry).await?;
        println!("✓ {} completed {} (+${})", yasu.name, clean_room.name, clean_room.value);

        // Yasu completes Math Practice
        let math_practice = tasks.iter().find(|t| t.name == "Math Practice").unwrap();
        let entry = LedgerEntry::earned(
            yasu.id,
            math_practice.value,
            format!("Completed: {}", math_practice.name),
        );
        ledger_repo.create_entry(entry).await?;
        println!("✓ {} completed {} (+${})", yasu.name, math_practice.name, math_practice.value);

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
