//! Claude API client for embedded chat functionality
//!
//! This module handles communication with the Claude API, including tool use
//! for kids to report completed tasks via natural conversation.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::oauth::AppState;
use loaa_core::db::{KidRepository, TaskRepository, LedgerRepository};
use loaa_core::events::{DataEvent, broadcast_event};
use loaa_core::models::{Cadence, TransactionStatus};
use loaa_core::workflows::TaskCompletionWorkflow;

const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";
const MODEL: &str = "claude-sonnet-4-20250514";

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: MessageContent,
}

/// Message content can be text or tool-related
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
    },
}

/// Claude API response
#[derive(Debug, Deserialize)]
pub struct ClaudeResponse {
    pub id: String,
    pub content: Vec<ContentBlock>,
    pub stop_reason: Option<String>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Tool definitions for Claude
fn get_tools() -> Vec<Value> {
    vec![
        json!({
            "name": "list_kids",
            "description": "List all kids in the family. Use this to find out which kids exist and their IDs.",
            "input_schema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }),
        json!({
            "name": "list_tasks",
            "description": "List all available tasks/chores. Use this to see what tasks exist, their values, and whether they can be completed.",
            "input_schema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }),
        json!({
            "name": "complete_task",
            "description": "Mark a task as complete for a specific kid. This awards them the task's value. Supports backdating - if the task was completed in the past (e.g., yesterday), provide completed_at to record the correct date. Backdated completions won't prevent the task from being available today.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "kid_id": {
                        "type": "string",
                        "description": "The UUID of the kid who completed the task"
                    },
                    "task_id": {
                        "type": "string",
                        "description": "The UUID of the task that was completed"
                    },
                    "completed_at": {
                        "type": "string",
                        "description": "Optional ISO 8601 datetime when the task was actually completed (e.g., '2024-01-15T10:00:00Z' for a specific time, or use relative terms in conversation like 'yesterday'). If not provided, defaults to now."
                    }
                },
                "required": ["kid_id", "task_id"]
            }
        }),
        json!({
            "name": "get_balance",
            "description": "Get a kid's current balance.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "kid_id": {
                        "type": "string",
                        "description": "The UUID of the kid"
                    }
                },
                "required": ["kid_id"]
            }
        }),
    ]
}

/// System prompt for the chat assistant
/// If kid_info is provided (name, id), the assistant knows who is talking
fn get_system_prompt(kid_info: Option<(&str, &str)>) -> String {
    let base_prompt = r#"You are a friendly assistant for Loa'a, a family chore tracking app. You help kids report completed chores and check their balances.

Your job is to:
1. Help them report completed tasks/chores
2. Tell them how much they earned and their new balance
3. Be encouraging and positive!

When they say they completed a task:
1. Use list_tasks to find the task they're talking about
2. Use complete_task to mark it done
3. Tell them how much they earned!

BACKDATING: If someone says they did a task "yesterday" or on a past date, use the completed_at parameter with an ISO 8601 datetime. For example:
- "yesterday" → use yesterday's date at noon (e.g., "2024-01-14T12:00:00Z")
- "two days ago" → calculate that date
Backdated completions are recorded with the correct date and don't prevent the task from being done again today.

Keep responses short and friendly - these are kids! Use simple language and be encouraging.

If you can't find a matching task, ask for clarification. Be helpful, not robotic."#;

    match kid_info {
        Some((name, id)) => {
            format!(
                "{}\n\nIMPORTANT: You are talking to {}. Their kid_id is {}. You do NOT need to ask for their name or look them up - use this ID directly when completing tasks or checking balance.",
                base_prompt, name, id
            )
        }
        None => {
            format!(
                "{}\n\nIf the user hasn't identified themselves, ask for their name. Use list_kids to find their ID by matching their name.",
                base_prompt
            )
        }
    }
}

