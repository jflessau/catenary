use super::footer::Footer;
use super::Titlebar;
use leptos::*;
use leptos_router::A;

#[component]
pub fn View() -> impl IntoView {
    view! {
        <Titlebar current_page="home"/>
        <div class="main-container">
            <div class="main">
                <div class="card">
                    <h1>Catenary</h1>
                    <p class="subtitle">
                        "Chat with people on the same"
                        <span> train, </span>
                        <span> tram, </span>
                        <span> bus, </span>
                        <span> subway, </span>
                        <span> ferry, </span>
                        <span> plane, </span>
                        <span> cabel car, </span>
                        <span> ... </span>
                    </p>
                    <p class="text">
                        "Catenary leverages your geolocation data to
                        put you in an anonymous chatroom with people
                        on the same vehicle as you."
                    </p>
                    <div class="buttons">
                        <A href="/chat" class="button clickable">"Let's go!"</A>
                        <A href="/faq" class="button dark clickable">"Not conviced. I need more info."</A>
                    </div>
                </div>
                <Footer/>
            </div>
        </div>
    }
}
