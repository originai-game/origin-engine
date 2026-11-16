//! CSS font selection and font properties on document elements (CSS
//! Fonts 4 and 5): which face an element's text is drawn in, at which
//! weight, width and slant, and what the draw synthesises where the
//! family lacks a face, checked on the text the frame draws.
//!
//! Ways these could fail, each covered below:
//! - `bolder` or `lighter` read against the wrong inherited weight, a
//!   weight between faces matched on the wrong side, or a plain text at a
//!   weight its family lacks drawn regular (or thickened when
//!   `font-synthesis-weight: none`) →
//!   `font_weight_picks_the_face_and_synthesises_a_missing_bold`
//! - a width matched to the nearest face instead of CSS's order (at or
//!   under normal the narrower faces first, over it the wider), a width
//!   not inherited, `font-width` not read as `font-stretch`, the
//!   `@font-face` width descriptor dropped, or a text measured in another
//!   face than it draws in →
//!   `font_stretch_takes_the_family_face_of_the_nearest_width_in_css_order`
//! - a variable face drawn at its default location (its atlas) whatever
//!   the axes ask, an axis setting dropped, overridden in the wrong order
//!   or not inherited, `opsz` not following the size (or following it
//!   under `font-optical-sizing: none`), a bold the `wght` axis reaches
//!   synthesised on top of it, or an instance drawn with the default
//!   location's font →
//!   `a_variable_face_is_drawn_at_the_axes_css_sets`

use crate::support::Doc;
use og_draw::FontHandle;
use og_ui::*;

const NOTO: &str = "templates/fonts/latin/NotoSans-Latin.ttf";
const INTER: &str = "tests/fixtures/ui-text/assets/fonts/Inter-Display.ttf";
const GARAMOND: &str = "tests/fixtures/ui-text/assets/fonts/EBGaramond-Medium.ttf";
/// A variable font whose axes each move one glyph's advance (1000 units
/// to the em; `fixtures/make-axes-font.py`): A by `wght` (100, 400, 900 →
/// 500, 600, 1000), B by `wdth` (50, 100, 200 → 300, 600, 900), C by
/// `opsz` (8, 14, 72 → 700, 600, 400), D by `GRAD` (-100, 0, 100 → 500,
/// 600, 800); E, 500, by none.
const AXES: &str = "crates/og-ui/tests/fixtures/axes-test.ttf";

/// The font at `path` (from the repository's root), shaped from its file,
/// drawn under handle `slot`.
fn face(path: &str, slot: usize) -> UiFont {
    let mut font = crate::features::font_at(path);
    font.handle = Some(FontHandle::from_index(slot));
    font
}

/// One `@font-face` of `family` at `weight`.
fn family(family: &str, weight: (u16, u16), font: UiFont) -> FamilyFace {
    FamilyFace {
        family: family.into(),
        document: "t".into(),
        weight,
        font,
        ..FamilyFace::default()
    }
}

/// The style's thickening for every role (`textWeight`, device pixels).
const TEXT_WEIGHT: f32 = 0.5;

/// A document of `body` under `css`, 400 px wide, its body in Noto Sans
/// (handle 1) and its `@font-face` families `faces`, in a style that
/// thickens text by [`TEXT_WEIGHT`].
fn doc(faces: Vec<FamilyFace>, body: &str, css: &str) -> Doc {
    let mut theme = Theme::default();
    theme.art.text_weights = [TEXT_WEIGHT; 3];
    let declared: String = faces
        .iter()
        .map(|f| {
            format!(
                "@font-face {{ font-family: \"{}\"; src: url(f.ttf); }}\n",
                f.family
            )
        })
        .collect();
    let mut fonts = Fonts {
        body: face(NOTO, 1),
        families: faces,
        ..Fonts::default()
    };
    fonts.link();
    let ui = Ui::new(UiDesc {
        theme,
        fonts,
        ..UiDesc::default()
    });
    let text = format!(
        "<meta name=\"component\" content=\"t\">\n<div><div id=\"Root\">{body}</div></div>\n<style>{declared}#Root {{ width: 400px; }} p {{ margin: 0; }} {css}</style>"
    );
    let d = Doc::with_ui(ui, &text);
    assert!(
        d.ui.frame().warnings.is_empty(),
        "{:?}",
        d.ui.frame().warnings
    );
    d
}

/// A text command drawn inside element `id`: its font, x and thickening.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Drawn {
    font: FontHandle,
    x: f32,
    weight: f32,
}

