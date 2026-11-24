//! A glyph's coverage at its drawn size: the outline (skrifa, hinted
//! vertically or not) filled by exact signed area with font-rs, one byte of
//! coverage per device pixel, the way browsers draw axis-aligned text.
//! font-rs's fill is IEEE `f32` arithmetic with no fused or vector
//! operations and no host maths library (its default features: no `sse`),
//! so a glyph rasters to the same bytes on the native and wasm hosts.
//!
//! This module is the adapter only: skrifa draws the outline into a pen
//! that hands font-rs its lines and quadratic curves, and splits a cubic
//! (CFF) into quadratics, which font-rs lacks. An overlap of two contours
//! (variable fonts keep them) sums past full and is clamped, as in font-rs
//! and FreeType's gray rasterizer.

use skrifa::instance::{LocationRef, NormalizedCoord, Size};
use skrifa::outline::{
    DrawSettings, Engine, HintingInstance, HintingOptions, OutlineGlyphFormat, OutlinePen,
    SmoothMode, Target,
};
use skrifa::{FontRef, GlyphId, MetadataProvider};

/// How far, in device pixels, a cubic's quadratic pieces may stray from it.
const TOLERANCE: f32 = 0.05;

/// The hinting UI text takes: light, vertical only, its advances kept as
/// designed so a hinted line measures as the unhinted one. As FreeType's
/// `FT_LOAD_TARGET_LIGHT` (Chrome's `slight`): the autohinter for TrueType
/// outlines, the font's own hints for PostScript (CFF) ones.
pub fn light_hinting(format: Option<OutlineGlyphFormat>) -> HintingOptions {
    let engine = match format {
        Some(OutlineGlyphFormat::Cff | OutlineGlyphFormat::Cff2) => Engine::Interpreter,
        _ => Engine::Auto(None),
    };
    HintingOptions {
        engine,
        target: Target::Smooth {
            mode: SmoothMode::Light,
            symmetric_rendering: true,
            preserve_linear_metrics: true,
        },
    }
}

/// A hinting instance for `font` at `size` device pixels per em and
/// variation `coords`, or `None` when the font cannot be hinted (it is then
/// drawn unhinted). Make one per (face, size, location) and keep it.
pub fn hinter(font: &FontRef, size: f32, coords: &[NormalizedCoord]) -> Option<HintingInstance> {
    let outlines = font.outline_glyphs();
    HintingInstance::new(
        &outlines,
        Size::new(size),
        LocationRef::new(coords),
        light_hinting(outlines.format()),
    )
    .ok()
}

/// What to raster: a glyph of a face at a size and position.
#[derive(Clone, Copy)]
pub struct Glyph<'a> {
    pub font: &'a FontRef<'a>,
    pub id: GlyphId,
    /// Device pixels per em.
    pub size: f32,
    /// The face's variation location (normalized), empty for its default.
    pub coords: &'a [NormalizedCoord],
    /// The hinting instance for this face, size and location
    /// ([`hinter`]); `None` draws the outline unhinted.
    pub hinting: Option<&'a HintingInstance>,
    /// The pen's fraction of a pixel in x, `0.0..1.0` (a subpixel bin).
    pub x_offset: f32,
    /// A slant the face lacks (synthetic italic): each point moves right
    /// by this times its height above the baseline, the angle's tangent;
    /// 0 upright.
    pub skew: f32,
}

/// A rastered glyph's coverage: `width` × `height` bytes, row by row from
/// the top, whose top left pixel sits `left` pixels right of the pen and
/// `top` pixels above the baseline.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Placement {
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
}

/// One piece of an outline in device pixels, y up: a line or a quadratic
/// curve, its control point in the middle.
#[derive(Clone, Copy)]
enum Piece {
    Line([f32; 2], [f32; 2]),
    Quad([f32; 2], [f32; 2], [f32; 2]),
}

/// Rasters glyphs into coverage, keeping skrifa's scratch memory and the
/// outline's pieces between glyphs. font-rs makes a buffer per glyph, a
/// cost of a glyph's first raster only.
#[derive(Default)]
pub struct Rasterizer {
    pieces: Vec<Piece>,
    /// skrifa's scratch memory for hinted and variable outlines.
    memory: Vec<u8>,
}

