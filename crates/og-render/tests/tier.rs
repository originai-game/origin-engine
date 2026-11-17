//! The tier an adapter is reported as (the probe's hello, the
//! reference-frame directories), from stubbed adapter infos: a software
//! rasterizer is `vulkan-cpu`, never `vulkan`, so a frame time measured in
//! an executor's sandbox is never taken for a GPU's.

use og_render::wgpu::{Backend, DeviceType};
use og_render::{is_software_adapter, tier_name};

#[test]
#[ignore = "requires GPU"]
fn hardware_only_rejects_software_before_rendering_but_allows_opt_in() {
    if std::env::var_os("OG_TEST_ADAPTER_CHILD").is_none() {
        for mode in ["hardware", "software"] {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "tier::hardware_only_rejects_software_before_rendering_but_allows_opt_in",
                ])
                .env("OG_TEST_ADAPTER_CHILD", "1")
                .env("OG_RENDERER", mode)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        return;
    }
    let desc = og_render::RenderDesc {
        width: 32,
        height: 32,
        ..Default::default()
    };
    let instance = desc.backend.instance();
    let adapter =
        pollster::block_on(instance.request_adapter(&Default::default())).expect("adapter");
    let info = adapter.get_info();
    let renderer = pollster::block_on(og_render::Renderer::new(&desc));
    if std::env::var("OG_RENDERER").as_deref() == Ok("hardware")
        && is_software_adapter(info.device_type, &info.name)
    {
        let error = renderer
            .err()
            .expect("software must not initialize under hardware-only policy")
            .to_string();
        assert!(error.starts_with("no GPU adapter:"), "{error}");
        assert!(error.contains(&info.name), "{error}");
        assert!(error.contains("OG_RENDERER=software"), "{error}");
    } else {
        assert!(renderer.is_ok(), "{:?}", renderer.err());
    }
}

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
