mod chat;
mod faq;
mod footer;
mod home;
mod rules;

use crate::{
    error_template::{AppError, ErrorTemplate},
    state::ChatMessageOut,
};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct Inbox {
    pub messages: VecDeque<ChatMessageOut>,
}

impl Default for Inbox {
    fn default() -> Self {
        Self {
            messages: VecDeque::with_capacity(1000),
        }
    }
}

impl Inbox {
    pub fn push(&mut self, m: ChatMessageOut) {
        if !self.messages.iter().any(|x| x.id == m.id) {
            self.messages.push_front(m);
        }
        if self.messages.len() > 999 {
            self.messages.pop_back();
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_context(create_rw_signal(Inbox::default()));

    view! {
        <Title text="Catenary - chat far and wide!"/>
        <Stylesheet id="leptos" href="/pkg/catenary.css"/>
        <meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1" />

        <link rel="apple-touch-icon" sizes="180x180" href="/favicon/apple-touch-icon.png"/>
        <link rel="icon" type="image/png" sizes="32x32" href="/favicon/favicon-32x32.png"/>
        <link rel="icon" type="image/png" sizes="16x16" href="/favicon/favicon-16x16.png"/>
        <link rel="manifest" href="/favicon/site.webmanifest"/>
        <link rel="mask-icon" href="/favicon/safari-pinned-tab.svg" color="#ff8811"/>
        <meta name="msapplication-TileColor" content="#392f5a"/>
        <meta name="theme-color" content="#f4d06f"/>

        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
            .into_view()
        }>
            <main>
                <Routes>
                    <Route path="" view=home::View/>
                    <Route path="/chat" view=chat::View />
                    <Route path="/faq" view=faq::View/>
                    <Route path="/rules" view=rules::View/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
pub fn Titlebar(current_page: &'static str) -> impl IntoView {
    let links = vec![
        ("Home".to_string(), "/".to_string(), false, false),
        ("Chat".to_string(), "/chat".to_string(), false, false),
        ("FAQ".to_string(), "/faq".to_string(), false, false),
        ("Rules".to_string(), "/rules".to_string(), false, false),
        (
            "Bug report".to_string(),
            "https://github.com/jflessau/catenary/issues".to_string(),
            true,
            false,
        ),
        (
            "GitHub".to_string(),
            "https://github.com/jflessau/catenary".to_string(),
            true,
            false,
        ),
        (
            "Legal Notice".to_string(),
            "https://jflessau.com/info/legal-notice/".to_string(),
            true,
            true,
        ),
        (
            "Privacy Policy".to_string(),
            "https://jflessau.com/info/privacy-policy/".to_string(),
            true,
            true,
        ),
    ];
    let (links, _) = create_signal(links);
    let (open, set_open) = create_signal(false);

    let menu_style = move || {
        if open.get() {
            "display: flex;".to_string()
        } else {
            "display: none;".to_string()
        }
    };

    view! {
        <div class="titlebar">
            <Bar open set_open/>
            <div class="menu" style=menu_style>
                <Bar open set_open/>
                <For
                    each=links
                    key=|item| item.0.clone()
                    children=move |item| {
                        let text = item.clone().0;
                        let href = item.clone().1;
                        let external = item.clone().2;
                        let text_classes = move || {
                            if item.clone().3 {
                                "text legal"
                            } else {
                                "text"
                            }
                        };
                        if external {
                            view! {
                                <a href=href class="item">
                                    <p class=text_classes>
                                        {text.clone()}
                                    </p>
                                </a>
                            }.into_view()
                        } else {
                            view! {
                                <A href=href class="item">
                                    <p class=text_classes>
                                        {text.clone()}
                                        <span>
                                            {
                                                if current_page.to_lowercase() == text.to_lowercase() {
                                                    " <- you are here"
                                                } else {
                                                    ""
                                                }
                                            }
                                        </span>
                                    </p>
                                </A>
                            }.into_view()
                        }
                    }
                />
            </div>
        </div>
    }
}

#[component]
pub fn Bar(open: ReadSignal<bool>, set_open: WriteSignal<bool>) -> impl IntoView {
    let burger_menu_classes = move || {
        if open.get() {
            "burger-menu-button open"
        } else {
            "burger-menu-button"
        }
    };

    view! {
        <div class="bar-container">
            <div class="bar">
                <button class=burger_menu_classes on:click=move |_| set_open(!open.get())>
                    <div/>
                    <div/>
                    <div/>
                </button>
                <p class="title"><A href="/">"Catenary"</A></p>
            </div>
        </div>
    }
}
