pub mod room;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    #[default]
    Base,
    Cities,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct RoomSettings {
    pub name: String,
    pub mode: Mode,
    pub capacity: usize,
    pub starter: Option<usize>,
}

impl Default for RoomSettings {
    fn default() -> Self {
        Self {
            name: "群岛之约".into(),
            mode: Mode::Base,
            capacity: 4,
            starter: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Seat {
    pub name: String,
    pub color: usize,
    pub connected: bool,
    pub ready: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Chat {
    pub name: String,
    pub text: String,
    pub time: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct RoomView {
    pub id: String,
    pub settings: RoomSettings,
    pub seats: Vec<Seat>,
    pub spectators: Vec<String>,
    pub you: Option<usize>,
    pub host: bool,
    pub chat: Vec<Chat>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RoomAction {
    Configure { settings: RoomSettings },
    Profile { name: String, color: usize },
    Ready { ready: bool },
    Chat { text: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub token: String,
    pub name: String,
    pub color: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct RoomInfo {
    pub id: String,
    pub name: String,
    pub address: String,
    pub players: usize,
    pub capacity: usize,
    pub mode: Mode,
    pub started: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Request {
    Join { identity: Identity },
    Action { action: RoomAction },
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Response {
    State { room: RoomView },
    Error { message: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ClientView {
    pub identity: Identity,
    pub room: Option<RoomView>,
    pub connection: Connection,
    pub address: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum Connection {
    Home,
    Connecting,
    Connected,
    Disconnected,
}
