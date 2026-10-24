//! og-ui's cooked-font text frames (`references/<tier>/text-bidi.png`,
//! `text-kerning`, `text-edit`, `text-mixed-runs`) as scenes ready to
//! render, from the fonts [`FONTS`] names cooked (the project
//! `tests/text-fonts`, its settings beside templates/ui-kit's font files).
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
//!   focused field with a mixed name.

use super::scenes::mixed_runs_ui;
use og_cook::runtime::{Bundle, CookedFont};
use og_core::glam::{Vec2, Vec3};
use og_render::{
    Background, ColorSpace, Glyph, Kern, Mips, MsdfFont, RenderDesc, RenderScene, TextureData,
    Tonemap,
};
use og_ui::shape::FontFile;
use og_ui::*;
use std::sync::Arc;

/// The fonts the frames draw with, in the order the scenes take them: the
/// files of templates/ui-kit/assets/fonts, each cooked with its settings
/// `tests/text-fonts/assets/fonts/<file>.ogt`.
pub const FONTS: [&str; 6] = [
    "Inter-Display.ttf",
    "NotoSansSC-Regular-subset.ttf",
    "NotoSansArabic-subset.ttf",
    "NotoSansHebrew-subset.ttf",
    "NotoSansDevanagari-subset.ttf",
    "NotoSansJP-subset.otf",
];

/// The cooked-font reference frames.
pub const TEXT_SCENES: [&str; 4] = ["text-bidi", "text-kerning", "text-edit", "text-mixed-runs"];

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
    let atlas = scene.add_texture(atlas_texture());
    let v = vp();
    let mut overlay = og_render::Overlay::new(v.width, v.height, v.dpr);
    for ui in uis {
        ui.frame().emit(&EmitDesc { atlas, layer: 0 }, &mut overlay);
    }
    scene.overlay = Some(overlay);
    let bg = uis[0].theme().palette.backdrop.linear();
    scene.environment.background = Background::Color(Vec3::new(bg[0], bg[1], bg[2]));
    scene.post.tonemap = Tonemap::None;
    let desc = RenderDesc {
        width: (v.width * v.dpr) as u32,
        height: (v.height * v.dpr) as u32,
        ..RenderDesc::default()
    };
    (scene, desc)
}

