//! A font per run inside one label: a text that mixes what the role's font
//! draws with what it does not is laid out run by run, each run from the
//! first of the UI's fonts that has it, and drawn with one text command per
//! font. The fonts: og-tests' MSDF test face cut to ASCII as the role's
//! (a Latin face), the whole face (ASCII and 原点引擎文字渲染) as a CJK
//! fallback, and the UI kit's Arabic file (shaped) after it.
//!
//! Ways it could fail, each covered below:
//! - one font per label again (the CJK fallback draws the Latin too), or a
//!   run from the wrong font → `each_run_takes_the_first_font_that_has_it`
//! - an Arabic run in a mixed label not shaped, or out of visual order →
//!   `an_arabic_run_is_shaped_with_its_own_font`
//! - spaces and punctuation start runs of their own (a lone comma in the
//!   Latin face between CJK) → `neutral_characters_stay_with_their_run`
//! - a locale's face, or a role's face that has everything, cut into runs
//!   → `a_text_one_font_sets_whole_is_not_cut`
//! - a base letter and its combining mark split across faces (the mark
//!   drawn from another face than its base), or a joiner pulling a run into
//!   another face → `a_grapheme_cluster_takes_one_font`
//! - measured with one font and drawn with another (the runs overlap or
//!   gap), a missing glyph in a drawn command, a different frame on a
//!   second build → `a_mixed_label_draws_one_command_per_run`
//! - a mixed label's runs placed by the role face's extents alone, so a
//!   taller fallback face rises out of the line box (or each run on a
//!   baseline of its own) → `a_mixed_labels_runs_share_a_baseline_that_holds_every_face`
//! - a style's Chinese companion passed over for the locale's face (which
//!   cooked the same characters), the role's Latin given to the companion,
//!   or the display role taking the body's companion →
//!   `a_styles_cjk_companion_draws_its_chinese`
//! - a display or numeric face's missing character (Orbitron has no
//!   middle dot) left missing though the style's body face has it →
//!   `a_display_face_falls_back_to_the_body_face`
//! - a `[wave]`, `[shake]` or `[pulse]` span that mixes faces drawn still,
//!   or its runs drawn from the wrong face when they move →
//!   `a_moving_span_that_mixes_faces_moves`
//! - a step's weight or an italic drawn with the regular face (faked) when
//!   the style has the real one, a weight within 100 of a real face still
//!   faked, or a Japanese or Korean text given the Simplified Chinese
//!   companion → `a_weight_and_a_locale_pick_the_styles_real_faces`
//! - a document's `@font-face` family matched by the nearest weight
//!   instead of CSS Fonts 4's order (a lighter face over a heavier one up
//!   to 500), a face drawing a character outside its `unicode-range`, an
//!   undeclared family in a `font-family` list not skipped, or a family
//!   with no face not falling back to its generic family's role (an
//!   element's or a `::part` run's) →
//!   `a_font_face_family_takes_its_faces_by_weight_and_range`

use crate::support;

use og_i18n::Direction;
use og_render::RenderScene;
use og_ui::shape::FontFile;
use og_ui::*;
use std::sync::Arc;
use support::*;

struct Faces {
    latin: UiFont,
    cjk: UiFont,
    arabic: UiFont,
}

/// The test face cut to ASCII (the role's), the whole face, and the kit's
/// Arabic file (fixed metrics: every shaped form is drawable).
fn faces(scene: &mut RenderScene) -> Faces {
    let (png, json, csv) = test_font_files();
    let ascii: String = csv
        .lines()
        .filter(|l| {
            l.split(',')
                .next()
                .and_then(|c| c.trim().parse::<u32>().ok())
                .is_some_and(|c| c < 0x80)
        })
        .map(|l| format!("{l}\n"))
        .collect();
    let latin = add_font(
        scene,
        &FontFiles {
            png: &png,
            json: &json,
            csv: &ascii,
        },
    );
    let cjk = add_font(
        scene,
        &FontFiles {
            png: &png,
            json: &json,
            csv: &csv,
        },
    );
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/ui-text/assets/fonts/NotoSansArabic-subset.ttf"
    );
    let file = FontFile::new(Arc::new(
        std::fs::read(path).expect("the kit's Arabic font"),
    ))
    .expect("a font rustybuzz reads");
    let arabic = UiFont {
        handle: cjk.handle,
        metrics: FontMetrics::fixed(0.5).with_file(Some(file)),
    };
    Faces { latin, cjk, arabic }
}

