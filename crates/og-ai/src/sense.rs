//! Sensing: what an agent sees and hears. The `og.Perception` and
//! `og.Perceivable` records, and the seams the host fills: a physics ray
//! for sight ([`SightRay`]) and the tick's sounds ([`Hearing`]).

use crate::nav::system::Transform;
use og_core::glam::Vec3;
use og_core::{Reflect, Result};
use og_ecs::{Entity, SystemContext};
use std::sync::Arc;

/// What an agent sees and hears, and how.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Perception")]
pub struct Perception {
    /// How far it sees.
    #[reflect(min = 0, max = 500, default = 8, unit = "m", tunable)]
    pub sight: f32,
    /// The width of its view cone (360: all round).
    #[reflect(min = 0, max = 360, default = 120, unit = "deg", tunable)]
    pub fov: f32,
    /// Its eyes are this far above its position (+y).
    #[reflect(name = "eyeHeight", min = -10, max = 10, unit = "m")]
    pub eye_height: f32,
    /// Colliders hide what is behind them.
    #[reflect(default = 1)]
    pub occlude: bool,
    /// Collision layers that block sight (0: every layer).
    pub occluders: u32,
    /// How far it hears a sound of loudness 1.
    #[reflect(min = 0, max = 500, default = 6, unit = "m", tunable)]
    pub hearing: f32,
    /// How long a lost target stays its target.
    #[reflect(min = 0, max = 600, default = 3, unit = "s", tunable)]
    pub memory: f32,
    /// Its team: it perceives og.Perceivable entities of other teams.
    pub team: u32,
    /// Where it looks; zero: the way it last moved (before it moves, the way its Transform faces: +Z in 3D, +X in 2D).
    pub forward: Vec3,
    /// What it sees or remembers seeing (written by og.ai).
    pub target: u64,
    /// It sees the target now (written by og.ai).
    pub sees: bool,
    /// Where it last saw the target (written by og.ai).
    #[reflect(name = "lastSeen")]
    pub last_seen: Vec3,
    /// Seconds since it last saw a target (written by og.ai).
    #[reflect(name = "sinceSeen")]
    pub since_seen: f32,
    /// It heard something this tick (written by og.ai).
    pub heard: bool,
    /// Where it last heard something (written by og.ai).
    #[reflect(name = "heardAt")]
    pub heard_at: Vec3,
    /// Seconds since it last heard something (written by og.ai).
    #[reflect(name = "sinceHeard")]
    pub since_heard: f32,
}

impl Default for Perception {
    fn default() -> Self {
        Perception {
            sight: 8.0,
            fov: 120.0,
            eye_height: 0.0,
            occlude: true,
            occluders: 0,
            hearing: 6.0,
            memory: 3.0,
            team: 0,
            forward: Vec3::ZERO,
            target: 0,
            sees: false,
            last_seen: Vec3::ZERO,
            since_seen: NEVER,
            heard: false,
            heard_at: Vec3::ZERO,
            since_heard: NEVER,
        }
    }
}

/// `sinceSeen` and `sinceHeard` before the first time.
pub const NEVER: f32 = 1.0e6;

/// Something agents of other teams can see.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Perceivable")]
pub struct Perceivable {
    /// Its team (agents of the same team ignore it).
    #[reflect(default = 1)]
    pub team: u32,
    /// What agents look at is this far above its position (+y).
    #[reflect(name = "eyeHeight", min = -10, max = 10, unit = "m")]
    pub eye_height: f32,
}

impl Default for Perceivable {
    fn default() -> Self {
        Perceivable {
            team: 1,
            eye_height: 0.0,
        }
    }
}

/// `og.Parent`, og-ecs's: a sound made by an agent's child is the
/// agent's own.
pub use og_ecs::transform::Parent;

/// A physics ray for sight: (origin, direction, max distance, layer mask,
/// entity to ignore) → the first hit's distance and entity.
pub type SightRay =
    Arc<dyn Fn(Vec3, Vec3, f32, u32, Entity) -> Option<(f32, Entity)> + Send + Sync>;

/// Where `og.ai` hears the world's sounds from. og-ai links no audio:
/// the host reads its own sound records and answers with what carries.
#[derive(Clone)]
pub struct Hearing {
    /// The records `sounds` reads, declared as `og.ai`'s reads (registered
    /// by the host before [`install`]).
    pub reads: Vec<String>,
    /// The sounds that started this tick and carry in space: the entity
    /// that made each and its loudness (1 a normal sound, clamped to
    /// 0..4). One whose entity is gone or has no og.Transform is not heard.
    pub sounds: HeardSounds,
}

/// [`Hearing::sounds`].
pub type HeardSounds = Arc<dyn Fn(&SystemContext<'_>) -> Result<Vec<(Entity, f32)>> + Send + Sync>;

/// A noise a game made (`og.ai.noise`), heard next tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Noise {
    pub position: Vec3,
    pub loudness: f32,
    /// The entity making it (0: none): it, and the agents it is a child
    /// of, do not hear it.
    pub source: u64,
}

/// An entity's position in the world: its og.Transform composed down its
/// og.Parent chain.
pub(crate) fn world_position(ctx: &SystemContext<'_>, e: Entity) -> Vec3 {
    let Ok(t) = ctx.get::<Transform>(e) else {
        return Vec3::ZERO;
    };
    let mut p = t.position;
    let mut at = e;
    for _ in 0..32 {
        let Ok(parent) = ctx.get::<Parent>(at) else {
            break;
        };
        at = Entity::from_bits(parent.entity);
        if !ctx.is_alive(at) {
            break;
        }
        let Ok(pt) = ctx.get::<Transform>(at) else {
            break;
        };
        p = pt.position + pt.quaternion.normalize() * (pt.scale * p);
    }
    p
}

/// `e` and the entities above it by og.Parent (none for 0).
pub(crate) fn lineage(ctx: &SystemContext<'_>, e: u64) -> Vec<u64> {
    let mut out = Vec::new();
    let mut at = e;
    while at != 0 && out.len() < 32 && !out.contains(&at) {
        out.push(at);
        let ent = Entity::from_bits(at);
        if !ctx.is_alive(ent) {
            break;
        }
        at = ctx.get::<Parent>(ent).map_or(0, |p| p.entity);
    }
    out
}

pub(crate) struct Seen {
    pub(crate) e: Entity,
    pub(crate) pos: Vec3,
    pub(crate) team: u32,
}
