//! `og export gltf <asset> [project] -o <file.glb> [--lod N]`: a cooked
//! model as a glTF binary, so a model the cook generates (a checked model
//! of parts or a plant, a `.vox`) leaves the engine for another tool or
//! renderer, and a glTF comes back as the engine draws it.
//!
//! The project is cooked first; the model blob, its textures and its
//! `#anim` blob are read back as the runtime reads them (og-bundle's
//! decoders), so the file is what the game draws: the cook's conformed
//! geometry and colours, its tangents, and one level of its LOD chain.
//! Each primitive's level `--lod` (0, the full mesh, by default; a
//! primitive with fewer levels gives its coarsest), with that level's own
//! material when the cook reparameterised it. Textures are PNG, transcoded
//! from the cooked KTX2 (a normal map with its z rebuilt). Instances
//! (`at` blocks, `EXT_mesh_gpu_instancing`) become one child node each.
//! A skinned model keeps its skins, joints and weights; its clips are
//! written as the cook resampled them, and morph targets with their
//! default weights. A plant keeps its wind attributes (`_OG_PIVOT`,
//! `_OG_ANCHOR`) and leaf extras, which other loaders ignore. Not
//! exported: the impostor, the material variants, the colliders and the
//! visibility clips.

use crate::project::Project;
use og_anim::cooked::BundleAnim;
use og_cook::cook::vox::Bin;
use og_cook::runtime::{Bundle, CookedModel, anim_id, asset_id};
use og_core::bytemuck;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: og export gltf <asset> [project] -o <file.glb> [--lod N]";

