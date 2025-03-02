use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::auth;
use crate::routes::paths;

#[component]
pub fn WithUser(children: ChildrenFn) -> impl IntoView {
    let navigate = use_navigate();
    let logged_in = RwSignal::new(false);

    Effect::new(move || {
        logged_in.set(auth::logged_in());
    });

    Effect::new(move |_| {
        if !auth::logged_in() {
            navigate(paths::LOGIN, Default::default());
        }
    });

    move || {
        if logged_in.get() {
            children().into_any()
        } else {
            view! { <></> };
            ().into_any()
        }
    }
}
