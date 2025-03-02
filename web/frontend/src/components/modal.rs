use leptos::prelude::*;

#[component]
pub fn Modal(
    title: String,
    #[prop(into)] visible: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
    children: Children,
) -> impl IntoView {
    let on_close_overlay = on_close;
    let on_close_icon = on_close;

    view! {
        <div
            on:click=move |_: web_sys::MouseEvent| on_close_overlay.run(())
            class="modal"
            class:visible=visible>
            <div
                class="modal-content"
                on:click=move |e: web_sys::MouseEvent| e.stop_propagation()>
                <div class="modal-header">
                    <b class="modal-title">{ title }</b>
                    <i
                        on:click=move |_: web_sys::MouseEvent| on_close_icon.run(())
                        class="modal-close fa-solid fa-xmark"
                    ></i>
                </div>
                { children() }
            </div>
        </div>
    }
}
