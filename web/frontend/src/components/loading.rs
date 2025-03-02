use leptos::prelude::*;

#[component]
pub fn Loading() -> impl IntoView {
    view! {
        <div class="loader">
            <div class="ripple"></div>
            <div class="pulse"></div>
        </div>
    }
}
