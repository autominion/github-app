use leptos::prelude::*;
use pulldown_cmark::{Options, Parser};

use super::parse::Message;

#[component]
pub fn MessageItem(message: Message) -> impl IntoView {
    let Message { role, content, .. } = message;

    let options =
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let parser = Parser::new_ext(&content, options);
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);

    view! {
        <li class="message-item">
            <b>{ role.to_string() }</b>{": "}
            <div class="message-content" inner_html=html_output></div>
        </li>
    }
}
