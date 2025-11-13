use axum::{
    extract::State,
    response::Html,
    routing::get,
    Router,
};
use loaa_core::{Database, KidRepository, TaskRepository};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Loa'a web server...");

    // Initialize database - connect to SurrealDB server
    let db_url = std::env::var("LOAA_DB_URL").unwrap_or_else(|_| "127.0.0.1:8000".to_string());
    let db = Database::init(&db_url).await?;
    let state = AppState {
        db: Arc::new(db),
    };

    // Build router
    let app = Router::new()
        .route("/", get(index))
        .route("/kids", get(list_kids))
        .route("/tasks", get(list_tasks))
        .with_state(state);

    // Start server
    let addr = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("✨ Server running at http://{}", addr);
    println!("   • View homepage: http://127.0.0.1:3000");
    println!("   • View kids: http://127.0.0.1:3000/kids");
    println!("   • View tasks: http://127.0.0.1:3000/tasks");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn index() -> Html<String> {
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Loa'a - Chore Tracker</title>
    <style>
        body {
            font-family: system-ui, -apple-system, sans-serif;
            max-width: 800px;
            margin: 40px auto;
            padding: 20px;
            background: #f5f5f5;
        }
        h1 { color: #2563eb; }
        .card {
            background: white;
            padding: 20px;
            margin: 20px 0;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        a {
            color: #2563eb;
            text-decoration: none;
            font-weight: 500;
        }
        a:hover { text-decoration: underline; }
        .emoji { font-size: 24px; margin-right: 10px; }
    </style>
</head>
<body>
    <h1>🎯 Loa'a</h1>
    <p>Chore and rewards tracking system</p>

    <div class="card">
        <h2><span class="emoji">👦</span>Kids</h2>
        <p>View all kids in the system</p>
        <a href="/kids">→ View Kids</a>
    </div>

    <div class="card">
        <h2><span class="emoji">📋</span>Tasks</h2>
        <p>View all available tasks</p>
        <a href="/tasks">→ View Tasks</a>
    </div>

    <div class="card">
        <p><strong>✅ Database Status:</strong> Connected and working!</p>
        <p><strong>🎉 SurrealDB Integration:</strong> Fully functional</p>
    </div>
</body>
</html>
    "#.to_string())
}

async fn list_kids(State(state): State<AppState>) -> Html<String> {
    let kid_repo = KidRepository::new(state.db.client.clone());

    let kids = match kid_repo.list().await {
        Ok(kids) => kids,
        Err(e) => {
            return Html(format!("<html><body><h1>Error</h1><p>{}</p></body></html>", e));
        }
    };

    let kids_html = if kids.is_empty() {
        "<p>No kids yet. Create one using the API!</p>".to_string()
    } else {
        kids.iter()
            .map(|kid| format!(
                "<div class='kid'><h3>{}</h3><p>Created: {}</p></div>",
                kid.name,
                kid.created_at.format("%Y-%m-%d %H:%M")
            ))
            .collect::<Vec<_>>()
            .join("\n")
    };

    Html(format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Kids - Loa'a</title>
    <style>
        body {{
            font-family: system-ui, -apple-system, sans-serif;
            max-width: 800px;
            margin: 40px auto;
            padding: 20px;
            background: #f5f5f5;
        }}
        h1 {{ color: #2563eb; }}
        .kid {{
            background: white;
            padding: 15px;
            margin: 10px 0;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        a {{
            color: #2563eb;
            text-decoration: none;
        }}
        a:hover {{ text-decoration: underline; }}
    </style>
</head>
<body>
    <p><a href="/">← Back to Home</a></p>
    <h1>👦 Kids ({count})</h1>
    {kids_html}
</body>
</html>
    "#, count = kids.len(), kids_html = kids_html))
}

async fn list_tasks(State(state): State<AppState>) -> Html<String> {
    let task_repo = TaskRepository::new(state.db.client.clone());

    let tasks = match task_repo.list().await {
        Ok(tasks) => tasks,
        Err(e) => {
            return Html(format!("<html><body><h1>Error</h1><p>{}</p></body></html>", e));
        }
    };

    let tasks_html = if tasks.is_empty() {
        "<p>No tasks yet. Create one using the API!</p>".to_string()
    } else {
        tasks.iter()
            .map(|task| format!(
                "<div class='task'><h3>{}</h3><p>{}</p><p><strong>${}</strong> - {:?}</p></div>",
                task.name,
                task.description,
                task.value,
                task.cadence
            ))
            .collect::<Vec<_>>()
            .join("\n")
    };

    Html(format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Tasks - Loa'a</title>
    <style>
        body {{
            font-family: system-ui, -apple-system, sans-serif;
            max-width: 800px;
            margin: 40px auto;
            padding: 20px;
            background: #f5f5f5;
        }}
        h1 {{ color: #2563eb; }}
        .task {{
            background: white;
            padding: 15px;
            margin: 10px 0;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        a {{
            color: #2563eb;
            text-decoration: none;
        }}
        a:hover {{ text-decoration: underline; }}
    </style>
</head>
<body>
    <p><a href="/">← Back to Home</a></p>
    <h1>📋 Tasks ({count})</h1>
    {tasks_html}
</body>
</html>
    "#, count = tasks.len(), tasks_html = tasks_html))
}
