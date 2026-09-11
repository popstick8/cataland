use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use cataland_core::{
    ClientView, Connection, Identity, Request, RoomAction, RoomInfo, RoomSettings, RoomView,
    room::Room,
};
use mdns_sd::ServiceDaemon;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{
    network::{self, Host},
    storage,
};

pub struct Desktop {
    pub directory: PathBuf,
    pub daemon: ServiceDaemon,
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
        match &self.outgoing {
            Outgoing::Local(host) => host.stop(),
            Outgoing::Remote(_) => self.cancel.cancel(),
        }
    }
}

impl Desktop {
    pub fn new(directory: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        fs::create_dir_all(directory.join("games"))?;
        let identity =
            storage::read(&directory.join("identity.json"))?.unwrap_or_else(|| Identity {
                token: Uuid::new_v4().to_string(),
                name: "旅人".into(),
                color: 0,
            });
        let recent = storage::read(&directory.join("recent.json"))?.unwrap_or_default();
        let saves = storage::games(&directory)?;
        let preferences = storage::read(&directory.join("preferences.json"))?.unwrap_or_default();
        storage::write(&directory.join("identity.json"), &identity)?;
        Ok(Self {
            directory,
            daemon: ServiceDaemon::new()?,
            state: Mutex::new(Session {
                view: ClientView {
                    preferences,
                    identity,
                    room: None,
                    connection: Connection::Home,
                    addresses: Vec::new(),
                    nearby: Vec::new(),
                    recent,
                    saves,
                },
                link: None,
            }),
        })
    }

    fn save_identity(&self, identity: &Identity) -> Result<(), String> {
        storage::write(&self.directory.join("identity.json"), identity)
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
    if !room.host {
        let recent = RoomInfo {
            id: room.id.clone(),
            name: room.settings.name.clone(),
            addresses: state.view.addresses.clone(),
            players: room.seats.len(),
            capacity: room.settings.capacity,
            mode: room.settings.mode,
            started: room.game.is_some(),
        };
        if state.view.recent.first() != Some(&recent) {
            state.view.recent.retain(|info| info.id != recent.id);
            state.view.recent.insert(0, recent);
            state.view.recent.truncate(12);
            storage::write(&desktop.directory.join("recent.json"), &state.view.recent)?;
        }
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
    let token = desktop
        .state
        .lock()
        .map_err(|e| e.to_string())?
        .view
        .identity
        .token
        .clone();
    let identity = Identity {
        name: name.trim().into(),
        color,
        token,
    };
    let room = Room::new(Uuid::new_v4().to_string(), settings, identity)?;
    open_room(room, app, &desktop)
}

#[tauri::command]
pub async fn resume(id: String, app: AppHandle, desktop: State<'_, Desktop>) -> Result<(), String> {
    let path = storage::game_path(&desktop.directory, &id)?;
    let mut room = storage::read::<Room>(&path)?.ok_or("找不到这份存档")?;
    for member in &mut room.members {
        member.connected = false;
    }
    room.members
        .first_mut()
        .ok_or("存档中缺少房主席位")?
        .connected = true;
    open_room(room, app, &desktop)
}

fn open_room(room: Room, app: AppHandle, desktop: &Desktop) -> Result<(), String> {
    let identity = room
        .members
        .first()
        .ok_or("房间中缺少房主席位")?
        .identity
        .clone();
    let mut state = desktop.state.lock().map_err(|e| e.to_string())?;
    desktop.save_identity(&identity)?;
    let room_id = room.id.clone();
    let host = Host::start(room, desktop.daemon.clone(), &desktop.directory)?;
    state.link = Some(Link {
        outgoing: Outgoing::Local(host.clone()),
        cancel: host.cancel.clone(),
    });
    state.view.identity = identity.clone();
    state.view.saves = storage::games(&desktop.directory)?;
    state.view.addresses = vec![format!("cataland-{room_id}.local.:{}", host.port)];
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
    addresses: Vec<String>,
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
    let addresses: Vec<_> = addresses
        .into_iter()
        .map(|address| {
            address
                .trim()
                .trim_start_matches("ws://")
                .trim_end_matches('/')
                .to_owned()
        })
        .filter(|address| !address.is_empty())
        .collect();
    if addresses.is_empty() {
        return Err("请输入主机地址与端口".into());
    }
    desktop.save_identity(&identity)?;
    let (sender, receiver) = mpsc::unbounded_channel();
    let cancel = CancellationToken::new();
    state.link = Some(Link {
        outgoing: Outgoing::Remote(sender),
        cancel: cancel.clone(),
    });
    state.view.identity = identity.clone();
    state.view.connection = Connection::Connecting;
    state.view.addresses = addresses.clone();
    app.emit("session", &state.view)
        .map_err(|e| e.to_string())?;
    tauri::async_runtime::spawn(network::guest(app, addresses, identity, receiver, cancel));
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
    state.view.addresses.clear();
    state.view.saves = storage::games(&desktop.directory)?;
    app.emit("session", &state.view).map_err(|e| e.to_string())
}
