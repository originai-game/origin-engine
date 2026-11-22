//! The 3D world's ground made with the renderer: the terrain, the
//! scattered copies and the grass, over the default textures and the lit
//! layouts.

use super::*;

pub(super) struct WorldGroundInputs<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub frame_layout: &'a wgpu::BindGroupLayout,
    pub shadow_bind_layout: &'a wgpu::BindGroupLayout,
    pub material_layout: &'a wgpu::BindGroupLayout,
    pub fx_layout: &'a wgpu::BindGroupLayout,
    pub off: pipeline_features::Features,
    pub samples: u32,
    pub default_white: &'a wgpu::TextureView,
    pub default_normal: &'a wgpu::TextureView,
    pub vegetation_compute: bool,
    pub info: &'a RenderInfo,
    pub wind_field: &'a crate::wind_field::WindFieldSim,
    pub trample: &'a crate::trample::Trample,
}

pub(super) struct WorldGroundOutputs {
    pub terrain: OnDemand<terrain::TerrainRenderer>,
    pub terrain_fx: terrain::fx::TerrainFx,
    pub scatter: OnDemand<scatter::ScatterRenderer>,
    pub grass: OnDemand<grass::GrassRenderer>,
}

impl Renderer {
    /// The terrain, scatter and grass renderers, each made by the first
    /// frame that draws with it (OnDemand, the 3D world's engine module),
    /// and the terrains' effect group, which every frame binds.
    #[allow(clippy::too_many_lines)]
    pub(super) fn world_ground(
        WorldGroundInputs {
            device,
            queue,
            frame_layout,
            shadow_bind_layout,
            material_layout,
            fx_layout,
            off,
            samples,
            default_white,
            default_normal,
            vegetation_compute,
            info,
            wind_field,
            trample,
        }: WorldGroundInputs<'_>,
    ) -> WorldGroundOutputs {
        let terrain_fx = terrain::fx::TerrainFx::new(device);
        let (device, queue) = (device.clone(), queue.clone());
        let (frame_layout, shadow_layout) = (frame_layout.clone(), shadow_bind_layout.clone());
        let terrain = {
            let (device, queue) = (device.clone(), queue.clone());
            let (frame_layout, shadow_layout) = (frame_layout.clone(), shadow_layout.clone());
            let fx_layout = terrain_fx.layout.clone();
            let (white, flat_normal) = (default_white.clone(), default_normal.clone());
            OnDemand::new(Box::new(move || {
                terrain::TerrainRenderer::new(
                    &device,
                    &queue,
                    terrain::TerrainPipelines {
                        frame_layout: &frame_layout,
                        shadow_layout: &shadow_layout,
                        fx_layout: &fx_layout,
                        off,
                        samples,
                        white,
                        flat_normal,
                    },
                )
            }))
        };
        let (wind, trample) = (wind_field.lean_view.clone(), trample.view.clone());
        let scatter = {
            let (device, queue) = (device.clone(), queue.clone());
            let (frame_layout, shadow_layout) = (frame_layout.clone(), shadow_layout.clone());
            let (material_layout, fx_layout) = (material_layout.clone(), fx_layout.clone());
            let (wind, trample) = (wind.clone(), trample.clone());
            let indirect_per_draw = info.indirect_per_draw;
            let kept_depth = info.caps.contains(Caps::RG32_STORAGE);
            OnDemand::new(Box::new(move || {
                scatter::ScatterRenderer::new(
                    &device,
                    &queue,
                    scatter::ScatterPipelines {
                        frame_layout: &frame_layout,
                        material_layout: &material_layout,
                        shadow_layout: &shadow_layout,
                        fx_layout: &fx_layout,
                        lit_wgsl: Box::new(move |hooks: &str| {
                            off.leave_out(scatter_lit_source(hooks, kept_depth))
                        }),
                        shadow_wgsl: scatter_shadow_source(),
                        samples,
                        wind_field: wind,
                        trample,
                        compute: vegetation_compute,
                        kept_depth,
                        indirect_per_draw,
                    },
                )
            }))
        };
        let grass = {
            let fx_layout = fx_layout.clone();
            OnDemand::new(Box::new(move || {
                grass::GrassRenderer::new(
                    &device,
                    &queue,
                    grass::GrassPipelines {
                        frame_layout: &frame_layout,
                        shadow_layout: &shadow_layout,
                        fx_layout: &fx_layout,
                        samples,
                        wind_field: wind,
                        trample,
                        off,
                    },
                )
            }))
        };
        WorldGroundOutputs {
            terrain,
            terrain_fx,
            scatter,
            grass,
        }
    }
}
