//! What a frame draws of the scene: instances, culling, levels of detail, meshlets.

pub(crate) mod cull_gpu;
pub(crate) mod instances;
pub mod lod;
pub(crate) mod meshlet_gpu;
pub(crate) mod upload;
