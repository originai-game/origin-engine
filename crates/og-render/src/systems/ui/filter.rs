//! SVG filter graphs over UI layers (Filter Effects 1 §9-15): a
//! [`UiFilter`] on a [`super::UiLayer`] runs its primitives over what the
//! layer drew (SourceGraphic) once it is drawn, or over what lies under it
//! for a backdrop layer ([`super::UiLayer::backdrop`]), each a full-target
//! pass into a result of the layer's size (`shaders/ui_filter.wgsl`; a
//! wide blur's inner passes run on its input halved), and
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
    /// Used pixels: in1 (xy), an element-layer auxiliary input (zw).
    extent: [f32; 4],
    /// x: in1 px per px of the pass's target; y: the pass's level, its
    /// target the layer's size halved that many times (read by
    /// `prepare`, which writes x); zw: in2's used pixels (in2 is at the
    /// layer's size).
    res: [f32; 4],
}

const FLAG_SRGB: u32 = 1;
const FLAG_IN1_ALPHA: u32 = 2;
const FLAG_IN2_ALPHA: u32 = 4;
const FLAG_STITCH: u32 = 16;
const FLAG_FRACTAL: u32 = 32;
/// Reads past an input's edge take its edge (a backdrop: what is under
/// the element goes on past what was captured of it).
const FLAG_CLAMP: u32 = 64;
/// The pass then runs the colour matrix in `m` on its result (a blur's
/// last pass, for the matrix after it).
const FLAG_MATRIX: u32 = 128;

/// The widest deviation a blur pass runs at (device px of its level): a
/// wider one runs on its input halved until it is this or less, as Skia
/// blurs (ui_filter.wgsl's case 0 reads 3 deviations a side).
const BLUR_SIGMA: f32 = 4.0;
/// The most times a blur halves its input.
const BLUR_LEVELS: f32 = 8.0;
/// The widest deviation a drop shadow's alpha is blurred at: its soft
/// edge halved sooner than a picture.
const SHADOW_SIGMA: f32 = 2.0;
/// The most times a filter's source is halved as it is drawn or captured
/// ([`source_level`]).
const SOURCE_LEVELS: u32 = 2;

/// A layer of `size` device px halved `level` times.
fn shrink(size: (u32, u32), level: u32) -> (u32, u32) {
    let k = (1u32 << level) - 1;
    (
        ((size.0 + k) >> level).max(1),
        ((size.1 + k) >> level).max(1),
    )
}

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

/// A pass of a primitive at `level` (its target the layer's size halved
/// that many times): op `op` with `a` its first two parameters, its input
/// read with `flags` only by the primitive's first pass, and drawn
/// everywhere (the caller cuts its last pass to the subregion).
fn step(f: FilterParams, flags: u32, op: u32, a: [f32; 2], level: u32) -> FilterParams {
    let mut pass = f;
    pass.op = [op, flags, 0, 0];
    pass.a = [a[0], a[1], 0.0, 0.0];
    pass.origin[2] = f.origin[2] / (1u32 << level) as f32;
    pass.res[1] = level as f32;
    pass.subregion = [-1.0e9, -1.0e9, 2.0e9, 2.0e9];
    pass
}

/// How many times a blur of deviations `std_dev` (the space's units)
/// halves its input in `frame`: until the wider is `most` device px or
/// less (BLUR_SIGMA, or SHADOW_SIGMA for a drop shadow's alpha).
fn blur_levels(frame: &Frame, std_dev: [f32; 2], most: f32) -> u32 {
    let x = frame.device([std_dev[0].max(0.0), 0.0]);
    let y = frame.device([0.0, std_dev[1].max(0.0)]);
    let wide = x[0].hypot(x[1]).max(y[0].hypot(y[1]));
    (wide / most).log2().ceil().clamp(0.0, BLUR_LEVELS) as u32
}

