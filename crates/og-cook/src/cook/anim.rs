//! glTF skins, animations and morph targets → the `<model>#anim` blob
//! (`og.Animation`, `format.rs`): the node hierarchy in the model blob's
//! order, skins with their inverse bind matrices, clips resampled at a
//! fixed rate, the skinned primitives' joints and weights per cooked vertex
//! and the morph targets' deltas per cooked vertex.
//!
//! Resampling evaluates the glTF sampler in f64 exactly as the
//! specification defines it (STEP; LINEAR, with shortest-arc slerp for
//! rotations; CUBICSPLINE Hermite, rotations normalised), at `i / rate` for
//! `i` in `0..frames` and at the clip's end; a key within a microsecond of
//! a sample time counts as reached. `rate` is the import settings'
//! `animation { rate }`, else the source's own key rate rounded (1 / its
//! smallest key interval), clamped to 1..=120 Hz; the source rate is kept.

use super::model::{self, GltfSource, Strings};
use super::{CookError, MeshOptions};
use crate::format::{self, *};
use og_blob::BlobWriter;

/// Per-vertex deltas of one morph target.
type Deltas = Vec<[f32; 3]>;

/// `animation { … }` import settings of a glTF.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimSettings {
    /// Resampling rate for every clip (Hz); the source's own rate when absent.
    pub rate: Option<f32>,
    /// Extra markers: (clip name, marker name, seconds).
    pub events: Vec<(String, String, f32)>,
}

impl AnimSettings {
    pub fn describe(&self) -> String {
        let mut s = format!(
            "rate {} resample f64 skin u16x4/unorm16x4 morph f32 codec meshopt-v1",
            self.rate.map_or("source".to_string(), |r| r.to_string())
        );
        for (c, n, t) in &self.events {
            s.push_str(&format!(" event {c:?} {n:?} {t}"));
        }
        s
    }
}

/// Whether a glTF has anything for an animation blob.
pub fn wanted(src: &GltfSource) -> bool {
    src.doc.skins().next().is_some()
        || src.doc.animations().next().is_some()
        || src
            .doc
            .meshes()
            .flat_map(|m| m.primitives())
            .any(|p| p.morph_targets().next().is_some())
}

/// The glTF JSON (for `extras`, which the gltf crate is built without).
fn json(bytes: &[u8]) -> Option<serde_json::Value> {
    if bytes.len() >= 20 && &bytes[0..4] == b"glTF" {
        let len = u32::from_le_bytes(bytes[12..16].try_into().ok()?) as usize;
        serde_json::from_slice(bytes.get(20..20 + len)?).ok()
    } else {
        serde_json::from_slice(bytes).ok()
    }
}

