use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    Mode, Seat,
    awards::Awards,
    board::{Board, Resource, Terrain},
    development::{Card, HeldCard},
};

pub type Cards = [u16; 8];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum BuildingKind {
    Settlement,
    City,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct Building {
    pub player: usize,
    pub kind: BuildingKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub color: usize,
    pub hand: Cards,
    pub roads: u8,
    pub settlements: u8,
    pub cities: u8,
    pub cards: Vec<HeldCard>,
    pub army: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Stage {
    Setup { step: usize, road: Option<usize> },
    Production,
    Action,
    Ended,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct Turn {
    pub player: usize,
    pub primary: usize,
    pub number: u32,
    pub dice: Vec<[u8; 2]>,
    pub development: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type", content = "id", rename_all = "camelCase")]
pub enum Target {
    Vertex(usize),
    Edge(usize),
    Hex(usize),
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Action {
    BuildSettlement { vertex: usize },
    BuildRoad { edge: usize },
    BuildCity { vertex: usize },
    BankTrade { give: Resource, take: Resource },
    OfferTrade { give: Cards, want: Cards },
    RespondTrade { accept: bool },
    CompleteTrade { partner: usize },
    CancelTrade,
    BuyDevelopment,
    PlayCard { card: Card },
    Roll,
    EndTurn,
    Pick { value: usize },
    SelectCards { cards: Cards },
    Skip,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Effect {
    Discard { player: usize, count: u16 },
    Robber { player: usize },
    Steal { player: usize, targets: Vec<usize> },
    FreeRoad { player: usize, remaining: u8 },
    BankCards { player: usize, count: u16 },
    Monopoly { player: usize },
}

impl Effect {
    pub fn player(&self) -> usize {
        match self {
            Self::Discard { player, .. }
            | Self::Robber { player }
            | Self::Steal { player, .. }
            | Self::FreeRoad { player, .. }
            | Self::BankCards { player, .. }
            | Self::Monopoly { player } => *player,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct GameEvent {
    pub seq: u64,
    pub player: Option<usize>,
    pub kind: String,
    pub text: String,
    pub target: Option<Target>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub message: GameEvent,
    pub audience: Vec<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Game {
    pub mode: Mode,
    pub humans: usize,
    pub board: Board,
    pub players: Vec<Player>,
    pub buildings: Vec<Option<Building>>,
    pub roads: Vec<Option<usize>>,
    pub bank: Cards,
    pub robber: Option<usize>,
    pub stage: Stage,
    pub turn: Turn,
    pub starter: usize,
    pub pending: VecDeque<Effect>,
    pub events: Vec<Event>,
    pub trade: Option<crate::economy::Trade>,
    pub awards: Awards,
    pub dev_deck: Vec<Card>,
    pub winner: Option<usize>,
}

impl Game {
    pub fn new(mode: Mode, seats: &[Seat], starter: Option<usize>) -> Result<Self, String> {
        if !(2..=6).contains(&seats.len()) {
            return Err("一局需要 2–6 名玩家".into());
        }
        let starter = starter.unwrap_or_else(|| fastrand::usize(..seats.len()));
        if starter >= seats.len() {
            return Err("起始玩家尚未加入".into());
        }
        let big = seats.len() >= 5;
        let board = Board::generate(big, &mut fastrand::Rng::new());
        let robber = if mode == Mode::Base {
            board
                .hexes
                .iter()
                .position(|hex| hex.terrain == Terrain::Desert)
        } else {
            None
        };
        let mut bank = [0; 8];
        bank[..5].fill(if big { 24 } else { 19 });
        if mode == Mode::Cities {
            bank[5..].fill(if big { 18 } else { 12 });
        }
        let mut dev_deck = Vec::new();
        if mode == Mode::Base {
            for (card, count) in [
                (Card::Knight, if big { 20 } else { 14 }),
                (Card::VictoryPoint, 5),
                (Card::RoadBuilding, if big { 3 } else { 2 }),
                (Card::Plenty, if big { 3 } else { 2 }),
                (Card::Monopoly, if big { 3 } else { 2 }),
            ] {
                dev_deck.extend(std::iter::repeat_n(card, count));
            }
            fastrand::shuffle(&mut dev_deck);
        }
        Ok(Self {
            mode,
            humans: seats.len(),
            buildings: vec![None; board.vertices.len()],
            roads: vec![None; board.edges.len()],
            board,
            players: seats
                .iter()
                .map(|seat| Player {
                    name: seat.name.clone(),
                    color: seat.color,
                    hand: [0; 8],
                    roads: 15,
                    settlements: 5,
                    cities: 4,
                    cards: Vec::new(),
                    army: 0,
                })
                .collect(),
            bank,
            robber,
            stage: Stage::Setup {
                step: 0,
                road: None,
            },
            turn: Turn {
                player: starter,
                primary: starter,
                number: 0,
                dice: Vec::new(),
                development: false,
            },
            starter,
            pending: VecDeque::new(),
            events: Vec::new(),
            trade: None,
            awards: Awards {
                road: None,
                army: None,
                lengths: vec![0; seats.len()],
            },
            dev_deck,
            winner: None,
        })
    }

    pub fn apply(&mut self, player: usize, action: Action) -> Result<(), String> {
        if player >= self.humans {
            return Err("这个席位不在对局中".into());
        }
        if self.stage == Stage::Ended {
            return Err("这场对局已经结束".into());
        }
        if let Action::RespondTrade { accept } = action {
            return self.respond_trade(player, accept);
        }
        if !self.pending.is_empty() {
            self.resolve(player, action)?;
            self.update_awards();
            self.check_victory();
            return Ok(());
        }
        if player != self.turn.player {
            return Err("当前由其他玩家行动".into());
        }
        match (&self.stage, action) {
            (Stage::Setup { step, road: None }, Action::BuildSettlement { vertex }) => {
                let step = *step;
                if !self.can_settle(player, vertex, true) {
                    return Err("请选择与已有建筑间隔一个顶点的空位置".into());
                }
                let kind = if self.mode == Mode::Cities && step >= self.humans {
                    BuildingKind::City
                } else {
                    BuildingKind::Settlement
                };
                self.buildings[vertex] = Some(Building { player, kind });
                match kind {
                    BuildingKind::Settlement => self.players[player].settlements -= 1,
                    BuildingKind::City => self.players[player].cities -= 1,
                }
                if step >= self.humans {
                    for hex in self.board.vertices[vertex].hexes.clone() {
                        if let Some(resource) = self.board.hexes[hex].terrain.resource() {
                            self.take_bank(player, resource, 1);
                        }
                    }
                }
                self.record(
                    Some(player),
                    "build",
                    format!(
                        "{}放置了起始{}",
                        self.players[player].name,
                        if kind == BuildingKind::City {
                            "城市"
                        } else {
                            "村庄"
                        }
                    ),
                    Some(Target::Vertex(vertex)),
                );
                self.stage = Stage::Setup {
                    step,
                    road: Some(vertex),
                };
            }
            (
                Stage::Setup {
                    step,
                    road: Some(vertex),
                },
                Action::BuildRoad { edge },
            ) => {
                let (step, vertex) = (*step, *vertex);
                if !self
                    .board
                    .edges
                    .get(edge)
                    .is_some_and(|e| e.vertices.contains(&vertex))
                    || self.roads[edge].is_some()
                {
                    return Err("起始道路需要连接刚放置的建筑".into());
                }
                self.roads[edge] = Some(player);
                self.players[player].roads -= 1;
                self.record(
                    Some(player),
                    "build",
                    format!("{}放置了起始道路", self.players[player].name),
                    Some(Target::Edge(edge)),
                );
                let next = step + 1;
                if next == self.humans * 2 {
                    self.stage = Stage::Production;
                    self.turn.player = self.starter;
                    self.turn.number = 1;
                } else {
                    let position = if next < self.humans {
                        next
                    } else {
                        self.humans * 2 - 1 - next
                    };
                    self.turn.player = (self.starter + position) % self.humans;
                    self.stage = Stage::Setup {
                        step: next,
                        road: None,
                    };
                }
            }
            (Stage::Production, Action::Roll) => self.roll(),
            (Stage::Production | Stage::Action, Action::PlayCard { card }) => {
                self.play_card(player, card)?
            }
            (Stage::Action, Action::BuyDevelopment) => self.buy_development(player)?,
            (Stage::Action, Action::EndTurn) => self.end_turn(),
            (Stage::Action, action) => self.economy(player, action)?,
            _ => return Err("这个动作不属于当前阶段".into()),
        }
        self.update_awards();
        self.check_victory();
        Ok(())
    }

    pub fn can_settle(&self, player: usize, vertex: usize, initial: bool) -> bool {
        self.board.vertices.get(vertex).is_some_and(|point| {
            self.buildings[vertex].is_none()
                && point
                    .neighbors
                    .iter()
                    .all(|&other| self.buildings[other].is_none())
                && (initial
                    || point
                        .edges
                        .iter()
                        .any(|&edge| self.roads[edge] == Some(player)))
        })
    }

    pub fn blocked(&self, player: usize, vertex: usize) -> bool {
        self.buildings[vertex]
            .as_ref()
            .is_some_and(|building| building.player != player)
    }

    pub fn can_road(&self, player: usize, edge: usize) -> bool {
        self.board.edges.get(edge).is_some_and(|line| {
            self.roads[edge].is_none()
                && line.vertices.iter().any(|&vertex| {
                    !self.blocked(player, vertex)
                        && (self.buildings[vertex].is_some()
                            || self.board.vertices[vertex]
                                .edges
                                .iter()
                                .any(|&other| self.roads[other] == Some(player)))
                })
        })
    }

    pub fn points(&self, player: usize) -> u16 {
        let buildings: u16 = self
            .buildings
            .iter()
            .flatten()
            .filter(|building| building.player == player)
            .map(|building| match building.kind {
                BuildingKind::Settlement => 1,
                BuildingKind::City => 2,
            })
            .sum();
        buildings
            + u16::from(self.awards.road == Some(player)) * 2
            + u16::from(self.awards.army == Some(player)) * 2
    }

    pub fn record(
        &mut self,
        player: Option<usize>,
        kind: &str,
        text: String,
        target: Option<Target>,
    ) {
        self.events.push(Event {
            message: GameEvent {
                seq: self.events.len() as u64 + 1,
                player,
                kind: kind.into(),
                text,
                target,
            },
            audience: Vec::new(),
        });
    }

    pub fn take_bank(&mut self, player: usize, resource: Resource, count: u16) -> u16 {
        let index = resource.index();
        let count = count.min(self.bank[index]);
        self.bank[index] -= count;
        self.players[player].hand[index] += count;
        count
    }

    pub fn can_pay(&self, player: usize, cards: &Cards) -> bool {
        self.players[player]
            .hand
            .iter()
            .zip(cards)
            .all(|(have, need)| have >= need)
    }

    pub fn pay(&mut self, player: usize, cards: &Cards) -> Result<(), String> {
        if !self.can_pay(player, cards) {
            return Err("手牌数量不足".into());
        }
        for (index, &count) in cards.iter().enumerate() {
            self.players[player].hand[index] -= count;
            self.bank[index] += count;
        }
        Ok(())
    }

    fn roll(&mut self) {
        let previous = self.turn.dice.first().map(|dice| dice[0] + dice[1]);
        let dice = loop {
            let dice = [fastrand::u8(1..=6), fastrand::u8(1..=6)];
            if self.humans != 2 || Some(dice[0] + dice[1]) != previous {
                break dice;
            }
        };
        let total = dice[0] + dice[1];
        self.turn.dice.push(dice);
        self.stage = if self.humans == 2 && self.turn.dice.len() == 1 {
            Stage::Production
        } else {
            Stage::Action
        };
        self.record(
            Some(self.turn.player),
            "roll",
            format!(
                "{}掷出 {} + {} = {}",
                self.players[self.turn.player].name, dice[0], dice[1], total
            ),
            None,
        );
        if total == 7 {
            for (player, data) in self.players.iter().take(self.humans).enumerate() {
                let count: u16 = data.hand.iter().sum();
                if count > 7 {
                    self.pending.push_back(Effect::Discard {
                        player,
                        count: count / 2,
                    });
                }
            }
            if self.robber.is_some() {
                self.pending.push_back(Effect::Robber {
                    player: self.turn.player,
                });
            }
        } else {
            self.produce(total);
        }
    }

    fn produce(&mut self, total: u8) {
        let mut demand = vec![[0; 8]; self.humans];
        for (vertex, building) in self.buildings.iter().enumerate() {
            let Some(building) = building else {
                continue;
            };
            for &id in &self.board.vertices[vertex].hexes {
                let hex = &self.board.hexes[id];
                if hex.number != total || self.robber == Some(id) {
                    continue;
                }
                let Some(resource) = hex.terrain.resource() else {
                    continue;
                };
                demand[building.player][resource.index()] += 1;
                if building.kind == BuildingKind::City {
                    let extra = if self.mode == Mode::Cities {
                        match resource {
                            Resource::Wood => Resource::Paper,
                            Resource::Wool => Resource::Cloth,
                            Resource::Ore => Resource::Coin,
                            _ => resource,
                        }
                    } else {
                        resource
                    };
                    demand[building.player][extra.index()] += 1;
                }
            }
        }
        for resource in Resource::ALL {
            let index = resource.index();
            let total: u16 = demand.iter().map(|hand| hand[index]).sum();
            let recipients = demand.iter().filter(|hand| hand[index] > 0).count();
            if total > self.bank[index] && recipients > 1 {
                continue;
            }
            for (player, hand) in demand.iter().enumerate() {
                let amount = self.take_bank(player, resource, hand[index]);
                if amount > 0 {
                    self.record(
                        Some(player),
                        "production",
                        format!(
                            "{}获得 {} 张{}",
                            self.players[player].name,
                            amount,
                            resource.name()
                        ),
                        None,
                    );
                }
            }
        }
    }

    fn end_turn(&mut self) {
        self.trade = None;
        self.turn.development = false;
        self.turn.number += 1;
        if self.humans >= 5 && self.turn.player == self.turn.primary {
            self.turn.player = (self.turn.primary + 3) % self.humans;
            self.stage = Stage::Action;
        } else {
            self.turn.primary = (self.turn.primary + 1) % self.humans;
            self.turn.player = self.turn.primary;
            self.turn.dice.clear();
            self.stage = Stage::Production;
        }
    }

    pub fn pending_index(&self, player: usize) -> Option<usize> {
        if matches!(self.pending.front(), Some(Effect::Discard { .. })) {
            self.pending
                .iter()
                .take_while(|effect| matches!(effect, Effect::Discard { .. }))
                .position(|effect| effect.player() == player)
        } else {
            self.pending
                .front()
                .filter(|effect| effect.player() == player)
                .map(|_| 0)
        }
    }

    fn resolve(&mut self, player: usize, action: Action) -> Result<(), String> {
        let index = self.pending_index(player).ok_or("当前由其他玩家完成选择")?;
        let effect = self.pending[index].clone();
        match (effect, action) {
            (Effect::Discard { count, .. }, Action::SelectCards { cards }) => {
                if cards.iter().map(|&count| u32::from(count)).sum::<u32>() != u32::from(count) {
                    return Err(format!("需要选择 {count} 张牌"));
                }
                self.pay(player, &cards)?;
                self.pending.remove(index);
                self.record(
                    Some(player),
                    "discard",
                    format!("{}弃掉了 {count} 张牌", self.players[player].name),
                    None,
                );
            }
            (Effect::Robber { .. }, Action::Pick { value }) => {
                if value >= self.board.hexes.len() || self.robber == Some(value) {
                    return Err("请选择强盗当前所在位置以外的地块".into());
                }
                self.robber = Some(value);
                self.pending.remove(index);
                self.record(
                    Some(player),
                    "robber",
                    format!("{}移动了强盗", self.players[player].name),
                    Some(Target::Hex(value)),
                );
                let mut targets = Vec::new();
                for &vertex in &self.board.hexes[value].vertices {
                    if let Some(building) = &self.buildings[vertex] {
                        let target = building.player;
                        if target != player
                            && self.players[target].hand.iter().any(|&count| count > 0)
                            && !targets.contains(&target)
                        {
                            targets.push(target);
                        }
                    }
                }
                match targets.as_slice() {
                    [] => {}
                    [target] => self.steal(player, *target),
                    _ => self.pending.push_front(Effect::Steal { player, targets }),
                }
            }
            (Effect::Steal { targets, .. }, Action::Pick { value }) => {
                if !targets.contains(&value) {
                    return Err("请选择这块地形相邻的玩家".into());
                }
                self.pending.remove(index);
                self.steal(player, value);
            }
            (effect, action) => self.resolve_card(player, index, effect, action)?,
        }
        Ok(())
    }

    fn steal(&mut self, player: usize, target: usize) {
        let total: u16 = self.players[target].hand.iter().sum();
        if total == 0 {
            return;
        }
        let mut pick = fastrand::u16(..total);
        for resource in Resource::ALL {
            let index = resource.index();
            let count = self.players[target].hand[index];
            if pick < count {
                self.players[target].hand[index] -= 1;
                self.players[player].hand[index] += 1;
                self.record(
                    Some(player),
                    "steal",
                    format!(
                        "{}从{}处取得一张牌",
                        self.players[player].name, self.players[target].name
                    ),
                    None,
                );
                self.events.push(Event {
                    message: GameEvent {
                        seq: self.events.len() as u64 + 1,
                        player: Some(player),
                        kind: "private".into(),
                        text: format!("取得的牌是{}", resource.name()),
                        target: None,
                    },
                    audience: vec![player, target],
                });
                return;
            }
            pick -= count;
        }
    }
}
