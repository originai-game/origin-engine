//! The 2D buoyancy solver uses area samples rather than 3D hull triangles.
use super::rp;
use og_core::glam::Vec3;

pub(crate) fn triangles(_: &rp::SharedShape) -> Option<Vec<[Vec3; 3]>> {
    None
}
