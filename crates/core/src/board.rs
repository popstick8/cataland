use std::collections::BTreeMap;

use fastrand::Rng;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum Resource {
    Wood,
    Brick,
    Wool,
    Grain,
    Ore,
    Cloth,
    Coin,
    Paper,
}

impl Resource {
    pub const ALL: [Self; 8] = [
        Self::Wood,
        Self::Brick,
        Self::Wool,
        Self::Grain,
        Self::Ore,
        Self::Cloth,
        Self::Coin,
        Self::Paper,
    ];
    pub const BASE: [Self; 5] = [Self::Wood, Self::Brick, Self::Wool, Self::Grain, Self::Ore];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Wood => "木材",
            Self::Brick => "砖块",
            Self::Wool => "羊毛",
            Self::Grain => "小麦",
            Self::Ore => "矿石",
            Self::Cloth => "布匹",
            Self::Coin => "铸币",
            Self::Paper => "纸张",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum Terrain {
    Forest,
    Hills,
    Pasture,
    Fields,
    Mountains,
    Desert,
}

impl Terrain {
    pub fn resource(self) -> Option<Resource> {
        match self {
            Self::Forest => Some(Resource::Wood),
            Self::Hills => Some(Resource::Brick),
            Self::Pasture => Some(Resource::Wool),
            Self::Fields => Some(Resource::Grain),
            Self::Mountains => Some(Resource::Ore),
            Self::Desert => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Hex {
    pub x: f32,
    pub y: f32,
    pub terrain: Terrain,
    pub number: u8,
    pub vertices: [usize; 6],
    pub edges: [usize; 6],
    pub neighbors: Vec<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub neighbors: Vec<usize>,
    pub edges: Vec<usize>,
    pub hexes: Vec<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct Edge {
    pub vertices: [usize; 2],
    pub hexes: Vec<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct Harbor {
    pub edge: usize,
    pub resource: Option<Resource>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct Board {
    pub hexes: Vec<Hex>,
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,
    pub harbors: Vec<Harbor>,
}

impl Board {
    pub fn generate(big: bool, rng: &mut Rng) -> Self {
        let mut board = Self {
            hexes: Vec::new(),
            vertices: Vec::new(),
            edges: Vec::new(),
            harbors: Vec::new(),
        };
        let rows: &[i32] = if big {
            &[3, 4, 5, 6, 5, 4, 3]
        } else {
            &[3, 4, 5, 4, 3]
        };
        let quantities = if big {
            [6, 5, 6, 6, 5, 2]
        } else {
            [4, 3, 4, 4, 3, 1]
        };
        let terrain = [
            Terrain::Forest,
            Terrain::Hills,
            Terrain::Pasture,
            Terrain::Fields,
            Terrain::Mountains,
            Terrain::Desert,
        ];
        let mut tiles: Vec<_> = terrain
            .into_iter()
            .zip(quantities)
            .flat_map(|(terrain, count)| std::iter::repeat_n(terrain, count))
            .collect();
        rng.shuffle(&mut tiles);
        let mut vertices = BTreeMap::new();
        let mut edges = BTreeMap::new();
        let corners = [(0, -2), (1, -1), (1, 1), (0, 2), (-1, 1), (-1, -1)];
        for (row, &width) in rows.iter().enumerate() {
            for column in 0..width {
                let x = column * 2 - width + 1;
                let y = (row as i32 - rows.len() as i32 / 2) * 3;
                let id = board.hexes.len();
                let mut hex = Hex {
                    x: x as f32 * 3.0_f32.sqrt() / 2.0,
                    y: y as f32 / 2.0,
                    terrain: tiles[id],
                    number: 0,
                    vertices: [0; 6],
                    edges: [0; 6],
                    neighbors: Vec::new(),
                };
                for (corner, (dx, dy)) in corners.iter().enumerate() {
                    let key = (x + dx, y + dy);
                    let vertex = *vertices.entry(key).or_insert_with(|| {
                        let vertex = board.vertices.len();
                        board.vertices.push(Vertex {
                            x: key.0 as f32 * 3.0_f32.sqrt() / 2.0,
                            y: key.1 as f32 / 2.0,
                            neighbors: Vec::new(),
                            edges: Vec::new(),
                            hexes: Vec::new(),
                        });
                        vertex
                    });
                    board.vertices[vertex].hexes.push(id);
                    hex.vertices[corner] = vertex;
                }
                for side in 0..6 {
                    let a = hex.vertices[side];
                    let b = hex.vertices[(side + 1) % 6];
                    let key = (a.min(b), a.max(b));
                    let edge = *edges.entry(key).or_insert_with(|| {
                        let edge = board.edges.len();
                        board.edges.push(Edge {
                            vertices: [a, b],
                            hexes: Vec::new(),
                        });
                        board.vertices[a].neighbors.push(b);
                        board.vertices[b].neighbors.push(a);
                        board.vertices[a].edges.push(edge);
                        board.vertices[b].edges.push(edge);
                        edge
                    });
                    board.edges[edge].hexes.push(id);
                    hex.edges[side] = edge;
                }
                board.hexes.push(hex);
            }
        }
        for edge in &board.edges {
            if let [a, b] = edge.hexes[..] {
                board.hexes[a].neighbors.push(b);
                board.hexes[b].neighbors.push(a);
            }
        }
        let mut candidates: Vec<_> = board
            .hexes
            .iter()
            .enumerate()
            .filter(|(_, hex)| hex.terrain != Terrain::Desert)
            .map(|(id, _)| id)
            .collect();
        rng.shuffle(&mut candidates);
        let mut hot = Vec::new();
        assert!(select_hot(
            &board,
            &candidates,
            &mut hot,
            if big { 6 } else { 4 }
        ));
        rng.shuffle(&mut hot);
        for (i, &id) in hot.iter().enumerate() {
            board.hexes[id].number = if i % 2 == 0 { 6 } else { 8 };
        }
        let mut numbers: Vec<_> = [2, 3, 4, 5, 9, 10, 11, 12]
            .into_iter()
            .flat_map(|value| {
                let count = if value == 2 || value == 12 {
                    1 + usize::from(big)
                } else {
                    2 + usize::from(big)
                };
                std::iter::repeat_n(value, count)
            })
            .collect();
        rng.shuffle(&mut numbers);
        for (id, number) in candidates
            .into_iter()
            .filter(|id| !hot.contains(id))
            .zip(numbers)
        {
            board.hexes[id].number = number;
        }
        let mut coast: Vec<_> = board
            .edges
            .iter()
            .enumerate()
            .filter(|(_, edge)| edge.hexes.len() == 1)
            .map(|(id, edge)| {
                let a = &board.vertices[edge.vertices[0]];
                let b = &board.vertices[edge.vertices[1]];
                (id, (a.y + b.y).atan2(a.x + b.x))
            })
            .collect();
        coast.sort_by(|a, b| a.1.total_cmp(&b.1));
        let mut harbors = vec![None; if big { 5 } else { 4 }];
        harbors.extend(Resource::BASE.map(Some));
        if big {
            harbors.push(Some(Resource::Wool));
        }
        rng.shuffle(&mut harbors);
        let rotation = rng.usize(..coast.len());
        let count = harbors.len();
        for (i, resource) in harbors.into_iter().enumerate() {
            board.harbors.push(Harbor {
                edge: coast[(rotation + i * coast.len() / count) % coast.len()].0,
                resource,
            });
        }
        board
    }
}

fn select_hot(
    board: &Board,
    candidates: &[usize],
    selected: &mut Vec<usize>,
    count: usize,
) -> bool {
    if selected.len() == count {
        return true;
    }
    for (i, &id) in candidates.iter().enumerate() {
        if candidates.len() - i < count - selected.len() {
            break;
        }
        if selected
            .iter()
            .all(|other| !board.hexes[id].neighbors.contains(other))
        {
            selected.push(id);
            if select_hot(board, &candidates[i + 1..], selected, count) {
                return true;
            }
            selected.pop();
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_islands_have_consistent_topology() {
        for big in [false, true] {
            for seed in 0..128 {
                let board = Board::generate(big, &mut Rng::with_seed(seed));
                assert_eq!(board.hexes.len(), if big { 30 } else { 19 });
                assert_eq!(
                    board.vertices.len() + board.hexes.len(),
                    board.edges.len() + 1
                );
                if !big {
                    assert_eq!((board.vertices.len(), board.edges.len()), (54, 72));
                }
                for (id, hex) in board.hexes.iter().enumerate() {
                    for &vertex in &hex.vertices {
                        assert!(board.vertices[vertex].hexes.contains(&id));
                    }
                    for &other in &hex.neighbors {
                        assert!(board.hexes[other].neighbors.contains(&id));
                        assert!(
                            !matches!(hex.number, 6 | 8)
                                || !matches!(board.hexes[other].number, 6 | 8)
                        );
                    }
                }
                let mut occupied = Vec::new();
                for harbor in &board.harbors {
                    let edge = &board.edges[harbor.edge];
                    assert_eq!(edge.hexes.len(), 1);
                    for &vertex in &edge.vertices {
                        assert!(!occupied.contains(&vertex));
                        occupied.push(vertex);
                    }
                }
            }
        }
    }
}
