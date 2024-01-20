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

// TODO: Move these constants to environment variables, but only after
// experiencing the pain of building, deploying, running, and realizing
// one of these needs to be changed. Repeat that process a few times,
// and _then_ move them to environment variables, as is tradition.

// max. amount of messages hold in memory
const MAX_MESSAGES_IN_MEMORY: usize = 100_000;
// max. amount of characters in a message
const MAX_MESSAGE_LENGTH: usize = 144;
// max. message age in minutes before removing from memory
const MAX_MESSAGE_AGE_MINUTES: i64 = 10;

// max. amount of locations stored in history
const MAX_LOCATIONS_IN_HISTORY: usize = 4;
// max. age of location in seconds before removing from history
const MAX_LOCATION_AGE_SECONDS: usize = 60;
// min. amount of seconds between first and last location in history, below that, the trace is not valid
const MIN_LOCATION_TIME_DELTA_SECONDS: f64 = 1.5;
// min. speed in meters per second, below that, the trace is not valid
const MIN_SPEED_METERS_PER_SECOND: f64 = 3.0;

// match traces if distance covered of self in x seconds is smaller than distance diff between self and other
const TRACE_MATCH_MAX_MOVE_SECONDS: f64 = 180.0;
// max. slope diff between two traces in degrees
const TRACE_MATCH_MAX_SLOPE_DIFF_DEGREES: f64 = 32.0;

#[derive(Debug, Clone, Default)]
pub struct Plane {
    messages: VecDeque<ChatMessage>,
    author_usernames_by_id: HashMap<Uuid, String>,
}

