use leptos::prelude::*;

use user_api::Repo;

use crate::components::*;

use super::add_repo_item::AddRepoItem;

#[component]
pub fn AddRepoModal(
    #[prop(into)] visible: Signal<bool>,
    repos: Vec<Repo>,
    on_add: Callback<Repo>,
    on_close: Callback<()>,
) -> impl IntoView {
    let filter = RwSignal::new(String::new());
    let on_filter_change = Callback::new(move |value: String| filter.set(value));

    let filtered_repos = Memo::new(move |_| {
        let filter_lower = filter.get().to_lowercase();
        repos
            .iter()
            .filter(|repo| !repo.active && repo.name.to_lowercase().contains(&filter_lower))
            .cloned()
            .collect::<Vec<Repo>>()
    });

    view! {
        <Modal title="Enable repository".to_string() visible on_close>
            <p>
                "Note that you can only enable repositories if you are the owner or if you have admin access to the organization."
            </p>
            <TextInput
                value=filter
                on_change=on_filter_change
                placeholder="Filter".to_string()
            />
            <ul class="listbox scrollable">
                {move || {
                    if filtered_repos.get().is_empty() {
                        view! {
                            <li class="placeholder">"No repositories found"</li>
                        }.into_any()
                    } else {
                        view! {
                            <For
                                each=move || filtered_repos.get()
                                key=|repo: &Repo| repo.id.clone()
                                children=move |repo: Repo| {
                                    let on_add = on_add;
                                    view! {
                                        <AddRepoItem on_add=on_add repo=repo.clone()/>
                                    }
                                }
                            />
                        }.into_any()
                    }
                }}
            </ul>
        </Modal>
    }
}
