//! Renderer implementation and GPU resources.

use super::*;

mod assets;
mod batches;
mod buffers;
mod config;
mod init;
mod pipelines;
mod presentation;
mod readback;
mod render;
mod resources;
mod settings;
mod shader_source;
mod uploads;
pub(crate) use buffers::*;
pub use config::*;
pub(crate) use pipelines::*;
pub use presentation::{GpuMemory, MAX_RENDER_SCALE, MIN_RENDER_SCALE, Successor};
pub(crate) use presentation::{LostFlag, texture_bytes, view_bytes};
pub(crate) use resources::*;
pub(crate) use shader_source::*;

/// The sampler every tiled surface texture reads through: lit materials,
/// terrain layers and water ripples. Repeated, trilinear and 16x anisotropic,
/// so a grazing-angle footprint filters along both axes (wgpu clamps the
/// anisotropy to 1 on devices without ANISOTROPIC_FILTERING). One descriptor,
/// so the surfaces cannot drift apart (terrain stayed at 8 when materials
/// moved to 16, and a cliff seen edge-on smeared into streaks).
pub(crate) fn surface_sampler(label: &'static str) -> wgpu::SamplerDescriptor<'static> {
    wgpu::SamplerDescriptor {
        label: Some(label),
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        anisotropy_clamp: 16,
        ..Default::default()
    }
}

#[cfg(test)]
mod surface_sampler_tests {
    /// Lit materials, terrain layers and water ripples read one descriptor,
    /// filtered 16x anisotropic (terrain was left at 8 when materials moved
    /// to 16, and an edge-on cliff smeared).
    #[test]
    fn every_tiled_surface_filters_at_16x() {
        let d = super::surface_sampler("any");
        assert_eq!(d.anisotropy_clamp, 16);
        assert_eq!(d.mipmap_filter, wgpu::MipmapFilterMode::Linear);
        assert_eq!(d.address_mode_u, wgpu::AddressMode::Repeat);
    }
}
