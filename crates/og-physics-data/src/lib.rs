//! Origin Engine physics data (`docs/design.md` §2.1, "Formats below
//! features"): the meshes colliders are built from ([`MeshDesc`]), a
//! baked triangle mesh's format and its bake ([`bake_trimesh`], feature
//! `physics3d`), and the codec physics state is saved with, so the cook
//! that bakes triangle meshes (og-cook) does not link the simulation.
//! og-physics re-exports every item at the path it had there.

#[cfg(feature = "physics3d")]
use og_core::Result;
use og_core::glam::Vec3;

mod codec;
pub use codec::*;
#[cfg(feature = "physics3d")]
pub mod trimesh;

/// Geometry for convex and triangle-mesh colliders, registered under the
/// mesh handle the colliders name ([`Collider::mesh`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MeshDesc {
    /// Vertex positions in the mesh's own space (2D polygons use x and y).
    pub positions: Vec<Vec3>,
    /// Triangles as indices into `positions` (a convex hull ignores them).
    pub indices: Vec<[u32; 3]>,
}

/// A SHAPE_TRIMESH collider's shape over `mesh` at scale 1, as a collider
/// naming the mesh builds it (rapier's triangle mesh with its BVH), in
/// bytes for the cook to put beside the mesh: [`PhysicsSystem::set_baked_trimesh`]
/// takes it back without building anything. 3D only.
#[cfg(feature = "physics3d")]
pub fn bake_trimesh(mesh: &MeshDesc) -> Result<Vec<u8>> {
    trimesh::bake(mesh)
}