/// Execute a tool call and return the result
async fn execute_tool(
    name: &str,
    input: &Value,
    state: &AppState,
    account_id: uuid::Uuid,
    is_kid: bool,
) -> Result<String> {
    match name {
        "list_kids" => {
            let kid_repo = KidRepository::new(state.db.client.clone());
            let kids = kid_repo.list_by_account(account_id).await?;
            let result: Vec<Value> = kids
                .iter()
                .map(|k| json!({
                    "id": k.id.to_string(),
                    "name": k.name
                }))
                .collect();
            Ok(serde_json::to_string_pretty(&result)?)
        }
        "list_tasks" => {
            let task_repo = TaskRepository::new(state.db.client.clone());
            let kid_repo = KidRepository::new(state.db.client.clone());
            let tasks = task_repo.list_by_account(account_id).await?;
            let kids = kid_repo.list_by_account(account_id).await?;

            // Build a map of kid IDs to names for resolving completed_by
            let kid_names: std::collections::HashMap<uuid::Uuid, String> = kids
                .iter()
                .map(|k| (k.id, k.name.clone()))
                .collect();

            let result: Vec<Value> = tasks
                .iter()
                .map(|t| {
                    // Resolve completed_by UUIDs to kid names
                    let completed_by_names: Vec<String> = t.completed_by
                        .iter()
                        .filter_map(|id| kid_names.get(id).cloned())
                        .collect();

                    json!({
                        "id": t.id.to_string(),
                        "name": t.name,
                        "description": t.description,
                        "value": t.value.to_string(),
                        "cadence": match t.cadence {
                            Cadence::Daily => "daily",
                            Cadence::Weekly => "weekly",
                            Cadence::OneTime => "one-time",
                        },
                        "collaborative": t.collaborative,
                        "completed_by": completed_by_names,
                        "needs_reset": t.needs_reset()
                    })
                })
                .collect();
            Ok(serde_json::to_string_pretty(&result)?)
        }
        "complete_task" => {
            let kid_id = input["kid_id"]
                .as_str()
                .ok_or_else(|| anyhow!("Missing kid_id"))?;
            let task_id = input["task_id"]
                .as_str()
                .ok_or_else(|| anyhow!("Missing task_id"))?;

            // Parse optional completed_at for backdating
            let completed_at = input["completed_at"]
                .as_str()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));

            let kid_uuid = uuid::Uuid::parse_str(kid_id)?;
            let task_uuid = uuid::Uuid::parse_str(task_id)?;

            let workflow = TaskCompletionWorkflow::new(
                TaskRepository::new(state.db.client.clone()),
                KidRepository::new(state.db.client.clone()),
                LedgerRepository::new(state.db.client.clone()),
            );

            // Kids create pending entries, parents create confirmed entries
            let status = if is_kid {
                TransactionStatus::Pending
            } else {
                TransactionStatus::Confirmed
            };
            let entry = workflow.complete_task_with_status(task_uuid, kid_uuid, status, None, completed_at).await?;

            // Broadcast SSE event for live dashboard updates
            if let Some(ref tx) = state.event_sender {
                broadcast_event(tx, DataEvent::TaskCompleted {
                    kid_id: kid_uuid.to_string(),
                    task_id: task_uuid.to_string(),
                    amount: entry.amount.to_string(),
                });
            }

            // Get updated balance
            let ledger_repo = LedgerRepository::new(state.db.client.clone());
            let ledger = ledger_repo.get_ledger(kid_uuid).await?;

            Ok(json!({
                "success": true,
                "amount_earned": entry.amount.to_string(),
                "description": entry.description,
                "new_balance": ledger.balance.to_string()
            }).to_string())
        }
        "get_balance" => {
            let kid_id = input["kid_id"]
                .as_str()
                .ok_or_else(|| anyhow!("Missing kid_id"))?;
            let kid_uuid = uuid::Uuid::parse_str(kid_id)?;

            let ledger_repo = LedgerRepository::new(state.db.client.clone());
            let ledger = ledger_repo.get_ledger(kid_uuid).await?;

            Ok(json!({
                "balance": ledger.balance.to_string()
            }).to_string())
        }
        _ => Err(anyhow!("Unknown tool: {}", name)),
    }
}