/// The font of each visual glyph.
fn runs(m: &FontMetrics, text: &str) -> Vec<u8> {
    m.visual(text, Direction::Ltr, 0.0)
        .iter()
        .map(|g| g.font)
        .collect()
}

/// The text commands of a frame: (text, font, x, baseline).
fn commands(frame: &UiFrame) -> Vec<(String, og_render::FontHandle, f32, f32)> {
    frame
        .items
        .iter()
        .filter_map(|i| match &i.cmd {
            DrawCmd::Text {
                text,
                font,
                x,
                baseline,
                ..
            } => Some((text.clone(), *font, *x, *baseline)),
            _ => None,
        })
        .collect()
}

#[test]
fn a_styles_cjk_companion_draws_its_chinese() {
    let mut scene = RenderScene::new();
    let f = faces(&mut scene);
    // The style's body and display companions as the engine ships them: an
    // atlas without the Chinese (ASCII here) and the font file, whose
    // glyphs render at run time.
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/cook-assets/assets/fonts/NotoSansSC-Regular-subset.ttf"
    );
    let file = FontFile::new(Arc::new(
        std::fs::read(path).expect("the template's Chinese font"),
    ))
    .expect("a font rustybuzz reads");
    let (png, json, csv) = test_font_files();
    let ascii: String = csv
        .lines()
        .filter(|l| {
            l.split(',')
                .next()
                .and_then(|c| c.trim().parse::<u32>().ok())
                .is_some_and(|c| c < 0x80)
        })
        .map(|l| format!("{l}\n"))
        .collect();
    let companion = |scene: &mut RenderScene| {
        let mut c = add_font(
            scene,
            &FontFiles {
                png: &png,
                json: &json,
                csv: &ascii,
            },
        );
        c.metrics = c.metrics.clone().with_file(Some(file.clone()));
        c
    };
    let (text_cjk, title_cjk) = (companion(&mut scene), companion(&mut scene));
    assert!(!text_cjk.metrics.in_atlas('原') && text_cjk.metrics.has('原'));
    let mut fonts = Fonts {
        body: f.latin.clone(),
        display: Some(f.latin.clone()),
        cjk: Some(text_cjk.clone()),
        cjk_display: Some(title_cjk.clone()),
        // The locale's face has every character cooked.
        chain: vec![f.cjk.clone()],
        ..Fonts::default()
    };
    fonts.link();
    for (role, companion) in [(FontRole::Body, &text_cjk), (FontRole::Display, &title_cjk)] {
        let font = fonts.for_text(role, "Score 原点");
        assert!(
            font.metrics.is_mixed(),
            "{role:?}: Latin and Chinese laid out per run"
        );
        assert_eq!(runs(&font.metrics, "Score 原点"), [0, 0, 0, 0, 0, 0, 1, 1]);
        assert_eq!(
            font.handle_of(0),
            f.latin.handle,
            "{role:?}: Latin from the role's face"
        );
        assert_eq!(
            font.handle_of(1),
            companion.handle,
            "{role:?}: Chinese from its companion"
        );
        // A label all in Chinese takes the companion too, not the locale's face.
        let whole = fonts.for_text(role, "原点");
        assert_eq!(runs(&whole.metrics, "原点"), [1, 1]);
        assert_eq!(whole.handle_of(1), companion.handle);
    }
    // Without companions the locale's face sets a Chinese label whole, as before.
    let mut plain = Fonts {
        body: f.latin.clone(),
        chain: vec![f.cjk.clone()],
        ..Fonts::default()
    };
    plain.link();
    assert_eq!(plain.for_text(FontRole::Body, "原点").handle, f.cjk.handle);
}