/// The text commands drawn inside element `id`, left to right.
fn drawn(d: &Doc, id: &str) -> Vec<Drawn> {
    let r = d.get(id);
    let mut out: Vec<Drawn> = d
        .ui
        .frame()
        .items
        .iter()
        .filter_map(|it| match it.cmd.unstyled() {
            DrawCmd::Text {
                font,
                x,
                bounds,
                weight,
                ..
            } if (r.y..r.y + r.height).contains(&(bounds.y + bounds.height / 2.0)) => Some(Drawn {
                font: *font,
                x: *x,
                weight: *weight,
            }),
            _ => None,
        })
        .collect();
    out.sort_by(|a, b| a.x.total_cmp(&b.x));
    out
}

/// CSS Fonts 4 §2.2 and §5.2: a numeric weight picks the family's face
/// nearest it (from 500 up heavier first, below 400 lighter first);
/// `bolder` and `lighter` step from the inherited weight by §2.2's table
/// (400 → 700, 700 → 400). A family whose face is lighter than 600 draws
/// a weight of 600 or more in it thickened, twice, one device pixel
/// apart (§7.1, the line browsers draw), unless
/// `font-synthesis-weight: none`. The synthesised bold replaces the
/// style's `textWeight` thickening, so plain and rich text draw it alike.
#[test]
fn font_weight_picks_the_face_and_synthesises_a_missing_bold() {
    let (regular, bold) = (face(NOTO, 2), face(INTER, 3));
    let (r, b) = (regular.handle.unwrap(), bold.handle.unwrap());
    let d = doc(
        vec![
            family("Pair", (400, 400), regular.clone()),
            family("Pair", (700, 700), bold),
            family("Solo", (400, 400), regular),
        ],
        r#"<p id="N">Ab</p><p id="B">Ab</p><p id="W650">Ab</p><p id="W300">Ab</p><p id="W550">Ab</p>
<div id="Up"><p id="Bolder">Ab</p></div><div id="Down"><p id="Lighter">Ab</p></div>
<p id="S400">Ab</p><p id="S500">Ab</p><p id="S600">Ab</p><p id="S700">Ab</p><p id="Snone">Ab</p>
<p id="SRich">Ab</p>"#,
        r#"#Root { font-family: "Pair"; }
#B { font-weight: bold; } #W650 { font-weight: 650; } #W300 { font-weight: 300; }
#W550 { font-weight: 550; }
#Up { font-weight: 400; } #Bolder { font-weight: bolder; }
#Down { font-weight: 700; } #Lighter { font-weight: lighter; }
[id^=S] { font-family: "Solo"; } #S400 { font-weight: 400; } #S500 { font-weight: 500; }
#S600 { font-weight: 600; } #S700, #Snone { font-weight: 700; }
#Snone { font-synthesis-weight: none; }
#SRich { font-weight: 600; text-decoration: underline; }"#,
    );
    let face_of = |id: &str| {
        let t = drawn(&d, id);
        assert!(!t.is_empty(), "#{id} draws");
        t[0].font
    };
    for (id, want) in [
        ("N", r),
        ("B", b),
        ("W650", b),
        ("W300", r),
        ("W550", b),
        ("Bolder", b),
        ("Lighter", r),
    ] {
        assert_eq!(face_of(id), want, "#{id}");
    }
    // The family's real faces draw as they are, in the style's thickening.
    let plain = drawn(&d, "N")[0].weight;
    assert!(plain > 0.0, "the style's textWeight: {plain}");
    assert_eq!(drawn(&d, "B").len(), 1, "a real bold face is drawn once");
    assert_eq!(drawn(&d, "B")[0].weight, plain);
    // A rich text (one with a decoration) at 600 synthesises its bold too,
    // with no style thickening: the plain texts' bold must match it.
    let rich = match drawn(&d, "SRich").as_slice() {
        [a, b] if a.font == r && b.font == r && a.weight == b.weight => a.weight,
        t => panic!("a rich 600 span draws twice in the regular face: {t:?}"),
    };
    let synthesised = |id: &str| {
        let t = drawn(&d, id);
        assert!(t.iter().all(|t| t.font == r), "#{id} in the regular face");
        match t.as_slice() {
            [one] => {
                assert_eq!(one.weight, plain, "#{id}");
                false
            }
            [a, b] => {
                // The synthesised bold in place of the style's thickening,
                // not on top of it: what a rich text's bold span draws.
                assert_eq!(a.weight, b.weight, "#{id}: {t:?}");
                assert_eq!(a.weight, rich, "#{id}: {t:?}");
                assert!(
                    (b.x - a.x - 1.0).abs() < 1e-3,
                    "one device pixel apart: {t:?}"
                );
                true
            }
            _ => panic!("#{id}: {t:?}"),
        }
    };
    assert!(!synthesised("S400") && !synthesised("S500"));
    assert!(synthesised("S600") && synthesised("S700"));
    assert!(!synthesised("Snone"), "font-synthesis-weight: none");
}

