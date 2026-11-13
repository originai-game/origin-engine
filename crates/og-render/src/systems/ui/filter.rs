//! SVG filter graphs over UI layers (Filter Effects 1 §9-15): a
//! [`UiFilter`] on a [`super::UiLayer`] runs its primitives over what the
//! layer drew (SourceGraphic) once it is drawn, or over what lies under it
//! for a backdrop layer ([`super::UiLayer::backdrop`]), each a full-target
//! pass into a result of the layer's size (`shaders/ui_filter.wgsl`), and
//! the layer composites the last result instead of what it drew. The
//! layer's rect is the filter region on the screen. Results are
//! premultiplied linear light, as the layers are
//! (`color-interpolation-filters: linearRGB`); a primitive marked `srgb`
//! works on sRGB-encoded values.
//!
//! A filter is written in its own space ([`UiFilter::space`], the
//! element's user space): subregions, lengths, positions and lights are
//! in its units, and every primitive runs there, whatever turn, skew or
//! scale takes that space onto the screen. A blur or a morphology runs
//! along the space's axes (a Gaussian's two passes along the images of
//! its axes are the turned Gaussian exactly); lighting computes its
//! surface and its lights in it.

use super::UiBlend;
use crate::TextureHandle;
pub use og_draw::ui::filter::*;

/// `struct FilterParams` in ui_filter.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct FilterParams {
    op: [u32; 4],
    origin: [f32; 4],
    subregion: [f32; 4],
    /// The filter's space to CSS px: its linear part (a, b, c, d) and
    /// its translation (e, f), then the inverse's.
    space: [f32; 4],
    space_t: [f32; 4],
    inverse: [f32; 4],
    inverse_t: [f32; 4],
    a: [f32; 4],
    b: [f32; 4],
    c: [f32; 4],
    m: [[f32; 4]; 5],
    table: [[f32; 4]; 16],
}

const FLAG_SRGB: u32 = 1;
const FLAG_IN1_ALPHA: u32 = 2;
const FLAG_IN2_ALPHA: u32 = 4;
const FLAG_STITCH: u32 = 16;
const FLAG_FRACTAL: u32 = 32;
/// Reads past an input's edge take its edge (a backdrop: what is under
/// the element goes on past what was captured of it).
const FLAG_CLAMP: u32 = 64;

/// The most texels a blur pass reads a side (ui_filter.wgsl's case 0).
const BLUR_REACH: f32 = 128.0;

/// The ops' codes in ui_filter.wgsl beyond a primitive's.
const OP_LAYER: u32 = 16;
const OP_CAPTURE: u32 = 17;

/// UiBlend's code in the shaders (src/ui/build.rs).
fn blend_code(b: UiBlend) -> f32 {
    b as u32 as f32
}

/// Where a layer's filter runs: the layer target's top left (CSS px), its
/// device px per CSS px, and the filter's space.
#[derive(Clone, Copy, Debug)]
struct Frame {
    origin: [f32; 2],
    scale: f32,
    space: [f32; 6],
}

impl Frame {
    /// A vector of the filter's space in device px.
    fn device(&self, v: [f32; 2]) -> [f32; 2] {
        let [a, b, c, d, ..] = self.space;
        [
            (a * v[0] + c * v[1]) * self.scale,
            (b * v[0] + d * v[1]) * self.scale,
        ]
    }

    /// A cell's two edges in device px: `unit` along the space's axes, or
    /// one device px along each; and how much a step along each is in
    /// cells of the surface's normal (1 for a given unit, else the space
    /// units a device px is, the normal then per unit).
    fn cells(&self, unit: Option<[f32; 2]>) -> ([f32; 4], [f32; 2]) {
        match unit {
            Some([x, y]) => {
                let (u, v) = (self.device([x, 0.0]), self.device([0.0, y]));
                ([u[0], u[1], v[0], v[1]], [1.0, 1.0])
            }
            None => {
                let (u, v) = (self.device([1.0, 0.0]), self.device([0.0, 1.0]));
                let (lu, lv) = (u[0].hypot(u[1]).max(1e-6), v[0].hypot(v[1]).max(1e-6));
                ([u[0] / lu, u[1] / lu, v[0] / lv, v[1] / lv], [lu, lv])
            }
        }
    }
}

