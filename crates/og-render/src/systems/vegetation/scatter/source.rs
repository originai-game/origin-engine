//! The scattered copies' modules.

use crate::{Capability, particles_gpu};
use og_render_data::assemble;
use og_render_data::shader_source::*;

/// What a scattered copy takes of the shared shading: the weather, the
/// effect group (at group 3 here) with the screen AO, and the decals.
const SHADING: crate::Shading = crate::Shading::lit().weather().effects(3).decals();

/// The scattered copies' lit module with a material shader's `hooks`
/// (vertex_offset, surface_color, surface_normal and surface_emissive run
/// on the copies as on meshes).
pub(crate) fn scatter_lit_source(hooks: &str) -> String {
    assemble(
        &[
            FRAME_WGSL,
            particles_gpu::NOISE_WGSL,
            MATERIAL_WGSL,
            MATERIAL_MAPS_WGSL,
            MATERIAL_DISSOLVE_WGSL,
            hooks,
            &SHADING.text(),
            INSTANCE_WGSL,
            crate::wind::WIND_COMMON_WGSL,
            crate::wind::WIND_LEAN_WGSL,
            crate::foliage::WGSL,
            wgsl3d!("scatter_common.wgsl"),
            wgsl3d!("scatter.wgsl"),
            wgsl3d!("impostor_common.wgsl"),
            wgsl3d!("impostor.wgsl"),
            wgsl3d!("crowd_card_common.wgsl"),
            wgsl3d!("crowd_card.wgsl"),
        ],
        &Capability::OPAQUE_SURFACE,
    )
}

/// The scattered copies' shadow module.
pub(crate) fn scatter_shadow_source() -> String {
    format!(
        "{LIBRARY_WGSL}{}{MATERIAL_WGSL}{MATERIAL_MAPS_WGSL}{MATERIAL_DISSOLVE_WGSL}{}{SHADOW_PASS_WGSL}{INSTANCE_WGSL}{}{}{}{}{}{}{}",
        particles_gpu::NOISE_WGSL,
        crate::wind::WIND_COMMON_WGSL,
        crate::wind::WIND_LEAN_WGSL,
        wgsl3d!("scatter_common.wgsl"),
        wgsl3d!("scatter_shadow.wgsl"),
        wgsl3d!("impostor_common.wgsl"),
        wgsl3d!("impostor_shadow.wgsl"),
        wgsl3d!("crowd_card_common.wgsl"),
        wgsl3d!("crowd_card_shadow.wgsl")
    )
}
