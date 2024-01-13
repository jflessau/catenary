use crate::api::*;
use crate::error_template::{AppError, ErrorTemplate};
use crate::state::*;
use geo::Point;
use leptos::{leptos_dom::helpers::IntervalHandle, *};
use leptos_meta::*;
use leptos_router::*;
use leptos_use::{
    use_geolocation, use_geolocation_with_options, use_window, UseGeolocationOptions,
    UseGeolocationReturn,
};
use std::time::Duration;

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
                    <Route path="" view=MainView/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn MainView() -> impl IntoView {
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
        log::info!("run geolocation effect");
        if let Some(prev_handle) = prev_handle {
            log::info!("clearing previous interval in geolocation effect");
            prev_handle.clear();
        };

        let UseGeolocationReturn { coords, error, .. } = use_geolocation();

        let locate = move || {
            if let Some(coords) = coords.get() {
                set_location(Some(Point::new(coords.longitude(), coords.latitude())));
            } else {
                log::info!("no coords, error: {:?}", error.get());
            };
        };

        set_interval_with_handle(locate, Duration::from_millis(1000))
            .expect("could not create interval")
    });

    view! {
        <Titlebar/>
        <div class="main-container">
            <NoTrace trace/>
            <Chat trace/>
        </div>
    }
}

#[component]
fn NoTrace(trace: ReadSignal<Result<Trace, NoTrace>>) -> impl IntoView {
    if trace.get().is_ok() {
        return view! {
            <div></div>
        }
        .into_view();
    }

    let text = move || {
        match trace() {
        Err(NoTrace::NoPermission) => view! {
            <p class="error">"Please allow location access and reload the page."</p>
        }.into_view(),
        Err(NoTrace::PositionUnavailable) | Err(NoTrace::Timeout) => view! {
            <p class="error">"Catenary needs your location to work. Please allow location access and reload the page."</p>
        }.into_view(),
        Err(NoTrace::WaitingForMoreLocations {
            received_locations,
            required_locations,
        }) => {
            let percentage = if received_locations > 0 && required_locations > 0 {
                (received_locations as f64 / required_locations as f64) * 100.0
            } else {
                0.0
            };
            view! {
                <p class="wait">
                    "More location data is needed to match you with other users."
                    <br/>
                    "Please wait."
                </p>
            }.into_view()
        }
        Err(NoTrace::WaitingForTimeToPass) => view! {
            <p class="wait">
                "More location data is needed to match you with other users."
                <br/>
                "Please wait."
            </p>
        }.into_view(),
        Err(NoTrace::TooSlow {
            current_speed,
            required_speed,
        }) => view! {
            <p class="info">
                {"Your speed is too slow to match you with other users."}
                <br/>
                {"This site groups users on the same bus, train, etc. into the same chat room."}
                <br/>
                {format!("Therefore, you need to be moving at least {required_speed} meters per second.")}
                <br/>
                {format!("Your current speed is {current_speed} meters per second.")}
            </p>
        }.into_view(),
        Ok(_) => view! { <div></div> }.into_view(),
    }
    };

    view! {
        <div class="no-trace">
            {text()}
        </div>
    }
    .into_view()
}

#[component]
fn Chat(trace: ReadSignal<Result<Trace, NoTrace>>) -> impl IntoView {
    let (messages, set_messages) = create_signal(vec![]);
    let (load_messages, set_load_messages) = create_signal(false);

    let loader = create_resource(load_messages, move |load_messages| async move {
        if load_messages {
            set_load_messages(false);
            set_messages(list_messages().await.expect("couldn't list message"));
        }
        return "".to_owned();
    });

    create_effect(move |_| {
        set_interval_with_handle(
            move || {
                spawn_local(async move {
                    set_load_messages(true);
                });
            },
            Duration::from_millis(500),
        )
        .expect("could not create interval");
    });

    view! {
        <Transition fallback=move || view! {
            <div class="loading-container">
                <span class="loader"></span>
            </div>
        }>
            {move || loader.get()}
            <Messages messages set_load_messages/>
            <BottomBar set_load_messages trace />
        </Transition>
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
fn BottomBar(
    set_load_messages: WriteSignal<bool>,
    trace: ReadSignal<Result<Trace, NoTrace>>,
) -> impl IntoView {
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
                        let Ok(trace) = trace.get() else {
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
