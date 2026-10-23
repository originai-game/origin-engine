//! Debug draw's shapes as line sets (og.debug: `line`, `box`, `sphere`,
//! `arrow`, `axes`; the physics, navigation and bounds views): each a
//! [`Lines`] of [`LineMode::Segments`] in world space, drawn over the
//! finished frame ([`LineLayer::Overlay`]) so it reads in every capture.

use crate::lines::{LineLayer, LineMode, Lines};
use glam::{Mat4, Quat, Vec3};
use std::f32::consts::TAU;

/// A set of segments over the frame, `width` CSS pixels, in one colour.
pub fn segments(points: Vec<Vec3>, color: [f32; 4], width: f32) -> Lines {
    Lines {
        points,
        color,
        width,
        mode: LineMode::Segments,
        depth_test: false,
        layer: LineLayer::Overlay,
        ..Lines::default()
    }
}

/// The twelve edges of a box of `half` extents, placed by `transform`.
pub fn box_edges(transform: Mat4, half: Vec3) -> Vec<Vec3> {
    let c = |x: f32, y: f32, z: f32| transform.transform_point3(Vec3::new(x, y, z) * half);
    let mut out = Vec::with_capacity(24);
    for s in [-1.0, 1.0] {
        for t in [-1.0, 1.0] {
            out.extend([c(-1.0, s, t), c(1.0, s, t)]);
            out.extend([c(s, -1.0, t), c(s, 1.0, t)]);
            out.extend([c(s, t, -1.0), c(s, t, 1.0)]);
        }
    }
    out
}

/// A circle of `radius` about `centre` in the plane across `normal`, as
/// `segments` segments.
pub fn circle(centre: Vec3, normal: Vec3, radius: f32, segments: u32) -> Vec<Vec3> {
    let n = normal.normalize_or(Vec3::Y);
    let u = n.any_orthonormal_vector();
    let v = n.cross(u);
    let seg = segments.max(3);
    let at = |k: u32| {
        let a = k as f32 / seg as f32 * TAU;
        centre + (u * a.cos() + v * a.sin()) * radius
    };
    (0..seg).flat_map(|k| [at(k), at(k + 1)]).collect()
}

/// A sphere as its three great circles (across x, y and z), turned by `rotation`.
pub fn sphere(centre: Vec3, radius: f32, rotation: Quat) -> Vec<Vec3> {
    [Vec3::X, Vec3::Y, Vec3::Z]
        .into_iter()
        .flat_map(|a| circle(centre, rotation * a, radius, 32))
        .collect()
}

/// A capsule along `axis` (unit): a cylinder of `half_height` each side of
/// `centre` and its two hemispheres, as rings and four sides.
pub fn capsule(centre: Vec3, axis: Vec3, half_height: f32, radius: f32) -> Vec<Vec3> {
    let a = axis.normalize_or(Vec3::Y);
    let (top, bottom) = (centre + a * half_height, centre - a * half_height);
    let mut out = circle(top, a, radius, 24);
    out.extend(circle(bottom, a, radius, 24));
    let u = a.any_orthonormal_vector();
    let v = a.cross(u);
    for d in [u, -u, v, -v] {
        out.extend([top + d * radius, bottom + d * radius]);
    }
    // The caps: half circles through the poles.
    for d in [u, v] {
        for (end, sign) in [(top, 1.0f32), (bottom, -1.0)] {
            for k in 0..12 {
                let p = |k: u32| {
                    let t = k as f32 / 12.0 * std::f32::consts::PI;
                    end + (d * t.cos() + a * sign * t.sin()) * radius
                };
                out.extend([p(k), p(k + 1)]);
            }
        }
    }
    out
}

/// An arrow from `from` to `to`: the shaft and a four-sided head a fifth
/// of its length (at most `head` world units).
pub fn arrow(from: Vec3, to: Vec3, head: f32) -> Vec<Vec3> {
    let d = to - from;
    let len = d.length();
    let mut out = vec![from, to];
    if len <= 1e-6 {
        return out;
    }
    let dir = d / len;
    let h = (len * 0.2).min(head.max(0.0));
    let u = dir.any_orthonormal_vector();
    let v = dir.cross(u);
    let base = to - dir * h;
    for side in [u, -u, v, -v] {
        out.extend([to, base + side * h * 0.4]);
    }
    out
}

/// The three axes of a frame `transform`, `size` long: +x red, +y green,
/// +z blue (three.js `AxesHelper`), as three sets of segments.
pub fn axes(transform: Mat4, size: f32, width: f32) -> [Lines; 3] {
    let o = transform.transform_point3(Vec3::ZERO);
    let end = |a: Vec3| transform.transform_point3(a * size);
    [
        segments(vec![o, end(Vec3::X)], [1.0, 0.2, 0.2, 1.0], width),
        segments(vec![o, end(Vec3::Y)], [0.2, 1.0, 0.2, 1.0], width),
        segments(vec![o, end(Vec3::Z)], [0.25, 0.45, 1.0, 1.0], width),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_box_has_twelve_edges_on_its_corners() {
        let e = box_edges(
            Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0)),
            Vec3::new(1.0, 2.0, 3.0),
        );
        assert_eq!(e.len(), 24);
        for p in &e {
            let q = *p - Vec3::new(1.0, 2.0, 3.0);
            assert!(
                (q.abs() - Vec3::new(1.0, 2.0, 3.0)).abs().max_element() < 1e-6,
                "{p}"
            );
        }
    }

    #[test]
    fn circles_lie_on_their_radius_and_arrows_end_where_asked() {
        for p in sphere(Vec3::ONE, 2.0, Quat::IDENTITY) {
            assert!((p.distance(Vec3::ONE) - 2.0).abs() < 1e-5);
        }
        let a = arrow(Vec3::ZERO, Vec3::new(0.0, 5.0, 0.0), 0.5);
        assert_eq!(a.len(), 2 + 8);
        assert_eq!(a[1], Vec3::new(0.0, 5.0, 0.0));
    }
}
