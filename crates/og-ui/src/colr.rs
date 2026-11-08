//! COLR colour glyphs drawn into images, at cook time (`font { color true
//! }`) and at run time for a colour font that ships its file (`color true`
//! with `runtime true`, og-runtime's glyphs.rs): skrifa walks a glyph's COLR v0 or v1 paint graph (layers, solid
//! and gradient brushes, transforms, glyph and box clips, composite modes)
//! and this painter draws it into RGBA at `color_px` pixels per em. The
//! cooked image is drawn as a textured quad on the font's colour pages, so
//! a colour glyph costs a run-time text what a sprite costs, on either host.
//!
//! Coverage is the exact area under each flattened edge, accumulated and
//! summed along the rows (font-rs's accumulation rasterizer, Raph Levien,
//! Apache-2.0, `third_party/font-rs/`). Colours are composited and
//! gradients interpolated in premultiplied sRGB, as browsers draw COLR;
//! the blend modes are the W3C Compositing and Blending Level 1 formulas.

use skrifa::color::{Brush, ColorPainter, ColorStop, CompositeMode, Extend, Transform};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::prelude::{LocationRef, Size};
use skrifa::raw::TableProvider;
use skrifa::raw::types::BoundingBox;
use skrifa::{FontRef, GlyphId, MetadataProvider};

/// A colour glyph's image: straight-alpha sRGB rows top to bottom, its
/// transparent pixels carrying their neighbours' colour so filtering and
/// mips never darken an edge.
pub struct ColorImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    /// The quad around the glyph relative to the pen on the baseline, em,
    /// y up: left, bottom, right, top.
    pub plane: [f32; 4],
    /// Its paint uses the text's colour (palette entry 0xFFFF), drawn in
    /// the `foreground` it was rendered with.
    pub foreground: bool,
}

/// The largest image edge, in em: a glyph whose paint reaches further is
/// cut there (the COLR clip boxes of real fonts stay well inside).
const MAX_EM: f32 = 4.0;

/// Transparent pixels round an image's ink.
const PAD: i32 = 4;

/// Draws glyph `gid` in colour at `em_px` pixels per em, palette entry
/// 0xFFFF (the text's colour) as `foreground` (sRGB, opaque); `None` when
/// it has no colour paint or paints nothing.
pub fn render(
    font: &FontRef,
    gid: GlyphId,
    em_px: f32,
    palette: &[[f32; 4]],
    foreground: [f32; 3],
) -> Result<Option<ColorImage>, String> {
    let Some(glyph) = font.color_glyphs().get(gid) else {
        return Ok(None);
    };
    let upem = font.head().map(|h| h.units_per_em()).unwrap_or(1000).max(1);
    let scale = f64::from(em_px) / f64::from(upem);
    let location = LocationRef::default();
    // Bounds in font units: the clip box, else what the paint reaches.
    let bounds = match glyph.bounding_box(location, Size::unscaled()) {
        Some(b) => Some(Rect::of(b)),
        None => {
            let mut b = Bounds::new(font);
            glyph
                .paint(location, &mut b)
                .map_err(|e| format!("COLR glyph {}: {e}", gid.to_u32()))?;
            b.reach
        }
    };
    let Some(r) = bounds.filter(|r| r.x1 > r.x0 && r.y1 > r.y0) else {
        return Ok(None);
    };
    let reach = f64::from(MAX_EM) * f64::from(upem);
    let r = Rect {
        x0: r.x0.max(-reach),
        y0: r.y0.max(-reach),
        x1: r.x1.min(reach),
        y1: r.y1.min(reach),
    };
    // Transparent pixels round the ink: its antialiased edge, and room for
    // the mips the page is drawn with to blur it without reaching a
    // neighbour's.
    let left = (r.x0 * scale).floor() as i32 - PAD;
    let right = (r.x1 * scale).ceil() as i32 + PAD;
    let bottom = (r.y0 * scale).floor() as i32 - PAD;
    let top = (r.y1 * scale).ceil() as i32 + PAD;
    let (w, h) = ((right - left) as usize, (top - bottom) as usize);
    // Font units to pixels, y down.
    let device = Affine([scale, 0.0, 0.0, -scale, -f64::from(left), f64::from(top)]);
    let mut p = Raster {
        font,
        w,
        h,
        palette,
        foreground,
        used_foreground: std::cell::Cell::new(false),
        transforms: vec![device],
        clips: Vec::new(),
        layers: vec![(vec![[0.0; 4]; w * h], CompositeMode::SrcOver)],
    };
    glyph
        .paint(location, &mut p)
        .map_err(|e| format!("COLR glyph {}: {e}", gid.to_u32()))?;
    let canvas = p.layers.swap_remove(0).0;
    let px = em_px;
    Ok(Some(ColorImage {
        width: w as u32,
        height: h as u32,
        rgba: encode(&canvas, w, h),
        plane: [
            left as f32 / px,
            bottom as f32 / px,
            right as f32 / px,
            top as f32 / px,
        ],
        foreground: p.used_foreground.get(),
    }))
}

/// A colour glyph painted at run time (`color true` with `runtime true`):
/// what a colour font's cooked glyph carries, made from its file when a
/// text first shows it.
pub struct RuntimeColor {
    pub key: og_render::GlyphKey,
    /// Horizontal advance in em.
    pub advance: f32,
    /// Painted with a black foreground (the only paint unless
    /// `image.foreground`).
    pub image: ColorImage,
    /// With `image.foreground`: the same glyph painted with a white
    /// foreground, the twin the renderer mixes by the text's colour.
    pub white: Option<Vec<u8>>,
    /// The silhouette's distance field ([`silhouette_field`] at
    /// [`FIELD_RANGE`]), the image's size: its shadows, glow and outline.
    pub field: Vec<u8>,
}

