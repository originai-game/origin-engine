//! Seamless terrain outlines, passed directly to native polyline colliders.

use super::{Cell, PixelTerrain};
use crate::PolylineDesc;
use glam::Vec3;

/// Chunk-local segments. Open chains stop at the chunk boundary; no
/// artificial closing edge is added through solid ground. Holes are just
/// more chains, with no triangulation or containment probe.
pub fn chunk_polyline(
    grid: &PixelTerrain,
    k: u32,
    tolerance: f32,
    cell: f32,
) -> Option<PolylineDesc> {
    let c = grid.chunk_size();
    let (cx, _) = grid.chunks();
    let corner = Vec3::new(((k % cx) * c) as f32, ((k / cx) * c) as f32, 0.0);
    let mut mesh = PolylineDesc::default();
    for chain in grid.chunk_chains(k, tolerance) {
        let base = mesh.positions.len() as u32;
        mesh.positions.extend(
            chain
                .iter()
                .map(|p| (Vec3::new(p[0], p[1], 0.0) - corner) * cell),
        );
        for i in 1..chain.len() as u32 {
            mesh.segments.push([base + i - 1, base + i]);
        }
    }
    (!mesh.segments.is_empty()).then_some(mesh)
}

/// An exact convex decomposition of a pixel piece into non-overlapping
/// rectangles (x, y, width, height), in canonical bottom-row-first order.
pub fn piece_rectangles(width: u32, height: u32, cells: &[Cell]) -> Vec<[u32; 4]> {
    let mut used = vec![false; cells.len()];
    let mut out = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let solid = |x: u32, y: u32, used: &[bool]| {
                let i = (y * width + x) as usize;
                cells[i] != 0 && !used[i]
            };
            if !solid(x, y, &used) {
                continue;
            }
            let mut w = 1;
            while x + w < width && solid(x + w, y, &used) {
                w += 1;
            }
            let mut h = 1;
            while y + h < height && (x..x + w).all(|xx| solid(xx, y + h, &used)) {
                h += 1;
            }
            for yy in y..y + h {
                for xx in x..x + w {
                    used[(yy * width + xx) as usize] = true;
                }
            }
            out.push([x, y, w, h]);
        }
    }
    out
}
