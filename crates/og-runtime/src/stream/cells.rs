//! World cells (IDL `stream`, G3-3): entities that stream, not assets.
//!
//! A cell is an entity with an `og.StreamCell` whose `scene` is a sub-scene
//! (a scene's `scene "<id>"` line on the entity, [`super::super::assets::spawn_placed`]
//! sets it). As every tick starts (the probe's before-tick hook, so a
//! replay, a rollback's re-simulation and a restored world all run it),
//! [`before_tick`] decides from world state alone which cells are loaded:
//! the anchors (`og.StreamAnchor` entities' world positions; without any,
//! the `og.Camera` resource's, refused in an og.net session) against each
//! cell's box, with `load` and `unload` as hysteresis, and `pin`. A cell
//! that should load spawns its sub-scene ([`super::super::assets::Placement::Cell`]):
//! top-level entities as roots at the cell's pose after their own, every
//! entity with an `og.CellMember`; one that should unload despawns its
//! members and their descendants. Then it writes `loaded` and sends
//! `og.CellEvent`, delivered in the same tick.
//!
//! Order is fixed so entity ids are too: cells by entity (slot index, then
//! generation), a sub-scene's entities in its blob's order (the text's,
//! depth first), and an unload's despawns children first, by entity. The
//! same world therefore spawns the same ids on both hosts, in a replay and
//! after a restore. Nothing here reads I/O: the sub-scene is a decoded
//! blob of the boot bundle (loaded with the scene that places the cell),
//! or of its cell group's pack, which the hold gate waits for before a
//! tick that would load the cell ([`super::cell_groups`]).
//!
//! What the probe's `dump_stream` lists under `cells` (when each loaded and
//! unloaded, how often) is kept beside the world, for the probe only.

use super::super::assets::{self, Payload, Placement};
use super::super::engine::Engine;
use super::cell_store::{self, Loading, Source, lock};
use super::kept;
use og_core::math::Pose;
use og_ecs::{Entity, QueryDesc, World};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The cell component.
pub(crate) const CELL: &str = "og.StreamCell";
/// The anchor component.
pub(crate) const ANCHOR: &str = "og.StreamAnchor";
/// What every entity a cell spawned carries.
pub(crate) const MEMBER: &str = "og.CellMember";
/// The event a load or unload sends.
pub(crate) const EVENT: &str = "og.CellEvent";

/// The probe's view of one cell's history.
#[derive(Clone, Debug, Default)]
pub(crate) struct CellLog {
    pub loaded_at: Option<u64>,
    pub unloaded_at: Option<u64>,
    pub loads: u32,
    pub entities: u32,
    /// Whether the last load brought back kept entities (`persist`).
    pub kept: bool,
    /// The last refusal (a sub-scene that would not spawn), once per text.
    pub refused: Option<String>,
}

/// A record's layout from the engine's table.
fn layout(eng: &Engine, name: &str) -> Option<super::super::codec::Layout> {
    eng.state.borrow().layouts.get(name).cloned()
}

fn value(world: &World, layout: &super::super::codec::Layout, e: Entity) -> Option<Value> {
    let id = world.id_by_name(&layout.schema).ok()?;
    world.get_bytes(e, id).ok().map(|b| layout.decode(&b))
}

fn vec3(v: &Value) -> glam::Vec3 {
    let f = |k: &str| v[k].as_f64().unwrap_or(0.0) as f32;
    glam::Vec3::new(f("x"), f("y"), f("z"))
}

fn quat(v: &Value) -> glam::Quat {
    let f = |k: &str, d: f64| v[k].as_f64().unwrap_or(d) as f32;
    glam::Quat::from_xyzw(f("x", 0.0), f("y", 0.0), f("z", 0.0), f("w", 1.0))
}

fn xyz(v: glam::Vec3) -> Value {
    json!({ "x": v.x as f64, "y": v.y as f64, "z": v.z as f64 })
}

