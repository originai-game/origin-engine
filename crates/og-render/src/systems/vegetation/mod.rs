//! Vegetation: grass, scattered copies, impostors and trample.

pub(crate) mod grass;
pub(crate) mod impostor;
pub(crate) mod scatter;
pub(crate) mod scatter_cull;
pub(crate) mod scatter_place;

/// The lit features the scene's vegetation asks for
/// (`pipeline_features.rs`): the far grass on the ground.
pub(crate) fn features(scene: &crate::RenderScene) -> crate::pipeline_features::Features {
    use crate::pipeline_features::Features;
    Features::NONE.with(Features::GRASS, !scene.world.grass.is_empty())
}

/// The system's modules for [`crate::shader_sources`]: a label and its WGSL.
pub(crate) fn shader_sources(add: &mut dyn FnMut(&str, String)) {
    add("scatter cull", crate::scatter_cull::source());
    add("scatter place", crate::scatter_place::source(false));
    add(
        "scatter place (layer arrays)",
        crate::scatter_place::source(true),
    );
    // Wind on placed models (R6): the lit and shadow modules with its hook.
    let wind = crate::wind_shader().hooks_wgsl();
    add("wind", crate::lit_source_with(&wind, false));
    add("wind shadow", crate::shadow_source(&wind));
    add(
        "scatter",
        crate::scatter_lit_source(&crate::hooks::default_hooks(), false),
    );
    // Built only with Caps::RG32_STORAGE (wgsl_rules.rs `compat`).
    add(
        "scatter (prepass depth)",
        crate::scatter_lit_source(&crate::hooks::default_hooks(), true),
    );
    add("scatter shadow", crate::scatter_shadow_source());
    add("grass", crate::grass::lit_source(false));
    add("grass shadow", crate::grass::shadow_source(false));
    add(
        "grass generate",
        crate::grass::generate_source(false, false),
    );
    add(
        "grass generate (layer arrays)",
        crate::grass::generate_source(true, false),
    );
}

/// Grass modules compiled only when [`crate::Caps::VERTEX_STORAGE`] is present.
pub(crate) fn vertex_storage_shader_sources(out: &mut Vec<crate::ShaderSource>) {
    let caps = crate::Caps::VERTEX_STORAGE;
    out.push(crate::ShaderSource::with_caps(
        "grass (vertex storage)",
        crate::grass::lit_source(true),
        caps,
    ));
    out.push(crate::ShaderSource::with_caps(
        "grass shadow (vertex storage)",
        crate::grass::shadow_source(true),
        caps,
    ));
    out.push(crate::ShaderSource::with_caps(
        "grass generate (vertex storage)",
        crate::grass::generate_source(false, true),
        caps,
    ));
    out.push(crate::ShaderSource::with_caps(
        "grass generate (layer arrays, vertex storage)",
        crate::grass::generate_source(true, true),
        caps,
    ));
}
