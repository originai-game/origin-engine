//! Every WGSL module the renderer compiles (`og_render::shader_sources`), in
//! every variant of each tier, through naga at build time: parsed and
//! validated, its overrides resolved, then written as Metal's MSL,
//! Vulkan's SPIR-V and DX12's HLSL (shader model 6; the full tier) or GLSL
//! ES 3.0 for WebGL2 (the GL tier),
//! as wgpu does when a pipeline is made. A shader error fails the gate
//! instead of a player's renderer at startup (owed in og-render's README:
//! shader cooking per backend). Broken once by a function returning an
//! unknown identifier at the end of `scatter.wgsl`: both tiers' `scatter`
//! modules fail with naga's message and the line.

use naga::back::{glsl, hlsl, msl, spv};
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

fn full_tier_failures(s: &og_render::ShaderSource) -> Vec<String> {
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

/// One vertex or fragment entry point as GLSL ES 3.0 for WebGL2, as wgpu's
/// GL backend writes it.
fn glsl_es(
    module: &naga::Module,
    info: &naga::valid::ModuleInfo,
    entry: &naga::EntryPoint,
) -> Result<String, String> {
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
    let pipeline = glsl::PipelineOptions {
        shader_stage: entry.stage,
        entry_point: entry.name.clone(),
        multiview: None,
    };
    let mut out = String::new();
    glsl::Writer::new(
        &mut out,
        module,
        info,
        &options,
        &pipeline,
        naga::proc::BoundsCheckPolicies::default(),
    )
    .and_then(|mut w| w.write().map(|_| ()))
    .map_err(|e| e.to_string())?;
    Ok(out)
}

fn gl_tier_failures(s: &og_render::ShaderSource) -> Vec<String> {
    let label = &s.label;
    let (module, info) = match checked(label, &s.wgsl, &s.constants, Capabilities::empty()) {
        Ok(m) => m,
        Err(e) => return vec![e],
    };
    let mut failures = Vec::new();
    for entry in &module.entry_points {
        if !matches!(
            entry.stage,
            naga::ShaderStage::Vertex | naga::ShaderStage::Fragment
        ) {
            continue;
        }
        if let Err(e) = glsl_es(&module, &info, entry) {
            failures.push(format!("{label}::{}: GLSL ES 3.0: {e}", entry.name));
        }
    }
    failures
}

#[test]
fn every_full_tier_module_validates_and_translates_to_msl_spirv_and_hlsl() {
    let sources = og_render::shader_sources(true);
    assert!(sources.len() >= 30, "{} modules", sources.len());
    let failures: Vec<String> = sources.iter().flat_map(full_tier_failures).collect();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn every_gl_tier_module_validates_and_translates_to_glsl_es_300() {
    let sources = og_render::shader_sources(false);
    assert!(sources.len() >= 25, "{} modules", sources.len());
    assert!(sources.iter().all(|s| s.gl));
    let failures: Vec<String> = sources.iter().flat_map(gl_tier_failures).collect();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// Every `v[i]` with `i` not a constant and `v` a vector or a matrix (or a
/// pointer to one), as (function, the WGSL line it is on).
fn dynamic_vector_indexes(label: &str, wgsl: &str) -> Vec<(String, String)> {
    let module = naga::front::wgsl::parse_str(wgsl).expect("parses");
    let info = Validator::new(ValidationFlags::all(), Capabilities::empty())
        .validate(&module)
        .unwrap_or_else(|e| panic!("{label}: {}", e.emit_to_string(wgsl)));
    let mut found = Vec::new();
    let mut scan = |name: &str, f: &naga::Function, fi: &naga::valid::FunctionInfo| {
        for (h, e) in f.expressions.iter() {
            let naga::Expression::Access { base, .. } = *e else {
                continue;
            };
            let inner = match *fi[base].ty.inner_with(&module.types) {
                naga::TypeInner::Pointer { base, .. } => &module.types[base].inner,
                ref other => other,
            };
            let vector = match *inner {
                naga::TypeInner::Vector { .. } | naga::TypeInner::Matrix { .. } => true,
                naga::TypeInner::ValuePointer { size, .. } => size.is_some(),
                _ => false,
            };
            if vector {
                let line = f.expressions.get_span(h).location(wgsl).line_number;
                let text = wgsl.lines().nth(line as usize - 1).unwrap_or("").trim();
                found.push((name.to_string(), format!("line {line}: {text}")));
            }
        }
    };
    for (h, f) in module.functions.iter() {
        scan(f.name.as_deref().unwrap_or("?"), f, &info[h]);
    }
    for (i, e) in module.entry_points.iter().enumerate() {
        scan(&e.name, &e.function, info.get_entry_point(i));
    }
    found
}

/// The only functions of GL-tier modules (by module label) allowed to index
/// a vector or a matrix with a dynamic index. Each indexes a local vector,
/// not uniform data, and links in Chrome 155 on WebGL2 over ANGLE's D3D11
/// backend (`tests/evidence/release/windows-x64-webgl2/NOTES.md`).
const DYNAMIC_VECTOR_INDEXES: &[(&str, &str)] = &[
    ("pbr (texture palette)", "object_id"),
    ("pbr array maps (texture palette)", "object_id"),
    ("pbr lean (texture palette)", "object_id"),
    ("physical (texture palette)", "object_id"),
    ("stylise object (texture palette)", "object_id"),
    ("stylise", "hatch_surface"),
    ("stylise", "dots_surface"),
    ("terrain", "layer_weights"),
    ("terrain", "terrain_look"),
    ("terrain (layer arrays)", "layer_weights"),
    ("terrain (layer arrays)", "terrain_look"),
    ("pack", "fs"),
    ("custom post (crt)", "post"),
];

/// No GL-tier module indexes a vector or a matrix with a dynamic index
/// outside `DYNAMIC_VECTOR_INDEXES`, and every entry there is still used.
/// ANGLE's D3D11 backend (Chrome and Edge on Windows) rewrites `v[i]` into
/// a `dyn_index_*` helper. The light tables' (`cluster_table[c >> 2u][c &
/// 3u]` and the references') were written twice in every lit shader, which
/// FXC refuses (`X3003: redefinition`), so WebGL2 did not start
/// (`tests/evidence/release/windows-x64-webgl2/NOTES.md`). The tables now
/// read a whole vec4 by a dynamic array index and pick its word with masks
/// (clusters.rs). A new site must be linked in Chrome or Edge on Windows on
/// WebGL2 before it is listed.
#[test]
fn gl_tier_dynamic_vector_indexes_are_known() {
    let mut used = std::collections::BTreeSet::new();
    let mut failures = Vec::new();
    for s in og_render::shader_sources(false).iter() {
        for (function, at) in dynamic_vector_indexes(&s.label, &s.wgsl) {
            let key = (s.label.as_str(), function.as_str());
            match DYNAMIC_VECTOR_INDEXES.iter().find(|k| **k == key) {
                Some(k) => {
                    used.insert(*k);
                }
                None => failures.push(format!("{}: fn {function}, {at}", s.label)),
            }
        }
    }
    let stale: Vec<_> = DYNAMIC_VECTOR_INDEXES
        .iter()
        .filter(|k| !used.contains(*k))
        .collect();
    assert!(
        failures.is_empty(),
        "a dynamic index into a vector or matrix; ANGLE's D3D11 backend rewrites it into a \
         dyn_index helper that FXC can refuse (X3003) or compile for minutes on WebGL2 in \
         Chrome and Edge on Windows. Pick the component with masks or select, or index an \
         array of scalars, or link it there and list it in DYNAMIC_VECTOR_INDEXES:\n{}",
        failures.join("\n")
    );
    assert!(
        stale.is_empty(),
        "no longer indexed dynamically; remove from DYNAMIC_VECTOR_INDEXES: {stale:?}"
    );
}
/// TAA reads the depth buffer (binding 2, `Depth32Float`) through a
/// sampler on WebGL2, never with `texelFetch` alone: wgpu's GL backend binds
/// no sampler object to a texture read only with `texelFetch`, a depth
/// texture keeps GL's default linear filtering, and a `DEPTH_COMPONENT32F`
/// texture filtered linearly with no compare mode is incomplete under GLSL
/// ES 3.0, which reads it as 0. TAA then reprojected every pixel from the
/// near plane on WebGL2 (row 68; og-render README, "Where TAA reads its
/// history").
#[test]
fn taa_reads_the_depth_buffer_through_a_sampler_on_gl() {
    let taa = og_render::shader_sources(false)
        .into_iter()
        .find(|s| s.label == "taa")
        .expect("the taa module");
    let (module, info) = checked(&taa.label, &taa.wgsl, &taa.constants, Capabilities::empty())
        .expect("taa validates");
    let mut read = 0;
    for entry in module
        .entry_points
        .iter()
        .filter(|e| e.stage == naga::ShaderStage::Fragment)
    {
        let glsl = glsl_es(&module, &info, entry).expect("taa translates");
        assert!(
            !glsl.contains("texelFetch(_group_0_binding_2_"),
            "taa::{} reads the depth with texelFetch:\n{glsl}",
            entry.name
        );
        read += glsl.matches("textureLod(_group_0_binding_2_").count();
    }
    assert!(read >= 2, "the resolves sample the depth ({read} reads)");
}

/// The modules whose passes read one level of a mip chain through a view
/// whose base mip level is that level (the glass pyramid, a view's glow
/// chain, mip generation, the IBL prefilter) never ask it for its size on
/// WebGL2: ANGLE on D3D11 (Chrome and Edge on Windows) answers `textureSize`
/// with base level b as level 2b's size, which blurred og-ui/glass about
/// twice as wide as every other backend. Each pass takes the size it reads
/// from a uniform instead.
#[test]
fn level_chain_passes_take_no_texture_size_on_gl() {
    let sources = og_render::shader_sources(false);
    let mut checked_entries = 0;
    for label in ["glass", "view glow", "mipmap", "ibl"] {
        let s = sources
            .iter()
            .find(|s| s.label == label)
            .unwrap_or_else(|| panic!("the {label} module"));
        let (module, info) = checked(&s.label, &s.wgsl, &s.constants, Capabilities::empty())
            .unwrap_or_else(|e| panic!("{e}"));
        for entry in module
            .entry_points
            .iter()
            .filter(|e| e.stage == naga::ShaderStage::Fragment)
        {
            let glsl = glsl_es(&module, &info, entry).expect("translates");
            assert!(
                !glsl.contains("textureSize("),
                "{label}::{} asks a level view for its size:\n{glsl}",
                entry.name
            );
            checked_entries += 1;
        }
    }
    assert!(checked_entries >= 6, "{checked_entries} fragment entries");
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

/// WebGL2's volumetric fog (the raymarch, volumetric.rs) is lit by the
/// point, spot and rect lights of each step's cluster, not the sun alone
/// (debt render-light #5): its fragment stage reads the light list and the
/// cluster table (frame bindings 7 and 8), bounded by RAYMARCH_LIGHTS.
#[test]
fn the_gl_volumetric_raymarch_reads_the_local_lights() {
    let vol = og_render::shader_sources(false)
        .into_iter()
        .find(|s| s.label == "volumetric")
        .expect("the volumetric module");
    assert!(vol.wgsl.contains(&format!(
        "const RAYMARCH_LIGHTS: u32 = {}u;",
        og_render::RAYMARCH_LIGHTS
    )));
    let (module, info) = checked(&vol.label, &vol.wgsl, &vol.constants, Capabilities::empty())
        .expect("the raymarch validates");
    let entry = module
        .entry_points
        .iter()
        .find(|e| e.name == "fs_raymarch")
        .expect("fs_raymarch");
    let glsl = glsl_es(&module, &info, entry).expect("the raymarch translates");
    for binding in ["_group_0_binding_7_", "_group_0_binding_8_"] {
        assert!(
            glsl.contains(binding),
            "fs_raymarch reads no {binding} (the light tables): lit by the sun alone"
        );
    }
}
