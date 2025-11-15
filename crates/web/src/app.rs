use leptos::*;
use crate::components::Dashboard;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <div class="container">
            <header>
                <h1>"Loa'a"</h1>
                <p>"Chore and rewards tracking system"</p>
            </header>
            <main>
                <Dashboard />
            </main>
        </div>
    }
}
