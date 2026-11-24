//! og.Water drawn over a mesh (`geometry`): still water standing where the
//! mesh's triangles are, at their height.

use og_core::glam::{Vec2, Vec3, Vec3Swizzles};

/// The most cells along either side of a mesh's grid.
const MAX_SIDE: u32 = 256;

/// A water mesh's triangles in world space, binned on a grid over xz so a
/// point reads only the few under it: a lake of any outline, a channel, a
/// pool. Its surface over a point is the highest triangle there, its height
/// between its corners'; off every triangle there is no water. Plain
/// `+ - * /` on f32 in a fixed order, so every host answers the same.
#[derive(Clone, Debug, PartialEq)]
pub struct WaterMesh {
    triangles: Vec<[Vec3; 3]>,
    min: Vec2,
    max: Vec2,
    cell: f32,
    columns: u32,
    rows: u32,
    /// Cell `c`'s triangles are `index[start[c]..start[c + 1]]`.
    start: Vec<u32>,
    index: Vec<u32>,
}

/// Twice a triangle's signed area seen from above.
fn area2(t: &[Vec3; 3]) -> f32 {
    (t[1].xz() - t[0].xz()).perp_dot(t[2].xz() - t[0].xz())
}

impl WaterMesh {
    /// The water over `triangles` (world space). Triangles seen edge-on
    /// from above (a wall) or not finite hold no water; `None` when none
    /// is left.
    pub fn new(triangles: impl IntoIterator<Item = [Vec3; 3]>) -> Option<WaterMesh> {
        let triangles: Vec<[Vec3; 3]> = triangles
            .into_iter()
            .filter(|t| t.iter().all(|p| p.is_finite()) && area2(t).abs() > 1e-9)
            .collect();
        if triangles.is_empty() {
            return None;
        }
        let (mut min, mut max) = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
        for p in triangles.iter().flatten() {
            min = min.min(p.xz());
            max = max.max(p.xz());
        }
        // About two triangles a cell.
        let extent = (max - min).max(Vec2::splat(1e-3));
        let side = ((triangles.len() as f32 * 0.5).sqrt().ceil() as u32).clamp(1, MAX_SIDE);
        let cell = extent.max_element() / side as f32;
        let columns = ((extent.x / cell).ceil() as u32).clamp(1, MAX_SIDE);
        let rows = ((extent.y / cell).ceil() as u32).clamp(1, MAX_SIDE);
        let mut mesh = WaterMesh {
            triangles,
            min,
            max,
            cell,
            columns,
            rows,
            start: vec![0; (columns * rows + 1) as usize],
            index: Vec::new(),
        };
        // Each triangle in every cell its box touches: counted, then placed.
        let spans: Vec<_> = mesh.triangles.iter().map(|t| mesh.span(t)).collect();
        for &((c0, r0), (c1, r1)) in &spans {
            for r in r0..=r1 {
                for c in c0..=c1 {
                    mesh.start[(r * columns + c) as usize + 1] += 1;
                }
            }
        }
        for i in 1..mesh.start.len() {
            mesh.start[i] += mesh.start[i - 1];
        }
        let mut next = mesh.start.clone();
        mesh.index = vec![0; *mesh.start.last().unwrap() as usize];
        for (i, &((c0, r0), (c1, r1))) in spans.iter().enumerate() {
            for r in r0..=r1 {
                for c in c0..=c1 {
                    let at = &mut next[(r * columns + c) as usize];
                    mesh.index[*at as usize] = i as u32;
                    *at += 1;
                }
            }
        }
        Some(mesh)
    }

    /// The grid cell over a point, clamped to the grid.
    fn cell_of(&self, p: Vec2) -> (u32, u32) {
        let at = ((p - self.min) / self.cell).floor();
        (
            (at.x.max(0.0) as u32).min(self.columns - 1),
            (at.y.max(0.0) as u32).min(self.rows - 1),
        )
    }

    /// The first and last cells a triangle's box touches.
    fn span(&self, t: &[Vec3; 3]) -> ((u32, u32), (u32, u32)) {
        let lo = t[0].xz().min(t[1].xz()).min(t[2].xz());
        let hi = t[0].xz().max(t[1].xz()).max(t[2].xz());
        (self.cell_of(lo), self.cell_of(hi))
    }

    /// The surface's world height over (`x`, `z`): the highest triangle
    /// over it, between its corners' heights; `None` off the mesh.
    pub fn level(&self, x: f32, z: f32) -> Option<f32> {
        let p = Vec2::new(x, z);
        if !(p.cmpge(self.min).all() && p.cmple(self.max).all()) {
            return None;
        }
        let (c, r) = self.cell_of(p);
        let cell = (r * self.columns + c) as usize;
        let mut best: Option<f32> = None;
        for &i in &self.index[self.start[cell] as usize..self.start[cell + 1] as usize] {
            let t = &self.triangles[i as usize];
            let (a, b, c) = (t[0].xz(), t[1].xz(), t[2].xz());
            let d = area2(t);
            // Each corner's share of the point, read so a point on an edge
            // two triangles share is in both.
            let u = (b - p).perp_dot(c - p) / d;
            let v = (c - p).perp_dot(a - p) / d;
            let w = 1.0 - u - v;
            const EDGE: f32 = -1e-6;
            if u < EDGE || v < EDGE || w < EDGE {
                continue;
            }
            // From its first corner, so flat water reads exactly level.
            let y = t[0].y + v * (t[1].y - t[0].y) + w * (t[2].y - t[0].y);
            if best.is_none_or(|h| y > h) {
                best = Some(y);
            }
        }
        best
    }
}
