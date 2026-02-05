use leptos::*;
use server_fn::codec::Json;
use crate::dto::*;

#[cfg(feature = "ssr")]
use loaa_core::{
    Database, KidRepository, TaskRepository, LedgerRepository, UserRepository, AccountRepository,
    AccountMembershipRepository,
    init_database_with_config, Config, Uuid, verify_password, hash_password,
    EventSender,
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
#[cfg(feature = "ssr")]
use std::sync::OnceLock;

// Global event sender for SSE broadcasts
// This is set once during server initialization and read by server functions
#[cfg(feature = "ssr")]
static EVENT_SENDER: OnceLock<EventSender> = OnceLock::new();

/// Set the global event sender (called from main.rs during initialization)
#[cfg(feature = "ssr")]
pub fn set_event_sender(sender: EventSender) {
    let _ = EVENT_SENDER.set(sender);
}

/// Get the global event sender for SSE broadcasts
#[cfg(feature = "ssr")]
pub fn get_event_sender() -> Option<EventSender> {
    EVENT_SENDER.get().cloned()
}

// Helper to get database connection - shared across all server functions
// This uses OnceCell to ensure only one database instance exists
#[cfg(feature = "ssr")]
pub async fn get_db() -> Result<Arc<Database>, ServerFnError> {
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
    let account_id = get_account_id().await?;
    eprintln!("🔍 get_kids: account_id = {}", account_id);
    let db = get_db().await?;
    let kid_repo = KidRepository::new(db.client.clone());
    let kids = kid_repo.list_by_account(account_id).await
        .map_err(|e| ServerFnError::new(format!("Failed to list kids: {}", e)))?;
    eprintln!("🔍 get_kids: found {} kids", kids.len());
    Ok(kids.into_iter().map(Into::into).collect())
}

#[server]
pub async fn create_kid(name: String) -> Result<KidDto, ServerFnError> {
    let owner_id = get_owner_id().await?;
    let account_id = get_account_id().await?;
    let kid = Kid::new(name, account_id, owner_id)
        .map_err(|e| ServerFnError::new(format!("Validation error: {}", e)))?;
    let db = get_db().await?;
    let kid_repo = KidRepository::new(db.client.clone());
    let created = kid_repo.create(kid).await
        .map_err(|e| ServerFnError::new(format!("Failed to create kid: {}", e)))?;
    Ok(created.into())
}

#[server]
pub async fn get_tasks() -> Result<Vec<TaskDto>, ServerFnError> {
    let account_id = get_account_id().await?;
    let db = get_db().await?;
    let task_repo = TaskRepository::new(db.client.clone());
    let tasks = task_repo.list_by_account(account_id).await
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
    let owner_id = get_owner_id().await?;
    let account_id = get_account_id().await?;
    let task = Task::new(name, description, value, cadence.into(), account_id, owner_id)
        .map_err(|e| ServerFnError::new(format!("Validation error: {}", e)))?;
    let db = get_db().await?;
    let task_repo = TaskRepository::new(db.client.clone());
    let created = task_repo.create(task).await
        .map_err(|e| ServerFnError::new(format!("Failed to create task: {}", e)))?;
    Ok(created.into())
}

/// Complete a task for a kid
/// - completed_at: Optional timestamp for when the task was actually completed (for backdating)
///   If None, the task is completed as of now
///   If provided, creates a ledger entry dated to that time
///   Backdated completions (from a previous period) don't mark the task as unavailable today
#[server]
pub async fn complete_task(
    kid_id: UuidDto,
    task_id: UuidDto,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<(), ServerFnError> {
    let db = get_db().await?;

    let kid_uuid = Uuid::from_str(&kid_id)
        .map_err(|e| ServerFnError::new(format!("Invalid kid ID: {}", e)))?;
    let task_uuid = Uuid::from_str(&task_id)
        .map_err(|e| ServerFnError::new(format!("Invalid task ID: {}", e)))?;

    // Get session to check membership role
    let session = extract::<Session>().await
        .map_err(|e| ServerFnError::new(format!("Failed to extract session: {}", e)))?;

    let membership_role: Option<String> = session.get("membership_role").await
        .map_err(|e| ServerFnError::new(format!("Session error: {}", e)))?;

    let user_id: Option<String> = session.get("user_id").await
        .map_err(|e| ServerFnError::new(format!("Session error: {}", e)))?;

    // Determine transaction status based on role
    // Kid users create pending transactions, parents create confirmed
    let status = match membership_role.as_deref() {
        Some("kid") => TransactionStatus::Pending,
        _ => TransactionStatus::Confirmed,
    };

    // Use the TaskCompletionWorkflow to handle task completion
    // This ensures recurring tasks are properly reset
    let task_repo = TaskRepository::new(db.client.clone());
    let kid_repo = KidRepository::new(db.client.clone());
    let ledger_repo = LedgerRepository::new(db.client.clone());

    let workflow = TaskCompletionWorkflow::new(task_repo, kid_repo, ledger_repo);
    workflow.complete_task_with_status(task_uuid, kid_uuid, status, user_id, completed_at).await
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
    let account_id = get_account_id().await?;
    eprintln!("🔍 get_dashboard_data: account_id = {}", account_id);
    let db = get_db().await?;
    let kid_repo = KidRepository::new(db.client.clone());
    let task_repo = TaskRepository::new(db.client.clone());
    let ledger_repo = LedgerRepository::new(db.client.clone());

    // Debug: list ALL kids first to see if any exist
    let all_kids = kid_repo.list().await
        .map_err(|e| ServerFnError::new(format!("Failed to list all kids: {}", e)))?;
    eprintln!("🔍 get_dashboard_data: total kids in DB = {}", all_kids.len());
    for k in &all_kids {
        eprintln!("🔍   kid: {} account_id={}", k.name, k.account_id);
    }

    let kids = kid_repo.list_by_account(account_id).await
        .map_err(|e| ServerFnError::new(format!("Failed to list kids: {}", e)))?;
    eprintln!("🔍 get_dashboard_data: found {} kids for account {}", kids.len(), account_id);

    let tasks = task_repo.list_by_account(account_id).await
        .map_err(|e| ServerFnError::new(format!("Failed to list tasks: {}", e)))?;
    eprintln!("🔍 get_dashboard_data: found {} tasks", tasks.len());

    let mut kid_summaries = Vec::new();
    for kid in kids.iter() {
        let ledger = ledger_repo.get_ledger(kid.id).await
            .map_err(|e| ServerFnError::new(format!("Failed to get ledger: {}", e)))?;

        let recent_entry = ledger.entries.first().cloned().map(Into::into);

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
    let account_id = get_account_id().await?;
    let db = get_db().await?;
    let kid_repo = KidRepository::new(db.client.clone());
    let ledger_repo = LedgerRepository::new(db.client.clone());

    let kids = kid_repo.list_by_account(account_id).await
        .map_err(|e| ServerFnError::new(format!("Failed to list kids: {}", e)))?;

    let mut all_entries = Vec::new();
    for kid in kids {
        let entries = ledger_repo.list_entries(kid.id).await
            .map_err(|e| ServerFnError::new(format!("Failed to get ledger entries: {}", e)))?;
        all_entries.extend(entries);
    }

    // Sort by effective date: completed_at if backdated, otherwise created_at
    all_entries.sort_by(|a, b| b.effective_completed_at().cmp(&a.effective_completed_at()));
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

        // Look up membership role (create Parent if missing for existing users)
        let membership_repo = AccountMembershipRepository::new(db.client.clone());
        let membership = match membership_repo.get_primary_for_user(user.id).await {
            Ok(m) => m,
            Err(_) => {
                // No membership found - create one for existing user
                // This handles backward compatibility for users created before memberships
                let new_membership = AccountMembership::new_parent(user.account_id, user.id);
                membership_repo.create(new_membership).await
                    .map_err(|e| ServerFnError::new(format!("Failed to create membership: {}", e)))?
            }
        };

        // Store membership role in session
        let role_str = match membership.role {
            MembershipRole::Parent => "parent",
            MembershipRole::Kid => "kid",
        };
        session.insert("membership_role", role_str.to_string())
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to set session: {}", e)))?;

        // Store linked kid_id if present (for kid users linked to a specific kid)
        if let Some(kid_id) = membership.kid_id {
            session.insert("linked_kid_id", kid_id.to_string())
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to set session: {}", e)))?;
        }

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

/// Check if the current user is a parent (not a kid)
#[server]
pub async fn is_parent() -> Result<bool, ServerFnError> {
    let session = extract::<Session>().await
        .map_err(|e| ServerFnError::new(format!("Failed to extract session: {}", e)))?;

    let membership_role: Option<String> = session.get("membership_role")
        .await
        .map_err(|e| ServerFnError::new(format!("Session error: {}", e)))?;

    // User is a parent if their role is "parent" or if they have no role (legacy users)
    Ok(membership_role.as_deref() != Some("kid"))
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

// Helper to get the current owner_id from session
#[cfg(feature = "ssr")]
async fn get_owner_id() -> Result<String, ServerFnError> {
    let session = extract::<Session>().await
        .map_err(|e| ServerFnError::new(format!("Failed to extract session: {}", e)))?;

    let user_id: Option<String> = session.get("user_id")
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to get session: {}", e)))?;

    user_id.ok_or_else(|| ServerFnError::new("Not authenticated".to_string()))
}

// Helper to get the current account_id by looking up the user
#[cfg(feature = "ssr")]
async fn get_account_id() -> Result<Uuid, ServerFnError> {
    let owner_id = get_owner_id().await?;
    let user_id = Uuid::parse_str(&owner_id)
        .map_err(|e| ServerFnError::new(format!("Invalid user ID: {}", e)))?;

    let db = get_db().await?;
    let user_repo = UserRepository::new(db.client.clone());

    let user = user_repo.get(user_id).await
        .map_err(|e| ServerFnError::new(format!("Failed to get user: {}", e)))?;

    Ok(user.account_id)
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
    let account_repo = AccountRepository::new(db.client.clone());

    // Check if username already exists
    if user_repo.get_by_username(&username).await.is_ok() {
        return Err(ServerFnError::new(format!("Username '{}' already exists", username)));
    }

    // Create a new account for this user (their household)
    let account = loaa_core::models::Account::new(format!("{}'s Household", username))
        .map_err(|e| ServerFnError::new(format!("Failed to create account: {}", e)))?;
    let created_account = account_repo.create(account).await
        .map_err(|e| ServerFnError::new(format!("Failed to save account: {}", e)))?;

    // Create new user with the account_id
    let mut user = loaa_core::models::User::new(username, created_account.id)
        .map_err(|e| ServerFnError::new(format!("Invalid user data: {}", e)))?;

    // Hash password
    user.password_hash = hash_password(&password)
        .map_err(|e| ServerFnError::new(format!("Failed to hash password: {}", e)))?;

    // Save to database
    let created = user_repo.create(user).await
        .map_err(|e| ServerFnError::new(format!("Failed to create user: {}", e)))?;

    eprintln!("✅ Created account: {} (account: {})", created.username, created_account.id);
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

/// Send a chat message and get a response from Claude
#[server(input = Json)]
pub async fn send_chat_message(
    message: String,
    #[server(default)]
    history: Vec<ChatMessageDto>,
) -> Result<String, ServerFnError> {
    use crate::claude::{chat, ChatMessage, MessageContent};
    use crate::oauth::AppState;

    // Get database connection
    let db = get_db().await?;

    // Get account_id (works for both parents and kids)
    let account_id = get_account_id().await?;

    // Get membership role and linked kid from session
    let session = extract::<Session>().await
        .map_err(|e| ServerFnError::new(format!("Failed to extract session: {}", e)))?;
    let membership_role: Option<String> = session.get("membership_role").await
        .map_err(|e| ServerFnError::new(format!("Session error: {}", e)))?;
    let is_kid = membership_role.as_deref() == Some("kid");

    // Get kid info if this is a kid user with a linked kid
    let kid_info: Option<(String, String)> = if is_kid {
        let linked_kid_id: Option<String> = session.get("linked_kid_id").await
            .map_err(|e| ServerFnError::new(format!("Session error: {}", e)))?;

        if let Some(kid_id_str) = linked_kid_id {
            // Look up the kid's name
            let kid_repo = KidRepository::new(db.client.clone());
            let kid_id = Uuid::from_str(&kid_id_str)
                .map_err(|e| ServerFnError::new(format!("Invalid kid ID: {}", e)))?;

            match kid_repo.get(kid_id).await {
                Ok(kid) => Some((kid.name, kid_id_str)),
                Err(_) => None, // Kid not found, proceed without kid info
            }
        } else {
            None // Shared kid account, no specific kid
        }
    } else {
        None // Parent user
    };

    // Create a minimal AppState for the chat function
    // We only need the db and event_sender fields for tool execution
    let app_state = AppState {
        leptos_options: leptos::LeptosOptions::default(),
        oauth_state: std::sync::Arc::new(tokio::sync::RwLock::new(
            crate::oauth::OAuthState::new()
        )),
        base_url: String::new(),
        jwt_secret: String::new(),
        db,
        event_sender: get_event_sender(),
    };

    // Convert history to internal format
    let mut messages: Vec<ChatMessage> = history
        .into_iter()
        .map(|m| ChatMessage {
            role: m.role,
            content: MessageContent::Text(m.content),
        })
        .collect();

    // Add the new user message
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: MessageContent::Text(message),
    });

    // Call Claude with account_id, role, and kid info for multi-user support
    let response = chat(messages, &app_state, account_id, is_kid, kid_info).await
        .map_err(|e| ServerFnError::new(format!("Chat error: {}", e)))?;

    Ok(response)
}

// ============================================================================
// Transaction Approval Functions (for kid login workflow)
// ============================================================================

/// List pending transactions for the current user's account
/// Only parents can see pending transactions
#[server]
pub async fn list_pending_transactions() -> Result<Vec<LedgerEntryDto>, ServerFnError> {
    let account_id = get_account_id().await?;
    let db = get_db().await?;

    // Get session to check membership role
    let session = extract::<Session>().await
        .map_err(|e| ServerFnError::new(format!("Failed to extract session: {}", e)))?;

    let membership_role: Option<String> = session.get("membership_role").await
        .map_err(|e| ServerFnError::new(format!("Session error: {}", e)))?;

    // Only parents can view pending transactions
    if membership_role.as_deref() == Some("kid") {
        return Ok(Vec::new());
    }

    // Get all kids for this account
    let kid_repo = KidRepository::new(db.client.clone());
    let kids = kid_repo.list_by_account(account_id).await
        .map_err(|e| ServerFnError::new(format!("Failed to list kids: {}", e)))?;

    let kid_ids: Vec<Uuid> = kids.iter().map(|k| k.id).collect();

    // Get pending transactions for all kids
    let ledger_repo = LedgerRepository::new(db.client.clone());
    let pending = ledger_repo.list_pending_for_kids(&kid_ids).await
        .map_err(|e| ServerFnError::new(format!("Failed to list pending: {}", e)))?;

    Ok(pending.into_iter().map(Into::into).collect())
}

/// Approve a pending transaction (parent only)
#[server]
pub async fn approve_transaction(entry_id: UuidDto) -> Result<(), ServerFnError> {
    let db = get_db().await?;

    // Get session to check membership role
    let session = extract::<Session>().await
        .map_err(|e| ServerFnError::new(format!("Failed to extract session: {}", e)))?;

    let membership_role: Option<String> = session.get("membership_role").await
        .map_err(|e| ServerFnError::new(format!("Session error: {}", e)))?;

    // Only parents can approve transactions
    if membership_role.as_deref() == Some("kid") {
        return Err(ServerFnError::new("Only parents can approve transactions".to_string()));
    }

    let entry_uuid = Uuid::from_str(&entry_id)
        .map_err(|e| ServerFnError::new(format!("Invalid entry ID: {}", e)))?;

    let ledger_repo = LedgerRepository::new(db.client.clone());

    // Verify the entry exists and is pending
    let entry = ledger_repo.get_entry(entry_uuid).await
        .map_err(|e| ServerFnError::new(format!("Failed to get entry: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Transaction not found".to_string()))?;

    if entry.status != TransactionStatus::Pending {
        return Err(ServerFnError::new("Transaction is not pending".to_string()));
    }

    // Update status to Confirmed
    ledger_repo.update_status(entry_uuid, TransactionStatus::Confirmed).await
        .map_err(|e| ServerFnError::new(format!("Failed to approve: {}", e)))?;

    Ok(())
}

/// Create a kid login for the current account (parent only)
/// The kid login can optionally be linked to a specific kid record
#[server]
pub async fn create_kid_login(
    username: String,
    password: String,
    linked_kid_id: Option<UuidDto>,
) -> Result<(), ServerFnError> {
    let db = get_db().await?;

    // Get session to check membership role and get user info
    let session = extract::<Session>().await
        .map_err(|e| ServerFnError::new(format!("Failed to extract session: {}", e)))?;

    let membership_role: Option<String> = session.get("membership_role").await
        .map_err(|e| ServerFnError::new(format!("Session error: {}", e)))?;

    // Only parents can create kid logins
    if membership_role.as_deref() == Some("kid") {
        return Err(ServerFnError::new("Only parents can create kid logins".to_string()));
    }

    let user_id_str: Option<String> = session.get("user_id").await
        .map_err(|e| ServerFnError::new(format!("Session error: {}", e)))?;

    let user_id_str = user_id_str.ok_or_else(|| ServerFnError::new("Not logged in".to_string()))?;

    // Get the parent user to get account_id
    let user_repo = UserRepository::new(db.client.clone());
    let parent_user = if user_id_str == "admin" {
        return Err(ServerFnError::new("Admin cannot create kid logins".to_string()));
    } else {
        let user_id = Uuid::from_str(&user_id_str)
            .map_err(|e| ServerFnError::new(format!("Invalid user ID: {}", e)))?;
        user_repo.get(user_id).await
            .map_err(|e| ServerFnError::new(format!("Failed to get user: {}", e)))?
    };

    // Check if username already exists
    if user_repo.get_by_username(&username).await.is_ok() {
        return Err(ServerFnError::new("Username already exists".to_string()));
    }

    // Check if the kid already has a login (enforce one user per kid)
    let membership_repo = AccountMembershipRepository::new(db.client.clone());
    if let Some(kid_id_str) = &linked_kid_id {
        let kid_id = Uuid::from_str(kid_id_str)
            .map_err(|e| ServerFnError::new(format!("Invalid kid ID: {}", e)))?;
        if let Ok(Some(_existing)) = membership_repo.get_by_kid_id(kid_id).await {
            return Err(ServerFnError::new("This kid already has a login. Delete the existing login first or edit their password.".to_string()));
        }
    }

    // Hash password
    let password_hash = hash_password(&password)
        .map_err(|e| ServerFnError::new(format!("Failed to hash password: {}", e)))?;

    // Create the kid user
    let mut kid_user = User::new(username.clone(), parent_user.account_id)
        .map_err(|e| ServerFnError::new(format!("Failed to create user: {}", e)))?;
    kid_user.password_hash = password_hash;

    let created_user = user_repo.create(kid_user).await
        .map_err(|e| ServerFnError::new(format!("Failed to save user: {}", e)))?;

    // Create AccountMembership with Kid role
    let membership = if let Some(kid_id_str) = linked_kid_id {
        let kid_id = Uuid::from_str(&kid_id_str)
            .map_err(|e| ServerFnError::new(format!("Invalid kid ID: {}", e)))?;
        AccountMembership::new_kid(parent_user.account_id, created_user.id, kid_id)
    } else {
        AccountMembership::new_kid_shared(parent_user.account_id, created_user.id)
    };

    membership_repo.create(membership).await
        .map_err(|e| ServerFnError::new(format!("Failed to create membership: {}", e)))?;

    eprintln!("✅ Created kid login: {}", username);
    Ok(())
}

/// List kid logins for the current account (parent only)
#[server]
pub async fn list_kid_logins() -> Result<Vec<KidLoginDto>, ServerFnError> {
    let db = get_db().await?;

    // Get session to check membership role and get user info
    let session = extract::<Session>().await
        .map_err(|e| ServerFnError::new(format!("Failed to extract session: {}", e)))?;

    let membership_role: Option<String> = session.get("membership_role").await
        .map_err(|e| ServerFnError::new(format!("Session error: {}", e)))?;

    // Only parents can list kid logins
    if membership_role.as_deref() == Some("kid") {
        return Ok(Vec::new());
    }

    let user_id_str: Option<String> = session.get("user_id").await
        .map_err(|e| ServerFnError::new(format!("Session error: {}", e)))?;

    let user_id_str = user_id_str.ok_or_else(|| ServerFnError::new("Not logged in".to_string()))?;

    if user_id_str == "admin" {
        return Ok(Vec::new());
    }

    let user_id = Uuid::from_str(&user_id_str)
        .map_err(|e| ServerFnError::new(format!("Invalid user ID: {}", e)))?;

    // Get the parent user to get account_id
    let user_repo = UserRepository::new(db.client.clone());
    let parent_user = user_repo.get(user_id).await
        .map_err(|e| ServerFnError::new(format!("Failed to get user: {}", e)))?;

    // Get all memberships for this account
    let membership_repo = AccountMembershipRepository::new(db.client.clone());
    let memberships = membership_repo.list_by_account(parent_user.account_id).await
        .map_err(|e| ServerFnError::new(format!("Failed to list memberships: {}", e)))?;

    // Filter for kid memberships and get user info
    let kid_repo = KidRepository::new(db.client.clone());
    let mut kid_logins = Vec::new();
    for membership in memberships {
        if membership.is_kid() {
            if let Ok(user) = user_repo.get(membership.user_id).await {
                // Look up kid name if linked
                let linked_kid_name = if let Some(kid_id) = membership.kid_id {
                    kid_repo.get(kid_id).await.ok().map(|kid| kid.name)
                } else {
                    None
                };

                kid_logins.push(KidLoginDto {
                    user_id: user.id.to_string(),
                    username: user.username,
                    linked_kid_id: membership.kid_id.map(|id| id.to_string()),
                    linked_kid_name,
                });
            }
        }
    }

    Ok(kid_logins)
}

/// Update a kid login's password (parent only)
#[server]
pub async fn update_kid_password(user_id: UuidDto, new_password: String) -> Result<(), ServerFnError> {
    let db = get_db().await?;

    // Get session to check membership role
    let session = extract::<Session>().await
        .map_err(|e| ServerFnError::new(format!("Failed to extract session: {}", e)))?;

    let membership_role: Option<String> = session.get("membership_role").await
        .map_err(|e| ServerFnError::new(format!("Session error: {}", e)))?;

    // Only parents can update kid passwords
    if membership_role.as_deref() == Some("kid") {
        return Err(ServerFnError::new("Only parents can update kid passwords".to_string()));
    }

    let parent_user_id_str: Option<String> = session.get("user_id").await
        .map_err(|e| ServerFnError::new(format!("Session error: {}", e)))?;

    let parent_user_id_str = parent_user_id_str.ok_or_else(|| ServerFnError::new("Not logged in".to_string()))?;

    if parent_user_id_str == "admin" {
        return Err(ServerFnError::new("Admin cannot update kid passwords".to_string()));
    }

    let parent_user_id = Uuid::from_str(&parent_user_id_str)
        .map_err(|e| ServerFnError::new(format!("Invalid user ID: {}", e)))?;

    // Get the parent user to get account_id
    let user_repo = UserRepository::new(db.client.clone());
    let parent_user = user_repo.get(parent_user_id).await
        .map_err(|e| ServerFnError::new(format!("Failed to get user: {}", e)))?;

    // Parse the kid user ID
    let kid_user_id = Uuid::from_str(&user_id)
        .map_err(|e| ServerFnError::new(format!("Invalid user ID: {}", e)))?;

    // Get the kid user and verify they belong to the same account
    let kid_user = user_repo.get(kid_user_id).await
        .map_err(|e| ServerFnError::new(format!("Failed to get kid user: {}", e)))?;

    if kid_user.account_id != parent_user.account_id {
        return Err(ServerFnError::new("Cannot update password for user in different account".to_string()));
    }

    // Verify this user is actually a kid (has kid membership)
    let membership_repo = AccountMembershipRepository::new(db.client.clone());
    let membership = membership_repo.get_by_user_and_account(kid_user_id, parent_user.account_id).await
        .map_err(|e| ServerFnError::new(format!("Failed to get membership: {}", e)))?;

    if !membership.is_kid() {
        return Err(ServerFnError::new("Cannot update password for non-kid user".to_string()));
    }

    // Hash the new password
    let password_hash = hash_password(&new_password)
        .map_err(|e| ServerFnError::new(format!("Failed to hash password: {}", e)))?;

    // Update the user
    let mut updated_user = kid_user;
    updated_user.password_hash = password_hash;
    user_repo.update(updated_user).await
        .map_err(|e| ServerFnError::new(format!("Failed to update user: {}", e)))?;

    eprintln!("✅ Updated password for kid user: {}", user_id);
    Ok(())
}

/// Reject a pending transaction (parent only)
/// This deletes the ledger entry and removes the kid from the task's completed_by list
#[server]
pub async fn reject_transaction(entry_id: UuidDto) -> Result<(), ServerFnError> {
    let db = get_db().await?;

    // Get session to check membership role
    let session = extract::<Session>().await
        .map_err(|e| ServerFnError::new(format!("Failed to extract session: {}", e)))?;

    let membership_role: Option<String> = session.get("membership_role").await
        .map_err(|e| ServerFnError::new(format!("Session error: {}", e)))?;

    // Only parents can reject transactions
    if membership_role.as_deref() == Some("kid") {
        return Err(ServerFnError::new("Only parents can reject transactions".to_string()));
    }

    let entry_uuid = Uuid::from_str(&entry_id)
        .map_err(|e| ServerFnError::new(format!("Invalid entry ID: {}", e)))?;

    let ledger_repo = LedgerRepository::new(db.client.clone());

    // Get the entry to find task_id and kid_id
    let entry = ledger_repo.get_entry(entry_uuid).await
        .map_err(|e| ServerFnError::new(format!("Failed to get entry: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Transaction not found".to_string()))?;

    if entry.status != TransactionStatus::Pending {
        return Err(ServerFnError::new("Transaction is not pending".to_string()));
    }

    // Remove kid from task's completed_by list (if task_id is present)
    if let Some(task_id) = entry.task_id {
        let task_repo = TaskRepository::new(db.client.clone());
        if let Ok(mut task) = task_repo.get(task_id).await {
            task.completed_by.retain(|&id| id != entry.kid_id);
            task_repo.update(task).await
                .map_err(|e| ServerFnError::new(format!("Failed to update task: {}", e)))?;
        }
    }

    // Delete the ledger entry
    ledger_repo.delete(entry_uuid).await
        .map_err(|e| ServerFnError::new(format!("Failed to delete entry: {}", e)))?;

    Ok(())
}
