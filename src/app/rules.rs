use super::Titlebar;
use crate::app::faq::FAQ;
use leptos::*;
use std::{collections::HashSet, fs, io::Read};

#[component]
pub fn View() -> impl IntoView {
    let q_and_a = vec![
        (
            "Be nice!".to_string(),
            r#"Catenary is a chat application for people on the same bus, train, boat or other means of public transport.

            Using the geolocation data from your device, Catenary will automatically connect you to
            other people with a similar location, speed and direction of travel."#.to_string(),
        ),
    ];

    let mut opened = HashSet::<String>::new();
    q_and_a.first().map(|(q, _)| opened.insert(q.clone()));
    let (opened, set_opened) = create_signal(opened);

    view! {
        <Titlebar go_back_link=Some("/")/>
        <div class="main-container">
            <FAQ title="Rules" q_and_a/>
        </div>
    }
}
