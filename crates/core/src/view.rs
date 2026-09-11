use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    Mode,
    board::Board,
    game::{Action, Building, Cards, Effect, Game, GameEvent, Stage, Target, Turn},
};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PlayerView {
    pub name: String,
    pub color: usize,
    pub points: u16,
    pub hand_count: u16,
    pub roads: u8,
    pub settlements: u8,
    pub cities: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct PrivateView {
    pub hand: Cards,
    pub points: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct AvailableAction {
    pub action: Action,
    pub label: String,
    pub target: Option<Target>,
    pub cost: Cards,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct Pick {
    pub value: usize,
    pub label: String,
    pub target: Option<Target>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct CardChoice {
    pub available: Cards,
    pub count: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct Prompt {
    pub player: usize,
    pub title: String,
    pub choices: Vec<Pick>,
    pub cards: Option<CardChoice>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct GameView {
    pub mode: Mode,
    pub board: Board,
    pub players: Vec<PlayerView>,
    pub buildings: Vec<Option<Building>>,
    pub roads: Vec<Option<usize>>,
    pub bank: Cards,
    pub robber: Option<usize>,
    pub stage: Stage,
    pub turn: Turn,
    pub private: Option<PrivateView>,
    pub actions: Vec<AvailableAction>,
    pub prompt: Option<Prompt>,
    pub events: Vec<GameEvent>,
    pub winner: Option<usize>,
}

impl Game {
    pub fn view(&self, viewer: Option<usize>) -> GameView {
        let viewer = viewer.filter(|&player| player < self.players.len());
        GameView {
            mode: self.mode,
            board: self.board.clone(),
            players: self
                .players
                .iter()
                .enumerate()
                .map(|(player, data)| PlayerView {
                    name: data.name.clone(),
                    color: data.color,
                    points: self.points(player),
                    hand_count: data.hand.iter().sum(),
                    roads: data.roads,
                    settlements: data.settlements,
                    cities: data.cities,
                })
                .collect(),
            buildings: self.buildings.clone(),
            roads: self.roads.clone(),
            bank: self.bank,
            robber: self.robber,
            stage: self.stage.clone(),
            turn: self.turn.clone(),
            private: viewer.map(|player| PrivateView {
                hand: self.players[player].hand,
                points: self.points(player),
            }),
            actions: viewer.map_or_else(Vec::new, |player| self.actions(player)),
            prompt: self.prompt(viewer),
            events: self
                .events
                .iter()
                .filter(|event| {
                    event.audience.is_empty()
                        || viewer.is_some_and(|player| event.audience.contains(&player))
                })
                .map(|event| event.message.clone())
                .collect(),
            winner: self.winner,
        }
    }

    pub fn actions(&self, player: usize) -> Vec<AvailableAction> {
        let mut actions = Vec::new();
        if player >= self.players.len() || player != self.turn.player || !self.pending.is_empty() {
            return actions;
        }
        match &self.stage {
            Stage::Setup { road: None, .. } => {
                for vertex in 0..self.board.vertices.len() {
                    if self.can_settle(player, vertex, true) {
                        actions.push(AvailableAction {
                            action: Action::BuildSettlement { vertex },
                            label: "放置起始建筑".into(),
                            target: Some(Target::Vertex(vertex)),
                            cost: [0; 8],
                        });
                    }
                }
            }
            Stage::Setup {
                road: Some(vertex), ..
            } => {
                for &edge in &self.board.vertices[*vertex].edges {
                    if self.roads[edge].is_none() {
                        actions.push(AvailableAction {
                            action: Action::BuildRoad { edge },
                            label: "放置起始道路".into(),
                            target: Some(Target::Edge(edge)),
                            cost: [0; 8],
                        });
                    }
                }
            }
            Stage::Production => actions.push(AvailableAction {
                action: Action::Roll,
                label: "掷骰子".into(),
                target: None,
                cost: [0; 8],
            }),
            Stage::Action => actions.push(AvailableAction {
                action: Action::EndTurn,
                label: "结束回合".into(),
                target: None,
                cost: [0; 8],
            }),
            Stage::Ended => {}
        }
        actions
    }

    pub fn prompt(&self, viewer: Option<usize>) -> Option<Prompt> {
        let effect = self.pending.get(
            viewer
                .and_then(|player| self.pending_index(player))
                .unwrap_or(0),
        )?;
        let player = effect.player();
        let active = viewer == Some(player);
        let mut prompt = Prompt {
            player,
            title: String::new(),
            choices: Vec::new(),
            cards: None,
        };
        match effect {
            Effect::Discard { count, .. } => {
                prompt.title = format!("弃掉 {count} 张手牌");
                if active {
                    prompt.cards = Some(CardChoice {
                        available: self.players[player].hand,
                        count: *count,
                    });
                }
            }
            Effect::Robber { .. } => {
                prompt.title = "选择强盗的新位置".into();
                if active {
                    prompt.choices = self
                        .board
                        .hexes
                        .iter()
                        .enumerate()
                        .filter(|(id, _)| self.robber != Some(*id))
                        .map(|(value, _)| Pick {
                            value,
                            label: "移动强盗".into(),
                            target: Some(Target::Hex(value)),
                        })
                        .collect();
                }
            }
            Effect::Steal { targets, .. } => {
                prompt.title = "选择取得手牌的玩家".into();
                if active {
                    prompt.choices = targets
                        .iter()
                        .map(|&value| Pick {
                            value,
                            label: self.players[value].name.clone(),
                            target: None,
                        })
                        .collect();
                }
            }
        }
        Some(prompt)
    }
}
