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
                    <Route path="/faq" view=faq::View/>
                    <Route path="/rules" view=rules::View/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
pub fn Titlebar(go_back_link: Option<&'static str>) -> impl IntoView {
    view! {
        <div class="titlebar">
            <A href="/">
                <h1>"Catenary"</h1>
            </A>
            <Show when=move || go_back_link.is_some()
                fallback=move || view!{
                    <div class="links">
                        <p>
                            <A href="/faq">"FAQ"</A>
                        </p>
                        <p>
                            <A href="/rules">"Rules"</A>
                        </p>
                    </div>
                }>
                <p class="back">
                    <A href=go_back_link.unwrap_or("/")>"Back"</A>
                </p>
            </Show>
        </div>
    }
}

fn faq_text() -> Vec<(String, String)> {
    vec![
        (
            "What is Catenary?".to_string(),
            r#"Catenary is a chat application for people on the same bus, train, boat or other means of public transport.

            Using the geolocation data from your device, Catenary will automatically connect you to
            other people with a similar location, speed and direction of travel."#.to_string(),
        ),
        (
            "What data does Catenary collect and store?".to_string(),
            r#"When you send a message, Catenary stores the content of the message, your location data neccessary to connect you to other users, and the time the message was sent.

            Messages and their metadata are stored on the server for less than 20 minutes. Note that you and other users who got the message may have a copy of it on their devices that persists for longer.

            The usernames you see in the chat are randomly generated. Messages from the same user will have the same username. To achieve this, Catenary stores a cookie on your device with a randomly generated ID. This cookie has a lifetime of 12 hours.

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
    ]
}