#[test]
fn a_weight_and_a_locale_pick_the_styles_real_faces() {
    let mut scene = RenderScene::new();
    let f = faces(&mut scene);
    let (png, json, csv) = test_font_files();
    let ascii: String = csv
        .lines()
        .filter(|l| {
            l.split(',')
                .next()
                .and_then(|c| c.trim().parse::<u32>().ok())
                .is_some_and(|c| c < 0x80)
        })
        .map(|l| format!("{l}\n"))
        .collect();
    // Each face its own handle: Latin ones cut to ASCII, CJK ones whole.
    let mut face = |csv: &str| {
        add_font(
            &mut scene,
            &FontFiles {
                png: &png,
                json: &json,
                csv,
            },
        )
    };
    let (bold, italic) = (face(&ascii), face(&ascii));
    let (sc, ja, ja_bold) = (face(&csv), face(&csv), face(&csv));
    let latin = |role, weight, italic, font: &UiFont| FontFace {
        role,
        cjk: false,
        locale: CjkLocale::Hans,
        weight,
        italic,
        font: font.clone(),
    };
    let companion = |locale, weight, font: &UiFont| FontFace {
        role: FontRole::Body,
        cjk: true,
        locale,
        weight,
        italic: false,
        font: font.clone(),
    };
    let mut fonts = Fonts {
        body: f.latin.clone(),
        cjk: Some(sc.clone()),
        faces: vec![
            latin(FontRole::Body, 700, false, &bold),
            latin(FontRole::Body, 400, true, &italic),
            companion(CjkLocale::Ja, 400, &ja),
            companion(CjkLocale::Ja, 700, &ja_bold),
        ],
        // The locale's own face, every character cooked.
        chain: vec![f.cjk.clone()],
        ..Fonts::default()
    };
    fonts.link();
    // Legacy labels read the semantic body role, not the default ramp:
    // the real bold face, tracked glyphs, breakpoint size and text scale
    // must all reach the draw commands, without scaling twice next frame.
    let mut theme = Theme::dark();
    theme.unit_pin = 1.0;
    theme.text_scale = 1.5;
    for bp in Breakpoint::ALL {
        let token = &mut theme.tokens.typography.steps[tokens::TypeRole::Body.index()][bp as usize];
        token.size = if bp == Breakpoint::PhonePortrait {
            20.0
        } else {
            26.0
        };
        token.line_height = token.size + 8.0;
        token.weight = 700;
        token.tracking_em = 0.1;
    }
    let mut ui = Ui::new(UiDesc {
        theme,
        fonts: fonts.clone(),
        ..UiDesc::default()
    });
    for (vp, expected) in [
        (Viewport::new(1280.0, 720.0), 39.0),
        (Viewport::new(390.0, 844.0), 30.0),
    ] {
        for _ in 0..2 {
            ui.begin(InputFrame::default(), vp);
            ui.label("Tracked", "Hi", LabelDesc::default());
            ui.end();
            let text = ui
                .frame()
                .items
                .iter()
                .find_map(|item| match &item.cmd {
                    DrawCmd::Text {
                        font,
                        size,
                        tracking,
                        ..
                    } => Some((*font, *size, *tracking)),
                    _ => None,
                })
                .expect("the label draws");
            assert_eq!(Some(text.0), bold.handle);
            assert_eq!(text.1, expected);
            assert_eq!(text.2, 0.1);
            let width = bold.metrics.measure("Hi", expected) + 0.2 * expected;
            assert!((ui.element("Tracked").unwrap().width - width).abs() < 0.6);
        }
    }
    let styled = |fonts: &Fonts, weight, italic, text| {
        let (font, synth) = fonts.for_styled(FontRole::Body, weight, italic, text);
        (font.handle_of(0), synth)
    };
    // The weight picks the real face, which needs no thickening.
    assert_eq!(
        styled(&fonts, 0, false, "Score"),
        (f.latin.handle, Synth::default())
    );
    assert_eq!(
        styled(&fonts, 700, false, "Score"),
        (bold.handle, Synth::default())
    );
    // 600 is within 100 of the bold face; 500 is nearer the regular.
    assert_eq!(styled(&fonts, 600, false, "Score").0, bold.handle);
    assert_eq!(styled(&fonts, 500, false, "Score").0, f.latin.handle);
    // An italic face first; its weight is short of bold, so bold is faked.
    assert_eq!(
        styled(&fonts, 700, true, "Score"),
        (
            italic.handle,
            Synth {
                bold: true,
                italic: false
            }
        )
    );
    // A style with one face fakes both, as before.
    let mut one = Fonts {
        body: f.latin.clone(),
        ..Fonts::default()
    };
    one.link();
    assert_eq!(
        styled(&one, 700, true, "Score"),
        (
            f.latin.handle,
            Synth {
                bold: true,
                italic: true
            }
        )
    );

    // Simplified Chinese: the style's own companion.
    let cjk = |fonts: &Fonts, weight| {
        let font = fonts
            .for_styled(FontRole::Body, weight, false, "Score 原点")
            .0;
        assert!(font.metrics.is_mixed());
        font.handle_of(1)
    };
    assert_eq!(cjk(&fonts, 0), sc.handle);
    // Japanese: its own companion, at the text's weight, never the SC one.
    fonts.cjk_locale = CjkLocale::of_locale("ja-JP");
    fonts.link();
    assert_eq!(cjk(&fonts, 0), ja.handle);
    assert_eq!(cjk(&fonts, 700), ja_bold.handle);
    // Korean, which the style names no companion for: the locale's face
    // sets it whole rather than another locale's forms.
    fonts.cjk_locale = CjkLocale::of_locale("ko");
    fonts.link();
    assert_eq!(fonts.for_text(FontRole::Body, "原点").handle, f.cjk.handle);
    assert_eq!(CjkLocale::of_locale("zh-TW"), CjkLocale::Hant);
    assert_eq!(CjkLocale::of_locale("zh_Hant_HK"), CjkLocale::Hant);
    assert_eq!(CjkLocale::of_locale("zh-CN"), CjkLocale::Hans);

    // Rich text's [b] draws the bold face once, with no faked copy; without
    // one it is the regular face twice (the smear), as before.
    let drawn = |fonts: Fonts| {
        let mut ui = Ui::new(UiDesc {
            fonts,
            ..UiDesc::default()
        });
        ui.begin(InputFrame::default(), Viewport::new(400.0, 200.0));
        ui.rich_text("Hint", "[b]Go[/b]", RichTextDesc::default());
        ui.end();
        commands(ui.frame())
            .into_iter()
            .map(|c| (c.0, c.1))
            .collect::<Vec<_>>()
    };
    let go = |h: Option<og_render::FontHandle>| ("Go".to_string(), h.unwrap());
    fonts.cjk_locale = CjkLocale::Hans;
    assert_eq!(drawn(fonts), [go(bold.handle)]);
    assert_eq!(drawn(one), [go(f.latin.handle), go(f.latin.handle)]);
}

