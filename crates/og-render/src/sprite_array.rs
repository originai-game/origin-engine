//! Sprites drawn from a layer of a texture array ([`crate::SpriteArray`],
//! 2D particles of an emitter with an array, and the overlay's
//! [`crate::Overlay::sprite_arrays`]): the sprite batch's instances (with
//! their `layer`) through pipeline variants that sample a
//! `texture_2d_array` (`shaders/sprite_array.wgsl`, and
//! `shaders/overlay_array.wgsl` for the overlay), so a run of one array is
//! one instanced draw whichever layer each sprite shows. One variant per
//! sprite pipeline: alpha, additive, the glow buffer's, and the overlay's
//! (in the output's format and its sRGB view's). Made on the first frame
//! that draws one (the overlay's on the first frame with an overlay array
//! sprite), so a renderer that never does pays nothing.

use crate::effects::SpriteBlend;
use crate::sprites::SPRITE_ARRAY_LAYOUT;

pub(crate) const SPRITE_ARRAY_WGSL: &str = include_str!("../shaders/sprite_array.wgsl");
pub(crate) const OVERLAY_ARRAY_WGSL: &str = include_str!("../shaders/overlay_array.wgsl");

/// The module's source as the renderer compiles it.
pub(crate) fn source() -> String {
    format!(
        "{}{}{SPRITE_ARRAY_WGSL}",
        crate::FRAME_WGSL,
        crate::LIGHTING_WGSL
    )
}

/// The array sprite pipelines (alpha, additive and glow) and their group
/// layout; the overlay's are made by [`SpriteArrayPipelines::overlay`].
pub(crate) struct SpriteArrayPipelines {
    pub layout: wgpu::BindGroupLayout,
    alpha: wgpu::RenderPipeline,
    additive: wgpu::RenderPipeline,
    /// Alpha-blended and untonemapped ([`crate::UNTONEMAPPED_BLEND`]).
    untonemapped: wgpu::RenderPipeline,
    pub glow: wgpu::RenderPipeline,
    /// The overlay's: in the output's format, and in its sRGB view's.
    overlay: Option<(wgpu::RenderPipeline, Option<wgpu::RenderPipeline>)>,
}

impl SpriteArrayPipelines {
    /// `frame_layout` and `fx_layout`: groups 0 and 2 of the sprite
    /// pipelines (the frame, and the light map with the screen size).
    pub fn new(
        device: &wgpu::Device,
        frame_layout: &wgpu::BindGroupLayout,
        fx_layout: &wgpu::BindGroupLayout,
        samples: u32,
    ) -> SpriteArrayPipelines {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite array"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sprite array"),
            bind_group_layouts: &[Some(frame_layout), Some(&layout), Some(fx_layout)],
            immediate_size: 0,
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sprite array"),
            source: wgpu::ShaderSource::Wgsl(source().into()),
        });
        let buffers = [Some(SPRITE_ARRAY_LAYOUT)];
        let pipeline = |label, fs, blend| {
            crate::hdr_pipeline(
                device,
                crate::HdrPipeline {
                    label,
                    layout: &pl,
                    module: &module,
                    vs: "vs",
                    fs,
                    buffers: &buffers,
                    cull: None,
                    depth_write: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    blend: Some(blend),
                    samples,
                },
            )
        };
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
        // The glow buffer, as the sprite glow pipeline: no depth, one
        // sample, added.
        let glow = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sprite array glow"),
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &buffers,
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_glow"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: crate::HDR_FORMAT,
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
            cache: crate::pipeline_cache::of(device).as_ref(),
        });
        SpriteArrayPipelines {
            alpha: pipeline("sprite array", "fs", wgpu::BlendState::ALPHA_BLENDING),
            additive: pipeline("sprite array additive", "fs", additive),
            untonemapped: pipeline(
                "sprite array untonemapped",
                "fs_untonemapped",
                crate::UNTONEMAPPED_BLEND,
            ),
            glow,
            overlay: None,
            layout,
        }
    }

    /// The pipeline of a run of `blend`; `tone_mapped` false: the
    /// untonemapped one (alpha-blended runs only, `sprites::build`).
    pub fn pipeline(&self, blend: SpriteBlend, tone_mapped: bool) -> &wgpu::RenderPipeline {
        match (blend, tone_mapped) {
            (SpriteBlend::Alpha, false) => &self.untonemapped,
            (SpriteBlend::Alpha, true) => &self.alpha,
            (SpriteBlend::Additive, _) => &self.additive,
        }
    }

    /// Makes the overlay's pipelines once: `overlay_layout` is the
    /// overlay's group 0, `output` the output's format and `linear` its
    /// sRGB view's (the overlay draws through it when there is one).
    pub fn make_overlay(
        &mut self,
        device: &wgpu::Device,
        overlay_layout: &wgpu::BindGroupLayout,
        output: wgpu::TextureFormat,
        linear: Option<wgpu::TextureFormat>,
    ) {
        if self.overlay.is_some() {
            return;
        }
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("overlay array"),
            bind_group_layouts: &[Some(overlay_layout), Some(&self.layout)],
            immediate_size: 0,
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("overlay array"),
            source: wgpu::ShaderSource::Wgsl(OVERLAY_ARRAY_WGSL.into()),
        });
        let buffers = [Some(crate::overlay::OVERLAY_LAYOUT)];
        let pipeline = |format| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("overlay array"),
                layout: Some(&pl),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs"),
                    compilation_options: Default::default(),
                    buffers: &buffers,
                },
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some("fs"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: crate::pipeline_cache::of(device).as_ref(),
            })
        };
        self.overlay = Some((pipeline(output), linear.map(pipeline)));
    }

    /// The overlay's pipeline: through the output's sRGB view (`linear`)
    /// or into the output itself.
    pub fn overlay(&self, linear: bool) -> &wgpu::RenderPipeline {
        let (output, srgb) = self.overlay.as_ref().expect("made before the overlay pass");
        match (linear, srgb) {
            (true, Some(p)) => p,
            _ => output,
        }
    }

    /// One array's group: every layer viewed as a 2D array (explicitly, so
    /// the GL backend binds it as one), with `sampler`.
    pub fn bind_group(
        &self,
        device: &wgpu::Device,
        texture: &wgpu::Texture,
        sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("sprite array"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sprite array"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        })
    }
}
