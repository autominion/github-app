use leptos::prelude::*;

use user_api::Repo;

use crate::components::Modal;

#[component]
pub fn RemoveRepoModal(
    repo: Option<Repo>,
    #[prop(into)] on_remove: Callback<()>,
    #[prop(into)] on_cancel: Callback<()>,
    #[prop(into)] on_close: Callback<()>,
) -> impl IntoView {
    let visible = RwSignal::new(repo.is_some());
    let repo_name = repo.as_ref().map(|r| r.name.clone()).unwrap_or_default();
    let repo_clone = repo.clone();

    let on_remove_click = {
        move |_| {
            if repo_clone.is_some() {
                on_remove.run(());
            }
        }
    };

    let on_cancel_click = {
        move |_| {
            on_cancel.run(());
        }
    };

    view! {
        <Modal title={format!("Disable {}", repo_name)} visible on_close>
            <p>{ format!("Are you sure you want to disable {}?", repo_name) }</p>
            <div class="buttons-row">
                <button class="danger" on:click=on_remove_click>{ "Disable" }</button>
                <button on:click=on_cancel_click>{ "Keep" }</button>
            </div>
        </Modal>
    }
}
