use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use web_sys::window;

use crate::api;
use crate::components::*;
use crate::errors::handle_api_result;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let settings_content: ChildrenFn =
        Arc::new(move || view! { <><SettingsPageContent /></> }.into_any());

    view! {
        <StandardPage children=settings_content/>
    }
}

#[component]
pub fn SettingsPageContent() -> impl IntoView {
    let status_resource = api::use_openrouter_status();
    let navigate = Arc::new(use_navigate());
    let error_store = expect_context::<RwSignal<crate::errors::ErrorStore>>();
    let confirm_modal = RwSignal::new(false);

    let on_disconnect = {
        move |_| {
            confirm_modal.set(false);
            let status_resource = status_resource;
            let error_store = error_store;
            let navigate = navigate.clone();
            spawn_local(async move {
                let result = api::http::disconnect_openrouter().await;
                let _ = handle_api_result(result, navigate, &error_store);
                status_resource.refetch();
            });
        }
    };

    let on_connect = move |_| {
        if let Some(win) = window() {
            let _ = win.location().set_href("/auth/openrouter");
        }
    };

    move || {
        let on_disconnect = on_disconnect.clone();

        if let Some(Ok(status)) = status_resource.get().map(|sw| sw.take()) {
            view! {
                <>
                    <h1>"Settings"</h1>
                    <h2>"OpenRouter"</h2>
                    { if status.connected {
                        view! {
                            <p>
                                "You have connected "
                                <a target="_blank" href="https://openrouter.ai/settings/keys/">
                                    "your OpenRouter account"
                                </a>
                                ". Agents running on your behalf can access LLMs via an API proxy. The service will forward valid requests to your OpenRouter account."
                            </p>
                        }.into_any()
                    } else {
                        view! {
                            <p>
                                "Connect your "
                                <a target="_blank" href="https://openrouter.ai/">
                                    "OpenRouter"
                                </a>
                                { format!(" account to {}.", crate::whitelabel::SERVICE_NAME) }
                            </p>
                        }.into_any()
                    }}
                    { if status.connected {
                        view! {
                            <button class="danger" on:click=move |_| confirm_modal.set(true)>
                                "Disconnect OpenRouter"
                            </button>
                        }.into_any()
                    } else {
                        view! {
                            <button on:click=on_connect>
                                "Connect OpenRouter"
                            </button>
                        }.into_any()
                    }}
                    <p>
                        <b>"Security: "</b>
                        "Agents running on this service will "
                        <b>"not"</b>
                        " have access to your OpenRouter API key. Agents are fully isolated and can only access the API proxy which proxies valid requests."
                    </p>
                    <p>
                        <b>"Billing: "</b>
                        "OpenRouter charges you for any tokens that agents running on this service consume on your behalf. You can revoke access at any time, both on this page and on the "
                        <a target="_blank" href="https://openrouter.ai/settings/keys/">
                            "OpenRouter settings page"
                        </a>
                        ". This service does "
                        <b>"not"</b>
                        " apply any extra charges for using OpenRouter."
                    </p>
                    <Modal
                        title="Disconnect OpenRouter".to_owned()
                        visible=confirm_modal
                        on_close=move || confirm_modal.set(false)
                    >
                        <p>
                            "Are you sure you want to disconnect OpenRouter? Agents will no longer be able to access LLMs via the API proxy."
                        </p>
                        <div class="buttons-row">
                            <button on:click=on_disconnect class="danger">
                                "Disconnect"
                            </button>
                            <button on:click=move |_| confirm_modal.set(false) class="primary">
                                "Keep"
                            </button>
                        </div>
                    </Modal>
                </>
            }.into_any()
        } else {
            view! { <></> };
            ().into_any()
        }
    }
}
