use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    cities::Track,
    game::{Effect, Game},
    view::{Pick, Prompt},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum Progress {
    CommercialHarbor,
    GuildDues,
    Merchant,
    MerchantFleet,
    ResourceMonopoly,
    CommodityMonopoly,
    Diplomacy,
    Encouragement,
    Espionage,
    Intrigue,
    Sabotage,
    Taxation,
    Treason,
    Constitution,
    Wedding,
    Alchemy,
    Crane,
    Engineering,
    Invention,
    Irrigation,
    Medicine,
    Mining,
    RoadBuilding,
    Smithing,
    Printing,
}

pub struct CardInfo {
    pub track: Track,
    pub copies: usize,
    pub name: &'static str,
    pub description: &'static str,
}

impl Progress {
    pub const ALL: [Self; 25] = [
        Self::CommercialHarbor,
        Self::GuildDues,
        Self::Merchant,
        Self::MerchantFleet,
        Self::ResourceMonopoly,
        Self::CommodityMonopoly,
        Self::Diplomacy,
        Self::Encouragement,
        Self::Espionage,
        Self::Intrigue,
        Self::Sabotage,
        Self::Taxation,
        Self::Treason,
        Self::Constitution,
        Self::Wedding,
        Self::Alchemy,
        Self::Crane,
        Self::Engineering,
        Self::Invention,
        Self::Irrigation,
        Self::Medicine,
        Self::Mining,
        Self::RoadBuilding,
        Self::Smithing,
        Self::Printing,
    ];

    pub fn info(self) -> CardInfo {
        use Track::{Politics, Science, Trade};
        let (track, copies, name, description) = match self {
            Self::CommercialHarbor => (
                Trade,
                2,
                "商业港",
                "本回合可向每名其他玩家各提供一张资源，换取对方选择的一张商品。",
            ),
            Self::GuildDues => (
                Trade,
                2,
                "行会会费",
                "查看一名分数更高的玩家的资源与商品，选择其中两张。",
            ),
            Self::Merchant => (
                Trade,
                6,
                "商人",
                "将商人放在自己建筑旁的资源地，获得一分和该资源的 2:1 汇率。",
            ),
            Self::MerchantFleet => (
                Trade,
                2,
                "商船队",
                "选择一种资源或商品，本回合以 2:1 与银行交易。",
            ),
            Self::ResourceMonopoly => (
                Trade,
                4,
                "资源垄断",
                "选择一种资源，每名其他玩家交出至多两张。",
            ),
            Self::CommodityMonopoly => {
                (Trade, 2, "商品垄断", "选择一种商品，每名其他玩家交出一张。")
            }
            Self::Diplomacy => (
                Politics,
                2,
                "外交",
                "移除一条开放道路；自己的道路可以免费放到新的合法位置。",
            ),
            Self::Encouragement => (Politics, 2, "鼓舞", "免费激活自己的全部骑士。"),
            Self::Espionage => (
                Politics,
                3,
                "谍报",
                "查看一名其他玩家的进步卡，选择其中一张。",
            ),
            Self::Intrigue => (Politics, 2, "阴谋", "驱逐自己道路所连接的一名敌方骑士。"),
            Self::Sabotage => (
                Politics,
                2,
                "破坏",
                "分数不少于自己的其他玩家弃掉一半资源与商品。",
            ),
            Self::Taxation => (
                Politics,
                2,
                "征税",
                "移动强盗，从新地块旁每名其他玩家处各随机取得一张资源或商品。",
            ),
            Self::Treason => (
                Politics,
                2,
                "叛变",
                "令一名其他玩家移除骑士，随后放置一名同级或更低等级的自己的骑士。",
            ),
            Self::Constitution => (Politics, 1, "宪法", "公开获得一分。"),
            Self::Wedding => (
                Politics,
                2,
                "婚礼",
                "分数更高的其他玩家各选择两张资源或商品作为礼物。",
            ),
            Self::Alchemy => (
                Science,
                2,
                "炼金术",
                "掷骰前指定两枚生产骰的点数，事件骰照常结算。",
            ),
            Self::Crane => (
                Science,
                2,
                "起重机",
                "本行动阶段下一次城市升级少支付一张商品。",
            ),
            Self::Engineering => (Science, 1, "工程", "免费修建一座城墙。"),
            Self::Invention => (
                Science,
                2,
                "发明",
                "交换两块数字为 3、4、5、9、10 或 11 的地块数字。",
            ),
            Self::Irrigation => (Science, 2, "灌溉", "自己建筑相邻的每块农田提供两张小麦。"),
            Self::Medicine => (
                Science,
                2,
                "医学",
                "支付两张矿石和一张小麦，将一座村庄升级为城市。",
            ),
            Self::Mining => (Science, 2, "采矿", "自己建筑相邻的每块山脉提供两张矿石。"),
            Self::RoadBuilding => (Science, 2, "修路", "免费放置至多两条道路。"),
            Self::Smithing => (Science, 2, "锻造", "免费晋升至多两名自己的骑士。"),
            Self::Printing => (Science, 1, "印刷术", "公开获得一分。"),
        };
        CardInfo {
            track,
            copies,
            name,
            description,
        }
    }

    pub fn victory(self) -> bool {
        matches!(self, Self::Constitution | Self::Printing)
    }

    pub fn decks() -> [Vec<Self>; 3] {
        std::array::from_fn(|track| {
            let mut deck: Vec<_> = Self::ALL
                .into_iter()
                .filter(|card| card.info().track.index() == track)
                .flat_map(|card| std::iter::repeat_n(card, card.info().copies))
                .collect();
            fastrand::shuffle(&mut deck);
            deck
        })
    }
}

impl Game {
    pub fn draw_progress(&mut self, player: usize, track: Track) {
        let Some(card) = self
            .cities
            .as_mut()
            .and_then(|cities| cities.decks[track.index()].pop())
        else {
            return;
        };
        if card.victory() {
            self.players[player].revealed.push(card);
            self.record(
                Some(player),
                "award",
                format!(
                    "{}获得{}，增加一分",
                    self.players[player].name,
                    card.info().name
                ),
                None,
            );
        } else {
            self.players[player].progress.push(card);
            self.record(
                Some(player),
                "progress",
                format!(
                    "{}取得一张{}进步卡",
                    self.players[player].name,
                    track.name()
                ),
                None,
            );
            self.private_event(vec![player], format!("取得{}", card.info().name));
            if player != self.turn.player && self.players[player].progress.len() > 4 {
                self.pending.push_front(Effect::ProgressDiscard { player });
            }
        }
    }

    pub fn discard_progress(&mut self, player: usize, index: usize) -> Result<(), String> {
        let card = *self.players[player]
            .progress
            .get(index)
            .ok_or("请选择一张进步卡")?;
        self.players[player].progress.remove(index);
        self.cities.as_mut().ok_or("当前模式没有进步卡")?.decks[card.info().track.index()]
            .insert(0, card);
        self.record(
            Some(player),
            "progress",
            format!("{}归还了一张进步卡", self.players[player].name),
            None,
        );
        self.private_event(vec![player], format!("归还{}", card.info().name));
        Ok(())
    }

    pub fn progress_discard_prompt(&self, player: usize, active: bool, prompt: &mut Prompt) {
        prompt.title = format!(
            "归还 {} 张进步卡",
            self.players[player].progress.len().saturating_sub(4)
        );
        if active {
            prompt.choices = self.players[player]
                .progress
                .iter()
                .enumerate()
                .map(|(value, card)| Pick {
                    value,
                    label: card.info().name.into(),
                    target: None,
                })
                .collect();
        }
    }
}
