use serde::{Deserialize, Serialize};

use crate::{
    game::{Action, Building, BuildingKind, Effect, Game, Player, Target},
    view::{Pick, Prompt},
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum NeutralBuild {
    Road,
    Settlement,
}

impl Game {
    pub fn setup_neutrals(&mut self) {
        if self.humans != 2 {
            return;
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
        } else if matches!(kind, NeutralBuild::Settlement) {
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
            NeutralBuild::Road => "道路",
            NeutralBuild::Settlement => "村庄",
        };
        prompt.title = owner.map_or_else(
            || format!("选择中立势力，免费建造{name}"),
            |owner| format!("为{}建造{name}", self.players[owner].name),
        );
        if !active {
            return;
        }
        prompt.choices = if let Some(owner) = owner {
            self.neutral_sites(owner, kind)
                .into_iter()
                .map(|value| Pick {
                    value,
                    label: format!("建造{name}"),
                    target: Some(match kind {
                        NeutralBuild::Road => Target::Edge(value),
                        NeutralBuild::Settlement => Target::Vertex(value),
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