/// An entity's world pose: its og.Transform after its parents' down the
/// og.Parent chain ([`og_core::math::world_pose`], as physics and audio
/// compose it).
fn pose_of(world: &World, transform: &super::super::codec::Layout, e: Entity) -> Pose {
    let parent = layout_parent(world);
    let local = |e: Entity| -> Option<Pose> {
        let v = value(world, transform, e)?;
        Some((
            vec3(&v["position"]),
            quat(&v["quaternion"]),
            v.get("scale").map_or(glam::Vec3::ONE, vec3),
        ))
    };
    let own = local(e).unwrap_or((glam::Vec3::ZERO, glam::Quat::IDENTITY, glam::Vec3::ONE));
    og_core::math::world_pose(
        e,
        own,
        |c| {
            let (id, field_at) = parent?;
            let b = world.get_bytes(c, id).ok()?;
            let bits = u64::from_le_bytes(b.get(field_at..field_at + 8)?.try_into().ok()?);
            (bits != 0).then(|| Entity::from_bits(bits))
        },
        local,
    )
}

/// og.Parent's id and its `entity` field's byte offset (its only field).
fn layout_parent(world: &World) -> Option<(og_ecs::SchemaId, usize)> {
    world.id_by_name("og.Parent").ok().map(|id| (id, 0))
}

/// A cell's top-level entity's og.Transform value (`local`, as the scene
/// gives it; missing fields at their defaults) placed at the cell's world
/// pose: position, rotation and scale composed as a child's under the
/// cell, written as the root's own.
pub(crate) fn placed(cell: Pose, local: &Value) -> Value {
    let (cp, cq, cs) = cell;
    let p = local.get("position").map_or(glam::Vec3::ZERO, vec3);
    let q = local.get("quaternion").map_or(glam::Quat::IDENTITY, quat);
    let s = local.get("scale").map_or(glam::Vec3::ONE, vec3);
    let mut out = local.as_object().cloned().unwrap_or_default();
    let (wp, wq, ws) = (cp + cq * (cs * p), (cq * q).normalize(), cs * s);
    out.insert("position".into(), xyz(wp));
    out.insert(
        "quaternion".into(),
        json!({ "x": wq.x as f64, "y": wq.y as f64, "z": wq.z as f64, "w": wq.w as f64 }),
    );
    out.insert("scale".into(), xyz(ws));
    Value::Object(out)
}

/// Sets a cell's sub-scene handle (its scene's `scene` line), keeping its
/// other fields.
pub(crate) fn set_scene(eng: &Engine, cell: Entity, handle: u64) -> Result<(), String> {
    write(eng, eng.world()?, cell, &json!({ "scene": handle as f64 }))
}

fn write(eng: &Engine, world: &mut World, cell: Entity, fields: &Value) -> Result<(), String> {
    let l = layout(eng, CELL).ok_or("og.StreamCell has no layout")?;
    let id = world.id_by_name(CELL).map_err(|e| e.reason().to_string())?;
    let base = world
        .get_bytes(cell, id)
        .map_err(|e| e.reason().to_string())?;
    let bytes = l.encode(Some(&base), Some(fields))?;
    world
        .insert_bytes(cell, id, &bytes)
        .map_err(|e| e.reason().to_string())
}

/// One anchor: its world position, reach and declared top speed
/// (`maxSpeed`, read only to fetch cell groups ahead).
type Anchor = (glam::Vec3, f32, f32);

