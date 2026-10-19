//! The tier an adapter is reported as (the probe's hello, og polish, the
//! reference-frame directories), from stubbed adapter infos: a software
//! rasterizer is `vulkan-cpu`, never `vulkan`, so a frame time measured in
//! an executor's sandbox is never taken for a GPU's.

use og_render::wgpu::{Backend, DeviceType};
use og_render::{is_software_adapter, tier_name};

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

/// Drivers that report a software rasterizer as another device type are
/// known by name.
#[test]
fn software_rasterizers_are_known_by_name_too() {
    for name in [
        "llvmpipe (LLVM 19.1.7, 256 bits)",
        "Lavapipe",
        "SwiftShader Device (Subzero)",
    ] {
        assert!(is_software_adapter(DeviceType::Other, name), "{name}");
        assert_eq!(
            tier_name(Backend::Vulkan, DeviceType::Other, name),
            "vulkan-cpu",
            "{name}"
        );
        assert_eq!(
            tier_name(Backend::Vulkan, DeviceType::VirtualGpu, name),
            "vulkan-cpu",
            "{name}"
        );
    }
    assert!(!is_software_adapter(
        DeviceType::DiscreteGpu,
        "Apple M4 Pro"
    ));
    assert!(!is_software_adapter(
        DeviceType::VirtualGpu,
        "Virtio-GPU Venus (Apple M4 Pro)"
    ));
}

#[test]
fn other_backends_keep_their_names() {
    assert_eq!(
        tier_name(Backend::Metal, DeviceType::IntegratedGpu, "Apple M4 Pro"),
        "metal"
    );
    assert_eq!(
        tier_name(Backend::BrowserWebGpu, DeviceType::Other, ""),
        "webgpu"
    );
    // Native GL on llvmpipe is the GL tier (no CPU variant is filed).
    assert_eq!(tier_name(Backend::Gl, DeviceType::Cpu, "llvmpipe"), "gl");
}
