//! The surface's display output: which gamut and range it presents
//! ([`DisplayOutput`]) and the last pass that carries the frame there. The
//! scene and the UI compose in linear sRGB relative to SDR white (1.0),
//! where a wide UI colour is a negative channel and a highlight a value
//! above 1; this pass limits the frame to Display P3 and the headroom and
//! writes it in the surface's colour space.

use crate::{DisplayOutput, RenderDesc, post};
use wgpu::{SurfaceColorSpace as Space, TextureFormat as Format};

/// The surface format and colour space for `output` on a surface with
/// `caps`, `None` for plain sRGB at standard range (the 8-bit path, no
/// transfer pass). `headroom` is the display's highlight multiplier where
/// the platform reports one: 1 (an SDR display) keeps standard range.
pub(crate) fn choose(
    caps: &wgpu::SurfaceCapabilities,
    output: DisplayOutput,
    headroom: Option<f32>,
) -> Option<(Format, Space)> {
    let offered = |format: Format, space: Space| {
        caps.color_spaces(format)
            .contains(space.to_color_spaces().expect("a named colour space"))
            .then_some((format, space))
    };
    let extended = output == DisplayOutput::Best && headroom.is_none_or(|h| h > 1.0);
    let float = Format::Rgba16Float;
    let eight = caps
        .formats
        .iter()
        .copied()
        .filter(|f| {
            matches!(
                f.remove_srgb_suffix(),
                Format::Bgra8Unorm | Format::Rgba8Unorm
            )
        })
        .collect::<Vec<_>>();
    match output {
        DisplayOutput::Srgb => None,
        DisplayOutput::Best | DisplayOutput::Wide => extended
            .then(|| {
                // scRGB carries P3 as negative channels and needs no encoding.
                [
                    Space::ExtendedSrgbLinear,
                    Space::ExtendedDisplayP3,
                    Space::ExtendedSrgb,
                ]
                .into_iter()
                .find_map(|s| offered(float, s))
            })
            .flatten()
            .or_else(|| offered(float, Space::DisplayP3))
            .or_else(|| eight.iter().find_map(|&f| offered(f, Space::DisplayP3))),
    }
}

/// Whether `space` presents colours outside sRGB.
pub(crate) fn wide(space: Space) -> bool {
    matches!(
        space,
        Space::ExtendedSrgbLinear
            | Space::ExtendedSrgb
            | Space::ExtendedDisplayP3
            | Space::DisplayP3
    )
}

/// The highlight multiplier the tone curves end at: `peak_nits` (1000
/// left out) over `paper_white_nits` (203), no more than the display's
/// reported headroom; 1 at standard range.
pub(crate) fn headroom(desc: &RenderDesc, space: Space, display: Option<f32>) -> f32 {
    if !space.is_hdr() {
        return 1.0;
    }
    let peak = if desc.peak_nits > 0.0 {
        desc.peak_nits.clamp(80.0, 10000.0)
    } else {
        1000.0
    };
    let wanted = peak / paper_white(desc);
    display.map_or(wanted, |d| wanted.min(d)).max(1.0)
}

