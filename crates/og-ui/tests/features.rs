//! OpenType features of a type step (docs/ui.md §15.5.4): a run with
//! features is shaped with its font file, so it draws the feature's glyphs,
//! and it is measured exactly as it is drawn.
//!
//! Ways it could fail, each covered below:
//! - the features never reach the shaper, so tabular figures keep their
//!   proportional advances → `tabular_figures_share_one_advance`
//! - measuring and drawing take different paths (one shaped, one not, or
//!   one tracked and the other not), so a small-caps label is cut or
//!   overflows its box → `a_small_caps_line_measures_as_it_draws`
//! - a face that cannot shape `tnum` (no file, or no feature) keeps its
//!   proportional figures, so a timer changes width as it ticks →
//!   `tabular_figures_hold_a_width_without_the_feature`

use og_render::{Glyph, MsdfFont, TextureHandle};
use og_ui::shape::{Features, FontFile, GLYPH_PUA};
use og_ui::*;
use std::sync::Arc;

/// A kit font as the cook makes it with `features`: its printable ASCII by
/// code point and every glyph by id (U+F0000 + id), advances from `hmtx`.
pub(crate) fn font(file: &str) -> UiFont {
    font_at(&format!("tests/fixtures/ui-text/assets/fonts/{file}"))
}

/// [`font`] for a font at `path` from the repository's root.
fn font_at(path: &str) -> UiFont {
    let path = format!("{}/../../{path}", env!("CARGO_MANIFEST_DIR"));
    let bytes = std::fs::read(&path).expect("a kit font");
    let face = ttf_parser::Face::parse(&bytes, 0).expect("a font ttf-parser reads");
    let em = f32::from(face.units_per_em());
    let advance = |g: ttf_parser::GlyphId| f32::from(face.glyph_hor_advance(g).unwrap_or(0)) / em;
    let glyph = |codepoint: char, advance: f32| Glyph {
        codepoint,
        advance,
        plane_bounds: [0.0, 0.0, advance, 0.7],
        atlas_bounds: [0.0, 0.0, 1.0, 1.0],
    };
    let mut glyphs: Vec<Glyph> = (0x20u8..0x7f)
        .map(char::from)
        .filter_map(|c| Some(glyph(c, advance(face.glyph_index(c)?))))
        .collect();
    glyphs.extend((0..face.number_of_glyphs()).filter_map(|g| {
        Some(glyph(
            char::from_u32(GLYPH_PUA + u32::from(g))?,
            advance(ttf_parser::GlyphId(g)),
        ))
    }));
    let msdf = MsdfFont {
        atlas: TextureHandle::from_slot(1).expect("a handle"),
        distance_range: 4.0,
        line_height: 1.2,
        glyphs,
        kerning: Vec::new(),
    };
    let file = FontFile::new(Arc::new(bytes)).expect("a font rustybuzz reads");
    // No render handle: the pieces are laid out, not drawn.
    UiFont {
        handle: None,
        metrics: FontMetrics::from_msdf(&msdf).with_file(Some(file)),
    }
}

/// A pixel font (`font { pixel 12 }`) is set at whole multiples of its
/// grid (the typography audit, fix 5): a 14 px body label at 12, a 20 px
/// one at 24, so each of its pixels is a whole CSS pixel; its tracking is
/// whole pixels. An outline font keeps the step's size.
#[test]
fn a_pixel_font_is_set_at_multiples_of_its_grid() {
    let width = |metrics: FontMetrics, size: f32, tracking: f32| {
        let mut ui = Ui::new(UiDesc::default());
        let f = UiFont {
            handle: None,
            metrics,
        };
        let mut fonts = Fonts {
            body: f,
            ..Fonts::default()
        };
        fonts.link();
        ui.set_fonts(fonts);
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.label(
            "N",
            "abcd",
            LabelDesc {
                nowrap: true,
                step: Some(TypeStep {
                    size,
                    line_height: size * 1.25,
                    tracking_em: tracking,
                    cjk_line_height: size * 1.25,
                    cjk_tracking_em: 0.0,
                    features: Features::NONE,
                    ..ui.theme().type_ramp.body
                }),
                ..LabelDesc::default()
            },
        );
        ui.end();
        ui.frame().element("N").expect("the label").width
    };
    let pixel = || FontMetrics::fixed(0.5).with_pixel(12);
    // Half an em a character: 6 px at 12, 12 px at 24.
    assert!((width(pixel(), 14.0, 0.0) - 24.0).abs() < 0.01);
    assert!((width(pixel(), 20.0, 0.0) - 48.0).abs() < 0.01);
    assert!(
        (width(pixel(), 4.0, 0.0) - 24.0).abs() < 0.01,
        "never under one grid"
    );
    // 0.02 em is a quarter pixel of a 12-pixel em: none; 0.1 em is one.
    assert!((width(pixel(), 12.0, 0.02) - 24.0).abs() < 0.01);
    assert!((width(pixel(), 12.0, 0.1) - 28.0).abs() < 0.01);
    assert!((width(FontMetrics::fixed(0.5), 14.0, 0.0) - 28.0).abs() < 0.01);
}

/// A style's `fonts { displayMin }`: display-role text under the size is
/// set in the body face, at and over it in the display face (wasteland's
/// stencil for titles, its body face for 14 px eyebrows).
#[test]
fn display_text_under_the_styles_display_min_takes_the_body_face() {
    let pack = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/styles/storybook/style.ogt"
    ))
    .expect("the storybook seed");
    let text = pack.replacen("fonts {\n", "fonts {\n  displayMin 18\n", 1);
    let file = og_ui::style_file::parse_style(&text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(file.theme.art.display_min, 18.0);
    let mut narrow = font("Inter-Display.ttf");
    narrow.metrics = narrow.metrics.clone().with_file(None);
    let mut wide = font("EBGaramond-Medium.ttf");
    wide.metrics = wide.metrics.clone().with_file(None);
    let width = |min: f32, size: f32, role: FontRole| {
        let mut ui = Ui::new(UiDesc::default());
        let mut theme = Theme::default();
        theme.art.display_min = min;
        ui.set_theme(theme);
        let mut fonts = Fonts {
            body: narrow.clone(),
            display: Some(wide.clone()),
            ..Fonts::default()
        };
        fonts.link();
        ui.set_fonts(fonts);
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        let mut step = ui.type_step(og_ui::tokens::TypeRole::Body);
        step.size = size;
        ui.label(
            "N",
            "SALVAGE YARD",
            LabelDesc {
                nowrap: true,
                font: role,
                step: Some(step),
                ..LabelDesc::default()
            },
        );
        ui.end();
        ui.frame().element("N").expect("the label").width
    };
    // Wrong: the threshold ignored (14 px display drawn in the display
    // face), or applied at and over it, or to the other roles.
    assert_eq!(
        width(18.0, 14.0, FontRole::Display),
        width(0.0, 14.0, FontRole::Body),
        "under: the body face"
    );
    assert_ne!(
        width(18.0, 14.0, FontRole::Display),
        width(0.0, 14.0, FontRole::Display),
        "the faces differ"
    );
    assert_eq!(
        width(18.0, 18.0, FontRole::Display),
        width(0.0, 18.0, FontRole::Display),
        "at it: the display face"
    );
}
