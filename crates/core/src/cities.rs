use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    Mode,
    board::Resource,
    game::{Action, BuildingKind, Cards, Effect, Game, Target},
    view::{AvailableAction, Pick, Prompt},
};

pub const WALL: Cards = [0, 2, 0, 0, 0, 0, 0, 0];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum Track {
    Trade,
    Politics,
    Science,
}

impl Track {
    pub const ALL: [Self; 3] = [Self::Trade, Self::Politics, Self::Science];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Trade => "贸易",
            Self::Politics => "政治",
            Self::Science => "科学",
        }
    }

    pub fn commodity(self) -> Resource {
        match self {
            Self::Trade => Resource::Cloth,
            Self::Politics => Resource::Coin,
            Self::Science => Resource::Paper,
        }
    }

    pub fn building(self, level: u8) -> &'static str {
        let names = match self {
            Self::Trade => ["基础城市", "市场", "商栈", "商会", "银行", "大交易所"],
            Self::Politics => ["基础城市", "市政厅", "使馆", "城堡", "法院", "议会"],
            Self::Science => ["基础城市", "学校", "图书馆", "引水渠", "剧院", "大学"],
        };
        names[usize::from(level)]
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Cities {
    pub walls: Vec<usize>,
    pub metropolises: [Option<usize>; 3],
    pub ruins: Vec<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct CityView {
    pub upgrades: Vec<[u8; 3]>,
    pub walls: Vec<usize>,
    pub metropolises: [Option<usize>; 3],
    pub ruins: Vec<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct Improvement {
    pub track: Track,
    pub level: u8,
    pub name: String,
    pub next: Option<String>,
    pub cost: Cards,
}

impl Game {
    pub fn city_view(&self) -> Option<CityView> {
        self.cities.as_ref().map(|cities| CityView {
            upgrades: self.players.iter().map(|player| player.upgrades).collect(),
            walls: cities.walls.clone(),
            metropolises: cities.metropolises,
            ruins: cities.ruins.clone(),
        })
    }

    pub fn ordinary_cities(&self, player: usize) -> Vec<usize> {
        self.buildings
            .iter()
            .enumerate()
            .filter(|(vertex, building)| {
                building.as_ref().is_some_and(|building| {
                    building.player == player && building.kind == BuildingKind::City
                }) && self
                    .cities
                    .as_ref()
                    .is_some_and(|cities| !cities.metropolises.contains(&Some(*vertex)))
            })
            .map(|(vertex, _)| vertex)
            .collect()
    }

    pub fn metropolis_owner(&self, track: Track) -> Option<usize> {
        let vertex = self.cities.as_ref()?.metropolises[track.index()]?;
        self.buildings[vertex]
            .as_ref()
            .map(|building| building.player)
    }

    pub fn improvement_cost(&self, player: usize, track: Track, discount: u8) -> Cards {
        let mut cost = [0; 8];
        cost[track.commodity().index()] =
            u16::from((self.players[player].upgrades[track.index()] + 1).saturating_sub(discount));
        cost
    }

    pub fn can_improve(&self, player: usize, track: Track, discount: u8) -> bool {
        if self.cities.is_none() || player >= self.humans {
            return false;
        }
        let level = self.players[player].upgrades[track.index()];
        level < 5
            && self
                .buildings
                .iter()
                .flatten()
                .any(|building| building.player == player && building.kind == BuildingKind::City)
            && (level < 3
                || self.metropolis_owner(track) == Some(player)
                || !self.ordinary_cities(player).is_empty())
            && self.can_pay(player, &self.improvement_cost(player, track, discount))
    }

    pub fn improve(&mut self, player: usize, track: Track, discount: u8) -> Result<(), String> {
        if !self.can_improve(player, track, discount) {
            return Err("需要足够的商品、城市与可用的大都会位置".into());
        }
        self.pay(player, &self.improvement_cost(player, track, discount))?;
        self.players[player].upgrades[track.index()] += 1;
        let level = self.players[player].upgrades[track.index()];
        self.record(
            Some(player),
            "improvement",
            format!(
                "{}的{}提升至 {level} 级：{}",
                self.players[player].name,
                track.name(),
                track.building(level)
            ),
            None,
        );
        let owner = self.metropolis_owner(track);
        if level >= 4
            && owner != Some(player)
            && owner.is_none_or(|owner| self.players[owner].upgrades[track.index()] < level)
        {
            let locations = self.ordinary_cities(player);
            if let [vertex] = locations[..] {
                self.place_metropolis(player, track, vertex)?;
            } else {
                self.pending
                    .push_front(Effect::Metropolis { player, track });
            }
        }
        Ok(())
    }

    pub fn place_metropolis(
        &mut self,
        player: usize,
        track: Track,
        vertex: usize,
    ) -> Result<(), String> {
        if !self.ordinary_cities(player).contains(&vertex) {
            return Err("请选择自己的普通城市".into());
        }
        self.cities
            .as_mut()
            .ok_or("当前模式没有大都会")?
            .metropolises[track.index()] = Some(vertex);
        self.record(
            Some(player),
            "award",
            format!("{}取得{}大都会", self.players[player].name, track.name()),
            Some(Target::Vertex(vertex)),
        );
        Ok(())
    }

    pub fn can_wall(&self, player: usize, vertex: usize) -> bool {
        self.cities.as_ref().is_some_and(|cities| {
            !cities.walls.contains(&vertex)
                && cities
                    .walls
                    .iter()
                    .filter(|&&vertex| {
                        self.buildings[vertex]
                            .as_ref()
                            .is_some_and(|building| building.player == player)
                    })
                    .count()
                    < 3
                && self
                    .buildings
                    .get(vertex)
                    .and_then(Option::as_ref)
                    .is_some_and(|building| {
                        building.player == player && building.kind == BuildingKind::City
                    })
        })
    }

    pub fn build_wall(&mut self, player: usize, vertex: usize, cost: &Cards) -> Result<(), String> {
        if !self.can_wall(player, vertex) {
            return Err("请选择自己的无城墙城市，每名玩家最多建造三座城墙".into());
        }
        self.pay(player, cost)?;
        self.cities
            .as_mut()
            .ok_or("当前模式没有城墙")?
            .walls
            .push(vertex);
        self.record(
            Some(player),
            "build",
            format!("{}修建了城墙", self.players[player].name),
            Some(Target::Vertex(vertex)),
        );
        Ok(())
    }

    pub fn hand_limit(&self, player: usize) -> u16 {
        let walls = self.cities.as_ref().map_or(0, |cities| {
            cities
                .walls
                .iter()
                .filter(|&&vertex| {
                    self.buildings[vertex]
                        .as_ref()
                        .is_some_and(|building| building.player == player)
                })
                .count()
        });
        7 + walls as u16 * 2
    }

    pub fn improvements(&self, player: usize) -> Vec<Improvement> {
        if self.mode != Mode::Cities {
            return Vec::new();
        }
        Track::ALL
            .into_iter()
            .map(|track| {
                let level = self.players[player].upgrades[track.index()];
                Improvement {
                    track,
                    level,
                    name: track.building(level).into(),
                    next: (level < 5).then(|| track.building(level + 1).into()),
                    cost: if level < 5 {
                        self.improvement_cost(player, track, 0)
                    } else {
                        [0; 8]
                    },
                }
            })
            .collect()
    }

    pub fn city_actions(&self, player: usize) -> Vec<AvailableAction> {
        let mut actions = Vec::new();
        if self.cities.is_none() {
            return actions;
        }
        for track in Track::ALL {
            if self.can_improve(player, track, 0) {
                actions.push(AvailableAction {
                    action: Action::Improve { track },
                    label: format!("提升{}", track.name()),
                    target: None,
                    cost: self.improvement_cost(player, track, 0),
                });
            }
        }
        if self.can_pay(player, &WALL) {
            for vertex in 0..self.board.vertices.len() {
                if self.can_wall(player, vertex) {
                    actions.push(AvailableAction {
                        action: Action::BuildWall { vertex },
                        label: "修建城墙".into(),
                        target: Some(Target::Vertex(vertex)),
                        cost: WALL,
                    });
                }
            }
        }
        actions
    }

    pub fn metropolis_prompt(
        &self,
        player: usize,
        track: Track,
        active: bool,
        prompt: &mut Prompt,
    ) {
        prompt.title = format!("选择{}大都会的位置", track.name());
        if active {
            prompt.choices = self
                .ordinary_cities(player)
                .into_iter()
                .map(|value| Pick {
                    value,
                    label: "建立大都会".into(),
                    target: Some(Target::Vertex(value)),
                })
                .collect();
        }
    }
}
