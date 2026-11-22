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

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes(path: &str) -> Vec<u8> {
        std::fs::read(format!(
            "{}/../../templates/{path}",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap()
    }

    fn gid(font: &FontRef, c: char) -> GlyphId {
        font.charmap().map(c).unwrap()
    }

    /// The unhinted outline of `g` at `size`, shifted `dx`, as fine chords
    /// (64 per curve): the oracle's own flattening, independent of the pen.
    fn chords(font: &FontRef, g: GlyphId, size: f32, dx: f32) -> Vec<[f32; 4]> {
        struct Chords {
            lines: Vec<[f32; 4]>,
            dx: f32,
            start: [f32; 2],
            at: [f32; 2],
        }
        impl Chords {
            fn to(&mut self, p: [f32; 2]) {
                self.lines.push([self.at[0], self.at[1], p[0], p[1]]);
                self.at = p;
            }
            fn curve(&mut self, f: impl Fn(f32) -> [f32; 2]) {
                for i in 1..=64 {
                    self.to(f(i as f32 / 64.0));
                }
            }
        }
        impl OutlinePen for Chords {
            fn move_to(&mut self, x: f32, y: f32) {
                self.close();
                self.start = [x + self.dx, y];
                self.at = self.start;
            }
            fn line_to(&mut self, x: f32, y: f32) {
                self.to([x + self.dx, y]);
            }
            fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
                let (p0, c, p1) = (self.at, [cx + self.dx, cy], [x + self.dx, y]);
                self.curve(|t| {
                    let u = 1.0 - t;
                    [
                        u * u * p0[0] + 2.0 * u * t * c[0] + t * t * p1[0],
                        u * u * p0[1] + 2.0 * u * t * c[1] + t * t * p1[1],
                    ]
                });
            }
            fn curve_to(&mut self, ax: f32, ay: f32, bx: f32, by: f32, x: f32, y: f32) {
                let (p0, a, b, p1) = (
                    self.at,
                    [ax + self.dx, ay],
                    [bx + self.dx, by],
                    [x + self.dx, y],
                );
                self.curve(|t| {
                    let u = 1.0 - t;
                    let (k0, k1, k2, k3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                    [
                        k0 * p0[0] + k1 * a[0] + k2 * b[0] + k3 * p1[0],
                        k0 * p0[1] + k1 * a[1] + k2 * b[1] + k3 * p1[1],
                    ]
                });
            }
            fn close(&mut self) {
                if self.at != self.start {
                    let s = self.start;
                    self.to(s);
                }
            }
        }
        let mut pen = Chords {
            lines: Vec::new(),
            dx,
            start: [0.0; 2],
            at: [0.0; 2],
        };
        let settings = DrawSettings::unhinted(Size::new(size), LocationRef::default());
        font.outline_glyphs()
            .get(g)
            .unwrap()
            .draw(settings, &mut pen)
            .unwrap();
        pen.close();
        pen.lines
    }

    /// Nonzero winding at a point of the flattened outline: the oracle the
    /// fill is checked against, independent of its cell arithmetic.
    fn winding(lines: &[[f32; 4]], x: f32, y: f32) -> i32 {
        let mut w = 0;
        for &[x0, y0, x1, y1] in lines {
            if (y0 <= y) != (y1 <= y) {
                let cx = x0 + (y - y0) / (y1 - y0) * (x1 - x0);
                if cx > x {
                    w += if y1 > y0 { 1 } else { -1 };
                }
            }
        }
        w
    }

    /// Each pixel's coverage is its share of the outline, as 16 × 16
    /// samples of the outline's winding find it: a fill that loses or
    /// double-counts area at a cell edge, carries it across a row or shifts
    /// it at a subpixel offset fails. Where two contours overlap inside a
    /// pixel (Q's tail, 游's strokes) area sums before it is clamped, as in
    /// FreeType's gray rasterizer (and so Chrome on Linux), so such a pixel
    /// reads heavier and is left out. Every other pixel holds to 0.09 and
    /// their total to 2 % (pixels beside an overlap share its edge cells),
    /// which upstream font-rs's 0.144 px chords miss (third_party/font-rs/
    /// PATCHED.md). Ideographs at 12 px, Latin at 14 and 48, every x bin.
    /// An outline whose leftmost point lies exactly on a pixel's edge
    /// rasters: Instrument Sans's glyph 14 at 20 px bulges left to x = 1.0,
    /// where font-rs's row-by-row sum lands a hair left of it, in a column
    /// before the first. Its ink is the same as a hair to the right.
    #[test]
    fn an_outline_touching_a_pixel_edge_rasters() {
        let data = bytes("styles/shared/InstrumentSans[wdth,wght].ttf");
        let font = FontRef::new(&data).unwrap();
        let ink = |x_offset: f32| {
            let g = Glyph {
                font: &font,
                id: GlyphId::new(14),
                size: 20.0,
                coords: &[],
                hinting: None,
                x_offset,
                skew: 0.0,
            };
            let mut out = Vec::new();
            let p = Rasterizer::new().raster(g, &mut out).unwrap();
            assert_eq!(out.len(), (p.width * p.height) as usize);
            out.iter().map(|&a| u32::from(a)).sum::<u32>()
        };
        let (on, beside) = (ink(0.0) as f32, ink(1.0 / 512.0) as f32);
        assert!(
            on > 0.0 && (on - beside).abs() < 0.002 * on,
            "{on} {beside}"
        );
    }

    #[test]
    fn coverage_is_each_pixels_share_of_the_outline() {
        let sc = bytes("styles/cjk/NotoSansSC-Regular.ttf");
        let latin = bytes("styles/shared/InstrumentSans-Regular.ttf");
        for (data, text, size) in [
            (&sc, "游戏设置", 12.0),
            (&latin, "Qg8", 14.0),
            (&latin, "Qg", 48.0),
        ] {
            let font = FontRef::new(data).unwrap();
            let mut r = Rasterizer::new();
            let mut out = Vec::new();
            for c in text.chars() {
                for bin in 0..4 {
                    let g = Glyph {
                        font: &font,
                        id: gid(&font, c),
                        size,
                        coords: &[],
                        hinting: None,
                        x_offset: bin as f32 * 0.25,
                        skew: 0.0,
                    };
                    let p = r.raster(g, &mut out).unwrap();
                    assert_eq!(out.len(), (p.width * p.height) as usize);
                    let lines = chords(&font, g.id, size, g.x_offset);
                    let (mut worst, mut ink, mut want) = (0.0f32, 0.0, 0.0);
                    for py in 0..p.height as usize {
                        for px in 0..p.width as usize {
                            let (mut n, mut overlap) = (0, false);
                            for sy in 0..16 {
                                for sx in 0..16 {
                                    let x = p.left as f32 + px as f32 + (sx as f32 + 0.5) / 16.0;
                                    let y = p.top as f32 - py as f32 - (sy as f32 + 0.5) / 16.0;
                                    let w = winding(&lines, x, y);
                                    n += (w != 0) as u32;
                                    overlap |= w.abs() > 1;
                                }
                            }
                            let got = out[py * p.width as usize + px] as f32 / 255.0;
                            let exp = n as f32 / 256.0;
                            if !overlap {
                                worst = worst.max((got - exp).abs());
                                ink += got;
                                want += exp;
                            }
                        }
                    }
                    assert!(
                        worst < 0.09,
                        "{c} at {size} px, bin {bin}: a pixel off by {worst}"
                    );
                    assert!(
                        (ink - want).abs() < 0.02 * want,
                        "{c} at {size} px, bin {bin}: ink {ink}, outline {want}"
                    );
                }
            }
        }
    }

    /// Distance from `(x, y)` to the nearest of `lines`.
    fn distance(lines: &[[f32; 4]], x: f32, y: f32) -> f32 {
        let mut best = f32::MAX;
        for &[x0, y0, x1, y1] in lines {
            let (dx, dy) = (x1 - x0, y1 - y0);
            let len = dx * dx + dy * dy;
            let t = if len > 0.0 {
                (((x - x0) * dx + (y - y0) * dy) / len).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let (px, py) = (x0 + t * dx - x, y0 + t * dy - y);
            best = best.min((px * px + py * py).sqrt());
        }
        best
    }

    /// A stroke `w` wide covers exactly the points within `w / 2` of the
    /// outline (a round join is that band's edge): each pixel's coverage
    /// is its share of the band, as 16 × 16 samples find it, and all the
    /// ink totals the band's area to 1 %; at 48 px, widths 1, 2 and 4.
    /// kurbo's stroke overlaps itself at joins, and an overlap inside a
    /// pixel sums before it is clamped, as in FreeType's gray rasterizer
    /// (the fill test's rule), so such a pixel reads heavier and is left
    /// out of the per-pixel check.
    #[test]
    fn a_stroke_covers_the_band_around_the_outline() {
        let latin = bytes("styles/shared/InstrumentSans-Regular.ttf");
        let font = FontRef::new(&latin).unwrap();
        let mut r = Rasterizer::new();
        let mut out = Vec::new();
        for c in ['O', 'A', 'g'] {
            for w in [1.0f32, 2.0, 4.0] {
                let g = Glyph {
                    font: &font,
                    id: gid(&font, c),
                    size: 48.0,
                    coords: &[],
                    hinting: None,
                    x_offset: 0.25,
                    skew: 0.0,
                };
                let p = r.raster_stroke(g, w, &mut out).unwrap();
                let lines = chords(&font, g.id, 48.0, 0.25);
                // The stroke's own pieces as fine chords: where their
                // winding passes one, two pieces overlap.
                let mut own = Vec::new();
                for piece in &r.pieces {
                    match *piece {
                        Piece::Line(a, b) => own.push([a[0], a[1], b[0], b[1]]),
                        Piece::Quad(a, c, b) => {
                            let at = |t: f32| {
                                let u = 1.0 - t;
                                [
                                    u * u * a[0] + 2.0 * u * t * c[0] + t * t * b[0],
                                    u * u * a[1] + 2.0 * u * t * c[1] + t * t * b[1],
                                ]
                            };
                            for i in 0..32 {
                                let (p0, p1) = (at(i as f32 / 32.0), at((i + 1) as f32 / 32.0));
                                own.push([p0[0], p0[1], p1[0], p1[1]]);
                            }
                        }
                    }
                }
                let (mut worst, mut ink, mut want) = (0.0f32, 0.0, 0.0);
                for py in 0..p.height as usize {
                    for px in 0..p.width as usize {
                        let (mut n, mut overlap) = (0, false);
                        for sy in 0..16 {
                            for sx in 0..16 {
                                let x = p.left as f32 + px as f32 + (sx as f32 + 0.5) / 16.0;
                                let y = p.top as f32 - py as f32 - (sy as f32 + 0.5) / 16.0;
                                n += (distance(&lines, x, y) <= w / 2.0) as u32;
                                overlap |= winding(&own, x, y).abs() > 1;
                            }
                        }
                        let got = out[py * p.width as usize + px] as f32 / 255.0;
                        let exp = n as f32 / 256.0;
                        if !overlap {
                            worst = worst.max((got - exp).abs());
                        }
                        ink += got;
                        want += exp;
                    }
                }
                assert!(worst < 0.12, "{c}, width {w}: a pixel off by {worst}");
                assert!(
                    (ink - want).abs() < 0.01 * want,
                    "{c}, width {w}: ink {ink}, band {want}"
                );
            }
        }
    }

    /// A blur keeps a mask's ink, and across a long straight edge its
    /// profile is CSS's: the Gaussian of sigma radius / 2 integrated, so
    /// at d px from the edge the coverage is ½ (1 + erf(d / (σ √2))).
    #[test]
    fn a_blur_is_csss_gaussian() {
        let (w, h) = (40u32, 60u32);
        // Columns 0..20 full, the rest empty: an edge at x = 20.
        let src: Vec<u8> = (0..w * h)
            .map(|i| if i % w < 20 { 255 } else { 0 })
            .collect();
        let (mut scratch, mut out) = (Vec::new(), Vec::new());
        for radius in [2.0f32, 5.0, 9.0] {
            let b = blur(&src, w, h, radius, &mut scratch, &mut out);
            assert_eq!(b.pad, (1.5 * radius).ceil() as u32);
            assert_eq!(out.len(), (b.width * b.height) as usize);
            let ink: u64 = out.iter().map(|&c| u64::from(c)).sum();
            let want: u64 = src.iter().map(|&c| u64::from(c)).sum();
            let off = (ink as f64 - want as f64).abs() / want as f64;
            assert!(off < 0.005, "radius {radius}: ink off by {off}");
            // The middle row, away from the mask's top and bottom.
            let y = (b.height / 2) as usize;
            let sigma = radius / 2.0;
            for x in 0..b.width as usize {
                let d = 20.0 + b.pad as f32 - (x as f32 + 0.5);
                let want = 0.5 * (1.0 + libm::erff(d / (sigma * std::f32::consts::SQRT_2)));
                // Left of the mask the source's own edge at x = 0 blurs too.
                if (x as f32) < 2.0 * b.pad as f32 + 4.0 {
                    continue;
                }
                let got = out[y * b.width as usize + x] as f32 / 255.0;
                assert!(
                    (got - want).abs() < 0.02,
                    "radius {radius}, {d} px from the edge: {got}, CSS {want}"
                );
            }
        }
    }

    /// Hinting is light: an I's horizontal edges land on whole pixels, so
    /// its top and bottom rows are solid, where unhinted at 13 px they are
    /// grey (the cap height is 9.1 px).
    #[test]
    fn light_hinting_snaps_horizontal_edges() {
        let data = bytes("styles/shared/InstrumentSans-Regular.ttf");
        let font = FontRef::new(&data).unwrap();
        let size = 13.0;
        let h = hinter(&font, size, &[]).expect("hintable");
        let mut r = Rasterizer::new();
        let mut out = Vec::new();
        let id = gid(&font, 'I');
        let g = |hinting| Glyph {
            font: &font,
            id,
            size,
            coords: &[],
            hinting,
            x_offset: 0.0,
            skew: 0.0,
        };
        let p = r.raster(g(Some(&h)), &mut out).unwrap();
        let w = p.width as usize;
        let rows: Vec<u32> = out
            .chunks(w)
            .map(|r| r.iter().map(|&c| c as u32).sum())
            .collect();
        let full = *rows.iter().max().unwrap();
        assert!(
            rows[0] * 10 >= full * 9 && rows[rows.len() - 1] * 10 >= full * 9,
            "{rows:?}"
        );
    }
}
