//! The strips' pipelines made with the renderer (alpha and additive), and
//! the GPU trails that expand strips from their points where the device
//! can.

use super::*;

pub(super) struct StripPipelinesInputs<'a> {
    pub device: &'a wgpu::Device,
    pub ribbon_pl: &'a wgpu::PipelineLayout,
    pub ribbon_shader: &'a deferred::Module,
    pub samples: u32,
    pub additive: wgpu::BlendState,
    pub flags: wgpu::DownlevelFlags,
    pub frame_layout: &'a wgpu::BindGroupLayout,
    pub sprite_layout: &'a wgpu::BindGroupLayout,
    /// The effect materials' group 3 (effect.rs).
    pub effects_layout: &'a wgpu::BindGroupLayout,
}

pub(super) struct StripPipelinesOutputs {
    pub ribbon_pipelines: [Sampled<wgpu::RenderPipeline>; 2],
    pub gpu_trails: Option<ribbons_gpu::GpuTrails>,
}

impl Renderer {
    /// Makes the strips' pipelines and GPU trails.
    #[allow(clippy::too_many_lines)]
    pub(super) fn strip_pipelines(
        StripPipelinesInputs {
            device,
            ribbon_pl,
            ribbon_shader,
            samples,
            additive,
            flags,
            frame_layout,
            sprite_layout,
            effects_layout,
        }: StripPipelinesInputs<'_>,
    ) -> StripPipelinesOutputs {
        let ribbon_state = |blend| {
            deferred_hdr(
                device,
                ribbon_pl,
                ribbon_shader.clone(),
                samples,
                move |layout, module| HdrPipeline {
                    alpha_to_coverage: false,
                    label: "ribbon",
                    layout,
                    module,
                    vs: "vs",
                    fs: "fs",
                    buffers: RIBBON_BUFFERS,
                    cull: None,
                    depth_write: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    blend: Some(blend),
                    samples,
                },
            )
        };
        let ribbon_pipelines = [
            ribbon_state(additive),
            ribbon_state(wgpu::BlendState::ALPHA_BLENDING),
        ];
        // The strips expanded from their points (ribbons_gpu.rs), where
        // the device can: `vs_strip` reads them from group 2.
        let gpu_trails = ribbons_gpu::GpuTrails::supported(flags).then(|| {
            let points = ribbons_gpu::GpuTrails::draw_layout(device);
            let strip_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("strip"),
                bind_group_layouts: &[
                    Some(frame_layout),
                    Some(sprite_layout),
                    Some(&points),
                    Some(effects_layout),
                ],
                immediate_size: 0,
            });
            let strip_state = |blend| {
                deferred_hdr(
                    device,
                    &strip_pl,
                    ribbon_shader.clone(),
                    samples,
                    move |layout, module| HdrPipeline {
                        alpha_to_coverage: false,
                        label: "strip",
                        layout,
                        module,
                        vs: "vs_strip",
                        fs: "fs",
                        buffers: &[],
                        cull: None,
                        depth_write: false,
                        depth_compare: wgpu::CompareFunction::LessEqual,
                        blend: Some(blend),
                        samples,
                    },
                )
            };
            let pipelines = [
                strip_state(additive),
                strip_state(wgpu::BlendState::ALPHA_BLENDING),
            ];
            ribbons_gpu::GpuTrails::new(device, points, pipelines)
        });
        StripPipelinesOutputs {
            ribbon_pipelines,
            gpu_trails,
        }
    }
}