/// Every anchor's world position and reach, by entity; without any, the
/// og.Camera resource's position (refused in an og.net session: each
/// peer's camera is its own).
fn anchors(eng: &Engine, world: &World) -> Result<Vec<Anchor>, String> {
    let transform = layout(eng, "og.Transform").ok_or("og.Transform has no layout")?;
    let mut out = Vec::new();
    if let (Ok(id), Some(al)) = (world.id_by_name(ANCHOR), layout(eng, ANCHOR)) {
        let q = QueryDesc {
            read: vec![ANCHOR.into()],
            ..QueryDesc::default()
        };
        let mut list: Vec<Entity> = Vec::new();
        for a in world.query(&q).map_err(|e| e.reason().to_string())? {
            list.extend(a.entities().iter().copied());
        }
        og_core::sort::sort_by_key(&mut list, |e| (e.index(), e.generation()));
        for e in list {
            let v = world.get_bytes(e, id).ok().map(|b| al.decode(&b));
            let f = |k: &str| {
                v.as_ref()
                    .and_then(|v| v[k].as_f64())
                    .map_or(0.0, |x| (x as f32).max(0.0))
            };
            out.push((pose_of(world, &transform, e).0, f("reach"), f("maxSpeed")));
        }
    }
    if !out.is_empty() {
        return Ok(out);
    }
    if in_session(eng, world) {
        return Err(NEEDS_ANCHOR.into());
    }
    let camera = layout(eng, "og.Camera").and_then(|l| {
        let id = world.id_by_name("og.Camera").ok()?;
        Some(l.decode(&world.resource_bytes(id).ok()?))
    });
    Ok(camera
        .map(|c| vec![(vec3(&c["position"]), 0.0, 0.0)])
        .unwrap_or_default())
}

const NEEDS_ANCHOR: &str = "cells in an og.net session need an og.StreamAnchor on each player's entity: the og.Camera fallback would load each peer's cells by its own camera and desync";

/// Why a lockstep or rollback session cannot start on this world: it has
/// cells and no og.StreamAnchor, so they would load by each peer's own
/// camera. Asked as the session would start (og.net's gate), so it is
/// refused there and not at a tick of the session.
pub(crate) fn session_refusal(eng: &Engine, world: &World) -> Option<String> {
    if cells(eng, world).is_empty() {
        return None;
    }
    let anchored = world.id_by_name(ANCHOR).is_ok_and(|_| {
        let q = QueryDesc {
            read: vec![ANCHOR.into()],
            ..QueryDesc::default()
        };
        world
            .query(&q)
            .is_ok_and(|mut it| it.any(|a| !a.entities().is_empty()))
    });
    (!anchored).then(|| NEEDS_ANCHOR.to_string())
}

/// Whether the world runs an og.net session (og.Lockstep.enabled): world
/// state, the same on every peer.
pub(crate) fn in_session(eng: &Engine, world: &World) -> bool {
    layout(eng, "og.Lockstep")
        .and_then(|l| {
            let id = world.id_by_name("og.Lockstep").ok()?;
            Some(l.decode(&world.resource_bytes(id).ok()?)["enabled"] == json!(true))
        })
        .unwrap_or(false)
}

/// Every cell, by entity: its entity and og.StreamCell value.
fn cells(eng: &Engine, world: &World) -> Vec<(Entity, Value)> {
    let (Ok(id), Some(l)) = (world.id_by_name(CELL), layout(eng, CELL)) else {
        return Vec::new();
    };
    let q = QueryDesc {
        read: vec![CELL.into()],
        ..QueryDesc::default()
    };
    let mut list: Vec<Entity> = Vec::new();
    if let Ok(iter) = world.query(&q) {
        for a in iter {
            list.extend(a.entities().iter().copied());
        }
    }
    og_core::sort::sort_by_key(&mut list, |e| (e.index(), e.generation()));
    list.into_iter()
        .filter_map(|e| world.get_bytes(e, id).ok().map(|b| (e, l.decode(&b))))
        .collect()
}

/// The squared distance from `p` (world) to a cell's box: `size` about
/// the cell's world position, turned with it (its scale does not apply).
fn distance_sq(cell: Pose, size: glam::Vec3, p: glam::Vec3) -> f32 {
    let local = cell.1.inverse() * (p - cell.0);
    let out = (local.abs() - size.abs() * 0.5).max(glam::Vec3::ZERO);
    out.length_squared()
}

