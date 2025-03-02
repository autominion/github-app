use leptos::prelude::*;
use user_api::TaskStatus;

#[component]
pub fn TaskStatusIndicator(status: TaskStatus) -> impl IntoView {
    let status_class = match status {
        TaskStatus::Queued => "queued",
        TaskStatus::Running => "running",
        TaskStatus::Completed => "completed",
        TaskStatus::Failed => "failed",
    };

    let fa_icon = match status {
        TaskStatus::Queued => "fa-hourglass-start",
        TaskStatus::Running => "fa-spinner fa-spin",
        TaskStatus::Completed => "fa-check",
        TaskStatus::Failed => "fa-times",
    };

    let tooltip = match status {
        TaskStatus::Queued => "Task is queued",
        TaskStatus::Running => "Task is running",
        TaskStatus::Completed => "Task is completed",
        TaskStatus::Failed => "Task has failed",
    };

    let class = format!("task-status {} fa-solid {}", status_class, fa_icon);

    view! {
        <span class=class title=tooltip></span>
    }
}
