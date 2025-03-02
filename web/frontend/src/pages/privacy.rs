use leptos::prelude::*;
use pulldown_cmark::{Options, Parser};
use url::Url;

#[component]
pub fn PrivacyPage() -> impl IntoView {
    let markdown_content = LocalResource::new(|| async move {
        let window = web_sys::window().expect("Window object not found");
        let href = window.location().href().expect("Failed to retrieve window location href");
        let url = Url::parse(&href)
            .expect("Failed to parse window location URL")
            .join("/static/privacy.md")
            .expect("Failed to construct privacy policy URL");
        let response = reqwest::get(url).await;
        match response {
            Ok(res) => {
                if res.status().is_success() {
                    res.text()
                        .await
                        .unwrap_or_else(|_| "Failed to load privacy policy.".to_string())
                } else {
                    "Failed to load privacy policy.".to_string()
                }
            }
            Err(_) => "Failed to load privacy policy.".to_string(),
        }
    });

    let html_output = Memo::new(move |_| match markdown_content.get() {
        Some(content) => {
            let options =
                Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
            let parser = Parser::new_ext(&content, options);
            let mut html = String::new();
            pulldown_cmark::html::push_html(&mut html, parser);
            html
        }
        None => String::new(),
    });

    view! {
        <div class="content-container">
            <div class="content">
                <section inner_html=move || html_output.get()></section>
            </div>
        </div>
    }
}
