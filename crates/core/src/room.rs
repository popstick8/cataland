use serde::{Deserialize, Serialize};

use crate::{Chat, Identity, RoomAction, RoomSettings, RoomView, Seat, game::Game};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Member {
    pub identity: Identity,
    pub player: Option<usize>,
    pub connected: bool,
    pub ready: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Room {
    pub id: String,
    pub settings: RoomSettings,
    pub members: Vec<Member>,
    pub chat: Vec<Chat>,
    pub game: Option<Game>,
}

impl Room {
    pub fn new(id: String, settings: RoomSettings, identity: Identity) -> Result<Self, String> {
        validate_settings(&settings, 1)?;
        validate_name(&identity.name)?;
        if identity.color >= 6 {
            return Err("请选择一种玩家颜色".into());
        }
        Ok(Self {
            id,
            settings,
            members: vec![Member {
                identity,
                player: Some(0),
                connected: true,
                ready: true,
            }],
            chat: Vec::new(),
            game: None,
        })
    }

    pub fn join(&mut self, mut identity: Identity) -> Result<(), String> {
        validate_name(&identity.name)?;
        if let Some(member) = self
            .members
            .iter_mut()
            .find(|m| m.identity.token == identity.token)
        {
            member.connected = true;
            return Ok(());
        }
        let count = self.members.iter().filter(|m| m.player.is_some()).count();
        let player = (self.game.is_none() && count < self.settings.capacity).then_some(count);
        if player.is_some() {
            let used: Vec<_> = self
                .members
                .iter()
                .filter(|m| m.player.is_some())
                .map(|m| m.identity.color)
                .collect();
            if identity.color >= 6 || used.contains(&identity.color) {
                identity.color = (0..6).find(|c| !used.contains(c)).ok_or("房间已满")?;
            }
        }
        self.members.push(Member {
            identity,
            player,
            connected: true,
            ready: false,
        });
        Ok(())
    }

    pub fn disconnect(&mut self, token: &str) {
        if let Some(member) = self.members.iter_mut().find(|m| m.identity.token == token) {
            member.connected = false;
        }
    }

    pub fn apply(&mut self, token: &str, action: RoomAction, time: f64) -> Result<(), String> {
        let index = self
            .members
            .iter()
            .position(|m| m.identity.token == token)
            .ok_or("尚未加入这个房间")?;
        match action {
            RoomAction::Start => {
                if index != 0 {
                    return Err("由房主开始对局".into());
                }
                if self.game.is_some() {
                    return Err("对局已经开始".into());
                }
                let seats: Vec<_> = self
                    .members
                    .iter()
                    .filter(|m| m.player.is_some())
                    .map(|m| Seat {
                        name: m.identity.name.clone(),
                        color: m.identity.color,
                        connected: m.connected,
                        ready: m.ready,
                    })
                    .collect();
                if seats.iter().any(|seat| !seat.connected || !seat.ready) {
                    return Err("所有玩家准备就绪后即可开始".into());
                }
                self.game = Some(Game::new(
                    self.settings.mode,
                    &seats,
                    self.settings.starter,
                )?);
            }
            RoomAction::Game { action } => {
                let player = self.members[index]
                    .player
                    .ok_or("观战席可以查看对局和参与聊天")?;
                self.game
                    .as_mut()
                    .ok_or("对局尚未开始")?
                    .apply(player, action)?;
            }
            RoomAction::Configure { settings } => {
                if self.game.is_some() {
                    return Err("房间设置应用于开局前".into());
                }
                if index != 0 {
                    return Err("房间设置由房主修改".into());
                }
                let count = self.members.iter().filter(|m| m.player.is_some()).count();
                validate_settings(&settings, count)?;
                self.settings = settings;
                for (i, member) in self.members.iter_mut().enumerate() {
                    member.ready = i == 0;
                }
            }
            RoomAction::Profile { name, color } => {
                validate_name(&name)?;
                if color >= 6 {
                    return Err("请选择一种玩家颜色".into());
                }
                if self.members[index].player.is_some()
                    && self
                        .members
                        .iter()
                        .enumerate()
                        .any(|(i, m)| i != index && m.player.is_some() && m.identity.color == color)
                {
                    return Err("这个颜色已有玩家使用".into());
                }
                self.members[index].identity.name = name.trim().into();
                self.members[index].identity.color = color;
                if let Some(game) = &mut self.game
                    && let Some(player) = self.members[index].player
                {
                    game.players[player].name = name.trim().into();
                    game.players[player].color = color;
                }
            }
            RoomAction::Ready { ready } => {
                if self.members[index].player.is_none() {
                    return Err("观战席无需准备".into());
                }
                self.members[index].ready = ready;
            }
            RoomAction::Chat { text } => {
                let text = text.trim();
                if text.is_empty() || text.chars().count() > 1000 {
                    return Err("聊天内容需要在 1–1000 字之间".into());
                }
                self.chat.push(Chat {
                    name: self.members[index].identity.name.clone(),
                    text: text.into(),
                    time,
                });
                if self.chat.len() > 200 {
                    self.chat.remove(0);
                }
            }
        }
        Ok(())
    }

    pub fn view(&self, token: &str) -> RoomView {
        let viewer = self.members.iter().position(|m| m.identity.token == token);
        RoomView {
            id: self.id.clone(),
            settings: self.settings.clone(),
            seats: self
                .members
                .iter()
                .filter(|m| m.player.is_some())
                .map(|m| Seat {
                    name: m.identity.name.clone(),
                    color: m.identity.color,
                    connected: m.connected,
                    ready: m.ready,
                })
                .collect(),
            spectators: self
                .members
                .iter()
                .filter(|m| m.player.is_none() && m.connected)
                .map(|m| m.identity.name.clone())
                .collect(),
            you: viewer.and_then(|i| self.members[i].player),
            host: viewer == Some(0),
            chat: self.chat.clone(),
            game: self
                .game
                .as_ref()
                .map(|game| game.view(viewer.and_then(|i| self.members[i].player))),
        }
    }
}

fn validate_name(name: &str) -> Result<(), String> {
    let length = name.trim().chars().count();
    if !(1..=24).contains(&length) {
        return Err("玩家名称需要在 1–24 字之间".into());
    }
    Ok(())
}

fn validate_settings(settings: &RoomSettings, occupied: usize) -> Result<(), String> {
    if !(1..=48).contains(&settings.name.trim().chars().count()) {
        return Err("房间名称需要在 1–48 字之间".into());
    }
    if !(2..=6).contains(&settings.capacity) || occupied > settings.capacity {
        return Err("房间容纳 2–6 名玩家，人数应足够容纳已加入的玩家".into());
    }
    if settings.starter.is_some_and(|i| i >= occupied) {
        return Err("请选择已加入的玩家作为起始玩家".into());
    }
    Ok(())
}
