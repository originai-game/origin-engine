//! Every WGSL module the renderer compiles (`og_render::shader_sources`), in
//! every variant of each tier, through naga at build time: parsed and
//! validated, its overrides resolved, then written as Metal's MSL and
//! Vulkan's SPIR-V (the full tier) or GLSL ES 3.0 for WebGL2 (the GL tier),
//! as wgpu does when a pipeline is made. A shader error fails the gate
//! instead of a player's renderer at startup (owed in og-render's README:
//! shader cooking per backend). Broken once by a function returning an
//! unknown identifier at the end of `scatter.wgsl`: both tiers' `scatter`
//! modules fail with naga's message and the line.

use naga::back::{glsl, msl, spv};
use naga::valid::{Capabilities, ValidationFlags, Validator};

/// What wgpu asks naga to allow on the full tier for these modules: the
/// capabilities the renderer's features and downlevel flags give it.
fn full_capabilities() -> Capabilities {
    Capabilities::MULTISAMPLED_SHADING
        | Capabilities::CUBE_ARRAY_TEXTURES
        | Capabilities::SHADER_FLOAT32_ATOMIC
}

/// Parses, validates and resolves the overrides of one module.
fn checked(
    label: &str,
    wgsl: &str,
    caps: Capabilities,
) -> Result<(naga::Module, naga::valid::ModuleInfo), String> {
    let module = naga::front::wgsl::parse_str(wgsl)
        .map_err(|e| format!("{label}: {}", e.emit_to_string(wgsl)))?;
    let info = Validator::new(ValidationFlags::all(), caps)
        .validate(&module)
        .map_err(|e| format!("{label}: {}", e.emit_to_string(wgsl)))?;
    let mut constants = naga::back::PipelineConstants::default();
    for (_, o) in module.overrides.iter() {
        if let Some(name) = &o.name
            && o.init.is_none()
        {
            constants.insert(name.clone(), 1.0);
        }
    }
    let (module, info) =
        naga::back::pipeline_constants::process_overrides(&module, &info, None, &constants)
            .map_err(|e| format!("{label}: overrides: {e:?}"))?;
    Ok((module.into_owned(), info.into_owned()))
}

fn full_tier_failures(label: &str, wgsl: &str) -> Vec<String> {
    let (module, info) = match checked(label, wgsl, full_capabilities()) {
        Ok(m) => m,
        Err(e) => return vec![e],
    };
    let mut failures = Vec::new();
    let options = msl::Options {
        lang_version: (2, 4),
        ..Default::default()
    };
    if let Err(e) = msl::write_string(&module, &info, &options, &msl::PipelineOptions::default()) {
        failures.push(format!("{label}: MSL: {e}"));
    }
    if let Err(e) = spv::write_vec(&module, &info, &spv::Options::default(), None) {
        failures.push(format!("{label}: SPIR-V: {e}"));
    }
    failures
}

fn gl_tier_failures(label: &str, wgsl: &str) -> Vec<String> {
    let (module, info) = match checked(label, wgsl, Capabilities::empty()) {
        Ok(m) => m,
        Err(e) => return vec![e],
    };
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
        if let Err(e) = result {
            failures.push(format!("{label}::{}: GLSL ES 3.0: {e}", entry.name));
        }
    }
    failures
}

#[test]
fn every_full_tier_module_validates_and_translates_to_msl_and_spirv() {
    let sources = og_render::shader_sources(true);
    assert!(sources.len() >= 30, "{} modules", sources.len());
    let failures: Vec<String> = sources
        .iter()
        .flat_map(|s| full_tier_failures(&s.label, &s.wgsl))
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn every_gl_tier_module_validates_and_translates_to_glsl_es_300() {
    let sources = og_render::shader_sources(false);
    assert!(sources.len() >= 25, "{} modules", sources.len());
    assert!(sources.iter().all(|s| s.gl));
    let failures: Vec<String> = sources
        .iter()
        .flat_map(|s| gl_tier_failures(&s.label, &s.wgsl))
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
