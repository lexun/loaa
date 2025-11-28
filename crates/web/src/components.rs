use leptos::*;
use crate::server_functions::*;
use crate::dto::*;

#[derive(Debug, Clone)]
pub enum View {
    Login,
    Dashboard,
    Ledger(UuidDto),
}

#[component]
pub fn Login(set_view: WriteSignal<View>) -> impl IntoView {
    let (username, set_username) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (error, set_error) = create_signal(Option::<String>::None);
    let (logging_in, set_logging_in) = create_signal(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        set_logging_in.set(true);

        let username_val = username.get();
        let password_val = password.get();

        spawn_local(async move {
            match login(username_val, password_val).await {
                Ok(true) => {
                    set_view.set(View::Dashboard);
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
                    // User is authenticated, show dashboard
                    set_current_view.set(View::Dashboard);
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
    let dashboard_data = create_resource(|| (), |_| get_dashboard_data());

    view! {
        <Suspense fallback=move || view! { <p>"Loading dashboard..."</p> }>
            {move || {
                dashboard_data.get().map(|result| match result {
                    Ok(data) => {
                        view! {
                            <div>
                                <section class="overview">
                                    <h2>"Overview"</h2>
                                    <div class="stats">
                                        <div class="stat">
                                            <span class="stat-label">"Total Kids:"</span>
                                            <span class="stat-value">{data.total_kids}</span>
                                        </div>
                                        <div class="stat">
                                            <span class="stat-label">"Active Tasks:"</span>
                                            <span class="stat-value">{data.active_tasks}</span>
                                        </div>
                                    </div>
                                </section>

                                <section class="kids-section">
                                    <h2>"Kids"</h2>
                                    <div class="kids-grid">
                                        {data.kid_summaries.into_iter().map(|summary| {
                                            view! { <KidSummaryCard summary=summary set_view=set_view /> }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                </section>

                                <RecentActivity />
                            </div>
                        }.into_view()
                    }
                    Err(e) => view! {
                        <p class="error">"Error loading dashboard: " {e.to_string()}</p>
                    }.into_view(),
                })
            }}
        </Suspense>
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

#[component]
fn RecentActivity() -> impl IntoView {
    let activity = create_resource(|| (), |_| get_recent_activity(10));

    view! {
        <section class="recent-activity">
            <h2>"Recent Activity"</h2>
            <Suspense fallback=move || view! { <p>"Loading activity..."</p> }>
                {move || {
                    activity.get().map(|result| match result {
                        Ok(entries) => {
                            if entries.is_empty() {
                                view! { <p>"No activity yet."</p> }.into_view()
                            } else {
                                view! {
                                    <ul class="activity-list">
                                        {entries.into_iter().map(|entry| {
                                            let entry_type = match entry.entry_type {
                                                EntryTypeDto::Earned => "Earned",
                                                EntryTypeDto::Adjusted => "Adjusted",
                                            };
                                            let sign = if entry.amount >= rust_decimal::Decimal::ZERO { "+" } else { "" };
                                            let time_ago = format_time_ago(entry.created_at);
                                            view! {
                                                <li class="activity-item">
                                                    <span class="activity-time">{time_ago}</span>
                                                    <span class="activity-type">{entry_type}</span>
                                                    <span class="activity-description">{entry.description}</span>
                                                    <span class="activity-amount">{sign}{"$"}{entry.amount.to_string()}</span>
                                                </li>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </ul>
                                }.into_view()
                            }
                        }
                        Err(e) => view! {
                            <p class="error">"Error loading activity: " {e.to_string()}</p>
                        }.into_view(),
                    })
                }}
            </Suspense>
        </section>
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