/// Paints glyph `key` of font file `ttf` as the cook paints a colour
/// glyph, at `em_px` pixels per em in `palette` (the cooked font's); `None`
/// when the font lacks it or it has no colour paint (an outline glyph).
pub fn render_runtime(
    ttf: &[u8],
    key: og_render::GlyphKey,
    em_px: f32,
    palette: &[[f32; 4]],
) -> Option<RuntimeColor> {
    let font = FontRef::new(ttf).ok()?;
    let gid = match key {
        og_render::GlyphKey::Char(c) => font.charmap().map(c)?,
        og_render::GlyphKey::Id(id) => GlyphId::new(u32::from(id)),
    };
    let image = render(&font, gid, em_px, palette, [0.0; 3]).ok()??;
    let white = if image.foreground {
        Some(render(&font, gid, em_px, palette, [1.0; 3]).ok()??.rgba)
    } else {
        None
    };
    let upem = f32::from(font.head().ok()?.units_per_em().max(1));
    let advance = font
        .glyph_metrics(Size::unscaled(), LocationRef::default())
        .advance_width(gid)
        .unwrap_or(0.0)
        / upem;
    let field = silhouette_field(
        &image.rgba,
        image.width as usize,
        image.height as usize,
        FIELD_RANGE,
    );
    Some(RuntimeColor {
        key,
        advance,
        image,
        white,
        field,
    })
}

/// Premultiplied sRGB to straight 8-bit, then the colour of the nearest
/// ink spread into the transparent pixels.
fn encode(canvas: &[[f32; 4]], w: usize, h: usize) -> Vec<u8> {
    let mut rgb: Vec<Option<[f32; 3]>> = canvas
        .iter()
        .map(|c| (c[3] > 1.0 / 512.0).then(|| [c[0] / c[3], c[1] / c[3], c[2] / c[3]]))
        .collect();
    for _ in 0..8 {
        let before = rgb.clone();
        let mut grew = false;
        for y in 0..h {
            for x in 0..w {
                if before[y * w + x].is_some() {
                    continue;
                }
                let (mut sum, mut n) = ([0.0f32; 3], 0.0f32);
                for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                    let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                        continue;
                    }
                    if let Some(c) = before[ny as usize * w + nx as usize] {
                        (0..3).for_each(|k| sum[k] += c[k]);
                        n += 1.0;
                    }
                }
                if n > 0.0 {
                    rgb[y * w + x] = Some(sum.map(|v| v / n));
                    grew = true;
                }
            }
        }
        if !grew {
            break;
        }
    }
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    canvas
        .iter()
        .zip(&rgb)
        .flat_map(|(c, rgb)| {
            let [r, g, b] = rgb.unwrap_or([0.0; 3]);
            [byte(r), byte(g), byte(b), byte(c[3])]
        })
        .collect()
}

/// A 2D affine map `[xx, yx, xy, yy, dx, dy]`: x' = xx x + xy y + dx,
/// y' = yx x + yy y + dy.
#[derive(Clone, Copy, Debug)]
struct Affine([f64; 6]);

impl Affine {
    fn of(t: &Transform) -> Affine {
        Affine([t.xx, t.yx, t.xy, t.yy, t.dx, t.dy].map(f64::from))
    }

    /// `self` after `o`: o maps first.
    fn then(&self, o: &Affine) -> Affine {
        let [a, b, c, d, e, f] = self.0;
        let [oa, ob, oc, od, oe, of] = o.0;
        Affine([
            a * oa + c * ob,
            b * oa + d * ob,
            a * oc + c * od,
            b * oc + d * od,
            a * oe + c * of + e,
            b * oe + d * of + f,
        ])
    }

    fn apply(&self, x: f64, y: f64) -> (f64, f64) {
        let [a, b, c, d, e, f] = self.0;
        (a * x + c * y + e, b * x + d * y + f)
    }

    fn inverse(&self) -> Option<Affine> {
        let [a, b, c, d, e, f] = self.0;
        let det = a * d - b * c;
        if det.abs() < 1e-12 {
            return None;
        }
        let (ia, ib, ic, id) = (d / det, -b / det, -c / det, a / det);
        Some(Affine([
            ia,
            ib,
            ic,
            id,
            -(ia * e + ic * f),
            -(ib * e + id * f),
        ]))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Rect {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
}

impl Rect {
    fn of(b: BoundingBox<f32>) -> Rect {
        Rect {
            x0: f64::from(b.x_min),
            y0: f64::from(b.y_min),
            x1: f64::from(b.x_max),
            y1: f64::from(b.y_max),
        }
    }

    fn corners(&self) -> [(f64, f64); 4] {
        [
            (self.x0, self.y0),
            (self.x1, self.y0),
            (self.x1, self.y1),
            (self.x0, self.y1),
        ]
    }

    fn around(points: impl IntoIterator<Item = (f64, f64)>) -> Option<Rect> {
        points.into_iter().fold(None, |r: Option<Rect>, (x, y)| {
            Some(match r {
                None => Rect {
                    x0: x,
                    y0: y,
                    x1: x,
                    y1: y,
                },
                Some(r) => Rect {
                    x0: r.x0.min(x),
                    y0: r.y0.min(y),
                    x1: r.x1.max(x),
                    y1: r.y1.max(y),
                },
            })
        })
    }

    fn meet(&self, o: &Rect) -> Rect {
        Rect {
            x0: self.x0.max(o.x0),
            y0: self.y0.max(o.y0),
            x1: self.x1.min(o.x1),
            y1: self.y1.min(o.y1),
        }
    }

    fn join(&self, o: &Rect) -> Rect {
        Rect {
            x0: self.x0.min(o.x0),
            y0: self.y0.min(o.y0),
            x1: self.x1.max(o.x1),
            y1: self.y1.max(o.y1),
        }
    }
}

/// A glyph outline's points under a map (control points bound the curves).
struct Points<'a>(&'a Affine, Vec<(f64, f64)>);

impl OutlinePen for Points<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.1.push(self.0.apply(x.into(), y.into()));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.1.push(self.0.apply(x.into(), y.into()));
    }
    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.1.push(self.0.apply(cx0.into(), cy0.into()));
        self.1.push(self.0.apply(x.into(), y.into()));
    }
    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.1.push(self.0.apply(cx0.into(), cy0.into()));
        self.1.push(self.0.apply(cx1.into(), cy1.into()));
        self.1.push(self.0.apply(x.into(), y.into()));
    }
    fn close(&mut self) {}
}

