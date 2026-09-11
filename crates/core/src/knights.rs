use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    cities::Track,
    game::{Action, Cards, Effect, Game, Stage, Target},
    view::{AvailableAction, Pick, Prompt},
};

pub const KNIGHT: Cards = [0, 0, 1, 0, 1, 0, 0, 0];
pub const ACTIVATE: Cards = [0, 0, 0, 1, 0, 0, 0, 0];

#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
pub struct Knight {
    pub player: usize,
    pub level: u8,
    pub active: bool,
    pub activated: u32,
    pub upgraded: u32,
}

impl Game {
    pub fn knight(&self, vertex: usize) -> Option<&Knight> {
        self.cities.as_ref()?.knights.get(vertex)?.as_ref()
    }

    pub fn knight_count(&self, player: usize, level: u8) -> usize {
        self.cities.as_ref().map_or(0, |cities| {
            cities
                .knights
                .iter()
                .flatten()
                .filter(|knight| knight.player == player && knight.level == level)
                .count()
        })
    }

    pub fn knight_sites(&self, player: usize) -> Vec<usize> {
        if self.cities.is_none() {
            return Vec::new();
        }
        self.board
            .vertices
            .iter()
            .enumerate()
            .filter(|(vertex, point)| {
                self.buildings[*vertex].is_none()
                    && self.knight(*vertex).is_none()
                    && point
                        .edges
                        .iter()
                        .any(|&edge| self.roads[edge] == Some(player))
            })
            .map(|(vertex, _)| vertex)
            .collect()
    }

    pub fn place_knight(
        &mut self,
        player: usize,
        vertex: usize,
        level: u8,
        active: bool,
        cost: &Cards,
    ) -> Result<(), String> {
        if !(1..=3).contains(&level)
            || player >= self.players.len()
            || (player >= self.humans && (level > 2 || active))
            || self.knight_count(player, level) >= 2
            || !self.knight_sites(player).contains(&vertex)
        {
            return Err("请选择连接自己道路的空位置，并使用供应中的骑士棋子".into());
        }
        self.pay(player, cost)?;
        self.cities.as_mut().ok_or("当前模式没有骑士棋子")?.knights[vertex] = Some(Knight {
            player,
            level,
            active,
            activated: 0,
            upgraded: 0,
        });
        self.record(
            Some(player),
            "knight",
            format!("{}放置了一名 {level} 级骑士", self.players[player].name),
            Some(Target::Vertex(vertex)),
        );
        Ok(())
    }

    pub fn can_promote(&self, player: usize, vertex: usize) -> bool {
        self.knight(vertex).is_some_and(|knight| {
            knight.player == player
                && knight.level < 3
                && knight.upgraded != self.turn.number
                && self.knight_count(player, knight.level + 1) < 2
                && (knight.level == 1
                    || (player < self.humans
                        && self.players[player].upgrades[Track::Politics.index()] >= 3))
        })
    }

    pub fn promote_knight(
        &mut self,
        player: usize,
        vertex: usize,
        cost: &Cards,
    ) -> Result<u8, String> {
        if !self.can_promote(player, vertex) {
            return Err("需要可晋升的骑士、对应棋子与城市发展等级".into());
        }
        self.pay(player, cost)?;
        let knight = self
            .cities
            .as_mut()
            .and_then(|cities| cities.knights[vertex].as_mut())
            .ok_or("这个位置没有骑士")?;
        let previous = knight.level;
        knight.level += 1;
        knight.upgraded = self.turn.number;
        self.record(
            Some(player),
            "knight",
            format!(
                "{}将骑士晋升为 {} 级",
                self.players[player].name,
                previous + 1
            ),
            Some(Target::Vertex(vertex)),
        );
        Ok(previous)
    }

    pub fn activate_knight(
        &mut self,
        player: usize,
        vertex: usize,
        cost: &Cards,
    ) -> Result<(), String> {
        if !self
            .knight(vertex)
            .is_some_and(|knight| knight.player == player && !knight.active)
            || player >= self.humans
        {
            return Err("请选择自己的未激活骑士".into());
        }
        self.pay(player, cost)?;
        let knight = self
            .cities
            .as_mut()
            .and_then(|cities| cities.knights[vertex].as_mut())
            .ok_or("这个位置没有骑士")?;
        knight.active = true;
        knight.activated = self.turn.number;
        self.record(
            Some(player),
            "knight",
            format!("{}激活了一名骑士", self.players[player].name),
            Some(Target::Vertex(vertex)),
        );
        Ok(())
    }

    pub fn can_act(&self, player: usize, vertex: usize) -> bool {
        self.knight(vertex).is_some_and(|knight| {
            knight.player == player && knight.active && knight.activated != self.turn.number
        })
    }

