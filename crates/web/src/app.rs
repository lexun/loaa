use leptos::*;
use loaa_web::server_functions::*;

#[component]
pub fn App() -> impl IntoView {
    let kids = create_resource(|| (), |_| get_kids());
    let tasks = create_resource(|| (), |_| get_tasks());

    view! {
        <div class="container">
            <header>
                <h1>"Loa'a"</h1>
                <p>"Chore and rewards tracking system"</p>
            </header>
            <main>
                <section>
                    <h2>"Kids"</h2>
                    <Suspense fallback=move || view! { <p>"Loading kids..."</p> }>
                        {move || {
                            kids.get().map(|result| match result {
                                Ok(kids_list) => {
                                    if kids_list.is_empty() {
                                        view! { <p>"No kids yet. Add one below!"</p> }.into_view()
                                    } else {
                                        view! {
                                            <ul>
                                                {kids_list.into_iter().map(|kid| {
                                                    view! {
                                                        <li>{kid.name.clone()}</li>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </ul>
                                        }.into_view()
                                    }
                                }
                                Err(e) => view! { <p class="error">"Error: " {e.to_string()}</p> }.into_view(),
                            })
                        }}
                    </Suspense>
                </section>
                <section>
                    <h2>"Tasks"</h2>
                    <Suspense fallback=move || view! { <p>"Loading tasks..."</p> }>
                        {move || {
                            tasks.get().map(|result| match result {
                                Ok(tasks_list) => {
                                    if tasks_list.is_empty() {
                                        view! { <p>"No tasks yet. Add one below!"</p> }.into_view()
                                    } else {
                                        view! {
                                            <ul>
                                                {tasks_list.into_iter().map(|task| {
                                                    view! {
                                                        <li>
                                                            <strong>{task.name.clone()}</strong>
                                                            " - $"
                                                            {task.value.to_string()}
                                                            " ("
                                                            {format!("{:?}", task.cadence)}
                                                            ")"
                                                        </li>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </ul>
                                        }.into_view()
                                    }
                                }
                                Err(e) => view! { <p class="error">"Error: " {e.to_string()}</p> }.into_view(),
                            })
                        }}
                    </Suspense>
                </section>
            </main>
        </div>
    }
}
