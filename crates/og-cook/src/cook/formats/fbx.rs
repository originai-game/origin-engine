//! FBX and Wavefront OBJ → glTF: a file read by ufbx (FBX binary and
//! ASCII from version 3000 on, OBJ with its `.mtl`) is written as a glTF
//! binary in memory, which the model cook then treats like any `.glb`
//! (meshoptimizer, LODs, the BVH, the node graph, clip compression), so an
//! FBX or OBJ asset is the same model blob a glTF is, played by the same
//! runtime path.
//!
//! **Space.** ufbx converts the scene to the engine's: metres (the file's
//! unit scale applied), +Y up and right-handed, the conversion baked into
//! the geometry and the transforms (`ModifyGeometry`). An FBX node's
//! pivots, pre- and post-rotation and offsets fold into its translation,
//! rotation and scale; a geometric transform (one that moves a node's
//! mesh but not its children) is baked into the mesh, or hung from a
//! helper node when the mesh is instanced; a non-standard scale
//! inheritance is compensated, through helper nodes where it must be.
//! Every node keeps its name.
//!
//! **Meshes.** Polygons are triangulated (ufbx's ear clipping, which
//! keeps concave faces whole); one glTF primitive per material a mesh
//! uses (the node's materials, else the mesh's). Each corner carries its
//! position, normal (ufbx makes the missing ones), first two UV sets
//! (`TEXCOORD_0`, `TEXCOORD_1`; v flipped, glTF's origin is the top left)
//! and first colour set (`COLOR_0`, sRGB as the DCC shows it, made
//! linear). Corners equal in every attribute and of the same control
//! point share a vertex. A mesh shown by several nodes with the same
//! materials is one glTF mesh drawn instanced.
//!
//! **Skins.** A mesh's first skin deformer becomes a glTF skin: its
//! clusters' bones are the joints, each cluster's geometry-to-bone matrix
//! the inverse bind, and each vertex its four strongest weights,
//! normalised (more are dropped with a note). A skinned mesh hangs from a
//! node of its own at the scene's root with no transform, as glTF draws a
//! skinned mesh where its joints put it.
//!
//! **Blend shapes** become morph targets (position and, when the file has
//! them, normal deltas per control point), named by their channels, with
//! the channels' weights as the mesh's default weights. A channel with
//! in-between shapes keeps its full shape (noted).
//!
//! **Animation.** Every animation stack becomes one clip of its name,
//! baked by ufbx (`bake_anim`): each node's local translation, rotation
//! and scale (linear and stepped keys kept where they lie, curves
//! resampled), and each blend channel's weight as the mesh node's
//! `weights` track. The clip starts at 0 and lasts the stack's playback
//! range.
//!
//! **Materials.** ufbx reads every FBX shading model (Lambert, Phong,
//! Stingray PBS, Arnold and OSL standard surface, 3ds Max physical and PBR
//! materials, glTF and OpenPBR) and an OBJ's `.mtl` (`Kd`, `Ks`, `Ns`,
//! `d`/`Tr`, `Ke`, `map_*`, `norm`, `Pr`, `Pm`) into PBR maps; those become
//! a glTF metallic-roughness material: base colour (its factor and map),
//! roughness (glossiness inverted), metalness, normal map, emission
//! (`KHR_materials_emissive_strength` above 1), ambient occlusion,
//! opacity (alpha blend when below 1 or mapped), double-sided and unlit.
//! Separate roughness and metalness maps are packed into one
//! metallic-roughness image (G and B), and an opacity map other than the
//! base colour's is packed into its alpha, both at cook time. A texture
//! embedded in the FBX is packed into the glb; one beside the file is
//! named by its relative path, so it is read, hashed and consumed as a
//! `.gltf`'s textures are (found by its relative path, else by its file
//! name beside the model). A texture's UV transform becomes
//! `KHR_texture_transform`, its wrap modes the sampler.
//!
//! **Lights** become `KHR_lights_punctual` (point, spot, directional; the
//! intensity as the DCC shows it, 1 at 100 %), each on a child node of its
//! node turned to its direction. Cameras, area and volume lights are
//! dropped with a note.
//!
//! **Refused**, each with its fix: a file ufbx cannot read (re-export it),
//! a texture neither embedded nor found (put it beside the model or export
//! with embedded media), a `mtllib` not found beside the OBJ, and a file
//! with no polygon mesh (NURBS, patches and curves convert to polygons in
//! the DCC first). Whatever the glTF path refuses is refused the same way.

