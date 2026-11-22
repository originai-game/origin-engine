//! UVs and vertex colours of the input kept on the repaired solid.

use super::*;

/// After the fuse step: each corner of `after` on a vertex of `before`
/// (the same position, exactly) takes the UV `before` gives that vertex,
/// the nearest when it has several (one per side of a seam), in place of
/// the boolean's interpolation. Interpolating rounds differently in each
/// triangle around a vertex, which made every edge a UV seam: on an
/// image-to-3D mesh of 150 000 triangles, 31 611 seam edges after fusing
/// for 6 237 before, and decimation then stopped at 31 304 triangles for
/// a target of 2 000 (`docs/research.md` §28).
pub(crate) fn keep_input_uvs(before: &Solid, mut after: Solid) -> Result<Solid, RepairError> {
    let (Some(old), Some(new)) = (&before.uvs, &mut after.uvs) else {
        return Ok(after);
    };
    // `+ 0.0` makes −0 and 0 one key; positions are finite.
    let key = |p: DVec3| [p.x + 0.0, p.y + 0.0, p.z + 0.0].map(f64::to_bits);
    let mut at: BTreeMap<[u64; 3], Vec<DVec2>> = BTreeMap::new();
    for (t, tri) in before.triangles.iter().enumerate() {
        for (k, &v) in tri.iter().enumerate() {
            at.entry(key(before.positions[v as usize]))
                .or_default()
                .push(old[t][k]);
        }
    }
    for uvs in at.values_mut() {
        uvs.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
        uvs.dedup();
    }
    for (t, tri) in after.triangles.iter().enumerate() {
        for (k, &v) in tri.iter().enumerate() {
            let Some(uvs) = at.get(&key(after.positions[v as usize])) else {
                continue;
            };
            let here = new[t][k];
            // The nearest; the first of equals in sorted order.
            let mut best = uvs[0];
            for &u in &uvs[1..] {
                if u.distance_squared(here) < best.distance_squared(here) {
                    best = u;
                }
            }
            new[t][k] = best;
        }
    }
    after.canonicalize()?;
    Ok(after)
}

/// After the fuse step, as [`keep_input_uvs`] for colours: a corner on
/// a vertex of `before` takes the colour `before` gives it there (the
/// nearest of its colours), so no colour border appears where the file
/// had none. Corners the union made (on cut lines) keep its
/// interpolation.
pub(crate) fn keep_input_colors(before: &Solid, mut after: Solid) -> Result<Solid, RepairError> {
    let Some(old) = &before.colors else {
        return Ok(after);
    };
    let key = |p: DVec3| [p.x + 0.0, p.y + 0.0, p.z + 0.0].map(f64::to_bits);
    let mut at: BTreeMap<[u64; 3], Vec<[f32; 4]>> = BTreeMap::new();
    for (t, tri) in before.triangles.iter().enumerate() {
        for (k, &v) in tri.iter().enumerate() {
            at.entry(key(before.positions[v as usize]))
                .or_default()
                .push(old[t][k]);
        }
    }
    let cmp = |a: &[f32; 4], b: &[f32; 4]| {
        (0..4)
            .map(|k| a[k].total_cmp(&b[k]))
            .find(|o| o.is_ne())
            .unwrap_or(core::cmp::Ordering::Equal)
    };
    for c in at.values_mut() {
        c.sort_by(cmp);
        c.dedup();
    }
    let n = after.triangles.len();
    let new = after.colors.get_or_insert_with(|| vec![[WHITE; 3]; n]);
    let dist = |a: &[f32; 4], b: &[f32; 4]| {
        (0..4)
            .map(|k| {
                let d = f64::from(a[k]) - f64::from(b[k]);
                d * d
            })
            .sum::<f64>()
    };
    // Corners the union made, per vertex.
    let mut made: BTreeMap<u32, Vec<(usize, usize)>> = BTreeMap::new();
    for (t, tri) in after.triangles.iter().enumerate() {
        for (k, &v) in tri.iter().enumerate() {
            let Some(cs) = at.get(&key(after.positions[v as usize])) else {
                made.entry(v).or_default().push((t, k));
                continue;
            };
            let here = new[t][k];
            let mut best = cs[0];
            for c in &cs[1..] {
                if dist(c, &here) < dist(&best, &here) {
                    best = *c;
                }
            }
            new[t][k] = best;
        }
    }
    // A vertex on a cut line is interpolated once per triangle around it,
    // each rounding its own way: corners that differ by less than half an
    // 8-bit step take their mean (in sorted order), so rounding makes no
    // colour border; a larger difference (two sheets of different colour
    // meeting) stays one.
    for corners in made.values() {
        let mut cs: Vec<[f32; 4]> = corners.iter().map(|&(t, k)| new[t][k]).collect();
        let spread = (0..4)
            .map(|ch| {
                let (lo, hi) = cs
                    .iter()
                    .fold((f64::INFINITY, f64::NEG_INFINITY), |(l, h), c| {
                        (l.min(f64::from(c[ch])), h.max(f64::from(c[ch])))
                    });
                hi - lo
            })
            .fold(0.0, f64::max);
        if spread == 0.0 || spread >= 1.0 / 512.0 {
            continue;
        }
        cs.sort_by(cmp);
        let n = cs.len() as f64;
        let mean = crate::mesh::stored(core::array::from_fn(|ch| {
            cs.iter().map(|c| f64::from(c[ch])).sum::<f64>() / n
        }));
        for &(t, k) in corners {
            new[t][k] = mean;
        }
    }
    after.canonicalize()?;
    Ok(after)
}
