//! Shared shader sources and effect layouts.

use super::*;
use crate::oit::{ARENA_WGSL, OIT_WGSL};
pub use og_render_data::shader_source::*;
pub use og_render_data::{WEATHER_SURFACE_STUB, assemble};

/// The lit pipelines' effect group (group 2): the effect uniform, the screen
/// AO, the decal atlas and their sampler, with `planar` ([`planar_slot`])
/// the planar reflection material shaders read (planar.wgsl), and the decal
/// records and cells (bindings 5 and 6, `fx.rs`).
pub(crate) fn fx_layout_entries(planar: bool) -> Vec<wgpu::BindGroupLayoutEntry> {
    let mut entries = vec![
        uniform_entry(0, wgpu::ShaderStages::VERTEX_FRAGMENT),
        tex2d(1),
        tex2d(2),
        sampler_entry(3, wgpu::SamplerBindingType::Filtering),
    ];
    if planar {
        entries.push(tex2d(4));
    }
    entries.extend(crate::fx::decal_layout_entries());
    entries
}

/// The scattered copies' lit module with a material shader's `hooks`
/// (vertex_offset, surface_color, surface_normal and surface_emissive run
/// on the copies as on meshes).
pub(crate) fn scatter_lit_source(hooks: &str) -> String {
    assemble(
        &[
            FRAME_WGSL,
            &clusters::wgsl(),
            particles_gpu::NOISE_WGSL,
            MATERIAL_WGSL,
            MATERIAL_MAPS_WGSL,
            MATERIAL_DISSOLVE_WGSL,
            hooks,
            LIGHTING_WGSL,
            crate::WEATHER_RAIN_WGSL,
            crate::WEATHER_SURFACE_WGSL,
            &og_render_data::lightmap::probe_wgsl(),
            AREA_WGSL,
            INSTANCE_WGSL,
            crate::wind::WIND_COMMON_WGSL,
            crate::wind::WIND_LEAN_WGSL,
            crate::foliage::WGSL,
            &fx_wgsl(3),
            wgsl3d!("decals.wgsl"),
            SCREEN_AO_WGSL,
            SURFACE_SHADE_WGSL,
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

/// A crowd impostor's bake (`crowd_impostor.rs`): a part's material
/// (group 1) and the palette (group 2) under `crowd_bake.wgsl`.
pub(crate) fn crowd_bake_source() -> String {
    format!(
        "{LIBRARY_WGSL}{}{MATERIAL_WGSL}{MATERIAL_MAPS_WGSL}{}{}",
        particles_gpu::NOISE_WGSL,
        skin::palette_wgsl(2),
        wgsl3d!("crowd_bake.wgsl")
    )
}

/// What the 3D fluid's and the particles' modules start with: the frame
/// group, the light tables, the lighting, the light volumes' hook, the
/// area lights (area.wgsl) and the shared shading.
pub(crate) fn fluid_shared_source() -> String {
    format!(
        "{FRAME_WGSL}{}{LIGHTING_WGSL}{}{AREA_WGSL}{SURFACE_SHADE_WGSL}",
        clusters::wgsl(),
        og_render_data::lightmap::probe_wgsl()
    )
}
pub(crate) const SKY_WGSL: &str = wgsl!("sky.wgsl");
/// The physical sky's scattering (`sky_physical.rs`), its block named
/// `SCATTER` until [`sky_scatter_wgsl`] names it.
const SKY_SCATTER_WGSL: &str = wgsl!("sky_scatter.wgsl");

/// The physical sky's functions reading their block from `block` (the
/// frame's `frame.sky_scatter`, the image-based light's `params.scatter`).
pub(crate) fn sky_scatter_wgsl(block: &str) -> String {
    SKY_SCATTER_WGSL.replace("SCATTER[", &format!("{block}["))
}

/// The sky pass's module: the frame, the physical sky, the sky pass.
pub(crate) fn sky_source() -> String {
    format!(
        "{FRAME_WGSL}{}{SKY_WGSL}",
        sky_scatter_wgsl("frame.sky_scatter")
    )
}
pub(crate) const RIBBON_WGSL: &str = wgsl!("ribbon.wgsl");

/// The strips' module (alpha strips join the order-independent passes).
pub(crate) fn ribbon_source() -> String {
    format!(
        "{FRAME_WGSL}{OIT_WGSL}{ARENA_WGSL}{}{}{RIBBON_WGSL}",
        crate::effect::EFFECT_WGSL,
        crate::effect::hooks_source(None)
    )
}

/// The shadow pass slot of the weather occlusion map (weather.rs
/// `shelter_view`), after the cascades' and the local faces'.
pub(crate) const SHELTER_SLOT: usize = CASCADES + LOCAL_SHADOW_LAYERS;
/// Uniform slots of the shadow passes (dynamic offsets): the cascades, the
/// local faces, then the weather occlusion map.
pub(crate) const SHADOW_SLOTS: usize = SHELTER_SLOT + 1;
pub(crate) const SHADOW_SLOT_STRIDE: u64 = 256;

#[cfg(test)]
mod tests {
    use super::*;

    /// The sampled textures of a bind group layout's entries.
    fn textures(entries: &[wgpu::BindGroupLayoutEntry]) -> usize {
        entries
            .iter()
            .filter(|e| matches!(e.ty, wgpu::BindingType::Texture { .. }))
            .count()
    }

    /// One planar reflection keeps the effect group main had before
    /// material shaders could read the mirror (four bindings and the decal
    /// buffers, no lean group for the lightmapped pipelines, the lit
    /// module's `planar_reflection` the stub); two add binding 4, and only
    /// then do the lightmapped pipelines need the group without it.
    #[test]
    fn the_effect_group_carries_the_planar_reflection_only_with_two() {
        let hooks = hooks::default_hooks();
        for (count, slot) in [(1, false), (2, true)] {
            let at = format!("with {count}");
            assert_eq!(planar_slot(count), slot, "{at}");
            let fx = fx_layout_entries(planar_slot(count));
            let bindings: Vec<u32> = fx.iter().map(|e| e.binding).collect();
            if slot {
                assert_eq!(bindings, [0, 1, 2, 3, 4, 5, 6], "{at}");
                assert_eq!(textures(&fx), 3, "{at}");
                // The lightmapped pipelines' group: the default one's.
                let lean = fx_layout_entries(false);
                assert_eq!(textures(&lean), 2, "{at}");
            } else {
                assert_eq!(bindings, [0, 1, 2, 3, 5, 6], "{at}");
                assert_eq!(textures(&fx), 2, "{at}");
            }
            let source = lit_source_with(&hooks, planar_slot(count));
            assert_eq!(source.contains("var planar_tex"), slot, "{at}");
            assert!(source.contains("fn planar_reflection("), "{at}");
        }
    }
}
