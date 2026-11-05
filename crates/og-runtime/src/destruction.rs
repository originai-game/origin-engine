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
use super::engine::{Engine, engine};
use super::{ok, ok_void, parse, refuse};
use glam::{Quat, Vec3};
use og_core::{Error, ErrorKind};
use og_destruct::{Damage, FractureAsset, Instance, Material, Support, stress::StressLimits};
use og_ecs::{Entity, QueryDesc, SchemaId, StateBlock, SystemContext, SystemDesc, World};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::any::Any;
use std::cell::RefCell;
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

thread_local! {
    static SHARED: RefCell<og_core::IdMap<u32, Arc<Mutex<State>>>> = const { RefCell::new(og_core::IdMap::new()) };
    static ASSETS: RefCell<og_core::IdMap<u32, Arc<Mutex<Assets>>>> = const { RefCell::new(og_core::IdMap::new()) };
}

/// The fracture assets of an engine's loaded models, by asset handle.
type Assets = BTreeMap<u64, Arc<FractureAsset>>;

thread_local! {
    /// Each engine's physics, for the scene queries the system makes
    /// (what a loose island rests on).
    static PHYSICS: RefCell<BTreeMap<u32, og_physics::PhysicsSystem>> =
        const { RefCell::new(BTreeMap::new()) };
}

/// The physics an engine's destruction queries (made with its world).
pub(crate) fn set_physics(engine: u32, physics: &og_physics::PhysicsSystem) {
    PHYSICS.with(|m| m.borrow_mut().insert(engine, physics.clone()));
}

