use cfg_if::cfg_if;
use chrono::{DateTime, Utc};
use geo::{geometry::Point, GeodesicDistance};
use names::Generator;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;
use web_sys::PositionError;

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
    pub location: (f64, f64), // lat, lon
    pub speed: f64,           // meters per second
    pub slope: f64,           // degrees
}

impl Trace {
    pub fn new(location: (f64, f64), speed: f64, slope: f64) -> Self {
        Self {
            location,
            speed,
            slope,
        }
    }
}

#[derive(Clone, Debug)]
pub enum NoTrace {
    NoPermission,
    PositionUnavailable,
    Timeout,
    WaitingForMoreLocations {
        received_locations: usize,
        required_locations: usize,
    },
    WaitingForTimeToPass,
    TooSlow {
        current_speed: f64,
        required_speed: f64,
    },
}

impl From<PositionError> for NoTrace {
    fn from(err: PositionError) -> Self {
        match err.code() {
            1 => NoTrace::NoPermission,
            2 => NoTrace::PositionUnavailable,
            _ => NoTrace::Timeout,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LocationHistory {
    locations: Vec<(Point<f64>, DateTime<Utc>)>,
    size: usize,
    max_location_age: usize,      // seconds
    min_location_time_delta: f64, // seconds
    min_speed: f64,               // seconds
}

impl LocationHistory {
    pub fn new() -> Self {
        Self {
            locations: vec![],
            size: 3, // TODO: set this to, say, 8?
            max_location_age: 60,
            min_location_time_delta: 1.0, // TODO: set this to, say, 3.0?
            min_speed: 3.0,
        }
    }

    pub fn add_location(&mut self, location: Point<f64>) {
        self.locations.insert(0, (location, Utc::now()));
        if self.locations.len() > self.size {
            self.locations.pop();
        }
    }

    pub fn trace(&mut self) -> Result<Trace, NoTrace> {
        return Ok(Trace::new((0.0, 0.0), 0.0, 0.0));

        self.locations.retain(|(_, timestamp)| {
            let duration = Utc::now() - *timestamp;
            duration.num_seconds() < self.max_location_age as i64
        });

        if self.size > self.locations.len() {
            log::info!(
                "not enough points for trace: {} / {}",
                self.locations.len(),
                self.size
            );
            return Err(NoTrace::WaitingForMoreLocations {
                received_locations: self.locations.len(),
                required_locations: self.size,
            });
        }

        let earliest: Option<&(Point<f64>, DateTime<Utc>)> = self.locations.get(self.size - 1);
        let latest: Option<&(Point<f64>, DateTime<Utc>)> = self.locations.first();

        match (earliest, latest) {
            (Some((p_a, t_a)), Some((p_b, t_b))) => {
                let duration = (*t_b - *t_a).num_milliseconds() as f64 / 1000.0;
                if duration < self.min_location_time_delta {
                    Err(NoTrace::WaitingForTimeToPass)
                } else {
                    let distance = p_a.geodesic_distance(p_b);
                    let speed = distance / duration;
                    if speed <= self.min_speed {
                        log::info!("speed too low: {} m/s", speed);
                        return Err(NoTrace::TooSlow {
                            current_speed: speed as f64,
                            required_speed: self.min_speed,
                        });
                    }
                    let line = geo::Line::new(*p_a, *p_b);
                    let slope = line.slope().to_radians().to_degrees();
                    log::info!(
                        "duration: {} s, distance: {} m, speed: {} m/s, slope: {} deg",
                        duration,
                        distance,
                        speed,
                        slope
                    );
                    Ok(Trace::new((p_b.x(), p_b.y()), speed, slope))
                }
            }
            _ => {
                log::error!(
                    "couldn't get earliest and latest points, this shouldn't happen, deque: {:?}",
                    self.locations
                );
                Err(NoTrace::WaitingForTimeToPass)
            }
        }
    }
}
