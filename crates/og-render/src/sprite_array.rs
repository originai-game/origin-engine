//! Sprites drawn from a layer of a texture array ([`crate::SpriteArray`]):
//! the sprite batch's instances (with their `layer`) through a pipeline
//! variant that samples a `texture_2d_array` (`shaders/sprite_array.wgsl`),
//! so a run of one array is one instanced draw whichever layer each sprite
//! shows. Made on the first frame that draws one, so a renderer that never
//! does pays nothing.

use crate::effects::SpriteBlend;
use crate::sprites::SPRITE_ARRAY_LAYOUT;

pub(crate) const SPRITE_ARRAY_WGSL: &str = include_str!("../shaders/sprite_array.wgsl");

/// The module's source as the renderer compiles it.
pub(crate) fn source() -> String {
    format!("{}{SPRITE_ARRAY_WGSL}", crate::FRAME_WGSL)
}

/// The array sprite pipelines (alpha and additive) and their group layout.
pub(crate) struct SpriteArrayPipelines {
    pub layout: wgpu::BindGroupLayout,
    alpha: wgpu::RenderPipeline,
    additive: wgpu::RenderPipeline,
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
        let pipeline = |label, blend| {
            crate::hdr_pipeline(
                device,
                crate::HdrPipeline {
                    label,
                    layout: &pl,
                    module: &module,
                    vs: "vs",
                    fs: "fs",
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
        SpriteArrayPipelines {
            alpha: pipeline("sprite array", wgpu::BlendState::ALPHA_BLENDING),
            additive: pipeline("sprite array additive", additive),
            layout,
        }
    }

    pub fn pipeline(&self, blend: SpriteBlend) -> &wgpu::RenderPipeline {
        match blend {
            SpriteBlend::Alpha => &self.alpha,
            SpriteBlend::Additive => &self.additive,
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
