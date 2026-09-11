use serde::{Deserialize, Serialize};

use crate::{
    board::Resource,
    duel::NeutralBuild,
    game::{Action, Cards, Game, Target},
    progress_actions::Merchant,
    view::{CardChoice, Pick, Prompt},
};

pub const MEDICINE: Cards = [0, 0, 0, 1, 2, 0, 0, 0];

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ProgressChoice {
    Alchemy { red: Option<u8> },
    Wall,
    Medicine,
    Invention { first: Option<usize> },
    Smithing { remaining: u8 },
    Merchant,
    Fleet,
    Monopoly { commodities: bool },
    GuildTarget,
    GuildCards { target: usize },
    EspionageTarget,
    EspionageCard { target: usize },
    Diplomacy,
    RelocateRoad,
    Intrigue,
    Taxation,
    TreasonTarget,
    TreasonRemove { owner: usize, recipient: usize },
    TreasonLevel { maximum: u8, active: bool },
    TreasonPlace { level: u8, active: bool },
    Gift { recipient: usize },
    HarborGive { target: usize },
    HarborReturn { recipient: usize },
}

fn sites(
    values: impl Iterator<Item = usize>,
    label: &str,
    target: fn(usize) -> Target,
) -> Vec<Pick> {
    values
        .map(|value| Pick {
            value,
            label: label.into(),
            target: Some(target(value)),
        })
        .collect()
}

impl Game {
    pub fn progress_available(&self, player: usize, choice: &ProgressChoice) -> bool {
        let mut prompt = Prompt {
            player,
            title: String::new(),
            choices: Vec::new(),
            cards: None,
            can_skip: false,
        };
        self.progress_prompt(player, choice, true, &mut prompt);
        !prompt.choices.is_empty() || prompt.cards.is_some()
    }

    pub fn transfer_cards(&mut self, from: usize, to: usize, cards: &Cards) -> Result<(), String> {
        if !self.can_pay(from, cards) {
            return Err("手牌不足以完成交换".into());
        }
        for (resource, &count) in cards.iter().enumerate() {
            self.players[from].hand[resource] -= count;
            self.players[to].hand[resource] += count;
        }
        Ok(())
    }

