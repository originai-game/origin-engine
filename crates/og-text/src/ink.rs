//! Where glyph ink crosses a decoration line: CSS Text Decoration 4
//! §2.10.4 `text-decoration-skip-ink`, which interrupts underlines and
//! overlines where they would cross a glyph and a small distance to
//! either side of it. As Skia's glyph intercepts (BSD-3-Clause) do, each
//! glyph's outline is clipped to the line's band and the leftmost and
//! rightmost ink inside it make one interval.

use crate::fonts::Collection;
use crate::layout::GlyphRun;
use skrifa::MetadataProvider;
use skrifa::instance::{LocationRef, NormalizedCoord, Size};
use skrifa::outline::{DrawSettings, OutlinePen};

/// Pieces per quadratic and cubic curve: within a few hundredths of an em
/// of the curve, far below the gap around the ink.
const QUAD_STEPS: u32 = 6;
const CUBIC_STEPS: u32 = 8;

/// The x ranges, CSS px from the box's left, where `run`'s glyphs put ink
/// between `top` and `bottom` px below its baseline, each widened by `gap`
/// on both sides, sorted and merged. Whether a run is skipped at all (CSS
/// `auto` crosses a CJK run, §2.10.4's note) is its caller's choice.
pub fn ink_gaps(
    fonts: &Collection,
    run: &GlyphRun,
    top: f32,
    bottom: f32,
    gap: f32,
) -> Vec<(f32, f32)> {
    let Ok(face) = skrifa::FontRef::from_index(fonts.file(run.face), run.index) else {
        return Vec::new();
    };
    let coords: Vec<NormalizedCoord> = run
        .coords
        .iter()
        .map(|&c| NormalizedCoord::from_bits(c))
        .collect();
    let outlines = face.outline_glyphs();
    let mut out: Vec<(f32, f32)> = Vec::new();
    for g in run.glyphs {
        let Some(glyph) = outlines.get(skrifa::GlyphId::new(g.id)) else {
            continue;
        };
        // The band in px, y up from the glyph's own baseline.
        let dy = g.y - run.baseline;
        let mut clip = Clip {
            low: -(bottom - dy),
            high: -(top - dy),
            start: (0.0, 0.0),
            at: (0.0, 0.0),
            ink: None,
        };
        let settings = DrawSettings::unhinted(Size::new(run.size), LocationRef::new(&coords));
        if glyph.draw(settings, &mut clip).is_ok()
            && let Some((l, r)) = clip.ink
        {
            out.push((g.x + l - gap, g.x + r + gap));
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

/// An outline builder that keeps the x extent of its segments' parts
/// inside a horizontal band, in px.
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

impl OutlinePen for Clip {
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