fn outline(font: &FontRef, gid: GlyphId, pen: &mut impl OutlinePen) {
    if let Some(g) = font.outline_glyphs().get(gid) {
        let _ = g.draw(
            DrawSettings::unhinted(Size::unscaled(), LocationRef::default()),
            pen,
        );
    }
}

/// What a paint graph reaches in font units: the union of its fills, each
/// cut by the clips around it.
struct Bounds<'a> {
    font: &'a FontRef<'a>,
    transforms: Vec<Affine>,
    clips: Vec<Option<Rect>>,
    reach: Option<Rect>,
}

impl<'a> Bounds<'a> {
    fn new(font: &'a FontRef<'a>) -> Bounds<'a> {
        Bounds {
            font,
            transforms: vec![Affine([1.0, 0.0, 0.0, 1.0, 0.0, 0.0])],
            clips: Vec::new(),
            reach: None,
        }
    }

    fn clip(&mut self, r: Option<Rect>) {
        let r = match (self.clips.last().copied().flatten(), r) {
            (Some(a), Some(b)) => Some(a.meet(&b)),
            (a, b) => a.or(b),
        };
        self.clips.push(r);
    }
}

impl ColorPainter for Bounds<'_> {
    fn push_transform(&mut self, t: Transform) {
        let top = *self.transforms.last().expect("the root");
        self.transforms.push(top.then(&Affine::of(&t)));
    }
    fn pop_transform(&mut self) {
        self.transforms.pop();
    }
    fn push_clip_glyph(&mut self, gid: GlyphId) {
        let t = *self.transforms.last().expect("the root");
        let mut pen = Points(&t, Vec::new());
        outline(self.font, gid, &mut pen);
        let r = Rect::around(pen.1).unwrap_or(Rect {
            x0: 0.0,
            y0: 0.0,
            x1: 0.0,
            y1: 0.0,
        });
        self.clip(Some(r));
    }
    fn push_clip_box(&mut self, b: BoundingBox<f32>) {
        let t = *self.transforms.last().expect("the root");
        let r = Rect::around(Rect::of(b).corners().map(|(x, y)| t.apply(x, y)));
        self.clip(r);
    }
    fn pop_clip(&mut self) {
        self.clips.pop();
    }
    fn fill(&mut self, _brush: Brush<'_>) {
        // An unclipped fill covers everything: the glyph's own outline
        // bounds it then (a COLR v0 layer always has one).
        if let Some(r) = self.clips.last().copied().flatten() {
            self.reach = Some(self.reach.map_or(r, |a| a.join(&r)));
        }
    }
    fn push_layer(&mut self, _mode: CompositeMode) {}
}

/// The painter: layers of premultiplied sRGB, clips as coverage masks.
struct Raster<'a> {
    font: &'a FontRef<'a>,
    w: usize,
    h: usize,
    palette: &'a [[f32; 4]],
    foreground: [f32; 3],
    /// A brush read palette entry 0xFFFF.
    used_foreground: std::cell::Cell<bool>,
    transforms: Vec<Affine>,
    clips: Vec<Vec<f32>>,
    layers: Vec<(Vec<[f32; 4]>, CompositeMode)>,
}

impl Raster<'_> {
    fn top(&self) -> Affine {
        *self.transforms.last().expect("the root")
    }

    /// Coverage of the polygon `edges` (pixels, y down), intersected with
    /// the clip around it.
    fn push_mask(&mut self, edges: &[[f64; 4]]) {
        let mut mask = coverage(edges, self.w, self.h);
        if let Some(outer) = self.clips.last() {
            mask.iter_mut().zip(outer).for_each(|(m, o)| *m *= o);
        }
        self.clips.push(mask);
    }

    /// The premultiplied colour of palette entry `index` times `alpha`.
    fn color(&self, index: u16, alpha: f32) -> [f32; 4] {
        // 0xFFFF is the text's colour: the cook paints it twice, black and
        // white, and the renderer mixes the two by the text's colour.
        let c = if index == 0xFFFF {
            self.used_foreground.set(true);
            let [r, g, b] = self.foreground;
            [r, g, b, 1.0]
        } else {
            self.palette
                .get(index as usize)
                .copied()
                .unwrap_or([0.0; 4])
        };
        let a = c[3] * alpha.clamp(0.0, 1.0);
        [c[0] * a, c[1] * a, c[2] * a, a]
    }

    fn stops(&self, stops: &[ColorStop]) -> Vec<([f32; 4], f32)> {
        stops
            .iter()
            .map(|s| (self.color(s.palette_index, s.alpha), s.offset))
            .collect()
    }
}

/// Colour line `s` (premultiplied colours at offsets 0..1) at `t`,
/// stretched over 0..1 by `extend`.
fn line_at(s: &[([f32; 4], f32)], t: f64, extend: Extend) -> [f32; 4] {
    let t = match extend {
        Extend::Repeat => t - t.floor(),
        Extend::Reflect => {
            let m = t.rem_euclid(2.0);
            if m > 1.0 { 2.0 - m } else { m }
        }
        _ => t.clamp(0.0, 1.0),
    } as f32;
    match s.iter().position(|&(_, o)| o > t) {
        None => s.last().map_or([0.0; 4], |l| l.0),
        Some(0) => s[0].0,
        Some(i) => {
            let ((a, oa), (b, ob)) = (s[i - 1], s[i]);
            let k = if ob > oa { (t - oa) / (ob - oa) } else { 0.0 };
            [0, 1, 2, 3].map(|c| a[c] + (b[c] - a[c]) * k)
        }
    }
}

