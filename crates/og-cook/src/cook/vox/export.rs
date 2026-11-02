//! Voxel models and animation tracks exported as glTF binaries.

use super::*;

/// The glTF binary chunk being written: its bytes, buffer views and
/// accessors.
#[derive(Default)]
pub(crate) struct Bin {
    pub bytes: Vec<u8>,
    pub views: Vec<serde_json::Value>,
    pub accessors: Vec<serde_json::Value>,
}

impl Bin {
    /// Appends `data` (4-aligned) as a buffer view and an accessor over it
    /// (`accessor` gives everything but the view); answers the accessor.
    pub fn add(
        &mut self,
        data: &[u8],
        target: Option<u32>,
        mut accessor: serde_json::Value,
    ) -> usize {
        let view = self.view(data);
        if let Some(t) = target {
            self.views[view]["target"] = t.into();
        }
        accessor["bufferView"] = view.into();
        self.accessors.push(accessor);
        self.accessors.len() - 1
    }

    /// Appends `data` (4-aligned) as a buffer view with no accessor (an
    /// image's bytes); answers the view.
    pub fn view(&mut self, data: &[u8]) -> usize {
        while !self.bytes.len().is_multiple_of(4) {
            self.bytes.push(0);
        }
        self.views.push(serde_json::json!({
            "buffer": 0, "byteOffset": self.bytes.len(), "byteLength": data.len()
        }));
        self.bytes.extend_from_slice(data);
        self.views.len() - 1
    }
}

/// The clips of an animated file: (name, first frame, last frame).
fn clips_of(file: &VoxFile, settings: &VoxSettings) -> Result<Vec<(String, u32, u32)>, String> {
    let n = file.frame_count;
    if n == 0 {
        return match settings.clips.first() {
            Some(c) => Err(format!(
                "voxClip '{}': the file has no animation frames (an animated shape lists several models, each with its `_f` key frame)",
                c.name
            )),
            None => Ok(Vec::new()),
        };
    }
    if settings.clips.is_empty() {
        let (first, last) = file.playback.unwrap_or((0, n - 1));
        return Ok(vec![(DEFAULT_CLIP.to_string(), first, last)]);
    }
    settings
        .clips
        .iter()
        .map(|c| {
            if c.last >= n {
                Err(format!(
                    "voxClip '{}' ends at frame {}; the file has {n} frame(s), 0 to {}",
                    c.name,
                    c.last,
                    n - 1
                ))
            } else {
                Ok((c.name.clone(), c.first, c.last))
            }
        })
        .collect()
}

