//! The scenes `scene` tracks have in the world, put in place between ticks.
//!
//! og.timeline's system only decides: each tick it writes what a run's
//! scene tracks have in the world at its time (`Run::due`) when that is
//! not what the run holds, and hands the scenes of a run that stopped or
//! went to `Shared::leaving`. Both are in the og.timeline state block, so
//! a snapshot taken between the decision and the tick carries them.
//!
//! [`before_tick`] (the probe's before-tick hook, as og.scene.change's: a
//! replay, a rollback's re-simulation and a restored world all run it)
//! then makes the world agree as the next tick starts: the roots of the
//! scenes that go are despawned with everything under them, and each scene
//! that comes is spawned under a new root, a child of the og.Timeline's
//! entity. So a cut from one scene to another has no tick, and no frame,
//! with neither or both.
//!
//! Placing needs the scene's files. [`gate`] holds the tick that would
//! place a scene while a file of it is still on its way (the web reads a
//! runtime bundle's on demand; natively none ever is), so every host
//! places it at the same tick. [`waiting`] is the display's: while a scene
//! about to be placed is still being prepared (its models' textures, its
//! streaming groups, its pipelines), a frame holds the clock, as a video
//! player with `wait` does for footage, and the world before the cut stays
//! on screen. Neither reaches a tick.

use super::playback::shared;
use super::{lock, note};
use crate::engine::{Engine, before_structural};
use crate::world::assets::{self, Prepared};
use crate::world::scenes;
use og_ecs::{Entity, World};
use serde_json::json;
use std::collections::BTreeSet;

/// The scenes due and not yet in the world, by asset id.
fn coming(eng: &Engine) -> BTreeSet<String> {
    let sh = shared(eng.id);
    let sh = lock(&sh);
    let mut out = BTreeSet::new();
    for run in sh.runs.values() {
        for (track, key, scene) in run.due.iter().flatten() {
            if !run.scenes.iter().any(|s| (s.0, s.1) == (*track, *key)) {
                out.insert(scene.clone());
            }
        }
    }
    out
}

/// The probe's tick gate: false holds the next tick while it would place
/// a scene a file of which has not arrived.
pub(crate) fn gate(eng: &Engine) -> bool {
    coming(eng).iter().all(|id| scenes::placeable(eng, id))
}

/// Whether a scene the next tick places is still being prepared: a
/// display frame holds the clock for it. One that will not be drawn whole
/// whatever is waited for (a group over its budget, a file that failed)
/// holds nothing.
pub(crate) fn waiting(eng: &Engine) -> bool {
    coming(eng)
        .iter()
        .any(|id| scenes::prepared(eng, id) == Prepared::Busy)
}

/// As a tick starts: the scenes that go are despawned, then those due are
/// placed.
pub(crate) fn before_tick(eng: &Engine, world: &mut World) {
    let sh = shared(eng.id);
    // What goes, and what each run is due, with its run left as it will be.
    let (going, due) = {
        let mut sh = lock(&sh);
        let mut going = std::mem::take(&mut sh.leaving);
        let mut due = Vec::new();
        for (owner, run) in sh.runs.iter_mut() {
            let Some(want) = run.due.take() else { continue };
            run.scenes.retain(|s| {
                let stays = want.iter().any(|w| (w.0, w.1) == (s.0, s.1));
                if !stays && s.2 != 0 {
                    going.push(s.2);
                }
                stays
            });
            let held = &run.scenes;
            let new: Vec<_> = want
                .into_iter()
                .filter(|w| !held.iter().any(|s| (s.0, s.1) == (w.0, w.1)))
                .collect();
            if !new.is_empty() {
                due.push((*owner, new));
            }
        }
        (going, due)
    };
    if going.is_empty() && due.is_empty() {
        return;
    }
    remove(eng, world, &going);
    for (owner, new) in due {
        let mut placed = Vec::with_capacity(new.len());
        for (track, key, scene) in new {
            let root = match place(eng, world, Entity::from_bits(owner), &scene) {
                Ok(root) => root.to_bits(),
                Err(e) => {
                    note(
                        eng.id,
                        format!("og.timeline: scene {scene} is not placed: {e}"),
                    );
                    0
                }
            };
            placed.push((track, key, root));
        }
        if let Some(run) = lock(&sh).runs.get_mut(&owner) {
            run.scenes.extend(placed);
        }
    }
}

/// Despawns each live root with everything under it, children first.
fn remove(eng: &Engine, world: &mut World, roots: &[u64]) {
    let mut all = Vec::new();
    for root in roots.iter().map(|b| Entity::from_bits(*b)) {
        if world.is_alive(root) {
            all.extend(crate::host::api::with_descendants(world, root).unwrap_or_default());
        }
    }
    if all.is_empty() {
        return;
    }
    before_structural(eng);
    for e in all {
        if world.is_alive(e) && world.despawn(e).is_ok() {
            crate::host::api::forget_name(eng, e);
            eng.state.borrow_mut().game_spawned.remove(&e.to_bits());
        }
    }
    // Drop provenance for despawned scene instances.
    eng.state.borrow_mut().scene_instances.retain(|i| {
        i.entities
            .iter()
            .any(|b| world.is_alive(Entity::from_bits(*b)))
    });
}

/// Spawns scene `id` under a new root, a child of `owner`; the root.
fn place(eng: &Engine, world: &mut World, owner: Entity, id: &str) -> Result<Entity, String> {
    if !world.is_alive(owner) {
        return Err("its og.Timeline's entity is gone".into());
    }
    let Some((_, scene)) = scenes::loaded(eng, id)? else {
        return Err(
            "it is a cell's sub-scene (its blob is in a streaming group's pack), which a cell loads"
                .into(),
        );
    };
    let desc = json!({ "parent": owner.to_bits() as f64 });
    let root = crate::host::api::spawn(eng, "og.timeline", &desc)?;
    // The root is the scene's, not one the game spawned.
    eng.state.borrow_mut().game_spawned.remove(&root.to_bits());
    if let Err(e) = assets::spawn_scene(eng, &scene, Some(root)) {
        remove(eng, world, &[root.to_bits()]);
        return Err(e);
    }
    Ok(root)
}
