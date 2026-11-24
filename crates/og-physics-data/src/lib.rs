//! Origin Engine physics data (`docs/design.md` §2.1, "Formats below
//! features"): the meshes colliders are built from ([`MeshDesc`]), a
//! baked collider's format and its bake ([`bake_collider`]), the
//! prefractured asset destruction breaks ([`fracture`])
//! and the codec physics state is saved with, so the cook that bakes
//! triangle meshes and cuts fractures (og-cook) does not link the
//! simulation.
//! og-physics re-exports every item at the path it had there.

use og_core::glam::Vec3;

mod codec;
pub use codec::*;
pub mod fracture;
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

/// Native line segments, without triangulation or mesh outline extraction.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PolylineDesc {
    pub positions: Vec<Vec3>,
    pub segments: Vec<[u32; 2]>,
}

impl PolylineDesc {
    pub fn content_hash(&self) -> u64 {
        og_core::hash::hash_words(
            [self.positions.len() as u64, self.segments.len() as u64]
                .into_iter()
                .chain(
                    self.positions
                        .iter()
                        .flat_map(|p| p.to_array().map(|x| u64::from(x.to_bits()))),
                )
                .chain(self.segments.iter().flat_map(|s| s.map(u64::from))),
        )
    }
}

impl MeshDesc {
    /// Content key used by cooked collider assets, independent of handles and scale.
    pub fn content_hash(&self) -> u64 {
        og_core::hash::hash_words(
            [self.positions.len() as u64, self.indices.len() as u64]
                .into_iter()
                .chain(
                    self.positions
                        .iter()
                        .flat_map(|p| p.to_array().map(|x| u64::from(x.to_bits()))),
                )
                .chain(self.indices.iter().flat_map(|t| t.map(u64::from))),
        )
    }

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

pub use trimesh::{BakeKind, bake_collider};
