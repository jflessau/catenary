use super::footer::Footer;
use super::Titlebar;
use leptos::*;
use std::collections::HashSet;

#[component]
pub fn View() -> impl IntoView {
    let q_and_a = vec![
        (
            "What is Catenary?".to_string(),
            r#"Catenary is a chat application for people on the same bus, train, boat or other means of public transport.

            Using the geolocation data from your device, Catenary will automatically put you in an anonymous chatroom with other people, who have a similar location, speed and direction of travel as you."#.to_string(),
        ),
        (
            "What data does Catenary collect and store?".to_string(),
            r#"When you send a message, Catenary stores the content of the message, your location data neccessary to connect you to other users, and the time the message was sent.

            Messages and their metadata are stored on the server for less than 20 minutes. Note that you and other users who got the message may have a copy of it on their devices that persists for longer.

            Usernames you see in the chat are randomly generated. Messages from the same user will have the same username. To achieve this, Catenary stores a cookie on your device with a randomly generated ID. This cookie has a lifetime of 12 hours.

            There is no need to create an account to use Catenary."#.to_string(),
        ),
        (
            "I see a location data error. How can I fix it?".to_string(),
            r#"Catenary needs access to your location data to match you with other users.
            
            Here is a little checklist to help you troubleshoot:

            • Make sure your device supports location services (most phones and laptops do).
            • Make sure that your device has a GPS signal or is capable of determining its location in some other way (e.g. by connecting to a WiFi network).
            • Make sure your browser has permission to access your location data (check your OS settings).
            • Make sure your browser allows this site to access your location data (check your browser settings).
            "#.to_string(),
        ),
        (
            "Why do I need to be in motion to use Catenary?".to_string(),
            r#"Catenary is designed to connect you to people on the same bus, train, boat, etc. 
            It does that by matching you with people who have a similar location, speed and direction of travel.

            There are already apps out there matching people solely based on location, so Catenary is designed to fill a different niche; connecting people who are travelling.
            "#.to_string(),
        ),

        (
            "How does the voting system work?".to_string(),
            r#"Upvotes increase a message's text size, downvotes decrease its opacity.

            The effect of one vote is relatively small. It may need votes from several users before it becomes clearly visible. This is intentional.
            "#.to_string(),
        ),
    ];

    view! {
        <Titlebar current_page="faq"/>
        <div class="main-container">
            <div class="main">
                <FAQ title="FAQ" q_and_a/>
                <Footer/>
            </div>
        </div>
    }
}

#[component]
pub fn FAQ(title: &'static str, q_and_a: Vec<(String, String)>) -> impl IntoView {
    let (q_and_a, _) = create_signal(q_and_a);
    let (opened, set_opened) = create_signal(HashSet::<String>::new());

    view! {
        <div class="faq">
            <h1>{title}</h1>
            <div class="faq-items">
                <For
                    each=q_and_a
                    key=|item| item.0.clone()
                    children=move |item| {
                        let qa = item.clone();
                        let q = item.clone().0;
                        let a = item.clone().1;
                        view! {
                            <div class="faq-item">
                                <button type="button"
                                    class="q"
                                    on:click=move |_| {
                                        log::info!("clicked {}", qa.0);
                                        let mut opened = opened.get();
                                        if opened.contains(&qa.0) {
                                            opened.remove::<String>(&qa.0);
                                        } else {
                                            opened.insert(qa.0.clone());
                                        }
                                        set_opened(opened);
                                    }
                                >
                                    {q}
                                </button>
                                <Show when=move || opened.get().contains(&item.0)
                                    fallback=move || view! {}
                                >
                                    <p class="a">{a.clone()}</p>
                                </Show>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}
