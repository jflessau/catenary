mod chat;
mod faq;
mod home;
mod rules;

use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="Catenary - chat far and wide!"/>
        <Stylesheet id="leptos" href="/pkg/catenary.css"/>
        <meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1" />

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
                    <Route path="/chat" view=chat::View/>
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
        ("Home".to_string(), "/".to_string()),
        ("Chat".to_string(), "/chat".to_string()),
        ("FAQ".to_string(), "/faq".to_string()),
        ("Rules".to_string(), "/rules".to_string()),
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
                        view! {
                            <A href=href class="item">
                                <p class="text">
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
        <div class="bar">
            <div class="row">
                <div class=burger_menu_classes on:click=move |_| set_open(!open.get())>
                    <div/>
                    <div/>
                    <div/>
                </div>
                <p class="title"><A href="/">"Catenary"</A></p>
            </div>
        </div>
    }
}
