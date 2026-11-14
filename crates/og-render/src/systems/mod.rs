//! World systems: each describes itself and calls the mechanisms (`gpu`, `shader`, `frame`, `visibility`, `mesh`, `shading`, `media`, `post`, `particles`, `debug`), which never name a system (tests/structure.rs).

pub(crate) mod crowd;
pub(crate) mod destruction;
pub(crate) mod environment;
pub(crate) mod fluids;
pub(crate) mod softbody;
pub(crate) mod sprites;
pub(crate) mod stylise;
pub(crate) mod terrain;
pub mod ui;
pub(crate) mod vegetation;
pub(crate) mod water;

/// Every system's modules for [`crate::shader_sources`]: a label and its WGSL.
pub(crate) fn shader_sources(add: &mut dyn FnMut(&str, String)) {
    crowd::shader_sources(add);
    destruction::shader_sources(add);
    environment::shader_sources(add);
    fluids::shader_sources(add);
    softbody::shader_sources(add);
    sprites::shader_sources(add);
    stylise::shader_sources(add);
    terrain::shader_sources(add);
    ui::shader_sources(add);
    vegetation::shader_sources(add);
    water::shader_sources(add);
}
