use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use web_sys::MouseEvent;

use crate::api;
use crate::components::*;
use crate::errors::handle_api_result;
use crate::routes::paths;

#[component]
pub fn WaitlistPage() -> impl IntoView {
    let waitlist_children: ChildrenFn = Arc::new(|| {
        view! {
            <Waitlist/>
        }
        .into_any()
    });

    view! {
        <StandardPage children=waitlist_children/>
    }
}

#[component]
pub fn Waitlist() -> impl IntoView {
    let user_info = api::use_user_info();
    let navigate = Arc::new(use_navigate());
    let error_store = expect_context::<RwSignal<crate::errors::ErrorStore>>();
    let confirm_dialog = RwSignal::new(false);

    Effect::new({
        let navigate = navigate.clone();
        move |_| {
            if let Some(Ok(info)) = user_info.get().map(|sw| sw.clone().take()) {
                if info.active {
                    navigate(paths::LANDING, Default::default());
                }
            }
        }
    });

    move || match user_info.get().map(|sw| sw.clone().take()) {
        Some(Ok(info)) => {
            let waitlist_content = if info.on_waitlist {
                let confirm_dialog = confirm_dialog;
                let navigate = navigate.clone();
                let error_store = error_store;
                let user_info = user_info;

                view! {
                        <>
                            <p>
                                <b>"You are on the waitlist! "</b>
                                "We will notify you as soon as we can grant you access. "
                                "The notification will be sent to your primary "
                                <a target="_blank" href="https://github.com/settings/emails">
                                    "GitHub email address"
                                </a>
                                " at "
                                <code>{info.email_domain.clone()}</code>
                                "."
                            </p>
                            <button
                                on:click=move |_| confirm_dialog.set(true)
                                class="secondary"
                            >
                                "Leave waitlist"
                            </button>
                            <Modal
                                title="Leave waitlist".to_owned()
                                visible=Signal::derive(move || confirm_dialog.get())
                                on_close=move || confirm_dialog.set(false)
                            >
                                <p>
                                    "Are you sure you want to withdraw from the waitlist?
                                        You will lose your current position."
                                </p>
                                <div class="buttons-row">
                                    <button
                                        on:click=move |_: MouseEvent| {
                                            confirm_dialog.set(false);
                                            let navigate = navigate.clone();
                                            let error_store = error_store;
                                            let user_info = user_info;

                                            spawn_local(async move {
                                                let result = api::http::leave_waitlist().await;
                                                let _ = handle_api_result(result, navigate, &error_store);
                                                user_info.refetch();
                                            });
                                        }
                                        class="danger"
                                    >
                                        "Leave"
                                    </button>
                                    <button
                                        on:click=move |_| confirm_dialog.set(false)
                                        class="primary"
                                    >
                                        "Stay"
                                    </button>
                                </div>
                            </Modal>
                        </>
                    }
                    .into_any()
            } else {
                let navigate = navigate.clone();
                let error_store = error_store;
                let user_info = user_info;

                view! {
                    <>
                        <p>"You can register your interest in the hosted version by joining our waitlist."</p>
                        <p>
                            "Clicking " <b>"Join Waitlist"</b> " means you agree to our "
                            <a href=paths::PRIVACY>
                                "Privacy Policy"
                            </a>
                            "."
                        </p>
                        <button
                            on:click=move |_| {
                                let nav = navigate.clone();
                                let err = error_store;
                                let ui = user_info;

                                spawn_local(async move {
                                    let result = api::http::join_waitlist().await;
                                    let _ = handle_api_result(result, nav, &err);
                                    ui.refetch();
                                });
                            }
                            class="primary"
                        >
                            "Join Waitlist"
                        </button>
                    </>
                }
                .into_any()
            };

            view! {
                <>
                    <h3>"Hosted Instance"</h3>
                    <span>"Hey, " {info.name.clone()} "! "</span>
                    <span>
                        "Would you be interested in using a hosted instance of autominion? If there is enough community support, we will host a public instance. "
                    </span>
                    {waitlist_content}
                </>
            }
            .into_any()
        }
        _ => {
            view! { <></> };
            ().into_any()
        }
    }
}
