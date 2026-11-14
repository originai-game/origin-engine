//! og-ui's cooked-font text frames (`references/<tier>/text-bidi.png`,
//! `text-kerning`, `text-edit`, `text-mixed-runs`) as scenes ready to
//! render, from the fonts [`FONTS`] names cooked (the project
//! `tests/text-fonts`, its settings beside the former test sample's font files).
//! Shared by the native test (`tests/text_gfx.rs`, which cooks them with
//! og-cook) and the browser tiers (crates/og-tests-web includes this file;
//! packages/engine/tests/web/reference-gfx.ts cooks the same project with
//! `og cook` and hands the module the blobs), so every tier draws the same
//! frame from the same cook. Nothing here reads a file: the fonts come in
//! decoded ([`fonts_from`] decodes them from a bundle's bytes).
//!
//! - `text-bidi`: Arabic shaped and joined, a right-to-left paragraph with
//!   its number, an Arabic run inside a Latin label and inside one rich-text
//!   span (bidi inside the span), Hebrew, Devanagari's conjuncts and signs;
//! - `text-kerning`: the display face and the body face each with and
//!   without their kerning pairs, one above the other, and labels cut with
//!   an ellipsis on one line and on a second;
//! - `text-edit`: a field with a selection made by Shift+←, a field with an
//!   input method's composition underlined at the caret, and a text area of
//!   three lines with its caret;
//! - `text-mixed-runs` ([`mixed_runs_ui`]): labels mixing Latin (the
//!   display face as the body) with Japanese, Arabic (shaped), Hebrew and
//!   Devanagari, each run in its own face, one wrapping; rich text; a
//!   focused field with a mixed name;
//! - `text-colr`: COLR v1 colour glyphs (gradients, a composite) on the
//!   font's colour page, and glyphs in the text's colour (palette entry
//!   0xFFFF) on its foreground page, under a green and a magenta label;
//! - `text-vertical`: CSS writing modes in a document: a `vertical-rl`
//!   paragraph of Japanese with Latin lying sideways, its comma and full
//!   stop set for a vertical line and its brackets turned (their own face,
//!   which has no other text), wrapping into a second line to its left; the
//!   same `upright`; a `vertical-lr` box of two blocks; a
//!   `sideways-lr` label reading up; and `text-combine-upright` digits.

use super::scenes::mixed_runs_ui;
use og_bundle::runtime::{Bundle, CookedFont};
use og_core::glam::{Vec2, Vec3};
use og_render::{
    Background, ColorSpace, Glyph, Kern, Mips, MsdfFont, RenderDesc, RenderScene, TextureData,
    Tonemap,
};
use og_ui::shape::FontFile;
use og_ui::*;
use std::sync::Arc;

/// The fonts the frames draw with, in the order the scenes take them: the
/// files of tests/fixtures/ui-text/assets/fonts, each cooked with its settings
/// `tests/text-fonts/assets/fonts/<file>.ogt`.
pub const FONTS: [&str; 8] = [
    "Inter-Display.ttf",
    "NotoSansSC-Regular-subset.ttf",
    "NotoSansArabic-subset.ttf",
    "NotoSansHebrew-subset.ttf",
    "NotoSansDevanagari-subset.ttf",
    "NotoSansJP-subset.otf",
    "test_glyphs-glyf_colr_1.ttf",
    "NotoSansJP-brackets.ttf",
];

/// The cooked-font reference frames.
pub const TEXT_SCENES: [&str; 6] = [
    "text-bidi",
    "text-kerning",
    "text-edit",
    "text-mixed-runs",
    "text-colr",
    "text-vertical",
];

/// The cooked [`FONTS`] of a bundle (the cook's manifest), each blob's
/// bytes given by `read` (its file under `cooked/`).
pub fn fonts_from(
    bundle: &Bundle,
    mut read: impl FnMut(&str) -> Option<Vec<u8>>,
) -> Result<Vec<CookedFont>, String> {
    FONTS
        .iter()
        .map(|f| {
            let id = format!("fonts/{f}");
            let out = bundle.output(&id).map_err(|e| e.to_string())?;
            let bytes =
                read(&out.file).ok_or_else(|| format!("{id}: no cooked file {}", out.file))?;
            bundle.font(&id, &bytes).map_err(|e| e.to_string())
        })
        .collect()
}

