use leptos::prelude::*;

use serde_json::Value;
use web_sys::{HtmlLiElement, ScrollBehavior, ScrollIntoViewOptions};

use super::message_item::MessageItem;
use super::parse::{parse_chat_request, parse_chat_response};

#[component]
pub fn InteractionItem(
    request: Option<Value>,
    response: Option<Value>,
    #[prop(into)] is_active: Signal<bool>,
    interaction_number: usize,
    #[prop(into)] onclick: Callback<()>,
) -> impl IntoView {
    let node_ref = NodeRef::new();

    Effect::new(move |_| {
        if is_active.get() {
            if let Some(element) = node_ref.get() {
                let element: HtmlLiElement = element;
                let scroll_options = ScrollIntoViewOptions::new();
                scroll_options.set_behavior(ScrollBehavior::Smooth);
                element.scroll_into_view_with_scroll_into_view_options(&scroll_options);
            }
        }
    });

    let icon_class = move || {
        if is_active.get() {
            "prompt-icon fas fa-chevron-up"
        } else {
            "prompt-icon fas fa-chevron-down"
        }
    };

    let prompt_item_class = move || {
        if is_active.get() {
            "prompt-item active"
        } else {
            "prompt-item"
        }
    };

    let onclick_header = move |_| {
        if let Some(element) = node_ref.get() {
            let element: HtmlLiElement = element;
            let header_rect = element.get_bounding_client_rect();
            let header_top = header_rect.top();
            if header_top <= 0.0 {
                let scroll_options = ScrollIntoViewOptions::new();
                scroll_options.set_behavior(ScrollBehavior::Smooth);
                element.scroll_into_view_with_scroll_into_view_options(&scroll_options);
            } else {
                onclick.run(());
            }
        }
    };

    let request_messages = request.as_ref().map(parse_chat_request);
    let response_messages = response.as_ref().map(parse_chat_response);

    let body = move || {
        view! {
            <div class="prompt-content">
                <div class="request-section">
                    <b>"Request"</b>
                    {
                        match &request_messages {
                            Some(Ok(messages)) => {
                                if messages.is_empty() {
                                    view! { <p>"No request messages available."</p> }.into_any()
                                } else {
                                    view! {
                                        <ul class="message-list">
                                        {messages.iter().map(|message| view! { <MessageItem message=message.clone()/> }).collect::<Vec<_>>()}
                                        </ul>
                                    }.into_any()
                                }
                            },
                            Some(Err(err)) => {
                                view! {
                                    <p class="error">{format!("Error parsing request: {}", err)}</p>
                                }.into_any()
                            },
                            None => {
                                view! {
                                    <p>"Request unavailable."</p>
                                }.into_any()
                            },
                        }
                    }
                </div>
                <div class="response-section">
                    <b>"Response"</b>
                    {
                        match &response_messages {
                            Some(Ok(messages)) => {
                                if messages.is_empty() {
                                    view! { <p>"No response messages available."</p> }.into_any()
                                } else {
                                    view! {
                                        <ul class="message-list">
                                            {messages.iter().map(|message| {
                                                view! { <MessageItem message=message.clone()/> }
                                            }).collect::<Vec<_>>()}
                                        </ul>
                                    }.into_any()
                                }
                            },
                            Some(Err(err)) => {
                                view! { <p class="error">{format!("Error parsing response: {}", err)}</p> }.into_any()
                            },
                            None => {
                                view! { <p>"Response unavailable."</p> }.into_any()
                            },
                        }
                    }
                </div>
            </div>
        }
    };

    view! {
        <li class=prompt_item_class node_ref=node_ref>
            <div class="prompt-header" on:click=onclick_header>
                {format!("Prompt #{}", interaction_number)}
                <i class=icon_class></i>
            </div>
            {move || if is_active.get() { body.clone().into_any() } else { view! { <></> };
            ().into_any() }}
        </li>
    }
}
