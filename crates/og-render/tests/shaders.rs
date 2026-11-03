//! Every WGSL module the renderer compiles (`og_render::shader_sources`), in
//! every variant, through naga at build time: parsed and validated, its
//! overrides resolved, then written as Metal's MSL, Vulkan's SPIR-V and
//! DX12's HLSL (shader model 6), as wgpu does when a pipeline is made (the
//! browser's WebGPU takes the WGSL validated here). A shader error fails
//! the gate instead of a player's renderer at startup (owed in og-render's
//! README: shader cooking per backend). Broken once by a function returning
//! an unknown identifier at the end of `scatter.wgsl`: the `scatter` module
//! fails with naga's message and the line.

use naga::back::{hlsl, msl, spv};
use naga::valid::{Capabilities, ValidationFlags, Validator};

/// What wgpu asks naga to allow for these modules: the
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
    set: &[(String, f64)],
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
    // The variant's own constants (the lean lit pipelines' `DISCARD`).
    for (name, value) in set {
        constants.insert(name.clone(), *value);
    }
    let (module, info) =
        naga::back::pipeline_constants::process_overrides(&module, &info, None, &constants)
            .map_err(|e| format!("{label}: overrides: {e:?}"))?;
    Ok((module.into_owned(), info.into_owned()))
}

fn failures(s: &og_render::ShaderSource) -> Vec<String> {
    let label = &s.label;
    let (module, info) = match checked(label, &s.wgsl, &s.constants, full_capabilities()) {
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
    // DX12: wgpu's dx12 backend compiles HLSL with DXC at shader model 6.
    let options = hlsl::Options {
        shader_model: hlsl::ShaderModel::V6_0,
        ..Default::default()
    };
    let mut out = String::new();
    if let Err(e) = hlsl::Writer::new(&mut out, &options, &hlsl::PipelineOptions::default())
        .write(&module, &info, None)
    {
        failures.push(format!("{label}: HLSL: {e}"));
    }
    failures
}

#[test]
fn every_module_validates_and_translates_to_msl_spirv_and_hlsl() {
    let mut sources = og_render::shader_sources();
    // The binding-free foliage contract before the meshlet caller lands:
    // make its deformation reachable with runtime (not folded) inputs.
    sources.push(og_render::ShaderSource {
        label: "foliage deformation".into(),
        wgsl: format!(
            "{}\n{}",
            og_render::foliage::WGSL,
            r#"
@vertex fn foliage_test(@location(0) p: vec3f, @location(1) f0: vec4f,
                       @location(2) f1: vec4f) -> @builtin(position) vec4f {
  let model = mat4x4f(vec4f(1, 0, 0, 0), vec4f(0, 1, 0, 0),
                     vec4f(0, 0, 1, 0), vec4f(0, 0, 0, 1));
  let wind = FoliageWind(vec4f(1, 0, 0, 0.3), vec4f(0.6, 10, 1, 2), vec4f(0, 0, 0, 0.628));
  return vec4f(p + foliage_offset(p, f0, f1, model, wind), 1);
}
"#
        ),
        constants: Vec::new(),
    });
    assert!(sources.len() >= 30, "{} modules", sources.len());
    let failures: Vec<String> = sources.iter().flat_map(failures).collect();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// What the renderer compiles is `shaders/` without its comments (build.rs
/// and the `wgsl!` macro): every file is there, line for line, each line
/// the source's with only a `//` comment and the spaces before it gone. WGSL
/// has no string literals, so that keeps every token of every module the
/// tests above compile, and the line naga names in a message.
#[test]
fn the_compiled_wgsl_is_the_source_without_its_comments() {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("shaders");
    let built = std::path::Path::new(env!("OUT_DIR")).join("shaders");
    let mut files = 0;
    let mut removed = 0;
    for entry in std::fs::read_dir(&source).expect("shaders/") {
        let path = entry.expect("an entry").path();
        if path.extension().is_none_or(|e| e != "wgsl") {
            continue;
        }
        let name = path.file_name().expect("a name");
        let src = std::fs::read_to_string(&path).expect("a WGSL file");
        let out = std::fs::read_to_string(built.join(name))
            .unwrap_or_else(|e| panic!("{name:?} was not built: {e}"));
        let (src_lines, out_lines): (Vec<_>, Vec<_>) =
            (src.split('\n').collect(), out.split('\n').collect());
        assert_eq!(src_lines.len(), out_lines.len(), "{name:?}: the lines");
        for (n, (s, o)) in src_lines.iter().zip(&out_lines).enumerate() {
            let expected = match s.find("//") {
                Some(at) => s[..at].trim_end(),
                None => s,
            };
            assert_eq!(o, &expected, "{name:?}:{}", n + 1);
            assert!(!o.contains("/*"), "{name:?}:{}: a block comment", n + 1);
        }
        removed += src.len() - out.len();
        files += 1;
    }
    assert!(files >= 50, "{files} WGSL files");
    assert!(removed > 50_000, "{removed} bytes of comments");
}

/// The volumetric fog's raymarch ([`og_render::VolumetricMethod::Raymarch`],
/// volumetric.rs) is lit by the point, spot and rect lights of each step's
/// cluster, not the sun alone (debt render-light #5): its fragment stage
/// reads the light list and the cluster table (frame bindings 7 and 8),
/// bounded by RAYMARCH_LIGHTS.
#[test]
fn the_volumetric_raymarch_reads_the_local_lights() {
    let vol = og_render::shader_sources()
        .into_iter()
        .find(|s| s.label == "volumetric raymarch")
        .expect("the raymarch module");
    assert!(vol.wgsl.contains(&format!(
        "const RAYMARCH_LIGHTS: u32 = {}u;",
        og_render::RAYMARCH_LIGHTS
    )));
    let (module, info) = checked(&vol.label, &vol.wgsl, &vol.constants, full_capabilities())
        .expect("the raymarch validates");
    let (i, _) = module
        .entry_points
        .iter()
        .enumerate()
        .find(|(_, e)| e.name == "fs_raymarch")
        .expect("fs_raymarch");
    let uses = info.get_entry_point(i);
    for binding in [7, 8] {
        let (global, _) = module
            .global_variables
            .iter()
            .find(|(_, g)| {
                g.binding
                    .as_ref()
                    .is_some_and(|b| b.group == 0 && b.binding == binding)
            })
            .unwrap_or_else(|| panic!("frame binding {binding}"));
        assert!(
            !uses[global].is_empty(),
            "fs_raymarch reads no frame binding {binding} (the light tables): lit by the sun alone"
        );
    }
}
