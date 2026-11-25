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
pub(crate) fn full_capabilities() -> Capabilities {
    Capabilities::MULTISAMPLED_SHADING
        | Capabilities::CUBE_ARRAY_TEXTURES
        | Capabilities::SHADER_FLOAT32_ATOMIC
}

/// Naga validation capabilities for a module requiring `needs`.
pub(crate) fn capabilities_for(needs: og_render::Caps) -> Capabilities {
    let mut caps = full_capabilities();
    if needs.contains(og_render::Caps::SHADER_F16) {
        caps |= Capabilities::SHADER_FLOAT16;
    }
    if needs.contains(og_render::Caps::SUBGROUP) {
        caps |= Capabilities::SUBGROUP;
    }
    caps
}

/// Parses, validates and resolves the overrides of one module.
pub(crate) fn checked(
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
    let (module, info) = match checked(label, &s.wgsl, &s.constants, capabilities_for(s.needs)) {
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
    let mut spv_options = spv::Options::default();
    if (s.needs & (og_render::Caps::SUBGROUP | og_render::Caps::SHADER_F16))
        != og_render::Caps::NONE
    {
        spv_options.lang_version = (1, 3);
    }
    if let Err(e) = spv::write_vec(&module, &info, &spv_options, None) {
        failures.push(format!("{label}: SPIR-V: {e}"));
    }
    // DX12: wgpu's dx12 backend compiles HLSL with DXC at shader model 6
    // (6.2 for native float16_t when Caps::SHADER_F16 is active).
    let options = hlsl::Options {
        shader_model: if s.needs.contains(og_render::Caps::SHADER_F16) {
            hlsl::ShaderModel::V6_2
        } else {
            hlsl::ShaderModel::V6_0
        },
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
            "{}\n{}\n{}\n{}",
            og_render_data::LIBRARY_WGSL,
            og_render::foliage::WIND_WGSL,
            og_render::foliage::WGSL,
            r#"
@vertex fn foliage_test(@location(0) p: vec3f, @location(1) f0: vec4f,
                       @location(2) f1: vec4f) -> @builtin(position) vec4f {
  let model = mat4x4f(vec4f(1, 0, 0, 0), vec4f(0, 1, 0, 0),
                     vec4f(0, 0, 1, 0), vec4f(0, 0, 0, 1));
  let field = WindField(vec4f(1, 0, 0.3, 0.6), vec4f(1, 2, 1, 2), vec4f(3, 4, 3, 4), vec4f(1, 1, 0, 0), vec4f(0.0), vec4f(0.0, 0.0, 0.0, 128.0));
  let wind = FoliageWind(vec4f(1, 0, 0, 0.3), vec4f(0.6, 10, 1, 2), vec4f(0, 0, 0, 0.628), field);
  let normal = foliage_normal(vec3f(0, 1, 0), f1.xyz, vec3f(0, 5, 0), f1.w);
  let lean = wind_lean_analytic(wind.field, p.xz, false);
  return vec4f(p + normal + foliage_offset(p, f0, f1, model, wind, lean), 1);
}
"#
        ),
        constants: Vec::new(),
        needs: og_render::Caps::NONE,
    });
    assert!(sources.len() >= 30, "{} modules", sources.len());
    let failures: Vec<String> = sources.iter().flat_map(failures).collect();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// A mechanism's shader code has one copy, in the file that owns it: no
