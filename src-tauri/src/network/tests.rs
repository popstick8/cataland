use std::{fs, time::Duration};

use cataland_core::{Mode, RoomSettings, game::Stage};
use uuid::Uuid;

use super::*;

async fn request(socket: &mut WebSocketStream<TcpStream>, request: Request) {
    socket
        .send(Message::text(serde_json::to_string(&request).unwrap()))
        .await
        .unwrap();
}

async fn state(
    socket: &mut WebSocketStream<TcpStream>,
    ready: impl Fn(&RoomView) -> bool,
) -> RoomView {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let message = socket.next().await.unwrap().unwrap();
            if let Message::Text(text) = message {
                match serde_json::from_str::<Response>(&text).unwrap() {
                    Response::State { room } if ready(&room) => return *room,
                    Response::State { .. } => {}
                    Response::Error { message } => panic!("{message}"),
                }
            }
        }
    })
    .await
    .expect("Expected room update")
}

#[tokio::test]
async fn guests_resume_their_game_after_the_host_reopens_it() {
    let directory = tempfile::tempdir().unwrap();
    fs::create_dir(directory.path().join("games")).unwrap();
    let identity = |name: &str, color| Identity {
        token: Uuid::new_v4().to_string(),
        name: name.into(),
        color,
    };
    let owner = identity("Host", 0);
    let guest = identity("Guest", 1);
    let observer = identity("Observer", 2);
    let room = Room::new(
        Uuid::new_v4().to_string(),
        RoomSettings {
            name: "LAN test".into(),
            mode: Mode::Cities,
            capacity: 2,
            starter: Some(0),
        },
        owner.clone(),
    )
    .unwrap();
    let daemon = ServiceDaemon::new().unwrap();
    let host = Host::start(room, daemon.clone(), directory.path()).unwrap();
    let (mut socket, _) = connect(&[format!("127.0.0.1:{}", host.port)])
        .await
        .unwrap();
    request(
        &mut socket,
        Request::Join {
            identity: guest.clone(),
        },
    )
    .await;
    assert_eq!(state(&mut socket, |_| true).await.you, Some(1));
    request(
        &mut socket,
        Request::Action {
            action: RoomAction::Ready { ready: true },
        },
    )
    .await;
    state(&mut socket, |room| room.seats[1].ready).await;
    host.apply(&owner.token, RoomAction::Start).unwrap();
    let mut view = state(&mut socket, |room| room.game.is_some()).await;
    while matches!(view.game.as_ref().unwrap().stage, Stage::Setup { .. }) {
        let game = view.game.as_ref().unwrap();
        let player = game.turn.player;
        let seq = game.events.last().map_or(0, |event| event.seq);
        let private = host
            .view(if player == 0 {
                &owner.token
            } else {
                &guest.token
            })
            .unwrap()
            .game
            .unwrap();
        let action = private
            .actions
            .iter()
            .max_by_key(|choice| match choice.target {
                Some(cataland_core::game::Target::Vertex(vertex)) => {
                    private.board.vertices[vertex].hexes.len()
                }
                _ => 0,
            })
            .unwrap()
            .action
            .clone();
        if player == 0 {
            host.apply(&owner.token, RoomAction::Game { action })
                .unwrap();
        } else {
            request(
                &mut socket,
                Request::Action {
                    action: RoomAction::Game { action },
                },
            )
            .await;
        }
        view = state(&mut socket, |room| {
            room.game
                .as_ref()
                .unwrap()
                .events
                .last()
                .is_some_and(|event| event.seq > seq)
        })
        .await;
    }
    let hand = view.game.as_ref().unwrap().private.as_ref().unwrap().hand;
    assert!(hand.iter().any(|&count| count > 0));
    assert_eq!(view.game.as_ref().unwrap().stage, Stage::Production);
    let (mut spectator, _) = connect(&[format!("[::1]:{}", host.port)]).await.unwrap();
    request(&mut spectator, Request::Join { identity: observer }).await;
    let public = state(&mut spectator, |_| true).await;
    assert_eq!(public.you, None);
    let game = public.game.unwrap();
    assert!(game.private.is_none());
    assert!(game.actions.is_empty());
    assert_eq!(public.seats.len(), 2);
    state(&mut socket, |room| !room.spectators.is_empty()).await;
    request(
        &mut socket,
        Request::Action {
            action: RoomAction::Start,
        },
    )
    .await;
    let response = tokio::time::timeout(Duration::from_secs(5), socket.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let Response::Error { message } = serde_json::from_str(response.to_text().unwrap()).unwrap()
    else {
        panic!("Expected rejected action")
    };
    assert_eq!(message.to_string(), "由房主开始对局");
    let mut changes = host.changes.subscribe();
    socket.close(None).await.unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
        while host.view(&owner.token).unwrap().seats[1].connected {
            changes.changed().await.unwrap();
        }
    })
    .await
    .unwrap();
    host.stop();
    tokio::time::timeout(Duration::from_secs(5), async {
        while let Some(message) = spectator.next().await {
            if message.is_err() || matches!(message, Ok(Message::Close(_))) {
                break;
            }
        }
    })
    .await
    .unwrap();
    let mut restored = storage::read::<Room>(&host.save).unwrap().unwrap();
    for member in &mut restored.members {
        member.connected = member.player == Some(0);
    }
    let resumed = Host::start(restored, daemon.clone(), directory.path()).unwrap();
    let (mut socket, _) = connect(&[format!("127.0.0.1:{}", resumed.port)])
        .await
        .unwrap();
    request(&mut socket, Request::Join { identity: guest }).await;
    let view = state(&mut socket, |_| true).await;
    assert_eq!(view.you, Some(1));
    assert_eq!(view.seats.len(), 2);
    let game = view.game.unwrap();
    assert_eq!(game.private.unwrap().hand, hand);
    assert_eq!(game.stage, Stage::Production);
    socket.close(None).await.unwrap();
    resumed.stop();
    daemon.shutdown().unwrap();
}
