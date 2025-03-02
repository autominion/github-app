use leptos::prelude::*;

use super::ErrorStore;

#[component]
pub fn ErrorModal() -> impl IntoView {
    let error_store =
        use_context::<RwSignal<ErrorStore>>().expect("ErrorStore context not provided");

    let modal_class = move || {
        if error_store.get().error.is_some() {
            "modal visible".to_string()
        } else {
            "modal".to_string()
        }
    };

    let on_close = move |_| {
        error_store.update(|store| {
            store.error = None;
        });
        web_sys::window().unwrap().location().reload().unwrap();
    };

    view! {
        <div class=modal_class>
            <div class="modal-content">
                <h2>"An error occurred"</h2>
                <Show
                    when=move || error_store.get().error.is_some()
                    fallback=|| ()
                >
                    {move || {
                        let store = error_store.get();
                        let error = store.error.as_ref().unwrap();
                        view! {
                            <p>{error.to_string()}</p>
                        }
                    }}
                </Show>
                <p>
                    "If the problem persists, please contact us."
                </p>
                <div class="buttons-row">
                    <button class="primary" on:click=on_close>
                        "Close"
                    </button>
                </div>
            </div>
        </div>
    }
}