/// A `.vox` as a glTF binary: one mesh per placed model under a node at
/// its translation, each a flat-shaded greedy mesh with vertex colours and
/// the shared white material, `settings.scale` metres per voxel, its
/// origin where `settings.origin` says.
///
/// An animated file (frames: [`VoxFile::frame_count`]) is a glTF whose
/// clips swap meshes: each model of an animated shape is a mesh node of
/// its own, and each clip is a visibility track per model on its
/// mesh node (`OG_animation_visibility`: STEP, 1 on the frames the model
/// shows and 0 elsewhere), at `settings.fps` (one key per frame, and one
/// at the clip's end holding its last frame). A model that never moves
/// is a plain mesh node (no joint). A model a keyed transform moves
/// ([`Placement::track`]) is skinned whole to a joint of its own (at its
/// translation, scale 1, inverse bind the identity) with a STEP
/// translation channel on that joint, its place on each frame (held from
/// its nearest shown frame where it is folded away). The rest pose is
/// frame 0: the other frames' mesh nodes are hidden at rest
/// (`KHR_node_visibility`; a skinned one also scaled to 0 for a reader
/// without the extension), so the model drawn without a skeleton shows
/// frame 0 too.
pub fn to_glb(bytes: &[u8], settings: &VoxSettings) -> Result<Vec<u8>, String> {
    let file = parse(bytes)?;
    let clips = clips_of(&file, settings)?;
    let scale = settings.scale;
    let mut bin = Bin::default();
    let mut meshes = Vec::new();
    // Per placed model: its index, its mesh, its translation (glTF frame,
    // metres), the frames it shows (animated only) and its translation on
    // each of them (empty: it stays put).
    type Placed = (
        usize,
        usize,
        [f32; 3],
        Option<std::ops::Range<u32>>,
        Vec<[f32; 3]>,
    );
    let mut placed: Vec<Placed> = Vec::new();
    let mut joints = 0u16;
    let mut floor = f32::MAX;
    let meshed: Vec<Vec<Quad>> = file.models.iter().map(mesh_model).collect();
    for (pi, placement) in file.placements.iter().enumerate() {
        let model = &file.models[placement.model];
        let quads = &meshed[placement.model];
        if quads.is_empty() {
            continue;
        }
        // The model's pivot: its centre, floored, as the editor exports.
        let pivot = model.size.map(|s| (s / 2) as f32);
        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(quads.len() * 4);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(quads.len() * 4);
        let mut colors: Vec<[f32; 4]> = Vec::with_capacity(quads.len() * 4);
        let mut indices: Vec<u32> = Vec::with_capacity(quads.len() * 6);
        let rot = placement.rotation;
        let mirrored = determinant(rot) < 0;
        let turn = |p: [f32; 3]| -> [f32; 3] {
            let mut o = [0f32; 3];
            for i in 0..3 {
                o[i] = p[rot[i].0] * rot[i].1 as f32;
            }
            o
        };
        let mut lo = [f32::MAX; 3];
        let mut hi = [f32::MIN; 3];
        for q in quads {
            let base = positions.len() as u32;
            let c = file.palette[q.color as usize];
            let col = [linear(c[0]), linear(c[1]), linear(c[2]), 1.0];
            let n = turn(q.normal);
            // MagicaVoxel (x, y, z) z-up → glTF (x, z, -y) y-up.
            let n = [n[0], n[2], -n[1]];
            for corner in q.corners {
                let p = turn([
                    corner[0] - pivot[0],
                    corner[1] - pivot[1],
                    corner[2] - pivot[2],
                ]);
                let p = [p[0] * scale, p[2] * scale, -p[1] * scale];
                for i in 0..3 {
                    lo[i] = lo[i].min(p[i]);
                    hi[i] = hi[i].max(p[i]);
                }
                positions.push(p);
                normals.push(n);
                colors.push(col);
            }
            // (x, y, z) → (x, z, -y) is a rotation, so the winding stays
            // counter-clockwise from outside, unless the frame mirrors.
            if mirrored {
                indices.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
            } else {
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            }
        }
        let count = positions.len();
        let pos = bin.add(
            bytemuck::cast_slice(&positions),
            Some(34962),
            serde_json::json!({
                "componentType": 5126, "count": count, "type": "VEC3", "min": lo, "max": hi
            }),
        );
        let nrm = bin.add(
            bytemuck::cast_slice(&normals),
            Some(34962),
            serde_json::json!({ "componentType": 5126, "count": count, "type": "VEC3" }),
        );
        let col = bin.add(
            bytemuck::cast_slice(&colors),
            Some(34962),
            serde_json::json!({ "componentType": 5126, "count": count, "type": "VEC4" }),
        );
        let idx = bin.add(
            bytemuck::cast_slice(&indices),
            Some(34963),
            serde_json::json!({ "componentType": 5125, "count": indices.len(), "type": "SCALAR" }),
        );
        let mut attributes = serde_json::json!({ "POSITION": pos, "NORMAL": nrm, "COLOR_0": col });
        if placement.frames.is_some() && !placement.track.is_empty() {
            // Moved by a keyed transform: skinned whole to its own joint.
            let j: Vec<[u16; 4]> = vec![[joints, 0, 0, 0]; count];
            let w: Vec<[f32; 4]> = vec![[1.0, 0.0, 0.0, 0.0]; count];
            attributes["JOINTS_0"] = bin
                .add(
                    bytemuck::cast_slice(&j),
                    Some(34962),
                    serde_json::json!({ "componentType": 5123, "count": count, "type": "VEC4" }),
                )
                .into();
            attributes["WEIGHTS_0"] = bin
                .add(
                    bytemuck::cast_slice(&w),
                    Some(34962),
                    serde_json::json!({ "componentType": 5126, "count": count, "type": "VEC4" }),
                )
                .into();
            joints += 1;
        }
        meshes.push(serde_json::json!({
            "name": format!("vox{pi}"),
            "primitives": [{ "attributes": attributes, "indices": idx, "material": 0 }]
        }));
        // MagicaVoxel (x, y, z) z-up → glTF (x, z, -y) y-up, in metres.
        let to_gltf = |t: [i32; 3]| {
            [
                t[0] as f32 * scale,
                t[2] as f32 * scale,
                -(t[1] as f32) * scale,
            ]
        };
        let at = to_gltf(placement.translation);
        let track: Vec<[f32; 3]> = placement.track.iter().map(|t| to_gltf(*t)).collect();
        for p in std::iter::once(&at).chain(&track) {
            floor = floor.min(p[1] + lo[1]);
        }
        placed.push((pi, meshes.len() - 1, at, placement.frames.clone(), track));
    }
    if meshes.is_empty() {
        return Err("vox: every model is empty".into());
    }
    // `bottom`: every node lifted by the same height, the lowest voxel on 0.
    let lift = match settings.origin {
        VoxOrigin::Center => 0.0,
        VoxOrigin::Bottom => -floor,
    };
    let mut nodes: Vec<serde_json::Value> = Vec::new();
    // Per animated model: its mesh node, the frames it shows, and when a
    // keyed transform moves it, its joint's node and its translation on
    // each frame (lifted).
    type Animated = (usize, std::ops::Range<u32>, Option<(usize, Vec<[f32; 3]>)>);
    let mut animated: Vec<Animated> = Vec::new();
    for (pi, mesh, at, frames, track) in &placed {
        let at = [at[0], at[1] + lift, at[2]];
        let Some(f) = frames else {
            nodes.push(serde_json::json!({
                "name": format!("vox{pi}"), "mesh": mesh, "translation": at
            }));
            continue;
        };
        // Hidden at rest unless it is frame 0's.
        let shown = f.contains(&0);
        let hidden = serde_json::json!({ "KHR_node_visibility": { "visible": false } });
        if track.is_empty() {
            // A frame that never moves: a plain mesh node, swapped in and
            // out by its visibility track alone (no joint, no palette
            // matrix; drawn as a static instance when shown).
            let mut node = serde_json::json!({
                "name": format!("vox{pi}"), "mesh": mesh, "translation": at
            });
            if !shown {
                node["extensions"] = hidden;
            }
            nodes.push(node);
            animated.push((nodes.len() - 1, f.clone(), None));
            continue;
        }
        // Skinned to a joint of its own; the mesh node's scale says the
        // rest visibility to a reader without the extension (a skinned
        // mesh's node transform is otherwise unused).
        let s = if shown { 1.0 } else { 0.0 };
        let mut node = serde_json::json!({
            "name": format!("vox{pi}"), "mesh": mesh, "skin": 0,
            "translation": at, "scale": [s, s, s]
        });
        if !shown {
            node["extensions"] = hidden;
        }
        nodes.push(node);
        nodes.push(serde_json::json!({
            "name": format!("vox{pi}-frames"), "translation": at
        }));
        let track = track.iter().map(|t| [t[0], t[1] + lift, t[2]]).collect();
        animated.push((nodes.len() - 2, f.clone(), Some((nodes.len() - 1, track))));
    }
    let mut json = serde_json::json!({
        "asset": { "version": "2.0", "generator": "og-cook vox" },
        "scene": 0,
        "scenes": [{ "nodes": (0..nodes.len()).collect::<Vec<_>>() }],
        "nodes": nodes,
        "meshes": meshes,
        "materials": [{
            "name": "voxels",
            "pbrMetallicRoughness": { "baseColorFactor": [1.0, 1.0, 1.0, 1.0], "metallicFactor": 0.0, "roughnessFactor": 0.9 }
        }],
    });
    let joint_nodes: Vec<usize> = animated
        .iter()
        .filter_map(|(_, _, j)| j.as_ref().map(|(n, _)| *n))
        .collect();
    if !joint_nodes.is_empty() {
        let identity: Vec<f32> = (0..joint_nodes.len()).flat_map(|_| IDENTITY_MAT4).collect();
        let ibm = bin.add(
            bytemuck::cast_slice(&identity),
            None,
            serde_json::json!({ "componentType": 5126, "count": joint_nodes.len(), "type": "MAT4" }),
        );
        json["skins"] = serde_json::json!([{
            "name": "frames",
            "joints": joint_nodes,
            "inverseBindMatrices": ibm
        }]);
    }
    if !animated.is_empty() {
        let mut animations = Vec::new();
        for (name, first, last) in &clips {
            // One key per frame, and one at the end holding the last.
            let len = last - first + 1;
            let times: Vec<f32> = (0..=len).map(|i| i as f32 / settings.fps).collect();
            let input = bin.add(
                bytemuck::cast_slice(&times),
                None,
                serde_json::json!({
                    "componentType": 5126, "count": times.len(), "type": "SCALAR",
                    "min": [0.0], "max": [times[times.len() - 1]]
                }),
            );
            let mut samplers = Vec::new();
            let mut channels = Vec::new();
            let mut visibility = Vec::new();
            for (mesh_node, frames, joint) in &animated {
                let shown: Vec<f32> = (0..=len)
                    .map(|i| {
                        let shown = frames.contains(&(first + i.min(len - 1)));
                        if shown { 1.0 } else { 0.0 }
                    })
                    .collect();
                let output = bin.add(
                    bytemuck::cast_slice(&shown),
                    None,
                    serde_json::json!({ "componentType": 5126, "count": shown.len(), "type": "SCALAR" }),
                );
                visibility.push(serde_json::json!({
                    "sampler": samplers.len(), "node": mesh_node
                }));
                samplers.push(serde_json::json!({
                    "input": input, "output": output, "interpolation": "STEP"
                }));
                let Some((node, track)) = joint else {
                    continue;
                };
                // Where the keyed transform puts the model on each frame;
                // on a frame it is folded away, where it was or will be
                // nearest (a fold is not a move).
                let places: Vec<[f32; 3]> = (0..=len)
                    .map(|i| {
                        let f = (first + i.min(len - 1)).clamp(frames.start, frames.end - 1);
                        track[(f - frames.start) as usize]
                    })
                    .collect();
                let output = bin.add(
                    bytemuck::cast_slice(&places),
                    None,
                    serde_json::json!({ "componentType": 5126, "count": places.len(), "type": "VEC3" }),
                );
                channels.push(serde_json::json!({
                    "sampler": samplers.len(), "target": { "node": node, "path": "translation" }
                }));
                samplers.push(serde_json::json!({
                    "input": input, "output": output, "interpolation": "STEP"
                }));
            }
            animations.push(serde_json::json!({
                "name": name, "samplers": samplers, "channels": channels,
                "extensions": { (VISIBILITY): { "channels": visibility } }
            }));
        }
        json["animations"] = animations.into();
        json["extensionsUsed"] = serde_json::json!(["KHR_node_visibility", VISIBILITY]);
    }
    bin.finish(json)
}