/// CSS Fonts 4 §5.2 step 4a: a face whose width range holds the text's
/// width; else, at or under 100 %, the narrower faces nearest first and
/// then the wider; over 100 %, the wider then the narrower. `font-width`
/// is `font-stretch` (Fonts 5), the keywords are §2.4's percentages, and
/// the width is inherited. Each face is a different font here, so which
/// one a text takes is the handle it is drawn with, and the layout
/// measures it in that face.
#[test]
fn font_stretch_takes_the_family_face_of_the_nearest_width_in_css_order() {
    let at = |path, slot, width: u16| FamilyFace {
        stretch: (width, width),
        ..family("Wide", (400, 400), face(path, slot))
    };
    let (normal, condensed, expanded) =
        (at(NOTO, 4, 1000), at(INTER, 5, 750), at(GARAMOND, 6, 1250));
    let (n, c, e) = (
        normal.font.handle.unwrap(),
        condensed.font.handle.unwrap(),
        expanded.font.handle.unwrap(),
    );
    let d = doc(
        vec![
            normal,
            condensed.clone(),
            expanded,
            FamilyFace {
                family: "Narrow".into(),
                ..condensed
            },
        ],
        r#"<p id="N">mmmmmmmm</p><p id="C">mmmmmmmm</p><p id="P95">Ab</p><p id="P105">Ab</p><p id="Ultra">Ab</p>
<p id="Width">Ab</p><div id="Up"><p id="Inherit">Ab</p></div><p id="Narrow">mmmmmmmm</p>"#,
        r#"#Root { font-family: "Wide"; } p { width: max-content; }
#C { font-stretch: condensed; } #P95 { font-stretch: 95%; } #P105 { font-stretch: 105%; }
#Ultra { font-stretch: ultra-condensed; } #Width { font-width: expanded; }
#Up { font-stretch: 75%; } #Narrow { font-family: "Narrow"; }"#,
    );
    for (id, want) in [
        ("N", n),
        ("C", c),
        // Nearer the normal face, but narrower ones come first under 100 %.
        ("P95", c),
        // Nearer the normal face, but wider ones come first over 100 %.
        ("P105", e),
        // Nothing narrower than 50 %: the nearest wider.
        ("Ultra", c),
        ("Width", e),
        ("Inherit", c),
    ] {
        let t = drawn(&d, id);
        assert!(!t.is_empty(), "#{id} draws");
        assert!(t.iter().all(|t| t.font == want), "#{id}: {t:?}");
    }
    // Measured in the face it draws: the condensed text is as wide as the
    // same text in a family whose only face is that one (Inter's m is
    // 0.87 em, Noto Sans's 0.935).
    let w = |id: &str| d.get(id).width;
    assert_eq!(w("C"), w("Narrow"));
    assert_ne!(w("C"), w("N"));
    // The @font-face descriptors: one width, a range, and `font-width`.
    let sheet = UiDocument::parse(
        "ui/f.svelte",
        r#"<meta name="component" content="f"><div></div>
<style>
@font-face { font-family: A; src: url(a.ttf); font-stretch: condensed; }
@font-face { font-family: A; src: url(b.ttf); font-stretch: 125% 75%; }
@font-face { font-family: A; src: url(c.ttf); font-width: semi-expanded; }
@font-face { font-family: A; src: url(d.ttf); }
</style>"#,
    )
    .unwrap_or_else(|e| panic!("{e:?}"))
    .sheet;
    let widths: Vec<Option<(f32, f32)>> = sheet.font_faces.iter().map(|f| f.stretch).collect();
    assert_eq!(
        widths,
        [
            Some((75.0, 75.0)),
            Some((75.0, 125.0)),
            Some((112.5, 112.5)),
            // `auto`: a variable font's wdth range, else normal.
            None
        ]
    );
}

