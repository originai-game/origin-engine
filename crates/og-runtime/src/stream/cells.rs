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
//! blob of the boot bundle (loaded with the scene that places the cell).
//!
//! What the probe's `dump_stream` lists under `cells` (when each loaded and
//! unloaded, how often) is kept beside the world, for the probe only.

use super::super::assets::{self, Payload, Placement};
use super::super::engine::Engine;
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

/// One anchor: its world position and reach.
type Anchor = (glam::Vec3, f32);

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
            let reach = world
                .get_bytes(e, id)
                .ok()
                .map(|b| al.decode(&b)["reach"].as_f64().unwrap_or(0.0) as f32)
                .unwrap_or(0.0);
            out.push((pose_of(world, &transform, e).0, reach.max(0.0)));
        }
    }
    if !out.is_empty() {
        return Ok(out);
    }
    if in_session(eng, world) {
        return Err("cells in an og.net session need an og.StreamAnchor on each player's entity: the og.Camera fallback would load each peer's cells by its own camera and desync".into());
    }
    let camera = layout(eng, "og.Camera").and_then(|l| {
        let id = world.id_by_name("og.Camera").ok()?;
        Some(l.decode(&world.resource_bytes(id).ok()?))
    });
    Ok(camera
        .map(|c| vec![(vec3(&c["position"]), 0.0)])
        .unwrap_or_default())
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

/// The probe's start-of-tick hook: loads and unloads the world's cells.
pub(crate) fn before_tick(eng: &Engine, world: &mut World) {
    if world.id_by_name(CELL).is_err() {
        return;
    }
    let list = cells(eng, world);
    if list.is_empty() {
        return;
    }
    let tick = world.time().tick;
    let anchors = match anchors(eng, world) {
        Ok(a) => a,
        Err(e) => {
            warn_once(eng, None, tick, e);
            return;
        }
    };
    let Some(transform) = layout(eng, "og.Transform") else {
        return;
    };
    for (cell, v) in list {
        let loaded = v["loaded"] == json!(true);
        let scene = v["scene"].as_f64().unwrap_or(0.0) as u64;
        let pose = pose_of(world, &transform, cell);
        let size = vec3(&v["size"]);
        let load = v["load"].as_f64().unwrap_or(0.0).max(0.0) as f32;
        let unload = match v["unload"].as_f64().unwrap_or(0.0) as f32 {
            u if u > 0.0 => u.max(load),
            _ => load * 1.25,
        };
        let reach = if loaded { unload } else { load };
        let near = anchors.iter().any(|&(p, extra)| {
            let r = reach + extra;
            distance_sq(pose, size, p) <= r * r
        });
        let want = scene != 0 && (v["pin"] == json!(true) || near);
        if want && !loaded {
            load_cell(eng, world, cell, scene, pose, tick);
        } else if !want && loaded {
            unload_cell(eng, world, cell, tick);
        }
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

fn load_cell(eng: &Engine, world: &mut World, cell: Entity, scene: u64, pose: Pose, tick: u64) {
    let desc = {
        let a = eng.assets.borrow();
        assets::index_of(scene as f64)
            .and_then(|i| a.entries.get(i))
            .and_then(|entry| match &entry.payload {
                Payload::Scene(s) => Some(s.clone()),
                _ => None,
            })
    };
    let Some(desc) = desc else {
        warn_once(
            eng,
            Some(cell),
            tick,
            format!("cell {cell}: StreamCell.scene {scene} is not a loaded scene"),
        );
        return;
    };
    match assets::spawn_placed(eng, &desc, Placement::Cell { cell, pose }) {
        Ok(spawned) => {
            let n = spawned.len() as u32;
            if let Err(e) = write(eng, world, cell, &json!({ "loaded": true })) {
                warn_once(eng, Some(cell), tick, e);
            }
            send(eng, world, cell, true, n, tick);
            let mut a = eng.assets.borrow_mut();
            let log = a.stream.cells.entry(cell.to_bits()).or_default();
            log.loaded_at = Some(tick);
            log.loads += 1;
            log.entities = n;
            log.refused = None;
        }
        Err(e) => {
            // Whatever spawned before the refusal goes again: the cell
            // stays unloaded and tries again next tick.
            despawn_members(eng, world, cell);
            warn_once(
                eng,
                Some(cell),
                tick,
                format!("cell {cell} ({}): {e}", desc.id),
            );
        }
    }
}

fn unload_cell(eng: &Engine, world: &mut World, cell: Entity, tick: u64) {
    let n = despawn_members(eng, world, cell);
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
    let Ok(member) = world.id_by_name(MEMBER) else {
        return 0;
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
    // Children by parent, each list by entity.
    let mut children: BTreeMap<u64, Vec<Entity>> = BTreeMap::new();
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
                    }
                }
            }
        }
    }
    for list in children.values_mut() {
        og_core::sort::sort_by_key(list, |e| (e.index(), e.generation()));
    }
    og_core::sort::sort_by_key(&mut members, |e| (e.index(), e.generation()));
    let mut seen: std::collections::BTreeSet<u64> = members.iter().map(|e| e.to_bits()).collect();
    let mut order = members;
    let mut i = 0;
    while i < order.len() {
        if let Some(kids) = children.get(&order[i].to_bits()) {
            for k in kids {
                if seen.insert(k.to_bits()) {
                    order.push(*k);
                }
            }
        }
        i += 1;
    }
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
            })
        })
        .collect();
    json!(out)
}
