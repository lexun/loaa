use loaa_core::db::{init_database_with_config, TaskRepository, KidRepository, LedgerRepository};
use loaa_core::config::{DatabaseConfig, DatabaseMode};
use loaa_core::models::{Task, Kid, Cadence, TransactionStatus};
use loaa_core::workflows::TaskCompletionWorkflow;
use rust_decimal_macros::dec;
use tempfile::TempDir;
use uuid::Uuid;
use chrono::{Duration, Utc};

fn test_account_id() -> Uuid {
    // Use a fixed UUID for test account to keep tests deterministic
    Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
}

async fn setup_test() -> (TempDir, TaskCompletionWorkflow, TaskRepository, KidRepository, LedgerRepository) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Use in-memory database for tests
    let config = DatabaseConfig {
        mode: DatabaseMode::Memory,
        url: None,
        path: None,
        namespace: Some("test".to_string()),
        database: Some("test".to_string()),
        username: None,
        password: None,
        token: None,
    };
    let database = init_database_with_config(&config)
        .await
        .expect("Failed to initialize database");

    let task_repo = TaskRepository::new(database.client.clone());
    let kid_repo = KidRepository::new(database.client.clone());
    let ledger_repo = LedgerRepository::new(database.client.clone());

    let workflow = TaskCompletionWorkflow::new(
        TaskRepository::new(database.client.clone()),
        KidRepository::new(database.client.clone()),
        LedgerRepository::new(database.client.clone()),
    );

    (temp_dir, workflow, task_repo, kid_repo, ledger_repo)
}

#[tokio::test]
async fn test_complete_one_time_task() {
    let (_temp_dir, workflow, task_repo, kid_repo, ledger_repo) = setup_test().await;

    // Create a kid
    let kid = Kid::new("Alice".to_string(), test_account_id(), "test-owner".to_string()).unwrap();
    let kid_id = kid.id;
    kid_repo.create(kid).await.unwrap();

    // Create a one-time task
    let task = Task::new(
        "Clean garage".to_string(),
        "Organize and clean the garage".to_string(),
        dec!(10.00),
        Cadence::OneTime,
        test_account_id(),
        "test-owner".to_string(),
    )
    .unwrap();
    let task_id = task.id;
    task_repo.create(task).await.unwrap();

    // Complete the task
    let entry = workflow.complete_task(task_id, kid_id).await.unwrap();

    // Verify ledger entry was created
    assert_eq!(entry.kid_id, kid_id);
    assert_eq!(entry.amount, dec!(10.00));
    assert!(entry.description.contains("Clean garage"));

    // Verify ledger balance
    let ledger = ledger_repo.get_ledger(kid_id).await.unwrap();
    assert_eq!(ledger.balance, dec!(10.00));

    // Verify task was NOT reset (one-time tasks don't reset)
    let task_after = task_repo.get(task_id).await.unwrap();
    assert!(!task_after.needs_reset());
}

#[tokio::test]
async fn test_complete_daily_task() {
    let (_temp_dir, workflow, task_repo, kid_repo, _ledger_repo) = setup_test().await;

    // Create a kid
    let kid = Kid::new("Bob".to_string(), test_account_id(), "test-owner".to_string()).unwrap();
    let kid_id = kid.id;
    kid_repo.create(kid).await.unwrap();

    // Create a daily task
    let mut task = Task::new(
        "Take out trash".to_string(),
        "Empty all trash bins".to_string(),
        dec!(1.50),
        Cadence::Daily,
        test_account_id(),
        "test-owner".to_string(),
    )
    .unwrap();
    let task_id = task.id;

    // Set last_reset to 2 days ago so it needs resetting
    task.last_reset = chrono::Utc::now() - chrono::Duration::days(2);
    task_repo.create(task.clone()).await.unwrap();

    // Verify task needs reset before completion
    assert!(task.needs_reset());

    // Complete the task
    let entry = workflow.complete_task(task_id, kid_id).await.unwrap();

    // Verify ledger entry
    assert_eq!(entry.amount, dec!(1.50));

    // Verify task was reset
    let task_after = task_repo.get(task_id).await.unwrap();
    assert!(!task_after.needs_reset(), "Task should be reset after completion");

    // Verify last_reset was updated
    assert!(task_after.last_reset > task.last_reset);
}

#[tokio::test]
async fn test_complete_task_multiple_times() {
    let (_temp_dir, workflow, task_repo, kid_repo, ledger_repo) = setup_test().await;

    // Create a kid
    let kid = Kid::new("Charlie".to_string(), test_account_id(), "test-owner".to_string()).unwrap();
    let kid_id = kid.id;
    kid_repo.create(kid).await.unwrap();

    // Create a daily task
    let task = Task::new(
        "Do dishes".to_string(),
        "Wash and dry all dishes".to_string(),
        dec!(2.00),
        Cadence::Daily,
        test_account_id(),
        "test-owner".to_string(),
    )
    .unwrap();
    let task_id = task.id;
    task_repo.create(task).await.unwrap();

    // Complete for today
    workflow.complete_task(task_id, kid_id).await.unwrap();

    // Use backdating to complete for yesterday and day before
    let yesterday = Utc::now() - Duration::days(1);
    workflow.complete_task_with_status(
        task_id, kid_id, TransactionStatus::Confirmed,
        Some("test".to_string()), Some(yesterday),
    ).await.unwrap();

    let two_days_ago = Utc::now() - Duration::days(2);
    workflow.complete_task_with_status(
        task_id, kid_id, TransactionStatus::Confirmed,
        Some("test".to_string()), Some(two_days_ago),
    ).await.unwrap();

    // Verify ledger has 3 entries totaling $6.00
    let ledger = ledger_repo.get_ledger(kid_id).await.unwrap();
    assert_eq!(ledger.entries.len(), 3);
    assert_eq!(ledger.balance, dec!(6.00));
}

