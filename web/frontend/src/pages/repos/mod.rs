use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use std::sync::Arc;
use user_api::Repo;

use crate::api;
use crate::api::http;
use crate::components::*;
use crate::routes::paths;

mod add_repo_item;
mod add_repo_modal;
mod repo_item;

use add_repo_modal::AddRepoModal;
use repo_item::RepoItem;

#[component]
pub fn ReposPage() -> impl IntoView {
    // Provide children as an Arc-wrapped closure.
    let repos_page_content: ChildrenFn =
        Arc::new(move || view! { <><ReposPageContent /></> }.into_any());
    view! {
        <StandardPage children=repos_page_content />
    }
}

#[component]
pub fn ReposPageContent() -> impl IntoView {
    let repos_resource = api::use_repos();
    let navigate = use_navigate();

    move || match repos_resource.get().map(|sw| sw.take()) {
        Some(Ok(repos)) => {
            let adding_repo = RwSignal::new(false);

            let add_repo_modal_callback = move |_| {
                adding_repo.set(true);
            };

            let on_add_action = Callback::new({
                let repos_resource = repos_resource;
                move |repo: Repo| {
                    let id = repo.id.clone();
                    spawn_local(async move {
                        let _ = http::add_repo(&id).await;
                        repos_resource.refetch();
                    });
                }
            });

            let active_repos: Vec<Repo> =
                repos.iter().filter(|&repo| repo.active).cloned().collect();

            view! {
                <>
                    <h1>"Repositories"</h1>
                    <p>
                        "You can interact with " <b> { crate::whitelabel::GITHUB_BOT_HANDLE } </b> " on all repositories listed below."
                    </p>
                    <div class="buttons-row small-vertical-margins">
                        <b>"Enabled repositories"</b>
                        <div class="stretch"></div>
                        <button class="primary" on:click=add_repo_modal_callback>
                            "Add"
                        </button>
                    </div>
                    <ul class="listbox">
                        {
                            if active_repos.is_empty() {
                                view! {
                                    <>
                                        <li class="placeholder">"No repositories enabled"</li>
                                    </>
                                }.into_any()
                            } else {
                                let navigate = navigate.clone();
                                view! {
                                    <>
                                        <For
                                            each=move || active_repos.clone()
                                            key=|repo: &Repo| repo.id.clone()
                                            children=move |repo: Repo| {
                                                let repo_id = repo.id.clone();
                                                let navigate = navigate.clone();
                                                let on_manage_click = Callback::new(move |_| {
                                                    navigate(
                                                        &format!("{}/{}", paths::REPOS, repo_id),
                                                        Default::default(),
                                                    );
                                                });
                                                view! {
                                                    <RepoItem repo=repo.clone() on_manage=on_manage_click />
                                                }
                                            }
                                        />
                                    </>
                                }.into_any()
                            }
                        }
                    </ul>
                    <AddRepoModal
                        visible=adding_repo
                        repos=repos.clone()
                        on_add=Callback::new({
                            move |repo: Repo| {
                                on_add_action.run(repo);
                                adding_repo.set(false);
                            }
                        })
                        on_close=Callback::new({
                            move |_| {
                                adding_repo.set(false);
                            }
                        })
                    />
                </>
            }
            .into_any()
        }
        Some(Err(_)) => view! { <p>"Error loading repositories"</p> }.into_any(),
        None => view! { <p>"Loading repositories..."</p> }.into_any(),
    }
}
