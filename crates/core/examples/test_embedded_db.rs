use loaa_core::{
    init_database_with_config, DatabaseConfig, DatabaseMode, Kid, KidRepository,
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing embedded database mode...\n");

    // Create config for embedded database
    let config = DatabaseConfig {
        mode: DatabaseMode::Embedded,
        url: None,
        path: Some(PathBuf::from("./data/test_embedded.db")),
    };

    // Initialize database
    println!("Connecting to embedded database at {:?}", config.path);
    let db = init_database_with_config(&config).await?;
    println!("✓ Connected successfully\n");

    // Test CRUD operations
    let kid_repo = KidRepository::new(db.client.clone());

    // Create a kid
    println!("Creating a test kid...");
    let kid = Kid::new("Test Kid".to_string(), "test-owner".to_string())?;
    let created = kid_repo.create(kid).await?;
    println!("✓ Created: {} (ID: {})\n", created.name, created.id);

    // List all kids
    println!("Listing all kids...");
    let kids = kid_repo.list().await?;
    println!("✓ Found {} kid(s)", kids.len());
    for k in &kids {
        println!("  - {} (ID: {})", k.name, k.id);
    }

    println!("\n✅ Embedded database test completed successfully!");
    println!("Data is persisted to: {:?}", config.path.unwrap());

    Ok(())
}
