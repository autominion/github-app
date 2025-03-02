use leptos::prelude::*;

use crate::routes::paths;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="footer">
            <div class="footer-content">
                <div class="footer-row">
                    <a class="footer-title" href=paths::LEGAL_NOTICE>{ "Legal Notice" }</a>
                    <span style="color: lightgrey">{ " | " }</span>
                    <a class="footer-title" href=paths::PRIVACY>{ "Privacy Policy" }</a>
                </div>
            </div>
        </footer>
    }
}