/// The scene and renderer description of cooked-font frame `name`.
pub fn text_scene(name: &str, fonts: &[CookedFont]) -> Result<(RenderScene, RenderDesc), String> {
    if fonts.len() != FONTS.len() {
        return Err(format!(
            "{} fonts given, the frames draw with {}",
            fonts.len(),
            FONTS.len()
        ));
    }
    Ok(match name {
        "text-bidi" => bidi_scene(fonts),
        "text-kerning" => kerning_scene(fonts),
        "text-edit" => edit_scene(fonts),
        "text-mixed-runs" => mixed_runs_scene(fonts),
        "text-colr" => colr_scene(fonts),
        "text-vertical" => vertical_scene(fonts),
        _ => {
            return Err(format!(
                "no cooked-font text frame {name} (there are {})",
                TEXT_SCENES.join(", ")
            ));
        }
    })
}

/// The UI frames drawn over the theme's background into one scene, at
/// [`vp`]'s device size, untonemapped.
fn shot(uis: &[&Ui], mut scene: RenderScene) -> (RenderScene, RenderDesc) {
    let v = vp();
    let mut overlay = og_render::Overlay::new(v.width, v.height, v.dpr);
    for ui in uis {
        ui.frame().emit(&EmitDesc::default(), &mut overlay);
    }
    scene.ui.overlay = Some(overlay);
    let bg = uis[0].theme().palette.backdrop.linear();
    scene.lighting.environment.background = Background::Color(Vec3::new(bg[0], bg[1], bg[2]));
    scene.look.post.tonemap = Tonemap::None;
    let desc = RenderDesc {
        width: (v.width * v.dpr) as u32,
        height: (v.height * v.dpr) as u32,
        ..RenderDesc::default()
    };
    (scene, desc)
}

/// A cooked font (one page) in the scene, as og-runtime adds it, kerned or not.
pub fn add(scene: &mut RenderScene, f: &CookedFont, kern: bool) -> UiFont {
    assert_eq!(f.pages.len(), 1, "{}: one page", f.id);
    let page = &f.pages[0];
    let atlas = scene.add_texture(TextureData {
        mips: Mips::None,
        color_space: ColorSpace::Linear,
        ..page.clone()
    });
    let m = &f.meta;
    let font = MsdfFont {
        atlas,
        distance_range: m.px_range,
        foreground: false,
        silhouette: false,
        line_height: m.ascender - m.descender + m.line_gap,
        glyphs: f
            .glyphs
            .iter()
            .filter_map(|g| {
                Some(Glyph {
                    key: g.key()?,
                    advance: g.advance,
                    plane_bounds: [g.plane_left, g.plane_bottom, g.plane_right, g.plane_top],
                    atlas_bounds: [
                        g.atlas_x as f32,
                        g.atlas_y as f32,
                        (g.atlas_x + g.atlas_w) as f32,
                        (g.atlas_y + g.atlas_h) as f32,
                    ],
                })
            })
            .collect(),
        kerning: if kern {
            f.kerning
                .iter()
                .filter_map(|k| {
                    Some(Kern {
                        left: char::from_u32(k.left)?,
                        right: char::from_u32(k.right)?,
                        adjust: k.adjust,
                    })
                })
                .collect()
        } else {
            Vec::new()
        },
    };
    // The unkerned control is atlas-only: retaining OpenType data would
    // let the paragraph shaper read GPOS despite clearing the atlas pairs.
    let metrics = FontMetrics::from_msdf(&font).with_file(
        kern.then(|| {
            f.ttf
                .clone()
                .and_then(|t| FontFile::new(Arc::new(t)))
                .or_else(|| {
                    f.shaping
                        .clone()
                        .and_then(|t| FontFile::for_shaping(Arc::new(t)))
                })
        })
        .flatten(),
    );
    UiFont {
        handle: Some(scene.add_font(font)),
        metrics,
    }
}

