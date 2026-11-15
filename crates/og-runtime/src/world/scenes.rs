//! Scene preloading and scene changes (IDL `scene`, guides/loading.md §12).
//!
//! og.scene.preload loads a scene and asks for every streaming group it
//! declares (og.stream.request, kept until og.scene.release), so its packs
//! are fetched while the current scene plays; og.scene.progress sums what
//! of them has arrived. Both are presentation: which groups are resident
//! never reaches a tick (their assets draw stand-ins until then, world §4).
//!
//! og.scene.change is world state: it writes `og.SceneState.pending`, and
//! [`before_tick`] (the probe's before-tick hook, so a replay, a rollback's
//! re-simulation and a restored world all run it) puts the scene in place
//! as the next tick starts: the current scene's entities go (its root's
//! subtree; the boot scene's own entities and their descendants for the
//! first change), the new scene spawns under a new root, the state is
//! written and og.SceneChanged is sent for that tick's systems. The
//! resource and the event are lazy (`crate::engine::lazy`): a world that
//! never changes scene keeps its type table, saves and hash.

use crate::engine::{Engine, before_structural, engine, running};
use crate::host::codec::Layout;
use crate::host::wire::{ScenePreload, record};
use crate::world::assets::{self, Payload, handle};
use crate::world::stream;
use og_ecs::{Entity, World};
use serde_json::json;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

/// The resource a change writes.
pub(crate) const STATE: &str = "og.SceneState";
/// The event a change sends.
pub(crate) const CHANGED: &str = "og.SceneChanged";
/// Registered together.
pub(crate) const RECORDS: [&str; 2] = [STATE, CHANGED];

/// The groups og.scene.preload asked for, by scene asset id (the engine's,
/// not the world's: og.stream.request is too).
#[derive(Default)]
pub(crate) struct ModuleState(RefCell<BTreeMap<String, Vec<String>>>);

/// A scene's asset id from a GameDesc.scenes name or an asset id.
fn resolve(eng: &Engine, scene: &str) -> Result<String, String> {
    if let Some(path) = eng.state.borrow().scenes.get(scene) {
        return Ok(og_bundle::runtime::asset_id(path).to_string());
    }
    let id = og_bundle::runtime::asset_id(scene).to_string();
    if eng.assets.borrow().bundle_for(&id).is_ok() {
        return Ok(id);
    }
    let names: Vec<String> = eng.state.borrow().scenes.keys().cloned().collect();
    Err(if names.is_empty() {
        format!(
            "no scene '{scene}': GameDesc.scenes names none, and no cooked scene has that asset id (run og cook)"
        )
    } else {
        format!(
            "no scene '{scene}': GameDesc.scenes names {}, or give a cooked scene's asset id",
            names.join(", ")
        )
    })
}

/// The streaming groups scene `id` declares, by name.
fn groups_of(eng: &Engine, id: &str) -> Vec<String> {
    stream::scene_groups(eng, id)
        .into_iter()
        .map(|g| g.name)
        .collect()
}

/// Loads scene `id` and asks for its groups; its handle.
fn preload_id(eng: &Engine, id: &str) -> Result<u64, String> {
    let index = assets::load_scene(eng, id)?;
    let groups = groups_of(eng, id);
    for g in &groups {
        stream::og_stream_request(eng.id, g.clone())?;
    }
    eng.modules
        .scenes
        .0
        .borrow_mut()
        .insert(id.to_string(), groups);
    Ok(handle(index))
}

/// At a tick boundary, before the stream update: the groups of the scene
/// og.scene.change put in place and of one it asked for are requested (and
/// recorded, so changing away releases them). The world names both; the
/// requests are engine state a snapshot does not carry, so a save restored
/// in another process asks for the same groups before the same update as
/// the world it came from, and both send the same og.StreamEvent.
pub(crate) fn before_boundary(eng: &Engine) {
    let Some(s) = eng.world().ok().and_then(|w| state(eng, w)) else {
        return;
    };
    for id in [&s.current, &s.pending] {
        if !id.is_empty() {
            // A scene that no longer loads is reported by before_tick.
            let _ = preload_id(eng, id);
        }
    }
}