/// One cell as the tick sees it: its entity, og.StreamCell value and
/// world pose, whether a load of it is in progress, whether the anchors
/// (or `pin`) want it, and whether they would were they `ahead` seconds
/// further along at their `maxSpeed` (the fetch's question).
struct Seen {
    cell: Entity,
    v: Value,
    pose: Pose,
    loading: bool,
    want: bool,
    ahead: bool,
}

impl Seen {
    fn loaded(&self) -> bool {
        self.v["loaded"] == json!(true)
    }

    fn scene(&self) -> u64 {
        self.v["scene"].as_f64().unwrap_or(0.0) as u64
    }
}

/// Every cell, by entity, with what the anchors want of it ([`Seen`]);
/// `Err` when the anchors cannot decide (a session on the camera), `None`
/// in a world without cells.
fn seen(
    eng: &Engine,
    world: &World,
    store: &cell_store::Shared,
    ahead: f32,
) -> Option<Result<Vec<Seen>, String>> {
    if world.id_by_name(CELL).is_err() {
        return None;
    }
    let list = cells(eng, world);
    if list.is_empty() {
        return None;
    }
    let anchors = match anchors(eng, world) {
        Ok(a) => a,
        Err(e) => return Some(Err(e)),
    };
    let transform = layout(eng, "og.Transform")?;
    let mut out = Vec::with_capacity(list.len());
    for (cell, v) in list {
        let loaded = v["loaded"] == json!(true);
        let loading = lock(store).loading.contains_key(&cell.to_bits());
        let scene = v["scene"].as_f64().unwrap_or(0.0) as u64;
        let pose = pose_of(world, &transform, cell);
        let reach = reach_of(&v, loaded || loading);
        let size = vec3(&v["size"]);
        let within = |lead: f32| {
            anchors.iter().any(|&(p, extra, speed)| {
                let r = reach + extra + speed * lead;
                distance_sq(pose, size, p) <= r * r
            })
        };
        let pin = v["pin"] == json!(true);
        let want = scene != 0 && (pin || within(0.0));
        let ahead = scene != 0 && (pin || within(ahead));
        out.push(Seen {
            cell,
            v,
            pose,
            loading,
            want,
            ahead,
        });
    }
    Some(Ok(out))
}

/// The probe's start-of-tick hook: loads and unloads the world's cells.
pub(crate) fn before_tick(eng: &Engine, world: &mut World) {
    let Some(store) = store(eng) else {
        return;
    };
    let tick = world.time().tick;
    let list = match seen(eng, world, &store, 0.0) {
        None => return,
        Some(Ok(l)) => l,
        Some(Err(e)) => {
            warn_once(eng, None, tick, e);
            return;
        }
    };
    for c in list {
        let loaded = c.loaded();
        if c.want && !loaded {
            step_load(eng, world, &store, c.cell, &c.v, c.pose, tick);
        } else if !c.want && c.loading {
            abandon(eng, world, &store, c.cell);
        } else if !c.want && loaded {
            unload_cell(eng, world, &store, c.cell, &c.v, tick);
        }
    }
}

/// The first cell (by entity) this tick would load from its sub-scene
/// while the blob is still on its way in its cell group's pack: the cell
/// and the group (the hold gate's question, [`super::cell_groups::gate`]).
pub(crate) fn waiting(eng: &Engine, world: &World) -> Option<(Entity, String)> {
    let store = store(eng)?;
    let list = seen(eng, world, &store, 0.0)?.ok()?;
    let a = eng.assets.borrow();
    for c in list {
        if !c.want || c.loaded() {
            continue;
        }
        // A load from kept rows needs no blob.
        let bits = c.cell.to_bits();
        let from_kept = {
            let s = lock(&store);
            match s.loading.get(&bits) {
                Some(l) => l.source == Source::Kept,
                None => c.v["persist"] == json!(true) && s.kept.contains_key(&bits),
            }
        };
        if from_kept {
            continue;
        }
        if let Some(Payload::SceneWaiting { group }) = assets::index_of(c.scene() as f64)
            .and_then(|i| a.entries.get(i))
            .map(|e| &e.payload)
        {
            return Some((c.cell, group.clone()));
        }
    }
    None
}