/// The level a filter's source may be drawn or captured at (the layer's
/// size halved that many times, at most SOURCE_LEVELS): its blur's first
/// halvings, when a wide blur is the one primitive that reads it; drawn
/// with its edges' coverage there, or captured as each texel's box, it is
/// what those halvings would make. 0 otherwise.
pub(crate) fn source_level(filter: &super::UiFilter, scale: f32) -> u32 {
    let p = &filter.primitives;
    let frame = Frame {
        origin: [0.0; 2],
        scale,
        space: filter.space,
    };
    match p.first() {
        Some(UiFilterPrimitive {
            op: UiFilterOp::Blur { std_dev },
            inputs: [UiFilterInput::Source, _],
            ..
        }) if !p[1..].iter().any(|p| {
            p.inputs[0] == UiFilterInput::Source
                || (reads_in2(p) && p.inputs[1] == UiFilterInput::Source)
        }) =>
        {
            blur_levels(&frame, *std_dev, BLUR_SIGMA).min(SOURCE_LEVELS)
        }
        _ => 0,
    }
}

/// Whether `p` works per pixel on its `in` only (it runs at the level its
/// input is at).
fn pointwise(p: &UiFilterPrimitive) -> bool {
    matches!(
        p.op,
        UiFilterOp::ColorMatrix(_) | UiFilterOp::ComponentTransfer(_)
    )
}

/// Writes colour matrix `m` (rows R, G, B, A of 5) where a pass reads it.
fn matrix(f: &mut FilterParams, m: &[f32; 20]) {
    for r in 0..4 {
        f.m[r] = [m[r * 5], m[r * 5 + 1], m[r * 5 + 2], m[r * 5 + 3]];
        f.table[0][r] = m[r * 5 + 4];
    }
}

/// Whether `p` reads its `in2`.
fn reads_in2(p: &UiFilterPrimitive) -> bool {
    matches!(
        p.op,
        UiFilterOp::Composite { .. }
            | UiFilterOp::Blend(_)
            | UiFilterOp::Displacement { .. }
            | UiFilterOp::DropShadow { .. }
            | UiFilterOp::Merge
    )
}

/// Whether blur `i` of `primitives` leaves its result at the level it ran
/// at, for a reader that scales it back bilinearly as it reads: the
/// filter's last primitive, or the one before a last that works per pixel
/// on it alone (which runs at that level too).
fn kept(primitives: &[UiFilterPrimitive], i: usize) -> bool {
    let n = primitives.len();
    matches!(primitives[i].op, UiFilterOp::Blur { .. })
        && (i + 1 == n
            || (i + 2 == n
                && pointwise(&primitives[n - 1])
                && primitives[n - 1].inputs[0] == UiFilterInput::Result(i as u32)))
}

/// The level of `filter`'s result in a frame at `scale` device px per CSS
/// px when its reader takes one below the layer's size
/// ([`FilterRunner::prepare`] with `low`): the layer's size halved that
/// many times.
pub(crate) fn result_level(filter: &super::UiFilter, scale: f32) -> u32 {
    let frame = Frame {
        origin: [0.0; 2],
        scale,
        space: filter.space,
    };
    let p = &filter.primitives;
    let blur = |i: usize| match p[i].op {
        UiFilterOp::Blur { std_dev } if kept(p, i) => blur_levels(&frame, std_dev, BLUR_SIGMA),
        _ => 0,
    };
    match p.len() {
        0 => 0,
        n if n > 1 && pointwise(&p[n - 1]) => blur(n - 2),
        n => blur(n - 1),
    }
}