/// `og.scene.preload`.
pub fn og_scene_preload(world: u32, scene: String) -> Result<f64, String> {
    let eng = engine(world)?;
    let id = resolve(&eng, &scene)?;
    preload_id(&eng, &id).map(|h| h as f64)
}

/// `og.scene.progress`.
pub fn og_scene_progress(world: u32, scene: String) -> Result<ScenePreload, String> {
    let eng = engine(world)?;
    let id = resolve(&eng, &scene)?;
    let groups = stream::scene_groups(&eng, &id);
    let bytes: f64 = groups.iter().map(|g| g.bytes).sum();
    let received: f64 = groups.iter().map(|g| g.received).sum();
    let resident =
        |g: &crate::host::wire::StreamStatus| g.state == crate::host::wire::StreamState::Resident;
    let failed = groups
        .iter()
        .find(|g| g.state == crate::host::wire::StreamState::Failed)
        .map(|g| format!("streaming group '{}': {}", g.name, g.reason))
        .unwrap_or_default();
    let ready = groups.iter().all(resident);
    Ok(ScenePreload {
        scene: id,
        ready,
        fraction: if ready {
            1.0
        } else if bytes > 0.0 {
            (received / bytes).clamp(0.0, 1.0) as f32
        } else {
            0.0
        },
        received,
        bytes,
        waiting: groups
            .iter()
            .filter(|g| !resident(g))
            .map(|g| g.name.clone())
            .collect(),
        groups: groups.into_iter().map(|g| g.name).collect(),
        failed,
    })
}

/// Stops asking for the groups `id`'s preload asked for, but those of the
/// scene in place (`keep`).
fn release_id(eng: &Engine, id: &str, keep: Option<&str>) -> Result<(), String> {
    let Some(groups) = eng.modules.scenes.0.borrow_mut().remove(id) else {
        return Ok(());
    };
    let kept: BTreeSet<String> = keep
        .map(|k| groups_of(eng, k))
        .unwrap_or_default()
        .into_iter()
        .collect();
    for g in groups {
        if !kept.contains(&g) {
            stream::og_stream_release(eng.id, g)?;
        }
    }
    Ok(())
}

/// `og.scene.release`.
pub fn og_scene_release(world: u32, scene: String) -> Result<(), String> {
    let eng = engine(world)?;
    let id = resolve(&eng, &scene)?;
    let current = current_id(&eng);
    if current.as_deref() == Some(id.as_str()) {
        // The scene in place keeps its groups; forget the preload alone.
        eng.modules.scenes.0.borrow_mut().remove(&id);
        return Ok(());
    }
    release_id(&eng, &id, current.as_deref())
}

fn layout(eng: &Engine, name: &str) -> Result<Layout, String> {
    eng.state
        .borrow()
        .layouts
        .get(name)
        .cloned()
        .ok_or_else(|| format!("{name} has no layout"))
}

/// og.SceneState: the scenes by asset id, so a save restored in another
/// process (whose asset handles were made in another order) names the
/// same scenes.
#[derive(Clone, Debug, Default)]
struct SceneState {
    current: String,
    pending: String,
    root: u64,
    changes: u32,
}

/// The world's og.SceneState, when it is registered.
fn state(eng: &Engine, world: &World) -> Option<SceneState> {
    let id = world.id_by_name(STATE).ok()?;
    let bytes = world.resource_bytes(id).ok()?;
    let r = layout(eng, STATE)
        .ok()?
        .read::<record::SceneState>(&bytes)
        .ok()?;
    let text = |v: &serde_json::Value| v.as_str().unwrap_or_default().to_string();
    Some(SceneState {
        current: text(&r.current),
        pending: text(&r.pending),
        root: r.root,
        changes: r.changes,
    })
}