impl ColorPainter for Raster<'_> {
    fn push_transform(&mut self, t: Transform) {
        let top = self.top();
        self.transforms.push(top.then(&Affine::of(&t)));
    }

    fn pop_transform(&mut self) {
        self.transforms.pop();
    }

    fn push_clip_glyph(&mut self, gid: GlyphId) {
        let mut pen = Flatten::new(self.top());
        outline(self.font, gid, &mut pen);
        pen.close();
        self.push_mask(&pen.edges);
    }

    fn push_clip_box(&mut self, b: BoundingBox<f32>) {
        let t = self.top();
        let c = Rect::of(b).corners().map(|(x, y)| t.apply(x, y));
        let edges: Vec<[f64; 4]> = (0..4)
            .map(|i| {
                let (a, b) = (c[i], c[(i + 1) % 4]);
                [a.0, a.1, b.0, b.1]
            })
            .collect();
        self.push_mask(&edges);
    }

    fn pop_clip(&mut self) {
        self.clips.pop();
    }

    fn fill(&mut self, brush: Brush<'_>) {
        // Brush space: pixels back through the paint's transforms.
        let Some(inv) = self.top().inverse() else {
            return;
        };
        let shade: Box<dyn Fn(f64, f64) -> Option<[f32; 4]>> = match brush {
            Brush::Solid {
                palette_index,
                alpha,
            } => {
                let c = self.color(palette_index, alpha);
                Box::new(move |_, _| Some(c))
            }
            Brush::LinearGradient {
                p0,
                p1,
                color_stops,
                extend,
            } => {
                let stops = self.stops(color_stops);
                let (x0, y0) = (f64::from(p0.x), f64::from(p0.y));
                let (dx, dy) = (f64::from(p1.x) - x0, f64::from(p1.y) - y0);
                let len2 = dx * dx + dy * dy;
                if len2 < 1e-12 {
                    return;
                }
                Box::new(move |x, y| {
                    Some(line_at(
                        &stops,
                        ((x - x0) * dx + (y - y0) * dy) / len2,
                        extend,
                    ))
                })
            }
            Brush::RadialGradient {
                c0,
                r0,
                c1,
                r1,
                color_stops,
                extend,
            } => {
                let stops = self.stops(color_stops);
                let (cx, cy, r0) = (f64::from(c0.x), f64::from(c0.y), f64::from(r0));
                let (cdx, cdy) = (f64::from(c1.x) - cx, f64::from(c1.y) - cy);
                let dr = f64::from(r1) - r0;
                let a = cdx * cdx + cdy * cdy - dr * dr;
                Box::new(move |x, y| {
                    // The largest t whose circle passes through the point
                    // with a radius of zero or more (two-point conical).
                    let (px, py) = (x - cx, y - cy);
                    let b = px * cdx + py * cdy + r0 * dr;
                    let c = px * px + py * py - r0 * r0;
                    let ok = |t: f64| r0 + t * dr >= 0.0;
                    let t = if a.abs() < 1e-9 {
                        if b.abs() < 1e-12 {
                            return None;
                        }
                        Some(c / (2.0 * b)).filter(|&t| ok(t))
                    } else {
                        let disc = b * b - a * c;
                        if disc < 0.0 {
                            return None;
                        }
                        let s = disc.sqrt();
                        let (hi, lo) = if a > 0.0 {
                            ((b + s) / a, (b - s) / a)
                        } else {
                            ((b - s) / a, (b + s) / a)
                        };
                        if ok(hi) {
                            Some(hi)
                        } else {
                            Some(lo).filter(|&t| ok(t))
                        }
                    };
                    t.map(|t| line_at(&stops, t, extend))
                })
            }
            Brush::SweepGradient {
                c0,
                start_angle,
                end_angle,
                color_stops,
                extend,
            } => {
                let stops = self.stops(color_stops);
                let (cx, cy) = (f64::from(c0.x), f64::from(c0.y));
                let (start, end) = (f64::from(start_angle), f64::from(end_angle));
                if end <= start && extend != Extend::Pad {
                    return;
                }
                Box::new(move |x, y| {
                    // skrifa's angles run clockwise in font space (y up).
                    let a = (-(y - cy)).atan2(x - cx).to_degrees().rem_euclid(360.0);
                    let t = if end > start {
                        (a - start) / (end - start)
                    } else if a < start {
                        0.0
                    } else {
                        1.0
                    };
                    Some(line_at(&stops, t, extend))
                })
            }
        };
        let clip = self.clips.last();
        let out = &mut self.layers.last_mut().expect("the root").0;
        for y in 0..self.h {
            for x in 0..self.w {
                let i = y * self.w + x;
                let cov = clip.map_or(1.0, |c| c[i]);
                if cov <= 0.0 {
                    continue;
                }
                let (bx, by) = inv.apply(x as f64 + 0.5, y as f64 + 0.5);
                let Some(s) = shade(bx, by) else { continue };
                let s = s.map(|v| v * cov);
                let d = &mut out[i];
                *d = [0, 1, 2, 3].map(|c| s[c] + d[c] * (1.0 - s[3]));
            }
        }
    }

    fn push_layer(&mut self, mode: CompositeMode) {
        self.layers.push((vec![[0.0; 4]; self.w * self.h], mode));
    }

    fn pop_layer(&mut self) {
        if self.layers.len() < 2 {
            return;
        }
        let (src, mode) = self.layers.pop().expect("a pushed layer");
        let clip = self.clips.last();
        let dst = &mut self.layers.last_mut().expect("the root").0;
        for (i, (d, s)) in dst.iter_mut().zip(&src).enumerate() {
            let cov = clip.map_or(1.0, |c| c[i]);
            if cov <= 0.0 {
                continue;
            }
            let c = composite(mode, *s, *d);
            *d = [0, 1, 2, 3].map(|k| d[k] + (c[k] - d[k]) * cov);
        }
    }
}

/// Line segments of an outline in pixels, curves cut finely enough that
/// none strays a twentieth of a pixel from its curve.
struct Flatten {
    t: Affine,
    start: (f64, f64),
    at: (f64, f64),
    edges: Vec<[f64; 4]>,
}

const TOLERANCE: f64 = 0.05;

impl Flatten {
    fn new(t: Affine) -> Flatten {
        Flatten {
            t,
            start: (0.0, 0.0),
            at: (0.0, 0.0),
            edges: Vec::new(),
        }
    }

