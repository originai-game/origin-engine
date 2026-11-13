//! Point queries: the polygon under a point, heights, the nearest point
//! on the mesh and the walls round one.

use super::*;

impl NavMesh {
    // ── queries ──────────────────────────────────────────────────────────

    pub fn poly(&self, r: PolyRef) -> Option<(&Tile, &Poly)> {
        let (t, p) = split_ref(r);
        let tile = self.tiles.get(t)?;
        Some((tile, tile.polys.get(p)?))
    }

    pub(in crate::nav::mesh) fn xz(v: Vec3) -> Vec2 {
        Vec2::new(v.x, v.z)
    }

    /// Whether `p` (x, z) is inside a polygon or on its edges.
    pub(in crate::nav::mesh) fn inside(tile: &Tile, poly: &Poly, p: Vec2) -> bool {
        let n = poly.verts.len();
        (0..n).all(|i| {
            let a = Self::xz(tile.verts[poly.verts[i] as usize]);
            let b = Self::xz(tile.verts[poly.verts[(i + 1) % n] as usize]);
            (b - a).perp_dot(p - a) >= -1e-5
        })
    }

    /// The surface height of a polygon at (x, z): its fan triangle's plane.
    pub fn height_at(tile: &Tile, poly: &Poly, p: Vec2) -> f32 {
        let v = |i: usize| tile.verts[poly.verts[i] as usize];
        let mut best = (f32::MAX, v(0).y);
        for i in 1..poly.verts.len() - 1 {
            let (a, b, c) = (v(0), v(i), v(i + 1));
            let (a2, b2, c2) = (Self::xz(a), Self::xz(b), Self::xz(c));
            let den = (b2 - a2).perp_dot(c2 - a2);
            if den.abs() < 1e-9 {
                continue;
            }
            let u = (p - a2).perp_dot(c2 - a2) / den;
            let w = (b2 - a2).perp_dot(p - a2) / den;
            let outside = (-u).max(0.0) + (-w).max(0.0) + (u + w - 1.0).max(0.0);
            let y = a.y + (b.y - a.y) * u + (c.y - a.y) * w;
            if outside < best.0 {
                best = (outside, y);
            }
            if outside == 0.0 {
                break;
            }
        }
        best.1
    }

    /// The closest point of a polygon to `p` (on its surface when `p` is above or below it).
    pub fn closest_on(tile: &Tile, poly: &Poly, p: Vec3) -> Vec3 {
        let q = Self::xz(p);
        if Self::inside(tile, poly, q) {
            return Vec3::new(p.x, Self::height_at(tile, poly, q), p.z);
        }
        let n = poly.verts.len();
        let mut best = (f32::MAX, Vec3::ZERO);
        for i in 0..n {
            let a = tile.verts[poly.verts[i] as usize];
            let b = tile.verts[poly.verts[(i + 1) % n] as usize];
            let ab = Self::xz(b) - Self::xz(a);
            let len2 = ab.length_squared();
            let t = if len2 > 0.0 {
                ((q - Self::xz(a)).dot(ab) / len2).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let c = a + (b - a) * t;
            let d = (Self::xz(c) - q).length_squared();
            if d < best.0 {
                best = (d, c);
            }
        }
        best.1
    }

    /// The boundary edges (a wall, a drop: an edge with no polygon across
    /// it) within `range` of `p` on the plane, found walking out from
    /// polygon `from` through polygons within `range` (at most 64), in that
    /// walk's order: what crowd avoidance keeps clear of.
    pub fn walls_near(&self, from: PolyRef, p: Vec3, range: f32) -> Vec<(Vec3, Vec3)> {
        let mut out = Vec::new();
        if self.poly(from).is_none() || range.is_nan() || range < 0.0 {
            return out;
        }
        let q = Self::xz(p);
        let r2 = range * range;
        let mut seen = vec![from];
        let mut next = 0;
        while next < seen.len() && seen.len() <= 64 {
            let r = seen[next];
            next += 1;
            let Some((tile, poly)) = self.poly(r) else {
                continue;
            };
            let n = poly.verts.len();
            for i in 0..n {
                let a = tile.verts[poly.verts[i] as usize];
                let b = tile.verts[poly.verts[(i + 1) % n] as usize];
                let (a2, b2) = (Self::xz(a), Self::xz(b));
                let ab = b2 - a2;
                let len2 = ab.length_squared();
                let t = if len2 > 0.0 {
                    ((q - a2).dot(ab) / len2).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                if (a2 + ab * t - q).length_squared() > r2 {
                    continue;
                }
                match poly.links[i] {
                    NONE => out.push((a, b)),
                    other if !seen.contains(&other) => seen.push(other),
                    _ => {}
                }
            }
        }
        out
    }

    /// The polygon nearest `p` within `extent` (half sizes) and the nearest
    /// point on it: polygons under or over `p` first, by height difference.
    pub fn nearest(&self, p: Vec3, extent: Vec3) -> Option<(PolyRef, Vec3)> {
        let (x0, z0, x1, z1) = self.cells_of(p - extent, p + extent);
        let ts = self.field.params.tile_size.max(4) as i32;
        let mut best: Option<(f32, PolyRef, Vec3)> = None;
        for tz in (z0 / ts)..=((z1 - 1).max(0) / ts) {
            for tx in (x0 / ts)..=((x1 - 1).max(0) / ts) {
                if tx < 0 || tz < 0 || tx >= self.tiles_x as i32 || tz >= self.tiles_z as i32 {
                    continue;
                }
                let t = (tz as u32 * self.tiles_x + tx as u32) as usize;
                let tile = &self.tiles[t];
                for (pi, poly) in tile.polys.iter().enumerate() {
                    let c = Self::closest_on(tile, poly, p);
                    let d = c - p;
                    if d.x.abs() > extent.x || d.y.abs() > extent.y || d.z.abs() > extent.z {
                        continue;
                    }
                    let score = d.length_squared();
                    let r = poly_ref(t, pi);
                    if best.is_none_or(|(bs, br, _)| score < bs || (score == bs && r < br)) {
                        best = Some((score, r, c));
                    }
                }
            }
        }
        best.map(|(_, r, c)| (r, c))
    }

    pub(in crate::nav::mesh) fn edge(&self, r: PolyRef, e: usize) -> (Vec3, Vec3) {
        let (tile, poly) = self.poly(r).expect("a live polygon");
        let n = poly.verts.len();
        (
            tile.verts[poly.verts[e] as usize],
            tile.verts[poly.verts[(e + 1) % n] as usize],
        )
    }
}
