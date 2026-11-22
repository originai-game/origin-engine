//! GLB of a scene: nodes, meshes, materials and their textures, skins, and the joints in node extras.

use std::collections::{BTreeMap, BTreeSet};

use og_shape::math::DVec3;

use og_core::math::double;
use og_shape::mesh::skin::Mat4;
use og_shape::tools::export::*;
use og_shape::tools::image::Texture;
use og_shape::tools::json::{Json, f32n, nums};
use og_shape::tools::surface::{METRES, split, y_up};

pub mod animation;
pub mod compress;
pub mod plain;
pub mod reorder;
pub mod skin;
pub(crate) use animation::*;

/// sRGB component to linear (glTF factors are linear).
pub(crate) fn linear(c: f64) -> f64 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.04045 {
        c / 12.92
    } else {
        double::pow((c + 0.055) / 1.055, 2.4)
    }
}

/// The glTF texture index of each of a material's maps, in
/// [`MAP_KINDS`] order.
pub(crate) type MapIndices = [Option<u64>; 5];

/// A material as glTF JSON, its maps at `maps`, and whether it needs
/// `KHR_materials_emissive_strength`. Fields a zero-value material does
/// not set are left out, so its JSON is as it always was.
pub(crate) fn material_json(
    m: &Material,
    part: &str,
    maps: MapIndices,
) -> Result<(Json, Vec<&'static str>), ExportError> {
    let name = if m.name.is_empty() { part } else { &m.name };
    let what = || format!("material {name:?}");
    let color = m.color.unwrap_or([1.0; 3]);
    let opacity = m.opacity.unwrap_or(1.0);
    let roughness = m.roughness.unwrap_or(1.0);
    let extra = [
        m.emissive_intensity.unwrap_or(1.0),
        m.alpha_test.unwrap_or(0.5),
        m.normal_scale.unwrap_or(1.0),
        m.ao_map_intensity.unwrap_or(1.0),
    ];
    for v in color
        .iter()
        .chain(&m.emissive)
        .chain([&opacity, &roughness, &m.metalness])
        .chain(&extra)
    {
        finite(*v, what)?;
    }
    let space = |c: f64| match m.color_space {
        ColorSpace::Srgb => linear(c),
        ColorSpace::Linear => c.clamp(0.0, 1.0),
    };
    let base = [
        space(color[0]),
        space(color[1]),
        space(color[2]),
        opacity.clamp(0.0, 1.0),
    ];
    let index = |k: u64| Json::Obj(vec![("index", Json::Int(k))]);
    let mut pbr = vec![("baseColorFactor", nums(&base))];
    if let Some(k) = maps[0] {
        pbr.push(("baseColorTexture", index(k)));
    }
    pbr.push(("metallicFactor", Json::Num(m.metalness.clamp(0.0, 1.0))));
    pbr.push(("roughnessFactor", Json::Num(roughness.clamp(0.0, 1.0))));
    if let Some(k) = maps[1] {
        pbr.push(("metallicRoughnessTexture", index(k)));
    }
    let mut fields = vec![
        ("name", Json::Str(name.to_string())),
        ("pbrMetallicRoughness", Json::Obj(pbr)),
    ];
    if let Some(k) = maps[2] {
        let mut t = vec![("index", Json::Int(k))];
        if let Some(scale) = m.normal_scale.filter(|&x| x != 1.0) {
            t.push(("scale", Json::Num(scale)));
        }
        fields.push(("normalTexture", Json::Obj(t)));
    }
    if let Some(k) = maps[4] {
        let mut t = vec![("index", Json::Int(k))];
        if let Some(strength) = m.ao_map_intensity.filter(|&x| x != 1.0) {
            t.push(("strength", Json::Num(strength.clamp(0.0, 1.0))));
        }
        fields.push(("occlusionTexture", Json::Obj(t)));
    }
    if let Some(k) = maps[3] {
        fields.push(("emissiveTexture", index(k)));
    }
    if m.emissive != [0.0; 3] {
        fields.push(("emissiveFactor", nums(&m.emissive.map(space))));
    }
    if let Some(cutoff) = m.alpha_test {
        fields.push(("alphaMode", Json::Str("MASK".into())));
        fields.push(("alphaCutoff", Json::Num(cutoff.max(0.0))));
    } else if m.transparent.unwrap_or(opacity < 1.0) {
        fields.push(("alphaMode", Json::Str("BLEND".into())));
    }
    if m.double_sided {
        fields.push(("doubleSided", Json::Bool(true)));
    }
    let strength = m.emissive_intensity.filter(|&x| x != 1.0);
    let mut extensions: Vec<(&'static str, Json)> = Vec::new();
    if let Some(x) = strength {
        extensions.push((
            "KHR_materials_emissive_strength",
            Json::Obj(vec![("emissiveStrength", Json::Num(x.max(0.0)))]),
        ));
    }
    let unit = |v: f64| Json::Num(v.clamp(0.0, 1.0));
    if m.clearcoat > 0.0 {
        extensions.push((
            "KHR_materials_clearcoat",
            Json::Obj(vec![
                ("clearcoatFactor", unit(m.clearcoat)),
                ("clearcoatRoughnessFactor", unit(m.clearcoat_roughness)),
            ]),
        ));
    }
    if m.sheen.iter().any(|c| *c > 0.0) {
        extensions.push((
            "KHR_materials_sheen",
            Json::Obj(vec![
                (
                    "sheenColorFactor",
                    nums(&m.sheen.map(|c| c.clamp(0.0, 1.0))),
                ),
                ("sheenRoughnessFactor", unit(m.sheen_roughness)),
            ]),
        ));
    }
    if m.transmission > 0.0 {
        extensions.push((
            "KHR_materials_transmission",
            Json::Obj(vec![("transmissionFactor", unit(m.transmission))]),
        ));
    }
    if m.iridescence > 0.0 {
        extensions.push((
            "KHR_materials_iridescence",
            Json::Obj(vec![("iridescenceFactor", unit(m.iridescence))]),
        ));
    }
    let used: Vec<&'static str> = extensions.iter().map(|e| e.0).collect();
    if !extensions.is_empty() {
        fields.push(("extensions", Json::Obj(extensions)));
    }
    Ok((Json::Obj(fields), used))
}

/// Each of a material's maps checked ([`check_texture`]) and given its
/// index in `textures` (shared when two maps are the same image).
pub(crate) fn map_indices<'t>(
    m: &'t Material,
    part: &str,
    has_uvs: bool,
    textures: &mut Vec<&'t Texture>,
) -> Result<MapIndices, ExportError> {
    let mut out: MapIndices = [None; 5];
    for (k, (_, slot)) in m.map_slots().into_iter().enumerate() {
        if let Some(t) = slot {
            check_texture(t, part, has_uvs)?;
            let i = textures.iter().position(|x| *x == t).unwrap_or_else(|| {
                textures.push(t);
                textures.len() - 1
            });
            out[k] = Some(i as u64);
        }
    }
    Ok(out)
}

