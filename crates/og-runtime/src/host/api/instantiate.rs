//! World.instantiate: a prefab's entities placed in the world.

use super::*;

/// A ready asset `world.instantiate` places: its index and its scene
/// (none: a glTF).
pub(super) fn instantiable(
    eng: &Engine,
    asset: f64,
) -> Result<(usize, Option<og_bundle::runtime::SceneDesc>), String> {
    let i = index_of(asset).ok_or_else(|| format!("{asset} is not an asset handle"))?;
    let a = eng.assets.borrow();
    let entry = a
        .entries
        .get(i)
        .ok_or_else(|| format!("asset {asset} is not alive"))?;
    match (&entry.status, &entry.payload) {
        (Status::Pending, _) => Err(format!(
            "{} is still loading; instantiate it once og.load.status says ready",
            entry.path
        )),
        (Status::Failed(e), _) => Err(e.clone()),
        (_, Payload::Scene(s)) => Ok((i, Some((**s).clone()))),
        (_, Payload::Gltf { .. }) => Ok((i, None)),
        _ => Err(format!("{} is a bundle, not a scene or a glTF", entry.path)),
    }
}

/// What follows an instance's root: the scene's entities under it, or the
/// glTF's og.Model on it and its lights. Refused, the root goes with what
/// spawned.
pub(super) fn fill_instance(
    eng: &Engine,
    root: Entity,
    asset: f64,
    index: usize,
    scene: Option<&og_bundle::runtime::SceneDesc>,
) -> Result<(), String> {
    let filled = (|| match scene {
        Some(s) => assets::spawn_scene(eng, s, Some(root)).map(|_| ()),
        None => {
            let (layout, id) = component(eng, "og.Model")?;
            let fields = eng
                .assets
                .borrow()
                .model_shadows(index)
                .model_fields(asset as u64);
            let bytes = layout.encode(None, Some(&fields))?;
            before_structural(eng);
            eng.world()?.insert_bytes(root, id, &bytes).map_err(err)?;
            assets::spawn_lights(eng, index, root)
        }
    })();
    if filled.is_err() {
        let _ = og_world_despawn(eng.id, root.to_bits() as f64);
    }
    filled
}

/// An instance a system asked for, until its deferred placement runs: the
/// scene's names are held for it. Dropped unrun (the system failed, and its
/// commands were discarded with the root), the names are free again and the
/// root's own name is forgotten.
pub(super) struct Unplaced {
    pub(super) engine: u32,
    pub(super) names: Vec<String>,
    /// The root, until it is placed.
    pub(super) root: Option<Entity>,
}

impl Drop for Unplaced {
    fn drop(&mut self) {
        let Ok(eng) = engine(self.engine) else { return };
        let Ok(mut st) = eng.state.try_borrow_mut() else {
            return;
        };
        for n in &self.names {
            st.held_names.remove(n);
        }
        drop(st);
        if let Some(root) = self.root {
            forget_name(&eng, root);
        }
    }
}

/// `world.instantiate` inside a system. The root is spawned as
/// `world.spawn` spawns one there: queued, its id usable at once, with
/// og.Transform (and og.Parent, given a parent) among the system's writes.
/// What follows it is placed on the whole world when the system returns,
/// right after the root (og-ecs `SystemContext::defer`), so it needs no
/// writes of the system, and the systems after it this tick, and a
/// snapshot they take, have the scene. The scene's names are held from the
/// call on: a second instance of it in the same system is refused here,
/// as between ticks. What can still fail then (a sub-scene or a prefab
/// that does not load) takes the root with it and is a warning in the
/// engine log, since the call has answered.
pub(super) fn instantiate_in_system(
    eng: &Engine,
    f: &crate::engine::Frame,
    asset: f64,
    desc: &Value,
) -> Result<Entity, String> {
    let (index, scene) = instantiable(eng, asset)?;
    let names = match &scene {
        Some(s) => {
            let root = spawn_desc("world.instantiate", desc)?.name;
            assets::names_free(eng, s, &root)?
        }
        None => Vec::new(),
    };
    let root = spawn_in_system(eng, f, "world.instantiate", desc)?;
    eng.state
        .borrow_mut()
        .held_names
        .extend(names.iter().cloned());
    let mut unplaced = Unplaced {
        engine: eng.id,
        names,
        root: Some(root),
    };
    let id = eng.id;
    // SAFETY: the frame is published while its system runs.
    unsafe { f.ctx() }.defer(move |_| {
        // The root is in the world, and the names are the scene's to take.
        unplaced.root = None;
        drop(unplaced);
        // The step's borrow of the world is not used here: the scene goes
        // in through the engine, as it does between ticks.
        let Ok(eng) = engine(id) else { return Ok(()) };
        if let Err(e) = fill_instance(&eng, root, asset, index, scene.as_ref()) {
            let tick = eng.world().map_or(0, |w| w.time().tick);
            eng.state.borrow_mut().log.push(json!({
                "tick": tick, "warning": true,
                "text": format!("world.instantiate: {e}; its root (entity {}) is despawned", root.to_bits()),
            }));
        }
        // Into the world now, not at the next probe call: a snapshot a
        // later system takes this tick has the scene's names.
        rprobe::flush_names(&eng);
        Ok(())
    });
    Ok(root)
}

pub fn og_world_instantiate(
    world: u32,
    asset: f64,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    (|| {
        let eng = engine(world)?;
        let desc = parse(desc.as_deref())?;
        if let Some(f) = running(world) {
            return instantiate_in_system(&eng, f, asset, &desc).map(entity_number);
        }
        let (index, scene) = instantiable(&eng, asset)?;
        let root = spawn(&eng, "world.instantiate", &desc)?;
        fill_instance(&eng, root, asset, index, scene.as_ref())?;
        Ok(entity_number(root))
    })()
}
