//! A model's shape facts (format's `ShapeRow`): its long and up axes, its
//! length and its top surface along that length, in model space, so a game
//! stands a character on a turned or moving model without measuring it.

use super::*;
use crate::format::{SHAPE_STATIONS, ShapeRow};

/// The default scene's triangles at rest, in model space.
pub(super) fn triangles(
    src: &GltfSource,
    instanced: &std::collections::BTreeMap<usize, Vec<Mat4>>,
) -> Vec<[Vec3; 3]> {
    let mut out = Vec::new();
    visit_scene(src, instanced, |prim, at| {
        let reader = prim.reader(|b| Some(&src.buffers[b.index()].0[..]));
        let Some(pos) = reader.read_positions() else {
            return;
        };
        let pos: Vec<Vec3> = pos.map(Vec3::from).collect();
        let index: Vec<u32> = match reader.read_indices() {
            Some(i) => i.into_u32().collect(),
            None => (0..pos.len() as u32).collect(),
        };
        for m in at {
            for t in index.as_chunks::<3>().0 {
                let p = |i: u32| pos.get(i as usize).map(|v| m.transform_point3(*v));
                if let (Some(a), Some(b), Some(c)) = (p(t[0]), p(t[1]), p(t[2])) {
                    out.push([a, b, c]);
                }
            }
        }
    });
    out
}

/// The eigenvectors of a symmetric 3x3 matrix (columns), largest
/// eigenvalue first: cyclic Jacobi rotations.
fn principal_axes(m: [[f64; 3]; 3]) -> [glam::DVec3; 3] {
    let mut a = m;
    let mut v = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    for _ in 0..50 {
        let off = a[0][1].abs() + a[0][2].abs() + a[1][2].abs();
        if off < 1e-15 * (a[0][0].abs() + a[1][1].abs() + a[2][2].abs() + 1e-300) {
            break;
        }
        for (p, q) in [(0, 1), (0, 2), (1, 2)] {
            if a[p][q].abs() < 1e-300 {
                continue;
            }
            let theta = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
            let t = theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt());
            let t = if theta == 0.0 { 1.0 } else { t };
            let c = 1.0 / (t * t + 1.0).sqrt();
            let s = t * c;
            for row in &mut a {
                let (akp, akq) = (row[p], row[q]);
                row[p] = c * akp - s * akq;
                row[q] = s * akp + c * akq;
            }
            for k in 0..3 {
                let (apk, aqk) = (a[p][k], a[q][k]);
                a[p][k] = c * apk - s * aqk;
                a[q][k] = s * apk + c * aqk;
            }
            for row in &mut v {
                let (vp, vq) = (row[p], row[q]);
                row[p] = c * vp - s * vq;
                row[q] = s * vp + c * vq;
            }
        }
    }
    let mut order = [0usize, 1, 2];
    order.sort_by(|&i, &j| a[j][j].total_cmp(&a[i][i]));
    order.map(|i| glam::DVec3::new(v[0][i], v[1][i], v[2][i]))
}

/// The shape facts of `tris`, or none for a model without area.
pub(super) fn shape(tris: &[[Vec3; 3]]) -> Option<ShapeRow> {
    // Area-weighted centroid and covariance of the surface (each
    // triangle's second moment exactly: its vertices and their sums).
    let (mut area, mut centroid) = (0.0f64, glam::DVec3::ZERO);
    for t in tris {
        let [a, b, c] = t.map(|p| p.as_dvec3());
        let w = 0.5 * (b - a).cross(c - a).length();
        area += w;
        centroid += w * (a + b + c) / 3.0;
    }
    if area <= 0.0 {
        return None;
    }
    centroid /= area;
    let mut cov = [[0.0f64; 3]; 3];
    for t in tris {
        let [a, b, c] = t.map(|p| p.as_dvec3() - centroid);
        let w = 0.5 * (b - a).cross(c - a).length();
        let s = a + b + c;
        for i in 0..3 {
            for j in 0..3 {
                cov[i][j] += w / 12.0 * (s[i] * s[j] + a[i] * a[j] + b[i] * b[j] + c[i] * c[j]);
            }
        }
    }
    let axes = principal_axes(cov);
    // Forward: the longest axis, signed towards +Z (else +X).
    let mut forward = axes[0];
    if forward.z < -1e-9 || (forward.z.abs() <= 1e-9 && forward.x < 0.0) {
        forward = -forward;
    }
    // Up: of the other two, the one nearest +Y, signed upward.
    let mut up = if axes[1].y.abs() >= axes[2].y.abs() {
        axes[1]
    } else {
        axes[2]
    };
    if up.y < 0.0 {
        up = -up;
    }
    let (f, u) = (forward.as_vec3(), up.as_vec3());
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for t in tris {
        for p in t {
            let d = (*p - centroid.as_vec3()).dot(f);
            lo = lo.min(d);
            hi = hi.max(d);
        }
    }
    let length = hi - lo;
    let centre = centroid.as_vec3() + f * (lo + hi) * 0.5;
    // A cast along -up from above the model, down the centre line.
    let mut top = [0.0f32; SHAPE_STATIONS];
    let reach = tris
        .iter()
        .flatten()
        .map(|p| (*p - centre).length())
        .fold(0.0f32, f32::max)
        + 1.0;
    for (k, h) in top.iter_mut().enumerate() {
        let along = -length * 0.5 + length * (k as f32 + 0.5) / SHAPE_STATIONS as f32;
        let at = centre + f * along;
        let origin = at + u * reach;
        let mut best: Option<f32> = None;
        for t in tris {
            if let Some(d) = ray_triangle(origin, -u, t) {
                best = Some(best.map_or(d, |b| b.min(d)));
            }
        }
        *h = match best {
            Some(d) => reach - d,
            // Nothing under the line (a hole, a gap between legs): the
            // slab's highest point, so the profile has no gaps.
            None => {
                let slab = length / SHAPE_STATIONS as f32 * 0.5;
                tris.iter()
                    .flatten()
                    .filter(|p| ((**p - at).dot(f)).abs() <= slab)
                    .map(|p| (*p - centre).dot(u))
                    .fold(f32::MIN, f32::max)
                    .max(f32::MIN / 2.0)
            }
        };
        if *h < f32::MIN / 4.0 {
            *h = 0.0;
        }
    }
    Some(ShapeRow {
        forward_x: f.x,
        forward_y: f.y,
        forward_z: f.z,
        up_x: u.x,
        up_y: u.y,
        up_z: u.z,
        centre_x: centre.x,
        centre_y: centre.y,
        centre_z: centre.z,
        length,
        t0: top[0],
        t1: top[1],
        t2: top[2],
        t3: top[3],
        t4: top[4],
        t5: top[5],
        t6: top[6],
        t7: top[7],
        t8: top[8],
        t9: top[9],
        t10: top[10],
        t11: top[11],
        t12: top[12],
        t13: top[13],
        t14: top[14],
        t15: top[15],
    })
}

/// Möller–Trumbore: the distance along `dir` (unit) to triangle `t`, both sides.
fn ray_triangle(origin: Vec3, dir: Vec3, t: &[Vec3; 3]) -> Option<f32> {
    let (e1, e2) = (t[1] - t[0], t[2] - t[0]);
    let p = dir.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1e-12 {
        return None;
    }
    let inv = 1.0 / det;
    let s = origin - t[0];
    let u = s.dot(p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = s.cross(e1);
    let v = dir.dot(q) * inv;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let d = e2.dot(q) * inv;
    (d >= 0.0).then_some(d)
}
