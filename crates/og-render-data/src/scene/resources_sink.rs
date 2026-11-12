//! What a cooked model is added to.

use super::*;
use crate::material::StandardMaterial;
use crate::meshlet::{MeshClusters, MeshFoliage};
use glam::Mat4;

/// A scene that takes a cooked model's textures, materials, meshes and
/// instances: og-render's `RenderScene` (its methods of the same names, as
/// documented there), and whatever else holds what a model adds
/// (og-bundle's `CookedModel::add_to` and its kin add to any of them, so
/// the bundle loads models without linking the renderer).
pub trait SceneResources {
    /// Adds a texture from its source.
    fn add_texture_source(&mut self, source: TextureSource) -> TextureHandle;
    /// How `texture` is sampled; false for a handle that is not live.
    fn set_texture_filter(&mut self, texture: TextureHandle, filter: TextureFilter) -> bool;
    /// Adds a material in its glTF shape.
    fn add_material(&mut self, material: Material) -> MaterialHandle;
    /// Adds a material as the standard material it draws as.
    fn add_standard_material(&mut self, material: StandardMaterial) -> MaterialHandle;
    /// The standard material `handle` draws as.
    fn standard_material(&self, handle: MaterialHandle) -> Option<&StandardMaterial>;
    /// Draws `copy` in the batches of `of` while the two are alike; false
    /// when refused.
    fn alias_material_when_alike(&mut self, copy: MaterialHandle, of: MaterialHandle) -> bool;
    /// Adds a mesh.
    fn add_mesh(&mut self, mesh: MeshData) -> MeshHandle;
    /// The mesh `handle` holds.
    fn mesh(&self, handle: MeshHandle) -> Option<&MeshData>;
    /// A mesh's foliage deformation.
    fn mesh_foliage(&self, handle: MeshHandle) -> Option<&MeshFoliage>;
    /// Gives a mesh its clusters (`None`: takes them away); false when
    /// refused.
    fn set_mesh_clusters(&mut self, handle: MeshHandle, clusters: Option<MeshClusters>) -> bool;
    /// Gives a mesh its foliage deformation (`None`: takes it away); false
    /// when refused.
    fn set_mesh_foliage(&mut self, handle: MeshHandle, foliage: Option<MeshFoliage>) -> bool;
    /// Submits one instance of `mesh` drawn with `material` at `transform`.
    fn add_instance(&mut self, mesh: MeshHandle, material: MaterialHandle, transform: Mat4);
}