    pub fn reachable(&self, player: usize, from: usize) -> Vec<usize> {
        if from >= self.board.vertices.len() {
            return Vec::new();
        }
        let mut visited = vec![false; self.board.vertices.len()];
        let mut queue = VecDeque::from([from]);
        let mut result = Vec::new();
        visited[from] = true;
        while let Some(vertex) = queue.pop_front() {
            for &edge in &self.board.vertices[vertex].edges {
                if self.roads[edge] != Some(player) {
                    continue;
                }
                let ends = self.board.edges[edge].vertices;
                let next = if ends[0] == vertex { ends[1] } else { ends[0] };
                if visited[next] {
                    continue;
                }
                visited[next] = true;
                result.push(next);
                if !self.blocked(player, next) {
                    queue.push_back(next);
                }
            }
        }
        result
    }

    pub fn move_sites(&self, player: usize, from: usize) -> Vec<usize> {
        if !self.can_act(player, from) {
            return Vec::new();
        }
        let Some(knight) = self.knight(from) else {
            return Vec::new();
        };
        self.reachable(player, from)
            .into_iter()
            .filter(|&vertex| {
                self.buildings[vertex].is_none()
                    && self
                        .knight(vertex)
                        .is_none_or(|target| target.player != player && target.level < knight.level)
            })
            .collect()
    }

    pub fn relocation_sites(&self, knight: &Knight, from: usize) -> Vec<usize> {
        self.reachable(knight.player, from)
            .into_iter()
            .filter(|&vertex| self.buildings[vertex].is_none() && self.knight(vertex).is_none())
            .collect()
    }

    pub fn move_knight(
        &mut self,
        player: usize,
        from: usize,
        to: usize,
    ) -> Result<Option<Knight>, String> {
        if !self.move_sites(player, from).contains(&to) {
            return Err("骑士需要沿自己的道路移动到空位或更弱的敌方骑士处".into());
        }
        let cities = self.cities.as_mut().ok_or("当前模式没有骑士棋子")?;
        let mut knight = cities.knights[from].take().ok_or("这个位置没有骑士")?;
        knight.active = false;
        let displaced = cities.knights[to].replace(knight);
        self.record(
            Some(player),
            "knight",
            format!("{}移动了骑士", self.players[player].name),
            Some(Target::Vertex(to)),
        );
        Ok(displaced)
    }

    pub fn queue_displacement(&mut self, knight: Knight, from: usize) {
        let locations = self.relocation_sites(&knight, from);
        match locations.as_slice() {
            [] => self.record(
                Some(knight.player),
                "knight",
                format!("{}的骑士返回供应", self.players[knight.player].name),
                Some(Target::Vertex(from)),
            ),
            [vertex] => {
                if let Some(cities) = &mut self.cities {
                    cities.knights[*vertex] = Some(knight);
                }
                self.record(
                    Some(knight.player),
                    "knight",
                    format!("{}重新安置了骑士", self.players[knight.player].name),
                    Some(Target::Vertex(*vertex)),
                );
            }
            _ => self.pending.push_front(Effect::Displace {
                player: if knight.player < self.humans {
                    knight.player
                } else {
                    self.turn.player
                },
                knight,
                from,
            }),
        }
    }

    pub fn can_expel(&self, player: usize, vertex: usize) -> bool {
        self.can_act(player, vertex)
            && self
                .robber
                .is_some_and(|hex| self.board.hexes[hex].vertices.contains(&vertex))
    }

    pub fn expel_robber(&mut self, player: usize, vertex: usize) -> Result<(), String> {
        if !self.can_expel(player, vertex) {
            return Err("请选择与强盗相邻且可以行动的骑士".into());
        }
        self.cities
            .as_mut()
            .and_then(|cities| cities.knights[vertex].as_mut())
            .ok_or("这个位置没有骑士")?
            .active = false;
        self.pending.push_front(Effect::Robber { player });
        self.record(
            Some(player),
            "knight",
            format!("{}派骑士驱逐强盗", self.players[player].name),
            Some(Target::Vertex(vertex)),
        );
        Ok(())
    }

    pub fn retire_knight(&mut self, player: usize, vertex: usize) -> Result<(), String> {
        if self.humans != 2
            || !self
                .knight(vertex)
                .is_some_and(|knight| knight.player == player)
        {
            return Err("双人规则中可以交回自己的骑士换取筹码".into());
        }
        let knight = self
            .cities
            .as_mut()
            .and_then(|cities| cities.knights[vertex].take())
            .ok_or("这个位置没有骑士")?;
        self.grant_tokens(player, knight.level);
        self.record(
            Some(player),
            "knight",
            format!("{}交回了一名骑士", self.players[player].name),
            Some(Target::Vertex(vertex)),
        );
        Ok(())
    }

    pub fn defense(&self) -> Vec<u8> {
        let mut strength = vec![0; self.players.len()];
        if let Some(cities) = &self.cities {
            for knight in cities
                .knights
                .iter()
                .flatten()
                .filter(|knight| knight.active)
            {
                strength[knight.player] += knight.level;
            }
        }
        strength
    }

