//! Cooked environment maps (IDL `load.environment`, `render.Environment`):
//! a `type environment` blob decoded into og-render's `EnvironmentMapData`
//! and added to the renderer once per path; og.Environment picks the map a
//! frame is lit by (`RenderScene::environment_map`) and whether it is the
//! background.
//!
//! The blob is read like any other: from the boot bundle (the web host
//! fetched it with the bundle's other files before the engine started), or
//! from a streaming group's pack once the group is resident. A map whose
//! group is not resident yet is pending: its handle is live, the sky lights
//! the scene, and the map is added at the first tick boundary its pack is
//! there ([`resolve`]). When the group unloads, `super::stream` removes the
//! map from the renderer and makes its handle pending again (kept, with a
//! warning, while og.Environment still lights the frame with it).

use super::engine::Engine;
use super::extract::RenderState;
use og_ecs::World;
use std::collections::BTreeMap;

/// The environment maps an engine loaded.
#[derive(Default)]
pub(crate) struct EnvState {
    /// By load index: the asset id and its map once added.
    pub maps: Vec<(String, Option<og_render::EnvironmentMapHandle>)>,
    pub by_path: BTreeMap<String, usize>,
}

/// Decodes `id` and adds it to the renderer, when its bytes can be read.
fn add(eng: &Engine, id: &str) -> Result<og_render::EnvironmentMapHandle, String> {
    let map = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(id)?;
        bundle
            .environment(id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    Ok(eng.render.borrow_mut().scene.add_environment_map(map))
}

/// `og.load.environment`: the map's load index (its handle's).
pub(crate) fn load(eng: &Engine, path: &str) -> Result<usize, String> {
    let id = og_cook::runtime::asset_id(path).to_string();
    if let Some(&i) = eng.render.borrow().env.by_path.get(&id) {
        return Ok(i);
    }
    let group = eng.assets.borrow().group_of(&id);
    let handle = match group {
        // In a streaming group's pack: pending until the group is resident.
        Some(_) => add(eng, &id).ok(),
        None => Some(add(eng, &id)?),
    };
    let mut rs = eng.render.borrow_mut();
    rs.env.maps.push((id.clone(), handle));
    let i = rs.env.maps.len() - 1;
    rs.env.by_path.insert(id, i);
    Ok(i)
}

/// At a tick boundary: adds the pending maps whose pack is now resident.
pub(crate) fn resolve(eng: &Engine) {
    let pending: Vec<(usize, String)> = eng
        .render
        .borrow()
        .env
        .maps
        .iter()
        .enumerate()
        .filter(|(_, (_, h))| h.is_none())
        .map(|(i, (id, _))| (i, id.clone()))
        .collect();
    for (i, id) in pending {
        if let Ok(h) = add(eng, &id) {
            eng.render.borrow_mut().env.maps[i].1 = Some(h);
        }
    }
}

/// og.Environment into the frame: the map (and its light's strength, which
/// replaces RenderSettings.environmentIntensity), its turn, and the
/// background when asked. Runs after the sky, the day-night cycle and the
/// camera's background, which a map's background replaces.
pub(crate) fn extract(world: &World, rs: &mut RenderState) {
    rs.scene.environment_map = None;
    rs.scene.hemisphere = ([0.0; 3], [0.0; 3], 0.0);
    let Some(v) = super::extract::resource(world, rs.draw.environment.as_ref()) else {
        return;
    };
    let hi = v["hemiIntensity"]
        .as_f64()
        .map_or(0.0, |x| x as f32)
        .max(0.0);
    if hi > 0.0 {
        let s = super::fx::color(&v["hemiSky"], glam::Vec3::ONE);
        let g = super::fx::color(&v["hemiGround"], glam::Vec3::ZERO);
        rs.scene.hemisphere = (s.to_array(), g.to_array(), hi);
    }
    let map = super::fx::handle_index(&v["map"])
        .and_then(|i| rs.env.maps.get(i))
        .and_then(|(_, h)| *h);
    let Some(map) = map else {
        return;
    };
    let f = |k: &str, d: f32| v[k].as_f64().map_or(d, |x| x as f32);
    rs.scene.environment.intensity = f("intensity", 1.0).max(0.0);
    rs.scene.environment_map = Some(og_render::EnvironmentMap {
        map,
        rotation: f("rotation", 0.0).to_radians(),
        background_intensity: f("backgroundIntensity", 1.0).max(0.0),
    });
    if v["background"].as_bool().unwrap_or(false) {
        rs.scene.environment.background = og_render::Background::Sky;
    }
}
