//! Scene preloading and scene changes (`og api show scene`).
//!
//! og.scene.preload loads a scene and asks for every streaming group it
//! declares (og.stream.request, kept until og.scene.release), so its packs
//! are fetched while the current scene plays; og.scene.progress sums what
//! of them has arrived. Both are presentation: which groups are resident
//! never reaches a tick (their assets draw stand-ins until then, world §4).
//!
//! Preloading also loads what the scene spawns with, ahead of placing it
//! ([`assets::load_closure`]): its models into the renderer with their
//! pipelines started, its sub-scenes and prefabs. A scene so prepared is
//! drawn whole the frame it is placed (`ScenePreload.prepared`,
//! [`prepared`]). og.timeline prepares the scenes its timelines' `scene`
//! tracks name the same way ([`prepare`], [`release`]).
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
/// not the world's: og.stream.request is too); and og.SceneState.current
/// as this tick started, which og.scene.current answers while a system
/// runs (the world is the step's then, and a system need not declare the
/// resource; it changes only in [`before_tick`]).
///
/// Last, the scenes prepared ([`prepare`]), by asset id.
#[derive(Default)]
pub(crate) struct ModuleState(
    RefCell<BTreeMap<String, Vec<String>>>,
    RefCell<String>,
    RefCell<BTreeMap<String, Prepare>>,
);

/// Who asked for a scene to be prepared.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum By {
    /// og.scene.preload, until og.scene.release.
    Game,
    /// og.timeline, while a timeline that names it is in pre-roll or plays.
    Timeline,
}

/// A scene [`prepare`] was asked for.
#[derive(Default)]
pub(crate) struct Prepare {
    game: bool,
    timeline: bool,
    /// Why part of what it spawns with did not load (empty: all of it did).
    failed: String,
}

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

/// [`preload_id`], and what scene `id` spawns with loaded ahead (see the
/// module's head), for `by`. Asked again, it only notes who asks.
pub(crate) fn prepare(eng: &Engine, id: &str, by: By) -> Result<u64, String> {
    let h = preload_id(eng, id)?;
    let known = eng.modules.scenes.2.borrow().contains_key(id);
    let failed = match known {
        true => None,
        false => assets::index_of(h as f64).map(|i| assets::load_closure(eng, i).join("; ")),
    };
    let mut all = eng.modules.scenes.2.borrow_mut();
    let p = all.entry(id.to_string()).or_default();
    match by {
        By::Game => p.game = true,
        By::Timeline => p.timeline = true,
    }
    if let Some(failed) = failed {
        p.failed = failed;
    }
    Ok(h)
}

/// How far scene `id` is from being drawn whole the frame it is placed
/// (`ScenePreload.prepared`). Presentation only.
pub(crate) fn prepared(eng: &Engine, id: &str) -> assets::Prepared {
    use assets::Prepared;
    match eng.modules.scenes.2.borrow().get(id) {
        None => return Prepared::No("og.scene.preload was not asked for it".into()),
        Some(p) if !p.failed.is_empty() => return Prepared::No(p.failed.clone()),
        Some(_) => {}
    }
    let a = eng.assets.borrow();
    let Some(entry) = a
        .index_of_kind("scene", id, None)
        .and_then(|i| a.entries.get(i))
    else {
        return Prepared::No(format!("{id} is not loaded"));
    };
    match (&entry.status, &entry.payload) {
        (assets::Status::Failed(e), _) => Prepared::No(e.clone()),
        (_, Payload::Scene(s)) => {
            // The groups it declares, as its models' are judged.
            let own = a
                .stream
                .names_of_scene(id)
                .into_iter()
                .filter(|g| !a.stream.upgraded(g))
                .map(|g| match a.stream.arriving(&g) {
                    true => Prepared::Busy,
                    false => Prepared::No(format!("streaming group '{g}' is not loaded")),
                })
                .max_by_key(|p| matches!(p, Prepared::No(_)));
            match (own, assets::closure_prepared(&a, &eng.render.borrow(), s)) {
                (Some(no @ Prepared::No(_)), _) | (_, no @ Prepared::No(_)) => no,
                (Some(_), _) | (_, Prepared::Busy) => Prepared::Busy,
                (None, Prepared::Yes) => Prepared::Yes,
            }
        }
        // A cell's sub-scene waits for its group's pack, which a cell asks for.
        (_, Payload::SceneWaiting { group }) => Prepared::No(format!(
            "it is a cell's sub-scene, in streaming group '{group}'"
        )),
        // Its blob is on its way.
        _ => Prepared::Busy,
    }
}

