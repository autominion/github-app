use std::sync::Arc;

use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use user_api::RepoUserInfo;

use crate::components::*;
use crate::errors::handle_api_result;
use crate::{
    api::{http, use_repo, use_repo_users},
    routes::paths,
};

mod remove_repo_modal;
mod remove_user_modal;

use remove_repo_modal::RemoveRepoModal;
use remove_user_modal::RemoveUserModal;

#[component]
pub fn RepoPage(id: String) -> impl IntoView {
    let repo_page_content: ChildrenFn = Arc::new({
        let id = id.clone();
        move || view! { <><RepoPageContent id=id.clone() /></> }.into_any()
    });
    view! {
        <StandardPage children=repo_page_content/>
    }
}

#[component]
pub fn RepoPageContent(id: String) -> impl IntoView {
    let repo_resource = use_repo(id.clone());
    let repo_users_resource = use_repo_users(id.clone());
    let navigate = Arc::new(use_navigate());
    let error_store = expect_context::<RwSignal<crate::errors::ErrorStore>>();

    // Reactive signals for the form state and modals.
    let new_user_login = RwSignal::new(String::new());
    let removing_user = RwSignal::new(None::<RepoUserInfo>);
    let removing_repo = RwSignal::new(false);

    let on_change = Callback::new(move |value: String| new_user_login.set(value));

    let on_add_user = {
        let id = id.clone();
        let navigate = navigate.clone();
        move |login: String| {
            let id = id.clone();
            let navigate = navigate.clone();
            let repo_users_resource = repo_users_resource;
            let error_store = error_store;
            spawn_local(async move {
                let result = http::add_repo_user(&id, &login).await;
                let _ = handle_api_result(result, navigate, &error_store);
                repo_users_resource.refetch();
            });
        }
    };

    let on_remove_user = {
        let id = id.clone();
        let navigate = navigate.clone();
        move |user: RepoUserInfo| {
            let id = id.clone();
            let repo_users_resource = repo_users_resource;
            let navigate = navigate.clone();
            let error_store = error_store;
            spawn_local(async move {
                let result = http::delete_repo_users(&id, &user.id).await;
                let _ = handle_api_result(result, navigate, &error_store);
                repo_users_resource.refetch();
            });
        }
    };

    let on_add_user_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let login = new_user_login.get().trim().to_string();
        if !login.is_empty() {
            on_add_user(login);
            new_user_login.set(String::new());
        }
    };

    let open_remove_user_modal = Callback::new({
        move |user: RepoUserInfo| {
            removing_user.set(Some(user));
        }
    });
    let on_confirm_remove_user = move || {
        if let Some(user) = removing_user.get() {
            on_remove_user(user);
            removing_user.set(None);
        }
    };
    let on_cancel_remove_user = move || {
        removing_user.set(None);
    };

    let on_remove_click = move |_| {
        removing_repo.set(true);
    };

    let on_remove_action = {
        let id = id.clone();
        let navigate = navigate.clone();
        move || {
            removing_repo.set(false);
            let id = id.clone();
            let navigate = navigate.clone();
            let error_store = error_store;
            spawn_local(async move {
                let result = http::remove_repo(&id).await;
                let _ = handle_api_result(result, navigate.clone(), &error_store);
                navigate(paths::REPOS, Default::default());
            });
        }
    };

    move || {
        let repo_option = repo_resource.get().map(|sw| sw.take());
        let users_option = repo_users_resource.get().map(|sw| sw.take());
        let on_add_user_submit = on_add_user_submit.clone();
        let on_confirm_remove_user = on_confirm_remove_user.clone();
        let on_remove_action = on_remove_action.clone();
        if let (Some(Ok(repo)), Some(Ok(users))) = (repo_option, users_option) {
            view! {
                    <>
                        <h1>{repo.name.clone()}</h1>
                        <p>
                            { format!("Configure {} for ", crate::whitelabel::SERVICE_NAME) }
                            <a
                                href=format!("https://github.com/{}", repo.name)
                                target="_blank"
                            >
                                {repo.name.clone()}
                            </a>
                            { "." }
                        </p>

                        <div style="margin-top: 2em"></div>
                        <h2>{ "Users" }</h2>
                        <p>
                            { "Users listed here can interact with " }
                            <b>{ crate::whitelabel::GITHUB_BOT_HANDLE }</b>
                            { " on this repository. " }
                            { "You can only add users who have previously signed in to this service. " }
                            { "If you cannot add them, please ask them to sign in first." }
                        </p>

                        <form on:submit=on_add_user_submit>
                            <div class="input-row">
                                <TextInput
                                    value=new_user_login
                                    on_change=on_change
                                    placeholder="GitHub username".to_owned()
                                />
                                <button type="submit" class="primary">
                                    { "Add User" }
                                </button>
                            </div>
                        </form>

                        {if users.is_empty() {
                            view! {
                                <ul class="listbox">
                                    <li class="placeholder">
                                        { "No users associated with this repository." }
                                    </li>
                                </ul>
                            }.into_any()
                        } else {
                            view! {
                                <ul class="listbox">
                                    <For
                                        each=move || users.clone()
                                        key=|user: &RepoUserInfo| user.id.clone()
                                        children=move |user: RepoUserInfo| {
                                            view! {
                                                <RepoUserItem
                                                    user=user
                                                    on_remove=open_remove_user_modal
                                                />
                                            }
                                        }
                                    />
                                </ul>
                            }.into_any()
                        }}

                        <div style="margin-top: 2em"></div>
                        <h2>{ "Danger Zone" }</h2>
                        <p>
                            { "Disabling this repository means you and any configured users can no longer interact with " }
                            <b>{ crate::whitelabel::GITHUB_BOT_HANDLE }</b>
                            { " on this repository. " }
                        </p>
                        <button class="danger" on:click=on_remove_click>
                            { "Disable" }
                        </button>

                        <RemoveUserModal
                            user=removing_user.get()
                            on_remove=on_confirm_remove_user
                            on_cancel=on_cancel_remove_user
                            on_close=on_cancel_remove_user
                        />

                        <RemoveRepoModal
                            repo=removing_repo.get().then(|| repo.clone())
                            on_remove=on_remove_action
                            on_cancel=move || { removing_repo.set(false); }
                            on_close=move || { removing_repo.set(false); }
                        />
                    </>
                }.into_any()
        } else {
            view! { <></> };
            ().into_any()
        }
    }
}

#[component]
pub fn RepoUserItem(user: RepoUserInfo, on_remove: Callback<RepoUserInfo>) -> impl IntoView {
    let user_clone = user.clone();
    view! {
        <li class="listitem">
            <span>{user.github_login.clone()}</span>
            <div class="stretch"></div>
            <span class="role-badge">{ "Member" }</span>
            <button class="icon-button" on:click=move |_| on_remove.run(user_clone.clone())>
                <i class="fa-solid fa-trash"></i>
            </button>
        </li>
    }
}