/// A Gaussian of deviations `std_dev` (the space's units) along the
/// space's axes, as Skia blurs: the input halved until the wider
/// deviation is BLUR_SIGMA or less (each half a 2×2 box, one bilinear read
/// a texel), then the Gaussian's two passes there, its result at that
/// level for the next pass to read bilinearly. The boxes and that read
/// blur too: their variance (a box of two texels s px apart s²/4, a
/// bilinear scale by k k²/6) comes off the Gaussian's, so the whole is
/// the deviation asked for. The first pass reads the input with `flags`;
/// an input already at level `from` (a backdrop captured halved) is not
/// halved again to it.
fn blur(
    f: FilterParams,
    flags: u32,
    frame: &Frame,
    std_dev: [f32; 2],
    (from, most): (u32, f32),
) -> Vec<FilterParams> {
    let x = frame.device([std_dev[0].max(0.0), 0.0]);
    let y = frame.device([0.0, std_dev[1].max(0.0)]);
    let len = |v: [f32; 2]| v[0].hypot(v[1]);
    let levels = blur_levels(frame, std_dev, most).max(from);
    let k = (1u32 << levels) as f32;
    let extra = if levels == 0 {
        0.0
    } else {
        (1..=levels)
            .map(|l| (1u32 << (2 * (l - 1))) as f32 / 4.0)
            .sum::<f32>()
            + k * k / 6.0
    };
    let narrowed = |v: [f32; 2]| {
        let l = len(v);
        let r = if l > 0.0 {
            (l * l - extra).max(0.0).sqrt() / l / k
        } else {
            0.0
        };
        [v[0] * r, v[1] * r]
    };
    let rest = flags & (FLAG_SRGB | FLAG_CLAMP);
    let mut out: Vec<FilterParams> = (from + 1..=levels)
        .map(|l| step(f, rest, 1, [0.0; 2], l))
        .collect();
    out.push(step(f, rest, 0, narrowed(x), levels));
    out.push(step(f, rest, 0, narrowed(y), levels));
    out[0].op[1] = flags;
    out
}

/// The parameters of each pass of a primitive (two for a blur or a
/// morphology, one axis of the space each) in `frame`, every read past
/// an input's edge taking the edge when `clamp`.
fn passes(
    p: &UiFilterPrimitive,
    frame: &Frame,
    clamp: bool,
    keep: bool,
    from: u32,
) -> Vec<FilterParams> {
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
            let mut out = blur(f, flags, frame, *std_dev, (from, BLUR_SIGMA));
            let level = out.last().map_or(0.0, |s| s.res[1]);
            if level > 0.0 && !keep {
                // Back to the layer's size: an offset of none.
                out.push(step(f, flags & (FLAG_SRGB | FLAG_CLAMP), 1, [0.0; 2], 0));
            }
            if let Some(last) = out.last_mut() {
                last.subregion = p.subregion;
            }
            return out;
        }
        UiFilterOp::DropShadow { std_dev, d, color } => {
            // The blur of in's alpha where it ran, then one pass: that
            // moved, flooded and under in2 (in), its read the scale back.
            let mut out = blur(
                f,
                flags | FLAG_IN1_ALPHA,
                frame,
                *std_dev,
                (0, SHADOW_SIGMA),
            );
            let [x, y] = frame.device(*d);
            let [r, g, b, a] = *color;
            let mut last = step(f, flags & !FLAG_IN1_ALPHA, 18, [x, y], 0);
            last.b = [r * a, g * a, b * a, a];
            last.subregion = p.subregion;
            out.push(last);
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
            matrix(&mut f, m);
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
    view: wgpu::TextureView,
}

/// How many frames a pooled target stays unused before it is let go.
const POOL_IDLE_FRAMES: u8 = 8;

/// Things made by size and kept across frames: a frame takes what it
/// needs, gives back what it no longer reads, and makes one only when
/// none of its size is free, so a frame that needs what the last needed
/// makes nothing. One no frame took for [`POOL_IDLE_FRAMES`] is let go.
struct Pool<T> {
    slots: Vec<Slot<T>>,
}

struct Slot<T> {
    size: (u32, u32),
    item: T,
    /// Held now, and taken at all this frame (given back or not).
    used: bool,
    taken: bool,
    idle: u8,
}

impl<T> Pool<T> {
    const fn new() -> Self {
        Pool { slots: Vec::new() }
    }

    /// Frees everything for a new frame, letting go of what stood idle.
    fn begin(&mut self) {
        for s in &mut self.slots {
            s.idle = if s.taken { 0 } else { s.idle.saturating_add(1) };
            (s.used, s.taken) = (false, false);
        }
        self.slots.retain(|s| s.idle < POOL_IDLE_FRAMES);
    }

