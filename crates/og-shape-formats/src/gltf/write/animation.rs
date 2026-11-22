//! A scene's clips and skin clips written as glTF animations, sampled and thinned.

use super::*;

/// Skinned parts' clips as glTF animations ([`glb_skin_clips`]): a clip
/// name is one animation, with channels for every part that has it.
pub(crate) fn skin_animations(
    skin_clips: &[(&str, &[og_shape::mesh::SkinClip])],
    skin_nodes: &BTreeMap<&str, (&og_shape::mesh::Skin, Vec<usize>)>,
    (part_node, morph_counts): (&BTreeMap<&str, usize>, &BTreeMap<&str, usize>),
    bin: &mut Vec<u8>,
    views: &mut Vec<Json>,
    accessors: &mut Vec<Json>,
) -> Result<Vec<Json>, ExportError> {
    use og_shape::mesh::skin::{affine_inverse, mat_mul};
    let mut names: Vec<&str> = Vec::new();
    for (_, clips) in skin_clips {
        for c in *clips {
            if !names.contains(&c.name.as_str()) {
                names.push(&c.name);
            }
        }
    }
    let mut out = Vec::new();
    for name in names {
        let bad = |why: String| ExportError::BadClip {
            clip: name.to_string(),
            why,
        };
        let mut samplers = Vec::new();
        let mut channels = Vec::new();
        for (part, clips) in skin_clips {
            let Some(clip) = clips.iter().find(|c| c.name == name) else {
                continue;
            };
            let targets = morph_counts.get(part).copied().unwrap_or(0);
            let driven = !clip.morphs.is_empty();
            let none = (&og_shape::mesh::Skin::default(), Vec::new());
            let (skin, nodes) = match skin_nodes.get(part) {
                Some((skin, nodes)) => (*skin, nodes),
                // A clip of morph weights alone needs no skin.
                None if driven && targets > 0 => (none.0, &none.1),
                None => return Err(bad(format!("part {part:?} has no skin in the scene"))),
            };
            if driven
                && (clip.morphs.len() != clip.times.len()
                    || clip
                        .morphs
                        .iter()
                        .any(|r| r.len() != targets || r.iter().any(|w| !w.is_finite())))
            {
                return Err(bad(format!(
                    "part {part:?}: a weight a morph target ({targets}) at each time"
                )));
            }
            let n = skin.bones.len();
            if clip.times.is_empty()
                || clip.poses.len() != clip.times.len()
                || clip.poses.iter().any(|p| p.len() != n)
                || clip.times.iter().any(|t| !t.is_finite())
                || clip.times.windows(2).any(|w| w[1] <= w[0])
            {
                return Err(bad(format!(
                    "part {part:?}: its times must rise, with a matrix a bone at each"
                )));
            }
            let unbind: Vec<Mat4> = skin
                .bones
                .iter()
                .map(|b| {
                    affine_inverse(&b.inverse_bind).ok_or_else(|| {
                        bad(format!(
                            "bone {:?}'s inverse bind matrix is singular",
                            b.name
                        ))
                    })
                })
                .collect::<Result<_, _>>()?;
            // Per bone, per key: translation, rotation, scale (glTF space).
            let mut tracks: Vec<Vec<Trs>> = vec![Vec::with_capacity(clip.times.len()); n];
            for pose in &clip.poses {
                // Each bone in the world then: its skinning matrix undone
                // of its inverse bind. Its node is that under its parent's.
                let world: Vec<Mat4> = (0..n).map(|b| mat_mul(&pose[b], &unbind[b])).collect();
                for b in 0..n {
                    let local = match skin.bones[b].parent {
                        Some(p) => {
                            let up = affine_inverse(&world[p as usize]).ok_or_else(|| {
                                bad(format!(
                                    "bone {:?} is flattened by the clip",
                                    skin.bones[p as usize].name
                                ))
                            })?;
                            mat_mul(&up, &world[b])
                        }
                        None => world[b],
                    };
                    let mut key = decompose(&crate::gltf::write::skin::gltf_matrix(&local));
                    if let Some(last) = tracks[b].last() {
                        // The same rotation the short way from the last key.
                        let dot: f64 = (0..4).map(|k| key.1[k] * last.1[k]).sum();
                        if dot < 0.0 {
                            key.1 = key.1.map(|c| -c);
                        }
                    }
                    if !key
                        .0
                        .iter()
                        .chain(&key.1)
                        .chain(&key.2)
                        .all(|c| c.is_finite())
                    {
                        return Err(bad(format!(
                            "bone {:?} is not finite in it",
                            skin.bones[b].name
                        )));
                    }
                    tracks[b].push(key);
                }
            }
            let mut plain = |bin: &Vec<u8>, start: usize| {
                views.push(Json::Obj(vec![
                    ("buffer", Json::Int(0)),
                    ("byteOffset", Json::Int(start as u64)),
                    ("byteLength", Json::Int((bin.len() - start) as u64)),
                ]));
                (views.len() - 1) as u64
            };
            for (b, track) in tracks.iter().enumerate() {
                // A scale channel only for a bone the clip scales.
                let scaled = track
                    .iter()
                    .any(|k| k.2.iter().any(|s| (s - 1.0).abs() > 1e-6));
                for (path, kind, on) in [
                    ("translation", "VEC3", true),
                    ("rotation", "VEC4", true),
                    ("scale", "VEC3", scaled),
                ] {
                    if !on {
                        continue;
                    }
                    let values: Vec<[f64; 4]> = track
                        .iter()
                        .map(|k| match path {
                            "translation" => [k.0[0], k.0[1], k.0[2], 0.0],
                            "rotation" => k.1,
                            _ => [k.2[0], k.2[1], k.2[2], 0.0],
                        })
                        .collect();
                    let width = if path == "rotation" { 4 } else { 3 };
                    // The samples a straight line between the kept ones
                    // does not give back, to a hundredth of a millimetre
                    // (a few thousandths of a degree).
                    let tolerance = match path {
                        "translation" => KEEP_METRES,
                        "rotation" => KEEP_TURN,
                        _ => KEEP_SCALE,
                    };
                    // The first bone's translation keeps the clip's last
                    // time, so the clip is as long as it was.
                    let whole = b == 0 && path == "translation";
                    let kept = thin(&clip.times, &values, tolerance, whole);
                    let start = bin.len();
                    for &k in &kept {
                        push_f32(bin, clip.times[k] as f32);
                    }
                    let tv = plain(bin, start);
                    accessors.push(Json::Obj(vec![
                        ("bufferView", Json::Int(tv)),
                        ("componentType", Json::Int(FLOAT)),
                        ("count", Json::Int(kept.len() as u64)),
                        ("type", Json::Str("SCALAR".into())),
                        ("min", Json::Arr(vec![f32n(clip.times[kept[0]] as f32)])),
                        (
                            "max",
                            Json::Arr(vec![f32n(clip.times[kept[kept.len() - 1]] as f32)]),
                        ),
                    ]));
                    let input = (accessors.len() - 1) as u64;
                    let start = bin.len();
                    for &k in &kept {
                        for &c in &values[k][..width] {
                            push_f32(bin, c as f32);
                        }
                    }
                    let ov = plain(bin, start);
                    accessors.push(Json::Obj(vec![
                        ("bufferView", Json::Int(ov)),
                        ("componentType", Json::Int(FLOAT)),
                        ("count", Json::Int(kept.len() as u64)),
                        ("type", Json::Str(kind.into())),
                    ]));
                    let output = (accessors.len() - 1) as u64;
                    channels.push(Json::Obj(vec![
                        ("sampler", Json::Int(samplers.len() as u64)),
                        (
                            "target",
                            Json::Obj(vec![
                                ("node", Json::Int(nodes[b] as u64)),
                                ("path", Json::Str(path.into())),
                            ]),
                        ),
                    ]));
                    samplers.push(Json::Obj(vec![
                        ("input", Json::Int(input)),
                        ("interpolation", Json::Str("LINEAR".into())),
                        ("output", Json::Int(output)),
                    ]));
                }
            }
            if driven {
                // A weight a target at each key kept, the targets in order.
                let Some(&node) = part_node.get(part) else {
                    return Err(bad(format!("part {part:?} is not in the scene")));
                };
                let kept = thin(&clip.times, &clip.morphs, KEEP_WEIGHT, true);
                let start = bin.len();
                for &k in &kept {
                    push_f32(bin, clip.times[k] as f32);
                }
                let tv = plain(bin, start);
                accessors.push(Json::Obj(vec![
                    ("bufferView", Json::Int(tv)),
                    ("componentType", Json::Int(FLOAT)),
                    ("count", Json::Int(kept.len() as u64)),
                    ("type", Json::Str("SCALAR".into())),
                    ("min", Json::Arr(vec![f32n(clip.times[kept[0]] as f32)])),
                    (
                        "max",
                        Json::Arr(vec![f32n(clip.times[kept[kept.len() - 1]] as f32)]),
                    ),
                ]));
                let input = (accessors.len() - 1) as u64;
                let start = bin.len();
                for &k in &kept {
                    for &w in &clip.morphs[k] {
                        push_f32(bin, w as f32);
                    }
                }
                let ov = plain(bin, start);
                accessors.push(Json::Obj(vec![
                    ("bufferView", Json::Int(ov)),
                    ("componentType", Json::Int(FLOAT)),
                    ("count", Json::Int((kept.len() * targets) as u64)),
                    ("type", Json::Str("SCALAR".into())),
                ]));
                let output = (accessors.len() - 1) as u64;
                channels.push(Json::Obj(vec![
                    ("sampler", Json::Int(samplers.len() as u64)),
                    (
                        "target",
                        Json::Obj(vec![
                            ("node", Json::Int(node as u64)),
                            ("path", Json::Str("weights".into())),
                        ]),
                    ),
                ]));
                samplers.push(Json::Obj(vec![
                    ("input", Json::Int(input)),
                    ("interpolation", Json::Str("LINEAR".into())),
                    ("output", Json::Int(output)),
                ]));
            }
        }
        if !channels.is_empty() {
            out.push(Json::Obj(vec![
                ("name", Json::Str(name.to_string())),
                ("channels", Json::Arr(channels)),
                ("samplers", Json::Arr(samplers)),
            ]));
        }
    }
    Ok(out)
}

