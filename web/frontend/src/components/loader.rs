use leptos::prelude::*;

use super::loading::Loading;

#[component]
pub fn Loader(children: ChildrenFn) -> impl IntoView {
    let fallback = || view! { <Loading/> };

    view! {
        <Transition fallback=fallback>
            { children() }
        </Transition>
    }
}
