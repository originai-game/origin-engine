//! A glyph's coverage at its drawn size: the outline (skrifa, hinted
//! vertically or not) flattened to lines and filled by exact signed area,
//! one byte of coverage per device pixel, the way browsers draw axis-aligned
//! text. Every step is IEEE `f32` arithmetic with no fused or vector
//! operations and no host maths library, so a glyph rasters to the same
//! bytes on the native and wasm hosts. A [`Rasterizer`] keeps its buffers
//! between glyphs: rastering a glyph allocates only while a buffer grows.
//!
//! The fill is the accumulation rasterizer of font-rs (Raph Levien,
//! Apache-2.0 or MIT): each line adds the area it covers to the cells it
//! crosses, and a running sum along each row turns those into coverage. An
//! overlap of two contours (variable fonts keep them) sums past full and is
//! clamped, so it draws no seam.

use skrifa::instance::{LocationRef, NormalizedCoord, Size};
use skrifa::outline::{
    DrawSettings, Engine, HintingInstance, HintingOptions, OutlineGlyphFormat, OutlinePen,
    SmoothMode, Target,
};
use skrifa::{FontRef, GlyphId, MetadataProvider};

/// Flattening tolerance: a curve's chords stay within this many device
/// pixels of it.
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

/// Rasters glyphs into coverage, reusing its buffers.
#[derive(Default)]
pub struct Rasterizer {
    /// The flattened outline, `[x0, y0, x1, y1]` in device pixels, y up.
    lines: Vec<[f32; 4]>,
    /// skrifa's scratch memory for hinted and variable outlines.
    memory: Vec<u8>,
    /// Signed area per cell, `stride` cells to a row.
    acc: Vec<f32>,
}

impl Rasterizer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Rasters `glyph` into `out` (cleared first) and says where it sits;
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
        self.lines.clear();
        let mut pen = Flatten {
            lines: &mut self.lines,
            dx: glyph.x_offset,
            start: [0.0; 2],
            at: [0.0; 2],
        };
        outline.draw(settings, &mut pen).ok()?;
        pen.close();
        Some(self.fill(out))
    }

    /// Fills the flattened outline into `out`.
    fn fill(&mut self, out: &mut Vec<u8>) -> Placement {
        if self.lines.is_empty() {
            return Placement::default();
        }
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for l in &self.lines {
            x0 = x0.min(l[0]).min(l[2]);
            x1 = x1.max(l[0]).max(l[2]);
            y0 = y0.min(l[1]).min(l[3]);
            y1 = y1.max(l[1]).max(l[3]);
        }
        let left = x0.floor() as i32;
        let top = y1.ceil() as i32;
        let width = (x1.ceil() as i32 - left).max(0) as usize;
        let height = (top - y0.floor() as i32).max(0) as usize;
        if width == 0 || height == 0 {
            return Placement::default();
        }
        // Two spare cells a row: a line on the right edge adds to the cell
        // past it, which the row's sum never reads into the next row.
        let stride = width + 2;
        self.acc.clear();
        self.acc.resize(stride * height, 0.0);
        let (ox, oy) = (left as f32, top as f32);
        for i in 0..self.lines.len() {
            let [ax, ay, bx, by] = self.lines[i];
            line(
                &mut self.acc,
                stride,
                height,
                [ax - ox, oy - ay],
                [bx - ox, oy - by],
            );
        }
        out.reserve(width * height);
        for row in self.acc.chunks_exact(stride) {
            let mut sum = 0.0f32;
            for &a in &row[..width] {
                sum += a;
                let c = sum.abs().min(1.0);
                out.push((c * 255.0 + 0.5) as u8);
            }
        }
        Placement {
            left,
            top,
            width: width as u32,
            height: height as u32,
        }
    }
}