    fn to(&mut self, p: (f64, f64)) {
        if p != self.at {
            self.edges.push([self.at.0, self.at.1, p.0, p.1]);
        }
        self.at = p;
    }

    fn pieces(dd: f64) -> usize {
        ((dd / TOLERANCE).sqrt().ceil() as usize).clamp(1, 128)
    }
}

impl OutlinePen for Flatten {
    fn move_to(&mut self, x: f32, y: f32) {
        self.close();
        let p = self.t.apply(x.into(), y.into());
        self.start = p;
        self.at = p;
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.t.apply(x.into(), y.into());
        self.to(p);
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        let p0 = self.at;
        let p1 = self.t.apply(cx0.into(), cy0.into());
        let p2 = self.t.apply(x.into(), y.into());
        let dd = ((p0.0 - 2.0 * p1.0 + p2.0).powi(2) + (p0.1 - 2.0 * p1.1 + p2.1).powi(2)).sqrt();
        let n = Flatten::pieces(dd / 8.0);
        for i in 1..=n {
            let t = i as f64 / n as f64;
            let u = 1.0 - t;
            self.to((
                u * u * p0.0 + 2.0 * u * t * p1.0 + t * t * p2.0,
                u * u * p0.1 + 2.0 * u * t * p1.1 + t * t * p2.1,
            ));
        }
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let p0 = self.at;
        let p1 = self.t.apply(cx0.into(), cy0.into());
        let p2 = self.t.apply(cx1.into(), cy1.into());
        let p3 = self.t.apply(x.into(), y.into());
        let d = |a: (f64, f64), b: (f64, f64), c: (f64, f64)| {
            ((a.0 - 2.0 * b.0 + c.0).powi(2) + (a.1 - 2.0 * b.1 + c.1).powi(2)).sqrt()
        };
        let n = Flatten::pieces(0.75 * d(p0, p1, p2).max(d(p1, p2, p3)));
        for i in 1..=n {
            let t = i as f64 / n as f64;
            let u = 1.0 - t;
            let (a, b, c, e) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            self.to((
                a * p0.0 + b * p1.0 + c * p2.0 + e * p3.0,
                a * p0.1 + b * p1.1 + c * p2.1 + e * p3.1,
            ));
        }
    }

    fn close(&mut self) {
        let s = self.start;
        self.to(s);
    }
}

/// The non-zero coverage of closed polygon `edges` (pixels, y down) on a
/// `w`×`h` grid: each edge adds its signed area to the cells it crosses
/// and a row's running sum is the coverage (font-rs `accumulate`).
fn coverage(edges: &[[f64; 4]], w: usize, h: usize) -> Vec<f32> {
    let stride = w + 2;
    let mut acc = vec![0.0f64; stride * h + 2];
    let xmax = w as f64;
    for &[ax, ay, bx, by] in edges {
        if ay == by {
            continue;
        }
        let (dir, (x0, y0), (x1, y1)) = if ay < by {
            (1.0, (ax, ay), (bx, by))
        } else {
            (-1.0, (bx, by), (ax, ay))
        };
        let dxdy = (x1 - x0) / (y1 - y0);
        let ystart = y0.max(0.0);
        let yend = y1.min(h as f64);
        if ystart >= yend {
            continue;
        }
        let mut x = x0 + (ystart - y0) * dxdy;
        let mut yi = ystart.floor() as usize;
        while (yi as f64) < yend {
            let top = (yi as f64).max(ystart);
            let bot = ((yi + 1) as f64).min(yend);
            let dy = bot - top;
            let xnext = x + dxdy * dy;
            let d = dy * dir;
            let row = yi * stride;
            // x past either side counts at the edge column: what is left
            // of the grid still winds the pixels to its right.
            let (lo, hi) = if x < xnext { (x, xnext) } else { (xnext, x) };
            let (lo, hi) = (lo.clamp(0.0, xmax), hi.clamp(0.0, xmax));
            let lo_floor = lo.floor();
            let lo_i = lo_floor as usize;
            let hi_ceil = hi.ceil();
            let hi_i = hi_ceil as usize;
            if hi_i <= lo_i + 1 {
                let mid = 0.5 * (lo + hi) - lo_floor;
                acc[row + lo_i] += d - d * mid;
                acc[row + lo_i + 1] += d * mid;
            } else {
                let s = 1.0 / (hi - lo);
                let lo_f = lo - lo_floor;
                let a0 = 0.5 * s * (1.0 - lo_f).powi(2);
                let hi_f = hi - hi_ceil + 1.0;
                let am = 0.5 * s * hi_f * hi_f;
                acc[row + lo_i] += d * a0;
                if hi_i == lo_i + 2 {
                    acc[row + lo_i + 1] += d * (1.0 - a0 - am);
                } else {
                    let a1 = s * (1.5 - lo_f);
                    acc[row + lo_i + 1] += d * (a1 - a0);
                    for xi in lo_i + 2..hi_i - 1 {
                        acc[row + xi] += d * s;
                    }
                    let a2 = a1 + (hi_i - lo_i - 3) as f64 * s;
                    acc[row + hi_i - 1] += d * (1.0 - a2 - am);
                }
                acc[row + hi_i] += d * am;
            }
            x = xnext;
            yi += 1;
        }
    }
    let mut out = vec![0.0f32; w * h];
    for y in 0..h {
        let mut sum = 0.0;
        for x in 0..w {
            sum += acc[y * stride + x];
            out[y * w + x] = sum.abs().min(1.0) as f32;
        }
    }
    out
}

/// The text colours a glyph in the text's colour is painted in once more,
/// to check the renderer's mix of its black and white paints
/// (`foreground_mix`) against them: two greys (a piecewise blend such as
/// overlay or hard-light breaks at 0.5 and is off the mix either side of
/// it) and a colour with channels apart (the hue, saturation, colour and
/// luminosity modes mix the channels).
pub const MIX_PROBES: [[f32; 3]; 3] = [[0.5; 3], [0.25; 3], [0.2, 0.6, 0.9]];