    /// A free one of `size`, made by `make` when there is none.
    fn take(&mut self, size: (u32, u32), make: impl FnOnce() -> T) -> usize {
        if let Some(i) = self.slots.iter().position(|s| !s.used && s.size == size) {
            (self.slots[i].used, self.slots[i].taken) = (true, true);
            return i;
        }
        self.slots.push(Slot {
            size,
            item: make(),
            used: true,
            taken: true,
            idle: 0,
        });
        self.slots.len() - 1
    }

    /// Gives `i` back for the rest of the frame.
    fn give(&mut self, i: usize) {
        self.slots[i].used = false;
    }

    fn items(&self) -> impl Iterator<Item = &T> {
        self.slots.iter().map(|s| &s.item)
    }
}

impl<T> std::ops::Index<usize> for Pool<T> {
    type Output = T;
    fn index(&self, i: usize) -> &T {
        &self.slots[i].item
    }
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
    /// The pooled target its result is in.
    target: usize,
    pub result: wgpu::TextureView,
}

impl Prepared {
    /// Its passes, which [`FilterRunner::record`] returns when it records
    /// them all.
    pub(crate) fn passes(&self) -> u32 {
        self.passes.len() as u32
    }
}

/// What a layer's filter reads as its source.
pub(crate) enum Source<'a> {
    /// What the layer drew, at a level ([`source_level`]: the layer's
    /// size halved that many times).
    Drawn(&'a wgpu::TextureView, u32),
    /// What is under a backdrop layer.
    Capture(Capture<'a>),
}

/// The target a backdrop layer's composite draws into, as drawn so far:
/// its view, its top left (CSS px), its device px per CSS px, and whether
/// it holds sRGB-encoded values (a non-sRGB output, which the tonemap
/// encodes) the filter decodes, or linear ones it encodes (the paint
/// pass's filters work on encoded values).
pub(crate) struct Capture<'a> {
    pub view: &'a wgpu::TextureView,
    pub origin: [f32; 2],
    pub scale: f32,
    pub decode: bool,
    pub encode: bool,
    pub size: (u32, u32),
}

/// A layer read by a filter: its used pixel rectangle in a pooled target.
pub(crate) struct LayerInput {
    pub view: wgpu::TextureView,
    pub origin: [f32; 2],
    pub scale: f32,
    pub size: (u32, u32),
}

/// What a filter's primitives read besides their inputs: `white` for a
/// texture a pass does not read, an image's texture by handle, and a
/// layer drawn this frame by index (its target's view, top left in CSS
/// px, and device px per CSS px).
pub(crate) struct Inputs<'a> {
    pub white: &'a wgpu::TextureView,
    pub texture_view: &'a dyn Fn(TextureHandle) -> Option<wgpu::TextureView>,
    pub layer: &'a dyn Fn(u32) -> Option<LayerInput>,
}

/// The filter pipeline, its result targets and parameters, and the
/// turbulence lattices by seed.
pub(crate) struct FilterRunner {
    /// Made the first time a layer's filter runs (deferred.rs): most UIs
    /// have none, and the module with every primitive took 0.4 s of a cold
    /// first frame on Metal.
    pipeline: crate::deferred::Deferred<wgpu::RenderPipeline>,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    pool: Pool<Pooled>,
    uniforms: Vec<wgpu::Buffer>,
    next_uniform: usize,
    lattices: Vec<(u32, wgpu::TextureView)>,
}

const FILTER_WGSL: &str = wgsl_library!("ui_filter.wgsl");

