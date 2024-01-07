use cfg_if::cfg_if;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use leptos::LeptosOptions;
        use axum::extract::FromRef;
        use tokio::sync::mpsc::{Sender};
        use std::sync::{Arc, Mutex};
        use tokio::sync::broadcast::Sender as BroadcastSender;

        #[derive(FromRef, Debug, Clone)]
        pub struct AppState{
            pub leptos_options: LeptosOptions,
            pub chat_msg_tx: Sender<ChatMessage>,
            pub plane: Arc<Mutex<Plane>>,
            pub chat_msg_broadcast_tx: BroadcastSender<ChatMessage>,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Plane {
    messages: VecDeque<ChatMessage>,
}

impl Plane {
    pub fn new() -> Self {
        Plane {
            messages: VecDeque::with_capacity(100_000),
        }
    }

    pub fn add_message(&mut self, msg: ChatMessage) {
        self.messages.push_front(msg);
        if self.messages.len() >= self.messages.capacity() {
            self.messages.pop_back();
        }
    }

    // TODO: get messages based on trace
    pub fn get_messages(&self) -> Vec<ChatMessage> {
        // get latest 100 messages
        self.messages.iter().take(100).cloned().collect()

    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Uuid,
    pub author: String,
    pub text: String,
    pub trace: Trace,
    pub timestamp: DateTime<Utc>,
}

impl ChatMessage {
    pub fn new(author: String, mut text: String, trace: Trace) -> Self {
        if text.len() > 144 {
            log::warn!("message too long: {}", text.len());
            text.truncate(144);
        }

        Self {
            id: Uuid::new_v4(),
            author,
            text,
            trace,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Trace {
    pub location: GeoLocation,
    pub velocity: Velocity,
}

impl Trace {
    pub fn new(location: GeoLocation, velocity: Velocity) -> Self {
        Self { location, velocity }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GeoLocation {
    pub x: f32,
    pub y: f32,
}

impl GeoLocation {
    pub fn new(x: f32, y: f32) -> Self {
        if x.is_nan() || y.is_nan() {
            log::warn!("location is nan: {}, {}", x, y);
            return Self { x: 0.0, y: 0.0 };
        }
        if x.is_infinite() || y.is_infinite() {
            log::warn!("location is infinite: {}, {}", x, y);
            return Self { x: 0.0, y: 0.0 };
        }
        if x < -180.0 || x > 180.0 || y < -90.0 || y > 90.0 {
            log::warn!("location is out of bounds: {}, {}", x, y);
            return Self { x: 0.0, y: 0.0 };
        }

        Self { x, y }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        if x.is_nan() || y.is_nan() {
            log::warn!("velocity is nan: {}, {}", x, y);
            return Self { x: 0.0, y: 0.0 };
        }
        if x.is_infinite() || y.is_infinite() {
            log::warn!("velocity is infinite: {}, {}", x, y);
            return Self { x: 0.0, y: 0.0 };
        }
        if x > 2000.0 || x < -2000.0 || y > 2000.0 || y < -2000.0 {
            log::warn!("velocity too high: {}, {}", x, y);
            return Self { x: 0.0, y: 0.0 };
        }

        Self { x, y }
    }

    pub fn angle(&self) -> f32 {
        self.y.atan2(self.x).to_degrees()
    }

    pub fn magnitude(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}