/// How far `probe`, a glyph painted with foreground `fg` (sRGB), is from
/// what the renderer draws for it at `fg`: its `black` and `white` paints
/// mixed channel by channel by `fg` in sRGB, at the black one's alpha. In
/// 8-bit levels, the largest over the pixels, each colour difference
/// weighted by the pixel's coverage; 0 or 1 (rounding) wherever the paint
/// is affine in the foreground: fills, gradients, Porter-Duff composites,
/// and a blend that uses the foreground on one side only and linearly.
pub fn mix_error(black: &[u8], white: &[u8], probe: &[u8], fg: [f32; 3]) -> u8 {
    let mut worst = 0.0f32;
    for ((b, w), p) in black
        .as_chunks::<4>()
        .0
        .iter()
        .zip(white.as_chunks::<4>().0)
        .zip(probe.as_chunks::<4>().0)
    {
        let mut e = (f32::from(p[3]) - f32::from(b[3])).abs();
        let cover = f32::from(p[3].max(b[3])) / 255.0;
        for k in 0..3 {
            let mix = f32::from(b[k]) + (f32::from(w[k]) - f32::from(b[k])) * fg[k];
            e = e.max((f32::from(p[k]) - mix).abs() * cover);
        }
        worst = worst.max(e);
    }
    worst.round().min(255.0) as u8
}

/// Source `s` composited onto backdrop `b`, both premultiplied.
fn composite(mode: CompositeMode, s: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    use CompositeMode as M;
    let (sa, ba) = (s[3], b[3]);
    let porter = |fs: f32, fb: f32| [0, 1, 2, 3].map(|k| s[k] * fs + b[k] * fb);
    let separable = |f: fn(f32, f32) -> f32| {
        let un = |c: [f32; 4], k: usize| if c[3] > 0.0 { c[k] / c[3] } else { 0.0 };
        let mut o = [0.0; 4];
        for k in 0..3 {
            o[k] = s[k] * (1.0 - ba) + b[k] * (1.0 - sa) + sa * ba * f(un(b, k), un(s, k));
        }
        o[3] = sa + ba - sa * ba;
        o
    };
    let hsl = |f: fn([f32; 3], [f32; 3]) -> [f32; 3]| {
        let un = |c: [f32; 4]| {
            if c[3] > 0.0 {
                [c[0] / c[3], c[1] / c[3], c[2] / c[3]]
            } else {
                [0.0; 3]
            }
        };
        let m = f(un(b), un(s));
        let mut o = [0.0; 4];
        for k in 0..3 {
            o[k] = s[k] * (1.0 - ba) + b[k] * (1.0 - sa) + sa * ba * m[k];
        }
        o[3] = sa + ba - sa * ba;
        o
    };
    match mode {
        M::Clear => [0.0; 4],
        M::Src => s,
        M::Dest => b,
        M::DestOver => porter(1.0 - ba, 1.0),
        M::SrcIn => porter(ba, 0.0),
        M::DestIn => porter(0.0, sa),
        M::SrcOut => porter(1.0 - ba, 0.0),
        M::DestOut => porter(0.0, 1.0 - sa),
        M::SrcAtop => porter(ba, 1.0 - sa),
        M::DestAtop => porter(1.0 - ba, sa),
        M::Xor => porter(1.0 - ba, 1.0 - sa),
        M::Plus => porter(1.0, 1.0).map(|v| v.min(1.0)),
        M::Screen => separable(|b, s| b + s - b * s),
        M::Overlay => separable(|b, s| hard_light(s, b)),
        M::Darken => separable(f32::min),
        M::Lighten => separable(f32::max),
        M::ColorDodge => separable(|b, s| {
            if b <= 0.0 {
                0.0
            } else if s >= 1.0 {
                1.0
            } else {
                (b / (1.0 - s)).min(1.0)
            }
        }),
        M::ColorBurn => separable(|b, s| {
            if b >= 1.0 {
                1.0
            } else if s <= 0.0 {
                0.0
            } else {
                1.0 - ((1.0 - b) / s).min(1.0)
            }
        }),
        M::HardLight => separable(hard_light),
        M::SoftLight => separable(|b, s| {
            if s <= 0.5 {
                b - (1.0 - 2.0 * s) * b * (1.0 - b)
            } else {
                let d = if b <= 0.25 {
                    ((16.0 * b - 12.0) * b + 4.0) * b
                } else {
                    b.sqrt()
                };
                b + (2.0 * s - 1.0) * (d - b)
            }
        }),
        M::Difference => separable(|b, s| (b - s).abs()),
        M::Exclusion => separable(|b, s| b + s - 2.0 * b * s),
        M::Multiply => separable(|b, s| b * s),
        M::HslHue => hsl(|b, s| set_lum(set_sat(s, sat(b)), lum(b))),
        M::HslSaturation => hsl(|b, s| set_lum(set_sat(b, sat(s)), lum(b))),
        M::HslColor => hsl(|b, s| set_lum(s, lum(b))),
        M::HslLuminosity => hsl(|b, s| set_lum(b, lum(s))),
        _ => porter(1.0, 1.0 - sa),
    }
}

/// HardLight(b, s) of the W3C formulas (Overlay is it with the two swapped).
fn hard_light(b: f32, s: f32) -> f32 {
    if s <= 0.5 {
        b * 2.0 * s
    } else {
        let s2 = 2.0 * s - 1.0;
        b + s2 - b * s2
    }
}

fn lum(c: [f32; 3]) -> f32 {
    0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2]
}

fn clip_color(c: [f32; 3]) -> [f32; 3] {
    let l = lum(c);
    let n = c[0].min(c[1]).min(c[2]);
    let x = c[0].max(c[1]).max(c[2]);
    let mut c = c;
    if n < 0.0 {
        c = c.map(|v| l + (v - l) * l / (l - n).max(1e-6));
    }
    if x > 1.0 {
        c = c.map(|v| l + (v - l) * (1.0 - l) / (x - l).max(1e-6));
    }
    c
}

fn set_lum(c: [f32; 3], l: f32) -> [f32; 3] {
    let d = l - lum(c);
    clip_color(c.map(|v| v + d))
}

