//! Conformal parametrization of a chart (least squares), for surfaces no projection flattens.

use super::*;

/// The triangles `members` flattened as one chart that keeps its angles
/// (a least squares conformal map, Lévy et al. 2002; `docs/research.md`
/// §55), in millimetres, each triangle in `members`' order; `None`
/// where the chart folds, lies over itself, or stretches a triangle's
/// area past twice or under half.
///
/// Each triangle asks that its flat shape be its own shape turned and
/// scaled: two linear equations in its corners' coordinates, weighted
/// by the root of its area. Two vertices far apart are pinned at their
/// true distance, and the rest is the least squares answer, found by
/// conjugate gradients on the equations (CGLS) from the chart's
/// projection on its mean plane.
pub(crate) fn conformal(
    s: &Solid,
    members: &[u32],
    normals: &[DVec3],
    areas: &[f64],
    cell: f64,
    turns: &mut usize,
) -> Option<Vec<[DVec2; 3]>> {
    // The chart's vertices, numbered in the order met.
    let mut number: BTreeMap<u32, usize> = BTreeMap::new();
    let mut points: Vec<DVec3> = Vec::new();
    let tris: Vec<[usize; 3]> = members
        .iter()
        .map(|&i| {
            s.triangles[i as usize].map(|v| {
                *number.entry(v).or_insert_with(|| {
                    points.push(s.positions[v as usize]);
                    points.len() - 1
                })
            })
        })
        .collect();
    let patch = Patch::new(points, tris, members, normals, areas);
    let x = patch.solve(turns)?;
    let judged = patch.judge(&x, cell);
    judged.well.then_some(judged.laid)
}

/// A chart to flatten: its vertices (a vertex of the solid once, or
/// once on each side of a seam), its triangles over them, each
/// triangle's area and the unit sum of their normals.
pub(crate) struct Patch {
    pub(crate) points: Vec<DVec3>,
    pub(crate) tris: Vec<[usize; 3]>,
    pub(crate) areas: Vec<f64>,
    pub(crate) mean: DVec3,
}

impl Patch {
    pub(crate) fn new(
        points: Vec<DVec3>,
        tris: Vec<[usize; 3]>,
        members: &[u32],
        normals: &[DVec3],
        areas: &[f64],
    ) -> Self {
        let mean = unit(
            members
                .iter()
                .map(|&i| normals[i as usize] * areas[i as usize])
                .sum::<DVec3>(),
        );
        Self {
            points,
            tris,
            areas: members.iter().map(|&i| areas[i as usize]).collect(),
            mean,
        }
    }

