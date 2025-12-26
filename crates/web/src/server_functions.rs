use leptos::*;
use crate::dto::*;

#[cfg(feature = "ssr")]
use loaa_core::{
    Database, KidRepository, TaskRepository, LedgerRepository, UserRepository,
    init_database_with_config, Config, Uuid, verify_password, hash_password
};
#[cfg(feature = "ssr")]
use loaa_core::models::*;
#[cfg(feature = "ssr")]
use loaa_core::workflows::TaskCompletionWorkflow;
#[cfg(feature = "ssr")]
use std::sync::Arc;
#[cfg(feature = "ssr")]
use tokio::sync::OnceCell;
#[cfg(feature = "ssr")]
use std::str::FromStr;
#[cfg(feature = "ssr")]
use tower_sessions::Session;
#[cfg(feature = "ssr")]
use leptos_axum::extract;

// Helper to get database connection
#[cfg(feature = "ssr")]
async fn get_db() -> Result<Arc<Database>, ServerFnError> {
    static DB: OnceCell<Arc<Database>> = OnceCell::const_new();
    DB.get_or_try_init(|| async {
        // Load configuration from environment
        let config = Config::from_env();
        config.validate()
            .map_err(|e| ServerFnError::new(format!("Config validation error: {}", e)))?;

        init_database_with_config(&config.database).await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))
            .map(Arc::new)
    })
    .await
    .cloned()
}

#[server]
pub async fn get_kids() -> Result<Vec<KidDto>, ServerFnError> {
    let db = get_db().await?;
    let kid_repo = KidRepository::new(db.client.clone());
    let kids = kid_repo.list().await
        .map_err(|e| ServerFnError::new(format!("Failed to list kids: {}", e)))?;
    Ok(kids.into_iter().map(Into::into).collect())
}

#[server]
pub async fn create_kid(name: String) -> Result<KidDto, ServerFnError> {
    let kid = Kid::new(name)
        .map_err(|e| ServerFnError::new(format!("Validation error: {}", e)))?;
    let db = get_db().await?;
    let kid_repo = KidRepository::new(db.client.clone());
    let created = kid_repo.create(kid).await
        .map_err(|e| ServerFnError::new(format!("Failed to create kid: {}", e)))?;
    Ok(created.into())
}

#[server]
pub async fn get_tasks() -> Result<Vec<TaskDto>, ServerFnError> {
    let db = get_db().await?;
    let task_repo = TaskRepository::new(db.client.clone());
    let tasks = task_repo.list().await
        .map_err(|e| ServerFnError::new(format!("Failed to list tasks: {}", e)))?;
    Ok(tasks.into_iter().map(Into::into).collect())
}

#[server]
pub async fn create_task(
    name: String,
    description: String,
    value: rust_decimal::Decimal,
    cadence: CadenceDto,
) -> Result<TaskDto, ServerFnError> {
    let task = Task::new(name, description, value, cadence.into())
        .map_err(|e| ServerFnError::new(format!("Validation error: {}", e)))?;
    let db = get_db().await?;
    let task_repo = TaskRepository::new(db.client.clone());
    let created = task_repo.create(task).await
        .map_err(|e| ServerFnError::new(format!("Failed to create task: {}", e)))?;
    Ok(created.into())
}

#[server]
pub async fn complete_task(kid_id: UuidDto, task_id: UuidDto) -> Result<(), ServerFnError> {
    let db = get_db().await?;

    let kid_uuid = Uuid::from_str(&kid_id)
        .map_err(|e| ServerFnError::new(format!("Invalid kid ID: {}", e)))?;
    let task_uuid = Uuid::from_str(&task_id)
        .map_err(|e| ServerFnError::new(format!("Invalid task ID: {}", e)))?;

    // Use the TaskCompletionWorkflow to handle task completion
    // This ensures recurring tasks are properly reset
    let task_repo = TaskRepository::new(db.client.clone());
    let kid_repo = KidRepository::new(db.client.clone());
    let ledger_repo = LedgerRepository::new(db.client.clone());

    let workflow = TaskCompletionWorkflow::new(task_repo, kid_repo, ledger_repo);
    workflow.complete_task(task_uuid, kid_uuid).await
        .map_err(|e| ServerFnError::new(format!("Failed to complete task: {}", e)))?;

    Ok(())
}

#[server]
pub async fn get_ledger(kid_id: UuidDto) -> Result<LedgerDto, ServerFnError> {
    let db = get_db().await?;
    let kid_uuid = Uuid::from_str(&kid_id)
        .map_err(|e| ServerFnError::new(format!("Invalid kid ID: {}", e)))?;
    let ledger_repo = LedgerRepository::new(db.client.clone());
    let ledger = ledger_repo.get_ledger(kid_uuid).await
        .map_err(|e| ServerFnError::new(format!("Failed to get ledger: {}", e)))?;
    Ok(ledger.into())
}

