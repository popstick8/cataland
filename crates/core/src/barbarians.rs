use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    board::Terrain,
    cities::Track,
    game::{BuildingKind, Effect, Game, Target},
    view::{Pick, Prompt},
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum EventDie {
    Barbarians,
    Trade,
    Politics,
    Science,
}

impl EventDie {
    pub fn track(self) -> Option<Track> {
        match self {
            Self::Barbarians => None,
            Self::Trade => Some(Track::Trade),
            Self::Politics => Some(Track::Politics),
            Self::Science => Some(Track::Science),
        }
    }
}

impl Game {
    pub fn event_die(&mut self, red: u8) {
        let event = match fastrand::u8(..6) {
            0..=2 => EventDie::Barbarians,
            3 => EventDie::Trade,
            4 => EventDie::Politics,
            _ => EventDie::Science,
        };
        let Some(cities) = &mut self.cities else {
            return;
        };
        cities.event = Some(event);
        if let Some(track) = event.track() {
            self.record(
                Some(self.turn.player),
                "event",
                format!("{}城门出现，红骰为 {red}", track.name()),
                None,
            );
            for offset in 0..self.humans {
                let player = (self.turn.player + offset) % self.humans;
                let level = self.players[player].upgrades[track.index()];
                if level > 0 && red <= level + 1 {
                    self.pending
                        .push_back(Effect::DrawProgress { player, track });
                }
            }
        } else {
            cities.barbarians += 1;
            let position = cities.barbarians;
            self.record(
                None,
                "barbarians",
                format!("野蛮人船前进至第 {position} 格"),
                None,
            );
            if position == 7 {
                self.barbarian_attack();
            }
        }
    }

    pub fn barbarian_strength(&self) -> usize {
        self.buildings
            .iter()
            .flatten()
            .filter(|building| building.kind == BuildingKind::City)
            .count()
    }

    pub fn barbarian_attack(&mut self) {
        let defense = self.defense();
        let strength = self.barbarian_strength();
        let total: usize = defense.iter().map(|&power| usize::from(power)).sum();
        let Some(cities) = &mut self.cities else {
            return;
        };
        cities.barbarians = 0;
        cities.attacks += 1;
        if cities.attacks == 1 {
            self.robber = self
                .board
                .hexes
                .iter()
                .position(|hex| hex.terrain == Terrain::Desert);
        }
        for knight in cities.knights.iter_mut().flatten() {
            knight.active = false;
        }
        if total >= strength {
            self.record(
                None,
                "defense",
                format!("岛屿守卫以 {total} 点防御力击退 {strength} 点野蛮人"),
                None,
            );
            let highest = defense[..self.humans].iter().copied().max().unwrap_or(0);
            let leaders: Vec<_> = (0..self.humans)
                .filter(|&player| defense[player] == highest)
                .collect();
            if let [player] = leaders[..] {
                self.players[player].defender += 1;
                self.record(
                    Some(player),
                    "award",
                    format!("{}成为卡坦守护者，增加一分", self.players[player].name),
                    None,
                );
            } else {
                for offset in 0..self.humans {
                    let player = (self.turn.player + offset) % self.humans;
                    if leaders.contains(&player) {
                        self.pending.push_back(Effect::Defender { player });
                    }
                }
            }
        } else {
            self.record(
                None,
                "pillage",
                format!("{strength} 点野蛮人突破了 {total} 点防御"),
                None,
            );
            let eligible: Vec<_> = (0..self.humans)
                .filter(|&player| !self.ordinary_cities(player).is_empty())
                .collect();
            let lowest = eligible.iter().map(|&player| defense[player]).min();
            for offset in 0..self.humans {
                let player = (self.turn.player + offset) % self.humans;
                if eligible.contains(&player) && Some(defense[player]) == lowest {
                    self.pending.push_back(Effect::Pillage { player });
                }
            }
        }
    }

    pub fn pillage(&mut self, player: usize, vertex: usize) -> Result<(), String> {
        if !self.ordinary_cities(player).contains(&vertex) {
            return Err("请选择自己的普通城市承受损失".into());
        }
        let cities = self.cities.as_mut().ok_or("当前模式没有野蛮人")?;
        cities.walls.retain(|&wall| wall != vertex);
        if self.players[player].settlements > 0 {
            self.players[player].settlements -= 1;
            self.players[player].cities += 1;
        } else {
            cities.ruins.push(vertex);
        }
        self.buildings[vertex]
            .as_mut()
            .ok_or("这个位置没有城市")?
            .kind = BuildingKind::Settlement;
        self.record(
            Some(player),
            "pillage",
            format!("{}的一座城市降为村庄", self.players[player].name),
            Some(Target::Vertex(vertex)),
        );
        Ok(())
    }

    pub fn barbarian_prompt(&self, effect: &Effect, prompt: &mut Prompt, active: bool) {
        match effect {
            Effect::Pillage { player } => {
                prompt.title = "选择承受野蛮人袭击的城市".into();
                if active {
                    prompt.choices = self
                        .ordinary_cities(*player)
                        .into_iter()
                        .map(|value| Pick {
                            value,
                            label: "城市降为村庄".into(),
                            target: Some(Target::Vertex(value)),
                        })
                        .collect();
                }
            }
            Effect::Defender { .. } => {
                prompt.title = "选择守卫奖励的进步牌堆".into();
                if active && let Some(cities) = &self.cities {
                    prompt.choices = Track::ALL
                        .into_iter()
                        .filter(|track| !cities.decks[track.index()].is_empty())
                        .map(|track| Pick {
                            value: track.index(),
                            label: track.name().into(),
                            target: None,
                        })
                        .collect();
                }
            }
            _ => {}
        }
    }
}
