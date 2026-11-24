//! Exact geometric predicates (Shewchuk's adaptive predicates, from the
//! `robust` crate).
//!
//! Every geometric side the kernel decides goes through these. The result
//! is the exact sign for the f64 inputs given, so it never depends on
//! rounding.

use core::cmp::Ordering;

use glam::{DVec2, DVec3};

fn sign(value: f64) -> Ordering {
    if value > 0.0 {
        Ordering::Greater
    } else if value < 0.0 {
        Ordering::Less
    } else {
        Ordering::Equal
    }
}

fn c2(p: DVec2) -> robust::Coord<f64> {
    robust::Coord { x: p.x, y: p.y }
}

fn c3(p: DVec3) -> robust::Coord3D<f64> {
    robust::Coord3D {
        x: p.x,
        y: p.y,
        z: p.z,
    }
}

/// The side of `c` relative to the directed line `a → b`: `Greater` if
/// `a, b, c` turn counter-clockwise, `Less` if clockwise, `Equal` if
/// collinear.
pub fn orient2d(a: DVec2, b: DVec2, c: DVec2) -> Ordering {
    sign(robust::orient2d(c2(a), c2(b), c2(c)))
}

/// The side of `d` relative to the plane through `a, b, c`: `Greater` if
/// `d` lies below it (where `a, b, c` appear counter-clockwise from
/// above), `Less` if above, `Equal` if coplanar. This is Shewchuk's sign
/// convention.
pub fn orient3d(a: DVec3, b: DVec3, c: DVec3, d: DVec3) -> Ordering {
    sign(robust::orient3d(c3(a), c3(b), c3(c), c3(d)))
}

/// `Greater` if `d` lies inside the circle through `a, b, c` (given
/// counter-clockwise), `Less` if outside, `Equal` if on it.
pub fn incircle(a: DVec2, b: DVec2, c: DVec2, d: DVec2) -> Ordering {
    sign(robust::incircle(c2(a), c2(b), c2(c), c2(d)))
}

/// `Greater` if `e` lies inside the sphere through `a, b, c, d` (with
/// `orient3d(a, b, c, d)` positive), `Less` if outside, `Equal` if on it.
pub fn insphere(a: DVec3, b: DVec3, c: DVec3, d: DVec3, e: DVec3) -> Ordering {
    sign(robust::insphere(c3(a), c3(b), c3(c), c3(d), c3(e)))
}
