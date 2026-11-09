//! SVG filter graphs over UI layers (Filter Effects 1 §9-15): a
//! [`UiFilter`] on a [`super::UiLayer`] runs its primitives over what the
//! layer drew (SourceGraphic) once it is drawn, each a full-target pass
//! into a result of the layer's size (`shaders/ui_filter.wgsl`), and the
//! layer composites the last result instead of what it drew. The layer's
//! rect is the filter region. Results are premultiplied linear light,
//! as the layers are (`color-interpolation-filters: linearRGB`); a
//! primitive marked `srgb` works on sRGB-encoded values.

use super::UiBlend;
use crate::TextureHandle;
use og_core::glam::Vec3;

/// A filter: its primitives in order, each reading the source or earlier
/// results; the last one's result is the filter's.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiFilter {
    pub primitives: Vec<UiFilterPrimitive>,
}

/// One primitive: what it does, its inputs (`in`, `in2`), its subregion
/// (`[x, y, width, height]`, CSS px; outside it the result is
/// transparent) and whether it works on sRGB-encoded values.
#[derive(Clone, Debug, PartialEq)]
pub struct UiFilterPrimitive {
    pub op: UiFilterOp,
    pub inputs: [UiFilterInput; 2],
    pub subregion: [f32; 4],
    pub srgb: bool,
}

/// What a primitive reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiFilterInput {
    /// What the layer drew (SourceGraphic).
    #[default]
    Source,
    /// Its alpha, black (SourceAlpha).
    SourceAlpha,
    /// An earlier primitive's result, by index.
    Result(u32),
}

/// `feComposite`'s operators.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiFilterComposite {
    #[default]
    Over,
    In,
    Out,
    Atop,
    Xor,
    Lighter,
    /// k1·i1·i2 + k2·i1 + k3·i2 + k4.
    Arithmetic,
}

/// One channel's `feFunc*`.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum UiTransfer {
    #[default]
    Identity,
    /// Linear between the values (at most 16).
    Table(Vec<f32>),
    /// Steps of the values (at most 16).
    Discrete(Vec<f32>),
    Linear {
        slope: f32,
        intercept: f32,
    },
    Gamma {
        amplitude: f32,
        exponent: f32,
        offset: f32,
    },
}

/// A lighting primitive's light, CSS px (z above the surface).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiLight {
    /// Radians.
    Distant {
        azimuth: f32,
        elevation: f32,
    },
    Point {
        at: Vec3,
    },
    Spot {
        at: Vec3,
        points_at: Vec3,
        exponent: f32,
        /// The limiting cone's half angle, radians.
        cone: Option<f32>,
    },
}

