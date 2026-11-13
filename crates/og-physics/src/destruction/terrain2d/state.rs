//! What `og.destruction2d` keeps between ticks: world state, saved as the
//! `og.destruction2d` state block.

use super::{Cell, PixelTerrain};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// How a terrain's cells are drawn (copied into each piece cut from it).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Look {
    pub cell: f32,
    pub color: [f32; 3],
    pub bedrock: [f32; 3],
    pub grain: f32,
    pub seed: u32,
    pub layer: i32,
}

/// A piece's collider settings, the terrain's when it was cut.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Body {
    pub friction: f32,
    pub density: f32,
    pub layers: u32,
    pub mask: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ground {
    pub grid: PixelTerrain,
    /// The width, height and chunk it was filled at: another starts it over.
    pub shape: [u32; 3],
    /// Where its colliders stand: its og.Transform's position.
    pub origin: [f32; 3],
    /// The bits of what its colliders were built with (cell size,
    /// tolerance, friction, layers, mask): another rebuilds them all.
    pub built: [u32; 5],
    pub look: Look,
    /// Per chunk with a collider: its entity and its mesh handle.
    pub colliders: BTreeMap<u32, (u64, u64)>,
    /// Counts changes to its cells (what drawing repaints on).
    pub revision: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Piece {
    pub terrain: u64,
    pub width: u32,
    pub height: u32,
    pub cells: Vec<Cell>,
    /// Its cell (0, 0)'s place in the terrain it was cut from (its grain).
    pub grain: [i32; 2],
    pub look: Look,
    pub body: Body,
    pub mesh: u64,
    pub tick: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Op {
    Carve {
        entity: u64,
        point: [f32; 2],
        radius: f32,
    },
    Fill {
        entity: u64,
        point: [f32; 2],
        radius: f32,
        bedrock: bool,
    },
    Explode {
        point: [f32; 2],
        radius: f32,
        debris: f32,
        cells: u32,
        speed: f32,
    },
    Fracture {
        entity: u64,
        pieces: u32,
    },
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub terrains: BTreeMap<u64, Ground>,
    pub pieces: BTreeMap<u64, Piece>,
    pub ops: Vec<Op>,
    /// The generated meshes the colliders name.
    pub meshes: BTreeSet<u64>,
}
