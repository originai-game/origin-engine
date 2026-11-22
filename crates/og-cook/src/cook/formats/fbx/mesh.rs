//! A ufbx mesh as glTF primitives: triangulated corners welded into
//! vertices, one primitive per material, its skin and its morph targets.

use super::*;
use glam::{DMat3, DVec2, DVec4};

/// A primitive's vertices before they are written.
#[derive(Default)]
struct Prim {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uv0: Vec<[f32; 2]>,
    uv1: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,
    /// The control point each vertex came from (its skin and morph data).
    points: Vec<u32>,
    indices: Vec<u32>,
}

/// A morph target: its name, per control point its position and normal
/// deltas, and its default weight.
struct Target {
    name: String,
    positions: Vec<DVec3>,
    normals: Option<Vec<DVec3>>,
    weight: f64,
}

/// Per control point: its four joints and their weights.
type SkinData = (Vec<[u16; 4]>, Vec<[f32; 4]>);

/// sRGB to linear, as the DCC shows a vertex colour.
fn linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn bounds(v: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];
    for p in v {
        for i in 0..3 {
            lo[i] = lo[i].min(p[i]);
            hi[i] = hi[i].max(p[i]);
        }
    }
    (lo, hi)
}

impl Writer<'_> {
    /// The mesh `m` shown by node `n` (glTF node `me`): a glTF mesh on
    /// that node, or, skinned, on a root node of its own with its skin.
    pub(super) fn write_node_mesh(
        &mut self,
        n: &ufbx::Node,
        m: &ufbx::Mesh,
        me: usize,
    ) -> Result<(), String> {
        if m.num_triangles == 0 {
            return Ok(());
        }
        let mats: Vec<&ufbx::Material> = if n.materials.is_empty() {
            m.materials.iter().map(|r| &**r).collect()
        } else {
            n.materials.iter().map(|r| &**r).collect()
        };
        let skin = m.skin_deformers.first();
        if m.skin_deformers.len() > 1 {
            self.notes.push(format!(
                "{}: mesh '{}' has {} skin deformers; the first is kept",
                self.what,
                m.element.name,
                m.skin_deformers.len()
            ));
        }
        let key = (
            m.element.typed_id,
            mats.iter().map(|x| x.element.typed_id).collect::<Vec<_>>(),
        );
        if skin.is_none()
            && let Some(&mesh) = self.mesh_of.get(&key)
        {
            self.nodes[me]["mesh"] = mesh.into();
            self.record_morphs(m, me);
            return Ok(());
        }
        // A plain mesh is drawn by its node: its geometry transform, if
        // ufbx kept one, is baked in. A skinned mesh is drawn by its joints.
        let to_node = if skin.is_some() {
            DMat4::IDENTITY
        } else {
            dmat4(&n.geometry_to_node)
        };
        let mesh = self.write_mesh(n, m, &mats, to_node, skin.map(|s| &**s))?;
        let target = match skin {
            None => {
                self.mesh_of.insert(key, mesh);
                me
            }
            Some(_) => {
                let node = self.nodes.len();
                self.nodes.push(json!({
                    "name": n.element.name.to_string(),
                    "mesh": mesh,
                    "skin": self.skins.len() - 1,
                }));
                node
            }
        };
        self.nodes[target]["mesh"] = mesh.into();
        self.record_morphs(m, target);
        Ok(())
    }

    /// Where each of the mesh's blend channels lands: the node, its first
    /// target and the weights its shapes are whole at.
    fn record_morphs(&mut self, m: &ufbx::Mesh, node: usize) {
        let channels: Vec<(u32, Vec<f64>)> = morph_channels(m)
            .iter()
            .map(|c| {
                (
                    c.element.element_id,
                    shapes(c).into_iter().map(|s| s.0).collect(),
                )
            })
            .collect();
        let count = channels.iter().map(|c| c.1.len()).sum();
        let mut first = 0;
        for (c, keys) in channels {
            let n = keys.len();
            self.morph_of.entry(c).or_default().push(MorphSlot {
                node,
                first,
                shapes: keys,
                count,
            });
            first += n;
        }
    }

    fn write_mesh(
        &mut self,
        n: &ufbx::Node,
        m: &ufbx::Mesh,
        mats: &[&ufbx::Material],
        to_node: DMat4,
        skin: Option<&ufbx::SkinDeformer>,
    ) -> Result<usize, String> {
        let normal_m = DMat3::from_mat4(to_node).inverse().transpose();
        let uv0 = m.uv_sets.first().map(|s| &s.vertex_uv);
        let uv1 = m.uv_sets.get(1).map(|s| &s.vertex_uv);
        let color = m.color_sets.first().map(|s| &s.vertex_color);
        let mut tri = Vec::new();
        let mut prims: Vec<(Option<usize>, Prim)> = Vec::new();
        for part in m.material_parts.iter() {
            if part.num_triangles == 0 {
                continue;
            }
            let mut prim = Prim::default();
            let mut welded: HashMap<[u32; 12], u32> = HashMap::new();
            for &fi in part.face_indices.iter() {
                if m.face_hole.get(fi as usize).copied().unwrap_or(false) {
                    continue;
                }
                let face = m.faces[fi as usize];
                if ufbx::triangulate_face_vec(&mut tri, m, face) == 0 {
                    continue;
                }
                for &i in &tri {
                    let i = i as usize;
                    let p = to_node.transform_point3(vec3(m.vertex_position[i]));
                    let nrm = if m.vertex_normal.exists {
                        (normal_m * vec3(m.vertex_normal[i])).normalize_or_zero()
                    } else {
                        DVec3::ZERO
                    };
                    let flip = |v: ufbx::Vec2| DVec2::new(v.x, 1.0 - v.y);
                    let t0 = uv0.filter(|u| u.exists).map_or(DVec2::ZERO, |u| flip(u[i]));
                    let t1 = uv1.filter(|u| u.exists).map_or(DVec2::ZERO, |u| flip(u[i]));
                    let c = color.filter(|c| c.exists).map_or(DVec4::ONE, |c| {
                        let v = c[i];
                        DVec4::new(linear(v.x), linear(v.y), linear(v.z), v.w)
                    });
                    let point = m.vertex_indices[i];
                    let (p, nrm, t0, t1, c) = (
                        p.as_vec3().to_array(),
                        nrm.as_vec3().to_array(),
                        t0.as_vec2().to_array(),
                        t1.as_vec2().to_array(),
                        c.as_vec4().to_array(),
                    );
                    let key = [
                        point,
                        nrm[0].to_bits(),
                        nrm[1].to_bits(),
                        nrm[2].to_bits(),
                        t0[0].to_bits(),
                        t0[1].to_bits(),
                        t1[0].to_bits(),
                        t1[1].to_bits(),
                        c[0].to_bits(),
                        c[1].to_bits(),
                        c[2].to_bits(),
                        c[3].to_bits(),
                    ];
                    let next = prim.positions.len() as u32;
                    let v = *welded.entry(key).or_insert_with(|| {
                        prim.positions.push(p);
                        prim.normals.push(nrm);
                        prim.uv0.push(t0);
                        prim.uv1.push(t1);
                        prim.colors.push(c);
                        prim.points.push(point);
                        next
                    });
                    prim.indices.push(v);
                }
            }
            if prim.indices.is_empty() {
                continue;
            }
            let mat = match mats.get(part.index as usize) {
                Some(mat) => Some(self.materials.material(
                    mat,
                    self.dir,
                    self.what,
                    &mut self.read,
                    self.notes,
                )?),
                None => None,
            };
            prims.push((mat, prim));
        }
        // Skin: per control point, its four strongest joints.
        let skin_data = match skin {
            Some(s) => Some(self.write_skin(n, m, s)?),
            None => None,
        };
        // Morph targets: per channel, per control point, its deltas.
        let channels = morph_channels(m);
        let mut targets: Vec<Target> = Vec::new();
        for c in &channels {
            // Each shape is a target: the full one (the largest weight)
            // bears the channel's name, an in-between `name@<percent>`;
            // the channel's weight is spread over them as the DCC does.
            let shapes = shapes(c);
            let keys: Vec<f64> = shapes.iter().map(|s| s.0).collect();
            let rest = shape_weights(c.weight, &keys);
            let top = keys.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            for (k, (at, shape)) in shapes.iter().enumerate() {
                let mut pos = vec![DVec3::ZERO; m.num_vertices];
                let mut nrm =
                    (!shape.normal_offsets.is_empty()).then(|| vec![DVec3::ZERO; m.num_vertices]);
                for i in 0..shape.num_offsets {
                    let v = shape.offset_vertices[i] as usize;
                    if v < pos.len() {
                        pos[v] = to_node.transform_vector3(vec3(shape.position_offsets[i]));
                        if let Some(nrm) = &mut nrm {
                            nrm[v] = normal_m * vec3(shape.normal_offsets[i]);
                        }
                    }
                }
                let name = c.element.name.to_string();
                targets.push(Target {
                    name: if *at == top {
                        name
                    } else {
                        // In percent, to a thousandth.
                        format!("{name}@{}", (at * 100_000.0).round() / 1000.0)
                    },
                    positions: pos,
                    normals: nrm,
                    weight: rest[k],
                });
            }
        }
        let mut primitives = Vec::new();
        for (mat, prim) in &prims {
            let (lo, hi) = bounds(&prim.positions);
            let pos = self.bin.add(
                &f32s(prim.positions.as_flattened()),
                Some(34962),
                json!({ "componentType": 5126, "count": prim.positions.len(), "type": "VEC3", "min": lo, "max": hi }),
            );
            let count = prim.positions.len();
            let mut attributes = json!({ "POSITION": pos });
            if m.vertex_normal.exists {
                attributes["NORMAL"] = self
                    .bin
                    .add(
                        &f32s(prim.normals.as_flattened()),
                        Some(34962),
                        json!({ "componentType": 5126, "count": count, "type": "VEC3" }),
                    )
                    .into();
            }
            if uv0.is_some_and(|u| u.exists) {
                attributes["TEXCOORD_0"] = self
                    .bin
                    .add(
                        &f32s(prim.uv0.as_flattened()),
                        Some(34962),
                        json!({ "componentType": 5126, "count": count, "type": "VEC2" }),
                    )
                    .into();
            }
            if uv1.is_some_and(|u| u.exists) {
                attributes["TEXCOORD_1"] = self
                    .bin
                    .add(
                        &f32s(prim.uv1.as_flattened()),
                        Some(34962),
                        json!({ "componentType": 5126, "count": count, "type": "VEC2" }),
                    )
                    .into();
            }
            if color.is_some_and(|c| c.exists) {
                attributes["COLOR_0"] = self
                    .bin
                    .add(
                        &f32s(prim.colors.as_flattened()),
                        Some(34962),
                        json!({ "componentType": 5126, "count": count, "type": "VEC4" }),
                    )
                    .into();
            }
            if let Some((joints, weights)) = &skin_data {
                let j: Vec<u16> = prim
                    .points
                    .iter()
                    .flat_map(|&p| joints[p as usize])
                    .collect();
                let w: Vec<f32> = prim
                    .points
                    .iter()
                    .flat_map(|&p| weights[p as usize])
                    .collect();
                attributes["JOINTS_0"] = self
                    .bin
                    .add(
                        bytemuck::cast_slice(&j),
                        Some(34962),
                        json!({ "componentType": 5123, "count": count, "type": "VEC4" }),
                    )
                    .into();
                attributes["WEIGHTS_0"] = self
                    .bin
                    .add(
                        &f32s(&w),
                        Some(34962),
                        json!({ "componentType": 5126, "count": count, "type": "VEC4" }),
                    )
                    .into();
            }
            let indices = self.bin.add(
                bytemuck::cast_slice(&prim.indices),
                Some(34963),
                json!({ "componentType": 5125, "count": prim.indices.len(), "type": "SCALAR" }),
            );
            let mut p = json!({ "attributes": attributes, "indices": indices });
            if let Some(mat) = mat {
                p["material"] = (*mat).into();
            }
            if !targets.is_empty() {
                let mut ts = Vec::new();
                for Target {
                    positions: pos,
                    normals: nrm,
                    ..
                } in &targets
                {
                    let d: Vec<[f32; 3]> = prim
                        .points
                        .iter()
                        .map(|&v| pos[v as usize].as_vec3().to_array())
                        .collect();
                    let (lo, hi) = bounds(&d);
                    let mut t = json!({
                        "POSITION": self.bin.add(&f32s(d.as_flattened()), Some(34962), json!({ "componentType": 5126, "count": count, "type": "VEC3", "min": lo, "max": hi }))
                    });
                    if let Some(nrm) = nrm
                        && m.vertex_normal.exists
                    {
                        let d: Vec<f32> = prim
                            .points
                            .iter()
                            .flat_map(|&v| nrm[v as usize].as_vec3().to_array())
                            .collect();
                        t["NORMAL"] = self
                            .bin
                            .add(
                                &f32s(&d),
                                Some(34962),
                                json!({ "componentType": 5126, "count": count, "type": "VEC3" }),
                            )
                            .into();
                    }
                    ts.push(t);
                }
                p["targets"] = ts.into();
            }
            primitives.push(p);
        }
        let mut mesh = json!({ "name": m.element.name.to_string(), "primitives": primitives });
        if !targets.is_empty() {
            mesh["weights"] = targets
                .iter()
                .map(|t| t.weight as f32)
                .collect::<Vec<_>>()
                .into();
            mesh["extras"] = json!({ "targetNames": targets.iter().map(|t| t.name.clone()).collect::<Vec<_>>() });
        }
        self.meshes.push(mesh);
        Ok(self.meshes.len() - 1)
    }

    /// A glTF skin for `s` (its clusters' bones as joints, their
    /// geometry-to-bone matrices as inverse binds), and per control point
    /// its four strongest joints and weights, normalised. A vertex no
    /// cluster weights follows the mesh's own node, an extra joint whose
    /// inverse bind is the geometry-to-node matrix.
    fn write_skin(
        &mut self,
        n: &ufbx::Node,
        m: &ufbx::Mesh,
        s: &ufbx::SkinDeformer,
    ) -> Result<SkinData, String> {
        let mut joints: Vec<usize> = Vec::new();
        let mut ibms: Vec<f32> = Vec::new();
        let mut joint_of: Vec<Option<u16>> = Vec::new();
        for c in s.clusters.iter() {
            match &c.bone_node {
                Some(b) => {
                    joint_of.push(Some(joints.len() as u16));
                    joints.push(self.node_of[&b.element.typed_id]);
                    ibms.extend(mat4(&c.geometry_to_bone));
                }
                None => joint_of.push(None),
            }
        }
        let mut out_j = vec![[0u16; 4]; m.num_vertices];
        let mut out_w = vec![[0f32; 4]; m.num_vertices];
        let mut own_joint: Option<u16> = None;
        let mut dropped = 0usize;
        for v in 0..m.num_vertices {
            let mut ws: Vec<(u16, f64)> = Vec::new();
            if let Some(sv) = s.vertices.get(v) {
                for k in sv.weight_begin..sv.weight_begin + sv.num_weights {
                    let w = s.weights[k as usize];
                    if let Some(Some(j)) = joint_of.get(w.cluster_index as usize)
                        && w.weight > 0.0
                    {
                        ws.push((*j, w.weight));
                    }
                }
            }
            ws.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            if ws.len() > 4 {
                dropped += 1;
                ws.truncate(4);
            }
            let total: f64 = ws.iter().map(|w| w.1).sum();
            if ws.is_empty() || total <= 0.0 {
                let j = *own_joint.get_or_insert_with(|| {
                    joints.push(self.node_of[&n.element.typed_id]);
                    ibms.extend(mat4(&n.geometry_to_node));
                    (joints.len() - 1) as u16
                });
                out_j[v] = [j, 0, 0, 0];
                out_w[v] = [1.0, 0.0, 0.0, 0.0];
                continue;
            }
            for (i, (j, w)) in ws.iter().enumerate() {
                out_j[v][i] = *j;
                out_w[v][i] = (w / total) as f32;
            }
        }
        if dropped > 0 {
            self.notes.push(format!(
                "{}: mesh '{}': {dropped} vertices have more than 4 joint weights; the 4 strongest are kept, normalised",
                self.what, m.element.name
            ));
        }
        if joints.is_empty() {
            return Err(format!(
                "{}: mesh '{}' has a skin with no bones; bind it to its skeleton again before exporting",
                self.what, m.element.name
            ));
        }
        let ibm = self.bin.add(
            &f32s(&ibms),
            None,
            json!({ "componentType": 5126, "count": joints.len(), "type": "MAT4" }),
        );
        self.skins.push(json!({
            "name": s.element.name.to_string(),
            "joints": joints,
            "inverseBindMatrices": ibm,
        }));
        Ok((out_j, out_w))
    }
}