use glam::{DMat4, DQuat, DVec3, Quat, Vec3};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::Path;

mod material;
mod mesh;

use super::vox::Bin;

/// A file read through ufbx, as the model cook reads it.
pub struct Imported {
    /// The glTF binary.
    pub glb: Vec<u8>,
    /// Files read beside the source that the glb does not name (an OBJ's
    /// `.mtl`, the sources of packed images): (path relative to the
    /// source's folder, bytes), part of its hash and consumed with it.
    pub read: Vec<(String, Vec<u8>)>,
    /// What the import dropped or changed.
    pub notes: Vec<String>,
}

/// Whether a source extension is read by this importer.
pub fn handles(ext: &str) -> bool {
    matches!(ext, "fbx" | "obj")
}

fn vec3(v: ufbx::Vec3) -> DVec3 {
    DVec3::new(v.x, v.y, v.z)
}

fn quat(q: ufbx::Quat) -> DQuat {
    DQuat::from_xyzw(q.x, q.y, q.z, q.w)
}

/// A ufbx affine matrix as glTF's column-major 4x4.
fn mat4(m: &ufbx::Matrix) -> [f32; 16] {
    [
        m.m00 as f32,
        m.m10 as f32,
        m.m20 as f32,
        0.0,
        m.m01 as f32,
        m.m11 as f32,
        m.m21 as f32,
        0.0,
        m.m02 as f32,
        m.m12 as f32,
        m.m22 as f32,
        0.0,
        m.m03 as f32,
        m.m13 as f32,
        m.m23 as f32,
        1.0,
    ]
}

fn f32s(v: &[f32]) -> Vec<u8> {
    bytemuck::cast_slice(v).to_vec()
}

/// The `mtllib` lines of an OBJ: each named file, read beside it.
fn mtl_files(obj: &[u8], dir: &Path) -> Result<Vec<(String, Vec<u8>)>, String> {
    let text = String::from_utf8_lossy(obj);
    let mut out = Vec::new();
    for line in text.lines() {
        let Some(rest) = line.trim_start().strip_prefix("mtllib") else {
            continue;
        };
        if !rest.starts_with([' ', '\t']) {
            continue;
        }
        let rest = rest.trim();
        // A name may hold spaces: the whole rest of the line first, then
        // each word (the format's list of names).
        let names: Vec<&str> = if dir.join(rest).is_file() {
            vec![rest]
        } else {
            rest.split_whitespace().collect()
        };
        for name in names {
            let path = dir.join(name);
            let bytes = std::fs::read(&path).map_err(|_| {
                format!(
                    "OBJ: `mtllib {name}`: {name} is not beside the file; put it there, or remove the line to cook the model with a plain material"
                )
            })?;
            out.push((name.replace('\\', "/"), bytes));
        }
    }
    Ok(out)
}

/// The ufbx error, with what to do about it.
fn load_error(what: &str, e: &ufbx::Error) -> String {
    let info = e.info();
    let detail = if info.is_empty() {
        e.description.to_string()
    } else {
        format!("{} ({info})", e.description)
    };
    format!(
        "{what}: {detail}; ufbx could not read it, so re-export it from the DCC (FBX: binary, 2013 or later; OBJ: with its .mtl)"
    )
}