pub(crate) fn joint_extras(j: &Checked) -> Json {
    let mut fields = vec![
        ("name", Json::Str(j.name.clone())),
        ("type", Json::Str(j.kind.into())),
        ("value", Json::Num(j.value)),
    ];
    if let Some(a) = j.axis {
        fields.push(("axis", nums(&[a.x, a.y, a.z])));
    }
    if let Some((lo, hi)) = j.span {
        fields.push(("lower", Json::Num(lo)));
        fields.push(("upper", Json::Num(hi)));
    }
    if let Some((leader, k)) = &j.mimic {
        fields.push((
            "mimic",
            Json::Obj(vec![
                ("joint", Json::Str(leader.clone())),
                ("multiplier", Json::Num(*k)),
            ]),
        ));
    }
    Json::Obj(vec![("s3d_joint", Json::Obj(fields))])
}

pub(crate) const ARRAY_BUFFER: u64 = 34962;

pub(crate) const ELEMENT_ARRAY_BUFFER: u64 = 34963;

pub(crate) const FLOAT: u64 = 5126;

pub(crate) const UNSIGNED_INT: u64 = 5125;

pub(crate) const UNSIGNED_SHORT: u64 = 5123;

pub(crate) fn push_f32(bin: &mut Vec<u8>, v: f32) {
    bin.extend_from_slice(&v.to_le_bytes());
}

/// A skin to write: inverse bind bytes, joint node indices, root bone node.
pub(crate) type SkinOut = (Vec<u8>, Vec<usize>, Option<usize>);

