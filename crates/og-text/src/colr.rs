//! A colour glyph (COLRv0 layers, COLRv1 paint graphs) painted at its
//! drawn size into premultiplied RGBA. The colour tier of the glyph cache.
//!
//! This module is the adapter only. skrifa walks the glyph's paint graph
//! (`ColorGlyph::paint`) and resolves its variations, and tiny-skia does
//! the painting: clips by glyph and box, solid fills, linear, two-circle
//! radial and sweep gradients with their extend modes, and every composite
//! mode. tiny-skia is vendored with exact reciprocals
//! (third_party/tiny-skia/PATCHED.md), so both hosts paint the same bytes.

use crate::raster::Placement;
use skrifa::color::{Brush, ColorPainter, ColorStop, CompositeMode, Extend, Transform};
use skrifa::instance::{LocationRef, NormalizedCoord, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::raw::TableProvider;
use skrifa::raw::types::BoundingBox;
use skrifa::{FontRef, GlyphId, MetadataProvider};
use tiny_skia as ts;

/// What to paint: a colour glyph of a face at a size and position, its
/// palette's colours (CSS `font-palette`, resolved) and the text colour
/// that palette entry 0xFFFF names, each non-premultiplied sRGB, 0 to 1.
#[derive(Clone, Copy)]
pub struct ColorGlyph<'a> {
    pub font: &'a FontRef<'a>,
    pub id: GlyphId,
    /// Device pixels per em.
    pub size: f32,
    pub coords: &'a [NormalizedCoord],
    /// The pen's fraction of a pixel in x.
    pub x_offset: f32,
    pub palette: &'a [[f32; 4]],
    pub foreground: [f32; 4],
}

/// Paints `glyph` into `out` (replaced) as premultiplied RGBA, rows from
/// the top, placed as [`crate::raster::Rasterizer::raster`] places a
/// glyph. `None` when the face has no colour glyph for it.
pub fn paint(glyph: ColorGlyph, out: &mut Vec<u8>) -> Option<Placement> {
    out.clear();
    let glyphs = glyph.font.color_glyphs();
    let color = glyphs.get(glyph.id)?;
    let location = LocationRef::new(glyph.coords);
    let upem = f32::from(glyph.font.head().ok()?.units_per_em().max(1));
    let scale = glyph.size / upem;
    // The glyph's box in font units: its clip box, else its layers' union.
    let bounds = match color.bounding_box(location, Size::unscaled()) {
        Some(b) => b,
        None => {
            let mut b = Bounds {
                font: glyph.font,
                coords: glyph.coords,
                transforms: vec![Transform::default()],
                bounds: None,
            };
            color.paint(location, &mut b).ok()?;
            b.bounds?
        }
    };
    let x0 = bounds.x_min * scale + glyph.x_offset;
    let x1 = bounds.x_max * scale + glyph.x_offset;
    let left = x0.floor() as i32;
    let top = (bounds.y_max * scale).ceil() as i32;
    let width = (x1.ceil() as i32 - left).max(0) as u32;
    let height = (top - (bounds.y_min * scale).floor() as i32).max(0) as u32;
    let Some(pixmap) = ts::Pixmap::new(width, height) else {
        return Some(Placement::default());
    };
    // Font units (y up) to the pixmap's pixels (y down).
    let base = ts::Transform::from_row(
        scale,
        0.0,
        0.0,
        -scale,
        glyph.x_offset - left as f32,
        top as f32,
    );
    let mut painter = Painter {
        glyph,
        transforms: vec![base],
        clips: vec![None],
        layers: vec![(pixmap, CompositeMode::SrcOver)],
    };
    color.paint(location, &mut painter).ok()?;
    let (pixmap, _) = painter.layers.pop()?;
    *out = pixmap.take();
    Some(Placement {
        left,
        top,
        width,
        height,
    })
}

/// skrifa's transform as tiny-skia's.
fn transform(t: Transform) -> ts::Transform {
    ts::Transform::from_row(t.xx, t.yx, t.xy, t.yy, t.dx, t.dy)
}

/// A glyph's outline in font units as a tiny-skia path.
fn outline(font: &FontRef, coords: &[NormalizedCoord], id: GlyphId) -> Option<ts::Path> {
    struct P(ts::PathBuilder);
    impl OutlinePen for P {
        fn move_to(&mut self, x: f32, y: f32) {
            self.0.move_to(x, y);
        }
        fn line_to(&mut self, x: f32, y: f32) {
            self.0.line_to(x, y);
        }
        fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
            self.0.quad_to(cx, cy, x, y);
        }
        fn curve_to(&mut self, ax: f32, ay: f32, bx: f32, by: f32, x: f32, y: f32) {
            self.0.cubic_to(ax, ay, bx, by, x, y);
        }
        fn close(&mut self) {
            self.0.close();
        }
    }
    let mut pen = P(ts::PathBuilder::new());
    let settings = DrawSettings::unhinted(Size::unscaled(), LocationRef::new(coords));
    font.outline_glyphs()
        .get(id)?
        .draw(settings, &mut pen)
        .ok()?;
    pen.0.finish()
}