/// The cell groups to fetch now: each one a cell whose sub-scene it holds
/// is loaded or loading, pinned, or wanted by an anchor `ahead` seconds
/// further along at its maxSpeed. Presentation only: what this answers
/// decides when packs are read, never what a tick does.
pub(crate) fn wanted_groups(eng: &Engine, ahead: f32) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    let (Some(store), Ok(world)) = (store(eng), eng.world()) else {
        return out;
    };
    let Some(Ok(list)) = seen(eng, world, &store, ahead) else {
        return out;
    };
    let a = eng.assets.borrow();
    for c in list {
        if !(c.ahead || c.loading || c.loaded()) {
            continue;
        }
        let Some(i) = assets::index_of(c.scene() as f64) else {
            continue;
        };
        if let Some(g) = a.stream.group_of_scene(i) {
            out.insert(g);
        }
    }
    out
}

/// How far from its box a cell is wanted: `load` while it is not loaded,
/// `unload` (at least `load`; 1.25 × load when 0) while it is or is loading.
fn reach_of(v: &Value, held: bool) -> f32 {
    let load = v["load"].as_f64().unwrap_or(0.0).max(0.0) as f32;
    if !held {
        return load;
    }
    match v["unload"].as_f64().unwrap_or(0.0) as f32 {
        u if u > 0.0 => u.max(load),
        _ => load * 1.25,
    }
}

thread_local! {
    /// Each engine's og.cells store, the one its world's block holds.
    static STORES: std::cell::RefCell<og_core::IdMap<u32, cell_store::Shared>> =
        const { std::cell::RefCell::new(og_core::IdMap::new()) };
}

/// The world's og.cells store (added with the cell's records).
fn store(eng: &Engine) -> Option<cell_store::Shared> {
    STORES.with(|s| s.borrow().get(&eng.id).cloned())
}

/// Adds the og.cells state block to `world` (engine `engine`'s, whose
/// cell records were just registered) with a store of its own, keeping
/// the one it has.
pub(crate) fn attach(engine: u32, world: &mut World) -> Result<(), String> {
    let shared = cell_store::Shared::default();
    match world.add_state_block(
        cell_store::BLOCK,
        Box::new(cell_store::Block(shared.clone())),
    ) {
        Ok(()) => {
            STORES.with(|s| s.borrow_mut().insert(engine, shared));
            Ok(())
        }
        Err(e) if e.reason().contains("already added") => Ok(()),
        Err(e) => Err(e.reason().to_string()),
    }
}

/// A warning in the engine's log, once per cell and text.
fn warn_once(eng: &Engine, cell: Option<Entity>, tick: u64, text: String) {
    let key = cell.map_or(0, Entity::to_bits);
    {
        let mut a = eng.assets.borrow_mut();
        let log = a.stream.cells.entry(key).or_default();
        if log.refused.as_deref() == Some(text.as_str()) {
            return;
        }
        log.refused = Some(text.clone());
    }
    eng.state
        .borrow_mut()
        .log
        .push(json!({ "tick": tick, "warning": true, "text": text }));
}

/// The scene a handle names, when it is a loaded one.
fn scene_of(eng: &Engine, scene: u64) -> Option<Box<og_bundle::runtime::SceneDesc>> {
    let a = eng.assets.borrow();
    assets::index_of(scene as f64)
        .and_then(|i| a.entries.get(i))
        .and_then(|entry| match &entry.payload {
            Payload::Scene(s) => Some(s.clone()),
            _ => None,
        })
}

/// Whether a scene handle names a cell group's sub-scene not yet landed.
fn waiting_scene(eng: &Engine, scene: u64) -> bool {
    let a = eng.assets.borrow();
    assets::index_of(scene as f64)
        .and_then(|i| a.entries.get(i))
        .is_some_and(|e| matches!(e.payload, Payload::SceneWaiting { .. }))
}

