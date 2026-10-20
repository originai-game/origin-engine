//! Glyphs rendered at run time (`font { runtime true }`, and every font
//! that ships its file for shaping): before a frame's text is laid out for
//! drawing, each character the UI measured from a font's file
//! (`og_ui::shape::FontFile::take_wanted`) or a queued text asked of it,
//! and that its atlas lacks, is rendered from the font's outlines
//! (`og_ui::glyphs::render_glyph`, the cook's geometry) into a 512² page
//! of the font's own. Every page of the font learns the glyph's advance,
//! so text drawn page by page keeps one layout; the page that holds it
//! learns its bounds. Pages are added as they fill.

use super::extract::RenderState;
use og_render::{ColorSpace, FontHandle, Glyph, Mips, TextureData, TextureHandle};
use og_ui::shape::FontFile;
use std::collections::BTreeSet;
use std::sync::Arc;

/// The edge of a run-time page, in pixels.
const PAGE: u32 = 512;

/// A font whose missing characters are rendered at run time.
pub(crate) struct RuntimeFont {
    pub file: Arc<FontFile>,
    pub em_px: f32,
    pub px_range: f32,
    /// Characters handled (rendered, or not in the font either).
    done: BTreeSet<char>,
    /// The page being filled.
    page: Option<Page>,
}

struct Page {
    font: FontHandle,
    atlas: TextureHandle,
    rgba: Vec<u8>,
    x: u32,
    y: u32,
    shelf: u32,
}

impl RuntimeFont {
    pub fn new(file: Arc<FontFile>, em_px: f32, px_range: f32) -> RuntimeFont {
        RuntimeFont {
            file,
            em_px,
            px_range,
            done: BTreeSet::new(),
            page: None,
        }
    }
}

