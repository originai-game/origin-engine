//! Rays, rings and screen distances the gizmo picks with.

use super::*;

/// The parameter along the line `o + axis·t` nearest the ray; none when
/// the ray runs along it.
pub fn line_param((r0, rd): (Vec3, Vec3), o: Vec3, axis: Vec3) -> Option<f32> {
    let b = axis.dot(rd);
    let denom = 1.0 - b * b;
    if denom < 1e-5 {
        return None;
    }
    let w = o - r0;
    let d = axis.dot(w);
    let e = rd.dot(w);
    Some((b * e - d) / denom)
}

/// Where the ray meets the plane through `o` across `n`; none when it runs
/// along the plane or meets it behind the camera.
pub fn plane_hit((r0, rd): (Vec3, Vec3), o: Vec3, n: Vec3) -> Option<Vec3> {
    let den = rd.dot(n);
    if den.abs() < 1e-4 {
        return None;
    }
    let t = (o - r0).dot(n) / den;
    (t >= 0.0).then(|| r0 + rd * t)
}

/// A ring of `n` points about `o` across `axis`.
pub(super) fn ring(o: Vec3, axis: Vec3, radius: f32, n: usize) -> Vec<Vec3> {
    let a = axis.normalize_or(Vec3::Y);
    let u = a.any_orthonormal_vector();
    let v = a.cross(u);
    (0..n)
        .map(|k| {
            let t = k as f32 / n as f32 * std::f32::consts::TAU;
            let (sin, cos) = og_core::math::sin_cos(t);
            o + (u * cos + v * sin) * radius
        })
        .collect()
}

/// The nearest a closed polyline comes to `at` on screen.
pub(super) fn polyline_distance(view: &View, at: Vec2, pts: &[Vec3]) -> Option<f32> {
    let s: Vec<Vec2> = pts.iter().filter_map(|p| view.project(*p)).collect();
    if s.len() < 2 {
        return None;
    }
    (0..s.len())
        .map(|k| segment_distance(at, s[k], s[(k + 1) % s.len()]))
        .reduce(f32::min)
}

/// The distance from `p` to the segment `a`–`b`.
pub fn segment_distance(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len = ab.length_squared();
    let t = if len > 1e-9 {
        ((p - a).dot(ab) / len).clamp(0.0, 1.0)
    } else {
        0.0
    };
    p.distance(a + ab * t)
}

/// Whether `p` is inside the convex quad `q` (either winding).
pub(super) fn inside(p: Vec2, q: [Vec2; 4]) -> bool {
    let mut sign = 0.0;
    for k in 0..4 {
        let (a, b) = (q[k], q[(k + 1) % 4]);
        let c = (b - a).perp_dot(p - a);
        if c.abs() < 1e-6 {
            continue;
        }
        if sign == 0.0 {
            sign = c.signum();
        } else if c.signum() != sign {
            return false;
        }
    }
    true
}

/// A quad's area on screen (a square seen edge on has none to grab).
pub(super) fn area(q: [Vec2; 4]) -> f32 {
    let mut a = 0.0;
    for k in 0..4 {
        a += q[k].perp_dot(q[(k + 1) % 4]);
    }
    (a * 0.5).abs()
}