fn fonts(f: &Faces) -> Fonts {
    let mut fonts = Fonts {
        body: f.latin.clone(),
        fallbacks: vec![f.cjk.clone(), f.arabic.clone()],
        ..Fonts::default()
    };
    fonts.link();
    fonts
}

#[test]
fn an_arabic_run_is_shaped_with_its_own_font() {
    let mut scene = RenderScene::new();
    let f = faces(&mut scene);
    let fonts = fonts(&f);
    let m = &fonts.for_text(FontRole::Body, "Level 3 مرحبا").metrics;
    let glyphs = m.visual("Level 3 مرحبا", Direction::Ltr, 0.0);
    let latin: Vec<&VisualGlyph> = glyphs.iter().filter(|g| g.font == 0).collect();
    let arabic: Vec<&VisualGlyph> = glyphs.iter().filter(|g| g.font == 2).collect();
    assert_eq!(
        latin.iter().map(|g| g.ch).collect::<String>(),
        "Level 3 ",
        "the Latin, the digit and the spaces in the role's face"
    );
    assert!(
        arabic.len() >= 4 && arabic.iter().all(|g| g.glyph.is_some() && g.rtl),
        "the Arabic run is shaped by the Arabic file: {arabic:?}"
    );
    // Visual order: the Latin first, then the Arabic run (its first letter last).
    assert!(glyphs[..8].iter().all(|g| g.font == 0));
    assert_eq!(
        glyphs.last().map(|g| g.index),
        Some(8),
        "مـ drawn at the right"
    );
    // The same shaping as the Arabic file alone gives the word.
    let alone: Vec<_> = f
        .arabic
        .metrics
        .visual("مرحبا", Direction::Ltr, 0.0)
        .iter()
        .map(|g| g.key())
        .collect();
    assert_eq!(arabic.iter().map(|g| g.key()).collect::<Vec<_>>(), alone);
}

#[test]
fn named_fonts_keep_runtime_fallback_glyphs_beside_digits() {
    let mut scene = RenderScene::new();
    let f = faces(&mut scene);
    let file = FontFile::new(Arc::new(
        std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/cook-assets/assets/fonts/NotoSansSC-Regular-subset.ttf"
        ))
        .unwrap(),
    ))
    .unwrap();
    let fallback = UiFont {
        handle: f.cjk.handle,
        metrics: f.latin.metrics.clone().with_file(Some(file)),
    };
    assert!(fallback.metrics.has('原'));
    assert!(!fallback.metrics.in_atlas('原'));
    let source = r#"<meta name="component" content="t"><p>原3点</p><style>p { font-family: Brand; }</style>"#;
    crate::support::both(source, &[], |d| {
        d.ui.set_fonts(Fonts {
            families: vec![FamilyFace {
                family: "Brand".into(),
                weight: (400, 400),
                font: f.latin.clone(),
                ..FamilyFace::default()
            }],
            chain: vec![fallback.clone()],
            ..Fonts::default()
        });
        d.step(InputFrame::default());
        let drawn = commands(d.ui.frame());
        assert_eq!(
            drawn.iter().map(|c| c.0.as_str()).collect::<String>(),
            "原3点",
            "{drawn:?}"
        );
        assert!(
            drawn
                .iter()
                .any(|c| c.0 == "3" && Some(c.1) == f.latin.handle)
        );
        assert!(
            drawn
                .iter()
                .any(|c| c.0 == "原" && Some(c.1) == fallback.handle)
        );
    });
}

