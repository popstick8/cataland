use std::{
    collections::HashMap,
    net::{Ipv6Addr, SocketAddr},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use cataland_core::{Cursor, Identity, Request, Response, RoomAction, Text, room::Room};
use futures_util::{SinkExt, StreamExt};
use mdns_sd::ServiceDaemon;
use socket2::{Domain, Protocol, Socket, Type};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, watch},
};
use tokio_tungstenite::{WebSocketStream, accept_async, client_async, tungstenite::Message};
use tokio_util::sync::CancellationToken;

use crate::{desktop, discovery, storage};

#[cfg(test)]
mod tests;

struct Hosted {
    room: Room,
    revision: u64,
    connections: HashMap<String, usize>,
}

pub struct Host {
    data: Mutex<Hosted>,
    pub changes: watch::Sender<()>,
    pub cancel: CancellationToken,
    pub port: u16,
    daemon: ServiceDaemon,
    fullname: String,
    save: PathBuf,
}

impl Host {
    pub fn start(
        room: Room,
        daemon: ServiceDaemon,
        directory: &std::path::Path,
    ) -> Result<Arc<Self>, Text> {
        let socket = Socket::new(Domain::IPV6, Type::STREAM, Some(Protocol::TCP))
            .map_err(|e| Text::from(e.to_string()))?;
        socket
            .set_only_v6(false)
            .map_err(|e| Text::from(e.to_string()))?;
        socket
            .set_nonblocking(true)
            .map_err(|e| Text::from(e.to_string()))?;
        socket
            .bind(&SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 0).into())
            .map_err(|e| cataland_core::text!("房间监听失败：{0}", e.to_string()))?;
        socket.listen(128).map_err(|e| Text::from(e.to_string()))?;
        let listener =
            TcpListener::from_std(socket.into()).map_err(|e| Text::from(e.to_string()))?;
        let port = listener
            .local_addr()
            .map_err(|e| Text::from(e.to_string()))?
            .port();
        let save = storage::game_path(directory, &room.id)?;
        storage::write(&save, &room)?;
        let fullname = discovery::advertise(&daemon, &room, port)?;
        let connections = HashMap::from([(room.members[0].identity.token.clone(), 1)]);
        let host = Arc::new(Self {
            data: Mutex::new(Hosted {
                room,
                connections,
                revision: 0,
            }),
            changes: watch::channel(()).0,
            cancel: CancellationToken::new(),
            port,
            daemon,
            fullname,
            save,
        });
        let server = host.clone();
        tauri::async_runtime::spawn(async move {
            loop {
                tokio::select! {
                    () = server.cancel.cancelled() => break,
                    incoming = listener.accept() => match incoming {
                        Ok((stream, _)) => {
                            let peer = server.clone();
                            tauri::async_runtime::spawn(async move {
                                if let Err(error) = peer.serve(stream).await {
                                    eprintln!("Room connection: {error}");
                                }
                            });
                        }
                        Err(error) => {
                            eprintln!("Room listener: {error}");
                            server.stop();
                            break;
                        }
                    }
                }
            }
        });
        Ok(host)
    }

    pub fn stop(&self) {
        let _data = self
            .data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !self.cancel.is_cancelled() {
            self.cancel.cancel();
            if let Err(error) = self.daemon.unregister(&self.fullname) {
                eprintln!("Room advertisement removal: {error}");
            }
        }
    }

    fn announce(&self) {
        let result = self
            .data
            .lock()
            .map_err(|e| Text::from(e.to_string()))
            .and_then(|data| discovery::advertise(&self.daemon, &data.room, self.port));
        if let Err(error) = result {
            eprintln!("Room advertisement: {error}");
        }
    }

    #[cfg(test)]
    pub fn view(&self, token: &str) -> Result<cataland_core::RoomView, Text> {
        Ok(self
            .data
            .lock()
            .map_err(|e| Text::from(e.to_string()))?
            .room
            .view(token, 0))
    }

    pub fn updates(
        &self,
        token: &str,
        cursor: &mut Cursor,
        revision: &mut Option<u64>,
    ) -> Result<Vec<Response>, Text> {
        let data = self.data.lock().map_err(|e| Text::from(e.to_string()))?;
        if cursor.room != data.room.id {
            *cursor = Cursor {
                room: data.room.id.clone(),
                ..Cursor::default()
            };
        }
        let mut updates = Vec::new();
        if *revision != Some(data.revision) {
            let mut room = data.room.view(token, cursor.events);
            room.chat.clear();
            cursor.events = data
                .room
                .game
                .as_ref()
                .map_or(0, |game| game.events.len() as u64);
            updates.push(Response::State {
                room: Box::new(room),
            });
            *revision = Some(data.revision);
        }
        if cursor.chat < data.room.chat.len() {
            updates.push(Response::Chat {
                room: data.room.id.clone(),
                offset: cursor.chat,
                messages: data.room.chat[cursor.chat..].to_vec(),
            });
            cursor.chat = data.room.chat.len();
        }
        Ok(updates)
    }

    pub fn apply(&self, token: &str, action: RoomAction) -> Result<(), Text> {
        let changed = !matches!(action, RoomAction::Chat { .. });
        let announce = matches!(action, RoomAction::Configure { .. } | RoomAction::Start);
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Text::from(e.to_string()))?
            .as_millis() as f64;
        let mut data = self.data.lock().map_err(|e| Text::from(e.to_string()))?;
        if self.cancel.is_cancelled() {
            return Err("房间已经关闭".into());
        }
        let mut room = data.room.clone();
        room.apply(token, action, time)?;
        storage::write(&self.save, &room)?;
        data.room = room;
        data.revision += u64::from(changed);
        drop(data);
        self.changes.send_replace(());
        if announce {
            self.announce();
        }
        Ok(())
    }

    fn attach(&self, identity: Identity) -> Result<(), Text> {
        let mut data = self.data.lock().map_err(|e| Text::from(e.to_string()))?;
        let token = identity.token.clone();
        if self.cancel.is_cancelled() {
            return Err("房间已经关闭".into());
        }
        let mut room = data.room.clone();
        room.join(identity)?;
        storage::write(&self.save, &room)?;
        data.room = room;
        *data.connections.entry(token).or_default() += 1;
        data.revision += 1;
        drop(data);
        self.announce();
        self.changes.send_replace(());
        Ok(())
    }

    fn detach(&self, token: &str) -> Result<(), Text> {
        let mut data = self.data.lock().map_err(|e| Text::from(e.to_string()))?;
        if self.cancel.is_cancelled() {
            return Ok(());
        }
        if let Some(count) = data.connections.get_mut(token) {
            *count -= 1;
            if *count == 0 {
                data.connections.remove(token);
                data.room.disconnect(token);
                data.revision += 1;
            }
        }
        self.changes.send_replace(());
        storage::write(&self.save, &data.room)
    }

    async fn serve(&self, stream: TcpStream) -> Result<(), Text> {
        let mut socket = tokio::select! {
            () = self.cancel.cancelled() => return Ok(()),
            result = accept_async(stream) => result.map_err(|e| Text::from(e.to_string()))?,
        };
        let first = tokio::select! {
            () = self.cancel.cancelled() => return Ok(()),
            message = socket.next() => message.ok_or("连接已关闭")?.map_err(|e| Text::from(e.to_string()))?,
        };
        let Request::Join { identity, cursor } =
            serde_json::from_str(first.to_text().map_err(|e| Text::from(e.to_string()))?)
                .map_err(|e| Text::from(e.to_string()))?
        else {
            return Err("请先加入房间".into());
        };
        let token = identity.token.clone();
        let mut changes = self.changes.subscribe();
        if let Err(message) = self.attach(identity) {
            send(&mut socket, &Response::Error { message }).await?;
            return Ok(());
        }
        let result = self
            .exchange(
                &mut socket,
                &token,
                &mut changes,
                cursor.unwrap_or_default(),
            )
            .await;
        self.detach(&token)?;
        result
    }

    async fn exchange(
        &self,
        socket: &mut WebSocketStream<TcpStream>,
        token: &str,
        changes: &mut watch::Receiver<()>,
        mut cursor: Cursor,
    ) -> Result<(), Text> {
        let mut revision = None;
        changes.borrow_and_update();
        for response in self.updates(token, &mut cursor, &mut revision)? {
            send(socket, &response).await?;
        }
        loop {
            tokio::select! {
                () = self.cancel.cancelled() => return Ok(()),
                changed = changes.changed() => {
                    changed.map_err(|e| Text::from(e.to_string()))?;
                    changes.borrow_and_update();
                    for response in self.updates(token, &mut cursor, &mut revision)? {
                        send(socket, &response).await?;
                    }
                }
                incoming = socket.next() => {
                    let Some(message) = incoming else { return Ok(()) };
                    match message.map_err(|e| Text::from(e.to_string()))? {
                        Message::Text(text) => {
                            let result = serde_json::from_str::<Request>(&text).map_err(|e| Text::from(e.to_string())).and_then(|request| match request {
                                Request::Action { action } => self.apply(token, action),
                                Request::Join { .. } => Err("这个连接已经加入房间".into()),
                            });
                            if let Err(message) = result {
                                send(socket, &Response::Error { message }).await?;
                            }
                        }
                        Message::Close(_) => return Ok(()),
                        Message::Ping(_) => socket.flush().await.map_err(|e| Text::from(e.to_string()))?,
                        _ => {}
                    }
                }
            }
        }
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        self.stop();
    }
}