/// The affine map's inverse, `None` when it has none.
fn invert(m: [f32; 6]) -> Option<[f32; 6]> {
    let [a, b, c, d, e, f] = m;
    let det = a * d - b * c;
    if !det.is_finite() || det.abs() < 1e-12 {
        return None;
    }
    let (ia, ib, ic, id) = (d / det, -b / det, -c / det, a / det);
    Some([ia, ib, ic, id, -(ia * e + ic * f), -(ib * e + id * f)])
}

/// The parameters of each pass of a primitive (two for a blur or a
/// morphology, one axis of the space each) in `frame`, every read past
/// an input's edge taking the edge when `clamp`.
fn passes(p: &UiFilterPrimitive, frame: &Frame, clamp: bool) -> Vec<FilterParams> {
    let scale = frame.scale;
    let space = frame.space;
    let inverse = invert(space).unwrap_or([0.0; 6]);
    let mut f = FilterParams {
        origin: [frame.origin[0], frame.origin[1], scale, 0.0],
        subregion: p.subregion,
        space: [space[0], space[1], space[2], space[3]],
        space_t: [space[4], space[5], 0.0, 0.0],
        inverse: [inverse[0], inverse[1], inverse[2], inverse[3]],
        inverse_t: [inverse[4], inverse[5], 0.0, 0.0],
        ..Default::default()
    };
    let mut flags = if p.srgb { FLAG_SRGB } else { 0 };
    if clamp {
        flags |= FLAG_CLAMP;
    }
    let alpha = |i: UiFilterInput, flag| {
        if i == UiFilterInput::SourceAlpha {
            flag
        } else {
            0
        }
    };
    flags |= alpha(p.inputs[0], FLAG_IN1_ALPHA) | alpha(p.inputs[1], FLAG_IN2_ALPHA);
    let code = |f: &mut FilterParams, op: u32| f.op = [op, flags, 0, 0];
    // Two passes along the images of the space's axes: the second reads
    // the first's result, not the source.
    let axes = |f: FilterParams, x: [f32; 2], y: [f32; 2]| {
        let mut first = f;
        first.a[0] = x[0];
        first.a[1] = x[1];
        let mut second = f;
        second.a[0] = y[0];
        second.a[1] = y[1];
        second.op[1] = flags & (FLAG_SRGB | FLAG_CLAMP);
        vec![first, second]
    };
    match &p.op {
        UiFilterOp::Blur { std_dev } => {
            code(&mut f, 0);
            let x = frame.device([std_dev[0].max(0.0), 0.0]);
            let y = frame.device([0.0, std_dev[1].max(0.0)]);
            // A pass takes every texel within 3σ, up to BLUR_REACH a side;
            // a wider Gaussian is n passes of σ/√n (Gaussians convolve to
            // the Gaussian whose variance is their sum), each exact.
            let split = |v: [f32; 2]| {
                let n = (v[0].hypot(v[1]) * 3.0 / BLUR_REACH)
                    .powi(2)
                    .ceil()
                    .max(1.0);
                let k = n.sqrt().recip();
                (n as usize, [v[0] * k, v[1] * k])
            };
            let ((nx, x), (ny, y)) = (split(x), split(y));
            let mut out = Vec::with_capacity(nx + ny);
            for (k, v) in std::iter::repeat_n(x, nx)
                .chain(std::iter::repeat_n(y, ny))
                .enumerate()
            {
                let mut pass = f;
                pass.a[0] = v[0];
                pass.a[1] = v[1];
                // Only the first reads the input; the rest read a result.
                if k > 0 {
                    pass.op[1] = flags & (FLAG_SRGB | FLAG_CLAMP);
                }
                out.push(pass);
            }
            return out;
        }
        UiFilterOp::Morphology { radius, dilate } => {
            code(&mut f, 7);
            f.a[2] = f32::from(u8::from(*dilate));
            let x = frame.device([radius[0].max(0.0), 0.0]);
            let y = frame.device([0.0, radius[1].max(0.0)]);
            return axes(f, x, y);
        }
        UiFilterOp::Offset { d } => {
            code(&mut f, 1);
            let [x, y] = frame.device(*d);
            f.a = [x, y, 0.0, 0.0];
        }
        UiFilterOp::Flood { color } => {
            code(&mut f, 2);
            let [r, g, b, a] = *color;
            f.a = [r * a, g * a, b * a, a];
        }
        UiFilterOp::Composite { op, k } => {
            code(&mut f, 3);
            f.a = *k;
            f.b[0] = *op as u32 as f32;
        }
        UiFilterOp::Blend(mode) => {
            code(&mut f, 4);
            f.b[0] = blend_code(*mode);
        }
        UiFilterOp::ColorMatrix(m) => {
            code(&mut f, 5);
            for r in 0..4 {
                f.m[r] = [m[r * 5], m[r * 5 + 1], m[r * 5 + 2], m[r * 5 + 3]];
                f.table[0][r] = m[r * 5 + 4];
            }
        }
        UiFilterOp::ComponentTransfer(funcs) => {
            code(&mut f, 6);
            for (ch, fun) in funcs.iter().enumerate() {
                let (kind, args, values): (f32, [f32; 3], &[f32]) = match fun {
                    UiTransfer::Identity => (0.0, [0.0; 3], &[]),
                    UiTransfer::Table(v) => (1.0, [0.0; 3], v),
                    UiTransfer::Discrete(v) => (2.0, [0.0; 3], v),
                    UiTransfer::Linear { slope, intercept } => {
                        (3.0, [*slope, *intercept, 0.0], &[])
                    }
                    UiTransfer::Gamma {
                        amplitude,
                        exponent,
                        offset,
                    } => (4.0, [*amplitude, *exponent, *offset], &[]),
                };
                f.m[ch] = [kind, args[0], args[1], args[2]];
                let n = values.len().min(16);
                f.m[4][ch] = n as f32;
                for (k, v) in values.iter().take(16).enumerate() {
                    let i = ch * 16 + k;
                    f.table[i / 4][i % 4] = *v;
                }
            }
        }
        UiFilterOp::Displacement { scale: s, channels } => {
            code(&mut f, 8);
            f.op[2] = u32::from(channels[0].min(3));
            f.op[3] = u32::from(channels[1].min(3));
            f.a[0] = *s;
        }
        UiFilterOp::Turbulence {
            base_frequency,
            octaves,
            stitch,
            fractal,
            user,
            tile,
            ..
        } => {
            code(&mut f, 9);
            if *stitch {
                f.op[1] |= FLAG_STITCH;
            }
            if *fractal {
                f.op[1] |= FLAG_FRACTAL;
            }
            f.a = [base_frequency[0], base_frequency[1], *octaves as f32, 0.0];
            f.b = [user[0], user[1], user[2], 0.0];
            f.c = *tile;
        }
        UiFilterOp::Image { uv, rect, .. } => {
            code(&mut f, 10);
            f.a = *rect;
            f.b = *uv;
        }
        // Written by `prepare`, which knows where the layer is.
        UiFilterOp::Layer { .. } => code(&mut f, OP_LAYER),
        UiFilterOp::Tile { tile } => {
            code(&mut f, 11);
            f.a = *tile;
        }
        UiFilterOp::Lighting {
            specular,
            surface_scale,
            constant,
            exponent,
            color,
            light,
            kernel_unit,
        } => {
            code(&mut f, if *specular { 13 } else { 12 });
            let (cells, per) = frame.cells(*kernel_unit);
            // The surface's height is surface scale × alpha in cells: a
            // cell of a given unit is that many units tall.
            let tall = kernel_unit.map_or(1.0, |[x, y]| (x * y).abs().sqrt());
            f.a = [*surface_scale, *constant, *exponent, 0.0];
            f.m[0] = [color[0], color[1], color[2], 1.0];
            f.m[1] = [-2.0, tall, 0.0, 0.0];
            f.m[2] = cells;
            f.m[3] = [per[0] * tall, per[1] * tall, 0.0, 0.0];
            match light {
                UiLight::Distant { azimuth, elevation } => {
                    f.b = [*azimuth, *elevation, 0.0, 0.0];
                }
                UiLight::Point { at } => {
                    f.a[3] = 1.0;
                    f.b = [at.x, at.y, at.z, 0.0];
                }
                UiLight::Spot {
                    at,
                    points_at,
                    exponent,
                    cone,
                } => {
                    f.a[3] = 2.0;
                    f.b = [at.x, at.y, at.z, 0.0];
                    f.c = [points_at.x, points_at.y, points_at.z, *exponent];
                    if let Some(c) = cone {
                        f.m[1][0] = c.cos();
                    }
                }
            }
        }
        UiFilterOp::Merge => code(&mut f, 14),
        UiFilterOp::Convolve {
            order,
            kernel,
            divisor,
            bias,
            target,
            edge,
            preserve_alpha,
            kernel_unit,
        } => {
            code(&mut f, 15);
            let n = kernel.len().min(64);
            let order = if (order[0] * order[1]) as usize == n {
                *order
            } else {
                [n as u32, 1]
            };
            f.op[2] = order[0];
            f.op[3] = order[1];
            let divisor = if *divisor == 0.0 { 1.0 } else { *divisor };
            f.a = [divisor, *bias, target[0] as f32, target[1] as f32];
            f.b = [
                f32::from(*edge),
                f32::from(u8::from(*preserve_alpha)),
                0.0,
                0.0,
            ];
            f.m[2] = frame.cells(*kernel_unit).0;
            for (k, v) in kernel.iter().take(64).enumerate() {
                f.table[k / 4][k % 4] = *v;
            }
        }
    }
    vec![f]
}

