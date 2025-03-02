use leptos::prelude::*;
use user_api::{Repo, UserRole};

#[component]
pub fn AddRepoItem(repo: Repo, on_add: Callback<Repo>) -> impl IntoView {
    let is_admin = repo.role == UserRole::Admin;
    let repo_clone = repo.clone();
    let role_label = if is_admin { "Admin" } else { "Member" };

    let on_add_click = {
        move |_| {
            if is_admin {
                on_add.run(repo_clone.clone());
            }
        }
    };

    view! {
        <li
            class=if is_admin { "listitem clickable" } else { "listitem" }
            on:click=on_add_click>
            <span class="small-space"></span>
            {repo.name.clone()}
            <div class="stretch"></div>
            <span class="role-badge">{role_label}</span>

            {
                if is_admin {
                    view! {
                        <>
                            <i class="icon-button fa-solid fa-plus"
                               on:click=move |_| {
                                   on_add.run(repo.clone());
                               }
                            />
                        </>
                    }.into_any()
                } else {
                    view! {
                        <> </>
                    };
                    ().into_any()
                }
            }
        </li>
    }
}
