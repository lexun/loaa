use anyhow::Result;
use loaa_core::db::{init_database, KidRepository, LedgerRepository, TaskRepository};
use loaa_core::models::{Cadence, EntryType, Kid, LedgerEntry, Task};
use loaa_core::workflows::TaskCompletionWorkflow;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{tool, tool_router, ErrorData as McpError, ServiceExt};
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Context shared across all MCP tool calls
#[derive(Clone)]
struct LoaaServer {
    task_repo: Arc<RwLock<TaskRepository>>,
    kid_repo: Arc<RwLock<KidRepository>>,
    ledger_repo: Arc<RwLock<LedgerRepository>>,
    workflow: Arc<RwLock<TaskCompletionWorkflow>>,
    tool_router: ToolRouter<Self>,
}

// Tool parameter structures
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct CreateKidParams {
    #[schemars(description = "Name of the kid")]
    name: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct CreateTaskParams {
    #[schemars(description = "Name of the task")]
    name: String,
    #[schemars(description = "Description of the task")]
    description: String,
    #[schemars(description = "Value as decimal string (e.g., '1.50')")]
    value: String,
    #[schemars(description = "Cadence: 'daily', 'weekly', or 'onetime'")]
    cadence: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct UpdateTaskParams {
    #[schemars(description = "ID of the task to update")]
    id: String,
    #[schemars(description = "New name for the task (optional)")]
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[schemars(description = "New description for the task (optional)")]
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[schemars(description = "New value as decimal string (optional)")]
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
    #[schemars(description = "New cadence (optional)")]
    #[serde(skip_serializing_if = "Option::is_none")]
    cadence: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct DeleteTaskParams {
    #[schemars(description = "ID of the task to delete")]
    id: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct CompleteTaskParams {
    #[schemars(description = "ID of the task to complete")]
    task_id: String,
    #[schemars(description = "ID of the kid completing the task")]
    kid_id: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct GetLedgerParams {
    #[schemars(description = "ID of the kid whose ledger to retrieve")]
    kid_id: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct AdjustBalanceParams {
    #[schemars(description = "ID of the kid whose balance to adjust")]
    kid_id: String,
    #[schemars(description = "Amount as decimal string (e.g., '5.00' or '-2.50')")]
    amount: String,
    #[schemars(description = "Description of the adjustment")]
    description: String,
}

#[tool_router]
impl LoaaServer {
    async fn new(db_path: &str) -> Result<Self> {
        let database = init_database(db_path).await?;
        let task_repo = TaskRepository::new(database.client.clone());
        let kid_repo = KidRepository::new(database.client.clone());
        let ledger_repo = LedgerRepository::new(database.client.clone());

        let workflow = TaskCompletionWorkflow::new(
            TaskRepository::new(database.client.clone()),
            KidRepository::new(database.client.clone()),
            LedgerRepository::new(database.client.clone()),
        );

        Ok(Self {
            task_repo: Arc::new(RwLock::new(task_repo)),
            kid_repo: Arc::new(RwLock::new(kid_repo)),
            ledger_repo: Arc::new(RwLock::new(ledger_repo)),
            workflow: Arc::new(RwLock::new(workflow)),
            tool_router: Self::tool_router(),
        })
    }

    #[tool(description = "Create a new kid in the system. Returns the created kid with their ID.")]
    async fn create_kid(
        &self,
        Parameters(params): Parameters<CreateKidParams>,
    ) -> Result<CallToolResult, McpError> {
        let kid = Kid::new(params.name).map_err(|e| {
            McpError::invalid_request(e.to_string(), None)
        })?;

        let kid_repo = self.kid_repo.read().await;
        let created = kid_repo.create(kid).await.map_err(|e| {
            McpError::internal_error("database_error", Some(json!({"error": e.to_string()})))
        })?;

        let response = json!({
            "id": created.id.to_string(),
            "name": created.name,
            "created_at": created.created_at.to_rfc3339()
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap(),
        )]))
    }

    #[tool(description = "List all kids in the system.")]
    async fn list_kids(&self) -> Result<CallToolResult, McpError> {
        let kid_repo = self.kid_repo.read().await;
        let kids = kid_repo.list().await.map_err(|e| {
            McpError::internal_error("database_error", Some(json!({"error": e.to_string()})))
        })?;

        let response = json!({
            "kids": kids.iter().map(|k| json!({
                "id": k.id.to_string(),
                "name": k.name,
                "created_at": k.created_at.to_rfc3339()
            })).collect::<Vec<_>>()
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap(),
        )]))
    }

    #[tool(description = "Create a new task. Value should be a decimal string (e.g., '1.50'). Cadence must be one of: 'daily', 'weekly', 'onetime'.")]
    async fn create_task(
        &self,
        Parameters(params): Parameters<CreateTaskParams>,
    ) -> Result<CallToolResult, McpError> {
        let value_dec =
            Decimal::from_str(&params.value).map_err(|e| {
                McpError::invalid_request(format!("Invalid value format: {}", e), None)
            })?;

        let cadence_enum = match params.cadence.to_lowercase().as_str() {
            "daily" => Cadence::Daily,
            "weekly" => Cadence::Weekly,
            "onetime" | "one-time" | "one_time" => Cadence::OneTime,
            _ => {
                return Err(McpError::invalid_request(
                    "Invalid cadence. Must be 'daily', 'weekly', or 'onetime'",
                    None,
                ))
            }
        };

        let task = Task::new(params.name, params.description, value_dec, cadence_enum)
            .map_err(|e| {
                McpError::invalid_request(e.to_string(), None)
            })?;

        let task_repo = self.task_repo.read().await;
        let created = task_repo.create(task).await.map_err(|e| {
            McpError::internal_error("database_error", Some(json!({"error": e.to_string()})))
        })?;

        let response = json!({
            "id": created.id.to_string(),
            "name": created.name,
            "description": created.description,
            "value": created.value.to_string(),
            "cadence": match created.cadence {
                Cadence::Daily => "daily",
                Cadence::Weekly => "weekly",
                Cadence::OneTime => "onetime"
            },
            "created_at": created.created_at.to_rfc3339(),
            "needs_reset": created.needs_reset()
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap(),
        )]))
    }

    #[tool(description = "List all tasks in the system.")]
    async fn list_tasks(&self) -> Result<CallToolResult, McpError> {
        let task_repo = self.task_repo.read().await;
        let tasks = task_repo.list().await.map_err(|e| {
            McpError::internal_error("database_error", Some(json!({"error": e.to_string()})))
        })?;

        let response = json!({
            "tasks": tasks.iter().map(|t| json!({
                "id": t.id.to_string(),
                "name": t.name,
                "description": t.description,
                "value": t.value.to_string(),
                "cadence": match t.cadence {
                    Cadence::Daily => "daily",
                    Cadence::Weekly => "weekly",
                    Cadence::OneTime => "onetime"
                },
                "created_at": t.created_at.to_rfc3339(),
                "last_reset": t.last_reset.to_rfc3339(),
                "needs_reset": t.needs_reset()
            })).collect::<Vec<_>>()
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap(),
        )]))
    }

    #[tool(description = "Update an existing task. All fields except id are optional. Value should be a decimal string (e.g., '1.50'). Cadence must be one of: 'daily', 'weekly', 'onetime'.")]
    async fn update_task(
        &self,
        Parameters(params): Parameters<UpdateTaskParams>,
    ) -> Result<CallToolResult, McpError> {
        let task_id = Uuid::parse_str(&params.id).map_err(|e| {
            McpError::invalid_request(format!("Invalid task ID: {}", e), None)
        })?;

        let task_repo = self.task_repo.read().await;
        let mut task = task_repo.get(task_id).await.map_err(|e| {
            McpError::resource_not_found(format!("Task not found: {}", e), None)
        })?;

        if let Some(n) = params.name {
            task.name = n;
        }
        if let Some(d) = params.description {
            task.description = d;
        }
        if let Some(v) = params.value {
            task.value = Decimal::from_str(&v).map_err(|e| {
                McpError::invalid_request(format!("Invalid value format: {}", e), None)
            })?;
        }
        if let Some(c) = params.cadence {
            task.cadence = match c.to_lowercase().as_str() {
                "daily" => Cadence::Daily,
                "weekly" => Cadence::Weekly,
                "onetime" | "one-time" | "one_time" => Cadence::OneTime,
                _ => {
                    return Err(McpError::invalid_request(
                        "Invalid cadence. Must be 'daily', 'weekly', or 'onetime'",
                        None,
                    ))
                }
            };
        }

        let updated = task_repo.update(task).await.map_err(|e| {
            McpError::internal_error("database_error", Some(json!({"error": e.to_string()})))
        })?;

        let response = json!({
            "id": updated.id.to_string(),
            "name": updated.name,
            "description": updated.description,
            "value": updated.value.to_string(),
            "cadence": match updated.cadence {
                Cadence::Daily => "daily",
                Cadence::Weekly => "weekly",
                Cadence::OneTime => "onetime"
            },
            "needs_reset": updated.needs_reset()
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap(),
        )]))
    }

    #[tool(description = "Delete a task by ID.")]
    async fn delete_task(
        &self,
        Parameters(params): Parameters<DeleteTaskParams>,
    ) -> Result<CallToolResult, McpError> {
        let task_id = Uuid::parse_str(&params.id).map_err(|e| {
            McpError::invalid_request(format!("Invalid task ID: {}", e), None)
        })?;

        let task_repo = self.task_repo.read().await;
        task_repo.delete(task_id).await.map_err(|e| {
            McpError::internal_error("database_error", Some(json!({"error": e.to_string()})))
        })?;

        let response = json!({
            "success": true,
            "message": format!("Task {} deleted successfully", task_id)
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap(),
        )]))
    }

    #[tool(description = "Mark a task as complete for a specific kid. This creates a ledger entry and resets the task if it's a recurring task (daily/weekly).")]
    async fn complete_task(
        &self,
        Parameters(params): Parameters<CompleteTaskParams>,
    ) -> Result<CallToolResult, McpError> {
        let task_uuid = Uuid::parse_str(&params.task_id).map_err(|e| {
            McpError::invalid_request(format!("Invalid task ID: {}", e), None)
        })?;
        let kid_uuid = Uuid::parse_str(&params.kid_id).map_err(|e| {
            McpError::invalid_request(format!("Invalid kid ID: {}", e), None)
        })?;

        let workflow = self.workflow.read().await;
        let entry = workflow
            .complete_task(task_uuid, kid_uuid)
            .await
            .map_err(|e| {
                McpError::internal_error("workflow_error", Some(json!({"error": e.to_string()})))
            })?;

        let response = json!({
            "success": true,
            "ledger_entry": {
                "id": entry.id.to_string(),
                "kid_id": entry.kid_id.to_string(),
                "amount": entry.amount.to_string(),
                "entry_type": match entry.entry_type {
                    EntryType::Earned => "earned",
                    EntryType::Adjusted => "adjusted"
                },
                "description": entry.description,
                "created_at": entry.created_at.to_rfc3339()
            }
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap(),
        )]))
    }

    #[tool(description = "Get the ledger (transaction history and balance) for a specific kid.")]
    async fn get_ledger(
        &self,
        Parameters(params): Parameters<GetLedgerParams>,
    ) -> Result<CallToolResult, McpError> {
        let kid_uuid = Uuid::parse_str(&params.kid_id).map_err(|e| {
            McpError::invalid_request(format!("Invalid kid ID: {}", e), None)
        })?;

        let ledger_repo = self.ledger_repo.read().await;
        let ledger = ledger_repo.get_ledger(kid_uuid).await.map_err(|e| {
            McpError::internal_error("database_error", Some(json!({"error": e.to_string()})))
        })?;

        let response = json!({
            "kid_id": ledger.kid_id.to_string(),
            "balance": ledger.balance.to_string(),
            "entries": ledger.entries.iter().map(|e| json!({
                "id": e.id.to_string(),
                "amount": e.amount.to_string(),
                "entry_type": match e.entry_type {
                    EntryType::Earned => "earned",
                    EntryType::Adjusted => "adjusted"
                },
                "description": e.description,
                "created_at": e.created_at.to_rfc3339()
            })).collect::<Vec<_>>()
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap(),
        )]))
    }

    #[tool(description = "Manually adjust a kid's balance. Use positive amounts to add money, negative to deduct. Amount should be a decimal string (e.g., '5.00' or '-2.50').")]
    async fn adjust_balance(
        &self,
        Parameters(params): Parameters<AdjustBalanceParams>,
    ) -> Result<CallToolResult, McpError> {
        let kid_uuid = Uuid::parse_str(&params.kid_id).map_err(|e| {
            McpError::invalid_request(format!("Invalid kid ID: {}", e), None)
        })?;
        let amount_dec = Decimal::from_str(&params.amount).map_err(|e| {
            McpError::invalid_request(format!("Invalid amount format: {}", e), None)
        })?;

        let entry = LedgerEntry::adjusted(kid_uuid, amount_dec, params.description);
        let ledger_repo = self.ledger_repo.read().await;
        let created = ledger_repo.create_entry(entry).await.map_err(|e| {
            McpError::internal_error("database_error", Some(json!({"error": e.to_string()})))
        })?;

        let response = json!({
            "success": true,
            "ledger_entry": {
                "id": created.id.to_string(),
                "kid_id": created.kid_id.to_string(),
                "amount": created.amount.to_string(),
                "entry_type": "adjusted",
                "description": created.description,
                "created_at": created.created_at.to_rfc3339()
            }
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap(),
        )]))
    }
}