    pub fn resolve_progress(
        &mut self,
        player: usize,
        index: usize,
        choice: ProgressChoice,
        action: Action,
    ) -> Result<(), String> {
        let prompt = self.prompt(Some(player)).ok_or("当前没有卡牌选择")?;
        match &action {
            Action::Pick { value } if prompt.choices.iter().any(|pick| pick.value == *value) => {}
            Action::SelectCards { cards }
                if prompt.cards.as_ref().is_some_and(|selection| {
                    cards.iter().map(|&count| u32::from(count)).sum::<u32>()
                        == u32::from(selection.count)
                        && cards
                            .iter()
                            .zip(selection.available)
                            .all(|(&count, available)| count <= available)
                }) => {}
            Action::Skip if prompt.can_skip => {
                self.pending.remove(index);
                return Ok(());
            }
            _ => return Err("请选择当前卡牌提供的位置、玩家或手牌".into()),
        }
        self.pending.remove(index);
        match (choice, action) {
            (ProgressChoice::Alchemy { red }, Action::Pick { value }) => {
                if let Some(red) = red {
                    self.resolve_roll([red, value as u8]);
                } else {
                    self.queue_progress(
                        player,
                        ProgressChoice::Alchemy {
                            red: Some(value as u8),
                        },
                    );
                }
            }
            (ProgressChoice::Wall, Action::Pick { value }) => {
                self.build_wall(player, value, &[0; 8])?
            }
            (ProgressChoice::Medicine, Action::Pick { value }) => {
                self.build_city(player, value, &MEDICINE)?
            }
            (ProgressChoice::Invention { first }, Action::Pick { value }) => {
                if let Some(first) = first {
                    let number = self.board.hexes[first].number;
                    self.board.hexes[first].number = self.board.hexes[value].number;
                    self.board.hexes[value].number = number;
                    self.record(
                        Some(player),
                        "invention",
                        format!("{}交换了两块地形的数字", self.players[player].name),
                        Some(Target::Hex(value)),
                    );
                } else {
                    self.queue_progress(player, ProgressChoice::Invention { first: Some(value) });
                }
            }
            (ProgressChoice::Smithing { remaining }, Action::Pick { value }) => {
                let level = self.promote_knight(player, value, &[0; 8])?;
                if remaining > 1 {
                    self.queue_progress(
                        player,
                        ProgressChoice::Smithing {
                            remaining: remaining - 1,
                        },
                    );
                }
                if level == 1 {
                    self.queue_neutral(player, NeutralBuild::Promotion);
                }
            }
            (ProgressChoice::Merchant, Action::Pick { value }) => {
                self.cities.as_mut().ok_or("当前模式没有商人")?.merchant =
                    Some(Merchant { player, hex: value });
                self.record(
                    Some(player),
                    "merchant",
                    format!("{}取得了商人", self.players[player].name),
                    Some(Target::Hex(value)),
                );
            }
            (ProgressChoice::Fleet, Action::Pick { value }) => {
                self.turn.fleet = Some(Resource::ALL[value]);
            }
            (ProgressChoice::Monopoly { commodities }, Action::Pick { value }) => {
                let mut count = 0;
                for other in 0..self.humans {
                    if other != player {
                        let amount =
                            self.players[other].hand[value].min(if commodities { 1 } else { 2 });
                        self.players[other].hand[value] -= amount;
                        self.players[player].hand[value] += amount;
                        count += amount;
                    }
                }
                self.record(
                    Some(player),
                    "trade",
                    format!(
                        "{}取得 {count} 张{}",
                        self.players[player].name,
                        Resource::ALL[value].name()
                    ),
                    None,
                );
            }
            (ProgressChoice::GuildTarget, Action::Pick { value }) => {
                self.queue_progress(player, ProgressChoice::GuildCards { target: value })
            }
            (ProgressChoice::GuildCards { target }, Action::SelectCards { cards }) => {
                self.transfer_cards(target, player, &cards)?;
                self.record(
                    Some(player),
                    "trade",
                    format!(
                        "{}向{}收取行会会费",
                        self.players[player].name, self.players[target].name
                    ),
                    None,
                );
            }
            (ProgressChoice::EspionageTarget, Action::Pick { value }) => {
                self.queue_progress(player, ProgressChoice::EspionageCard { target: value })
            }
            (ProgressChoice::EspionageCard { target }, Action::Pick { value }) => {
                let card = self.players[target].progress.remove(value);
                self.players[player].progress.push(card);
                self.record(
                    Some(player),
                    "progress",
                    format!(
                        "{}从{}处取得一张进步卡",
                        self.players[player].name, self.players[target].name
                    ),
                    None,
                );
                self.private_event(
                    vec![player, target],
                    format!("转移的进步卡是{}", card.info().name),
                );
            }
            (ProgressChoice::Diplomacy, Action::Pick { value }) => {
                let owner = self.roads[value].take().ok_or("这个位置没有道路")?;
                self.players[owner].roads += 1;
                self.record(
                    Some(player),
                    "road",
                    format!(
                        "{}移除了{}的一条开放道路",
                        self.players[player].name, self.players[owner].name
                    ),
                    Some(Target::Edge(value)),
                );
                if owner == player {
                    self.queue_progress(player, ProgressChoice::RelocateRoad);
                }
            }
            (ProgressChoice::RelocateRoad, Action::Pick { value }) => {
                self.build_road(player, value, &[0; 8])?
            }
            (ProgressChoice::Intrigue, Action::Pick { value }) => {
                let knight = self
                    .cities
                    .as_mut()
                    .and_then(|cities| cities.knights[value].take())
                    .ok_or("这个位置没有骑士")?;
                self.queue_displacement(knight, value);
                self.record(
                    Some(player),
                    "knight",
                    format!("{}驱逐了一名敌方骑士", self.players[player].name),
                    Some(Target::Vertex(value)),
                );
            }
            (ProgressChoice::Taxation, Action::Pick { value }) => {
                self.robber = Some(value);
                self.record(
                    Some(player),
                    "robber",
                    format!("{}移动强盗并征税", self.players[player].name),
                    Some(Target::Hex(value)),
                );
                let mut targets = Vec::new();
                for &vertex in &self.board.hexes[value].vertices {
                    if let Some(building) = &self.buildings[vertex]
                        && building.player != player
                        && building.player < self.humans
                        && !targets.contains(&building.player)
                    {
                        targets.push(building.player);
                    }
                }
                for target in targets {
                    self.steal(player, target);
                }
            }
            (ProgressChoice::TreasonTarget, Action::Pick { value }) => {
                let chooser = if value < self.humans { value } else { player };
                self.queue_progress(
                    chooser,
                    ProgressChoice::TreasonRemove {
                        owner: value,
                        recipient: player,
                    },
                );
            }
            (ProgressChoice::TreasonRemove { owner, recipient }, Action::Pick { value }) => {
                let knight = self
                    .cities
                    .as_mut()
                    .and_then(|cities| cities.knights[value].take())
                    .ok_or("这个位置没有骑士")?;
                self.record(
                    Some(owner),
                    "knight",
                    format!("{}的一名骑士离开了棋盘", self.players[owner].name),
                    Some(Target::Vertex(value)),
                );
                self.queue_progress(
                    recipient,
                    ProgressChoice::TreasonLevel {
                        maximum: knight.level,
                        active: knight.active,
                    },
                );
            }
            (ProgressChoice::TreasonLevel { active, .. }, Action::Pick { value }) => self
                .queue_progress(
                    player,
                    ProgressChoice::TreasonPlace {
                        level: value as u8,
                        active,
                    },
                ),
            (ProgressChoice::TreasonPlace { level, active }, Action::Pick { value }) => {
                self.place_knight(player, value, level, active, &[0; 8])?
            }
            (ProgressChoice::Gift { recipient }, Action::SelectCards { cards }) => {
                self.transfer_cards(player, recipient, &cards)?;
                self.record(
                    Some(player),
                    "trade",
                    format!(
                        "{}向{}送出婚礼礼物",
                        self.players[player].name, self.players[recipient].name
                    ),
                    None,
                );
            }
            (ProgressChoice::HarborGive { target }, Action::SelectCards { cards }) => {
                self.turn.harbors[target] -= 1;
                if self.players[target].hand[5..]
                    .iter()
                    .any(|&count| count > 0)
                {
                    self.transfer_cards(player, target, &cards)?;
                    self.queue_progress(target, ProgressChoice::HarborReturn { recipient: player });
                } else {
                    self.record(
                        Some(player),
                        "trade",
                        format!("{}没有商品，商业港交换结束", self.players[target].name),
                        None,
                    );
                }
            }
            (ProgressChoice::HarborReturn { recipient }, Action::SelectCards { cards }) => {
                self.transfer_cards(player, recipient, &cards)?;
                self.record(
                    Some(player),
                    "trade",
                    format!(
                        "{}与{}完成商业港交换",
                        self.players[player].name, self.players[recipient].name
                    ),
                    None,
                );
            }
            _ => return Err("当前卡牌需要另一种选择".into()),
        }
        Ok(())
    }

