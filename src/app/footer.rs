use leptos::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <div class="footer">
            <div class="creator-links">
                <p>
                    "Found a bug? "
                    <br/>
                    <a href="https://github.com/jflessau/catenary/issues">"Open an issue"</a>
                    " or "
                    <a href="https://github.com/jflessau/catenary">"fork me"</a>
                    " on GitHub."
                </p>
            </div>
            <div class="legal-links">
                <p>
                    <a href="https://jflessau.com/info/legal-notice/">"Legal Notice"</a>
                    <br/>
                    <br/>
                    <a href="https://jflessau.com/info/privacy-policy/">"Privacy Policy"</a>
                </p>
            </div>
        </div>
    }
}
