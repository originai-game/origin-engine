//! Shaped scripts and right-to-left layout (E6 localization), through
//! og-ui's public boundary: `FontMetrics::visual` with the UI kit's Arabic font
//! (`templates/ui-kit/assets/fonts/NotoSansArabic-subset.ttf`) and a `Ui`
//! laid out right to left.
//!
//! Ways it could fail, and the test that catches each:
//! - Arabic letters drawn in their isolated forms (no shaping), or the
//!   shaped glyphs in logical instead of visual order →
//!   `arabic_words_are_shaped_into_joined_forms_in_visual_order`
//! - a right-to-left paragraph's words, or its digits, in the wrong order;
//!   a left-to-right paragraph's embedded Arabic run not reversed; Hebrew
//!   (unshaped) not reversed; brackets not mirrored; neutrals not resolved
//!   by UAX #9; an isolate's text setting the paragraph's direction or
//!   losing its punctuation, or a bidi control drawn →
//!   `paragraphs_order_their_runs_by_the_bidi_algorithm`
//! - kerning ignored, or measured and drawn differently →
//!   `kerning_pairs_tighten_the_measure`
//! - an Arabic line measured word by word but drawn unshaped when its
//!   full stop is no shaped form of the atlas, so it draws wider than its
//!   box → `an_arabic_line_is_drawn_as_wide_as_it_was_laid_out`
//! - a right-to-left UI not mirrored, or its text not aligned to the right
//!   → `a_right_to_left_ui_mirrors_its_layout`
//! - a scroller off the surface's centre line clipped by its unmirrored box,
//!   so its content vanishes right to left
//!   → `a_scrollers_content_shows_where_it_is_mirrored_to`

use og_i18n::Direction;
use og_ui::shape::{FontFile, GLYPH_PUA};
use og_ui::*;
use std::sync::Arc;

fn arabic() -> FontMetrics {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/ui-kit/assets/fonts/NotoSansArabic-subset.ttf"
    );
    let file = FontFile::new(Arc::new(
        std::fs::read(path).expect("the kit's Arabic font"),
    ))
    .expect("a font rustybuzz reads");
    // Fixed metrics have every glyph, so every shaped form is drawable.
    FontMetrics::fixed(0.5).with_file(Some(file))
}

/// A line as drawn: its visual glyphs' characters.
fn visual(m: &FontMetrics, s: &str) -> String {
    m.visual(s, Direction::Ltr, 0.0)
        .iter()
        .map(|g| g.ch)
        .collect()
}

fn glyphs(s: &str) -> Vec<u32> {
    s.chars().map(|c| c as u32 - GLYPH_PUA).collect()
}

#[test]
fn arabic_words_are_shaped_into_joined_forms_in_visual_order() {
    let m = arabic();
    // "مرحبا" (hello): مـ initial, ـر final, ح initial, ـبـ medial, ـا final.
    let word = visual(&m, "مرحبا");
    assert!(
        word.chars().all(|c| c as u32 >= GLYPH_PUA),
        "every glyph is a shaped form: {word:?}"
    );
    let isolated: Vec<u32> = "مرحبا"
        .chars()
        .map(|c| glyphs(&visual(&m, &c.to_string()))[0])
        .collect();
    let joined = glyphs(&word);
    assert!(joined.len() >= 5, "{joined:?}");
    let forms = joined.iter().filter(|g| !isolated.contains(g)).count();
    assert!(
        forms >= 3,
        "most letters take a joined form ({forms} of {}): {joined:?} vs {isolated:?}",
        joined.len()
    );
    // Visual order: the word's first letter (مـ, initial) is drawn last, at the right,
    // as it is in "ما".
    let ma = glyphs(&visual(&m, "ما"));
    assert_eq!(
        joined.last(),
        ma.last(),
        "both end, at the right, with the initial مـ"
    );
    assert_ne!(
        ma[1], isolated[0],
        "the initial form differs from the isolated"
    );
    // The glyphs carry where they were in the text: the rightmost is the first character.
    let g = m.visual("مرحبا", Direction::Ltr, 0.0);
    assert_eq!(g.last().map(|g| g.index), Some(0));
    assert!(g.iter().all(|g| g.rtl));
}

