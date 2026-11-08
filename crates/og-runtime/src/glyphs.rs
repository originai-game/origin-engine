//! Glyphs rendered at run time (`font { runtime true }`, and every font
//! that ships its file for shaping): before a frame's text is laid out for
//! drawing, each character the UI measured from a font's file
//! (`og_ui::shape::FontFile::take_wanted`) or a queued text asked of it,
//! and that its atlas lacks, is rendered from the font's outlines
//! (`og_ui::glyphs::render_glyph`, the cook's geometry) into a 512² page
//! of the font's own. Every page of the font learns the glyph's advance,
//! so text drawn page by page keeps one layout; the page that holds it
//! learns its bounds. Pages are added as they fill. A colour font
//! (`font { color true runtime true }`) paints a glyph with a COLR paint
//! as the cook paints one (`og_ui::colr::render_runtime`) onto colour
//! pages, each with a silhouette page for its shadows, glow and outline.

use super::extract::RenderState;
use og_render::{ColorSpace, FontHandle, Glyph, GlyphKey, Mips, TextureData, TextureHandle};
use og_ui::glyphs::GlyphImage;
use og_ui::shape::FontFile;
use std::collections::BTreeSet;
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc::{Receiver, Sender};

/// The edge of a run-time page, in pixels (a foreground page is two wide).
const PAGE: u32 = 512;

/// A font whose missing characters are rendered at run time.
pub(crate) struct RuntimeFont {
    pub file: Arc<FontFile>,
    pub em_px: f32,
    pub px_range: f32,
    /// A pixel font's grid (0: an outline font): its glyphs are rendered
    /// as their cells, as the cook renders them.
    pub pixel: u32,
    /// A colour font's (`font { color true runtime true }`): a glyph with
    /// a COLR paint is painted in colour as the cook paints one, onto
    /// colour pages, instead of a distance field.
    pub color: Option<og_bundle::runtime::ColorRuntime>,
    /// Glyphs handled (rendered, or not in the font either).
    done: BTreeSet<GlyphKey>,
    /// The page of each kind being filled ([`Kind`] as index).
    pages: [Option<Page>; 3],
    /// Glyph batches rendering off the frame (natively, on a thread each):
    /// how many are out, and where they arrive.
    #[cfg(not(target_arch = "wasm32"))]
    flight: (usize, Sender<Vec<Rendered>>, Receiver<Vec<Rendered>>),
    /// Glyphs waiting their turn (the web: a few rendered each frame).
    #[cfg(target_arch = "wasm32")]
    queue: std::collections::VecDeque<GlyphKey>,
}

/// Glyphs the single-threaded web renders per frame and font, in distance
/// fields: a frame keeps its budget while a new script's characters arrive
/// over a few.
#[cfg(target_arch = "wasm32")]
const WEB_BUDGET: usize = 3;

/// What a colour glyph takes of [`WEB_BUDGET`]: painting its layers and
/// its silhouette's field costs about two distance fields (3.0 ms against
/// 1.5 natively, unoptimised; Noto Color Emoji at 64 px per em).
#[cfg(target_arch = "wasm32")]
const WEB_COLOUR_COST: usize = 2;

/// A glyph rendered off the frame.
enum Rendered {
    /// A distance field (an outline glyph).
    Field(GlyphImage),
    /// A colour glyph's images and silhouette field.
    Colour(og_ui::colr::RuntimeColor),
}

/// A run-time page's kind, as the cooked font's pages: a distance field;
/// colour images (sRGB, drawn as sprites); colour images in the text's
/// colour (each glyph painted black, its white twin half the page to the
/// right). A colour page has a silhouette page beside it, its glyphs at the
/// same places as a distance field, for their shadows, glow and outline.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Field = 0,
    Colour = 1,
    Foreground = 2,
}

struct Page {
    kind: Kind,
    font: FontHandle,
    atlas: TextureHandle,
    rgba: Vec<u8>,
    /// A colour page's silhouette page: its font, atlas and pixels.
    silhouette: Option<(FontHandle, TextureHandle, Vec<u8>)>,
    x: u32,
    y: u32,
    shelf: u32,
    /// Drawn into since its last upload.
    dirty: bool,
}

impl Kind {
    fn width(self) -> u32 {
        if self == Kind::Foreground {
            2 * PAGE
        } else {
            PAGE
        }
    }
}

impl Page {
    /// Where an image of `w`×`h` goes, a pixel of gutter round it (a new
    /// shelf when the row is full); `None` when the page is full.
    fn spot(&self, w: u32, h: u32) -> Option<(u32, u32)> {
        let (x, y) = if self.x + w + 1 > PAGE {
            (1, self.y + self.shelf + 1)
        } else {
            (self.x, self.y)
        };
        (y + h < PAGE && x + w < PAGE).then_some((x, y))
    }