/// feTurbulence's lattice for `seed` (the reference implementation's
/// init, Filter Effects 1 §15.24) as the 514 × 4 texels ui_filter.wgsl
/// reads: channel k's gradient at i in xy, the lattice selector in z.
fn lattice(seed: f32) -> Vec<[f32; 4]> {
    const B: usize = 0x100;
    const RAND_M: i64 = 2147483647;
    const RAND_A: i64 = 16807;
    const RAND_Q: i64 = 127773;
    const RAND_R: i64 = 2836;
    let random = |s: i64| {
        let r = RAND_A * (s % RAND_Q) - RAND_R * (s / RAND_Q);
        if r <= 0 { r + RAND_M } else { r }
    };
    let mut s = seed.round() as i64;
    if s <= 0 {
        s = -(s % (RAND_M - 1)) + 1;
    }
    if s > RAND_M - 1 {
        s = RAND_M - 1;
    }
    let mut selector = [0usize; B + B + 2];
    let mut gradient = [[[0.0f64; 2]; B + B + 2]; 4];
    for g in &mut gradient {
        for (i, v) in g.iter_mut().enumerate().take(B) {
            selector[i] = i;
            for c in v.iter_mut() {
                s = random(s);
                *c = ((s % (B + B) as i64) - B as i64) as f64 / B as f64;
            }
            let n = (v[0] * v[0] + v[1] * v[1]).sqrt();
            if n > 0.0 {
                v[0] /= n;
                v[1] /= n;
            }
        }
    }
    let mut i = B;
    while {
        i -= 1;
        i > 0
    } {
        let k = selector[i];
        s = random(s);
        let j = (s % B as i64) as usize;
        selector[i] = selector[j];
        selector[j] = k;
    }
    for i in 0..B + 2 {
        selector[B + i] = selector[i];
        for g in &mut gradient {
            g[B + i] = g[i];
        }
    }
    let mut out = vec![[0.0; 4]; (B + B + 2) * 4];
    for (k, g) in gradient.iter().enumerate() {
        for i in 0..B + B + 2 {
            out[k * (B + B + 2) + i] = [g[i][0] as f32, g[i][1] as f32, selector[i] as f32, 0.0];
        }
    }
    out
}

