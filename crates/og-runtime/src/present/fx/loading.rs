//! The effect assets a game loads: lookup tables, emitters and particle meshes.

use super::*;

/// Reads a cooked `.cube` table and adds it to the renderer.
fn add_lut(eng: &crate::engine::Engine, id: &str) -> Result<og_render::LutHandle, String> {
    let lut = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(id)?;
        bundle.lut(id, bytes.bytes()).map_err(|e| e.to_string())?
    };
    Ok(eng.render.borrow_mut().scene.add_lut(lut))
}

/// `og.load.lut`: a cooked `.cube` table added to the renderer once. One
/// in a streaming group's pack loads once its group is resident (refused
/// before, naming the group); when the group unloads the renderer lets it
/// go (`crate::world::stream`) and PostEffects.lut grades nothing with it until
/// the group is resident again, when it comes back under the same handle.
pub(crate) fn load_lut(eng: &crate::engine::Engine, path: &str) -> Result<usize, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    let key = format!("lut:{id}");
    if let Some(&i) = eng.render.borrow().fx.by_path.get(&key) {
        return Ok(i);
    }
    let h = add_lut(eng, &id)?;
    let mut rs = eng.render.borrow_mut();
    rs.fx.luts.push(LoadedLut {
        id,
        handle: Some(h),
    });
    let i = rs.fx.luts.len() - 1;
    rs.fx.by_path.insert(key, i);
    Ok(i)
}

/// At a tick boundary: adds again the LUTs whose streaming group is
/// resident again after an unload released them.
pub(crate) fn resolve_luts(eng: &crate::engine::Engine) {
    let pending: Vec<(usize, String)> = eng
        .render
        .borrow()
        .fx
        .luts
        .iter()
        .enumerate()
        .filter(|(_, l)| l.handle.is_none())
        .map(|(i, l)| (i, l.id.clone()))
        .collect();
    for (i, id) in pending {
        if let Ok(h) = add_lut(eng, &id) {
            eng.render.borrow_mut().fx.luts[i].handle = Some(h);
        }
    }
}

/// `og.load.emitter`: a cooked emitter, its textures (or its texture
/// array), its mesh (`render mesh`) and its sub-emitters loaded with it.
pub(crate) fn load_emitter(eng: &crate::engine::Engine, path: &str) -> Result<usize, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    let key = format!("emitter:{id}");
    if let Some(&i) = eng.render.borrow().fx.by_path.get(&key) {
        return Ok(i);
    }
    if eng.render.borrow().fx.loading.contains(&id) {
        return Err(format!("{id}: a sub-emitter cycle ({id} fires itself)"));
    }
    let text = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        bundle
            .emitter_text(&id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    eng.render.borrow_mut().fx.loading.push(id.clone());
    let parsed = parse_linked(eng, &text);
    eng.render.borrow_mut().fx.loading.pop();
    let (emitter, array) = parsed.map_err(|e| format!("{id}: {e}"))?;
    let mut rs = eng.render.borrow_mut();
    let host = crate::bind::particles::shared(eng);
    rs.fx.host = Some(host.clone());
    // An emitter that sends events: the tick simulates its systems.
    let definition = (!emitter.events.is_empty()).then(|| {
        let subs = emitter
            .sub_emitters
            .iter()
            .map(|s| rs.scene.emitter(s.emitter).cloned())
            .collect();
        let mesh = match emitter.shape {
            og_render::EmitterShape::Mesh { mesh } => rs.scene.mesh(mesh),
            _ => None,
        };
        let points = og_render::shape_points(&emitter.shape, mesh);
        (emitter.clone(), subs, points)
    });
    let h = rs.scene.add_emitter(emitter);
    rs.fx.emitters.push(h);
    let i = rs.fx.emitters.len() - 1;
    if let Some((emitter, subs, points)) = definition {
        let d = crate::bind::particles::Definition {
            handle: h,
            emitter,
            subs,
            points,
        };
        crate::bind::particles::lock(&host)
            .emitters
            .insert(i, Arc::new(d));
    }
    if let Some(a) = array {
        rs.fx.emitter_arrays.insert(i, a);
    }
    rs.fx.by_path.insert(key, i);
    Ok(i)
}

/// An emitter's text with every reference loaded: the emitter and the load
/// index of the texture array it draws from.
fn parse_linked(
    eng: &crate::engine::Engine,
    text: &str,
) -> Result<(og_render::ParticleEmitter, Option<usize>), String> {
    let mut array = None;
    let emitter = og_bundle::fx::parse_emitter_linked(
        text,
        &mut og_bundle::fx::EmitterLinks {
            texture: &mut |tex| {
                let i = crate::world::assets::load_texture(eng, tex)?;
                Ok(eng.render.borrow().textures.get(i).copied())
            },
            array: &mut |a| {
                let i = crate::world::assets::load_texture_array_index(eng, a)?;
                array = Some(i);
                Ok(eng
                    .render
                    .borrow()
                    .texture_arrays
                    .get(i)
                    .and_then(|a| a.handle))
            },
            mesh: &mut |m| particle_mesh(eng, m).map(Some),
            emitter: &mut |e| {
                let i = load_emitter(eng, e)?;
                Ok(eng.render.borrow().fx.emitters.get(i).copied())
            },
            effect: &mut |id| crate::present::effect::load_effect(eng, id).map(Some),
        },
    )?;
    Ok((emitter, array))
}

/// A mesh particle's mesh: a built-in shape (made once) or a model's first
/// mesh, the model loaded with the emitter.
pub(crate) fn particle_mesh(
    eng: &crate::engine::Engine,
    mesh: og_bundle::fx::ParticleMesh<'_>,
) -> Result<MeshHandle, String> {
    use og_bundle::fx::ParticleMesh;
    match mesh {
        ParticleMesh::Builtin(name) => {
            let name = og_bundle::fx::PARTICLE_MESHES
                .iter()
                .copied()
                .find(|n| *n == name)
                .ok_or_else(|| format!("`{name}` is not a built-in particle mesh"))?;
            let mut rs = eng.render.borrow_mut();
            if let Some(&h) = rs.fx.particle_meshes.get(name) {
                return Ok(h);
            }
            let data = match name {
                "box" => og_render::shapes::cuboid(glam::Vec3::ONE),
                "sphere" => og_render::shapes::sphere(12, 8),
                _ => og_render::shapes::tetrahedron(),
            };
            let h = rs.scene.add_mesh(data);
            rs.fx.particle_meshes.insert(name.to_string(), h);
            Ok(h)
        }
        ParticleMesh::Asset(model) => {
            let i = crate::world::assets::load_gltf(eng, model)?;
            let geometry = match eng.assets.borrow().entries.get(i).map(|e| &e.payload) {
                Some(crate::world::assets::Payload::Gltf { geometries, .. }) => {
                    geometries.first().copied()
                }
                _ => None,
            }
            .ok_or_else(|| format!("{model}: the model has no mesh"))?;
            let rs = eng.render.borrow();
            crate::world::assets::index_of(geometry as f64)
                .and_then(|g| rs.geometries.get(g).copied())
                .ok_or_else(|| format!("{model}: the model's mesh is not loaded"))
        }
    }
}
