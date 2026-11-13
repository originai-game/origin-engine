//! Prefabs at run time (`docs/design.md` §2.5, G2-1): `og.load.prefab`,
//! the `og.PrefabLink` every entity placed from a prefab carries, and
//! `world.prefabOf`.
//!
//! The cook flattens a scene's instances into its entities and records,
//! per entity, the prefab and node it came from; [`crate::world::assets::spawn_scene`]
//! writes that as an `og.PrefabLink`. The component is a lazy record
//! ([`crate::engine::lazy`]): registered with the world only when the first such
//! entity spawns (or a save holding one is restored, or a system or hook
//! declares it), so a world that never places a prefab has the same type
//! table, saves and hash as before.

use crate::engine::Engine;
use og_ecs::{SchemaId, World};
use serde_json::json;

/// The component's name.
pub(crate) const LINK: &str = "og.PrefabLink";

/// Registers `og.PrefabLink` with `world` if it is not yet (a lazy
/// record, [`crate::engine::lazy`]); its id.
pub(crate) fn register(eng: &Engine, world: &mut World) -> Result<SchemaId, String> {
    crate::engine::lazy::ensure(eng, world, LINK)?;
    world.id_by_name(LINK).map_err(|e| e.reason().to_string())
}

/// `og.load.prefab`: the prefab's cooked blob (a scene blob whose
/// entities all link to it), decoded and kept; one asset per prefab,
/// shared with the links that name it.
pub(crate) fn load(eng: &Engine, path: &str) -> Result<usize, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    let index = crate::world::assets::load_scene(eng, &id)?;
    let a = eng.assets.borrow();
    match a.entries.get(index).map(|e| &e.payload) {
        Some(crate::world::assets::Payload::Scene(s)) if s.prefabs.first() == Some(&id) => {
            Ok(index)
        }
        _ => Err(format!(
            "'{id}' is not a prefab (an .ogt of type prefab); load a scene with og.load.scene"
        )),
    }
}

/// The `og.PrefabLink` value of an entity placed from prefab asset
/// `prefab` (a load index) at `node`, under instance root `root` (bits).
pub(crate) fn link_value(prefab: usize, node: u32, root: u64) -> serde_json::Value {
    json!({
        "prefab": crate::world::assets::handle(prefab) as f64,
        "node": node,
        "root": root as f64,
    })
}