pub fn run(args: &[&str]) -> ExitCode {
    let ["gltf", rest @ ..] = args else {
        eprintln!("og export: {USAGE}");
        return ExitCode::from(2);
    };
    let mut asset = None;
    let mut project = ".".to_string();
    let mut out = None;
    let mut lod = 0usize;
    let mut i = 0;
    while i < rest.len() {
        match rest[i] {
            "-o" | "--out" => {
                let Some(p) = rest.get(i + 1) else {
                    eprintln!("og export gltf: -o takes a file. {USAGE}");
                    return ExitCode::from(2);
                };
                out = Some(PathBuf::from(p));
                i += 1;
            }
            "--lod" => {
                let Some(n) = rest.get(i + 1).and_then(|v| v.parse().ok()) else {
                    eprintln!("og export gltf: --lod takes a level number. {USAGE}");
                    return ExitCode::from(2);
                };
                lod = n;
                i += 1;
            }
            a if a.starts_with('-') => {
                eprintln!("og export gltf: unknown option `{a}`. {USAGE}");
                return ExitCode::from(2);
            }
            a if asset.is_none() => asset = Some(a.to_string()),
            a => project = a.to_string(),
        }
        i += 1;
    }
    let (Some(asset), Some(out)) = (asset, out) else {
        eprintln!("og export gltf: name the model asset and -o <file.glb>. {USAGE}");
        return ExitCode::from(2);
    };
    match export(Path::new(&project), &asset, &out, lod) {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("og export gltf: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Cooks the project at `project`, writes model `asset` to `out` and
/// answers a line saying what it wrote.
fn export(project: &Path, asset: &str, out: &Path, lod: usize) -> Result<String, String> {
    let p = Project::open(project)?;
    let root = PathBuf::from(&p.root);
    super::cook::cook(&root, 0, false)?;
    let bundle = Bundle::open(&root.join("cooked")).map_err(|e| e.to_string())?;
    let id = asset_id(asset);
    let model = bundle.load_model(id).map_err(|e| {
        match bundle.manifest().outputs().any(|o| o.id == id) {
            true => e.to_string(),
            false => format!(
                "{id} is not a cooked model of {} (its id is its path under assets/)",
                root.display()
            ),
        }
    })?;
    let levels = model
        .primitives
        .iter()
        .map(|p| p.lods.len())
        .max()
        .unwrap_or(1);
    if lod >= levels {
        return Err(format!(
            "{id} has {levels} level(s) of detail: --lod is 0 to {}",
            levels - 1
        ));
    }
    let anim = bundle
        .manifest()
        .outputs()
        .any(|o| o.id == anim_id(id))
        .then(|| bundle.load_animation(id))
        .transpose()
        .map_err(|e| e.to_string())?;
    let mut textures = Vec::with_capacity(model.textures.len());
    for t in &model.textures {
        let data = bundle
            .load_texture(&t.id)
            .and_then(|c| c.to_texture_data())
            .map_err(|e| e.to_string())?;
        let img = image::RgbaImage::from_raw(data.width, data.height, data.rgba)
            .ok_or_else(|| format!("{}: its pixels do not fill its size", t.id))?;
        let mut png = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .map_err(|e| format!("{}: {e}", t.id))?;
        textures.push(png);
    }
    let glb = to_glb(&model, anim.as_ref(), &textures, lod)?;
    if let Some(dir) = out.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(out, &glb).map_err(|e| format!("{}: {e}", out.display()))?;
    let triangles: usize = model.primitives.iter().map(|p| p.triangle_count(lod)).sum();
    Ok(format!(
        "{}: {id} level {lod} of {levels}, {} primitive(s), {triangles} triangles, {} material(s), {} texture(s){}, {} bytes",
        out.display(),
        model.primitives.len(),
        model.materials.len(),
        model.textures.len(),
        match &anim {
            Some(a) => format!(
                ", {} skin(s), {} clip(s), {} morph primitive(s)",
                a.skeleton.skins.len(),
                a.clips.len(),
                a.morphs.len()
            ),
            None => String::new(),
        },
        glb.len()
    ))
}

const ARRAY_BUFFER: u32 = 34962;
const ELEMENT_ARRAY_BUFFER: u32 = 34963;
const FLOAT: u32 = 5126;
const UNSIGNED_SHORT: u32 = 5123;
const UNSIGNED_INT: u32 = 5125;

fn floats(bin: &mut Bin, v: &[f32], ty: &str, count: usize, target: Option<u32>) -> usize {
    bin.add(
        bytemuck::cast_slice(v),
        target,
        json!({ "componentType": FLOAT, "count": count, "type": ty }),
    )
}

/// A VEC3 accessor with the min and max glTF requires of positions.
fn vec3_bounded(bin: &mut Bin, v: &[[f32; 3]]) -> usize {
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for p in v {
        for a in 0..3 {
            lo[a] = lo[a].min(p[a]);
            hi[a] = hi[a].max(p[a]);
        }
    }
    if v.is_empty() {
        (lo, hi) = ([0.0; 3], [0.0; 3]);
    }
    bin.add(
        bytemuck::cast_slice(v),
        Some(ARRAY_BUFFER),
        json!({ "componentType": FLOAT, "count": v.len(), "type": "VEC3", "min": lo, "max": hi }),
    )
}

/// The glTF binary of `model` at level `lod`, its textures given as PNGs in
/// `model.textures`' order.
pub fn to_glb(
    model: &CookedModel,
    anim: Option<&og_anim::cooked::CookedAnimation>,
    pngs: &[Vec<u8>],
    lod: usize,
) -> Result<Vec<u8>, String> {
    let mut bin = Bin::default();
    let mut used: std::collections::BTreeSet<String> = Default::default();

    // Textures: one image and one texture each, one shared sampler.
    let images: Vec<Value> = pngs
        .iter()
        .map(|png| json!({ "bufferView": bin.view(png), "mimeType": "image/png" }))
        .collect();
    let textures: Vec<Value> = (0..pngs.len())
        .map(|i| json!({ "source": i, "sampler": 0 }))
        .collect();

    // Materials, the default one (last) among them.
    let mut materials = Vec::with_capacity(model.materials.len());
    for m in &model.materials {
        let tex = |i: Option<usize>, slot: usize, used: &mut std::collections::BTreeSet<String>| {
            i.map(|i| {
                let mut t = json!({ "index": i });
                let x = m.uv_transforms[slot];
                if !x.is_identity() {
                    used.insert("KHR_texture_transform".into());
                    t["extensions"] = json!({ "KHR_texture_transform": {
                        "offset": [x.offset.x, x.offset.y],
                        "scale": [x.scale.x, x.scale.y],
                        "rotation": x.rotation,
                    }});
                }
                t
            })
        };
        use og_render_data::UvTransform as Uv;
        let mut pbr = json!({
            "baseColorFactor": m.base_color,
            "metallicFactor": m.metallic,
            "roughnessFactor": m.roughness,
        });
        if let Some(t) = tex(m.base_color_texture, Uv::MAP, &mut used) {
            pbr["baseColorTexture"] = t;
        }
        if let Some(t) = tex(
            m.metallic_roughness_texture,
            Uv::METALNESS_ROUGHNESS,
            &mut used,
        ) {
            pbr["metallicRoughnessTexture"] = t;
        }
        let mut j = json!({
            "name": m.name,
            "pbrMetallicRoughness": pbr,
            "doubleSided": m.double_sided,
        });
        if let Some(mut t) = tex(m.normal_texture, Uv::NORMAL, &mut used) {
            t["scale"] = m.normal_scale.into();
            j["normalTexture"] = t;
        }
        if let Some(mut t) = tex(m.occlusion_texture, Uv::AO, &mut used) {
            t["strength"] = m.occlusion_strength.into();
            j["occlusionTexture"] = t;
        }
        if let Some(t) = tex(m.emissive_texture, Uv::EMISSIVE, &mut used) {
            j["emissiveTexture"] = t;
        }
        let mut ext = serde_json::Map::new();
        // The cooked emissive carries its strength: a factor over 1 is
        // written back as glTF's unit colour and a strength.
        let peak = m.emissive.iter().copied().fold(0.0f32, f32::max);
        if peak > 0.0 {
            let k = peak.max(1.0);
            j["emissiveFactor"] = json!(m.emissive.map(|c| c / k));
            if k > 1.0 {
                ext.insert(
                    "KHR_materials_emissive_strength".into(),
                    json!({ "emissiveStrength": k }),
                );
            }
        }
        match m.alpha_mode {
            og_cook::format::alpha::MASK => {
                j["alphaMode"] = "MASK".into();
                j["alphaCutoff"] = m.alpha_cutoff.into();
            }
            og_cook::format::alpha::BLEND => j["alphaMode"] = "BLEND".into(),
            _ => {}
        }
        if m.unlit {
            ext.insert("KHR_materials_unlit".into(), json!({}));
        }
        if m.translucency > 0.0 {
            ext.insert(
                "KHR_materials_diffuse_transmission".into(),
                json!({ "diffuseTransmissionFactor": m.translucency }),
            );
        }
        let p = &m.physical;
        type P = og_render_data::material::PhysicalExtensions<usize>;
        let mut phys = |bit: u32, name: &str, v: Value| {
            if p.present & bit != 0 {
                ext.insert(name.into(), v);
            }
        };
        let t = |i: Option<usize>| i.map(|i| json!({ "index": i }));
        let put = |mut v: Value, k: &str, t: Option<Value>| {
            if let Some(t) = t {
                v[k] = t;
            }
            v
        };
        phys(
            P::CLEARCOAT,
            "KHR_materials_clearcoat",
            put(
                json!({ "clearcoatFactor": p.clearcoat, "clearcoatRoughnessFactor": p.clearcoat_roughness }),
                "clearcoatTexture",
                t(p.clearcoat_texture),
            ),
        );
        phys(
            P::SHEEN,
            "KHR_materials_sheen",
            json!({ "sheenColorFactor": p.sheen_color, "sheenRoughnessFactor": p.sheen_roughness }),
        );
        phys(
            P::TRANSMISSION,
            "KHR_materials_transmission",
            put(
                json!({ "transmissionFactor": p.transmission }),
                "transmissionTexture",
                t(p.transmission_texture),
            ),
        );
        let mut volume =
            json!({ "thicknessFactor": p.thickness, "attenuationColor": p.attenuation_color });
        if p.attenuation_distance > 0.0 {
            volume["attenuationDistance"] = p.attenuation_distance.into();
        }
        phys(P::VOLUME, "KHR_materials_volume", volume);
        phys(P::IOR, "KHR_materials_ior", json!({ "ior": p.ior }));
        phys(
            P::IRIDESCENCE,
            "KHR_materials_iridescence",
            json!({
                "iridescenceFactor": p.iridescence,
                "iridescenceIor": p.iridescence_ior,
                "iridescenceThicknessMinimum": p.iridescence_thickness[0],
                "iridescenceThicknessMaximum": p.iridescence_thickness[1],
            }),
        );
        phys(
            P::ANISOTROPY,
            "KHR_materials_anisotropy",
            json!({ "anisotropyStrength": p.anisotropy, "anisotropyRotation": p.anisotropy_rotation }),
        );
        used.extend(ext.keys().cloned());
        if !ext.is_empty() {
            j["extensions"] = ext.into();
        }
        // A leaf's canopy and season, as the cook reads them back.
        let mut leaf = serde_json::Map::new();
        if m.dome_normals > 0.0 {
            leaf.insert("domeNormals".into(), m.dome_normals.into());
        }
        if m.season.season > 0.0 {
            leaf.insert("season".into(), m.season.season.into());
            leaf.insert("autumnColor".into(), json!(m.season.autumn_color));
            leaf.insert("autumnColor2".into(), json!(m.season.autumn_color2));
            leaf.insert("seasonShift".into(), m.season.shift.into());
        }
        if !leaf.is_empty() {
            j["extras"] = json!({ "ogFoliage": leaf });
        }
        materials.push(j);
    }

    // Per primitive, the level's vertices in the full mesh's order (to carry
    // the skin and morph streams), indexed by the primitive.
    let skin_of = |p: usize| anim.and_then(|a| a.skin_streams.iter().find(|s| s.primitive == p));
    let morph_of = |p: usize| anim.and_then(|a| a.morphs.iter().find(|m| m.primitive == p));
    let mut meshes = Vec::with_capacity(model.meshes.len());
    for mesh in &model.meshes {
        let targets = mesh
            .primitives
            .clone()
            .filter_map(morph_of)
            .map(|m| m.names.len())
            .max()
            .unwrap_or(0);
        let mut prims = Vec::new();
        for pi in mesh.primitives.clone() {
            let p = &model.primitives[pi];
            let level = lod.min(p.lods.len() - 1);
            let m = p.mesh_compact(level);
            let sources = p.lod_sources(level);
            let n = m.vertices.len();
            let pos: Vec<[f32; 3]> = m.vertices.iter().map(|v| v.position).collect();
            let mut attributes = json!({ "POSITION": vec3_bounded(&mut bin, &pos) });
            let normals: Vec<f32> = m.vertices.iter().flat_map(|v| v.normal).collect();
            attributes["NORMAL"] = floats(&mut bin, &normals, "VEC3", n, Some(ARRAY_BUFFER)).into();
            let uvs: Vec<f32> = m.vertices.iter().flat_map(|v| v.uv).collect();
            attributes["TEXCOORD_0"] = floats(&mut bin, &uvs, "VEC2", n, Some(ARRAY_BUFFER)).into();
            if !p.tangents.is_empty() && m.vertices.iter().all(|v| v.tangent[3] != 0.0) {
                let t: Vec<f32> = m.vertices.iter().flat_map(|v| v.tangent).collect();
                attributes["TANGENT"] = floats(&mut bin, &t, "VEC4", n, Some(ARRAY_BUFFER)).into();
            }
            if m.vertices.iter().any(|v| v.color != [1.0; 4]) {
                let c: Vec<f32> = m.vertices.iter().flat_map(|v| v.color).collect();
                attributes["COLOR_0"] = floats(&mut bin, &c, "VEC4", n, Some(ARRAY_BUFFER)).into();
            }
            if let Some((f, _)) = &p.foliage {
                let at = |s: u32| f.get(s as usize).copied().unwrap_or_default();
                let pivot: Vec<f32> = sources.iter().flat_map(|&s| at(s).branch).collect();
                let anchor: Vec<f32> = sources.iter().flat_map(|&s| at(s).leaf).collect();
                attributes["_OG_PIVOT"] =
                    floats(&mut bin, &pivot, "VEC4", n, Some(ARRAY_BUFFER)).into();
                attributes["_OG_ANCHOR"] =
                    floats(&mut bin, &anchor, "VEC4", n, Some(ARRAY_BUFFER)).into();
            }
            if let Some(s) = skin_of(pi) {
                // A joint the cook quantised to no weight is written as 0,
                // as glTF asks of an unused joint.
                let joints: Vec<u16> = sources
                    .iter()
                    .flat_map(|&v| {
                        let (j, w) = (s.joints.get(v as usize), s.weights.get(v as usize));
                        let (j, w) = (j.copied().unwrap_or([0; 4]), w.copied().unwrap_or([0.0; 4]));
                        std::array::from_fn::<u16, 4, _>(|k| if w[k] > 0.0 { j[k] } else { 0 })
                    })
                    .collect();
                let weights: Vec<f32> = sources
                    .iter()
                    .flat_map(|&v| {
                        s.weights
                            .get(v as usize)
                            .copied()
                            .unwrap_or([1.0, 0.0, 0.0, 0.0])
                    })
                    .collect();
                attributes["JOINTS_0"] = bin
                    .add(
                        bytemuck::cast_slice(&joints),
                        Some(ARRAY_BUFFER),
                        json!({ "componentType": UNSIGNED_SHORT, "count": n, "type": "VEC4" }),
                    )
                    .into();
                attributes["WEIGHTS_0"] =
                    floats(&mut bin, &weights, "VEC4", n, Some(ARRAY_BUFFER)).into();
            }
            let indices = bin.add(
                bytemuck::cast_slice(&m.indices),
                Some(ELEMENT_ARRAY_BUFFER),
                json!({ "componentType": UNSIGNED_INT, "count": m.indices.len(), "type": "SCALAR" }),
            );
            let material = p.lods[level].material.unwrap_or(p.material);
            let mut prim =
                json!({ "attributes": attributes, "indices": indices, "material": material });
            if targets > 0 {
                let morph = morph_of(pi);
                let mut list = Vec::with_capacity(targets);
                for t in 0..targets {
                    let pick = |d: Option<&Vec<Vec<glam::Vec3>>>| -> Vec<[f32; 3]> {
                        sources
                            .iter()
                            .map(|&v| {
                                d.and_then(|d| d.get(t))
                                    .and_then(|d| d.get(v as usize))
                                    .map_or([0.0; 3], |d| d.to_array())
                            })
                            .collect()
                    };
                    let mut target = json!({ "POSITION": vec3_bounded(&mut bin, &pick(morph.map(|m| &m.positions))) });
                    if let Some(normals) = morph.and_then(|m| m.normals.as_ref()) {
                        let d: Vec<f32> = pick(Some(normals)).concat();
                        target["NORMAL"] =
                            floats(&mut bin, &d, "VEC3", n, Some(ARRAY_BUFFER)).into();
                    }
                    list.push(target);
                }
                prim["targets"] = list.into();
            }
            if p.foliage.as_ref().is_some_and(|(_, leaves)| *leaves) {
                prim["extras"] = json!({ "ogFoliage": { "leaves": true } });
            }
            prims.push(prim);
        }
        let mut j = json!({ "name": mesh.name, "primitives": prims });
        if targets > 0 {
            let first = mesh.primitives.clone().find_map(morph_of);
            if let Some(m) = first {
                j["weights"] = json!(m.default_weights);
                j["extras"] = json!({ "targetNames": m.names });
            }
        }
        meshes.push(j);
    }

    // Nodes, in the cooked (and the skeleton's) order; instances after them.
    let skinned = anim.is_some_and(|a| !a.skeleton.skins.is_empty() && !a.skin_streams.is_empty());
    let skinned_meshes: std::collections::BTreeSet<usize> = anim
        .map(|a| a.skin_streams.iter().map(|s| s.mesh).collect())
        .unwrap_or_default();
    let mut nodes: Vec<Value> = Vec::with_capacity(model.nodes.len());
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); model.nodes.len()];
    for (i, n) in model.nodes.iter().enumerate() {
        let mut j = json!({
            "name": n.name,
            "translation": n.translation.to_array(),
            "rotation": n.rotation.to_array(),
            "scale": n.scale.to_array(),
        });
        if let Some(m) = n.mesh {
            if model.instances.contains_key(&i) {
                // Drawn only at its instances: one child node each.
            } else {
                j["mesh"] = m.into();
                if skinned && skinned_meshes.contains(&m) {
                    j["skin"] = 0.into();
                }
            }
        }
        if !n.visible {
            used.insert("KHR_node_visibility".into());
            j["extensions"] = json!({ "KHR_node_visibility": { "visible": false } });
        }
        if let Some(parent) = n.parent {
            children[parent].push(i);
        }
        nodes.push(j);
    }
    for (&node, transforms) in &model.instances {
        let Some(mesh) = model.nodes[node].mesh else {
            continue;
        };
        for (k, t) in transforms.iter().enumerate() {
            let (s, r, tr) = t.to_scale_rotation_translation();
            children[node].push(nodes.len());
            nodes.push(json!({
                "name": format!("{}#{k}", model.nodes[node].name),
                "mesh": mesh,
                "translation": tr.to_array(),
                "rotation": r.to_array(),
                "scale": s.to_array(),
            }));
        }
    }
    for (i, c) in children.into_iter().enumerate() {
        if !c.is_empty() {
            nodes[i]["children"] = c.into();
        }
    }

    // Lights.
    let mut lights = Vec::new();
    for l in &model.lights {
        let kind = match l.kind {
            og_cook::format::light::DIRECTIONAL => "directional",
            og_cook::format::light::SPOT => "spot",
            _ => "point",
        };
        let mut j =
            json!({ "name": l.name, "type": kind, "color": l.color, "intensity": l.intensity });
        if l.range > 0.0 {
            j["range"] = l.range.into();
        }
        if kind == "spot" {
            j["spot"] = json!({ "innerConeAngle": l.cone[0], "outerConeAngle": l.cone[1] });
        }
        nodes[l.node]["extensions"]["KHR_lights_punctual"] = json!({ "light": lights.len() });
        lights.push(j);
    }

    let scenes: Vec<Value> = model
        .scenes
        .iter()
        .map(|s| json!({ "name": s.name, "nodes": s.roots }))
        .collect();
    let mut root = json!({
        "asset": { "version": "2.0", "generator": format!("og export gltf {}", crate::project::own_version()) },
        "scenes": scenes,
        "nodes": nodes,
        "meshes": meshes,
        "materials": materials,
    });
    if let Some(s) = model.default_scene {
        root["scene"] = s.into();
    }
    if !pngs.is_empty() {
        root["images"] = images.into();
        root["textures"] = textures.into();
        root["samplers"] =
            json!([{ "magFilter": 9729, "minFilter": 9987, "wrapS": 10497, "wrapT": 10497 }]);
    }
    if !lights.is_empty() {
        used.insert("KHR_lights_punctual".into());
        root["extensions"] = json!({ "KHR_lights_punctual": { "lights": lights } });
    }

    // Skins and clips.
    if let Some(a) = anim {
        if skinned {
            root["skins"] = a
                .skeleton
                .skins
                .iter()
                .map(|s| {
                    let ibm: Vec<f32> = s
                        .inverse_bind
                        .iter()
                        .flat_map(|m| m.to_cols_array())
                        .collect();
                    json!({
                        "name": s.name,
                        "joints": s.joints,
                        "inverseBindMatrices": floats(&mut bin, &ibm, "MAT4", s.joints.len(), None),
                    })
                })
                .collect::<Vec<_>>()
                .into();
        }
        let mut animations = Vec::new();
        for c in &a.clips {
            let times: Vec<f32> = (0..c.frames)
                .map(|i| (i as f32 / c.rate).min(c.duration))
                .collect();
            let mut input = None;
            let (mut samplers, mut channels) = (Vec::new(), Vec::new());
            for ch in &c.channels {
                let (path, ty) = match ch.path {
                    og_anim::ChannelPath::Translation => ("translation", "VEC3"),
                    og_anim::ChannelPath::Rotation => ("rotation", "VEC4"),
                    og_anim::ChannelPath::Scale => ("scale", "VEC3"),
                    og_anim::ChannelPath::Weights => ("weights", "SCALAR"),
                    og_anim::ChannelPath::Visibility => continue,
                };
                let input = *input.get_or_insert_with(|| {
                    let lo = times.first().copied().unwrap_or(0.0);
                    let hi = times.last().copied().unwrap_or(0.0);
                    bin.add(
                        bytemuck::cast_slice(&times),
                        None,
                        json!({ "componentType": FLOAT, "count": times.len(), "type": "SCALAR", "min": [lo], "max": [hi] }),
                    )
                });
                let data = ch.decoded(c.frames);
                let count = if ty == "SCALAR" {
                    data.len()
                } else {
                    c.frames as usize
                };
                let output = floats(&mut bin, &data, ty, count, None);
                channels.push(json!({ "sampler": samplers.len(), "target": { "node": ch.node, "path": path } }));
                samplers.push(json!({
                    "input": input,
                    "output": output,
                    "interpolation": if ch.step { "STEP" } else { "LINEAR" },
                }));
            }
            if !channels.is_empty() {
                animations
                    .push(json!({ "name": c.name, "samplers": samplers, "channels": channels }));
            }
        }
        if !animations.is_empty() {
            root["animations"] = animations.into();
        }
    }
    if !used.is_empty() {
        root["extensionsUsed"] = json!(used.into_iter().collect::<Vec<_>>());
    }
    bin.finish(root)
}