#[test]
fn paragraphs_order_their_runs_by_the_bidi_algorithm() {
    let m = arabic();
    let ar = |w: &str| visual(&m, w);
    let space = visual(&m, "ا ا").chars().nth(1).expect("a shaped space");
    // A right-to-left paragraph: the first word is drawn rightmost; digits keep their order.
    let line = visual(&m, "الإصدار 0.1.0");
    assert_eq!(line, format!("0.1.0{space}{}", ar("الإصدار")));
    // One right-to-left run is shaped whole: its words from right to left.
    let two = visual(&m, "مغامرة صغيرة");
    assert_eq!(two, format!("{}{space}{}", ar("صغيرة"), ar("مغامرة")));
    // A left-to-right paragraph keeps its order; its Arabic run reads right
    // to left inside, and the final "!" (neutral, at the paragraph's end)
    // takes the paragraph's direction (UAX #9 N1/N2).
    let mixed = visual(&m, "Level: مغامرة صغيرة!");
    assert_eq!(mixed, format!("Level: {two}!"));
    // Brackets mirror inside a right-to-left run.
    let hebrew = FontMetrics::fixed(0.5);
    assert_eq!(
        hebrew
            .visual("שלום (עולם)", Direction::Ltr, 0.0)
            .iter()
            .map(|g| g.ch)
            .collect::<String>(),
        "(םלוע) םולש"
    );
    // Hebrew is not shaped, only reversed; without a font file Arabic is reversed too.
    assert_eq!(
        hebrew
            .visual("שלום", Direction::Ltr, 0.0)
            .iter()
            .map(|g| g.ch)
            .collect::<String>(),
        "םולש"
    );
    assert_eq!(visual(&m, "plain text"), "plain text");
    // A number inside a right-to-left paragraph stays left to right, and the
    // paragraph's own words around it run right to left (UAX #9 W/L rules).
    let h = hebrew
        .visual("שלב 12 מתוך 40", Direction::Ltr, 0.0)
        .iter()
        .map(|g| g.ch)
        .collect::<String>();
    assert_eq!(h, "40 ךותמ 12 בלש");
    // Game text isolated (FSI … PDI) inside a right-to-left string keeps its
    // full stop; without the isolate the stop goes to the paragraph's end
    // (the left). The isolate sets no paragraph direction (UAX #9 P2), and
    // no bidi control is drawn.
    let line = |s: &str| {
        hebrew
            .visual(s, Direction::Ltr, 0.0)
            .iter()
            .map(|g| g.ch)
            .collect::<String>()
    };
    assert_eq!(line("שלום: Hold it."), ".Hold it :םולש");
    assert_eq!(line("שלום: \u{2068}Hold it.\u{2069}"), "Hold it. :םולש");
    assert_eq!(line("\u{2068}Hold\u{2069} שלום"), "םולש Hold");
}

/// Kerning: the font's pairs move the pen between neighbours in measuring
/// and in the visual layout alike; a font without pairs measures as before.
#[test]
fn kerning_pairs_tighten_the_measure() {
    use og_render::{Glyph, Kern, MsdfFont, TextureHandle};
    let glyph = |c: char| Glyph {
        codepoint: c,
        advance: 0.6,
        plane_bounds: [0.0, 0.0, 0.5, 0.7],
        atlas_bounds: [0.0, 0.0, 1.0, 1.0],
    };
    let font = |kerning: Vec<Kern>| MsdfFont {
        atlas: TextureHandle::from_slot(1).expect("a handle"),
        distance_range: 4.0,
        line_height: 1.2,
        glyphs: "AVT o".chars().map(glyph).collect(),
        kerning,
    };
    let plain = FontMetrics::from_msdf(&font(Vec::new()));
    let kerned = FontMetrics::from_msdf(&font(vec![
        Kern {
            left: 'A',
            right: 'V',
            adjust: -0.1,
        },
        Kern {
            left: 'T',
            right: 'o',
            adjust: -0.05,
        },
    ]));
    assert_eq!(plain.measure("AVTo", 10.0), 4.0 * 0.6 * 10.0);
    assert!((kerned.measure("AVTo", 10.0) - (2.4 - 0.15) * 10.0).abs() < 1e-4);
    assert!(
        (kerned.measure("VA", 10.0) - 12.0).abs() < 1e-4,
        "pairs are ordered"
    );
    let sum: f32 = kerned
        .visual("AVTo", Direction::Ltr, 0.0)
        .iter()
        .map(|g| g.advance)
        .sum();
    assert!((sum * 10.0 - kerned.measure("AVTo", 10.0)).abs() < 1e-4);
}

