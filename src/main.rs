#[cfg(not(feature = "ssr"))]
pub fn main() {
    use leptos::{*};
    use catenary::app::{App};

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
    use catenary::app::{App};
    use catenary::fileserv::file_and_error_handler;
    use catenary::state::{AppState, Plane};
    use catenary::state::ChatMessage;
    use leptos::{get_configuration, provide_context, view};
    use leptos_axum::LeptosRoutes;
    use leptos_axum::{generate_route_list, handle_server_fns_with_context};
    use std::sync::{Arc, Mutex};
    use tokio::sync::mpsc::{channel, Receiver, Sender};
    use tokio::sync::broadcast::{channel as broadcast_channel, Sender as BroadcastSender};

    async fn server_fn_handler(
        State(app_state): State<AppState>,
        path: Path<String>,
        headers: HeaderMap,
        raw_query: RawQuery,
        request: Request<AxumBody>,
    ) -> impl IntoResponse {
        handle_server_fns_with_context(
            path,
            headers,
            raw_query,
            move || {
                provide_context(app_state.chat_msg_tx.clone());
                provide_context(app_state.plane.clone());
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
                provide_context(app_state.chat_msg_tx.clone());
            },
            || view! { <App/> },
        );
        handler(req).await.into_response()
    }

    // setup logging

    simple_logger::init_with_level(log::Level::Warn).expect("couldn't initialize logging");

    // configure leptos

    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(|| view! {<App/> });

    // build app state

    let messages = Arc::new(Mutex::new(Vec::<ChatMessage>::new()));
    let messages_clone = messages.clone();
    let (tx, mut rx): (Sender<ChatMessage>, Receiver<ChatMessage>) = channel(1000);
    let (broadcast_tx, _): (BroadcastSender<ChatMessage>, _) = broadcast_channel(1000);
    let plane = Arc::new(Mutex::new(Plane::new()));
    let state = AppState {
        leptos_options: leptos_options,
        chat_msg_tx: tx,
        plane: plane.clone(),
        chat_msg_broadcast_tx: broadcast_tx.clone(),
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
            let msg = rx.recv().await.unwrap();
            println!("got message in listener loop: {:?}", msg);
            let Ok(mut plane) = plane.lock() else {
                log::warn!("couldn't lock plane mutex in listener loop");
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