impl rmcp::handler::server::ServerHandler for LoaaServer {
    fn get_info(&self) -> rmcp::model::InitializeResult {
        InitializeResult {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities {
                prompts: None,
                resources: None,
                tools: Some(ToolsCapability { list_changed: None }),
                experimental: None,
                logging: None,
                completions: None,
            },
            server_info: Implementation {
                name: "loaa".to_string(),
                version: "0.1.0".to_string(),
                icons: None,
                title: Some("Loa'a".to_string()),
                website_url: None,
            },
            instructions: Some("Loa'a chore tracking system".to_string()),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Get database path from environment or use default
    let db_path = std::env::var("LOAA_DB_PATH").unwrap_or_else(|_| {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../../data/loaa.db");
        path.to_string_lossy().to_string()
    });

    eprintln!("Initializing Loa'a MCP Server...");
    eprintln!("Database path: {}", db_path);

    let server = LoaaServer::new(&db_path).await?;

    eprintln!("Loa'a MCP Server started successfully!");
    eprintln!("Available tools:");
    eprintln!("  - create_kid: Create a new kid");
    eprintln!("  - list_kids: List all kids");
    eprintln!("  - create_task: Create a new task");
    eprintln!("  - list_tasks: List all tasks");
    eprintln!("  - update_task: Update an existing task");
    eprintln!("  - delete_task: Delete a task");
    eprintln!("  - complete_task: Mark a task as complete");
    eprintln!("  - get_ledger: Get ledger for a kid");
    eprintln!("  - adjust_balance: Manually adjust a kid's balance");

    // Use stdio transport
    use tokio::io::{stdin, stdout};
    server.serve((stdin(), stdout())).await?;

    Ok(())
}
