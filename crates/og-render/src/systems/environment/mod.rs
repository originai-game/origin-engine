//! The environment: sky, atmosphere, aurora, weather and wind.

pub(crate) mod atmosphere;
pub(crate) mod aurora;
pub(crate) mod sky;
pub(crate) mod sky_capture;
pub(crate) mod weather;

/// The system's modules for [`crate::shader_sources`]: a label and its WGSL.
pub(crate) fn shader_sources(add: &mut dyn FnMut(&str, String)) {
    add("sky", crate::sky_source());
    add(
        "aurora footprint",
        crate::aurora::FOOTPRINT_WGSL.to_string(),
    );
}
