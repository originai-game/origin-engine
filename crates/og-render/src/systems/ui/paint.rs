//! The paint pass: og-ui's paint arena ([`og_draw::paint::Paint`]) drawn
//! with one uber shader (`shaders/ui_paint.wgsl`) and one pipeline, made
//! with the renderer: no UI pipeline is made after boot. The pass blends
//! sRGB-encoded values, as CSS does and as its components are designed
//! against: it draws in the output's plain format ([`encoded_format`]),
//! through the output itself, its plain view, or else a layer of its own
//! drawn over the output once ([`PaintRenderer::composite`]), blended in the
//! shader over a copy of what is behind it where the renderer owns the
//! target. The arena is
//! uploaded only when its generation, the target or a font's pages change;
//! a glyph quad is resolved against its font's pages then (its first page
//! with the glyph), and the quads draw in runs of one page each, in the
//! arena's order, where the UI's layer lies among the overlay's runs
//! ([`crate::overlay::OverlayFrame::ui_at`]).
//!
//! A raster quad ([`PaintKind::Raster`]) reads the glyph atlas
//! ([`og_draw::glyphs::Atlas`]), whose pages are textures here: one made
//! when the atlas makes a page, and only the plots whose pixels changed
//! written each frame.
//!
//! A group ([`og_draw::paint::PaintGroup`]) is drawn offscreen: a filter
//! group's quads draw into a layer of its own (innermost first, a nested
//! group's composite in its parent's), [`FilterRunner`] runs its blur and
//! drop shadows over it, and its composite quad ([`PaintKind::Group`])
//! reads the result where it lands. A backdrop group ends the pass it is
//! drawn in where it stands: what is under it (the target as drawn so
//! far, or its group's layer) is captured, blurred and colour-filtered,
//! and its composite draws that under the element. Every layer and
//! filter pass is in the encoded values the paint blends in, as CSS
//! filters them.

use super::filter::{
    Capture, FilterRunner, Inputs, LayerInput, Source, UiFilter, UiFilterComposite, UiFilterInput,
    UiFilterOp, UiFilterPrimitive,
};
use crate::TextureHandle;
use crate::gpu::data_texture::{DataTexture, data_entry};
use bytemuck::{Pod, Zeroable};
use og_draw::glyphs::{Atlas, GlyphTier, PLOT};
use og_draw::paint::{Paint, PaintEffect, PaintKind};
use std::cell::RefCell;
use std::ops::Range;

/// `struct PaintGpu` in ui_paint.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub(crate) struct PaintGpu {
    rect: [f32; 4],
    clip: [f32; 4],
    color: [f32; 4],
    uv: [f32; 4],
    r: [f32; 4],
    p: [f32; 4],
    k: [u32; 4],
    w: [f32; 4],
    /// x: its colour matrix's first row in `gradients` + 1, y: its
    /// rounded clip's (0 none).
    f: [u32; 4],
}

/// The largest side of a group's layer, device px (WebGPU's least
/// `maxTextureDimension2D`).
const LAYER_SIDE: i32 = 8192;

// ui_paint.wgsl `paint_quad` reads a quad as nine texels.
const _: () = assert!(std::mem::size_of::<PaintGpu>() == 9 * 16);

/// What a run binds as group 1: the white sprite (nothing samples it), a
/// font page, a texture's sprite group, or a page of a tier of the glyph
/// atlas (the paint pass's own).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PaintBind {
    White,
    Texture(TextureHandle),
    Glyphs(u8, u16),
    /// A group's filtered layer or capture, by its index in this upload's
    /// layers (the pass's own).
    Group(u16),
}

/// What a draw list does in order: runs (of [`PaintRenderer::runs`]),
/// and a backdrop's capture of what the list drew so far.
#[derive(Clone, Debug, PartialEq)]
enum Step {
    Runs(Range<usize>),
    Backdrop(usize),
}

/// A group this upload ([`og_draw::paint::PaintGroup`]): its rectangle
/// in device px (x0, y0, width, height; a filter group's layer, a
/// backdrop's capture), and for a filter group what draws into it.
struct GroupLayer {
    backdrop: bool,
    rect: [i32; 4],
    /// The level a filter group's layer is drawn at: its rect halved that
    /// many times ([`super::filter::source_level`]).
    level: u32,
    steps: Vec<Step>,
    /// Its filter over what it filters; a backdrop's also over the pass's
    /// own layer drawn over the copy of the output under it.
    filter: UiFilter,
    filter_over: Option<UiFilter>,
}

/// A filter group's layer target, kept between frames.
struct LayerTarget {
    size: (u32, u32),
    view: wgpu::TextureView,
}

/// The work of a frame's groups: the filter passes, each group's
/// filtered result as group 1 of its composite (`None` while it could
/// not draw), and what binds a result (by its view).
struct Fx {
    filters: FilterRunner,
    results: Vec<Option<wgpu::BindGroup>>,
    binds: Vec<(wgpu::TextureView, wgpu::BindGroup)>,
}

/// What the main draw's backdrops read: the target itself (its view,
/// readable as a texture, of `size` device px), or the pass's own layer
/// drawn over its copy of the output when it made one ([`Under::Layer`]
/// with `true`), or nothing (a backdrop draws nothing).
pub(crate) enum Under<'a> {
    Target(&'a wgpu::TextureView, (u32, u32)),
    Layer(bool),
    None,
}

/// What a group's draw reads from the renderer: a run's group 1 (a font
/// page or texture), and the sprites' layout and sampler that bind a
/// result to a composite.
pub(crate) struct GroupCx<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub group1: &'a dyn Fn(PaintBind) -> &'a wgpu::BindGroup,
    pub layout: &'a wgpu::BindGroupLayout,
    pub sampler: &'a wgpu::Sampler,
}

/// A page of the glyph atlas on the GPU.
struct GlyphPage {
    texture: wgpu::Texture,
    group: wgpu::BindGroup,
    /// What binds it for a world or overlay text's glyphs: a field page
    /// with its range and side (the sprite shaders' `msdf`), a colour page
    /// as a plain sprite texture; none for a coverage page.
    drawn: Option<wgpu::BindGroup>,
    /// Each plot's revision as last written.
    written: Vec<u32>,
}

/// Consecutive instances drawn with one group 1 and one blend mode bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PaintRun {
    pub bind: PaintBind,
    pub blend: u8,
    pub first: u32,
    pub count: u32,
}

/// What an upload was made from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Uploaded {
    generation: u64,
    revisions: u64,
    width: u32,
    height: u32,
    /// The glyph atlas's pages on the GPU, by tier.
    glyph_pages: [u32; 3],
}

/// The pass's own target for an output it cannot blend in.
struct Layer {
    size: (u32, u32),
    view: wgpu::TextureView,
    /// The layer as group 1 of the composite.
    group: wgpu::BindGroup,
    /// What the layer is drawn over, copied from the target in the
    /// output's format before the composite, and as its group 2.
    behind: wgpu::Texture,
    behind_view: wgpu::TextureView,
    behind_group: wgpu::BindGroup,
}

/// The format the paint pass draws in for an output in `output`: one whose
/// stored value is the encoded one the shader returns. An sRGB output's
/// plain format, and eight bits for a float output.
pub(crate) fn encoded_format(output: wgpu::TextureFormat) -> wgpu::TextureFormat {
    if output == crate::HDR_FORMAT {
        wgpu::TextureFormat::Rgba8Unorm
    } else {
        output.remove_srgb_suffix()
    }
}

