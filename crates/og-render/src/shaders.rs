//! Every WGSL module the renderer compiles, per backend tier, as text: the
//! list `tests/shaders.rs` validates with naga (and translates to GLSL ES
//! 3.0 for the GL tier) in the gate, so a shader error is a failed build
//! and never reaches a player. Each entry is built by the same function or
//! constant the renderer's own call site uses; a new `create_shader_module`
//! site adds its module here. Material shaders a game declares are game
//! content: the cook validates them (`lit_module_source`).

use crate::{clusters::LightTables, skin::PaletteMode};

/// One module: a label, whether the GL tier compiles it, its WGSL and the
/// pipeline-overridable constants a pipeline sets on it.
#[derive(Clone, Debug)]
pub struct ShaderSource {
    pub label: String,
    /// The GL tier (WebGL2) compiles this module (translated to GLSL ES 3.0).
    pub gl: bool,
    pub wgsl: String,
    /// Overrides set for this variant (the rest keep their defaults): the
    /// lean lit pipelines' `DISCARD` false, which drops the fragment discard;
    /// the GL tier's lit pipelines' `AREA_LIGHTS` false, which drops the
    /// rect lights.
    pub constants: Vec<(String, f64)>,
}

/// Every module the renderer can compile on the full tier (`full`) or the
/// GL tier, in each variant the tier has: the joint palette as storage or
/// texture (full tier adapters without vertex storage take the texture),
/// the light tables as storage or uniform buffers, multisampled or single
/// depth for the occlusion pyramid.
pub fn shader_sources(full: bool) -> Vec<ShaderSource> {
    let mut out = Vec::new();
    let mut add = |label: &str, wgsl: String| {
        out.push(ShaderSource {
            label: label.to_string(),
            gl: !full,
            wgsl,
            constants: Vec::new(),
        })
    };
    let tables = if full {
        LightTables::Storage
    } else {
        LightTables::Uniform
    };
    let palettes: &[PaletteMode] = if full {
        &[PaletteMode::Storage, PaletteMode::Texture]
    } else {
        &[PaletteMode::Texture]
    };
    let hooks = crate::hooks::default_hooks();
    for &palette in palettes {
        let p = match palette {
            PaletteMode::Storage => "storage palette",
            PaletteMode::Texture => "texture palette",
        };
        // With two planar reflections (RenderDesc::planar_reflections) the
        // lit module reads the effect group's planar reflection where the
        // palette leaves it a texture slot.
        for (planars, planar) in planar_variants(palette) {
            add(
                &format!("pbr ({p}{planars})"),
                crate::lit_source_with(&hooks, palette, tables, planar),
            );
            add(
                &format!("pbr array maps ({p}{planars})"),
                crate::material::array_lit_source(&crate::lit_source_with(
                    &hooks, palette, tables, planar,
                )),
            );
        }
        add(
            &format!("physical ({p})"),
            crate::physical_source_with(&hooks, palette, tables),
        );
        add(
            &format!("shadow ({p})"),
            crate::shadow_source(&hooks, palette),
        );
        add(
            &format!("shadow array maps ({p})"),
            crate::material::array_shadow_source(&crate::shadow_source(&hooks, palette)),
        );
        add(
            &format!("stylise object ({p})"),
            crate::stylise_object::object_source(palette, tables),
        );
        // A full custom shader (custom.rs), lit and unlit, and its casters.
        let example = crate::custom::example();
        for lit in [true, false] {
            add(
                &format!("custom full lit={lit} ({p})"),
                crate::custom::lit_source(&example, lit, palette, tables),
            );
        }
        add(
            &format!("custom full shadow ({p})"),
            crate::custom::shadow_source(&example, palette),
        );
    }
    add("view resolve", crate::views::resolve_color_source());
    add("view glow", crate::views::glow_source());
    add(
        "custom post (crt)",
        crate::post_custom::post_module_source(&crate::post_custom::example_crt()),
    );
    // A target's depth texture, on every tier: from the view's linear
    // depth prepass, a colour target GLSL ES 3.0 loads.
    add("view resolve depth", crate::views::resolve_depth_source());
    add("lines", crate::lines::LINES_WGSL.to_string());
    add("sky", format!("{}{}", crate::FRAME_WGSL, crate::SKY_WGSL));
    add(
        "sprite",
        format!(
            "{}{}{}",
            crate::FRAME_WGSL,
            crate::LIGHTING_WGSL,
            crate::SPRITE_WGSL
        ),
    );
    add("sprite array", crate::sprite_array::source());
    add("sprite mesh", crate::sprite_mesh::source());
    add(
        "ribbon",
        format!("{}{}", crate::FRAME_WGSL, crate::RIBBON_WGSL),
    );
    add(
        "scatter",
        crate::scatter_lit_source(tables, &crate::hooks::default_hooks()),
    );
    add("scatter shadow", crate::scatter_shadow_source());
    add(
        "terrain",
        crate::terrain::terrain_lit_source(crate::FRAME_WGSL, &tables.wgsl(), crate::LIGHTING_WGSL),
    );
    add("terrain shadow", crate::terrain::terrain_shadow_source());
    let terrain =
        crate::terrain::terrain_lit_source(crate::FRAME_WGSL, &tables.wgsl(), crate::LIGHTING_WGSL);
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
    add(
        "metaballs",
        crate::metaballs::metaballs_source(crate::FRAME_WGSL),
    );
    add(
        "fluid 3d",
        crate::fluid3d::fluid_source(&crate::fluid_shared_source(tables)),
    );
    add("fluid 3d blur", crate::fluid3d::FLUID_BLUR_WGSL.to_string());
    add("volumetric", crate::volumetric::source(tables, full));
    if full {
        // RenderDesc::volumetric draws WebGL2's raymarch on a full-tier
        // device too.
        add(
            "volumetric raymarch",
            crate::volumetric::source(tables, false),
        );
    }
    add("cookie", crate::area::cookie_source());
    add(
        "particles",
        crate::particles_gpu::draw_source(crate::FRAME_WGSL),
    );
    for (label, wgsl) in crate::post::post_sources(full) {
        add(label, wgsl);
    }
    add("glass", crate::glass::GLASS_WGSL.to_string());
    add("overlay", crate::overlay::OVERLAY_WGSL.to_string());
    add(
        "overlay array",
        crate::sprite_array::OVERLAY_ARRAY_WGSL.to_string(),
    );
    add("mipmap", crate::mips::MIPMAP_WGSL.to_string());
    add("ibl", crate::ibl::IBL_WGSL.to_string());
    add("lights 2d", crate::light2d::LIGHT2D_WGSL.to_string());
    add("pack", crate::physical::PACK_WGSL.to_string());
    if full {
        // Compute: GPU particles and GPU culling run on the full tier only.
        add("particle sim", crate::particles_gpu::sim_source());
        add("cull", crate::cull_gpu::CULL_WGSL.to_string());
        add("depth pyramid", crate::cull_gpu::hiz_source(false));
        add(
            "depth pyramid (multisampled)",
            crate::cull_gpu::hiz_source(true),
        );
        add("occluder depth", crate::cull_gpu::DEPTH_WGSL.to_string());
    }
    // The lean lit pipelines' variant of the standard lit module (plain
    // materials, `pbr_lean.wgsl`): `DISCARD` false, no discard.
    for &palette in palettes {
        let p = match palette {
            PaletteMode::Storage => "storage palette",
            PaletteMode::Texture => "texture palette",
        };
        for (planars, planar) in planar_variants(palette) {
            out.push(ShaderSource {
                label: format!("pbr lean ({p}{planars})"),
                gl: !full,
                wgsl: crate::lit_source_with(&hooks, palette, tables, planar),
                constants: vec![("DISCARD".to_string(), 0.0)],
            });
        }
    }
    // The GL tier's lit modules as a scene without rect lights has them
    // made (`area::wgsl`): `AREA_LIGHTS` false, no rect-light shading.
    if !full {
        let palette = PaletteMode::Texture;
        let example = crate::custom::example();
        for (label, wgsl) in [
            (
                "pbr (texture palette, no rect lights)",
                crate::lit_source_with(&hooks, palette, tables, crate::planar_slot(palette, 1)),
            ),
            (
                "physical (texture palette, no rect lights)",
                crate::physical_source_with(&hooks, palette, tables),
            ),
            (
                "custom full lit=true (texture palette, no rect lights)",
                crate::custom::lit_source(&example, true, palette, tables),
            ),
        ] {
            out.push(ShaderSource {
                label: label.to_string(),
                gl: true,
                wgsl,
                constants: vec![("AREA_LIGHTS".to_string(), 0.0)],
            });
        }
    }
    out
}

/// The lit module's planar reflection variants for a joint palette mode: a
/// label suffix and whether the effect group carries the reflection, for
/// each count of [`crate::RenderDesc::planar_reflections`] that differs.
fn planar_variants(palette: PaletteMode) -> Vec<(&'static str, bool)> {
    let mut v = vec![("", crate::planar_slot(palette, 1))];
    if crate::planar_slot(palette, crate::MAX_PLANAR_REFLECTIONS) {
        v.push((", two planar reflections", true));
    }
    v
}
