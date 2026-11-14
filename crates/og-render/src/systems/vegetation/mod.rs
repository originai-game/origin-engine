//! Vegetation: grass, scattered copies, impostors and trample.

pub(crate) mod grass;
pub(crate) mod impostor;
pub(crate) mod scatter;
pub(crate) mod scatter_cull;
pub(crate) mod scatter_place;
pub mod trample;

/// The lit features the scene's vegetation asks for
/// (`pipeline_features.rs`): the far grass on the ground.
pub(crate) fn features(scene: &crate::RenderScene) -> crate::pipeline_features::Features {
    use crate::pipeline_features::Features;
    Features::NONE.with(Features::GRASS, !scene.world.grass.is_empty())
}

/// The system's modules for [`crate::shader_sources`]: a label and its WGSL.
pub(crate) fn shader_sources(add: &mut dyn FnMut(&str, String)) {
    add("grass trample", crate::trample::source());
    add("scatter cull", crate::scatter_cull::source());
    // Wind on placed models (R6): the lit and shadow modules with its hook.
    let wind = crate::wind_shader().hooks_wgsl();
    add("wind", crate::lit_source_with(&wind, false));
    add("wind shadow", crate::shadow_source(&wind));
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
}
