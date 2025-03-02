use leptos::prelude::*;

use user_api::{Repo, UserRole};

#[component]
pub fn RepoItem(repo: Repo, on_manage: Callback<Repo>) -> impl IntoView {
    let is_admin = repo.role == UserRole::Admin;
    let on_manage_click = {
        let repo_clone = repo.clone();
        move |_| {
            if is_admin {
                on_manage.run(repo_clone.clone());
            }
        }
    };

    let role_label = if is_admin { "Admin" } else { "Member" };

    view! {
        <li
            class=if is_admin { "listitem clickable" } else { "listitem" }
            on:click=on_manage_click>
            {repo.name}
            <div class="stretch"></div>
            <span class="role-badge">{role_label}</span>
        </li>
    }
}
