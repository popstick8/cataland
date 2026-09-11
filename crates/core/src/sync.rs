use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Chat, Identity, RoomAction, RoomView, Text};

#[derive(Clone, Debug, Default, Serialize, Deserialize, TS)]
pub struct Cursor {
    pub room: String,
    pub events: u64,
    pub chat: usize,
}

impl RoomView {
    pub fn cursor(&self) -> Cursor {
        Cursor {
            room: self.id.clone(),
            events: self
                .game
                .as_ref()
                .and_then(|game| game.events.last())
                .map_or(0, |event| event.seq),
            chat: self.chat.len(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Request {
    Join {
        identity: Identity,
        cursor: Option<Cursor>,
    },
    Action {
        action: RoomAction,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Response {
    State {
        room: Box<RoomView>,
    },
    Chat {
        room: String,
        offset: usize,
        messages: Vec<Chat>,
    },
    Error {
        message: Text,
    },
}

impl Response {
    pub fn apply(self, current: &mut Option<RoomView>) -> Result<(), Text> {
        match self {
            Self::State { mut room } => {
                if let Some(previous) = current
                    .as_mut()
                    .filter(|previous| previous.id == room.id && previous.you == room.you)
                {
                    room.chat = std::mem::take(&mut previous.chat);
                    if let (Some(game), Some(old)) = (&mut room.game, &mut previous.game) {
                        old.events.append(&mut game.events);
                        game.events = std::mem::take(&mut old.events);
                    }
                }
                *current = Some(*room);
            }
            Self::Chat {
                room,
                offset,
                messages,
            } => {
                let current = current
                    .as_mut()
                    .filter(|current| current.id == room)
                    .ok_or("请先进入房间")?;
                current.chat.truncate(offset);
                current.chat.extend(messages);
            }
            Self::Error { message } => return Err(message),
        }
        Ok(())
    }
}