/// The scene as glTF 2.0 binary (GLB).
///
/// - Positions in metres, Y up (kernel Z becomes Y, Y becomes −Z), as f32.
/// - Normals smooth within a surface id and split at real edges: each
///   surface has its own vertices.
/// - u32 indices; one node, one mesh and one material per part.
/// - A part with a joint hangs under a node named after the joint, at its
///   pivot, which hangs under the parent part's node; its extras are
///   `{"s3d_joint": {name, type, value, axis, lower, upper, mimic}}` in SI
///   units, the axis in the joint node's frame (`docs/research.md` §1.5).
///   Face and edge names are not written.
/// - A skinned part (no joint) also gets a glTF skin: its bones as nodes
///   under the part's node, each at its rest frame relative to its parent
///   bone, inverse bind matrices, and JOINTS_0 (u16) and WEIGHTS_0 (f32)
///   per vertex (`docs/research.md` §11).
///
/// The bytes depend only on the scene.
pub fn glb(scene: &Scene) -> Result<Vec<u8>, ExportError> {
    glb_with(scene, false, &[])
}

/// [`glb`], with parts whose materials are written alike (the same
/// name, factors and maps) sharing one glTF material, as an atlas they
/// share asks ([`og_shape::tools::atlas::repack`]).
pub fn glb_shared(scene: &Scene) -> Result<Vec<u8>, ExportError> {
    glb_with(scene, true, &[])
}

/// [`glb`], with `clips` written as glTF animations: one a clip, a
/// sampler and a channel per joint it moves (a follower's from its
/// leader's), on the joint's node: `rotation` for a joint that turns,
/// `translation` for one that slides, linear between keys, times in
/// seconds. A turn of more than a quarter between two keys is written
/// with keys between them, so a viewer's shortest-way rotation follows
/// the joint round its axis. The nodes stand at the scene's pose, as
/// [`glb`] writes them; a viewer playing a clip moves them from there.
///
/// A clip is refused, saying why, when its name is empty or used twice,
/// it has no key, its times are not finite, not rising or negative, it
/// names a joint the scene has not, one that is fixed, floating or a
/// follower, or a value outside the joint's range.
pub fn glb_clips(scene: &Scene, clips: &[Clip]) -> Result<Vec<u8>, ExportError> {
    glb_with(scene, false, clips)
}

/// [`glb`], with the clips of its skinned parts as animations of their
/// bones' nodes: `skin_clips` gives each such part's, by its name. Every
/// bone gets its translation and rotation at each of the clip's times,
/// and its scale where the clip scales it. Clips of one name on several
/// parts are one animation.
pub fn glb_skin_clips(
    scene: &Scene,
    skin_clips: &[(&str, &[og_shape::mesh::SkinClip])],
) -> Result<Vec<u8>, ExportError> {
    glb_all(scene, false, &[], skin_clips)
}

pub(crate) fn glb_with(scene: &Scene, share: bool, clips: &[Clip]) -> Result<Vec<u8>, ExportError> {
    glb_all(scene, share, clips, &[])
}