/// The blend channels of a mesh's blend deformers, in order.
fn morph_channels(m: &ufbx::Mesh) -> Vec<&ufbx::BlendChannel> {
    m.blend_deformers
        .iter()
        .flat_map(|d| d.channels.iter().map(|c| &**c))
        .filter(|c| c.target_shape.is_some())
        .collect()
}

/// A blend channel's shapes with the weight each is whole at, rising:
/// its keyframes (an in-between before the full shape), or its target
/// shape alone at 1.
fn shapes(c: &ufbx::BlendChannel) -> Vec<(f64, &ufbx::BlendShape)> {
    let mut out: Vec<(f64, &ufbx::BlendShape)> = c
        .keyframes
        .iter()
        .map(|k| (k.target_weight, &*k.shape))
        .filter(|(w, _)| w.is_finite() && *w != 0.0)
        .collect();
    if out.is_empty()
        && let Some(s) = &c.target_shape
    {
        out.push((1.0, &**s));
    }
    out.sort_by(|a, b| a.0.total_cmp(&b.0));
    out
}

/// How much of each of a blend channel's shapes there is at the
/// channel's `weight`: the shapes are whole at `keys` (rising), and
/// between two of them, or between none and the first, the channel is
/// part one and part the other; past the last it runs on as between
/// the last two.
pub(super) fn shape_weights(weight: f64, keys: &[f64]) -> Vec<f64> {
    let n = keys.len();
    let mut out = vec![0.0; n];
    // The shapes below none (negative weights) end here.
    let below = keys.iter().take_while(|k| **k < 0.0).count();
    // `None` is no shape at all, at weight 0.
    let (mut prev, mut next): (Option<usize>, Option<usize>) = (None, None);
    if weight > 0.0 {
        prev = below.checked_sub(1);
        for (i, key) in keys.iter().enumerate().skip(below) {
            prev = next;
            next = Some(i);
            if *key > weight {
                break;
            }
        }
    } else {
        if below < n {
            prev = Some(below);
        }
        for i in (0..below).rev() {
            prev = next;
            next = Some(i);
            if keys[i] < weight {
                break;
            }
        }
    }
    let at = |k: Option<usize>| k.map_or(0.0, |i| keys[i]);
    let span = at(next) - at(prev);
    if span != 0.0 {
        let t = (weight - at(prev)) / span;
        if let Some(i) = prev {
            out[i] = 1.0 - t;
        }
        if let Some(i) = next {
            out[i] = t;
        }
    }
    out
}

fn dmat4(m: &ufbx::Matrix) -> DMat4 {
    DMat4::from_cols_array(&[
        m.m00, m.m10, m.m20, 0.0, m.m01, m.m11, m.m21, 0.0, m.m02, m.m12, m.m22, 0.0, m.m03, m.m13,
        m.m23, 1.0,
    ])
}
