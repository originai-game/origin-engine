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
    // Pipeline-overridable constants (the lit shaders' EFFECTS) are
    // resolved before translation, as wgpu does; with the code they gate on.
    let mut constants = naga::back::PipelineConstants::default();
    if module
        .overrides
        .iter()
        .any(|(_, o)| o.name.as_deref() == Some("EFFECTS"))
    {
        constants.insert("EFFECTS".to_string(), 1.0);
    }
    let (module, info) =
        naga::back::pipeline_constants::process_overrides(&module, &info, None, &constants)
            .unwrap_or_else(|e| panic!("{name}: {e:?}"));
    let (module, info) = (module.as_ref(), info.as_ref());
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
            module,
            info,
            &options,
            &pipeline,
            naga::proc::BoundsCheckPolicies::default(),
        )
        .and_then(|mut w| w.write().map(|_| ()));
        // naga renames a second `gl` in a module to `gl_1`, which GLSL ES
        // reserves (the browser refuses the pipeline; naga does not).
        let reserved = out
            .match_indices("gl_")
            .any(|(i, _)| out[i + 3..].starts_with(|c: char| c.is_ascii_digit()));
        match result {
            Ok(()) if reserved => failures.push(format!(
                "{name}::{}: a `gl_<n>` identifier (GLSL reserves gl_)",
                entry.name
            )),
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

/// The physical lit shader (`physical.rs`) on the WebGL2 tier: it
/// translates, and its static and skinned pipelines sample at most the
/// sixteen textures a WebGL2 pipeline has across its groups (wgpu's GL
/// backend binds every group's textures to one set of units), with the
/// texture joint palette the tier uses. The pack pass translates too.
#[test]
fn the_physical_shaders_translate_and_fit_webgl2s_texture_units() {
    let builtin = og_render::MaterialShader {
        name: "built-in".into(),
        code: String::new(),
        params: Vec::new(),
    };
    let source = og_render::physical_module_source(&builtin, false);
    let failures = webgl2_failures("physical", &source);
    assert!(failures.is_empty(), "{failures:#?}");
    let pack = webgl2_failures("pack.wgsl", include_str!("../shaders/pack.wgsl"));
    assert!(pack.is_empty(), "{pack:#?}");
    let module = naga::front::wgsl::parse_str(&source).expect("parses");
    let textures = module
        .global_variables
        .iter()
        .filter(|(_, g)| matches!(module.types[g.ty].inner, naga::TypeInner::Image { .. }))
        .count();
    assert!(
        textures <= 16,
        "{textures} textures in the physical module: WebGL2 has 16 per pipeline"
    );
    assert_eq!(textures, 14, "frame 5, material 5, effects 3, skin 1");
}
