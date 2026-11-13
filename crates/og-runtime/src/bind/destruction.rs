//! `og.destruction` (docs/design.md §2.7, "Destruction"): props that break
//! into the chunks the cook cut them into (og-cook `cook/fracture.rs`).
//!
//! - **The asset** (`<model>#fracture`, og_physics::destruction::FractureAsset) is read
//!   with its model ([`load_asset`]), and each chunk's hull is given to
//!   physics as a convex mesh under [`hull_handle`]: the step that first
//!   collides with a chunk builds nothing.
//! - **The state** is world state: per og.Destructible its bonds' health
//!   (og_physics::destruction::Instance), its chunks' entities and bodies, and the
//!   damage queued by og.destruction.damage, .explode and .fracture, in the
//!   `og.destruction` state block (snapshots, restores, replays and the
//!   world hash carry it).
//! - **The system** runs after the game's systems and before og.physics:
//!   impacts (og.CollisionStarted) and the queued calls damage the bonds,
//!   the stress solver breaks what can no longer hold itself up, and every
//!   island of chunks whose membership changed is given its body: an
//!   anchored island is static colliders, a loose one a dynamic
//!   og.RigidBody on its first chunk with the rest as its compound children
//!   (og.Parent). Fixed f32 arithmetic in fixed orders only.
//! - **Drawing** ([`drawn`]): a fractured prop's model is not drawn; each
//!   of its chunks draws its node of the same model at the chunk's
//!   transform, with the model's look.
//!
//! The records are lazy (`crate::engine::lazy`): a world that never names them has
//! neither the system nor the block, so its saves and hash are unchanged.

use crate::engine::{Engine, Mount, engine};
use crate::host::codec::{Layout, RecordKind};
use crate::host::wire::{
    self, DamageDesc, DestructibleInfo, DestructibleMaterial, DestructionCapabilities,
    ExplosionDesc, FromJson, record,
};
use glam::{Quat, Vec3};
use og_core::hash::mix64 as scramble;
use og_core::{Error, ErrorKind};
use og_ecs::{Entity, QueryDesc, SchemaId, StateBlock, SystemContext, SystemDesc, World};
use og_physics::destruction::prop::blast::{Explosion, blast_push, push_blasted_pieces};
use og_physics::destruction::prop::breaking::{
    Breaking, Hits, REST_EVERY, REST_SPEED, Settings, WEDGED_AFTER, break_masonry, crumble,
    islands_of, place_chunk, shed_crumbs, wear,
};
use og_physics::destruction::prop::budget::loose_islands;
use og_physics::destruction::prop::debris::{Crumb, Debris, spread};
use og_physics::destruction::prop::materials::MATERIALS;
use og_physics::destruction::prop::pose::{
    Pose, frame_of_pose, place_on, pose_of_frame, relative, uniform,
};
pub(crate) use og_physics::destruction::prop::state::hull_handle;
use og_physics::destruction::prop::state::{
    ANCHORED, Assets, CAUSE_DAMAGE, CAUSE_EXPLOSION, CAUSE_IMPACT, CAUSE_SHATTER, CAUSE_STRESS, Op,
    Prop, State,
};
use og_physics::destruction::prop::support::supports_of;
use og_physics::destruction::{Damage, FractureAsset, Instance, Material, stress::StressLimits};
use serde_json::{Value, json};
use std::any::Any;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};

mod api;
mod asset;
mod damage;
mod draw;
mod islands;
mod push;
mod spawn;
mod tick;

pub use api::*;
pub(crate) use asset::*;
use damage::*;
pub(crate) use draw::*;
use islands::*;
use push::*;
use spawn::*;
use tick::*;

pub(crate) const SYSTEM: &str = "og.destruction";
pub(crate) const DESTRUCTIBLE: &str = "og.Destructible";
pub(crate) const CHUNK: &str = "og.Chunk";
pub(crate) const FRACTURED: &str = "og.Fractured";
pub(crate) const RESOURCE: &str = "og.Destruction";
/// Registered together (crate::engine::lazy's TOGETHER): naming one names all.
pub(crate) const RECORDS: [&str; 4] = [DESTRUCTIBLE, CHUNK, FRACTURED, RESOURCE];