/// Adds the signed area line `p0`–`p1` covers (raster space: y down, cells
/// `0..stride` by `0..height`) to the cells it crosses.
fn line(acc: &mut [f32], stride: usize, height: usize, p0: [f32; 2], p1: [f32; 2]) {
    if p0[1] == p1[1] {
        return;
    }
    let (dir, p0, p1) = if p0[1] < p1[1] {
        (1.0, p0, p1)
    } else {
        (-1.0, p1, p0)
    };
    let dxdy = (p1[0] - p0[0]) / (p1[1] - p0[1]);
    let mut x = p0[0];
    let first = p0[1].max(0.0) as usize;
    if p0[1] < 0.0 {
        x -= p0[1] * dxdy;
    }
    let last = (p1[1].ceil() as usize).min(height);
    for y in first..last {
        let row = y * stride;
        let dy = ((y + 1) as f32).min(p1[1]) - (y as f32).max(p0[1]);
        let next = x + dxdy * dy;
        let d = dy * dir;
        let (a, b) = if x < next { (x, next) } else { (next, x) };
        let a_floor = a.floor();
        let ai = a_floor as usize;
        let b_ceil = b.ceil();
        let bi = b_ceil as usize;
        if bi <= ai + 1 {
            // Within one cell: its area splits at the line's mean x.
            let mid = 0.5 * (x + next) - a_floor;
            acc[row + ai] += d - d * mid;
            acc[row + ai + 1] += d * mid;
        } else {
            let s = 1.0 / (b - a);
            let af = a - a_floor;
            let a0 = 0.5 * s * (1.0 - af) * (1.0 - af);
            let bf = b - b_ceil + 1.0;
            let am = 0.5 * s * bf * bf;
            acc[row + ai] += d * a0;
            if bi == ai + 2 {
                acc[row + ai + 1] += d * (1.0 - a0 - am);
            } else {
                let a1 = s * (1.5 - af);
                acc[row + ai + 1] += d * (a1 - a0);
                for xi in ai + 2..bi - 1 {
                    acc[row + xi] += d * s;
                }
                let a2 = a1 + (bi - ai - 3) as f32 * s;
                acc[row + bi - 1] += d * (1.0 - a2 - am);
            }
            acc[row + bi] += d * am;
        }
        x = next;
    }
}

/// An outline pen that flattens curves into lines, shifted `dx` in x.
struct Flatten<'a> {
    lines: &'a mut Vec<[f32; 4]>,
    dx: f32,
    start: [f32; 2],
    at: [f32; 2],
}

impl Flatten<'_> {
    fn to(&mut self, p: [f32; 2]) {
        if p != self.at {
            self.lines.push([self.at[0], self.at[1], p[0], p[1]]);
        }
        self.at = p;
    }

    /// Chords for a curve whose second difference is `dev` long: the
    /// deviation of `n` chords is at most `dev / (8 n²)`.
    fn count(dev: f32) -> u32 {
        ((dev / (8.0 * TOLERANCE)).sqrt().ceil() as u32).clamp(1, 64)
    }
}

impl OutlinePen for Flatten<'_> {
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
        let e = [p0[0] - 2.0 * c[0] + p1[0], p0[1] - 2.0 * c[1] + p1[1]];
        let n = Self::count(2.0 * (e[0] * e[0] + e[1] * e[1]).sqrt());
        for i in 1..=n {
            let t = i as f32 / n as f32;
            let u = 1.0 - t;
            self.to([
                u * u * p0[0] + 2.0 * u * t * c[0] + t * t * p1[0],
                u * u * p0[1] + 2.0 * u * t * c[1] + t * t * p1[1],
            ]);
        }
    }

    fn curve_to(&mut self, c0x: f32, c0y: f32, c1x: f32, c1y: f32, x: f32, y: f32) {
        let p0 = self.at;
        let (c0, c1, p1) = ([c0x + self.dx, c0y], [c1x + self.dx, c1y], [x + self.dx, y]);
        let e0 = [p0[0] - 2.0 * c0[0] + c1[0], p0[1] - 2.0 * c0[1] + c1[1]];
        let e1 = [c0[0] - 2.0 * c1[0] + p1[0], c0[1] - 2.0 * c1[1] + p1[1]];
        let dev = (e0[0] * e0[0] + e0[1] * e0[1]).max(e1[0] * e1[0] + e1[1] * e1[1]);
        let n = Self::count(6.0 * dev.sqrt());
        for i in 1..=n {
            let t = i as f32 / n as f32;
            let u = 1.0 - t;
            let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            self.to([
                a * p0[0] + b * c0[0] + c * c1[0] + d * p1[0],
                a * p0[1] + b * c0[1] + c * c1[1] + d * p1[1],
            ]);
        }
    }

    fn close(&mut self) {
        let start = self.start;
        self.to(start);
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
    /// reads heavier and is left out; every other pixel holds, and so does
    /// their total to 2 % (pixels beside an overlap share its edge cells). Ideographs at 12 px,
    /// Latin at 14 and 48, every x bin.
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
                    };
                    let p = r.raster(g, &mut out).unwrap();
                    assert_eq!(out.len(), (p.width * p.height) as usize);
                    let (mut worst, mut ink, mut want) = (0.0f32, 0.0, 0.0);
                    for py in 0..p.height as usize {
                        for px in 0..p.width as usize {
                            let (mut n, mut overlap) = (0, false);
                            for sy in 0..16 {
                                for sx in 0..16 {
                                    let x = p.left as f32 + px as f32 + (sx as f32 + 0.5) / 16.0;
                                    let y = p.top as f32 - py as f32 - (sy as f32 + 0.5) / 16.0;
                                    let w = winding(&r.lines, x, y);
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
