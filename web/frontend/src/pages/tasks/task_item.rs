use leptos::prelude::*;

use user_api::TaskInfo;

use super::task_status_indicator::TaskStatusIndicator;

#[component]
pub fn TaskItem(task: TaskInfo, on_manage: Callback<String>) -> impl IntoView {
    let task_id = task.id.clone();
    let on_click = move |_: web_sys::MouseEvent| {
        on_manage.run(task_id.clone());
    };

    view! {
        <li class="listitem clickable" on:click=on_click>
            <span class="small-space"></span>
            <TaskStatusIndicator status=task.status.clone() />
            <span class="medium-space"></span>
            {task.repo_name} { "#" } {task.issue_number}
            <div class="stretch"></div>
        </li>
    }
}
