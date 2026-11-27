//! Closing a runtime bundle after its placements and groups have gone.

use super::*;
use crate::host::codec::{Kind, RecordKind};
use std::collections::BTreeSet;

pub(crate) fn release_bundle(eng: &Engine, bundle: usize) -> Result<(), String> {
    {
        let mut a = eng.assets.borrow_mut();
        a.entries[bundle].status = Status::Pending;
        a.pending_releases.insert(bundle);
    }
    if !eng.stepping.get() {
        resolve_releases(eng);
    }
    Ok(())
}

/// Presentation-only completion: no world commands, events or load journal.
pub(crate) fn resolve_releases(eng: &Engine) {
    let pending = std::mem::take(&mut eng.assets.borrow_mut().pending_releases);
    for bundle in pending {
        if let Err(reason) = close_bundle(eng, bundle) {
            eng.assets.borrow_mut().entries[bundle].status =
                Status::Failed(format!("bundle release refused: {reason}"));
        }
    }
    // The released bundles' terrains nothing names any more (none to free
    // in an engine that loaded none: the 3D world's engine module).
    if eng.render.borrow().terrain.loaded.iter().next().is_some() {
        crate::bind::terrain::free_released(eng);
    }
}

/// A record that names what a release looks for.
pub(crate) enum Naming {
    /// A live component or resource, by its schema.
    Live(String),
    /// A persisted cell's kept component, by its schema.
    Kept(String),
}

/// The first record `names` says names what a release looks for: a
/// component on a live entity, a resource, or a component a live cell
/// keeps for its next load. Decodes every row it reads, so only a release
/// runs it.
pub(crate) fn naming(
    eng: &Engine,
    world: &og_ecs::World,
    names: &dyn Fn(&Layout, &Value) -> bool,
) -> Option<Naming> {
    for layout in eng.state.borrow().layouts.values() {
        let used = match layout.kind {
            RecordKind::Component => {
                let rows = crate::present::extract::column_rows(world, layout);
                rows.iter().any(|(e, _)| {
                    let id = world.id_by_name(&layout.schema).expect("registered layout");
                    world
                        .get_bytes(Entity::from_bits(e), id)
                        .ok()
                        .is_some_and(|b| names(layout, &layout.decode_kept(b)))
                })
            }
            RecordKind::Resource => world
                .id_by_name(&layout.schema)
                .ok()
                .and_then(|id| world.resource_bytes(id).ok())
                .is_some_and(|b| names(layout, &layout.decode_kept(b))),
            RecordKind::Event => false,
        };
        if used {
            return Some(Naming::Live(layout.schema.clone()));
        }
    }
    let store = crate::world::stream::cells::store(eng)?;
    let store = crate::world::stream::cell_store::lock(&store);
    let state = eng.state.borrow();
    for (&cell, rows) in &store.kept {
        if !world.is_alive(Entity::from_bits(cell)) {
            continue;
        }
        for row in rows {
            for (name, _, bytes) in &row.components {
                if let Some(layout) = state.layouts.get(name)
                    && names(layout, &layout.decode_kept(bytes.clone()))
                {
                    return Some(Naming::Kept(name.clone()));
                }
            }
        }
    }
    None
}