/// other file writes these tokens (a definition of a library function or
/// of a constant pi; Oklab's matrix, or Rec.601's weights where every
/// shader means Rec.709's; a local light's window and a spot's cone; a
/// phase function; the GGX lobe; the cofactor that carries a normal; a
/// vertex's joint blend; an FFT stage; the choice between a cascade's
/// depth and a local light's value, which every caster takes from
/// `shadow_depth` or `shadow_card`; the fog and the water body between the
/// eye and a point; the sky of a look; a read of the occluders' depth
/// pyramid, a cull's scan, or a level's error against the pixel
/// threshold).
#[test]
fn a_mechanisms_shader_code_is_written_once() {
    const OWNED: &[(&[&str], &[&str])] = &[
        (
            &["hash.wgsl"],
            &[
                "fn pcg(",
                "fn hash_lowbias(",
                "fn hash_unit(",
                "fn lattice3(",
            ],
        ),
        (
            &["common.wgsl"],
            &[
                "fn oct_encode(",
                "fn oct_decode(",
                "fn fullscreen_clip(",
                "fn view_point(",
                "fn ign(",
                "fn is_sky_depth(",
                "fn quad_corner(",
                "fn orthonormal_basis(",
                "fn face_dir(",
                "const SKY_VIEW_DEPTH",
                "PI: f32 =",
            ],
        ),
        (
            &["color.wgsl"],
            &[
                "fn srgb_encode(",
                "fn srgb_decode(",
                "fn luma(",
                "const LUMA",
                "fn hue_rotate(",
                "fn oklab_encode(",
                "fn oklab_decode(",
                "0.4122214708",
                "0.299, 0.587",
            ],
        ),
        (
            &["noise.wgsl"],
            &[
                "fn value_noise2(",
                "fn value_noise3(",
                "fn value_noise3_at(",
            ],
        ),
        (
            &["frame.wgsl"],
            &[
                "fn toward_eye(",
                "fn range_window(",
                "fn spot_cone(",
                "1.0 - ratio * ratio",
                "* light.cone.y",
            ],
        ),
        (
            &["ggx.wgsl"],
            &["fn d_ggx(", "fn v_ggx(", "* sqrt(n_dot_v * n_dot_v"],
        ),
        (
            &["instance.wgsl"],
            &["fn world_normal(", "cross(b, c), cross(c, a)"],
        ),
        (
            &["skin_storage.wgsl"],
            &[
                "fn skin_matrix(",
                "palette_joint(palette, joints.x)",
                "s.joints.x)",
            ],
        ),
        (
            &["fft.wgsl"],
            &["fn cmul(", "fn bitrev(", "fn signed_index(", "<< (s + 1u)"],
        ),
        (&["fullscreen.wgsl"], &["log2(0.18)"]),
        (
            &["tonemap.wgsl"],
            &["fn aces(", "fn agx(", "fn neutral(", "fn tone_curve("],
        ),
        (&["shadow_pass.wgsl"], &["local_value(", "local_distance("]),
        (
            &["media.wgsl"],
            &[
                "fn fog_amount(",
                "fn apply_fog(",
                "fn fog_keep(",
                "fn underwater_light(",
                "fn underwater_body(",
                "fn hg_phase(",
                "2.0 * g * cos",
            ],
        ),
        (
            &["sky_look.wgsl"],
            &[
                "fn sky_look(",
                "fn moon_at(",
                "fn clear_sky_light(",
                "fn cloud_deck(",
                "fn star_veil(",
            ],
        ),
        (&["cull_hiz.wgsl"], &["textureLoad(pyramid"]),
        (&["cull_scan.wgsl"], &["fn scan_shared("]),
        (&["lod.wgsl"], &["< LOD_PIXEL_ERROR"]),
    ];
    let built = std::path::Path::new(env!("OG_RENDER_SHADERS"));
    let mut copies = Vec::new();
    for entry in std::fs::read_dir(built).expect("the built shaders") {
        let path = entry.expect("an entry").path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let text = std::fs::read_to_string(&path).expect("a WGSL file");
        for (owners, tokens) in OWNED {
            if owners.contains(&name.as_str()) {
                continue;
            }
            for token in tokens.iter().filter(|t| text.contains(**t)) {
                copies.push(format!("{name}: `{token}` belongs to {owners:?}"));
            }
        }
    }
    assert!(copies.is_empty(), "{}", copies.join("\n"));
}

/// Every non-empty `.wgsl` file built into `OG_RENDER_SHADERS` must appear in
/// at least one validated module returned by `og_render::shader_sources()`
/// (or be an explicitly documented standalone fragment). Prevents new runtime
/// shaders from bypassing build-time validation and backend translation.
#[test]
fn every_wgsl_file_is_validated() {
    let sources = og_render::shader_sources();
    let built = std::path::Path::new(env!("OG_RENDER_SHADERS"));
    // Fragments whose text is only used in `og-cook` or standalone stubs
    // rewritten before compile.
    const EXEMPT: &[&str] = &[
        "foliage.wgsl", // validated in `every_module_validates_and_translates_to_msl_spirv_and_hlsl`
    ];
    let mut uncovered = Vec::new();
    for entry in std::fs::read_dir(built).expect("the built shaders") {
        let path = entry.expect("an entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("wgsl") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if EXEMPT.contains(&name.as_str()) {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("a WGSL file");
        let trimmed = text.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Check either full text match or first significant non-directive line
        // (some fragments undergo string substitution such as array-map
        // rewriting or stub substitution).
        let covered = sources.iter().any(|s| s.wgsl.contains(trimmed))
            || trimmed
                .lines()
                .map(str::trim)
                .filter(|l| {
                    l.starts_with("fn ")
                        || l.starts_with("struct ")
                        || l.starts_with("@compute")
                        || l.starts_with("@vertex")
                        || l.starts_with("@fragment")
                })
                .any(|sig| sources.iter().any(|s| s.wgsl.contains(sig)));
        if !covered {
            uncovered.push(name);
        }
    }
    uncovered.sort();
    assert!(
        uncovered.is_empty(),
        "WGSL files not covered by shader_sources(): {uncovered:#?}"
    );
}