#[tokio::test]
async fn test_complete_task_for_different_kids() {
    let (_temp_dir, workflow, task_repo, kid_repo, ledger_repo) = setup_test().await;

    // Create two kids
    let kid1 = Kid::new("Alice".to_string(), test_account_id(), "test-owner".to_string()).unwrap();
    let kid1_id = kid1.id;
    kid_repo.create(kid1).await.unwrap();

    let kid2 = Kid::new("Bob".to_string(), test_account_id(), "test-owner".to_string()).unwrap();
    let kid2_id = kid2.id;
    kid_repo.create(kid2).await.unwrap();

    // Create a collaborative task (multiple kids can each complete it)
    let task = Task::new_collaborative(
        "Vacuum living room".to_string(),
        "Vacuum the entire living room".to_string(),
        dec!(3.00),
        Cadence::Weekly,
        test_account_id(),
        "test-owner".to_string(),
    )
    .unwrap();
    let task_id = task.id;
    task_repo.create(task).await.unwrap();

    // Alice completes it
    workflow.complete_task(task_id, kid1_id).await.unwrap();

    // Bob completes it (allowed because task is collaborative)
    workflow.complete_task(task_id, kid2_id).await.unwrap();

    // Verify each kid's ledger
    let ledger1 = ledger_repo.get_ledger(kid1_id).await.unwrap();
    assert_eq!(ledger1.balance, dec!(3.00));

    let ledger2 = ledger_repo.get_ledger(kid2_id).await.unwrap();
    assert_eq!(ledger2.balance, dec!(3.00));
}

#[tokio::test]
async fn test_complete_task_with_nonexistent_kid() {
    let (_temp_dir, workflow, task_repo, _kid_repo, _ledger_repo) = setup_test().await;

    // Create a task
    let task = Task::new(
        "Test task".to_string(),
        "Test".to_string(),
        dec!(1.00),
        Cadence::OneTime,
        test_account_id(),
        "test-owner".to_string(),
    )
    .unwrap();
    let task_id = task.id;
    task_repo.create(task).await.unwrap();

    // Try to complete with non-existent kid
    let fake_kid_id = uuid::Uuid::new_v4();
    let result = workflow.complete_task(task_id, fake_kid_id).await;

    assert!(result.is_err(), "Should fail when kid doesn't exist");
}

#[tokio::test]
async fn test_complete_nonexistent_task() {
    let (_temp_dir, workflow, _task_repo, kid_repo, _ledger_repo) = setup_test().await;

    // Create a kid
    let kid = Kid::new("Alice".to_string(), test_account_id(), "test-owner".to_string()).unwrap();
    let kid_id = kid.id;
    kid_repo.create(kid).await.unwrap();

    // Try to complete non-existent task
    let fake_task_id = uuid::Uuid::new_v4();
    let result = workflow.complete_task(fake_task_id, kid_id).await;

    assert!(result.is_err(), "Should fail when task doesn't exist");
}

#[tokio::test]
async fn test_backdate_task_completion() {
    let (_temp_dir, workflow, task_repo, kid_repo, ledger_repo) = setup_test().await;

    // Create a kid
    let kid = Kid::new("Diana".to_string(), test_account_id(), "test-owner".to_string()).unwrap();
    let kid_id = kid.id;
    kid_repo.create(kid).await.unwrap();

    // Create a daily task
    let task = Task::new(
        "Make bed".to_string(),
        "Make the bed each morning".to_string(),
        dec!(0.50),
        Cadence::Daily,
        test_account_id(),
        "test-owner".to_string(),
    )
    .unwrap();
    let task_id = task.id;
    task_repo.create(task).await.unwrap();

    // Complete the task with a backdated date (yesterday)
    let yesterday = Utc::now() - Duration::days(1);
    let entry = workflow.complete_task_with_status(
        task_id,
        kid_id,
        TransactionStatus::Confirmed,
        Some("parent-user".to_string()),
        Some(yesterday),
    ).await.unwrap();

    // Verify ledger entry was created with the backdated completion time
    assert_eq!(entry.kid_id, kid_id);
    assert_eq!(entry.amount, dec!(0.50));
    assert!(entry.completed_at.is_some());
    assert_eq!(entry.completed_at.unwrap(), yesterday);

    // Verify ledger balance
    let ledger = ledger_repo.get_ledger(kid_id).await.unwrap();
    assert_eq!(ledger.balance, dec!(0.50));

    // The task should still be available today (kid not in completed_by)
    // because the backdated completion was from a previous period
    let task_after = task_repo.get(task_id).await.unwrap();
    assert!(!task_after.completed_by.contains(&kid_id), "Kid should not be in completed_by for backdated completion");

    // Now complete the task for today
    let entry2 = workflow.complete_task(task_id, kid_id).await.unwrap();
    assert_eq!(entry2.amount, dec!(0.50));

    // Verify ledger now has 2 entries totaling $1.00
    let ledger2 = ledger_repo.get_ledger(kid_id).await.unwrap();
    assert_eq!(ledger2.entries.len(), 2);
    assert_eq!(ledger2.balance, dec!(1.00));

    // Now the task should show kid as completed (for today's period)
    let task_final = task_repo.get(task_id).await.unwrap();
    assert!(task_final.completed_by.contains(&kid_id), "Kid should be in completed_by after today's completion");
}
