//! Local Jacobi relaxation on position-welded adjacency. Ordered groups,
//! neighbours and joint tie breaks make the same input produce the same weights.

use super::{Pose, Vec3, stretched};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn relax(
    positions: &[Vec3],
    indices: &[u32],
    palettes: &[&Pose],
    joints: &mut [[u16; 4]],
    weights: &mut [[f32; 4]],
) -> usize {
    let bad = |js: &[[u16; 4]], ws: &[[f32; 4]]| -> Vec<Vec<usize>> {
        palettes
            .iter()
            .map(|p| stretched(positions, indices, js, ws, p))
            .collect()
    };
    let mut found = bad(joints, weights);
    if found.iter().all(Vec::is_empty) {
        return 0;
    }
    let original = (joints.to_vec(), weights.to_vec());
    let mut best = original.clone();
    let mut best_counts: Vec<_> = found.iter().map(Vec::len).collect();
    // Five decimal places matches the source-position seam weld used by the
    // authoring repair; do not weld by normals or UVs (those split seams).
    let mut groups = BTreeMap::new();
    let weld: Vec<_> = positions
        .iter()
        .map(|p| {
            let key = palettes[0]
                .bind
                .transform_point3(*p)
                .to_array()
                .map(|v| (f64::from(v) * 100_000.0).round() as i64);
            let next = groups.len();
            *groups.entry(key).or_insert(next)
        })
        .collect();
    let mut near = vec![BTreeSet::new(); groups.len()];
    for t in indices.as_chunks::<3>().0 {
        let ids = t.map(|v| weld[v as usize]);
        for a in ids {
            for b in ids {
                if a != b {
                    near[a].insert(b);
                }
            }
        }
    }
    // Averaging cannot introduce a joint absent from the source. This also
    // keeps zero-weight slots valid for bindings with no topology probes.
    let count = joints
        .iter()
        .flatten()
        .copied()
        .max()
        .map_or(0, |j| j as usize + 1);
    let mut dense = vec![vec![0.0f32; count]; groups.len()];
    let mut counts = vec![0; groups.len()];
    for ((&g, js), ws) in weld.iter().zip(joints.iter()).zip(weights.iter()) {
        counts[g] += 1;
        for (&j, &w) in js.iter().zip(ws) {
            dense[g][j as usize] += w;
        }
    }
    for (row, n) in dense.iter_mut().zip(counts) {
        for w in row {
            *w /= n as f32;
        }
    }
    let mut touched = BTreeSet::new();
    for pass in 0..12 {
        if found.iter().all(Vec::is_empty) {
            break;
        }
        let mut region: BTreeSet<_> = found
            .iter()
            .flatten()
            .flat_map(|&id| {
                indices[id * 3..id * 3 + 3]
                    .iter()
                    .map(|&v| weld[v as usize])
            })
            .collect();
        for _ in 0..1 + pass {
            let neighbours: Vec<_> = region
                .iter()
                .flat_map(|&g| near[g].iter().copied())
                .collect();
            region.extend(neighbours);
        }
        for _ in 0..16 {
            let mut next = dense.clone();
            for &g in &region {
                if near[g].is_empty() {
                    continue;
                }
                for j in 0..count {
                    let mean =
                        near[g].iter().map(|&n| dense[n][j]).sum::<f32>() / near[g].len() as f32;
                    next[g][j] = 0.5 * dense[g][j] + 0.5 * mean;
                }
            }
            dense = next;
        }
        touched.extend(region);
        let mut packed = BTreeMap::new();
        for &g in &touched {
            let mut top: Vec<_> = (0..count).collect();
            top.sort_by(|&a, &b| dense[g][b].total_cmp(&dense[g][a]).then(a.cmp(&b)));
            let mut js = [0; 4];
            let mut ws = [0.0; 4];
            for (slot, &j) in top.iter().take(4).enumerate() {
                js[slot] = j as u16;
                ws[slot] = dense[g][j];
            }
            let sum: f32 = ws.iter().sum();
            if sum > 0.0 {
                for w in &mut ws {
                    *w /= sum;
                }
            }
            packed.insert(g, (js, ws));
        }
        for ((&g, js), ws) in weld.iter().zip(joints.iter_mut()).zip(weights.iter_mut()) {
            if let Some(&(j, w)) = packed.get(&g) {
                *js = j;
                *ws = w;
            }
        }
        found = bad(joints, weights);
        // Exploration may temporarily spread stretch across the boundary of
        // the relaxed region. Never publish a pass that worsens any probe.
        if found
            .iter()
            .zip(&best_counts)
            .all(|(ids, &n)| ids.len() <= n)
            && found
                .iter()
                .zip(&best_counts)
                .any(|(ids, &n)| ids.len() < n)
        {
            best = (joints.to_vec(), weights.to_vec());
            best_counts = found.iter().map(Vec::len).collect();
        }
    }
    joints.copy_from_slice(&best.0);
    weights.copy_from_slice(&best.1);
    joints
        .iter()
        .zip(weights.iter())
        .zip(original.0.iter().zip(&original.1))
        .filter(|((js, ws), (old_js, old_ws))| js != old_js || ws != old_ws)
        .count()
}