impl Rasterizer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Rasters `glyph` into `out` (replaced) and says where it sits;
    /// `None` when the face has no outline for it. A glyph with no ink
    /// (a space) is placed with a zero size.
    pub fn raster(&mut self, glyph: Glyph<'_>, out: &mut Vec<u8>) -> Option<Placement> {
        out.clear();
        let outlines = glyph.font.outline_glyphs();
        let outline = outlines.get(glyph.id)?;
        let hinting = match glyph.hinting {
            Some(_) => skrifa::outline::Hinting::Embedded,
            None => skrifa::outline::Hinting::None,
        };
        let need = outline.draw_memory_size(hinting);
        if self.memory.len() < need {
            self.memory.resize(need, 0);
        }
        let settings = match glyph.hinting {
            Some(h) => DrawSettings::hinted(h, false),
            None => DrawSettings::unhinted(Size::new(glyph.size), LocationRef::new(glyph.coords)),
        }
        .with_memory(Some(&mut self.memory[..need]));
        self.pieces.clear();
        let mut pen = Pen {
            pieces: &mut self.pieces,
            dx: glyph.x_offset,
            skew: glyph.skew,
            start: [0.0; 2],
            at: [0.0; 2],
        };
        outline.draw(settings, &mut pen).ok()?;
        pen.close();
        Some(self.fill(out))
    }

    /// Rasters the band `width` device pixels wide centred on `glyph`'s
    /// outline, round-joined (CSS `-webkit-text-stroke`, Skia's and
    /// Godot's outline): kurbo strokes the outline and font-rs fills the
    /// stroke. Placed as [`Rasterizer::raster`] places a glyph.
    pub fn raster_stroke(
        &mut self,
        glyph: Glyph<'_>,
        width: f32,
        out: &mut Vec<u8>,
    ) -> Option<Placement> {
        out.clear();
        let outlines = glyph.font.outline_glyphs();
        let outline = outlines.get(glyph.id)?;
        let hinting = match glyph.hinting {
            Some(_) => skrifa::outline::Hinting::Embedded,
            None => skrifa::outline::Hinting::None,
        };
        let need = outline.draw_memory_size(hinting);
        if self.memory.len() < need {
            self.memory.resize(need, 0);
        }
        let settings = match glyph.hinting {
            Some(h) => DrawSettings::hinted(h, false),
            None => DrawSettings::unhinted(Size::new(glyph.size), LocationRef::new(glyph.coords)),
        }
        .with_memory(Some(&mut self.memory[..need]));
        let mut path = PathPen {
            path: kurbo::BezPath::new(),
            dx: f64::from(glyph.x_offset),
            skew: f64::from(glyph.skew),
        };
        outline.draw(settings, &mut path).ok()?;
        let stroke = kurbo::Stroke::new(f64::from(width)).with_join(kurbo::Join::Round);
        let band = kurbo::stroke(
            path.path.elements().iter().copied(),
            &stroke,
            &kurbo::StrokeOpts::default(),
            f64::from(TOLERANCE),
        );
        self.pieces.clear();
        let mut pen = Pen {
            pieces: &mut self.pieces,
            dx: 0.0,
            skew: 0.0,
            start: [0.0; 2],
            at: [0.0; 2],
        };
        let p = |p: kurbo::Point| (p.x as f32, p.y as f32);
        for el in band.elements() {
            match *el {
                kurbo::PathEl::MoveTo(a) => pen.move_to(p(a).0, p(a).1),
                kurbo::PathEl::LineTo(a) => pen.line_to(p(a).0, p(a).1),
                kurbo::PathEl::QuadTo(c, a) => pen.quad_to(p(c).0, p(c).1, p(a).0, p(a).1),
                kurbo::PathEl::CurveTo(c0, c1, a) => {
                    pen.curve_to(p(c0).0, p(c0).1, p(c1).0, p(c1).1, p(a).0, p(a).1)
                }
                kurbo::PathEl::ClosePath => pen.close(),
            }
        }
        pen.close();
        Some(self.fill(out))
    }

    /// Fills the outline's pieces into `out` with font-rs.
    fn fill(&self, out: &mut Vec<u8>) -> Placement {
        if self.pieces.is_empty() {
            return Placement::default();
        }
        // A quadratic lies inside its points' hull, so they bound it.
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        let mut bound = |p: [f32; 2]| {
            x0 = x0.min(p[0]);
            x1 = x1.max(p[0]);
            y0 = y0.min(p[1]);
            y1 = y1.max(p[1]);
        };
        for piece in &self.pieces {
            match *piece {
                Piece::Line(a, b) => [a, b].into_iter().for_each(&mut bound),
                Piece::Quad(a, c, b) => [a, c, b].into_iter().for_each(&mut bound),
            }
        }
        // font-rs walks a line row by row, adding to its x, and the sum
        // strays from the outline by float error: an extreme lying exactly
        // on a pixel's edge lands a hair outside it, in a column the raster
        // would not have (an index below zero). The bounds keep this room.
        const EDGE: f32 = 1.0 / 256.0;
        let left = (x0 - EDGE).floor() as i32;
        let top = y1.ceil() as i32;
        let width = ((x1 + EDGE).ceil() as i32 - left).max(0) as usize;
        let height = (top - y0.floor() as i32).max(0) as usize;
        if width == 0 || height == 0 {
            return Placement::default();
        }
        // font-rs's raster space: from the top left, y down.
        let (ox, oy) = (left as f32, top as f32);
        let at = |p: [f32; 2]| font_rs::geom::Point {
            x: p[0] - ox,
            y: oy - p[1],
        };
        let mut raster = font_rs::raster::Raster::new(width, height);
        for piece in &self.pieces {
            match *piece {
                Piece::Line(a, b) => raster.draw_line(&at(a), &at(b)),
                Piece::Quad(a, c, b) => raster.draw_quad(&at(a), &at(c), &at(b)),
            }
        }
        *out = raster.get_bitmap();
        Placement {
            left,
            top,
            width: width as u32,
            height: height as u32,
        }
    }
}

