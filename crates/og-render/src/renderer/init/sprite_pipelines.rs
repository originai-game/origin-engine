//! The sprites' pipelines, made with the renderer: plain, additive,
//! untonemapped, cover, text (MSDF) and glow, and the additive blend the
//! strips share.

use super::*;

pub(super) struct SpritePipelinesInputs<'a> {
    pub device: &'a wgpu::Device,
    pub sprite_pl: &'a wgpu::PipelineLayout,
    pub sprite_shader: &'a deferred::Module,
    pub samples: u32,
}

pub(super) struct SpritePipelinesOutputs {
    pub additive: wgpu::BlendState,
    pub sprite_pipeline: Sampled<wgpu::RenderPipeline>,
    pub sprite_additive_pipeline: Sampled<wgpu::RenderPipeline>,
    pub sprite_untonemapped_pipeline: Sampled<wgpu::RenderPipeline>,
    pub sprite_cover_pipeline: Sampled<wgpu::RenderPipeline>,
    pub msdf_pipeline: Sampled<wgpu::RenderPipeline>,
    pub sprite_glow_pipeline: Deferred<wgpu::RenderPipeline>,
}

impl Renderer {
    /// Makes the sprites' pipelines.
    #[allow(clippy::too_many_lines)]
    pub(super) fn sprite_pipelines(
        SpritePipelinesInputs {
            device,
            sprite_pl,
            sprite_shader,
            samples,
        }: SpritePipelinesInputs<'_>,
    ) -> SpritePipelinesOutputs {
        let additive = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        };
        let sprite_state = |label: &'static str, fs: &'static str, blend| {
            deferred_hdr(
                device,
                sprite_pl,
                sprite_shader.clone(),
                samples,
                move |layout, module| HdrPipeline {
                    alpha_to_coverage: false,
                    label,
                    layout,
                    module,
                    vs: "vs",
                    fs,
                    buffers: SPRITE_BUFFERS,
                    cull: None,
                    depth_write: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    blend: Some(blend),
                    samples,
                },
            )
        };
        let sprite_pipeline = sprite_state("sprite", "fs", wgpu::BlendState::ALPHA_BLENDING);
        let sprite_additive_pipeline = sprite_state("sprite additive", "fs", additive);
        let sprite_untonemapped_pipeline =
            sprite_state("sprite untonemapped", "fs_untonemapped", UNTONEMAPPED_BLEND);
        // The colour kept, the alpha replaced by 1, over every pixel. No
        // groups: whichever sprite or array group is bound stays valid.
        let cover_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sprite cover"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });
        let sprite_cover_pipeline = deferred_hdr(
            device,
            &cover_pl,
            sprite_shader.clone(),
            samples,
            move |layout, module| HdrPipeline {
                alpha_to_coverage: false,
                label: "sprite cover",
                layout,
                module,
                vs: "vs_cover",
                fs: "fs_cover",
                buffers: &[],
                cull: None,
                depth_write: false,
                depth_compare: wgpu::CompareFunction::Always,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::Zero,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                samples,
            },
        );
        let msdf_pipeline = sprite_state("msdf text", "fs_msdf", wgpu::BlendState::ALPHA_BLENDING);
        // The glow buffer: no depth, one sample, added.
        let sprite_glow_pipeline = {
            let (device, sprite_pl, sprite_shader) =
                (device.clone(), sprite_pl.clone(), sprite_shader.clone());
            Deferred::new(move || {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("sprite glow"),
                    layout: Some(&sprite_pl),
                    vertex: wgpu::VertexState {
                        module: &sprite_shader,
                        entry_point: Some("vs"),
                        compilation_options: Default::default(),
                        buffers: SPRITE_BUFFERS,
                    },
                    primitive: Default::default(),
                    depth_stencil: None,
                    multisample: Default::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &sprite_shader,
                        entry_point: Some("fs_glow"),
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: HDR_FORMAT,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::One,
                                    dst_factor: wgpu::BlendFactor::One,
                                    operation: wgpu::BlendOperation::Add,
                                },
                                alpha: wgpu::BlendComponent::REPLACE,
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview_mask: None,
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                })
            })
        };
        SpritePipelinesOutputs {
            additive,
            sprite_pipeline,
            sprite_additive_pipeline,
            sprite_untonemapped_pipeline,
            sprite_cover_pipeline,
            msdf_pipeline,
            sprite_glow_pipeline,
        }
    }
}
