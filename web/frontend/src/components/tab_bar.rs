use leptos::prelude::*;

#[component]
pub fn TabBar(
    tabs: Vec<String>,
    #[prop(into)] active_tab: Signal<usize>,
    on_tab_change: Callback<usize>,
) -> impl IntoView {
    let buttons = tabs
        .into_iter()
        .enumerate()
        .map(|(i, label)| {
            view! {
                <button
                    class=move || if i == active_tab.get() { "tab-button active" } else { "tab-button" }
                    on:click=move |_| on_tab_change.run(i)
                >
                    {label}
                </button>
            }
        })
        .collect_view();

    view! {
        <div class="tab-bar-container">
            <div class="tab-bar">
                {buttons}
            </div>
        </div>
    }
}