#[test]
fn ordered_named_families_preserve_primary_glyphs_and_inherit() {
    let mut scene = RenderScene::new();
    let f = faces(&mut scene);
    for (markup, css, digit) in [
        (
            "<div><p>原3点</p></div>",
            "div { font-family: Missing, Brand, Companion, serif; }",
            f.latin.handle,
        ),
        (
            "<p><span part=\"word\">原3点</span></p>",
            "p::part(word) { font-family: Brand, Companion, serif; }",
            f.latin.handle,
        ),
        (
            "<p>原3点</p>",
            "p { font-family: Companion, Brand, serif; }",
            f.cjk.handle,
        ),
        (
            "<button>原3点</button>",
            "button::part(base) { font-family: Brand, Companion, serif; }",
            f.latin.handle,
        ),
        (
            "<div><p>原3点</p></div>",
            "div { font-family: serif, Companion; }",
            f.latin.handle,
        ),
        (
            "<p>原3点</p>",
            "p { font-family: serif, monospace, Brand; }",
            f.latin.handle,
        ),
        (
            "<p>原3点</p>",
            "p { font-family: monospace, serif, Brand; }",
            f.cjk.handle,
        ),
        (
            "<button>原3点</button>",
            "button::part(base) { font-family: Brand, serif, Companion; }",
            f.latin.handle,
        ),
        (
            "<p><span part=\"word\">原3点</span></p>",
            "p::part(word) { font-family: 'serif', serif; }",
            f.cjk.handle,
        ),
    ] {
        let source = format!("<meta name=\"component\" content=\"t\">{markup}<style>{css}</style>");
        crate::support::both(&source, &[], |d| {
            d.ui.set_fonts(Fonts {
                body: f.latin.clone(),
                mono: Some(f.cjk.clone()),
                families: vec![
                    FamilyFace {
                        family: "Brand".into(),
                        weight: (400, 400),
                        font: f.latin.clone(),
                        ..FamilyFace::default()
                    },
                    FamilyFace {
                        family: "Companion".into(),
                        weight: (400, 400),
                        font: f.cjk.clone(),
                        ..FamilyFace::default()
                    },
                    FamilyFace {
                        family: "serif".into(),
                        weight: (400, 400),
                        font: f.cjk.clone(),
                        ..FamilyFace::default()
                    },
                ],
                ..Fonts::default()
            });
            d.step(InputFrame::default());
            let drawn = commands(d.ui.frame());
            let glyphs: Vec<_> = drawn
                .iter()
                .flat_map(|c| c.0.chars().map(move |ch| (ch, Some(c.1))))
                .collect();
            assert_eq!(
                glyphs,
                [('原', f.cjk.handle), ('3', digit), ('点', f.cjk.handle)],
                "{source}"
            );
        });
    }
}

