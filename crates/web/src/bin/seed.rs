use loaa_core::{Database, Kid, KidRepository, Task, TaskRepository, Cadence};
use rust_decimal_macros::dec;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌱 Seeding Loa'a database...\n");

    // Initialize database - connect to SurrealDB server
    let db_url = std::env::var("LOAA_DB_URL").unwrap_or_else(|_| "127.0.0.1:8000".to_string());
    let db = Database::init(&db_url).await?;
    let kid_repo = KidRepository::new(db.client.clone());
    let task_repo = TaskRepository::new(db.client.clone());

    // Create kids
    println!("👦 Creating kids...");
    let kids = vec![
        Kid::new("Auri".to_string())?,
        Kid::new("Zevi".to_string())?,
        Kid::new("Yasu".to_string())?,
    ];

    for kid in kids {
        let created = kid_repo.create(kid.clone()).await?;
        println!("  ✓ Created: {} (ID: {})", created.name, created.id);
    }

    // Create tasks
    println!("\n📋 Creating tasks...");
    let tasks = vec![
        Task::new(
            "Math Lesson".to_string(),
            "Complete daily math lesson".to_string(),
            dec!(2.00),
            Cadence::Daily,
        )?,
        Task::new(
            "Feed Pets".to_string(),
            "Feed and water all pets".to_string(),
            dec!(1.00),
            Cadence::Daily,
        )?,
        Task::new(
            "Typing Practice".to_string(),
            "Practice typing for 10 minutes".to_string(),
            dec!(1.50),
            Cadence::Daily,
        )?,
        Task::new(
            "Math Practice".to_string(),
            "Extra math practice problems".to_string(),
            dec!(1.50),
            Cadence::Daily,
        )?,
        Task::new(
            "Dusting Surfaces".to_string(),
            "Dust all surfaces in common areas".to_string(),
            dec!(2.50),
            Cadence::Weekly,
        )?,
        Task::new(
            "Clean Floors".to_string(),
            "Vacuum and mop floors".to_string(),
            dec!(3.00),
            Cadence::Weekly,
        )?,
        Task::new(
            "Wash Dishes".to_string(),
            "Wash, dry, and put away dishes".to_string(),
            dec!(2.00),
            Cadence::Daily,
        )?,
        Task::new(
            "Clean Room".to_string(),
            "Clean and organize bedroom".to_string(),
            dec!(2.50),
            Cadence::Weekly,
        )?,
    ];

    for task in tasks {
        let created = task_repo.create(task.clone()).await?;
        println!("  ✓ Created: {} - ${} ({:?})", created.name, created.value, created.cadence);
    }

    println!("\n✅ Database seeded successfully!");
    println!("🎯 Ready to track chores!\n");
    println!("Visit http://127.0.0.1:3000 to see the data");

    Ok(())
}
