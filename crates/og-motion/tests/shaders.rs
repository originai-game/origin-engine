//! Every og-motion kernel through naga: parsed, validated and written as
//! Metal's MSL, Vulkan's SPIR-V and DX12's HLSL (shader model 6), as wgpu
//! does when a pipeline is made. Runs on any machine, GPU or not.

use naga::back::{hlsl, msl, spv};
use naga::valid::{Capabilities, ValidationFlags, Validator};

#[test]
fn every_kernel_validates_and_translates() {
    let mut failures = Vec::new();
    let mut count = 0;
    for (name, wgsl) in og_motion::gpu::shader_sources() {
        count += 1;
        let module = match naga::front::wgsl::parse_str(wgsl) {
            Ok(m) => m,
            Err(e) => {
                failures.push(format!("{name}: {}", e.emit_to_string(wgsl)));
                continue;
            }
        };
        let info =
            match Validator::new(ValidationFlags::all(), Capabilities::empty()).validate(&module) {
                Ok(i) => i,
                Err(e) => {
                    failures.push(format!("{name}: {}", e.emit_to_string(wgsl)));
                    continue;
                }
            };
        let options = msl::Options {
            lang_version: (2, 4),
            ..Default::default()
        };
        if let Err(e) =
            msl::write_string(&module, &info, &options, &msl::PipelineOptions::default())
        {
            failures.push(format!("{name}: MSL: {e}"));
        }
        if let Err(e) = spv::write_vec(&module, &info, &spv::Options::default(), None) {
            failures.push(format!("{name}: SPIR-V: {e}"));
        }
        let options = hlsl::Options {
            shader_model: hlsl::ShaderModel::V6_0,
            ..Default::default()
        };
        let mut out = String::new();
        if let Err(e) = hlsl::Writer::new(&mut out, &options, &hlsl::PipelineOptions::default())
            .write(&module, &info, None)
        {
            failures.push(format!("{name}: HLSL: {e}"));
        }
    }
    assert_eq!(count, 9);
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