/// The most breakFx and hitFx bursts spawned in one tick.
const MAX_BURSTS: usize = 8;

// ── the state ────────────────────────────────────────────────────────────────

/// og.destruction on one engine: the state its world's block saves (shared
/// with its system and its calls), its models' fracture assets, and the
/// physics its system queries (what a loose island rests on), set with the
/// world's.
#[derive(Clone, Default)]
pub(crate) struct ModuleState {
    state: Arc<Mutex<State>>,
    assets: Arc<Mutex<Assets>>,
    physics: Arc<Mutex<Option<og_physics::PhysicsSystem>>>,
}

/// The physics an engine's destruction queries (made with its world).
pub(crate) fn set_physics(m: &ModuleState, physics: &og_physics::PhysicsSystem) {
    *lock(&m.physics) = Some(physics.clone());
}

/// The physics of an engine (og.destruction2d registers its meshes there).
pub(crate) fn physics(m: &ModuleState) -> Option<og_physics::PhysicsSystem> {
    lock(&m.physics).clone()
}

/// Engine `id`'s og.destruction.
fn module(id: u32) -> Result<ModuleState, String> {
    Ok(crate::engine::engine(id)?.modules.destruction.clone())
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn invalid(m: impl Into<String>) -> Error {
    Error::new(ErrorKind::Invalid, m.into())
}

// ── the system ───────────────────────────────────────────────────────────────

/// The records the system reads and writes.
struct Ls {
    destructible: (SchemaId, Layout),
    chunk: (SchemaId, Layout),
    fractured: (SchemaId, Layout),
    resource: (SchemaId, Layout),
    model: (SchemaId, Layout),
    transform: (SchemaId, Layout),
    parent: (SchemaId, Layout),
    rigid: (SchemaId, Layout),
    velocity: (SchemaId, Layout),
    collider: (SchemaId, Layout),
    particles: (SchemaId, Layout),
    collision: (SchemaId, Layout),
    physics: (SchemaId, Layout),
}

const READS: [&str; 4] = [
    "og.Model",
    "og.CollisionStarted",
    "og.Physics",
    "og.MatrixWorld",
];
const WRITES: [&str; 6] = [
    "og.Transform",
    "og.Parent",
    "og.RigidBody",
    "og.Velocity",
    "og.Collider",
    "og.ParticleSystem",
];

/// Registers the `og.destruction` system and its state block once every
/// one of [`RECORDS`] is (crate::engine::lazy registers them one after another).
pub(crate) fn enable(world: &mut World, mount: Mount<'_>) -> og_core::Result<()> {
    if world.systems().contains(&SYSTEM) || RECORDS.iter().any(|n| world.id_by_name(n).is_err()) {
        return Ok(());
    }
    let layout = |n: &str, kind: RecordKind| -> og_core::Result<(SchemaId, Layout)> {
        let id = world.id_by_name(n).map_err(|_| {
            invalid(format!(
                "og.Destructible needs {n}: destruction is the 3D profile's, with og.physics"
            ))
        })?;
        // The IDL's layout, with its defaults (a schema read back has none).
        let layout = og_api::host::RECORDS
            .iter()
            .find(|d| d.schema == n)
            .and_then(|d| Layout::from_decl(d).ok());
        let layout = match layout {
            Some(l) => l,
            None => Layout::from_schema(world.schema(id)?, kind),
        };
        Ok((id, layout))
    };
    use RecordKind::{Component as C, Event as E, Resource as R};
    let ls = Ls {
        destructible: layout(DESTRUCTIBLE, C)?,
        chunk: layout(CHUNK, C)?,
        fractured: layout(FRACTURED, E)?,
        resource: layout(RESOURCE, R)?,
        model: layout("og.Model", C)?,
        transform: layout("og.Transform", C)?,
        parent: layout("og.Parent", C)?,
        rigid: layout("og.RigidBody", C)?,
        velocity: layout("og.Velocity", C)?,
        collider: layout("og.Collider", C)?,
        particles: layout("og.ParticleSystem", C)?,
        collision: layout("og.CollisionStarted", E)?,
        physics: layout("og.Physics", R)?,
    };
    let m = mount.modules.destruction.clone();
    *lock(&m.state) = State::default();
    let sh = m.state.clone();
    let s = |v: &[&str]| v.iter().map(|n| n.to_string()).collect::<Vec<_>>();
    world.add_system(
        SystemDesc {
            name: SYSTEM.into(),
            reads: s(&READS),
            writes: [s(&RECORDS), s(&WRITES)].concat(),
            queries: vec![QueryDesc {
                write: vec![DESTRUCTIBLE.to_string()],
                ..QueryDesc::default()
            }],
            ..Default::default()
        },
        move |ctx| run(ctx, &m, &ls),
    )?;
    if world.systems().contains(&"og.physics") {
        world.move_system_before(SYSTEM, "og.physics")?;
    }
    world.add_state_block(SYSTEM, Box::new(Block(sh)))
}

/// Layout::encode with the tick's error type.
trait Enc {
    fn enc(&self, base: Option<&[u8]>, value: Option<&Value>) -> og_core::Result<Vec<u8>>;
}

impl Enc for Layout {
    fn enc(&self, base: Option<&[u8]>, value: Option<&Value>) -> og_core::Result<Vec<u8>> {
        self.encode(base, value).map_err(invalid)
    }
}

fn jv(p: Vec3) -> Value {
    json!({ "x": p.x, "y": p.y, "z": p.z })
}

fn jq(q: Quat) -> Value {
    json!({ "x": q.x, "y": q.y, "z": q.z, "w": q.w })
}

/// A row read through its IDL type, with the tick's error type.
fn row<T: FromJson>(l: &Layout, bytes: &[u8]) -> og_core::Result<T> {
    l.read(bytes).map_err(invalid)
}

/// An og.Destructible's row as the tick's settings.
fn settings_of(v: &record::Destructible) -> Settings {
    Settings {
        mat: Material {
            toughness: v.toughness.max(1e-3),
            limits: StressLimits::from_strength(v.strength, v.friction, v.tension.clamp(0.0, 1.0)),
            density: v.density.max(1.0),
        },
        threshold: v.impact_threshold.max(0.0),
        radius: v.impact_radius.max(0.01),
        anchored: v.anchored,
        break_fx: v.break_fx,
        hit_fx: v.hit_fx,
        debris: v.debris,
    }
}

/// The world pose of `e`: its og.Transform after its og.Parent's (a
/// chunk's body), as og_core::math::world_pose composes it.
fn world_pose(ctx: &SystemContext<'_>, l: &Ls, e: Entity) -> Option<Pose> {
    let pose = |e: Entity| -> Option<Pose> {
        let t: record::Transform =
            row(&l.transform.1, &ctx.get_bytes(e, l.transform.0).ok()?).ok()?;
        Some((t.position, t.quaternion.normalize(), t.scale))
    };
    let parent = |e: Entity| -> Option<Entity> {
        let p: record::Parent = row(&l.parent.1, &ctx.get_bytes(e, l.parent.0).ok()?).ok()?;
        let b = p.entity;
        (b != 0).then(|| Entity::from_bits(b))
    };
    let local = pose(e)?;
    Some(og_core::math::world_pose(e, local, parent, pose))
}

// ── the state block ──────────────────────────────────────────────────────────

struct Block(Arc<Mutex<State>>);

impl StateBlock for Block {
    fn save(&self, out: &mut Vec<u8>) {
        lock(&self.0).save(out);
    }
    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn Any + Send>> {
        Ok(Box::new(State::decode(bytes)?))
    }
    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        *lock(&self.0) = decoded
            .and_then(|d| d.downcast::<State>().ok())
            .map(|b| *b)
            .unwrap_or_default();
    }
    fn shift(&mut self, s: &og_ecs::Shift) {
        lock(&self.0).shift(s.d);
    }
}
