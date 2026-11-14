//! Every WGSL module the renderer compiles, as text: the list
//! `tests/shaders.rs` validates with naga (and translates for Metal,
//! Vulkan and DX12) in the gate, so a shader error is a failed build and
//! never reaches a player. Each entry is built by the same function or
//! constant the renderer's own call site uses; a new `create_shader_module`
//! site adds its module here. Material shaders a game declares are game
//! content: the cook validates them (`lit_module_source`).

/// One module: a label, its WGSL and the pipeline-overridable constants a
/// pipeline sets on it.
#[derive(Clone, Debug)]
pub struct ShaderSource {
    pub label: String,
    pub wgsl: String,
    /// Overrides set for this variant (the rest keep their defaults): the
    /// lean lit pipelines' `DISCARD` false, which drops the fragment discard.
    pub constants: Vec<(String, f64)>,
}

impl ShaderSource {
    /// A module with no overrides set.
    pub(crate) fn plain(label: &str, wgsl: String) -> ShaderSource {
        ShaderSource {
            label: label.to_string(),
            wgsl,
            constants: Vec::new(),
        }
    }
}

/// The renderer's own modules made without wgpu's loop bounding
/// (`unbounded_shader`), by their label in [`shader_sources`]
/// (their variants too). `tests/shader_loops.rs` proves every loop in them
/// ends.
pub const UNBOUNDED_MODULES: &[&str] = &[
    "stylise",
    "terrain",
    "terrain shadow",
    "volumetric",
    "volumetric raymarch",
];

/// The mechanisms' modules, in each variant they have: with and without
/// the planar reflection in the lit modules' effect group, multisampled
/// or single depth for the occlusion pyramid, the volumetric fog's froxel
/// grid or raymarch. A world system lists its own (`systems/`).
pub(crate) fn mechanism_sources(out: &mut Vec<ShaderSource>) {
    let mut add = |label: &str, wgsl: String| out.push(ShaderSource::plain(label, wgsl));
    let hooks = crate::hooks::default_hooks();
    // With two planar reflections (RenderDesc::planar_reflections) the lit
    // module reads the effect group's planar reflection.
    for (planars, planar) in planar_variants() {
        add(
            &format!("pbr{planars}"),
            crate::lit_source_with(&hooks, planar),
        );
        add(
            &format!("pbr array maps{planars}"),
            crate::material::array_lit_source(&crate::lit_source_with(&hooks, planar)),
        );
    }
    add("physical", crate::physical_source(&hooks));
    add(
        "meshlet cull",
        crate::meshlet_gpu::MESHLET_CULL_WGSL.to_string(),
    );
    add(
        "meshlet compact",
        crate::meshlet_gpu::compact::WGSL.to_string(),
    );
    add("shadow", crate::shadow_source(&hooks));
    // The vegetation's simulated wind field (G2).
    add("wind field", crate::wind_field::source());
    add("grass trample", crate::trample::source());
    add("meshlet lit", crate::meshlet_gpu::lit_source(false));
    add("meshlet lit (planar)", crate::meshlet_gpu::lit_source(true));
    add("meshlet shadow", crate::meshlet_gpu::shadow_source());
    add("meshlet early depth", crate::meshlet_gpu::depth_source());
    add("meshlet raster", crate::meshlet_gpu::raster_source());
    add(
        "shadow array maps",
        crate::material::array_shadow_source(&crate::shadow_source(&hooks)),
    );
    // A full custom shader (custom.rs), lit and unlit, and its casters.
    let example = crate::custom::example();
    for lit in [true, false] {
        add(
            &format!("custom full lit={lit}"),
            crate::custom::lit_source(&example, lit),
        );
    }
    add("custom full shadow", crate::custom::shadow_source(&example));
    add("view resolve", crate::views::resolve_color_source());
    add("view glow", crate::views::glow_source());
    add(
        "custom post (crt)",
        crate::post_custom::post_module_source(&crate::post_custom::example_crt()),
    );
    // A target's depth texture: from the view's linear depth prepass.
    add("view resolve depth", crate::views::resolve_depth_source());
    add("lines", crate::lines::LINES_WGSL.to_string());
    // The probe GI's trace and blend (gi_probes.rs).
    add("gi probes trace", crate::gi_probes::trace_source());
    add("gi probes blend", crate::gi_probes::blend_source());
    add("ssr", crate::ssr::source());
    add("ribbon", crate::ribbon_source());
    add("volumetric", crate::volumetric::source(true));
    // RenderDesc::volumetric's raymarch.
    add("volumetric raymarch", crate::volumetric::source(false));
    add("cookie", crate::area::cookie_source());
    add("shadow keep", crate::shadow_keep::source());
    add(
        "particles",
        crate::particles_gpu::draw_source(&crate::fluid_shared_source(), None),
    );
    for (label, wgsl) in crate::post::post_sources() {
        add(label, wgsl);
    }
    for (label, wgsl) in crate::post::fsr::sources() {
        add(label, wgsl);
    }
    add("glass", crate::glass::GLASS_WGSL.to_string());
    add(
        "color filter",
        crate::color_vision::COLOR_FILTER_WGSL.to_string(),
    );
    add("mipmap", crate::mips::MIPMAP_WGSL.to_string());
    add("ibl", crate::ibl::ibl_source());
    add("cloud noise", crate::clouds::NOISE_WGSL.to_string());
    add("cloud shadow", crate::clouds::bake_source());
    add("trails", crate::ribbons_gpu::WGSL.to_string());
    add("probe grid", crate::probe_grid::PROBE_GRID_WGSL.to_string());
    add("particle sprites", crate::particles_gpu::sprites_source());
    add("pack", crate::physical::PACK_WGSL.to_string());
    // Compute: GPU particles and GPU culling.
    add("particle sim", crate::particles_gpu::sim_source());
    add("cull", crate::cull_gpu::CULL_WGSL.to_string());
    add("depth pyramid", crate::cull_gpu::hiz_source(false));
    add(
        "depth pyramid (multisampled)",
        crate::cull_gpu::hiz_source(true),
    );
    add("occluder depth", crate::cull_gpu::DEPTH_WGSL.to_string());
    // Order-independent transparency's resolve (oit.rs), each variant.
    for (msaa, list) in [(false, false), (true, false), (false, true), (true, true)] {
        add(
            &format!("oit resolve (msaa {msaa}, lists {list})"),
            crate::oit::resolve_source(msaa, list),
        );
    }
    // The lean lit pipelines' variant of the standard lit module (plain
    // materials, `pbr_lean.wgsl`): `DISCARD` false, no discard.
    for (planars, planar) in planar_variants() {
        out.push(ShaderSource {
            label: format!("pbr lean{planars}"),
            wgsl: crate::lit_source_with(&hooks, planar),
            constants: vec![("DISCARD".to_string(), 0.0)],
        });
    }
    // The volumetric pass's pipelines under water (volumetric.rs
    // `Medium::Water`): `VOL_WATER` true.
    for (label, froxel) in [("volumetric", true), ("volumetric raymarch", false)] {
        out.push(ShaderSource {
            label: format!("{label} (water)"),
            wgsl: crate::volumetric::source(froxel),
            constants: vec![("VOL_WATER".to_string(), 1.0)],
        });
    }
}

