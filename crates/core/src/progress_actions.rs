use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    board::Resource,
    cities::Track,
    game::{Action, Effect, Game, Stage},
    progress::Progress,
    progress_choices::ProgressChoice,
    view::AvailableAction,
};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct Merchant {
    pub player: usize,
    pub hex: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct ProgressView {
    pub card: Progress,
    pub track: Track,
    pub name: String,
    pub description: String,
    pub playable: bool,
}

impl Game {
    pub fn can_play_progress(&self, player: usize, card: Progress) -> bool {
        self.cities.is_some()
            && player < self.humans
            && player == self.turn.player
            && self.pending.is_empty()
            && self.players[player].progress.contains(&card)
            && match card {
                Progress::Alchemy => self.stage == Stage::Production && self.turn.dice.is_empty(),
                Progress::Taxation => self.stage == Stage::Action && self.robber.is_some(),
                Progress::Constitution | Progress::Printing => false,
                _ => self.stage == Stage::Action,
            }
    }

    pub fn progress_view(&self, player: usize) -> Vec<ProgressView> {
        self.players[player]
            .progress
            .iter()
            .map(|&card| {
                let info = card.info();
                ProgressView {
                    card,
                    track: info.track,
                    name: info.name.into(),
                    description: info.description.into(),
                    playable: self.can_play_progress(player, card),
                }
            })
            .collect()
    }

    pub fn progress_actions(&self, player: usize) -> Vec<AvailableAction> {
        let mut actions = Vec::new();
        for card in Progress::ALL {
            if self.can_play_progress(player, card) {
                actions.push(AvailableAction {
                    action: Action::PlayProgress { card },
                    label: card.info().name.into(),
                    target: None,
                    cost: [0; 8],
                });
            }
        }
        for target in 0..self.humans {
            if self.can_harbor(player, target) {
                actions.push(AvailableAction {
                    action: Action::HarborTrade { target },
                    label: format!("与{}进行商业港交换", self.players[target].name),
                    target: None,
                    cost: [0; 8],
                });
            }
        }
        actions
    }

    pub fn queue_progress(&mut self, player: usize, choice: ProgressChoice) {
        self.pending.push_front(Effect::Progress { player, choice });
    }

    pub fn play_progress(&mut self, player: usize, card: Progress) -> Result<(), String> {
        if !self.can_play_progress(player, card) {
            return Err("这张进步卡当前无法使用".into());
        }
        let index = self.players[player]
            .progress
            .iter()
            .position(|&held| held == card)
            .ok_or("手中没有这张进步卡")?;
        self.players[player].progress.remove(index);
        self.cities.as_mut().ok_or("当前模式没有进步卡")?.decks[card.info().track.index()]
            .insert(0, card);
        self.record(
            Some(player),
            "progress",
            format!("{}使用了{}", self.players[player].name, card.info().name),
            None,
        );
        let choice = match card {
            Progress::CommercialHarbor => {
                for target in 0..self.humans {
                    if target != player {
                        self.turn.harbors[target] += 1;
                    }
                }
                None
            }
            Progress::GuildDues => Some(ProgressChoice::GuildTarget),
            Progress::Merchant => Some(ProgressChoice::Merchant),
            Progress::MerchantFleet => Some(ProgressChoice::Fleet),
            Progress::ResourceMonopoly => Some(ProgressChoice::Monopoly { commodities: false }),
            Progress::CommodityMonopoly => Some(ProgressChoice::Monopoly { commodities: true }),
            Progress::Diplomacy => Some(ProgressChoice::Diplomacy),
            Progress::Encouragement => {
                if let Some(cities) = &mut self.cities {
                    for knight in cities
                        .knights
                        .iter_mut()
                        .flatten()
                        .filter(|knight| knight.player == player && !knight.active)
                    {
                        knight.active = true;
                        knight.activated = self.turn.number;
                    }
                }
                None
            }
            Progress::Espionage => Some(ProgressChoice::EspionageTarget),
            Progress::Intrigue => Some(ProgressChoice::Intrigue),
            Progress::Sabotage => {
                for offset in 1..self.humans {
                    let other = (player + offset) % self.humans;
                    let count: u16 = self.players[other].hand.iter().sum::<u16>() / 2;
                    if count > 0 && self.points(other) >= self.points(player) {
                        self.pending.push_back(Effect::Discard {
                            player: other,
                            count,
                        });
                    }
                }
                None
            }
            Progress::Taxation => Some(ProgressChoice::Taxation),
            Progress::Treason => Some(ProgressChoice::TreasonTarget),
            Progress::Wedding => {
                for offset in 1..self.humans {
                    let other = (player + offset) % self.humans;
                    if self.points(other) > self.points(player)
                        && self.players[other].hand.iter().any(|&count| count > 0)
                    {
                        self.pending.push_back(Effect::Progress {
                            player: other,
                            choice: ProgressChoice::Gift { recipient: player },
                        });
                    }
                }
                None
            }
            Progress::Alchemy => Some(ProgressChoice::Alchemy { red: None }),
            Progress::Crane => {
                self.turn.cranes += 1;
                None
            }
            Progress::Engineering => Some(ProgressChoice::Wall),
            Progress::Invention => Some(ProgressChoice::Invention { first: None }),
            Progress::Irrigation | Progress::Mining => {
                let resource = if card == Progress::Irrigation {
                    Resource::Grain
                } else {
                    Resource::Ore
                };
                let count = self
                    .board
                    .hexes
                    .iter()
                    .filter(|hex| {
                        hex.terrain.resource() == Some(resource)
                            && hex.vertices.iter().any(|&vertex| {
                                self.buildings[vertex]
                                    .as_ref()
                                    .is_some_and(|building| building.player == player)
                            })
                    })
                    .count();
                let received = self.take_bank(player, resource, count as u16 * 2);
                self.record(
                    Some(player),
                    "production",
                    format!(
                        "{}获得 {received} 张{}",
                        self.players[player].name,
                        resource.name()
                    ),
                    None,
                );
                None
            }
            Progress::Medicine => Some(ProgressChoice::Medicine),
            Progress::RoadBuilding => {
                self.pending.push_front(Effect::FreeRoad {
                    player,
                    remaining: 2,
                });
                None
            }
            Progress::Smithing => Some(ProgressChoice::Smithing { remaining: 2 }),
            Progress::Constitution | Progress::Printing => None,
        };
        if let Some(choice) = choice {
            self.queue_progress(player, choice);
        }
        Ok(())
    }

    pub fn can_harbor(&self, player: usize, target: usize) -> bool {
        player == self.turn.player
            && target < self.humans
            && player != target
            && self.stage == Stage::Action
            && self.pending.is_empty()
            && self
                .turn
                .harbors
                .get(target)
                .is_some_and(|&count| count > 0)
            && self.players[player].hand[..5]
                .iter()
                .any(|&count| count > 0)
    }

    pub fn harbor_trade(&mut self, player: usize, target: usize) -> Result<(), String> {
        if !self.can_harbor(player, target) {
            return Err("请选择商业港可以交换的玩家，并预留一张基础资源".into());
        }
        self.queue_progress(player, ProgressChoice::HarborGive { target });
        Ok(())
    }

    pub fn crane_discount(&self, player: usize) -> u8 {
        u8::from(player == self.turn.player && self.turn.cranes > 0)
    }
}
