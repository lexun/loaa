use leptos::*;
use crate::server_functions::*;
use crate::dto::*;

#[cfg(feature = "hydrate")]
use wasm_bindgen::prelude::*;
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;

/// Convert simple markdown to HTML for chat messages
/// Handles: **bold**, *italic*, `code`, ```code blocks```, and newlines
fn markdown_to_html(text: &str) -> String {
    let mut result = text.to_string();

    // Escape HTML entities first
    result = result
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");

    // Bold: **text** -> <strong>text</strong>
    while let Some(start) = result.find("**") {
        if let Some(end) = result[start + 2..].find("**") {
            let content = &result[start + 2..start + 2 + end];
            // Only convert if there's actual content
            if !content.is_empty() {
                let before = &result[..start];
                let after = &result[start + 2 + end + 2..];
                result = format!("{}<strong>{}</strong>{}", before, content, after);
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // Code blocks: ```code``` -> <pre><code>code</code></pre>
    // Process BEFORE inline code
    while let Some(start) = result.find("```") {
        if let Some(end) = result[start + 3..].find("```") {
            let content = &result[start + 3..start + 3 + end];
            let before = &result[..start];
            let after = &result[start + 3 + end + 3..];
            // Trim leading/trailing newlines from code block content
            let trimmed = content.trim_matches(|c| c == '\n' || c == '\r');
            result = format!("{}<pre><code>{}</code></pre>{}", before, trimmed, after);
        } else {
            break;
        }
    }

    // Inline code: `code` -> <code>code</code>
    // Process BEFORE italics to avoid conflicts
    while let Some(start) = result.find('`') {
        if let Some(end) = result[start + 1..].find('`') {
            let content = &result[start + 1..start + 1 + end];
            // Only convert if there's actual content
            if !content.is_empty() {
                let before = &result[..start];
                let after = &result[start + 1 + end + 1..];
                result = format!("{}<code>{}</code>{}", before, content, after);
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // Italic: *text* -> <em>text</em>
    // Must come after bold processing since ** contains *
    while let Some(start) = result.find('*') {
        if let Some(end) = result[start + 1..].find('*') {
            let content = &result[start + 1..start + 1 + end];
            // Only convert if there's actual content and not just spaces
            if !content.is_empty() && content.trim().len() == content.len() {
                let before = &result[..start];
                let after = &result[start + 1 + end + 1..];
                result = format!("{}<em>{}</em>{}", before, content, after);
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // Newlines to <br>
    result = result.replace('\n', "<br>");

    result
}

#[derive(Debug, Clone)]
pub enum View {
    Login,
    Admin,
    Dashboard,
    Ledger(UuidDto),
}

#[component]
pub fn Login(set_view: WriteSignal<View>) -> impl IntoView {
    let (username, set_username) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (error, set_error) = create_signal(Option::<String>::None);
    let (logging_in, set_logging_in) = create_signal(false);
    let (oauth_completing, set_oauth_completing) = create_signal(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        set_logging_in.set(true);

        let username_val = username.get();
        let password_val = password.get();

        spawn_local(async move {
            match login(username_val, password_val).await {
                Ok(true) => {
                    // Check if there's a pending OAuth flow
                    match check_pending_oauth().await {
                        Ok(Some(oauth_url)) => {
                            // Redirect to OAuth authorization endpoint
                            leptos::logging::log!("Redirecting to OAuth: {}", oauth_url);
                            // Update UI to show OAuth is completing (not stuck on "logging in")
                            set_logging_in.set(false);
                            set_oauth_completing.set(true);
                            let window = leptos::window();
                            if let Ok(location) = window.location().href() {
                                leptos::logging::log!("Current location: {}", location);
                            }
                            let _ = window.location().set_href(&oauth_url);
                        }
                        Ok(None) => {
                            leptos::logging::log!("No pending OAuth, checking account type");
                            // No pending OAuth, check account type to determine view
                            match get_account_type().await {
                                Ok(AccountTypeDto::Admin) => {
                                    leptos::logging::log!("Admin user, going to admin view");
                                    set_view.set(View::Admin);
                                }
                                _ => {
                                    leptos::logging::log!("Regular user, going to dashboard");
                                    set_view.set(View::Dashboard);
                                }
                            }
                        }
                        Err(e) => {
                            leptos::logging::log!("Error checking OAuth: {}", e);
                            // Error checking OAuth, check account type
                            match get_account_type().await {
                                Ok(AccountTypeDto::Admin) => {
                                    set_view.set(View::Admin);
                                }
                                _ => {
                                    set_view.set(View::Dashboard);
                                }
                            }
                        }
                    }
                }
                Ok(false) => {
                    set_error.set(Some("Invalid username or password".to_string()));
                    set_logging_in.set(false);
                }
                Err(e) => {
                    set_error.set(Some(format!("Login error: {}", e)));
                    set_logging_in.set(false);
                }
            }
        });
    };

    view! {
        <div class="login-container">
            <div class="login-box">
                <h1>"Loa'a"</h1>
                <p class="subtitle">"Chore and rewards tracking system"</p>

                {move || if oauth_completing.get() {
                    view! {
                        <div class="oauth-completing">
                            <p class="oauth-message">"Completing authorization..."</p>
                            <p class="oauth-hint">"You can close this window and return to Claude."</p>
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <form on:submit=on_submit>
                            <div class="form-group">
                                <label for="username">"Username"</label>
                                <input
                                    type="text"
                                    id="username"
                                    name="username"
                                    required
                                    disabled=move || logging_in.get()
                                    on:input=move |ev| set_username.set(event_target_value(&ev))
                                    prop:value=move || username.get()
                                />
                            </div>

                            <div class="form-group">
                                <label for="password">"Password"</label>
                                <input
                                    type="password"
                                    id="password"
                                    name="password"
                                    required
                                    disabled=move || logging_in.get()
                                    on:input=move |ev| set_password.set(event_target_value(&ev))
                                    prop:value=move || password.get()
                                />
                            </div>

                            {move || error.get().map(|err| view! {
                                <p class="error">{err}</p>
                            })}

                            <button
                                type="submit"
                                class="login-btn"
                                disabled=move || logging_in.get()
                            >
                                {move || if logging_in.get() { "Logging in..." } else { "Log In" }}
                            </button>
                        </form>
                    }.into_view()
                }}
            </div>
        </div>
    }
}

#[component]
pub fn Dashboard() -> impl IntoView {
    let (current_view, set_current_view) = create_signal(View::Login);

    // Check if user is authenticated on mount
    create_effect(move |_| {
        spawn_local(async move {
            match check_auth().await {
                Ok(true) => {
                    // User is authenticated, check account type
                    match get_account_type().await {
                        Ok(AccountTypeDto::Admin) => {
                            set_current_view.set(View::Admin);
                        }
                        _ => {
                            set_current_view.set(View::Dashboard);
                        }
                    }
                }
                Ok(false) => {
                    // Not authenticated, show login
                    set_current_view.set(View::Login);
                }
                Err(_) => {
                    // Error checking auth, default to login
                    set_current_view.set(View::Login);
                }
            }
        });
    });

    let handle_logout = move |_| {
        spawn_local(async move {
            let _ = logout().await;
            set_current_view.set(View::Login);
        });
    };

    view! {
        <div class="app-wrapper">
            {move || match current_view.get() {
                View::Login => view! {
                    <Login set_view=set_current_view />
                }.into_view(),
                View::Admin => view! {
                    <div>
                        <nav class="navbar">
                            <div class="navbar-brand">"Loa'a Admin"</div>
                            <button class="logout-btn" on:click=handle_logout>
                                "Log Out"
                            </button>
                        </nav>
                        <div class="container">
                            <main>
                                <AdminPanel />
                            </main>
                        </div>
                    </div>
                }.into_view(),
                View::Dashboard => view! {
                    <div>
                        <nav class="navbar">
                            <div class="navbar-brand">"Loa'a"</div>
                            <button class="logout-btn" on:click=handle_logout>
                                "Log Out"
                            </button>
                        </nav>
                        <div class="container">
                            <main>
                                <DashboardView set_view=set_current_view />
                            </main>
                        </div>
                        // Chat widget at root level for proper fixed positioning
                        <ChatWidget />
                    </div>
                }.into_view(),
                View::Ledger(kid_id) => view! {
                    <div>
                        <nav class="navbar">
                            <div class="navbar-brand">"Loa'a"</div>
                            <button class="logout-btn" on:click=handle_logout>
                                "Log Out"
                            </button>
                        </nav>
                        <div class="container">
                            <main>
                                <LedgerView kid_id=kid_id set_view=set_current_view />
                            </main>
                        </div>
                    </div>
                }.into_view(),
            }}
        </div>
    }
}

#[component]
fn DashboardView(set_view: WriteSignal<View>) -> impl IntoView {
    // Initial data load
    let dashboard_data = create_resource(|| (), |_| get_dashboard_data());

    // Fine-grained signals for each piece of data to avoid full DOM replacement
    let (kid_summaries, set_kid_summaries) = create_signal(Vec::<KidSummaryDto>::new());
    let (tasks, set_tasks) = create_signal(Vec::<TaskDto>::new());
    let (is_loaded, set_is_loaded) = create_signal(false);
    let (recent_activity, set_recent_activity) = create_signal(Vec::<LedgerEntryDto>::new());
    let (pending_transactions, set_pending_transactions) = create_signal(Vec::<LedgerEntryDto>::new());

    // Kid login management (parent-only)
    let (is_parent_user, set_is_parent_user) = create_signal(true); // Default to true for loading
    let (kid_logins, set_kid_logins) = create_signal(Vec::<KidLoginDto>::new());
    let (new_kid_username, set_new_kid_username) = create_signal(String::new());
    let (new_kid_password, set_new_kid_password) = create_signal(String::new());
    let (selected_kid_id, set_selected_kid_id) = create_signal(Option::<String>::None);
    let (creating_kid_login, set_creating_kid_login) = create_signal(false);
    let (kid_login_error, set_kid_login_error) = create_signal(Option::<String>::None);
    let (editing_kid_user_id, set_editing_kid_user_id) = create_signal(Option::<String>::None);
    let (edit_kid_password, set_edit_kid_password) = create_signal(String::new());
    let (updating_kid_password, set_updating_kid_password) = create_signal(false);

    // Update signals when resource loads
    create_effect(move |_| {
        if let Some(Ok(data)) = dashboard_data.get() {
            set_kid_summaries.set(data.kid_summaries);
            set_is_loaded.set(true);
            // Also fetch tasks, recent activity, pending transactions, and parent status
            spawn_local(async move {
                if let Ok(task_list) = get_tasks().await {
                    set_tasks.set(task_list);
                }
                if let Ok(entries) = get_recent_activity(10).await {
                    set_recent_activity.set(entries);
                }
                if let Ok(pending) = list_pending_transactions().await {
                    set_pending_transactions.set(pending);
                }
                // Check if user is a parent (controls visibility of parent-only sections)
                if let Ok(is_parent) = is_parent().await {
                    set_is_parent_user.set(is_parent);
                    // Only fetch kid logins for parents
                    if is_parent {
                        if let Ok(logins) = list_kid_logins().await {
                            set_kid_logins.set(logins);
                        }
                    }
                }
            });
        }
    });

    // Set up SSE connection for real-time updates (client-side only)
    #[cfg(feature = "hydrate")]
    {
        // Use create_effect with a guard to only set up SSE once
        let (sse_initialized, set_sse_initialized) = create_signal(false);

        create_effect(move |_| {
            // Only initialize SSE once
            if sse_initialized.get() {
                return;
            }
            set_sse_initialized.set(true);

            use web_sys::{EventSource, MessageEvent};

            let es = EventSource::new("/api/events").ok();
            if let Some(event_source) = es {
                let onmessage = Closure::<dyn Fn(MessageEvent)>::new(move |_event: MessageEvent| {
                    // Fetch new data in background and update fine-grained signals
                    spawn_local(async move {
                        if let Ok(data) = get_dashboard_data().await {
                            set_kid_summaries.set(data.kid_summaries);
                        }
                        if let Ok(task_list) = get_tasks().await {
                            set_tasks.set(task_list);
                        }
                        if let Ok(entries) = get_recent_activity(10).await {
                            set_recent_activity.set(entries);
                        }
                        if let Ok(pending) = list_pending_transactions().await {
                            set_pending_transactions.set(pending);
                        }
                        if let Ok(logins) = list_kid_logins().await {
                            set_kid_logins.set(logins);
                        }
                    });
                });
                event_source.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
                onmessage.forget();

                let onerror = Closure::<dyn Fn()>::new(move || {
                    leptos::logging::log!("SSE connection error - will auto-reconnect");
                });
                event_source.set_onerror(Some(onerror.as_ref().unchecked_ref()));
                onerror.forget();

                leptos::logging::log!("SSE connection established");
            }
        });
    }

    view! {
        <Show
            when=move || is_loaded.get()
            fallback=|| view! { <p>"Loading dashboard..."</p> }
        >
            <div>
                <section class="kids-section">
                    <h2>"Kids"</h2>
                    <div class="kids-grid">
                        {move || kid_summaries.get().into_iter().map(|summary| {
                            view! { <KidSummaryCard summary=summary set_view=set_view /> }
                        }).collect::<Vec<_>>()}
                    </div>
                </section>

                <section class="tasks-section">
                    <h2>"Tasks"</h2>
                    {move || {
                        let task_list = tasks.get();
                        if task_list.is_empty() {
                            view! { <p class="empty-state">"No tasks yet. Create tasks via Claude."</p> }.into_view()
                        } else {
                            view! {
                                <div class="tasks-grid">
                                    {task_list.into_iter().map(|task| {
                                        let cadence_label = match task.cadence {
                                            CadenceDto::Daily => "Daily",
                                            CadenceDto::Weekly => "Weekly",
                                            CadenceDto::OneTime => "One-time",
                                        };
                                        view! {
                                            <div class="task-card">
                                                <div class="task-header">
                                                    <h3>{task.name}</h3>
                                                    <span class="task-value">"$"{task.value.to_string()}</span>
                                                </div>
                                                <p class="task-description">{task.description}</p>
                                                <span class="task-cadence">{cadence_label}</span>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_view()
                        }
                    }}
                </section>

                // Pending transactions section (only shown to parents when there are pending transactions)
                {move || {
                    let pending = pending_transactions.get();
                    if pending.is_empty() || !is_parent_user.get() {
                        view! { <div></div> }.into_view()
                    } else {
                        view! {
                            <section class="pending-section">
                                <h2>"Pending Approvals"</h2>
                                <ul class="pending-list">
                                    <For
                                        each=move || pending_transactions.get()
                                        key=|entry| entry.id.clone()
                                        children=move |entry| {
                                            let entry_id = entry.id.clone();
                                            let entry_id_approve = entry_id.clone();
                                            let entry_id_reject = entry_id.clone();
                                            let time_ago = format_time_ago(entry.created_at);

                                            let handle_approve = move |_| {
                                                let id = entry_id_approve.clone();
                                                spawn_local(async move {
                                                    if approve_transaction(id).await.is_ok() {
                                                        // Refresh pending list
                                                        if let Ok(pending) = list_pending_transactions().await {
                                                            set_pending_transactions.set(pending);
                                                        }
                                                        if let Ok(data) = get_dashboard_data().await {
                                                            set_kid_summaries.set(data.kid_summaries);
                                                        }
                                                        if let Ok(entries) = get_recent_activity(10).await {
                                                            set_recent_activity.set(entries);
                                                        }
                                                    }
                                                });
                                            };

                                            let handle_reject = move |_| {
                                                let id = entry_id_reject.clone();
                                                spawn_local(async move {
                                                    if reject_transaction(id).await.is_ok() {
                                                        // Refresh pending list and tasks (task may now be available again)
                                                        if let Ok(pending) = list_pending_transactions().await {
                                                            set_pending_transactions.set(pending);
                                                        }
                                                        if let Ok(task_list) = get_tasks().await {
                                                            set_tasks.set(task_list);
                                                        }
                                                        if let Ok(data) = get_dashboard_data().await {
                                                            set_kid_summaries.set(data.kid_summaries);
                                                        }
                                                    }
                                                });
                                            };

                                            view! {
                                                <li class="pending-item">
                                                    <span class="pending-time">{time_ago}</span>
                                                    <span class="pending-description">{entry.description.clone()}</span>
                                                    <span class="pending-amount">{"$"}{entry.amount.to_string()}</span>
                                                    <div class="pending-actions">
                                                        <button class="approve-btn" on:click=handle_approve>"Approve"</button>
                                                        <button class="reject-btn" on:click=handle_reject>"Reject"</button>
                                                    </div>
                                                </li>
                                            }
                                        }
                                    />
                                </ul>
                            </section>
                        }.into_view()
                    }
                }}

                <section class="recent-activity">
                    <h2>"Recent Activity"</h2>
                    {move || {
                        let entries = recent_activity.get();
                        if entries.is_empty() {
                            view! { <p>"No activity yet."</p> }.into_view()
                        } else {
                            view! {
                                <ul class="activity-list">
                                    <For
                                        each=move || recent_activity.get()
                                        key=|entry| entry.id.clone()
                                        children=move |entry| {
                                            let entry_type = match entry.entry_type {
                                                EntryTypeDto::Earned => "Earned",
                                                EntryTypeDto::Adjusted => "Adjusted",
                                            };
                                            let is_pending = entry.status == TransactionStatusDto::Pending;
                                            let sign = if entry.amount >= rust_decimal::Decimal::ZERO { "+" } else { "" };
                                            let time_ago = format_time_ago(entry.created_at);
                                            let pending_class = if is_pending { "activity-item pending" } else { "activity-item" };
                                            view! {
                                                <li class=pending_class>
                                                    <span class="activity-time">{time_ago}</span>
                                                    <span class="activity-type">{entry_type}</span>
                                                    <span class="activity-description">{entry.description.clone()}</span>
                                                    <span class="activity-amount">{sign}{"$"}{entry.amount.to_string()}</span>
                                                </li>
                                            }
                                        }
                                    />
                                </ul>
                            }.into_view()
                        }
                    }}
                </section>

                // Kid Logins section (parent-only feature)
                <Show when=move || is_parent_user.get()>
                <section class="kid-logins-section">
                    <h2>"Kid Logins"</h2>

                    {move || kid_login_error.get().map(|err| view! {
                        <div class="error-banner">{err}</div>
                    })}

                    <div class="create-kid-login">
                        <h3>"Create Kid Login"</h3>
                        <div class="form-row">
                            <input
                                type="text"
                                placeholder="Username"
                                disabled=move || creating_kid_login.get()
                                on:input=move |ev| set_new_kid_username.set(event_target_value(&ev))
                                prop:value=move || new_kid_username.get()
                            />
                            <input
                                type="password"
                                placeholder="Password"
                                disabled=move || creating_kid_login.get()
                                on:input=move |ev| set_new_kid_password.set(event_target_value(&ev))
                                prop:value=move || new_kid_password.get()
                            />
                            <select
                                disabled=move || creating_kid_login.get()
                                on:change=move |ev| {
                                    let val = event_target_value(&ev);
                                    set_selected_kid_id.set(if val.is_empty() { None } else { Some(val) });
                                }
                            >
                                <option value="">"(No specific kid)"</option>
                                {move || kid_summaries.get().into_iter().map(|kid| {
                                    view! {
                                        <option value=kid.kid.id.clone()>{kid.kid.name.clone()}</option>
                                    }
                                }).collect::<Vec<_>>()}
                            </select>
                            <button
                                class="create-btn"
                                disabled=move || creating_kid_login.get() || new_kid_username.get().trim().is_empty() || new_kid_password.get().trim().is_empty()
                                on:click=move |_| {
                                    let username = new_kid_username.get().trim().to_string();
                                    let password = new_kid_password.get().trim().to_string();
                                    let kid_id = selected_kid_id.get();

                                    if username.is_empty() || password.is_empty() {
                                        return;
                                    }

                                    set_creating_kid_login.set(true);
                                    set_kid_login_error.set(None);

                                    spawn_local(async move {
                                        match create_kid_login(username, password, kid_id).await {
                                            Ok(_) => {
                                                set_new_kid_username.set(String::new());
                                                set_new_kid_password.set(String::new());
                                                set_selected_kid_id.set(None);
                                                if let Ok(logins) = list_kid_logins().await {
                                                    set_kid_logins.set(logins);
                                                }
                                            }
                                            Err(e) => {
                                                set_kid_login_error.set(Some(format!("Failed to create login: {}", e)));
                                            }
                                        }
                                        set_creating_kid_login.set(false);
                                    });
                                }
                            >
                                {move || if creating_kid_login.get() { "Creating..." } else { "Create" }}
                            </button>
                        </div>
                    </div>

                    {move || {
                        let logins = kid_logins.get();
                        if logins.is_empty() {
                            view! { <p class="empty-state">"No kid logins yet."</p> }.into_view()
                        } else {
                            view! {
                                <div class="kid-logins-list">
                                    {logins.into_iter().map(|login| {
                                        let user_id = login.user_id.clone();
                                        let user_id_for_edit = user_id.clone();
                                        let user_id_for_save = user_id.clone();
                                        let user_id_for_cancel = user_id.clone();
                                        let user_id_for_check = user_id.clone();
                                        let linked_name = login.linked_kid_name.clone().unwrap_or_else(|| "(no specific kid)".to_string());
                                        let username = login.username.clone();

                                        view! {
                                            <div class="kid-login-card">
                                                <span class="kid-login-username">{username}</span>
                                                <span class="kid-login-linked">"→ "{linked_name}</span>
                                                {move || {
                                                    let is_editing = editing_kid_user_id.get().as_ref() == Some(&user_id_for_check);
                                                    if is_editing {
                                                        let user_id_save = user_id_for_save.clone();
                                                        let user_id_cancel = user_id_for_cancel.clone();
                                                        view! {
                                                            <div class="edit-password-form">
                                                                <input
                                                                    type="password"
                                                                    placeholder="New password"
                                                                    class="edit-password-input"
                                                                    disabled=move || updating_kid_password.get()
                                                                    on:input=move |ev| set_edit_kid_password.set(event_target_value(&ev))
                                                                    prop:value=move || edit_kid_password.get()
                                                                />
                                                                <button
                                                                    class="save-btn"
                                                                    disabled=move || updating_kid_password.get() || edit_kid_password.get().trim().is_empty()
                                                                    on:click=move |_| {
                                                                        let password = edit_kid_password.get().trim().to_string();
                                                                        let uid = user_id_save.clone();
                                                                        if password.is_empty() {
                                                                            return;
                                                                        }
                                                                        set_updating_kid_password.set(true);
                                                                        spawn_local(async move {
                                                                            match update_kid_password(uid, password).await {
                                                                                Ok(_) => {
                                                                                    set_editing_kid_user_id.set(None);
                                                                                    set_edit_kid_password.set(String::new());
                                                                                }
                                                                                Err(e) => {
                                                                                    set_kid_login_error.set(Some(format!("Failed to update password: {}", e)));
                                                                                }
                                                                            }
                                                                            set_updating_kid_password.set(false);
                                                                        });
                                                                    }
                                                                >
                                                                    {move || if updating_kid_password.get() { "Saving..." } else { "Save" }}
                                                                </button>
                                                                <button
                                                                    class="cancel-btn"
                                                                    disabled=move || updating_kid_password.get()
                                                                    on:click=move |_| {
                                                                        let _ = &user_id_cancel;
                                                                        set_editing_kid_user_id.set(None);
                                                                        set_edit_kid_password.set(String::new());
                                                                    }
                                                                >
                                                                    "Cancel"
                                                                </button>
                                                            </div>
                                                        }.into_view()
                                                    } else {
                                                        let user_id_edit = user_id_for_edit.clone();
                                                        view! {
                                                            <button
                                                                class="edit-password-btn"
                                                                on:click=move |_| {
                                                                    set_editing_kid_user_id.set(Some(user_id_edit.clone()));
                                                                    set_edit_kid_password.set(String::new());
                                                                }
                                                            >
                                                                "Edit Password"
                                                            </button>
                                                        }.into_view()
                                                    }
                                                }}
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_view()
                        }
                    }}
                </section>
                </Show>
            </div>
        </Show>
    }
}

#[component]
fn KidSummaryCard(summary: KidSummaryDto, set_view: WriteSignal<View>) -> impl IntoView {
    let kid_id = summary.kid.id.clone();
    view! {
        <div class="kid-card">
            <div class="kid-header">
                <h3>{summary.kid.name.clone()}</h3>
                <span class="balance">"Balance: $"{summary.balance.to_string()}</span>
            </div>
            {summary.recent_entry.map(|entry| {
                let entry_type = match entry.entry_type {
                    EntryTypeDto::Earned => "Earned",
                    EntryTypeDto::Adjusted => "Adjusted",
                };
                let sign = if entry.amount >= rust_decimal::Decimal::ZERO { "+" } else { "" };
                view! {
                    <div class="recent-entry">
                        <span class="entry-type">{entry_type}</span>
                        <span class="entry-description">{entry.description}</span>
                        <span class="entry-amount">{sign}{"$"}{entry.amount.to_string()}</span>
                    </div>
                }
            })}
            <button
                class="view-ledger-btn"
                on:click=move |_| set_view.set(View::Ledger(kid_id.clone()))
            >
                "View Ledger"
            </button>
        </div>
    }
}


fn format_time_ago(dt: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(dt);

    if duration.num_days() > 0 {
        format!("{}d ago", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{}h ago", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{}m ago", duration.num_minutes())
    } else {
        "just now".to_string()
    }
}

#[component]
pub fn LedgerView(kid_id: UuidDto, set_view: WriteSignal<View>) -> impl IntoView {
    let ledger = create_resource(move || kid_id.clone(), get_ledger);

    view! {
        <div class="ledger-view">
            <Suspense fallback=move || view! { <p>"Loading ledger..."</p> }>
                {move || {
                    ledger.get().map(|result| match result {
                        Ok(ledger_data) => {
                            view! {
                                <div>
                                    <div class="ledger-header">
                                        <button
                                            class="back-btn"
                                            on:click=move |_| set_view.set(View::Dashboard)
                                        >
                                            "← Back to Dashboard"
                                        </button>
                                        <h2>"Ledger"</h2>
                                        <div class="balance-display">
                                            <span class="balance-label">"Current Balance:"</span>
                                            <span class="balance-value">"$"{ledger_data.balance.to_string()}</span>
                                        </div>
                                    </div>

                                    <section class="transactions">
                                        <h3>"All Transactions"</h3>
                                        {if ledger_data.entries.is_empty() {
                                            view! { <p>"No transactions yet."</p> }.into_view()
                                        } else {
                                            let mut running_balance = rust_decimal::Decimal::ZERO;
                                            view! {
                                                <table class="ledger-table">
                                                    <thead>
                                                        <tr>
                                                            <th>"Date"</th>
                                                            <th>"Type"</th>
                                                            <th>"Description"</th>
                                                            <th>"Amount"</th>
                                                            <th>"Balance"</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        {ledger_data.entries.into_iter().map(|entry| {
                                                            running_balance += entry.amount;
                                                            let entry_type = match entry.entry_type {
                                                                EntryTypeDto::Earned => "Earned",
                                                                EntryTypeDto::Adjusted => "Adjusted",
                                                            };
                                                            let sign = if entry.amount >= rust_decimal::Decimal::ZERO { "+" } else { "" };
                                                            let date_str = entry.created_at.format("%Y-%m-%d").to_string();
                                                            let time_str = entry.created_at.format("%H:%M").to_string();
                                                            let balance_at_time = running_balance;

                                                            view! {
                                                                <tr class="ledger-row">
                                                                    <td class="date-cell">
                                                                        <div class="date">{date_str}</div>
                                                                        <div class="time">{time_str}</div>
                                                                    </td>
                                                                    <td class="type-cell">
                                                                        <span class={format!("badge badge-{}", entry_type.to_lowercase())}>
                                                                            {entry_type}
                                                                        </span>
                                                                    </td>
                                                                    <td class="description-cell">{entry.description}</td>
                                                                    <td class={format!("amount-cell {}", if entry.amount >= rust_decimal::Decimal::ZERO { "positive" } else { "negative" })}>
                                                                        {sign}{"$"}{entry.amount.abs().to_string()}
                                                                    </td>
                                                                    <td class="balance-cell">"$"{balance_at_time.to_string()}</td>
                                                                </tr>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </tbody>
                                                </table>
                                            }.into_view()
                                        }}
                                    </section>
                                </div>
                            }.into_view()
                        }
                        Err(e) => view! {
                            <div>
                                <button
                                    class="back-btn"
                                    on:click=move |_| set_view.set(View::Dashboard)
                                >
                                    "← Back to Dashboard"
                                </button>
                                <p class="error">"Error loading ledger: " {e.to_string()}</p>
                            </div>
                        }.into_view(),
                    })
                }}
            </Suspense>
        </div>
    }
}

/// Floating chat widget for kids to report completed tasks
#[component]
fn ChatWidget() -> impl IntoView {
    let (is_open, set_is_open) = create_signal(false);
    let (messages, set_messages) = create_signal(Vec::<ChatMessageDto>::new());
    let (input_value, set_input_value) = create_signal(String::new());
    let (is_sending, set_is_sending) = create_signal(false);

    let on_send = move |_| {
        let message = input_value.get().trim().to_string();
        if message.is_empty() || is_sending.get() {
            return;
        }

        set_is_sending.set(true);
        let history = messages.get();

        // Add user message to display immediately
        set_messages.update(|msgs| {
            msgs.push(ChatMessageDto {
                role: "user".to_string(),
                content: message.clone(),
            });
        });
        set_input_value.set(String::new());

        spawn_local(async move {
            match send_chat_message(message, history).await {
                Ok(response) => {
                    set_messages.update(|msgs| {
                        msgs.push(ChatMessageDto {
                            role: "assistant".to_string(),
                            content: response,
                        });
                    });
                }
                Err(e) => {
                    set_messages.update(|msgs| {
                        msgs.push(ChatMessageDto {
                            role: "assistant".to_string(),
                            content: format!("Oops! Something went wrong: {}", e),
                        });
                    });
                }
            }
            set_is_sending.set(false);
        });
    };

    let on_key_press = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Enter" && !ev.shift_key() {
            ev.prevent_default();
            on_send(());
        }
    };

    view! {
        <div class="chat-widget">
            // Chat toggle button
            <button
                class="chat-toggle"
                class:open=move || is_open.get()
                on:click=move |_| set_is_open.update(|v| *v = !*v)
            >
                {move || if is_open.get() { "✕" } else { "💬" }}
            </button>

            // Chat window
            <Show when=move || is_open.get()>
                <div class="chat-window">
                    <div class="chat-header">
                        <h3>"Hi! Need help?"</h3>
                    </div>

                    <div class="chat-messages">
                        {move || {
                            let msg_list = messages.get();
                            if msg_list.is_empty() {
                                view! {
                                    <div class="chat-empty">
                                        <p>"Tell me what chore you finished!"</p>
                                        <p class="hint">"e.g., \"I did the dishes\" or \"I fed the cat\""</p>
                                    </div>
                                }.into_view()
                            } else {
                                view! {
                                    <div class="messages-list">
                                        {msg_list.into_iter().map(|msg| {
                                            let role_class = if msg.role == "user" { "user" } else { "assistant" };
                                            let html_content = if msg.role == "assistant" {
                                                markdown_to_html(&msg.content)
                                            } else {
                                                // For user messages, just escape HTML
                                                msg.content
                                                    .replace('&', "&amp;")
                                                    .replace('<', "&lt;")
                                                    .replace('>', "&gt;")
                                            };
                                            view! {
                                                <div class=format!("message {}", role_class) inner_html=html_content>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                        {move || if is_sending.get() {
                                            view! {
                                                <div class="message assistant typing">
                                                    <span class="dot"></span>
                                                    <span class="dot"></span>
                                                    <span class="dot"></span>
                                                </div>
                                            }.into_view()
                                        } else {
                                            view! {}.into_view()
                                        }}
                                    </div>
                                }.into_view()
                            }
                        }}
                    </div>

                    <div class="chat-input">
                        <input
                            type="text"
                            placeholder="What did you do?"
                            disabled=move || is_sending.get()
                            on:input=move |ev| set_input_value.set(event_target_value(&ev))
                            on:keypress=on_key_press
                            prop:value=move || input_value.get()
                        />
                        <button
                            class="send-btn"
                            disabled=move || is_sending.get() || input_value.get().trim().is_empty()
                            on:click=move |_| on_send(())
                        >
                            "→"
                        </button>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn AdminPanel() -> impl IntoView {
    let (accounts, set_accounts) = create_signal(Vec::<AccountDto>::new());
    let (is_loaded, set_is_loaded) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);

    // Form state for creating new account
    let (new_username, set_new_username) = create_signal(String::new());
    let (new_password, set_new_password) = create_signal(String::new());
    let (creating, set_creating) = create_signal(false);

    // Load accounts on mount
    create_effect(move |_| {
        spawn_local(async move {
            match list_accounts().await {
                Ok(account_list) => {
                    set_accounts.set(account_list);
                    set_is_loaded.set(true);
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to load accounts: {}", e)));
                    set_is_loaded.set(true);
                }
            }
        });
    });

    let on_create = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        set_creating.set(true);

        let username = new_username.get();
        let password = new_password.get();

        spawn_local(async move {
            match create_account(username, password).await {
                Ok(new_account) => {
                    // Add to list
                    set_accounts.update(|accts| accts.push(new_account));
                    // Clear form
                    set_new_username.set(String::new());
                    set_new_password.set(String::new());
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to create account: {}", e)));
                }
            }
            set_creating.set(false);
        });
    };

    let handle_delete = move |account_id: String, username: String| {
        spawn_local(async move {
            match delete_account(account_id.clone()).await {
                Ok(()) => {
                    set_accounts.update(|accts| {
                        accts.retain(|a| a.id != account_id);
                    });
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to delete {}: {}", username, e)));
                }
            }
        });
    };

    view! {
        <div class="admin-panel">
            <h1>"Account Management"</h1>

            {move || error.get().map(|err| view! {
                <div class="error-banner">{err}</div>
            })}

            <Show
                when=move || is_loaded.get()
                fallback=|| view! { <p>"Loading accounts..."</p> }
            >
                <section class="create-account-section">
                    <h2>"Create New Account"</h2>
                    <form class="create-account-form" on:submit=on_create>
                        <div class="form-row">
                            <input
                                type="text"
                                placeholder="Username"
                                required
                                disabled=move || creating.get()
                                on:input=move |ev| set_new_username.set(event_target_value(&ev))
                                prop:value=move || new_username.get()
                            />
                            <input
                                type="password"
                                placeholder="Password"
                                required
                                disabled=move || creating.get()
                                on:input=move |ev| set_new_password.set(event_target_value(&ev))
                                prop:value=move || new_password.get()
                            />
                            <button type="submit" class="create-btn" disabled=move || creating.get()>
                                {move || if creating.get() { "Creating..." } else { "Create Account" }}
                            </button>
                        </div>
                    </form>
                </section>

                <section class="accounts-section">
                    <h2>"Existing Accounts"</h2>
                    {move || {
                        let account_list = accounts.get();
                        if account_list.is_empty() {
                            view! { <p class="empty-state">"No accounts yet."</p> }.into_view()
                        } else {
                            view! {
                                <div class="accounts-list">
                                    {account_list.into_iter().map(|account| {
                                        let account_id = account.id.clone();
                                        let username = account.username.clone();
                                        let username_for_delete = account.username.clone();
                                        let created = account.created_at.format("%Y-%m-%d").to_string();
                                        view! {
                                            <div class="account-card">
                                                <div class="account-info">
                                                    <span class="account-username">{username}</span>
                                                    <span class="account-created">"Created: "{created}</span>
                                                </div>
                                                <button
                                                    class="delete-btn"
                                                    on:click=move |_| handle_delete(account_id.clone(), username_for_delete.clone())
                                                >
                                                    "Delete"
                                                </button>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_view()
                        }
                    }}
                </section>
            </Show>
        </div>
    }
}