struct Faces {
    display: UiFont,
    display_flat: UiFont,
    body: UiFont,
    body_flat: UiFont,
    arabic: UiFont,
    hebrew: UiFont,
    devanagari: UiFont,
}

fn faces(scene: &mut RenderScene, fonts: &[CookedFont]) -> Faces {
    Faces {
        display: add(scene, &fonts[0], true),
        display_flat: add(scene, &fonts[0], false),
        body: add(scene, &fonts[1], true),
        body_flat: add(scene, &fonts[1], false),
        arabic: add(scene, &fonts[2], true),
        hebrew: add(scene, &fonts[3], true),
        devanagari: add(scene, &fonts[4], true),
    }
}

fn ui_of(f: &Faces, display: &UiFont, body: &UiFont) -> Ui {
    Ui::new(UiDesc {
        fonts: Fonts {
            body: body.clone(),
            display: Some(display.clone()),
            fallbacks: vec![f.arabic.clone(), f.hebrew.clone(), f.devanagari.clone()],
            ..Fonts::default()
        },
        ..UiDesc::default()
    })
}

fn vp() -> Viewport {
    Viewport {
        width: 640.0,
        height: 400.0,
        dpr: 1.5,
        ..Viewport::default()
    }
}

/// Mounts in `ui` a document `name` of `body` in a column starting `top`
/// CSS px down the surface (several UIs, each with its own focus, are
/// drawn into one frame, one band each), dressed by `style`, and builds it
/// twice.
fn band(ui: &mut Ui, name: &str, top: f32, body: &str, style: &str) {
    band_with(ui, name, top, body, style, &[]);
}

/// [`band`] reading the stores `stores`.
fn band_with(
    ui: &mut Ui,
    name: &str,
    top: f32,
    body: &str,
    style: &str,
    stores: &[(&str, serde_json::Value)],
) {
    let doc = format!(
        r#"<meta name="component" content="{name}">
<column id="Panel">{body}</column>
<style>
#Panel {{ width: stretch; padding: calc(var(--space-lg) + {top}px) var(--space-lg) 0; gap: var(--space-sm); }}
{style}
</style>"#
    );
    super::scenes::mount_screen_document(ui, name, &doc, stores);
    for _ in 0..2 {
        step(ui, InputFrame::default());
    }
}

/// One frame of `ui`'s mounted documents at [`vp`].
fn step(ui: &mut Ui, input: InputFrame) {
    super::scenes::step_screen(ui, input, vp());
}

/// A colour font's pages in the scene as og-runtime adds them
/// (extract::add_font): the field's; the colour glyphs' silhouettes (their
/// shadows, glow and outline, drawn first and so under their images); the
/// colour pages (a distance range of 0); those in the text's colour. The
/// first page's handle lays the text out, every page draws it.
pub fn add_pages(scene: &mut RenderScene, f: &CookedFont) -> Vec<og_render::FontHandle> {
    let m = &f.meta;
    let (n, c) = (f.pages.len(), f.color_pages.len());
    let fields = f
        .color_fields
        .iter()
        .enumerate()
        .map(|(i, d)| (d, n + i))
        .chain(
            f.foreground_fields
                .iter()
                .enumerate()
                .map(|(i, d)| (d, n + c + i)),
        );
    // (data, the blob page whose rows it draws, colour, foreground, silhouette)
    let all: Vec<(&TextureData, usize, bool, bool, bool)> = f
        .pages
        .iter()
        .enumerate()
        .map(|(i, d)| (d, i, false, false, false))
        .chain(fields.map(|(d, p)| (d, p, false, false, true)))
        .chain(
            f.color_pages
                .iter()
                .enumerate()
                .map(|(i, d)| (d, n + i, true, false, false)),
        )
        .chain(
            f.foreground_pages
                .iter()
                .enumerate()
                .map(|(i, d)| (d, n + c + i, true, true, false)),
        )
        .collect();
    let mut handles = Vec::new();
    for (data, p, colour, foreground, silhouette) in all {
        let atlas = scene.add_texture(if colour || silhouette {
            data.clone()
        } else {
            TextureData {
                mips: Mips::None,
                color_space: ColorSpace::Linear,
                ..data.clone()
            }
        });
        let glyphs = f
            .glyphs
            .iter()
            .filter_map(|g| {
                let own = g.page as usize == p;
                let bounds = |b: [f32; 4]| if own { b } else { [0.0; 4] };
                Some(Glyph {
                    key: g.key()?,
                    advance: g.advance,
                    plane_bounds: bounds([
                        g.plane_left,
                        g.plane_bottom,
                        g.plane_right,
                        g.plane_top,
                    ]),
                    atlas_bounds: bounds([
                        g.atlas_x as f32,
                        g.atlas_y as f32,
                        (g.atlas_x + g.atlas_w) as f32,
                        (g.atlas_y + g.atlas_h) as f32,
                    ]),
                })
            })
            .collect();
        handles.push(scene.add_font(MsdfFont {
            atlas,
            distance_range: if colour {
                0.0
            } else if silhouette {
                og_bundle::format::COLOR_FIELD_RANGE
            } else {
                m.px_range
            },
            foreground,
            silhouette,
            line_height: m.ascender - m.descender + m.line_gap,
            glyphs,
            kerning: Vec::new(),
        }));
    }
    handles
}

