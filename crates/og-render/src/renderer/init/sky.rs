//! The sky's resources made with the renderer: its two pipelines (the sky
//! cube's and the procedural look's, at the far plane after the opaque
//! batches), the IBL generator, the cloud noise and aurora placeholders and
//! the sky parameters.

use super::*;

pub(super) struct SkyResourcesInputs<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub sky_pl: &'a wgpu::PipelineLayout,
    pub sky_shader: &'a deferred::Module,
    pub samples: u32,
}

pub(super) struct SkyResourcesOutputs {
    pub sky_pipeline: Sampled<wgpu::RenderPipeline>,
    pub sky_look_pipeline: Sampled<wgpu::RenderPipeline>,
    pub ibl: ibl::IblGenerator,
    pub cloud_noise: clouds::CloudNoise,
    pub sky_params: wgpu::Buffer,
    pub aurora_map: aurora::AuroraMap,
}

impl Renderer {
    /// Makes the sky's pipelines and resources.
    #[allow(clippy::too_many_lines)]
    pub(super) fn sky_resources(
        SkyResourcesInputs {
            device,
            queue,
            sky_pl,
            sky_shader,
            samples,
        }: SkyResourcesInputs<'_>,
    ) -> SkyResourcesOutputs {
        // At the far plane, after the opaque batches: only uncovered pixels pass.
        let sky_state = |fs: &'static str| {
            deferred_hdr(
                device,
                sky_pl,
                sky_shader.clone(),
                samples,
                move |layout, module| HdrPipeline {
                    alpha_to_coverage: false,
                    label: "sky",
                    layout,
                    module,
                    vs: "vs_sky",
                    fs,
                    buffers: &[],
                    cull: None,
                    depth_write: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    blend: None,
                    samples,
                },
            )
        };
        let sky_pipeline = sky_state("fs_sky");
        let sky_look_pipeline = sky_state("fs_sky_look");
        let ibl = ibl::IblGenerator::new(device);
        let cloud_noise = clouds::CloudNoise::placeholder(device, queue);
        let sky_params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sky params"),
            size: std::mem::size_of::<SkyParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let aurora_map = aurora::AuroraMap::placeholder(device, queue);
        SkyResourcesOutputs {
            sky_pipeline,
            sky_look_pipeline,
            ibl,
            cloud_noise,
            sky_params,
            aurora_map,
        }
    }
}