/// The union of the boxes of the glyphs a paint graph clips by, under
/// their transforms: a COLRv0 glyph's box, which has no clip box.
struct Bounds<'a> {
    font: &'a FontRef<'a>,
    coords: &'a [NormalizedCoord],
    transforms: Vec<Transform>,
    bounds: Option<BoundingBox<f32>>,
}

impl ColorPainter for Bounds<'_> {
    fn push_transform(&mut self, t: Transform) {
        let top = *self.transforms.last().unwrap_or(&Transform::default());
        self.transforms.push(top * t);
    }

    fn pop_transform(&mut self) {
        self.transforms.pop();
    }

    fn push_clip_glyph(&mut self, id: GlyphId) {
        let Some(path) = outline(self.font, self.coords, id) else {
            return;
        };
        let t = transform(*self.transforms.last().unwrap_or(&Transform::default()));
        let Some(path) = path.transform(t) else {
            return;
        };
        let r = path.bounds();
        let b = self.bounds.get_or_insert(BoundingBox {
            x_min: r.left(),
            y_min: r.top(),
            x_max: r.right(),
            y_max: r.bottom(),
        });
        b.x_min = b.x_min.min(r.left());
        b.y_min = b.y_min.min(r.top());
        b.x_max = b.x_max.max(r.right());
        b.y_max = b.y_max.max(r.bottom());
    }

    fn push_clip_box(&mut self, _: BoundingBox<f32>) {}
    fn pop_clip(&mut self) {}
    fn fill(&mut self, _: Brush<'_>) {}
    fn push_layer(&mut self, _: CompositeMode) {}
}

/// Paints a paint graph with tiny-skia: a transform stack (font units to
/// pixels at its root), a clip stack and a layer stack.
struct Painter<'a> {
    glyph: ColorGlyph<'a>,
    transforms: Vec<ts::Transform>,
    clips: Vec<Option<ts::Mask>>,
    layers: Vec<(ts::Pixmap, CompositeMode)>,
}

impl Painter<'_> {
    fn top(&self) -> ts::Transform {
        *self.transforms.last().unwrap_or(&ts::Transform::identity())
    }

    /// Clips to `path` in font units, within the clip already in force.
    fn clip(&mut self, path: Option<ts::Path>) {
        let (w, h) = self.size();
        let mut mask = match self.clips.last() {
            Some(Some(m)) => m.clone(),
            _ => {
                let mut m = ts::Mask::new(w, h).expect("a non-empty glyph");
                m.data_mut().fill(255);
                m
            }
        };
        match path {
            Some(p) => mask.intersect_path(&p, ts::FillRule::Winding, true, self.top()),
            None => mask.data_mut().fill(0),
        }
        self.clips.push(Some(mask));
    }

    fn size(&self) -> (u32, u32) {
        let p = &self.layers[0].0;
        (p.width(), p.height())
    }

    /// Palette entry `index` at `alpha`, 0xFFFF the text colour.
    fn color(&self, index: u16, alpha: f32) -> ts::Color {
        let c = if index == 0xFFFF {
            self.glyph.foreground
        } else {
            self.glyph
                .palette
                .get(usize::from(index))
                .copied()
                .unwrap_or([0.0; 4])
        };
        ts::Color::from_rgba(c[0], c[1], c[2], (c[3] * alpha).clamp(0.0, 1.0))
            .unwrap_or(ts::Color::TRANSPARENT)
    }

    fn stops(&self, stops: &[ColorStop]) -> Vec<ts::GradientStop> {
        stops
            .iter()
            .map(|s| ts::GradientStop::new(s.offset, self.color(s.palette_index, s.alpha)))
            .collect()
    }
}

fn spread(extend: Extend) -> ts::SpreadMode {
    match extend {
        Extend::Repeat => ts::SpreadMode::Repeat,
        Extend::Reflect => ts::SpreadMode::Reflect,
        _ => ts::SpreadMode::Pad,
    }
}

