//! The paint pass: og-ui's paint arena ([`og_draw::paint::Paint`]) drawn
//! with one uber shader (`shaders/ui_paint.wgsl`), one pipeline for the
//! output and one for its sRGB view, both made with the renderer: no UI
//! pipeline is made after boot. The arena is uploaded only when its
//! generation, the target or a font's pages change; a glyph quad is
//! resolved against its font's pages then (its first page with the glyph),
//! and the quads draw in runs of one page each, in the arena's order,
//! inside the overlay pass where the UI's layer lies
//! ([`crate::overlay::OverlayFrame::ui_at`]).

use crate::FontHandle;
use crate::scene::GlyphKey;
use crate::sprites::FontLayout;
use bytemuck::{Pod, Zeroable};
use og_draw::paint::{Paint, PaintKind};

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
}

/// Consecutive instances drawn with one font page bound (`None`: the white
/// sprite, nothing samples it).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PaintRun {
    pub page: Option<FontHandle>,
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
}

pub(crate) struct PaintRenderer {
    output: wgpu::RenderPipeline,
    linear: Option<wgpu::RenderPipeline>,
    layout: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
    quads: wgpu::Buffer,
    gradients: wgpu::Buffer,
    stops: wgpu::Buffer,
    segments: wgpu::Buffer,
    group: Option<wgpu::BindGroup>,
    uploaded: Option<Uploaded>,
    /// The icon table uploaded (its address and length).
    icons: (usize, usize),
    gpu: Vec<PaintGpu>,
    gradient_rows: Vec<[f32; 4]>,
    stop_rows: Vec<[f32; 4]>,
    pub runs: Vec<PaintRun>,
    /// Glyphs no page of their font has (yet).
    pub missing: u32,
}