#[server]
pub async fn get_dashboard_data() -> Result<DashboardDataDto, ServerFnError> {
    let db = get_db().await?;
    let kid_repo = KidRepository::new(db.client.clone());
    let task_repo = TaskRepository::new(db.client.clone());
    let ledger_repo = LedgerRepository::new(db.client.clone());

    let kids = kid_repo.list().await
        .map_err(|e| ServerFnError::new(format!("Failed to list kids: {}", e)))?;

    let tasks = task_repo.list().await
        .map_err(|e| ServerFnError::new(format!("Failed to list tasks: {}", e)))?;

    let mut kid_summaries = Vec::new();
    for kid in kids.iter() {
        let ledger = ledger_repo.get_ledger(kid.id).await
            .map_err(|e| ServerFnError::new(format!("Failed to get ledger: {}", e)))?;

        let recent_entry = ledger.entries.last().cloned().map(Into::into);

        kid_summaries.push(KidSummaryDto {
            kid: kid.clone().into(),
            balance: ledger.balance,
            recent_entry,
        });
    }

    Ok(DashboardDataDto {
        kid_summaries,
        total_kids: kids.len(),
        active_tasks: tasks.len(),
    })
}

#[server]
pub async fn get_recent_activity(limit: usize) -> Result<Vec<LedgerEntryDto>, ServerFnError> {
    let db = get_db().await?;
    let kid_repo = KidRepository::new(db.client.clone());
    let ledger_repo = LedgerRepository::new(db.client.clone());

    let kids = kid_repo.list().await
        .map_err(|e| ServerFnError::new(format!("Failed to list kids: {}", e)))?;

    let mut all_entries = Vec::new();
    for kid in kids {
        let entries = ledger_repo.list_entries(kid.id).await
            .map_err(|e| ServerFnError::new(format!("Failed to get ledger entries: {}", e)))?;
        all_entries.extend(entries);
    }

    all_entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    all_entries.truncate(limit);

    Ok(all_entries.into_iter().map(Into::into).collect())
}

#[server]
pub async fn login(username: String, password: String) -> Result<bool, ServerFnError> {
    // Special case: admin user authenticated via environment variable
    if username == "admin" {
        let admin_password = std::env::var("LOAA_ADMIN_PASSWORD")
            .map_err(|_| ServerFnError::new("Admin password not configured. Set LOAA_ADMIN_PASSWORD environment variable.".to_string()))?;

        if password == admin_password {
            // Get session from Axum extractor
            let session = extract::<Session>().await
                .map_err(|e| ServerFnError::new(format!("Failed to extract session: {}", e)))?;

            // Store special admin marker in session
            session.insert("user_id", "admin".to_string())
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to set session: {}", e)))?;

            // Store account type as admin
            session.insert("account_type", "admin".to_string())
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to set session: {}", e)))?;

            eprintln!("✅ Admin login successful");
            return Ok(true);
        } else {
            eprintln!("❌ Admin login failed: incorrect password");
            return Ok(false);
        }
    }

    // Regular database users
    let db = get_db().await?;
    let user_repo = UserRepository::new(db.client.clone());

    // Look up user by username
    let user = match user_repo.get_by_username(&username).await {
        Ok(user) => user,
        Err(_) => return Ok(false), // User not found
    };

    // Verify password
    let is_valid = verify_password(&password, &user.password_hash)
        .map_err(|e| ServerFnError::new(format!("Password verification error: {}", e)))?;

    if is_valid {
        // Get session from Axum extractor
        let session = extract::<Session>().await
            .map_err(|e| ServerFnError::new(format!("Failed to extract session: {}", e)))?;

        // Store user ID in session
        session.insert("user_id", user.id.to_string())
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to set session: {}", e)))?;

        // Store account type in session
        let account_type_str = match user.account_type {
            loaa_core::models::AccountType::Admin => "admin",
            loaa_core::models::AccountType::User => "user",
        };
        session.insert("account_type", account_type_str.to_string())
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to set session: {}", e)))?;

        Ok(true)
    } else {
        Ok(false)
    }
}