async fn send(socket: &mut WebSocketStream<TcpStream>, response: &Response) -> Result<(), Text> {
    let text = serde_json::to_string(response).map_err(|e| Text::from(e.to_string()))?;
    socket
        .send(Message::text(text))
        .await
        .map_err(|e| Text::from(e.to_string()))
}

pub async fn guest(
    app: tauri::AppHandle,
    addresses: Vec<String>,
    identity: Identity,
    cursor: Option<Cursor>,
    mut outgoing: mpsc::UnboundedReceiver<Request>,
    cancel: CancellationToken,
) {
    let result = async {
        let (mut socket, _) = tokio::select! {
            () = cancel.cancelled() => return Ok(()),
            result = connect(&addresses) => result?,
        };
        socket.send(Message::text(serde_json::to_string(&Request::Join { identity, cursor }).map_err(|e| Text::from(e.to_string()))?)).await.map_err(|e| Text::from(e.to_string()))?;
        loop {
            tokio::select! {
                () = cancel.cancelled() => return Ok(()),
                request = outgoing.recv() => {
                    let Some(request) = request else { return Ok(()) };
                    socket.send(Message::text(serde_json::to_string(&request).map_err(|e| Text::from(e.to_string()))?)).await.map_err(|e| Text::from(e.to_string()))?;
                }
                incoming = socket.next() => {
                    let Some(message) = incoming else { return Err("房主已关闭连接".into()) };
                    match message.map_err(|e| Text::from(e.to_string()))? {
                        Message::Text(text) => desktop::receive(&app, &cancel, serde_json::from_str::<Response>(&text).map_err(|e| Text::from(e.to_string()))?)?,
                        Message::Close(_) => return Err("房主已关闭连接".into()),
                        Message::Ping(_) => socket.flush().await.map_err(|e| Text::from(e.to_string()))?,
                        _ => {}
                    }
                }
            }
        }
    }.await;
    if let Err(error) = desktop::disconnected(&app, &cancel, result.err()) {
        eprintln!("Connection state: {error}");
    }
}

async fn connect(
    addresses: &[String],
) -> Result<
    (
        WebSocketStream<TcpStream>,
        tokio_tungstenite::tungstenite::handshake::client::Response,
    ),
    Text,
> {
    let mut targets = Vec::new();
    for address in addresses {
        targets.extend(
            tokio::net::lookup_host(address)
                .await
                .map_err(|e| cataland_core::text!("主机地址无法解析：{0}", e.to_string()))?,
        );
    }
    let stream = TcpStream::connect(targets.as_slice())
        .await
        .map_err(|e| cataland_core::text!("连接房间失败：{0}", e.to_string()))?;
    client_async("ws://cataland/", stream)
        .await
        .map_err(|e| cataland_core::text!("房间连接失败：{0}", e.to_string()))
}