/// Whether scene `id` can be placed now: its blob decoded and no file of
/// what it spawns with still on its way (asking starts those fetches). A
/// scene that never loads can: placing it reports why.
pub(crate) fn placeable(eng: &Engine, id: &str) -> bool {
    let Ok(index) = assets::load_scene(eng, id) else {
        return true;
    };
    let a = eng.assets.borrow();
    match (&a.entries[index].status, &a.entries[index].payload) {
        (assets::Status::Pending, Payload::ScenePending) => false,
        (_, Payload::Scene(s)) => !a.closure_fetching(s),
        _ => true,
    }
}

/// `by` stops asking for scene `id` to be prepared; with no one left, its
/// groups are let go as og.scene.release lets them go.
pub(crate) fn release(eng: &Engine, id: &str, by: By) -> Result<(), String> {
    {
        let mut all = eng.modules.scenes.2.borrow_mut();
        let Some(p) = all.get_mut(id) else {
            drop(all);
            // og.scene.change's own preload, never prepared.
            return match by {
                By::Game => unpreload(eng, id),
                By::Timeline => Ok(()),
            };
        };
        match by {
            By::Game => p.game = false,
            By::Timeline => p.timeline = false,
        }
        if p.game || p.timeline {
            return Ok(());
        }
        all.remove(id);
    }
    unpreload(eng, id)
}

/// Stops asking for the groups scene `id`'s preload asked for; the scene
/// in place keeps its own.
fn unpreload(eng: &Engine, id: &str) -> Result<(), String> {
    let current = current_id(eng);
    if current.as_deref() == Some(id) {
        // The scene in place keeps its groups; forget the preload alone.
        eng.modules.scenes.0.borrow_mut().remove(id);
        return Ok(());
    }
    release_id(eng, id, current.as_deref())
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
    prepare(&eng, &id, By::Game).map(|h| h as f64)
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
    let prepared = prepared(&eng, &id) == assets::Prepared::Yes;
    Ok(ScenePreload {
        scene: id,
        ready,
        prepared,
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
    release(&eng, &id, By::Game)
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
pub(crate) fn loaded(
    eng: &Engine,
    id: &str,
) -> Result<Option<(u64, Box<assets::LoadedScene>)>, String> {
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

/// The asset id of the scene in place: og.SceneState's, else the boot
/// scene's. While a system runs, the one this tick started with.
fn current_id(eng: &Engine) -> Option<String> {
    let current = match running(eng.id) {
        Some(_) => eng.modules.scenes.1.borrow().clone(),
        None => eng
            .world()
            .ok()
            .and_then(|w| state(eng, w))
            .map(|s| s.current)
            .unwrap_or_default(),
    };
    if current.is_empty() {
        return eng.state.borrow().scene_source.clone();
    }
    Some(current)
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

/// As each tick starts: a pending change put in place, and the scene in
/// place kept for og.scene.current inside this tick's systems.
pub(crate) fn before_tick(eng: &Engine, world: &mut World) {
    place(eng, world);
    let current = state(eng, world).map(|s| s.current).unwrap_or_default();
    *eng.modules.scenes.1.borrow_mut() = current;
}

/// Puts a scene og.scene.change asked for in place.
fn place(eng: &Engine, world: &mut World) {
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
            .find(|e| {
                st.names.get(&e.name).is_some_and(|&b| {
                    !going.contains(&b) && world.is_alive(og_ecs::Entity::from_bits(b))
                })
            })
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
    // Drop provenance for unloaded scene instances.
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
