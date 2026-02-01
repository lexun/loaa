//! REST API endpoints that bypass Leptos server functions
//!
//! These endpoints use Axum's direct extractors to avoid the ResponseOptions
//! context race condition bug in Leptos 0.6 (GitHub issue #2112).

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use tower_sessions::Session;
use uuid::Uuid;
use loaa_core::{KidRepository, TaskRepository, LedgerRepository, UserRepository};

use crate::dto::{DashboardDataDto, KidSummaryDto};
use crate::oauth::AppState;

/// Get dashboard data - bypasses Leptos server function to avoid context panics
pub async fn get_dashboard_data_handler(
    State(state): State<AppState>,
    session: Session,
) -> Result<Json<DashboardDataDto>, (StatusCode, String)> {
    // Get user_id from session
    let user_id: Option<String> = session.get("user_id")
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Session error: {}", e)))?;

    let user_id_str = user_id.ok_or_else(|| {
        (StatusCode::UNAUTHORIZED, "Not authenticated".to_string())
    })?;

    // Parse user ID
    let user_id = Uuid::parse_str(&user_id_str)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Invalid user ID: {}", e)))?;

    // Get user's account_id
    let user_repo = UserRepository::new(state.db.client.clone());
    let user = user_repo.get(user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get user: {}", e)))?;

    let account_id = user.account_id;
    eprintln!("🔍 API get_dashboard_data: account_id = {}", account_id);

    // Get repositories
    let kid_repo = KidRepository::new(state.db.client.clone());
    let task_repo = TaskRepository::new(state.db.client.clone());
    let ledger_repo = LedgerRepository::new(state.db.client.clone());

    // Fetch kids
    let kids = kid_repo.list_by_account(account_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to list kids: {}", e)))?;
    eprintln!("🔍 API get_dashboard_data: found {} kids", kids.len());

    // Fetch tasks
    let tasks = task_repo.list_by_account(account_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to list tasks: {}", e)))?;
    eprintln!("🔍 API get_dashboard_data: found {} tasks", tasks.len());

    // Build kid summaries with ledger data
    let mut kid_summaries = Vec::new();
    for kid in kids.iter() {
        let ledger = ledger_repo.get_ledger(kid.id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get ledger: {}", e)))?;

        let recent_entry = ledger.entries.first().cloned().map(Into::into);

        kid_summaries.push(KidSummaryDto {
            kid: kid.clone().into(),
            balance: ledger.balance,
            recent_entry,
        });
    }

    Ok(Json(DashboardDataDto {
        kid_summaries,
        total_kids: kids.len(),
        active_tasks: tasks.len(),
    }))
}
