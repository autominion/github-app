use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::routes::paths;

#[component]
pub fn LandingPage() -> impl IntoView {
    let navigate = use_navigate();
    let to_get_access = move |_| navigate(paths::WAITLIST, Default::default());

    let header_message = view! {
        <>
            <h1 class="hero-title">"Open Source Agent Runtime"</h1>
            <p class="hero-subtitle">
                "We are building an open-source runtime that connects software engineering tools with AI agents."<br/>
                "It's still early stages - would you like to shape this project with us?"
            </p>
            <div class="buttons-center">
                <a class="button light" href="https://github.com/autominion" target="_blank">"Contribute"</a>
            </div>
        </>
    };

    view! {
        <>
            <div class="hero">
                <div class="hero-bg"></div>
                <div class="hero-content">
                    {header_message}
                </div>
            </div>
            <div class="main-container">
                <div class="content-wrapper">
                    <div class="content">
                        <section class="section light">
                            <h1 class="section-title">"Bring your agents, bring your tools"</h1>
                            <div class="section-body">
                                <div class="section-text">
                                    <p>
                                        "autominion provides a common interface to connect different AI agents and software engineering tools. Rather than entangling agentic code with a particular user interface, we decouple the two. Especially in the current landscape of rapidly evolving tools and agentic systems, it is an advantage to be able to switch out either."
                                    </p>
                                    <p class="section-text">
                                    "Sounds exciting? Join us on GitHub and let's shape this project together!"
                                    </p>
                                    <div class="buttons-row">
                                        <a href="https://github.com/autominion" target="_blank" class="button light">"Contribute"</a>
                                    </div>
                                </div>
                                <img class="section-img img-card" src="/img/common-interface.svg" alt="Common Interface" />
                            </div>
                        </section>

                        <section class="section dark card">
                            <h1 class="section-title">"Pragmatic Approach"</h1>
                            <p class="section-text">
                                "We aim to achieve all of this with a minimal amount of reinventing the wheel. We use existing protocols and standards where possible."
                            </p>
                            <h2 class="section-subtitle">"What does this mean for agents?"</h2>
                            <ul class="feature-list">
                                <li>"Agents are arbitrary programs written in any language, distributed as OCI container images."</li>
                                <li>"They access an OpenAI-compatible LLM API proxy, forwarding to a configured provider (e.g. OpenRouter)."</li>
                                <li>"They access a Git proxy that restricts pushing to selected repositories and branches."</li>
                                <li>"They access a minimal additional HTTP API to retrieve their task and interact with the user."</li>
                            </ul>
                            <p class="section-text">
                                "Agents can optionally use our open source " <code>"minion"</code> " library that implements the API specification, or use any custom implementation. The library is written in Rust which means that wrappers for other languages can be easily created."
                            </p>
                            <h2 class="section-subtitle">"What does this mean for tools?"</h2>
                            <p class="section-text">
                                "There is no daemon or separate application that tools have to integrate with. Tools can include "<code>"autominion"</code>", our open source library and have full control on how they expose the functionality to the user. The library is written in Rust which means that wrappers for other languages can be easily created. Alternatively, tools can also implement the API specification independently."
                            </p>
                        </section>
                        <section class="section light">
                            <h1 class="section-title">"Strict Isolation"</h1>
                            <p class="section-text">
                                "We think it is important to securely isolate agentic systems and their capabilities to interact with the outside world. We use containerization to isolate agents from the host system. LLM APIs, git and other tools that agents interact with have to pass through the autominion proxy."
                            </p>
                        </section>
                        <section class="section dark card">
                            <h1 class="section-title">"GitHub Integration"</h1>
                            <div class="section-body reverse">
                                <img class="section-img img-card" src="/img/github-comment.svg" alt="GitHub Integration" />
                                <p class="section-text">
                                    "To build a good game engine, you should build a game! Eating our own "<span style="text-decoration: line-through">"dogfood"</span>" cake, we developed autominion alongside our first application: An open source GitHub integration that lets you interact with any open source agent from your GitHub repositories. The integration is fully open source and self-hostable."
                                </p>
                            </div>
                            <p class="section-text">
                                "Would you be interested in using a hosted instance of the GitHub integration? If there is enough community support, we will host a public instance."
                            </p>
                            <p class="section-text">
                                "Clicking " <b>"Register Interest"</b> " means you agree to our " <a href=paths::PRIVACY>"Privacy Policy"</a> "."
                            </p>
                            <div class="buttons-row">
                                <button class="button light" on:click=to_get_access>"Register Interest"</button>
                            </div>
                        </section>
                        <section class="section light">
                            <h1 class="section-title"> "Everything is Open"</h1>
                            <p class="section-text">
                                "Last, but certainly not least, everything here is open source! "
                                "We want a future where developer tools remain accessible to all developers. "
                                "Everyone should be empowered to build great software, with full sovereignty in the process. "
                                "If you share this vision, we would love to have you on board!"
                            </p>
                            <div class="buttons-row">
                                <a href="https://github.com/autominion" target="_blank" class="button light">"Contribute"</a>
                            </div>
                        </section>
                    </div>
                </div>
            </div>
            <div class="landing-footer">
                <div class="footer-bg"></div>
                <div class="footer-content">
                    <h1 class="footer-title">"What will you build?"</h1>
                </div>
            </div>
        </>
    }
}