#[test]
fn a_right_to_left_ui_mirrors_its_layout() {
    let frame = |dir: Direction| {
        let mut ui = Ui::new(UiDesc::default());
        ui.set_direction(dir);
        let vp = Viewport {
            width: 400.0,
            height: 200.0,
            dpr: 1.0,
            ..Viewport::default()
        };
        ui.begin(InputFrame::default(), vp);
        ui.row(
            "Bar",
            Element {
                width: Size::grow(),
                gap: 10.0,
                ..Element::default()
            },
            |ui| {
                ui.label("First", "one", LabelDesc::default());
                ui.label("Second", "two", LabelDesc::default());
            },
        );
        ui.end().clone()
    };
    let ltr = frame(Direction::Ltr);
    let rtl = frame(Direction::Rtl);
    let x = |f: &UiFrame, id: &str| f.element(id).unwrap().x;
    let w = |f: &UiFrame, id: &str| f.element(id).unwrap().width;
    assert!(x(&ltr, "First") < x(&ltr, "Second"));
    assert!(
        x(&rtl, "First") > x(&rtl, "Second"),
        "right to left, the first element is rightmost"
    );
    for id in ["First", "Second", "Bar"] {
        assert!(
            (x(&rtl, id) - (400.0 - x(&ltr, id) - w(&ltr, id))).abs() < 0.01,
            "{id} is mirrored about the centre"
        );
    }
}

#[test]
fn a_scrollers_content_shows_where_it_is_mirrored_to() {
    let frame = |dir: Direction| {
        let mut ui = Ui::new(UiDesc::default());
        ui.set_direction(dir);
        let vp = Viewport {
            width: 400.0,
            height: 200.0,
            dpr: 1.0,
            ..Viewport::default()
        };
        ui.begin(InputFrame::default(), vp);
        // A sideways scroller in the row's last quarter: its box is far from its mirror image.
        ui.row(
            "Bar",
            Element {
                width: Size::grow(),
                ..Element::default()
            },
            |ui| {
                ui.row(
                    "Pad",
                    Element {
                        width: Size::Fixed(300.0),
                        height: Size::Fixed(20.0),
                        ..Element::default()
                    },
                    |_| {},
                );
                ui.scroll(
                    "Strip",
                    ScrollDesc {
                        width: Size::Fixed(100.0),
                        height: Size::Fixed(20.0),
                        axes: ScrollAxes::Horizontal,
                        ..ScrollDesc::default()
                    },
                    |ui| ui.label("Inside", "tab", LabelDesc::default()),
                );
            },
        );
        ui.end().clone()
    };
    for dir in [Direction::Ltr, Direction::Rtl] {
        let f = frame(dir);
        let strip = f.element("Strip").unwrap();
        let inside = f
            .element("Inside")
            .unwrap_or_else(|| panic!("{dir:?}: the scroller's content is on screen"));
        assert!(
            inside.x >= strip.x - 0.01 && inside.x + inside.width <= strip.x + strip.width + 0.01,
            "{dir:?}: the content sits in its scroller ({} in {}..{})",
            inside.x,
            strip.x,
            strip.x + strip.width
        );
    }
}

/// Devanagari through rustybuzz: a conjunct (स + ् + त) becomes fewer glyphs
/// than characters, the vowel sign ि is drawn before the consonant it
/// follows in the text, and the run keeps its logical indices.
#[test]
fn devanagari_forms_conjuncts_and_reorders_its_vowel_signs() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/ui-kit/assets/fonts/NotoSansDevanagari-subset.ttf"
    );
    let file = FontFile::new(Arc::new(
        std::fs::read(path).expect("the kit's Devanagari font"),
    ))
    .expect("a font rustybuzz reads");
    let m = FontMetrics::fixed(0.5).with_file(Some(file));
    let word = "नमस्ते";
    let g = m.visual(word, Direction::Ltr, 0.0);
    assert!(g.iter().all(|g| g.ch as u32 >= GLYPH_PUA), "shaped: {g:?}");
    assert!(
        g.len() < word.chars().count(),
        "the conjunct स्त is one glyph or fewer than its three characters: {} glyphs",
        g.len()
    );
    assert!(!g.iter().any(|g| g.rtl));
    // हिन्दी: the vowel sign ि follows ह in the text and is drawn before it,
    // both in ह's cluster (logical index 0).
    let hi = m.visual("हिन्दी", Direction::Ltr, 0.0);
    let ha = m.visual("ह", Direction::Ltr, 0.0)[0].ch;
    assert_eq!((hi[0].index, hi[1].index), (0, 0), "{hi:?}");
    assert_ne!(hi[0].ch, ha, "the sign comes first");
    assert_eq!(hi[1].ch, ha, "then ह");
}