/// A pooled result target.
struct Pooled {
    size: (u32, u32),
    view: wgpu::TextureView,
}

/// One pass of a prepared filter: its group and the pooled target it
/// draws into.
struct Pass {
    group: wgpu::BindGroup,
    target: usize,
}

/// A layer's filter this frame: its passes in order and the view of its
/// result.
pub(crate) struct Prepared {
    passes: Vec<Pass>,
    pub result: wgpu::TextureView,
}

/// What a layer's filter reads as its source.
pub(crate) enum Source<'a> {
    /// What the layer drew.
    Drawn(&'a wgpu::TextureView),
    /// What is under a backdrop layer.
    Capture(Capture<'a>),
}

/// The target a backdrop layer's composite draws into, as drawn so far:
/// its view, its top left (CSS px), its device px per CSS px, and whether
/// it holds sRGB-encoded values (a non-sRGB output, which the tonemap
/// encodes).
pub(crate) struct Capture<'a> {
    pub view: &'a wgpu::TextureView,
    pub origin: [f32; 2],
    pub scale: f32,
    pub decode: bool,
}

/// What a filter's primitives read besides their inputs: `white` for a
/// texture a pass does not read, an image's texture by handle, and a
/// layer drawn this frame by index (its target's view, top left in CSS
/// px, and device px per CSS px).
pub(crate) struct Inputs<'a> {
    pub white: &'a wgpu::TextureView,
    pub texture_view: &'a dyn Fn(TextureHandle) -> Option<wgpu::TextureView>,
    pub layer: &'a dyn Fn(u32) -> Option<(wgpu::TextureView, [f32; 2], f32)>,
}