    /// Uploads its atlas, and its silhouette's, as they now stand.
    fn upload(&mut self, scene: &mut og_render::RenderScene) {
        let colour = self.kind != Kind::Field;
        scene.set_texture(
            self.atlas,
            TextureData {
                width: self.kind.width(),
                height: PAGE,
                rgba: self.rgba.clone(),
                color_space: if colour {
                    ColorSpace::Srgb
                } else {
                    ColorSpace::Linear
                },
                mips: if colour { Mips::Generate } else { Mips::None },
            },
        );
        if let Some((_, atlas, rgba)) = &self.silhouette {
            scene.set_texture(*atlas, field_texture(rgba.clone()));
        }
        self.dirty = false;
    }
}

fn field_texture(rgba: Vec<u8>) -> TextureData {
    TextureData {
        width: PAGE,
        height: PAGE,
        rgba,
        color_space: ColorSpace::Linear,
        mips: Mips::None,
    }
}

impl RuntimeFont {
    pub fn new(
        file: Arc<FontFile>,
        em_px: f32,
        px_range: f32,
        pixel: u32,
        color: Option<og_bundle::runtime::ColorRuntime>,
    ) -> RuntimeFont {
        RuntimeFont {
            file,
            em_px,
            px_range,
            pixel,
            color,
            done: BTreeSet::new(),
            pages: [None, None, None],
            #[cfg(not(target_arch = "wasm32"))]
            flight: {
                let (tx, rx) = std::sync::mpsc::channel();
                (0, tx, rx)
            },
            #[cfg(target_arch = "wasm32")]
            queue: Default::default(),
        }
    }

    /// Glyphs asked for and not yet drawable.
    fn pending(&self) -> usize {
        #[cfg(not(target_arch = "wasm32"))]
        return self.flight.0;
        #[cfg(target_arch = "wasm32")]
        return self.queue.len();
    }

    /// Starts rendering `want` off the frame and answers the glyphs that
    /// finished since the last call; with `settle`, every one asked for.
    /// The distance fields are the costly part (milliseconds a glyph):
    /// rendered on the frame, a new script's first frame hitched.
    fn render(&mut self, want: Vec<GlyphKey>, settle: bool) -> Vec<Rendered> {
        let (em, range, grid) = (self.em_px, self.px_range, self.pixel);
        #[cfg(not(target_arch = "wasm32"))]
        {
            if !want.is_empty() {
                let (file, tx) = (self.file.clone(), self.flight.1.clone());
                let color = self.color.clone();
                self.flight.0 += 1;
                std::thread::spawn(move || {
                    let images = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        want.into_iter()
                            .filter_map(|k| render_one(file.bytes(), k, em, range, grid, &color))
                            .collect()
                    }))
                    .unwrap_or_default();
                    let _ = tx.send(images);
                });
            }
            let mut out = Vec::new();
            while self.flight.0 > 0 {
                let batch = if settle {
                    self.flight.2.recv().ok()
                } else {
                    self.flight.2.try_recv().ok()
                };
                let Some(batch) = batch else { break };
                self.flight.0 -= 1;
                out.extend(batch);
            }
            out
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.queue.extend(want);
            let file = self.file.clone();
            let mut out = Vec::new();
            let mut spent = 0;
            while settle || spent < WEB_BUDGET {
                let Some(k) = self.queue.pop_front() else {
                    break;
                };
                let r = render_one(file.bytes(), k, em, range, grid, &self.color);
                spent += match r {
                    Some(Rendered::Colour(_)) => WEB_COLOUR_COST,
                    _ => 1,
                };
                out.extend(r);
            }
            out
        }
    }
}

/// Glyph `k` of a font file: in colour when the font paints colour glyphs
/// and `k` has a COLR paint, else its distance field.
fn render_one(
    ttf: &[u8],
    k: GlyphKey,
    em: f32,
    range: f32,
    grid: u32,
    color: &Option<og_bundle::runtime::ColorRuntime>,
) -> Option<Rendered> {
    if let Some(c) = color
        && let Some(g) = og_ui::colr::render_runtime(ttf, k, c.em_px, &c.palette)
    {
        return Some(Rendered::Colour(g));
    }
    og_ui::glyphs::render_glyph_on(ttf, k, em, range, grid).map(Rendered::Field)
}

/// Glyphs an engine's run-time fonts were asked for that are still being
/// rendered: they draw from a later frame.
pub fn glyphs_pending(engine: u32) -> u32 {
    crate::engine::engine(engine).map_or(0, |eng| {
        eng.render.try_borrow().map_or(0, |rs| {
            rs.runtime_fonts.values().map(|f| f.pending() as u32).sum()
        })
    })
}

