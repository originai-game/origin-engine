//! Cooked models: their lights, meshes and LODs.

use super::*;

/// A glTF's `KHR_lights_punctual` light as the entity each instance of the
/// model spawns under it: the light component and its fields, at the
/// light's node's transform in the model.
#[derive(Clone, Debug)]
pub(crate) struct ModelLight {
    pub component: &'static str,
    pub fields: Value,
    pub transform: Value,
}

/// Where a light of glTF's infinite range (no `range`) reaches zero: the
/// distance its inverse-square radiance falls to 0.01 (the engine's lights
/// always end; 10 m for intensity 1, 100 m for 100).
fn light_reach(intensity: f32) -> f32 {
    (intensity.max(0.0) / 0.01).sqrt().max(0.01)
}

/// The model's lights in the default scene, as [`ModelLight`]s: glTF's
/// point and spot lights as og.PointLight and og.SpotLight (intensity in
/// candela as the radiance at one metre, `range` as distance, the spot's
/// outer cone as its angle and `1 - inner / outer` as its penumbra, as
/// three.js GLTFLoader), a directional one as og.DirectionalLight.
fn model_lights(model: &og_bundle::runtime::CookedModel) -> Vec<ModelLight> {
    use og_bundle::format::light;
    let world = model.node_world();
    let in_scene: std::collections::BTreeSet<usize> = model
        .default_scene
        .map(|s| model.scene_nodes(s).into_iter().collect())
        .unwrap_or_default();
    model
        .lights
        .iter()
        .filter(|l| in_scene.contains(&l.node))
        .map(|l| {
            let color = json!({ "r": l.color[0], "g": l.color[1], "b": l.color[2] });
            let distance = if l.range > 0.0 {
                l.range
            } else {
                light_reach(l.intensity)
            };
            let (component, fields) = match l.kind {
                light::DIRECTIONAL => (
                    "og.DirectionalLight",
                    json!({ "color": color, "intensity": l.intensity }),
                ),
                light::SPOT => {
                    let [inner, outer] = l.cone;
                    let penumbra = if outer > 0.0 {
                        (1.0 - inner / outer).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    (
                        "og.SpotLight",
                        json!({ "color": color, "intensity": l.intensity, "distance": distance,
                                "angle": outer, "penumbra": penumbra }),
                    )
                }
                _ => (
                    "og.PointLight",
                    json!({ "color": color, "intensity": l.intensity, "distance": distance }),
                ),
            };
            let (s, r, t) = world[l.node].to_scale_rotation_translation();
            ModelLight {
                component,
                fields,
                transform: json!({
                    "position": { "x": t.x, "y": t.y, "z": t.z },
                    "quaternion": { "x": r.x, "y": r.y, "z": r.z, "w": r.w },
                    "scale": { "x": s.x, "y": s.y, "z": s.z },
                }),
            }
        })
        .collect()
}

/// Spawns the lights of glTF asset `index` (its `KHR_lights_punctual`)
/// as children of `root`, the entity drawing the model: each an entity
/// with og.Transform at its node's place in the model, og.Parent and its
/// light component, so the lights move, hide and despawn with the model.
pub(crate) fn spawn_lights(eng: &Engine, index: usize, root: Entity) -> Result<(), String> {
    let lights = match eng.assets.borrow().entries.get(index).map(|e| &e.payload) {
        Some(Payload::Gltf { lights, .. }) => lights.clone(),
        _ => return Ok(()),
    };
    for l in lights {
        let mut desc = l.transform.clone();
        desc["parent"] = json!(root.to_bits() as f64);
        let e = crate::host::api::spawn_entity(eng, "a glTF's light", &desc)?;
        let world = eng.world()?;
        let (layout, id) = crate::host::api::component(eng, l.component)?;
        let bytes = layout.encode(None, Some(&l.fields))?;
        before_structural(eng);
        world
            .insert_bytes(e, id, &bytes)
            .map_err(|e| e.reason().to_string())?;
    }
    Ok(())
}

/// `og.load.gltf`: the model's meshes, materials and textures join the
/// renderer once per asset id; the answer is the asset index.
pub(crate) fn load_gltf(eng: &Engine, path: &str) -> Result<usize, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    if let Some(&i) = eng.assets.borrow().by_path.get(&format!("gltf:{id}")) {
        return Ok(i);
    }
    add_gltf(eng, &id, None)
}