impl FilterRunner {
    pub(crate) fn new(device: &wgpu::Device) -> FilterRunner {
        let tex = |binding, filterable| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ui filter"),
            entries: &[
                tex(0, true),
                tex(1, true),
                // aux_tex may hold an Rgba32Float turbulence lattice (unfilterable).
                tex(2, false),
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
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ui filter linear"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ui filter"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let module = crate::gpu::shader_module(
            device,
            wgpu::ShaderModuleDescriptor {
                label: Some("ui filter"),
                source: wgpu::ShaderSource::Wgsl(source().into()),
            },
        );
        let device = device.clone();
        let pipeline = crate::deferred::Deferred::new(move || {
            crate::traffic::create_render_pipeline(
                &device,
                &wgpu::RenderPipelineDescriptor {
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
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                },
            )
        });
        FilterRunner {
            pipeline,
            layout,
            sampler,
            pool: Pool::new(),
            uniforms: Vec::new(),
            next_uniform: 0,
            lattices: Vec::new(),
        }
    }

    /// Makes its pipeline now, before a frame needs it.
    pub(crate) fn warm(&self) {
        crate::deferred::Start::start(&self.pipeline);
    }

    /// Frees every target and parameter buffer for a new frame; the
    /// targets stay made for it (let go only when they stood idle).
    pub(crate) fn begin(&mut self) {
        self.pool.begin();
        self.next_uniform = 0;
    }

    /// Whether `view` is one of its targets, still made.
    pub(crate) fn holds(&self, view: &wgpu::TextureView) -> bool {
        self.pool.items().any(|p| p.view == *view)
    }

    /// Gives back the targets `prepared` drew in but its result, once its
    /// passes are recorded: a filter prepared after it draws in them.
    pub(crate) fn done(&mut self, prepared: &Prepared) {
        for pass in &prepared.passes {
            if pass.target != prepared.target {
                self.pool.give(pass.target);
            }
        }
    }

    fn take(&mut self, device: &wgpu::Device, size: (u32, u32)) -> usize {
        self.pool.take(size, || Self::target(device, size))
    }

    fn target(device: &wgpu::Device, size: (u32, u32)) -> Pooled {
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
            texture_binding_view_dimension: None,
        });
        Pooled {
            view: texture.create_view(&Default::default()),
        }
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
                texture_binding_view_dimension: None,
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
            self.uniforms.push(crate::traffic::create_buffer(
                device,
                &wgpu::BufferDescriptor {
                    label: Some("ui filter"),
                    size: std::mem::size_of::<FilterParams>() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                },
            ));
        }
        self.next_uniform += 1;
        crate::traffic::write_buffer(
            queue,
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
        low: bool,
    ) -> Option<Prepared> {
        let mut passes = Vec::new();
        let clamp = matches!(source, Source::Capture(_));
        let primitives = filter.map_or(&[][..], |f| &f.primitives);
        let frame = Frame {
            origin,
            scale,
            space: filter.map_or(super::UiFilter::CSS, |f| f.space),
        };
        // A capture only a wide blur reads is taken at that blur's level
        // (each texel the box under it, `source_level`); what a layer drew
        // is at the level it was drawn at.
        let cap = match (&source, filter) {
            (Source::Capture(_), Some(filter)) => source_level(filter, scale),
            (Source::Drawn(_, level), _) => *level,
            _ => 0,
        };
        let source = match source {
            Source::Drawn(v, _) if filter.is_some_and(|f| !f.primitives.is_empty()) => v.clone(),
            Source::Drawn(..) => return None,
            Source::Capture(c) => {
                let params = FilterParams {
                    op: [OP_CAPTURE, FLAG_CLAMP, 0, 0],
                    origin: [origin[0], origin[1], scale / (1u32 << cap) as f32, 0.0],
                    subregion: [-1.0e9, -1.0e9, 2.0e9, 2.0e9],
                    space: [1.0, 0.0, 0.0, 1.0],
                    inverse: [1.0, 0.0, 0.0, 1.0],
                    extent: [c.size.0 as f32, c.size.1 as f32, 0.0, 0.0],
                    res: [1.0, cap as f32, size.0 as f32, size.1 as f32],
                    // Bilinear reads a side, each the 2×2 under it.
                    b: [(1u32 << cap.max(1)) as f32 / 2.0, 0.0, 0.0, 0.0],
                    a: [
                        c.origin[0],
                        c.origin[1],
                        c.scale,
                        if c.encode {
                            2.0
                        } else {
                            f32::from(u8::from(c.decode))
                        },
                    ],
                    ..Default::default()
                };
                let out = self.take(device, shrink(size, cap));
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
                        wgpu::BindingResource::Sampler(&self.sampler),
                    ],
                );
                passes.push(Pass { group, target: out });
                self.pool[out].view.clone()
            }
        };
        // A space that cannot be inverted draws nothing.
        let draws = invert(frame.space).is_some();
        let mut results: Vec<usize> = Vec::new();
        // Each result's level (the layer's size halved that many times).
        let mut levels: Vec<u32> = Vec::new();
        let mut folded = false;
        for (i, p) in primitives.iter().enumerate().filter(|_| draws) {
            let input = |i: UiFilterInput, results: &[usize], pool: &Pool<Pooled>| match i {
                UiFilterInput::Result(k) if (k as usize) < results.len() => {
                    pool[results[k as usize]].view.clone()
                }
                _ => source.clone(),
            };
            // The level its in1 is at.
            let at = match p.inputs[0] {
                UiFilterInput::Result(k) => levels.get(k as usize).copied().unwrap_or(0),
                UiFilterInput::Source => cap,
                _ => 0,
            };
            // A colour matrix on this blur's result alone, run by its
            // last pass.
            if folded {
                folded = false;
                results.push(*results.last().expect("the blur's"));
                levels.push(*levels.last().expect("the blur's"));
                continue;
            }
            let keep = low && kept(primitives, i);
            let mut steps = self::passes(p, &frame, clamp, keep, at);
            if keep
                && let Some(UiFilterOp::ColorMatrix(m)) = primitives.get(i + 1).map(|p| &p.op)
                && let Some(last) = steps.last_mut()
            {
                matrix(last, m);
                last.op[1] |= FLAG_MATRIX;
                folded = true;
            }
            // Per pixel on a result below the layer's size: at its level.
            if pointwise(p) && at > 0 {
                for s in &mut steps {
                    s.res[1] = at as f32;
                    s.origin[2] = scale / (1u32 << at) as f32;
                }
            }
            let aux = match &p.op {
                UiFilterOp::Turbulence { seed, .. } => self.lattice(device, queue, *seed),
                UiFilterOp::Image { texture, .. } => {
                    (inputs.texture_view)(*texture).unwrap_or_else(|| inputs.white.clone())
                }
                UiFilterOp::Layer { layer } => match (inputs.layer)(*layer) {
                    Some(input) => {
                        steps[0].a = [input.origin[0], input.origin[1], input.scale, 0.0];
                        steps[0].extent[2] = input.size.0 as f32;
                        steps[0].extent[3] = input.size.1 as f32;
                        input.view
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
            // The level in1 is at.
            let mut from = at;
            for (n, mut params) in steps.into_iter().enumerate() {
                let to = params.res[1] as u32;
                let read = shrink(size, from);
                params.extent[0] = read.0 as f32;
                params.extent[1] = read.1 as f32;
                params.res[0] = (to as f32 - from as f32).exp2();
                params.res[2] = size.0 as f32;
                params.res[3] = size.1 as f32;
                from = to;
                out = self.take(device, shrink(size, to));
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
                        wgpu::BindingResource::Sampler(&self.sampler),
                    ],
                );
                passes.push(Pass { group, target: out });
                if n < last {
                    in1 = self.pool[out].view.clone();
                }
            }
            results.push(out);
            levels.push(from);
        }
        let target = match (results.last(), passes.last()) {
            (Some(&r), _) => r,
            // A capture with no primitives (or a space that draws
            // nothing: transparent) composites what it captured.
            (None, Some(p)) if draws => p.target,
            (None, _) => {
                let out = self.take(device, size);
                let params = FilterParams {
                    op: [2, 0, 0, 0],
                    origin: [origin[0], origin[1], scale, 0.0],
                    res: [1.0, 0.0, size.0 as f32, size.1 as f32],
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
                        wgpu::BindingResource::Sampler(&self.sampler),
                    ],
                );
                passes.push(Pass { group, target: out });
                out
            }
        };
        let result = self.pool[target].view.clone();
        Some(Prepared {
            passes,
            target,
            result,
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
            let Some(pipeline) = self.pipeline.get() else {
                return 0;
            };
            rp.set_pipeline(pipeline);
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
