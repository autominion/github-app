use leptos::prelude::*;

use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{HtmlInputElement, InputEvent};

fn get_value_from_input_event(e: InputEvent) -> String {
    let event: web_sys::Event = e.dyn_into().unwrap_throw();
    let event_target = event.target().unwrap_throw();
    let target: HtmlInputElement = event_target.dyn_into().unwrap_throw();
    target.value()
}

#[component]
pub fn TextInput(
    /// A writable signal holding the current value.
    #[prop(into)]
    value: RwSignal<String>,
    /// An optional placeholder (defaults to empty string).
    #[prop(optional)]
    placeholder: Option<String>,
    /// An optional callback to run whenever the input value changes.
    #[prop(optional)]
    on_change: Option<Callback<String>>,
) -> impl IntoView {
    let placeholder = placeholder.unwrap_or_default();

    view! {
        <input
            type="text"
            bind:value=value
            placeholder=placeholder
            on:input=move |e: web_sys::Event| {
                let input_event: InputEvent = e.dyn_into().unwrap_throw();
                let new_value = get_value_from_input_event(input_event);
                if let Some(callback) = on_change.as_ref() {
                    callback.run(new_value);
                }
            }
        />
    }
}