/// Adds glTF `id` to the renderer as a new asset, or `into` an existing
/// asset index (a refresh: its handle then names the new meshes).
pub(super) fn add_gltf(eng: &Engine, id: &str, into: Option<usize>) -> Result<usize, String> {
    let id = id.to_string();
    // A streamed model draws its proxy until its group is resident.
    let group = eng.assets.borrow().group_of(&id);
    let source = match &group {
        Some(_) => format!("{id}#proxy"),
        None => id.clone(),
    };
    let formats = texture_formats(eng);
    let (model, textures, impostor) = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&source)?;
        let model = bundle
            .model(&source, bytes.bytes())
            .map_err(|e| e.to_string())?;
        let texture = |id: &str| {
            let (tb, bundle) = assets.blob(id)?;
            let tex = bundle.texture(id, tb.bytes()).map_err(|e| e.to_string())?;
            match hooks().map(|h| h.texture(&tex, formats.as_ref())) {
                Some(Fetch::Ready(d)) => Ok(d),
                Some(Fetch::Pending) => Err(format!("{id}: the texture is still being transcoded")),
                Some(Fetch::Failed(e)) => Err(format!("{id}: {e}")),
                None => Err("no host is installed to decode textures".to_string()),
            }
        };
        let mut textures = Vec::with_capacity(model.textures.len());
        for t in &model.textures {
            // A texture another unstreamed model loaded (the cook writes one
            // per image and use, under one id) is shared, not decoded again.
            if group.is_none() && eng.render.borrow().shared.has_texture(&t.id) {
                textures.push(og_render::TextureSource::Rgba(
                    crate::world::stream::placeholder(),
                ));
                continue;
            }
            textures.push(texture(&t.id)?);
        }
        // The impostor's atlases: a model whose atlases are missing draws
        // its meshes at every distance.
        // The material atlas (G5) is optional: a model cooked before it
        // shades its cards by the material's factors.
        let impostor = match &model.impostor {
            Some(i) => Some((
                [texture(&i.albedo)?, texture(&i.normal)?],
                i.material.as_deref().and_then(|m| texture(m).ok()),
            )),
            None => None,
        };
        (model, textures, impostor)
    };
    let (index, _) = insert_model(eng, &id, group, model, &textures, impostor, into, true)?;
    // A prefractured model's chunks (`fracture { }`): og.destruction's.
    crate::bind::destruction::load_asset(eng, &id, index)?;
    Ok(index)
}

/// Place each primitive in the same model space used by its impostor bake,
/// and its coarser levels (`levels`: per part, each level's mesh and its
/// error at the part's scale) with it.
#[cfg(not(feature = "profile2d"))]
fn scatter_meshes(
    scene: &mut og_render::RenderScene,
    parts: &crate::present::extract::Parts,
    levels: &[Vec<(og_render::MeshHandle, f32)>],
) -> Vec<(
    og_render::MeshHandle,
    og_render::MeshHandle,
    crate::present::extract::ScatterLods,
)> {
    let mut out: Vec<(_, _, crate::present::extract::ScatterLods)> = Vec::new();
    for &(mesh, _, _) in parts {
        if out.iter().any(|&(source, ..)| source == mesh) {
            continue;
        }
        let placements: Vec<_> = parts.iter().filter(|p| p.0 == mesh).map(|p| p.2).collect();
        // A level's error where the part is placed largest.
        let mut lods = [None; 2];
        for (k, to) in lods.iter_mut().enumerate() {
            let placed = parts.iter().zip(levels).filter(|(p, _)| p.0 == mesh);
            let level: Option<Vec<_>> = placed.map(|(_, l)| l.get(k).copied()).collect();
            let Some(level) = level.filter(|l| !l.is_empty()) else {
                break;
            };
            let error = level.iter().map(|l| l.1).fold(0.0, f32::max);
            *to = Some((placed_mesh(scene, level[0].0, &placements), error));
        }
        out.push((mesh, placed_mesh(scene, mesh, &placements), lods));
    }
    out
}

