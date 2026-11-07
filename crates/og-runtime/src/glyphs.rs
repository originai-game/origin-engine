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
use og_render::{ColorSpace, FontHandle, Glyph, GlyphKey, Mips, TextureData, TextureHandle};
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
    /// A pixel font's grid (0: an outline font): its glyphs are rendered
    /// as their cells, as the cook renders them.
    pub pixel: u32,
    /// Glyphs handled (rendered, or not in the font either).
    done: BTreeSet<GlyphKey>,
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
    pub fn new(file: Arc<FontFile>, em_px: f32, px_range: f32, pixel: u32) -> RuntimeFont {
        RuntimeFont {
            file,
            em_px,
            px_range,
            pixel,
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
        let mut want: BTreeSet<GlyphKey> = og_core::sort::set(rf.file.take_wanted());
        for t in rs.texts.iter().filter(|t| t.font == render) {
            want.extend(t.text.chars().map(GlyphKey::Char));
        }
        let has = |k: GlyphKey| first.glyphs.iter().any(|g| g.key == k);
        let want: Vec<GlyphKey> = want
            .into_iter()
            .filter(|&k| {
                !k.char()
                    .is_some_and(|c| c.is_whitespace() || c.is_control())
                    && !rf.done.contains(&k)
                    && !has(k)
            })
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
            let Some(img) =
                og_ui::glyphs::render_glyph_on(rf.file.bytes(), c, rf.em_px, rf.px_range, rf.pixel)
            else {
                continue;
            };
            rendered += 1;
            if img.width == 0 {
                added.push((
                    Glyph {
                        key: c,
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
                // The page this batch leaves keeps what it drew: uploaded
                // now, or its glyphs sample the texture of its last upload
                // (a page made in this batch, all zero) and draw blank.
                if let Some(old) = rf.page.take()
                    && touched.contains(&(old.font.index() as u32))
                {
                    upload(&mut rs.scene, &old);
                }
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
                    key: c,
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
            upload(&mut rs.scene, p);
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
                if font.glyphs.iter().any(|h| h.key == g.key) {
                    // A page created in this batch already lists it by advance.
                    if *owner == Some(p)
                        && let Some(h) = font.glyphs.iter_mut().find(|h| h.key == g.key)
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

/// Uploads a run-time page's atlas as it now stands.
fn upload(scene: &mut og_render::RenderScene, page: &Page) {
    scene.set_texture(
        page.atlas,
        TextureData {
            width: PAGE,
            height: PAGE,
            rgba: page.rgba.clone(),
            color_space: ColorSpace::Linear,
            mips: Mips::None,
        },
    );
}

/// The characters of `text` that font `render` (an index into
/// `RenderState::fonts`) lacks, in its atlas and in the file it renders
/// from at run time, with its first page; each (font, character) is judged
/// once a run (`RenderState::glyphs_judged`). Read on the CPU when the text
/// is queued, so a run with no renderer warns as one with a GPU does.
pub(crate) fn lacking(
    rs: &mut RenderState,
    render: usize,
    text: &str,
) -> Option<(FontHandle, Vec<char>)> {
    let first = *rs.fonts.get(render)?.pages.first()?;
    let font = rs.scene.font(first)?;
    let file = rs.runtime_fonts.get(&render).map(|f| f.file.clone());
    let mut lacks = Vec::new();
    for c in text.chars() {
        if c.is_whitespace() || c.is_control() || !rs.glyphs_judged.insert((first, c)) {
            continue;
        }
        if !font.glyphs.iter().any(|g| g.key == GlyphKey::Char(c))
            && !file.as_ref().is_some_and(|f| f.covers(c))
        {
            lacks.push(c);
        }
    }
    (!lacks.is_empty()).then_some((first, lacks))
}

/// Logs the missing-glyph warnings of `missing` (per font, the characters
/// its texts draw that it lacks) on `tick`: each character named once per
/// font, with the font's asset and what its atlas holds (the charset words
/// its glyphs show, og_bundle::format::charset_words).
pub(crate) fn warn_missing(
    eng: &crate::engine::Engine,
    tick: u64,
    missing: Vec<(FontHandle, Vec<char>)>,
) {
    if missing.is_empty() {
        return;
    }
    let notes: Vec<String> = {
        let (rs, assets) = (eng.render.borrow(), eng.assets.borrow());
        missing
            .iter()
            .map(|(handle, chars)| note(&rs, &assets, *handle, chars))
            .collect()
    };
    let mut st = eng.state.borrow_mut();
    for n in notes {
        if st.ui_warned.insert(n.clone()) {
            st.log
                .push(serde_json::json!({ "tick": tick, "warning": true, "text": n }));
        }
    }
}

/// Of what og-ui's frame found missing (`og_ui::UiFrame::missing_glyphs`),
/// the (font, character) pairs no queued text has judged yet, judged now.
pub(crate) fn unjudged(
    rs: &mut RenderState,
    missing: &[(FontHandle, Vec<char>)],
) -> Vec<(FontHandle, Vec<char>)> {
    missing
        .iter()
        .filter_map(|(handle, chars)| {
            let chars: Vec<char> = chars
                .iter()
                .copied()
                .filter(|&c| rs.glyphs_judged.insert((*handle, c)))
                .collect();
            (!chars.is_empty()).then_some((*handle, chars))
        })
        .collect()
}

/// One font's warning: the characters (the first eight), the font's
/// asset, what its atlas holds and the three ways to fix it.
fn note(
    rs: &RenderState,
    assets: &crate::assets::Assets,
    handle: FontHandle,
    chars: &[char],
) -> String {
    let which = chars
        .iter()
        .take(8)
        .map(|c| format!("\"{c}\" U+{:04X}", *c as u32))
        .collect::<Vec<_>>()
        .join(", ");
    let more = if chars.len() > 8 {
        format!(" and {} more", chars.len() - 8)
    } else {
        String::new()
    };
    let cps: Vec<u32> = rs.scene.font(handle).map_or(Vec::new(), |f| {
        f.glyphs
            .iter()
            .filter_map(|g| g.key.char())
            .map(|c| c as u32)
            .collect()
    });
    let (words, extra) = og_bundle::format::charset_words(&cps);
    let holds = match (words.is_empty(), extra) {
        (true, _) => format!("{} characters", cps.len()),
        (false, 0) => format!("charset {}", words.join(" ")),
        (false, n) => format!("charset {} and {n} more characters", words.join(" ")),
    };
    let font = rs
        .fonts
        .iter()
        .position(|f| f.pages.contains(&handle))
        .and_then(|at| assets.fonts.iter().find(|f| f.render == at))
        .map_or("a font".to_owned(), |f| format!("font {}", f.id));
    let chars_arg: String = chars.iter().collect();
    format!(
        "a frame drew {which}{more}, which {font} lacks (its atlas holds {holds}): add them to its .ogt (`font {{ chars {chars_arg:?} }}`), draw a character it holds, or ship the font file (`font {{ runtime true }}`)"
    )
}

/// What a text in the UI's body face lacks (a floating text with no font
/// of its own): per piece, the characters its face has neither in its atlas
/// nor in its file, judged as [`unjudged`] judges og-ui's.
pub(crate) fn ui_lacking(rs: &mut RenderState, text: &str) -> Vec<(FontHandle, Vec<char>)> {
    let Some(ui) = rs.ui.clone() else {
        return Vec::new();
    };
    let found: Vec<(FontHandle, Vec<char>)> = {
        let u = ui.lock();
        let fonts = u.fonts();
        let font = fonts.for_text(og_ui::FontRole::Body, text);
        og_ui::pieces(font, text, u.direction(), 0.0)
            .into_iter()
            .filter_map(|p| {
                let h = p.font?;
                let face = fonts.face(h)?;
                let lacks: Vec<char> = p
                    .text
                    .chars()
                    .filter(|&c| !c.is_whitespace() && !c.is_control() && !face.metrics.has(c))
                    .collect();
                (!lacks.is_empty()).then_some((h, lacks))
            })
            .collect()
    };
    unjudged(rs, &found)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::FontPages;

    /// One batch that fills more than a page (the kit's ja ability cards:
    /// the katakana of one frame on one page, its kanji on the next): every
    /// glyph a page lists has ink in that page's uploaded atlas. Each page
    /// but the last once kept the texture it was made with, all zero, so
    /// its glyphs drew blank (ン, リ and ド of 偏向スクリーン and ターン終了).
    #[test]
    fn a_batch_over_several_pages_uploads_every_page() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../templates/styles/cjk/NotoSansSC-Regular.ttf"
        );
        let file = FontFile::new(Arc::new(std::fs::read(path).unwrap())).unwrap();
        let mut rs = RenderState::new(serde_json::Value::Null);
        let atlas = rs.scene.add_texture(TextureData {
            width: 4,
            height: 4,
            rgba: vec![0; 64],
            color_space: ColorSpace::Linear,
            mips: Mips::None,
        });
        let first = rs.scene.add_font(og_render::MsdfFont {
            atlas,
            distance_range: 16.0,
            foreground: false,
            line_height: 1.2,
            glyphs: Vec::new(),
            kerning: Vec::new(),
        });
        rs.fonts.push(FontPages { pages: vec![first] });
        rs.runtime_fonts
            .insert(0, RuntimeFont::new(file.clone(), 48.0, 16.0, 0));
        let wanted: Vec<char> = "偏向スクリーンターン終了修理ドローン"
            .chars()
            .chain((0x4e00..0x4e60).filter_map(char::from_u32))
            .collect();
        for &c in &wanted {
            file.advance(c);
        }
        assert!(generate(&mut rs) > 0);
        let pages = rs.fonts[0].pages.clone();
        assert!(
            pages.len() > 2,
            "the batch fills more than one page: {}",
            pages.len()
        );
        let mut checked = 0;
        for &p in &pages[1..] {
            let font = rs.scene.font(p).unwrap().clone();
            let tex = rs.scene.texture(font.atlas).unwrap();
            for g in font
                .glyphs
                .iter()
                .filter(|g| g.atlas_bounds[2] > g.atlas_bounds[0])
            {
                let [x0, y0, x1, y1] = g.atlas_bounds.map(|v| v as usize);
                let inked = (y0..y1).any(|y| {
                    (x0..x1).any(|x| {
                        let o = (y * tex.width as usize + x) * 4;
                        let mut m = [tex.rgba[o], tex.rgba[o + 1], tex.rgba[o + 2]];
                        m.sort_unstable();
                        m[1] > 127
                    })
                });
                assert!(inked, "{:?} is blank on its page", g.key);
                checked += 1;
            }
        }
        let covered: BTreeSet<char> = wanted.iter().copied().filter(|&c| file.covers(c)).collect();
        assert!("ンリド".chars().all(|c| covered.contains(&c)));
        assert_eq!(checked, covered.len());
    }
}