/// One tick of a cell's load: starts it (from its kept rows when it
/// persists and has them, else its sub-scene) or goes on with it, spawning
/// at most `budget` entities; the tick the last one spawns, the cell is
/// loaded and says so.
fn step_load(
    eng: &Engine,
    world: &mut World,
    store: &cell_store::Shared,
    cell: Entity,
    v: &Value,
    pose: Pose,
    tick: u64,
) {
    let bits = cell.to_bits();
    let budget = match v["budget"].as_u64().unwrap_or(0) {
        0 => usize::MAX,
        n => n as usize,
    };
    let persist = v["persist"] == json!(true);
    let mut progress = {
        let mut s = lock(store);
        match s.loading.get(&bits) {
            Some(l) => l.clone(),
            None => {
                let source = if persist && s.kept.contains_key(&bits) {
                    Source::Kept
                } else {
                    // Not kept (or no longer persisting): as authored.
                    s.kept.remove(&bits);
                    Source::Scene
                };
                Loading {
                    source,
                    spawned: Vec::new(),
                }
            }
        }
    };
    if progress.source == Source::Kept && progress.spawned.is_empty() {
        let rows = lock(store).kept.get(&bits).cloned().unwrap_or_default();
        if let Err(e) = kept::check(eng, world, &rows) {
            // Kept by a build whose components differ: as authored.
            warn_once(
                eng,
                Some(cell),
                tick,
                format!(
                    "cell {cell}: its kept entities cannot come back ({e}); it loads as authored"
                ),
            );
            lock(store).kept.remove(&bits);
            progress.source = Source::Scene;
        }
    }
    let mut spawned: Vec<Entity> = progress
        .spawned
        .iter()
        .map(|b| Entity::from_bits(*b))
        .collect();
    let (result, total) = match progress.source {
        Source::Scene => {
            let scene = v["scene"].as_f64().unwrap_or(0.0) as u64;
            let Some(desc) = scene_of(eng, scene) else {
                // Its cell group's pack has not landed: the hold gate
                // holds such a tick, so only a tick run without the probe
                // gets here, and the cell waits.
                if waiting_scene(eng, scene) {
                    return;
                }
                warn_once(
                    eng,
                    Some(cell),
                    tick,
                    format!("cell {cell}: StreamCell.scene {scene} is not a loaded scene"),
                );
                return;
            };
            let end = spawned.len().saturating_add(budget);
            let r = assets::spawn_part(
                eng,
                &desc,
                Placement::Cell { cell, pose },
                &mut spawned,
                end,
            )
            .map_err(|e| format!("cell {cell} ({}): {e}", desc.id));
            (r, desc.entities.len())
        }
        Source::Kept => {
            let rows = lock(store).kept.get(&bits).cloned().unwrap_or_default();
            let end = spawned.len().saturating_add(budget).min(rows.len());
            let r = kept::spawn(eng, world, &rows, &mut spawned, end)
                .map_err(|e| format!("cell {cell} (kept): {e}"));
            (r, rows.len())
        }
    };
    if let Err(e) = result {
        // Whatever spawned before the refusal goes again: the cell stays
        // unloaded and tries again next tick.
        despawn_members(eng, world, cell);
        lock(store).loading.remove(&bits);
        warn_once(eng, Some(cell), tick, e);
        return;
    }
    if spawned.len() < total {
        progress.spawned = spawned.iter().map(|e| e.to_bits()).collect();
        lock(store).loading.insert(bits, progress);
        return;
    }
    {
        let mut s = lock(store);
        s.loading.remove(&bits);
        if progress.source == Source::Kept {
            s.kept.remove(&bits);
        }
    }
    let n = spawned.len() as u32;
    if let Err(e) = write(eng, world, cell, &json!({ "loaded": true })) {
        warn_once(eng, Some(cell), tick, e);
    }
    send(eng, world, cell, true, n, tick);
    let mut a = eng.assets.borrow_mut();
    let log = a.stream.cells.entry(bits).or_default();
    log.loaded_at = Some(tick);
    log.loads += 1;
    log.entities = n;
    log.kept = progress.source == Source::Kept;
    log.refused = None;
}