pub(crate) fn glb_all(
    scene: &Scene,
    share: bool,
    clips: &[Clip],
    skin_clips: &[(&str, &[og_shape::mesh::SkinClip])],
) -> Result<Vec<u8>, ExportError> {
    let checked = check_joints(scene)?;
    let mut joint_node: BTreeMap<&str, usize> = BTreeMap::new();
    let mut bin: Vec<u8> = Vec::new();
    let mut views: Vec<Json> = Vec::new();
    let mut accessors: Vec<Json> = Vec::new();
    let mut meshes: Vec<Json> = Vec::new();
    let mut materials: Vec<Json> = Vec::new();
    let mut nodes: Vec<Json> = Vec::new();
    // Node index of each part, and the children of each node.
    let mut part_node: BTreeMap<&str, usize> = BTreeMap::new();
    let mut children: Vec<Vec<usize>> = Vec::new();
    let mut roots: Vec<usize> = Vec::new();
    let mut skins: Vec<SkinOut> = Vec::new();
    // Per skinned part: its skin and its bones' nodes.
    let mut skin_nodes: BTreeMap<&str, (&og_shape::mesh::Skin, Vec<usize>)> = BTreeMap::new();
    // Per part with morph targets: how many.
    let mut morph_counts: BTreeMap<&str, usize> = BTreeMap::new();
    let mut textures: Vec<&Texture> = Vec::new();
    let mut material_extensions: Vec<&'static str> = Vec::new();
    let mut view = |bin: &mut Vec<u8>, start: usize, target: u64| -> u64 {
        views.push(Json::Obj(vec![
            ("buffer", Json::Int(0)),
            ("byteOffset", Json::Int(start as u64)),
            ("byteLength", Json::Int((bin.len() - start) as u64)),
            ("target", Json::Int(target)),
        ]));
        (views.len() - 1) as u64
    };
    for (part, (joint, _)) in scene.parts.iter().zip(&checked) {
        let skin = crate::gltf::write::skin::checked(part)?;
        let s = split(part.solid);
        // Positions and normals in glTF space, f32.
        let pos: Vec<[f32; 3]> = s
            .positions
            .iter()
            .map(|&p| {
                let q = y_up(p) * METRES;
                [q.x as f32, q.y as f32, q.z as f32]
            })
            .collect();
        let nrm: Vec<[f32; 3]> = s
            .normals
            .iter()
            .map(|&n| {
                let q = y_up(n);
                [q.x as f32, q.y as f32, q.z as f32]
            })
            .collect();
        let mut lo = [f32::INFINITY; 3];
        let mut hi = [f32::NEG_INFINITY; 3];
        for p in &pos {
            for k in 0..3 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
        let mut attributes = Vec::new();
        let mut primitive = Vec::new();
        let mut morph_fields: Vec<(&str, Json)> = Vec::new();
        if !pos.is_empty() {
            let start = bin.len();
            pos.iter().flatten().for_each(|&v| push_f32(&mut bin, v));
            let pv = view(&mut bin, start, ARRAY_BUFFER);
            accessors.push(Json::Obj(vec![
                ("bufferView", Json::Int(pv)),
                ("componentType", Json::Int(FLOAT)),
                ("count", Json::Int(pos.len() as u64)),
                ("type", Json::Str("VEC3".into())),
                ("min", Json::Arr(lo.iter().map(|&v| f32n(v)).collect())),
                ("max", Json::Arr(hi.iter().map(|&v| f32n(v)).collect())),
            ]));
            attributes.push(("POSITION", Json::Int((accessors.len() - 1) as u64)));
            let start = bin.len();
            nrm.iter().flatten().for_each(|&v| push_f32(&mut bin, v));
            let nv = view(&mut bin, start, ARRAY_BUFFER);
            accessors.push(Json::Obj(vec![
                ("bufferView", Json::Int(nv)),
                ("componentType", Json::Int(FLOAT)),
                ("count", Json::Int(nrm.len() as u64)),
                ("type", Json::Str("VEC3".into())),
            ]));
            attributes.push(("NORMAL", Json::Int((accessors.len() - 1) as u64)));
            if !s.uvs.is_empty() {
                if s.uvs.iter().any(|q| !q.is_finite()) {
                    return Err(ExportError::NotFinite(format!(
                        "part {:?}'s UVs",
                        part.name
                    )));
                }
                let start = bin.len();
                s.uvs
                    .iter()
                    .flat_map(|&q| gltf_uv(q))
                    .for_each(|v| push_f32(&mut bin, v));
                let tv = view(&mut bin, start, ARRAY_BUFFER);
                accessors.push(Json::Obj(vec![
                    ("bufferView", Json::Int(tv)),
                    ("componentType", Json::Int(FLOAT)),
                    ("count", Json::Int(s.uvs.len() as u64)),
                    ("type", Json::Str("VEC2".into())),
                ]));
                attributes.push(("TEXCOORD_0", Json::Int((accessors.len() - 1) as u64)));
            }
            if !s.colors.is_empty() {
                if s.colors.iter().flatten().any(|c| !c.is_finite()) {
                    return Err(ExportError::NotFinite(format!(
                        "part {:?}'s vertex colours",
                        part.name
                    )));
                }
                let start = bin.len();
                s.colors
                    .iter()
                    .flatten()
                    .for_each(|&c| push_f32(&mut bin, c.clamp(0.0, 1.0)));
                let cv = view(&mut bin, start, ARRAY_BUFFER);
                accessors.push(Json::Obj(vec![
                    ("bufferView", Json::Int(cv)),
                    ("componentType", Json::Int(FLOAT)),
                    ("count", Json::Int(s.colors.len() as u64)),
                    ("type", Json::Str("VEC4".into())),
                ]));
                attributes.push(("COLOR_0", Json::Int((accessors.len() - 1) as u64)));
            }
            if part.tangents && !s.uvs.is_empty() {
                // Each vertex's frame as the plain GLB has it.
                let frames = og_shape::tools::surface::corner_frames(part.solid);
                let start = bin.len();
                for &(i, k) in &s.corners {
                    let f = frames[i as usize][usize::from(k)];
                    let t = y_up(f.tangent);
                    for c in [t.x, t.y, t.z, f.w] {
                        push_f32(&mut bin, c as f32);
                    }
                }
                let tv = view(&mut bin, start, ARRAY_BUFFER);
                accessors.push(Json::Obj(vec![
                    ("bufferView", Json::Int(tv)),
                    ("componentType", Json::Int(FLOAT)),
                    ("count", Json::Int(s.corners.len() as u64)),
                    ("type", Json::Str("VEC4".into())),
                ]));
                attributes.push(("TANGENT", Json::Int((accessors.len() - 1) as u64)));
            }
            if let Some(sk) = skin {
                let (j, w) = crate::gltf::write::skin::vertex_data(sk, &s.sources);
                for (bytes, kind, name) in
                    [(j, UNSIGNED_SHORT, "JOINTS_0"), (w, FLOAT, "WEIGHTS_0")]
                {
                    let start = bin.len();
                    bin.extend_from_slice(&bytes);
                    let v = view(&mut bin, start, ARRAY_BUFFER);
                    accessors.push(Json::Obj(vec![
                        ("bufferView", Json::Int(v)),
                        ("componentType", Json::Int(kind)),
                        ("count", Json::Int(s.sources.len() as u64)),
                        ("type", Json::Str("VEC4".into())),
                    ]));
                    attributes.push((name, Json::Int((accessors.len() - 1) as u64)));
                }
            }
            let start = bin.len();
            s.indices
                .iter()
                .for_each(|&i| bin.extend_from_slice(&i.to_le_bytes()));
            let iv = view(&mut bin, start, ELEMENT_ARRAY_BUFFER);
            accessors.push(Json::Obj(vec![
                ("bufferView", Json::Int(iv)),
                ("componentType", Json::Int(UNSIGNED_INT)),
                ("count", Json::Int(s.indices.len() as u64)),
                ("type", Json::Str("SCALAR".into())),
            ]));
            primitive.push(("attributes", Json::Obj(attributes)));
            primitive.push(("indices", Json::Int((accessors.len() - 1) as u64)));
            // Morph targets: each vertex's offset, in glTF space.
            if let Some(m) = &part.solid.morphs {
                m.check(part.solid.positions.len())
                    .map_err(|e| ExportError::NotFinite(format!("part {:?}: {e}", part.name)))?;
                let mut targets = Vec::with_capacity(m.targets.len());
                for t in &m.targets {
                    let start = bin.len();
                    let mut lo = [f32::INFINITY; 3];
                    let mut hi = [f32::NEG_INFINITY; 3];
                    for &v in &s.sources {
                        let q = y_up(t.deltas[v as usize]) * METRES;
                        for (k, c) in [q.x as f32, q.y as f32, q.z as f32].into_iter().enumerate() {
                            lo[k] = lo[k].min(c);
                            hi[k] = hi[k].max(c);
                            push_f32(&mut bin, c);
                        }
                    }
                    let dv = view(&mut bin, start, ARRAY_BUFFER);
                    accessors.push(Json::Obj(vec![
                        ("bufferView", Json::Int(dv)),
                        ("componentType", Json::Int(FLOAT)),
                        ("count", Json::Int(s.sources.len() as u64)),
                        ("type", Json::Str("VEC3".into())),
                        ("min", Json::Arr(lo.iter().map(|&v| f32n(v)).collect())),
                        ("max", Json::Arr(hi.iter().map(|&v| f32n(v)).collect())),
                    ]));
                    targets.push(Json::Obj(vec![(
                        "POSITION",
                        Json::Int((accessors.len() - 1) as u64),
                    )]));
                }
                if !targets.is_empty() {
                    primitive.push(("targets", Json::Arr(targets)));
                    morph_fields = vec![
                        (
                            "weights",
                            Json::Arr(m.targets.iter().map(|t| f32n(t.weight as f32)).collect()),
                        ),
                        (
                            "extras",
                            Json::Obj(vec![(
                                "targetNames",
                                Json::Arr(
                                    m.targets
                                        .iter()
                                        .map(|t| Json::Str(t.name.clone()))
                                        .collect(),
                                ),
                            )]),
                        ),
                    ];
                    morph_counts.insert(part.name.as_str(), m.targets.len());
                }
            }
        }
        let maps = map_indices(
            &part.material,
            &part.name,
            part.solid.uvs.is_some(),
            &mut textures,
        )?;
        let (material, used) = material_json(&part.material, &part.name, maps)?;
        material_extensions.extend(used);
        let same = |m: &Json| {
            let (mut a, mut b) = (String::new(), String::new());
            m.write(&mut a);
            material.write(&mut b);
            a == b
        };
        let index = match materials.iter().position(same).filter(|_| share) {
            Some(k) => k,
            None => {
                materials.push(material);
                materials.len() - 1
            }
        };
        let mesh_index = meshes.len() as u64;
        if !primitive.is_empty() {
            primitive.push(("material", Json::Int(index as u64)));
            primitive.push(("mode", Json::Int(4)));
            let mut mesh = vec![
                ("name", Json::Str(part.name.clone())),
                ("primitives", Json::Arr(vec![Json::Obj(primitive)])),
            ];
            mesh.extend(morph_fields);
            meshes.push(Json::Obj(mesh));
        }
        // Where the part's node hangs.
        let parent = if part.parent.is_empty() {
            None
        } else {
            Some(part_node[part.parent.as_str()])
        };
        let mut attach = |child: usize, children: &mut Vec<Vec<usize>>| match parent {
            Some(p) => children[p].push(child),
            None => roots.push(child),
        };
        let mut node = vec![("name", Json::Str(part.name.clone()))];
        if !pos.is_empty() {
            node.push(("mesh", Json::Int(mesh_index)));
            if skin.is_some() {
                node.push(("skin", Json::Int(skins.len() as u64)));
            }
        }
        match joint {
            Some(j) => {
                // The joint's node at the pivot; the part's node undoes the
                // pivot, so its mesh stays where it was built.
                let jn = nodes.len();
                joint_node.insert(j.name.as_str(), jn);
                let (t, r) = j.node.gltf();
                nodes.push(Json::Obj(vec![
                    ("name", Json::Str(j.name.clone())),
                    ("translation", vec3(t)),
                    ("rotation", nums(&r)),
                    ("extras", joint_extras(j)),
                ]));
                children.push(Vec::new());
                attach(jn, &mut children);
                let (t, r) = j.inverse.gltf();
                node.push(("translation", vec3(t)));
                node.push(("rotation", nums(&r)));
                let pn = nodes.len();
                nodes.push(Json::Obj(node));
                children.push(Vec::new());
                children[jn].push(pn);
                part_node.insert(part.name.as_str(), pn);
            }
            None => {
                let pn = nodes.len();
                nodes.push(Json::Obj(node));
                children.push(Vec::new());
                attach(pn, &mut children);
                part_node.insert(part.name.as_str(), pn);
                if let (Some(sk), false) = (skin, pos.is_empty()) {
                    // The bones hang under the part's node, at the origin.
                    let mut joints: Vec<usize> = Vec::with_capacity(sk.bones.len());
                    for (fields, parent) in crate::gltf::write::skin::bone_nodes(sk) {
                        let bn = nodes.len();
                        nodes.push(Json::Obj(fields));
                        children.push(Vec::new());
                        match parent {
                            Some(p) => children[joints[p as usize]].push(bn),
                            None => children[pn].push(bn),
                        }
                        joints.push(bn);
                    }
                    let roots_of = sk.bones.iter().filter(|b| b.parent.is_none()).count();
                    let skeleton = (roots_of == 1).then(|| joints[0]);
                    skin_nodes.insert(part.name.as_str(), (sk, joints.clone()));
                    skins.push((
                        crate::gltf::write::skin::inverse_binds(sk),
                        joints,
                        skeleton,
                    ));
                }
            }
        }
    }
    // Inverse bind matrices go in last: their views carry no target.
    let mut skins_json = Vec::with_capacity(skins.len());
    for (ibm, joints, skeleton) in skins {
        let start = bin.len();
        bin.extend_from_slice(&ibm);
        views.push(Json::Obj(vec![
            ("buffer", Json::Int(0)),
            ("byteOffset", Json::Int(start as u64)),
            ("byteLength", Json::Int(ibm.len() as u64)),
        ]));
        accessors.push(Json::Obj(vec![
            ("bufferView", Json::Int((views.len() - 1) as u64)),
            ("componentType", Json::Int(FLOAT)),
            ("count", Json::Int(joints.len() as u64)),
            ("type", Json::Str("MAT4".into())),
        ]));
        let mut fields = vec![(
            "inverseBindMatrices",
            Json::Int((accessors.len() - 1) as u64),
        )];
        if let Some(root) = skeleton {
            fields.push(("skeleton", Json::Int(root as u64)));
        }
        fields.push((
            "joints",
            Json::Arr(joints.iter().map(|&k| Json::Int(k as u64)).collect()),
        ));
        skins_json.push(Json::Obj(fields));
    }
    let mut animations = animations(
        clips,
        &checked,
        &joint_node,
        &mut bin,
        &mut views,
        &mut accessors,
    )?;
    animations.extend(skin_animations(
        skin_clips,
        &skin_nodes,
        (&part_node, &morph_counts),
        &mut bin,
        &mut views,
        &mut accessors,
    )?);
    let texture_fields = texture_json(&textures, &mut bin, &mut views);
    // Children lists go in last, now that every node exists.
    for (n, kids) in nodes.iter_mut().zip(&children) {
        if let (Json::Obj(fields), false) = (n, kids.is_empty()) {
            fields.push((
                "children",
                Json::Arr(kids.iter().map(|&k| Json::Int(k as u64)).collect()),
            ));
        }
    }
    let mut doc = vec![
        (
            "asset",
            Json::Obj(vec![
                ("version", Json::Str("2.0".into())),
                ("generator", Json::Str("og-shape".into())),
            ]),
        ),
        ("scene", Json::Int(0)),
        (
            "scenes",
            Json::Arr(vec![Json::Obj(vec![
                ("name", Json::Str(scene.name.clone())),
                (
                    "nodes",
                    Json::Arr(roots.iter().map(|&k| Json::Int(k as u64)).collect()),
                ),
            ])]),
        ),
        ("nodes", Json::Arr(nodes)),
    ];
    for (k, field) in extension_fields(&material_extensions, &textures)
        .into_iter()
        .enumerate()
    {
        doc.insert(1 + k, field);
    }
    if !meshes.is_empty() {
        doc.push(("meshes", Json::Arr(meshes)));
        doc.push(("materials", Json::Arr(materials)));
        doc.push(("accessors", Json::Arr(accessors)));
        if !skins_json.is_empty() {
            doc.push(("skins", Json::Arr(skins_json)));
        }
        if !animations.is_empty() {
            doc.push(("animations", Json::Arr(animations)));
        }
        doc.extend(texture_fields);
        doc.push(("bufferViews", Json::Arr(views)));
        doc.push((
            "buffers",
            Json::Arr(vec![Json::Obj(vec![(
                "byteLength",
                Json::Int(bin.len() as u64),
            )])]),
        ));
    }
    let mut text = String::new();
    Json::Obj(doc).write(&mut text);
    let mut json = text.into_bytes();
    while json.len() % 4 != 0 {
        json.push(b' ');
    }
    while bin.len() % 4 != 0 {
        bin.push(0);
    }
    let with_bin = !bin.is_empty();
    let total = 12 + 8 + json.len() + if with_bin { 8 + bin.len() } else { 0 };
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(b"glTF");
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&(total as u32).to_le_bytes());
    out.extend_from_slice(&(json.len() as u32).to_le_bytes());
    out.extend_from_slice(b"JSON");
    out.extend_from_slice(&json);
    if with_bin {
        out.extend_from_slice(&(bin.len() as u32).to_le_bytes());
        out.extend_from_slice(b"BIN\0");
        out.extend_from_slice(&bin);
    }
    Ok(out)
}

