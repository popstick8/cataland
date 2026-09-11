use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Mode, game::Game};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct Awards {
    pub road: Option<usize>,
    pub army: Option<usize>,
    pub lengths: Vec<u8>,
}

impl Game {
    pub fn longest_route(&self, player: usize) -> u8 {
        let mut used = vec![false; self.board.edges.len()];
        (0..self.board.vertices.len())
            .map(|vertex| self.walk_road(player, vertex, &mut used, false))
            .max()
            .unwrap_or(0)
    }

    fn walk_road(&self, player: usize, vertex: usize, used: &mut [bool], arrived: bool) -> u8 {
        if arrived && self.blocked(player, vertex) {
            return 0;
        }
        let mut longest = 0;
        for &edge in &self.board.vertices[vertex].edges {
            if used[edge] || self.roads[edge] != Some(player) {
                continue;
            }
            used[edge] = true;
            let endpoints = self.board.edges[edge].vertices;
            let next = if endpoints[0] == vertex {
                endpoints[1]
            } else {
                endpoints[0]
            };
            longest = longest.max(1 + self.walk_road(player, next, used, true));
            used[edge] = false;
        }
        longest
    }

    pub fn update_awards(&mut self) {
        let lengths: Vec<_> = (0..self.players.len())
            .map(|player| self.longest_route(player))
            .collect();
        let road = holder(self.awards.road, &lengths, 5);
        if road != self.awards.road {
            self.record(
                road,
                "award",
                road.map_or_else(
                    || "最长道路暂时无人持有".into(),
                    |player| crate::text!("{0}获得最长道路", self.player_name(player)),
                ),
                None,
            );
        }
        self.awards.road = road;
        self.awards.lengths = lengths;
        if self.mode == Mode::Base {
            let armies: Vec<_> = self.players.iter().map(|player| player.army).collect();
            let army = holder(self.awards.army, &armies, 3);
            if army != self.awards.army {
                self.record(
                    army,
                    "award",
                    army.map_or_else(
                        || "最大军队暂时无人持有".into(),
                        |player| crate::text!("{0}获得最大军队", self.player_name(player)),
                    ),
                    None,
                );
            }
            self.awards.army = army;
        }
    }
}

fn holder(previous: Option<usize>, values: &[u8], minimum: u8) -> Option<usize> {
    let highest = values.iter().copied().max()?;
    if highest < minimum {
        return None;
    }
    if previous.is_some_and(|player| values[player] == highest) {
        return previous;
    }
    let mut leaders = values
        .iter()
        .enumerate()
        .filter(|(_, value)| **value == highest)
        .map(|(player, _)| player);
    let leader = leaders.next()?;
    leaders.next().is_none().then_some(leader)
}