fn colr_scene(fonts: &[CookedFont]) -> (RenderScene, RenderDesc) {
    let mut scene = RenderScene::new();
    let f = faces(&mut scene, fonts);
    let pages = add_pages(&mut scene, &fonts[6]);
    let colr = UiFont::new(pages[0], scene.font(pages[0]).expect("the field page"));
    let mut ui = ui_of(&f, &colr, &f.body);
    // A linear and a radial gradient, a composite, then a gradient through
    // the text's colour and the text's colour alone, in the display face.
    let glyphs = "\u{F0100}\u{F0503}\u{F0600} \u{F0B00}\u{F0B06}";
    band(
        &mut ui,
        "colr",
        0.0,
        &format!(
            r#"<text id="Caption">COLR v1: gradients, a composite, the text's colour</text>
<text id="Green" class="colr">{glyphs}</text>
<text id="Magenta" class="colr">{glyphs}</text>"#
        ),
        ".colr { font-size: 64px; line-height: 80px; font-family: fantasy; white-space: nowrap; }
#Green { color: rgb(0 200 60); }
#Magenta { color: rgb(220 40 200); }",
    );
    let (mut scene, desc) = shot(&[&ui], scene);
    // The colour font's other pages draw each of its texts too.
    let overlay = scene.ui.overlay.as_mut().expect("the frame's overlay");
    let more: Vec<og_render::Text> = overlay
        .texts
        .iter()
        .filter(|t| t.font == pages[0])
        .flat_map(|t| {
            pages[1..].iter().map(move |&h| og_render::Text {
                font: h,
                ..t.clone()
            })
        })
        .collect();
    overlay.texts.extend(more);
    (scene, desc)
}

fn bidi_scene(fonts: &[CookedFont]) -> (RenderScene, RenderDesc) {
    let mut scene = RenderScene::new();
    let f = faces(&mut scene, fonts);
    let mut ui = ui_of(&f, &f.display, &f.body);
    band(
        &mut ui,
        "bidi",
        0.0,
        r#"<text id="Joined" class="big">مغامرة صغيرة</text>
<text id="Rtl">الإصدار 0.1.0 متابعة</text>
<text id="Mixed">Level 1-2: مغامرة صغيرة!</text>
<text id="Span"><span part="primary">Go مغامرة صغيرة now</span>, then <b>العب</b> 2</text>
<text id="Hebrew" class="big">שלום עולם</text>
<text id="Devanagari" class="big">नमस्ते दुनिया हिन्दी</text>"#,
        ".big { font: var(--text-xl); letter-spacing: var(--type-xl-tracking); }
#Span::part(primary) { color: var(--primary); }",
    );
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    // A right-to-left UI: start is the right edge, and a Latin word and a
    // number keep their own order inside the Arabic line.
    let mut rtl = ui_of(&f, &f.display, &f.body);
    rtl.set_direction(og_i18n::Direction::Rtl);
    band(
        &mut rtl,
        "rtl",
        300.0,
        r#"<text id="Title">الإعدادات</text>
<text id="Line">حذف <b>Slot 2</b> مغامرة صغيرة؟</text>"#,
        "#Line { width: stretch; }",
    );
    shot(&[&ui, &rtl], scene)
}

