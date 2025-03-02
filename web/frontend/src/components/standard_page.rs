use leptos::prelude::*;

use crate::components::Loader;

#[component]
pub fn StandardPage(#[prop(into)] children: ChildrenFn) -> impl IntoView {
    view! {
        <div class="content-container">
            <div class="content">
                <section>
                    <Loader>
                        {children()}
                    </Loader>
                </section>
            </div>
        </div>
    }
}
