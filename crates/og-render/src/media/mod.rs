//! The air and what is in it: fog and the sky's look (`atmosphere`), the
//! physical sky, the clouds' march, fog lit as a volume, the aurora, the
//! sky's capture for the image-based light and the weather. Every lit
//! shader reads it through the frame's uniform, so it is a mechanism.

pub(crate) mod atmosphere;
pub(crate) mod aurora;
pub(crate) mod clouds;
pub(crate) mod sky;
pub(crate) mod sky_capture;
pub(crate) mod sky_physical;
pub(crate) mod volumetric;
pub(crate) mod weather;

/// The sky's and the aurora's modules for [`crate::shader_sources`]: a
/// label and its WGSL.
pub(crate) fn shader_sources(add: &mut dyn FnMut(&str, String)) {
    add("sky", crate::sky_source());
    add(
        "aurora footprint",
        crate::aurora::FOOTPRINT_WGSL.to_string(),
    );
}