fn write_state(eng: &Engine, world: &mut World, s: &SceneState) -> Result<(), String> {
    crate::engine::lazy::ensure(eng, world, STATE)?;
    let id = world
        .id_by_name(STATE)
        .map_err(|e| e.reason().to_string())?;
    let v = json!({
        "current": s.current, "pending": s.pending,
        "root": s.root as f64, "changes": s.changes,
    });
    let bytes = layout(eng, STATE)?.encode(None, Some(&v))?;
    world
        .insert_resource_bytes(id, &bytes)
        .map_err(|e| e.reason().to_string())
}

/// Asks for scene `id` as the next tick starts (registers the resource first).
fn ask(eng: &Engine, world: &mut World, id: &str) -> Result<(), String> {
    crate::engine::lazy::ensure(eng, world, STATE)?;
    let mut s = state(eng, world).unwrap_or_default();
    s.pending = id.to_string();
    write_state(eng, world, &s)
}

/// Scene `id` loaded, with its handle: `None` for a cell's sub-scene (its
/// blob is in a streaming group's pack, which a cell loads).
fn loaded(
    eng: &Engine,
    id: &str,
) -> Result<Option<(u64, Box<og_bundle::runtime::SceneDesc>)>, String> {
    let i = assets::load_scene(eng, id)?;
    Ok(match &eng.assets.borrow().entries[i].payload {
        Payload::Scene(d) => Some((handle(i), d.clone())),
        _ => None,
    })
}

/// `og.scene.change`.
pub fn og_scene_change(world: u32, scene: String) -> Result<f64, String> {
    let eng = engine(world)?;
    let id = resolve(&eng, &scene)?;
    let h = preload_id(&eng, &id)?;
    if loaded(&eng, &id)?.is_none() {
        return Err(format!(
            "{id} is a cell's sub-scene (its blob is in a streaming group's pack): a cell loads it, og.scene.change cannot"
        ));
    }
    if let Some(f) = running(world) {
        // SAFETY: the frame is published while its system runs.
        unsafe { f.ctx() }.defer(move |w| {
            let bad = |e: String| og_core::Error::new(og_core::ErrorKind::Invalid, e);
            let eng = engine(world).map_err(bad)?;
            ask(&eng, w, &id).map_err(bad)
        });
        return Ok(h as f64);
    }
    ask(&eng, eng.world()?, &id)?;
    Ok(h as f64)
}

/// The asset id of the scene in place.
fn current_id(eng: &Engine) -> Option<String> {
    match eng.world().ok().and_then(|w| state(eng, w)) {
        Some(s) if !s.current.is_empty() => Some(s.current),
        _ => eng.state.borrow().scene_source.clone(),
    }
}

/// `og.scene.current`.
pub fn og_scene_current(world: u32) -> Option<String> {
    engine(world).ok().and_then(|eng| current_id(&eng))
}

fn warn(eng: &Engine, tick: u64, text: String) {
    let mut st = eng.state.borrow_mut();
    st.log
        .push(json!({ "tick": tick, "warning": true, "text": text }));
}

/// The entities the scene in place spawned, with their descendants,
/// children first: under its root, or the boot scene's own.
fn in_place(eng: &Engine, world: &World, s: &SceneState) -> Vec<Entity> {
    let roots: Vec<Entity> = if s.root != 0 {
        vec![Entity::from_bits(s.root)]
    } else {
        let st = eng.state.borrow();
        let boot = st.scene_source.clone();
        st.scene_instances
            .iter()
            .find(|i| i.root.is_none() && Some(&i.id) == boot.as_ref())
            .map(|i| i.entities.iter().map(|b| Entity::from_bits(*b)).collect())
            .unwrap_or_default()
    };
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for r in roots {
        if !world.is_alive(r) {
            continue;
        }
        for e in crate::host::api::with_descendants(world, r).unwrap_or_default() {
            if seen.insert(e.to_bits()) {
                out.push(e);
            }
        }
    }
    out
}

