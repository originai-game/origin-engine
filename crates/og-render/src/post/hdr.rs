//! Explicit HDR surface selection and final display transfer. The scene
//! and overlay compose in linear sRGB, relative to paper white.

use crate::{RenderDesc, post};

pub(crate) fn surface_space(caps: &wgpu::SurfaceCapabilities) -> Option<wgpu::SurfaceColorSpace> {
    let spaces = caps.color_spaces(wgpu::TextureFormat::Rgba16Float);
    [
        wgpu::SurfaceColorSpace::ExtendedSrgbLinear,
        wgpu::SurfaceColorSpace::ExtendedSrgb,
    ]
    .into_iter()
    .find(|s| spaces.contains(s.to_color_spaces().expect("explicit color space")))
}

/// LPM's contrast=1, shoulder=1 tone-scale calibration, solving the two
/// anchors directly: scene white 1 -> paper white; scene maximum 256 -> peak.
pub(crate) fn curve(desc: &RenderDesc) -> [f32; 4] {
    let white = if desc.paper_white_nits > 0.0 {
        desc.paper_white_nits.clamp(80.0, 1000.0)
    } else {
        203.0
    };
    let peak = if desc.peak_nits > 0.0 {
        desc.peak_nits.clamp(80.0, 10000.0)
    } else {
        1000.0
    }
    .max(white);
    let headroom = peak / white;
    let scale = (256.0 - headroom) / 255.0;
    [headroom, scale, headroom - scale, white]
}

pub(crate) const TRANSFER: &str = wgsl!("hdr_transfer.wgsl");

pub(crate) struct Output {
    pub curve: [f32; 4],
    layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    uniform: wgpu::Buffer,
}

impl Output {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        desc: &RenderDesc,
        space: wgpu::SurfaceColorSpace,
        backend: wgpu::Backend,
    ) -> Self {
        let curve = curve(desc);
        // DXGI/Vulkan scRGB fixes 1 at 80 nits. Apple EDR and web use the OS's
        // SDR reference white, whose absolute luminance they do not expose.
        let scale = if matches!(backend, wgpu::Backend::Dx12 | wgpu::Backend::Vulkan) {
            curve[3] / 80.0
        } else {
            1.0
        };
        let uniform = crate::buffer_init(
            device,
            queue,
            &wgpu::util::BufferInitDescriptor {
                label: Some("hdr output"),
                contents: bytemuck::bytes_of(&[
                    scale,
                    f32::from(space == wgpu::SurfaceColorSpace::ExtendedSrgb),
                    curve[0],
                    0.0,
                ]),
                usage: wgpu::BufferUsages::UNIFORM,
            },
        );
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("hdr output"),
            entries: &[
                post::layout_entry(0, post::float_texture(true)),
                post::layout_entry(
                    1,
                    wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                ),
                post::layout_entry(2, post::UNIFORM),
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("hdr output"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("hdr output"),
            source: wgpu::ShaderSource::Wgsl(post::fullscreen_source(TRANSFER).into()),
        });
        let pipeline = post::fullscreen_pipeline(
            device,
            "hdr output",
            &shader,
            "fs",
            &pl,
            wgpu::TextureFormat::Rgba16Float,
            None,
        );
        Self {
            curve,
            layout,
            pipeline,
            uniform,
            sampler: device.create_sampler(&wgpu::SamplerDescriptor::default()),
        }
    }

    pub fn encode(
        &self,
        device: &wgpu::Device,
        encoder: &mut crate::Encoder,
        source: &wgpu::TextureView,
        output: &wgpu::TextureView,
    ) {
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("hdr output"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(source),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.uniform.as_entire_binding(),
                },
            ],
        });
        post::fullscreen_pass(
            encoder,
            "hdr output",
            output,
            wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            &self.pipeline,
            &group,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn float_format_alone_is_not_hdr_and_web_needs_encoded_extended_srgb() {
        let caps = |format, color_spaces| wgpu::SurfaceCapabilities {
            format_capabilities: vec![wgpu::SurfaceFormatCapabilities {
                format,
                color_spaces,
            }],
            ..Default::default()
        };
        use wgpu::{SurfaceColorSpace as S, SurfaceColorSpaces as F, TextureFormat as T};
        assert_eq!(surface_space(&caps(T::Rgba16Float, F::SRGB)), None);
        assert_eq!(surface_space(&caps(T::Rgba8Unorm, F::EXTENDED_SRGB)), None);
        assert_eq!(
            surface_space(&caps(T::Rgba16Float, F::EXTENDED_SRGB)),
            Some(S::ExtendedSrgb)
        );
        assert_eq!(
            surface_space(&caps(
                T::Rgba16Float,
                F::EXTENDED_SRGB | F::EXTENDED_SRGB_LINEAR
            )),
            Some(S::ExtendedSrgbLinear)
        );
        assert_eq!(surface_space(&wgpu::SurfaceCapabilities::default()), None);
    }

    #[test]
    fn lpm_white_and_peak_are_distinct_and_calibrated() {
        for (white, peak) in [(203.0, 1000.0), (100.0, 4000.0), (800.0, 1000.0)] {
            let c = curve(&RenderDesc {
                paper_white_nits: white,
                peak_nits: peak,
                ..Default::default()
            });
            let map = |x: f32| x / (x * c[1] + c[2]) * c[0] * white;
            assert_eq!(map(0.0), 0.0);
            assert!((map(1.0) - white).abs() < 0.001);
            assert!((map(256.0) - peak).abs() < 0.001);
            assert!(map(8.0) > white && map(8.0) < peak);
        }
        assert_eq!(
            curve(&RenderDesc::default()),
            curve(&RenderDesc {
                paper_white_nits: 203.0,
                peak_nits: 1000.0,
                ..Default::default()
            })
        );
        assert_eq!(
            curve(&RenderDesc {
                paper_white_nits: 500.0,
                peak_nits: 80.0,
                ..Default::default()
            })[0],
            1.0
        );
    }
}