/// The filter pipeline, its result targets and parameters, and the
/// turbulence lattices by seed.
pub(crate) struct FilterRunner {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    pool: Vec<Pooled>,
    used: Vec<bool>,
    uniforms: Vec<wgpu::Buffer>,
    next_uniform: usize,
    lattices: Vec<(u32, wgpu::TextureView)>,
}

const FILTER_WGSL: &str = wgsl_library!("ui_filter.wgsl");

impl FilterRunner {
    pub(crate) fn new(device: &wgpu::Device) -> FilterRunner {
        let tex = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ui filter"),
            entries: &[
                tex(0),
                tex(1),
                tex(2),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
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
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ui filter"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ui filter"),
            source: wgpu::ShaderSource::Wgsl(source().into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ui filter"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_filter"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_filter"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: super::pass::LAYER_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        FilterRunner {
            pipeline,
            layout,
            pool: Vec::new(),
            used: Vec::new(),
            uniforms: Vec::new(),
            next_uniform: 0,
            lattices: Vec::new(),
        }
    }

    /// Frees every target and parameter buffer for a new frame.
    pub(crate) fn begin(&mut self) {
        self.used = vec![false; self.pool.len()];
        self.next_uniform = 0;
        // Targets no frame used are let go when there are many.
        if self.pool.len() > 32 {
            self.pool.truncate(32);
            self.used.truncate(32);
        }
    }

    fn take(&mut self, device: &wgpu::Device, size: (u32, u32)) -> usize {
        if let Some(i) = (0..self.pool.len()).find(|&i| !self.used[i] && self.pool[i].size == size)
        {
            self.used[i] = true;
            return i;
        }
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ui filter result"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: super::pass::LAYER_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        self.pool.push(Pooled { size, view });
        self.used.push(true);
        self.pool.len() - 1
    }

    fn lattice(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        seed: f32,
    ) -> wgpu::TextureView {
        let key = seed.round().to_bits();
        if let Some((_, v)) = self.lattices.iter().find(|(k, _)| *k == key) {
            return v.clone();
        }
        let texels = lattice(seed);
        let texture = wgpu::util::DeviceExt::create_texture_with_data(
            device,
            queue,
            &wgpu::TextureDescriptor {
                label: Some("ui turbulence lattice"),
                size: wgpu::Extent3d {
                    width: 514,
                    height: 4,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            bytemuck::cast_slice(&texels),
        );
        let view = texture.create_view(&Default::default());
        if self.lattices.len() >= 8 {
            self.lattices.remove(0);
        }
        self.lattices.push((key, view.clone()));
        view
    }

    fn uniform(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        params: &FilterParams,
    ) -> usize {
        if self.next_uniform == self.uniforms.len() {
            self.uniforms
                .push(device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("ui filter"),
                    size: std::mem::size_of::<FilterParams>() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }));
        }
        self.next_uniform += 1;
        queue.write_buffer(
            &self.uniforms[self.next_uniform - 1],
            0,
            bytemuck::bytes_of(params),
        );
        self.next_uniform - 1
    }

