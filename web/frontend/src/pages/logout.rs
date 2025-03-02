use leptos::prelude::*;

use crate::{auth::delete_cookies, routes::paths};

#[component]
pub fn LogoutPage() -> impl IntoView {
    Effect::new(move |_| {
        delete_cookies();
        let window = web_sys::window().expect("no global `window` exists");
        window.location().set_href(paths::LANDING).unwrap();
    });

    view! {
        <></>
    }
}
