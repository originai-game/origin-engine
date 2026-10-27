//! Glass: the frame under the overlay, blurred, for the overlay's glass
//! panels ([`crate::scene::Glass`]). Only when an overlay has glass: the
//! tonemap writes into `source` instead of the output, a blit copies it to
//! the output unchanged, and a chain of downsamples writes the mip levels
//! of `pyramid` (level k at 1/2^(k+1) of the output, each a 13-tap blur of
//! the level above). A glass panel samples the pyramid at its screen
//! position, at the level its blur radius asks for (trilinear between
//! levels), tints it and masks it to its rounded rectangle. A frame
//! without glass pays nothing.
//!
//! Glass over overlay content: the overlay draws in its sort order (layer,
//! then sprites, text and glass), so a glass run lies over every run before
//! it. When overlay content comes before a glass run ([`rebuilds`]), the
//! overlay draws into `source` (its sRGB view when the frame blends
//! through one) instead of the output, the pyramid is written again from
//! `source` before that glass run, and `source` is copied to the output
//! once the overlay is done: a glass panel blurs the scene and the overlay
//! layers below it. Each rebuild costs one pyramid (about a third of the
//! output's pixels written); a frame whose glass lies under all of the
//! overlay takes the one-pyramid path as before.

use crate::post::fullscreen_pass;

/// Mip levels of the pyramid: level 5 is 1/64 of the output, a blur of
/// about 64 device pixels.
pub(crate) const LEVELS: u32 = 6;

pub(crate) struct GlassPasses {
    layout: wgpu::BindGroupLayout,
    down: crate::deferred::Deferred<wgpu::RenderPipeline>,
    blit: crate::deferred::Deferred<wgpu::RenderPipeline>,
    sampler: wgpu::Sampler,
    /// `[decode, 0, width, height]` for the first level: decode 1 when the
    /// source holds sRGB-encoded values (a non-sRGB output, which the
    /// tonemap encodes); the source's size (glass.wgsl's `GlassParams`).
    uniform: wgpu::Buffer,
    /// Zeros: the blit's.
    off: wgpu::Buffer,
    /// The overlay's sprite params for the pyramid: `[0, 0, 0, LEVELS]`,
    /// the level count the glass shader clamps its level of detail to.
    pyramid_params: wgpu::Buffer,
}

/// Where the pyramid is (re)built for an overlay's sorted runs: before
/// each glass run that has overlay content drawn since the last build (the
/// first build is before the first glass run). Empty when the overlay has
/// no glass; `[i]` with nothing drawn before run `i` is the scene-only case.
pub(crate) fn rebuilds(glass: &[bool]) -> Vec<usize> {
    let mut out = Vec::new();
    let mut drawn = false;
    for (i, &is_glass) in glass.iter().enumerate() {
        if is_glass {
            if out.is_empty() || drawn {
                out.push(i);
            }
            drawn = false;
        } else {
            drawn = true;
        }
    }
    out
}

/// The textures of one output size.
pub(crate) struct GlassTargets {
    pub size: (u32, u32),
    pub source_view: wgpu::TextureView,
    /// `source` through the overlay's sRGB view format, when the output
    /// blends through one (the overlay draws into `source` when glass lies
    /// over overlay content).
    pub source_linear_view: Option<wgpu::TextureView>,
    levels: Vec<wgpu::TextureView>,
    /// The pyramid for the overlay's glass runs (the sprite layout).
    pub overlay_group: wgpu::BindGroup,
    blit_group: wgpu::BindGroup,
    down_groups: Vec<wgpu::BindGroup>,
}

