use crate::api::*;
use crate::error_template::{AppError, ErrorTemplate};
use crate::state::{ChatMessageOut, Vote};
use leptos::{leptos_dom::helpers::IntervalHandle, *};
use leptos_meta::*;
use leptos_router::*;
use std::time::Duration;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
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
                    <Route path="" view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let (messages, set_messages) = create_signal(vec![]);
    let (load_messages, set_load_messages) = create_signal(false);

    let loader = create_resource(load_messages, move |load_messages| async move {
        if load_messages {
            set_load_messages(false);
            set_messages(list_messages().await.expect("couldn't list message"));
        }
        return "".to_owned();
    });

    use_interval(1000, move || {
        spawn_local(async move {
            set_load_messages(true);
        });
    });

    view! {
        <Titlebar/>
        <MainContainer>
            <Transition fallback=move || view! {
                <div class="loading-container">
                    <span class="loader"></span>
                </div>
            }>
                {move || loader.get()}
                <Messages messages set_load_messages/>
            </Transition>
            <BottomBar set_load_messages/>
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
fn BottomBar(set_load_messages: WriteSignal<bool>) -> impl IntoView {
    let (msg, set_msg) = create_signal("".to_string());
    let (sending, set_sending) = create_signal(false);

    let send_button_props = move || match (sending.get(), msg.get().is_empty()) {
        (true, _) => ("Sending...", "clickable disabled"),
        (_, true) => ("Send", "clickable disabled"),
        _ => ("Send", "clickable"),
    };

    view! {
        <div class="bottom-bar">
            <textarea
                placeholder="Type a message..."
                type="text"
                maxlength="144"
                on:input=move |ev| {
                    set_msg(event_target_value(&ev));
                }
                prop:value={msg.clone()}
            />
            <button class={move || send_button_props().1}
                on:click=move |_| {
                    if sending.get() {
                        return;
                    }
                    spawn_local(async move {
                        set_sending(true);
                        let msg_text = msg.get_untracked();
                        set_msg("".to_string());
                        send_message(msg_text)
                            .await
                            .expect("couldn't send message");
                        set_load_messages(true);
                        set_sending(false);
                    });
                }
            >
                {move || send_button_props().0}
            </button>
        </div>
    }
}

#[component]
fn Messages(
    messages: ReadSignal<Vec<ChatMessageOut>>,
    set_load_messages: WriteSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="messages">
            <For
                each=messages
                key=|message| format!("{}-{:?}-{}-{}", message.id, message.vote, message.upvoters, message.downvoters)
                children=move |msg| {
                    view! {
                        <Message msg set_load_messages/>
                    }
                }
            />
        </div>
    }
}

#[component]
fn Message(msg: ChatMessageOut, set_load_messages: WriteSignal<bool>) -> impl IntoView {
    let timestamp = msg.timestamp.format("%H:%M").to_string();

    let opacity = 10_usize.saturating_sub(msg.downvoters);
    let bubble_style = if opacity >= 10 {
        "opacity: 1.0;".to_owned()
    } else if opacity <= 2 {
        "opacity: 0.2;".to_owned()
    } else {
        format!("opacity: 0.{};", opacity)
    };

    view! {
        <div class="message message-in">
            <p class="author">{msg.username}</p>
            <div class="content">
                <div class="bubble" style={bubble_style}>
                    <p class="text">{msg.text}</p>
                    <p class="time">{timestamp}</p>
                </div>
                <div class="votes">
                    <img
                        src="/arrow.svg"
                        alt="upvote"
                        class={
                            if msg.vote == Some(Vote::Up) {
                                "on".to_string()
                            } else {
                                "".to_string()
                            }
                        }
                        on:click=move |_| {
                            spawn_local(async move {
                                vote_message(msg.id, true).await.expect("couldn't send message");
                                set_load_messages(true);
                            });
                        }
                    />
                    <img
                        src="/arrow.svg"
                        alt="downvote"
                        class={move || {
                            if msg.vote == Some(Vote::Down) {
                                "on".to_string()
                            } else {
                                "".to_string()
                            }
                        }}
                        on:click=move |_| {
                            spawn_local(async move {
                                vote_message(msg.id, false).await.expect("couldn't send message");
                                set_load_messages(true);
                            });
                        }
                    />
                </div>
            </div>
        </div>
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
