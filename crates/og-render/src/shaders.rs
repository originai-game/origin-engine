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

/// Every module the renderer can compile, in each variant it has: with and
/// without the planar reflection in the lit modules' effect group,
/// multisampled or single depth for the occlusion pyramid, the volumetric
/// fog's froxel grid or raymarch.
pub fn shader_sources() -> Vec<ShaderSource> {
    let mut out = Vec::new();
    let mut add = |label: &str, wgsl: String| {
        out.push(ShaderSource {
            label: label.to_string(),
            wgsl,
            constants: Vec::new(),
        })
    };
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
    add("simulation surface", crate::surface::WGSL.to_string());
    add("garment draw", crate::garment_gpu::WGSL.to_string());
    add(
        "meshlet cull",
        crate::meshlet_gpu::MESHLET_CULL_WGSL.to_string(),
    );
    add("shadow", crate::shadow_source(&hooks));
    // Wind on placed models (R6): the lit and shadow modules with its hook.
    let wind = crate::wind_shader().hooks_wgsl();
    add("wind", crate::lit_source_with(&wind, false));
    add("wind shadow", crate::shadow_source(&wind));
    add("meshlet lit", crate::meshlet_gpu::lit_source(false));
    add("meshlet lit (planar)", crate::meshlet_gpu::lit_source(true));
    add("meshlet shadow", crate::meshlet_gpu::shadow_source());
    add("meshlet early depth", crate::meshlet_gpu::depth_source());
    add("meshlet raster", crate::meshlet_gpu::raster_source());
    add(
        "shadow array maps",
        crate::material::array_shadow_source(&crate::shadow_source(&hooks)),
    );
    add("stylise object", crate::stylise_object::object_source());
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
    add("sky", crate::sky_source());
    add(
        "sprite",
        format!(
            "{}{}{}",
            crate::FRAME_WGSL,
            crate::LIGHTING_WGSL,
            crate::SPRITE_WGSL
        ),
    );
    // The probe GI's trace and blend (gi_probes.rs).
    add("gi probes trace", crate::gi_probes::trace_source());
    add("gi probes blend", crate::gi_probes::blend_source());
    add("sprite array", crate::sprite_array::source());
    add("sprite mesh", crate::sprite_mesh::source());
    add("sprite skin", crate::sprite_skin::source());
    add("tiles", crate::tiles_gpu::source());
    add(
        "ribbon",
        format!("{}{}", crate::FRAME_WGSL, crate::RIBBON_WGSL),
    );
    add(
        "scatter",
        crate::scatter_lit_source(&crate::hooks::default_hooks()),
    );
    add("scatter shadow", crate::scatter_shadow_source());
    add("grass", crate::grass::lit_source());
    add("grass shadow", crate::grass::shadow_source());
    add("grass generate", crate::grass::generate_source(false));
    add(
        "grass generate (layer arrays)",
        crate::grass::generate_source(true),
    );
    add(
        "terrain",
        crate::terrain::terrain_lit_source(
            crate::FRAME_WGSL,
            &crate::clusters::wgsl(),
            crate::LIGHTING_WGSL,
        ),
    );
    add("terrain shadow", crate::terrain::terrain_shadow_source());
    let terrain = crate::terrain::terrain_lit_source(
        crate::FRAME_WGSL,
        &crate::clusters::wgsl(),
        crate::LIGHTING_WGSL,
    );
    add(
        "terrain (layer arrays)",
        crate::terrain::terrain_lit_array_source(&terrain),
    );
    add(
        "terrain shadow (layer arrays)",
        crate::terrain::terrain_shadow_array_source(),
    );
    add(
        "water",
        crate::water::water_source(crate::FRAME_WGSL, crate::LIGHTING_WGSL),
    );
    for (label, wgsl) in crate::ocean_fft::sources() {
        add(label, wgsl.to_string());
    }
    add(
        "metaballs",
        crate::metaballs::metaballs_source(crate::FRAME_WGSL),
    );
    add(
        "fluid 3d",
        crate::fluid3d::fluid_source(&crate::fluid_shared_source()),
    );
    add("fluid 3d blur", crate::fluid3d::FLUID_BLUR_WGSL.to_string());
    add("volumetric", crate::volumetric::source(true));
    // RenderDesc::volumetric's raymarch.
    add("volumetric raymarch", crate::volumetric::source(false));
    add("cookie", crate::area::cookie_source());
    add(
        "particles",
        crate::particles_gpu::draw_source(&crate::fluid_shared_source()),
    );
    for (label, wgsl) in crate::post::post_sources() {
        add(label, wgsl);
    }
    add("glass", crate::glass::GLASS_WGSL.to_string());
    add("overlay", crate::overlay::overlay_source());
    // The UI pass (src/ui): its shapes, the built-in materials' composite
    // and a custom material's.
    add("ui", crate::ui::pass::ui_source());
    add("ui builtin materials", crate::ui::pass::builtin_source());
    add("ui mass", crate::ui::pass::mass_source());
    add(
        "ui material (duotone)",
        crate::ui::ui_material_module_source(&crate::ui::example_ui_material()),
    );
    add(
        "overlay array",
        crate::sprite_array::OVERLAY_ARRAY_WGSL.to_string(),
    );
    add("mipmap", crate::mips::MIPMAP_WGSL.to_string());
    add("ibl", crate::ibl::ibl_source());
    add("cloud noise", crate::clouds::NOISE_WGSL.to_string());
    add(
        "aurora footprint",
        crate::aurora::FOOTPRINT_WGSL.to_string(),
    );
    add("water ripples", crate::water_ripples::WGSL.to_string());
    add("trails", crate::ribbons_gpu::WGSL.to_string());
    add("probe grid", crate::probe_grid::PROBE_GRID_WGSL.to_string());
    add("particle sprites", crate::particles_gpu::sprites_source());
    add("lights 2d", crate::light2d::LIGHT2D_WGSL.to_string());
    add(
        "lights 2d shadows",
        crate::light2d::LIGHT2D_SHADOW_WGSL.to_string(),
    );
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
    // The lean lit pipelines' variant of the standard lit module (plain
    // materials, `pbr_lean.wgsl`): `DISCARD` false, no discard.
    for (planars, planar) in planar_variants() {
        out.push(ShaderSource {
            label: format!("pbr lean{planars}"),
            wgsl: crate::lit_source_with(&hooks, planar),
            constants: vec![("DISCARD".to_string(), 0.0)],
        });
    }
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