/// The extension a texture's image format needs, if it is not one of
/// glTF's core formats.
pub(crate) fn texture_extension(t: &Texture) -> Option<&'static str> {
    match t.mime.as_str() {
        "image/ktx2" => Some("KHR_texture_basisu"),
        "image/webp" => Some("EXT_texture_webp"),
        _ => None,
    }
}

/// `extensionsUsed` and `extensionsRequired` for what a file carries:
/// its materials' extensions (used), and textures in KTX2 or
/// WebP, written as they came (used and required: they have no image a
/// reader without the extension could take).
pub(crate) fn extension_fields(
    materials: &[&'static str],
    textures: &[&Texture],
) -> Vec<(&'static str, Json)> {
    let mut required: Vec<&'static str> = textures
        .iter()
        .filter_map(|t| texture_extension(t))
        .collect();
    required.sort_unstable();
    required.dedup();
    // The materials' extensions (used, never required: a reader
    // without one draws the material without it), then the textures'.
    let mut used: Vec<&'static str> = materials.to_vec();
    used.sort_unstable();
    used.dedup();
    used.extend(&required);
    let list = |names: &[&str]| Json::Arr(names.iter().map(|n| Json::Str((*n).into())).collect());
    let mut out = Vec::new();
    if !used.is_empty() {
        out.push(("extensionsUsed", list(&used)));
    }
    if !required.is_empty() {
        out.push(("extensionsRequired", list(&required)));
    }
    out
}

/// Checks that a texture can be written: PNG or JPEG (glTF's core
/// formats), or KTX2 or WebP (written as they came, under their
/// extensions), on a part with UVs.
pub(crate) fn check_texture(t: &Texture, part: &str, has_uvs: bool) -> Result<(), ExportError> {
    let why = if !has_uvs {
        "the part has no UVs to sample it with"
    } else if t.mime != "image/png" && t.mime != "image/jpeg" && texture_extension(t).is_none() {
        "only PNG, JPEG, KTX2 and WebP images are written"
    } else {
        return Ok(());
    };
    Err(ExportError::BadTexture {
        part: part.to_string(),
        why: why.to_string(),
    })
}

/// The textures' images, appended to `bin` (each 4-byte aligned) with
/// their buffer views, and the glTF `images`, `textures` and `samplers`
/// (one sampler: linear, repeat).
pub(crate) fn texture_json(
    textures: &[&Texture],
    bin: &mut Vec<u8>,
    views: &mut Vec<Json>,
) -> Vec<(&'static str, Json)> {
    if textures.is_empty() {
        return Vec::new();
    }
    let mut images = Vec::with_capacity(textures.len());
    for t in textures {
        while bin.len() % 4 != 0 {
            bin.push(0);
        }
        let start = bin.len();
        bin.extend_from_slice(&t.bytes);
        views.push(Json::Obj(vec![
            ("buffer", Json::Int(0)),
            ("byteOffset", Json::Int(start as u64)),
            ("byteLength", Json::Int(t.bytes.len() as u64)),
        ]));
        let mut image = vec![
            ("bufferView", Json::Int(views.len() as u64 - 1)),
            ("mimeType", Json::Str(t.mime.clone())),
        ];
        if !t.name.is_empty() {
            image.insert(0, ("name", Json::Str(t.name.clone())));
        }
        images.push(Json::Obj(image));
    }
    vec![
        ("images", Json::Arr(images)),
        (
            "samplers",
            Json::Arr(vec![Json::Obj(vec![
                ("magFilter", Json::Int(9729)),
                ("minFilter", Json::Int(9987)),
            ])]),
        ),
        (
            "textures",
            Json::Arr(
                textures
                    .iter()
                    .enumerate()
                    .map(|(k, t)| {
                        // A core image is the texture's source; another
                        // format's is under its extension.
                        let source = ("source", Json::Int(k as u64));
                        Json::Obj(vec![
                            ("sampler", Json::Int(0)),
                            match texture_extension(t) {
                                Some(ext) => (
                                    "extensions",
                                    Json::Obj(vec![(ext, Json::Obj(vec![source]))]),
                                ),
                                None => source,
                            },
                        ])
                    })
                    .collect(),
            ),
        ),
    ]
}

/// UVs as glTF writes them: f32, v flipped (glTF's origin is the top
/// left).
pub(crate) fn gltf_uv(q: og_shape::math::DVec2) -> [f32; 2] {
    [q.x as f32, (1.0 - q.y) as f32]
}