impl GlassPasses {
    /// Makes one of its pipelines no frame used yet (Renderer::warm).
    pub(crate) fn warm(&self) -> bool {
        crate::deferred::warm_one(&[&self.down, &self.blit])
    }

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
        let module = crate::deferred::module(device, "glass", GLASS_WGSL.into());
        let down = crate::post::deferred_fullscreen(
            device,
            "glass down",
            &module,
            "fs_down",
            &pl,
            PYRAMID_FORMAT,
            None,
        );
        let blit = crate::post::deferred_fullscreen(
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

    /// The textures for an output of `width`×`height` in `format` (with a
    /// `linear` view format: the sRGB one the overlay blends through).
    pub fn targets(
        &self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        linear: Option<wgpu::TextureFormat>,
        width: u32,
        height: u32,
        sprite_layout: &wgpu::BindGroupLayout,
    ) -> GlassTargets {
        let view_formats: Vec<wgpu::TextureFormat> = linear.into_iter().collect();
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
            view_formats: &view_formats,
        });
        let source_view = source.create_view(&Default::default());
        let source_linear_view = linear.map(|f| {
            source.create_view(&wgpu::TextureViewDescriptor {
                label: Some("glass source (sRGB view)"),
                format: Some(f),
                ..Default::default()
            })
        });
        // Half the output, but never too small for LEVELS mips (an output
        // under 64 pixels on its longer side): the levels are sampled by uv,
        // so a larger pyramid only changes how much a tiny output blurs.
        let (mut pw, mut ph) = ((width / 2).max(1), (height / 2).max(1));
        let least = 1 << (LEVELS - 1);
        if pw.max(ph) < least {
            (pw, ph) = (pw.max(least), ph.max(least));
        }
        let pyramid = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("glass pyramid"),
            size: wgpu::Extent3d {
                width: pw,
                height: ph,
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
        // Level 0 from the source, level k from level k - 1, each with the
        // size of the level it reads.
        let mut down_groups = vec![group(&source_view, &self.uniform)];
        for k in 1..LEVELS as usize {
            let params = wgpu::util::DeviceExt::create_buffer_init(
                device,
                &wgpu::util::BufferInitDescriptor {
                    label: Some("glass level params"),
                    contents: bytemuck::bytes_of(&[
                        0.0f32,
                        0.0,
                        (pw >> (k - 1)).max(1) as f32,
                        (ph >> (k - 1)).max(1) as f32,
                    ]),
                    usage: wgpu::BufferUsages::UNIFORM,
                },
            );
            down_groups.push(group(&levels[k - 1], &params));
        }
        GlassTargets {
            size: (width, height),
            source_view,
            source_linear_view,
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
        encoder: &mut crate::Encoder,
        targets: &GlassTargets,
        output: &wgpu::TextureView,
        decode: bool,
    ) -> u32 {
        self.blit(encoder, targets, output) + self.pyramid(queue, encoder, targets, decode)
    }

    /// Copies the source to the output, unchanged.
    pub fn blit(
        &self,
        encoder: &mut crate::Encoder,
        targets: &GlassTargets,
        output: &wgpu::TextureView,
    ) -> u32 {
        fullscreen_pass(
            encoder,
            "glass blit",
            output,
            wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            &self.blit,
            &targets.blit_group,
        );
        1
    }

    /// Writes the pyramid's levels from the source as it is now (`decode`:
    /// the source holds sRGB-encoded values).
    pub fn pyramid(
        &self,
        queue: &wgpu::Queue,
        encoder: &mut crate::Encoder,
        targets: &GlassTargets,
        decode: bool,
    ) -> u32 {
        queue.write_buffer(
            &self.uniform,
            0,
            bytemuck::cast_slice(&[
                f32::from(u8::from(decode)),
                0.0,
                targets.size.0 as f32,
                targets.size.1 as f32,
            ]),
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
        LEVELS
    }
}

/// The pyramid holds linear light (decoded from the source when it is
/// encoded) at half precision.
pub(crate) const PYRAMID_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// The glass passes' module (`crate::shaders` validates it).
pub(crate) const GLASS_WGSL: &str = wgsl!("glass.wgsl");

#[cfg(test)]
mod tests {
    use super::rebuilds;

    #[test]
    fn the_pyramid_is_rebuilt_before_glass_with_overlay_under_it() {
        // No glass: no pyramid.
        assert!(rebuilds(&[false, false]).is_empty());
        // Glass under everything: one pyramid, of the scene.
        assert_eq!(rebuilds(&[true, false, false]), [0]);
        assert_eq!(rebuilds(&[true, true, false]), [0]);
        // Content under the glass: built after it.
        assert_eq!(rebuilds(&[false, true, false]), [1]);
        // Glass, content, glass: built again for the second.
        assert_eq!(rebuilds(&[true, false, true, false]), [0, 2]);
        // Two glass runs with nothing between share a pyramid.
        assert_eq!(rebuilds(&[false, true, true, false]), [1]);
    }
}
