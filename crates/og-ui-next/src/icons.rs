//! The engine's built-in icons (Lucide, `third_party/lucide`) as one table
//! of stroke segments in their 24-unit space, made once when the first
//! icon paints and shared by every paint arena: an icon is a quad naming
//! its span, which the paint shader strokes 2 units wide and round.

use og_draw::TextureHandle;
use og_draw::paint::IconSegment;
use std::sync::{Arc, OnceLock};

/// The table and each icon's span of it, by name.
pub struct Icons {
    pub segments: Arc<[IconSegment]>,
    spans: Vec<(&'static str, u32, u32)>,
}

impl Icons {
    /// Icon `name`'s first segment and their count.
    pub fn find(&self, name: &str) -> Option<(u32, u32)> {
        self.spans
            .iter()
            .find(|(n, ..)| *n == name)
            .map(|&(_, first, count)| (first, count))
    }
}

/// The built-in icons' table.
pub fn icons() -> &'static Icons {
    static ICONS: OnceLock<Icons> = OnceLock::new();
    ICONS.get_or_init(|| {
        let mut segments = Vec::new();
        let mut spans = Vec::new();
        for &name in crate::default_icons::DEFAULT_ICON_NAMES {
            let first = segments.len() as u32;
            let svg = crate::default_icons::svg(name)
                .and_then(|s| crate::svg::parse_xml(s).ok())
                .expect("bundled Lucide SVG");
            for el in &svg.children {
                let Ok(Some(path)) =
                    crate::svg::shape_path(&el.name, &|n| el.get(n), [0.0, 0.0, 24.0, 24.0])
                else {
                    continue;
                };
                for (points, closed) in crate::svg::subpaths(&path) {
                    flatten(&points, closed, &mut segments);
                }
            }
            spans.push((name, first, segments.len() as u32 - first));
        }
        Icons {
            segments: segments.into(),
            spans,
        }
    })
}

/// A subpath (its start, then three points a cubic) as segments: a cubic
/// whose controls lie on its chord is one, a curve one per 1.5 units of
/// its control polygon (2 to 16).
fn flatten(points: &[[f32; 2]], closed: bool, out: &mut Vec<IconSegment>) {
    let Some(&start) = points.first() else {
        return;
    };
    let mut at = start;
    let (cubics, _) = points[1..].as_chunks::<3>();
    for &[p1, p2, p3] in cubics {
        let off = |p: [f32; 2]| {
            let (dx, dy) = (p3[0] - at[0], p3[1] - at[1]);
            let len = (dx * dx + dy * dy).sqrt().max(1e-6);
            ((p[0] - at[0]) * dy - (p[1] - at[1]) * dx).abs() / len
        };
        if off(p1) < 0.01 && off(p2) < 0.01 {
            out.push([at[0], at[1], p3[0], p3[1]]);
            at = p3;
            continue;
        }
        let d = |a: [f32; 2], b: [f32; 2]| {
            let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
            (dx * dx + dy * dy).sqrt()
        };
        let n = ((d(at, p1) + d(p1, p2) + d(p2, p3)) / 1.5)
            .ceil()
            .clamp(2.0, 16.0) as usize;
        let mut prev = at;
        for k in 1..=n {
            let t = k as f32 / n as f32;
            let u = 1.0 - t;
            let b = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
            let p = [
                b[0] * at[0] + b[1] * p1[0] + b[2] * p2[0] + b[3] * p3[0],
                b[0] * at[1] + b[1] * p1[1] + b[2] * p2[1] + b[3] * p3[1],
            ];
            out.push([prev[0], prev[1], p[0], p[1]]);
            prev = p;
        }
        at = p3;
    }
    if closed && (at[0] - start[0]).abs() + (at[1] - start[1]).abs() > 1e-4 {
        out.push([at[0], at[1], start[0], start[1]]);
    }
}

