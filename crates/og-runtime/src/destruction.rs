//! `og.destruction` (docs/design.md §2.7, "Destruction"): props that break
//! into the chunks the cook cut them into (og-cook `cook/fracture.rs`).
//!
//! - **The asset** (`<model>#fracture`, og_destruct::FractureAsset) is read
//!   with its model ([`load_asset`]), and each chunk's hull is given to
//!   physics as a convex mesh under [`hull_handle`]: the step that first
//!   collides with a chunk builds nothing.
//! - **The state** is world state: per og.Destructible its bonds' health
//!   (og_destruct::Instance), its chunks' entities and bodies, and the
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
//! The records are lazy (`super::lazy`): a world that never names them has
//! neither the system nor the block, so its saves and hash are unchanged.

use super::codec::{Layout, RecordKind};
use super::engine::{Engine, Mount, engine};
use super::wire::{
    self, DamageDesc, DestructibleInfo, DestructibleMaterial, DestructionCapabilities,
    ExplosionDesc, FromJson, record,
};
use glam::{Quat, Vec3};
use og_core::{Error, ErrorKind};
use og_destruct::{Damage, FractureAsset, Instance, Material, Support, stress::StressLimits};
use og_ecs::{Entity, QueryDesc, SchemaId, StateBlock, SystemContext, SystemDesc, World};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::any::Any;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};

pub(crate) const SYSTEM: &str = "og.destruction";
pub(crate) const DESTRUCTIBLE: &str = "og.Destructible";
pub(crate) const CHUNK: &str = "og.Chunk";
pub(crate) const FRACTURED: &str = "og.Fractured";
pub(crate) const RESOURCE: &str = "og.Destruction";
/// Registered together (super::lazy's TOGETHER): naming one names all.
pub(crate) const RECORDS: [&str; 4] = [DESTRUCTIBLE, CHUNK, FRACTURED, RESOURCE];

const CAUSE_IMPACT: u32 = 0;
const CAUSE_DAMAGE: u32 = 1;
const CAUSE_EXPLOSION: u32 = 2;
const CAUSE_STRESS: u32 = 3;
const CAUSE_SHATTER: u32 = 4;
const CAUSE_FRACTURE: u32 = 5;
/// og.Chunk.looseTick of an anchored chunk.
const ANCHORED: u32 = u32::MAX;
/// Physics mesh handles of the chunks' hulls: the tag, the model's asset
/// index (24 bits) and the chunk (16 bits), below 2^53.
const HULL_TAG: u64 = 0xd5 << 40;
/// The most breakFx and hitFx bursts spawned in one tick.
const MAX_BURSTS: usize = 8;

/// og.Transform's position, rotation and scale.
type Pose = (Vec3, Quat, Vec3);

/// The physics mesh of chunk `chunk` of the model whose asset handle is
/// `asset`.
pub(crate) fn hull_handle(asset: u64, chunk: u32) -> u64 {
    HULL_TAG | ((asset & 0xff_ffff) << 16) | u64::from(chunk & 0xffff)
}

// ── the state ────────────────────────────────────────────────────────────────

/// What `og.destruction` keeps: world state.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct State {
    /// By the og.Destructible's entity bits.
    props: BTreeMap<u64, Prop>,
    /// Calls queued for the next run, in the order made.
    ops: Vec<Op>,
    /// breakFx and hitFx bursts: (entity bits, the tick it is despawned at).
    fx: Vec<(u64, u64)>,
    /// Fine pieces broken off as visuals only (og.Destructible.debris):
    /// the renderer's debris layer moves and draws them from here; they
    /// are never bodies.
    debris: Vec<Debris>,
    /// Joints broken with a debris emitter set: the grit each sheds
    /// (clay and mortar chips, the renderer's debris layer's) flies from
    /// here.
    crumbs: Vec<Crumb>,
}

/// A broken joint's grit as it fell out: the renderer throws its chips
/// from here.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Crumb {
    /// Its og.Destructible's entity bits and its bond.
    prop: u64,
    bond: u32,
    /// The tick it broke.
    tick: u64,
    /// The joint's centre and its normal in the world then.
    point: [f32; 3],
    normal: [f32; 3],
    /// What the blow or the blast gave the pieces beside it.
    velocity: [f32; 3],
}

/// The most joints a prop sheds grit from in one tick (spread over all
/// that broke), and the most at once.
const CRUMBS_A_TICK: usize = 48;
const MAX_CRUMBS: usize = 384;

