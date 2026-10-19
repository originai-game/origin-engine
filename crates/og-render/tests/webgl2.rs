//! The WebGL2 tier (design §8, the reduced tier: no compute, no texture
//! queries) runs each pipeline's WGSL through naga's GLSL ES 3.0 writer,
//! the translation wgpu's GL backend performs when the pipeline is created.
//! A shader that writer refuses panics every WebGL2 renderer at startup
//! (42d7294's glass shader asked for textureNumLevels, which needs
//! TEXTURE_LEVELS), and no lane runs a browser, so the translation is
//! checked here for the overlay and glass passes every UI frame builds.

use naga::back::glsl;

/// Translates every vertex and fragment entry point of `source` to GLSL
/// ES 3.0 for WebGL, as wgpu-hal's GL device does; the failures, by entry
/// point.
fn webgl2_failures(name: &str, source: &str) -> Vec<String> {
    let module = naga::front::wgsl::parse_str(source).unwrap_or_else(|e| panic!("{name}: {e}"));
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    )
    .validate(&module)
    .unwrap_or_else(|e| panic!("{name}: {e:?}"));
    let options = glsl::Options {
        version: glsl::Version::Embedded {
            version: 300,
            is_webgl: true,
        },
        writer_flags: glsl::WriterFlags::ADJUST_COORDINATE_SPACE
            | glsl::WriterFlags::FORCE_POINT_SIZE,
        binding_map: Default::default(),
        zero_initialize_workgroup_memory: true,
    };
    let mut failures = Vec::new();
    let mut translated = 0;
    for entry in &module.entry_points {
        if !matches!(
            entry.stage,
            naga::ShaderStage::Vertex | naga::ShaderStage::Fragment
        ) {
            continue;
        }
        let pipeline = glsl::PipelineOptions {
            shader_stage: entry.stage,
            entry_point: entry.name.clone(),
            multiview: None,
        };
        let mut out = String::new();
        let result = glsl::Writer::new(
            &mut out,
            &module,
            &info,
            &options,
            &pipeline,
            naga::proc::BoundsCheckPolicies::default(),
        )
        .and_then(|mut w| w.write().map(|_| ()));
        match result {
            Ok(()) => translated += 1,
            Err(e) => failures.push(format!("{name}::{}: {e}", entry.name)),
        }
    }
    assert!(translated + failures.len() > 0, "{name}: no entry points");
    failures
}

#[test]
fn the_overlay_and_glass_shaders_translate_to_glsl_es_300() {
    let shaders = [
        ("overlay.wgsl", include_str!("../shaders/overlay.wgsl")),
        ("glass.wgsl", include_str!("../shaders/glass.wgsl")),
    ];
    let failures: Vec<String> = shaders
        .iter()
        .flat_map(|(name, source)| webgl2_failures(name, source))
        .collect();
    assert!(failures.is_empty(), "{failures:#?}");
}
