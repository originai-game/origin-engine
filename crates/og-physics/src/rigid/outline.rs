//! The outline of a flat triangle mesh: what a 2D [`crate::SHAPE_TRIMESH`]
//! collider is (a polyline of the edges no two triangles share), and what a
//! generated prism's walls are built from (a tilemap's merged solid
//! regions, G2-6).

use crate::MeshDesc;
use og_core::glam::Vec2;

/// The closed loops around the triangles of `mesh` in the xy plane, as
/// indices into its positions: every edge only one triangle has, chained
/// end to start with the triangles on the left (counter-clockwise around
/// solid, clockwise around a hole), collinear runs merged into one edge.
/// Vertices are matched by their exact xy, so triangles that share a corner
/// must give it the same coordinates (a corner is named by its lowest
/// index). Zero-area triangles are skipped. Loops come in the order of
/// their first edge in the mesh; each starts at that edge.
pub fn outline(mesh: &MeshDesc) -> Vec<Vec<u32>> {
    // One id per distinct xy, in the order positions first give it: an
    // open-addressed table of the xy bits (its slots are never iterated).
    let xy: Vec<Vec2> = mesh.positions.iter().map(|p| p.truncate()).collect();
    let bits = (2 * xy.len()).max(2).next_power_of_two();
    let shift = 64 - bits.trailing_zeros();
    let mut slots: Vec<(u64, u32)> = vec![(0, 0); bits];
    let mut id = vec![0u32; xy.len()];
    let mut name: Vec<u32> = Vec::new();
    for (i, p) in xy.iter().enumerate() {
        let cx = if p.x == 0.0 { 0.0f32 } else { p.x };
        let cy = if p.y == 0.0 { 0.0f32 } else { p.y };
        let key = (cx.to_bits() as u64) << 32 | cy.to_bits() as u64;
        let mut s = (key.wrapping_mul(0x9e37_79b9_7f4a_7c15) >> shift) as usize;
        loop {
            match slots[s] {
                (_, 0) => {
                    name.push(i as u32);
                    slots[s] = (key, name.len() as u32);
                    id[i] = name.len() as u32 - 1;
                    break;
                }
                (k, v) if k == key => {
                    id[i] = v - 1;
                    break;
                }
                _ => s = (s + 1) & (bits - 1),
            }
        }
    }
    let at = |v: u32| xy[name[v as usize] as usize];
    // Directed edges, each unique non-degenerate triangle turned counter-clockwise.
    let mut seen_tris = std::collections::BTreeSet::new();
    let mut edges: Vec<(u32, u32)> = Vec::with_capacity(mesh.indices.len() * 3);
    for t in &mesh.indices {
        let [a, b, c] = t.map(|i| id[i as usize]);
        let area = (at(b) - at(a)).perp_dot(at(c) - at(a));
        if area == 0.0 || !area.is_finite() {
            continue;
        }
        let mut tri_key = [a, b, c];
        tri_key.sort_unstable();
        if !seen_tris.insert(tri_key) {
            continue;
        }
        let (b, c) = if area > 0.0 { (b, c) } else { (c, b) };
        edges.extend([(a, b), (b, c), (c, a)]);
    }
    // Each edge filed under its lower vertex (a counting sort): an edge is
    // on the outline when it has no opposite neighbour edge (deduplicated).
    let n = name.len();
    let mut start = vec![0u32; n + 1];
    for &(a, b) in &edges {
        start[a.min(b) as usize + 1] += 1;
    }
    for v in 0..n {
        start[v + 1] += start[v];
    }
    let mut fill = start.clone();
    let mut filed = vec![0u32; edges.len()];
    for (e, &(a, b)) in edges.iter().enumerate() {
        let slot = &mut fill[a.min(b) as usize];
        filed[*slot as usize] = e as u32;
        *slot += 1;
    }
    let mut on = vec![false; edges.len()];
    for v in 0..n {
        let bucket = &filed[start[v] as usize..start[v + 1] as usize];
        for (pos, &e) in bucket.iter().enumerate() {
            let (a, b) = edges[e as usize];
            // Emit each boundary directed edge at most once (first occurrence).
            if bucket[..pos].iter().any(|&f| edges[f as usize] == (a, b)) {
                continue;
            }
            let net: i32 = bucket
                .iter()
                .map(|&f| match edges[f as usize] {
                    (c, d) if (c, d) == (a, b) => 1,
                    (c, d) if (c, d) == (b, a) => -1,
                    _ => 0,
                })
                .sum();
            on[e as usize] = net > 0;
        }
    }
    let boundary: Vec<(u32, u32)> = edges
        .iter()
        .zip(&on)
        .filter(|(_, on)| **on)
        .map(|(e, _)| *e)
        .collect();
    // Outgoing edges per start vertex, in mesh order; each edge used once.
    let mut out_start = vec![0u32; n + 1];
    for &(a, _) in &boundary {
        out_start[a as usize + 1] += 1;
    }
    for v in 0..n {
        out_start[v + 1] += out_start[v];
    }
    let mut fill = out_start.clone();
    let mut outgoing = vec![0u32; boundary.len()];
    for (e, &(a, _)) in boundary.iter().enumerate() {
        let slot = &mut fill[a as usize];
        outgoing[*slot as usize] = e as u32;
        *slot += 1;
    }
    let mut used = vec![false; boundary.len()];
    let mut loops = Vec::new();
    for first in 0..boundary.len() {
        if used[first] {
            continue;
        }
        let start_vertex = boundary[first].0;
        let mut points = Vec::new();
        let mut i = first;
        let mut closed = false;
        loop {
            used[i] = true;
            let (a, b) = boundary[i];
            points.push(a);
            if b == start_vertex {
                closed = true;
                break;
            }
            let cands = &outgoing[out_start[b as usize] as usize..out_start[b as usize + 1] as usize];
            // At a pinch vertex (multiple unused outgoing edges), pick the
            // sharpest right turn (keeps solid on the left, avoids figure-8).
            let mut next = None;
            let mut best_angle = f32::INFINITY;
            let in_dir = (at(b) - at(a)).normalize_or_zero();
            for &e_u32 in cands {
                let e = e_u32 as usize;
                if used[e] {
                    continue;
                }
                if next.is_none() && cands.len() == 1 {
                    next = Some(e);
                    break;
                }
                let (_, c) = boundary[e];
                let out_dir = (at(c) - at(b)).normalize_or_zero();
                // Turn angle in (-pi, pi]: most negative is sharpest right turn.
                let angle = og_core::math::atan2(in_dir.perp_dot(out_dir), in_dir.dot(out_dir));
                if angle < best_angle {
                    best_angle = angle;
                    next = Some(e);
                }
            }
            match next {
                Some(e) => i = e,
                None => break,
            }
        }
        if !closed {
            continue;
        }
        let points = merge_collinear(points, &at);
        if points.len() >= 3 {
            loops.push(points.into_iter().map(|p| name[p as usize]).collect());
        }
    }
    loops
}

/// A closed loop without points that lie on the straight line between
/// their neighbours (sequentially, so a gentle curve never collapses whole).
fn merge_collinear(mut points: Vec<u32>, at: &dyn Fn(u32) -> Vec2) -> Vec<u32> {
    let mut i = 0;
    while points.len() >= 3 && i < points.len() {
        let n = points.len();
        let p = at(points[(i + n - 1) % n]);
        let q = at(points[i]);
        let r = at(points[(i + 1) % n]);
        let (u, v) = (q - p, r - q);
        if u.perp_dot(v).abs() <= 1e-6 * u.length() * v.length() && u.dot(v) > 0.0 {
            points.remove(i);
            i = i.saturating_sub(1);
        } else {
            i += 1;
        }
    }
    points
}
