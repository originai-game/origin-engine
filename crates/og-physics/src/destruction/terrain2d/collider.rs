//! The collision meshes of a pixel terrain's chunks and of the pieces cut
//! from it.

use super::{Cell, PixelTerrain};
use crate::MeshDesc;
use glam::{DVec2, Vec3};

/// A point on the border of the rectangle `r` (x0, y0, x1, y1), as its
/// distance clockwise from (x0, y0): up the left side, along the top, down
/// the right side, back along the bottom.
pub fn perimeter(r: [i64; 4], p: [i64; 2]) -> Option<i64> {
    let [x0, y0, x1, y1] = r;
    let (w, h) = (x1 - x0, y1 - y0);
    if p[0] == x0 && (y0..=y1).contains(&p[1]) {
        Some(p[1] - y0)
    } else if p[1] == y1 && (x0..=x1).contains(&p[0]) {
        Some(h + p[0] - x0)
    } else if p[0] == x1 && (y0..=y1).contains(&p[1]) {
        Some(h + w + y1 - p[1])
    } else if p[1] == y0 && (x0..=x1).contains(&p[0]) {
        Some(2 * h + w + x1 - p[0])
    } else {
        None
    }
}

/// Chunk `k`'s solid as closed loops in cell units: its outline's chains
/// (solid on their right), the open ones joined clockwise along the
/// chunk's border (the solid there is cut at the border), and the whole
/// chunk when solid ground runs on past every side with no outline across
/// it. Outer loops run clockwise, holes counter-clockwise.
pub fn chunk_loops(grid: &PixelTerrain, k: u32, tolerance: f32) -> Vec<Vec<[f32; 2]>> {
    let c = grid.chunk_size();
    let (cx, _) = grid.chunks();
    let (x0, y0) = ((k % cx) * c, (k / cx) * c);
    let (x1, y1) = ((x0 + c).min(grid.width()), (y0 + c).min(grid.height()));
    let r = [x0 as i64, y0 as i64, x1 as i64, y1 as i64];
    let corners = [
        (0, [x0, y0]),
        ((y1 - y0) as i64, [x0, y1]),
        ((y1 - y0 + x1 - x0) as i64, [x1, y1]),
        ((2 * (y1 - y0) + x1 - x0) as i64, [x1, y0]),
    ];
    let around = 2 * ((x1 - x0) + (y1 - y0)) as i64;
    let mut loops = Vec::new();
    let mut open: Vec<(Vec<[f32; 2]>, i64, i64)> = Vec::new();
    for chain in grid.chunk_chains(k, tolerance) {
        let n = chain.len();
        if n > 2 && chain[0] == chain[n - 1] {
            loops.push(chain[..n - 1].to_vec());
            continue;
        }
        let at = |p: [f32; 2]| perimeter(r, [p[0] as i64, p[1] as i64]);
        match (at(chain[0]), at(chain[n - 1])) {
            (Some(s), Some(e)) => open.push((chain, s, e)),
            // Not on the border (it cannot be): kept as it is.
            _ => loops.push(chain),
        }
    }
    let mut used = vec![false; open.len()];
    for first in 0..open.len() {
        if used[first] {
            continue;
        }
        let mut poly: Vec<[f32; 2]> = Vec::new();
        let mut cur = first;
        loop {
            used[cur] = true;
            poly.extend_from_slice(&open[cur].0);
            let end = open[cur].2;
            // The next chain starting clockwise from here (this one's own
            // start closes the loop).
            let next = (0..open.len())
                .filter(|&j| !used[j] || j == first)
                .min_by_key(|&j| ((open[j].1 - end).rem_euclid(around), j))
                .expect("the first chain is a candidate");
            let gap = (open[next].1 - end).rem_euclid(around);
            let mut passed: Vec<(i64, [u32; 2])> = corners
                .iter()
                .map(|&(t, p)| ((t - end).rem_euclid(around), p))
                .filter(|&(d, _)| d > 0 && d < gap)
                .collect();
            passed.sort();
            poly.extend(passed.iter().map(|(_, p)| [p[0] as f32, p[1] as f32]));
            if next == first {
                break;
            }
            cur = next;
        }
        poly.dedup();
        if poly.len() > 2 && poly[0] == poly[poly.len() - 1] {
            poly.pop();
        }
        loops.push(poly);
    }
    if open.is_empty() && runs_on(grid, r) {
        loops.push(vec![
            [x0 as f32, y0 as f32],
            [x0 as f32, y1 as f32],
            [x1 as f32, y1 as f32],
            [x1 as f32, y0 as f32],
        ]);
    }
    loops.retain(|l| l.len() > 2);
    loops
}

