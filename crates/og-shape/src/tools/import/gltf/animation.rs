//! glTF animations sampled into clips on the part's bones and morph targets.

use super::*;

/// A translation, a rotation (a quaternion x, y, z, w, made unit) and a
/// scale as one map: scale, then turn, then move (glTF's order).
pub(crate) fn trs(t: DVec3, q: [f64; 4], s: DVec3) -> Affine {
    let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    let (x, y, z, w) = if len > 0.0 {
        (q[0] / len, q[1] / len, q[2] / len, q[3] / len)
    } else {
        (0.0, 0.0, 0.0, 1.0)
    };
    let cx = DVec3::new(
        1.0 - 2.0 * (y * y + z * z),
        2.0 * (x * y + z * w),
        2.0 * (x * z - y * w),
    );
    let cy = DVec3::new(
        2.0 * (x * y - z * w),
        1.0 - 2.0 * (x * x + z * z),
        2.0 * (y * z + x * w),
    );
    let cz = DVec3::new(
        2.0 * (x * z + y * w),
        2.0 * (y * z - x * w),
        1.0 - 2.0 * (x * x + y * y),
    );
    Affine {
        x: cx * s.x,
        y: cy * s.y,
        z: cz * s.z,
        t,
    }
}

/// A curve of one node's translation, rotation or scale: key times and
/// 3 or 4 numbers a key.
pub(crate) struct Track {
    pub(crate) times: Vec<f64>,
    pub(crate) values: Vec<f64>,
    pub(crate) width: usize,
    pub(crate) step: bool,
}

impl Track {
    /// The value at `time`: held before the first key and after the last;
    /// between two, straight (a rotation along the shorter way, made unit
    /// by [`trs`]), or the earlier key's for a stepped curve.
    pub(crate) fn at(&self, time: f64) -> [f64; 4] {
        let w = self.width;
        let key = |k: usize| {
            let mut v = [0.0; 4];
            v[..w].copy_from_slice(&self.values[k * w..k * w + w]);
            v
        };
        let n = self.times.len();
        let k = self.times.partition_point(|&t| t <= time);
        if k == 0 {
            return key(0);
        }
        if k >= n || self.step {
            return key(k - 1);
        }
        let (t0, t1) = (self.times[k - 1], self.times[k]);
        let f = if t1 > t0 {
            (time - t0) / (t1 - t0)
        } else {
            0.0
        };
        let (a, mut b) = (key(k - 1), key(k));
        if w == 4 && a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3] < 0.0 {
            b = b.map(|x| -x);
        }
        let mut v = [0.0; 4];
        for i in 0..w {
            v[i] = a[i] + (b[i] - a[i]) * f;
        }
        v
    }
}

impl Track {
    /// Every number of the key at `time`, as [`Track::at`] takes them
    /// (morph weights: any count a key, mixed straight).
    pub(crate) fn row(&self, time: f64) -> Vec<f64> {
        let w = self.width;
        let key = |k: usize| &self.values[k * w..k * w + w];
        let n = self.times.len();
        let k = self.times.partition_point(|&t| t <= time);
        if k == 0 {
            return key(0).to_vec();
        }
        if k >= n || self.step {
            return key(k - 1).to_vec();
        }
        let (t0, t1) = (self.times[k - 1], self.times[k]);
        let f = if t1 > t0 {
            (time - t0) / (t1 - t0)
        } else {
            0.0
        };
        key(k - 1)
            .iter()
            .zip(key(k))
            .map(|(a, b)| a + (b - a) * f)
            .collect()
    }
}

/// More keys than this in one animation and it is sampled evenly instead
/// of at every key time.
pub(crate) const MAX_KEYS: usize = 4096;

/// How densely an animation is sampled between its own keys.
pub(crate) const SAMPLES_A_SECOND: f64 = 60.0;

