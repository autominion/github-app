use leptos::prelude::*;
use leptos_router::hooks::*;

use crate::auth::set_cookies;
use crate::routes::paths;

#[component]
pub fn LoginPage() -> impl IntoView {
    let query = use_query_map();

    Effect::new(move |_| {
        let params = query.get();
        let window = web_sys::window().expect("no global `window` exists");
        if let (Some(access_token), Some(refresh_token)) =
            (params.get("access_token"), params.get("refresh_token"))
        {
            set_cookies(&access_token, &refresh_token);
            // Redirect to the landing page with a full page reload.
            // This ensures the login status is updated.
            window.location().set_href(paths::LANDING).unwrap();
        } else {
            window.location().set_href("/auth").unwrap();
        }
    });

    view! { <></> }
}
