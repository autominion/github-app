use leptos::prelude::*;

use user_api::LLMInteraction;

use super::interaction_item::InteractionItem;

#[component]
pub fn LlmInteractions(
    interactions: Vec<LLMInteraction>,
    #[prop(into)] active_interaction_id: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <>
            <p>"LLM interactions of this agent run."</p>
            <ul class="prompts-list">
                <For
                    each=move || {
                        interactions
                            .clone()
                            .into_iter()
                            .enumerate()
                            .map(|(index, interaction)| (interaction, index))
                            .collect::<Vec<_>>()
                    }
                    key=|(interaction, _)| interaction.id.clone()
                    children=move |(interaction, index)| {
                        let id = interaction.id.clone();
                        let is_active = Signal::derive({
                            let id = id.clone();
                            move || active_interaction_id.get().as_ref() == Some(&id)
                        });
                        let onclick = move || {
                            if active_interaction_id.get().as_ref() == Some(&id) {
                                active_interaction_id.set(None);
                            } else {
                                active_interaction_id.set(Some(id.clone()));
                            }
                        };
                        view! {
                            <InteractionItem
                                is_active
                                onclick={onclick}
                                request={interaction.request.clone()}
                                response={interaction.response.clone()}
                                interaction_number={index + 1}
                            />
                        }
                    }
                />
            </ul>
        </>
    }
}