impl Doc<'_> {
    /// The file's animations as they move the skin read into `space`:
    /// each sampled at its own key times and sixty times a second between
    /// them (or evenly, past [`MAX_KEYS`]).
    /// A channel on a node that is no ancestor of a bone moves nothing
    /// here. A cubic-spline curve is read through its key values, straight
    /// between them.
    /// `morph`: the mesh's node and how many morph targets it has, for
    /// the clips' `weights` channels on that node.
    pub(crate) fn clips(
        &self,
        space: &SkinSpace,
        morph: Option<(usize, usize)>,
    ) -> Result<Vec<crate::mesh::SkinClip>, ImportError> {
        let nodes = self.json.get("nodes").items();
        let parents = self.parents();
        // The nodes that move a bone: the bones and all above them.
        let mut above = vec![false; nodes.len()];
        for &j in &space.nodes {
            let mut at = Some(j);
            let mut steps = 0;
            while let Some(i) = at {
                if above[i] || steps > nodes.len() {
                    break;
                }
                above[i] = true;
                at = parents[i];
                steps += 1;
            }
        }
        let mut out = Vec::new();
        let mut used = BTreeMap::new();
        for (ai, anim) in self.json.get("animations").items().iter().enumerate() {
            // Per node: its translation, rotation and scale curves.
            let mut tracks: BTreeMap<usize, [Option<Track>; 3]> = BTreeMap::new();
            let mut times: Vec<f64> = Vec::new();
            // The mesh's node's morph weights: one number a target a key.
            let mut weights: Option<Track> = None;
            for ch in anim.get("channels").items() {
                let target = ch.get("target");
                let (Some(node), Some(path)) =
                    (target.get("node").index(), target.get("path").str())
                else {
                    continue;
                };
                let (slot, width) = match (path, morph) {
                    ("translation", _) => (0, 3),
                    ("rotation", _) => (1, 4),
                    ("scale", _) => (2, 3),
                    ("weights", Some((n, targets))) if n == node && targets > 0 => (3, targets),
                    _ => continue,
                };
                if slot < 3
                    && (!above.get(node).copied().unwrap_or(false)
                        || !nodes[node].get("matrix").is_null())
                {
                    continue;
                }
                let sampler = anim
                    .get("samplers")
                    .at(ch.get("sampler").index().unwrap_or(usize::MAX));
                let (Some(input), Some(output)) =
                    (sampler.get("input").index(), sampler.get("output").index())
                else {
                    continue;
                };
                let (t, tw) = self.accessor(input)?;
                let (mut v, vw) = self.accessor(output)?;
                let interpolation = sampler.get("interpolation").str().unwrap_or("LINEAR");
                if interpolation == "CUBICSPLINE" && v.len() == t.len() * width * 3 {
                    // In-tangent, value, out-tangent a key: the value.
                    v = (0..t.len())
                        .flat_map(|k| v[(k * 3 + 1) * width..(k * 3 + 2) * width].to_vec())
                        .collect();
                }
                // Morph weights are scalars, a target after a target.
                let each = if slot == 3 { 1 } else { width };
                if tw != 1 || vw != each || t.is_empty() || v.len() != t.len() * width {
                    return Err(fmt_err(format!(
                        "animation {ai}: a channel's times and values do not match"
                    )));
                }
                if t.iter().any(|x| !x.is_finite()) || t.windows(2).any(|w| w[1] < w[0]) {
                    return Err(fmt_err(format!(
                        "animation {ai}: a channel's times do not rise"
                    )));
                }
                times.extend(&t);
                let track = Track {
                    times: t,
                    values: v,
                    width,
                    step: interpolation == "STEP",
                };
                if slot == 3 {
                    weights = Some(track);
                } else {
                    tracks.entry(node).or_insert([None, None, None])[slot] = Some(track);
                }
            }
            if tracks.is_empty() && weights.is_none() {
                continue;
            }
            // Between samples the matrices run straight, which is not how
            // a bone turns between two keys far apart: sixty samples a
            // second go in beside the keys' own times.
            let (start, end) = times
                .iter()
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &t| {
                    (lo.min(t), hi.max(t))
                });
            let steps = ((end - start) * SAMPLES_A_SECOND).ceil();
            if steps >= 1.0 && steps < MAX_KEYS as f64 {
                times.extend(
                    (1..steps as usize).map(|k| start + (end - start) * (k as f64 / steps)),
                );
            }
            times.sort_by(f64::total_cmp);
            times.dedup();
            if times.len() > MAX_KEYS {
                let (first, last) = (times[0], times[times.len() - 1]);
                times = (0..MAX_KEYS)
                    .map(|k| first + (last - first) * (k as f64 / (MAX_KEYS - 1) as f64))
                    .collect();
            }
            let mut poses = Vec::with_capacity(times.len());
            for &time in &times {
                // Each node above a bone, in the world at this time.
                let mut world: Vec<Option<Affine>> = vec![None; nodes.len()];
                let local = |i: usize| -> Affine {
                    let n = &nodes[i];
                    let Some(tr) = tracks.get(&i) else {
                        return self.local(n);
                    };
                    let rest = |key: &str, d: [f64; 4], w: usize| {
                        n.get(key).nums().filter(|v| v.len() == w).map_or(d, |v| {
                            let mut a = d;
                            a[..w].copy_from_slice(&v);
                            a
                        })
                    };
                    let t = tr[0]
                        .as_ref()
                        .map_or(rest("translation", [0.0; 4], 3), |c| c.at(time));
                    let q = tr[1]
                        .as_ref()
                        .map_or(rest("rotation", [0.0, 0.0, 0.0, 1.0], 4), |c| c.at(time));
                    let s = tr[2]
                        .as_ref()
                        .map_or(rest("scale", [1.0, 1.0, 1.0, 0.0], 3), |c| c.at(time));
                    trs(
                        DVec3::new(t[0], t[1], t[2]),
                        q,
                        DVec3::new(s[0], s[1], s[2]),
                    )
                };
                let mut pose = Vec::with_capacity(space.nodes.len());
                for (b, &j) in space.nodes.iter().enumerate() {
                    // Up from the bone to a node already placed, then down.
                    let mut chain = Vec::new();
                    let mut at = Some(j);
                    while let Some(i) = at {
                        if world[i].is_some() || chain.len() > nodes.len() {
                            break;
                        }
                        chain.push(i);
                        at = parents[i];
                    }
                    let mut m = at.and_then(|i| world[i]).unwrap_or(Affine::IDENTITY);
                    for &i in chain.iter().rev() {
                        m = m.then(&local(i));
                        world[i] = Some(m);
                    }
                    let k = conjugate(&m.then(&space.ibms[b]), 1000.0).then(&space.unbind);
                    pose.push([
                        k.x.x, k.x.y, k.x.z, 0.0, k.y.x, k.y.y, k.y.z, 0.0, k.z.x, k.z.y, k.z.z,
                        0.0, k.t.x, k.t.y, k.t.z, 1.0,
                    ]);
                }
                poses.push(pose);
            }
            let name = anim
                .get("name")
                .str()
                .map_or_else(|| format!("animation{ai}"), str::to_string);
            // From 0: a clip's time is from its first key.
            let first = times[0];
            out.push(crate::mesh::SkinClip {
                name: unique(&name, &mut used),
                morphs: weights
                    .iter()
                    .flat_map(|w| times.iter().map(|&t| w.row(t)))
                    .collect(),
                times: times.iter().map(|t| t - first).collect(),
                poses,
            });
        }
        Ok(out)
    }
}