fn sat(c: [f32; 3]) -> f32 {
    c[0].max(c[1]).max(c[2]) - c[0].min(c[1]).min(c[2])
}

fn set_sat(c: [f32; 3], s: f32) -> [f32; 3] {
    let (mx, mn) = (c[0].max(c[1]).max(c[2]), c[0].min(c[1]).min(c[2]));
    if mx <= mn {
        return [0.0; 3];
    }
    c.map(|v| (v - mn) * s / (mx - mn))
}

/// The distance range of a colour page's silhouette field, in its pixels:
/// twice the transparent border round each image, so the field reaches
/// its border and never a neighbour's ink.
pub const FIELD_RANGE: f32 = 2.0 * PAD as f32;

/// A colour page's silhouette as a distance field for its shadows, glow
/// and outline (og-render's MSDF passes): every channel the signed
/// distance to the alpha's half-coverage edge, `0.5 + d / range` with `d`
/// in pixels, positive inside. The distance is the exact Euclidean one to
/// the nearest pixel across the edge (Felzenszwalb and Huttenlocher's
/// transform), refined on the edge's own pixels by their coverage.
pub fn silhouette_field(rgba: &[u8], w: usize, h: usize, range: f32) -> Vec<u8> {
    let alpha: Vec<f32> = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| f32::from(p[3]) / 255.0)
        .collect();
    let inside: Vec<bool> = alpha.iter().map(|&a| a >= 0.5).collect();
    // Squared distance to the nearest pixel of the other side.
    let to = |side: bool| {
        let mut d: Vec<f32> = inside
            .iter()
            .map(|&i| if i == side { 0.0 } else { f32::INFINITY })
            .collect();
        edt(&mut d, w, h);
        d
    };
    let (to_in, to_out) = (to(true), to(false));
    let mut out = Vec::with_capacity(w * h * 4);
    for k in 0..w * h {
        let a = alpha[k];
        let d = if a > 0.0 && a < 1.0 && (to_in[k] <= 1.0 && to_out[k] <= 1.0) {
            // An edge pixel: its coverage places the edge within it.
            a - 0.5
        } else if inside[k] {
            to_out[k].sqrt() - 0.5
        } else {
            0.5 - to_in[k].sqrt()
        };
        let v = ((0.5 + d / range).clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        out.extend_from_slice(&[v; 4]);
    }
    out
}

/// The squared Euclidean distance transform of `d` (0 on the feature, ∞
/// elsewhere) in place: rows then columns of the 1D transform.
fn edt(d: &mut [f32], w: usize, h: usize) {
    let n = w.max(h);
    let (mut f, mut out) = (vec![0.0f32; n], vec![0.0f32; n]);
    let (mut v, mut z) = (vec![0usize; n], vec![0.0f32; n + 1]);
    for y in 0..h {
        f[..w].copy_from_slice(&d[y * w..][..w]);
        edt_1d(&f[..w], &mut out[..w], &mut v, &mut z);
        d[y * w..][..w].copy_from_slice(&out[..w]);
    }
    for x in 0..w {
        for y in 0..h {
            f[y] = d[y * w + x];
        }
        edt_1d(&f[..h], &mut out[..h], &mut v, &mut z);
        for y in 0..h {
            d[y * w + x] = out[y];
        }
    }
}

