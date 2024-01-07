use crate::error_template::{AppError, ErrorTemplate};
use crate::state::ChatMessage;
use std::time::Duration;
use leptos::{*, leptos_dom::helpers::IntervalHandle};
use leptos_meta::*;
use leptos_router::*;
use crate::api::*;

#[component]
pub fn App() -> impl IntoView { 
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/catenary.css"/>
        <meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1" />

        // sets the document title
        <Title text="Catenary - chat far and wide!"/>

        // content for this welcome page
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
                    <Route path="" view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {

    let (messages, set_messages) = create_signal(vec![]);

    use_interval(500, move || {
        log::info!("interval");
        spawn_local(async move {
            let messages = list_messages().await.expect("couldn't get messages");
            set_messages(messages);
        });
    });

    view! {
        <Titlebar/>
        <MainContainer>
            <div class="messages">
                <Messages messages=messages/>
            </div>
            <MessageForm/>
        </MainContainer>
    }
}

#[component]
fn Titlebar() -> impl IntoView {
    view! {
        <div class="titlebar">
            <h1><span>"🚃"</span>"Catenary"</h1>
        </div>
    }
}

#[component]
pub fn MainContainer(children: Children) -> impl IntoView {
    view! {
        <div class="main-container">
            {children()}
        </div>
    }
}

#[component]
fn MessageForm() -> impl IntoView {
    let (msg, set_msg) = create_signal("".to_string());

    view! {
        <div class="bottom-bar">
            <textarea
                placeholder="Type a message..."
                type="text"
                maxlength="144"
                on:input=move |ev| {
                    set_msg(event_target_value(&ev));
                }
                on:keydown=move |ev| {
                    if ev.key() == "Enter" {
                        spawn_local(async move {
                            send_message(msg.get_untracked()).await.expect("couldn't add todo");
                            set_msg("".to_string());
                        });
                    }
                }
                prop:value={msg}
            />
            <button class="clickable" 
                on:click=move |_| {
                    spawn_local(async move {
                        send_message(msg.get_untracked()).await.expect("couldn't send message");
                        set_msg("".to_string());
                    });
                }
            >
                "Send"
            </button>
        </div>
    }
}

#[component]
fn Message(
    text: String, 
    author: String, 
    time: String
) -> impl IntoView {
    view! {
        <div class="message message-in">
            <p class="author">{author}</p>
            <div class="bubble">
                <p class="text">{text}</p>
                <p class="time">{time}</p>
            </div>
        </div>
    }
}

#[component]
fn Messages(messages: ReadSignal<Vec<ChatMessage>>) -> impl IntoView {
    view! {
        <For
            each=messages
            key=|message| message.id
            children=move |msg| {
                let time = msg.timestamp.format("%H:%M").to_string();
                view! {
                    <Message 
                        author=msg.author 
                        text=msg.text
                        time=time
                    />
                }
            }
        />
    }
}

pub fn use_interval<T, F>(interval_millis: T, f: F)
where
    F: Fn() + Clone + 'static,
    T: Into<MaybeSignal<u64>> + 'static,
{
    let interval_millis = interval_millis.into();
    create_effect(move |prev_handle: Option<IntervalHandle>| {
        // effects get their previous return value as an argument
        // each time the effect runs, it will return the interval handle
        // so if we have a previous one, we cancel it
        if let Some(prev_handle) = prev_handle {
            prev_handle.clear();
        };

        // here, we return the handle
        set_interval_with_handle(
            f.clone(),
            // this is the only reactive access, so this effect will only
            // re-run when the interval changes
            Duration::from_millis(interval_millis.get()),
        )
        .expect("could not create interval")
    });
}