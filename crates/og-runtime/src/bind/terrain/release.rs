//! Freeing a terrain with its runtime bundle: once the bundle it was loaded
//! from is released (og.load.release) and no record names it any more, the
//! terrain goes whole: its compressed tiles, its field, its physics height
//! field, the renderer's terrain with its GPU tiles, and the layer textures
//! and arrays of that bundle no other terrain uses.

use super::*;
use crate::host::codec::Kind;
use crate::world::assets::{Payload, naming};

/// Frees each terrain whose bundle was released and that nothing names: no
/// built-in record's TerrainData field (og.Terrain, og.TerrainScatter,
/// og.Grass, ...) and no Geometry field (a collider on its height field),
/// live or kept by a persisted cell. Runs at each tick boundary, after the
/// bundles released by then closed; a terrain still named stays and is
/// looked at again at the next boundary.
pub(crate) fn free_released(eng: &Engine) {
    let candidates: Vec<(usize, bool)> = {
        let rs = eng.render.borrow();
        let a = eng.assets.borrow();
        rs.terrain
            .loaded
            .iter()
            .filter(|(_, t)| t.bundle.is_some_and(|b| !open(&a.entries[b].payload)))
            .map(|(i, t)| (i, t.collision))
            .collect()
    };
    if candidates.is_empty() {
        return;
    }
    let Ok(world) = eng.world() else {
        return;
    };
    // The terrains an og.Terrain still draws: read as extraction reads them
    // (cheap), so a released bundle's terrain still drawn costs no scan of
    // every record each tick.
    let drawn: BTreeSet<u64> = {
        let layout = eng.render.borrow().draw.terrain.clone();
        kept_rows(world, layout.as_ref())
            .iter()
            .filter_map(|(_, v)| v["data"].as_u64())
            .collect()
    };
    for (i, collision) in candidates {
        let data = handle(i);
        if drawn.contains(&data) {
            continue;
        }
        let geometry = GEOMETRY_BASE + i as u64;
        let names = |layout: &crate::host::codec::Layout, row: &Value| {
            layout.fields.iter().any(|f| {
                let wanted = match f.kind {
                    Kind::Geometry if collision => geometry,
                    Kind::Asset if is_terrain_data(&layout.schema, &f.name) => data,
                    _ => return false,
                };
                let value = &row[&f.name];
                let is = |v: &Value| v.as_u64() == Some(wanted);
                value
                    .as_array()
                    .map_or_else(|| is(value), |vs| vs.iter().any(is))
            })
        };
        if naming(eng, world, &names).is_none() {
            free(eng, i);
        }
    }
}

/// Whether an asset entry is a bundle still open (not released).
fn open(payload: &Payload) -> bool {
    matches!(payload, Payload::Bundle { .. })
}

/// Whether a built-in record's field holds a terrain (IDL type TerrainData).
fn is_terrain_data(schema: &str, field: &str) -> bool {
    og_api::host::RECORDS
        .iter()
        .find(|r| r.schema == schema)
        .and_then(|r| r.fields.iter().find(|f| f.name == field))
        .is_some_and(|f| f.column == "TerrainData")
}

/// Frees terrain `i`: everything it holds goes, and its handle answers
/// that it was freed.
fn free(eng: &Engine, i: usize) {
    let Some(t) = eng.render.borrow_mut().terrain.loaded.take(i) else {
        return;
    };
    if t.collision {
        let geometry = GEOMETRY_BASE + i as u64;
        let mut state = eng.state.borrow_mut();
        state.physics_heightfields.remove(&geometry);
        if let Some(physics) = &state.physics {
            physics.remove_mesh(geometry);
        }
    }
    let mut rs = eng.render.borrow_mut();
    rs.terrain.by_id.retain(|_, &mut v| v != i);
    rs.terrain.drawn.remove(&i);
    rs.terrain.streamer.remove(i);
    rs.scene.remove_terrain(t.render);
    // Its layer assets from a released bundle, unless another terrain
    // uses them: a bundle still open (a shared material pack, the boot
    // bundle) keeps its own.
    let mut assets = eng.assets.borrow_mut();
    for (asset, bundle) in &t.owned {
        let Some(b) = *bundle else {
            continue;
        };
        if open(&assets.entries[b].payload)
            || rs
                .terrain
                .loaded
                .iter()
                .any(|(_, o)| o.owned.iter().any(|(a, _)| a == asset))
        {
            continue;
        }
        match asset {
            LayerAsset::Texture(id) => {
                let Some(k) = assets.by_path.remove(&format!("texture:{id}")) else {
                    continue;
                };
                rs.decoded.forget_prepared(std::slice::from_ref(id));
                if let Some(&h) = rs.textures.get(k) {
                    rs.decoded.forget_textures(&[h]);
                    rs.decoded.ids.remove(&h);
                    rs.scene.remove_texture(h);
                }
            }
            LayerAsset::Array(id) => {
                let Some(v) = assets.texture_arrays.remove(id) else {
                    continue;
                };
                let k = <crate::host::wire::TextureArrayInfo as FromJson>::from_json(&v)
                    .ok()
                    .and_then(|info| crate::world::assets::index_of(info.array as f64));
                if let Some(h) = k.and_then(|k| rs.texture_arrays.get_mut(k)?.handle.take()) {
                    rs.scene.remove_texture_array(h);
                }
            }
        }
    }
}
