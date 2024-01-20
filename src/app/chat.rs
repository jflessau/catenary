use super::{Inbox, Titlebar};
use crate::api::*;
use crate::state::*;
use geo::Point;
use leptos::*;
use leptos_dom::helpers::IntervalHandle;
use leptos_router::A;
use leptos_use::{use_geolocation_with_options, UseGeolocationOptions, UseGeolocationReturn};
use std::time::Duration;

#[component]
pub fn View() -> impl IntoView {
    let (location, set_location) = create_signal(None as Option<Point<f64>>);
    let location_history = LocationHistory::new();
    let (trace, set_trace) = create_signal(location_history.clone().trace());
    let (location_history, set_location_history) = create_signal(location_history);

    create_effect(move |_| {
        let Some(location) = location.get() else {
            return;
        };
        let mut location_history = location_history.get_untracked();
        location_history.add_location(location);
        let new_trace = location_history.trace();
        set_location_history(location_history);
        set_trace(new_trace);
    });

    create_effect(move |prev_handle: Option<IntervalHandle>| {
        if let Some(prev_handle) = prev_handle {
            log::info!("clearing geolocation effect");
            prev_handle.clear();
        };

        log::info!("run geolocation effect");

        let UseGeolocationReturn { coords, error, .. } = use_geolocation_with_options(
            UseGeolocationOptions::default().enable_high_accuracy(true),
        );

        let locate = move || {
            if coords.try_get_untracked().is_none() || location.try_get_untracked().is_none() {
                log::info!("no coords yet");
                return;
            }
            if let Some(coords) = coords.get() {
                set_location(Some(Point::new(coords.longitude(), coords.latitude())));
                return;
            }
            if let Some(error) = error.get() {
                log::warn!("no coords, error: {:?}", error);
                set_trace(Err(NoTrace::from(error)));
            };
        };

        set_interval_with_handle(locate, Duration::from_millis(1000))
            .expect("could not create interval")
    });

    view! {
        <Titlebar current_page="chat"/>
        <div class="main-container">
            <div class="main fullscreen">
                <NoTrace trace/>
                <Chat trace/>
            </div>
        </div>
    }
}

#[component]
fn NoTrace(trace: ReadSignal<Result<Trace, NoTrace>>) -> impl IntoView {
    let text = move || {
        match trace() {
            Err(NoTrace::NoPermission) => view! {
                <div class="inner">
                    <img src="/location-no-permission.svg" alt="Map pin with a locked lock in it"></img>
                    <p>
                        "Please allow location access and reload the page."
                        <br/>
                        <br/>
                        See the <A href="/faq">"FAQ"</A> for more information.
                    </p>
                </div>
            }.into_view(),
            Err(NoTrace::PositionUnavailable) | Err(NoTrace::Timeout) => view! {
                <div class="inner">
                    <img src="/location-unavailable.svg" alt="Prohibition sign with a crossed out arrow"></img>
                    <p>"Failed to locate you. Please try again in a few moments by refreshing the page."</p>
                </div>
            }.into_view(),
            Err(NoTrace::WaitingForMoreLocations {
                received_locations,
                required_locations,
            }) => {
                let percentage = if received_locations > 0 && required_locations > 0 {
                    (received_locations as f64 / required_locations as f64) * 100.0
                } else {
                    2.0
                };
                view! {
                    <div class="inner">
                        <div class="progressbar">
                            <div class="progress" style={format!("width: {}%;", percentage)}></div>
                        </div>
                        <p>
                            "Matching you with other users..."
                        </p>
                    </div>
                }.into_view()
            }
            Err(NoTrace::WaitingForTimeToPass) => view! {
                <div class="inner">
                    <div class="loading-container">
                        <span class="loader"></span>
                    </div>
                    <p>"Calculating your speed. Please wait."</p>
                </div>
            }.into_view(),
            Err(NoTrace::TooSlow {
                current_speed,
                required_speed,
            }) => view! {
                <div class="inner">
                    <img src="/location-too-slow.svg" alt="Speedometer showing a low speed"></img>
                    <p>
                        {format!("You are moving at {current_speed:.1} meters per second.")}
                        <br/><br/>
                        {format!("{required_speed:.1} meters per second is the minimum speed required to match you with other users.")}
                        <br/>
                        <br/>
                        See the <A href="/faq">"FAQ"</A> for more information.
                    </p>
                </div>
            }.into_view(),
            Ok(_) => view! { <div></div> }.into_view(),
        }
    };

    view! {
        <Show
            when=move || trace.get().is_err()
            fallback=move || view! {}
        >
            <div class="no-trace">
                {text()}
            </div>
        </Show>
    }
    .into_view()
}

