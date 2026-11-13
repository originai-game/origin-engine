//! Font pages added and drawn.

use super::*;

/// A cooked font as one `MsdfFont` per atlas page (Render's question 1: the
/// mapping lives here). Pages are uploaded linear, without mips.
pub(crate) fn add_font(
    scene: &mut RenderScene,
    font: &og_bundle::runtime::CookedFont,
) -> FontPages {
    let meta = &font.meta;
    let line_height = meta.ascender - meta.descender + meta.line_gap;
    let kerning: Vec<og_render::Kern> = font
        .kerning
        .iter()
        .filter_map(|k| {
            Some(og_render::Kern {
                left: char::from_u32(k.left)?,
                right: char::from_u32(k.right)?,
                adjust: k.adjust,
            })
        })
        .collect();
    // The distance-field pages; the colour glyphs' silhouettes (drawing
    // their shadows, glow and outline, before and so under their images);
    // the colour glyphs' pages (a plain sRGB atlas with its mips, drawn as
    // sprites: a distance range of 0); those in the text's colour. Each
    // entry: its data, the blob page whose rows it draws, and its kind
    // (colour, foreground, silhouette).
    let (n, c) = (font.pages.len(), font.color_pages.len());
    let fields = font
        .color_fields
        .iter()
        .enumerate()
        .map(|(i, d)| (d, n + i))
        .chain(
            font.foreground_fields
                .iter()
                .enumerate()
                .map(|(i, d)| (d, n + c + i)),
        );
    let all: Vec<(&og_render::TextureData, usize, bool, bool, bool)> = font
        .pages
        .iter()
        .enumerate()
        .map(|(i, d)| (d, i, false, false, false))
        .chain(fields.map(|(d, p)| (d, p, false, false, true)))
        .chain(
            font.color_pages
                .iter()
                .enumerate()
                .map(|(i, d)| (d, n + i, true, false, false)),
        )
        .chain(
            font.foreground_pages
                .iter()
                .enumerate()
                .map(|(i, d)| (d, n + c + i, true, true, false)),
        )
        .collect();
    let mut pages = Vec::with_capacity(all.len());
    for (data, p, colour, foreground, silhouette) in all {
        let atlas = scene.add_texture(if colour || silhouette {
            data.clone()
        } else {
            og_render::TextureData {
                mips: og_render::Mips::None,
                color_space: og_render::ColorSpace::Linear,
                ..data.clone()
            }
        });
        let glyphs = font
            .glyphs
            .iter()
            .filter_map(|g| {
                let key = g.key()?;
                let own = g.page as usize == p;
                Some(og_render::Glyph {
                    key,
                    advance: g.advance,
                    plane_bounds: if own {
                        [g.plane_left, g.plane_bottom, g.plane_right, g.plane_top]
                    } else {
                        [0.0; 4]
                    },
                    atlas_bounds: if own {
                        [
                            g.atlas_x as f32,
                            g.atlas_y as f32,
                            (g.atlas_x + g.atlas_w) as f32,
                            (g.atlas_y + g.atlas_h) as f32,
                        ]
                    } else {
                        [0.0; 4]
                    },
                })
            })
            .collect();
        pages.push(scene.add_font(og_render::MsdfFont {
            atlas,
            distance_range: if colour {
                0.0
            } else if silhouette {
                og_bundle::format::COLOR_FIELD_RANGE
            } else {
                meta.px_range
            },
            foreground,
            silhouette,
            line_height,
            glyphs,
            kerning: kerning.clone(),
        }));
    }
    FontPages { pages }
}

/// A multi-page font draws once per page with one layout: each overlay text
/// from `first` on whose font has more pages (`pages`, by its first) is
/// drawn again with each, its effects (shadows, outline, gradient fill: the
/// UI's `text_effects`, by text index) with it.
pub(crate) fn draw_other_pages(
    overlay: &mut og_render::Overlay,
    first: usize,
    pages: &std::collections::BTreeMap<og_render::FontHandle, Vec<og_render::FontHandle>>,
) {
    let effects: std::collections::HashMap<u32, og_render::UiTextEffects> = overlay
        .ui
        .text_effects
        .iter()
        .filter(|e| e.text as usize >= first)
        .map(|e| (e.text, e.clone()))
        .collect();
    let end = overlay.texts.len();
    for i in first..end {
        let t = &overlay.texts[i];
        let Some(rest) = pages.get(&t.font).and_then(|p| p.get(1..)) else {
            continue;
        };
        let t = t.clone();
        for &page in rest {
            let k = overlay.texts.len() as u32;
            overlay.texts.push(og_render::Text {
                font: page,
                ..t.clone()
            });
            if let Some(e) = effects.get(&(i as u32)) {
                overlay.ui.text_effects.push(og_render::UiTextEffects {
                    text: k,
                    ..e.clone()
                });
            }
        }
    }
}
