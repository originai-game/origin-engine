//! A laid-out run's glyphs put into the glyph atlas (`og_draw::glyphs`):
//! each one rastered the first time it is drawn at a size, a subpixel bin
//! and a form (its fill, its outline's stroke or its blurred shadow) and
//! found there afterwards. A colour glyph is painted into the colour tier,
//! every other into the coverage tier.
//!
//! This module only names a raster ([`og_draw::glyphs::GlyphKey`]) and
//! calls [`crate::raster`] and [`crate::colr`] for its pixels; a glyph the
//! atlas holds costs one lookup and no allocation.

use crate::fonts::{Collection, FaceId};
use crate::layout::{GlyphRun, Style, Text};
use crate::raster::{Glyph, Placement, Rasterizer};
use og_draw::glyphs::{Atlas, GlyphKey, GlyphSlot, GlyphTier};
use skrifa::instance::NormalizedCoord;
use skrifa::outline::HintingInstance;
use skrifa::{FontRef, GlyphId, MetadataProvider};
use std::collections::BTreeMap;

/// What of a glyph is drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Form {
    /// Its outline, filled.
    Fill,
    /// The band this many device pixels wide centred on its outline
    /// (`-webkit-text-stroke`).
    Stroke(f32),
    /// Its fill blurred by this CSS blur radius in device pixels (a
    /// shadow or a glow).
    Shadow(f32),
}

/// One face at one size and location, as a laid-out run draws it: what
/// its glyphs' keys share.
#[derive(Clone, Copy, Debug)]
pub struct Strike {
    key: GlyphKey,
    face: FaceId,
    /// Device pixels per em, as keyed (whole 1/64).
    size: f32,
    /// The tangent of a synthetic slant, as keyed.
    skew: f32,
    /// A synthetic bold's stroke, device pixels.
    embolden: f32,
}

impl Strike {
    /// Device pixels per em, as its glyphs are keyed.
    pub fn size(&self) -> f32 {
        self.size
    }

    /// A synthetic bold's stroke width, device pixels; 0 none.
    pub fn embolden(&self) -> f32 {
        self.embolden
    }
}

/// The hinting instances kept: beyond this many the oldest sizes go.
const HINTERS: usize = 96;

/// Rasters glyphs into the atlas, keeping the rasterizer's scratch memory
/// and a hinting instance per face, size and location.
#[derive(Default)]
pub struct Strikes {
    raster: Rasterizer,
    /// By face, index, size (1/64 px) and location.
    hinters: BTreeMap<(u32, u32, u32, u32), Option<HintingInstance>>,
    coords: Vec<NormalizedCoord>,
    palette: Vec<[f32; 4]>,
    mask: Vec<u8>,
    other: Vec<u8>,
    blurred: Vec<u8>,
    scratch: Vec<f32>,
}

impl Strikes {
    pub fn new() -> Self {
        Self::default()
    }

    /// `run`'s strike at `scale` device pixels a CSS px, hinted (text
    /// drawn axis-aligned) or not.
    pub fn strike(
        &mut self,
        atlas: &mut Atlas,
        run: &GlyphRun,
        scale: f32,
        hinted: bool,
    ) -> Strike {
        // A size is whole 1/64 px, so a scale animating by less never
        // makes a raster of its own.
        let size = ((run.size * scale * 64.0).round().max(1.0)) as u32;
        let px = size as f32 / 64.0;
        // Skia's fake bold: the outline stroked by the size over 24 at 9 px
        // and under, over 32 at 36 px and over, between them in between.
        let embolden = if run.embolden {
            let k = ((px - 9.0) / 27.0).clamp(0.0, 1.0);
            px * (1.0 / 24.0 + k * (1.0 / 32.0 - 1.0 / 24.0))
        } else {
            0.0
        };
        let skew = run.skew.map_or(0, |deg| {
            (libm::tanf(deg.to_radians()) * 256.0).round() as i16
        });
        let mut key = GlyphKey::new(GlyphTier::Coverage, run.face.0, run.index, 0, size);
        key.location = atlas.location(run.coords);
        key.hinted = hinted;
        key.embolden = (embolden * 64.0).round().min(f32::from(u16::MAX)) as u16;
        key.skew = skew;
        Strike {
            key,
            face: run.face,
            size: px,
            skew: f32::from(skew) / 256.0,
            embolden: f32::from(key.embolden) / 64.0,
        }
    }