#[component]
fn Chat(trace: ReadSignal<Result<Trace, NoTrace>>) -> impl IntoView {
    let inbox = use_context::<RwSignal<Inbox>>().expect("no inbox context");
    log::info!("inbox: {:?}", inbox.get_untracked().messages.len());
    let (load_messages, set_load_messages) = create_signal(false);

    let loader = create_resource(load_messages, move |load_messages| async move {
        if load_messages {
            set_load_messages(false);
            let Ok(trace) = trace.get_untracked() else {
                println!("no trace for loading messages, this shouldn't happen");
                return "".to_owned();
            };
            println!("loading messages");
            list_messages(trace)
                .await
                .expect("couldn't list message")
                .into_iter()
                .for_each(|msg| {
                    let mut inbox_updated = inbox.get_untracked();
                    inbox_updated.push(msg.clone());
                    inbox.set(inbox_updated);
                });
        }
        "".to_owned()
    });

    create_effect(move |prev_handle: Option<IntervalHandle>| {
        if let Some(prev_handle) = prev_handle {
            prev_handle.clear();
        };

        set_interval_with_handle(
            move || {
                if load_messages.try_get_untracked().is_none() {
                    return;
                }
                spawn_local(async move {
                    set_load_messages(true);
                });
            },
            Duration::from_millis(500),
        )
        .expect("could not create interval")
    });

    view! {
        <Transition fallback=move || view! {
            <div class="loading-container">
                <span class="loader"></span>
            </div>
        }>
            {move || loader.get()}
            <Messages inbox set_load_messages trace/>
            <SendForm set_load_messages trace />
        </Transition>
    }
}

#[component]
fn SendForm(
    set_load_messages: WriteSignal<bool>,
    trace: ReadSignal<Result<Trace, NoTrace>>,
) -> impl IntoView {
    let (msg, set_msg) = create_signal("".to_string());
    let (sending, set_sending) = create_signal(false);

    let send_button_props = move || match (sending.get(), msg.get().is_empty()) {
        (true, _) => ("Sending", "clickable disabled"),
        (_, true) => ("Send", "clickable disabled"),
        _ => ("Send", "clickable"),
    };

    view! {
        <Show
            when=move || trace.get().is_ok()
            fallback=move || view! {<div></div>}
        >
            <div class="send-form">
                <textarea
                    placeholder="Type a message..."
                    type="text"
                    maxlength="144"
                    on:input=move |ev| {
                        set_msg(event_target_value(&ev));
                    }
                    prop:value={msg}
                />
                <button class={move || send_button_props().1}
                    on:click=move |_| {
                        if sending.get() {
                            return;
                        }
                        spawn_local(async move {
                            let Ok(trace) = trace.get_untracked() else {
                                log::error!("no trace for sending message, this shouldn't happen");
                                return;
                            };
                            set_sending(true);
                            let msg_text = msg.get_untracked();
                            set_msg("".to_string());
                            send_message(msg_text, trace)
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
        </Show>
    }
}

#[component]
fn Messages(
    inbox: RwSignal<Inbox>,
    set_load_messages: WriteSignal<bool>,
    trace: ReadSignal<Result<Trace, NoTrace>>,
) -> impl IntoView {
    view! {
        <Show
            when=move || trace.get().is_ok()
            fallback=move || view! {<div></div>}
        >
            <div class="messages">
                <For
                    each={move || inbox.get().messages}
                    key=|message| format!("{}-{:?}-{}-{}", message.id, message.vote, message.upvoters, message.downvoters)
                    children=move |msg| {
                        view! {
                            <Message msg set_load_messages/>
                        }
                    }
                />
            </div>
        </Show>
    }
}

#[component]
fn Message(msg: ChatMessageOut, set_load_messages: WriteSignal<bool>) -> impl IntoView {
    let timestamp = msg.timestamp.format("%H:%M").to_string();

    let bubble_style = if msg.downvoters == 0 {
        "opacity: 1.0;".to_owned()
    } else if msg.downvoters >= 8 {
        "opacity: 0.2;".to_owned()
    } else {
        format!("opacity: 0.{};", 10 - msg.downvoters)
    };

    let text_classes = if msg.upvoters >= 8 {
        "text scale-8".to_owned()
    } else if msg.upvoters == 0 {
        "text".to_owned()
    } else {
        format!("text scale-{}", msg.upvoters)
    };

    view! {
        <div class="message message-in">
            <p class="author">{msg.username}</p>
            <div class="content">
                <div class="bubble" style={bubble_style}>
                    <p class=text_classes>{msg.text}</p>
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