/// The hardware blend state for [`og_draw::paint::PaintBlend`] (`0` normal,
/// `1` add, `2` screen, `3` multiply). Screen and multiply pair with
/// `fs_paint`'s alpha-weighted output so `a = 0` is a no-op.
fn blend_state(mode: u8) -> wgpu::BlendState {
    let alpha = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    };
    match mode {
        1 => wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha,
        },
        2 => wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrc,
                operation: wgpu::BlendOperation::Add,
            },
            alpha,
        },
        3 => wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Dst,
                dst_factor: wgpu::BlendFactor::Zero,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        },
        _ => wgpu::BlendState::ALPHA_BLENDING,
    }
}

pub(crate) struct PaintRenderer {
    format: wgpu::TextureFormat,
    output_format: wgpu::TextureFormat,
    /// One pipeline per [`og_draw::paint::PaintBlend`] mode (`[0]` started at
    /// boot; `[1..=3]` started on first use).
    pipelines: [crate::deferred::Deferred<wgpu::RenderPipeline>; 4],
    /// Draws the layer over an output that is not in `format`: blended by
    /// the output, and over a copy of what is behind it, in encoded values.
    composite: Option<(
        crate::deferred::Deferred<wgpu::RenderPipeline>,
        crate::deferred::Deferred<wgpu::RenderPipeline>,
    )>,
    layer: Option<Layer>,
    layout: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
    /// Whether the uniform holds the preblend table yet (written once,
    /// behind the frame's floats).
    preblend: bool,
    /// The quads and the transforms, data textures (data_texture.rs) the
    /// vertex stage reads.
    quads: DataTexture,
    gradients: wgpu::Buffer,
    stops: wgpu::Buffer,
    segments: wgpu::Buffer,
    transforms: DataTexture,
    group: Option<wgpu::BindGroup>,
    uploaded: Option<Uploaded>,
    /// The icon table uploaded (its address and length).
    icons: (usize, usize),
    /// The glyph atlas's pages by tier, what binds one, and the atlas
    /// (its id) and revision they were last written from.
    glyph_pages: [Vec<GlyphPage>; 3],
    glyph_sides: [u32; 3],
    glyph_layout: wgpu::BindGroupLayout,
    glyph_sampler: wgpu::Sampler,
    glyphs_of: Option<(u64, u64)>,
    gpu: Vec<PaintGpu>,
    gradient_rows: Vec<[f32; 4]>,
    stop_rows: Vec<[f32; 4]>,
    /// Every draw list's runs: the main list's and each group's.
    pub runs: Vec<PaintRun>,
    /// Glyphs no page of their font has (yet).
    pub missing: u32,
    /// The pass in a group's layer (in the filter passes' format), per blend
    /// mode, asked for when a paint first has a group using that mode.
    layer_pipelines: [crate::deferred::Deferred<wgpu::RenderPipeline>; 4],
    /// This upload's groups, each group's (by its index in the paint),
    /// the main list, the filter groups' layers in the order they draw
    /// (innermost first), each quad's first instance and each instance's
    /// group 1 (`None`: it samples nothing) and blend mode.
    layers: Vec<GroupLayer>,
    layer_of: Vec<Option<usize>>,
    main: Vec<Step>,
    order: Vec<usize>,
    at: Vec<u32>,
    binds: Vec<Option<PaintBind>>,
    blends: Vec<u8>,
    used_blends: [bool; 4],
    /// A filter group's layer target, frame uniform and group 0, by layer.
    targets: Vec<Option<LayerTarget>>,
    frames: Vec<wgpu::Buffer>,
    layer_groups: Vec<Option<wgpu::BindGroup>>,
    /// What a filter pass binds that it does not read.
    blank: wgpu::TextureView,
    /// Device px a CSS px, as last uploaded.
    scale: f32,
    fx: RefCell<Fx>,
}

/// The frame's eight floats, then the preblend table.
const UNIFORM_SIZE: u64 = 32 + PREBLEND_TABLE as u64;

/// Skia's mask gamma tables (third_party/skia-mask-gamma), one row of 256
/// per luminance class.
const PREBLEND_TABLE: usize = 8 * 256;

/// Skia's text preblend with Chrome on Linux's gamma (1.2) and contrast
/// (0.2), ported from `SkTMaskGamma`: for each of eight ink luminances
/// (`sk_t_scale255<3>`), the coverage a glyph is drawn with for each
/// coverage it has, so that blending it as is gives the ink the weight
/// Skia's text has. Row `l >> 5` serves an ink of luminance `l`
/// (`computeLuminance` at gamma 1.2, in 8 bits).
fn preblend_table() -> [u8; PREBLEND_TABLE] {
    const GAMMA: f32 = 1.2;
    const CONTRAST: f32 = 0.2;
    let mut table = [0; PREBLEND_TABLE];
    for (row, lut) in table.chunks_mut(256).enumerate() {
        let base = (row as u32) << 5;
        let src = (base | base >> 3 | base >> 6) as f32 / 255.0;
        let lin_src = src.powf(GAMMA);
        let dst = 1.0 - src;
        let lin_dst = dst.powf(GAMMA);
        // Contrast tapers off to nothing as the ink nears white.
        let contrast = CONTRAST * lin_dst;
        for (i, out) in lut.iter_mut().enumerate() {
            let raw = i as f32 / 255.0;
            let srca = raw + (1.0 - raw) * contrast * raw;
            let result = if (src - dst).abs() < 1.0 / 256.0 {
                srca
            } else {
                let lin_out = lin_src * srca + (1.0 - srca) * lin_dst;
                (lin_out.powf(1.0 / GAMMA) - dst) / (src - dst)
            };
            *out = (255.0 * result + 0.5).floor().clamp(0.0, 255.0) as u8;
        }
    }
    table
}

/// Its module: the library every shader may call, then ui_paint.wgsl.
pub(crate) fn source() -> String {
    format!("{}{}", crate::LIBRARY_WGSL, wgsl!("ui_paint.wgsl"))
}

fn buffer(device: &wgpu::Device, label: &str, size: u64) -> wgpu::Buffer {
    crate::traffic::create_buffer(
        device,
        &wgpu::BufferDescriptor {
            label: Some(label),
            size: size.max(256),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        },
    )
}

/// Group `g`'s rectangle in device px at `scale` (x0, y0, width, height),
/// its composite being `q`: a filter group's layer, its composite's rect
/// rounded out; a backdrop's capture, its box on the screen (through its
/// transform) inside its clip, grown by its blur's reach (what the blur
/// at its edge reads), rounded out. `None` when it shows nothing.
fn group_rect(
    paint: &Paint,
    g: &og_draw::paint::PaintGroup,
    q: &og_draw::paint::PaintQuad,
    scale: f32,
) -> Option<[i32; 4]> {
    let [x, y, w, h] = q.rect;
    let [x0, y0, x1, y1] = if g.backdrop {
        let at = |xf: u32| {
            paint
                .transforms
                .get(xf as usize)
                .copied()
                .unwrap_or(og_draw::paint::IDENTITY)
        };
        let [a, b, c, d, e, f, _, _] = at(q.xf);
        let mut bound = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
        for [cx, cy] in [[x, y], [x + w, y], [x, y + h], [x + w, y + h]] {
            let (sx, sy) = (a * cx + c * cy + e, b * cx + d * cy + f);
            bound = [
                bound[0].min(sx),
                bound[1].min(sy),
                bound[2].max(sx),
                bound[3].max(sy),
            ];
        }
        let clip = q.clip;
        let [ga, gb, gc, gd, ..] = at(g.xf);
        let axis = ga.hypot(gb).max(gc.hypot(gd));
        let reach: f32 = paint.effects[g.first as usize..(g.first + g.count) as usize]
            .iter()
            .map(|f| match *f {
                PaintEffect::Blur(sigma) => 3.0 * sigma * axis,
                _ => 0.0,
            })
            .sum();
        [
            bound[0].max(clip[0]).max(0.0) - reach,
            bound[1].max(clip[1]).max(0.0) - reach,
            bound[2].min(clip[2]).min(paint.width) + reach,
            bound[3].min(clip[3]).min(paint.height) + reach,
        ]
    } else {
        [x, y, x + w, y + h]
    };
    let (dx0, dy0) = ((x0 * scale).floor(), (y0 * scale).floor());
    let (dx1, dy1) = ((x1 * scale).ceil(), (y1 * scale).ceil());
    if !(dx0 < dx1 && dy0 < dy1) {
        return None;
    }
    let (dw, dh) = (dx1 - dx0, dy1 - dy0);
    Some([
        dx0 as i32,
        dy0 as i32,
        (dw as i32).min(LAYER_SIDE),
        (dh as i32).min(LAYER_SIDE),
    ])
}

