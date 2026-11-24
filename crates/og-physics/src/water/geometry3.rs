//! Exact planar hulls for the 3D water solver, built only on shape change.
use super::{rp, unv};
use og_core::glam::Vec3;

pub(crate) fn triangles(shape: &rp::SharedShape) -> Option<Vec<[Vec3; 3]>> {
    let (points, indices) = if let Some(s) = shape.as_cuboid() {
        s.to_trimesh()
    } else {
        shape.as_convex_polyhedron()?.to_trimesh()
    };
    Some(
        indices
            .iter()
            .map(|t| t.map(|i| unv(points[i as usize])))
            .collect(),
    )
}