/// A load that stops being wanted before it completes: what it spawned
/// goes, and nothing is sent (the cell never said it loaded).
fn abandon(eng: &Engine, world: &mut World, store: &cell_store::Shared, cell: Entity) {
    despawn_members(eng, world, cell);
    lock(store).loading.remove(&cell.to_bits());
    eng.state
        .borrow_mut()
        .scene_instances
        .retain(|i| i.root != Some(cell.to_bits()));
}

fn unload_cell(
    eng: &Engine,
    world: &mut World,
    store: &cell_store::Shared,
    cell: Entity,
    v: &Value,
    tick: u64,
) {
    let order = subtree(world, cell);
    if v["persist"] == json!(true) {
        match kept::capture(eng, world, &order) {
            Ok(rows) => {
                lock(store).kept.insert(cell.to_bits(), rows);
            }
            Err(e) => warn_once(
                eng,
                Some(cell),
                tick,
                format!(
                    "cell {cell}: its entities could not be kept ({e}); it loads as authored next time"
                ),
            ),
        }
    }
    let n = despawn(eng, world, order);
    if let Err(e) = write(eng, world, cell, &json!({ "loaded": false })) {
        warn_once(eng, Some(cell), tick, e);
    }
    send(eng, world, cell, false, n, tick);
    eng.state
        .borrow_mut()
        .scene_instances
        .retain(|i| i.root != Some(cell.to_bits()));
    let mut a = eng.assets.borrow_mut();
    let log = a.stream.cells.entry(cell.to_bits()).or_default();
    log.unloaded_at = Some(tick);
    log.entities = 0;
}

/// Despawns every entity whose og.CellMember names `cell`, and their
/// descendants, children first and by entity; how many.
fn despawn_members(eng: &Engine, world: &mut World, cell: Entity) -> u32 {
    let order = subtree(world, cell);
    despawn(eng, world, order)
}

/// The entities whose og.CellMember names `cell` and their descendants,
/// parents before their children: the members that are roots of it (whose
/// parent is not in it) by entity, each followed depth first by its
/// children, by entity.
fn subtree(world: &World, cell: Entity) -> Vec<Entity> {
    let Ok(member) = world.id_by_name(MEMBER) else {
        return Vec::new();
    };
    let bits = cell.to_bits();
    let mut members: Vec<Entity> = Vec::new();
    let q = QueryDesc {
        read: vec![MEMBER.into()],
        ..QueryDesc::default()
    };
    if let (Ok(iter), Ok(field)) = (world.query(&q), world.field_by_id(member, "cell")) {
        for a in iter {
            if let Ok(col) = a.read::<u64>(field) {
                members.extend(
                    a.entities()
                        .iter()
                        .zip(col.iter())
                        .filter(|(_, c)| **c == bits)
                        .map(|(e, _)| *e),
                );
            }
        }
    }
    if members.is_empty() {
        return members;
    }
    // Children by parent, each list by entity.
    let mut children: BTreeMap<u64, Vec<Entity>> = BTreeMap::new();
    let mut parent_of: BTreeMap<u64, u64> = BTreeMap::new();
    if let Ok(pid) = world.id_by_name("og.Parent") {
        let q = QueryDesc {
            read: vec!["og.Parent".into()],
            ..QueryDesc::default()
        };
        if let (Ok(iter), Ok(field)) = (world.query(&q), world.field_by_id(pid, "entity")) {
            for a in iter {
                if let Ok(col) = a.read::<u64>(field) {
                    for (e, p) in a.entities().iter().zip(col.iter()) {
                        children.entry(*p).or_default().push(*e);
                        parent_of.insert(e.to_bits(), *p);
                    }
                }
            }
        }
    }
    for list in children.values_mut() {
        og_core::sort::sort_by_key(list, |e| (e.index(), e.generation()));
    }
    // Every member and descendant, then the roots among them.
    let mut set: std::collections::BTreeSet<u64> = members.iter().map(|e| e.to_bits()).collect();
    let mut i = 0;
    let mut all = members.clone();
    while i < all.len() {
        if let Some(kids) = children.get(&all[i].to_bits()) {
            for k in kids {
                if set.insert(k.to_bits()) {
                    all.push(*k);
                }
            }
        }
        i += 1;
    }
    let mut roots: Vec<Entity> = all
        .iter()
        .copied()
        .filter(|e| parent_of.get(&e.to_bits()).is_none_or(|p| !set.contains(p)))
        .collect();
    og_core::sort::sort_by_key(&mut roots, |e| (e.index(), e.generation()));
    let mut order = Vec::with_capacity(all.len());
    let mut stack: Vec<Entity> = roots.into_iter().rev().collect();
    while let Some(e) = stack.pop() {
        order.push(e);
        if let Some(kids) = children.get(&e.to_bits()) {
            stack.extend(kids.iter().rev().copied());
        }
    }
    order
}

