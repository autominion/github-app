use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::api::use_user_info;
use crate::auth;
use crate::routes::paths;

#[component]
pub fn Navbar() -> impl IntoView {
    let logged_in = RwSignal::new(false);

    Effect::new(move || {
        logged_in.set(auth::logged_in());
    });

    move || {
        if logged_in.get() {
            view! { <NavbarLoggedIn/> }.into_any()
        } else {
            view! { <NavbarLoggedOut/> }.into_any()
        }
    }
}

#[component]
pub fn NavbarLoggedIn() -> impl IntoView {
    let location = use_location();
    let user_info = use_user_info();

    let nav_item_class = move |path: &'static str| {
        if location.pathname.get().starts_with(path) {
            "nav-item active".to_string()
        } else {
            "nav-item".to_string()
        }
    };

    view! {
        <nav class="nav-bar">
            <div class="nav-content">
                <div class="nav-row">
                    <a class="nav-title" href=paths::LANDING>
                        { crate::whitelabel::SERVICE_NAME }
                    </a>
                    <Suspense fallback=move || view! { <></> }>
                        { move || {
                            if let Some(Ok(user)) = user_info.get().map(|sw| sw.take()) {
                                if user.active {
                                    view! {
                                        <a class=nav_item_class(paths::TASKS) href=paths::TASKS>
                                            "Tasks"
                                        </a>
                                        <a class=nav_item_class(paths::REPOS) href=paths::REPOS>
                                            "Repos"
                                        </a>
                                        <a class=nav_item_class(paths::SETTINGS) href=paths::SETTINGS>
                                            "Settings"
                                        </a>
                                    }
                                    .into_any()
                                } else {
                                    view! {
                                        <a class="nav-item" href=paths::WAITLIST>
                                            "Waitlist"
                                        </a>
                                    }
                                    .into_any()
                                }
                            } else {
                                view! { <></> };
                                ().into_any()
                            }
                        }}
                    </Suspense>
                </div>
                <div class="nav-row">
                    <a class="button light" href=paths::LOGOUT>
                        <i class="fa-solid fa-right-from-bracket"></i>
                        <span class="small-space"></span>
                        "Sign out"
                    </a>
                </div>
            </div>
        </nav>
    }
}

#[component]
pub fn NavbarLoggedOut() -> impl IntoView {
    view! {
        <nav class="nav-bar">
            <div class="nav-content">
                <div class="nav-row">
                    <a class="nav-title" href=paths::LANDING>
                        { crate::whitelabel::SERVICE_NAME }
                    </a>
                </div>
                <div class="nav-row">
                    <a class="button light" href=paths::LOGIN>
                        <i class="fa-brands fa-github fa-fw"></i>
                        <span class="small-space"></span>
                        "Sign in"
                    </a>
                </div>
            </div>
        </nav>
    }
}