    pub fn knight_actions(&self, player: usize) -> Vec<AvailableAction> {
        let mut actions = Vec::new();
        if self.cities.is_none() {
            return actions;
        }
        if self.stage == Stage::Action {
            if self.knight_count(player, 1) < 2 && self.can_pay(player, &KNIGHT) {
                for vertex in self.knight_sites(player) {
                    actions.push(AvailableAction {
                        action: Action::RecruitKnight { vertex },
                        label: "招募骑士".into(),
                        target: Some(Target::Vertex(vertex)),
                        cost: KNIGHT,
                    });
                }
            }
            for vertex in 0..self.board.vertices.len() {
                if !self
                    .knight(vertex)
                    .is_some_and(|knight| knight.player == player)
                {
                    continue;
                }
                if self.can_promote(player, vertex) && self.can_pay(player, &KNIGHT) {
                    actions.push(AvailableAction {
                        action: Action::PromoteKnight { vertex },
                        label: "晋升骑士".into(),
                        target: Some(Target::Vertex(vertex)),
                        cost: KNIGHT,
                    });
                }
                if self.knight(vertex).is_some_and(|knight| !knight.active)
                    && self.can_pay(player, &ACTIVATE)
                {
                    actions.push(AvailableAction {
                        action: Action::ActivateKnight { vertex },
                        label: "激活骑士".into(),
                        target: Some(Target::Vertex(vertex)),
                        cost: ACTIVATE,
                    });
                }
                if !self.move_sites(player, vertex).is_empty() {
                    actions.push(AvailableAction {
                        action: Action::MoveKnight { vertex },
                        label: "移动骑士".into(),
                        target: Some(Target::Vertex(vertex)),
                        cost: [0; 8],
                    });
                }
                if self.can_expel(player, vertex) {
                    actions.push(AvailableAction {
                        action: Action::ExpelRobber { vertex },
                        label: "驱逐强盗".into(),
                        target: Some(Target::Vertex(vertex)),
                        cost: [0; 8],
                    });
                }
            }
        }
        if self.humans == 2 && matches!(self.stage, Stage::Production | Stage::Action) {
            for vertex in 0..self.board.vertices.len() {
                if let Some(knight) = self.knight(vertex)
                    && knight.player == player
                {
                    actions.push(AvailableAction {
                        action: Action::RetireKnight { vertex },
                        label: format!("交回骑士 · 获得 {} 筹码", knight.level),
                        target: Some(Target::Vertex(vertex)),
                        cost: [0; 8],
                    });
                }
            }
        }
        actions
    }

    pub fn resolve_knight(
        &mut self,
        player: usize,
        index: usize,
        effect: Effect,
        action: Action,
    ) -> Result<(), String> {
        match (effect, action) {
            (Effect::MoveKnight { from, .. }, Action::Pick { value }) => {
                let displaced = self.move_knight(player, from, value)?;
                self.pending.remove(index);
                if let Some(knight) = displaced {
                    self.queue_displacement(knight, value);
                }
            }
            (Effect::MoveKnight { .. }, Action::Skip) => {
                self.pending.remove(index);
            }
            (Effect::Displace { knight, from, .. }, Action::Pick { value }) => {
                if !self.relocation_sites(&knight, from).contains(&value) {
                    return Err("请选择该骑士沿原道路网络可到达的空位置".into());
                }
                self.cities.as_mut().ok_or("当前模式没有骑士棋子")?.knights[value] = Some(knight);
                self.pending.remove(index);
                self.record(
                    Some(knight.player),
                    "knight",
                    format!("{}重新安置了骑士", self.players[knight.player].name),
                    Some(Target::Vertex(value)),
                );
            }
            _ => return Err("请完成骑士的位置选择".into()),
        }
        Ok(())
    }

    pub fn knight_prompt(&self, effect: &Effect, prompt: &mut Prompt, active: bool) {
        match effect {
            Effect::MoveKnight { player, from } => {
                prompt.title = "选择骑士的新位置".into();
                if active {
                    prompt.can_skip = true;
                    prompt.choices = self
                        .move_sites(*player, *from)
                        .into_iter()
                        .map(|value| Pick {
                            value,
                            label: "移动骑士".into(),
                            target: Some(Target::Vertex(value)),
                        })
                        .collect();
                }
            }
            Effect::Displace { knight, from, .. } => {
                prompt.title = format!("重新安置{}的骑士", self.players[knight.player].name);
                if active {
                    prompt.choices = self
                        .relocation_sites(knight, *from)
                        .into_iter()
                        .map(|value| Pick {
                            value,
                            label: "放置骑士".into(),
                            target: Some(Target::Vertex(value)),
                        })
                        .collect();
                }
            }
            _ => {}
        }
    }
}