/// An image to place on a page of `kind`: its size and pixels, a
/// foreground glyph's white twin, a colour glyph's silhouette field.
struct Image<'a> {
    kind: Kind,
    width: u32,
    height: u32,
    rgba: &'a [u8],
    white: Option<&'a [u8]>,
    field: Option<&'a [u8]>,
}

/// Renders what each run-time font was asked for since the last frame.
/// Returns how many glyphs were rendered.
pub(crate) fn generate(rs: &mut RenderState) -> usize {
    let mut rendered = 0;
    // A capture draws every glyph asked for (`RenderState::glyphs_settle`).
    let settle = std::mem::take(&mut rs.glyphs_settle);
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
        rf.done.extend(want.iter().copied());
        let images = rf.render(want, settle);
        if images.is_empty() {
            continue;
        }
        // Rendered glyphs and the pages that draw each (none: advance only).
        let mut added: Vec<(Glyph, Vec<FontHandle>)> = Vec::new();
        let mut new_pages: Vec<FontHandle> = Vec::new();
        for img in images {
            rendered += 1;
            let (key, advance, plane, image) = match &img {
                Rendered::Field(g) => (
                    g.key,
                    g.advance,
                    g.plane,
                    (g.width > 0).then_some(Image {
                        kind: Kind::Field,
                        width: g.width,
                        height: g.height,
                        rgba: &g.rgba,
                        white: None,
                        field: None,
                    }),
                ),
                Rendered::Colour(c) => (
                    c.key,
                    c.advance,
                    c.image.plane,
                    Some(Image {
                        kind: if c.white.is_some() {
                            Kind::Foreground
                        } else {
                            Kind::Colour
                        },
                        width: c.image.width,
                        height: c.image.height,
                        rgba: &c.image.rgba,
                        white: c.white.as_deref(),
                        field: Some(&c.field),
                    }),
                ),
            };
            let Some(image) = image else {
                added.push((
                    Glyph {
                        key,
                        advance,
                        plane_bounds: [0.0; 4],
                        atlas_bounds: [0.0; 4],
                    },
                    Vec::new(),
                ));
                continue;
            };
            if image.width + 2 > PAGE || image.height + 2 > PAGE {
                continue;
            }
            let known: Vec<Glyph> = first
                .glyphs
                .iter()
                .chain(added.iter().map(|(g, _)| g))
                .map(|g| Glyph {
                    plane_bounds: [0.0; 4],
                    atlas_bounds: [0.0; 4],
                    ..*g
                })
                .collect();
            let page = place(
                &mut rf.pages[image.kind as usize],
                &mut rs.scene,
                &first,
                known,
                &image,
                &mut new_pages,
            );
            let (gx, gy) = (page.x, page.y);
            let mut owners = vec![page.font];
            owners.extend(page.silhouette.as_ref().map(|s| s.0));
            page.x += image.width + 1;
            page.shelf = page.shelf.max(image.height);
            added.push((
                Glyph {
                    key,
                    advance,
                    plane_bounds: plane,
                    atlas_bounds: [
                        gx as f32,
                        gy as f32,
                        (gx + image.width) as f32,
                        (gy + image.height) as f32,
                    ],
                },
                owners,
            ));
        }
        if added.is_empty() {
            continue;
        }
        // The atlases of the pages being filled, uploaded again.
        for p in rf.pages.iter_mut().flatten().filter(|p| p.dirty) {
            p.upload(&mut rs.scene);
        }
        // Every page learns the new glyphs: bounds on the pages that draw
        // each, the advance everywhere else.
        let mut all = pages.clone();
        all.extend(new_pages.iter().copied());
        for &p in &all {
            let Some(mut font) = rs.scene.font(p).cloned() else {
                continue;
            };
            for (g, owners) in &added {
                let own = owners.contains(&p);
                if let Some(h) = font.glyphs.iter_mut().find(|h| h.key == g.key) {
                    // A page created in this batch already lists it by advance.
                    if own {
                        *h = *g;
                    }
                    continue;
                }
                font.glyphs.push(if own {
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
            let scene = &rs.scene;
            if let Some(fp) = rs.fonts.get_mut(render) {
                fp.pages.extend(new_pages.iter().copied());
                in_draw_order(scene, &mut fp.pages);
            }
            if let Some(ui) = rs.ui_pages.get_mut(&head) {
                ui.extend(new_pages.iter().copied());
                in_draw_order(scene, ui);
            }
        }
    }
    rendered
}

/// The page of `slot`'s kind with room for `image`, a new one (and its
/// silhouette page) when there is none or it is full; draws the image
/// there and answers the page, its pen still before the image.
fn place<'p>(
    slot: &'p mut Option<Page>,
    scene: &mut og_render::RenderScene,
    first: &og_render::MsdfFont,
    known: Vec<Glyph>,
    image: &Image,
    new_pages: &mut Vec<FontHandle>,
) -> &'p mut Page {
    let (w, h) = (image.width, image.height);
    if slot.as_ref().and_then(|p| p.spot(w, h)).is_none() {
        // The page this leaves keeps what it drew: uploaded now, or its
        // glyphs sample the texture of its last upload (a page made in
        // this batch, all zero) and draw blank.
        if let Some(mut old) = slot.take()
            && old.dirty
        {
            old.upload(scene);
        }
        let kind = image.kind;
        let size = (kind.width() * PAGE * 4) as usize;
        let colour = kind != Kind::Field;
        let atlas = scene.add_texture(TextureData {
            width: kind.width(),
            height: PAGE,
            rgba: vec![0; size],
            color_space: if colour {
                ColorSpace::Srgb
            } else {
                ColorSpace::Linear
            },
            mips: if colour { Mips::Generate } else { Mips::None },
        });
        // A silhouette page draws before its colour page: their effects
        // lie under the images.
        let silhouette = colour.then(|| {
            let rgba = vec![0; (PAGE * PAGE * 4) as usize];
            let atlas = scene.add_texture(field_texture(rgba.clone()));
            let font = scene.add_font(og_render::MsdfFont {
                atlas,
                distance_range: og_bundle::format::COLOR_FIELD_RANGE,
                foreground: false,
                silhouette: true,
                glyphs: known.clone(),
                ..first.clone()
            });
            new_pages.push(font);
            (font, atlas, rgba)
        });
        // Every glyph so far, by advance only: this page draws its own.
        let font = scene.add_font(og_render::MsdfFont {
            atlas,
            distance_range: if colour { 0.0 } else { first.distance_range },
            foreground: kind == Kind::Foreground,
            silhouette: false,
            glyphs: known,
            ..first.clone()
        });
        new_pages.push(font);
        *slot = Some(Page {
            kind,
            font,
            atlas,
            rgba: vec![0; size],
            silhouette,
            x: 1,
            y: 1,
            shelf: 0,
            dirty: false,
        });
    }
    let page = slot.as_mut().expect("a page");
    let (gx, gy) = page.spot(w, h).expect("room on a new page");
    if gy != page.y {
        page.x = 1;
        page.y = gy;
        page.shelf = 0;
    }
    let stride = page.kind.width();
    let blit = |to: &mut [u8], stride: u32, at: u32, from: &[u8]| {
        for row in 0..h {
            let s = (row * w * 4) as usize;
            let d = (((gy + row) * stride + at) * 4) as usize;
            to[d..d + (w * 4) as usize].copy_from_slice(&from[s..s + (w * 4) as usize]);
        }
    };
    blit(&mut page.rgba, stride, gx, image.rgba);
    if let Some(white) = image.white {
        blit(&mut page.rgba, stride, gx + PAGE, white);
    }
    if let (Some((_, _, rgba)), Some(field)) = (page.silhouette.as_mut(), image.field) {
        blit(rgba, PAGE, gx, field);
    }
    page.dirty = true;
    page
}