/// A coverage mask blurred as CSS blurs a shadow (`text-shadow`, glow):
/// a Gaussian of standard deviation half the blur `radius` (CSS Backgrounds
/// 3 §7.1), wide enough to hold three of them each side.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Blurred {
    /// How far the blurred mask reaches past the source's every side, px.
    pub pad: u32,
    pub width: u32,
    pub height: u32,
}

/// Blurs mask `src`, `width` × `height` bytes, into `out` (replaced) with a
/// separable Gaussian, `scratch` holding the pass between. Every step is
/// IEEE `f32` and libm's `expf`, so both hosts blur to the same bytes. A
/// shadow is blurred once per glyph and radius and cached, so the kernel
/// and the column pass allocate.
pub fn blur(
    src: &[u8],
    width: u32,
    height: u32,
    radius: f32,
    scratch: &mut Vec<f32>,
    out: &mut Vec<u8>,
) -> Blurred {
    let sigma = radius / 2.0;
    let pad = if sigma > 0.0 {
        (3.0 * sigma).ceil() as u32
    } else {
        0
    };
    let (w, h) = (width as usize, height as usize);
    let (bw, bh, p) = (w + 2 * pad as usize, h + 2 * pad as usize, pad as usize);
    out.clear();
    if pad == 0 {
        out.extend_from_slice(src);
        return Blurred { pad, width, height };
    }
    // The kernel, normalized so a mask's ink is kept.
    let mut kernel = vec![0.0f32; 2 * p + 1];
    let k = &mut kernel[..];
    let mut sum = 0.0;
    for (i, v) in k.iter_mut().enumerate() {
        let d = i as f32 - p as f32;
        *v = libm::expf(-(d * d) / (2.0 * sigma * sigma));
        sum += *v;
    }
    for v in k.iter_mut() {
        *v /= sum;
    }
    let half = p;
    // Rows: source width to padded width, into `scratch`.
    scratch.clear();
    scratch.resize(bw * h, 0.0);
    for y in 0..h {
        let row = &src[y * w..(y + 1) * w];
        let dst = &mut scratch[y * bw..(y + 1) * bw];
        for (x, &c) in row.iter().enumerate() {
            if c == 0 {
                continue;
            }
            let c = f32::from(c) / 255.0;
            let at = x + p - half;
            for (t, kv) in k.iter().enumerate() {
                dst[at + t] += c * kv;
            }
        }
    }
    // Columns: into the padded mask.
    let mut col = vec![0.0f32; bw * bh];
    for y in 0..h {
        let at = y + p - half;
        for x in 0..bw {
            let c = scratch[y * bw + x];
            if c == 0.0 {
                continue;
            }
            for (t, kv) in k.iter().enumerate() {
                col[(at + t) * bw + x] += c * kv;
            }
        }
    }
    out.extend(col.iter().map(|&c| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8));
    Blurred {
        pad,
        width: bw as u32,
        height: bh as u32,
    }
}

/// An outline pen that collects the outline as a kurbo path, shifted `dx`
/// and slanted by `skew`.
struct PathPen {
    path: kurbo::BezPath,
    dx: f64,
    skew: f64,
}