/// How far a translation may be from the straight line between the keys
/// kept round it, in metres (a hundredth of a millimetre).
pub(crate) const KEEP_METRES: f64 = 1e-5;

/// The same for a rotation's quaternion, by component: about two
/// thousandths of a degree (0.04 mm at the end of a metre).
pub(crate) const KEEP_TURN: f64 = 2e-5;

/// The same for a scale.
pub(crate) const KEEP_SCALE: f64 = 1e-5;

/// The keys of a curve to keep (indices, rising, the first always): the
/// fewest by halving such that every key left out is within `tolerance`,
/// in each of its numbers, of the straight line between the kept keys
/// either side of it. A curve that never leaves its first value keeps
/// that one key, unless `whole` asks for its last too (so a clip keeps
/// its length).
pub(crate) fn thin<V: AsRef<[f64]>>(
    times: &[f64],
    values: &[V],
    tolerance: f64,
    whole: bool,
) -> Vec<usize> {
    let n = values.len();
    let values: Vec<&[f64]> = values.iter().map(AsRef::as_ref).collect();
    let off = |a: usize, b: usize, k: usize| {
        let f = if times[b] > times[a] {
            (times[k] - times[a]) / (times[b] - times[a])
        } else {
            0.0
        };
        (0..values[k].len())
            .map(|i| (values[a][i] + (values[b][i] - values[a][i]) * f - values[k][i]).abs())
            .fold(0.0, f64::max)
    };
    let mut keep = vec![false; n];
    keep[0] = true;
    if n > 1 {
        keep[n - 1] = true;
        // Spans still to check, each split at the key furthest off.
        let mut spans = vec![(0usize, n - 1)];
        while let Some((a, b)) = spans.pop() {
            let mut worst = (tolerance, None);
            for k in a + 1..b {
                let d = off(a, b, k);
                if d > worst.0 {
                    worst = (d, Some(k));
                }
            }
            if let Some(k) = worst.1 {
                keep[k] = true;
                spans.push((a, k));
                spans.push((k, b));
            }
        }
        // A curve that stays where it starts is one key.
        if !whole && keep.iter().filter(|&&k| k).count() == 2 && off(0, 0, n - 1) <= tolerance {
            keep[n - 1] = false;
        }
    }
    (0..n).filter(|&k| keep[k]).collect()
}

