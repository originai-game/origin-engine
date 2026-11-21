//! A glyph's multi-channel true signed distance field (MTSDF), made at run
//! time from the same skrifa outline the coverage raster reads: the second
//! raster tier, for world and facing text, text under a transform that is
//! not axis-aligned or that changes every frame, glyphs above the coverage
//! cache's size, and effects (outline, glow) drawn from the field.
//!
//! This module is the adapter only: skrifa draws the unhinted outline into
//! fdsm's shape, [`crate::overlap`] resolves overlapping contours, and fdsm
//! colours the edges, generates the field, corrects its signs and its
//! errors, in the steps the cook's atlas took. fdsm is vendored with libm's
//! maths (third_party/fdsm/PATCHED.md), so both hosts make the same bytes.

use fdsm::bezier::scanline::FillRule;
use fdsm::bezier::{Point, Segment};
use fdsm::shape::{Contour, Shape};
use fdsm::transform::Transform;
use skrifa::instance::{LocationRef, NormalizedCoord, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::{FontRef, GlyphId, MetadataProvider};

/// Field pixels per em.
pub const EM_PX: f32 = 48.0;
/// The distances the field holds, field pixels across: `-RANGE / 2` to
/// `RANGE / 2` around the edge.
pub const RANGE: f32 = 12.0;

/// A glyph's field: `width` × `height` RGBA texels, rows from the top,
/// whose top left texel sits `left` field pixels right of the pen and `top`
/// above the baseline. Red, green and blue hold the multi-channel distance,
/// alpha the true distance, each 0 at `-RANGE / 2` and 255 at `RANGE / 2`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Field {
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
}

