//! Fluids: 3D particle fluids, visual fluids and 2D metaballs.

pub(crate) mod fluid3d;
pub(crate) mod metaballs;
pub(crate) mod visual_fluid;

/// The system's modules for [`crate::shader_sources`]: a label and its WGSL.
pub(crate) fn shader_sources(add: &mut dyn FnMut(&str, String)) {
    add("visual fluid", crate::visual_fluid::source());
    add(
        "metaballs",
        crate::metaballs::metaballs_source(crate::FRAME_WGSL),
    );
    add(
        "fluid 3d",
        crate::fluid3d::fluid_source(&crate::fluid_shared_source()),
    );
    add("fluid 3d blur", crate::fluid3d::FLUID_BLUR_WGSL.to_string());
}
