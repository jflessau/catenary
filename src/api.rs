use crate::state::ChatMessage;
use leptos::*;

#[cfg(feature = "ssr")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "ssr")]
use tokio::sync::mpsc::Sender;
#[cfg(feature = "ssr")]
use crate::state::*;


#[server(SendMessage, "/api")]
pub async fn send_message(text: String) -> Result<(), ServerFnError> {
    let tx = use_context::<Sender<ChatMessage>>().expect("couldn't get sender context");

    if text.trim().is_empty() {
        return Ok(());
    }

    let msg = ChatMessage::new(
        "anonymous".to_string(),
        text,
        Trace::new(GeoLocation::new(0.0, 0.0), Velocity::new(0.0, 0.0)),
    );

    tx.send(msg.clone())
        .await
        .expect("couldn't send chat message");

    Ok(())
}

#[server(ListMessages, "/api")]
pub async fn list_messages() -> Result<Vec<ChatMessage>, ServerFnError> {
    log::info!("list_messages");
    let tx = use_context::<Arc<Mutex<Plane>>>().expect("couldn't get plane context");

    let Ok(plane) = tx.try_lock() else {
        log::warn!("couldn't lock plane mutex in list handler");
        return Ok(vec![]);
    };

    Ok(plane.get_messages())
}