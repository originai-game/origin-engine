//! Glyphs rendered at run time from a cooked font's file (`font { runtime
//! true }`): a character outside the atlas, a CJK name a player typed, is
//! drawn from the font's own outlines into a page the host adds to the font.
//!
//! The field is a true signed distance (the same value in R, G, B and A),
//! which og-render's MTSDF shader reads as it reads a cooked glyph (the
//! median of three equal channels is the distance; A is the glow's true
//! distance): corners come out slightly rounded, which a CJK face at UI
//! sizes does not show. The geometry is the cook's (`em_px`, `px_range`,
//! the same padding and plane bounds), so a run-time glyph sits exactly
//! where a cooked one would. Only `+ − × ÷` and square roots on f32, in a
//! fixed order, so both hosts render the same bytes.

/// A glyph rendered from outlines: RGBA8 rows top to bottom.
#[derive(Clone, Debug, PartialEq)]
pub struct GlyphImage {
    pub codepoint: char,
    /// Horizontal advance in em.
    pub advance: f32,
    /// The quad around the glyph relative to the pen on the baseline, em,
    /// y up: left, bottom, right, top. All zero for a blank glyph.
    pub plane: [f32; 4],
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Line segments of a glyph's outline in atlas pixels (y up), curves
/// flattened into a fixed number of pieces.
struct Flatten {
    scale: f32,
    tx: f32,
    ty: f32,
    start: (f32, f32),
    at: (f32, f32),
    segments: Vec<[f32; 4]>,
}

/// Pieces per quadratic and cubic curve: under a tenth of a pixel from the
/// curve at 32 px per em.
const QUAD_STEPS: u32 = 8;
const CUBIC_STEPS: u32 = 12;

impl Flatten {
    fn point(&self, x: f32, y: f32) -> (f32, f32) {
        (x * self.scale + self.tx, y * self.scale + self.ty)
    }

    fn line(&mut self, to: (f32, f32)) {
        if to != self.at {
            self.segments.push([self.at.0, self.at.1, to.0, to.1]);
        }
        self.at = to;
    }
}

impl ttf_parser::OutlineBuilder for Flatten {
    fn move_to(&mut self, x: f32, y: f32) {
        let p = self.point(x, y);
        self.start = p;
        self.at = p;
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.point(x, y);
        self.line(p);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (p0, p1, p2) = (self.at, self.point(x1, y1), self.point(x, y));
        for i in 1..=QUAD_STEPS {
            let t = i as f32 / QUAD_STEPS as f32;
            let u = 1.0 - t;
            let q = (
                u * u * p0.0 + 2.0 * u * t * p1.0 + t * t * p2.0,
                u * u * p0.1 + 2.0 * u * t * p1.1 + t * t * p2.1,
            );
            self.line(if i == QUAD_STEPS { p2 } else { q });
        }
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (p0, p1, p2, p3) = (
            self.at,
            self.point(x1, y1),
            self.point(x2, y2),
            self.point(x, y),
        );
        for i in 1..=CUBIC_STEPS {
            let t = i as f32 / CUBIC_STEPS as f32;
            let u = 1.0 - t;
            let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            let q = (
                a * p0.0 + b * p1.0 + c * p2.0 + d * p3.0,
                a * p0.1 + b * p1.1 + c * p2.1 + d * p3.1,
            );
            self.line(if i == CUBIC_STEPS { p3 } else { q });
        }
    }

    fn close(&mut self) {
        let s = self.start;
        self.line(s);
    }
}

/// Renders `c` from the font file at the cook's geometry: `em_px` atlas
/// pixels per em, a distance field spanning `px_range` pixels. `None` when
/// the font has no glyph for it.
pub fn render_glyph(ttf: &[u8], c: char, em_px: f32, px_range: f32) -> Option<GlyphImage> {
    let face = ttf_parser::Face::parse(ttf, 0).ok()?;
    let gid = face.glyph_index(c)?;
    let upem = f32::from(face.units_per_em());
    let advance = f32::from(face.glyph_hor_advance(gid).unwrap_or(0)) / upem;
    let blank = GlyphImage {
        codepoint: c,
        advance,
        plane: [0.0; 4],
        width: 0,
        height: 0,
        rgba: Vec::new(),
    };
    let Some(bbox) = face.glyph_bounding_box(gid) else {
        return Some(blank);
    };
    if bbox.width() <= 0 || bbox.height() <= 0 {
        return Some(blank);
    }
    // The cook's geometry (og-cook's font cook, in f64 there; f32 here is
    // within a thousandth of a pixel of it).
    let scale = em_px / upem;
    let pad = (px_range / 2.0).ceil() + 1.0;
    let width = (f32::from(bbox.width()) * scale + 2.0 * pad).ceil() as u32;
    let height = (f32::from(bbox.height()) * scale + 2.0 * pad).ceil() as u32;
    let mut outline = Flatten {
        scale,
        tx: pad - f32::from(bbox.x_min) * scale,
        ty: pad - f32::from(bbox.y_min) * scale,
        start: (0.0, 0.0),
        at: (0.0, 0.0),
        segments: Vec::new(),
    };
    face.outline_glyph(gid, &mut outline)?;
    if outline.segments.is_empty() {
        return Some(blank);
    }
    let segments = outline.segments;
    let mut rgba = vec![0u8; (width * height * 4) as usize];
    for row in 0..height {
        // Atlas rows go top to bottom; the outline is y up.
        let py = (height - 1 - row) as f32 + 0.5;
        for col in 0..width {
            let px = col as f32 + 0.5;
            let mut best = f32::MAX;
            let mut winding = 0i32;
            for s in &segments {
                let [x0, y0, x1, y1] = *s;
                // Distance to the segment, squared.
                let (dx, dy) = (x1 - x0, y1 - y0);
                let len = dx * dx + dy * dy;
                let t = if len > 0.0 {
                    (((px - x0) * dx + (py - y0) * dy) / len).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let (ex, ey) = (x0 + t * dx - px, y0 + t * dy - py);
                let d = ex * ex + ey * ey;
                if d < best {
                    best = d;
                }
                // The nonzero rule along a ray to +x.
                if (y0 <= py) != (y1 <= py) {
                    let x = x0 + (py - y0) / (y1 - y0) * dx;
                    if x > px {
                        winding += if y1 > y0 { 1 } else { -1 };
                    }
                }
            }
            let d = best.sqrt();
            let signed = if winding != 0 { d } else { -d };
            let v = (signed / px_range + 0.5).clamp(0.0, 1.0);
            let byte = (f64::from(v) * 255.0) as u8;
            let o = ((row * width + col) * 4) as usize;
            rgba[o..o + 4].copy_from_slice(&[byte; 4]);
        }
    }
    let left = (f32::from(bbox.x_min) * scale - pad) / em_px;
    let bottom = (f32::from(bbox.y_min) * scale - pad) / em_px;
    Some(GlyphImage {
        codepoint: c,
        advance,
        plane: [
            left,
            bottom,
            left + width as f32 / em_px,
            bottom + height as f32 / em_px,
        ],
        width,
        height,
        rgba,
    })
}