/// Makes glyph `id`'s field at variation `coords` into `out` (replaced).
/// `None` when the face has no outline for it; a glyph with no ink (a
/// space) is a field of zero size.
pub fn field(
    font: &FontRef,
    id: GlyphId,
    coords: &[NormalizedCoord],
    out: &mut Vec<u8>,
) -> Option<Field> {
    out.clear();
    let outline = font.outline_glyphs().get(id)?;
    let mut pen = ShapePen::default();
    let settings = DrawSettings::unhinted(Size::new(EM_PX), LocationRef::new(coords));
    outline.draw(settings, &mut pen).ok()?;
    pen.close();
    let shape = crate::overlap::remove_overlaps(pen.shape);
    let Some([x0, y0, x1, y1]) = bounds(&shape) else {
        return Some(Field::default());
    };
    let pad = (RANGE / 2.0).ceil() as i32 + 1;
    let left = x0.floor() as i32 - pad;
    let bottom = y0.floor() as i32 - pad;
    let width = (x1.ceil() as i32 + pad - left) as u32;
    let height = (y1.ceil() as i32 + pad - bottom) as u32;
    let mut shape = shape;
    shape.transform(&nalgebra::convert::<_, nalgebra::Affine2<f64>>(
        nalgebra::Translation2::new(-f64::from(left), -f64::from(bottom)),
    ));
    let colored = Shape::edge_coloring_simple(shape, 0.03, 0x6f67_7465_7874);
    let prepared = colored.prepare();
    let range = f64::from(RANGE);
    let mut f: image::ImageBuffer<image::Rgba<f32>, Vec<f32>> =
        image::ImageBuffer::new(width, height);
    fdsm::generate::generate_mtsdf(&prepared, range, &mut f);
    // The cook's order: signs first, then error correction without
    // distance checks.
    fdsm::render::correct_sign_mtsdf(&mut f, &prepared, FillRule::Nonzero);
    let mut correction = fdsm::correct_error::ErrorCorrectionConfig::default();
    correction.distance_check = fdsm::correct_error::DistanceCheckMode::Never;
    fdsm::correct_error::correct_error_mtsdf(&mut f, &colored, &prepared, range, &correction);
    // Field rows are y up; out's rows run from the top.
    out.reserve((width * height * 4) as usize);
    for y in (0..height).rev() {
        for x in 0..width {
            for &v in &f.get_pixel(x, y).0 {
                out.push((v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
            }
        }
    }
    Some(Field {
        left,
        top: bottom + height as i32,
        width,
        height,
    })
}

/// A shape's control-point box (left, bottom, right, top), which holds
/// its curves; `None` for no contour.
fn bounds(shape: &Shape<Contour>) -> Option<[f64; 4]> {
    let mut b: Option<[f64; 4]> = None;
    for s in shape.contours.iter().flat_map(|c| &c.segments) {
        for i in 0..=s.order() as usize + 1 {
            let p = s.control_point(i);
            let v = b.get_or_insert([p.x, p.y, p.x, p.y]);
            *v = [v[0].min(p.x), v[1].min(p.y), v[2].max(p.x), v[3].max(p.y)];
        }
    }
    b.filter(|b| b[2] > b[0] && b[3] > b[1])
}

/// An outline pen building fdsm's shape, each contour closed.
#[derive(Default)]
struct ShapePen {
    shape: Shape<Contour>,
    start: Option<Point>,
    at: Point,
}

impl ShapePen {
    fn push(&mut self, s: Segment) {
        if let Some(c) = self.shape.contours.last_mut() {
            c.segments.push(s);
        }
    }

    fn p(x: f32, y: f32) -> Point {
        Point::new(f64::from(x), f64::from(y))
    }
}

impl OutlinePen for ShapePen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.close();
        let p = Self::p(x, y);
        self.start = Some(p);
        self.at = p;
        self.shape.contours.push(Contour::default());
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let p = Self::p(x, y);
        if p != self.at {
            self.push(Segment::line(self.at, p));
        }
        self.at = p;
    }

    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let p = Self::p(x, y);
        self.push(Segment::quad(self.at, Self::p(cx, cy), p));
        self.at = p;
    }

    fn curve_to(&mut self, c0x: f32, c0y: f32, c1x: f32, c1y: f32, x: f32, y: f32) {
        let p = Self::p(x, y);
        self.push(Segment::cubic(
            self.at,
            Self::p(c0x, c0y),
            Self::p(c1x, c1y),
            p,
        ));
        self.at = p;
    }

    fn close(&mut self) {
        if let Some(start) = self.start.take()
            && start != self.at
        {
            self.push(Segment::line(self.at, start));
            self.at = start;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes(path: &str) -> Vec<u8> {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../templates/");
        std::fs::read(format!("{root}{path}")).unwrap()
    }

    /// The field draws the glyph: thresholded at its middle (the median of
    /// red, green and blue at 0.5, as the glyph shader reads it), each
    /// texel is inside exactly where the coverage raster of the same glyph
    /// at the same size is more than half covered, but for texels on the
    /// edge itself (1 % may differ at corners, which a distance field
    /// rounds). No texel 1 px or more inside the fill reads as an edge:
    /// variable Latin and ideographs, whose contours overlap, where a seam
    /// left by an overlap would.
    #[test]
    fn the_field_draws_what_the_raster_covers() {
        // The variable builds: their contours overlap, as static
        // instances' do not.
        let sc = bytes("styles/cjk/NotoSansSC[wght].ttf");
        let latin = bytes("styles/shared/InstrumentSans[wdth,wght].ttf");
        let mut r = crate::raster::Rasterizer::new();
        let (mut cov, mut out) = (Vec::new(), Vec::new());
        for (data, text) in [(&latin, "Qg8&"), (&sc, "游戏设置")] {
            let font = FontRef::new(data).unwrap();
            for c in text.chars() {
                let id = font.charmap().map(c).unwrap();
                let f = field(&font, id, &[], &mut out).unwrap();
                assert_eq!(out.len(), (f.width * f.height * 4) as usize);
                let g = crate::raster::Glyph {
                    font: &font,
                    id,
                    size: EM_PX,
                    coords: &[],
                    hinting: None,
                    x_offset: 0.0,
                };
                let p = r.raster(g, &mut cov).unwrap();
                let at = |x: i32, y: i32| {
                    let (rx, ry) = (f.left + x - p.left, p.top - (f.top - y));
                    if (0..p.width as i32).contains(&rx) && (0..p.height as i32).contains(&ry) {
                        cov[(ry * p.width as i32 + rx) as usize]
                    } else {
                        0
                    }
                };
                let (mut wrong, mut inside, mut seams) = (0, 0, 0);
                for y in 0..f.height as i32 {
                    for x in 0..f.width as i32 {
                        let t = &out[((y * f.width as i32 + x) * 4) as usize..][..4];
                        let mut m = [t[0], t[1], t[2]];
                        m.sort_unstable();
                        let in_field = m[1] > 127;
                        // The same texel in the raster's grid.
                        let c = at(x, y);
                        // A texel 1 px or more inside the fill lies at
                        // least 1/12 of the range inside the edge.
                        let deep = (-1..=1).all(|dy| (-1..=1).all(|dx| at(x + dx, y + dy) == 255));
                        seams += usize::from(deep && m[1] < 140);
                        // Texels the edge crosses are left out.
                        if !(40..=215).contains(&c) {
                            wrong += usize::from(in_field != (c > 127));
                            inside += usize::from(c > 127);
                        }
                    }
                }
                assert!(inside > 0, "{c}: no ink");
                assert!(
                    wrong * 100 <= inside,
                    "{c}: {wrong} of {inside} texels disagree"
                );
                assert_eq!(seams, 0, "{c}: texels deep inside read as edge");
            }
        }
    }
}