/// As each tick starts: a pending change put in place.
pub(crate) fn before_tick(eng: &Engine, world: &mut World) {
    let Some(mut s) = state(eng, world) else {
        return;
    };
    if s.pending.is_empty() {
        return;
    }
    let tick = world.time().tick;
    let pending = std::mem::take(&mut s.pending);
    // Loaded here too: a save restored in another process names the scene
    // by its asset id, whatever that process loaded.
    let (scene, desc) = match loaded(eng, &pending) {
        Ok(Some(d)) => d,
        Ok(None) => {
            warn(
                eng,
                tick,
                format!(
                    "og.scene.change: {pending} is a cell's sub-scene; the scene in place stays"
                ),
            );
            let _ = write_state(eng, world, &s);
            return;
        }
        Err(e) => {
            warn(
                eng,
                tick,
                format!("og.scene.change: {e}; the scene in place stays"),
            );
            let _ = write_state(eng, world, &s);
            return;
        }
    };
    let old = in_place(eng, world, &s);
    // Names the new scene takes must be free once the old scene is gone.
    let clash = {
        let st = eng.state.borrow();
        let going: BTreeSet<u64> = old.iter().map(|e| e.to_bits()).collect();
        desc.entities
            .iter()
            .filter(|e| !e.name.is_empty())
            .find(|e| st.names.get(&e.name).is_some_and(|b| !going.contains(b)))
            .map(|e| e.name.clone())
    };
    if let Some(name) = clash {
        warn(
            eng,
            tick,
            format!(
                "og.scene.change: {} names an entity '{name}', and an entity outside the scene in place has that name; the scene in place stays (rename one)",
                desc.id
            ),
        );
        let _ = write_state(eng, world, &s);
        return;
    }
    let previous = (!s.current.is_empty())
        .then(|| assets::load_scene(eng, &s.current).ok())
        .flatten()
        .map_or(0, handle);
    let previous_id = current_id(eng);
    if !old.is_empty() {
        before_structural(eng);
        for e in &old {
            if world.is_alive(*e) && world.despawn(*e).is_ok() {
                crate::host::api::forget_name(eng, *e);
                eng.state.borrow_mut().game_spawned.remove(&e.to_bits());
            }
        }
    }
    // The old scene's records for save_scene go with it.
    eng.state.borrow_mut().scene_instances.retain(|i| {
        i.entities
            .iter()
            .any(|b| world.is_alive(Entity::from_bits(*b)))
    });
    let spawned = crate::host::api::spawn(eng, "og.scene.change", &json!({})).and_then(|root| {
        // The root is the scene's, not one the game spawned.
        eng.state.borrow_mut().game_spawned.remove(&root.to_bits());
        assets::spawn_scene(eng, &desc, Some(root)).map(|v| (root, v))
    });
    let (root, entities) = match spawned {
        Ok(r) => r,
        Err(e) => {
            warn(eng, tick, format!("og.scene.change: {}: {e}", desc.id));
            s.current.clear();
            s.root = 0;
            let _ = write_state(eng, world, &s);
            return;
        }
    };
    s.current.clone_from(&desc.id);
    s.root = root.to_bits();
    s.changes += 1;
    if let Err(e) = write_state(eng, world, &s) {
        warn(eng, tick, format!("{STATE}: {e}"));
    }
    let sent = (|| {
        crate::engine::lazy::ensure(eng, world, CHANGED)?;
        let id = world
            .id_by_name(CHANGED)
            .map_err(|e| e.reason().to_string())?;
        let v = json!({
            "scene": scene as f64, "previous": previous as f64,
            "root": root.to_bits() as f64, "entities": entities.len(),
        });
        world
            .send_bytes(id, &layout(eng, CHANGED)?.encode(None, Some(&v))?)
            .map_err(|e| e.reason().to_string())
    })();
    if let Err(e) = sent {
        warn(eng, tick, format!("{CHANGED}: {e}"));
    }
    // The previous scene's preload goes, the new one's groups stay asked for.
    if let Some(prev) = previous_id.filter(|p| *p != desc.id) {
        let _ = release_id(eng, &prev, Some(&desc.id));
    }
}
