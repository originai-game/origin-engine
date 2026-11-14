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
    pub terrain: terrain::TerrainRenderer,
    pub scatter: scatter::ScatterRenderer,
    pub grass: grass::GrassRenderer,
}

impl Renderer {
    /// Makes the terrain, scatter and grass renderers.
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
        let terrain = terrain::TerrainRenderer::new(
            device,
            queue,
            terrain::TerrainPipelines {
                frame_layout,
                shadow_layout: shadow_bind_layout,
                off,
                samples,
                white: default_white.clone(),
                flat_normal: default_normal.clone(),
            },
        );
        let scatter = scatter::ScatterRenderer::new(
            device,
            queue,
            scatter::ScatterPipelines {
                frame_layout,
                material_layout,
                shadow_layout: shadow_bind_layout,
                fx_layout,
                lit_wgsl: Box::new(move |hooks: &str| off.leave_out(scatter_lit_source(hooks))),
                shadow_wgsl: scatter_shadow_source(),
                samples,
                wind_field: wind_field.lean_view.clone(),
                trample: trample.view.clone(),
                compute: vegetation_compute,
                indirect_per_draw: info.indirect_per_draw,
            },
        );
        let grass = grass::GrassRenderer::new(
            device,
            queue,
            grass::GrassPipelines {
                frame_layout,
                shadow_layout: shadow_bind_layout,
                fx_layout,
                samples,
                wind_field: wind_field.lean_view.clone(),
                trample: trample.view.clone(),
                off,
            },
        );
        WorldGroundOutputs {
            terrain,
            scatter,
            grass,
        }
    }
}