/// `out` and its modules' variants.
pub(crate) fn with_variants(mut out: Vec<ShaderSource>) -> Vec<ShaderSource> {
    // Each module with lit features, as its variant without any
    // (pipeline_features.rs): the code the overrides fold away must leave a
    // module every backend takes.
    let off: Vec<ShaderSource> = out
        .iter()
        .filter_map(|s| {
            use crate::pipeline_features::Features;
            let declared = Features::declared(&s.wgsl);
            (declared != Features::NONE).then(|| ShaderSource {
                label: format!("{} (features off)", s.label),
                wgsl: s.wgsl.clone(),
                constants: s
                    .constants
                    .iter()
                    .cloned()
                    .chain(
                        Features::NONE
                            .constants(declared)
                            .into_iter()
                            .map(|(k, v)| (k.to_string(), v)),
                    )
                    .collect(),
            })
        })
        .collect();
    // And as a renderer made without every listed feature makes it
    // (RenderDesc::without): each override false in its text, the
    // weather's stub in the weather's place.
    let every = crate::LitFeatures::NAMED
        .iter()
        .fold(crate::LitFeatures::NONE, |a, (f, _)| a | *f)
        .features();
    let left_out: Vec<ShaderSource> = out
        .iter()
        .filter(|s| {
            crate::pipeline_features::Features::declared(&s.wgsl)
                != crate::pipeline_features::Features::NONE
        })
        .map(|s| ShaderSource {
            label: format!("{} (left out)", s.label),
            wgsl: every.leave_out(s.wgsl.clone()),
            constants: s.constants.clone(),
        })
        .collect();
    out.extend(off);
    out.extend(left_out);
    out
}

/// The lit module's planar reflection variants: a label suffix and whether
/// the effect group carries the reflection, for each count of
/// [`crate::RenderDesc::planar_reflections`] that differs.
fn planar_variants() -> Vec<(&'static str, bool)> {
    let mut v = vec![("", crate::planar_slot(1))];
    if crate::planar_slot(crate::MAX_PLANAR_REFLECTIONS) {
        v.push((", two planar reflections", true));
    }
    v
}