impl Plane {
    pub fn new() -> Self {
        Plane {
            messages: VecDeque::with_capacity(MAX_MESSAGES_IN_MEMORY),
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

        self.messages
            .retain(|msg| (Utc::now() - msg.timestamp).num_minutes() < MAX_MESSAGE_AGE_MINUTES);

        if self.messages.len() >= self.messages.capacity() {
            self.messages.pop_back();
        }
    }

    // TODO: get messages based on trace
    pub fn get_messages(&self, user_id: Option<Uuid>, trace: Trace) -> Vec<ChatMessageOut> {
        // get latest 100 messages
        let mut messages: Vec<ChatMessageOut> = self
            .messages
            .iter()
            .take(10000)
            .filter(|&msg| trace.overlaps_with(&msg.trace))
            .cloned()
            .map(|msg| ChatMessageOut::from((msg, user_id)))
            .collect();

        messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        messages
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
        } else if msg.downvoters.contains(&user_id) {
            msg.downvoters.remove(&user_id);
        } else {
            msg.downvoters.insert(user_id);
            msg.upvoters.remove(&user_id);
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
        if text.len() > MAX_MESSAGE_LENGTH {
            log::warn!("message too long: {}", text.len());
            text.truncate(MAX_MESSAGE_LENGTH);
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
        let vote = user_id.and_then(|user_id| {
            if msg.upvoters.contains(&user_id) {
                Some(Vote::Up)
            } else if msg.downvoters.contains(&user_id) {
                Some(Vote::Down)
            } else {
                None
            }
        });

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

    fn overlaps_with(&self, other: &Self) -> bool {
        if self.speed < MIN_SPEED_METERS_PER_SECOND || other.speed < MIN_SPEED_METERS_PER_SECOND {
            return false;
        }

        let distance_meters = Point::new(self.location.0, self.location.1)
            .geodesic_distance(&Point::new(other.location.0, other.location.1));
        // let speed_diff_meter_per_second = (self.speed - other.speed).abs();
        let slope_diff = (other.slope - self.slope).abs();

        // match if distance diff is smaller than distance covered by self in 2 minutes
        distance_meters < self.speed * TRACE_MATCH_MAX_MOVE_SECONDS
            // match if speed diff is smaller than 1 m/s
            // && speed_diff_meter_per_second < 20.0 // TODO: think about that for a while
            // match if slope diff is smaller than x degrees
            && slope_diff < TRACE_MATCH_MAX_SLOPE_DIFF_DEGREES // TODO: maybe the allowed diff should be higher for lower speeds?
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn overlaps_with() {
        use super::*;

        // low speed

        let trace_a = Trace::new((0.0, 0.0), MIN_SPEED_METERS_PER_SECOND - 1.0, 0.0);
        let trace_b = Trace::new((0.0, 0.0), MIN_SPEED_METERS_PER_SECOND + 1.0, 0.0);
        assert!(!trace_a.overlaps_with(&trace_b), "self has low speed");
        let trace_a = Trace::new((0.0, 0.0), MIN_SPEED_METERS_PER_SECOND + 1.0, 0.0);
        let trace_b = Trace::new((0.0, 0.0), MIN_SPEED_METERS_PER_SECOND - 1.0, 0.0);
        assert!(!trace_a.overlaps_with(&trace_b), "other has low speed");
        let trace_a = Trace::new((0.0, 0.0), MIN_SPEED_METERS_PER_SECOND - 1.0, 0.0);
        let trace_b = Trace::new((0.0, 0.0), MIN_SPEED_METERS_PER_SECOND - 1.0, 0.0);
        assert!(!trace_a.overlaps_with(&trace_b), "both have low speed");

        // slope diff

        let trace_a = Trace::new((0.0, 0.0), MIN_SPEED_METERS_PER_SECOND + 1.0, 0.0);
        let trace_b = Trace::new((0.0, 0.0), MIN_SPEED_METERS_PER_SECOND + 1.0, 0.0);
        assert!(trace_a.overlaps_with(&trace_b), "same slope");
        let trace_a = Trace::new((0.0, 0.0), MIN_SPEED_METERS_PER_SECOND + 1.0, 0.0);
        let trace_b = Trace::new(
            (0.0, 0.0),
            MIN_SPEED_METERS_PER_SECOND + 1.0,
            TRACE_MATCH_MAX_SLOPE_DIFF_DEGREES - 1.0,
        );
        assert!(trace_a.overlaps_with(&trace_b), "small slope diff");
        let trace_a = Trace::new((0.0, 0.0), MIN_SPEED_METERS_PER_SECOND + 1.0, 0.0);
        let trace_b = Trace::new(
            (0.0, 0.0),
            MIN_SPEED_METERS_PER_SECOND + 1.0,
            TRACE_MATCH_MAX_SLOPE_DIFF_DEGREES + 1.0,
        );
        assert!(!trace_a.overlaps_with(&trace_b), "big slope diff");

        // distance diff

        let bus_speed_rush_hour = 6.0;
        let trace_a = Trace::new((53.552196, 9.994872), 12.0, 0.0);
        let trace_b = Trace::new((53.555574, 10.000226), bus_speed_rush_hour, 0.0);
        assert!(
            trace_a.overlaps_with(&trace_b),
            "bus rush hour, Europapassage -> Kunsthalle"
        );

        let trace_a = Trace::new((53.552196, 9.994872), 12.0, 0.0);
        let trace_b = Trace::new((53.564007, 10.015946), 12.0, 0.0);
        assert!(
            !trace_a.overlaps_with(&trace_b),
            "bus rush hour, Europapassage -> Schwanenwik"
        );

        let bus_speed = 13.0;
        let trace_a = Trace::new((53.559220, 10.007939), bus_speed, 0.0);
        assert!(
            trace_a.overlaps_with(&trace_b),
            "bus, Europapassage -> Gurlittinsel"
        );

        // TODO: add more tests

        // let bus_speed_on_highway = 25.0;
        // let train_speed = 35.0;
        // let high_speed_train_speed = 75.0;
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
    max_location_age_seconds: usize,      // seconds
    min_location_time_delta_seconds: f64, // seconds
    min_speed_meters_pers_second: f64,    // meters per second
}

impl LocationHistory {
    pub fn new() -> Self {
        Self {
            locations: vec![],
            size: MAX_LOCATIONS_IN_HISTORY,
            max_location_age_seconds: MAX_LOCATION_AGE_SECONDS,
            min_location_time_delta_seconds: MIN_LOCATION_TIME_DELTA_SECONDS,
            min_speed_meters_pers_second: MIN_SPEED_METERS_PER_SECOND,
        }
    }

    pub fn add_location(&mut self, location: Point<f64>) {
        self.locations.insert(0, (location, Utc::now()));
        if self.locations.len() > self.size {
            self.locations.pop();
        }
    }

    pub fn trace(&mut self) -> Result<Trace, NoTrace> {
        // return Ok(Trace::new((0.0, 0.0), 5.0, 20.0));
        // return Ok(Trace::new((0.0, 0.0), 0.0, 0.0));
        // return Err(NoTrace::NoPermission);
        // return Err(NoTrace::PositionUnavailable);
        // return Err(NoTrace::Timeout);
        // return Err(NoTrace::WaitingForMoreLocations {
        //     received_locations: 2,
        //     required_locations: 3,
        // });
        // return (Err(NoTrace::WaitingForTimeToPass));
        // return Err(NoTrace::TooSlow {
        //     current_speed: 0.3,
        //     required_speed: 3.0,
        // });

        self.locations.retain(|(_, timestamp)| {
            let duration = Utc::now() - *timestamp;
            duration.num_seconds() < self.max_location_age_seconds as i64
        });

        if self.size > self.locations.len() {
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
                if duration < self.min_location_time_delta_seconds {
                    Err(NoTrace::WaitingForTimeToPass)
                } else {
                    let distance = p_a.geodesic_distance(p_b);
                    let speed = distance / duration;
                    if speed < self.min_speed_meters_pers_second - 0.001 {
                        return Err(NoTrace::TooSlow {
                            current_speed: speed,
                            required_speed: self.min_speed_meters_pers_second,
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
