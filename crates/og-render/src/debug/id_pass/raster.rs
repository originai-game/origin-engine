//! The id pass's software raster: depth-tested triangles of ids.

use super::*;

/// Whether a pixel whose point (a terrain's xz) is this is cut out: a
/// terrain's hole, or where a grass field grows nothing.
pub(super) type CutTest<'a> = dyn Fn(Vec2) -> bool + 'a;

/// A triangle's corners' points and the test that cuts a pixel out.
pub(super) type Cut<'a> = ([Vec2; 3], &'a CutTest<'a>);

pub(super) struct Raster {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) ids: Vec<u32>,
    pub(super) depth: Vec<f32>,
    /// The meshes' depth, which sprites test against (taken before the first sprite).
    pub(super) mesh_depth: Vec<f32>,
    /// Per pixel, the last group that covered it drawn alone.
    pub(super) stamp: Vec<u32>,
    pub(super) group: u32,
    /// Per id: pixels covered drawn alone, and their bounds.
    pub(super) drawn: IdMap<u32, (u32, [u32; 4])>,
}

impl Raster {
    /// Clips a clip-space triangle to the near plane (z >= 0) and fills it.
    /// With `cut`, each corner's point (a terrain's xz) is carried to every
    /// pixel, perspective-correct, and a pixel whose point the test cuts
    /// out is not covered (a terrain's holes).
    pub(super) fn triangle(&mut self, id: u32, sprite: bool, tri: [Vec4; 3], cut: Option<Cut>) {
        let at = |i: usize| cut.map_or(Vec2::ZERO, |(p, _)| p[i]);
        let mut poly: Vec<(Vec4, Vec2)> = Vec::with_capacity(4);
        for i in 0..3 {
            let j = (i + 1) % 3;
            let (a, b) = (tri[i], tri[j]);
            let (da, db) = (a.z, b.z);
            if da >= 0.0 {
                poly.push((a, at(i)));
            }
            if (da >= 0.0) != (db >= 0.0) {
                let t = da / (da - db);
                poly.push((a + (b - a) * t, at(i) + (at(j) - at(i)) * t));
            }
        }
        if poly.len() < 3 || poly.iter().any(|(p, _)| p.w <= 1e-6) {
            return;
        }
        let screen: Vec<(Vec3, Vec3)> = poly
            .iter()
            .map(|(p, q)| {
                let ndc = p.truncate() / p.w;
                let s = Vec3::new(
                    (ndc.x * 0.5 + 0.5) * self.width as f32,
                    (0.5 - ndc.y * 0.5) * self.height as f32,
                    ndc.z,
                );
                (s, (*q / p.w).extend(1.0 / p.w))
            })
            .collect();
        for k in 1..screen.len() - 1 {
            let [a, b, c] = [screen[0], screen[k], screen[k + 1]];
            let test = cut.map(|(_, hole)| ([a.1, b.1, c.1], hole));
            self.fill(id, sprite, [a.0, b.0, c.0], test);
        }
    }

    /// Fills a screen-space triangle (x, y in pixels, z the NDC depth):
    /// pixels whose centre is inside, either winding. `test`: per corner
    /// a point over w and 1 / w, and the test that cuts a pixel out.
    pub(super) fn fill(
        &mut self,
        id: u32,
        sprite: bool,
        [a, b, c]: [Vec3; 3],
        test: Option<([Vec3; 3], &CutTest)>,
    ) {
        let area = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
        if area.abs() < 1e-12 || !area.is_finite() {
            return;
        }
        let lo_x = a.x.min(b.x).min(c.x).floor().max(0.0) as u32;
        let lo_y = a.y.min(b.y).min(c.y).floor().max(0.0) as u32;
        let hi_x = (a.x.max(b.x).max(c.x).ceil().max(0.0) as u32).min(self.width);
        let hi_y = (a.y.max(b.y).max(c.y).ceil().max(0.0) as u32).min(self.height);
        let edge =
            |p: Vec3, q: Vec3, x: f32, y: f32| (q.x - p.x) * (y - p.y) - (q.y - p.y) * (x - p.x);
        for y in lo_y..hi_y {
            for x in lo_x..hi_x {
                let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                let w0 = edge(b, c, px, py) / area;
                let w1 = edge(c, a, px, py) / area;
                let w2 = edge(a, b, px, py) / area;
                if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                    continue;
                }
                let z = w0 * a.z + w1 * b.z + w2 * c.z;
                if !(0.0..=1.0).contains(&z) {
                    continue;
                }
                if let Some(([qa, qb, qc], hole)) = test {
                    let q = qa * w0 + qb * w1 + qc * w2;
                    if hole(q.truncate() / q.z) {
                        continue;
                    }
                }
                let k = (y * self.width + x) as usize;
                if self.stamp[k] != self.group {
                    self.stamp[k] = self.group;
                    let entry = self.drawn.entry(id).or_insert((0, [x, y, x + 1, y + 1]));
                    entry.0 += 1;
                    let bb = &mut entry.1;
                    *bb = [
                        bb[0].min(x),
                        bb[1].min(y),
                        bb[2].max(x + 1),
                        bb[3].max(y + 1),
                    ];
                }
                if sprite {
                    // Over the meshes in front of nothing nearer; sprites
                    // over sprites in their draw order.
                    if z <= self.mesh_depth[k] {
                        self.ids[k] = id;
                        self.depth[k] = z;
                    }
                } else if z < self.depth[k] {
                    self.ids[k] = id;
                    self.depth[k] = z;
                }
            }
        }
    }
}
