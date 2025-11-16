use leptos::*;
use leptos_meta::*;
use crate::components::Dashboard;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/loaa-web.css"/>
        <Link rel="stylesheet" href="/style/main.css"/>
        <Title text="Loa'a - Chore Tracker"/>

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

#[cfg(feature = "ssr")]
pub fn shell(_options: LeptosOptions) -> impl IntoView {
    view! {
        <App/>
    }
}
