use loaa_core::db::{init_database, TaskRepository};
use loaa_core::models::{Task, Cadence};
use rust_decimal_macros::dec;
use tempfile::TempDir;
use uuid::Uuid;

async fn setup_test_db() -> (TempDir, TaskRepository) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let database = init_database(db_path.to_str().unwrap())
        .await
        .expect("Failed to initialize database");
    let repo = TaskRepository::new(database.client.clone());
    (temp_dir, repo)
}

#[tokio::test]
async fn test_create_task() {
    let (_temp_dir, repo) = setup_test_db().await;

    let task = Task::new(
        "Take out trash".to_string(),
        "Empty all trash bins".to_string(),
        dec!(1.50),
        Cadence::Daily,
        "test-owner".to_string(),
    )
    .unwrap();

    let created = repo.create(task.clone()).await.unwrap();
    assert_eq!(created.name, "Take out trash");
    assert_eq!(created.value, dec!(1.50));
    assert_eq!(created.cadence, Cadence::Daily);
}

#[tokio::test]
async fn test_get_task() {
    let (_temp_dir, repo) = setup_test_db().await;

    let task = Task::new(
        "Do dishes".to_string(),
        "Wash and dry all dishes".to_string(),
        dec!(2.00),
        Cadence::Daily,
        "test-owner".to_string(),
    )
    .unwrap();

    let task_id = task.id;
    repo.create(task).await.unwrap();

    let retrieved = repo.get(task_id).await.unwrap();
    assert_eq!(retrieved.id, task_id);
    assert_eq!(retrieved.name, "Do dishes");
    assert_eq!(retrieved.value, dec!(2.00));
}

#[tokio::test]
async fn test_get_nonexistent_task() {
    let (_temp_dir, repo) = setup_test_db().await;

    let result = repo.get(Uuid::new_v4()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_tasks() {
    let (_temp_dir, repo) = setup_test_db().await;

    // Create multiple tasks
    let task1 = Task::new(
        "Task 1".to_string(),
        "Description 1".to_string(),
        dec!(1.00),
        Cadence::Daily,
        "test-owner".to_string(),
    )
    .unwrap();

    let task2 = Task::new(
        "Task 2".to_string(),
        "Description 2".to_string(),
        dec!(2.00),
        Cadence::Weekly,
        "test-owner".to_string(),
    )
    .unwrap();

    let task3 = Task::new(
        "Task 3".to_string(),
        "Description 3".to_string(),
        dec!(3.00),
        Cadence::OneTime,
        "test-owner".to_string(),
    )
    .unwrap();

    repo.create(task1).await.unwrap();
    repo.create(task2).await.unwrap();
    repo.create(task3).await.unwrap();

    let tasks = repo.list().await.unwrap();
    assert_eq!(tasks.len(), 3);
}

#[tokio::test]
async fn test_update_task() {
    let (_temp_dir, repo) = setup_test_db().await;

    let task = Task::new(
        "Original name".to_string(),
        "Original description".to_string(),
        dec!(1.00),
        Cadence::Daily,
        "test-owner".to_string(),
    )
    .unwrap();

    let task_id = task.id;
    let created = repo.create(task).await.unwrap();

    // Update the task
    let mut updated_task = created.clone();
    updated_task.name = "Updated name".to_string();
    updated_task.description = "Updated description".to_string();
    updated_task.value = dec!(5.00);
    updated_task.cadence = Cadence::Weekly;

    let result = repo.update(updated_task).await.unwrap();
    assert_eq!(result.id, task_id);
    assert_eq!(result.name, "Updated name");
    assert_eq!(result.description, "Updated description");
    assert_eq!(result.value, dec!(5.00));
    assert_eq!(result.cadence, Cadence::Weekly);

    // Verify the update persisted
    let retrieved = repo.get(task_id).await.unwrap();
    assert_eq!(retrieved.name, "Updated name");
}

#[tokio::test]
async fn test_update_nonexistent_task() {
    let (_temp_dir, repo) = setup_test_db().await;

    let task = Task::new(
        "Test".to_string(),
        "Test".to_string(),
        dec!(1.00),
        Cadence::Daily,
        "test-owner".to_string(),
    )
    .unwrap();

    let result = repo.update(task).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_task() {
    let (_temp_dir, repo) = setup_test_db().await;

    let task = Task::new(
        "To delete".to_string(),
        "This will be deleted".to_string(),
        dec!(1.00),
        Cadence::Daily,
        "test-owner".to_string(),
    )
    .unwrap();

    let task_id = task.id;
    repo.create(task).await.unwrap();

    // Verify it exists
    let retrieved = repo.get(task_id).await;
    assert!(retrieved.is_ok());

    // Delete it
    repo.delete(task_id).await.unwrap();

    // Verify it's gone
    let result = repo.get(task_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_nonexistent_task() {
    let (_temp_dir, repo) = setup_test_db().await;

    // Deleting a non-existent task should not error (idempotent)
    let result = repo.delete(Uuid::new_v4()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_empty() {
    let (_temp_dir, repo) = setup_test_db().await;

    let tasks = repo.list().await.unwrap();
    assert_eq!(tasks.len(), 0);
}