/// `mesh` once at each of `placements`, as one mesh (itself where it
/// stands once, as made).
#[cfg(not(feature = "profile2d"))]
fn placed_mesh(
    scene: &mut og_render::RenderScene,
    mesh: og_render::MeshHandle,
    placements: &[glam::Mat4],
) -> og_render::MeshHandle {
    if placements == [glam::Mat4::IDENTITY] {
        return mesh;
    }
    let source = scene.mesh(mesh).expect("loaded model mesh");
    let mut placed = og_render::MeshData {
        vertices: Vec::new(),
        indices: Vec::new(),
    };
    for &transform in placements {
        let normal = glam::Mat3::from_mat4(transform).inverse().transpose();
        let mirrored = transform.determinant() < 0.0;
        let base = placed.vertices.len() as u32;
        placed.vertices.extend(source.vertices.iter().map(|v| {
            let mut v = *v;
            v.position = transform.transform_point3(v.position.into()).to_array();
            v.normal = (normal * glam::Vec3::from(v.normal))
                .normalize_or_zero()
                .to_array();
            let tangent = glam::Vec4::from(v.tangent);
            v.tangent = transform
                .transform_vector3(tangent.truncate())
                .normalize_or_zero()
                .extend(if mirrored { -tangent.w } else { tangent.w })
                .to_array();
            v
        }));
        for &[a, b, c] in source.indices.as_chunks::<3>().0 {
            let triangle = if mirrored { [a, c, b] } else { [a, b, c] };
            placed.indices.extend(triangle.map(|i| i + base));
        }
    }
    scene.add_mesh(placed)
}