    /// Each vertex's place in the plane by the least squares conformal
    /// map (not yet to the surface's scale); `None` where there is
    /// nothing to solve. `turns` counts the work: triangles times
    /// steps of the solver.
    pub(crate) fn solve(&self, turns: &mut usize) -> Option<Vec<DVec2>> {
        let (points, tris) = (&self.points, &self.tris);
        let count = points.len();
        // The pins: the vertex furthest from the first, and the one
        // furthest from that.
        let furthest = |from: usize| {
            let mut best = (0.0, from);
            for (i, p) in points.iter().enumerate() {
                let d = (*p - points[from]).length_squared();
                if d > best.0 {
                    best = (d, i);
                }
            }
            best.1
        };
        let a = furthest(0);
        let b = furthest(a);
        let apart = (points[b] - points[a]).length();
        if a == b || apart == 0.0 {
            return None;
        }
        // Where to start: the projection on the mean plane, turned and
        // scaled to put the pins where they are held.
        let mean = self.mean;
        let (eu, ev) = if mean == DVec3::ZERO {
            (DVec3::X, DVec3::Y)
        } else {
            basis(mean)
        };
        let mut x: Vec<DVec2> = points
            .iter()
            .map(|p| DVec2::new(p.dot(eu), p.dot(ev)))
            .collect();
        let span = x[b] - x[a];
        let length = span.length_squared();
        let origin = x[a];
        for q in &mut x {
            let d = *q - origin;
            *q = if length > 0.0 {
                // d / span as complex numbers, times the true distance.
                DVec2::new(d.x * span.x + d.y * span.y, d.y * span.x - d.x * span.y)
                    * (apart / length)
            } else {
                DVec2::ZERO
            };
        }
        x[a] = DVec2::ZERO;
        x[b] = DVec2::new(apart, 0.0);
        // Unknowns: each free vertex's u, then its v.
        let mut slot = vec![usize::MAX; count];
        let mut free = 0;
        for (i, sl) in slot.iter_mut().enumerate() {
            if i != a && i != b {
                *sl = free;
                free += 1;
            }
        }
        if free == 0 {
            return None;
        }
        // Per triangle, its corners as complex weights (the opposite edge
        // in the triangle's own plane, over the root of twice its area).
        let mut weights: Vec<[DVec2; 3]> = Vec::with_capacity(tris.len());
        for t in tris {
            let [p0, p1, p2] = t.map(|v| points[v]);
            let e = p1 - p0;
            let len = e.length();
            let normal = e.cross(p2 - p0);
            let double = normal.length();
            if len == 0.0 || double == 0.0 {
                weights.push([DVec2::ZERO; 3]);
                continue;
            }
            let ex = e / len;
            let ey = normal.cross(e) / (double * len);
            let q = [
                DVec2::ZERO,
                DVec2::new(len, 0.0),
                DVec2::new((p2 - p0).dot(ex), (p2 - p0).dot(ey)),
            ];
            let root = double.sqrt();
            weights.push([
                (q[2] - q[1]) / root,
                (q[0] - q[2]) / root,
                (q[1] - q[0]) / root,
            ]);
        }
        // A x (two rows a triangle) and its transpose, over the free
        // unknowns; `held` is what the pins put on the right-hand side.
        let apply = |x: &[f64], out: &mut [f64]| {
            for (k, (t, w)) in tris.iter().zip(&weights).enumerate() {
                let (mut re, mut im) = (0.0, 0.0);
                for j in 0..3 {
                    let sl = slot[t[j]];
                    if sl == usize::MAX {
                        continue;
                    }
                    let (u, v) = (x[sl], x[free + sl]);
                    re += w[j].x * u - w[j].y * v;
                    im += w[j].y * u + w[j].x * v;
                }
                out[2 * k] = re;
                out[2 * k + 1] = im;
            }
        };
        let transpose = |r: &[f64], out: &mut [f64]| {
            out.fill(0.0);
            for (k, (t, w)) in tris.iter().zip(&weights).enumerate() {
                let (re, im) = (r[2 * k], r[2 * k + 1]);
                for j in 0..3 {
                    let sl = slot[t[j]];
                    if sl == usize::MAX {
                        continue;
                    }
                    out[sl] += w[j].x * re + w[j].y * im;
                    out[free + sl] += w[j].x * im - w[j].y * re;
                }
            }
        };
        let rows = 2 * tris.len();
        let mut held = vec![0.0; rows];
        for (k, (t, w)) in tris.iter().zip(&weights).enumerate() {
            for j in 0..3 {
                if t[j] == b {
                    // The pin at (apart, 0); the one at the origin adds nothing.
                    held[2 * k] -= w[j].x * apart;
                    held[2 * k + 1] -= w[j].y * apart;
                }
            }
        }
        let mut sol = vec![0.0; 2 * free];
        for (i, &sl) in slot.iter().enumerate() {
            if sl != usize::MAX {
                sol[sl] = x[i].x;
                sol[free + sl] = x[i].y;
            }
        }
        let dot = |p: &[f64], q: &[f64]| p.iter().zip(q).map(|(a, b)| a * b).sum::<f64>();
        let mut r = vec![0.0; rows];
        apply(&sol, &mut r);
        for (ri, h) in r.iter_mut().zip(&held) {
            *ri = h - *ri;
        }
        // Each unknown is weighed by the inverse of its column's
        // squared length (the solver's steps then depend less on how
        // the triangles' sizes differ).
        let weigh: Vec<f64> = {
            let mut col = vec![0.0; 2 * free];
            for (t, w) in tris.iter().zip(&weights) {
                for j in 0..3 {
                    let sl = slot[t[j]];
                    if sl != usize::MAX {
                        let l = w[j].length_squared();
                        col[sl] += l;
                        col[free + sl] += l;
                    }
                }
            }
            col.iter()
                .map(|&l| if l > 0.0 { 1.0 / l } else { 1.0 })
                .collect()
        };
        let weighed = |g: &mut [f64]| {
            for (gi, wi) in g.iter_mut().zip(&weigh) {
                *gi *= wi;
            }
        };
        let mut g = vec![0.0; 2 * free];
        let mut z = vec![0.0; 2 * free];
        transpose(&r, &mut g);
        z.copy_from_slice(&g);
        weighed(&mut z);
        let mut dir = z.clone();
        let mut gamma = dot(&g, &z);
        let first = gamma;
        let mut q = vec![0.0; rows];
        for _ in 0..(4 * free + 100).min(8000) {
            if gamma <= first * 1e-12 || gamma == 0.0 {
                break;
            }
            *turns += tris.len();
            apply(&dir, &mut q);
            let qq = dot(&q, &q);
            if qq == 0.0 {
                break;
            }
            let step = gamma / qq;
            for (v, d) in sol.iter_mut().zip(&dir) {
                *v += step * d;
            }
            for (ri, qi) in r.iter_mut().zip(&q) {
                *ri -= step * qi;
            }
            transpose(&r, &mut g);
            z.copy_from_slice(&g);
            weighed(&mut z);
            let next = dot(&g, &z);
            let turn = next / gamma;
            gamma = next;
            for (d, zi) in dir.iter_mut().zip(&z) {
                *d = zi + turn * *d;
            }
        }
        for (i, &sl) in slot.iter().enumerate() {
            if sl != usize::MAX {
                x[i] = DVec2::new(sol[sl], sol[free + sl]);
            }
        }
        if x.iter().any(|q| !q.is_finite()) {
            return None;
        }
        Some(x)
    }