fn blend(mode: CompositeMode) -> ts::BlendMode {
    use CompositeMode as C;
    use ts::BlendMode as B;
    match mode {
        C::Clear => B::Clear,
        C::Src => B::Source,
        C::Dest => B::Destination,
        C::DestOver => B::DestinationOver,
        C::SrcIn => B::SourceIn,
        C::DestIn => B::DestinationIn,
        C::SrcOut => B::SourceOut,
        C::DestOut => B::DestinationOut,
        C::SrcAtop => B::SourceAtop,
        C::DestAtop => B::DestinationAtop,
        C::Xor => B::Xor,
        C::Plus => B::Plus,
        C::Screen => B::Screen,
        C::Overlay => B::Overlay,
        C::Darken => B::Darken,
        C::Lighten => B::Lighten,
        C::ColorDodge => B::ColorDodge,
        C::ColorBurn => B::ColorBurn,
        C::HardLight => B::HardLight,
        C::SoftLight => B::SoftLight,
        C::Difference => B::Difference,
        C::Exclusion => B::Exclusion,
        C::Multiply => B::Multiply,
        C::HslHue => B::Hue,
        C::HslSaturation => B::Saturation,
        C::HslColor => B::Color,
        C::HslLuminosity => B::Luminosity,
        _ => B::SourceOver,
    }
}

