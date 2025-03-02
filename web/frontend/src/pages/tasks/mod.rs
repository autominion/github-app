use std::sync::Arc;

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::api::use_tasks;
use crate::components::*;
use crate::routes::paths;

mod task_item;
mod task_status_indicator;

use task_item::TaskItem;

#[component]
pub fn TasksPage() -> impl IntoView {
    let tasks_content: ChildrenFn =
        Arc::new(move || view! { <><TasksPageContent /></> }.into_any());
    view! {
        <StandardPage children=tasks_content />
    }
}

#[component]
pub fn TasksPageContent() -> impl IntoView {
    let navigate = Arc::new(use_navigate());
    let tasks = use_tasks();

    move || {
        let navigate = navigate.clone();
        match tasks.get().map(|sw| sw.take()) {
            Some(Ok(tasks)) => view! {
                <>
                    <h1>"Tasks"</h1>
                    <p>"Manage the tasks you created."</p>
                    <ul class="listbox">
                        <For
                            each=move || tasks.clone()
                            key=|task| task.id.clone()
                            children=move |task| {
                                let task_id = task.id.clone();
                                let navigate = navigate.clone();
                                let on_manage_click: Callback<String> =
                                    Callback::new(move |_: String| {
                                        let route = format!("{}/{}", paths::TASKS, task_id);
                                        navigate(&route, Default::default());
                                    });
                                view! {
                                    <TaskItem task=task on_manage=on_manage_click />
                                }
                            }
                        />
                    </ul>
                </>
            }
            .into_any(),
            Some(Err(_)) | None => {
                view! { <></> };
                ().into_any()
            }
        }
    }
}
