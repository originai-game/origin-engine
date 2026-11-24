//! The fine pieces and the grit a breaking prop sheds: visuals only, moved
//! and drawn by the renderer's debris layer from what is recorded here.

use serde::{Deserialize, Serialize};

/// A broken joint's grit as it fell out: the renderer throws its chips
/// from here.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Crumb {
    /// Its og.Destructible's entity bits and its bond.
    pub prop: u64,
    pub bond: u32,
    /// The tick it broke.
    pub tick: u64,
    /// The joint's centre and its normal in the world then.
    pub point: [f32; 3],
    pub normal: [f32; 3],
    /// What the blow or the blast gave the pieces beside it.
    pub velocity: [f32; 3],
}

/// The most joints a prop sheds grit from in one tick (spread over all
/// that broke), and the most at once.
pub const CRUMBS_A_TICK: usize = 48;

pub const MAX_CRUMBS: usize = 384;

/// A fine piece as it broke off: the renderer simulates it from here.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Debris {
    /// Its og.Destructible's entity bits and its chunk.
    pub prop: u64,
    pub chunk: u32,
    /// The tick it broke off.
    pub tick: u64,
    /// Its centre of mass, rotation (x y z w) and scale then.
    pub pose: [f32; 10],
    pub velocity: [f32; 3],
    pub angular: [f32; 3],
}
