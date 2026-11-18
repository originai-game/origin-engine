//! The sky's resources made with the renderer: its two pipelines (the sky
//! cube's and the procedural look's, at the far plane after the opaque
//! batches), the IBL generator, the cloud noise and aurora placeholders and
//! the sky parameters.

use super::*;

pub(super) struct SkyResourcesInputs<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub sky_pl: &'a wgpu::PipelineLayout,
    pub frame_layout: &'a wgpu::BindGroupLayout,
    pub sky_layout: &'a wgpu::BindGroupLayout,
    pub sky_shader: &'a deferred::Module,
    pub samples: u32,
}

pub(super) struct SkyResourcesOutputs {
    pub sky_pipeline: Sampled<wgpu::RenderPipeline>,
    pub sky_look_pipeline: Sampled<wgpu::RenderPipeline>,
    pub clouds_low: clouds::CloudsLow,
    pub ground: ground_heights::GroundCatcher,
    pub ibl: ibl::IblGenerator,
    pub cloud_noise: clouds::CloudNoise,
    pub cloud_shadow: clouds::CloudShadow,
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
            frame_layout,
            sky_layout,
            sky_shader,
            samples,
        }: SkyResourcesInputs<'_>,
    ) -> SkyResourcesOutputs {
        // At the far plane, after the opaque batches: only uncovered pixels pass.
        let sky_state = |layout: &wgpu::PipelineLayout, vs: &'static str, fs: &'static str| {
            deferred_hdr(
                device,
                layout,
                sky_shader.clone(),
                samples,
                move |layout, module| HdrPipeline {
                    alpha_to_coverage: false,
                    label: "sky",
                    layout,
                    module,
                    vs,
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
        let sky_pipeline = sky_state(sky_pl, "vs_sky", "fs_sky");
        let sky_look_pipeline = sky_state(sky_pl, "vs_sky_look", "fs_sky_look");
        // The main camera's clouds at half the frame's size: their march into the cloud target (group 2: the
        // history's step, with the frame's view depth in a frame with a prepass), the resolve that reprojects the
        // history around a march of one texel in four (group 2: the step, this frame's march, the history), and the
        // sky drawn under them, with the target as its group 2.
        let clouds_low = {
            use crate::gpu::layout::uniform_entry;
            let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("clouds low"),
                entries: &[texture_entry(0, true, wgpu::TextureViewDimension::D2)],
            });
            let sky_low_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("sky over low clouds"),
                bind_group_layouts: &[Some(frame_layout), Some(sky_layout), Some(&layout)],
                immediate_size: 0,
            });
            let step = uniform_entry(2, wgpu::ShaderStages::FRAGMENT);
            let temporal = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("clouds low march"),
                entries: &[step],
            });
            let depth = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("clouds low depth"),
                entries: &[
                    texture_entry(1, false, wgpu::TextureViewDimension::D2),
                    step,
                ],
            });
            let resolve = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("clouds low resolve"),
                entries: &[
                    step,
                    texture_entry(3, true, wgpu::TextureViewDimension::D2),
                    texture_entry(4, true, wgpu::TextureViewDimension::D2),
                ],
            });
            let with = |group: &wgpu::BindGroupLayout| {
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("clouds low"),
                    bind_group_layouts: &[Some(frame_layout), Some(sky_layout), Some(group)],
                    immediate_size: 0,
                })
            };
            let march = |layout: &wgpu::PipelineLayout, vs: &'static str, fs: &'static str| {
                let (device, layout, module) = (device.clone(), layout.clone(), sky_shader.clone());
                deferred::Deferred::anywhere(move || {
                    deferred::note(|| format!("clouds low {vs}/{fs}"));
                    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("clouds low"),
                        layout: Some(&layout),
                        vertex: wgpu::VertexState {
                            module: module.get(),
                            entry_point: Some(vs),
                            compilation_options: Default::default(),
                            buffers: &[],
                        },
                        primitive: wgpu::PrimitiveState::default(),
                        depth_stencil: None,
                        multisample: Default::default(),
                        fragment: Some(wgpu::FragmentState {
                            module: module.get(),
                            entry_point: Some(fs),
                            compilation_options: Default::default(),
                            targets: &[Some(wgpu::ColorTargetState {
                                format: HDR_FORMAT,
                                blend: None,
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                        }),
                        multiview_mask: None,
                        cache: crate::pipeline_cache::of(&device).as_ref(),
                    })
                })
            };
            // Only the march reads the overcast strip's grey, which
            // vs_sky_look marches: the sky over the target and the resolve
            // take vs_sky's triangle and compile no march of their own.
            clouds::CloudsLow {
                sky: sky_state(&sky_low_pl, "vs_sky", "fs_sky_look_low"),
                march: march(&with(&temporal), "vs_sky_look", "fs_clouds_low"),
                march_depth: march(&with(&depth), "vs_sky_look", "fs_clouds_low_depth"),
                resolve: march(&with(&resolve), "vs_sky", "fs_clouds_resolve"),
                temporal: clouds::CloudsTemporal::new(device, temporal, depth, resolve),
                layout,
            }
        };
        // The grounded ground's catcher (ground_heights.rs): made when first drawn.
        let ground =
            ground_heights::GroundCatcher::new(device, [frame_layout, sky_layout], samples);
        let ibl = ibl::IblGenerator::new(device);
        let cloud_noise = clouds::CloudNoise::placeholder(device, queue);
        let cloud_shadow = clouds::CloudShadow::placeholder(device);
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
            clouds_low,
            ground,
            ibl,
            cloud_noise,
            cloud_shadow,
            sky_params,
            aurora_map,
        }
    }
}