/// Adds a decoded model and its textures (in [`CookedModel::textures`]
/// order) to the renderer as asset `id`: a new asset, or `into` an existing
/// index. A cooked model is `registered` under its path (og.load.gltf of it
/// answers this asset, a hot reload refreshes it); an uncooked one
/// (og.load.gltfData) is not. A streamed model (`group`) is its proxy.
/// `impostor` holds its impostor's atlases (albedo, normal, and the
/// material atlas when it was baked), when it
/// has one. Answers the asset index and the model's textures in the scene.
/// A registered, unstreamed model shares identical textures and meshes with
/// the models loaded before, and its materials, its own, draw in the
/// batches of identical ones (og_bundle's `SharedParts`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn insert_model(
    eng: &Engine,
    id: &str,
    group: Option<String>,
    model: og_bundle::runtime::CookedModel,
    textures: &[og_render::TextureSource],
    impostor: Option<(
        [og_render::TextureSource; 2],
        Option<og_render::TextureSource>,
    )>,
    into: Option<usize>,
    registered: bool,
) -> Result<(usize, Vec<og_render::TextureHandle>), String> {
    let id = id.to_string();
    let source = match &group {
        Some(_) => format!("{id}#proxy"),
        None => id.clone(),
    };
    let mut rs = eng.render.borrow_mut();
    // The meshes the model adds itself start here (the others are shared).
    let base = rs.scene.mesh_count();
    let mut prim_mesh = vec![0u32; model.primitives.len()];
    for (m, mesh) in model.meshes.iter().enumerate() {
        for p in mesh.primitives.clone() {
            if let Some(slot) = prim_mesh.get_mut(p) {
                *slot = m as u32;
            }
        }
    }
    let material_base = rs.scene.material_count();
    // Unstreamed models share identical textures and meshes with those
    // loaded before (a streamed one swaps its own stand-ins in place).
    let (parts, prim_meshes) = {
        let rs = &mut *rs;
        let shared = (registered && group.is_none()).then_some(&mut rs.shared);
        model.add_to_shared(&mut rs.scene, textures, shared)
    };
    for (h, t) in parts.textures.iter().zip(&model.textures) {
        rs.decoded.ids.insert(*h, t.id.clone());
    }
    let texture_handles = parts.textures.clone();
    #[cfg(not(feature = "profile2d"))]
    let scattered = impostor.is_some();
    // Each primitive's raycast BVH, cooked into the blob: loaded with the
    // mesh, so the first og.render.raycast builds nothing (a mesh shared
    // with a model loaded before has its own already).
    for (p, prim) in model.primitives.iter().enumerate() {
        if let Some(&h) = prim_meshes.get(p)
            && h.index() >= base
        {
            crate::present::geometry::adopt_cooked_bvh(&mut rs, h, prim)
                .map_err(|e| format!("{source}: primitive {p}: {e}"))?;
        }
    }
    // The coarser levels of detail join the scene after the full meshes (a
    // shared mesh's are shared too). A streamed model draws its proxy (one
    // level) and then its full meshes without levels.
    let lods = match group {
        Some(_) => None,
        None => {
            let levels = match registered {
                true => {
                    let rs = &mut *rs;
                    model.add_lod_levels_shared(&mut rs.scene, &prim_meshes, &mut rs.shared)
                }
                false => model.add_lod_levels(&mut rs.scene),
            };
            let prim_of = |mesh| prim_meshes.iter().position(|&m| m == mesh);
            // Each part's levels with their errors in the model's units: a
            // part's node scale is folded in.
            let model_parts = &parts.parts;
            let parts: Vec<Vec<(og_render::MeshHandle, f32)>> = model_parts
                .iter()
                .map(|&(mesh, _, local)| {
                    let s = local.to_scale_rotation_translation().0.abs().max_element();
                    prim_of(mesh)
                        .map(|p| levels[p].iter().map(|&(m, e)| (m, e * s)).collect())
                        .unwrap_or_default()
                })
                .collect();
            // The impostor: the model's last level, a card drawn from
            // its atlases as a StandardMaterial (`map` the albedo,
            // `normalMap` the normal and depth, `roughnessMap` the
            // material atlas; og-render's `impostor`).
            let impostor = match (impostor, &model.impostor) {
                (Some(([albedo, normal], atlas)), Some(i)) => {
                    let albedo = rs.scene.add_texture_source(albedo);
                    let normal = rs.scene.add_texture_source(normal);
                    let atlas = atlas.map(|a| rs.scene.add_texture_source(a));
                    let material = rs.scene.add_standard_material(og_render::StandardMaterial {
                        map: albedo.slot(),
                        normalMap: normal.slot(),
                        roughnessMap: atlas.map_or(0, |a| a.slot()),
                        alphaTest: 0.5,
                        side: 2,
                        roughness: 0.85,
                        ..Default::default()
                    });
                    let grid = i.grid.clamp(2, og_render::MAX_IMPOSTOR_GRID);
                    Some((
                        og_render::Impostor {
                            material,
                            grid,
                            center: i.center,
                            radius: i.radius,
                            material_atlas: atlas.is_some(),
                        },
                        i.error,
                    ))
                }
                _ => None,
            };
            #[cfg(not(feature = "profile2d"))]
            let scatter_meshes = match scattered {
                true => scatter_meshes(&mut rs.scene, model_parts, &parts),
                false => Vec::new(),
            };
            (parts.iter().any(|l| !l.is_empty()) || impostor.is_some()).then(|| {
                crate::present::extract::ModelLods {
                    parts,
                    center: (model.min + model.max) * 0.5,
                    radius: ((model.max - model.min) * 0.5).length().max(1e-3),
                    #[cfg(not(feature = "profile2d"))]
                    scatter_meshes,
                    impostor,
                }
            })
        }
    };
    let stand_in = group.as_ref().map(|_| {
        let meshes: Vec<_> = prim_meshes
            .iter()
            .filter_map(|&h| Some((h, rs.scene.mesh(h)?.clone())))
            .collect();
        let materials: Vec<_> = rs
            .scene
            .material_handles()
            .skip(material_base)
            .take(model.materials.len())
            .filter_map(|h| Some((h, *rs.scene.standard_material(h)?)))
            .collect();
        (meshes, materials)
    });
    let mut geometries = Vec::new();
    let mut materials = Vec::new();
    // The colliders the cook baked (`collider true`), by Geometry handle.
    let mut baked: Vec<(u64, Vec<u8>)> = Vec::new();
    for &(mesh, material, _) in &parts.parts {
        if !rs.geometries.contains(&mesh) {
            rs.geometries.push(mesh);
        }
        if !rs.materials.contains(&material) {
            rs.materials.push(material);
        }
        let g = rs.geometries.iter().position(|m| *m == mesh).unwrap_or(0);
        let m = rs
            .materials
            .iter()
            .position(|x| *x == material)
            .unwrap_or(0);
        if !geometries.contains(&handle(g)) {
            geometries.push(handle(g));
            let bake = prim_meshes
                .iter()
                .position(|&m| m == mesh)
                .and_then(|p| model.primitives[p].collider.clone());
            baked.extend(bake.map(|b| (handle(g), b)));
        }
        if !materials.contains(&handle(m)) {
            materials.push(handle(m));
        }
    }
    let mut assets = eng.assets.borrow_mut();
    let payload = Payload::Gltf {
        scenes: model.scenes.len() as u32,
        geometries: geometries.clone(),
        materials,
        variants: model.variants.clone(),
        lights: model_lights(&model),
    };
    let index = match into {
        Some(i) => {
            assets.entries[i].status = Status::Ready;
            assets.entries[i].payload = payload;
            i
        }
        None => assets.push(&id, Status::Ready, payload),
    };
    rs.models.insert(handle(index), parts.parts);
    // Its top above its origin: how high og.Model.sway's wind reaches.
    rs.wind.tops.insert(handle(index), model.max.y);
    match lods {
        Some(l) => rs.model_lods.insert(handle(index), l),
        None => rs.model_lods.remove(&handle(index)),
    };
    rs.fx.models.insert(
        handle(index),
        crate::present::fx::ModelInfo {
            id: id.clone(),
            meshes: prim_meshes,
            prim_mesh,
            part_nodes: parts.nodes,
            part_shown: parts.shown,
            part_variants: parts.variants,
        },
    );
    rs.fx.model_materials.insert(handle(index), parts.materials);
    if registered {
        assets.by_path.insert(format!("gltf:{id}"), index);
    }
    // Its geometries are physics meshes under their Geometry handles, for
    // convex and triangle-mesh colliders: built when a collider first names
    // one (modules::geometry_mesh, physics' mesh source), not copied for
    // every geometry here. A streamed model's are registered now, while they
    // are its proxy's, since physics collides with the proxy.
    let meshes: Vec<(u64, og_physics::MeshDesc)> = if group.is_some() {
        geometries
            .iter()
            .filter_map(|&h| {
                let m = *rs.geometries.get(index_of(h as f64)?)?;
                Some((h, crate::bind::modules::mesh_of(rs.scene.mesh(m)?)))
            })
            .collect()
    } else {
        Vec::new()
    };
    drop(assets);
    drop(rs);
    for (h, mesh) in meshes {
        if !eng.state.borrow().physics_meshes.contains_key(&h) {
            let _ = crate::bind::modules::set_mesh(eng, h, mesh);
        }
    }
    // Decoded now, between ticks: the step that first needs one builds
    // nothing.
    for (h, bytes) in baked {
        crate::bind::modules::set_baked_trimesh(eng, h, bytes)
            .map_err(|e| format!("{id}: geometry {h}: {e}"))?;
    }
    // Physics keeps the proxy's meshes; the renderer's take the full model
    // whenever its group is resident.
    if let (Some(group), Some((meshes, materials))) = (group, stand_in) {
        crate::world::stream::add_model(eng, &group, &id, meshes, materials);
    }
    Ok((index, texture_handles))
}