/// Despawns `order` (a [`subtree`]) children first; how many.
fn despawn(eng: &Engine, world: &mut World, mut order: Vec<Entity>) -> u32 {
    order.reverse();
    if order.is_empty() {
        return 0;
    }
    super::super::engine::before_structural(eng);
    let mut n = 0;
    for e in order {
        if world.is_alive(e) && world.despawn(e).is_ok() {
            super::super::api::forget_name(eng, e);
            n += 1;
        }
    }
    n
}

fn send(eng: &Engine, world: &mut World, cell: Entity, loaded: bool, entities: u32, tick: u64) {
    let sent = (|| {
        let l = layout(eng, EVENT).ok_or("og.CellEvent has no layout")?;
        super::super::lazy::ensure(eng, world, EVENT)?;
        let id = world
            .id_by_name(EVENT)
            .map_err(|e| e.reason().to_string())?;
        let v = json!({ "cell": cell.to_bits() as f64, "loaded": loaded, "entities": entities });
        world
            .send_bytes(id, &l.encode(None, Some(&v))?)
            .map_err(|e| e.reason().to_string())
    })();
    if let Err(e) = sent {
        warn_once(eng, Some(cell), tick, format!("{EVENT}: {e}"));
    }
}

/// `dump_stream`'s `cells`: every cell by entity, its sub-scene, whether
/// it is loaded, and when it last loaded and unloaded.
pub(crate) fn dump(eng: &Engine) -> Value {
    let Ok(world) = eng.world() else {
        return json!([]);
    };
    let list = cells(eng, world);
    let a = eng.assets.borrow();
    let out: Vec<Value> = list
        .iter()
        .map(|(e, v)| {
            let scene = v["scene"].as_f64().unwrap_or(0.0) as u64;
            let id = assets::index_of(scene as f64)
                .and_then(|i| a.entries.get(i))
                .map_or(String::new(), |entry| entry.path.clone());
            let log = a
                .stream
                .cells
                .get(&e.to_bits())
                .cloned()
                .unwrap_or_default();
            json!({
                "entity": e.to_bits() as f64,
                "name": world.name(*e),
                "scene": id,
                "loaded": v["loaded"],
                "pin": v["pin"],
                "entities": log.entities,
                "loads": log.loads,
                "loadedAt": log.loaded_at.map(|t| t as f64),
                "unloadedAt": log.unloaded_at.map(|t| t as f64),
                "refused": log.refused,
                "group": assets::index_of(scene as f64).and_then(|i| a.stream.group_of_scene(i)),
                "waiting": assets::index_of(scene as f64)
                    .and_then(|i| a.entries.get(i))
                    .is_some_and(|e| matches!(e.payload, Payload::SceneWaiting { .. })),
            })
        })
        .collect();
    json!(out)
}
