use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    Mode,
    board::{Resource, Terrain},
    game::{Action, Building, BuildingKind, Effect, Game, Player, Stage, Target},
    view::{AvailableAction, CardChoice, Pick, Prompt},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TokenAction {
    Trade { commodities: bool },
    MoveRobber,
    Sacrifice,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum NeutralBuild {
    Road,
    Settlement,
    Knight,
    Promotion,
}

impl Game {
    pub fn setup_neutrals(&mut self) {
        if self.humans != 2 {
            return;
        }
        for player in &mut self.players {
            player.tokens = 5;
        }
        let colors: Vec<_> = (0..6)
            .filter(|color| self.players.iter().all(|player| player.color != *color))
            .take(2)
            .collect();
        for (number, color) in colors.into_iter().enumerate() {
            let owner = self.players.len();
            let mut neutral = Player::new(format!("中立势力 {}", number + 1), color);
            neutral.settlements -= 1;
            self.players.push(neutral);
            let y = if number == 0 { -2.0 } else { 2.0 };
            let vertex = self
                .board
                .vertices
                .iter()
                .position(|point| point.x == 0.0 && point.y == y)
                .expect("Standard island neutral intersection");
            self.buildings[vertex] = Some(Building {
                player: owner,
                kind: BuildingKind::Settlement,
            });
            self.awards.lengths.push(0);
        }
    }

    fn neutral_sites(&self, owner: usize, kind: NeutralBuild) -> Vec<usize> {
        match kind {
            NeutralBuild::Road if self.players[owner].roads > 0 => (0..self.board.edges.len())
                .filter(|&edge| self.can_road(owner, edge))
                .collect(),
            NeutralBuild::Settlement if self.players[owner].settlements > 0 => {
                (0..self.board.vertices.len())
                    .filter(|&vertex| self.can_settle(owner, vertex, false))
                    .collect()
            }
            NeutralBuild::Knight if self.knight_count(owner, 1) < 2 => self.knight_sites(owner),
            NeutralBuild::Promotion => (0..self.board.vertices.len())
                .filter(|&vertex| {
                    self.knight(vertex).is_some_and(|knight| knight.level == 1)
                        && self.can_promote(owner, vertex)
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    pub fn queue_neutral(&mut self, player: usize, kind: NeutralBuild) {
        if self.humans != 2 || player >= self.humans {
            return;
        }
        let possible = (self.humans..self.players.len())
            .any(|owner| !self.neutral_sites(owner, kind).is_empty());
        if possible {
            self.pending.push_front(Effect::Neutral {
                player,
                kind,
                owner: None,
            });
        } else if matches!(kind, NeutralBuild::Settlement | NeutralBuild::Knight) {
            self.queue_neutral(player, NeutralBuild::Road);
        }
    }

    pub fn resolve_neutral(
        &mut self,
        player: usize,
        kind: NeutralBuild,
        owner: Option<usize>,
        action: Action,
    ) -> Result<(), String> {
        let Action::Pick { value } = action else {
            return Err("请选择中立势力与建造位置".into());
        };
        if let Some(owner) = owner {
            if !self.neutral_sites(owner, kind).contains(&value) {
                return Err("请选择中立势力可以建造的位置".into());
            }
            match kind {
                NeutralBuild::Road => self.build_road(owner, value, &[0; 8])?,
                NeutralBuild::Knight => self.place_knight(owner, value, 1, false, &[0; 8])?,
                NeutralBuild::Promotion => {
                    self.promote_knight(owner, value, &[0; 8])?;
                }
                NeutralBuild::Settlement => {
                    self.players[owner].settlements -= 1;
                    self.buildings[value] = Some(Building {
                        player: owner,
                        kind: BuildingKind::Settlement,
                    });
                    self.record(
                        Some(owner),
                        "build",
                        format!("{}建造了一座村庄", self.players[owner].name),
                        Some(Target::Vertex(value)),
                    );
                }
            }
            self.pending.pop_front();
        } else {
            if !(self.humans..self.players.len()).contains(&value)
                || self.neutral_sites(value, kind).is_empty()
            {
                return Err("请选择能够建造的中立势力".into());
            }
            self.pending.pop_front();
            self.pending.push_front(Effect::Neutral {
                player,
                kind,
                owner: Some(value),
            });
        }
        Ok(())
    }

    pub fn neutral_prompt(
        &self,
        kind: NeutralBuild,
        owner: Option<usize>,
        prompt: &mut Prompt,
        active: bool,
    ) {
        let name = match kind {
            NeutralBuild::Road => "建造道路",
            NeutralBuild::Settlement => "建造村庄",
            NeutralBuild::Knight => "招募骑士",
            NeutralBuild::Promotion => "晋升骑士",
        };
        prompt.title = owner.map_or_else(
            || format!("选择中立势力，{name}"),
            |owner| format!("为{}{name}", self.players[owner].name),
        );
        if !active {
            return;
        }
        prompt.choices = if let Some(owner) = owner {
            self.neutral_sites(owner, kind)
                .into_iter()
                .map(|value| Pick {
                    value,
                    label: name.into(),
                    target: Some(match kind {
                        NeutralBuild::Road => Target::Edge(value),
                        NeutralBuild::Settlement
                        | NeutralBuild::Knight
                        | NeutralBuild::Promotion => Target::Vertex(value),
                    }),
                })
                .collect()
        } else {
            (self.humans..self.players.len())
                .filter(|&owner| !self.neutral_sites(owner, kind).is_empty())
                .map(|value| Pick {
                    value,
                    label: self.players[value].name.clone(),
                    target: None,
                })
                .collect()
        };
    }
}

impl Game {
    pub fn token_cost(&self, player: usize) -> u8 {
        if self.points(player) <= self.points(1 - player) {
            1
        } else {
            2
        }
    }

    pub fn grant_tokens(&mut self, player: usize, count: u8) {
        if self.humans != 2 || player >= self.humans {
            return;
        }
        let count = count.min(self.tokens);
        self.tokens -= count;
        self.players[player].tokens += count;
        if count > 0 {
            self.record(
                Some(player),
                "tokens",
                format!("{}获得 {count} 个贸易筹码", self.players[player].name),
                None,
            );
        }
    }

    pub fn settlement_tokens(&mut self, player: usize, vertex: usize) {
        let point = &self.board.vertices[vertex];
        let coast = point
            .edges
            .iter()
            .any(|&edge| self.board.edges[edge].hexes.len() == 1);
        let desert = point
            .hexes
            .iter()
            .any(|&hex| self.board.hexes[hex].terrain == Terrain::Desert);
        self.grant_tokens(player, u8::from(coast) + u8::from(desert) * 2);
    }

    pub fn can_token(&self, player: usize, action: TokenAction) -> bool {
        if self.humans != 2
            || player >= self.humans
            || player != self.turn.player
            || !self.pending.is_empty()
            || !matches!(self.stage, Stage::Production | Stage::Action)
        {
            return false;
        }
        if action == TokenAction::Sacrifice {
            return self.mode == Mode::Base
                && !self.turn.sacrifice
                && self.players[player].army > 0;
        }
        if self.turn.token_action || (self.stage == Stage::Production && !self.turn.dice.is_empty())
        {
            return false;
        }
        let cost = self.token_cost(player);
        match action {
            TokenAction::Trade { commodities } => {
                if commodities && self.mode != Mode::Cities {
                    return false;
                }
                let available = if commodities { 8 } else { 5 };
                let other: u16 = self.players[1 - player].hand[..available].iter().sum();
                let own: u16 = self.players[player].hand[..available].iter().sum();
                self.players[player].tokens >= cost * if commodities { 2 } else { 1 }
                    && other > 0
                    && own + other.min(2) >= 2
            }
            TokenAction::MoveRobber => {
                self.players[player].tokens >= cost
                    && self
                        .robber
                        .is_some_and(|hex| self.board.hexes[hex].terrain != Terrain::Desert)
            }
            TokenAction::Sacrifice => false,
        }
    }

    pub fn token_actions(&self, player: usize) -> Vec<AvailableAction> {
        [
            TokenAction::Trade { commodities: false },
            TokenAction::Trade { commodities: true },
            TokenAction::MoveRobber,
            TokenAction::Sacrifice,
        ]
        .into_iter()
        .filter(|&action| self.can_token(player, action))
        .map(|action| {
            let label = match action {
                TokenAction::Trade { commodities: false } => {
                    format!("强制交换资源 · {} 筹码", self.token_cost(player))
                }
                TokenAction::Trade { commodities: true } => {
                    format!("强制交换资源与商品 · {} 筹码", self.token_cost(player) * 2)
                }
                TokenAction::MoveRobber => {
                    format!("强盗返回沙漠 · {} 筹码", self.token_cost(player))
                }
                TokenAction::Sacrifice => "交回一张已使用的骑士卡 · 获得 2 筹码".into(),
            };
            AvailableAction {
                action: Action::Tokens { action },
                label,
                target: None,
                cost: [0; 8],
            }
        })
        .collect()
    }

    pub fn use_token(&mut self, player: usize, action: TokenAction) -> Result<(), String> {
        if !self.can_token(player, action) {
            return Err("当前无法执行这个贸易筹码行动".into());
        }
        if action == TokenAction::Sacrifice {
            self.players[player].army -= 1;
            self.turn.sacrifice = true;
            self.grant_tokens(player, 2);
            return Ok(());
        }
        let cost = self.token_cost(player)
            * if matches!(action, TokenAction::Trade { commodities: true }) {
                2
            } else {
                1
            };
        self.players[player].tokens -= cost;
        self.tokens += cost;
        self.turn.token_action = true;
        match action {
            TokenAction::Trade { commodities } => {
                let resources: &[Resource] = if commodities {
                    &Resource::ALL
                } else {
                    &Resource::BASE
                };
                let mut drawn = Vec::new();
                for _ in 0..2 {
                    if let Some(resource) = self.take_random(player, 1 - player, resources) {
                        drawn.push(resource.name());
                    }
                }
                self.record(
                    Some(player),
                    "trade",
                    format!(
                        "{}发起了强制交易，取得 {} 张牌",
                        self.players[player].name,
                        drawn.len()
                    ),
                    None,
                );
                self.private_event(
                    vec![player, 1 - player],
                    format!("强制交易取得：{}", drawn.join("、")),
                );
                self.pending.push_front(Effect::ReturnCards {
                    player,
                    commodities,
                });
            }
            TokenAction::MoveRobber => {
                let desert = self
                    .board
                    .hexes
                    .iter()
                    .position(|hex| hex.terrain == Terrain::Desert)
                    .expect("Island desert");
                self.robber = Some(desert);
                self.record(
                    Some(player),
                    "robber",
                    format!("{}使用贸易筹码，将强盗送回沙漠", self.players[player].name),
                    Some(Target::Hex(desert)),
                );
            }
            TokenAction::Sacrifice => {}
        }
        Ok(())
    }

    pub fn return_cards(
        &mut self,
        player: usize,
        commodities: bool,
        action: Action,
    ) -> Result<(), String> {
        let Action::SelectCards { cards } = action else {
            return Err("请选择交给对方的两张牌".into());
        };
        if cards.iter().map(|&count| u32::from(count)).sum::<u32>() != 2
            || (!commodities && cards[5..].iter().any(|&count| count > 0))
        {
            return Err("请选择本次交易范围内的两张牌".into());
        }
        if !self.can_pay(player, &cards) {
            return Err("手牌数量不足".into());
        }
        for (index, count) in cards.into_iter().enumerate() {
            self.players[player].hand[index] -= count;
            self.players[1 - player].hand[index] += count;
        }
        self.pending.pop_front();
        self.record(
            Some(player),
            "trade",
            format!("{}交回两张牌，完成强制交易", self.players[player].name),
            None,
        );
        Ok(())
    }

    pub fn return_prompt(&self, commodities: bool, prompt: &mut Prompt, active: bool) {
        prompt.title = "强制交易：交给对方两张牌".into();
        if active {
            let mut available = self.players[prompt.player].hand;
            if !commodities {
                available[5..].fill(0);
            }
            prompt.cards = Some(CardChoice {
                available,
                count: 2,
            });
        }
    }
}