/// The same for a morph target's weight.
pub(crate) const KEEP_WEIGHT: f64 = 1e-4;

/// A translation, a rotation (a unit quaternion x, y, z, w) and a scale.
pub(crate) type Trs = ([f64; 3], [f64; 4], [f64; 3]);

/// An affine matrix as a translation, a rotation (a unit quaternion x, y,
/// z, w) and a scale: the columns' lengths are the scale (the first
/// negative for a mirror), what is left the rotation.
pub(crate) fn decompose(m: &Mat4) -> Trs {
    let col = |c: usize| DVec3::new(m[4 * c], m[4 * c + 1], m[4 * c + 2]);
    let (mut x, y, z) = (col(0), col(1), col(2));
    let mut scale = [x.length(), y.length(), z.length()];
    if x.cross(y).dot(z) < 0.0 {
        scale[0] = -scale[0];
        x = -x;
    }
    let unit = |v: DVec3, fallback: DVec3| {
        let l = v.length();
        if l > 0.0 { v / l } else { fallback }
    };
    let (x, y, z) = (unit(x, DVec3::X), unit(y, DVec3::Y), unit(z, DVec3::Z));
    // The quaternion of the rotation with columns x, y, z.
    let (m00, m11, m22) = (x.x, y.y, z.z);
    let trace = m00 + m11 + m22;
    let q = if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;
        [(y.z - z.y) / s, (z.x - x.z) / s, (x.y - y.x) / s, 0.25 * s]
    } else if m00 > m11 && m00 > m22 {
        let s = (1.0 + m00 - m11 - m22).sqrt() * 2.0;
        [0.25 * s, (y.x + x.y) / s, (z.x + x.z) / s, (y.z - z.y) / s]
    } else if m11 > m22 {
        let s = (1.0 + m11 - m00 - m22).sqrt() * 2.0;
        [(y.x + x.y) / s, 0.25 * s, (z.y + y.z) / s, (z.x - x.z) / s]
    } else {
        let s = (1.0 + m22 - m00 - m11).sqrt() * 2.0;
        [(z.x + x.z) / s, (z.y + y.z) / s, 0.25 * s, (x.y - y.x) / s]
    };
    let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    let q = if len > 0.0 {
        q.map(|c| c / len)
    } else {
        [0.0, 0.0, 0.0, 1.0]
    };
    ([m[12], m[13], m[14]], q, scale)
}

