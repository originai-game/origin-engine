//! Where glyph ink crosses a decoration line: CSS Text Decoration 4
//! §2.10.4 `text-decoration-skip-ink`, which interrupts underlines and
//! overlines where they would cross a glyph and a small distance to
//! either side of it. As Skia's glyph intercepts (BSD-3-Clause) do, each
//! glyph's outline is clipped to the line's band and the leftmost and
//! rightmost ink inside it make one interval.

use crate::text::{FontMetrics, UiFont, VisualGlyph};

/// Pieces per quadratic and cubic curve: within a few hundredths of an em
/// of the curve, far below the gap around the ink.
const QUAD_STEPS: u32 = 6;
const CUBIC_STEPS: u32 = 8;

/// The x ranges, px from the line's start, where `glyphs` (left to right,
/// drawn `size` px tall in `font`) put ink between `top` and `bottom` px
/// below the baseline, each widened by `gap` on both sides, sorted and
/// merged. `cjk`: whether CJK text is skipped too (`all`); under `auto`
/// a CJK script run is crossed (§2.10.4's note). A glyph from a face
/// without its file has no outline here and is crossed.
pub fn ink_gaps(
    font: &UiFont,
    glyphs: &[VisualGlyph],
    size: f32,
    top: f32,
    bottom: f32,
    gap: f32,
    cjk: bool,
) -> Vec<(f32, f32)> {
    let mut out: Vec<(f32, f32)> = Vec::new();
    let mut pen = 0.0f32;
    // The script run so far: a Common character takes the one before it.
    let mut run_cjk = false;
    for g in glyphs {
        let x = pen + g.x * size;
        pen += g.advance * size;
        let script = og_i18n::text::cjk_script(g.ch);
        let is_cjk = script.unwrap_or(run_cjk);
        if script.is_some() {
            run_cjk = is_cjk;
        }
        if is_cjk && !cjk {
            continue;
        }
        let Some(m) = metrics_of(font, g.font) else {
            continue;
        };
        // The band in the glyph's em, y up from its own baseline.
        let dy = g.y * size;
        if let Some((l, r)) = glyph_ink(m, g, (-(bottom + dy) / size, -(top + dy) / size)) {
            out.push((x + l * size - gap, x + r * size + gap));
        }
    }
    out.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut merged: Vec<(f32, f32)> = Vec::with_capacity(out.len());
    for (l, r) in out {
        match merged.last_mut() {
            Some(last) if l <= last.1 => last.1 = last.1.max(r),
            _ => merged.push((l, r)),
        }
    }
    merged
}

/// The metrics of [`VisualGlyph::font`] `k`.
fn metrics_of(font: &UiFont, k: u8) -> Option<&FontMetrics> {
    match k {
        0 => Some(&font.metrics),
        k => Some(&font.metrics.runs.as_ref()?.get(k as usize - 1)?.metrics),
    }
}

/// The leftmost and rightmost ink of `g`'s outline between `band.0` and
/// `band.1` em above its baseline, em from its origin.
fn glyph_ink(m: &FontMetrics, g: &VisualGlyph, band: (f32, f32)) -> Option<(f32, f32)> {
    let file = m.file.as_ref()?;
    let mut face = ttf_parser::Face::parse(file.bytes(), 0).ok()?;
    for &(tag, value) in file.variations() {
        face.set_variation(ttf_parser::Tag::from_bytes(&tag), value);
    }
    let id = match g.glyph {
        Some(id) => ttf_parser::GlyphId(id),
        None => face.glyph_index(g.ch)?,
    };
    let upem = f32::from(face.units_per_em());
    let mut clip = Clip {
        low: band.0 * upem,
        high: band.1 * upem,
        start: (0.0, 0.0),
        at: (0.0, 0.0),
        ink: None,
    };
    face.outline_glyph(id, &mut clip)?;
    clip.ink.map(|(l, r)| (l / upem, r / upem))
}

/// An outline builder that keeps the x extent of its segments' parts
/// inside a horizontal band, in font units.
struct Clip {
    low: f32,
    high: f32,
    start: (f32, f32),
    at: (f32, f32),
    ink: Option<(f32, f32)>,
}

impl Clip {
    fn line(&mut self, to: (f32, f32)) {
        let (a, b) = (self.at, to);
        self.at = to;
        let (y0, y1) = (a.1.min(b.1), a.1.max(b.1));
        if y1 < self.low || y0 > self.high {
            return;
        }
        // The part of the segment inside the band: its ends clamped to it.
        let x_at = |y: f32| {
            if (b.1 - a.1).abs() < f32::EPSILON {
                a.0
            } else {
                a.0 + (b.0 - a.0) * (y - a.1) / (b.1 - a.1)
            }
        };
        let (xa, xb) = if (b.1 - a.1).abs() < f32::EPSILON {
            (a.0, b.0)
        } else {
            (
                x_at(a.1.clamp(self.low, self.high)),
                x_at(b.1.clamp(self.low, self.high)),
            )
        };
        let (l, r) = (xa.min(xb), xa.max(xb));
        self.ink = Some(self.ink.map_or((l, r), |(il, ir)| (il.min(l), ir.max(r))));
    }
}

impl ttf_parser::OutlineBuilder for Clip {
    fn move_to(&mut self, x: f32, y: f32) {
        self.start = (x, y);
        self.at = (x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.line((x, y));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let p0 = self.at;
        for i in 1..=QUAD_STEPS {
            let t = i as f32 / QUAD_STEPS as f32;
            let u = 1.0 - t;
            self.line((
                u * u * p0.0 + 2.0 * u * t * x1 + t * t * x,
                u * u * p0.1 + 2.0 * u * t * y1 + t * t * y,
            ));
        }
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let p0 = self.at;
        for i in 1..=CUBIC_STEPS {
            let t = i as f32 / CUBIC_STEPS as f32;
            let u = 1.0 - t;
            let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            self.line((
                a * p0.0 + b * x1 + c * x2 + d * x,
                a * p0.1 + b * y1 + c * y2 + d * y,
            ));
        }
    }

    fn close(&mut self) {
        self.line(self.start);
    }
}