/// Send a chat message and handle the full conversation loop with tool calls
/// kid_info: Optional (name, id) tuple for the logged-in kid
pub async fn chat(
    messages: Vec<ChatMessage>,
    state: &AppState,
    account_id: uuid::Uuid,
    is_kid: bool,
    kid_info: Option<(String, String)>,
) -> Result<String> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| anyhow!("ANTHROPIC_API_KEY not set"))?;

    let client = reqwest::Client::new();
    let mut conversation = messages;

    // Build system prompt with kid info if available
    let system_prompt = match &kid_info {
        Some((name, id)) => get_system_prompt(Some((name, id))),
        None => get_system_prompt(None),
    };

    loop {
        // Build the request
        let request_body = json!({
            "model": MODEL,
            "max_tokens": 1024,
            "system": system_prompt,
            "tools": get_tools(),
            "messages": conversation
        });

        // Call Claude API
        let response = client
            .post(CLAUDE_API_URL)
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Claude API error: {}", error_text));
        }

        let claude_response: ClaudeResponse = response.json().await?;

        // Check if we need to handle tool calls
        let has_tool_use = claude_response
            .content
            .iter()
            .any(|block| matches!(block, ContentBlock::ToolUse { .. }));

        if has_tool_use {
            // Add assistant's response to conversation
            conversation.push(ChatMessage {
                role: "assistant".to_string(),
                content: MessageContent::Blocks(claude_response.content.clone()),
            });

            // Execute each tool call and collect results
            let mut tool_results = Vec::new();
            for block in &claude_response.content {
                if let ContentBlock::ToolUse { id, name, input } = block {
                    let result = match execute_tool(name, input, state, account_id, is_kid).await {
                        Ok(r) => r,
                        Err(e) => json!({"error": e.to_string()}).to_string(),
                    };
                    tool_results.push(ContentBlock::ToolResult {
                        tool_use_id: id.clone(),
                        content: result,
                    });
                }
            }

            // Add tool results to conversation
            conversation.push(ChatMessage {
                role: "user".to_string(),
                content: MessageContent::Blocks(tool_results),
            });

            // Continue the loop to get Claude's final response
            continue;
        }

        // No tool use - extract text response and return
        let text_response = claude_response
            .content
            .iter()
            .filter_map(|block| {
                if let ContentBlock::Text { text } = block {
                    Some(text.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        return Ok(text_response);
    }
}

// === Axum Handler ===

use axum::{
    extract::State,
    Json,
    http::StatusCode,
};

/// Request body for the chat endpoint
#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    /// Previous conversation history (optional)
    #[serde(default)]
    pub history: Vec<SimpleMessage>,
}

/// Simple message format for the API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleMessage {
    pub role: String,
    pub content: String,
}

/// Response from the chat endpoint
#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub response: String,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ChatError {
    pub error: String,
}

/// Chat endpoint handler
///
/// POST /api/chat
/// Body: { "message": "I did the dishes", "history": [...] }
pub async fn chat_handler(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, Json<ChatError>)> {
    // Get account_id from the first user (fallback for direct API access)
    let account_id = get_default_account_id(&state).await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ChatError { error: format!("Failed to get account: {}", e) })
        ))?;

    // Build conversation from history + new message
    let mut messages: Vec<ChatMessage> = request.history
        .into_iter()
        .map(|m| ChatMessage {
            role: m.role,
            content: MessageContent::Text(m.content),
        })
        .collect();

    // Add the new user message
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: MessageContent::Text(request.message),
    });

    // Call Claude with account_id (direct API access defaults to parent role, no kid info)
    let response = chat(messages, &state, account_id, false, None).await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ChatError { error: e.to_string() })
        ))?;

    Ok(Json(ChatResponse { response }))
}

/// Get default account ID (from first user in database)
/// This is a fallback for direct API access without session
async fn get_default_account_id(state: &AppState) -> Result<uuid::Uuid> {
    use loaa_core::db::UserRepository;

    let user_repo = UserRepository::new(state.db.client.clone());
    let users = user_repo.list().await?;

    users.first()
        .map(|u| u.account_id)
        .ok_or_else(|| anyhow!("No users found in database"))
}
