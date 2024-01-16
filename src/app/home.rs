use super::Titlebar;
use leptos::*;
use leptos_router::A;

#[component]
pub fn View() -> impl IntoView {
    view! {
        <Titlebar current_page="home"/>
        <div class="main-container">
            <p><A href="/chat">"Chat"</A></p>
        </div>
    }
}