fn close_bundle(eng: &Engine, bundle: usize) -> Result<(), String> {
    let (indices, geometries, materials) = {
        let a = eng.assets.borrow();
        if a.default_bundle == Some(bundle) {
            return Err("the boot bundle cannot be released".into());
        }
        if let Some((&dependent, _)) = a.required.iter().find(|(i, deps)| {
            **i != bundle
                && matches!(a.entries[**i].payload, Payload::Bundle { .. })
                && deps.contains(&bundle)
        }) {
            return Err(format!(
                "{} still requires this delivery bundle",
                a.entries[dependent].path
            ));
        }
        let mut indices = BTreeSet::from([bundle]);
        let mut geometries = BTreeSet::new();
        let mut materials = BTreeSet::new();
        for (i, e) in a
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.bundle == Some(bundle))
        {
            indices.insert(i);
            match &e.payload {
                Payload::Gltf {
                    geometries: gs,
                    materials: ms,
                    ..
                } => {
                    geometries.extend(gs);
                    materials.extend(ms);
                }
                Payload::Scene(_) | Payload::SceneWaiting { .. } | Payload::None => {}
                _ => {
                    return Err(format!(
                        "{} still owns {}; release its loads first",
                        a.entries[bundle].path, e.path
                    ));
                }
            }
        }
        (indices, geometries, materials)
    };
    crate::world::stream::check_bundle_release(eng, bundle)?;
    let world = eng.world()?;
    let handles: BTreeSet<_> = indices.iter().map(|&i| handle(i)).collect();

    // Compute which loaded effect materials, emitters, and composed VFX depend
    // on this bundle directly or via meshes/textures owned by this bundle.
    let (
        bundle_effects,
        bundle_effect_handles,
        bundle_emitters,
        bundle_emitter_handles,
        bundle_vfx,
        bundle_vfx_handles,
        candidate_meshes,
        candidate_textures,
    ) = {
        let rs = eng.render.borrow();
        let mut candidate_meshes = BTreeSet::new();
        for &i in &indices {
            let key = handle(i);
            if let Some(parts) = rs.models.get(&key) {
                candidate_meshes.extend(parts.iter().map(|(m, _, _)| *m));
            }
            if let Some(lods) = rs.model_lods.get(&key) {
                candidate_meshes.extend(lods.parts.iter().flatten().map(|(m, _)| *m));
                for (raw, placed, levels) in &lods.scatter_meshes {
                    candidate_meshes.extend([*raw, *placed]);
                    candidate_meshes.extend(levels.iter().flatten().map(|(m, _)| *m));
                }
            }
            if let Some(model) = rs.fx.models.get(&key) {
                candidate_meshes.extend(model.meshes.iter().copied());
            }
        }
        let candidate_textures: BTreeSet<_> = rs
            .decoded
            .bundles
            .iter()
            .filter(|(_, owner)| **owner == bundle)
            .map(|(&h, _)| h)
            .collect();

        let mut bundle_effects = BTreeSet::new();
        let mut bundle_effect_render = BTreeSet::new();
        for (i, &h) in rs.effects.handles.iter().enumerate() {
            let owned = rs.effects.bundles.get(&i) == Some(&bundle);
            let uses_tex = rs.scene.effect_material(h).is_some_and(|m| {
                [
                    m.main.texture,
                    m.mask.as_ref().and_then(|t| t.texture),
                    m.erosion.as_ref().and_then(|e| e.noise.texture),
                    m.flow.as_ref().and_then(|f| f.map.texture),
                ]
                .into_iter()
                .flatten()
                .any(|t| candidate_textures.contains(&t))
            });
            if owned || uses_tex {
                bundle_effects.insert(i);
                bundle_effect_render.insert(h);
            }
        }
        let bundle_effect_handles: BTreeSet<u64> =
            bundle_effects.iter().map(|&i| handle(i)).collect();

        let mut bundle_emitters = BTreeSet::new();
        let mut bundle_emitter_render = BTreeSet::new();
        loop {
            let before = bundle_emitters.len();
            for (i, &h) in rs.fx.emitters.iter().enumerate() {
                if bundle_emitters.contains(&i) {
                    continue;
                }
                let owned = rs.fx.emitter_bundles.get(&i) == Some(&bundle);
                let uses_bundle = rs.scene.emitter(h).is_some_and(|e| {
                    e.texture.is_some_and(|t| candidate_textures.contains(&t))
                        || matches!(e.shape, og_render::EmitterShape::Mesh { mesh } if candidate_meshes.contains(&mesh))
                        || matches!(e.render, og_render::ParticleRender::Mesh { mesh } if candidate_meshes.contains(&mesh))
                        || e.effect.is_some_and(|m| bundle_effect_render.contains(&m))
                        || e.sub_emitters
                            .iter()
                            .any(|s| bundle_emitter_render.contains(&s.emitter))
                });
                if owned || uses_bundle {
                    bundle_emitters.insert(i);
                    bundle_emitter_render.insert(h);
                }
            }
            if bundle_emitters.len() == before {
                break;
            }
        }
        let bundle_emitter_handles: BTreeSet<u64> =
            bundle_emitters.iter().map(|&i| handle(i)).collect();

        let mut bundle_vfx = BTreeSet::new();
        if let Some(reg) = &rs.effects.vfx {
            let loaded = crate::bind::vfx::lock(reg);
            for (i, l) in loaded.iter().enumerate() {
                let owned = l.bundle == Some(bundle);
                let uses_bundle = l.links.iter().any(|lk| match lk {
                    crate::bind::vfx::Link::Emitter(_, h) => bundle_emitter_render.contains(h),
                    crate::bind::vfx::Link::Mesh(mesh, mat) => {
                        candidate_meshes.contains(mesh) || bundle_effect_render.contains(mat)
                    }
                    crate::bind::vfx::Link::Effect(Some(mat)) => bundle_effect_render.contains(mat),
                    crate::bind::vfx::Link::Decal { texture, normal } => [texture, normal]
                        .into_iter()
                        .flatten()
                        .any(|t| candidate_textures.contains(t)),
                    _ => false,
                });
                if owned || uses_bundle {
                    bundle_vfx.insert(i);
                }
            }
        }
        let bundle_vfx_handles: BTreeSet<u64> = bundle_vfx
            .iter()
            .map(|&i| crate::bind::vfx::handle(i) as u64)
            .collect();

        (
            bundle_effects,
            bundle_effect_handles,
            bundle_emitters,
            bundle_emitter_handles,
            bundle_vfx,
            bundle_vfx_handles,
            candidate_meshes,
            candidate_textures,
        )
    };

    // A custom component or resource can keep a model/scene or one of its
    // exposed geometry/material/FX handles just as the built-in records can.
    let referenced = |layout: &Layout, row: &Value| {
        layout.fields.iter().any(|f| {
            let owned = match f.kind {
                Kind::Asset => {
                    // Font, Texture, ClipAsset, etc. use independent handle
                    // spaces even though their packed column kind is Asset.
                    let col = og_api::host::RECORDS
                        .iter()
                        .find(|r| r.schema == layout.schema)
                        .and_then(|r| r.fields.iter().find(|field| field.name == f.name))
                        .map(|field| field.column)
                        .or_else(|| (!f.column.is_empty()).then_some(f.column.as_str()));
                    match col {
                        Some("ParticleEmitter") => &bundle_emitter_handles,
                        Some("EffectMaterial") => &bundle_effect_handles,
                        Some("VfxAsset") => &bundle_vfx_handles,
                        Some(c) if c != "Asset" => return false,
                        _ => &handles,
                    }
                }
                Kind::Geometry => &geometries,
                Kind::Material => &materials,
                _ => return false,
            };
            let value = &row[&f.name];
            let contains = |v: &Value| v.as_u64().is_some_and(|h| owned.contains(&h));
            value
                .as_array()
                .map_or_else(|| contains(value), |vs| vs.iter().any(contains))
        })
    };
    match naming(eng, world, &referenced) {
        Some(Naming::Live(schema)) => {
            return Err(format!(
                "{schema} still references this bundle; despawn its placements and clear its handles before releasing it"
            ));
        }
        Some(Naming::Kept(name)) => {
            return Err(format!(
                "a persisted cell's {name} still references this bundle; discard its kept entities before releasing it"
            ));
        }
        None => {}
    }
    if crate::present::hlod::bundle_in_use(world, &eng.render.borrow(), bundle) {
        return Err("an HLOD placement still references this bundle; despawn it before releasing the bundle".into());
    }
    {
        let mut rs = eng.render.borrow_mut();
        let prepared = rs.decoded.prepared_ids();
        if !prepared.is_empty() {
            let a = eng.assets.borrow();
            let prepared: Vec<_> = match &a.entries[bundle].payload {
                Payload::Bundle { bundle, .. } => bundle
                    .manifest()
                    .outputs()
                    .filter(|o| o.ty == "og.Texture" && prepared.contains(&o.id))
                    .map(|o| o.id.clone())
                    .collect(),
                _ => Vec::new(),
            };
            let mut closed = Vec::new();
            let mut kept = Vec::new();
            for (owner, entry) in a.entries.iter().enumerate() {
                let Payload::Bundle { bundle: source, .. } = &entry.payload else {
                    continue;
                };
                for output in source
                    .manifest()
                    .outputs()
                    .filter(|o| prepared.contains(&o.id))
                {
                    let Ok(texture) = a.blob_from(&output.id, Some(owner)).and_then(|(b, s)| {
                        s.texture(&output.id, b.bytes()).map_err(|e| e.to_string())
                    }) else {
                        continue;
                    };
                    if owner == bundle {
                        closed.push(texture);
                    } else {
                        kept.push(texture);
                    }
                }
            }
            let formats = rs.backend.as_ref().and_then(|b| b.texture_formats());
            rs.decoded.forget_sources(&closed, &kept, formats.as_ref());
        }
        let textures: Vec<_> = rs
            .decoded
            .bundles
            .iter()
            .filter(|(_, owner)| **owner == bundle)
            .map(|(&h, _)| h)
            .collect();
        crate::present::hlod::release_bundle(world, &mut rs, bundle);
        let mut meshes = BTreeSet::new();
        let mut render_materials = BTreeSet::new();
        for &i in &indices {
            let key = handle(i);
            rs.decoded.hidden_models.remove(&key);
            if let Some(parts) = rs.models.remove(&key) {
                for (mesh, material, _) in parts {
                    meshes.insert(mesh);
                    render_materials.insert(material);
                }
            }
            if let Some(lods) = rs.model_lods.remove(&key) {
                meshes.extend(lods.parts.into_iter().flatten().map(|(m, _)| m));
                render_materials.extend(lods.materials.into_iter().flatten().flatten());
                for (raw, placed, levels) in lods.scatter_meshes {
                    meshes.extend([raw, placed]);
                    meshes.extend(levels.into_iter().flatten().map(|(m, _)| m));
                }
            }
            if let Some(model) = rs.fx.models.remove(&key) {
                meshes.extend(model.meshes);
            }
            if let Some(ms) = rs.fx.model_materials.remove(&key) {
                render_materials.extend(ms);
            }
            rs.wind.tops.remove(&key);
        }

        // Scrub unreferenced FX caches that belonged to or pointed into the
        // closing bundle so no dangling mesh or texture handles survive, and
        // reload paths will fetch fresh assets if the bundle is re-opened.
        for &i in &bundle_effects {
            if let Some(&h) = rs.effects.handles.get(i)
                && let Some(mut m) = rs.scene.effect_material(h).cloned()
            {
                if m.main
                    .texture
                    .is_some_and(|t| candidate_textures.contains(&t))
                {
                    m.main.texture = None;
                }
                if let Some(mask) = &mut m.mask
                    && mask
                        .texture
                        .is_some_and(|t| candidate_textures.contains(&t))
                {
                    mask.texture = None;
                }
                if let Some(er) = &mut m.erosion
                    && er
                        .noise
                        .texture
                        .is_some_and(|t| candidate_textures.contains(&t))
                {
                    er.noise.texture = None;
                }
                if let Some(fl) = &mut m.flow
                    && fl
                        .map
                        .texture
                        .is_some_and(|t| candidate_textures.contains(&t))
                {
                    fl.map.texture = None;
                }
                rs.scene.set_effect_material(h, m);
            }
            rs.effects.bundles.remove(&i);
        }
        rs.effects.paths.retain(|_, i| !bundle_effects.contains(i));

        for &i in &bundle_emitters {
            if let Some(&h) = rs.fx.emitters.get(i)
                && let Some(mut e) = rs.scene.emitter(h).cloned()
            {
                if e.texture.is_some_and(|t| candidate_textures.contains(&t)) {
                    e.texture = None;
                }
                if matches!(e.shape, og_render::EmitterShape::Mesh { mesh } if candidate_meshes.contains(&mesh))
                {
                    e.shape = og_render::EmitterShape::Point;
                }
                if matches!(e.render, og_render::ParticleRender::Mesh { mesh } if candidate_meshes.contains(&mesh))
                {
                    e.render = og_render::ParticleRender::Billboard;
                }
                rs.scene.set_emitter(h, e);
            }
            rs.fx.emitter_bundles.remove(&i);
            if let Some(host) = &rs.fx.host {
                crate::bind::particles::lock(host).emitters.remove(&i);
            }
        }
        rs.fx
            .by_path
            .retain(|k, i| !(k.starts_with("emitter:") && bundle_emitters.contains(i)));

        if let Some(reg) = &rs.effects.vfx
            && !bundle_vfx.is_empty()
        {
            let mut loaded = crate::bind::vfx::lock(reg);
            for &i in &bundle_vfx {
                if let Some(slot) = loaded.get_mut(i) {
                    *slot = std::sync::Arc::new(crate::bind::vfx::Loaded {
                        id: String::new(),
                        bundle: None,
                        vfx: slot.vfx.clone(),
                        links: vec![crate::bind::vfx::Link::None; slot.links.len()],
                        clips: vec![0.0; slot.clips.len()],
                        has_emitters: false,
                        has_sim_elements: false,
                        has_visual_elements: false,
                    });
                }
            }
        }

        for mesh in meshes {
            crate::present::geometry::forget_mesh(&mut rs, mesh);
            rs.scene.remove_mesh(mesh);
        }
        for material in render_materials {
            rs.scene.remove_material(material);
        }
        rs.decoded.forget_textures(&textures);
        for texture in textures {
            rs.scene.remove_texture(texture);
            rs.decoded.ids.remove(&texture);
            rs.decoded.bundles.remove(&texture);
        }
        rs.shared.remove(&bundle);
        rs.kept_models.forget();
    }
    {
        let mut state = eng.state.borrow_mut();
        for geometry in geometries {
            state.physics_meshes.remove(&geometry);
            state.physics_baked.remove(&geometry);
            if let Some(physics) = &state.physics {
                physics.remove_mesh(geometry);
            }
        }
    }
    crate::world::stream::forget_bundle(eng, bundle);
    let mut a = eng.assets.borrow_mut();
    a.required.remove(&bundle);
    let path = a.entries[bundle].path.clone();
    for &i in &indices {
        let e = &mut a.entries[i];
        e.status = Status::Failed(format!("{} was released (og.load.release)", e.path));
        e.payload = Payload::None;
        a.model_shadows.remove(&i);
    }
    a.by_path.retain(|key, _| {
        key != &path
            && !key.starts_with(&format!("gltf:{bundle}:"))
            && !key.starts_with(&format!("scene:{bundle}:"))
    });
    Ok(())
}
