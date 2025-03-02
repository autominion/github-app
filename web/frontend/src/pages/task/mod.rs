use std::sync::Arc;

use leptos::prelude::*;
use user_api::TaskStatus;
use web_sys::{window, ScrollBehavior, ScrollToOptions};

use crate::api::use_task;
use crate::components::*;

mod interaction_item;
mod llm_interactions;
mod logs;
mod message_item;
mod parse;
mod request;
mod response;

use llm_interactions::LlmInteractions;
use logs::Logs;

#[component]
pub fn TaskPage(id: String) -> impl IntoView {
    // Wrap the child in an Arc so it implements ChildrenFn.
    let task_content: ChildrenFn =
        Arc::new(move || view! { <><TaskContent id=id.clone() /></> }.into_any());
    view! {
        <StandardPage children=task_content />
    }
}

#[component]
pub fn TaskContent(id: String) -> impl IntoView {
    let task_resource = use_task(id.clone());

    move || match task_resource.get().map(|sw| sw.take()) {
        Some(Ok(task)) => {
            let github_url =
                format!("https://github.com/{}/issues/{}", task.repo_name, task.issue_number);
            let interactions = task.interactions.clone();

            let initial_interaction_id =
                interactions.last().map(|interaction| interaction.id.clone());
            let active_interaction_id = RwSignal::new(initial_interaction_id);
            let active_tab = RwSignal::new(0);
            let tab_labels = vec!["Prompts".to_string(), "Logs".to_string()];

            let onclick_fab = {
                let last_interaction_id =
                    interactions.last().map(|interaction| interaction.id.clone());
                move |_| {
                    if let Some(id) = last_interaction_id.clone() {
                        active_interaction_id.set(Some(id.clone()));
                        if let Some(win) = window() {
                            let scroll_options = ScrollToOptions::new();
                            if let Some(document) = win.document() {
                                if let Some(body) = document.body() {
                                    let scroll_height = body.scroll_height() as f64;
                                    scroll_options.set_top(scroll_height);
                                }
                            }
                            scroll_options.set_behavior(ScrollBehavior::Smooth);
                            win.scroll_to_with_scroll_to_options(&scroll_options);
                        }
                    }
                }
            };
            let on_tab_change = Callback::new(move |new_tab: usize| {
                active_tab.set(new_tab);
            });

            view! {
                <>
                    <h3>
                        <a
                            href=github_url
                            target="_blank"
                            class="icon-button fa-brands fa-github fa-fw">
                        </a>
                        <span class="small-space"></span>
                        {format!("{}#{}", task.repo_name, task.issue_number)}
                    </h3>

                    <TabBar
                        tabs=tab_labels
                        active_tab
                        on_tab_change
                    />

                    {
                        move || {
                            if active_tab.get() == 0 {
                                view! {
                                    <>
                                        <LlmInteractions
                                            interactions=interactions.clone()
                                            active_interaction_id=active_interaction_id
                                        />
                                        <button class="follow-prompt-fab" on:click=onclick_fab.clone()>
                                            <i class="fas fa-chevron-down"></i>
                                        </button>
                                    </>
                                }
                                .into_any()
                            } else {
                                view! {
                                    <Logs
                                        task_id=task.id.clone()
                                        running=task.status == TaskStatus::Running
                                    />
                                }
                                .into_any()
                            }
                        }
                    }
                </>
            }
            .into_any()
        }
        Some(Err(_)) | None => {
            view! { <></> };
            ().into_any()
        }
    }
}
