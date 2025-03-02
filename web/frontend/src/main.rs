use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::hooks::use_params_map;
use leptos_router_macro::path;

mod api;
mod auth;
mod components;
mod errors;
mod pages;
mod routes;
mod whitelabel;

use components::*;
use errors::ErrorModal;
use pages::*;

#[component]
pub fn App() -> impl IntoView {
    let error_store = RwSignal::new(errors::ErrorStore::default());
    provide_context(error_store);

    view! {
        <>
            <ErrorModal/>
            <Router>
                <Navbar/>
                <Routes fallback=|| view! { <div>"Not Found"</div> }>
                    <Route
                        path=path!("/")
                        view=|| view! { <LandingPage/> }
                    />
                    <Route
                        path=path!("/login")
                        view=|| view! { <LoginPage/> }
                    />
                    <Route
                        path=path!("logout")
                        view=|| view! { <LogoutPage/> }
                    />
                    <Route
                        path=path!("repos")
                        view=|| view! {
                            <WithUser>
                                <ReposPage/>
                            </WithUser>
                        }
                    />
                    <Route
                        path=path!("repos/:id")
                        view=move || {
                            let params = use_params_map();
                            move || {
                                let id = params.get().get("id").unwrap_or_default();
                                view! {
                                    <WithUser>
                                        <RepoPage id=id.clone() />
                                    </WithUser>
                                }
                            }
                        }
                    />
                    <Route
                        path=path!("settings")
                        view=|| view! { <WithUser><SettingsPage/></WithUser> }
                    />
                    <Route
                        path=path!("waitlist")
                        view=|| view! { <WithUser><WaitlistPage/></WithUser> }
                    />
                    <Route
                        path=path!("tasks")
                        view=|| view! {
                            <WithUser>
                                <TasksPage/>
                            </WithUser>
                        }
                    />
                    <Route
                        path=path!("tasks/:id")
                        view=move || {
                            let params = use_params_map();
                            move || {
                                let id = params.get().get("id").unwrap_or_default();
                                view! {
                                    <WithUser>
                                        <TaskPage id=id.clone() />
                                    </WithUser>
                                }
                            }
                        }
                    />
                    <Route
                        path=path!("privacy")
                        view=|| view! { <PrivacyPage/> }
                    />
                    <Route
                        path=path!("legal-notice")
                        view=|| view! { <LegalNoticePage/> }
                    />
                </Routes>
                <Footer/>
            </Router>
        </>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    mount_to_body(|| view! { <App/> })
}
