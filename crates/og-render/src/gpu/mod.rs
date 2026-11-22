//! The device and what lives on it: capabilities, targets, textures, the pipeline cache, objects made on first use.

pub(crate) mod buffers;
pub mod caps;
pub(crate) mod copy;
pub(crate) mod data_texture;
pub(crate) mod deferred;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod headless;
pub(crate) mod indirect_own;
pub(crate) mod layout;
pub(crate) mod mips;
pub(crate) mod pipeline_cache;
#[cfg(any(target_os = "macos", target_os = "linux", windows))]
pub mod shared_frames;
pub(crate) mod targets;
pub(crate) mod texture;
pub(crate) mod texture_array;
pub mod traffic;
pub(crate) mod video;

/// `wgsl` as a WebGPU compatibility device compiles it: the per-pixel
/// derivatives (`dpdxFine`, `dpdyFine`, `fwidthFine`, which every shader
/// takes so D3D12 filters as Metal does) as the plain ones, which that
/// mode allows alone (gpuweb's compatibility-mode proposal).
pub fn compat_wgsl(wgsl: &str) -> String {
    wgsl.replace("dpdxFine", "dpdx")
        .replace("dpdyFine", "dpdy")
        .replace("fwidthFine", "fwidth")
}

/// `device.create_shader_module(desc)`, its WGSL rewritten by
/// [`compat_wgsl`] on a WebGPU device without `core-features-and-limits`,
/// a compatibility-mode device, which refuses the module otherwise.
pub(crate) fn shader_module(
    device: &wgpu::Device,
    mut desc: wgpu::ShaderModuleDescriptor,
) -> wgpu::ShaderModule {
    if device.adapter_info().backend == wgpu::Backend::BrowserWebGpu
        && !device
            .features()
            .contains(wgpu::Features::CORE_FEATURES_AND_LIMITS)
        && let wgpu::ShaderSource::Wgsl(wgsl) = &desc.source
    {
        desc.source = wgpu::ShaderSource::Wgsl(compat_wgsl(wgsl).into());
    }
    device.create_shader_module(desc)
}