/// The clips as glTF animations ([`glb_clips`]): their accessors and
/// views are appended.
pub(crate) fn animations(
    clips: &[Clip],
    checked: &[(Option<Checked>, Rigid)],
    joint_node: &BTreeMap<&str, usize>,
    bin: &mut Vec<u8>,
    views: &mut Vec<Json>,
    accessors: &mut Vec<Json>,
) -> Result<Vec<Json>, ExportError> {
    let joints: BTreeMap<&str, &Checked> = checked
        .iter()
        .filter_map(|(c, _)| c.as_ref())
        .map(|c| (c.name.as_str(), c))
        .collect();
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut out = Vec::new();
    for clip in clips {
        let bad = |why: String| ExportError::BadClip {
            clip: clip.name.clone(),
            why,
        };
        if clip.name.is_empty() {
            return Err(bad("it has no name".into()));
        }
        if !seen.insert(clip.name.as_str()) {
            return Err(bad("another clip has this name".into()));
        }
        if clip.keys.is_empty() {
            return Err(bad("it has no key".into()));
        }
        let mut last = -1.0;
        // Each leader's own keys, in kernel units.
        let mut tracks: BTreeMap<&str, Vec<(f64, f64)>> = BTreeMap::new();
        for key in &clip.keys {
            if !(key.time.is_finite() && key.time >= 0.0 && key.time > last) {
                return Err(bad(format!(
                    "its times must be 0 or more and rise: {} after {}",
                    key.time,
                    if last < 0.0 { 0.0 } else { last }
                )));
            }
            last = key.time;
            for (name, value) in &key.pose {
                let Some(j) = joints.get(name.as_str()) else {
                    return Err(bad(format!(
                        "no joint {name:?}; joints: {}",
                        joints.keys().copied().collect::<Vec<_>>().join(", ")
                    )));
                };
                if j.local.is_none() {
                    return Err(bad(format!(
                        "joint {name:?} is fixed or floating: it has no value"
                    )));
                }
                if let Some((leader, _)) = &j.follows {
                    return Err(bad(format!(
                        "joint {name:?} follows {leader:?}: key that one"
                    )));
                }
                if !value.is_finite() {
                    return Err(bad(format!("joint {name:?}'s value is not finite")));
                }
                let track = tracks.entry(j.name.as_str()).or_default();
                if track.last().is_some_and(|&(t, _)| t == key.time) {
                    return Err(bad(format!(
                        "joint {name:?} is keyed twice at {}",
                        key.time
                    )));
                }
                track.push((key.time, *value));
            }
        }
        // Followers take their leader's keys.
        let led: Vec<(&str, Vec<(f64, f64)>)> = joints
            .values()
            .filter_map(|j| {
                let (leader, k) = j.follows.as_ref()?;
                let track = tracks.get(leader.as_str())?;
                Some((
                    j.name.as_str(),
                    track.iter().map(|&(t, v)| (t, v * k)).collect(),
                ))
            })
            .collect();
        tracks.extend(led);

        let mut samplers = Vec::new();
        let mut channels = Vec::new();
        for (name, track) in &tracks {
            let j = joints[name];
            let (Some(axis), Some(&node)) = (j.local, joint_node.get(name)) else {
                continue;
            };
            if let Some((lo, hi)) = j.kernel_span {
                let slack = 1e-9 * lo.abs().max(hi.abs()).max(1.0);
                if let Some(&(t, v)) = track
                    .iter()
                    .find(|&&(_, v)| !(lo - slack <= v && v <= hi + slack))
                {
                    return Err(bad(format!(
                        "joint {name:?} at {t}: {v} is outside its range {lo} to {hi}"
                    )));
                }
            }
            // A turn is written a quarter at a time at most.
            let mut keys: Vec<(f64, f64)> = Vec::with_capacity(track.len());
            for &(t, v) in track {
                if let (Some(&(t0, v0)), false) = (keys.last(), j.slides) {
                    let steps = ((v - v0).abs() / core::f64::consts::FRAC_PI_2).ceil();
                    let mut k = 1.0;
                    while k < steps {
                        let f = k / steps;
                        keys.push((t0 + (t - t0) * f, v0 + (v - v0) * f));
                        k += 1.0;
                    }
                }
                keys.push((t, v));
            }
            let start = bin.len();
            for &(t, _) in &keys {
                push_f32(bin, t as f32);
            }
            let mut plain = |bin: &Vec<u8>, start: usize| {
                views.push(Json::Obj(vec![
                    ("buffer", Json::Int(0)),
                    ("byteOffset", Json::Int(start as u64)),
                    ("byteLength", Json::Int((bin.len() - start) as u64)),
                ]));
                (views.len() - 1) as u64
            };
            let tv = plain(bin, start);
            accessors.push(Json::Obj(vec![
                ("bufferView", Json::Int(tv)),
                ("componentType", Json::Int(FLOAT)),
                ("count", Json::Int(keys.len() as u64)),
                ("type", Json::Str("SCALAR".into())),
                ("min", Json::Arr(vec![f32n(keys[0].0 as f32)])),
                ("max", Json::Arr(vec![f32n(keys[keys.len() - 1].0 as f32)])),
            ]));
            let input = (accessors.len() - 1) as u64;
            let start = bin.len();
            let mut before: Option<[f64; 4]> = None;
            for &(_, v) in &keys {
                let motion = if j.slides {
                    Rigid {
                        t: axis * v,
                        ..Rigid::IDENTITY
                    }
                } else {
                    Rigid::turn(axis, v)
                };
                let (t, mut q) = j.pivot.then(&motion).gltf();
                if j.slides {
                    for c in [t.x, t.y, t.z] {
                        push_f32(bin, c as f32);
                    }
                } else {
                    // The same rotation the short way from the last key.
                    if let Some(b) = before {
                        let dot: f64 = (0..4).map(|k| q[k] * b[k]).sum();
                        if dot < 0.0 {
                            q = q.map(|c| -c);
                        }
                    }
                    before = Some(q);
                    for c in q {
                        push_f32(bin, c as f32);
                    }
                }
            }
            let ov = plain(bin, start);
            accessors.push(Json::Obj(vec![
                ("bufferView", Json::Int(ov)),
                ("componentType", Json::Int(FLOAT)),
                ("count", Json::Int(keys.len() as u64)),
                (
                    "type",
                    Json::Str(if j.slides { "VEC3" } else { "VEC4" }.into()),
                ),
            ]));
            let output = (accessors.len() - 1) as u64;
            channels.push(Json::Obj(vec![
                ("sampler", Json::Int(samplers.len() as u64)),
                (
                    "target",
                    Json::Obj(vec![
                        ("node", Json::Int(node as u64)),
                        (
                            "path",
                            Json::Str(if j.slides { "translation" } else { "rotation" }.into()),
                        ),
                    ]),
                ),
            ]));
            samplers.push(Json::Obj(vec![
                ("input", Json::Int(input)),
                ("interpolation", Json::Str("LINEAR".into())),
                ("output", Json::Int(output)),
            ]));
        }
        if channels.is_empty() {
            return Err(bad("it moves no joint".into()));
        }
        out.push(Json::Obj(vec![
            ("name", Json::Str(clip.name.clone())),
            ("channels", Json::Arr(channels)),
            ("samplers", Json::Arr(samplers)),
        ]));
    }
    Ok(out)
}

pub(crate) fn vec3(v: DVec3) -> Json {
    nums(&[v.x, v.y, v.z])
}