impl Bin {
    /// The glTF binary of `json` (its accessors, views and buffer set from
    /// this chunk) and this chunk.
    pub fn finish(mut self, mut json: serde_json::Value) -> Result<Vec<u8>, String> {
        while !self.bytes.len().is_multiple_of(4) {
            self.bytes.push(0);
        }
        json["accessors"] = std::mem::take(&mut self.accessors).into();
        json["bufferViews"] = std::mem::take(&mut self.views).into();
        json["buffers"] = serde_json::json!([{ "byteLength": self.bytes.len() }]);
        let bin = self.bytes;
        let mut json_bytes = serde_json::to_vec(&json).map_err(|e| e.to_string())?;
        while !json_bytes.len().is_multiple_of(4) {
            json_bytes.push(b' ');
        }
        let total = 12 + 8 + json_bytes.len() + 8 + bin.len();
        let mut out = Vec::with_capacity(total);
        out.extend_from_slice(b"glTF");
        out.extend_from_slice(&2u32.to_le_bytes());
        out.extend_from_slice(&(total as u32).to_le_bytes());
        out.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
        out.extend_from_slice(b"JSON");
        out.extend_from_slice(&json_bytes);
        out.extend_from_slice(&(bin.len() as u32).to_le_bytes());
        out.extend_from_slice(b"BIN\0");
        out.extend_from_slice(&bin);
        Ok(out)
    }
}