/// One row of [`edt`]: the lower envelope of the parabolas rooted at each
/// finite sample.
fn edt_1d(f: &[f32], d: &mut [f32], v: &mut [usize], z: &mut [f32]) {
    let n = f.len();
    let Some(first) = (0..n).find(|&q| f[q].is_finite()) else {
        d.fill(f32::INFINITY);
        return;
    };
    let mut k = 0;
    v[0] = first;
    z[0] = f32::NEG_INFINITY;
    z[1] = f32::INFINITY;
    let sq = |q: usize| (q * q) as f32;
    for q in first + 1..n {
        if !f[q].is_finite() {
            continue;
        }
        loop {
            let p = v[k];
            let s = ((f[q] + sq(q)) - (f[p] + sq(p))) / (2.0 * (q as f32 - p as f32));
            if s <= z[k] && k > 0 {
                k -= 1;
            } else {
                k += 1;
                v[k] = q;
                z[k] = s;
                z[k + 1] = f32::INFINITY;
                break;
            }
        }
    }
    let mut k = 0;
    for (q, out) in d.iter_mut().enumerate() {
        while z[k + 1] < q as f32 {
            k += 1;
        }
        let p = v[k];
        let dq = q as f32 - p as f32;
        *out = dq * dq + f[p];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The renderer's black and white mix is exact for a glyph whose paint
    /// is affine in the foreground and off for one routed through a
    /// piecewise, self-multiplying, clamping or channel-mixing blend;
    /// `mix_error` tells them apart. Each paint is one pixel per fixed
    /// colour: the foreground composited onto it (`Source`), it onto the
    /// foreground (`Backdrop`), or the foreground onto itself (`Both`).
    #[test]
    fn the_foreground_mix_is_checked_where_a_blend_bends_it() {
        use CompositeMode as M;
        #[derive(Clone, Copy, Debug)]
        enum Side {
            Source,
            Backdrop,
            Both,
        }
        let fixed = [
            [0.9, 0.2, 0.1, 1.0],
            [0.3, 0.3, 0.3, 1.0],
            [0.1, 0.5, 0.8, 0.5],
        ];
        let pixels = |mode: M, side: Side, f: [f32; 3]| -> Vec<u8> {
            let fg = [f[0], f[1], f[2], 1.0];
            let mut out = Vec::new();
            for c in fixed {
                let c = [c[0] * c[3], c[1] * c[3], c[2] * c[3], c[3]];
                let (s, b) = match side {
                    Side::Source => (fg, c),
                    Side::Backdrop => (c, fg),
                    Side::Both => (fg, fg),
                };
                let o = composite(mode, s, b);
                let a = o[3].max(1e-6);
                out.extend([o[0] / a, o[1] / a, o[2] / a, o[3]].map(|v| (v * 255.0).round() as u8));
            }
            out
        };
        let error = |mode: M, side: Side, probes: &[[f32; 3]]| {
            let (black, white) = (pixels(mode, side, [0.0; 3]), pixels(mode, side, [1.0; 3]));
            probes
                .iter()
                .map(|&f| mix_error(&black, &white, &pixels(mode, side, f), f))
                .max()
                .unwrap()
        };
        // Mid grey: the label colour the mix is checked at first.
        let grey = [[0.5f32; 3]];
        let affine = [
            (M::SrcOver, Side::Source),
            (M::SrcOver, Side::Backdrop),
            (M::DestOver, Side::Source),
            (M::SrcAtop, Side::Source),
            (M::Xor, Side::Backdrop),
            (M::SrcOver, Side::Both),
            (M::Multiply, Side::Source),
            (M::Multiply, Side::Backdrop),
            (M::Screen, Side::Source),
            (M::Exclusion, Side::Source),
            // Overlay branches on the backdrop, hard-light on the source:
            // each is linear in the other operand.
            (M::Overlay, Side::Source),
            (M::HardLight, Side::Backdrop),
        ];
        for (mode, side) in affine {
            let e = error(mode, side, &MIX_PROBES);
            assert!(e <= 1, "{mode:?}, foreground as {side:?}: {e}");
        }
        let bent = [
            (M::Multiply, Side::Both),
            (M::Screen, Side::Both),
            (M::Overlay, Side::Backdrop),
            (M::HardLight, Side::Source),
            (M::SoftLight, Side::Backdrop),
            (M::Darken, Side::Source),
            (M::Lighten, Side::Source),
            (M::ColorDodge, Side::Backdrop),
            (M::ColorBurn, Side::Backdrop),
            (M::Difference, Side::Source),
            (M::Plus, Side::Source),
        ];
        for (mode, side) in bent {
            let e = error(mode, side, &grey);
            assert!(e >= 10, "{mode:?}, foreground as {side:?}, mid grey: {e}");
        }
        // A grey has no hue or saturation, so on greys these may stay put;
        // the coloured probe moves them.
        for mode in [M::HslHue, M::HslSaturation, M::HslColor, M::HslLuminosity] {
            let e = error(mode, Side::Source, &MIX_PROBES).max(error(
                mode,
                Side::Backdrop,
                &MIX_PROBES,
            ));
            assert!(e >= 10, "{mode:?}: {e}");
        }
    }

    /// The silhouette field of a 6x6 opaque square on a 16x16 page: 0.5 at
    /// its edge, rising a range's fraction per pixel inside and falling
    /// outside, the same along a row and a column and round a corner by
    /// the Euclidean distance.
    #[test]
    fn a_silhouette_field_is_the_distance_to_its_edge() {
        let (w, h) = (16, 16);
        let mut rgba = vec![0u8; w * h * 4];
        for y in 5..11 {
            for x in 5..11 {
                rgba[(y * w + x) * 4 + 3] = 255;
            }
        }
        let f = silhouette_field(&rgba, w, h, 8.0);
        let d = |x: usize, y: usize| (f32::from(f[(y * w + x) * 4]) / 255.0 - 0.5) * 8.0;
        // Pixel centres: the edge is at x = 5, so pixel 5 is 0.5 in, 7 is
        // 2.5 in, 3 is 1.5 out, 1 is 3.5 out.
        for (x, want) in [(5, 0.5), (7, 2.5), (3, -1.5), (1, -3.5)] {
            assert!(
                (d(x, 8) - want).abs() < 0.03,
                "x {x}: {} for {want}",
                d(x, 8)
            );
            assert!(
                (d(8, x) - want).abs() < 0.03,
                "y {x}: {} for {want}",
                d(8, x)
            );
        }
        // Off the corner (5, 5) by 2 and 2 pixel centres: √8 − ½ out.
        let corner = -(8.0f32.sqrt() - 0.5);
        assert!((d(3, 3) - corner).abs() < 0.03, "{} for {corner}", d(3, 3));
        // Past the range the field holds at 0.
        assert_eq!(f[0], 0);
    }

    #[test]
    fn a_square_covers_its_pixels_and_half_the_ones_its_edge_halves() {
        // 1.5..3.5 by 1..3 on a 5x4 grid.
        let sq = [
            [1.5, 1.0, 3.5, 1.0],
            [3.5, 1.0, 3.5, 3.0],
            [3.5, 3.0, 1.5, 3.0],
            [1.5, 3.0, 1.5, 1.0],
        ];
        let c = coverage(&sq, 5, 4);
        let row: Vec<f32> = c[5..10].to_vec();
        assert_eq!(row, vec![0.0, 0.5, 1.0, 0.5, 0.0]);
        assert_eq!(c[..5], [0.0; 5]);
        // Wound the other way, the same coverage.
        let rev: Vec<[f64; 4]> = sq.iter().rev().map(|e| [e[2], e[3], e[0], e[1]]).collect();
        assert_eq!(coverage(&rev, 5, 4), c);
    }

    #[test]
    fn separable_and_porter_duff_modes_follow_their_formulas() {
        let red = [0.5, 0.0, 0.0, 0.5];
        let blue = [0.0, 0.0, 1.0, 1.0];
        assert_eq!(composite(CompositeMode::SrcIn, red, [0.0; 4]), [0.0; 4]);
        assert_eq!(
            composite(CompositeMode::DestOut, red, blue),
            [0.0, 0.0, 0.5, 0.5]
        );
        // Multiply of opaque colours is the product.
        let m = composite(
            CompositeMode::Multiply,
            [0.5, 1.0, 0.25, 1.0],
            [0.5, 0.5, 1.0, 1.0],
        );
        assert_eq!(m, [0.25, 0.5, 0.25, 1.0]);
        // Screen of half grey on itself.
        let s = composite(
            CompositeMode::Screen,
            [0.5, 0.5, 0.5, 1.0],
            [0.5, 0.5, 0.5, 1.0],
        );
        assert_eq!(s, [0.75, 0.75, 0.75, 1.0]);
    }
}