/// Renders what each run-time font was asked for since the last frame.
/// Returns how many glyphs were rendered.
pub(crate) fn generate(rs: &mut RenderState) -> usize {
    let mut rendered = 0;
    let renders: Vec<usize> = rs.runtime_fonts.keys().copied().collect();
    for render in renders {
        let Some(pages) = rs.fonts.get(render).map(|p| p.pages.clone()) else {
            continue;
        };
        let Some(first) = pages.first().and_then(|p| rs.scene.font(*p)).cloned() else {
            continue;
        };
        let rf = rs.runtime_fonts.get_mut(&render).expect("listed");
        let mut want: BTreeSet<char> = rf.file.take_wanted().into_iter().collect();
        for t in rs.texts.iter().filter(|t| t.font == render) {
            want.extend(t.text.chars());
        }
        let has = |c: char| first.glyphs.iter().any(|g| g.codepoint == c);
        let want: Vec<char> = want
            .into_iter()
            .filter(|&c| !c.is_whitespace() && !c.is_control() && !rf.done.contains(&c) && !has(c))
            .collect();
        if want.is_empty() {
            continue;
        }
        // Rendered glyphs and the page each landed on.
        let mut added: Vec<(Glyph, Option<FontHandle>)> = Vec::new();
        let mut new_pages: Vec<FontHandle> = Vec::new();
        let mut touched: BTreeSet<u32> = BTreeSet::new();
        for c in want {
            rf.done.insert(c);
            let Some(img) = og_ui::glyphs::render_glyph(rf.file.bytes(), c, rf.em_px, rf.px_range)
            else {
                continue;
            };
            rendered += 1;
            if img.width == 0 {
                added.push((
                    Glyph {
                        codepoint: c,
                        advance: img.advance,
                        plane_bounds: [0.0; 4],
                        atlas_bounds: [0.0; 4],
                    },
                    None,
                ));
                continue;
            }
            if img.width + 2 > PAGE || img.height + 2 > PAGE {
                continue;
            }
            // Shelf packing with a pixel of gutter, a new page when full.
            let fits = |p: &Page| {
                let (x, y) = if p.x + img.width + 1 > PAGE {
                    (1, p.y + p.shelf + 1)
                } else {
                    (p.x, p.y)
                };
                y + img.height < PAGE && x + img.width < PAGE
            };
            if !rf.page.as_ref().is_some_and(fits) {
                let atlas = rs.scene.add_texture(TextureData {
                    width: PAGE,
                    height: PAGE,
                    rgba: vec![0; (PAGE * PAGE * 4) as usize],
                    color_space: ColorSpace::Linear,
                    mips: Mips::None,
                });
                // Every glyph so far, by advance only: this page draws its own.
                let mut glyphs: Vec<Glyph> = first
                    .glyphs
                    .iter()
                    .map(|g| Glyph {
                        plane_bounds: [0.0; 4],
                        atlas_bounds: [0.0; 4],
                        ..*g
                    })
                    .collect();
                glyphs.extend(added.iter().map(|(g, _)| Glyph {
                    plane_bounds: [0.0; 4],
                    atlas_bounds: [0.0; 4],
                    ..*g
                }));
                let font = rs.scene.add_font(og_render::MsdfFont {
                    atlas,
                    glyphs,
                    ..first.clone()
                });
                new_pages.push(font);
                rf.page = Some(Page {
                    font,
                    atlas,
                    rgba: vec![0; (PAGE * PAGE * 4) as usize],
                    x: 1,
                    y: 1,
                    shelf: 0,
                });
            }
            let page = rf.page.as_mut().expect("a page");
            if page.x + img.width + 1 > PAGE {
                page.x = 1;
                page.y += page.shelf + 1;
                page.shelf = 0;
            }
            let (gx, gy) = (page.x, page.y);
            for row in 0..img.height {
                let s = (row * img.width * 4) as usize;
                let d = (((gy + row) * PAGE + gx) * 4) as usize;
                page.rgba[d..d + (img.width * 4) as usize]
                    .copy_from_slice(&img.rgba[s..s + (img.width * 4) as usize]);
            }
            page.x += img.width + 1;
            page.shelf = page.shelf.max(img.height);
            touched.insert(page.font.index() as u32);
            added.push((
                Glyph {
                    codepoint: c,
                    advance: img.advance,
                    plane_bounds: img.plane,
                    atlas_bounds: [
                        gx as f32,
                        gy as f32,
                        (gx + img.width) as f32,
                        (gy + img.height) as f32,
                    ],
                },
                Some(page.font),
            ));
        }
        if added.is_empty() {
            continue;
        }
        // The atlas of the page being filled, uploaded again.
        if let Some(p) = &rf.page
            && touched.contains(&(p.font.index() as u32))
        {
            rs.scene.set_texture(
                p.atlas,
                TextureData {
                    width: PAGE,
                    height: PAGE,
                    rgba: p.rgba.clone(),
                    color_space: ColorSpace::Linear,
                    mips: Mips::None,
                },
            );
        }
        // Every page learns the new glyphs: bounds on the page that holds
        // each, the advance everywhere else.
        let mut all = pages.clone();
        all.extend(new_pages.iter().copied());
        for &p in &all {
            let Some(mut font) = rs.scene.font(p).cloned() else {
                continue;
            };
            for (g, owner) in &added {
                if font.glyphs.iter().any(|h| h.codepoint == g.codepoint) {
                    // A page created in this batch already lists it by advance.
                    if *owner == Some(p)
                        && let Some(h) = font.glyphs.iter_mut().find(|h| h.codepoint == g.codepoint)
                    {
                        *h = *g;
                    }
                    continue;
                }
                font.glyphs.push(if *owner == Some(p) {
                    *g
                } else {
                    Glyph {
                        plane_bounds: [0.0; 4],
                        atlas_bounds: [0.0; 4],
                        ..*g
                    }
                });
            }
            rs.scene.set_font(p, font);
        }
        if !new_pages.is_empty() {
            let head = pages[0];
            if let Some(fp) = rs.fonts.get_mut(render) {
                fp.pages.extend(new_pages.iter().copied());
            }
            if let Some(ui) = rs.ui_pages.get_mut(&head) {
                ui.extend(new_pages.iter().copied());
            }
        }
    }
    rendered
}
