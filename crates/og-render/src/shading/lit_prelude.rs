//! What a lit module outside the material pipelines starts with.

use og_render_data::shader_source::*;

/// What the 3D fluid's and the particles' modules start with: the frame
/// group, the light tables, the lighting, the light volumes' hook, the
/// area lights (area.wgsl) and the shared shading.
pub(crate) fn fluid_shared_source() -> String {
    format!("{FRAME_WGSL}{}", crate::Shading::lit().text())
}