/// The filter that does `effects` in the space `space` (an element's
/// transform), on encoded, premultiplied values as the paint blends:
/// each step reads the last one's result (the first reads `from`); a
/// drop shadow is its input's alpha blurred, moved and flooded, under
/// its input. `matrices` are the paint's colour matrices.
fn filter_of(
    effects: &[PaintEffect],
    matrices: &[og_draw::paint::ColorMatrix],
    space: [f32; 8],
    mut primitives: Vec<UiFilterPrimitive>,
    from: UiFilterInput,
) -> UiFilter {
    let everywhere = [-1.0e9, -1.0e9, 2.0e9, 2.0e9];
    let mut input = from;
    let push = |primitives: &mut Vec<UiFilterPrimitive>, op, inputs| {
        primitives.push(UiFilterPrimitive {
            op,
            inputs,
            subregion: everywhere,
            srgb: false,
        });
        UiFilterInput::Result(primitives.len() as u32 - 1)
    };
    let source = UiFilterInput::Source;
    for f in effects {
        input = match *f {
            PaintEffect::Blur(sigma) => push(
                &mut primitives,
                UiFilterOp::Blur {
                    std_dev: [sigma; 2],
                },
                [input, source],
            ),
            PaintEffect::DropShadow {
                offset,
                sigma,
                color,
            } => {
                let [r, g, b, a] = color;
                let encoded = [r, g, b].map(og_core::color::linear_to_srgb);
                push(
                    &mut primitives,
                    UiFilterOp::DropShadow {
                        std_dev: [sigma; 2],
                        d: offset,
                        color: [encoded[0], encoded[1], encoded[2], a],
                    },
                    [input, input],
                )
            }
            PaintEffect::Matrix(k) => {
                let Some(m) = matrices.get(k as usize) else {
                    continue;
                };
                let mut rows = [0.0; 20];
                for r in 0..4 {
                    rows[r * 5..r * 5 + 4].copy_from_slice(&m[r]);
                    rows[r * 5 + 4] = m[4][r];
                }
                push(
                    &mut primitives,
                    UiFilterOp::ColorMatrix(rows),
                    [input, source],
                )
            }
        };
    }
    let [a, b, c, d, e, f, _, _] = space;
    UiFilter {
        primitives,
        space: [a, b, c, d, e, f],
    }
}

/// Writes `rows` to `buffer`, growing it (to the next power of two)
/// when they do not fit; whether it grew.
fn write<T: Pod>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    buffer: &mut wgpu::Buffer,
    label: &str,
    rows: &[T],
) -> bool {
    let bytes: &[u8] = bytemuck::cast_slice(rows);
    let grew = bytes.len() as u64 > buffer.size();
    if grew {
        *buffer = self::buffer(device, label, (bytes.len() as u64).next_power_of_two());
    }
    if !bytes.is_empty() {
        crate::traffic::write_buffer(queue, buffer, 0, bytes);
    }
    grew
}