    /// Where glyph `id` of `strike` is in the atlas, drawn as `form` with
    /// its pen in quarter `bin` of a pixel, and its tier: rastered from
    /// `fonts` and `coords` (the run's location) when the atlas lacks it.
    /// `ink` is the text colour a colour glyph's foreground entry takes
    /// (sRGB). `None` when the face has no such glyph or the atlas no
    /// room for it this frame.
    #[allow(clippy::too_many_arguments)]
    pub fn glyph(
        &mut self,
        fonts: &Collection,
        atlas: &mut Atlas,
        strike: &Strike,
        coords: &[i16],
        id: u32,
        bin: u8,
        form: Form,
        ink: [f32; 3],
    ) -> Option<(GlyphTier, GlyphSlot)> {
        let q = |px: f32| (px * 64.0).round().clamp(1.0, f32::from(u16::MAX)) as u16;
        let mut key = GlyphKey {
            glyph: id,
            bin,
            ..strike.key
        };
        match form {
            Form::Fill => {}
            Form::Stroke(w) => key.stroke = q(w),
            Form::Shadow(r) => key.blur = q(r),
        }
        if let Some(slot) = atlas.get(&key) {
            return Some((GlyphTier::Coverage, slot));
        }
        // A colour glyph's fill is its own colours; its stroke and shadow
        // are its outline's, when it has one.
        let mut color = GlyphKey {
            tier: GlyphTier::Color,
            hinted: false,
            embolden: 0,
            ink: [
                (ink[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                (ink[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                (ink[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                255,
            ],
            ..key
        };
        color.skew = 0;
        if form == Form::Fill
            && let Some(slot) = atlas.get(&color)
        {
            return Some((GlyphTier::Color, slot));
        }
        let font = FontRef::from_index(fonts.file(strike.face), strike.key.index).ok()?;
        self.coords.clear();
        self.coords
            .extend(coords.iter().map(|c| NormalizedCoord::from_bits(*c)));
        let x_offset = f32::from(bin) * 0.25;
        if form == Form::Fill && font.color_glyphs().get(GlyphId::new(id)).is_some() {
            self.palette.clear();
            if let Some(p) = font.color_palettes().get(0) {
                self.palette.extend(
                    p.colors()
                        .iter()
                        .map(|c| [c.red, c.green, c.blue, c.alpha].map(|v| f32::from(v) / 255.0)),
                );
            }
            let at = crate::colr::paint(
                crate::colr::ColorGlyph {
                    font: &font,
                    id: GlyphId::new(id),
                    size: strike.size,
                    coords: &self.coords,
                    x_offset,
                    palette: &self.palette,
                    foreground: [ink[0], ink[1], ink[2], 1.0],
                },
                &mut self.mask,
            )?;
            let slot = atlas.insert(color, (at.left, at.top), (at.width, at.height), &self.mask)?;
            return Some((GlyphTier::Color, slot));
        }
        let hinting = if strike.key.hinted {
            let k = (
                strike.face.0,
                strike.key.index,
                strike.key.size,
                strike.key.location,
            );
            if !self.hinters.contains_key(&k) {
                if self.hinters.len() >= HINTERS {
                    self.hinters.clear();
                }
                let h = crate::raster::hinter(&font, strike.size, &self.coords);
                self.hinters.insert(k, h);
            }
            self.hinters.get(&k).and_then(Option::as_ref)
        } else {
            None
        };
        let glyph = Glyph {
            font: &font,
            id: GlyphId::new(id),
            size: strike.size,
            coords: &self.coords,
            hinting,
            x_offset,
            skew: strike.skew,
        };
        let at = match form {
            Form::Stroke(w) => self.raster.raster_stroke(glyph, w, &mut self.mask)?,
            Form::Fill | Form::Shadow(_) => {
                let mut at = self.raster.raster(glyph, &mut self.mask)?;
                if strike.embolden > 0.0
                    && let Some(band) =
                        self.raster
                            .raster_stroke(glyph, strike.embolden, &mut self.other)
                {
                    at = union(at, &mut self.mask, band, &self.other, &mut self.blurred);
                }
                at
            }
        };
        let at = match form {
            Form::Shadow(radius) if at.width > 0 && at.height > 0 => {
                let b = crate::raster::blur(
                    &self.mask,
                    at.width,
                    at.height,
                    radius,
                    &mut self.scratch,
                    &mut self.blurred,
                );
                std::mem::swap(&mut self.mask, &mut self.blurred);
                Placement {
                    left: at.left - b.pad as i32,
                    top: at.top + b.pad as i32,
                    width: b.width,
                    height: b.height,
                }
            }
            _ => at,
        };
        let slot = atlas.insert(key, (at.left, at.top), (at.width, at.height), &self.mask)?;
        Some((GlyphTier::Coverage, slot))
    }

    /// Where glyph `id` of `strike`'s face is in the atlas's field tier:
    /// one distance field for every size and transform
    /// ([`crate::sdf::field`]: [`crate::sdf::EM_PX`] field pixels an em),
    /// made when the atlas lacks it. Synthetic bold and slant are the
    /// painter's (a grown threshold, a transform).
    pub fn field(
        &mut self,
        fonts: &Collection,
        atlas: &mut Atlas,
        strike: &Strike,
        coords: &[i16],
        id: u32,
    ) -> Option<GlyphSlot> {
        let mut key = GlyphKey::new(GlyphTier::Field, strike.face.0, strike.key.index, id, 0);
        key.location = strike.key.location;
        if let Some(slot) = atlas.get(&key) {
            return Some(slot);
        }
        let font = FontRef::from_index(fonts.file(strike.face), strike.key.index).ok()?;
        self.coords.clear();
        self.coords
            .extend(coords.iter().map(|c| NormalizedCoord::from_bits(*c)));
        let f = crate::sdf::field(&font, GlyphId::new(id), &self.coords, &mut self.mask)?;
        atlas.insert(key, (f.left, f.top), (f.width, f.height), &self.mask)
    }
}

/// Lays `text` out in `style` (its lines broken only at `\n`, aligned in
/// the widest by `style.align`) and puts each glyph into `atlas` as world
/// and overlay text draws it ([`og_draw::Text`]): an outline glyph's
/// distance field, a colour glyph painted at `style.size` device pixels an
/// em in `ink` (sRGB). `out` is replaced with the glyphs in em from the
/// first line's baseline origin, y up, the whole block moved left by
/// `shift` of its width (0 start, 0.5 centre, 1 end). False when a glyph
/// found no room in the atlas this frame (drawn next frame).
#[allow(clippy::too_many_arguments)]
pub fn text_glyphs(
    strikes: &mut Strikes,
    fonts: &mut Collection,
    atlas: &mut Atlas,
    layout: &mut Text,
    text: &str,
    style: &Style,
    shift: f32,
    ink: [f32; 3],
    out: &mut Vec<og_draw::TextGlyph>,
) -> bool {
    out.clear();
    layout.shape(fonts, text, style);
    layout.break_at(fonts, text, style, None);
    let em = style.size.max(1e-3);
    let first = layout.first_baseline().unwrap_or(0.0);
    let dx = -shift * layout.width();
    let mut whole = true;
    let fonts = &*fonts;
    layout.runs(fonts, |run| {
        let font = FontRef::from_index(fonts.file(run.face), run.index).ok();
        let strike = strikes.strike(atlas, &run, 1.0, false);
        for g in run.glyphs.iter().filter(|g| g.id != 0) {
            // The pen in em, y up from the first baseline.
            let (x, y) = ((g.x + dx) / em, (first - g.y) / em);
            let colour = font
                .as_ref()
                .is_some_and(|f| f.color_glyphs().get(GlyphId::new(g.id)).is_some());
            let found = if colour {
                strikes
                    .glyph(fonts, atlas, &strike, run.coords, g.id, 0, Form::Fill, ink)
                    .map(|(tier, s)| (tier, s, run.size))
            } else {
                strikes
                    .field(fonts, atlas, &strike, run.coords, g.id)
                    .map(|s| (GlyphTier::Field, s, crate::sdf::EM_PX))
            };
            let Some((tier, s, px)) = found else {
                whole = false;
                continue;
            };
            if s.width == 0 || s.height == 0 {
                continue;
            }
            // Its pixels' em at the size they were made: the field's own,
            // or the run's for a colour glyph (laid out at `em`).
            let k = px * em / run.size.max(1e-3);
            let (left, top) = (f32::from(s.left) / k, f32::from(s.top) / k);
            let (w, h) = (f32::from(s.width) / k, f32::from(s.height) / k);
            out.push(og_draw::TextGlyph {
                rect: [x + left, y + top - h, x + left + w, y + top],
                tier,
                page: s.page,
                uv: [s.x, s.y, s.x + s.width, s.y + s.height],
            });
        }
    });
    whole
}

/// The mask that covers what either of two masks does (`a`, placed `at`,
/// and `b`, placed `bt`), their coverage summed and clamped: a fill and the
/// stroke of its own outline. Left in `a`; `scratch` holds it meanwhile.
fn union(
    at: Placement,
    a: &mut Vec<u8>,
    bt: Placement,
    b: &[u8],
    scratch: &mut Vec<u8>,
) -> Placement {
    if bt.width == 0 || bt.height == 0 {
        return at;
    }
    if at.width == 0 || at.height == 0 {
        a.clear();
        a.extend_from_slice(b);
        return bt;
    }
    let left = at.left.min(bt.left);
    let top = at.top.max(bt.top);
    let right = (at.left + at.width as i32).max(bt.left + bt.width as i32);
    let bottom = (at.top - at.height as i32).min(bt.top - bt.height as i32);
    let (w, h) = ((right - left) as usize, (top - bottom) as usize);
    scratch.clear();
    scratch.resize(w * h, 0);
    for (p, mask) in [(at, &a[..]), (bt, b)] {
        let (dx, dy) = ((p.left - left) as usize, (top - p.top) as usize);
        for row in 0..p.height as usize {
            let src = &mask[row * p.width as usize..(row + 1) * p.width as usize];
            let dst = &mut scratch[(dy + row) * w + dx..(dy + row) * w + dx + src.len()];
            for (d, s) in dst.iter_mut().zip(src) {
                *d = d.saturating_add(*s);
            }
        }
    }
    std::mem::swap(a, scratch);
    Placement {
        left,
        top,
        width: w as u32,
        height: h as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn file(path: &str) -> Arc<Vec<u8>> {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../templates/");
        Arc::new(std::fs::read(format!("{root}{path}")).unwrap())
    }

    /// A glyph is rastered once and found afterwards; its stroke and its
    /// shadow are rasters of their own, each larger than the fill by what
    /// it adds; and a weight the face lacks draws more ink than the face's
    /// own.
    #[test]
    fn a_glyph_rasters_once_per_form() {
        let mut fonts = Collection::new();
        fonts
            .add(file("styles/shared/BarlowSemiCondensed-SemiBold.ttf"))
            .unwrap();
        let mut atlas = Atlas::default();
        let mut strikes = Strikes::new();
        let mut slots = |weight: f32, form: Form, atlas: &mut Atlas| {
            let style = Style {
                family: "\"Barlow Semi Condensed\"",
                weight,
                ..Default::default()
            };
            let mut t = Text::new();
            t.shape(&mut fonts, "B", &style);
            let mut out = None;
            t.runs(&fonts, |run| {
                let s = strikes.strike(atlas, &run, 2.0, true);
                let g = run.glyphs[0].id;
                out = strikes.glyph(&fonts, atlas, &s, run.coords, g, 0, form, [0.0; 3]);
            });
            out.expect("drawn").1
        };
        let ink = |atlas: &Atlas, s: GlyphSlot| -> u64 {
            let (pages, side) = atlas.pages(GlyphTier::Coverage);
            let p = &pages[usize::from(s.page)].pixels;
            (0..u32::from(s.height))
                .flat_map(|y| (0..u32::from(s.width)).map(move |x| (x, y)))
                .map(|(x, y)| {
                    u64::from(p[((u32::from(s.y) + y) * side + u32::from(s.x) + x) as usize])
                })
                .sum()
        };
        let fill = slots(600.0, Form::Fill, &mut atlas);
        let glyphs = atlas.stats().glyphs;
        assert_eq!(slots(600.0, Form::Fill, &mut atlas), fill);
        assert_eq!(atlas.stats().glyphs, glyphs, "found, not rastered again");
        let stroke = slots(600.0, Form::Stroke(4.0), &mut atlas);
        assert!(stroke.width >= fill.width + 3 && stroke.left < fill.left);
        let shadow = slots(600.0, Form::Shadow(6.0), &mut atlas);
        assert_eq!(shadow.width, fill.width + 18, "three sigmas a side");
        // The file is the family's one weight: 900 is drawn by emboldening.
        let bold = slots(900.0, Form::Fill, &mut atlas);
        assert!(ink(&atlas, bold) > ink(&atlas, fill) * 21 / 20);
        assert_eq!(atlas.stats().glyphs, glyphs + 3);
    }
}
