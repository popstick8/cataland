use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    Mode,
    board::Resource,
    game::{Action, Cards, Effect, Game, Stage, Target},
    view::{AvailableAction, CardChoice, Pick, Prompt},
};

pub const DEVELOPMENT: Cards = [0, 0, 1, 1, 1, 0, 0, 0];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum Card {
    Knight,
    VictoryPoint,
    RoadBuilding,
    Plenty,
    Monopoly,
}

impl Card {
    pub fn name(self) -> &'static str {
        match self {
            Self::Knight => "骑士",
            Self::VictoryPoint => "胜利点",
            Self::RoadBuilding => "修路",
            Self::Plenty => "丰收之年",
            Self::Monopoly => "垄断",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Knight => "移动强盗，从相邻的一名对手处随机取得一张牌。计入最大军队。",
            Self::VictoryPoint => "获得一分。获胜时向其他玩家揭示。",
            Self::RoadBuilding => "免费放置至多两条道路。",
            Self::Plenty => "从银行选择两张基础资源，可以选择相同资源。",
            Self::Monopoly => "选择一种基础资源，取得其他玩家手中全部该资源。",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeldCard {
    pub card: Card,
    pub acquired: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct CardView {
    pub card: Card,
    pub name: String,
    pub description: String,
    pub playable: bool,
}

impl Game {
    pub fn score(&self, player: usize) -> u16 {
        self.points(player)
            + self.players[player]
                .cards
                .iter()
                .filter(|held| held.card == Card::VictoryPoint)
                .count() as u16
    }

    pub fn buy_development(&mut self, player: usize) -> Result<(), String> {
        if self.mode != Mode::Base || self.dev_deck.is_empty() {
            return Err("发展牌堆中已经没有牌".into());
        }
        self.pay(player, &DEVELOPMENT)?;
        if let Some(card) = self.dev_deck.pop() {
            self.players[player].cards.push(HeldCard {
                card,
                acquired: self.turn.number,
            });
        }
        self.record(
            Some(player),
            "card",
            format!("{}购买了一张发展卡", self.players[player].name),
            None,
        );
        Ok(())
    }

    pub fn can_play(&self, player: usize, card: Card) -> bool {
        if self.mode != Mode::Base
            || player != self.turn.player
            || self.turn.development
            || !self.pending.is_empty()
            || !matches!(self.stage, Stage::Production | Stage::Action)
        {
            return false;
        }
        if !self.players[player]
            .cards
            .iter()
            .any(|held| held.card == card && held.acquired < self.turn.number)
        {
            return false;
        }
        match card {
            Card::VictoryPoint => false,
            Card::Knight | Card::Monopoly => true,
            Card::RoadBuilding => {
                self.players[player].roads > 0
                    && (0..self.board.edges.len()).any(|edge| self.can_road(player, edge))
            }
            Card::Plenty => self.bank[..5].iter().sum::<u16>() >= 2,
        }
    }

    pub fn play_card(&mut self, player: usize, card: Card) -> Result<(), String> {
        if !self.can_play(player, card) {
            return Err("这张卡当前无法使用".into());
        }
        let index = self.players[player]
            .cards
            .iter()
            .position(|held| held.card == card && held.acquired < self.turn.number)
            .ok_or("手中没有这张可用的卡")?;
        self.players[player].cards.remove(index);
        self.turn.development = true;
        self.record(
            Some(player),
            "card",
            format!("{}使用了{}", self.players[player].name, card.name()),
            None,
        );
        match card {
            Card::Knight => {
                self.players[player].army += 1;
                self.pending.push_back(Effect::Robber { player });
            }
            Card::RoadBuilding => self.pending.push_back(Effect::FreeRoad {
                player,
                remaining: 2,
            }),
            Card::Plenty => self
                .pending
                .push_back(Effect::BankCards { player, count: 2 }),
            Card::Monopoly => self.pending.push_back(Effect::Monopoly { player }),
            Card::VictoryPoint => unreachable!(),
        }
        Ok(())
    }

    pub fn card_actions(&self, player: usize) -> Vec<AvailableAction> {
        let mut actions = Vec::new();
        if self.stage == Stage::Action
            && self.mode == Mode::Base
            && !self.dev_deck.is_empty()
            && self.can_pay(player, &DEVELOPMENT)
        {
            actions.push(AvailableAction {
                action: Action::BuyDevelopment,
                label: "购买发展卡".into(),
                target: None,
                cost: DEVELOPMENT,
            });
        }
        let mut included = Vec::new();
        for held in &self.players[player].cards {
            if !included.contains(&held.card) && self.can_play(player, held.card) {
                actions.push(AvailableAction {
                    action: Action::PlayCard { card: held.card },
                    label: held.card.name().into(),
                    target: None,
                    cost: [0; 8],
                });
                included.push(held.card);
            }
        }
        actions
    }

    pub fn resolve_card(
        &mut self,
        player: usize,
        index: usize,
        effect: Effect,
        action: Action,
    ) -> Result<(), String> {
        match (effect, action) {
            (Effect::FreeRoad { remaining, .. }, Action::Pick { value }) => {
                self.build_road(player, value, &[0; 8])?;
                self.pending.remove(index);
                if remaining > 1
                    && self.players[player].roads > 0
                    && (0..self.board.edges.len()).any(|edge| self.can_road(player, edge))
                {
                    self.pending.push_front(Effect::FreeRoad {
                        player,
                        remaining: remaining - 1,
                    });
                }
                self.queue_neutral(player, crate::duel::NeutralBuild::Road);
            }
            (Effect::FreeRoad { .. }, Action::Skip) => {
                self.pending.remove(index);
            }
            (Effect::BankCards { count, .. }, Action::SelectCards { cards }) => {
                if cards.iter().map(|&value| u32::from(value)).sum::<u32>() != u32::from(count)
                    || cards[5..].iter().any(|&value| value != 0)
                {
                    return Err(format!("请选择 {count} 张基础资源"));
                }
                if cards
                    .iter()
                    .zip(self.bank)
                    .any(|(&need, available)| need > available)
                {
                    return Err("银行中没有足够的资源".into());
                }
                for resource in Resource::BASE {
                    self.take_bank(player, resource, cards[resource.index()]);
                }
                self.pending.remove(index);
                self.record(
                    Some(player),
                    "production",
                    format!("{}从银行取得 {count} 张资源", self.players[player].name),
                    None,
                );
            }
            (Effect::Monopoly { .. }, Action::Pick { value }) => {
                let resource = *Resource::BASE.get(value).ok_or("请选择一种基础资源")?;
                let mut total = 0;
                for other in 0..self.players.len() {
                    if other != player {
                        total += self.players[other].hand[value];
                        self.players[other].hand[value] = 0;
                    }
                }
                self.players[player].hand[value] += total;
                self.pending.remove(index);
                self.record(
                    Some(player),
                    "card",
                    format!(
                        "{}通过垄断取得 {total} 张{}",
                        self.players[player].name,
                        resource.name()
                    ),
                    None,
                );
            }
            _ => return Err("请完成当前卡牌的选择".into()),
        }
        Ok(())
    }

    pub fn card_prompt(&self, effect: &Effect, prompt: &mut Prompt, active: bool) {
        match effect {
            Effect::FreeRoad { remaining, .. } => {
                prompt.title = format!("免费修路，还可放置 {remaining} 条");
                if active {
                    prompt.can_skip = true;
                    prompt.choices = (0..self.board.edges.len())
                        .filter(|&edge| self.can_road(prompt.player, edge))
                        .map(|value| Pick {
                            value,
                            label: "放置道路".into(),
                            target: Some(Target::Edge(value)),
                        })
                        .collect();
                }
            }
            Effect::BankCards { count, .. } => {
                prompt.title = format!("从银行取得 {count} 张资源");
                if active {
                    let mut available = self.bank;
                    available[5..].fill(0);
                    prompt.cards = Some(CardChoice {
                        available,
                        count: *count,
                    });
                }
            }
            Effect::Monopoly { .. } => {
                prompt.title = "选择垄断的资源".into();
                if active {
                    prompt.choices = Resource::BASE
                        .into_iter()
                        .map(|resource| Pick {
                            value: resource.index(),
                            label: resource.name().into(),
                            target: None,
                        })
                        .collect();
                }
            }
            _ => {}
        }
    }
}