impl PaintRenderer {
    /// Its pipelines, made now, and its buffers.
    pub fn new(
        device: &wgpu::Device,
        output_format: wgpu::TextureFormat,
        sprite_layout: &wgpu::BindGroupLayout,
    ) -> PaintRenderer {
        let entry = |binding, ty| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let storage = wgpu::BufferBindingType::Storage { read_only: true };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ui paint"),
            entries: &[
                entry(0, wgpu::BufferBindingType::Uniform),
                data_entry(1, wgpu::ShaderStages::VERTEX_FRAGMENT, true),
                entry(2, storage),
                entry(3, storage),
                entry(4, storage),
                data_entry(5, wgpu::ShaderStages::VERTEX, false),
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ui paint"),
            bind_group_layouts: &[Some(&layout), Some(sprite_layout)],
            immediate_size: 0,
        });
        let module = crate::gpu::shader_module(
            device,
            wgpu::ShaderModuleDescriptor {
                label: Some("ui paint"),
                source: wgpu::ShaderSource::Wgsl(source().into()),
            },
        );
        // Entries of the pipeline cache (gpu/deferred.rs), asked for now.
        let pipeline = |format,
                        pl: &wgpu::PipelineLayout,
                        blend,
                        vertex: &'static str,
                        fragment: &'static str| {
            let (device, pl, module) = (device.clone(), pl.clone(), module.clone());
            crate::deferred::Deferred::anywhere(move || {
                let device = &device;
                crate::traffic::create_render_pipeline(
                    device,
                    &wgpu::RenderPipelineDescriptor {
                        label: Some("ui paint"),
                        layout: Some(&pl),
                        vertex: wgpu::VertexState {
                            module: &module,
                            entry_point: Some(vertex),
                            compilation_options: Default::default(),
                            buffers: &[],
                        },
                        primitive: Default::default(),
                        depth_stencil: None,
                        multisample: Default::default(),
                        fragment: Some(wgpu::FragmentState {
                            module: &module,
                            entry_point: Some(fragment),
                            compilation_options: Default::default(),
                            targets: &[Some(wgpu::ColorTargetState {
                                format,
                                blend,
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                        }),
                        multiview_mask: None,
                        cache: crate::pipeline_cache::of(device).as_ref(),
                    },
                )
            })
        };
        let format = encoded_format(output_format);
        let blend = Some(wgpu::BlendState::ALPHA_BLENDING);
        // The composite over a copy of what is behind it: the same groups,
        // and the copy as group 2.
        let over_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ui paint over"),
            bind_group_layouts: &[Some(&layout), Some(sprite_layout), Some(sprite_layout)],
            immediate_size: 0,
        });
        let pipelines = std::array::from_fn(|m| {
            pipeline(
                format,
                &pl,
                Some(blend_state(m as u8)),
                "vs_paint",
                "fs_paint",
            )
        });
        crate::deferred::Start::start(&pipelines[0]);
        let composite = (format != output_format).then(|| {
            let (blended, behind) = (
                pipeline(output_format, &pl, blend, "vs_layer", "fs_layer"),
                pipeline(output_format, &over_pl, None, "vs_layer", "fs_layer_over"),
            );
            crate::deferred::Start::start(&blended);
            crate::deferred::Start::start(&behind);
            (blended, behind)
        });
        // A layer stores premultiplied values: what it draws over is
        // transparent, so blending as alpha leaves them so.
        let layer_pipelines = std::array::from_fn(|m| {
            pipeline(
                super::pass::LAYER_FORMAT,
                &pl,
                Some(blend_state(m as u8)),
                "vs_paint",
                "fs_paint",
            )
        });
        let blank = crate::target_texture(
            device,
            "ui paint blank",
            1,
            1,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::TEXTURE_BINDING,
        )
        .create_view(&Default::default());
        PaintRenderer {
            format,
            output_format,
            pipelines,
            composite,
            layer: None,
            layout,
            uniform: crate::traffic::create_buffer(
                device,
                &wgpu::BufferDescriptor {
                    label: Some("ui paint frame"),
                    size: UNIFORM_SIZE,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                },
            ),
            preblend: false,
            quads: DataTexture::new(
                device,
                "ui paint quads",
                wgpu::TextureFormat::Rgba32Uint,
                1,
                wgpu::TextureUsages::COPY_DST,
            ),
            gradients: buffer(device, "ui paint gradients", 0),
            stops: buffer(device, "ui paint stops", 0),
            segments: buffer(device, "ui paint icons", 0),
            transforms: DataTexture::new(
                device,
                "ui paint transforms",
                wgpu::TextureFormat::Rgba32Float,
                1,
                wgpu::TextureUsages::COPY_DST,
            ),
            group: None,
            uploaded: None,
            icons: (0, 0),
            glyph_pages: Default::default(),
            glyph_sides: [0; 3],
            glyph_layout: sprite_layout.clone(),
            // A raster drawn at its own size on whole pixels samples its
            // texels' centres, so filtering changes nothing there; a
            // distance field is filtered.
            glyph_sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("ui paint glyphs"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }),
            glyphs_of: None,
            gpu: Vec::new(),
            gradient_rows: Vec::new(),
            stop_rows: Vec::new(),
            runs: Vec::new(),
            missing: 0,
            layer_pipelines,
            layers: Vec::new(),
            layer_of: Vec::new(),
            main: Vec::new(),
            order: Vec::new(),
            at: Vec::new(),
            binds: Vec::new(),
            blends: Vec::new(),
            used_blends: [true, false, false, false],
            targets: Vec::new(),
            frames: Vec::new(),
            layer_groups: Vec::new(),
            blank,
            scale: 1.0,
            fx: RefCell::new(Fx {
                filters: FilterRunner::new(device),
                results: Vec::new(),
                binds: Vec::new(),
            }),
        }
    }

    /// Uploads `paint` for an output of `width`×`height` device pixels
    /// and the plots of `glyphs` whose pixels changed, when it or the
    /// target changed since the last upload, and the frame uniform.
    #[allow(clippy::too_many_arguments)]
    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        paint: &Paint,
        glyphs: &Atlas,
        texture: impl Fn(TextureHandle) -> bool,
        revisions: u64,
        (width, height): (u32, u32),
    ) {
        let scale = if paint.height > 0.0 {
            height as f32 / paint.height
        } else {
            1.0
        };
        let u: [f32; 8] = [
            width as f32,
            height as f32,
            scale,
            0.0,
            crate::overlay::dither_step(self.format),
            0.0,
            0.0,
            0.0,
        ];
        crate::traffic::write_buffer(queue, &self.uniform, 0, bytemuck::cast_slice(&u));
        self.scale = scale;
        if !self.preblend {
            self.preblend = true;
            crate::traffic::write_buffer(queue, &self.uniform, 32, &preblend_table());
        }
        self.upload_glyphs(device, queue, glyphs);
        let now = Uploaded {
            generation: paint.generation,
            revisions,
            width,
            height,
            glyph_pages: [0, 1, 2].map(|t: usize| self.glyph_pages[t].len() as u32),
        };
        if self.uploaded == Some(now) {
            return;
        }
        self.uploaded = Some(now);
        self.resolve(paint, texture, scale);
        for m in 1..4 {
            if self.used_blends[m] {
                crate::deferred::Start::start(&self.pipelines[m]);
            }
        }
        let mut grew = self.group.is_none();
        grew |= self.quads.grow_and_write(
            device,
            queue,
            "ui paint quads",
            bytemuck::cast_slice(&self.gpu),
        );
        self.gradient_rows.clear();
        // Three rows a gradient: its angle, stops and shape, its centre
        // and radii, and a mask's box (ui_paint.wgsl `gradients`).
        for g in &paint.gradients {
            self.gradient_rows.extend([
                [g.angle, g.first as f32, g.count as f32, g.shape as f32],
                [g.center[0], g.center[1], g.radii[0], g.radii[1]],
                g.area,
            ]);
        }
        // Then five rows a colour matrix (`PaintGpu::f`), then four a
        // rounded clip: its box, its radii, and the inverse of its
        // transform (linear part, then translation and whether it is
        // inverted), which takes a fragment's place on screen into it.
        for m in &paint.filters {
            self.gradient_rows.extend(m);
        }
        for r in &paint.rounds {
            let [a, b, c, d, e, f, _, _] = paint
                .transforms
                .get(r.xf as usize)
                .copied()
                .unwrap_or(og_draw::paint::IDENTITY);
            let det = a * d - b * c;
            // A transform with no inverse leaves nothing of what it clips.
            let [ia, ib, ic, id, ie, iff] = if det.abs() > 1e-12 {
                let (ia, ib, ic, id) = (d / det, -b / det, -c / det, a / det);
                [ia, ib, ic, id, -(ia * e + ic * f), -(ib * e + id * f)]
            } else {
                [0.0, 0.0, 0.0, 0.0, f32::MAX, f32::MAX]
            };
            self.gradient_rows.extend([
                r.rect,
                r.radius,
                [ia, ib, ic, id],
                [ie, iff, r.invert as f32, 0.0],
            ]);
        }
        grew |= write(
            device,
            queue,
            &mut self.gradients,
            "ui paint gradients",
            &self.gradient_rows,
        );
        self.stop_rows.clear();
        for s in &paint.stops {
            self.stop_rows.push(s.color);
            self.stop_rows.push([s.offset, 0.0, 0.0, 0.0]);
        }
        grew |= write(
            device,
            queue,
            &mut self.stops,
            "ui paint stops",
            &self.stop_rows,
        );
        let identity = [og_draw::paint::IDENTITY];
        let transforms = if paint.transforms.is_empty() {
            &identity[..]
        } else {
            &paint.transforms[..]
        };
        grew |= self.transforms.grow_and_write(
            device,
            queue,
            "ui paint transforms",
            bytemuck::cast_slice(transforms),
        );
        if let Some(icons) = &paint.icons {
            let at = (icons.as_ptr() as usize, icons.len());
            if at != self.icons {
                self.icons = at;
                grew |= write(device, queue, &mut self.segments, "ui paint icons", icons);
            }
        }
        if grew {
            self.group = Some(self.group0(device, &self.uniform));
        }
        self.upload_layers(device, queue, scale);
    }

    /// Group 0 of a pass whose frame uniform is `uniform`.
    fn group0(&self, device: &wgpu::Device, uniform: &wgpu::Buffer) -> wgpu::BindGroup {
        crate::bind_group(
            device,
            "ui paint",
            &self.layout,
            &[
                uniform.as_entire_binding(),
                wgpu::BindingResource::TextureView(&self.quads.view),
                self.gradients.as_entire_binding(),
                self.stops.as_entire_binding(),
                self.segments.as_entire_binding(),
                wgpu::BindingResource::TextureView(&self.transforms.view),
            ],
        )
    }

    /// Each filter group's layer target (kept while its size holds), and
    /// the frame its pass draws in: the layer's size and its top left on
    /// the output, device px.
    fn upload_layers(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, scale: f32) {
        if self.layers.is_empty() {
            return;
        }
        for m in 0..4 {
            if self.used_blends[m] {
                crate::deferred::Start::start(&self.layer_pipelines[m]);
            }
        }
        self.fx.borrow().filters.warm();
        let n = self.layers.len();
        self.targets.resize_with(n.max(self.targets.len()), || None);
        self.layer_groups.clear();
        for k in 0..n {
            let l = &self.layers[k];
            if l.backdrop {
                self.layer_groups.push(None);
                continue;
            }
            // At its level: its rect's size and top left, and the scale,
            // halved that many times.
            let half = (1u32 << l.level) as f32;
            let [x, y, w, h] = l.rect;
            let size = (w as u32 >> l.level, h as u32 >> l.level);
            if self.targets[k].as_ref().is_none_or(|t| t.size != size) {
                let view = crate::target_texture(
                    device,
                    "ui paint group",
                    size.0,
                    size.1,
                    super::pass::LAYER_FORMAT,
                    wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                )
                .create_view(&Default::default());
                self.targets[k] = Some(LayerTarget { size, view });
            }
            while self.frames.len() <= k {
                let b = crate::traffic::create_buffer(
                    device,
                    &wgpu::BufferDescriptor {
                        label: Some("ui paint group frame"),
                        size: UNIFORM_SIZE,
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    },
                );
                crate::traffic::write_buffer(queue, &b, 32, &preblend_table());
                self.frames.push(b);
            }
            let u: [f32; 8] = [
                size.0 as f32,
                size.1 as f32,
                scale / half,
                0.0,
                crate::overlay::dither_step(self.format),
                0.0,
                x as f32 / half,
                y as f32 / half,
            ];
            crate::traffic::write_buffer(queue, &self.frames[k], 0, bytemuck::cast_slice(&u));
            self.layer_groups
                .push(Some(self.group0(device, &self.frames[k])));
        }
    }

    /// Makes a texture for each page `atlas` has and this has not, and
    /// writes the plots whose pixels changed since they were last written.
    pub(crate) fn upload_glyphs(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &Atlas,
    ) {
        let now = (atlas.id(), atlas.revision());
        if self.glyphs_of == Some(now) {
            return;
        }
        // Another atlas (another scene): its pages replace these.
        if self.glyphs_of.is_some_and(|(of, _)| of != now.0) {
            self.glyph_pages = Default::default();
        }
        self.glyphs_of = Some(now);
        for tier in GlyphTier::ALL {
            let (pages, side) = atlas.pages(tier);
            let (t, bytes) = (tier as usize, tier.bytes());
            self.glyph_sides[t] = side;
            let per_row = side / PLOT;
            for (k, page) in pages.iter().enumerate() {
                if self.glyph_pages[t].len() <= k {
                    let texture = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("ui paint glyphs"),
                        size: wgpu::Extent3d {
                            width: side,
                            height: side,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        // A colour glyph's bytes are premultiplied sRGB:
                        // the shader decodes them after it divides.
                        format: match tier {
                            GlyphTier::Coverage => wgpu::TextureFormat::R8Unorm,
                            GlyphTier::Color | GlyphTier::Field => wgpu::TextureFormat::Rgba8Unorm,
                        },
                        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                        view_formats: &[],
                        texture_binding_view_dimension: None,
                    });
                    let view = texture.create_view(&Default::default());
                    let group = crate::sprite_bind_group(
                        device,
                        queue,
                        &self.glyph_layout,
                        &view,
                        &self.glyph_sampler,
                        [0.0; 4],
                    );
                    let params = match tier {
                        GlyphTier::Coverage => None,
                        GlyphTier::Color => Some([0.0, side as f32, side as f32, 0.0]),
                        GlyphTier::Field => {
                            Some([og_draw::glyphs::FIELD_RANGE, side as f32, side as f32, 0.0])
                        }
                    };
                    let drawn = params.map(|p| {
                        crate::sprite_bind_group(
                            device,
                            queue,
                            &self.glyph_layout,
                            &view,
                            &self.glyph_sampler,
                            p,
                        )
                    });
                    // A new texture is clear, as a plot never written is.
                    self.glyph_pages[t].push(GlyphPage {
                        texture,
                        group,
                        drawn,
                        written: vec![0; (per_row * per_row) as usize],
                    });
                }
                let gpu = &mut self.glyph_pages[t][k];
                for (plot, revision) in page.revisions().enumerate() {
                    if gpu.written[plot] == revision {
                        continue;
                    }
                    gpu.written[plot] = revision;
                    let (x, y) = (plot as u32 % per_row * PLOT, plot as u32 / per_row * PLOT);
                    // The plot's rows where they lie in the page's.
                    let first = ((y * side + x) * bytes) as usize;
                    let last = first + (((PLOT - 1) * side + PLOT) * bytes) as usize;
                    crate::traffic::write_texture(
                        queue,
                        wgpu::TexelCopyTextureInfo {
                            texture: &gpu.texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d { x, y, z: 0 },
                            aspect: wgpu::TextureAspect::All,
                        },
                        &page.pixels[first..last],
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(side * bytes),
                            rows_per_image: Some(PLOT),
                        },
                        wgpu::Extent3d {
                            width: PLOT,
                            height: PLOT,
                            depth_or_array_layers: 1,
                        },
                    );
                }
            }
        }
    }

    /// The arena's quads as instances, each image of a loaded `texture`,
    /// each group's layer (its rectangle at `scale` device px a CSS px),
    /// and the draw lists' runs.
    fn resolve(&mut self, paint: &Paint, texture: impl Fn(TextureHandle) -> bool, scale: f32) {
        self.gpu.clear();
        self.binds.clear();
        self.blends.clear();
        self.used_blends = [true, false, false, false];
        self.runs.clear();
        self.at.clear();
        self.layers.clear();
        self.layer_of.clear();
        self.layer_of.resize(paint.groups.len(), None);
        self.order.clear();
        self.missing = 0;
        for (i, q) in paint.quads.iter().enumerate() {
            self.at.push(self.gpu.len() as u32);
            let blend = q.blend.min(3) as u8;
            self.used_blends[blend as usize] = true;
            let mut g = PaintGpu {
                // A border's other sides' colour (an image sets its own).
                uv: if q.kind == PaintKind::Border as u32 {
                    q.uv
                } else {
                    [0.0; 4]
                },
                rect: q.rect,
                clip: q.clip,
                color: q.color,
                r: q.radius,
                p: q.p,
                k: [q.kind, q.a, q.b, q.xf],
                // The mask in w, which only an image's wipe shares (xyz).
                w: [0.0, 0.0, 0.0, q.mask as f32],
                // Its colour matrix's five rows follow the gradients'; z carries blend mode.
                f: [
                    match q.filter {
                        0 => 0,
                        k => 3 * paint.gradients.len() as u32 + 5 * (k - 1) + 1,
                    },
                    match q.round {
                        0 => 0,
                        k => {
                            3 * paint.gradients.len() as u32
                                + 5 * paint.filters.len() as u32
                                + 4 * (k - 1)
                                + 1
                        }
                    },
                    u32::from(blend),
                    0,
                ],
            };
            if q.kind == PaintKind::Line as u32 {
                let from = self.gpu.len();
                self.segments_of(q, &paint.points, g);
                let added = self.gpu.len() - from;
                self.binds.resize(self.binds.len() + added, None);
                self.blends.resize(self.blends.len() + added, blend);
                continue;
            }
            // What the quad samples; a quad that samples nothing draws with
            // whatever is bound.
            let mut wants = None;
            if q.kind == PaintKind::Image as u32 {
                let t = TextureHandle::from_bits(q.a);
                if !texture(t) {
                    self.missing += 1;
                    continue;
                }
                g.uv = q.uv;
                g.w = [q.wipe[0], q.wipe[1], q.wipe[2], g.w[3]];
                wants = Some(PaintBind::Texture(t));
            } else if q.kind == PaintKind::Raster as u32 {
                // A page the atlas has not made (a paint older than its
                // atlas) draws nothing.
                let side = self.glyph_sides.get(q.a as usize).copied().unwrap_or(0);
                let made = self
                    .glyph_pages
                    .get(q.a as usize)
                    .is_some_and(|p| (q.b as usize) < p.len());
                if !made || side == 0 {
                    self.missing += 1;
                    continue;
                }
                let s = side as f32;
                g.uv = [q.uv[0] / s, q.uv[1] / s, q.uv[2] / s, q.uv[3] / s];
                wants = Some(PaintBind::Glyphs(q.a as u8, q.b as u16));
            } else if q.kind == PaintKind::Group as u32 {
                // Its layer, or none when it shows nothing (and what it
                // holds draws nowhere).
                let Some(group) = paint.groups.get(q.a as usize) else {
                    continue;
                };
                let Some(rect) = group_rect(paint, group, q, scale) else {
                    continue;
                };
                let k = self.layers.len();
                self.layer_of[q.a as usize] = Some(k);
                let effects = paint
                    .effects
                    .get(group.first as usize..(group.first + group.count) as usize)
                    .unwrap_or(&[]);
                let space = paint
                    .transforms
                    .get(group.xf as usize)
                    .copied()
                    .unwrap_or(og_draw::paint::IDENTITY);
                let filter = filter_of(
                    effects,
                    &paint.filters,
                    space,
                    Vec::new(),
                    UiFilterInput::Source,
                );
                // The pass's own layer as drawn so far (result 0) over the
                // copy of the output under it (the source), then the rest.
                let filter_over = group.backdrop.then(|| {
                    let drawn = UiFilterPrimitive {
                        op: UiFilterOp::Layer { layer: 0 },
                        inputs: [UiFilterInput::Source; 2],
                        subregion: [-1.0e9, -1.0e9, 2.0e9, 2.0e9],
                        srgb: false,
                    };
                    let over = UiFilterPrimitive {
                        op: UiFilterOp::Composite {
                            op: UiFilterComposite::Over,
                            k: [0.0; 4],
                        },
                        inputs: [UiFilterInput::Result(0), UiFilterInput::Source],
                        ..drawn.clone()
                    };
                    filter_of(
                        effects,
                        &paint.filters,
                        space,
                        vec![drawn, over],
                        UiFilterInput::Result(1),
                    )
                });
                // A group a wide blur filters first draws at that blur's
                // level (one holding another group's composite at the
                // layer's size: that composite reads its result by the
                // output's px). A wide blur's result stays below the
                // layer's size, which its composite reads bilinearly. The
                // layer is a whole number of either level's texels, so they
                // cover it exactly.
                let nested = paint.quads[i + 1..(group.end as usize).min(paint.quads.len())]
                    .iter()
                    .any(|q| q.kind == PaintKind::Group as u32);
                let level = if group.backdrop || nested {
                    0
                } else {
                    super::filter::source_level(&filter, scale)
                };
                let unit = 1i32 << super::filter::result_level(&filter, scale).max(level);
                let mut rect = rect;
                for side in &mut rect[2..] {
                    *side = (*side + unit - 1) / unit * unit;
                }
                self.layers.push(GroupLayer {
                    backdrop: group.backdrop,
                    rect,
                    level,
                    steps: Vec::new(),
                    filter,
                    filter_over,
                });
                // Where it reads its result: the rectangle's top left and
                // size, device px.
                g.uv = rect.map(|v| v as f32);
                wants = Some(PaintBind::Group(k as u16));
            }
            self.gpu.push(g);
            self.binds.push(wants);
            self.blends.push(blend);
        }
        self.at.push(self.gpu.len() as u32);
        self.main = self.steps(paint, 0..paint.quads.len());
    }

    /// The draw list of quads `span`: each run of instances that one
    /// group 1 and one blend mode draw; a filter group's composite, its quads
    /// drawn into its layer's own list; a backdrop's capture, then its composite.
    fn steps(&mut self, paint: &Paint, span: Range<usize>) -> Vec<Step> {
        let mut steps = Vec::new();
        // This list's runs not yet in `runs` (a nested group's list lands
        // there first), and the run being made.
        let mut list: Vec<PaintRun> = Vec::new();
        let mut run: Option<(Option<PaintBind>, u8, u32, u32)> = None;
        let close = |list: &mut Vec<PaintRun>,
                     run: &mut Option<(Option<PaintBind>, u8, u32, u32)>| {
            if let Some((bind, blend, first, count)) = run.take() {
                list.push(PaintRun {
                    bind: bind.unwrap_or(PaintBind::White),
                    blend,
                    first,
                    count,
                });
            }
        };
        let mut i = span.start;
        while i < span.end {
            let q = &paint.quads[i];
            let (a, b) = (self.at[i], self.at[i + 1]);
            if q.kind == PaintKind::Group as u32 {
                let gi = q.a as usize;
                let end = paint
                    .groups
                    .get(gi)
                    .map_or(i + 1, |g| (g.end as usize).clamp(i + 1, span.end));
                if let Some(k) = self.layer_of.get(gi).copied().flatten() {
                    close(&mut list, &mut run);
                    if self.layers[k].backdrop {
                        let from = self.runs.len();
                        self.runs.append(&mut list);
                        steps.push(Step::Runs(from..self.runs.len()));
                        steps.push(Step::Backdrop(k));
                    } else {
                        let inner = self.steps(paint, i + 1..end);
                        self.layers[k].steps = inner;
                        self.order.push(k);
                    }
                    // Its composite in a run of its own: its result may
                    // not draw this frame.
                    list.push(PaintRun {
                        bind: PaintBind::Group(k as u16),
                        blend: q.blend.min(3) as u8,
                        first: a,
                        count: b - a,
                    });
                }
                i = end;
                continue;
            }
            for inst in a..b {
                let want = self.binds[inst as usize];
                let blend = self.blends[inst as usize];
                match &mut run {
                    Some((bind, rblend, first, count))
                        if *first + *count == inst
                            && *rblend == blend
                            && (want.is_none() || bind.is_none() || *bind == want) =>
                    {
                        *count += 1;
                        *bind = bind.or(want);
                    }
                    _ => {
                        close(&mut list, &mut run);
                        run = Some((want, blend, inst, 1));
                    }
                }
            }
            i += 1;
        }
        close(&mut list, &mut run);
        let from = self.runs.len();
        self.runs.append(&mut list);
        steps.push(Step::Runs(from..self.runs.len()));
        steps
    }

    /// A polyline's instances, one a segment, each over its segment's
    /// bounds and the stroke's reach: `uv` its ends, `r` its neighbours'
    /// far ends, `p` the width and which neighbours it has and whether
    /// each wins a tie (the lower segment does), all in CSS px. A pixel
    /// paints in the segment nearest it of itself and its neighbours, so
    /// round joins are a union with no alpha drawn twice.
    fn segments_of(&mut self, q: &og_draw::paint::PaintQuad, points: &[[f32; 2]], g: PaintGpu) {
        let (width, closed) = (q.p[0], q.p[1] > 0.5);
        let Some(pts) = points.get(q.a as usize..(q.a + q.b) as usize) else {
            return;
        };
        if width <= 0.0 || pts.len() < 2 {
            return;
        }
        let n = pts.len();
        let at = |k: usize| [pts[k][0] + q.rect[0], pts[k][1] + q.rect[1]];
        let count = if closed && n > 2 { n } else { n - 1 };
        let reach = 0.5 * width + 1.0;
        for k in 0..count {
            let (a, b) = (at(k), at((k + 1) % n));
            let prev = (k > 0 || count == n).then(|| at((k + n - 1) % n));
            let next = (k + 1 < count || count == n).then(|| at((k + 2) % n));
            // Bits: 1 a neighbour before, 2 after; 4 the one before wins a
            // tie (it is lower), 8 the one after does (the closing wrap).
            let flags = u32::from(prev.is_some())
                | u32::from(next.is_some()) << 1
                | u32::from(k > 0) << 2
                | u32::from(k + 1 == count && count == n) << 3;
            let x0 = a[0].min(b[0]) - reach;
            let y0 = a[1].min(b[1]) - reach;
            let [pv, nx] = [prev.unwrap_or(a), next.unwrap_or(b)];
            self.gpu.push(PaintGpu {
                rect: [
                    x0,
                    y0,
                    a[0].max(b[0]) + reach - x0,
                    a[1].max(b[1]) + reach - y0,
                ],
                uv: [a[0], a[1], b[0], b[1]],
                r: [pv[0], pv[1], nx[0], nx[1]],
                p: [width, flags as f32, 0.0, 0.0],
                ..g
            });
        }
    }

    /// Whether anything is uploaded to draw.
    pub fn drawn(&self) -> bool {
        !self.runs.is_empty() && self.group.is_some()
    }

    /// Makes the pass's layer for an output of `size` it cannot blend
    /// in, where it has none of that size (`layout`, `sampler`: the
    /// sprites', which bind it to the composite).
    pub fn ensure_layer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        size: (u32, u32),
    ) {
        if self.layer.as_ref().is_some_and(|l| l.size == size) {
            return;
        }
        let view = crate::target_texture(
            device,
            "ui paint layer",
            size.0,
            size.1,
            self.format,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        )
        .create_view(&Default::default());
        let group = crate::sprite_bind_group(device, queue, layout, &view, sampler, [0.0; 4]);
        let behind = crate::target_texture(
            device,
            "ui paint behind",
            size.0,
            size.1,
            self.output_format,
            wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let behind_view = behind.create_view(&Default::default());
        let behind_group =
            crate::sprite_bind_group(device, queue, layout, &behind_view, sampler, [0.0; 4]);
        self.layer = Some(Layer {
            size,
            view,
            group,
            behind,
            behind_view,
            behind_group,
        });
    }

    /// What binds glyph atlas page `page` of `tier` for world and overlay
    /// text ([`og_draw::TextGlyph`]); `None` before it is uploaded.
    pub(crate) fn glyph_group(&self, tier: GlyphTier, page: u16) -> Option<&wgpu::BindGroup> {
        self.glyph_pages[tier as usize]
            .get(page as usize)?
            .drawn
            .as_ref()
    }

    /// The layer, where an output of its size needed one.
    pub fn layer(&self) -> Option<&wgpu::TextureView> {
        self.layer.as_ref().map(|l| &l.view)
    }

    /// Whether the main draw ends its pass at a backdrop: it is drawn by
    /// [`Self::record_main`], in passes of its own.
    pub fn splits(&self) -> bool {
        self.main.iter().any(|s| matches!(s, Step::Backdrop(_)))
    }

    /// Whether `paint` has a backdrop outside every filter group (one in
    /// a group reads that group's layer): the frame goes through a target
    /// its backdrops can read.
    pub fn reads_target(paint: &Paint) -> bool {
        let g = &paint.groups;
        g.iter().any(|b| {
            b.backdrop
                && !g
                    .iter()
                    .any(|f| !f.backdrop && f.quad < b.quad && b.quad < f.end)
        })
    }

    /// Draws the uploaded paint into `pass`, on a target in the pass's
    /// format ([`encoded_format`]); `group1` binds a run's font page or
    /// texture (a glyph atlas page and a group's result are bound here).
    /// The main list only, which has no backdrop ([`Self::splits`]).
    /// Returns the draws.
    pub fn draw<'p>(
        &'p self,
        pass: &mut wgpu::RenderPass<'p>,
        group1: impl Fn(PaintBind) -> &'p wgpu::BindGroup,
    ) -> u32 {
        let (Some(group), Some(normal)) = (&self.group, self.pipelines[0].get()) else {
            return 0;
        };
        let pipelines = [
            normal,
            self.pipelines[1].get().unwrap_or(normal),
            self.pipelines[2].get().unwrap_or(normal),
            self.pipelines[3].get().unwrap_or(normal),
        ];
        pass.set_bind_group(0, group, &[]);
        let fx = self.fx.borrow();
        let mut draws = 0;
        for step in &self.main {
            if let Step::Runs(runs) = step {
                draws += self.draw_runs(pass, &fx, &pipelines, runs.clone(), &group1);
            }
        }
        draws
    }

    /// Draws `runs` with group 0 bound, switching pipeline when a run's blend
    /// mode changes; a group's composite whose result did not draw is left out.
    fn draw_runs<'g>(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        fx: &Fx,
        pipelines: &[&wgpu::RenderPipeline; 4],
        runs: Range<usize>,
        group1: &dyn Fn(PaintBind) -> &'g wgpu::BindGroup,
    ) -> u32 {
        let mut draws = 0;
        let mut current: Option<u8> = None;
        for r in &self.runs[runs] {
            let bound = match r.bind {
                PaintBind::Glyphs(tier, page) => {
                    &self.glyph_pages[tier as usize][page as usize].group
                }
                PaintBind::Group(k) => match fx.results.get(k as usize) {
                    Some(Some(g)) => g,
                    _ => continue,
                },
                bind => group1(bind),
            };
            let blend = r.blend.min(3);
            if current != Some(blend) {
                pass.set_pipeline(pipelines[blend as usize]);
                current = Some(blend);
            }
            pass.set_bind_group(1, bound, &[]);
            pass.draw(0..6, r.first..r.first + r.count);
            draws += 1;
        }
        draws
    }

    /// Draws each filter group's layer, innermost first, and runs its
    /// filter over it, so its composite reads the result; a backdrop in a
    /// group reads that group's layer as drawn so far. Runs every frame
    /// before the main draw (it frees the last frame's filter targets).
    /// Returns the draws.
    pub(crate) fn record_groups(&self, encoder: &mut crate::Encoder, cx: &GroupCx<'_>) -> u32 {
        let mut fx = self.fx.borrow_mut();
        fx.filters.begin();
        // A result's binding lives as long as its target.
        let Fx { filters, binds, .. } = &mut *fx;
        binds.retain(|(v, _)| filters.holds(v));
        fx.results.clear();
        fx.results.resize(self.layers.len(), None);
        let mut draws = 0;
        let Some(normal) = self.layer_pipelines[0].get() else {
            return 0;
        };
        let pipelines = [
            normal,
            self.layer_pipelines[1].get().unwrap_or(normal),
            self.layer_pipelines[2].get().unwrap_or(normal),
            self.layer_pipelines[3].get().unwrap_or(normal),
        ];
        for &k in &self.order {
            let (Some(target), Some(Some(group0))) = (&self.targets[k], self.layer_groups.get(k))
            else {
                continue;
            };
            let l = &self.layers[k];
            let origin = [l.rect[0] as f32, l.rect[1] as f32];
            let scale = self.scale;
            let under = Under::Target(&target.view, target.size);
            draws += self.record_list(
                &mut fx,
                encoder,
                cx,
                (&target.view, wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)),
                (&pipelines, group0),
                &l.steps,
                (&under, origin.map(|v| v / scale)),
            );
            draws += self.run_filter(
                &mut fx,
                encoder,
                cx,
                k,
                &l.filter,
                Source::Drawn(&target.view, l.level),
            );
        }
        draws
    }

    /// Draws the main list into `view` (in the pass's format, loaded with
    /// `load`), ending its pass at each backdrop to capture what `under`
    /// says is under it. Returns the draws.
    pub(crate) fn record_main(
        &self,
        encoder: &mut crate::Encoder,
        cx: &GroupCx<'_>,
        view: &wgpu::TextureView,
        load: wgpu::LoadOp<wgpu::Color>,
        under: Under<'_>,
    ) -> u32 {
        let (Some(group), Some(normal)) = (&self.group, self.pipelines[0].get()) else {
            return 0;
        };
        let pipelines = [
            normal,
            self.pipelines[1].get().unwrap_or(normal),
            self.pipelines[2].get().unwrap_or(normal),
            self.pipelines[3].get().unwrap_or(normal),
        ];
        let mut fx = self.fx.borrow_mut();
        self.record_list(
            &mut fx,
            encoder,
            cx,
            (view, load),
            (&pipelines, group),
            &self.main,
            (&under, [0.0; 2]),
        )
    }

    /// Draws `steps` into `target` with `pass` (the blend pipelines and group 0),
    /// a pass a stretch, capturing for each backdrop what `under` (the
    /// target as a texture, its top left at `origin` CSS px) holds.
    #[allow(clippy::too_many_arguments)]
    fn record_list(
        &self,
        fx: &mut Fx,
        encoder: &mut crate::Encoder,
        cx: &GroupCx<'_>,
        (view, mut load): (&wgpu::TextureView, wgpu::LoadOp<wgpu::Color>),
        (pipelines, group0): (&[&wgpu::RenderPipeline; 4], &wgpu::BindGroup),
        steps: &[Step],
        (under, origin): (&Under<'_>, [f32; 2]),
    ) -> u32 {
        let mut draws = 0;
        for step in steps {
            match step {
                Step::Runs(runs) => {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("ui paint"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    load = wgpu::LoadOp::Load;
                    pass.set_bind_group(0, group0, &[]);
                    draws += self.draw_runs(&mut pass, fx, pipelines, runs.clone(), cx.group1);
                }
                &Step::Backdrop(k) => draws += self.capture(fx, encoder, cx, k, under, origin),
            }
        }
        draws
    }

    /// Backdrop `k`'s capture of what `under` holds below it, filtered.
    fn capture(
        &self,
        fx: &mut Fx,
        encoder: &mut crate::Encoder,
        cx: &GroupCx<'_>,
        k: usize,
        under: &Under<'_>,
        origin: [f32; 2],
    ) -> u32 {
        let l = &self.layers[k];
        let scale = self.scale;
        let (source, filter) = match (under, &self.layer) {
            (Under::Target(view, size), _) => (
                Source::Capture(Capture {
                    view,
                    origin,
                    scale,
                    decode: false,
                    encode: false,
                    size: *size,
                }),
                &l.filter,
            ),
            // The pass's own layer as drawn so far over the copy of the
            // output under it (stored in light, encoded as it is read).
            (Under::Layer(true), Some(layer)) => (
                Source::Capture(Capture {
                    view: &layer.behind_view,
                    origin: [0.0; 2],
                    scale,
                    decode: false,
                    encode: true,
                    size: layer.size,
                }),
                l.filter_over.as_ref().unwrap_or(&l.filter),
            ),
            (Under::Layer(false), Some(layer)) => (
                Source::Capture(Capture {
                    view: &layer.view,
                    origin: [0.0; 2],
                    scale,
                    decode: false,
                    encode: false,
                    size: layer.size,
                }),
                &l.filter,
            ),
            _ => return 0,
        };
        self.run_filter(fx, encoder, cx, k, filter, source)
    }

    /// Runs `filter` over `source` into layer `k`'s rectangle and binds
    /// the result for its composite.
    fn run_filter(
        &self,
        fx: &mut Fx,
        encoder: &mut crate::Encoder,
        cx: &GroupCx<'_>,
        k: usize,
        filter: &UiFilter,
        source: Source<'_>,
    ) -> u32 {
        let l = &self.layers[k];
        let scale = self.scale;
        let size = (l.rect[2] as u32, l.rect[3] as u32);
        let origin = [l.rect[0] as f32 / scale, l.rect[1] as f32 / scale];
        let paint_layer = self.layer.as_ref().map(|l| (l.view.clone(), l.size));
        let layer_input = move |_: u32| {
            paint_layer.clone().map(|(view, size)| LayerInput {
                view,
                origin: [0.0; 2],
                scale,
                size,
            })
        };
        let inputs = Inputs {
            white: &self.blank,
            texture_view: &|_| None,
            layer: &layer_input,
        };
        let Some(prepared) = fx.filters.prepare(
            cx.device,
            cx.queue,
            Some(filter),
            source,
            size,
            origin,
            scale,
            &inputs,
            true,
        ) else {
            return 0;
        };
        let draws = fx.filters.record(encoder, &prepared);
        fx.filters.done(&prepared);
        // A pass that did not run (its pipeline not made yet) leaves the
        // group out this frame.
        if draws < prepared.passes() {
            return draws;
        }
        let bind = match fx.binds.iter().find(|(v, _)| *v == prepared.result) {
            Some((_, b)) => b.clone(),
            None => {
                let b = crate::sprite_bind_group(
                    cx.device,
                    cx.queue,
                    cx.layout,
                    &prepared.result,
                    cx.sampler,
                    [0.0; 4],
                );
                fx.binds.push((prepared.result.clone(), b.clone()));
                b
            }
        };
        fx.results[k] = Some(bind);
        draws
    }

    /// Copies `target`, the texture the layer is about to be drawn over,
    /// into the layer's copy of what is behind it. Returns the copies.
    pub fn copy_behind(&self, encoder: &mut crate::Encoder, target: &wgpu::Texture) -> u32 {
        let Some(layer) = &self.layer else {
            return 0;
        };
        encoder.copy_texture_to_texture(
            target.as_image_copy(),
            layer.behind.as_image_copy(),
            wgpu::Extent3d {
                width: layer.size.0.min(target.width()),
                height: layer.size.1.min(target.height()),
                depth_or_array_layers: 1,
            },
        );
        1
    }

    /// Draws the layer over `pass`'s target, the output: what the pass
    /// blended in it stays as blended. With `over` (after
    /// [`Self::copy_behind`]), its translucent pixels meet what is behind
    /// them in encoded values too, as on an sRGB output; without it (a
    /// target the renderer cannot copy, a caller's view), in the output's
    /// own light. Returns the draws.
    pub fn composite<'p>(&'p self, pass: &mut wgpu::RenderPass<'p>, over: bool) -> u32 {
        let (Some(group), Some(layer), Some((blended, behind))) =
            (&self.group, &self.layer, &self.composite)
        else {
            return 0;
        };
        let Some(pipeline) = (if over { behind } else { blended }).get() else {
            return 0;
        };
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, group, &[]);
        pass.set_bind_group(1, &layer.group, &[]);
        if over {
            pass.set_bind_group(2, &layer.behind_group, &[]);
        }
        pass.draw(0..3, 0..1);
        1
    }
}
