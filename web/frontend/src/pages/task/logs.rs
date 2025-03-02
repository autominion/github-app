use leptos::prelude::*;

use crate::api::use_task_logs;

#[component]
pub fn Logs(task_id: String, #[prop(into)] running: Signal<bool>) -> impl IntoView {
    let logs = use_task_logs(&task_id);

    move || {
        let logs = logs.get();
        let Some(Ok(logs)) = logs.map(|sw| sw.take()) else {
            return {
                view! { <></> };
                ().into_any()
            };
        };

        if running.get() {
            view! {
                <>
                    <p>"Logs will be available once the agent run completes."</p>
                </>
            }
            .into_any()
        } else if let Some(logs_content) = logs {
            view! {
                <>
                    <p>"Logs for this agent run."</p>
                    <pre class="logs-container">{logs_content}</pre>
                </>
            }
            .into_any()
        } else {
            view! {
                <>
                    <p>"No logs available for this agent run."</p>
                </>
            }
            .into_any()
        }
    }
}