/// Reads an FBX or OBJ (`ext`) and writes it as a glTF binary (module docs).
/// `path` is the source file, whose folder holds what it names.
pub fn to_glb(bytes: &[u8], path: &Path, ext: &str) -> Result<Imported, String> {
    let dir = path.parent().unwrap_or(Path::new("."));
    let what = if ext == "obj" { "OBJ" } else { "FBX" };
    let mut read = Vec::new();
    let mut opts = ufbx::LoadOpts {
        target_axes: ufbx::CoordinateAxes {
            right: ufbx::CoordinateAxis::PositiveX,
            up: ufbx::CoordinateAxis::PositiveY,
            front: ufbx::CoordinateAxis::PositiveZ,
        },
        target_unit_meters: 1.0,
        space_conversion: ufbx::SpaceConversion::ModifyGeometry,
        geometry_transform_handling: ufbx::GeometryTransformHandling::ModifyGeometry,
        inherit_mode_handling: ufbx::InheritModeHandling::Compensate,
        generate_missing_normals: true,
        filename: path.to_string_lossy().to_string().into(),
        file_format: if ext == "obj" {
            ufbx::FileFormat::Obj
        } else {
            ufbx::FileFormat::Fbx
        },
        ..Default::default()
    };
    if ext == "obj" {
        let mtl = mtl_files(bytes, dir)?;
        let mut data = Vec::new();
        for (_, b) in &mtl {
            data.extend_from_slice(b);
            data.push(b'\n');
        }
        opts.obj_mtl_data = data.into();
        read.extend(mtl);
    }
    let scene = ufbx::load_memory(bytes, opts).map_err(|e| load_error(what, &e))?;
    let mut notes = Vec::new();
    if scene.meshes.iter().all(|m| m.num_triangles == 0) {
        let other = !scene.nurbs_surfaces.is_empty()
            || !scene.nurbs_curves.is_empty()
            || !scene.line_curves.is_empty()
            || !scene.procedural_geometries.is_empty();
        return Err(if other {
            format!(
                "{what}: no polygon mesh, only NURBS surfaces, patches or curves; convert them to polygons in the DCC (Maya: Modify > Convert > NURBS to Polygons; 3ds Max: Convert to Editable Poly) and export again"
            )
        } else {
            format!("{what}: no polygon mesh to cook; export the meshes with it")
        });
    }
    let mut out = Writer {
        bin: Bin::default(),
        dir,
        what,
        nodes: Vec::new(),
        node_of: HashMap::new(),
        meshes: Vec::new(),
        mesh_of: HashMap::new(),
        materials: material::Materials::default(),
        skins: Vec::new(),
        lights: Vec::new(),
        morph_of: HashMap::new(),
        read,
        notes: &mut notes,
    };
    out.write_nodes(&scene)?;
    let animations = out.write_animations(&scene)?;
    let mut json = json!({
        "asset": { "version": "2.0", "generator": format!("og-cook {what} import (ufbx)") },
        "scene": 0,
    });
    let Writer {
        mut bin,
        mut nodes,
        meshes,
        materials,
        skins,
        lights,
        read,
        ..
    } = out;
    // The roots: the file's top nodes, then the skinned meshes' nodes.
    let mut parented = vec![false; nodes.len()];
    for n in &nodes {
        if let Some(c) = n.get("children").and_then(Value::as_array) {
            for c in c {
                parented[c.as_u64().unwrap() as usize] = true;
            }
        }
    }
    let scene_nodes: Vec<usize> = (0..nodes.len()).filter(|&i| !parented[i]).collect();
    json["scenes"] = json!([{ "nodes": scene_nodes }]);
    for n in nodes.iter_mut() {
        if n.get("children")
            .and_then(Value::as_array)
            .is_some_and(Vec::is_empty)
        {
            n.as_object_mut().unwrap().remove("children");
        }
    }
    json["nodes"] = nodes.into();
    json["meshes"] = meshes.into();
    let mut used = Vec::new();
    let (mats, images, textures, samplers, mut read) = materials.finish(&mut bin, read)?;
    if !mats.is_empty() {
        json["materials"] = mats.into();
    }
    if !images.is_empty() {
        json["images"] = images.into();
        json["textures"] = textures.into();
        json["samplers"] = samplers.into();
    }
    if !skins.is_empty() {
        json["skins"] = skins.into();
    }
    if !animations.is_empty() {
        json["animations"] = animations.into();
    }
    if !lights.is_empty() {
        json["extensions"] = json!({ "KHR_lights_punctual": { "lights": lights } });
        used.push("KHR_lights_punctual");
    }
    let text = json.to_string();
    for ext in [
        "KHR_texture_transform",
        "KHR_materials_emissive_strength",
        "KHR_materials_unlit",
    ] {
        if text.contains(ext) {
            used.push(ext);
        }
    }
    if !used.is_empty() {
        json["extensionsUsed"] = used.into();
    }
    read.sort();
    read.dedup_by(|a, b| a.0 == b.0);
    Ok(Imported {
        glb: bin.finish(json)?,
        read,
        notes,
    })
}