/// The tone curves' HDR settings (`Post.hdr`, tonemap.wgsl's
/// `tone_curve_hdr`) for `headroom`: x the headroom, y and z the input
/// scales that keep ACES's and AgX's middle grey where SDR puts it while
/// their shoulders end at the headroom instead of SDR white. Zero (the SDR
/// curves) at standard range.
pub(crate) fn curve(space: Space, headroom: f32) -> [f32; 4] {
    if !space.is_hdr() {
        return [0.0; 4];
    }
    let scale = |f: fn(f32) -> f32| {
        // f(0.18 k) * headroom = f(0.18): bisect k, f rises monotonically.
        let target = f(GREY) / headroom;
        let (mut lo, mut hi) = (0.0f32, 1.0f32);
        for _ in 0..40 {
            let mid = 0.5 * (lo + hi);
            if f(GREY * mid) < target {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    };
    let scale_agx = |headroom: f32| {
        let target = agx_ev(GREY, 4.026069) / headroom;
        let max_ev = 4.026069 + headroom.max(1.0).log2();
        let (mut lo, mut hi) = (0.0f32, 4.0f32);
        for _ in 0..40 {
            let mid = 0.5 * (lo + hi);
            if agx_ev(GREY * mid, max_ev) < target {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    };
    [headroom, scale(aces), scale_agx(headroom), 0.0]
}

const GREY: f32 = 0.18;

/// tonemap.wgsl's `aces` on a grey.
fn aces(x: f32) -> f32 {
    ((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14)).clamp(0.0, 1.0)
}

/// tonemap.wgsl's `agx` on a grey with highlight ceiling `max_ev`.
fn agx_ev(x: f32, max_ev: f32) -> f32 {
    let min_ev = -12.47393f32;
    let c = ((x.max(1e-10).log2() - min_ev) / (max_ev - min_ev)).clamp(0.0, 1.0);
    let (x2, x4) = (c * c, c * c * c * c);
    let curve =
        15.5 * x4 * x2 - 40.14 * x4 * c + 31.96 * x4 - 6.868 * x2 * c + 0.4298 * x2 + 0.1191 * c
            - 0.00232;
    curve.max(0.0).powf(2.2).clamp(0.0, 1.0)
}

fn paper_white(desc: &RenderDesc) -> f32 {
    if desc.paper_white_nits > 0.0 {
        desc.paper_white_nits.clamp(80.0, 1000.0)
    } else {
        203.0
    }
}

/// The display as the renderer reads it: the surface's report, with the
/// host's word on its dynamic range ([`RenderDesc::display_high_dynamic_range`])
/// where the surface reports none (the web's Worker canvas).
pub(crate) fn display_info(
    mut info: wgpu::DisplayHdrInfo,
    high: Option<bool>,
) -> wgpu::DisplayHdrInfo {
    if let Some(high) = high {
        let coarse = info.coarse.get_or_insert_default();
        coarse.high_dynamic_range.get_or_insert(high);
    }
    info
}

/// The display's headroom as the platform reports it: Apple's potential
/// EDR headroom (its current one stays 1 until EDR content is shown), else
/// Windows' peak over SDR white; `None` where unknown (the web).
pub(crate) fn display_headroom(info: &wgpu::DisplayHdrInfo) -> Option<f32> {
    info.headroom
        .and_then(|h| h.potential)
        .filter(|h| h.is_finite() && *h >= 1.0)
        .or_else(|| info.tone_map_headroom())
}

pub(crate) const TRANSFER: &str = wgsl!("display_transfer.wgsl");

pub(crate) struct Output {
    /// The tone curves' HDR settings ([`curve`]) at the whole headroom.
    pub curve: [f32; 4],
    space: Space,
    /// The display's highlight multiplier ([`headroom`]).
    headroom: f32,
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
        (format, space): (Format, Space),
        headroom: f32,
        backend: wgpu::Backend,
    ) -> Self {
        // DXGI/Vulkan scRGB fixes 1 at 80 nits. Apple EDR and web use the OS's
        // SDR reference white, whose absolute luminance they do not expose.
        let scale = if space == Space::ExtendedSrgbLinear
            && matches!(backend, wgpu::Backend::Dx12 | wgpu::Backend::Vulkan)
        {
            paper_white(desc) / 80.0
        } else {
            1.0
        };
        let encode = space != Space::ExtendedSrgbLinear && !format.is_srgb();
        let p3 = matches!(space, Space::DisplayP3 | Space::ExtendedDisplayP3);
        let uniform = crate::buffer_init(
            device,
            queue,
            &wgpu::util::BufferInitDescriptor {
                label: Some("display output"),
                contents: bytemuck::bytes_of(&[scale, f32::from(encode), headroom, f32::from(p3)]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        );
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("display output"),
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
            label: Some("display output"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let shader = crate::gpu::shader_module(
            device,
            wgpu::ShaderModuleDescriptor {
                label: Some("display output"),
                source: wgpu::ShaderSource::Wgsl(post::fullscreen_source(TRANSFER).into()),
            },
        );
        let pipeline =
            post::fullscreen_pipeline(device, "display output", &shader, "fs", &pl, format, None);
        Self {
            curve: curve(space, headroom),
            space,
            headroom,
            layout,
            pipeline,
            uniform,
            sampler: device.create_sampler(&wgpu::SamplerDescriptor::default()),
        }
    }

    /// The brightest the frame shows under `limit` (CSS
    /// `dynamic-range-limit`), as a multiple of SDR white: the transfer
    /// clamps there.
    pub fn peak(&self, limit: og_draw::RangeLimit) -> f32 {
        limit.peak(self.headroom)
    }

    /// The tone curves' HDR settings under `limit`: their shoulders end at
    /// its [`Self::peak`], so a limited frame's highlights roll off below
    /// it rather than clip at it.
    pub fn curve(&self, limit: og_draw::RangeLimit) -> [f32; 4] {
        if limit == og_draw::RangeLimit::default() {
            return self.curve;
        }
        curve(self.space, self.peak(limit))
    }

    pub fn encode(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut crate::Encoder,
        (source, output): (&wgpu::TextureView, &wgpu::TextureView),
        limit: og_draw::RangeLimit,
    ) {
        // The uniform's z: the headroom, or less under `limit`.
        crate::traffic::write_buffer(
            queue,
            &self.uniform,
            8,
            bytemuck::bytes_of(&self.peak(limit)),
        );
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("display output"),
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
            "display output",
            output,
            wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            Some(&self.pipeline),
            &group,
        );
    }
}