/// A cooked font (one page) in the scene, as og-runtime adds it, kerned or not.
fn add(scene: &mut RenderScene, f: &CookedFont, kern: bool) -> UiFont {
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
        line_height: m.ascender - m.descender + m.line_gap,
        glyphs: f
            .glyphs
            .iter()
            .filter_map(|g| {
                Some(Glyph {
                    codepoint: char::from_u32(g.codepoint)?,
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
    let metrics = FontMetrics::from_msdf(&font)
        .with_file(f.ttf.clone().and_then(|t| FontFile::new(Arc::new(t))));
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

fn panel(ui: &mut Ui, f: impl FnOnce(&mut Ui)) {
    band(ui, 0.0, f);
}

/// A column starting `top` CSS px down the surface: several UIs (each with
/// its own focus) drawn into one frame, one band each.
fn band(ui: &mut Ui, top: f32, f: impl FnOnce(&mut Ui)) {
    let t = *ui.theme();
    ui.column(
        "Panel",
        Element {
            width: Size::grow(),
            padding: Insets {
                left: t.space.lg,
                top: t.space.lg + top,
                right: t.space.lg,
                bottom: 0.0,
            },
            gap: t.space.sm,
            ..Element::column()
        },
        f,
    );
}

fn bidi_scene(fonts: &[CookedFont]) -> (RenderScene, RenderDesc) {
    let mut scene = RenderScene::new();
    let f = faces(&mut scene, fonts);
    let mut ui = ui_of(&f, &f.display, &f.body);
    for _ in 0..2 {
        ui.begin(InputFrame::default(), vp());
        panel(&mut ui, |ui| {
            let t = *ui.theme();
            let big = Some(t.type_ramp.xl);
            ui.label(
                "Joined",
                "مغامرة صغيرة",
                LabelDesc {
                    step: big,
                    ..LabelDesc::default()
                },
            );
            ui.label("Rtl", "الإصدار 0.1.0 متابعة", LabelDesc::default());
            ui.label("Mixed", "Level 1-2: مغامرة صغيرة!", LabelDesc::default());
            ui.rich_text(
                "Span",
                "[color=primary]Go مغامرة صغيرة now[/color], then [b]العب[/b] 2",
                RichTextDesc::default(),
            );
            ui.label(
                "Hebrew",
                "שלום עולם",
                LabelDesc {
                    step: big,
                    ..LabelDesc::default()
                },
            );
            ui.label(
                "Devanagari",
                "नमस्ते दुनिया हिन्दी",
                LabelDesc {
                    step: big,
                    ..LabelDesc::default()
                },
            );
        });
        ui.end();
    }
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    // A right-to-left UI: start is the right edge, and a Latin word and a
    // number keep their own order inside the Arabic line.
    let mut rtl = ui_of(&f, &f.display, &f.body);
    rtl.set_direction(og_i18n::Direction::Rtl);
    for _ in 0..2 {
        rtl.begin(InputFrame::default(), vp());
        band(&mut rtl, 300.0, |ui| {
            ui.label("Title", "الإعدادات", LabelDesc::default());
            ui.rich_text(
                "Line",
                "حذف [b]Slot 2[/b] مغامرة صغيرة؟",
                RichTextDesc {
                    width: Size::grow(),
                    ..RichTextDesc::default()
                },
            );
        });
        rtl.end();
    }
    shot(&[&ui, &rtl], scene)
}

fn kerning_scene(fonts: &[CookedFont]) -> (RenderScene, RenderDesc) {
    let mut scene = RenderScene::new();
    let f = faces(&mut scene, fonts);
    // The same lines with and without the fonts' kerning pairs, one band each.
    let lines = |ui: &mut Ui, top: f32| {
        band(ui, top, |ui| {
            let t = *ui.theme();
            ui.label(
                "Title",
                "AVATAR Toyota WAVE",
                LabelDesc {
                    step: Some(t.type_ramp.display_md),
                    font: FontRole::Display,
                    nowrap: true,
                    ..LabelDesc::default()
                },
            );
            ui.label(
                "Body",
                "Tower, Yo-yo, AV, To, LT: kerned pairs",
                LabelDesc::default(),
            );
        });
    };
    let mut kerned = ui_of(&f, &f.display, &f.body);
    let mut flat = ui_of(&f, &f.display_flat, &f.body_flat);
    let mut cut = ui_of(&f, &f.display, &f.body);
    for _ in 0..2 {
        kerned.begin(InputFrame::default(), vp());
        lines(&mut kerned, 0.0);
        kerned.end();
        flat.begin(InputFrame::default(), vp());
        lines(&mut flat, 96.0);
        flat.end();
        cut.begin(InputFrame::default(), vp());
        band(&mut cut, 210.0, |ui| {
            ui.label(
                "Cut",
                "A label far too long for its one line ends in an ellipsis where it meets the edge of its column, however much of it is left",
                LabelDesc {
                    nowrap: true,
                    ellipsis: true,
                    width: Size::grow(),
                    ..LabelDesc::default()
                },
            );
            ui.label(
                "Clamp",
                "A paragraph clamped to two lines keeps what fits on them and ends the second with an ellipsis, so the rest of this sentence, which runs on well past two lines of the column, never shows below it.",
                LabelDesc {
                    ellipsis: true,
                    max_lines: 2,
                    width: Size::grow(),
                    ..LabelDesc::default()
                },
            );
        });
        cut.end();
    }
    let w = |ui: &Ui| ui.frame().element("Title").expect("title").width;
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
    // The body face cooks Latin-1 only: its ellipsis is three stops.
    let ends_cut = |t: &str| t.ends_with("...") || t.ends_with('…');
    assert!(
        ends_cut(&cut_text[0]) && ends_cut(&cut_text[2]) && !ends_cut(&cut_text[1]),
        "{cut_text:?}"
    );
    shot(&[&kerned, &flat, &cut], scene)
}

/// A tap at a point: press, then release a tick later.
fn tap(ui: &mut Ui, build: &mut dyn FnMut(&mut Ui, InputFrame), at: Vec2) {
    for (pressed, released) in [(true, false), (false, true)] {
        build(
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

fn keys(ui: &mut Ui, build: &mut dyn FnMut(&mut Ui, InputFrame), k: UiKey, n: usize, shift: bool) {
    for _ in 0..n {
        build(
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
    let e = ui.frame().element(id).expect(id);
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
    let line = ui
        .frame()
        .element("Line#0")
        .expect("the first line")
        .clone();
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
    shot(&[&ui], scene)
}

fn edit_scene(fonts: &[CookedFont]) -> (RenderScene, RenderDesc) {
    let mut scene = RenderScene::new();
    let f = faces(&mut scene, fonts);
    // Three UIs, each with its own focus: a field with a selection, a
    // field with a composition, a text area with a selection over a soft
    // line break and its caret.
    let mut name = "Ada Lovelace".to_string();
    let mut select = ui_of(&f, &f.display, &f.body);
    let mut b = |ui: &mut Ui, input: InputFrame| {
        ui.begin(input, vp());
        band(ui, 0.0, |ui| {
            ui.text_field("Select", &mut name, &FieldDesc::default());
        });
        ui.end();
    };
    b(&mut select, InputFrame::default());
    let at = end_of(&select, "Select");
    tap(&mut select, &mut b, at);
    keys(&mut select, &mut b, UiKey::Left, 8, true);
    assert_eq!(
        select.frame().text_input.clone().expect("field").selection,
        "Lovelace"
    );

    let mut hello = "Hello ".to_string();
    let mut compose = ui_of(&f, &f.display, &f.body);
    let mut b = |ui: &mut Ui, input: InputFrame| {
        ui.begin(input, vp());
        band(ui, 60.0, |ui| {
            ui.text_field("Compose", &mut hello, &FieldDesc::default());
        });
        ui.end();
    };
    b(&mut compose, InputFrame::default());
    let at = end_of(&compose, "Compose");
    tap(&mut compose, &mut b, at);
    b(
        &mut compose,
        InputFrame::default().with_text(&og_input::TextInput::split("café", true)),
    );
    assert!(compose.frame().text_input.clone().expect("field").composing);

    let mut notes =
        "First line\nSecond line, long enough to wrap past the width of the area it is typed in, so it breaks between two words\nThird".to_string();
    let mut area = ui_of(&f, &f.display, &f.body);
    let mut b = |ui: &mut Ui, input: InputFrame| {
        ui.begin(input, vp());
        band(ui, 120.0, |ui| {
            ui.text_area(
                "Area",
                &mut notes,
                &TextAreaDesc {
                    rows: 4,
                    ..TextAreaDesc::default()
                },
            );
        });
        ui.end();
    };
    b(&mut area, InputFrame::default());
    let e = area.frame().element("Area").expect("area");
    let at = Vec2::new(e.x + 4.0, e.y + 4.0);
    tap(&mut area, &mut b, at);
    keys(&mut area, &mut b, UiKey::Down, 1, false);
    keys(&mut area, &mut b, UiKey::Home, 1, false);
    keys(&mut area, &mut b, UiKey::Right, 11, false);
    keys(&mut area, &mut b, UiKey::Right, 6, true);
    assert_eq!(
        area.frame().text_input.clone().expect("area").selection,
        ", long"
    );
    shot(&[&select, &compose, &area], scene)
}
