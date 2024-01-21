use cfg_if::cfg_if;

pub mod api;
pub mod app;
pub mod error_template;
pub mod fileserv;
pub mod state;

cfg_if! {
    if #[cfg(feature = "hydrate")] {
        use wasm_bindgen::prelude::wasm_bindgen;
        use crate::app::App;
        use leptos::view;
        use dotenv::dotenv;

        #[wasm_bindgen]
        pub fn hydrate() {
            dotenv().ok();
            _ = console_log::init_with_level(log::Level::Debug);
            console_error_panic_hook::set_once();

            leptos::mount_to_body(|| {
                view! { <App/> }
            });
        }
    }
}