/// What a primitive does. Lengths are CSS px.
#[derive(Clone, Debug, PartialEq)]
pub enum UiFilterOp {
    /// `feGaussianBlur`: the standard deviation along x and y.
    Blur { std_dev: [f32; 2] },
    /// `feOffset`.
    Offset { d: [f32; 2] },
    /// `feFlood`: straight linear RGBA.
    Flood { color: [f32; 4] },
    /// `feComposite` of `in` over `in2` by `op`, `k` for arithmetic.
    Composite { op: UiFilterComposite, k: [f32; 4] },
    /// `feBlend` of `in` over `in2`.
    Blend(UiBlend),
    /// `feColorMatrix` as a 4×5 matrix, rows R, G, B, A.
    ColorMatrix([f32; 20]),
    /// `feComponentTransfer`: R, G, B, A.
    ComponentTransfer(Box<[UiTransfer; 4]>),
    /// `feMorphology`.
    Morphology { radius: [f32; 2], dilate: bool },
    /// `feDisplacementMap` of `in` by `in2`'s channels (0 R … 3 A).
    Displacement { scale: f32, channels: [u8; 2] },
    /// `feTurbulence` at user-space points: `user` maps a CSS point to
    /// them (origin x, y in CSS px, user units per CSS px); `tile` is the
    /// stitch tile in user units.
    Turbulence {
        base_frequency: [f32; 2],
        octaves: u32,
        seed: f32,
        stitch: bool,
        fractal: bool,
        user: [f32; 3],
        tile: [f32; 4],
    },
    /// `feImage` of a texture's `uv` rect over `rect`.
    Image {
        texture: TextureHandle,
        uv: [f32; 4],
        rect: [f32; 4],
    },
    /// `feTile` of `in`, whose subregion is `tile`.
    Tile { tile: [f32; 4] },
    /// `feDiffuseLighting` (`specular` false: `constant` is kd) or
    /// `feSpecularLighting` (ks and its exponent), lit by `light` in
    /// linear `color`, from `in`'s alpha as a surface.
    Lighting {
        specular: bool,
        surface_scale: f32,
        constant: f32,
        exponent: f32,
        color: [f32; 3],
        light: UiLight,
    },
    /// One step of `feMerge`: `in` over `in2`.
    Merge,
    /// `feConvolveMatrix`: `order` columns and rows (at most 64 cells),
    /// the kernel row-major, one device px a cell; `edge` 0 duplicate, 1
    /// wrap, 2 none.
    Convolve {
        order: [u32; 2],
        kernel: Vec<f32>,
        divisor: f32,
        bias: f32,
        target: [i32; 2],
        edge: u8,
        preserve_alpha: bool,
    },
}

/// `struct FilterParams` in ui_filter.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct FilterParams {
    op: [u32; 4],
    origin: [f32; 4],
    subregion: [f32; 4],
    a: [f32; 4],
    b: [f32; 4],
    c: [f32; 4],
    m: [[f32; 4]; 5],
    table: [[f32; 4]; 16],
}

const FLAG_SRGB: u32 = 1;
const FLAG_IN1_ALPHA: u32 = 2;
const FLAG_IN2_ALPHA: u32 = 4;
const FLAG_Y: u32 = 8;
const FLAG_STITCH: u32 = 16;
const FLAG_FRACTAL: u32 = 32;

/// UiBlend's code in the shaders (src/ui/build.rs).
fn blend_code(b: UiBlend) -> f32 {
    b as u32 as f32
}