impl ColorPainter for Painter<'_> {
    fn push_transform(&mut self, t: Transform) {
        let top = self.top();
        self.transforms.push(top.pre_concat(transform(t)));
    }

    fn pop_transform(&mut self) {
        self.transforms.pop();
    }

    fn push_clip_glyph(&mut self, id: GlyphId) {
        let path = outline(self.glyph.font, self.glyph.coords, id);
        self.clip(path);
    }

    fn push_clip_box(&mut self, b: BoundingBox<f32>) {
        let rect = ts::Rect::from_ltrb(b.x_min, b.y_min, b.x_max, b.y_max);
        self.clip(rect.map(ts::PathBuilder::from_rect));
    }

    fn pop_clip(&mut self) {
        self.clips.pop();
    }

    fn fill(&mut self, brush: Brush<'_>) {
        let t = self.top();
        let p = |p: skrifa::raw::types::Point<f32>| ts::Point::from_xy(p.x, p.y);
        let shader = match brush {
            Brush::Solid {
                palette_index,
                alpha,
            } => Some(ts::Shader::SolidColor(self.color(palette_index, alpha))),
            Brush::LinearGradient {
                p0,
                p1,
                color_stops,
                extend,
            } => ts::LinearGradient::new(p(p0), p(p1), self.stops(color_stops), spread(extend), t),
            Brush::RadialGradient {
                c0,
                r0,
                c1,
                r1,
                color_stops,
                extend,
            } => ts::RadialGradient::new(
                p(c0),
                r0.max(0.0),
                p(c1),
                r1.max(0.0),
                self.stops(color_stops),
                spread(extend),
                t,
            ),
            Brush::SweepGradient {
                c0,
                start_angle,
                end_angle,
                color_stops,
                extend,
            } => ts::SweepGradient::new(
                p(c0),
                start_angle,
                end_angle,
                self.stops(color_stops),
                spread(extend),
                t,
            ),
        };
        let Some(shader) = shader else {
            return;
        };
        let paint = ts::Paint {
            shader,
            anti_alias: true,
            ..Default::default()
        };
        let (w, h) = self.size();
        let rect = ts::Rect::from_xywh(0.0, 0.0, w as f32, h as f32).expect("a non-empty glyph");
        let mask = self.clips.last().and_then(|m| m.as_ref());
        let layer = &mut self.layers.last_mut().expect("the root layer").0;
        layer.fill_rect(rect, &paint, ts::Transform::identity(), mask);
    }

    fn push_layer(&mut self, mode: CompositeMode) {
        let (w, h) = self.size();
        let pixmap = ts::Pixmap::new(w, h).expect("a non-empty glyph");
        self.layers.push((pixmap, mode));
    }

    fn pop_layer(&mut self) {
        if self.layers.len() < 2 {
            return;
        }
        let (src, mode) = self.layers.pop().expect("a pushed layer");
        let paint = ts::PixmapPaint {
            blend_mode: blend(mode),
            ..Default::default()
        };
        let dst = &mut self.layers.last_mut().expect("the root layer").0;
        dst.draw_pixmap(0, 0, src.as_ref(), &paint, ts::Transform::identity(), None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `tests/fixtures/make-palette-font.py`: A a COLRv0 square in entry 0
    /// with its right half in entry 1, B a COLRv1 linear gradient from
    /// entry 0 at its left to entry 1 at its right, C a COLRv0 layer in the
    /// text's colour; every glyph an em wide and 0.8 em tall.
    fn font() -> Vec<u8> {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/palette-test.ttf"
        );
        std::fs::read(path).expect("bun scripts/assets.ts ensure crates/og-text")
    }

    /// The premultiplied pixel at `(x, y)` em from the glyph's origin.
    fn at(font: &FontRef, c: char, palette: &[[f32; 4]], x: f32, y: f32) -> [u8; 4] {
        let size = 64.0;
        let mut out = Vec::new();
        let g = ColorGlyph {
            font,
            id: font.charmap().map(c).unwrap(),
            size,
            coords: &[],
            x_offset: 0.0,
            palette,
            foreground: [0.0, 0.0, 1.0, 1.0],
        };
        let p = paint(g, &mut out).unwrap();
        assert_eq!(out.len(), (p.width * p.height * 4) as usize);
        let px = (x * size) as i32 - p.left;
        let py = p.top - (y * size) as i32 - 1;
        let i = ((py * p.width as i32 + px) * 4) as usize;
        out[i..i + 4].try_into().unwrap()
    }

    /// Layers paint in their palette entries, the gradient runs between
    /// them in the glyph's own direction, entry 0xFFFF takes the text
    /// colour, and another palette repaints the same glyph.
    #[test]
    fn colour_glyphs_paint_their_layers_gradients_and_palette() {
        let data = font();
        let font = FontRef::new(&data).unwrap();
        let red_green = [[1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 1.0]];
        let blue_yellow = [[0.0, 0.0, 1.0, 1.0], [1.0, 1.0, 0.0, 1.0]];
        assert_eq!(at(&font, 'A', &red_green, 0.25, 0.4), [255, 0, 0, 255]);
        assert_eq!(at(&font, 'A', &red_green, 0.75, 0.4), [0, 255, 0, 255]);
        assert_eq!(at(&font, 'A', &blue_yellow, 0.75, 0.4), [255, 255, 0, 255]);
        let left = at(&font, 'B', &red_green, 0.1, 0.4);
        let right = at(&font, 'B', &red_green, 0.9, 0.4);
        assert!(left[0] > 200 && left[1] < 55, "{left:?}");
        assert!(right[1] > 200 && right[0] < 55, "{right:?}");
        let mid = at(&font, 'B', &red_green, 0.5, 0.4);
        assert!(
            (100..156).contains(&mid[0]) && (100..156).contains(&mid[1]),
            "{mid:?}"
        );
        assert_eq!(at(&font, 'C', &red_green, 0.5, 0.4), [0, 0, 255, 255]);
        // Outside the 0.8 em tall glyph nothing is painted.
        let p = {
            let mut out = Vec::new();
            let g = ColorGlyph {
                font: &font,
                id: font.charmap().map('A').unwrap(),
                size: 64.0,
                coords: &[],
                x_offset: 0.0,
                palette: &red_green,
                foreground: [0.0; 4],
            };
            paint(g, &mut out).unwrap()
        };
        assert_eq!((p.left, p.top, p.width, p.height), (0, 52, 64, 52));
        assert!(
            paint(
                ColorGlyph {
                    font: &font,
                    id: font.charmap().map('D').unwrap(),
                    size: 64.0,
                    coords: &[],
                    x_offset: 0.0,
                    palette: &red_green,
                    foreground: [0.0; 4],
                },
                &mut Vec::new()
            )
            .is_none()
        );
    }

    /// A COLRv1 paint graph stands the right way up: Noto's 🔺 (the
    /// shipped emoji pack) is narrow at its top and wide at its bottom.
    #[test]
    fn a_colour_glyph_stands_upright() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../templates/fonts/emoji/Noto-COLRv1.ttf"
        );
        let data = std::fs::read(path).unwrap();
        let font = FontRef::new(&data).unwrap();
        let palette: Vec<[f32; 4]> = {
            let cpal = font.cpal().unwrap();
            let records = cpal.color_records_array().unwrap().unwrap();
            let n = usize::from(cpal.num_palette_entries());
            records[..n]
                .iter()
                .map(|c| [c.red, c.green, c.blue, c.alpha].map(|v| f32::from(v) / 255.0))
                .collect()
        };
        let mut out = Vec::new();
        let g = ColorGlyph {
            font: &font,
            id: font.charmap().map('\u{1F53A}').unwrap(),
            size: 64.0,
            coords: &[],
            x_offset: 0.0,
            palette: &palette,
            foreground: [0.0; 4],
        };
        let p = paint(g, &mut out).unwrap();
        let rows: Vec<u32> = out
            .chunks(p.width as usize * 4)
            .map(|r| r.chunks(4).map(|px| u32::from(px[3])).sum())
            .collect();
        let inked: Vec<u32> = rows.into_iter().filter(|&r| r > 0).collect();
        let q = inked.len() / 4;
        let (top, bottom): (u32, u32) = (
            inked[..q].iter().sum(),
            inked[inked.len() - q..].iter().sum(),
        );
        assert!(
            top * 2 < bottom,
            "top quarter {top}, bottom quarter {bottom}"
        );
    }
}