/// The value of a sampler at `t`, `lanes` floats, per the glTF specification.
fn evaluate(
    interpolation: u32,
    times: &[f32],
    values: &[f32],
    lanes: usize,
    rotation: bool,
    t: f64,
) -> Vec<f64> {
    let cubic = interpolation == interpolation::CUBICSPLINE;
    let stride = if cubic { 3 * lanes } else { lanes };
    let value = |k: usize| -> Vec<f64> {
        let at = k * stride + if cubic { lanes } else { 0 };
        values[at..at + lanes].iter().map(|&v| v as f64).collect()
    };
    let n = times.len();
    let reached = |k: usize| (times[k] as f64) <= t + 1e-6;
    if n == 1 || !reached(0) {
        return finish(value(0), rotation);
    }
    if reached(n - 1) {
        return finish(value(n - 1), rotation);
    }
    let mut k = 0;
    while k + 1 < n && reached(k + 1) {
        k += 1;
    }
    let (t0, t1) = (times[k] as f64, times[k + 1] as f64);
    let dt = t1 - t0;
    let u = if dt > 0.0 {
        ((t - t0) / dt).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (a, b) = (value(k), value(k + 1));
    let out = match interpolation {
        interpolation::STEP => a,
        interpolation::CUBICSPLINE => {
            let out_tangent = &values[k * stride + 2 * lanes..k * stride + 3 * lanes];
            let in_tangent = &values[(k + 1) * stride..(k + 1) * stride + lanes];
            let (u2, u3) = (u * u, u * u * u);
            (0..lanes)
                .map(|i| {
                    (2.0 * u3 - 3.0 * u2 + 1.0) * a[i]
                        + (u3 - 2.0 * u2 + u) * dt * out_tangent[i] as f64
                        + (-2.0 * u3 + 3.0 * u2) * b[i]
                        + (u3 - u2) * dt * in_tangent[i] as f64
                })
                .collect()
        }
        _ if rotation => slerp(&a, &b, u),
        _ => (0..lanes).map(|i| a[i] + (b[i] - a[i]) * u).collect(),
    };
    finish(out, rotation)
}

fn finish(mut v: Vec<f64>, rotation: bool) -> Vec<f64> {
    if rotation {
        let len = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        if len > 0.0 {
            v.iter_mut().for_each(|x| *x /= len);
        }
    }
    v
}

fn slerp(a: &[f64], b: &[f64], u: f64) -> Vec<f64> {
    let mut d: f64 = (0..4).map(|i| a[i] * b[i]).sum();
    let mut b = b.to_vec();
    if d < 0.0 {
        d = -d;
        b.iter_mut().for_each(|x| *x = -*x);
    }
    if d > 0.999_999 {
        return (0..4).map(|i| a[i] + (b[i] - a[i]) * u).collect();
    }
    let theta = d.acos();
    let s = theta.sin();
    let (wa, wb) = (((1.0 - u) * theta).sin() / s, (u * theta).sin() / s);
    (0..4).map(|i| wa * a[i] + wb * b[i]).collect()
}

struct SourceChannel {
    node: u32,
    path: u32,
    interpolation: u32,
    lanes: usize,
    times: Vec<f32>,
    values: Vec<f32>,
}

fn read_channels(
    src: &GltfSource,
    anim: &gltf::Animation,
    new_index: &[u32],
) -> Result<Vec<SourceChannel>, String> {
    let mut out = Vec::new();
    for (ci, ch) in anim.channels().enumerate() {
        let reader = ch.reader(|b| Some(&src.buffers[b.index()].0[..]));
        let times: Vec<f32> = reader
            .read_inputs()
            .ok_or_else(|| format!("channel {ci} has no input"))?
            .collect();
        use gltf::animation::util::ReadOutputs;
        let (path, lanes, values): (u32, usize, Vec<f32>) = match reader
            .read_outputs()
            .ok_or_else(|| format!("channel {ci} has no output"))?
        {
            ReadOutputs::Translations(v) => (path::TRANSLATION, 3, v.flatten().collect()),
            ReadOutputs::Rotations(v) => (path::ROTATION, 4, v.into_f32().flatten().collect()),
            ReadOutputs::Scales(v) => (path::SCALE, 3, v.flatten().collect()),
            ReadOutputs::MorphTargetWeights(v) => {
                let all: Vec<f32> = v.into_f32().collect();
                let per_key = if ch.sampler().interpolation()
                    == gltf::animation::Interpolation::CubicSpline
                {
                    3
                } else {
                    1
                };
                let lanes = all.len() / (times.len() * per_key).max(1);
                (path::WEIGHTS, lanes.max(1), all)
            }
        };
        let interpolation = match ch.sampler().interpolation() {
            gltf::animation::Interpolation::Linear => interpolation::LINEAR,
            gltf::animation::Interpolation::Step => interpolation::STEP,
            gltf::animation::Interpolation::CubicSpline => interpolation::CUBICSPLINE,
        };
        let per_key = if interpolation == interpolation::CUBICSPLINE {
            3
        } else {
            1
        };
        if times.is_empty() || values.len() != times.len() * per_key * lanes {
            return Err(format!(
                "channel {ci}: {} keys but {} output floats for {lanes} lanes",
                times.len(),
                values.len()
            ));
        }
        if times.windows(2).any(|w| w[1] < w[0]) || times.iter().any(|t| !t.is_finite()) {
            return Err(format!("channel {ci}: key times are not increasing"));
        }
        out.push(SourceChannel {
            node: new_index[ch.target().node().index()],
            path,
            interpolation,
            lanes,
            times,
            values,
        });
    }
    Ok(out)
}

fn events_of(
    json: Option<&serde_json::Value>,
    anim_index: usize,
    clip: &str,
    settings: &AnimSettings,
    duration: f32,
) -> Result<Vec<(String, f32)>, String> {
    let mut events: Vec<(String, f32)> = Vec::new();
    if let Some(list) = json
        .and_then(|j| {
            j.get("animations")?
                .get(anim_index)?
                .get("extras")?
                .get("og.events")
        })
        .and_then(|e| e.as_array())
    {
        for e in list {
            let name = e.get("name").and_then(|n| n.as_str());
            let time = e.get("time").and_then(|t| t.as_f64());
            match (name, time) {
                (Some(n), Some(t)) => events.push((n.to_string(), t as f32)),
                _ => {
                    return Err(format!(
                        "clip '{clip}': an extras.\"og.events\" entry needs \"name\" and \"time\", got {e}"
                    ))
                }
            }
        }
    }
    for (c, n, t) in &settings.events {
        if c == clip {
            events.push((n.clone(), *t));
        }
    }
    for (n, t) in &events {
        if n.is_empty() || n.len() > 64 || n.contains('\0') {
            return Err(format!(
                "clip '{clip}': event '{n}' must be 1..=64 bytes of UTF-8"
            ));
        }
        if !(0.0..=duration).contains(t) {
            return Err(format!(
                "clip '{clip}': event '{n}' at {t} s is outside the clip (0..={duration} s)"
            ));
        }
    }
    events.sort_by(|a, b| a.1.total_cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
    Ok(events)
}

/// Quantises weights to unorm16 summing to exactly 65535 (the remainder on
/// the largest weight).
fn quantize_weights(w: [f32; 4]) -> [u16; 4] {
    let sum: f32 = w.iter().map(|x| x.max(0.0)).sum();
    if sum <= 0.0 {
        return [65535, 0, 0, 0];
    }
    let mut q = [0u16; 4];
    let mut total = 0u32;
    for i in 0..4 {
        q[i] = ((w[i].max(0.0) / sum) * 65535.0).round() as u16;
        total += q[i] as u32;
    }
    let big = (0..4)
        .max_by(|&a, &b| q[a].cmp(&q[b]).then(b.cmp(&a)))
        .unwrap_or(0);
    let fixed = q[big] as i64 + 65535 - total as i64;
    q[big] = fixed.clamp(0, 65535) as u16;
    q
}

/// Cooks the animation blob of a parsed glTF. `bytes` is the source file
/// (for `extras`).
pub fn cook(
    src: &GltfSource,
    bytes: &[u8],
    mesh: &MeshOptions,
    settings: &AnimSettings,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let doc = &src.doc;
    let json = json(bytes);
    let mut notes = Vec::new();
    let mut strings = Strings(Vec::new());
    let order = model::node_order(doc, &mut |s| strings.add(s))?;
    let new_index = &order.new_index;

    // Skins.
    let mut skins = Vec::new();
    let mut joints: Vec<u32> = Vec::new();
    let mut inverse_binds: Vec<f32> = Vec::new();
    for skin in doc.skins() {
        let reader = skin.reader(|b| Some(&src.buffers[b.index()].0[..]));
        let ibms: Vec<[[f32; 4]; 4]> = reader
            .read_inverse_bind_matrices()
            .map(|m| m.collect())
            .unwrap_or_default();
        let first = joints.len() as u32;
        for (i, j) in skin.joints().enumerate() {
            joints.push(new_index[j.index()]);
            let m = ibms.get(i).copied().unwrap_or([
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]);
            inverse_binds.extend(m.iter().flatten());
        }
        if !ibms.is_empty() && ibms.len() != skin.joints().count() {
            return Err(CookError(format!(
                "skin {}: {} inverse bind matrices for {} joints",
                skin.index(),
                ibms.len(),
                skin.joints().count()
            )));
        }
        let (name_offset, name_len) = strings.add(skin.name().unwrap_or(""));
        skins.push(SkinRow {
            joint_first: first,
            joint_count: joints.len() as u32 - first,
            skeleton: skin.skeleton().map_or(NONE, |n| new_index[n.index()]),
            name_offset,
            name_len,
        });
    }

    // Clips.
    let mut clips = Vec::new();
    let mut channels: Vec<ChannelRow> = Vec::new();
    let mut samples: Vec<f32> = Vec::new();
    let mut events: Vec<EventRow> = Vec::new();
    let mut clip_names: Vec<String> = Vec::new();
    for anim in doc.animations() {
        let name = anim
            .name()
            .map_or_else(|| format!("clip{}", anim.index()), str::to_string);
        if clip_names.contains(&name) {
            return Err(CookError(format!(
                "two animations are called '{name}'; clips are found by name"
            )));
        }
        clip_names.push(name.clone());
        let source = read_channels(src, &anim, new_index)
            .map_err(|e| CookError(format!("animation '{name}': {e}")))?;
        let duration = source
            .iter()
            .flat_map(|c| c.times.last())
            .fold(0.0f32, |a, &b| a.max(b));
        let min_step = source
            .iter()
            .flat_map(|c| c.times.windows(2).map(|w| (w[1] - w[0]) as f64))
            .filter(|&d| d > 1e-6)
            .fold(f64::MAX, f64::min);
        let source_rate = if min_step < f64::MAX {
            (1.0 / min_step) as f32
        } else {
            0.0
        };
        let rate = settings.rate.unwrap_or(if source_rate > 0.0 {
            source_rate.round()
        } else {
            30.0
        });
        let rate = rate.clamp(1.0, 120.0);
        let frames = og_anim::ClipAsset::frames_for(duration, rate);
        let channel_first = channels.len() as u32;
        for c in &source {
            let sample_offset = samples.len() as u32;
            for i in 0..frames {
                let t = (i as f64 / rate as f64).min(duration as f64);
                let v = evaluate(
                    c.interpolation,
                    &c.times,
                    &c.values,
                    c.lanes,
                    c.path == path::ROTATION,
                    t,
                );
                samples.extend(v.iter().map(|&x| x as f32));
            }
            channels.push(ChannelRow {
                node: c.node,
                path: c.path,
                interpolation: c.interpolation,
                lanes: c.lanes as u32,
                sample_offset,
            });
        }
        let event_first = events.len() as u32;
        for (n, t) in
            events_of(json.as_ref(), anim.index(), &name, settings, duration).map_err(CookError)?
        {
            let (name_offset, name_len) = strings.add(&n);
            events.push(EventRow {
                clip: clips.len() as u32,
                time: t,
                name_offset,
                name_len,
            });
        }
        let (name_offset, name_len) = strings.add(&name);
        clips.push(ClipRow {
            name_offset,
            name_len,
            duration,
            rate,
            source_rate,
            frames,
            channel_first,
            channel_count: channels.len() as u32 - channel_first,
            event_first,
            event_count: events.len() as u32 - event_first,
        });
        notes.push(format!(
            "clip '{name}': {:.3} s, {} channels, source {:.2} Hz resampled at {rate} Hz ({frames} frames), {} event(s)",
            duration,
            source.len(),
            source_rate,
            events.len() as u32 - event_first
        ));
    }
    for (c, n, _) in &settings.events {
        if !clip_names.contains(c) {
            return Err(CookError(format!(
                "import settings: event '{n}' names clip '{c}', which the glTF does not have ({})",
                clip_names.join(", ")
            )));
        }
    }

    // Per-vertex data of the cooked primitives, indexed as the model blob
    // indexes them (model.rs cooks the same primitives in the same order).
    let mut skin_streams = Vec::new();
    let mut skin_data: Vec<u8> = Vec::new();
    let mut morphs = Vec::new();
    let mut targets = Vec::new();
    let mut morph_data: Vec<u8> = Vec::new();
    let default_material = doc.materials().count() as u32;
    let mut cooked_index = 0u32;
    for m in doc.meshes() {
        let target_names: Vec<String> = json
            .as_ref()
            .and_then(|j| {
                j.get("meshes")?
                    .get(m.index())?
                    .get("extras")?
                    .get("targetNames")
            })
            .and_then(|t| t.as_array())
            .map(|a| {
                a.iter()
                    .map(|n| n.as_str().unwrap_or("").to_string())
                    .collect()
            })
            .unwrap_or_default();
        let default_weights: Vec<f32> = m.weights().map(|w| w.to_vec()).unwrap_or_default();
        for (pi, prim) in m.primitives().enumerate() {
            if prim.mode() != gltf::mesh::Mode::Triangles {
                continue;
            }
            let material = prim
                .material()
                .index()
                .map_or(default_material, |i| i as u32);
            let normal_mapped = prim.material().normal_texture().is_some();
            let what = format!("mesh '{}' primitive {pi}", m.name().unwrap_or(""));
            let Some(cooked) = model::cook_primitive(src, &prim, material, normal_mapped, mesh)
                .map_err(|e| CookError(format!("{what}: {e}")))?
            else {
                continue;
            };
            let index = cooked_index;
            cooked_index += 1;
            let vertex_count = cooked.sources.len();
            let reader = prim.reader(|b| Some(&src.buffers[b.index()].0[..]));
            if let (Some(j), Some(w)) = (reader.read_joints(0), reader.read_weights(0)) {
                let j: Vec<[u16; 4]> = j.into_u16().collect();
                let w: Vec<[f32; 4]> = w.into_f32().collect();
                let mut packed = Vec::with_capacity(vertex_count * 16);
                for &s in &cooked.sources {
                    let (jj, ww) = (
                        j.get(s as usize).copied().unwrap_or_default(),
                        w.get(s as usize).copied().unwrap_or_default(),
                    );
                    let q = quantize_weights(ww);
                    for v in jj {
                        packed.extend_from_slice(&v.to_le_bytes());
                    }
                    for v in q {
                        packed.extend_from_slice(&v.to_le_bytes());
                    }
                }
                let enc = model::encode_vertices(&packed, vertex_count, 16)
                    .map_err(|e| CookError(format!("{what}: {e}")))?;
                while !skin_data.len().is_multiple_of(4) {
                    skin_data.push(0);
                }
                skin_streams.push(SkinStreamRow {
                    primitive: index,
                    mesh: m.index() as u32,
                    vertex_count: vertex_count as u32,
                    data_offset: skin_data.len() as u32,
                    data_size: enc.len() as u32,
                });
                skin_data.extend_from_slice(&enc);
            }
            let morph: Vec<(Deltas, Option<Deltas>)> = reader
                .read_morph_targets()
                .map(|(p, n, _)| {
                    (
                        p.map(|p| p.collect()).unwrap_or_default(),
                        n.map(|n| n.collect()),
                    )
                })
                .collect();
            if !morph.is_empty() {
                let normals = morph.iter().all(|t| t.1.is_some());
                let stride = if normals { 24 } else { 12 };
                let mut packed = Vec::with_capacity(morph.len() * vertex_count * stride);
                for (p, n) in &morph {
                    for &s in &cooked.sources {
                        let d = p.get(s as usize).copied().unwrap_or_default();
                        d.iter()
                            .for_each(|x| packed.extend_from_slice(&x.to_le_bytes()));
                        if normals {
                            let d = n
                                .as_ref()
                                .and_then(|n| n.get(s as usize))
                                .copied()
                                .unwrap_or_default();
                            d.iter()
                                .for_each(|x| packed.extend_from_slice(&x.to_le_bytes()));
                        }
                    }
                }
                let enc = model::encode_vertices(&packed, morph.len() * vertex_count, stride)
                    .map_err(|e| CookError(format!("{what}: {e}")))?;
                while !morph_data.len().is_multiple_of(4) {
                    morph_data.push(0);
                }
                let target_first = targets.len() as u32;
                for t in 0..morph.len() {
                    let (name_offset, name_len) =
                        strings.add(target_names.get(t).map_or("", |s| s.as_str()));
                    targets.push(TargetRow {
                        name_offset,
                        name_len,
                        default_weight: default_weights.get(t).copied().unwrap_or(0.0),
                    });
                }
                morphs.push(MorphRow {
                    primitive: index,
                    mesh: m.index() as u32,
                    vertex_count: vertex_count as u32,
                    target_first,
                    target_count: morph.len() as u32,
                    normals: normals as u32,
                    data_offset: morph_data.len() as u32,
                    data_size: enc.len() as u32,
                });
                morph_data.extend_from_slice(&enc);
            }
        }
    }
    notes.push(format!(
        "#anim: {} nodes, {} skin(s) with {} joints, {} clip(s), {} skinned and {} morphed primitive(s)",
        order.rows.len(),
        skins.len(),
        joints.len(),
        clips.len(),
        skin_streams.len(),
        morphs.len()
    ));
    let meta = AnimMeta {
        node_count: order.rows.len() as u32,
        skin_count: skins.len() as u32,
        joint_count: joints.len() as u32,
        clip_count: clips.len() as u32,
        channel_count: channels.len() as u32,
        event_count: events.len() as u32,
        skin_stream_count: skin_streams.len() as u32,
        morph_count: morphs.len() as u32,
    };
    let mut w = BlobWriter::new(
        format::Animation::type_hash(),
        format::Animation::schema_version(),
        stamp,
    );
    w.table("meta", &[meta], row_hash::<AnimMeta>())
        .table("nodes", &order.rows, row_hash::<NodeRow>())
        .table("skins", &skins, row_hash::<SkinRow>())
        .table("joints", &joints, 0)
        .table("inverse_binds", &inverse_binds, 0)
        .table("clips", &clips, row_hash::<ClipRow>())
        .table("channels", &channels, row_hash::<ChannelRow>())
        .table("samples", &samples, 0)
        .table("events", &events, row_hash::<EventRow>())
        .table("skin_streams", &skin_streams, row_hash::<SkinStreamRow>())
        .raw("skin_data", skin_data, 16)
        .table("morphs", &morphs, row_hash::<MorphRow>())
        .table("targets", &targets, row_hash::<TargetRow>())
        .raw("morph_data", morph_data, 16)
        .raw("strings", strings.0, 1);
    Ok((w.finish(), notes))
}