#[server]
pub async fn check_pending_oauth() -> Result<Option<String>, ServerFnError> {
    let session = extract::<Session>().await
        .map_err(|e| ServerFnError::new(format!("Failed to extract session: {}", e)))?;

    // Check if there's a pending OAuth flow
    let client_id: Option<String> = session.get("oauth_client_id").await
        .map_err(|e| ServerFnError::new(format!("Session error: {}", e)))?;

    eprintln!("🔍 check_pending_oauth: client_id = {:?}", client_id);

    if client_id.is_none() {
        eprintln!("🔍 check_pending_oauth: No pending OAuth found");
        return Ok(None);
    }

    // Get all OAuth parameters (unwrap client_id since we checked it's Some above)
    let client_id = client_id.unwrap();

    let redirect_uri: String = session.get("oauth_redirect_uri").await
        .map_err(|e| ServerFnError::new(format!("Session error: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Missing redirect_uri".to_string()))?;

    let scope: String = session.get("oauth_scope").await
        .map_err(|e| ServerFnError::new(format!("Session error: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Missing scope".to_string()))?;

    let state: String = session.get("oauth_state").await
        .map_err(|e| ServerFnError::new(format!("Session error: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Missing state".to_string()))?;

    let code_challenge: String = session.get("oauth_code_challenge").await
        .map_err(|e| ServerFnError::new(format!("Session error: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Missing code_challenge".to_string()))?;

    let code_challenge_method: String = session.get("oauth_code_challenge_method").await
        .map_err(|e| ServerFnError::new(format!("Session error: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Missing code_challenge_method".to_string()))?;

    // Build the OAuth authorize URL
    let oauth_url = format!(
        "/oauth/authorize?client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method={}",
        client_id, redirect_uri, scope, state, code_challenge, code_challenge_method
    );

    eprintln!("🔍 check_pending_oauth: Built OAuth URL: {}", oauth_url);

    Ok(Some(oauth_url))
}

#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    let session = extract::<Session>().await
        .map_err(|e| ServerFnError::new(format!("Failed to extract session: {}", e)))?;

    session.delete()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to delete session: {}", e)))?;

    Ok(())
}

#[server]
pub async fn check_auth() -> Result<bool, ServerFnError> {
    let session = extract::<Session>().await
        .map_err(|e| ServerFnError::new(format!("Failed to extract session: {}", e)))?;

    let user_id: Option<String> = session.get("user_id")
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to get session: {}", e)))?;

    Ok(user_id.is_some())
}

#[server]
pub async fn get_account_type() -> Result<AccountTypeDto, ServerFnError> {
    let session = extract::<Session>().await
        .map_err(|e| ServerFnError::new(format!("Failed to extract session: {}", e)))?;

    let account_type: Option<String> = session.get("account_type")
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to get session: {}", e)))?;

    match account_type.as_deref() {
        Some("admin") => Ok(AccountTypeDto::Admin),
        _ => Ok(AccountTypeDto::User),
    }
}

// Helper to verify admin access
#[cfg(feature = "ssr")]
async fn require_admin() -> Result<(), ServerFnError> {
    let session = extract::<Session>().await
        .map_err(|e| ServerFnError::new(format!("Failed to extract session: {}", e)))?;

    let account_type: Option<String> = session.get("account_type")
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to get session: {}", e)))?;

    if account_type.as_deref() != Some("admin") {
        return Err(ServerFnError::new("Admin access required".to_string()));
    }
    Ok(())
}

#[server]
pub async fn list_accounts() -> Result<Vec<AccountDto>, ServerFnError> {
    require_admin().await?;

    let db = get_db().await?;
    let user_repo = UserRepository::new(db.client.clone());

    let users = user_repo.list().await
        .map_err(|e| ServerFnError::new(format!("Failed to list users: {}", e)))?;

    Ok(users.into_iter().map(Into::into).collect())
}

#[server]
pub async fn create_account(username: String, password: String) -> Result<AccountDto, ServerFnError> {
    require_admin().await?;

    let db = get_db().await?;
    let user_repo = UserRepository::new(db.client.clone());

    // Check if username already exists
    if user_repo.get_by_username(&username).await.is_ok() {
        return Err(ServerFnError::new(format!("Username '{}' already exists", username)));
    }

    // Create new user
    let mut user = loaa_core::models::User::new(username)
        .map_err(|e| ServerFnError::new(format!("Invalid user data: {}", e)))?;

    // Hash password
    user.password_hash = hash_password(&password)
        .map_err(|e| ServerFnError::new(format!("Failed to hash password: {}", e)))?;

    // Save to database
    let created = user_repo.create(user).await
        .map_err(|e| ServerFnError::new(format!("Failed to create user: {}", e)))?;

    eprintln!("✅ Created account: {}", created.username);
    Ok(created.into())
}

#[server]
pub async fn delete_account(user_id: String) -> Result<(), ServerFnError> {
    require_admin().await?;

    let uuid = uuid::Uuid::parse_str(&user_id)
        .map_err(|e| ServerFnError::new(format!("Invalid user ID: {}", e)))?;

    let db = get_db().await?;
    let user_repo = UserRepository::new(db.client.clone());

    // Get the user first to log who we're deleting
    let user = user_repo.get(uuid).await
        .map_err(|e| ServerFnError::new(format!("User not found: {}", e)))?;

    user_repo.delete(uuid).await
        .map_err(|e| ServerFnError::new(format!("Failed to delete user: {}", e)))?;

    eprintln!("🗑️ Deleted account: {}", user.username);
    Ok(())
}
