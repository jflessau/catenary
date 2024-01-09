use crate::state::ChatMessageOut;
use leptos::*;
use uuid::Uuid;

#[cfg(feature = "ssr")]
use crate::state::*;
#[cfg(feature = "ssr")]
use leptos_axum::ResponseOptions;
#[cfg(feature = "ssr")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "ssr")]
use tokio::sync::mpsc::Sender;

#[server(SendMessage, "/api")]
pub async fn send_message(text: String) -> Result<(), ServerFnError> {
    let tx = use_context::<Sender<ChatMessageIn>>().expect("couldn't get sender context");
    let response = expect_context::<ResponseOptions>();

    let user_id = match use_context::<Uuid>() {
        Some(user_id) => user_id,
        None => Uuid::new_v4(),
    };

    response.insert_header(
        http::header::SET_COOKIE,
        http::HeaderValue::from_str(&format!("user={}", user_id))
            .expect("couldn't set user cookie"),
    );

    if text.trim().is_empty() {
        return Ok(());
    }

    let msg = ChatMessageIn::new(
        user_id,
        text,
        Trace::new(GeoLocation::new(0.0, 0.0), Velocity::new(0.0, 0.0)),
    );

    tx.send(msg.clone())
        .await
        .expect("couldn't send chat message");

    Ok(())
}

#[server(VoteMessage, "/api")]
pub async fn vote_message(id: Uuid, up: bool) -> Result<(), ServerFnError> {
    log::info!("vote_message with id {:?}, upvote: {}", id, up);
    let tx = use_context::<Arc<Mutex<Plane>>>().expect("couldn't get plane context");
    let Some(user_id) = use_context::<Uuid>() else {
        log::warn!("couldn't get user id in vote handler");
        return Ok(());
    };

    let Ok(mut plane) = tx.try_lock() else {
        log::warn!("couldn't lock plane mutex in vote handler");
        return Ok(());
    };

    plane.vote_message(id, user_id, up);

    Ok(())
}

#[server(ListMessages, "/api")]
pub async fn list_messages() -> Result<Vec<ChatMessageOut>, ServerFnError> {
    log::info!("list_messages");
    let tx = use_context::<Arc<Mutex<Plane>>>().expect("couldn't get plane context");
    let user_id = use_context::<Uuid>();

    let Ok(plane) = tx.try_lock() else {
        log::warn!("couldn't lock plane mutex in list handler");
        return Ok(vec![]);
    };

    Ok(plane.get_messages(user_id))
}
