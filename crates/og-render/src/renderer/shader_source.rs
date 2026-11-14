//! Shared shader sources and effect layouts.

use super::*;
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

/// Every module the renderer can compile, in each variant it has: with and
/// without the planar reflection in the lit modules' effect group,
/// multisampled or single depth for the occlusion pyramid, the volumetric
/// fog's froxel grid or raymarch.
pub fn shader_sources() -> Vec<ShaderSource> {
    let mut out = Vec::new();
    shaders::mechanism_sources(&mut out);
    let mut add = |label: &str, wgsl: String| out.push(ShaderSource::plain(label, wgsl));
    crate::sprites::shader_sources(&mut add);
    crate::media::shader_sources(&mut add);
    crate::systems::shader_sources(&mut add);
    shaders::with_variants(out)
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
