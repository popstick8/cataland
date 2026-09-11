use std::{
    collections::HashMap,
    net::{Ipv6Addr, SocketAddr},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use cataland_core::{Identity, Request, Response, RoomAction, RoomView, room::Room};
use futures_util::{SinkExt, StreamExt};
use socket2::{Domain, Protocol, Socket, Type};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, watch},
};
use tokio_tungstenite::{WebSocketStream, accept_async, connect_async, tungstenite::Message};
use tokio_util::sync::CancellationToken;

use crate::desktop;

struct Hosted {
    room: Room,
    connections: HashMap<String, usize>,
}

pub struct Host {
    data: Mutex<Hosted>,
    pub changes: watch::Sender<()>,
    pub cancel: CancellationToken,
    pub port: u16,
}

impl Host {
    pub fn start(room: Room) -> Result<Arc<Self>, String> {
        let socket = Socket::new(Domain::IPV6, Type::STREAM, Some(Protocol::TCP))
            .map_err(|e| e.to_string())?;
        socket.set_only_v6(false).map_err(|e| e.to_string())?;
        socket.set_nonblocking(true).map_err(|e| e.to_string())?;
        socket
            .bind(&SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 0).into())
            .map_err(|e| format!("房间监听失败：{e}"))?;
        socket.listen(128).map_err(|e| e.to_string())?;
        let listener = TcpListener::from_std(socket.into()).map_err(|e| e.to_string())?;
        let port = listener.local_addr().map_err(|e| e.to_string())?.port();
        let connections = HashMap::from([(room.members[0].identity.token.clone(), 1)]);
        let host = Arc::new(Self {
            data: Mutex::new(Hosted { room, connections }),
            changes: watch::channel(()).0,
            cancel: CancellationToken::new(),
            port,
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
                            server.cancel.cancel();
                            break;
                        }
                    }
                }
            }
        });
        Ok(host)
    }

    pub fn view(&self, token: &str) -> Result<RoomView, String> {
        Ok(self
            .data
            .lock()
            .map_err(|e| e.to_string())?
            .room
            .view(token))
    }

    pub fn apply(&self, token: &str, action: RoomAction) -> Result<(), String> {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs_f64()
            * 1000.0;
        self.data
            .lock()
            .map_err(|e| e.to_string())?
            .room
            .apply(token, action, time)?;
        self.changes.send_replace(());
        Ok(())
    }

    fn attach(&self, identity: Identity) -> Result<(), String> {
        let mut data = self.data.lock().map_err(|e| e.to_string())?;
        let token = identity.token.clone();
        data.room.join(identity)?;
        *data.connections.entry(token).or_default() += 1;
        self.changes.send_replace(());
        Ok(())
    }

    fn detach(&self, token: &str) -> Result<(), String> {
        let mut data = self.data.lock().map_err(|e| e.to_string())?;
        if let Some(count) = data.connections.get_mut(token) {
            *count -= 1;
            if *count == 0 {
                data.connections.remove(token);
                data.room.disconnect(token);
            }
        }
        self.changes.send_replace(());
        Ok(())
    }

    async fn serve(&self, stream: TcpStream) -> Result<(), String> {
        let mut socket = tokio::select! {
            () = self.cancel.cancelled() => return Ok(()),
            result = accept_async(stream) => result.map_err(|e| e.to_string())?,
        };
        let first = tokio::select! {
            () = self.cancel.cancelled() => return Ok(()),
            message = socket.next() => message.ok_or("连接已关闭")?.map_err(|e| e.to_string())?,
        };
        let Request::Join { identity } =
            serde_json::from_str(first.to_text().map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?
        else {
            return Err("请先加入房间".into());
        };
        let token = identity.token.clone();
        let mut changes = self.changes.subscribe();
        if let Err(message) = self.attach(identity) {
            send(&mut socket, &Response::Error { message }).await?;
            return Ok(());
        }
        let result = self.exchange(&mut socket, &token, &mut changes).await;
        self.detach(&token)?;
        result
    }

    async fn exchange(
        &self,
        socket: &mut WebSocketStream<TcpStream>,
        token: &str,
        changes: &mut watch::Receiver<()>,
    ) -> Result<(), String> {
        changes.borrow_and_update();
        send(
            socket,
            &Response::State {
                room: self.view(token)?,
            },
        )
        .await?;
        loop {
            tokio::select! {
                () = self.cancel.cancelled() => return Ok(()),
                changed = changes.changed() => {
                    changed.map_err(|e| e.to_string())?;
                    let response = Response::State { room: self.view(token)? };
                    send(socket, &response).await?;
                }
                incoming = socket.next() => {
                    let Some(message) = incoming else { return Ok(()) };
                    match message.map_err(|e| e.to_string())? {
                        Message::Text(text) => {
                            let result = serde_json::from_str::<Request>(&text).map_err(|e| e.to_string()).and_then(|request| match request {
                                Request::Action { action } => self.apply(token, action),
                                Request::Join { .. } => Err("这个连接已经加入房间".into()),
                            });
                            if let Err(message) = result {
                                send(socket, &Response::Error { message }).await?;
                            }
                        }
                        Message::Close(_) => return Ok(()),
                        Message::Ping(_) => socket.flush().await.map_err(|e| e.to_string())?,
                        _ => {}
                    }
                }
            }
        }
    }
}

async fn send(socket: &mut WebSocketStream<TcpStream>, response: &Response) -> Result<(), String> {
    let text = serde_json::to_string(response).map_err(|e| e.to_string())?;
    socket
        .send(Message::text(text))
        .await
        .map_err(|e| e.to_string())
}

pub async fn guest(
    app: tauri::AppHandle,
    address: String,
    identity: Identity,
    mut outgoing: mpsc::UnboundedReceiver<Request>,
    cancel: CancellationToken,
) {
    let result = async {
        let url = if address.starts_with("ws://") { address } else { format!("ws://{address}") };
        let (mut socket, _) = tokio::select! {
            () = cancel.cancelled() => return Ok(()),
            result = connect_async(url) => result.map_err(|e| format!("连接房间失败：{e}"))?,
        };
        socket.send(Message::text(serde_json::to_string(&Request::Join { identity }).map_err(|e| e.to_string())?)).await.map_err(|e| e.to_string())?;
        loop {
            tokio::select! {
                () = cancel.cancelled() => return Ok(()),
                request = outgoing.recv() => {
                    let Some(request) = request else { return Ok(()) };
                    socket.send(Message::text(serde_json::to_string(&request).map_err(|e| e.to_string())?)).await.map_err(|e| e.to_string())?;
                }
                incoming = socket.next() => {
                    let Some(message) = incoming else { return Err("房主已关闭连接".into()) };
                    match message.map_err(|e| e.to_string())? {
                        Message::Text(text) => match serde_json::from_str::<Response>(&text).map_err(|e| e.to_string())? {
                            Response::State { room } => desktop::receive(&app, &cancel, room)?,
                            Response::Error { message } => desktop::notice(&app, &cancel, message)?,
                        },
                        Message::Close(_) => return Err("房主已关闭连接".into()),
                        Message::Ping(_) => socket.flush().await.map_err(|e| e.to_string())?,
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
