use leptos::prelude::*;

use user_api::RepoUserInfo;

use crate::components::Modal;

#[component]
pub fn RemoveUserModal(
    user: Option<RepoUserInfo>,
    #[prop(into)] on_remove: Callback<()>,
    #[prop(into)] on_cancel: Callback<()>,
    #[prop(into)] on_close: Callback<()>,
) -> impl IntoView {
    let visible = RwSignal::new(user.is_some());
    let user_login = user.as_ref().map(|u| u.github_login.clone()).unwrap_or_default();

    view! {
        <Modal title={format!("Remove user {}", user_login)} visible on_close>
            <p>{ "Are you sure you want to remove this user from the repository?" }</p>
            <div class="buttons-row">
                <button class="danger" on:click=move |_| { on_remove.run(()); }>{ "Remove" }</button>
                <button on:click=move |_| { on_cancel.run(()); }>{ "Cancel" }</button>
            </div>
        </Modal>
    }
}
