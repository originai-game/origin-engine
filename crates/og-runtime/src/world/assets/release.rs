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
    // A custom component or resource can keep a model/scene or one of its
    // exposed geometry/material handles just as the built-in records can.
    let referenced = |layout: &Layout, row: &Value| {
        layout.fields.iter().any(|f| {
            let owned = match f.kind {
                Kind::Asset => {
                    // Font, Texture, ClipAsset, etc. use independent handle
                    // spaces even though their packed column kind is Asset.
                    if og_api::host::RECORDS
                        .iter()
                        .find(|r| r.schema == layout.schema)
                        .and_then(|r| r.fields.iter().find(|field| field.name == f.name))
                        .is_some_and(|field| field.column != "Asset")
                    {
                        return false;
                    }
                    &handles
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
                #[cfg(not(feature = "profile2d"))]
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
            #[cfg(feature = "physics3d")]
            state.physics_baked.remove(&geometry);
            if let Some(physics) = &state.physics {
                physics.remove_mesh(geometry);
            }
        }
    }
    crate::world::stream::forget_bundle(eng, bundle);
    let mut a = eng.assets.borrow_mut();
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
