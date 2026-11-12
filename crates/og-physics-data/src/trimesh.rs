//! A triangle mesh collider's shape and its bake.

use og_core::glam::Vec3;
use og_core::{Error, ErrorKind, Result};
use rapier3d::prelude as rp;

/// The triangle mesh (rapier's, with its BVH) over `mesh` scaled by `s`:
/// a SHAPE_TRIMESH collider's shape, and what [`bake`] bakes.
pub fn trimesh(mesh: &crate::MeshDesc, s: Vec3) -> Result<rp::SharedShape> {
    let points: Vec<rp::Vector> = mesh
        .positions
        .iter()
        .map(|p| {
            let p = *p * s;
            rp::Vector::new(p.x, p.y, p.z)
        })
        .collect();
    rp::SharedShape::trimesh(points, mesh.indices.clone()).map_err(|e| {
        Error::new(
            ErrorKind::Invalid,
            format!("the mesh is not a valid triangle mesh: {e}"),
        )
    })
}

/// What starts a baked triangle mesh's bytes: the format and its version,
/// moved with any rapier or parry update that changes `TriMesh`'s
/// serialisation (`a_baked_trimesh_is_the_build_bit_for_bit` pins a bake).
pub const BAKED: &[u8; 8] = b"OGTM\x01\x00\x00\x00";

/// [`crate::bake_trimesh`].
pub fn bake(mesh: &crate::MeshDesc) -> Result<Vec<u8>> {
    let shape = trimesh(mesh, Vec3::ONE)?;
    let mesh = shape.as_trimesh().expect("a triangle mesh");
    let mut out = BAKED.to_vec();
    crate::encode(&mut out, mesh)
        .map_err(|e| Error::new(ErrorKind::Invalid, format!("baking a triangle mesh: {e}")))?;
    Ok(out)
}

/// A [`bake`]'s shape.
pub fn unbake(bytes: &[u8]) -> Result<rp::SharedShape> {
    let body = bytes.strip_prefix(BAKED.as_slice()).ok_or_else(|| {
        Error::new(
            ErrorKind::Invalid,
            "not a baked triangle mesh of this engine's format (cook the model again)",
        )
    })?;
    let mesh: rp::TriMesh = crate::decode(body)
        .map_err(|e| Error::new(ErrorKind::Invalid, format!("a baked triangle mesh: {e}")))?;
    Ok(rp::SharedShape::new(mesh))
}
