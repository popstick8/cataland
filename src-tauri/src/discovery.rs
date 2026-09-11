use std::{
    collections::HashMap,
    net::{SocketAddr, SocketAddrV6},
};

use cataland_core::{Connection, Mode, RoomInfo, room::Room};
use mdns_sd::{ResolvedService, ScopedIp, ServiceDaemon, ServiceEvent, ServiceInfo};
use tauri::{AppHandle, Emitter, Manager};

use crate::desktop::{self, Desktop};

const SERVICE: &str = "_cataland._tcp.local.";

pub fn advertise(daemon: &ServiceDaemon, room: &Room, port: u16) -> Result<String, String> {
    let properties = HashMap::from([
        ("id".to_owned(), room.id.clone()),
        ("name".to_owned(), room.settings.name.clone()),
        (
            "mode".to_owned(),
            if room.settings.mode == Mode::Base {
                "base"
            } else {
                "cities"
            }
            .to_owned(),
        ),
        (
            "players".to_owned(),
            room.members
                .iter()
                .filter(|m| m.player.is_some())
                .count()
                .to_string(),
        ),
        ("capacity".to_owned(), room.settings.capacity.to_string()),
        ("started".to_owned(), "false".to_owned()),
    ]);
    let service = ServiceInfo::new(
        SERVICE,
        &room.id,
        &format!("cataland-{}.local.", room.id),
        "",
        port,
        properties,
    )
    .map_err(|e| e.to_string())?
    .enable_addr_auto();
    let fullname = service.get_fullname().to_owned();
    daemon
        .register(service)
        .map_err(|e| format!("房间广播失败：{e}"))?;
    Ok(fullname)
}

fn room_info(service: &ResolvedService) -> Option<RoomInfo> {
    let mut addresses = Vec::new();
    for ip in service.get_addresses() {
        let address = match ip {
            ScopedIp::V4(_) => SocketAddr::new(ip.to_ip_addr(), service.port),
            ScopedIp::V6(ip) => {
                SocketAddrV6::new(*ip.addr(), service.port, 0, ip.scope_id().index).into()
            }
            _ => continue,
        };
        if !address.ip().is_loopback() {
            addresses.push(address);
        }
    }
    addresses.sort_by_key(|address| (address.is_ipv6(), *address));
    if addresses.is_empty() {
        return None;
    }
    Some(RoomInfo {
        id: service.get_property_val_str("id")?.into(),
        name: service.get_property_val_str("name")?.into(),
        addresses: addresses
            .into_iter()
            .map(|address| address.to_string())
            .collect(),
        players: service.get_property_val_str("players")?.parse().ok()?,
        capacity: service.get_property_val_str("capacity")?.parse().ok()?,
        mode: match service.get_property_val_str("mode")? {
            "base" => Mode::Base,
            "cities" => Mode::Cities,
            _ => return None,
        },
        started: service.get_property_val_str("started")? == "true",
    })
}

pub fn start(app: AppHandle) -> Result<(), String> {
    let events = app
        .state::<Desktop>()
        .daemon
        .browse(SERVICE)
        .map_err(|e| format!("房间发现失败：{e}"))?;
    tauri::async_runtime::spawn(async move {
        let mut services = HashMap::<String, RoomInfo>::new();
        while let Ok(event) = events.recv_async().await {
            let resolved = match event {
                ServiceEvent::ServiceResolved(service) => {
                    let Some(info) = room_info(&service) else {
                        continue;
                    };
                    services.insert(service.fullname.clone(), info.clone());
                    Some(info)
                }
                ServiceEvent::ServiceRemoved(_, fullname) => {
                    services.remove(&fullname);
                    None
                }
                _ => continue,
            };
            let result = update(&app, &services, resolved);
            if let Err(error) = result {
                eprintln!("LAN discovery: {error}");
            }
        }
    });
    Ok(())
}

fn update(
    app: &AppHandle,
    services: &HashMap<String, RoomInfo>,
    resolved: Option<RoomInfo>,
) -> Result<(), String> {
    let desktop = app.state::<Desktop>();
    let mut state = desktop.state.lock().map_err(|e| e.to_string())?;
    state.view.nearby = services.values().cloned().collect();
    state
        .view
        .nearby
        .sort_by(|a, b| a.name.cmp(&b.name).then(a.id.cmp(&b.id)));
    let current = resolved.filter(|info| {
        state
            .view
            .room
            .as_ref()
            .is_some_and(|room| room.id == info.id)
    });
    let reconnect = current
        .as_ref()
        .filter(|_| state.view.connection == Connection::Disconnected)
        .cloned();
    if let Some(info) = current {
        state.view.addresses = info.addresses;
    }
    app.emit("session", &state.view)
        .map_err(|e| e.to_string())?;
    let identity = state.view.identity.clone();
    drop(state);
    if let Some(info) = reconnect {
        desktop::join(
            info.addresses,
            identity.name,
            identity.color,
            app.clone(),
            desktop,
        )?;
    }
    Ok(())
}
