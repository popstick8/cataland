use std::{
    fs,
    path::PathBuf,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use cataland_core::{ClientView, Identity, RoomAction, RoomSettings, room::Room};
use tauri::State;
use uuid::Uuid;

pub struct Desktop {
    pub directory: PathBuf,
    pub state: Mutex<Session>,
}

pub struct Session {
    pub identity: Identity,
    pub host: Option<Room>,
}

impl Desktop {
    pub fn new(directory: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        fs::create_dir_all(&directory)?;
        let identity = match fs::read(directory.join("identity.json")) {
            Ok(bytes) => serde_json::from_slice(&bytes)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Identity {
                token: Uuid::new_v4().to_string(),
                name: "旅人".into(),
                color: 0,
            },
            Err(error) => return Err(error.into()),
        };
        let desktop = Self {
            directory,
            state: Mutex::new(Session {
                identity,
                host: None,
            }),
        };
        desktop.save_identity(&desktop.state.lock().map_err(|e| e.to_string())?.identity)?;
        Ok(desktop)
    }

    pub fn save_identity(&self, identity: &Identity) -> Result<(), String> {
        let data = serde_json::to_vec(identity).map_err(|e| e.to_string())?;
        let temporary = self.directory.join("identity.tmp");
        fs::write(&temporary, data).map_err(|e| format!("玩家身份保存失败：{e}"))?;
        fs::rename(temporary, self.directory.join("identity.json"))
            .map_err(|e| format!("玩家身份保存失败：{e}"))
    }
}

impl Session {
    pub fn view(&self) -> ClientView {
        ClientView {
            identity: self.identity.clone(),
            room: self
                .host
                .as_ref()
                .map(|room| room.view(&self.identity.token)),
        }
    }
}

#[tauri::command]
pub fn session(desktop: State<'_, Desktop>) -> Result<ClientView, String> {
    Ok(desktop.state.lock().map_err(|e| e.to_string())?.view())
}

#[tauri::command]
pub fn host(
    settings: RoomSettings,
    name: String,
    color: usize,
    desktop: State<'_, Desktop>,
) -> Result<ClientView, String> {
    let mut state = desktop.state.lock().map_err(|e| e.to_string())?;
    let identity = Identity {
        name: name.trim().into(),
        color,
        token: state.identity.token.clone(),
    };
    let room = Room::new(Uuid::new_v4().to_string(), settings, identity.clone())?;
    desktop.save_identity(&identity)?;
    state.identity = identity;
    state.host = Some(room);
    Ok(state.view())
}

#[tauri::command]
pub fn room_action(action: RoomAction, desktop: State<'_, Desktop>) -> Result<ClientView, String> {
    let mut state = desktop.state.lock().map_err(|e| e.to_string())?;
    let token = state.identity.token.clone();
    let room = state.host.as_mut().ok_or("请先进入房间")?;
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs_f64()
        * 1000.0;
    room.apply(&token, action, time)?;
    let identity = room.members[0].identity.clone();
    desktop.save_identity(&identity)?;
    state.identity = identity;
    Ok(state.view())
}

#[tauri::command]
pub fn leave(desktop: State<'_, Desktop>) -> Result<ClientView, String> {
    let mut state = desktop.state.lock().map_err(|e| e.to_string())?;
    state.host = None;
    Ok(state.view())
}
