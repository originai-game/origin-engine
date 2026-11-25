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
    entries.extend(crate::decals::decal_layout_entries());
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
    add("pre-skin", crate::mesh::pre_skin::source());
    crate::systems::shader_sources(&mut add);
    drop(add);
    crate::systems::crowd::subgroup_shader_sources(&mut out);
    crate::systems::vegetation::vertex_storage_shader_sources(&mut out);
    crate::systems::terrain::vertex_storage_shader_sources(&mut out);
    shaders::with_variants(out)
}

/// The shadow pass slot of the weather occlusion map (weather.rs
/// `shelter_view`), after the cascades' and the local faces'.
pub(crate) const SHELTER_SLOT: usize = CASCADES + LOCAL_SHADOW_LAYERS;
/// The shadow pass slot of the grounded ground's height map (ground_heights.rs
/// `ground_view`), after the shelter map's.
pub(crate) const GROUND_SLOT: usize = SHELTER_SLOT + 1;
/// Uniform slots of the shadow passes (dynamic offsets): the cascades, the
/// local faces, the weather occlusion map, then the ground's height map.
pub(crate) const SHADOW_SLOTS: usize = GROUND_SLOT + 1;
/// A shadow pass slot's stride: its uniform rounded up to the dynamic
/// offset alignment.
pub(crate) const SHADOW_SLOT_STRIDE: u64 =
    (std::mem::size_of::<ShadowPassUniform>() as u64).next_multiple_of(256);
