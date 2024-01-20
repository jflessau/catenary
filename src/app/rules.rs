use super::footer::Footer;
use super::Titlebar;
use leptos::*;

#[component]
pub fn View() -> impl IntoView {
    view! {
        <Titlebar current_page="rules"/>
        <div class="main-container mg">
            <div class="main">
                <div class="text-content">
                    <h1>Rules</h1>
                    <h2>Be Nice</h2>
                    <p>
                        "Spread positivity! Treat others with kindness and respect. Remember, behind every screen is a real person. Avoid offensive language, bullying, or any form of discrimination."
                    </p>

                    <h2>No Spamming</h2>
                    <p>
                        "Avoid flooding the chat with repeated messages, links, or any irrelevant content."
                    </p>

                    <h2 id="respect">Respect Privacy</h2>
                    <p>
                        "Avoid sharing your or others' contact details, addresses, or any sensitive information."
                    </p>

                    <h2>No NSFW Content</h2>
                    <p>
                        "This is a safe space for everyone. Don't share or request any explicit, adult, or inappropriate content. Let's make sure our discussions are comfortable for users of all ages and backgrounds."
                    </p>

                    <h2>No Advertising</h2>
                    <p>
                        "We're here to chat, not to sell. Please refrain from promoting products or services."
                    </p>

                    <h2>No Illegal Content</h2>
                    <p>
                        "Stay on the right side of the law! Do not share or engage in discussions related to illegal activities, including but not limited to hacking, piracy, or any content that violates intellectual property rights. Let's create a space where everyone can feel secure and respected."
                    </p>
                </div>
                <Footer/>
            </div>
        </div>
    }
}
