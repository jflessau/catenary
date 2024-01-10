use cfg_if::cfg_if;
use chrono::{DateTime, Utc};
use names::Generator;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use leptos::LeptosOptions;
        use axum::extract::FromRef;
        use tokio::sync::mpsc::{Sender};
        use std::sync::{Arc, Mutex};

        #[derive(FromRef, Debug, Clone)]
        pub struct AppState{
            pub leptos_options: LeptosOptions,
            pub chat_msg_in_tx: Sender<ChatMessageIn>,
            pub plane: Arc<Mutex<Plane>>,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Plane {
    messages: VecDeque<ChatMessage>,
    author_usernames_by_id: HashMap<Uuid, String>,
}

impl Plane {
    pub fn new() -> Self {
        Plane {
            messages: VecDeque::with_capacity(100_000),
            author_usernames_by_id: HashMap::new(),
        }
    }

    pub fn add_message(&mut self, msg: ChatMessageIn) {
        let username = match self.author_usernames_by_id.get(&msg.author) {
            Some(username) => username.clone(),
            None => {
                let username = Generator::default()
                    .next()
                    .unwrap_or_else(|| "anonymous".to_string());

                self.author_usernames_by_id
                    .insert(msg.author, username.clone());

                username
            }
        };

        let msg = ChatMessage::from((msg, username));
        self.messages.push_front(msg.clone());
        if self.messages.len() >= self.messages.capacity() {
            self.messages.pop_back();
        }
    }

    // TODO: get messages based on trace
    pub fn get_messages(&self, user_id: Option<Uuid>) -> Vec<ChatMessageOut> {
        // get latest 100 messages
        self.messages
            .iter()
            .take(100)
            .cloned()
            .into_iter()
            .map(|msg| ChatMessageOut::from((msg, user_id)))
            .collect()
    }

    pub fn vote_message(&mut self, id: Uuid, user_id: Uuid, up: bool) {
        let Some(msg) = self.messages.iter_mut().find(|msg| msg.id == id) else {
            log::warn!("couldn't find message with id: {}", id);
            return;
        };

        if up {
            if msg.upvoters.contains(&user_id) {
                msg.upvoters.remove(&user_id);
            } else {
                msg.upvoters.insert(user_id);
                msg.downvoters.remove(&user_id);
            }
        } else {
            if msg.downvoters.contains(&user_id) {
                msg.downvoters.remove(&user_id);
            } else {
                msg.downvoters.insert(user_id);
                msg.upvoters.remove(&user_id);
            }
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ChatMessageIn {
    pub id: Uuid,
    pub author: Uuid,
    pub username: Option<String>,
    pub text: String,
    pub trace: Trace,
    pub timestamp: DateTime<Utc>,
}

impl ChatMessageIn {
    pub fn new(author: Uuid, mut text: String, trace: Trace) -> Self {
        if text.len() > 144 {
            log::warn!("message too long: {}", text.len());
            text.truncate(144);
        }

        let text = text.trim().to_string();

        Self {
            id: Uuid::new_v4(),
            author,
            username: None,
            text,
            trace,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Uuid,
    pub author: Uuid,
    pub username: String,
    pub text: String,
    pub trace: Trace,
    pub upvoters: HashSet<Uuid>,
    pub downvoters: HashSet<Uuid>,
    pub timestamp: DateTime<Utc>,
}

impl From<(ChatMessageIn, String)> for ChatMessage {
    fn from((msg, username): (ChatMessageIn, String)) -> Self {
        Self {
            id: msg.id,
            author: msg.author,
            username,
            text: msg.text,
            trace: msg.trace,
            upvoters: HashSet::new(),
            downvoters: HashSet::new(),
            timestamp: msg.timestamp,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ChatMessageOut {
    pub id: Uuid,
    pub username: String,
    pub text: String,
    pub upvoters: usize,
    pub downvoters: usize,
    pub vote: Option<Vote>,
    pub timestamp: DateTime<Utc>,
}

impl From<(ChatMessage, Option<Uuid>)> for ChatMessageOut {
    fn from((msg, user_id): (ChatMessage, Option<Uuid>)) -> Self {
        let vote = user_id
            .map(|user_id| {
                if msg.upvoters.contains(&user_id) {
                    Some(Vote::Up)
                } else if msg.downvoters.contains(&user_id) {
                    Some(Vote::Down)
                } else {
                    None
                }
            })
            .flatten();

        Self {
            id: msg.id,
            username: msg.username,
            text: msg.text,
            upvoters: msg.upvoters.len(),
            downvoters: msg.downvoters.len(),
            vote,
            timestamp: msg.timestamp,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Vote {
    Up,
    Down,
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
