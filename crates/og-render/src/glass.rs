//! Glass: the frame under the overlay, blurred, for the overlay's glass
//! panels ([`crate::scene::Glass`]). Only when an overlay has glass: the
//! tonemap writes into `source` instead of the output, a blit copies it to
//! the output unchanged, and a chain of downsamples writes the mip levels
//! of `pyramid` (level k at 1/2^(k+1) of the output, each a 13-tap blur of
//! the level above). A glass panel samples the pyramid at its screen
//! position, at the level its blur radius asks for (trilinear between
//! levels), tints it and masks it to its rounded rectangle. The panels blur
//! the scene, not other overlay content under them. A frame without glass
//! pays nothing.

use crate::post::fullscreen_pass;

/// Mip levels of the pyramid: level 5 is 1/64 of the output, a blur of
/// about 64 device pixels.
pub(crate) const LEVELS: u32 = 6;

pub(crate) struct GlassPasses {
    layout: wgpu::BindGroupLayout,
    down: wgpu::RenderPipeline,
    blit: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    /// `[decode, 0, 0, 0]` for the first level: 1 when the source holds
    /// sRGB-encoded values (a non-sRGB output, which the tonemap encodes).
    uniform: wgpu::Buffer,
    /// Zeros: the later levels, already linear.
    off: wgpu::Buffer,
    /// The overlay's sprite params for the pyramid: `[0, 0, 0, LEVELS]`,
    /// the level count the glass shader clamps its level of detail to.
    pyramid_params: wgpu::Buffer,
}

/// The textures of one output size.
pub(crate) struct GlassTargets {
    pub size: (u32, u32),
    pub source_view: wgpu::TextureView,
    levels: Vec<wgpu::TextureView>,
    /// The pyramid for the overlay's glass runs (the sprite layout).
    pub overlay_group: wgpu::BindGroup,
    blit_group: wgpu::BindGroup,
    down_groups: Vec<wgpu::BindGroup>,
}

impl GlassPasses {
    pub fn new(device: &wgpu::Device, output_format: wgpu::TextureFormat) -> GlassPasses {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("glass"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
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
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("glass"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("glass"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/glass.wgsl").into()),
        });
        let down = crate::post::fullscreen_pipeline(
            device,
            "glass down",
            &module,
            "fs_down",
            &pl,
            PYRAMID_FORMAT,
            None,
        );
        let blit = crate::post::fullscreen_pipeline(
            device,
            "glass blit",
            &module,
            "fs_blit",
            &pl,
            output_format,
            None,
        );
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("glass"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("glass"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let off = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("glass linear"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let pyramid_params = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("glass pyramid params"),
                contents: bytemuck::bytes_of(&[0.0f32, 0.0, 0.0, LEVELS as f32]),
                usage: wgpu::BufferUsages::UNIFORM,
            },
        );
        GlassPasses {
            layout,
            down,
            blit,
            sampler,
            uniform,
            off,
            pyramid_params,
        }
    }

    /// The textures for an output of `width`×`height` in `format`.
    pub fn targets(
        &self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        sprite_layout: &wgpu::BindGroupLayout,
    ) -> GlassTargets {
        let source = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("glass source"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let source_view = source.create_view(&Default::default());
        let pyramid = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("glass pyramid"),
            size: wgpu::Extent3d {
                width: (width / 2).max(1),
                height: (height / 2).max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: LEVELS,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: PYRAMID_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let pyramid_view = pyramid.create_view(&Default::default());
        let levels: Vec<wgpu::TextureView> = (0..LEVELS)
            .map(|k| {
                pyramid.create_view(&wgpu::TextureViewDescriptor {
                    label: Some("glass level"),
                    base_mip_level: k,
                    mip_level_count: Some(1),
                    ..Default::default()
                })
            })
            .collect();
        let group = |view: &wgpu::TextureView, params: &wgpu::Buffer| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("glass"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: params.as_entire_binding(),
                    },
                ],
            })
        };
        let blit_group = group(&source_view, &self.off);
        let overlay_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("glass pyramid"),
            layout: sprite_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&pyramid_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.pyramid_params.as_entire_binding(),
                },
            ],
        });
        // Level 0 from the source, level k from level k - 1.
        let mut down_groups = vec![group(&source_view, &self.uniform)];
        for k in 1..LEVELS as usize {
            down_groups.push(group(&levels[k - 1], &self.off));
        }
        GlassTargets {
            size: (width, height),
            source_view,
            levels,
            overlay_group,
            blit_group,
            down_groups,
        }
    }

    /// Copies the source to the output and writes the pyramid's levels.
    pub fn encode(
        &self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        targets: &GlassTargets,
        output: &wgpu::TextureView,
        decode: bool,
    ) -> u32 {
        queue.write_buffer(
            &self.uniform,
            0,
            bytemuck::cast_slice(&[f32::from(u8::from(decode)), 0.0, 0.0, 0.0]),
        );
        fullscreen_pass(
            encoder,
            "glass blit",
            output,
            wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            &self.blit,
            &targets.blit_group,
        );
        for (k, view) in targets.levels.iter().enumerate() {
            fullscreen_pass(
                encoder,
                "glass down",
                view,
                wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                &self.down,
                &targets.down_groups[k],
            );
        }
        1 + LEVELS
    }
}

/// The pyramid holds linear light (decoded from the source when it is
/// encoded) at half precision.
pub(crate) const PYRAMID_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