/// An Arabic line with a full stop, in a font cooked as og-cook cooks one:
/// its atlas holds the shaped forms of the Arabic words and the full stop
/// only under its code point (the stop is no shaped script's). Laid out as
/// a label and as rich text, in a box wide enough for it and in a narrow
/// one, every line is drawn shaped and as wide as the layout measured it,
/// inside its box: the line was drawn unshaped (its letters apart, a third
/// wider) because shaping it whole needed the stop's glyph as a shaped form,
/// while rich text measured it shaped word by word.
#[test]
fn an_arabic_line_is_drawn_as_wide_as_it_was_laid_out() {
    use og_render::{Glyph, MsdfFont, RenderScene};
    const LINE: &str = "تُفتح البوابة عند الفجر. هل أنت مستعد للعبور؟";
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/ui-kit/assets/fonts/NotoSansArabic-subset.ttf"
    );
    let file = FontFile::new(Arc::new(
        std::fs::read(path).expect("the kit's Arabic font"),
    ))
    .expect("a font rustybuzz reads");
    let glyph = |c: char, advance: f32| Glyph {
        codepoint: c,
        advance,
        plane_bounds: [0.0, -0.2, advance, 0.8],
        atlas_bounds: [0.0, 0.0, 1.0, 1.0],
    };
    // The code points (ASCII and the line's own) and the shaped forms of
    // its words alone, without their full stop, and of the space.
    let mut glyphs: Vec<Glyph> = (' '..='~')
        .chain(LINE.chars())
        .filter_map(|c| file.advance(c).map(|a| glyph(c, a)))
        .collect();
    for word in LINE.split(' ').chain([" "]) {
        let word = word.trim_end_matches('.');
        for g in file.shape(word, true, &|_| true).expect("shaped").iter() {
            glyphs.push(glyph(g.ch, g.advance));
        }
    }
    let mut scene = RenderScene::new();
    let atlas = scene.add_texture(og_render::TextureData {
        width: 1,
        height: 1,
        rgba: vec![255; 4],
        color_space: og_render::ColorSpace::Linear,
        mips: og_render::Mips::None,
    });
    let msdf = MsdfFont {
        atlas,
        distance_range: 4.0,
        line_height: 1.2,
        glyphs,
        kerning: Vec::new(),
    };
    let metrics = FontMetrics::from_msdf(&msdf).with_file(Some(file));
    let font = UiFont {
        handle: Some(scene.add_font(msdf)),
        metrics,
    };
    // The whole line, shaped: none of its letters drawn as written.
    // The line as rich text measures it, word by word, with a tenth to
    // spare: drawn unshaped it was a third wider.
    let size = Theme::default().type_ramp.body.size;
    let words: f32 = LINE
        .split(' ')
        .map(|w| font.metrics.measure(w, size))
        .sum::<f32>()
        + font.metrics.measure(" ", size) * (LINE.split(' ').count() - 1) as f32;
    for (width, lines) in [(words * 1.1, 1), (130.0, 2)] {
        let mut ui = Ui::new(UiDesc {
            fonts: Fonts {
                body: font.clone(),
                ..Fonts::default()
            },
            ..UiDesc::default()
        });
        ui.set_direction(Direction::Rtl);
        ui.begin(
            InputFrame::default(),
            Viewport {
                width: 480.0,
                height: 400.0,
                dpr: 1.0,
                ..Viewport::default()
            },
        );
        ui.column(
            "Box",
            Element {
                width: Size::Fixed(width),
                ..Element::default()
            },
            |ui| {
                ui.label(
                    "Label",
                    LINE,
                    LabelDesc {
                        width: Size::grow(),
                        ..LabelDesc::default()
                    },
                );
                ui.rich_text(
                    "Rich",
                    LINE,
                    RichTextDesc {
                        width: Size::grow(),
                        ..RichTextDesc::default()
                    },
                );
            },
        );
        let frame = ui.end().clone();
        for id in ["Rich", "Label"] {
            let e = frame.element(&format!("Box/{id}")).expect(id);
            assert!(
                e.width <= width + 0.01,
                "{id} fits its {width} px box: {}",
                e.width
            );
            // Its lines: each text command's glyph extent, inside the box.
            let drawn: Vec<(f32, f32, f32)> = frame
                .items
                .iter()
                .filter_map(|i| match &i.cmd {
                    DrawCmd::Text {
                        x,
                        baseline,
                        size,
                        text,
                        placements,
                        ..
                    } if !text.trim().is_empty()
                        && *baseline > e.y
                        && *baseline < e.y + e.height =>
                    {
                        let w = placements.iter().map(|p| p.advance).sum::<f32>() * size;
                        Some((*x, *x + w, *baseline))
                    }
                    _ => None,
                })
                .collect();
            let mut baselines: Vec<f32> = drawn.iter().map(|d| d.2).collect();
            baselines.dedup_by(|a, b| (*a - *b).abs() < 0.5);
            assert_eq!(
                baselines.len(),
                lines,
                "{id} in {width} px draws {lines} shaped line(s): {drawn:?}"
            );
            for (x0, x1, _) in &drawn {
                assert!(
                    *x0 >= e.x - 0.5 && *x1 <= e.x + e.width + 0.5,
                    "{id} in {width} px draws {x0:.1}..{x1:.1}, inside its {:.1}..{:.1}",
                    e.x,
                    e.x + e.width
                );
            }
        }
    }
}