    pub fn progress_prompt(
        &self,
        player: usize,
        choice: &ProgressChoice,
        active: bool,
        prompt: &mut Prompt,
    ) {
        prompt.title = match choice {
            ProgressChoice::Alchemy { red: None } => "选择红色生产骰点数",
            ProgressChoice::Alchemy { red: Some(_) } => "选择白色生产骰点数",
            ProgressChoice::Wall => "选择免费修建城墙的城市",
            ProgressChoice::Medicine => "选择使用医学升级的村庄",
            ProgressChoice::Invention { first: None } => "选择第一块要交换数字的地形",
            ProgressChoice::Invention { first: Some(_) } => "选择第二块要交换数字的地形",
            ProgressChoice::Smithing { .. } => "选择免费晋升的骑士",
            ProgressChoice::Merchant => "选择商人所在的资源地",
            ProgressChoice::Fleet => "选择商船队交易的资源或商品",
            ProgressChoice::Monopoly { commodities: false } => "选择垄断的资源",
            ProgressChoice::Monopoly { commodities: true } => "选择垄断的商品",
            ProgressChoice::GuildTarget => "选择收取行会会费的玩家",
            ProgressChoice::GuildCards { .. } => "选择收取的资源或商品",
            ProgressChoice::EspionageTarget => "选择谍报对象",
            ProgressChoice::EspionageCard { .. } => "选择取得的进步卡",
            ProgressChoice::Diplomacy => "选择移除的开放道路",
            ProgressChoice::RelocateRoad => "重新放置自己的道路",
            ProgressChoice::Intrigue => "选择驱逐的敌方骑士",
            ProgressChoice::Taxation => "选择强盗征税的地形",
            ProgressChoice::TreasonTarget => "选择发生叛变的玩家",
            ProgressChoice::TreasonRemove { .. } => "选择离开棋盘的骑士",
            ProgressChoice::TreasonLevel { .. } => "选择获得的骑士等级",
            ProgressChoice::TreasonPlace { .. } => "选择获得的骑士的位置",
            ProgressChoice::Gift { .. } => "选择婚礼礼物",
            ProgressChoice::HarborGive { .. } => "选择商业港提供的资源",
            ProgressChoice::HarborReturn { .. } => "选择商业港交换的商品",
        }
        .into();
        if !active {
            return;
        }
        prompt.can_skip = matches!(
            choice,
            ProgressChoice::Smithing { .. }
                | ProgressChoice::RelocateRoad
                | ProgressChoice::HarborGive { .. }
        );
        prompt.choices = match *choice {
            ProgressChoice::Alchemy { .. } => (1..=6)
                .map(|value| Pick {
                    value,
                    label: value.to_string(),
                    target: None,
                })
                .collect(),
            ProgressChoice::Wall => sites(
                (0..self.board.vertices.len()).filter(|&vertex| self.can_wall(player, vertex)),
                "修建城墙",
                Target::Vertex,
            ),
            ProgressChoice::Medicine => sites(
                (0..self.board.vertices.len()).filter(|&vertex| {
                    self.can_city(player, vertex) && self.can_pay(player, &MEDICINE)
                }),
                "升级城市",
                Target::Vertex,
            ),
            ProgressChoice::Invention { first } => sites(
                (0..self.board.hexes.len()).filter(|&hex| {
                    Some(hex) != first
                        && matches!(self.board.hexes[hex].number, 3 | 4 | 5 | 9 | 10 | 11)
                }),
                "交换数字",
                Target::Hex,
            ),
            ProgressChoice::Smithing { .. } => sites(
                (0..self.board.vertices.len()).filter(|&vertex| self.can_promote(player, vertex)),
                "晋升骑士",
                Target::Vertex,
            ),
            ProgressChoice::Merchant => sites(
                (0..self.board.hexes.len()).filter(|&hex| {
                    self.board.hexes[hex].terrain.resource().is_some()
                        && self.board.hexes[hex].vertices.iter().any(|&vertex| {
                            self.buildings[vertex]
                                .as_ref()
                                .is_some_and(|building| building.player == player)
                        })
                }),
                "放置商人",
                Target::Hex,
            ),
            ProgressChoice::Fleet => Resource::ALL
                .into_iter()
                .map(|resource| Pick {
                    value: resource.index(),
                    label: resource.name().into(),
                    target: None,
                })
                .collect(),
            ProgressChoice::Monopoly { commodities } => Resource::ALL
                .into_iter()
                .filter(|resource| (resource.index() >= 5) == commodities)
                .map(|resource| Pick {
                    value: resource.index(),
                    label: resource.name().into(),
                    target: None,
                })
                .collect(),
            ProgressChoice::GuildTarget => (0..self.humans)
                .filter(|&other| {
                    other != player
                        && self.points(other) > self.points(player)
                        && self.players[other].hand.iter().any(|&count| count > 0)
                })
                .map(|value| Pick {
                    value,
                    label: self.players[value].name.clone(),
                    target: None,
                })
                .collect(),
            ProgressChoice::EspionageTarget => (0..self.humans)
                .filter(|&other| other != player && !self.players[other].progress.is_empty())
                .map(|value| Pick {
                    value,
                    label: self.players[value].name.clone(),
                    target: None,
                })
                .collect(),
            ProgressChoice::EspionageCard { target } => self.players[target]
                .progress
                .iter()
                .enumerate()
                .map(|(value, card)| Pick {
                    value,
                    label: format!("{} · {}", card.info().name, card.info().description),
                    target: None,
                })
                .collect(),
            ProgressChoice::Diplomacy => sites(
                (0..self.board.edges.len()).filter(|&edge| {
                    self.roads[edge].is_some_and(|owner| {
                        self.board.edges[edge].vertices.iter().any(|&vertex| {
                            !self.buildings[vertex]
                                .as_ref()
                                .is_some_and(|building| building.player == owner)
                                && !self
                                    .knight(vertex)
                                    .is_some_and(|knight| knight.player == owner)
                                && self.board.vertices[vertex]
                                    .edges
                                    .iter()
                                    .all(|&other| other == edge || self.roads[other] != Some(owner))
                        })
                    })
                }),
                "移除道路",
                Target::Edge,
            ),
            ProgressChoice::RelocateRoad => sites(
                (0..self.board.edges.len())
                    .filter(|&edge| self.players[player].roads > 0 && self.can_road(player, edge)),
                "放置道路",
                Target::Edge,
            ),
            ProgressChoice::Intrigue => sites(
                (0..self.board.vertices.len()).filter(|&vertex| {
                    self.knight(vertex)
                        .is_some_and(|knight| knight.player != player)
                        && self.board.vertices[vertex]
                            .edges
                            .iter()
                            .any(|&edge| self.roads[edge] == Some(player))
                }),
                "驱逐骑士",
                Target::Vertex,
            ),
            ProgressChoice::Taxation => sites(
                (0..self.board.hexes.len()).filter(|&hex| self.robber != Some(hex)),
                "移动强盗",
                Target::Hex,
            ),
            ProgressChoice::TreasonTarget => (0..self.players.len())
                .filter(|&other| {
                    other != player && (1..=3).any(|level| self.knight_count(other, level) > 0)
                })
                .map(|value| Pick {
                    value,
                    label: self.players[value].name.clone(),
                    target: None,
                })
                .collect(),
            ProgressChoice::TreasonRemove { owner, .. } => {
                let lowest = (1..=3).find(|&level| self.knight_count(owner, level) > 0);
                sites(
                    (0..self.board.vertices.len()).filter(|&vertex| {
                        self.knight(vertex).is_some_and(|knight| {
                            knight.player == owner
                                && (owner < self.humans || Some(knight.level) == lowest)
                        })
                    }),
                    "移除骑士",
                    Target::Vertex,
                )
            }
            ProgressChoice::TreasonLevel { maximum, .. } => (1..=maximum)
                .filter(|&level| {
                    self.knight_count(player, level) < 2 && !self.knight_sites(player).is_empty()
                })
                .map(|level| Pick {
                    value: usize::from(level),
                    label: format!("{level} 级骑士"),
                    target: None,
                })
                .collect(),
            ProgressChoice::TreasonPlace { level, .. } => sites(
                self.knight_sites(player)
                    .into_iter()
                    .filter(|_| self.knight_count(player, level) < 2),
                "放置骑士",
                Target::Vertex,
            ),
            _ => Vec::new(),
        };
        let selection = match *choice {
            ProgressChoice::GuildCards { target } => Some((self.players[target].hand, 2)),
            ProgressChoice::Gift { .. } => Some((self.players[player].hand, 2)),
            ProgressChoice::HarborGive { .. } => {
                let mut cards = self.players[player].hand;
                cards[5..].fill(0);
                Some((cards, 1))
            }
            ProgressChoice::HarborReturn { .. } => {
                let mut cards = self.players[player].hand;
                cards[..5].fill(0);
                Some((cards, 1))
            }
            _ => None,
        };
        prompt.cards = selection.and_then(|(available, count)| {
            let count = count.min(available.iter().sum());
            (count > 0).then_some(CardChoice { available, count })
        });
    }
}