fn kerning_scene(fonts: &[CookedFont]) -> (RenderScene, RenderDesc) {
    let mut scene = RenderScene::new();
    let f = faces(&mut scene, fonts);
    // The same lines with and without the fonts' kerning pairs, one band each.
    let lines = |ui: &mut Ui, name: &str, top: f32| {
        band(
            ui,
            name,
            top,
            r#"<text id="Title">AVATAR Toyota WAVE</text>
<text id="Body">Tower, Yo-yo, AV, To, LT: kerned pairs</text>"#,
            "#Title { font: var(--text-display-md); font-family: fantasy; letter-spacing: var(--type-display-md-tracking); white-space: nowrap; width: max-content; }",
        );
    };
    let mut kerned = ui_of(&f, &f.display, &f.body);
    lines(&mut kerned, "kerned", 0.0);
    let mut flat = ui_of(&f, &f.display_flat, &f.body_flat);
    lines(&mut flat, "flat", 96.0);
    let mut cut = ui_of(&f, &f.display, &f.body);
    band(
        &mut cut,
        "cut",
        210.0,
        r#"<text id="Cut">A label far too long for its one line ends in an ellipsis where it meets the edge of its column, however much of it is left</text>
<text id="Clamp">A paragraph clamped to two lines keeps what fits on them and ends the second with an ellipsis, so the rest of this sentence, which runs on well past two lines of the column, never shows below it.</text>"#,
        "#Cut { white-space: nowrap; text-overflow: ellipsis; overflow: hidden; width: stretch; }
#Clamp { line-clamp: 2; text-overflow: ellipsis; overflow: hidden; width: stretch; }",
    );
    let w = |ui: &Ui| ui.element("Title").expect("title").width;
    println!(
        "title {} px kerned, {} px without the pairs",
        w(&kerned),
        w(&flat)
    );
    assert!(
        w(&kerned) + 4.0 < w(&flat),
        "kerning tightens the title: {} vs {} unkerned",
        w(&kerned),
        w(&flat)
    );
    let cut_text: Vec<String> = cut
        .frame()
        .items
        .iter()
        .filter_map(|i| match &i.cmd {
            DrawCmd::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        cut_text.len(),
        3,
        "one cut line and two clamped: {cut_text:?}"
    );
    // GSUB can turn three stops into one glyph; the drawn command uses
    // atlas glyph keys, not the source string. Check its shaped suffix.
    let dots = if f.body.metrics.has('…') {
        "…"
    } else {
        "..."
    };
    let suffix: String = f
        .body
        .metrics
        .visual(dots, og_i18n::Direction::Ltr, 0.0)
        .iter()
        .map(|g| g.ch)
        .collect();
    assert!(!suffix.is_empty());
    let ends_cut = |t: &str| t.ends_with(&suffix);
    assert!(
        ends_cut(&cut_text[0]) && ends_cut(&cut_text[2]) && !ends_cut(&cut_text[1]),
        "{cut_text:?}"
    );
    shot(&[&kerned, &flat, &cut], scene)
}

/// A tap at a point: press, then release a tick later.
fn tap(ui: &mut Ui, at: Vec2) {
    for (pressed, released) in [(true, false), (false, true)] {
        step(
            ui,
            InputFrame {
                pointer: at,
                pointer_inside: true,
                down: pressed,
                pressed,
                released,
                ..InputFrame::default()
            },
        );
    }
}

fn keys(ui: &mut Ui, k: UiKey, n: usize, shift: bool) {
    for _ in 0..n {
        step(
            ui,
            InputFrame {
                keys: vec![k],
                shift,
                ..InputFrame::default()
            },
        );
    }
}

fn end_of(ui: &Ui, id: &str) -> Vec2 {
    let e = ui.element(id).expect(id);
    Vec2::new(e.x + e.width - 4.0, e.y + e.height / 2.0)
}

fn mixed_runs_scene(fonts: &[CookedFont]) -> (RenderScene, RenderDesc) {
    let mut scene = RenderScene::new();
    let f = faces(&mut scene, fonts);
    let japanese = add(&mut scene, &fonts[5], true);
    let ui = mixed_runs_ui(
        Fonts {
            body: f.display.clone(),
            fallbacks: vec![
                japanese.clone(),
                f.arabic.clone(),
                f.hebrew.clone(),
                f.devanagari.clone(),
            ],
            ..Fonts::default()
        },
        vp(),
    );
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    // Each mixed line is drawn in more than one face.
    let line = ui.element("Line#0").expect("the first line").clone();
    let faces: std::collections::BTreeSet<og_render::FontHandle> = ui
        .frame()
        .items
        .iter()
        .filter_map(|i| match &i.cmd {
            DrawCmd::Text { font, bounds, .. }
                if (bounds.y - line.y).abs() < 1.0 && bounds.x < line.x + line.width =>
            {
                Some(*font)
            }
            _ => None,
        })
        .collect();
    assert_eq!(faces.len(), 2, "Latin and Japanese in their own faces");
    // Every character a command draws is in the atlas of the face it names.
    let all = [&f.display, &japanese, &f.arabic, &f.hebrew, &f.devanagari];
    for i in &ui.frame().items {
        if let DrawCmd::Text { font, text, .. } = &i.cmd {
            let face = all
                .iter()
                .find(|u| u.handle == Some(*font))
                .expect("one of the scene's faces");
            for c in text.chars() {
                assert!(
                    face.metrics.in_atlas(c),
                    "{c:?} of {text:?} is not in its face"
                );
            }
        }
    }
    let mut discretionary = ui_of(&f, &f.display, &f.body);
    discretionary.define_action(ActionDesc {
        name: "jump".into(),
        keys: vec!["KeyE".into()],
        ..Default::default()
    });
    super::scenes::mount_screen_document(
        &mut discretionary,
        "discretionary",
        "<meta name=\"component\" content=\"discretionary\">
<column id=\"Discretionary\">
  <text id=\"Narrow\"><span part=\"primary\">extra\u{ad}</span>ordinary <kbd data-action=\"jump\"></kbd></text>
  <text id=\"Wide\"><span part=\"primary\">extra\u{ad}</span>ordinary <kbd data-action=\"jump\"></kbd></text>
</column>
<style>
#Discretionary { width: stretch; padding: 240px 0 0 420px; gap: 12px; }
#Narrow { width: 80px; }
#Wide { width: 190px; }
#Narrow::part(primary), #Wide::part(primary) { color: var(--primary); }
</style>",
        &[],
    );
    step(&mut discretionary, InputFrame::default());
    assert!(
        discretionary.frame().warnings.is_empty(),
        "{:?}",
        discretionary.frame().warnings
    );
    shot(&[&ui, &discretionary], scene)
}

fn edit_scene(fonts: &[CookedFont]) -> (RenderScene, RenderDesc) {
    let mut scene = RenderScene::new();
    let f = faces(&mut scene, fonts);
    // Three UIs, each with its own focus: a field with a selection, a
    // field with a composition, a text area with a selection over a soft
    // line break and its caret.
    let mut select = ui_of(&f, &f.display, &f.body);
    band_with(
        &mut select,
        "select",
        0.0,
        r#"<input id="Select" bind:value={$name}>"#,
        "#Select { width: stretch; }",
        &[("name", serde_json::json!("Ada Lovelace"))],
    );
    let at = end_of(&select, "Select");
    tap(&mut select, at);
    keys(&mut select, UiKey::Left, 8, true);
    assert_eq!(
        select.frame().text_input.clone().expect("field").selection,
        "Lovelace"
    );

    let mut compose = ui_of(&f, &f.display, &f.body);
    band_with(
        &mut compose,
        "compose",
        60.0,
        r#"<input id="Compose" bind:value={$hello}>"#,
        "#Compose { width: stretch; }",
        &[("hello", serde_json::json!("Hello "))],
    );
    let at = end_of(&compose, "Compose");
    tap(&mut compose, at);
    step(
        &mut compose,
        InputFrame::default().with_text(&og_input::TextInput::split("café", true)),
    );
    assert!(compose.frame().text_input.clone().expect("field").composing);

    let mut area = ui_of(&f, &f.display, &f.body);
    band_with(
        &mut area,
        "area",
        120.0,
        r#"<textarea id="Area" rows="4" bind:value={$notes}/>"#,
        "#Area { width: stretch; }",
        &[(
            "notes",
            serde_json::json!(
                "First line\nSecond line, long enough to wrap past the width of the area it is typed in, so it breaks between two words\nThird"
            ),
        )],
    );
    let e = area.element("Area").expect("area");
    let at = Vec2::new(e.x + 4.0, e.y + 4.0);
    tap(&mut area, at);
    keys(&mut area, UiKey::Down, 1, false);
    keys(&mut area, UiKey::Home, 1, false);
    keys(&mut area, UiKey::Right, 11, false);
    keys(&mut area, UiKey::Right, 6, true);
    assert_eq!(
        area.frame().text_input.clone().expect("area").selection,
        ", long"
    );
    shot(&[&select, &compose, &area], scene)
}

/// `text-vertical`'s document.
const VERTICAL: &str = r#"<meta name="component" content="vertical">
<div id="Page">
  <p id="Mixed">古い水車小屋、「冒険」 Level 2 『古い小屋』。</p>
  <p id="Upright">冒険 HP</p>
  <div id="Lr"><p id="Lr1">水車小屋</p><p id="Lr2">Level 3</p></div>
  <p id="Side">Sideways up</p>
  <div id="Combo"><p id="Day">冒険</p><p id="Num">12</p></div>
</div>
<style>
#Page { display: flex; gap: 28px; padding: 28px; align-items: flex-start; }
p { margin: 0; color: #f2ead8; font-size: 26px; }
#Mixed { writing-mode: vertical-rl; height: 250px; padding: 10px; background: #3a2f4a; }
#Upright { writing-mode: vertical-rl; text-orientation: upright; padding: 10px; background: #2f3f4a; }
#Lr { writing-mode: vertical-lr; height: 200px; padding: 10px; background: #2f4a3a; }
#Lr2 { color: #ffd27a; }
#Side { writing-mode: sideways-lr; padding: 10px; background: #4a3a2f; }
#Combo { writing-mode: vertical-rl; padding: 10px; background: #4a2f35; }
#Num { text-combine-upright: all; color: #ffd27a; }
</style>"#;

fn vertical_scene(fonts: &[CookedFont]) -> (RenderScene, RenderDesc) {
    let mut scene = RenderScene::new();
    let f = faces(&mut scene, fonts);
    let japanese = add(&mut scene, &fonts[5], true);
    // The brackets: no other face here maps them, so they turn.
    let brackets = add(&mut scene, &fonts[7], true);
    let mut ui = Ui::new(UiDesc {
        fonts: Fonts {
            body: f.display.clone(),
            fallbacks: vec![japanese, brackets],
            ..Fonts::default()
        },
        ..UiDesc::default()
    });
    ui.define_document(UiDocument::parse("ui/vertical.svelte", VERTICAL).expect("parses"));
    ui.mount_document(
        reactive::SourceId::new("test", "vertical"),
        "vertical",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .expect("mounts");
    let mut stats = None;
    for _ in 0..4 {
        ui.begin(InputFrame::default(), vp());
        ui.build_retained_documents(&bind::NoEnv).expect("builds");
        stats = Some(ui.end().stats);
    }
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let stats = stats.expect("a frame");
    assert_eq!(
        (stats.bindings, stats.reconciled),
        (0, 0),
        "idle: {stats:?}"
    );
    // The paragraph is a column of lines: taller than wide, two lines.
    let mixed = ui.element("Mixed").expect("the paragraph").clone();
    assert!(mixed.height > mixed.width, "{mixed:?}");
    shot(&[&ui], scene)
}