/// Whether solid ground in rectangle `r` runs on into a neighbouring
/// chunk's (a solid cell on its border with a solid one across it).
pub fn runs_on(grid: &PixelTerrain, r: [i64; 4]) -> bool {
    let [x0, y0, x1, y1] = r;
    let s = |x: i64, y: i64| grid.get(x, y) != 0;
    (y0..y1).any(|y| (s(x0, y) && s(x0 - 1, y)) || (s(x1 - 1, y) && s(x1, y)))
        || (x0..x1).any(|x| (s(x, y0) && s(x, y0 - 1)) || (s(x, y1 - 1) && s(x, y1)))
}

pub fn area(l: &[[f32; 2]]) -> f32 {
    crate::destruction::terrain2d::polygon_area(l)
}

/// Whether `p` is inside loop `l` (even-odd).
pub fn inside(l: &[[f32; 2]], p: [f32; 2]) -> bool {
    let mut c = false;
    for i in 0..l.len() {
        let (a, b) = (l[i], l[(i + 1) % l.len()]);
        if (a[1] > p[1]) != (b[1] > p[1]) {
            let x = a[0] + (p[1] - a[1]) * (b[0] - a[0]) / (b[1] - a[1]);
            if p[0] < x {
                c = !c;
            }
        }
    }
    c
}

/// Chunk `k`'s collision mesh, in metres from the chunk's corner: its
/// loops triangulated (each outer with the holes inside it), so its
/// outline is the loops. None for a chunk with no solid.
pub fn chunk_mesh(grid: &PixelTerrain, k: u32, tolerance: f32, cell: f32) -> Option<MeshDesc> {
    let loops = chunk_loops(grid, k, tolerance);
    let c = grid.chunk_size();
    let (cx, _) = grid.chunks();
    let corner = [((k % cx) * c) as f32, ((k / cx) * c) as f32];
    let (outers, holes): (Vec<_>, Vec<_>) = loops.into_iter().partition(|l| area(l) < 0.0);
    // Each hole to the smallest outer round it.
    let mut of: Vec<Vec<usize>> = vec![Vec::new(); outers.len()];
    for (h, hole) in holes.iter().enumerate() {
        // A point just inside the hole's first edge's left (the empty side).
        let (a, b) = (hole[0], hole[1]);
        let probe = [
            (a[0] + b[0]) * 0.5 - (b[1] - a[1]) * 0.25,
            (a[1] + b[1]) * 0.5 + (b[0] - a[0]) * 0.25,
        ];
        let best = (0..outers.len())
            .filter(|&o| inside(&outers[o], probe))
            .min_by(|&i, &j| {
                (-area(&outers[i]))
                    .total_cmp(&-area(&outers[j]))
                    .then(i.cmp(&j))
            });
        if let Some(o) = best {
            of[o].push(h);
        }
    }
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let d = |p: [f32; 2]| DVec2::new(p[0] as f64, p[1] as f64);
    for (o, outer) in outers.iter().enumerate() {
        let mut contour: Vec<DVec2> = outer.iter().map(|&p| d(p)).collect();
        let mut hs: Vec<Vec<DVec2>> = of[o]
            .iter()
            .map(|&h| holes[h].iter().map(|&p| d(p)).collect())
            .collect();
        let tris = og_geom::shape::triangulate_shape(&mut contour, &mut hs);
        let base = positions.len() as u32;
        for p in contour.iter().chain(hs.iter().flatten()) {
            positions.push(Vec3::new(
                (p.x as f32 - corner[0]) * cell,
                (p.y as f32 - corner[1]) * cell,
                0.0,
            ));
        }
        indices.extend(
            tris.iter()
                .map(|t| [base + t[0] as u32, base + t[1] as u32, base + t[2] as u32]),
        );
    }
    (!indices.is_empty()).then_some(MeshDesc { positions, indices })
}

/// A piece's convex mesh: the corners of its cells' outline, in metres
/// from its cell (0, 0)'s corner.
pub fn piece_mesh(width: u32, height: u32, cells: &[Cell], cell: f32) -> MeshDesc {
    let grid = PixelTerrain::from_cells(width, height, width.max(height), cells.to_vec())
        .expect("a piece's cells fill its rectangle");
    let mut positions: Vec<Vec3> = Vec::new();
    for chain in grid.chunk_chains(0, 0.0) {
        for p in chain {
            let v = Vec3::new(p[0] * cell, p[1] * cell, 0.0);
            if !positions.contains(&v) {
                positions.push(v);
            }
        }
    }
    MeshDesc {
        positions,
        indices: Vec::new(),
    }
}