/// The parameters of each pass of a primitive (two for a blur or a
/// morphology, one axis each) at a layer's `origin` (CSS px) and `scale`
/// (device px per CSS px).
fn passes(p: &UiFilterPrimitive, origin: [f32; 2], scale: f32) -> Vec<FilterParams> {
    let mut f = FilterParams {
        origin: [origin[0], origin[1], scale, 0.0],
        subregion: p.subregion,
        ..Default::default()
    };
    let mut flags = if p.srgb { FLAG_SRGB } else { 0 };
    let alpha = |i: UiFilterInput, flag| {
        if i == UiFilterInput::SourceAlpha {
            flag
        } else {
            0
        }
    };
    flags |= alpha(p.inputs[0], FLAG_IN1_ALPHA) | alpha(p.inputs[1], FLAG_IN2_ALPHA);
    let code = |f: &mut FilterParams, op: u32| f.op = [op, flags, 0, 0];
    match &p.op {
        UiFilterOp::Blur { std_dev } => {
            code(&mut f, 0);
            f.a = [std_dev[0] * scale, std_dev[1] * scale, 0.0, 0.0];
            let mut y = f;
            // The second pass reads the first's result, not the source.
            y.op[1] = (flags & FLAG_SRGB) | FLAG_Y;
            return vec![f, y];
        }
        UiFilterOp::Morphology { radius, dilate } => {
            code(&mut f, 7);
            f.a = [
                radius[0] * scale,
                radius[1] * scale,
                f32::from(u8::from(*dilate)),
                0.0,
            ];
            let mut y = f;
            y.op[1] = (flags & FLAG_SRGB) | FLAG_Y;
            return vec![f, y];
        }
        UiFilterOp::Offset { d } => {
            code(&mut f, 1);
            f.a = [d[0] * scale, d[1] * scale, 0.0, 0.0];
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
            f.a[0] = s * scale;
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
        } => {
            code(&mut f, if *specular { 13 } else { 12 });
            // The surface in device px: its slopes and heights scale.
            f.a = [surface_scale * scale, *constant, *exponent, 0.0];
            let device = |v: Vec3| {
                [
                    (v.x - origin[0]) * scale,
                    (v.y - origin[1]) * scale,
                    v.z * scale,
                ]
            };
            f.m[0] = [color[0], color[1], color[2], 1.0];
            f.m[1][0] = -2.0;
            match light {
                UiLight::Distant { azimuth, elevation } => {
                    f.b = [*azimuth, *elevation, 0.0, 0.0];
                }
                UiLight::Point { at } => {
                    f.a[3] = 1.0;
                    let [x, y, z] = device(*at);
                    f.b = [x, y, z, 0.0];
                }
                UiLight::Spot {
                    at,
                    points_at,
                    exponent,
                    cone,
                } => {
                    f.a[3] = 2.0;
                    let [x, y, z] = device(*at);
                    f.b = [x, y, z, 0.0];
                    let [x, y, z] = device(*points_at);
                    f.c = [x, y, z, *exponent];
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

const FILTER_WGSL: &str = wgsl!("ui_filter.wgsl");

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

    /// Writes `filter`'s passes over a layer drawn into `source`, of
    /// `size` device px at `origin` (CSS px) and `scale`. `white` stands
    /// for a texture a pass does not read; `texture_view` finds an
    /// image's.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        filter: &super::UiFilter,
        source: &wgpu::TextureView,
        size: (u32, u32),
        origin: [f32; 2],
        scale: f32,
        white: &wgpu::TextureView,
        texture_view: &dyn Fn(TextureHandle) -> Option<wgpu::TextureView>,
    ) -> Option<Prepared> {
        if filter.primitives.is_empty() {
            return None;
        }
        let mut results: Vec<usize> = Vec::new();
        let mut passes = Vec::new();
        for p in &filter.primitives {
            let input = |i: UiFilterInput, results: &[usize], pool: &[Pooled]| match i {
                UiFilterInput::Result(k) if (k as usize) < results.len() => {
                    pool[results[k as usize]].view.clone()
                }
                _ => source.clone(),
            };
            let aux = match &p.op {
                UiFilterOp::Turbulence { seed, .. } => self.lattice(device, queue, *seed),
                UiFilterOp::Image { texture, .. } => {
                    texture_view(*texture).unwrap_or_else(|| white.clone())
                }
                _ => white.clone(),
            };
            let mut in1 = input(p.inputs[0], &results, &self.pool);
            let in2 = input(p.inputs[1], &results, &self.pool);
            let steps = self::passes(p, origin, scale);
            let last = steps.len() - 1;
            let mut out = 0;
            for (n, params) in steps.into_iter().enumerate() {
                out = self.take(device, size);
                if self.next_uniform == self.uniforms.len() {
                    self.uniforms
                        .push(device.create_buffer(&wgpu::BufferDescriptor {
                            label: Some("ui filter"),
                            size: std::mem::size_of::<FilterParams>() as u64,
                            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                            mapped_at_creation: false,
                        }));
                }
                let buffer = &self.uniforms[self.next_uniform];
                self.next_uniform += 1;
                queue.write_buffer(buffer, 0, bytemuck::bytes_of(&params));
                let group = crate::bind_group(
                    device,
                    "ui filter",
                    &self.layout,
                    &[
                        wgpu::BindingResource::TextureView(&in1),
                        wgpu::BindingResource::TextureView(&in2),
                        wgpu::BindingResource::TextureView(&aux),
                        buffer.as_entire_binding(),
                    ],
                );
                passes.push(Pass { group, target: out });
                if n < last {
                    in1 = self.pool[out].view.clone();
                }
            }
            results.push(out);
        }
        Some(Prepared {
            passes,
            result: self.pool[*results.last()?].view.clone(),
        })
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
