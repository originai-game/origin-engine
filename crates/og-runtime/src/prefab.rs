//! Prefabs at run time (`docs/design.md` §2.5, G2-1): `og.load.prefab`,
//! the `og.PrefabLink` every entity placed from a prefab carries, and
//! `world.prefabOf`.
//!
//! The cook flattens a scene's instances into its entities and records,
//! per entity, the prefab and node it came from; [`super::assets::spawn_scene`]
//! writes that as an `og.PrefabLink`. The component is registered with
//! the world only when the first such entity spawns (or a save holding one
//! is restored, or a system or hook declares it), so a world that never
//! places a prefab has the same type table, saves and hash as before.

use super::engine::Engine;
use og_ecs::{SchemaId, World};
use serde_json::json;

/// The component's name.
pub(crate) const LINK: &str = "og.PrefabLink";

/// Registers `og.PrefabLink` with `world` if it is not yet, from its
/// layout; its id.
pub(crate) fn register(eng: &Engine, world: &mut World) -> Result<SchemaId, String> {
    if let Ok(id) = world.id_by_name(LINK) {
        return Ok(id);
    }
    let layout = eng
        .state
        .borrow()
        .layouts
        .get(LINK)
        .cloned()
        .ok_or_else(|| format!("{LINK} has no layout"))?;
    let id = world
        .register_schema(layout.schema())
        .map_err(|e| e.reason().to_string())?;
    let mut st = eng.state.borrow_mut();
    // The runtime's table of names by id mirrors the registry's order.
    if st.names_by_id.len() == id.index() as usize {
        st.names_by_id.push(LINK.to_string());
    }
    Ok(id)
}

/// Before a restore: registers `og.PrefabLink` when the snapshot's type
/// table names it and the world lacks it (a save of a world that placed
/// prefabs, restored in a session that has not yet).
pub(crate) fn before_restore(eng: &Engine, world: &mut World, snapshot: &[u8]) {
    if world.id_by_name(LINK).is_err()
        && og_ecs::snapshot_type_names(snapshot).is_ok_and(|n| n.iter().any(|n| n == LINK))
    {
        let _ = register(eng, world);
    }
}

/// `og.load.prefab`: the prefab's cooked blob (a scene blob whose
/// entities all link to it), decoded and kept; one asset per prefab,
/// shared with the links that name it.
pub(crate) fn load(eng: &Engine, path: &str) -> Result<usize, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    let index = super::assets::load_scene(eng, &id)?;
    let a = eng.assets.borrow();
    match a.entries.get(index).map(|e| &e.payload) {
        Some(super::assets::Payload::Scene(s)) if s.prefabs.first() == Some(&id) => Ok(index),
        _ => Err(format!(
            "'{id}' is not a prefab (an .ogt of type prefab); load a scene with og.load.scene"
        )),
    }
}

/// The `og.PrefabLink` value of an entity placed from prefab asset
/// `prefab` (a load index) at `node`, under instance root `root` (bits).
pub(crate) fn link_value(prefab: usize, node: u32, root: u64) -> serde_json::Value {
    json!({
        "prefab": super::assets::handle(prefab) as f64,
        "node": node,
        "root": root as f64,
    })
}
