#[cfg(not(feature = "ssr"))]
pub fn main() {
    use catenary::app::App;
    use leptos::*;

    _ = console_log::init_with_level(log::Level::Info);
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! {  <App/> }
    });
}

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::{
        body::Body as AxumBody,
        extract::{Extension, Path, RawQuery, State},
        http::{header::HeaderMap, Request},
        response::{IntoResponse, Response},
        routing::get,
        Router,
    };
    use catenary::app::App;
    use catenary::fileserv::file_and_error_handler;
    use catenary::state::{AppState, Plane};
    use catenary::state::{ChatMessage, ChatMessageIn};
    use env_logger::Builder;
    use leptos::{get_configuration, provide_context, view};
    use leptos_axum::LeptosRoutes;
    use leptos_axum::{generate_route_list, handle_server_fns_with_context};
    use std::sync::{Arc, Mutex};
    use tokio::sync::mpsc::{channel, Receiver, Sender};
    use uuid::Uuid;

    // setup logging

    dotenv::dotenv().ok();
    Builder::new()
        .parse_filters(
            &std::env::var("RUST_LOG")
                .unwrap_or("info,tracing=warn,leptos_dom=warn,leptos_axum=warn".to_string()),
        )
        .init();

    #[axum::debug_handler]
    async fn server_fn_handler(
        State(app_state): State<AppState>,
        path: Path<String>,
        headers: HeaderMap,
        raw_query: RawQuery,
        request: Request<AxumBody>,
    ) -> impl IntoResponse {
        // TODO: this is a terrible hack and there must be a better way

        let user_re = regex::Regex::new(
            r#"(user=)([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})"#,
        )
        .expect("couldn't compile user regex");
        let cookie = request.headers().get("cookie");
        let user_id_str = cookie
            .and_then(|cookie| {
                user_re
                    .captures(cookie.to_str().unwrap())
                    .and_then(|captures| captures.get(2))
            })
            .map(|uuid| uuid.as_str().to_string());
        let user_uuid =
            Uuid::parse_str(&user_id_str.unwrap_or_default()).unwrap_or_else(|_| Uuid::new_v4());

        handle_server_fns_with_context(
            path,
            headers,
            raw_query,
            move || {
                provide_context(app_state.chat_msg_in_tx.clone());
                provide_context(app_state.plane.clone());
                provide_context(user_uuid);
            },
            request,
        )
        .await
    }

    async fn leptos_routes_handler(
        State(app_state): State<AppState>,
        req: Request<AxumBody>,
    ) -> Response {
        let handler = leptos_axum::render_app_to_stream_with_context(
            app_state.leptos_options.clone(),
            move || {
                provide_context(app_state.chat_msg_in_tx.clone());
            },
            || view! { <App/> },
        );
        handler(req).await.into_response()
    }

    // configure leptos

    let conf = get_configuration(None)
        .await
        .expect("couldn't get configuration");
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(|| view! {<App/> });

    // build app state

    let messages = Arc::new(Mutex::new(Vec::<ChatMessage>::new()));
    let messages_clone = messages.clone();
    let (chat_msg_in_tx, mut chat_msg_in_rx): (Sender<ChatMessageIn>, Receiver<ChatMessageIn>) =
        channel(1000);

    let plane = Arc::new(Mutex::new(Plane::new()));
    let state = AppState {
        leptos_options: leptos_options,
        chat_msg_in_tx,
        plane: plane.clone(),
    };

    // compose axum router

    let app = Router::new()
        .route(
            "/api/*fn_name",
            get(server_fn_handler).post(server_fn_handler),
        )
        .leptos_routes_with_handler(routes, get(leptos_routes_handler))
        .layer(Extension(messages_clone))
        .fallback(file_and_error_handler)
        .with_state(state);

    // start message listener

    tokio::spawn(async move {
        log::info!("starting message listener");
        loop {
            let Some(msg) = chat_msg_in_rx.recv().await else {
                log::warn!("couldn't receive message via chat_msg_in_rx");
                continue;
            };
            let Ok(mut plane) = plane.try_lock() else {
                log::warn!("couldn't lock plane mutex in message listener");
                continue;
            };
            plane.add_message(msg);
        }
    });

    // serve

    log::info!("listening on http://{}", &addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