/// How often (in ticks) a loose island at rest is solved on what holds it.
const REST_EVERY: u64 = 6;
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
    let mut bodies: BTreeMap<u64, (f32, Vec3)> = BTreeMap::new();
    for (c, &b) in p.body.iter().enumerate() {
        if b == 0 || p.entities[c] == 0 {
            continue;
        }
        let m = a.chunks[c].volume * scale * scale * scale * density;
        let e = bodies.entry(b).or_insert((0.0, Vec3::ZERO));
        e.0 += m;
        if p.entities[c] == b
            && let Some(ps) = poses[c]
        {
            e.1 = ps.0;
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

fn shared(engine: u32) -> Arc<Mutex<State>> {
    SHARED.with(|s| s.borrow_mut().entry(engine).or_default().clone())
}

fn assets(engine: u32) -> Arc<Mutex<Assets>> {
    ASSETS.with(|s| s.borrow_mut().entry(engine).or_default().clone())
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
            lock(&assets_of(eng)).remove(&handle);
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
    lock(&assets_of(eng)).insert(handle, Arc::new(a));
    Ok(())
}

fn assets_of(eng: &Engine) -> Arc<Mutex<Assets>> {
    assets(eng.id)
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
pub(crate) fn enable(world: &mut World, engine_id: u32) -> og_core::Result<()> {
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
    let sh = shared(engine_id);
    *lock(&sh) = State::default();
    let (state, assets) = (sh.clone(), assets(engine_id));
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
        move |ctx| run(ctx, engine_id, &state, &assets, &ls),
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

fn v3(v: &Value) -> Vec3 {
    let c = |k: &str| v[k].as_f64().unwrap_or(0.0) as f32;
    Vec3::new(c("x"), c("y"), c("z"))
}

fn jv(p: Vec3) -> Value {
    json!({ "x": p.x, "y": p.y, "z": p.z })
}

fn jq(q: Quat) -> Value {
    json!({ "x": q.x, "y": q.y, "z": q.z, "w": q.w })
}

fn num(v: &Value, k: &str, d: f32) -> f32 {
    v[k].as_f64().map_or(d, |x| x as f32)
}

fn bits(v: &Value, k: &str) -> u64 {
    v[k].as_f64().map_or(0, |x| x as u64)
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
    fn of(v: &Value) -> Settings {
        Settings {
            mat: Material {
                toughness: num(v, "toughness", 800.0).max(1e-3),
                limits: StressLimits::from_strength(
                    num(v, "strength", 30000.0),
                    num(v, "friction", 0.7),
                ),
                density: num(v, "density", 1000.0).max(1.0),
            },
            threshold: num(v, "impactThreshold", 30.0).max(0.0),
            radius: num(v, "impactRadius", 0.4).max(0.01),
            anchored: v["anchored"].as_bool().unwrap_or(true),
            break_fx: bits(v, "breakFx"),
            hit_fx: bits(v, "hitFx"),
            debris: bits(v, "debris"),
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
        let t = l.transform.1.decode(&ctx.get_bytes(e, l.transform.0).ok()?);
        let q = &t["quaternion"];
        let s = &t["scale"];
        Some((
            v3(&t["position"]),
            Quat::from_xyzw(
                num(q, "x", 0.0),
                num(q, "y", 0.0),
                num(q, "z", 0.0),
                num(q, "w", 1.0),
            )
            .normalize(),
            Vec3::new(num(s, "x", 1.0), num(s, "y", 1.0), num(s, "z", 1.0)),
        ))
    };
    let parent = |e: Entity| -> Option<Entity> {
        let p = l.parent.1.decode(&ctx.get_bytes(e, l.parent.0).ok()?);
        let b = bits(&p, "entity");
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

fn run(
    ctx: &mut SystemContext<'_>,
    engine: u32,
    state: &Mutex<State>,
    assets: &Mutex<Assets>,
    l: &Ls,
) -> og_core::Result<()> {
    let time = ctx.time();
    let tick = time.tick;
    let dt = time.dt.max(1e-6);
    let ticks_of = |s: f32| -> u64 { (s.max(0.0) / dt).ceil() as u64 };
    let res = l.resource.1.decode(&ctx.resource_bytes(l.resource.0)?);
    let debris_volume = num(&res, "debrisVolume", 0.004);
    let gravity = v3(&l.physics.1.decode(&ctx.resource_bytes(l.physics.0)?)["gravity"]);
    let assets = lock(assets).clone();
    let mut guard = lock(state);
    let st = &mut *guard;
    let alive = |ctx: &SystemContext<'_>, b: u64| b != 0 && ctx.is_alive(Entity::from_bits(b));

    // Bursts past their lifetime.
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

    // The destructibles and their settings; props whose entity lost its
    // og.Destructible (or died) are forgotten, their static chunks with them.
    let mut rows = Vec::new();
    for a in ctx.query(0)? {
        rows.extend_from_slice(a.entities());
    }
    let mut settings: BTreeMap<u64, Settings> = BTreeMap::new();
    let mut current: BTreeMap<u64, (u64, Option<Pose>, bool)> = BTreeMap::new();
    for &e in &rows {
        let d = l
            .destructible
            .1
            .decode(&ctx.get_bytes(e, l.destructible.0)?);
        let Ok(mb) = ctx.get_bytes(e, l.model.0) else {
            continue;
        };
        let asset = bits(&l.model.1.decode(&mb), "asset");
        // A prop that moves (a dynamic or kinematic body) breaks free whole.
        let moving = ctx
            .get_bytes(e, l.rigid.0)
            .ok()
            .is_some_and(|b| bits(&l.rigid.1.decode(&b), "kind") != 1);
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

    // Chunks despawned by the game or a budget: gone from the graph; a
    // body takes its compound children with it.
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

    // Each prop's chunks where they stand now.
    let mut poses: BTreeMap<u64, Vec<Option<Pose>>> = BTreeMap::new();
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

    // The hits: impacts, then the calls in the order made.
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
    // Hits that break pieces off drive them on: (prop, point, the
    // striker's direction, impactRadius, impulse past the threshold, its
    // closing speed).
    let mut strikes: Vec<(u64, Vec3, Vec3, f32, f32, f32)> = Vec::new();
    for ev in ctx.events_bytes(l.collision.0)? {
        let v = l.collision.1.decode(&ev);
        let point = v3(&v["point"]);
        // The solver's impulse is that of the step that made the contact,
        // which for a fast body is a speculative one (rapier opens it
        // before they touch, and that step barely pushes): the hit is at
        // least its closing speed times the pair's reduced mass.
        let (ea, eb) = (bits(&v, "a"), bits(&v, "b"));
        let closing = v3(&v["relativeVelocity"]).dot(v3(&v["normal"])).max(0.0);
        let (reduced_a, reduced_b) = (
            mass_of(ctx, l, &st.props, &assets, &settings, ea),
            mass_of(ctx, l, &st.props, &assets, &settings, eb),
        );
        let reduced = match (reduced_a, reduced_b) {
            (Mass::Finite(a), Mass::Finite(b)) => a * b / (a + b),
            (Mass::Finite(m), Mass::Infinite) | (Mass::Infinite, Mass::Finite(m)) => m,
            _ => 0.0,
        };
        let impulse = num(&v, "impulse", 0.0).max(closing * reduced);
        let mut seen = None;
        for side in ["a", "b"] {
            let Some(&k) = owner.get(&bits(&v, side)) else {
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
                    let rel = v3(&v["relativeVelocity"]) * if side == "b" { 1.0 } else { -1.0 };
                    let along = v3(&v["normal"]) * if side == "b" { 1.0 } else { -1.0 };
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
    let mut explosions: Vec<(Vec3, f32, f32, f32)> = Vec::new();
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

    let mut events: Vec<Vec<u8>> = Vec::new();
    // Bodies made this run: their velocity, inserted at the end (after
    // the explosions' push).
    let mut made: BTreeMap<u64, (Vec3, Vec3)> = BTreeMap::new();
    let mut fresh_bodies: Vec<u64> = Vec::new();
    let keys: Vec<u64> = st.props.keys().copied().collect();
    for k in keys {
        let p = st.props.get_mut(&k).expect("listed");
        let a = assets[&p.asset].clone();
        let s = &settings[&k];
        let pose = &poses[&k];
        let frame = pose_of_frame(&p.frame);
        let scale = uniform(frame.2);
        let before = p.inst.bonds.clone();
        let anchors_before = p.inst.anchors.clone();
        let shattered_before = p.inst.shattered.clone();
        let h = hits.remove(&k).unwrap_or_default();
        let mut cause = None;
        let place_fn = |c: u32, at: Vec3| -> Vec3 {
            let ch = &a.chunks[c as usize];
            match pose[c as usize] {
                Some(ps) => place_on(&ps, ch.centroid, at),
                None => place_on(&frame, Vec3::ZERO, at),
            }
        };
        for (d, c) in &h.damage {
            if p.inst.damage(&a, d, scale, &s.mat, &place_fn) && cause.is_none() {
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
                && p.inst.chunk_damage[c]
                    >= Instance::chunk_health(&a, c as u32, scale, s.mat.toughness)
            {
                p.inst.shattered[c] = true;
                p.inst.dirty = true;
                shatter.push(c as u32);
            }
        }
        let anchors = p.anchors;
        let islands_of = |inst: &Instance| {
            let mut isl = inst.islands(&a);
            if !anchors {
                isl.iter_mut().for_each(|i| i.anchored = false);
            }
            isl
        };
        if anchors && p.inst.dirty {
            let islands = islands_of(&p.inst);
            let down = frame.1.inverse() * gravity;
            let n = p.inst.stress(
                &a,
                &islands,
                &[],
                down,
                scale,
                &s.mat,
                res["stressIterations"].as_u64().unwrap_or(0) as u32,
            );
            if n > 0 && cause.is_none() {
                cause = Some(CAUSE_STRESS);
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
            && let Some(q) = PHYSICS
                .with(|m| m.borrow().get(&engine).cloned())
                .and_then(|ph| ph.tick_query().ok())
        {
            let held = p.inst.dirty;
            for isl in islands_of(&p.inst).into_iter().filter(|i| !i.anchored) {
                let body = p.body[isl.chunks[0] as usize];
                if body == 0 || isl.chunks.iter().any(|&c| p.body[c as usize] != body) {
                    continue;
                }
                let speed = ctx
                    .get_bytes(Entity::from_bits(body), l.velocity.0)
                    .map(|vb| v3(&l.velocity.1.decode(&vb)["linear"]).length())
                    .unwrap_or(0.0);
                if speed > REST_SPEED {
                    continue;
                }
                let supports = supports_of(&q, &a, pose, &isl.chunks, body, gravity);
                if supports.is_empty() {
                    let freed = isl.chunks.iter().map(|&c| p.loose[c as usize]).min();
                    if freed.is_some_and(|t| tick.saturating_sub(t) >= WEDGED_AFTER) {
                        crumble(&a, &mut p.inst, &isl.chunks, &mut shatter);
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
                    &a,
                    std::slice::from_ref(&isl),
                    &supports,
                    root.1.inverse() * gravity,
                    scale,
                    &s.mat,
                    res["stressIterations"].as_u64().unwrap_or(0) as u32,
                );
                if n > 0 && cause.is_none() {
                    cause = Some(CAUSE_STRESS);
                }
            }
            p.inst.dirty |= held;
        }
        let broken_now: Vec<bool> = (0..before.len())
            .map(|i| before[i] > 0.0 && p.inst.bonds[i] <= 0.0)
            .collect();
        let broke = |i: usize| broken_now[i];
        let changed = (0..before.len()).any(broke)
            || (0..anchors_before.len())
                .any(|i| anchors_before[i] > 0.0 && p.inst.anchors[i] <= 0.0)
            || p.inst.shattered != shattered_before;
        if !changed {
            continue;
        }
        let cause = cause.unwrap_or(if shatter.is_empty() {
            CAUSE_STRESS
        } else {
            CAUSE_SHATTER
        });
        // A moving prop breaks free: its chunks become entities moving
        // with it, and it loses its own body.
        let mut inherit = (Vec3::ZERO, Vec3::ZERO, frame.0);
        if !p.placed {
            let e = Entity::from_bits(k);
            if let Ok(vb) = ctx.get_bytes(e, l.velocity.0) {
                let v = l.velocity.1.decode(&vb);
                inherit = (v3(&v["linear"]), v3(&v["angular"]), frame.0);
            }
            place(ctx, l, k, p, &a)?;
        }
        p.fractured = true;
        let islands = islands_of(&p.inst);
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
                            let v = l.velocity.1.decode(&vb);
                            let (lin, ang) = (v3(&v["linear"]), v3(&v["angular"]));
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
                        let rb = l.rigid.1.enc(None, Some(&json!({ "kind": 0 })))?;
                        ctx.insert_bytes(e, l.rigid.0, &rb)?;
                        made.insert(root_entity, (lin, ang));
                        fresh_bodies.push(root_entity);
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
            let mut at = Vec3::ZERO;
            let mut n = 0.0;
            for (b, bond) in a.bonds.iter().enumerate() {
                let touches = isl.chunks.contains(&bond.a) || isl.chunks.contains(&bond.b);
                if touches && broke(b) {
                    at += place_fn(bond.a, bond.centroid);
                    n += 1.0;
                }
            }
            let point = if n > 0.0 {
                at / n
            } else {
                centre / volume.max(1e-12)
            };
            events.push(l.fractured.1.enc(
                None,
                Some(&json!({
                    "entity": k as f64, "body": root_entity as f64,
                    "chunks": isl.chunks.len(), "point": jv(point),
                    "volume": volume, "mass": mass, "cause": cause,
                })),
            )?);
            if s.break_fx != 0 {
                bursts.push((s.break_fx, point));
            }
        }
        // Shattered chunks: their children fly, each its own body.
        for &c in &shatter {
            let ch = &a.chunks[c as usize];
            let Some(cp) = pose[c as usize] else {
                continue;
            };
            let (lin, ang) = match p.body[c as usize] {
                0 => (Vec3::ZERO, Vec3::ZERO),
                b => match ctx.get_bytes(Entity::from_bits(b), l.velocity.0) {
                    Ok(vb) => {
                        let v = l.velocity.1.decode(&vb);
                        (v3(&v["linear"]), v3(&v["angular"]))
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
                // Pieces below debrisVolume, given a debris emitter, are
                // its burst on the GPU: never bodies.
                if s.debris != 0 && v < debris_volume {
                    fine += 1;
                    continue;
                }
                let kp = (place_on(&cp, ch.centroid, kid.centroid), cp.1, cp.2);
                let ke = spawn_chunk(ctx, l, k, p, i, 2, &kp, Role::Root, 0, tick)?;
                body = if body == 0 { ke } else { body };
                made.insert(ke, (lin + ang.cross(kp.0 - cp.0), ang));
                fresh_bodies.push(ke);
            }
            events.push(l.fractured.1.enc(
                None,
                Some(&json!({
                    "entity": k as f64, "body": body as f64, "chunks": count - fine,
                    "point": jv(cp.0), "volume": volume,
                    "mass": volume * s.mat.density, "cause": CAUSE_SHATTER,
                })),
            )?);
            if s.break_fx != 0 {
                bursts.push((s.break_fx, cp.0));
            }
            if fine > 0 {
                bursts.push((s.debris, cp.0));
            }
        }
    }

    // The explosions' push on every loose body in reach.
    for &(point, inner, radius, impulse) in &explosions {
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
            let bodies = loose_bodies(ctx, l, p, a, scale, density, &poses[&k], &made, point);
            for (b, (m, at)) in bodies {
                let j = falloff.at(at);
                if j <= 0.0 || m <= 0.0 {
                    continue;
                }
                let dir = (at - point).normalize_or(Vec3::Y);
                let dv = dir * (j / m);
                match made.get_mut(&b) {
                    Some(v) => v.0 += dv,
                    None => {
                        let e = Entity::from_bits(b);
                        if let Ok(vb) = ctx.get_bytes(e, l.velocity.0) {
                            let mut v = l.velocity.1.decode(&vb);
                            v["linear"] = jv(v3(&v["linear"]) + dv);
                            let out = l.velocity.1.enc(Some(&vb), Some(&v))?;
                            ctx.set_bytes(e, l.velocity.0, &out)?;
                        }
                    }
                }
            }
        }
    }
    // A hit's or a blow's push on the pieces it broke loose: the impulse
    // past the threshold (momentum the striker gave up breaking through),
    // or the blow's, shared by falloff over them, none driven faster than
    // the striker closed.
    for &(k, point, dir, radius, amount, speed) in &strikes {
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
        let freed: Vec<(u64, f32, f32)> =
            loose_bodies(ctx, l, p, a, scale, s.mat.density, &poses[&k], &made, point)
                .into_iter()
                .filter(|(b, _)| made.contains_key(b))
                .map(|(b, (m, at))| (b, m, falloff.at(at)))
                .filter(|&(_, m, w)| m > 0.0 && w > 0.0)
                .collect();
        // One velocity for the whole freed mass, scaled by falloff: the
        // momentum is conserved and the lightest piece is not flung.
        let weighed: f32 = freed.iter().map(|&(_, m, w)| m * w).sum();
        for (b, _, w) in freed {
            let dv = (amount * w / weighed).min(speed);
            if let Some(v) = made.get_mut(&b) {
                v.0 += dir * dv;
            }
        }
    }
    for b in fresh_bodies {
        let (lin, ang) = made[&b];
        let vb = l.velocity.1.enc(
            None,
            Some(&json!({ "linear": jv(lin), "angular": jv(ang) })),
        )?;
        ctx.insert_bytes(Entity::from_bits(b), l.velocity.0, &vb)?;
    }

    // Budgets: loose islands (a body and its children) by when they broke
    // loose; debris past its lifetime, then the oldest past maxChunks.
    let max = res["maxChunks"].as_u64().unwrap_or(256) as usize;

    let lifetime = num(&res, "debrisLifetime", 8.0);
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
    let life = ticks_of(lifetime);
    for isl in &order {
        let old =
            lifetime > 0.0 && isl.volume < debris_volume && tick >= isl.tick.saturating_add(life);
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

    // What the game reads: each destructible's written fields.
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
    // breakFx and hitFx: a one-shot burst each, despawned after fxLifetime.
    let fx_life = ticks_of(num(&res, "fxLifetime", 3.0)).max(1);
    for (i, (emitter, at)) in bursts.into_iter().take(MAX_BURSTS).enumerate() {
        let t = l
            .transform
            .1
            .enc(None, Some(&json!({ "position": jv(at) })))?;
        let seed = (tick as u32).wrapping_mul(2_654_435_761) ^ i as u32;
        let ps = l.particles.1.enc(
            None,
            Some(&json!({ "emitter": emitter as f64, "seed": seed, "startTick": tick as u32 })),
        )?;
        let e = ctx.spawn_bytes(vec![(l.transform.0, t), (l.particles.0, ps)])?;
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
    let rb = l.rigid.1.decode(&rb);
    if bits(&rb, "kind") != 0 {
        return Mass::Infinite;
    }
    let m = num(&rb, "mass", 0.0);
    if m > 0.0 {
        return Mass::Finite(m);
    }
    let Ok(cb) = ctx.get_bytes(e, l.collider.0) else {
        return Mass::Unknown;
    };
    let c = l.collider.1.decode(&cb);
    let s = world_pose(ctx, l, e).map_or(Vec3::ONE, |p| p.2.abs());
    let (r, h) = (num(&c, "radius", 0.5), num(&c, "height", 1.0));
    let pi = std::f32::consts::PI;
    let volume = match bits(&c, "shape") as u32 {
        og_physics::SHAPE_BOX => {
            let z = v3(&c["size"]);
            z.x * z.y * z.z * s.x * s.y * s.z
        }
        og_physics::SHAPE_SPHERE => 4.0 / 3.0 * pi * r * r * r * s.x * s.y * s.z,
        og_physics::SHAPE_CAPSULE => {
            (pi * r * r * h + 4.0 / 3.0 * pi * r * r * r) * s.x * s.y * s.z
        }
        og_physics::SHAPE_CYLINDER => pi * r * r * h * s.x * s.y * s.z,
        _ => return Mass::Unknown,
    };
    Mass::Finite(volume * num(&c, "density", 1000.0).max(0.0))
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
        Role::Root => parts.push((l.rigid.0, l.rigid.1.enc(None, Some(&json!({ "kind": 0 })))?)),
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

const MAGIC: &[u8; 8] = b"DESTRUC1";

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
/// each chunk's entity with the glTF node that draws it and its centre in
/// the model.
pub(crate) struct Drawn {
    pub entity: u64,
    pub chunks: Vec<(u64, u32, Vec3)>,
}

/// Engine `engine`'s fractured props.
pub(crate) fn drawn(engine: u32) -> Vec<Drawn> {
    let st = SHARED.with(|s| s.borrow().get(&engine).cloned());
    let Some(st) = st else {
        return Vec::new();
    };
    let assets = assets(engine);
    let assets = lock(&assets);
    let st = lock(&st);
    st.props
        .iter()
        .filter(|(_, p)| p.fractured)
        .filter_map(|(&k, p)| {
            let a = assets.get(&p.asset)?;
            Some(Drawn {
                entity: k,
                chunks: p
                    .entities
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| **e != 0)
                    .map(|(c, &e)| (e, a.chunks[c].node, a.chunks[c].centroid))
                    .collect(),
            })
        })
        .collect()
}

// ── the API ──────────────────────────────────────────────────────────────────

fn wire_void(r: Result<(), String>) -> String {
    match r {
        Ok(()) => ok_void(),
        Err(e) => refuse(e),
    }
}

fn record(eng: &Engine, name: &str, e: Entity) -> Option<Value> {
    let w = eng.world().ok()?;
    let id = w.id_by_name(name).ok()?;
    let bytes = w.get_bytes(e, id).ok()?;
    let layout = eng.state.borrow().layouts.get(name).cloned()?;
    Some(layout.decode(&bytes))
}

/// The og.Destructible `e` names: itself, or the one its og.Chunk is of.
fn destructible_of(eng: &Engine, e: Entity, chunks: bool) -> Result<Entity, String> {
    let w = eng.world()?;
    if !w.is_alive(e) {
        return Err(format!("entity {} is not alive", e.to_bits()));
    }
    if record(eng, DESTRUCTIBLE, e).is_some() {
        return Ok(e);
    }
    if chunks && let Some(c) = record(eng, CHUNK, e) {
        return Ok(Entity::from_bits(bits(&c, "destructible")));
    }
    Err(format!(
        "entity {} has no og.Destructible{}",
        e.to_bits(),
        if chunks { " or og.Chunk" } else { "" }
    ))
}

fn point(v: &Value, what: &str) -> Result<[f32; 3], String> {
    let p = v3(v);
    if p.is_finite() {
        Ok(p.to_array())
    } else {
        Err(format!("{what} is not finite"))
    }
}

fn radius(d: &Value, key: &str, default: f32) -> Result<f32, String> {
    let r = num(d, key, default);
    if r.is_finite() && r >= 0.0 {
        Ok(r)
    } else {
        Err(format!(
            "desc.{key} {r} is not a finite, non-negative number"
        ))
    }
}

pub fn og_destruction_damage(world: u32, entity: f64, desc: Option<String>) -> String {
    wire_void((|| {
        let eng = engine(world)?;
        let target = destructible_of(&eng, Entity::from_bits(entity as u64), true)?;
        let d = parse(desc.as_deref())?;
        let op = Op::Damage {
            entity: target.to_bits(),
            point: point(&d["point"], "desc.point")?,
            radius: radius(&d, "radius", 0.4)?.max(1e-3),
            inner: radius(&d, "innerRadius", 0.1)?,
            amount: radius(&d, "amount", 500.0)?,
            impulse: point(&d["impulse"], "desc.impulse")?,
        };
        lock(&shared(world)).ops.push(op);
        Ok(())
    })())
}

pub fn og_destruction_explode(world: u32, desc: Option<String>) -> String {
    wire_void((|| {
        let eng = engine(world)?;
        let d = parse(desc.as_deref())?;
        let op = Op::Explode {
            point: point(&d["point"], "desc.point")?,
            radius: radius(&d, "radius", 3.0)?.max(1e-3),
            inner: radius(&d, "innerRadius", 0.5)?,
            damage: radius(&d, "damage", 3000.0)?,
            impulse: radius(&d, "impulse", 200.0)?,
        };
        // A world with no destructible has nothing to break.
        if eng.world()?.systems().contains(&SYSTEM) {
            lock(&shared(world)).ops.push(op);
        }
        Ok(())
    })())
}

pub fn og_destruction_fracture(world: u32, entity: f64) -> String {
    wire_void((|| {
        let eng = engine(world)?;
        let target = destructible_of(&eng, Entity::from_bits(entity as u64), false)?;
        lock(&shared(world)).ops.push(Op::Fracture {
            entity: target.to_bits(),
        });
        Ok(())
    })())
}

pub fn og_destruction_info(world: u32, entity: f64) -> String {
    let r = (|| {
        let eng = engine(world)?;
        let e = destructible_of(&eng, Entity::from_bits(entity as u64), false)?;
        let model = record(&eng, "og.Model", e)
            .ok_or_else(|| format!("entity {} has no og.Model", e.to_bits()))?;
        let asset = bits(&model, "asset");
        let a = lock(&assets(world)).get(&asset).cloned().ok_or_else(|| {
            format!(
                "entity {}'s og.Model is not loaded, or was not cooked with a `fracture` block",
                e.to_bits()
            )
        })?;
        let inst = lock(&shared(world))
            .props
            .get(&e.to_bits())
            .filter(|p| p.asset == asset)
            .map(|p| p.inst.clone())
            .unwrap_or_else(|| Instance::new(&a));
        let total: f32 = a.bonds.iter().map(|b| b.area).sum();
        let left: f32 = a
            .bonds
            .iter()
            .zip(&inst.bonds)
            .map(|(b, h)| b.area * h)
            .sum();
        let support = a.support as usize;
        Ok::<_, String>(json!({
            "chunks": a.support,
            "children": a.chunks.len() - support,
            "bonds": a.bonds.len(),
            "intact": inst.bonds.iter().filter(|&&h| h > 0.0).count(),
            "islands": inst.islands(&a).len(),
            "anchored": (0..support).filter(|&c| inst.anchors[c] > 0.0 && !inst.shattered[c]).count(),
            "health": if total > 0.0 { left / total } else { 1.0 },
        }))
    })();
    match r {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

pub fn og_destruction_capabilities(world: u32) -> String {
    // og.Destructible.debris is a GPU particle burst: drawn where a
    // renderer runs compute passes (every renderer the engine makes:
    // og.render.capabilities' compute).
    let compute = engine(world).is_ok_and(|e| e.render.borrow().backend.is_some());
    json!({ "visualDebris": compute }).to_string()
}