#[test]
fn a_font_face_family_takes_its_faces_by_weight_and_range() {
    let mut scene = RenderScene::new();
    let f = faces(&mut scene);
    let face = |family: &str, weight: u16, font: &UiFont, range: &[(u32, u32)]| FamilyFace {
        family: family.into(),
        document: "t".into(),
        weight: (weight, weight),
        font: UiFont {
            metrics: font.metrics.clone().with_range(range),
            ..font.clone()
        },
        ..FamilyFace::default()
    };
    let mut fonts = Fonts {
        body: f.latin.clone(),
        families: vec![
            face("Brand", 350, &f.arabic, &[]),
            face("Brand", 500, &f.cjk, &[]),
            // Declared first, so the later face wins where both have it.
            face("Split", 400, &f.cjk, &[(0x4E00, 0x9FFF)]),
            face("Split", 400, &f.latin, &[(0, 0x7F)]),
        ],
        ..Fonts::default()
    };
    fonts.link();
    let mut unnamed = Fonts {
        families: vec![face("Heavy", 700, &f.latin, &[])],
        ..Fonts::default()
    };
    unnamed.link();
    let role = unnamed.family_list(
        &[
            FontFamily::Generic(FontRole::Body),
            FontFamily::Named("Heavy".into()),
        ],
        FontRole::Body,
    );
    let (font, synth) = unnamed.for_styled(role, 700, false, "Go");
    assert_eq!(font.handle, f.latin.handle);
    assert_eq!(
        synth,
        Synth::default(),
        "an unrendered generic must not double-bold a real heavy face"
    );
    let family = |name| FontRole::Family(Family::new(name, FontRole::Body));
    let at = |w| fonts.for_styled(family("brand"), w, false, "Go").0.handle;
    assert_eq!(
        at(400),
        f.cjk.handle,
        "from 400, a heavier face up to 500 before a nearer lighter one"
    );
    assert_eq!(at(300), f.arabic.handle, "below 400, lighter first");
    assert_eq!(at(700), f.cjk.handle, "above 500, the heaviest below");
    assert_eq!(
        fonts.for_text(family("Nope"), "Go").handle,
        f.latin.handle,
        "a family with no face is its role"
    );
    // Each character from the face whose unicode-range has it.
    let m = &fonts.for_text(family("Split"), "Go 原点").metrics;
    let font = fonts.for_text(family("Split"), "Go 原点");
    let drawn: Vec<(char, Option<og_render::FontHandle>)> = m
        .visual("Go 原点", Direction::Ltr, 0.0)
        .iter()
        .map(|g| (g.ch, font.handle_of(g.font)))
        .collect();
    assert_eq!(
        drawn,
        [
            ('G', f.latin.handle),
            ('o', f.latin.handle),
            (' ', f.latin.handle),
            ('原', f.cjk.handle),
            ('点', f.cjk.handle),
        ]
    );

    // A document's font-family list: the first family it declares, the
    // generic family's role after it.
    let text = r#"<meta name="component" content="t">
<div>
  <p id="Named">Go</p>
  <p id="Skipped">Go</p>
  <p id="Short">Go</p>
  <p id="Variable">Go</p>
  <p id="GenericFirst">Go</p>
  <p id="Comma">Go</p>
  <p id="Run">G<span part="w">o</span></p>
</div>
<style>
@font-face { font-family: "Wide Face"; src: url(fonts/wide.ttf); }
p { width: max-content; }
#Named { font-family: "Wide Face", monospace; }
#Skipped { font-family: Undeclared, monospace; }
#Short { font: 16px Wide Face, serif; }
#Variable { --families: Missing, Wide Face, monospace; font-family: var(--families); }
#GenericFirst { font-family: monospace, "Wide Face"; }
#Comma { font-family: 'Face, Wide', monospace; }
#Run::part(w) { font-family: Undeclared, "Wide Face", monospace; }
</style>"#;
    crate::support::both(text, &[], |d| {
        d.ui.set_fonts(Fonts {
            body: UiFont::unrendered(0.5),
            mono: Some(UiFont::unrendered(1.0)),
            families: vec![
                FamilyFace {
                    family: "wide face".into(),
                    weight: (400, 400),
                    font: UiFont::unrendered(2.0),
                    ..FamilyFace::default()
                },
                FamilyFace {
                    family: "face, wide".into(),
                    weight: (400, 400),
                    font: UiFont::unrendered(3.0),
                    ..FamilyFace::default()
                },
            ],
            ..Fonts::default()
        });
        d.step(InputFrame::default());
        let (named, skipped, short) = (d.get("Named"), d.get("Skipped"), d.get("Short"));
        assert_eq!(
            named.width,
            2.0 * skipped.width,
            "the declared family, else the monospace role"
        );
        assert_eq!(short.width, 2.0 * 2.0 * 16.0, "the font shorthand's family");
        assert_eq!(d.get("Variable").width, named.width);
        assert_eq!(d.get("GenericFirst").width, skipped.width);
        assert_eq!(d.get("Comma").width, 1.5 * named.width);
        // A ::part run's list resolves as an element's: "o" in Wide Face
        // (2 em) after the body's "G" (half an em), not the monospace role.
        let em = named.width / 4.0;
        assert_eq!(
            d.get("Run").width,
            2.5 * em,
            "a run's first declared family"
        );
    });
}