/// CSS Fonts 4 §7.2: a variable face is drawn at the instance its axes
/// ask for: `wght` from `font-weight` and `wdth` from `font-stretch`,
/// each clamped to the face's declared range, `opsz`
/// the size in CSS px under `font-optical-sizing: auto` (the initial
/// value) and its default under `none`, then `font-variation-settings`,
/// inherited, which overrides them. The instance is measured from its
/// file (the atlas holds the default location's glyphs) and drawn with a
/// font of its own, which the host makes from the frame it first appears.
/// A bold its `wght` axis reaches is not synthesised.
#[test]
fn a_variable_face_is_drawn_at_the_axes_css_sets() {
    let axes = face(AXES, 7);
    let file = axes.metrics.file.clone().expect("its file");
    let base = axes.handle.unwrap();
    let mut d = doc(
        vec![
            FamilyFace {
                stretch: (500, 2000),
                ..family("Axes", (100, 900), axes.clone())
            },
            family("Fixed", (400, 400), axes),
        ],
        r#"<p id="E">E</p><p id="A400">A</p><p id="A650">A</p><p id="A700">A</p><p id="AFixed">A</p><p id="B75">B</p>
<p id="C43">C</p><p id="C24">C</p><p id="CNone">C</p><p id="D50">D</p><p id="AOver">A</p>
<div id="Up"><p id="DInherit">D</p></div><div id="Normal"><p id="DNormal">D</p></div>"#,
        r#"#Root { font-family: "Axes"; } p { width: max-content; font-size: 50px; }
#A400 { font-optical-sizing: none; } #A650 { font-weight: 650; }
#A700 { font-weight: 700; } #AFixed { font-family: "Fixed"; font-weight: 700; }
#B75 { font-stretch: 75%; } #C43 { font-size: 43px; } #C24 { font-size: 24px; }
#CNone { font-size: 43px; font-optical-sizing: none; }
#D50 { font-variation-settings: "GRAD" 50; }
#AOver { font-weight: 900; font-variation-settings: "wght" 100; }
#Up { font-variation-settings: "GRAD" 100; }
#Normal { font-variation-settings: "GRAD" 100; } #DNormal { font-variation-settings: normal; }"#,
    );
    // The host makes each instance's font, as og-runtime does between
    // frames; the next frame draws with it.
    // Until then an instanced text measures but draws nothing (never the
    // default location's glyphs).
    assert!(drawn(&d, "A650").is_empty());
    let made = file.take_instances();
    assert!(!made.is_empty(), "instances made");
    for (i, f) in made.iter().enumerate() {
        f.set_handle(FontHandle::from_index(100 + i));
    }
    d.step(InputFrame::default());
    // Each text's advance, `want` em, read from its laid-out width (whole
    // pixels).
    let near = |id: &str, want: f32| {
        let size = match id {
            "C43" | "CNone" => 43.0,
            "C24" => 24.0,
            _ => 50.0,
        };
        let got = d.get(id).width;
        assert!(
            (got - want * size).abs() <= 0.5,
            "#{id}: {got} px, want {} px",
            want * size
        );
    };
    near("E", 0.5);
    near("A400", 0.6);
    near("A650", 0.8);
    near("A700", 0.84);
    // A face declared 400 is drawn at wght 400 whatever the weight.
    near("AFixed", 0.6);
    near("B75", 0.45);
    near("C43", 0.5);
    near("C24", 0.6 - 0.2 * 10.0 / 58.0);
    near("CNone", 0.6);
    near("D50", 0.7);
    near("AOver", 0.5);
    near("DInherit", 0.8);
    near("DNormal", 0.6);
    // Drawn with the instance's font, never the default location's.
    for id in ["A650", "A700", "B75", "C43", "D50", "DInherit"] {
        let t = drawn(&d, id);
        assert!(!t.is_empty(), "#{id} draws");
        assert!(
            t.iter().all(|t| t.font != base && t.font.index() >= 100),
            "#{id}: {t:?}"
        );
    }
    // The instance renders its own glyphs (its atlas has none): the host
    // is asked for A's glyph of it.
    let a650 = drawn(&d, "A650")[0].font;
    let instance = made
        .iter()
        .find(|f| f.handle() == Some(a650))
        .expect("A650's");
    let a = ttf_parser::Face::parse(file.bytes(), 0)
        .ok()
        .and_then(|f| f.glyph_index('A'))
        .expect("A");
    assert_eq!(instance.take_wanted(), [og_draw::GlyphKey::Id(a.0)]);
    // At the default location (no axis moved): the face itself.
    assert!(drawn(&d, "A400").iter().all(|t| t.font == base));
    // A 700 the face's wght range reaches is drawn once; one in a face
    // declared 400 is synthesised, thickened twice.
    assert_eq!(drawn(&d, "A700").len(), 1, "no synthesised bold");
    assert_eq!(drawn(&d, "AFixed").len(), 2, "a synthesised bold");
}