    /// The triangles at `x`, scaled to the surface's own area; each
    /// one's flat area over its own (0 for one that folds, −1 for the
    /// first found lying over another); and whether the chart is well:
    /// none folds, none's area is past twice or under half its own, and
    /// none lies over another. No triangles where the whole is flat to
    /// nothing.
    pub(crate) fn judge(&self, x: &[DVec2], cell: f64) -> Judged {
        let mut laid: Vec<[DVec2; 3]> = self.tris.iter().map(|t| t.map(|v| x[v])).collect();
        let flat_area = |t: &[DVec2; 3]| (t[1] - t[0]).perp_dot(t[2] - t[0]);
        let (mut on_surface, mut on_plane) = (0.0, 0.0);
        for (&area, t) in self.areas.iter().zip(&laid) {
            on_surface += area;
            on_plane += flat_area(t);
        }
        if !(on_plane > 0.0 && on_surface > 0.0) {
            return Judged::default();
        }
        let scale = (on_surface / on_plane).sqrt();
        for t in &mut laid {
            *t = t.map(|q| q * scale);
        }
        let mut shrunk = vec![1.0; laid.len()];
        let mut well = true;
        let (mut least, mut most) = (f64::INFINITY, 0.0f64);
        for (k, (&area, t)) in self.areas.iter().zip(&laid).enumerate() {
            if orient2d(t[0], t[1], t[2]) != Ordering::Greater {
                shrunk[k] = 0.0;
                well = false;
                continue;
            }
            let ratio = flat_area(t) / area;
            if area > 0.0 {
                shrunk[k] = ratio;
                least = least.min(ratio);
                most = most.max(ratio);
            }
        }
        if well && most > 0.0 && !(least >= 0.5 && most <= 2.0) {
            // Out at the surface's own area, but within four of each
            // other: the chart a little larger or smaller holds all.
            if most <= least * 3.999 {
                let fit = 1.0 / (least * most).sqrt();
                let side = fit.sqrt();
                for t in &mut laid {
                    *t = t.map(|q| q * side);
                }
                for r in &mut shrunk {
                    *r *= fit;
                }
            } else {
                well = false;
            }
        }
        if well && let Some(k) = first_over(&laid, cell) {
            shrunk[k] = -1.0;
            well = false;
        }
        Judged { laid, shrunk, well }
    }
}

/// The work the conformal charts of a solid may take in all
/// ([`Patch::solve`]'s count), a triangle of it.
pub(crate) const SOLVE_WORK: usize = 20_000;
