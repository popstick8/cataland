use std::{
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use cataland_core::{RoomSettings, SavedGame, Text, room::Member};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;

pub fn read<T: DeserializeOwned>(path: &Path) -> Result<Option<T>, Text> {
    match fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map(Some).map_err(|e| {
            cataland_core::text!(
                "无法读取 {0}：{1}",
                path.display().to_string(),
                e.to_string()
            )
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(cataland_core::text!(
            "无法读取 {0}：{1}",
            path.display().to_string(),
            error.to_string()
        )),
    }
}

pub fn write<T: Serialize>(path: &Path, value: &T) -> Result<(), Text> {
    let bytes = serde_json::to_vec(value).map_err(|e| Text::from(e.to_string()))?;
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, bytes).map_err(|e| {
        cataland_core::text!(
            "无法保存 {0}：{1}",
            path.display().to_string(),
            e.to_string()
        )
    })?;
    fs::rename(temporary, path).map_err(|e| {
        cataland_core::text!(
            "无法保存 {0}：{1}",
            path.display().to_string(),
            e.to_string()
        )
    })
}

pub fn game_path(directory: &Path, id: &str) -> Result<PathBuf, Text> {
    let id = Uuid::parse_str(id).map_err(|_| "对局标识无效")?;
    Ok(directory.join("games").join(format!("{id}.json")))
}

#[derive(Deserialize)]
struct Summary {
    id: String,
    settings: RoomSettings,
    members: Vec<Member>,
    game: Option<Outcome>,
}

#[derive(Deserialize)]
struct Outcome {
    winner: Option<usize>,
}

pub fn games(directory: &Path) -> Result<Vec<SavedGame>, Text> {
    let mut games = Vec::new();
    for entry in fs::read_dir(directory.join("games"))
        .map_err(|e| cataland_core::text!("无法读取存档目录：{0}", e.to_string()))?
    {
        let entry = entry.map_err(|e| Text::from(e.to_string()))?;
        if entry
            .path()
            .extension()
            .is_none_or(|extension| extension != "json")
        {
            continue;
        }
        let Some(room) = read::<Summary>(&entry.path())? else {
            continue;
        };
        let modified = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .map_err(|e| Text::from(e.to_string()))?;
        games.push(SavedGame {
            finished: room.game.is_some_and(|game| game.winner.is_some()),
            id: room.id,
            name: room.settings.name,
            mode: room.settings.mode,
            players: room
                .members
                .into_iter()
                .filter(|member| member.player.is_some())
                .map(|member| member.identity.name)
                .collect(),
            updated: modified
                .duration_since(UNIX_EPOCH)
                .map_err(|e| Text::from(e.to_string()))?
                .as_secs_f64()
                * 1000.0,
        });
    }
    games.sort_by(|a, b| b.updated.total_cmp(&a.updated));
    Ok(games)
}
