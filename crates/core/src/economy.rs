use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    Mode,
    board::Resource,
    game::{Action, Building, BuildingKind, Cards, Game, Stage, Target},
};

pub const ROAD: Cards = [1, 1, 0, 0, 0, 0, 0, 0];
pub const SETTLEMENT: Cards = [1, 1, 1, 1, 0, 0, 0, 0];
pub const CITY: Cards = [0, 0, 0, 2, 3, 0, 0, 0];

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct Trade {
    pub player: usize,
    pub give: Cards,
    pub want: Cards,
    pub responses: Vec<Option<bool>>,
}

impl Game {
    pub fn bank_rate(&self, player: usize, resource: Resource) -> u8 {
        let mut rate = 4;
        for harbor in &self.board.harbors {
            if self.board.edges[harbor.edge]
                .vertices
                .iter()
                .any(|&vertex| {
                    self.buildings[vertex]
                        .as_ref()
                        .is_some_and(|building| building.player == player)
                })
            {
                if harbor.resource == Some(resource) {
                    rate = rate.min(2);
                }
                if harbor.resource.is_none() {
                    rate = rate.min(3);
                }
            }
        }
        rate
    }

    pub fn can_city(&self, player: usize, vertex: usize) -> bool {
        self.players[player].cities > 0
            && self
                .buildings
                .get(vertex)
                .and_then(Option::as_ref)
                .is_some_and(|building| {
                    building.player == player && building.kind == BuildingKind::Settlement
                })
    }

    pub fn build_road(&mut self, player: usize, edge: usize, cost: &Cards) -> Result<(), String> {
        if self.players[player].roads == 0 || !self.can_road(player, edge) {
            return Err("请选择连接自己道路或建筑的空边".into());
        }
        self.pay(player, cost)?;
        self.players[player].roads -= 1;
        self.roads[edge] = Some(player);
        self.record(
            Some(player),
            "build",
            format!("{}建造了一条道路", self.players[player].name),
            Some(Target::Edge(edge)),
        );
        Ok(())
    }

    pub fn build_settlement(&mut self, player: usize, vertex: usize) -> Result<(), String> {
        if self.players[player].settlements == 0 || !self.can_settle(player, vertex, false) {
            return Err("请选择连接自己道路且满足建筑间距的位置".into());
        }
        self.pay(player, &SETTLEMENT)?;
        self.players[player].settlements -= 1;
        self.buildings[vertex] = Some(Building {
            player,
            kind: BuildingKind::Settlement,
        });
        self.record(
            Some(player),
            "build",
            format!("{}建造了一座村庄", self.players[player].name),
            Some(Target::Vertex(vertex)),
        );
        Ok(())
    }

    pub fn build_city(&mut self, player: usize, vertex: usize, cost: &Cards) -> Result<(), String> {
        if !self.can_city(player, vertex) {
            return Err("请选择自己的村庄，并预留一枚城市棋子".into());
        }
        self.pay(player, cost)?;
        self.players[player].cities -= 1;
        self.players[player].settlements += 1;
        self.buildings[vertex] = Some(Building {
            player,
            kind: BuildingKind::City,
        });
        self.record(
            Some(player),
            "build",
            format!("{}将村庄升级为城市", self.players[player].name),
            Some(Target::Vertex(vertex)),
        );
        Ok(())
    }

    pub fn economy(&mut self, player: usize, action: Action) -> Result<(), String> {
        match action {
            Action::BuildRoad { edge } => self.build_road(player, edge, &ROAD)?,
            Action::BuildSettlement { vertex } => self.build_settlement(player, vertex)?,
            Action::BuildCity { vertex } => self.build_city(player, vertex, &CITY)?,
            Action::BankTrade { give, take } => {
                if give == take || self.bank[take.index()] == 0 {
                    return Err("请选择银行有库存的另一种牌".into());
                }
                let mut cost = [0; 8];
                cost[give.index()] = u16::from(self.bank_rate(player, give));
                self.pay(player, &cost)?;
                self.take_bank(player, take, 1);
                self.record(
                    Some(player),
                    "trade",
                    format!(
                        "{}用 {} 张{}换取 1 张{}",
                        self.players[player].name,
                        cost[give.index()],
                        give.name(),
                        take.name()
                    ),
                    None,
                );
            }
            Action::OfferTrade { give, want } => {
                if !give.iter().any(|&count| count > 0) || !want.iter().any(|&count| count > 0) {
                    return Err("交易需要填写给出和索取的牌".into());
                }
                if give.iter().zip(want).any(|(&a, b)| a > 0 && b > 0) {
                    return Err("同一种牌在一笔交易中只能给出或索取".into());
                }
                if self.mode == Mode::Base
                    && give[5..].iter().chain(&want[5..]).any(|&count| count > 0)
                {
                    return Err("基础规则中的交易使用五种资源".into());
                }
                if !self.can_pay(player, &give) {
                    return Err("手牌不足以支付报价".into());
                }
                self.trade = Some(Trade {
                    player,
                    give,
                    want,
                    responses: vec![None; self.players.len()],
                });
            }
            Action::CancelTrade => {
                self.trade = None;
            }
            Action::CompleteTrade { partner } => {
                let trade = self.trade.as_ref().ok_or("当前没有交易报价")?.clone();
                if trade.player != player || trade.responses.get(partner) != Some(&Some(true)) {
                    return Err("请选择已经同意交易的玩家".into());
                }
                if !self.can_pay(player, &trade.give) || !self.can_pay(partner, &trade.want) {
                    return Err("双方当前的手牌数量不足以完成交易".into());
                }
                for index in 0..8 {
                    self.players[player].hand[index] =
                        self.players[player].hand[index] - trade.give[index] + trade.want[index];
                    self.players[partner].hand[index] =
                        self.players[partner].hand[index] - trade.want[index] + trade.give[index];
                }
                self.trade = None;
                self.record(
                    Some(player),
                    "trade",
                    format!(
                        "{}与{}完成交易",
                        self.players[player].name, self.players[partner].name
                    ),
                    None,
                );
            }
            _ => return Err("请选择建造或交易动作".into()),
        }
        Ok(())
    }

    pub fn respond_trade(&mut self, player: usize, accept: bool) -> Result<(), String> {
        if self.stage != Stage::Action || !self.pending.is_empty() {
            return Err("当前的结算完成后才能交易".into());
        }
        let trade = self.trade.as_ref().ok_or("当前没有交易报价")?;
        if player == trade.player {
            return Err("报价由其他玩家回应".into());
        }
        if accept && !self.can_pay(player, &trade.want) {
            return Err("手牌不足以接受这笔交易".into());
        }
        if let Some(trade) = &mut self.trade {
            trade.responses[player] = Some(accept);
        }
        Ok(())
    }

    pub fn check_victory(&mut self) {
        let player = self.turn.player;
        let target = if self.mode == Mode::Base { 10 } else { 13 };
        if self.winner.is_none() && self.points(player) >= target {
            self.winner = Some(player);
            self.stage = Stage::Ended;
            self.pending.clear();
            self.trade = None;
            self.record(
                Some(player),
                "victory",
                format!(
                    "{}以 {} 分获胜",
                    self.players[player].name,
                    self.points(player)
                ),
                None,
            );
        }
    }
}
