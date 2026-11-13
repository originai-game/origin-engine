//! What `og.destruction` keeps of its props between ticks: world state,
//! saved as the `og.destruction` state block (snapshots, restores, replays
//! and the world hash carry it).

use super::debris::{Crumb, Debris, MAX_CRUMBS};
use crate::destruction::{FractureAsset, Instance};
use glam::Vec3;
use og_core::{Error, ErrorKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

pub const CAUSE_IMPACT: u32 = 0;

pub const CAUSE_DAMAGE: u32 = 1;

pub const CAUSE_EXPLOSION: u32 = 2;

pub const CAUSE_STRESS: u32 = 3;

pub const CAUSE_SHATTER: u32 = 4;

pub const CAUSE_FRACTURE: u32 = 5;

/// og.Chunk.looseTick of an anchored chunk.
pub const ANCHORED: u32 = u32::MAX;

/// Physics mesh handles of the chunks' hulls: the tag, the model's asset
/// index (24 bits) and the chunk (16 bits), below 2^53.
pub const HULL_TAG: u64 = 0xd5 << 40;

/// The physics mesh of chunk `chunk` of the model whose asset handle is
/// `asset`.
pub fn hull_handle(asset: u64, chunk: u32) -> u64 {
    HULL_TAG | ((asset & 0xff_ffff) << 16) | u64::from(chunk & 0xffff)
}

/// What `og.destruction` keeps: world state.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct State {
    /// By the og.Destructible's entity bits.
    pub props: BTreeMap<u64, Prop>,
    /// Calls queued for the next run, in the order made.
    pub ops: Vec<Op>,
    /// breakFx and hitFx bursts: (entity bits, the tick it is despawned at).
    pub fx: Vec<(u64, u64)>,
    /// Fine pieces broken off as visuals only (og.Destructible.debris):
    /// the renderer's debris layer moves and draws them from here; they
    /// are never bodies.
    pub debris: Vec<Debris>,
    /// Joints broken with a debris emitter set: the grit each sheds
    /// (clay and mortar chips, the renderer's debris layer's) flies from
    /// here.
    pub crumbs: Vec<Crumb>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Prop {
    /// Its og.Model's asset handle.
    pub asset: u64,
    pub inst: Instance,
    /// The model's world pose when its chunks were placed: position,
    /// rotation (x y z w) and scale.
    pub frame: [f32; 10],
    /// Per chunk: its entity's bits (0: not spawned, or gone).
    pub entities: Vec<u64>,
    /// Per chunk: the entity whose og.RigidBody moves it (0: static).
    pub body: Vec<u64>,
    /// Per chunk: the tick it broke loose (u64::MAX while anchored).
    pub loose: Vec<u64>,
    /// Its og.Collider when its chunks were placed: theirs copy its
    /// friction, restitution, layers and mask.
    pub collider: Option<Vec<u8>>,
    /// Its chunks are entities.
    pub placed: bool,
    /// Its anchors hold it (it stood when placed, and `anchored`).
    pub anchors: bool,
    /// Its first bond broke: the model is hidden, the chunks drawn.
    pub fractured: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Op {
    Damage {
        entity: u64,
        point: [f32; 3],
        radius: f32,
        inner: f32,
        amount: f32,
        /// The blow's momentum (N·s); absent in worlds saved before it.
        #[serde(default)]
        impulse: [f32; 3],
    },
    Explode {
        point: [f32; 3],
        radius: f32,
        inner: f32,
        damage: f32,
        impulse: f32,
    },
    Fracture {
        entity: u64,
    },
}

/// The fracture assets of an engine's loaded models, by asset handle.
pub type Assets = BTreeMap<u64, Arc<FractureAsset>>;

pub const MAGIC: &[u8; 8] = b"DESTRUC2";

impl State {
    /// Fine pieces and grit: gone `life` ticks after they broke off (when
    /// they have a lifetime), or with their prop, and the oldest past
    /// `max_debris` (what the renderer's debris layer holds) and
    /// MAX_CRUMBS.
    pub fn trim_pieces(&mut self, tick: u64, life: Option<u64>, max_debris: usize) {
        let old = |born: u64| life.is_some_and(|life| tick >= born.saturating_add(life));
        let props = &self.props;
        self.debris
            .retain(|d| props.contains_key(&d.prop) && !old(d.tick));
        let over = self.debris.len().saturating_sub(max_debris);
        self.debris.drain(..over);
        self.crumbs
            .retain(|c| props.contains_key(&c.prop) && !old(c.tick));
        let over = self.crumbs.len().saturating_sub(MAX_CRUMBS);
        self.crumbs.drain(..over);
    }

    /// The og.destruction state block's bytes.
    pub fn save(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(MAGIC);
        out.extend(bincode::serialize(self).expect("the og.destruction state serialises"));
    }

    /// The state [`State::save`] wrote.
    pub fn decode(bytes: &[u8]) -> og_core::Result<State> {
        let rest = bytes
            .strip_prefix(MAGIC)
            .ok_or_else(|| invalid("not an og.destruction state"))?;
        bincode::deserialize(rest).map_err(|e| invalid(e.to_string()))
    }

    /// The origin moved by `d`: the frames and queued points are in the
    /// world.
    pub fn shift(&mut self, d: glam::DVec3) {
        let m = |p: &mut [f32]| {
            let v = (Vec3::new(p[0], p[1], p[2]).as_dvec3() + d).as_vec3();
            p[..3].copy_from_slice(&v.to_array());
        };
        for p in self.props.values_mut() {
            m(&mut p.frame);
        }
        for d in &mut self.debris {
            m(&mut d.pose);
        }
        for c in &mut self.crumbs {
            m(&mut c.point);
        }
        for op in &mut self.ops {
            match op {
                Op::Damage { point, .. } | Op::Explode { point, .. } => m(point),
                Op::Fracture { .. } => {}
            }
        }
    }
}

fn invalid(m: impl Into<String>) -> Error {
    Error::new(ErrorKind::Invalid, m.into())
}
