use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use cataland_core::{
    ClientView, Connection, Identity, Request, RoomAction, RoomSettings, RoomView, room::Room,
};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::network::{self, Host};

pub struct Desktop {
    pub directory: PathBuf,
    pub state: Mutex<Session>,
}

pub struct Session {
    pub view: ClientView,
    link: Option<Link>,
}

struct Link {
    outgoing: Outgoing,
    cancel: CancellationToken,
}

enum Outgoing {
    Local(Arc<Host>),
    Remote(mpsc::UnboundedSender<Request>),
}

impl Drop for Link {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
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
                view: ClientView {
                    identity,
                    room: None,
                    connection: Connection::Home,
                    address: String::new(),
                },
                link: None,
            }),
        };
        desktop.save_identity(
            &desktop
                .state
                .lock()
                .map_err(|e| e.to_string())?
                .view
                .identity,
        )?;
        Ok(desktop)
    }

    fn save_identity(&self, identity: &Identity) -> Result<(), String> {
        let data = serde_json::to_vec(identity).map_err(|e| e.to_string())?;
        let temporary = self.directory.join("identity.tmp");
        fs::write(&temporary, data).map_err(|e| format!("玩家身份保存失败：{e}"))?;
        fs::rename(temporary, self.directory.join("identity.json"))
            .map_err(|e| format!("玩家身份保存失败：{e}"))
    }
}

pub fn receive(app: &AppHandle, cancel: &CancellationToken, room: RoomView) -> Result<(), String> {
    let desktop = app.state::<Desktop>();
    let mut state = desktop.state.lock().map_err(|e| e.to_string())?;
    if cancel.is_cancelled() {
        return Ok(());
    }
    if let Some(you) = room.you.and_then(|i| room.seats.get(i))
        && (you.name != state.view.identity.name || you.color != state.view.identity.color)
    {
        state.view.identity.name = you.name.clone();
        state.view.identity.color = you.color;
        desktop.save_identity(&state.view.identity)?;
    }
    state.view.room = Some(room);
    state.view.connection = Connection::Connected;
    app.emit("session", &state.view).map_err(|e| e.to_string())
}

pub fn notice(app: &AppHandle, cancel: &CancellationToken, message: String) -> Result<(), String> {
    let desktop = app.state::<Desktop>();
    let _state = desktop.state.lock().map_err(|e| e.to_string())?;
    if !cancel.is_cancelled() {
        app.emit("notice", message).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn disconnected(
    app: &AppHandle,
    cancel: &CancellationToken,
    error: Option<String>,
) -> Result<(), String> {
    let desktop = app.state::<Desktop>();
    let mut state = desktop.state.lock().map_err(|e| e.to_string())?;
    if !cancel.is_cancelled() {
        state.view.connection = Connection::Disconnected;
        app.emit("session", &state.view)
            .map_err(|e| e.to_string())?;
        if let Some(message) = error {
            app.emit("notice", message).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn session(desktop: State<'_, Desktop>) -> Result<ClientView, String> {
    Ok(desktop
        .state
        .lock()
        .map_err(|e| e.to_string())?
        .view
        .clone())
}

#[tauri::command]
pub async fn host(
    settings: RoomSettings,
    name: String,
    color: usize,
    app: AppHandle,
    desktop: State<'_, Desktop>,
) -> Result<(), String> {
    let mut state = desktop.state.lock().map_err(|e| e.to_string())?;
    let identity = Identity {
        name: name.trim().into(),
        color,
        token: state.view.identity.token.clone(),
    };
    let room = Room::new(Uuid::new_v4().to_string(), settings, identity.clone())?;
    desktop.save_identity(&identity)?;
    let host = Host::start(room)?;
    state.link = Some(Link {
        outgoing: Outgoing::Local(host.clone()),
        cancel: host.cancel.clone(),
    });
    state.view.identity = identity.clone();
    state.view.address = format!("[::1]:{}", host.port);
    state.view.connection = Connection::Connected;
    let mut changes = host.changes.subscribe();
    state.view.room = Some(host.view(&identity.token)?);
    app.emit("session", &state.view)
        .map_err(|e| e.to_string())?;
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::select! {
                () = host.cancel.cancelled() => break,
                changed = changes.changed() => {
                    if changed.is_err() { break; }
                    let result = host.view(&identity.token).and_then(|room| receive(&app, &host.cancel, room));
                    if let Err(error) = result { eprintln!("Local room state: {error}"); }
                }
            }
        }
    });
    Ok(())
}

#[tauri::command]
pub fn join(
    address: String,
    name: String,
    color: usize,
    app: AppHandle,
    desktop: State<'_, Desktop>,
) -> Result<(), String> {
    let mut state = desktop.state.lock().map_err(|e| e.to_string())?;
    let identity = Identity {
        token: state.view.identity.token.clone(),
        name: name.trim().into(),
        color,
    };
    if !(1..=24).contains(&identity.name.chars().count()) || color >= 6 {
        return Err("请输入 1–24 字的名称并选择玩家颜色".into());
    }
    desktop.save_identity(&identity)?;
    let address = address.trim().to_owned();
    let (sender, receiver) = mpsc::unbounded_channel();
    let cancel = CancellationToken::new();
    state.link = Some(Link {
        outgoing: Outgoing::Remote(sender),
        cancel: cancel.clone(),
    });
    state.view.identity = identity.clone();
    state.view.connection = Connection::Connecting;
    state.view.address = address.clone();
    app.emit("session", &state.view)
        .map_err(|e| e.to_string())?;
    tauri::async_runtime::spawn(network::guest(app, address, identity, receiver, cancel));
    Ok(())
}

#[tauri::command]
pub fn room_action(action: RoomAction, desktop: State<'_, Desktop>) -> Result<(), String> {
    let state = desktop.state.lock().map_err(|e| e.to_string())?;
    if state.view.connection != Connection::Connected {
        return Err("房间连接尚未恢复".into());
    }
    match &state.link.as_ref().ok_or("请先进入房间")?.outgoing {
        Outgoing::Local(host) => host.apply(&state.view.identity.token, action),
        Outgoing::Remote(sender) => sender
            .send(Request::Action { action })
            .map_err(|_| "房间连接已断开".into()),
    }
}

#[tauri::command]
pub fn leave(app: AppHandle, desktop: State<'_, Desktop>) -> Result<(), String> {
    let mut state = desktop.state.lock().map_err(|e| e.to_string())?;
    state.link = None;
    state.view.room = None;
    state.view.connection = Connection::Home;
    state.view.address.clear();
    app.emit("session", &state.view).map_err(|e| e.to_string())
}
