//! Cloth, soft volumes and garments drawn or simulated for looks only.

pub(crate) mod garment;
pub(crate) mod garment_gpu;
pub(crate) mod surface;
pub(crate) mod visual_sim;

/// The system's modules for [`crate::shader_sources`]: a label and its WGSL.
pub(crate) fn shader_sources(add: &mut dyn FnMut(&str, String)) {
    add("simulation surface", crate::surface::WGSL.to_string());
    add("garment draw", crate::garment_gpu::WGSL.to_string());
    add("visual body", crate::visual_sim::surface_source());
    add("visual garment", crate::visual_sim::garment_source());
}
