//! The tier an adapter is reported as (the probe's hello, the
//! reference-frame directories), from stubbed adapter infos: a software
//! rasterizer is `vulkan-cpu`, never `vulkan`, so a frame time measured in
//! an executor's sandbox is never taken for a GPU's.

use og_render::tier_name;
use og_render::wgpu::{Backend, DeviceType};

#[test]
fn a_cpu_vulkan_adapter_is_vulkan_cpu_and_a_gpu_one_vulkan() {
    let llvmpipe = "llvmpipe (LLVM 15.0.7, 256 bits)";
    assert_eq!(
        tier_name(Backend::Vulkan, DeviceType::Cpu, llvmpipe),
        "vulkan-cpu"
    );
    assert_eq!(
        tier_name(
            Backend::Vulkan,
            DeviceType::DiscreteGpu,
            "NVIDIA GeForce RTX 4070"
        ),
        "vulkan"
    );
    assert_eq!(
        tier_name(
            Backend::Vulkan,
            DeviceType::IntegratedGpu,
            "AMD Radeon Graphics (RADV PHOENIX)"
        ),
        "vulkan"
    );
}