    /// Writes a layer's passes: for a backdrop (`source` a [`Source::Capture`])
    /// the copy of what is under it first, then `filter`'s over it or over
    /// what the layer drew, for a layer of `size` device px at `origin`
    /// (CSS px) and `scale`. `None` when there is nothing to run.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        filter: Option<&super::UiFilter>,
        source: Source<'_>,
        size: (u32, u32),
        origin: [f32; 2],
        scale: f32,
        inputs: &Inputs<'_>,
    ) -> Option<Prepared> {
        let mut passes = Vec::new();
        let clamp = matches!(source, Source::Capture(_));
        let source = match source {
            Source::Drawn(v) if filter.is_some_and(|f| !f.primitives.is_empty()) => v.clone(),
            Source::Drawn(_) => return None,
            Source::Capture(c) => {
                let params = FilterParams {
                    op: [OP_CAPTURE, FLAG_CLAMP, 0, 0],
                    origin: [origin[0], origin[1], scale, 0.0],
                    subregion: [-1.0e9, -1.0e9, 2.0e9, 2.0e9],
                    space: [1.0, 0.0, 0.0, 1.0],
                    inverse: [1.0, 0.0, 0.0, 1.0],
                    a: [
                        c.origin[0],
                        c.origin[1],
                        c.scale,
                        f32::from(u8::from(c.decode)),
                    ],
                    ..Default::default()
                };
                let out = self.take(device, size);
                let u = self.uniform(device, queue, &params);
                let group = crate::bind_group(
                    device,
                    "ui filter capture",
                    &self.layout,
                    &[
                        wgpu::BindingResource::TextureView(c.view),
                        wgpu::BindingResource::TextureView(inputs.white),
                        wgpu::BindingResource::TextureView(inputs.white),
                        self.uniforms[u].as_entire_binding(),
                    ],
                );
                passes.push(Pass { group, target: out });
                self.pool[out].view.clone()
            }
        };
        let primitives = filter.map_or(&[][..], |f| &f.primitives);
        let frame = Frame {
            origin,
            scale,
            space: filter.map_or(super::UiFilter::CSS, |f| f.space),
        };
        // A space that cannot be inverted draws nothing.
        let draws = invert(frame.space).is_some();
        let mut results: Vec<usize> = Vec::new();
        for p in primitives.iter().filter(|_| draws) {
            let input = |i: UiFilterInput, results: &[usize], pool: &[Pooled]| match i {
                UiFilterInput::Result(k) if (k as usize) < results.len() => {
                    pool[results[k as usize]].view.clone()
                }
                _ => source.clone(),
            };
            let mut steps = self::passes(p, &frame, clamp);
            let aux = match &p.op {
                UiFilterOp::Turbulence { seed, .. } => self.lattice(device, queue, *seed),
                UiFilterOp::Image { texture, .. } => {
                    (inputs.texture_view)(*texture).unwrap_or_else(|| inputs.white.clone())
                }
                UiFilterOp::Layer { layer } => match (inputs.layer)(*layer) {
                    Some((view, at, s)) => {
                        steps[0].a = [at[0], at[1], s, 0.0];
                        view
                    }
                    // Not drawn: transparent.
                    None => inputs.white.clone(),
                },
                _ => inputs.white.clone(),
            };
            let mut in1 = input(p.inputs[0], &results, &self.pool);
            let in2 = input(p.inputs[1], &results, &self.pool);
            let last = steps.len() - 1;
            let mut out = 0;
            for (n, params) in steps.into_iter().enumerate() {
                out = self.take(device, size);
                let u = self.uniform(device, queue, &params);
                let group = crate::bind_group(
                    device,
                    "ui filter",
                    &self.layout,
                    &[
                        wgpu::BindingResource::TextureView(&in1),
                        wgpu::BindingResource::TextureView(&in2),
                        wgpu::BindingResource::TextureView(&aux),
                        self.uniforms[u].as_entire_binding(),
                    ],
                );
                passes.push(Pass { group, target: out });
                if n < last {
                    in1 = self.pool[out].view.clone();
                }
            }
            results.push(out);
        }
        let result = match (results.last(), passes.last()) {
            (Some(&r), _) => self.pool[r].view.clone(),
            // A capture with no primitives (or a space that draws
            // nothing: transparent) composites what it captured.
            (None, Some(p)) if draws => self.pool[p.target].view.clone(),
            (None, _) => {
                let out = self.take(device, size);
                let params = FilterParams {
                    op: [2, 0, 0, 0],
                    origin: [origin[0], origin[1], scale, 0.0],
                    ..Default::default()
                };
                let u = self.uniform(device, queue, &params);
                let group = crate::bind_group(
                    device,
                    "ui filter",
                    &self.layout,
                    &[
                        wgpu::BindingResource::TextureView(inputs.white),
                        wgpu::BindingResource::TextureView(inputs.white),
                        wgpu::BindingResource::TextureView(inputs.white),
                        self.uniforms[u].as_entire_binding(),
                    ],
                );
                passes.push(Pass { group, target: out });
                self.pool[out].view.clone()
            }
        };
        Some(Prepared { passes, result })
    }

    /// Records a prepared filter's passes.
    pub(crate) fn record(&self, encoder: &mut crate::Encoder, prepared: &Prepared) -> u32 {
        for pass in &prepared.passes {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ui filter"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.pool[pass.target].view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            rp.set_pipeline(&self.pipeline);
            rp.set_bind_group(0, &pass.group, &[]);
            rp.draw(0..3, 0..1);
        }
        prepared.passes.len() as u32
    }
}

/// The filter module: the blend modes, then the passes.
pub(crate) fn source() -> String {
    format!("{}{}", super::UI_BLEND_MODES_WGSL, FILTER_WGSL)
}
