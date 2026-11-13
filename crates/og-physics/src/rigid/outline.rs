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
        let key = (p.x.to_bits() as u64) << 32 | p.y.to_bits() as u64;
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
    // Directed edges, each triangle turned counter-clockwise.
    let mut edges: Vec<(u32, u32)> = Vec::with_capacity(mesh.indices.len() * 3);
    for t in &mesh.indices {
        let [a, b, c] = t.map(|i| id[i as usize]);
        let area = (at(b) - at(a)).perp_dot(at(c) - at(a));
        if area == 0.0 || !area.is_finite() {
            continue;
        }
        let (b, c) = if area > 0.0 { (b, c) } else { (c, b) };
        edges.extend([(a, b), (b, c), (c, a)]);
    }
    // Each edge filed under its lower vertex (a counting sort): an edge is
    // on the outline when no triangle has it the other way (a neighbour
    // across it) as often.
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
        for &e in bucket {
            let (a, b) = edges[e as usize];
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
        let mut points = Vec::new();
        let mut i = first;
        loop {
            used[i] = true;
            let (a, b) = boundary[i];
            points.push(a);
            let next = outgoing[out_start[b as usize] as usize..out_start[b as usize + 1] as usize]
                .iter()
                .map(|&e| e as usize)
                .find(|&e| !used[e]);
            match next {
                Some(e) => i = e,
                None => break,
            }
        }
        let points = merge_collinear(points, &at);
        if points.len() >= 3 {
            loops.push(points.into_iter().map(|p| name[p as usize]).collect());
        }
    }
    loops
}

/// A closed loop without the points that lie on the straight line between
/// their neighbours (exactly, up to a relative 1e-6).
fn merge_collinear(points: Vec<u32>, at: &dyn Fn(u32) -> Vec2) -> Vec<u32> {
    let n = points.len();
    if n < 3 {
        return points;
    }
    let straight = |i: usize| {
        let [p, q, r] = [(i + n - 1) % n, i, (i + 1) % n].map(|k| at(points[k]));
        let (u, v) = (q - p, r - q);
        u.perp_dot(v).abs() <= 1e-6 * u.length() * v.length() && u.dot(v) > 0.0
    };
    (0..n)
        .filter(|&i| !straight(i))
        .map(|i| points[i])
        .collect()
}