/// The icon atlas `Ui::icon` draws from: a grid of equal cells of a
/// texture, or an icon set cooked like a font (`font`: an `.ogt` of
/// `type icons`, og-cook's `cook::icons`), whose icons are distance-field
/// glyphs drawn crisp at any size and tinted by their ink.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct IconAtlas {
    pub texture: Option<TextureHandle>,
    pub columns: u32,
    pub rows: u32,
    /// Optional names, one per cell in reading order, for `icon_named`;
    /// an icon set's names, glyph U+E000 + i being icon i.
    pub names: Vec<String>,
    /// An icon set: its MTSDF font (`texture` is then unused).
    pub font: Option<og_text::text::UiFont>,
    /// The texture's cells are full-colour pictures, drawn in their own
    /// colours as an `<img>` is; false, masks drawn in the icon's ink, as a
    /// CSS `mask-image` over `currentColor` is. An icon set is masks.
    pub pictures: bool,
    /// The atlas that draws the names this one lacks (`UiDesc.fallbackIcons`):
    /// a game's whole set under a style pack's few painted icons.
    pub fallback: Option<Box<IconAtlas>>,
}

/// Where an icon of the atlas is drawn from ([`IconAtlas::find`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IconCell {
    /// An icon set's distance-field glyph.
    Glyph(og_draw::FontHandle, char),
    /// A cell of a texture grid, `[u0, v0, u1, v1]`.
    Image(TextureHandle, [f32; 4]),
    /// A lazily parsed engine vector, after every game-supplied icon layer.
    Builtin(&'static str),
}

/// A picture rich text draws inline by name (`[img=<name>]`): a texture
/// region, one em tall at `aspect` (width over height).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineImage {
    pub texture: TextureHandle,
    /// `[u0, v0, u1, v1]`, v down.
    pub uv: [f32; 4],
    pub aspect: f32,
}

/// The code point of an icon set's first icon (og-cook's `ICON_PUA`).
pub const ICON_PUA: u32 = 0xE000;

impl IconAtlas {
    /// The UV rectangle of cell `index`.
    pub fn uv(&self, index: u32) -> [f32; 4] {
        let (c, r) = (self.columns.max(1), self.rows.max(1));
        let (x, y) = ((index % c) as f32, ((index / c) % r) as f32);
        let (w, h) = (1.0 / c as f32, 1.0 / r as f32);
        [x * w, y * h, (x + 1.0) * w, (y + 1.0) * h]
    }

    pub fn index_of(&self, name: &str) -> Option<u32> {
        self.names.iter().position(|n| n == name).map(|i| i as u32)
    }

    /// Whether it draws anything (a texture, or an icon set's font).
    pub fn is_drawn(&self) -> bool {
        self.texture.is_some() || self.font.as_ref().is_some_and(|f| f.handle.is_some())
    }

    /// An icon set's glyph for icon `index`: its font and character.
    pub fn glyph(&self, index: u32) -> Option<(og_draw::FontHandle, char)> {
        let f = self.font.as_ref()?.handle?;
        Some((f, char::from_u32(ICON_PUA + index)?))
    }

    /// The icon named `name`: from this atlas where it draws one, else
    /// from its fallback's, then the engine's default vector set.
    pub fn find(&self, name: &str) -> Option<IconCell> {
        self.find_drawn(name).map(|(cell, _)| cell)
    }

    /// The icon named `name`, and whether it is a full-colour picture
    /// (its atlas's `pictures`), drawn in its own colours.
    pub fn find_drawn(&self, name: &str) -> Option<(IconCell, bool)> {
        let own = self.index_of(name).and_then(|k| match self.glyph(k) {
            Some((font, ch)) => Some((IconCell::Glyph(font, ch), false)),
            None => self
                .texture
                .map(|t| (IconCell::Image(t, self.uv(k)), self.pictures)),
        });
        own.or_else(|| self.fallback.as_deref()?.find_drawn(name))
            .or_else(|| {
                crate::default_icons::find(name).map(|name| (IconCell::Builtin(name), false))
            })
    }

    /// This atlas and the fallbacks under it, top first.
    pub fn layers(&self) -> impl Iterator<Item = &IconAtlas> {
        std::iter::successors(Some(self), |a| a.fallback.as_deref())
    }
}
