use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use crate::components::{LoginPage, DashboardPage, LedgerPage, AdminPage};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/loaa-web.css"/>
        <Link rel="stylesheet" href="/style/main.css"/>
        <Title text="Loa'a - Chore Tracker"/>

        <Router>
            <main>
                <Routes>
                    <Route path="/" view=LoginPage />
                    <Route path="/login" view=LoginPage />
                    <Route path="/dashboard" view=DashboardPage />
                    <Route path="/kids/:id/ledger" view=LedgerPage />
                    <Route path="/admin" view=AdminPage />
                </Routes>
            </main>
        </Router>
    }
}

#[cfg(feature = "ssr")]
pub fn shell(_options: LeptosOptions) -> impl IntoView {
    view! {
        <App/>
    }
}
