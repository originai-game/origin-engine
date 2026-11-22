//! The scattered copies' modules.

use crate::Capability;
use og_render_data::assemble;
use og_render_data::shader_source::*;

/// What a scattered copy takes of the shared shading: the weather, the
/// effect group (at group 3 here) with the screen AO, and the decals.
const SHADING: crate::Shading = crate::Shading::lit().weather().effects(3).decals();

/// The scattered copies' lit module with a material shader's `hooks`
/// (vertex_offset, surface_color, surface_normal and surface_emissive run
/// on the copies as on meshes); with `kept`, the cards' pass over the
/// prepass's view depth (impostor_kept.wgsl), whose `rg32float` storage
/// binding a device without [`crate::Caps::RG32_STORAGE`] refuses with the
/// whole module.
pub(crate) fn scatter_lit_source(hooks: &str, kept: bool) -> String {
    assemble(
        &[
            FRAME_WGSL,
            MATERIAL_WGSL,
            MATERIAL_MAPS_WGSL,
            MATERIAL_DISSOLVE_WGSL,
            hooks,
            &SHADING.text(),
            INSTANCE_WGSL,
            crate::wind::WIND_COMMON_WGSL,
            crate::wind::WIND_LEAN_WGSL,
            crate::trample::READ_WGSL,
            crate::foliage::WGSL,
            wgsl!("scatter_common.wgsl"),
            wgsl!("scatter.wgsl"),
            &og_render_data::impostor::march_wgsl(),
            wgsl!("impostor_common.wgsl"),
            wgsl!("impostor.wgsl"),
            if kept {
                wgsl!("impostor_kept.wgsl")
            } else {
                ""
            },
            wgsl!("crowd_card_common.wgsl"),
            wgsl!("crowd_card.wgsl"),
        ],
        &Capability::OPAQUE_SURFACE,
    )
}

/// The scattered copies' shadow module.
pub(crate) fn scatter_shadow_source() -> String {
    format!(
        "{LIBRARY_WGSL}{MATERIAL_WGSL}{MATERIAL_MAPS_WGSL}{MATERIAL_DISSOLVE_WGSL}{}{SHADOW_PASS_WGSL}{INSTANCE_WGSL}{}{}{}{}{}{}{}{}{}{}",
        crate::wind::WIND_COMMON_WGSL,
        crate::wind::WIND_LEAN_WGSL,
        crate::trample::READ_WGSL,
        crate::foliage::WGSL,
        wgsl!("scatter_common.wgsl"),
        wgsl!("scatter_shadow.wgsl"),
        og_render_data::impostor::march_wgsl(),
        wgsl!("impostor_common.wgsl"),
        wgsl!("impostor_shadow.wgsl"),
        wgsl!("crowd_card_common.wgsl"),
        wgsl!("crowd_card_shadow.wgsl")
    )
}