/// A fine piece as it broke off: the renderer simulates it from here.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Debris {
    /// Its og.Destructible's entity bits and its chunk.
    prop: u64,
    chunk: u32,
    /// The tick it broke off.
    tick: u64,
    /// Its centre of mass, rotation (x y z w) and scale then.
    pose: [f32; 10],
    velocity: [f32; 3],
    angular: [f32; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Prop {
    /// Its og.Model's asset handle.
    asset: u64,
    inst: Instance,
    /// The model's world pose when its chunks were placed: position,
    /// rotation (x y z w) and scale.
    frame: [f32; 10],
    /// Per chunk: its entity's bits (0: not spawned, or gone).
    entities: Vec<u64>,
    /// Per chunk: the entity whose og.RigidBody moves it (0: static).
    body: Vec<u64>,
    /// Per chunk: the tick it broke loose (u64::MAX while anchored).
    loose: Vec<u64>,
    /// Its og.Collider when its chunks were placed: theirs copy its
    /// friction, restitution, layers and mask.
    collider: Option<Vec<u8>>,
    /// Its chunks are entities.
    placed: bool,
    /// Its anchors hold it (it stood when placed, and `anchored`).
    anchors: bool,
    /// Its first bond broke: the model is hidden, the chunks drawn.
    fractured: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum Op {
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
type Assets = BTreeMap<u64, Arc<FractureAsset>>;

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

/// How often (in ticks) a loose island at rest is solved on what holds it.
const REST_EVERY: u64 = 6;
/// The most blocks of no-tension masonry that fly loose as one piece: one
/// (mortar holds nothing pulled apart, so what breaks loose is its blocks,
/// each a cluster of bricks, never a run of them bonded into a beam).
const RUN: usize = 1;
/// How unevenly a blast pushes the pieces near it: each takes its push
/// times 1 ± SHARE / 2.
const SHARE: f32 = 0.6;
/// A blast's turn on a piece: this much of its push's speed over its size
/// (rad/s per m/s per m), enough to open its joints as it goes, not to
/// spin it into what stands.
const SPIN: f32 = 0.1;
/// The fastest turn (rad/s) a blast gives a piece.
const TURN_MAX: f32 = 12.0;

/// A fixed scramble of `x` (splitmix64's finaliser, public domain): a
/// piece's own share of a blast, the same on every run and host.
fn scramble(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

/// A blast's push on a piece of mass `m` at `at` (its velocity and turn),
/// `None` where it does not reach. A blast's front is not a smooth field
/// near the charge: each piece there takes its own share of the push (up
/// to SHARE either way) and a small turn from the uneven pressure on its
/// face, both fading with the push to none at the pier feet it barely
/// reaches, so the blocks of a wall go their own ways rather than in step
/// and the piers keep their footing. Fixed per piece by `first`, its first
/// chunk, so the same chunk of a model takes the same share in any scene.
fn blast_push(
    falloff: &Damage,
    impulse: f32,
    at: Vec3,
    m: f32,
    density: f32,
    first: u64,
) -> Option<(Vec3, Vec3)> {
    let j = falloff.at(at) * face(m / density);
    if m <= 0.0 || j <= 0.0 {
        return None;
    }
    let r = scramble(first);
    let u = |s: u32| ((r >> s) & 0x3ff) as f32 / 1023.0;
    let axis = Vec3::new(u(0) - 0.5, u(10) - 0.5, u(20) - 0.5).normalize_or(Vec3::X);
    let dir = (at - falloff.point).normalize_or(Vec3::Y);
    let near = falloff.at(at) / impulse;
    let dv = dir * (j / m) * (1.0 + near * SHARE * (u(40) - 0.5));
    let size = og_core::math::pow(m / density, 1.0 / 3.0).max(0.02);
    let dw = axis * (near * SPIN * dv.length() / size).min(TURN_MAX);
    Some((dv, dw))
}

/// Below this speed (m/s) a loose island is at rest.
const REST_SPEED: f32 = 0.5;

/// Ticks a loose island stays where it broke free before, at rest with
/// nothing under it, it counts as wedged (one just let go is at rest
/// too, for the tick before it falls).
const WEDGED_AFTER: u64 = 30;

/// Wedged loose island `chunks` gives way: its bonds break, and each chunk
/// with pieces shatters into them (`shatter` gathers those).
fn crumble(a: &FractureAsset, inst: &mut Instance, chunks: &[u32], shatter: &mut Vec<u32>) {
    for (k, b) in a.bonds.iter().enumerate() {
        if chunks.contains(&b.a) && chunks.contains(&b.b) {
            inst.bonds[k] = 0.0;
        }
    }
    for &c in chunks {
        let i = c as usize;
        if a.chunks[i].children.1 > 0 && !inst.shattered[i] {
            inst.shattered[i] = true;
            shatter.push(c);
        }
    }
    inst.dirty = true;
}

/// Each loose body of prop `p` (a support chunk's body with the chunks it
/// carries, or a shattered piece on its own): its mass and place. A body
/// made this run has no transform yet: a piece's place is then `fallback`.
#[allow(clippy::too_many_arguments)]
fn loose_bodies(
    ctx: &SystemContext<'_>,
    l: &Ls,
    p: &Prop,
    a: &FractureAsset,
    scale: f32,
    density: f32,
    poses: &[Option<Pose>],
    made: &BTreeMap<u64, (Vec3, Vec3)>,
    fallback: Vec3,
) -> BTreeMap<u64, (f32, Vec3)> {
    // An island's place is its centre of mass (its chunks' centres by
    // their masses), not its root chunk's: a long island whose root lies
    // by a blast is pushed as a whole far from it is.
    let mut bodies: BTreeMap<u64, (f32, Vec3)> = BTreeMap::new();
    let mut placed: BTreeMap<u64, f32> = BTreeMap::new();
    for (c, &b) in p.body.iter().enumerate() {
        if b == 0 || p.entities[c] == 0 {
            continue;
        }
        let m = a.chunks[c].volume * scale * scale * scale * density;
        let e = bodies.entry(b).or_insert((0.0, Vec3::ZERO));
        e.0 += m;
        if let Some(ps) = poses[c] {
            e.1 += ps.0 * m;
            *placed.entry(b).or_default() += m;
        }
    }
    for (b, e) in bodies.iter_mut() {
        if let Some(&m) = placed.get(b).filter(|&&m| m > 0.0) {
            e.1 /= m;
        }
    }
    for c in a.support as usize..a.chunks.len() {
        let e = p.entities[c];
        if e != 0 && !bodies.contains_key(&e) {
            let m = a.chunks[c].volume * scale * scale * scale * density;
            let at = match made.contains_key(&e) {
                true => None,
                false => world_pose(ctx, l, Entity::from_bits(e)).map(|x| x.0),
            };
            bodies.insert(e, (m, at.unwrap_or(fallback)));
        }
    }
    bodies
}

/// The supports of loose island `chunks` (its body `body`, at rest): for
/// each chunk, what a ray down from its centre meets within its depth
/// below it and 5 cm more, other than the island itself.
fn supports_of(
    q: &og_physics::PhysicsQuery,
    a: &FractureAsset,
    pose: &[Option<Pose>],
    chunks: &[u32],
    body: u64,
    gravity: Vec3,
) -> Vec<Support> {
    let down = gravity.normalize_or_zero();
    if down == Vec3::ZERO {
        return Vec::new();
    }
    let mut out = Vec::new();
    for &c in chunks {
        let Some(cp) = pose[c as usize] else {
            continue;
        };
        let ch = &a.chunks[c as usize];
        let depth = ch
            .hull
            .iter()
            .map(|h| (cp.1 * (cp.2 * *h)).dot(down))
            .fold(0.0, f32::max);
        let hits = q
            .raycast_all(&og_physics::RaycastAllDesc {
                ray: og_physics::RaycastDesc {
                    origin: cp.0,
                    direction: down,
                    max_distance: depth + 0.05,
                    filter: og_physics::QueryFilter::default(),
                },
                max_hits: 16,
            })
            .unwrap_or_default();
        let Some(hit) = hits.iter().find(|h| h.body.to_bits() != body) else {
            continue;
        };
        let s = cp.2.max_element().max(1e-6);
        let r = og_destruct::cube_root(ch.volume);
        out.push(Support {
            chunk: c,
            point: ch.centroid + cp.1.inverse() * (hit.point - cp.0) / s,
            area: r * r,
        });
    }
    out
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

fn pose_of_frame(f: &[f32; 10]) -> Pose {
    (
        Vec3::new(f[0], f[1], f[2]),
        Quat::from_xyzw(f[3], f[4], f[5], f[6]),
        Vec3::new(f[7], f[8], f[9]),
    )
}

fn frame_of_pose((p, q, s): Pose) -> [f32; 10] {
    [p.x, p.y, p.z, q.x, q.y, q.z, q.w, s.x, s.y, s.z]
}

/// A uniform scale's factor (the mean of the axes' magnitudes).
fn uniform(s: Vec3) -> f32 {
    (s.x.abs() + s.y.abs() + s.z.abs()) / 3.0
}

// ── the asset ────────────────────────────────────────────────────────────────

/// Reads model `id`'s fracture asset (`<id>#fracture`), when the cook wrote
/// one, for the model loaded as asset `index`, and gives physics its
/// chunks' hulls. A model without one forgets any it had (a hot reload).
pub(crate) fn load_asset(eng: &Engine, id: &str, index: usize) -> Result<(), String> {
    let handle = super::assets::handle(index);
    let blob = format!("{id}#fracture");
    let bytes = {
        let assets = eng.assets.borrow();
        let has = assets
            .bundle_for(id)
            .is_ok_and(|(b, _)| b.output(&blob).is_ok());
        if !has {
            lock(&eng.modules.destruction.assets).remove(&handle);
            return Ok(());
        }
        let (bytes, bundle) = assets.blob(&blob)?;
        bundle
            .fracture(&blob, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    let a = FractureAsset::from_bytes(&bytes).map_err(|e| format!("{blob}: {e}"))?;
    if a.chunks.len() > 0x1_0000 || index > 0xff_ffff {
        return Err(format!(
            "{blob}: {} chunks, past the 65536 a model holds",
            a.chunks.len()
        ));
    }
    if eng.state.borrow().physics.is_some() {
        for (i, c) in a.chunks.iter().enumerate() {
            let mesh = og_physics::MeshDesc {
                positions: c.hull.clone(),
                indices: Vec::new(),
            };
            super::modules::set_mesh(eng, hull_handle(handle, i as u32), mesh)?;
        }
    }
    lock(&eng.modules.destruction.assets).insert(handle, Arc::new(a));
    Ok(())
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
/// one of [`RECORDS`] is (super::lazy registers them one after another).
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

/// An og.Destructible's settings, as the tick reads them.
struct Settings {
    mat: Material,
    threshold: f32,
    radius: f32,
    anchored: bool,
    break_fx: u64,
    hit_fx: u64,
    debris: u64,
}

impl Settings {
    fn of(v: &record::Destructible) -> Settings {
        Settings {
            mat: Material {
                toughness: v.toughness.max(1e-3),
                limits: StressLimits::from_strength(
                    v.strength,
                    v.friction,
                    v.tension.clamp(0.0, 1.0),
                ),
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
}

/// What one run does to a prop besides its bonds.
#[derive(Default)]
struct Hits {
    damage: Vec<(Damage, u32)>,
    fracture: bool,
}

/// A chunk's place in its island's body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Role {
    Static,
    Root,
    /// A compound child of the chunk named.
    Child(u32),
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

/// `p` of model space, on a chunk whose model-space centre is `c`, placed
/// by the chunk's world pose.
fn place_on(pose: &Pose, c: Vec3, p: Vec3) -> Vec3 {
    pose.0 + pose.1 * (pose.2 * (p - c))
}

/// `child`'s pose relative to `root`'s, both in the world.
fn relative(root: &Pose, child: &Pose) -> Pose {
    let inv = root.1.inverse();
    (
        inv * (child.0 - root.0) / root.2,
        (inv * child.1).normalize(),
        child.2 / root.2,
    )
}

/// The og.destruction system, one tick, as its stages in order: bursts
/// expire; the props are surveyed (new ones placed, gone ones forgotten);
/// chunks the game despawned leave their graph; the chunks' poses are
/// read; the hits are gathered (impacts, then the calls); each prop
/// breaks; the blasts and the blows push what broke loose; the bodies made
/// take their velocity; the budgets despawn the oldest; and what the game
/// reads is written (the destructibles' fields, og.Fractured, the bursts).
fn run(ctx: &mut SystemContext<'_>, m: &ModuleState, l: &Ls) -> og_core::Result<()> {
    let t = Clock::read(ctx, l)?;
    let assets = lock(&m.assets).clone();
    let mut guard = lock(&m.state);
    let st = &mut *guard;

    expire_bursts(ctx, st, t.tick);
    let settings = survey(ctx, l, st, &assets)?;
    drop_lost_chunks(ctx, st)?;
    let poses = chunk_poses(ctx, l, st, &assets);
    let Gathered {
        mut hits,
        bursts,
        strikes,
        explosions,
    } = gather_hits(ctx, l, st, &assets, &settings)?;

    let mut out = Broken {
        bursts,
        ..Broken::default()
    };
    let mut stress_budget = match t.res.stress_budget {
        0 => u64::MAX,
        n => u64::from(n),
    };
    let keys: Vec<u64> = st.props.keys().copied().collect();
    for k in keys {
        let p = st.props.get_mut(&k).expect("listed");
        let a = assets[&p.asset].clone();
        let at = Breaking {
            key: k,
            a: &a,
            s: &settings[&k],
            pose: &poses[&k],
        };
        let h = hits.remove(&k).unwrap_or_default();
        break_prop(ctx, m, l, &t, at, p, h, &mut stress_budget, &mut out)?;
    }

    push_blasted_bodies(
        ctx,
        l,
        st,
        &assets,
        &settings,
        &poses,
        &explosions,
        &mut out,
    )?;
    push_blasted_pieces(&settings, &explosions, &mut out);
    push_struck(ctx, l, st, &assets, &settings, &poses, &strikes, &mut out);
    commit_fresh(ctx, l, st, &mut out)?;
    enforce_budgets(ctx, st, &assets, &t)?;
    publish(ctx, l, st, &t, out.events, out.bursts)
}

/// What the tick reads once: its clock, og.Destruction and the gravity.
struct Clock {
    tick: u64,
    dt: f32,
    /// og.Destruction, read.
    res: record::Destruction,
    debris_volume: f32,
    gravity: Vec3,
}

impl Clock {
    fn read(ctx: &SystemContext<'_>, l: &Ls) -> og_core::Result<Clock> {
        let time = ctx.time();
        let res: record::Destruction = row(&l.resource.1, &ctx.resource_bytes(l.resource.0)?)?;
        let physics: record::Physics = row(&l.physics.1, &ctx.resource_bytes(l.physics.0)?)?;
        Ok(Clock {
            tick: time.tick,
            dt: time.dt.max(1e-6),
            debris_volume: res.debris_volume,
            gravity: physics.gravity,
            res,
        })
    }

    /// The ticks `s` seconds take, rounded up.
    fn ticks_of(&self, s: f32) -> u64 {
        (s.max(0.0) / self.dt).ceil() as u64
    }
}

fn alive(ctx: &SystemContext<'_>, b: u64) -> bool {
    b != 0 && ctx.is_alive(Entity::from_bits(b))
}

/// Bursts past their lifetime.
fn expire_bursts(ctx: &mut SystemContext<'_>, st: &mut State, tick: u64) {
    let mut fx = std::mem::take(&mut st.fx);
    fx.retain(|&(e, at)| {
        if tick < at {
            return true;
        }
        if alive(ctx, e) {
            let _ = ctx.despawn(Entity::from_bits(e));
        }
        false
    });
    st.fx = fx;
}

/// The destructibles and their settings; props whose entity lost its
/// og.Destructible (or died) are forgotten, their static chunks with them,
/// and a new one is placed (a standing prop's chunks are its colliders
/// from the start).
fn survey(
    ctx: &mut SystemContext<'_>,
    l: &Ls,
    st: &mut State,
    assets: &Assets,
) -> og_core::Result<BTreeMap<u64, Settings>> {
    let mut rows = Vec::new();
    for a in ctx.query(0)? {
        rows.extend_from_slice(a.entities());
    }
    let mut settings: BTreeMap<u64, Settings> = BTreeMap::new();
    let mut current: BTreeMap<u64, (u64, Option<Pose>, bool)> = BTreeMap::new();
    for &e in &rows {
        let d: record::Destructible = row(&l.destructible.1, &ctx.get_bytes(e, l.destructible.0)?)?;
        let Ok(mb) = ctx.get_bytes(e, l.model.0) else {
            continue;
        };
        let asset = row::<record::Model>(&l.model.1, &mb)?.asset;
        // A prop that moves (a dynamic or kinematic body) breaks free whole.
        // og.RigidBody.kind is an enum, which the hand read this replaced
        // always took as 0 (dynamic): any body counts as moving, a fixed
        // one too (kept as it was).
        let moving = ctx.get_bytes(e, l.rigid.0).is_ok();
        settings.insert(e.to_bits(), Settings::of(&d));
        current.insert(e.to_bits(), (asset, world_pose(ctx, l, e), moving));
    }
    let gone: Vec<u64> = st
        .props
        .iter()
        .filter(|(k, p)| {
            current
                .get(k)
                .is_none_or(|c| c.0 != p.asset || !assets.contains_key(&c.0))
        })
        .map(|(k, _)| *k)
        .collect();
    for k in gone {
        let p = st.props.remove(&k).expect("listed");
        for (i, &e) in p.entities.iter().enumerate() {
            if p.body[i] == 0 && alive(ctx, e) {
                ctx.despawn(Entity::from_bits(e))?;
            }
        }
    }
    for (&k, &(asset, pose, moving)) in &current {
        let Some(a) = assets.get(&asset) else {
            continue;
        };
        let Some(pose) = pose else {
            continue;
        };
        let s = &settings[&k];
        let p = st.props.entry(k).or_insert_with(|| Prop {
            asset,
            inst: Instance::new(a),
            frame: frame_of_pose(pose),
            entities: vec![0; a.chunks.len()],
            body: vec![0; a.chunks.len()],
            loose: vec![u64::MAX; a.chunks.len()],
            collider: None,
            placed: false,
            anchors: false,
            fractured: false,
        });
        if !p.placed {
            p.frame = frame_of_pose(pose);
            p.anchors = !moving && s.anchored;
            // A standing prop's chunks are its colliders from the start.
            if !moving {
                place(ctx, l, k, p, a)?;
            }
        }
    }
    Ok(settings)
}

/// Chunks despawned by the game or a budget: gone from the graph; a body
/// takes its compound children with it.
fn drop_lost_chunks(ctx: &mut SystemContext<'_>, st: &mut State) -> og_core::Result<()> {
    for p in st.props.values_mut() {
        let dead: Vec<u64> = p
            .entities
            .iter()
            .copied()
            .filter(|&e| e != 0 && !alive(ctx, e))
            .collect();
        if dead.is_empty() {
            continue;
        }
        for (i, e) in p.entities.iter_mut().enumerate() {
            let lost = dead.contains(e) || (p.body[i] != 0 && dead.contains(&p.body[i]));
            if *e != 0 && lost {
                if alive(ctx, *e) {
                    ctx.despawn(Entity::from_bits(*e))?;
                }
                *e = 0;
                p.body[i] = 0;
                if let Some(sh) = p.inst.shattered.get_mut(i) {
                    *sh = true;
                }
                p.inst.dirty = true;
            }
        }
    }
    Ok(())
}

/// Each prop's chunks where they stand now, by prop.
type Poses = BTreeMap<u64, Vec<Option<Pose>>>;

fn chunk_poses(ctx: &SystemContext<'_>, l: &Ls, st: &State, assets: &Assets) -> Poses {
    let mut poses: Poses = BTreeMap::new();
    for (&k, p) in &st.props {
        let a = &assets[&p.asset];
        let frame = pose_of_frame(&p.frame);
        let list = (0..a.chunks.len())
            .map(|i| {
                let e = p.entities[i];
                if e != 0 {
                    return world_pose(ctx, l, Entity::from_bits(e));
                }
                // Not spawned: where the model places it.
                let c = a.chunks[i].centroid;
                Some((place_on(&frame, Vec3::ZERO, c), frame.1, frame.2))
            })
            .collect();
        poses.insert(k, list);
    }
    poses
}

/// A hit that breaks pieces off drives them on: its prop, its point, the
/// striker's direction, impactRadius, the impulse past the threshold and
/// its closing speed.
type Strike = (u64, Vec3, Vec3, f32, f32, f32);

/// An explosion: its point, inner radius, radius and impulse.
type Explosion = (Vec3, f32, f32, f32);

/// The tick's hits: what each prop takes, the bursts they make, the
/// strikes and the explosions.
struct Gathered {
    hits: BTreeMap<u64, Hits>,
    bursts: Vec<(u64, Vec3)>,
    strikes: Vec<Strike>,
    explosions: Vec<Explosion>,
}

/// The hits: impacts, then the calls in the order made.
fn gather_hits(
    ctx: &SystemContext<'_>,
    l: &Ls,
    st: &mut State,
    assets: &Assets,
    settings: &BTreeMap<u64, Settings>,
) -> og_core::Result<Gathered> {
    let mut hits: BTreeMap<u64, Hits> = BTreeMap::new();
    let mut bursts: Vec<(u64, Vec3)> = Vec::new();
    let mut owner: BTreeMap<u64, u64> = BTreeMap::new();
    for (&k, p) in &st.props {
        if !p.placed {
            owner.insert(k, k);
        }
        for &e in p.entities.iter().filter(|&&e| e != 0) {
            owner.insert(e, k);
        }
    }
    let mut strikes: Vec<Strike> = Vec::new();
    for ev in ctx.events_bytes(l.collision.0)? {
        let v: record::CollisionStarted = row(&l.collision.1, &ev)?;
        let point = v.point;
        // The solver's impulse is that of the step that made the contact,
        // which for a fast body is a speculative one (rapier opens it
        // before they touch, and that step barely pushes): the hit is at
        // least its closing speed times the pair's reduced mass.
        let (ea, eb) = (v.a, v.b);
        let closing = v.relative_velocity.dot(v.normal).max(0.0);
        let (reduced_a, reduced_b) = (
            mass_of(ctx, l, &st.props, assets, settings, ea),
            mass_of(ctx, l, &st.props, assets, settings, eb),
        );
        let reduced = match (reduced_a, reduced_b) {
            (Mass::Finite(a), Mass::Finite(b)) => a * b / (a + b),
            (Mass::Finite(m), Mass::Infinite) | (Mass::Infinite, Mass::Finite(m)) => m,
            _ => 0.0,
        };
        let impulse = v.impulse.max(closing * reduced);
        let mut seen = None;
        for (side, of) in [("a", ea), ("b", eb)] {
            let Some(&k) = owner.get(&of) else {
                continue;
            };
            if seen == Some(k) {
                continue;
            }
            seen = Some(k);
            let s = &settings[&k];
            let amount = impulse - s.threshold;
            if amount > 0.0 && point.is_finite() {
                // The striker's motion through the prop: a's velocity
                // relative to b, turned round when the prop is a.
                // Only a prop that stood still gives the pieces it loses
                // the hit's push: a moving one's answer to the hit is
                // already in its velocity, which its pieces inherit.
                let me = if side == "a" {
                    (ea, reduced_a)
                } else {
                    (eb, reduced_b)
                };
                if matches!(me.1, Mass::Infinite) {
                    let rel = v.relative_velocity * if side == "b" { 1.0 } else { -1.0 };
                    let along = v.normal * if side == "b" { 1.0 } else { -1.0 };
                    strikes.push((k, point, rel.normalize_or(along), s.radius, amount, closing));
                }
                hits.entry(k).or_default().damage.push((
                    Damage {
                        point,
                        min_radius: s.radius / 3.0,
                        max_radius: s.radius,
                        amount,
                    },
                    CAUSE_IMPACT,
                ));
                if s.hit_fx != 0 {
                    bursts.push((s.hit_fx, point));
                }
            }
        }
    }
    let mut explosions: Vec<Explosion> = Vec::new();
    for op in std::mem::take(&mut st.ops) {
        match op {
            Op::Damage {
                entity,
                point,
                radius,
                inner,
                amount,
                impulse,
            } => {
                if let Some(s) = settings.get(&entity)
                    && st.props.contains_key(&entity)
                {
                    let point = Vec3::from(point);
                    let push = Vec3::from(impulse);
                    if push != Vec3::ZERO {
                        let dir = push.normalize();
                        strikes.push((entity, point, dir, radius, push.length(), f32::INFINITY));
                    }
                    hits.entry(entity).or_default().damage.push((
                        Damage {
                            point,
                            min_radius: inner,
                            max_radius: radius,
                            amount,
                        },
                        CAUSE_DAMAGE,
                    ));
                    if s.hit_fx != 0 {
                        bursts.push((s.hit_fx, point));
                    }
                }
            }
            Op::Explode {
                point,
                radius,
                inner,
                damage,
                impulse,
            } => {
                let point = Vec3::from(point);
                for &k in st.props.keys() {
                    hits.entry(k).or_default().damage.push((
                        Damage {
                            point,
                            min_radius: inner,
                            max_radius: radius,
                            amount: damage,
                        },
                        CAUSE_EXPLOSION,
                    ));
                }
                explosions.push((point, inner, radius, impulse));
            }
            Op::Fracture { entity } => {
                if st.props.contains_key(&entity) {
                    hits.entry(entity).or_default().fracture = true;
                }
            }
        }
    }
    Ok(Gathered {
        hits,
        bursts,
        strikes,
        explosions,
    })
}

/// What the props' breaking made this tick.
#[derive(Default)]
struct Broken {
    /// og.Fractured, encoded, sent at the end.
    events: Vec<Vec<u8>>,
    /// Bodies made this run and their velocity, inserted at the end (after
    /// the explosions' push).
    made: BTreeMap<u64, (Vec3, Vec3)>,
    fresh_bodies: Vec<u64>,
    /// Fine pieces broken off this tick, and their mass.
    debris: Vec<(Debris, f32)>,
    /// Joints broken this tick that shed grit, and the mass of the piece
    /// beside each (which the blast's push is shared by).
    crumbs: Vec<(Crumb, f32)>,
    /// breakFx, hitFx and debris bursts: (emitter, where).
    bursts: Vec<(u64, Vec3)>,
}

/// One prop as it breaks: its key, asset, settings and chunk poses.
#[derive(Clone, Copy)]
struct Breaking<'a> {
    key: u64,
    a: &'a FractureAsset,
    s: &'a Settings,
    pose: &'a [Option<Pose>],
}

/// Model-space point `at` of chunk `c` in the world: on the chunk's pose,
/// else on the prop's frame.
fn place_chunk(at: Breaking<'_>, frame: &Pose, c: u32, x: Vec3) -> Vec3 {
    let ch = &at.a.chunks[c as usize];
    match at.pose[c as usize] {
        Some(ps) => place_on(&ps, ch.centroid, x),
        None => place_on(frame, Vec3::ZERO, x),
    }
}

/// The prop's islands; none anchored unless its anchors hold.
fn islands_of(inst: &Instance, a: &FractureAsset, anchors: bool) -> Vec<og_destruct::Island> {
    let mut isl = inst.islands(a);
    if !anchors {
        isl.iter_mut().for_each(|i| i.anchored = false);
    }
    isl
}

/// One prop's tick: its hits wear its bonds, stress breaks what can no
/// longer hold, broken joints shed grit, and what broke loose becomes
/// bodies (islands) or flies apart (shattered chunks).
#[allow(clippy::too_many_arguments)]
fn break_prop(
    ctx: &mut SystemContext<'_>,
    m: &ModuleState,
    l: &Ls,
    t: &Clock,
    at: Breaking<'_>,
    p: &mut Prop,
    h: Hits,
    stress_budget: &mut u64,
    out: &mut Broken,
) -> og_core::Result<()> {
    let (a, s, k) = (at.a, at.s, at.key);
    let frame = pose_of_frame(&p.frame);
    let scale = uniform(frame.2);
    let before = p.inst.bonds.clone();
    let anchors_before = p.inst.anchors.clone();
    let shattered_before = p.inst.shattered.clone();
    let (mut cause, mut shatter) = wear(at, &frame, p, &h, scale);
    solve_stress(
        ctx,
        m,
        l,
        t,
        at,
        &frame,
        p,
        scale,
        stress_budget,
        &mut cause,
        &mut shatter,
    );
    break_masonry(t, at, &frame, p, &before, &mut cause);
    let broken_now: Vec<bool> = (0..before.len())
        .map(|i| before[i] > 0.0 && p.inst.bonds[i] <= 0.0)
        .collect();
    if s.debris != 0 {
        shed_crumbs(t, at, &frame, &broken_now, scale, &mut out.crumbs);
    }
    let changed = broken_now.iter().any(|&b| b)
        || (0..anchors_before.len()).any(|i| anchors_before[i] > 0.0 && p.inst.anchors[i] <= 0.0)
        || p.inst.shattered != shattered_before;
    if !changed {
        return Ok(());
    }
    let cause = cause.unwrap_or(if shatter.is_empty() {
        CAUSE_STRESS
    } else {
        CAUSE_SHATTER
    });
    // A moving prop breaks free: its chunks become entities moving with
    // it, and it loses its own body.
    let mut inherit = (Vec3::ZERO, Vec3::ZERO, frame.0);
    if !p.placed {
        let e = Entity::from_bits(k);
        if let Ok(vb) = ctx.get_bytes(e, l.velocity.0) {
            let v: record::Velocity = row(&l.velocity.1, &vb)?;
            inherit = (v.linear, v.angular, frame.0);
        }
        place(ctx, l, k, p, a)?;
    }
    p.fractured = true;
    free_islands(
        ctx,
        l,
        t,
        at,
        &frame,
        p,
        scale,
        inherit,
        &broken_now,
        cause,
        out,
    )?;
    shatter_chunks(ctx, l, t, at, p, scale, &shatter, out)
}

/// The prop's hits on its bonds, and a forced fracture: the cause of the
/// first that broke something, and the chunks with children past their
/// health, which shatter into them.
fn wear(
    at: Breaking<'_>,
    frame: &Pose,
    p: &mut Prop,
    h: &Hits,
    scale: f32,
) -> (Option<u32>, Vec<u32>) {
    let (a, s) = (at.a, at.s);
    let mut cause = None;
    let place_fn = |c: u32, x: Vec3| place_chunk(at, frame, c, x);
    for (d, c) in &h.damage {
        if p.inst.damage(a, d, scale, &s.mat, &place_fn) && cause.is_none() {
            cause = Some(*c);
        }
    }
    if h.fracture {
        p.inst.bonds.iter_mut().for_each(|b| *b = 0.0);
        cause = Some(CAUSE_FRACTURE);
    }
    // A chunk with children past its health shatters into them.
    let mut shatter = Vec::new();
    for c in 0..a.support as usize {
        let ch = &a.chunks[c];
        if !p.inst.shattered[c]
            && ch.children.1 > 0
            && p.inst.chunk_damage[c] >= Instance::chunk_health(a, c as u32, scale, s.mat.toughness)
        {
            p.inst.shattered[c] = true;
            p.inst.dirty = true;
            shatter.push(c as u32);
        }
    }
    (cause, shatter)
}

/// The stress solver on what stands (when something changed) and, every
/// REST_EVERY ticks, on each loose island at rest on what it rests on.
#[allow(clippy::too_many_arguments)]
fn solve_stress(
    ctx: &SystemContext<'_>,
    m: &ModuleState,
    l: &Ls,
    t: &Clock,
    at: Breaking<'_>,
    frame: &Pose,
    p: &mut Prop,
    scale: f32,
    stress_budget: &mut u64,
    cause: &mut Option<u32>,
    shatter: &mut Vec<u32>,
) {
    let (a, s, pose) = (at.a, at.s, at.pose);
    let (tick, gravity) = (t.tick, t.gravity);
    let iterations = t.res.stress_iterations;
    let anchors = p.anchors;
    if anchors && p.inst.dirty {
        let islands = islands_of(&p.inst, a, anchors);
        let down = frame.1.inverse() * gravity;
        let n = p.inst.stress(
            a,
            &islands,
            &[],
            down,
            scale,
            &s.mat,
            iterations,
            stress_budget,
        );
        if n > 0 && cause.is_none() {
            *cause = Some(CAUSE_STRESS);
        }
    } else {
        p.inst.dirty = false;
    }
    // A loose island at rest is solved on what it rests on: what spans
    // a hole between two piers bends, and breaks over it, where a
    // rigid compound would lie across them as a lintel. One at rest
    // with nothing under it is wedged by its sides (blocks jammed
    // between piers into a flat arch, which loose masonry never holds:
    // its corners crush): it crumbles, into its pieces.
    if tick.is_multiple_of(REST_EVERY)
        && let Some(q) = physics(m).and_then(|ph| ph.tick_query().ok())
    {
        let held = p.inst.dirty;
        for isl in islands_of(&p.inst, a, anchors)
            .into_iter()
            .filter(|i| !i.anchored)
        {
            let body = p.body[isl.chunks[0] as usize];
            if body == 0 || isl.chunks.iter().any(|&c| p.body[c as usize] != body) {
                continue;
            }
            let speed = ctx
                .get_bytes(Entity::from_bits(body), l.velocity.0)
                .ok()
                .and_then(|vb| row::<record::Velocity>(&l.velocity.1, &vb).ok())
                .map_or(0.0, |v| v.linear.length());
            // Moving or not, what rests on something is solved on it: a
            // long run of loose courses pivoting off a pier is a
            // cantilever, and cracks at the pivot as it swings rather
            // than turning away whole like a plank.
            if speed > REST_SPEED && isl.chunks.len() < 2 {
                continue;
            }
            let supports = supports_of(&q, a, pose, &isl.chunks, body, gravity);
            if supports.is_empty() {
                if speed > REST_SPEED {
                    continue;
                }
                let freed = isl.chunks.iter().map(|&c| p.loose[c as usize]).min();
                if freed.is_some_and(|t| tick.saturating_sub(t) >= WEDGED_AFTER) {
                    crumble(a, &mut p.inst, &isl.chunks, shatter);
                    cause.get_or_insert(CAUSE_STRESS);
                }
                continue;
            }
            if isl.chunks.len() < 2 {
                continue;
            }
            let Some(root) = pose[isl.chunks[0] as usize] else {
                continue;
            };
            let n = p.inst.stress(
                a,
                std::slice::from_ref(&isl),
                &supports,
                root.1.inverse() * gravity,
                scale,
                &s.mat,
                iterations,
                stress_budget,
            );
            if n > 0 && cause.is_none() {
                *cause = Some(CAUSE_STRESS);
            }
        }
        p.inst.dirty |= held;
    }
}

/// No-tension masonry (tension at most a tenth of a strength above 0;
/// strength 0 turns stress off, and with it this): every REST_EVERY ticks
/// a block of what stands that nothing bears from below comes loose,
/// whole, and what it bore is judged again the next time; and a loose run
/// of mortared blocks longer than RUN comes apart at its joints the tick
/// it breaks loose.
fn break_masonry(
    t: &Clock,
    at: Breaking<'_>,
    frame: &Pose,
    p: &mut Prop,
    before: &[f32],
    cause: &mut Option<u32>,
) {
    let (a, s) = (at.a, at.s);
    let masonry = s.mat.limits.shear_elastic.is_finite()
        && s.mat.limits.tension_elastic <= 0.1 * s.mat.limits.shear_elastic;
    let anchors = p.anchors;
    if anchors && t.tick.is_multiple_of(REST_EVERY) && masonry {
        let up = -(frame.1.inverse() * t.gravity).normalize_or_zero();
        for c in p.inst.unsupported(a, &islands_of(&p.inst, a, anchors), up) {
            p.inst.free(a, c);
            cause.get_or_insert(CAUSE_STRESS);
        }
    }
    // No-tension masonry in the air holds no bending: a loose run of
    // mortared blocks longer than RUN comes apart at its joints the
    // tick it breaks loose, into its blocks (each its own cluster of
    // bricks), so the top of a blown wall goes up as a spray of
    // blocks, not a sheet that bends away whole or a beam of courses
    // lying across the hole.
    if masonry && (0..before.len()).any(|i| before[i] > 0.0 && p.inst.bonds[i] <= 0.0) {
        let mut run = vec![false; a.chunks.len()];
        for isl in islands_of(&p.inst, a, anchors)
            .into_iter()
            .filter(|i| !i.anchored && i.chunks.len() > RUN)
        {
            isl.chunks.iter().for_each(|&c| run[c as usize] = true);
        }
        for (k, b) in a.bonds.iter().enumerate() {
            if run[b.a as usize] && p.inst.bonds[k] > 0.0 {
                p.inst.bonds[k] = 0.0;
                p.inst.dirty = true;
            }
        }
    }
}

/// The grit the joints broken this tick shed (spread over at most
/// CRUMBS_A_TICK of them), with the mass of the piece beside each.
fn shed_crumbs(
    t: &Clock,
    at: Breaking<'_>,
    frame: &Pose,
    broken_now: &[bool],
    scale: f32,
    crumbs: &mut Vec<(Crumb, f32)>,
) {
    let (a, s) = (at.a, at.s);
    let fresh: Vec<usize> = (0..broken_now.len()).filter(|&i| broken_now[i]).collect();
    let step = fresh.len().div_ceil(CRUMBS_A_TICK).max(1);
    for &i in fresh.iter().step_by(step) {
        let b = &a.bonds[i];
        let rot = at.pose[b.a as usize].map_or(frame.1, |ps| ps.1);
        let v = a.chunks[b.a as usize].volume * scale * scale * scale;
        crumbs.push((
            Crumb {
                prop: at.key,
                bond: i as u32,
                tick: t.tick,
                point: place_chunk(at, frame, b.a, b.centroid).to_array(),
                normal: (rot * b.normal).to_array(),
                velocity: [0.0; 3],
            },
            v * s.mat.density,
        ));
    }
}

/// Each island that is not anchored and not already one body becomes
/// one: its root chunk a dynamic body with the velocity it had, the rest
/// its compound children; og.Fractured says where it broke.
#[allow(clippy::too_many_arguments)]
fn free_islands(
    ctx: &mut SystemContext<'_>,
    l: &Ls,
    t: &Clock,
    at: Breaking<'_>,
    frame: &Pose,
    p: &mut Prop,
    scale: f32,
    inherit: (Vec3, Vec3, Vec3),
    broken_now: &[bool],
    cause: u32,
    out: &mut Broken,
) -> og_core::Result<()> {
    let (a, s, k, pose, tick) = (at.a, at.s, at.key, at.pose, t.tick);
    let islands = islands_of(&p.inst, a, p.anchors);
    // Each support chunk's role now.
    let role_of = |p: &Prop, c: usize| -> Role {
        let (e, b) = (p.entities[c], p.body[c]);
        if b == 0 {
            Role::Static
        } else if b == e {
            Role::Root
        } else {
            match p.entities.iter().position(|&x| x == b) {
                Some(r) => Role::Child(r as u32),
                None => Role::Static,
            }
        }
    };
    let roles: Vec<Role> = (0..a.support as usize).map(|c| role_of(p, c)).collect();
    for isl in &islands {
        if isl.anchored {
            continue;
        }
        let root_now = |c: u32| match roles[c as usize] {
            Role::Root => Some(c),
            Role::Child(r) => Some(r),
            Role::Static => None,
        };
        let first = root_now(isl.chunks[0]);
        let same = first.is_some()
            && isl.chunks.iter().all(|&c| root_now(c) == first)
            && isl.chunks.contains(&first.expect("some"))
            && (0..a.support).all(|c| isl.chunks.contains(&c) || root_now(c) != first);
        if same {
            continue;
        }
        let root = isl
            .chunks
            .iter()
            .copied()
            .find(|&c| roles[c as usize] == Role::Root)
            .unwrap_or(isl.chunks[0]);
        let root_pose = pose[root as usize].expect("a placed chunk has a pose");
        let root_entity = p.entities[root as usize];
        // The velocity it starts with: its old body's at its place.
        let (lin, ang) = match roles[root as usize] {
            Role::Root => (Vec3::ZERO, Vec3::ZERO),
            Role::Child(r) => {
                let b = Entity::from_bits(p.entities[r as usize]);
                match ctx.get_bytes(b, l.velocity.0) {
                    Ok(vb) => {
                        let v: record::Velocity = row(&l.velocity.1, &vb)?;
                        let (lin, ang) = (v.linear, v.angular);
                        let at = pose[r as usize].map_or(root_pose.0, |x| x.0);
                        (lin + ang.cross(root_pose.0 - at), ang)
                    }
                    Err(_) => (Vec3::ZERO, Vec3::ZERO),
                }
            }
            Role::Static => (
                inherit.0 + inherit.1.cross(root_pose.0 - inherit.2),
                inherit.1,
            ),
        };
        let mut mass = 0.0;
        let mut volume = 0.0;
        let mut centre = Vec3::ZERO;
        for &c in &isl.chunks {
            let target = if c == root {
                Role::Root
            } else {
                Role::Child(root)
            };
            let was = roles[c as usize];
            let e = Entity::from_bits(p.entities[c as usize]);
            let cp = pose[c as usize].expect("a placed chunk has a pose");
            let v = a.chunks[c as usize].volume * scale * scale * scale;
            volume += v;
            mass += v * s.mat.density;
            centre += cp.0 * v;
            if p.loose[c as usize] == u64::MAX {
                p.loose[c as usize] = tick;
            }
            p.body[c as usize] = root_entity;
            set_chunk(ctx, l, e, k, c, 1, root_entity, p.loose[c as usize])?;
            if was == target {
                continue;
            }
            match target {
                Role::Root => {
                    if matches!(was, Role::Child(_)) {
                        ctx.remove_id(e, l.parent.0)?;
                        set_transform(ctx, l, e, &cp)?;
                    }
                    let rb = l.rigid.1.enc(None, Some(&json!({ "kind": "dynamic" })))?;
                    ctx.insert_bytes(e, l.rigid.0, &rb)?;
                    out.made.insert(root_entity, (lin, ang));
                    out.fresh_bodies.push(root_entity);
                }
                Role::Child(_) => {
                    if was == Role::Root {
                        ctx.remove_id(e, l.rigid.0)?;
                        ctx.remove_id(e, l.velocity.0)?;
                    }
                    let pb = l
                        .parent
                        .1
                        .enc(None, Some(&json!({ "entity": root_entity as f64 })))?;
                    ctx.insert_bytes(e, l.parent.0, &pb)?;
                    set_transform(ctx, l, e, &relative(&root_pose, &cp))?;
                }
                Role::Static => {}
            }
        }
        // Where it broke: the bonds broken this run along its edge.
        let mut at_ = Vec3::ZERO;
        let mut n = 0.0;
        for (b, bond) in a.bonds.iter().enumerate() {
            let touches = isl.chunks.contains(&bond.a) || isl.chunks.contains(&bond.b);
            if touches && broken_now[b] {
                at_ += place_chunk(at, frame, bond.a, bond.centroid);
                n += 1.0;
            }
        }
        let point = if n > 0.0 {
            at_ / n
        } else {
            centre / volume.max(1e-12)
        };
        out.events.push(l.fractured.1.enc(
            None,
            Some(&json!({
                "entity": k as f64, "body": root_entity as f64,
                "chunks": isl.chunks.len(), "point": jv(point),
                "volume": volume, "mass": mass, "cause": cause,
            })),
        )?);
        if s.break_fx != 0 {
            out.bursts.push((s.break_fx, point));
        }
    }
    Ok(())
}

/// Shattered chunks: their children fly, each its own body, or, below
/// debrisVolume with a debris emitter, as fine pieces on the GPU.
#[allow(clippy::too_many_arguments)]
fn shatter_chunks(
    ctx: &mut SystemContext<'_>,
    l: &Ls,
    t: &Clock,
    at: Breaking<'_>,
    p: &mut Prop,
    scale: f32,
    shatter: &[u32],
    out: &mut Broken,
) -> og_core::Result<()> {
    let (a, s, k, pose, tick) = (at.a, at.s, at.key, at.pose, t.tick);
    for &c in shatter {
        let ch = &a.chunks[c as usize];
        let Some(cp) = pose[c as usize] else {
            continue;
        };
        let (lin, ang) = match p.body[c as usize] {
            0 => (Vec3::ZERO, Vec3::ZERO),
            b => match ctx.get_bytes(Entity::from_bits(b), l.velocity.0) {
                Ok(vb) => {
                    let v: record::Velocity = row(&l.velocity.1, &vb)?;
                    (v.linear, v.angular)
                }
                Err(_) => (Vec3::ZERO, Vec3::ZERO),
            },
        };
        let e = p.entities[c as usize];
        if alive(ctx, e) {
            ctx.despawn(Entity::from_bits(e))?;
        }
        p.entities[c as usize] = 0;
        p.body[c as usize] = 0;
        let (first, count) = ch.children;
        let mut volume = 0.0;
        let mut body = 0;
        let mut fine = 0;
        for i in first..first + count {
            let kid = &a.chunks[i as usize];
            let v = kid.volume * scale * scale * scale;
            volume += v;
            let kp = (place_on(&cp, ch.centroid, kid.centroid), cp.1, cp.2);
            // Pieces below debrisVolume, given a debris emitter, are
            // the GPU's (its burst, and the pieces themselves in the
            // renderer's debris layer): never bodies.
            if s.debris != 0 && v < t.debris_volume {
                fine += 1;
                let at = kp.0;
                out.debris.push((
                    Debris {
                        prop: k,
                        chunk: i,
                        tick,
                        pose: frame_of_pose(kp),
                        velocity: (lin + ang.cross(at - cp.0)).to_array(),
                        angular: ang.to_array(),
                    },
                    v * s.mat.density,
                ));
                continue;
            }
            let ke = spawn_chunk(ctx, l, k, p, i, 2, &kp, Role::Root, 0, tick)?;
            body = if body == 0 { ke } else { body };
            out.made.insert(ke, (lin + ang.cross(kp.0 - cp.0), ang));
            out.fresh_bodies.push(ke);
        }
        out.events.push(l.fractured.1.enc(
            None,
            Some(&json!({
                "entity": k as f64, "body": body as f64, "chunks": count - fine,
                "point": jv(cp.0), "volume": volume,
                "mass": volume * s.mat.density, "cause": CAUSE_SHATTER,
            })),
        )?);
        if s.break_fx != 0 {
            out.bursts.push((s.break_fx, cp.0));
        }
        if fine > 0 {
            out.bursts.push((s.debris, cp.0));
        }
    }
    Ok(())
}

/// The explosions' push on every loose body in reach: a body made this
/// tick takes it with its starting velocity, another on its og.Velocity.
#[allow(clippy::too_many_arguments)]
fn push_blasted_bodies(
    ctx: &mut SystemContext<'_>,
    l: &Ls,
    st: &State,
    assets: &Assets,
    settings: &BTreeMap<u64, Settings>,
    poses: &Poses,
    explosions: &[Explosion],
    out: &mut Broken,
) -> og_core::Result<()> {
    let made = &mut out.made;
    for &(point, inner, radius, impulse) in explosions {
        if impulse <= 0.0 {
            continue;
        }
        let falloff = Damage {
            point,
            min_radius: inner,
            max_radius: radius,
            amount: impulse,
        };
        for (&k, p) in &st.props {
            let a = &assets[&p.asset];
            let scale = uniform(pose_of_frame(&p.frame).2);
            let density = settings.get(&k).map_or(1000.0, |s| s.mat.density);
            let bodies = loose_bodies(ctx, l, p, a, scale, density, &poses[&k], made, point);
            for (b, (m, at)) in bodies {
                if m <= 0.0 {
                    continue;
                }
                let first = (0..p.entities.len())
                    .find(|&c| p.entities[c] == b || (c < a.support as usize && p.body[c] == b))
                    .unwrap_or(0);
                let Some((dv, dw)) = blast_push(&falloff, impulse, at, m, density, first as u64)
                else {
                    continue;
                };
                match made.get_mut(&b) {
                    Some(v) => {
                        v.0 += dv;
                        v.1 += dw;
                    }
                    None => {
                        let e = Entity::from_bits(b);
                        if let Ok(vb) = ctx.get_bytes(e, l.velocity.0) {
                            let mut v = l.velocity.1.decode(&vb);
                            let now = record::Velocity::from_json(&v).map_err(invalid)?;
                            v["linear"] = jv(now.linear + dv);
                            v["angular"] = jv(now.angular + dw);
                            let vel = l.velocity.1.enc(Some(&vb), Some(&v))?;
                            ctx.set_bytes(e, l.velocity.0, &vel)?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// The blasts' push on the fine pieces broken off this tick, as on a body
/// (the pieces already on the GPU are the renderer's), and on the grit,
/// which goes with the piece beside its joint.
fn push_blasted_pieces(
    settings: &BTreeMap<u64, Settings>,
    explosions: &[Explosion],
    out: &mut Broken,
) {
    for &(point, inner, radius, impulse) in explosions {
        if impulse <= 0.0 {
            continue;
        }
        let falloff = Damage {
            point,
            min_radius: inner,
            max_radius: radius,
            amount: impulse,
        };
        for (d, m) in &mut out.debris {
            let density = settings.get(&d.prop).map_or(1000.0, |s| s.mat.density);
            let at = Vec3::new(d.pose[0], d.pose[1], d.pose[2]);
            if let Some((dv, dw)) =
                blast_push(&falloff, impulse, at, *m, density, u64::from(d.chunk))
            {
                d.velocity = (Vec3::from(d.velocity) + dv).to_array();
                d.angular = (Vec3::from(d.angular) + dw).to_array();
            }
        }
        // Grit goes with the piece beside its joint.
        for (c, m) in &mut out.crumbs {
            let density = settings.get(&c.prop).map_or(1000.0, |s| s.mat.density);
            let at = Vec3::from(c.point);
            if let Some((dv, _)) = blast_push(&falloff, impulse, at, *m, density, u64::from(c.bond))
            {
                c.velocity = (Vec3::from(c.velocity) + dv).to_array();
            }
        }
    }
}

/// A hit's or a blow's push on the pieces it broke loose: the impulse
/// past the threshold (momentum the striker gave up breaking through), or
/// the blow's, shared by falloff over them, none driven faster than the
/// striker closed.
#[allow(clippy::too_many_arguments)]
fn push_struck(
    ctx: &SystemContext<'_>,
    l: &Ls,
    st: &State,
    assets: &Assets,
    settings: &BTreeMap<u64, Settings>,
    poses: &Poses,
    strikes: &[Strike],
    out: &mut Broken,
) {
    for &(k, point, dir, radius, amount, speed) in strikes {
        let (Some(p), Some(s)) = (st.props.get(&k), settings.get(&k)) else {
            continue;
        };
        let a = &assets[&p.asset];
        let scale = uniform(pose_of_frame(&p.frame).2);
        let falloff = Damage {
            point,
            min_radius: radius / 3.0,
            max_radius: radius,
            amount: 1.0,
        };
        let made = &out.made;
        let freed: Vec<(u64, f32, f32)> =
            loose_bodies(ctx, l, p, a, scale, s.mat.density, &poses[&k], made, point)
                .into_iter()
                .filter(|(b, _)| made.contains_key(b))
                .map(|(b, (m, at))| (b, m, falloff.at(at)))
                .filter(|&(_, m, w)| m > 0.0 && w > 0.0)
                .collect();
        // One velocity for the whole freed mass, scaled by falloff: the
        // momentum is conserved and the lightest piece is not flung.
        let weighed: f32 = freed.iter().map(|&(_, m, w)| m * w).sum();
        // The fine pieces it broke off this tick move with them.
        let pieces: f32 = out
            .debris
            .iter()
            .filter(|(d, _)| d.prop == k)
            .map(|(d, m)| m * falloff.at(Vec3::new(d.pose[0], d.pose[1], d.pose[2])))
            .sum();
        let weighed = weighed + pieces;
        for (b, _, w) in freed {
            let dv = (amount * w / weighed).min(speed);
            if let Some(v) = out.made.get_mut(&b) {
                v.0 += dir * dv;
            }
        }
        for (d, _) in out.debris.iter_mut().filter(|(d, _)| d.prop == k) {
            let w = falloff.at(Vec3::new(d.pose[0], d.pose[1], d.pose[2]));
            if w > 0.0 && weighed > 0.0 {
                let dv = (amount * w / weighed).min(speed);
                d.velocity = (Vec3::from(d.velocity) + dir * dv).to_array();
            }
        }
        for (c, _) in out.crumbs.iter_mut().filter(|(c, _)| c.prop == k) {
            let w = falloff.at(Vec3::from(c.point));
            if w > 0.0 && weighed > 0.0 {
                let dv = (amount * w / weighed).min(speed);
                c.velocity = (Vec3::from(c.velocity) + dir * dv).to_array();
            }
        }
    }
}

/// The fine pieces and grit broken off join the state; the bodies made
/// this tick take their velocity.
fn commit_fresh(
    ctx: &mut SystemContext<'_>,
    l: &Ls,
    st: &mut State,
    out: &mut Broken,
) -> og_core::Result<()> {
    st.debris.extend(out.debris.drain(..).map(|(d, _)| d));
    st.crumbs.extend(out.crumbs.drain(..).map(|(c, _)| c));
    for &b in &out.fresh_bodies {
        let (lin, ang) = out.made[&b];
        let vb = l.velocity.1.enc(
            None,
            Some(&json!({ "linear": jv(lin), "angular": jv(ang) })),
        )?;
        ctx.insert_bytes(Entity::from_bits(b), l.velocity.0, &vb)?;
    }
    Ok(())
}

/// Budgets: loose islands (a body and its children) by when they broke
/// loose; debris past its lifetime, then the oldest past maxChunks; fine
/// pieces and grit after debrisLifetime, with their prop, and the oldest
/// past what the renderer holds.
fn enforce_budgets(
    ctx: &mut SystemContext<'_>,
    st: &mut State,
    assets: &Assets,
    t: &Clock,
) -> og_core::Result<()> {
    let tick = t.tick;
    let max = t.res.max_chunks as usize;

    let lifetime = t.res.debris_lifetime;
    struct Loose {
        tick: u64,
        volume: f32,
        body: u64,
        prop: u64,
        chunks: Vec<usize>,
    }
    let mut loose: BTreeMap<u64, Loose> = BTreeMap::new();
    for (&k, p) in &st.props {
        let a = &assets[&p.asset];
        let scale = uniform(pose_of_frame(&p.frame).2);
        for c in 0..a.chunks.len() {
            let e = p.entities[c];
            let b = if c < a.support as usize { p.body[c] } else { e };
            if e == 0 || b == 0 {
                continue;
            }
            let l = loose.entry(b).or_insert(Loose {
                tick: p.loose[c],
                volume: 0.0,
                body: b,
                prop: k,
                chunks: Vec::new(),
            });
            l.tick = l.tick.min(p.loose[c]);
            l.volume += a.chunks[c].volume * scale * scale * scale;
            l.chunks.push(c);
        }
    }
    let mut order: Vec<Loose> = loose.into_values().collect();
    order.sort_by(|x, y| {
        (x.tick, x.volume.to_bits(), x.body).cmp(&(y.tick, y.volume.to_bits(), y.body))
    });
    let mut total: usize = order.iter().map(|x| x.chunks.len()).sum();
    let life = t.ticks_of(lifetime);
    for isl in &order {
        let old =
            lifetime > 0.0 && isl.volume < t.debris_volume && tick >= isl.tick.saturating_add(life);
        if !old && total <= max {
            continue;
        }
        total -= isl.chunks.len();
        let p = st.props.get_mut(&isl.prop).expect("listed");
        for &c in &isl.chunks {
            let e = p.entities[c];
            if alive(ctx, e) {
                ctx.despawn(Entity::from_bits(e))?;
            }
            p.entities[c] = 0;
            if c < p.body.len() {
                p.body[c] = 0;
            }
            if let Some(sh) = p.inst.shattered.get_mut(c) {
                *sh = true;
            }
        }
    }
    // Fine pieces: gone after debrisLifetime, or with their prop, and the
    // oldest past what the renderer's debris layer holds.
    let props = &st.props;
    st.debris.retain(|d| {
        props.contains_key(&d.prop) && !(lifetime > 0.0 && tick >= d.tick.saturating_add(life))
    });
    let over = st.debris.len().saturating_sub(og_render::MAX_DEBRIS);
    st.debris.drain(..over);
    st.crumbs.retain(|c| {
        props.contains_key(&c.prop) && !(lifetime > 0.0 && tick >= c.tick.saturating_add(life))
    });
    let over = st.crumbs.len().saturating_sub(MAX_CRUMBS);
    st.crumbs.drain(..over);
    Ok(())
}

/// What the game reads: each destructible's written fields, og.Fractured,
/// and breakFx and hitFx as one-shot bursts, despawned after fxLifetime.
fn publish(
    ctx: &mut SystemContext<'_>,
    l: &Ls,
    st: &mut State,
    t: &Clock,
    events: Vec<Vec<u8>>,
    bursts: Vec<(u64, Vec3)>,
) -> og_core::Result<()> {
    let tick = t.tick;
    for (&k, p) in &st.props {
        let e = Entity::from_bits(k);
        let Ok(b) = ctx.get_bytes(e, l.destructible.0) else {
            continue;
        };
        let mut d = l.destructible.1.decode(&b);
        let broken = p.inst.bonds.iter().filter(|&&h| h <= 0.0).count();
        let n = (0..p.entities.len())
            .filter(|&c| p.entities[c] != 0 && p.loose[c] != u64::MAX)
            .count();
        d["fractured"] = json!(p.fractured);
        d["broken"] = json!(broken);
        d["loose"] = json!(n);
        let out = l.destructible.1.enc(Some(&b), Some(&d))?;
        if out != b {
            ctx.set_bytes(e, l.destructible.0, &out)?;
        }
    }
    for ev in events {
        ctx.send_bytes(l.fractured.0, &ev)?;
    }
    let fx_life = t.ticks_of(t.res.fx_lifetime).max(1);
    for (i, (emitter, at)) in bursts.into_iter().take(MAX_BURSTS).enumerate() {
        let tr = l
            .transform
            .1
            .enc(None, Some(&json!({ "position": jv(at) })))?;
        let seed = (tick as u32).wrapping_mul(2_654_435_761) ^ i as u32;
        let ps = l.particles.1.enc(
            None,
            Some(&json!({ "emitter": emitter as f64, "seed": seed, "startTick": tick as u32 })),
        )?;
        let e = ctx.spawn_bytes(vec![(l.transform.0, tr), (l.particles.0, ps)])?;
        st.fx.push((e.to_bits(), tick + fx_life));
    }
    Ok(())
}

/// A body's mass as a hit weighs it.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Mass {
    Finite(f32),
    /// Static, kinematic or no body: it does not give.
    Infinite,
    /// A shape whose volume is not known here (a mesh): the hit is the
    /// solver's impulse alone.
    Unknown,
}

/// The mass of the body entity `bits` belongs to: a chunk's island, an
/// unbroken prop's chunks, or another body from its og.RigidBody.mass or
/// its og.Collider's shape and density.
fn mass_of(
    ctx: &SystemContext<'_>,
    l: &Ls,
    props: &BTreeMap<u64, Prop>,
    assets: &Assets,
    settings: &BTreeMap<u64, Settings>,
    bits_: u64,
) -> Mass {
    let e = Entity::from_bits(bits_);
    let scale_of = |p: &Prop| uniform(pose_of_frame(&p.frame).2);
    for (k, p) in props {
        let Some(a) = assets.get(&p.asset) else {
            continue;
        };
        let density = settings.get(k).map_or(1000.0, |s| s.mat.density);
        let s3 = {
            let k = scale_of(p);
            k * k * k
        };
        if let Some(c) = p.entities.iter().position(|&x| x == bits_) {
            let body = p.body[c];
            if body == 0 {
                return Mass::Infinite;
            }
            let v: f32 = (0..p.entities.len())
                .filter(|&i| p.entities[i] != 0 && p.body[i] == body)
                .map(|i| a.chunks[i].volume)
                .sum();
            return Mass::Finite(v * s3 * density);
        }
        if *k == bits_ && !p.placed {
            let v: f32 = a.chunks[..a.support as usize]
                .iter()
                .map(|c| c.volume)
                .sum();
            return Mass::Finite(v * s3 * density);
        }
    }
    let Ok(rb) = ctx.get_bytes(e, l.rigid.0) else {
        return Mass::Infinite;
    };
    let Ok(rb) = row::<record::RigidBody>(&l.rigid.1, &rb) else {
        return Mass::Unknown;
    };
    // og.RigidBody.kind is an enum, which the hand read this replaced
    // always took as 0 (dynamic): a fixed or kinematic body is weighed as
    // a dynamic one (kept as it was).
    let m = rb.mass;
    if m > 0.0 {
        return Mass::Finite(m);
    }
    let Ok(cb) = ctx.get_bytes(e, l.collider.0) else {
        return Mass::Unknown;
    };
    let Ok(c) = row::<record::Collider>(&l.collider.1, &cb) else {
        return Mass::Unknown;
    };
    let s = world_pose(ctx, l, e).map_or(Vec3::ONE, |p| p.2.abs());
    let (r, h) = (c.radius, c.height);
    let pi = std::f32::consts::PI;
    let volume = match c.shape {
        og_physics::SHAPE_BOX => {
            let z = c.size;
            z.x * z.y * z.z * s.x * s.y * s.z
        }
        og_physics::SHAPE_SPHERE => 4.0 / 3.0 * pi * r * r * r * s.x * s.y * s.z,
        og_physics::SHAPE_CAPSULE => {
            (pi * r * r * h + 4.0 / 3.0 * pi * r * r * r) * s.x * s.y * s.z
        }
        og_physics::SHAPE_CYLINDER => pi * r * r * h * s.x * s.y * s.z,
        _ => return Mass::Unknown,
    };
    Mass::Finite(volume * c.density.max(0.0))
}

fn set_transform(ctx: &mut SystemContext<'_>, l: &Ls, e: Entity, p: &Pose) -> og_core::Result<()> {
    let t = l.transform.1.enc(
        None,
        Some(&json!({ "position": jv(p.0), "quaternion": jq(p.1), "scale": jv(p.2) })),
    )?;
    ctx.insert_bytes(e, l.transform.0, &t)
}

#[allow(clippy::too_many_arguments)]
fn set_chunk(
    ctx: &mut SystemContext<'_>,
    l: &Ls,
    e: Entity,
    prop: u64,
    chunk: u32,
    level: u32,
    body: u64,
    loose: u64,
) -> og_core::Result<()> {
    let b = l
        .chunk
        .1
        .enc(None, Some(&chunk_value(prop, chunk, level, body, loose)))?;
    ctx.insert_bytes(e, l.chunk.0, &b)
}

fn chunk_value(prop: u64, chunk: u32, level: u32, body: u64, loose: u64) -> Value {
    json!({
        "destructible": prop as f64, "chunk": chunk, "level": level,
        "body": body as f64,
        "looseTick": if loose == u64::MAX { ANCHORED } else { loose.min(u64::from(u32::MAX - 1)) as u32 },
    })
}

/// Spawns chunk `c` of prop `k` at `pose` (in the world for a static chunk
/// or a root; relative to its body for a child), as `role`.
#[allow(clippy::too_many_arguments)]
fn spawn_chunk(
    ctx: &mut SystemContext<'_>,
    l: &Ls,
    k: u64,
    p: &mut Prop,
    c: u32,
    level: u32,
    pose: &Pose,
    role: Role,
    body: u64,
    tick: u64,
) -> og_core::Result<u64> {
    let mut collider = match &p.collider {
        Some(b) => l.collider.1.decode(b),
        None => json!({}),
    };
    let keep = ["friction", "restitution", "layers", "mask"];
    if let Value::Object(o) = &mut collider {
        o.retain(|k, _| keep.contains(&k.as_str()));
    }
    collider["shape"] = json!(og_physics::SHAPE_CONVEX);
    collider["mesh"] = json!(hull_handle(p.asset, c) as f64);
    let mut parts = vec![
        (
            l.transform.0,
            l.transform.1.enc(
                None,
                Some(&json!({ "position": jv(pose.0), "quaternion": jq(pose.1), "scale": jv(pose.2) })),
            )?,
        ),
        (l.collider.0, l.collider.1.enc(None, Some(&collider))?),
    ];
    let (loose, body_bits) = match role {
        Role::Static => (u64::MAX, 0),
        _ => (tick, body),
    };
    match role {
        Role::Root => parts.push((
            l.rigid.0,
            l.rigid.1.enc(None, Some(&json!({ "kind": "dynamic" })))?,
        )),
        Role::Child(_) => parts.push((
            l.parent.0,
            l.parent
                .1
                .enc(None, Some(&json!({ "entity": body as f64 })))?,
        )),
        Role::Static => {}
    }
    // A root's body is itself, whose id the spawn gives: its og.Chunk is
    // inserted after (a reserved entity takes commands at once).
    let e = ctx.spawn_bytes(parts)?;
    let body_bits = if role == Role::Root {
        e.to_bits()
    } else {
        body_bits
    };
    set_chunk(ctx, l, e, k, c, level, body_bits, loose)?;
    p.entities[c as usize] = e.to_bits();
    p.body[c as usize] = body_bits;
    p.loose[c as usize] = loose;
    Ok(e.to_bits())
}

/// Spawns a prop's support chunks as static colliders where the model
/// stands, and takes its own collider and body away.
fn place(
    ctx: &mut SystemContext<'_>,
    l: &Ls,
    k: u64,
    p: &mut Prop,
    a: &FractureAsset,
) -> og_core::Result<()> {
    let e = Entity::from_bits(k);
    p.collider = ctx.get_bytes(e, l.collider.0).ok();
    for id in [l.collider.0, l.rigid.0, l.velocity.0] {
        if ctx.get_bytes(e, id).is_ok() {
            ctx.remove_id(e, id)?;
        }
    }
    let frame = pose_of_frame(&p.frame);
    for c in 0..a.support {
        if p.inst.shattered[c as usize] {
            continue;
        }
        let at = (
            place_on(&frame, Vec3::ZERO, a.chunks[c as usize].centroid),
            frame.1,
            frame.2,
        );
        spawn_chunk(ctx, l, k, p, c, 1, &at, Role::Static, 0, 0)?;
    }
    p.placed = true;
    Ok(())
}

// ── the state block ──────────────────────────────────────────────────────────

struct Block(Arc<Mutex<State>>);

const MAGIC: &[u8; 8] = b"DESTRUC2";

impl StateBlock for Block {
    fn save(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(MAGIC);
        out.extend(
            bincode::serialize(&*lock(&self.0)).expect("the og.destruction state serialises"),
        );
    }
    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn Any + Send>> {
        let rest = bytes
            .strip_prefix(MAGIC)
            .ok_or_else(|| invalid("not an og.destruction state"))?;
        let s: State = bincode::deserialize(rest).map_err(|e| invalid(e.to_string()))?;
        Ok(Box::new(s))
    }
    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        *lock(&self.0) = decoded
            .and_then(|d| d.downcast::<State>().ok())
            .map(|b| *b)
            .unwrap_or_default();
    }
    /// The frames and queued points are in the world.
    fn shift(&mut self, s: &og_ecs::Shift) {
        let mut st = lock(&self.0);
        let m = |p: &mut [f32]| {
            let v = (Vec3::new(p[0], p[1], p[2]).as_dvec3() + s.d).as_vec3();
            p[..3].copy_from_slice(&v.to_array());
        };
        for p in st.props.values_mut() {
            m(&mut p.frame);
        }
        for d in &mut st.debris {
            m(&mut d.pose);
        }
        for c in &mut st.crumbs {
            m(&mut c.point);
        }
        for op in &mut st.ops {
            match op {
                Op::Damage { point, .. } | Op::Explode { point, .. } => m(point),
                Op::Fracture { .. } => {}
            }
        }
    }
}

// ── drawing ──────────────────────────────────────────────────────────────────

/// A fractured prop as extraction draws it: its og.Model's entity, and
/// each chunk's entity with the glTF node that draws it, its centre in
/// the model and its outline group (og_render::InstanceFx::outline): the
/// chunks still bonded into one island share one, so an inked look lines
/// the island round and its cracks only once they open.
pub(crate) struct Drawn {
    pub entity: u64,
    pub chunks: Vec<(u64, u32, Vec3, u32)>,
    /// Its fine pieces, the renderer's to move (og_render::DebrisPiece).
    pub debris: Vec<DrawnDebris>,
    /// The grit its broken joints shed, and the glTF material of its
    /// mortar (og_destruct::NO_MATERIAL for none).
    pub crumbs: Vec<DrawnCrumb>,
    pub mortar: u32,
}

/// A broken joint's grit as extraction hands it to the renderer: an id
/// stable while it lives, the glTF node of the piece on its first side
/// (whose cut faces the clay chips are drawn with), the tick it broke, the
/// joint's centre and normal then, half its face's extent (m: a joint's
/// centre lies inside the prop, its crack at the face's edges), and what
/// the blow gave it.
pub(crate) struct DrawnCrumb {
    pub id: u64,
    pub node: u32,
    pub tick: u64,
    pub point: Vec3,
    pub normal: Vec3,
    pub half: f32,
    pub velocity: Vec3,
}

/// A fine piece as extraction hands it to the renderer: an id stable while
/// it lives, the glTF node that draws it and its centre in the model, the
/// tick it broke off with its pose (centre of mass, rotation, scale) and
/// velocities then, and the points it collides at (its hull's vertices
/// about its centre of mass, at most og_render::DEBRIS_POINTS).
pub(crate) struct DrawnDebris {
    pub id: u64,
    pub node: u32,
    pub centre: Vec3,
    pub tick: u64,
    pub pose: (Vec3, Quat, Vec3),
    pub velocity: Vec3,
    pub angular: Vec3,
    pub points: Vec<Vec3>,
}

/// Up to `n` of `hull`'s points spread over it (farthest-point sampling
/// from the first): what a fine piece collides at.
fn spread(hull: &[Vec3], n: usize) -> Vec<Vec3> {
    if hull.len() <= n {
        return hull.to_vec();
    }
    let mut out = vec![hull[0]];
    let mut near: Vec<f32> = hull.iter().map(|p| p.distance_squared(hull[0])).collect();
    while out.len() < n {
        let (far, _) = near
            .iter()
            .enumerate()
            .fold((0, -1.0f32), |b, (i, &d)| if d > b.1 { (i, d) } else { b });
        out.push(hull[far]);
        for (d, p) in near.iter_mut().zip(hull) {
            *d = d.min(p.distance_squared(hull[far]));
        }
    }
    out
}

/// Engine `engine`'s fractured props.
pub(crate) fn drawn(engine: u32) -> Vec<Drawn> {
    let Ok(m) = module(engine) else {
        return Vec::new();
    };
    let assets = lock(&m.assets);
    let st = lock(&m.state);
    st.props
        .iter()
        .filter(|(_, p)| p.fractured)
        .filter_map(|(&k, p)| {
            let a = assets.get(&p.asset)?;
            // Each support chunk's island (by its first chunk); a child
            // chunk, loose on its own, its own group.
            let mut island = vec![u32::MAX; a.chunks.len()];
            for isl in p.inst.islands(a) {
                isl.chunks
                    .iter()
                    .for_each(|&c| island[c as usize] = isl.chunks[0]);
            }
            let group = |c: usize| {
                let of = if island[c] == u32::MAX {
                    c as u32
                } else {
                    island[c]
                };
                // Never 0 (no group): mixed with the prop, so two props'
                // islands are two objects.
                let h = (k ^ (k >> 32)) as u32 ^ of.wrapping_mul(0x9e37_79b9);
                (h ^ (h >> 15)).wrapping_mul(0x2c1b_3c6d).max(1)
            };
            let debris = st
                .debris
                .iter()
                .filter(|d| d.prop == k)
                .filter_map(|d| {
                    let c = a.chunks.get(d.chunk as usize)?;
                    Some(DrawnDebris {
                        id: scramble(k.rotate_left(17) ^ (u64::from(d.chunk) << 1 | 1)),
                        node: c.node,
                        centre: c.centroid,
                        tick: d.tick,
                        pose: pose_of_frame(&d.pose),
                        velocity: Vec3::from(d.velocity),
                        angular: Vec3::from(d.angular),
                        points: spread(&c.hull, og_render::DEBRIS_POINTS),
                    })
                })
                .collect();
            Some(Drawn {
                entity: k,
                chunks: p
                    .entities
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| **e != 0)
                    .map(|(c, &e)| (e, a.chunks[c].node, a.chunks[c].centroid, group(c)))
                    .collect(),
                debris,
                crumbs: st
                    .crumbs
                    .iter()
                    .filter(|c| c.prop == k)
                    .filter_map(|c| {
                        let b = a.bonds.get(c.bond as usize)?;
                        Some(DrawnCrumb {
                            id: scramble(k.rotate_left(29) ^ (u64::from(c.bond) << 2 | 2)),
                            node: a.chunks.get(b.a as usize)?.node,
                            tick: c.tick,
                            point: Vec3::from(c.point),
                            normal: Vec3::from(c.normal),
                            half: 0.5 * b.area.max(0.0).sqrt() * pose_of_frame(&p.frame).2.x.abs(),
                            velocity: Vec3::from(c.velocity),
                        })
                    })
                    .collect(),
                mortar: a.mortar,
            })
        })
        .collect()
}

// ── the API ──────────────────────────────────────────────────────────────────

fn row_of<T: FromJson>(eng: &Engine, name: &str, e: Entity) -> Option<T> {
    let w = eng.world().ok()?;
    let id = w.id_by_name(name).ok()?;
    let bytes = w.get_bytes(e, id).ok()?;
    let layout = eng.state.borrow().layouts.get(name).cloned()?;
    layout.read(&bytes).ok()
}

/// The og.Destructible `e` names: itself, or the one its og.Chunk is of.
fn destructible_of(eng: &Engine, e: Entity, chunks: bool) -> Result<Entity, String> {
    let w = eng.world()?;
    if !w.is_alive(e) {
        return Err(format!("entity {} is not alive", e.to_bits()));
    }
    if row_of::<record::Destructible>(eng, DESTRUCTIBLE, e).is_some() {
        return Ok(e);
    }
    if chunks && let Some(c) = row_of::<record::Chunk>(eng, CHUNK, e) {
        return Ok(Entity::from_bits(c.destructible));
    }
    Err(format!(
        "entity {} has no og.Destructible{}",
        e.to_bits(),
        if chunks { " or og.Chunk" } else { "" }
    ))
}

fn point(p: Vec3, what: &str) -> Result<[f32; 3], String> {
    if p.is_finite() {
        Ok(p.to_array())
    } else {
        Err(format!("{what} is not finite"))
    }
}

fn radius(r: f32, key: &str) -> Result<f32, String> {
    if r.is_finite() && r >= 0.0 {
        Ok(r)
    } else {
        Err(format!(
            "desc.{key} {r} is not a finite, non-negative number"
        ))
    }
}

pub fn og_destruction_damage(world: u32, entity: f64, desc: Option<String>) -> Result<(), String> {
    (|| {
        let eng = engine(world)?;
        let target = destructible_of(&eng, Entity::from_bits(entity as u64), true)?;
        let d: DamageDesc = wire::arg(desc.as_deref(), "desc")?;
        let op = Op::Damage {
            entity: target.to_bits(),
            point: point(d.point, "desc.point")?,
            radius: radius(d.radius, "radius")?.max(1e-3),
            inner: radius(d.inner_radius, "innerRadius")?,
            amount: radius(d.amount, "amount")?,
            impulse: point(d.impulse, "desc.impulse")?,
        };
        lock(&eng.modules.destruction.state).ops.push(op);
        Ok(())
    })()
}

pub fn og_destruction_explode(world: u32, desc: Option<String>) -> Result<(), String> {
    (|| {
        let eng = engine(world)?;
        let d: ExplosionDesc = wire::arg(desc.as_deref(), "desc")?;
        let op = Op::Explode {
            point: point(d.point, "desc.point")?,
            radius: radius(d.radius, "radius")?.max(1e-3),
            inner: radius(d.inner_radius, "innerRadius")?,
            damage: radius(d.damage, "damage")?,
            impulse: radius(d.impulse, "impulse")?,
        };
        // A world with no destructible has nothing to break.
        if eng.world()?.systems().contains(&SYSTEM) {
            lock(&eng.modules.destruction.state).ops.push(op);
        }
        Ok(())
    })()
}

pub fn og_destruction_fracture(world: u32, entity: f64) -> Result<(), String> {
    (|| {
        let eng = engine(world)?;
        let target = destructible_of(&eng, Entity::from_bits(entity as u64), false)?;
        lock(&eng.modules.destruction.state).ops.push(Op::Fracture {
            entity: target.to_bits(),
        });
        Ok(())
    })()
}

pub fn og_destruction_info(
    world: u32,
    entity: f64,
) -> Result<crate::wire::DestructibleInfo, String> {
    (|| {
        let eng = engine(world)?;
        let e = destructible_of(&eng, Entity::from_bits(entity as u64), false)?;
        let model: record::Model = row_of(&eng, "og.Model", e)
            .ok_or_else(|| format!("entity {} has no og.Model", e.to_bits()))?;
        let asset = model.asset;
        let m = &eng.modules.destruction;
        let a = lock(&m.assets).get(&asset).cloned().ok_or_else(|| {
            format!(
                "entity {}'s og.Model is not loaded, or was not cooked with a `fracture` block",
                e.to_bits()
            )
        })?;
        let (inst, debris, crumbs) = {
            let st = lock(&m.state);
            let inst = st
                .props
                .get(&e.to_bits())
                .filter(|p| p.asset == asset)
                .map(|p| p.inst.clone())
                .unwrap_or_else(|| Instance::new(&a));
            let k = e.to_bits();
            (
                inst,
                st.debris.iter().filter(|d| d.prop == k).count(),
                st.crumbs.iter().filter(|c| c.prop == k).count(),
            )
        };
        let total: f32 = a.bonds.iter().map(|b| b.area).sum();
        let left: f32 = a
            .bonds
            .iter()
            .zip(&inst.bonds)
            .map(|(b, h)| b.area * h)
            .sum();
        let support = a.support as usize;
        Ok::<_, String>(DestructibleInfo {
            chunks: a.support,
            children: (a.chunks.len() - support) as u32,
            bonds: a.bonds.len() as u32,
            intact: inst.bonds.iter().filter(|&&h| h > 0.0).count() as u32,
            islands: inst.islands(&a).len() as u32,
            anchored: (0..support)
                .filter(|&c| inst.anchors[c] > 0.0 && !inst.shattered[c])
                .count() as u32,
            health: if total > 0.0 { left / total } else { 1.0 },
            debris: debris as u32,
            crumbs: crumbs as u32,
        })
    })()
}

/// The face a piece of `volume` m³ shows a blast (m²): a blast's push is
/// its impulse per square metre times that, so a brick is thrown faster
/// than a run of twenty bricks, by the cube root of twenty, not twenty
/// times (a blast pushes on surfaces: its momentum to a piece grows with
/// the piece's face, not as one shove per body however small).
fn face(volume: f32) -> f32 {
    og_core::math::pow(volume.max(0.0), 2.0 / 3.0)
}

/// og.destruction.material: (toughness, strength, friction, tension,
/// density, impactThreshold, impactRadius) per MATERIAL_*, the values the
/// demolition-yard props were tuned to.
const MATERIALS: [[f32; 7]; 5] = [
    // Brick: tough bricks, weak mortar held by friction where it is
    // pressed and next to nothing pulled apart; heavy blocks land at
    // hundreds of N·s, so a lower threshold lets falling blocks knock
    // the piers apart.
    [800.0, 3000.0, 0.7, 0.1, 1900.0, 400.0, 0.4],
    // Concrete: a column's top, dropping onto its stump when its middle is
    // shot out, lands at over 1000 N·s; below 1500 that crushes the stump.
    [2000.0, 20000.0, 0.7, 1.0, 2400.0, 1500.0, 0.4],
    // Wood: weak across the grain, a hit spread over most of a crate.
    [120.0, 30000.0, 0.7, 1.0, 600.0, 40.0, 0.9],
    // Glass: shatters at a touch, no stress solver.
    [25.0, 0.0, 0.7, 1.0, 2500.0, 2.0, 0.8],
    // Dry stone: no mortar, no tension.
    [1500.0, 4000.0, 0.8, 0.0, 2600.0, 500.0, 0.4],
];

pub fn og_destruction_material(
    world: u32,
    kind: u32,
) -> Result<crate::wire::DestructibleMaterial, String> {
    engine(world)?;
    let Some(m) = MATERIALS.get(kind as usize) else {
        return Err(format!("{kind} is none of og.MATERIAL_*"));
    };
    Ok(DestructibleMaterial {
        toughness: m[0],
        strength: m[1],
        friction: m[2],
        tension: m[3],
        density: m[4],
        impact_threshold: m[5],
        impact_radius: m[6],
    })
}

pub fn og_destruction_capabilities(world: u32) -> crate::wire::DestructionCapabilities {
    // og.Destructible.debris is the renderer's debris layer and a GPU
    // particle burst: drawn where a renderer runs compute passes (every
    // renderer the engine makes: og.render.capabilities' compute).
    let compute = engine(world).is_ok_and(|e| e.render.borrow().backend.is_some());
    DestructionCapabilities {
        visual_debris: compute,
        max_visual_debris: og_render::MAX_DEBRIS as u32,
        visual_debris_contacts: "world".into(),
    }
}