const UNIFORM_SIZE: u64 = 32;

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
    /// Its pipelines, made now (on the web asked for asynchronously, so
    /// the page's first frame waits for them behind its gate), and its
    /// buffers.
    pub fn new(
        device: &wgpu::Device,
        output_format: wgpu::TextureFormat,
        linear_format: Option<wgpu::TextureFormat>,
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
                entry(1, storage),
                entry(2, storage),
                entry(3, storage),
                entry(4, storage),
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ui paint"),
            bind_group_layouts: &[Some(&layout), Some(sprite_layout)],
            immediate_size: 0,
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ui paint"),
            source: wgpu::ShaderSource::Wgsl(source().into()),
        });
        let pipeline = |format| {
            crate::traffic::create_render_pipeline(
                device,
                &wgpu::RenderPipelineDescriptor {
                    label: Some("ui paint"),
                    layout: Some(&pl),
                    vertex: wgpu::VertexState {
                        module: &module,
                        entry_point: Some("vs_paint"),
                        compilation_options: Default::default(),
                        buffers: &[],
                    },
                    primitive: Default::default(),
                    depth_stencil: None,
                    multisample: Default::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &module,
                        entry_point: Some("fs_paint"),
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview_mask: None,
                    cache: crate::pipeline_cache::of(device).as_ref(),
                },
            )
        };
        let (output, linear) =
            crate::frame_pipelines(|| (pipeline(output_format), linear_format.map(pipeline)));
        PaintRenderer {
            output,
            linear,
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
            quads: buffer(device, "ui paint quads", 0),
            gradients: buffer(device, "ui paint gradients", 0),
            stops: buffer(device, "ui paint stops", 0),
            segments: buffer(device, "ui paint icons", 0),
            group: None,
            uploaded: None,
            icons: (0, 0),
            gpu: Vec::new(),
            gradient_rows: Vec::new(),
            stop_rows: Vec::new(),
            runs: Vec::new(),
            missing: 0,
        }
    }

    /// Uploads `paint` for an output of `width`×`height` device pixels
    /// when it, the target or the fonts' `revisions` changed since the
    /// last upload, and the frame uniform (`encode`: the shader encodes
    /// sRGB; `step`: one of the output's code values).
    #[allow(clippy::too_many_arguments)]
    pub fn upload<'a>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        paint: &Paint,
        font: impl Fn(FontHandle) -> Option<&'a FontLayout>,
        revisions: u64,
        (width, height): (u32, u32),
        encode: bool,
        step: f32,
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
            f32::from(u8::from(encode)),
            step,
            0.0,
            0.0,
            0.0,
        ];
        crate::traffic::write_buffer(queue, &self.uniform, 0, bytemuck::cast_slice(&u));
        let now = Uploaded {
            generation: paint.generation,
            revisions,
            width,
            height,
        };
        if self.uploaded == Some(now) {
            return;
        }
        self.uploaded = Some(now);
        self.resolve(paint, font);
        let mut grew = self.group.is_none();
        grew |= write(device, queue, &mut self.quads, "ui paint quads", &self.gpu);
        self.gradient_rows.clear();
        self.gradient_rows.extend(
            paint
                .gradients
                .iter()
                .map(|g| [g.angle, g.first as f32, g.count as f32, 0.0]),
        );
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
        if let Some(icons) = &paint.icons {
            let at = (icons.as_ptr() as usize, icons.len());
            if at != self.icons {
                self.icons = at;
                grew |= write(device, queue, &mut self.segments, "ui paint icons", icons);
            }
        }
        if grew {
            self.group = Some(crate::bind_group(
                device,
                "ui paint",
                &self.layout,
                &[
                    self.uniform.as_entire_binding(),
                    self.quads.as_entire_binding(),
                    self.gradients.as_entire_binding(),
                    self.stops.as_entire_binding(),
                    self.segments.as_entire_binding(),
                ],
            ));
        }
    }

    /// The arena's quads as instances, each glyph from the first page of
    /// its font that has it, and their runs.
    fn resolve<'a>(&mut self, paint: &Paint, font: impl Fn(FontHandle) -> Option<&'a FontLayout>) {
        self.gpu.clear();
        self.runs.clear();
        self.missing = 0;
        let mut page: Option<FontHandle> = None;
        let mut first = 0u32;
        for q in &paint.quads {
            let mut g = PaintGpu {
                rect: q.rect,
                clip: q.clip,
                color: q.color,
                r: q.radius,
                p: q.p,
                k: [q.kind, q.a, q.b, 0],
                ..Default::default()
            };
            if q.kind == PaintKind::Glyph as u32 {
                // Every page of a run-time font knows each glyph's advance;
                // only the page that holds it has its bounds.
                let found = char::from_u32(q.a).and_then(|ch| {
                    paint.fonts.get(q.b as usize)?.iter().find_map(|&h| {
                        let l = font(h)?;
                        let g = l.glyphs.get(&GlyphKey::Char(ch))?;
                        let [pl, pb, pr, pt] = g.plane_bounds;
                        (pr > pl && pt > pb).then_some((h, l, g))
                    })
                });
                let Some((h, l, glyph)) = found else {
                    self.missing += 1;
                    continue;
                };
                let [pl, pb, pr, pt] = glyph.plane_bounds;
                if pr <= pl || pt <= pb {
                    continue;
                }
                let [x, baseline, size, _] = q.rect;
                g.rect = [
                    x + pl * size,
                    baseline - pt * size,
                    (pr - pl) * size,
                    (pt - pb) * size,
                ];
                let [al, at, ar, ab] = glyph.atlas_bounds;
                let s = l.atlas_size;
                g.uv = [al / s.x, at / s.y, ar / s.x, ab / s.y];
                if l.color && !l.foreground {
                    g.color = [1.0, 1.0, 1.0, q.color[3]];
                }
                if page != Some(h) {
                    if page.is_some() || self.gpu.len() as u32 > first {
                        self.runs.push(PaintRun {
                            page,
                            first,
                            count: self.gpu.len() as u32 - first,
                        });
                        first = self.gpu.len() as u32;
                    }
                    page = Some(h);
                }
            }
            self.gpu.push(g);
        }
        if self.gpu.len() as u32 > first {
            self.runs.push(PaintRun {
                page,
                first,
                count: self.gpu.len() as u32 - first,
            });
        }
    }

    /// Whether anything is uploaded to draw.
    pub fn drawn(&self) -> bool {
        !self.runs.is_empty() && self.group.is_some()
    }

    /// Draws the uploaded paint into `pass` (an sRGB view's pipeline when
    /// `linear`); `group1` binds a run's page. Returns the draws.
    pub fn draw<'p>(
        &'p self,
        pass: &mut wgpu::RenderPass<'p>,
        linear: bool,
        group1: impl Fn(Option<FontHandle>) -> &'p wgpu::BindGroup,
    ) -> u32 {
        let Some(group) = &self.group else {
            return 0;
        };
        let pipeline = match (linear, &self.linear) {
            (true, Some(l)) => l,
            _ => &self.output,
        };
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, group, &[]);
        for r in &self.runs {
            pass.set_bind_group(1, group1(r.page), &[]);
            pass.draw(0..6, r.first..r.first + r.count);
        }
        self.runs.len() as u32
    }
}
