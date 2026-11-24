//! Exact intersection tests between triangles, in 3D (self-intersections
//! of a surface) and in 2D (overlapping UV triangles). Every side is
//! decided by Shewchuk's predicates; no tolerance is used.
//!
//! The 3D tests are `crate::boolean::selfcheck`'s, shared with the booleans'
//! operand check and self-union: one implementation.

use core::cmp::Ordering;

use crate::math::DVec2;
use crate::math::predicates::orient2d;

pub use crate::boolean::selfcheck::degenerate;

/// Pairs of triangles `(i, j)`, `i < j`, that share no vertex index and
/// still meet: a surface passing through (or touching) itself. Degenerate
/// triangles are left out (they are counted on their own).
pub fn self_intersections(
    positions: &[crate::math::DVec3],
    triangles: &[[u32; 3]],
) -> Vec<(u32, u32)> {
    crate::boolean::selfcheck::touching_pairs(positions, triangles)
}

/// Two 2D triangles overlap in a region of positive area (touching along
/// an edge or at a point does not count). Zero-area triangles overlap
/// nothing.
pub fn tris_overlap_2d(a: [DVec2; 3], b: [DVec2; 3]) -> bool {
    let ccw = |t: [DVec2; 3]| match orient2d(t[0], t[1], t[2]) {
        Ordering::Greater => Some(t),
        Ordering::Less => Some([t[0], t[2], t[1]]),
        Ordering::Equal => None,
    };
    let (Some(a), Some(b)) = (ccw(a), ccw(b)) else {
        return false;
    };
    // Separating axis: an edge of either with the other wholly on its
    // outer side (or on it).
    let separates = |t: [DVec2; 3], o: [DVec2; 3]| {
        (0..3).any(|k| {
            o.iter()
                .all(|&p| orient2d(t[k], t[(k + 1) % 3], p) != Ordering::Greater)
        })
    };
    !(separates(a, b) || separates(b, a))
}