/// Puts a font's pages in the cooked font's drawing order: distance-field
/// pages, then silhouettes (shadows, glow and outlines of colour glyphs),
/// then colour pages, so no glyph's effects draw over another's colour.
/// The first page, which names the font, stays first.
fn in_draw_order(scene: &og_render::RenderScene, pages: &mut [FontHandle]) {
    let rank = |p: &FontHandle| {
        scene.font(*p).map_or(0, |f| {
            if f.silhouette {
                1
            } else if f.distance_range == 0.0 {
                2
            } else {
                0
            }
        })
    };
    if let Some((_, rest)) = pages.split_first_mut() {
        rest.sort_by_key(rank);
    }
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
            silhouette: false,
            line_height: 1.2,
            glyphs: Vec::new(),
            kerning: Vec::new(),
        });
        rs.fonts.push(FontPages { pages: vec![first] });
        rs.runtime_fonts
            .insert(0, RuntimeFont::new(file.clone(), 48.0, 16.0, 0, None));
        let wanted: Vec<char> = "偏向スクリーンターン終了修理ドローン"
            .chars()
            .chain((0x4e00..0x4e60).filter_map(char::from_u32))
            .collect();
        for &c in &wanted {
            file.advance(c);
        }
        // The frame that asks for them does not wait for them; a capture
        // (settle) waits for every one.
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(generate(&mut rs), 0, "the frame waited for its glyphs");
        rs.glyphs_settle = true;
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
