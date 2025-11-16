use loaa_core::{init_database, KidRepository, TaskRepository, LedgerRepository, LedgerEntry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Creating test transactions...\n");

    // Connect to database
    let db_url = "127.0.0.1:8000";
    let db = init_database(db_url).await?;
    let kid_repo = KidRepository::new(db.client.clone());
    let task_repo = TaskRepository::new(db.client.clone());
    let ledger_repo = LedgerRepository::new(db.client.clone());

    // Get all kids and tasks
    let kids = kid_repo.list().await?;
    let tasks = task_repo.list().await?;

    if kids.is_empty() || tasks.is_empty() {
        println!("❌ No kids or tasks found. Run 'just seed' first.");
        return Ok(());
    }

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
    println!("🌐 Refresh http://127.0.0.1:3000 to see the updates");

    Ok(())
}
