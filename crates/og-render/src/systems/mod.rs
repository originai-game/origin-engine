//! World systems: each describes itself and calls the mechanisms (`gpu`, `shader`, `frame`, `visibility`, `mesh`, `shading`, `media`, `post`, `particles`, `sprites`, `debug`), which never name a system (tests/structure.rs).

pub(crate) mod crowd;
pub(crate) mod destruction;
pub(crate) mod fluids;
pub(crate) mod softbody;
pub(crate) mod stylise;
pub(crate) mod terrain;
pub mod ui;
pub(crate) mod vegetation;
pub(crate) mod water;

/// The lit features the scene's systems ask for, beside the mechanisms'
/// own ([`crate::pipeline_features::Features::of`]).
pub(crate) fn features(scene: &crate::RenderScene) -> crate::pipeline_features::Features {
    water::features(scene) | vegetation::features(scene)
}

/// Every system's modules for [`crate::shader_sources`]: a label and its WGSL.
pub(crate) fn shader_sources(add: &mut dyn FnMut(&str, String)) {
    crowd::shader_sources(add);
    destruction::shader_sources(add);
    fluids::shader_sources(add);
    softbody::shader_sources(add);
    stylise::shader_sources(add);
    terrain::shader_sources(add);
    ui::shader_sources(add);
    vegetation::shader_sources(add);
    water::shader_sources(add);
}