/// The glTF being written.
struct Writer<'a> {
    bin: Bin,
    dir: &'a Path,
    what: &'static str,
    nodes: Vec<Value>,
    /// A ufbx node's typed id → its glTF node.
    node_of: HashMap<u32, usize>,
    meshes: Vec<Value>,
    /// (ufbx mesh typed id, its materials' typed ids) → (glTF mesh, morph targets).
    mesh_of: HashMap<(u32, Vec<u32>), usize>,
    materials: material::Materials,
    skins: Vec<Value>,
    lights: Vec<Value>,
    /// A blend channel's element id → (the glTF node showing its mesh,
    /// the target's index, the mesh's target count).
    morph_of: HashMap<u32, Vec<(usize, usize, usize)>>,
    read: Vec<(String, Vec<u8>)>,
    notes: &'a mut Vec<String>,
}

impl Writer<'_> {
    /// Every node of the file (its root's descendants) with its TRS, mesh
    /// and light, then a root node per skinned mesh.
    fn write_nodes(&mut self, scene: &ufbx::Scene) -> Result<(), String> {
        for n in scene.nodes.iter().filter(|n| !n.is_root) {
            self.node_of.insert(n.element.typed_id, self.nodes.len());
            let t = &n.local_transform;
            let mut node = json!({ "name": n.element.name.to_string(), "children": [] });
            let tr = vec3(t.translation);
            let r = quat(t.rotation).normalize();
            let s = vec3(t.scale);
            if tr != DVec3::ZERO {
                node["translation"] = json!([tr.x as f32, tr.y as f32, tr.z as f32]);
            }
            if r != DQuat::IDENTITY {
                node["rotation"] = json!([r.x as f32, r.y as f32, r.z as f32, r.w as f32]);
            }
            if s != DVec3::ONE {
                node["scale"] = json!([s.x as f32, s.y as f32, s.z as f32]);
            }
            self.nodes.push(node);
        }
        for n in scene.nodes.iter().filter(|n| !n.is_root) {
            let me = self.node_of[&n.element.typed_id];
            let children: Vec<usize> = n
                .children
                .iter()
                .map(|c| self.node_of[&c.element.typed_id])
                .collect();
            self.nodes[me]["children"] = json!(children);
        }
        for n in scene.nodes.iter().filter(|n| !n.is_root) {
            let me = self.node_of[&n.element.typed_id];
            if let Some(m) = &n.mesh {
                self.write_node_mesh(n, m, me)?;
            }
            if let Some(l) = &n.light {
                self.write_light(n, l, me);
            }
            if let Some(c) = &n.camera {
                self.notes.push(format!(
                    "{}: camera '{}' dropped; a model carries no camera (place an og.Camera in the scene)",
                    self.what, c.element.name
                ));
            }
        }
        Ok(())
    }

    fn write_light(&mut self, n: &ufbx::Node, l: &ufbx::Light, parent: usize) {
        let ty = match l.type_ {
            ufbx::LightType::Point => "point",
            ufbx::LightType::Directional => "directional",
            ufbx::LightType::Spot => "spot",
            _ => {
                self.notes.push(format!(
                    "{}: light '{}' is an area or volume light; dropped (point, spot and directional lights are kept)",
                    self.what, n.element.name
                ));
                return;
            }
        };
        let c = vec3(l.color);
        let mut light = json!({
            "name": l.element.name.to_string(),
            "type": ty,
            "color": [c.x as f32, c.y as f32, c.z as f32],
            "intensity": l.intensity as f32,
        });
        if ty == "spot" {
            let outer = (l.outer_angle.to_radians() * 0.5).clamp(1e-4, std::f64::consts::FRAC_PI_2);
            let inner = (l.inner_angle.to_radians() * 0.5).clamp(0.0, outer - 1e-4);
            light["spot"] =
                json!({ "innerConeAngle": inner as f32, "outerConeAngle": outer as f32 });
        }
        self.lights.push(light);
        // glTF lights shine down -Z: a child node turned to the light's direction.
        let dir = vec3(l.local_direction).as_vec3().normalize_or(Vec3::NEG_Z);
        let q = Quat::from_rotation_arc(Vec3::NEG_Z, dir);
        let child = self.nodes.len();
        self.nodes.push(json!({
            "name": format!("{}-light", n.element.name),
            "rotation": [q.x, q.y, q.z, q.w],
            "extensions": { "KHR_lights_punctual": { "light": self.lights.len() - 1 } },
        }));
        self.nodes[parent]["children"]
            .as_array_mut()
            .unwrap()
            .push(child.into());
    }

    /// The clips: one per animation stack.
    fn write_animations(&mut self, scene: &ufbx::Scene) -> Result<Vec<Value>, String> {
        let mut out = Vec::new();
        for stack in scene.anim_stacks.iter() {
            let opts = ufbx::BakeOpts {
                trim_start_time: true,
                ..Default::default()
            };
            let baked = ufbx::bake_anim(scene, &stack.anim, opts).map_err(|e| {
                format!(
                    "{}: animation '{}' could not be baked: {}",
                    self.what, stack.element.name, e.description
                )
            })?;
            let mut channels = Vec::new();
            let mut samplers = Vec::new();
            let end = baked.playback_duration.max(0.0);
            for bn in baked.nodes.iter() {
                let Some(&node) = self.node_of.get(&bn.typed_id) else {
                    continue;
                };
                let rest = &scene.nodes[bn.typed_id as usize].local_transform;
                let tr: Vec<(f64, [f32; 3])> = bn
                    .translation_keys
                    .iter()
                    .map(|k| (k.time, vec3(k.value).as_vec3().to_array()))
                    .collect();
                let sc: Vec<(f64, [f32; 3])> = bn
                    .scale_keys
                    .iter()
                    .map(|k| (k.time, vec3(k.value).as_vec3().to_array()))
                    .collect();
                let mut prev = DQuat::IDENTITY;
                let ro: Vec<(f64, [f32; 4])> = bn
                    .rotation_keys
                    .iter()
                    .map(|k| {
                        // Keep neighbouring keys on one hemisphere, so a
                        // linear key interpolates the short way.
                        let mut q = quat(k.value).normalize();
                        if q.dot(prev) < 0.0 {
                            q = -q;
                        }
                        prev = q;
                        (k.time, q.as_quat().to_array())
                    })
                    .collect();
                let still3 = |keys: &[(f64, [f32; 3])], v: ufbx::Vec3| {
                    let v = vec3(v).as_vec3();
                    keys.iter()
                        .all(|(_, k)| Vec3::from(*k).abs_diff_eq(v, 1e-6))
                };
                if !still3(&tr, rest.translation) {
                    self.channel(&mut channels, &mut samplers, node, "translation", &tr, end);
                }
                let r = quat(rest.rotation).as_quat().normalize();
                if !ro
                    .iter()
                    .all(|(_, k)| Quat::from_array(*k).dot(r).abs() > 1.0 - 1e-7)
                {
                    self.channel(&mut channels, &mut samplers, node, "rotation", &ro, end);
                }
                if !still3(&sc, rest.scale) {
                    self.channel(&mut channels, &mut samplers, node, "scale", &sc, end);
                }
            }
            self.write_weights(&baked, &mut channels, &mut samplers, end);
            if channels.is_empty() {
                self.notes.push(format!(
                    "{}: animation '{}' moves nothing the model has; dropped",
                    self.what, stack.element.name
                ));
                continue;
            }
            out.push(json!({
                "name": stack.element.name.to_string(),
                "channels": channels,
                "samplers": samplers,
            }));
        }
        Ok(out)
    }

    /// A blend channel's `DeformPercent` keys, as the `weights` track of
    /// each node showing its mesh: every target of that mesh sampled at the
    /// union of its channels' key times.
    fn write_weights(
        &mut self,
        baked: &ufbx::BakedAnim,
        channels: &mut Vec<Value>,
        samplers: &mut Vec<Value>,
        end: f64,
    ) {
        // node → (target count, target → keys).
        // node → (target count, per target its (time, weight) keys).
        type Tracks = (usize, Vec<Vec<(f64, f64)>>);
        let mut per_node: std::collections::BTreeMap<usize, Tracks> = Default::default();
        for el in baked.elements.iter() {
            let Some(targets) = self.morph_of.get(&el.element_id) else {
                continue;
            };
            let Some(prop) = el.props.iter().find(|p| &*p.name == "DeformPercent") else {
                continue;
            };
            let keys: Vec<(f64, f64)> = prop
                .keys
                .iter()
                .map(|k| (k.time, k.value.x / 100.0))
                .collect();
            for &(node, target, count) in targets {
                let e = per_node
                    .entry(node)
                    .or_insert_with(|| (count, vec![Vec::new(); count]));
                e.1[target] = keys.clone();
            }
        }
        for (node, (count, tracks)) in per_node {
            let mesh = self.nodes[node]["mesh"].as_u64().unwrap() as usize;
            let rest: Vec<f64> = self.meshes[mesh]["weights"]
                .as_array()
                .map(|w| w.iter().map(|v| v.as_f64().unwrap_or(0.0)).collect())
                .unwrap_or_else(|| vec![0.0; count]);
            let mut times: Vec<f64> = tracks.iter().flatten().map(|k| k.0).collect();
            times.push(0.0);
            times.sort_by(f64::total_cmp);
            times.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
            let sample = |keys: &[(f64, f64)], t: f64, rest: f64| -> f64 {
                match keys.iter().position(|k| k.0 >= t) {
                    None => keys.last().map_or(rest, |k| k.1),
                    Some(0) => keys[0].1,
                    Some(i) => {
                        let (a, b) = (keys[i - 1], keys[i]);
                        let f = if b.0 > a.0 {
                            (t - a.0) / (b.0 - a.0)
                        } else {
                            1.0
                        };
                        a.1 + (b.1 - a.1) * f
                    }
                }
            };
            let mut values = Vec::with_capacity(times.len() * count);
            for &t in &times {
                for (i, keys) in tracks.iter().enumerate() {
                    values.push(sample(keys, t, rest.get(i).copied().unwrap_or(0.0)) as f32);
                }
            }
            // A clip that leaves every weight at rest has no weights track.
            if values.iter().enumerate().all(|(i, v)| {
                (*v as f64 - rest.get(i % count).copied().unwrap_or(0.0)).abs() < 1e-6
            }) {
                continue;
            }
            let times: Vec<f32> = times.iter().map(|&t| t as f32).collect();
            self.sampler_raw(
                channels, samplers, node, "weights", &times, &values, end, count,
            );
        }
    }

    /// Appends one TRS channel; a track that ends before the clip holds its
    /// last value to the end.
    fn channel<const N: usize>(
        &mut self,
        channels: &mut Vec<Value>,
        samplers: &mut Vec<Value>,
        node: usize,
        path: &str,
        keys: &[(f64, [f32; N])],
        end: f64,
    ) {
        if keys.is_empty() {
            return;
        }
        let times: Vec<f32> = keys.iter().map(|k| k.0 as f32).collect();
        let values: Vec<f32> = keys.iter().flat_map(|k| k.1).collect();
        self.sampler_raw(channels, samplers, node, path, &times, &values, end, N);
    }

    #[allow(clippy::too_many_arguments)]
    fn sampler_raw(
        &mut self,
        channels: &mut Vec<Value>,
        samplers: &mut Vec<Value>,
        node: usize,
        path: &str,
        times: &[f32],
        values: &[f32],
        end: f64,
        width: usize,
    ) {
        let mut times = times.to_vec();
        let mut values = values.to_vec();
        let last = *times.last().unwrap();
        if (last as f64) < end - 1e-6 {
            times.push(end as f32);
            let tail = values[values.len() - width..].to_vec();
            values.extend(tail);
        }
        let min = times[0];
        let max = *times.last().unwrap();
        let input = self.bin.add(
            &f32s(&times),
            None,
            json!({ "componentType": 5126, "count": times.len(), "type": "SCALAR", "min": [min], "max": [max] }),
        );
        let ty = match (path, width) {
            ("weights", _) => "SCALAR",
            (_, 4) => "VEC4",
            _ => "VEC3",
        };
        let count = if path == "weights" {
            values.len()
        } else {
            values.len() / width
        };
        let output = self.bin.add(
            &f32s(&values),
            None,
            json!({ "componentType": 5126, "count": count, "type": ty }),
        );
        samplers.push(json!({ "input": input, "output": output, "interpolation": "LINEAR" }));
        channels.push(json!({
            "sampler": samplers.len() - 1,
            "target": { "node": node, "path": path },
        }));
    }
}

#[cfg(test)]
mod tests;
