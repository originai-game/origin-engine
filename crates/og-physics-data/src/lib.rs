//! Origin Engine physics data (`docs/design.md` §2.1, "Formats below
//! features"): the meshes colliders are built from ([`MeshDesc`]), a
//! baked triangle mesh's format and its bake ([`bake_trimesh`], feature
//! `physics3d`), the prefractured asset destruction breaks ([`fracture`])
//! and the codec physics state is saved with, so the cook that bakes
//! triangle meshes and cuts fractures (og-cook) does not link the
//! simulation.
//! og-physics re-exports every item at the path it had there.

#[cfg(feature = "physics3d")]
use og_core::Result;
use og_core::glam::Vec3;

mod codec;
pub use codec::*;
pub mod fracture;
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

impl MeshDesc {
    /// Refuses geometry no collider can be built from: no positions, a
    /// triangle indexing past them, or a non-finite position. What
    /// `PhysicsSystem::set_mesh` checks, and what a host checks when it
    /// leaves the mesh to physics' mesh source until a collider names it.
    pub fn check(&self, handle: u64) -> og_core::Result<()> {
        use og_core::{Error, ErrorKind};
        if self.positions.is_empty() {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("mesh {handle} has no positions"),
            ));
        }
        let n = self.positions.len() as u32;
        if let Some(t) = self.indices.iter().find(|t| t.iter().any(|&i| i >= n)) {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("mesh {handle}: triangle {t:?} indexes past its {n} positions"),
            ));
        }
        if self.positions.iter().any(|p| !p.is_finite()) {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("mesh {handle} has a non-finite position"),
            ));
        }
        Ok(())
    }
}

/// A SHAPE_TRIMESH collider's shape over `mesh` at scale 1, as a collider
/// naming the mesh builds it (rapier's triangle mesh with its BVH), in
/// bytes for the cook to put beside the mesh: [`PhysicsSystem::set_baked_trimesh`]
/// takes it back without building anything. 3D only.
#[cfg(feature = "physics3d")]
pub fn bake_trimesh(mesh: &MeshDesc) -> Result<Vec<u8>> {
    trimesh::bake(mesh)
}