impl PathPen {
    fn at(&self, x: f32, y: f32) -> (f64, f64) {
        let y = f64::from(y);
        (f64::from(x) + self.dx + self.skew * y, y)
    }
}

impl OutlinePen for PathPen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.path.move_to(self.at(x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.path.line_to(self.at(x, y));
    }

    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.path.quad_to(self.at(cx, cy), self.at(x, y));
    }

    fn curve_to(&mut self, c0x: f32, c0y: f32, c1x: f32, c1y: f32, x: f32, y: f32) {
        self.path
            .curve_to(self.at(c0x, c0y), self.at(c1x, c1y), self.at(x, y));
    }

    fn close(&mut self) {
        self.path.close_path();
    }
}

/// An outline pen that collects lines and quadratics, shifted `dx` in x
/// and slanted by `skew`.
struct Pen<'a> {
    pieces: &'a mut Vec<Piece>,
    dx: f32,
    skew: f32,
    start: [f32; 2],
    at: [f32; 2],
}

impl Pen<'_> {
    fn point(&self, x: f32, y: f32) -> [f32; 2] {
        [x + self.dx + self.skew * y, y]
    }

    fn line(&mut self, p: [f32; 2]) {
        if p != self.at {
            self.pieces.push(Piece::Line(self.at, p));
        }
        self.at = p;
    }
}

impl OutlinePen for Pen<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.close();
        self.start = self.point(x, y);
        self.at = self.start;
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.line(self.point(x, y));
    }

    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let p = self.point(x, y);
        self.pieces
            .push(Piece::Quad(self.at, self.point(cx, cy), p));
        self.at = p;
    }

    /// A cubic as `n` quadratics, one per equal step of t, each through
    /// the step's ends with its control point at `(3 (c0 + c1) - (p0 + p1)) / 4`
    /// of the step's cubic. A piece strays from the cubic by at most
    /// `√3 / 36 · |p1 - 3 c1 + 3 c0 - p0| / n³`.
    fn curve_to(&mut self, c0x: f32, c0y: f32, c1x: f32, c1y: f32, x: f32, y: f32) {
        let p0 = self.at;
        let (c0, c1, p1) = (self.point(c0x, c0y), self.point(c1x, c1y), self.point(x, y));
        let d = [
            p1[0] - 3.0 * c1[0] + 3.0 * c0[0] - p0[0],
            p1[1] - 3.0 * c1[1] + 3.0 * c0[1] - p0[1],
        ];
        let err = 0.048_112_52 * (d[0] * d[0] + d[1] * d[1]).sqrt();
        let mut n = 1u32;
        while n < 16 && err > TOLERANCE * (n * n * n) as f32 {
            n += 1;
        }
        let point = |t: f32| {
            let u = 1.0 - t;
            let (a, b, c, e) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            [
                a * p0[0] + b * c0[0] + c * c1[0] + e * p1[0],
                a * p0[1] + b * c0[1] + c * c1[1] + e * p1[1],
            ]
        };
        // The cubic's derivative at t, for each step's control points.
        let slope = |t: f32| {
            let u = 1.0 - t;
            let (a, b, c) = (3.0 * u * u, 6.0 * u * t, 3.0 * t * t);
            [
                a * (c0[0] - p0[0]) + b * (c1[0] - c0[0]) + c * (p1[0] - c1[0]),
                a * (c0[1] - p0[1]) + b * (c1[1] - c0[1]) + c * (p1[1] - c1[1]),
            ]
        };
        let step = 1.0 / n as f32;
        let mut from = p0;
        for i in 0..n {
            let (t0, t1) = (i as f32 * step, (i + 1) as f32 * step);
            let to = if i + 1 == n { p1 } else { point(t1) };
            let (s0, s1) = (slope(t0), slope(t1));
            // The step's cubic control points, then the quadratic's.
            let q0 = [from[0] + s0[0] * step / 3.0, from[1] + s0[1] * step / 3.0];
            let q1 = [to[0] - s1[0] * step / 3.0, to[1] - s1[1] * step / 3.0];
            let c = [
                (3.0 * (q0[0] + q1[0]) - (from[0] + to[0])) * 0.25,
                (3.0 * (q0[1] + q1[1]) - (from[1] + to[1])) * 0.25,
            ];
            self.pieces.push(Piece::Quad(from, c, to));
            from = to;
        }
        self.at = p1;
    }

    fn close(&mut self) {
        let start = self.start;
        self.line(start);
    }
}
