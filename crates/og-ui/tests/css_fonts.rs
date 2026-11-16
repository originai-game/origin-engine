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
//! - a COLRv0 layer painted in the wrong palette entry, a COLRv1 gradient
//!   painted flat or reversed, or a foreground layer painted in a palette
//!   colour → `colr_glyphs_paint_their_layers_and_gradients_in_colour`
//! - `font-palette` ignored, `light` or `dark` taking the wrong CPAL
//!   palette, an `@font-palette-values` applied to a family it does not
//!   list, its overrides or base dropped, an index past the font's
//!   palettes not falling back to the first, or a palette not inherited →
//!   `font_palette_draws_a_colour_face_in_the_palette_css_picks`
//! - `palette-mix()` mixing in the wrong space or ignoring its
//!   percentages, `normal` in a mix not the font's first palette, a
//!   `font-palette` animation flipping at 50% instead of mixing in OKLab,
//!   or an animated palette's text blank while each frame's font is made →
//!   `font_palettes_mix_and_animate_entry_by_entry`
//! - an alternates function ignoring `@font-feature-values`, taking a
//!   rule for another family or an earlier definition of a name, a
//!   styleset of two values setting one, a name nobody defines not
//!   ignored, the shorthand dropping it, or `font-feature-settings` not
//!   applying after it → `font_variant_alternates_take_the_feature_values_named`
//! - `lang` not reaching shaping, not inherited or not cleared by an empty
//!   `lang`, `font-language-override` ignored or not winning over `lang`,
//!   or a Parley paragraph shaped without its language →
//!   `font_language_override_and_lang_pick_the_fonts_local_forms`
//! - `ui-monospace` or `ui-rounded` drawn in a face standing in for the
//!   role where the style has none of its own instead of going on down the
//!   list, `ui-serif` or `math` taken as the body face ahead of a later
//!   family, or `system-ui` not the style's UI face →
//!   `generic_families_take_the_style_faces_that_suit_them`

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
/// A colour font (`fixtures/make-palette-font.py`), each glyph an em wide
/// and 0.8 tall: A a COLRv0 square in entry 0 under a right half in entry
/// 1, B a COLRv1 linear gradient from entry 0 at its left to entry 1 at its
/// right, C a COLRv0 layer in the text's colour. Three CPAL palettes:
/// 0 red and green, 1 blue and yellow (marked light), 2 white and black
/// (marked dark).
const PALETTE: &str = "crates/og-ui/tests/fixtures/palette-test.ttf";

/// The sRGB pixel at (`x`, `y`) em from the pen of a colour glyph's image
/// painted at `em_px`.
fn pixel(image: &og_ui::colr::ColorImage, rgba: &[u8], em_px: f32, x: f32, y: f32) -> [u8; 4] {
    let col = ((x - image.plane[0]) * em_px) as usize;
    let row = ((image.plane[3] - y) * em_px) as usize;
    let o = (row * image.width as usize + col) * 4;
    rgba[o..o + 4].try_into().unwrap()
}

/// Channels within 2 of `want`'s.
fn near_rgb(got: [u8; 4], want: [u8; 3]) -> bool {
    got[..3].iter().zip(want).all(|(&g, w)| g.abs_diff(w) <= 2) && got[3] == 255
}

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
    doc_in(
        Fonts {
            body: face(NOTO, 1),
            ..Fonts::default()
        },
        faces,
        body,
        css,
    )
}

/// [`doc`] with the style's role faces `fonts`.
fn doc_in(fonts: Fonts, faces: Vec<FamilyFace>, body: &str, css: &str) -> Doc {
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
        families: faces,
        ..fonts
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

/// COLRv0 (OpenType 1.8's layers, each a glyph in one palette entry, the
/// later on top) and COLRv1 (paint graphs: here a linear gradient between
/// two palette entries, interpolated in sRGB as browsers paint it) draw a
/// colour font's glyphs in colour; a layer in entry 0xFFFF takes the
/// text's colour, painted black and white so the renderer mixes it.
#[test]
fn colr_glyphs_paint_their_layers_and_gradients_in_colour() {
    let bytes = std::fs::read(format!("{}/../../{PALETTE}", env!("CARGO_MANIFEST_DIR"))).unwrap();
    let em = 32.0;
    let palette = [[1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 1.0]];
    let paint = |c: char| {
        og_ui::colr::render_runtime(&bytes, og_draw::GlyphKey::Char(c), em, &palette)
            .unwrap_or_else(|| panic!("{c} paints"))
    };
    // v0: entry 0 under, entry 1 over the right half.
    let a = paint('A');
    let at = |g: &og_ui::colr::RuntimeColor, x| pixel(&g.image, &g.image.rgba, em, x, 0.4);
    assert!(near_rgb(at(&a, 0.25), [255, 0, 0]), "{:?}", at(&a, 0.25));
    assert!(near_rgb(at(&a, 0.75), [0, 255, 0]), "{:?}", at(&a, 0.75));
    assert!(!a.image.foreground);
    // v1: red at the left edge to green at the right, half and half between.
    let b = paint('B');
    let (left, middle, right) = (at(&b, 0.05), at(&b, 0.5), at(&b, 0.95));
    assert!(left[0] > 230 && left[1] < 25, "{left:?}");
    assert!(right[1] > 230 && right[0] < 25, "{right:?}");
    assert!(
        (115..=140).contains(&middle[0]) && (115..=140).contains(&middle[1]) && middle[2] == 0,
        "{middle:?}"
    );
    // The text's colour: black, with a white twin.
    let c = paint('C');
    assert!(c.image.foreground);
    assert!(near_rgb(at(&c, 0.5), [0, 0, 0]));
    let white = c.white.as_deref().expect("its white twin");
    assert!(near_rgb(
        pixel(&c.image, white, em, 0.5, 0.4),
        [255, 255, 255]
    ));
    // A glyph without a colour paint is not painted in colour.
    assert!(
        og_ui::colr::render_runtime(&bytes, og_draw::GlyphKey::Char('D'), em, &palette).is_none()
    );
}

/// CSS Fonts 4 §9: `font-palette` draws a colour face in another CPAL
/// palette: `light` and `dark` the first the font marks for that
/// background, a name the last `@font-palette-values` of that name whose
/// `font-family` lists the text's family, from its `base-palette` (an
/// index past the font's palettes: its first) with its `override-colors`
/// (an entry past the palette's ignored); a name no rule for the family
/// has is `normal`. The palette is inherited. A text in another palette
/// is drawn with an instance of its own whose colour glyphs the host
/// paints in it, from the frame after it first appears.
#[test]
fn font_palette_draws_a_colour_face_in_the_palette_css_picks() {
    let pal = face(PALETTE, 8);
    let file = pal.metrics.file.clone().expect("its file");
    let base = pal.handle.unwrap();
    let mut d = doc(
        vec![family("Pal", (400, 400), pal)],
        r#"<p id="N">AB</p><p id="L">AB</p><p id="D">AB</p><p id="Brand">AB</p><p id="Other">AB</p>
<p id="Far">AB</p><div id="Up"><p id="Inherit">AB</p></div><p id="Rich">AB</p>"#,
        r#"#Root { font-family: "Pal"; }
@font-palette-values --brand { font-family: "Pal", Elsewhere; base-palette: 1; override-colors: 1 #808080, 7 red; }
@font-palette-values --brand { font-family: Elsewhere; base-palette: 2; }
@font-palette-values --other { font-family: Elsewhere; base-palette: 2; }
@font-palette-values --far { font-family: Pal; base-palette: 9; }
#L { font-palette: light; } #D { font-palette: dark; } #Brand { font-palette: --brand; }
#Other { font-palette: --other; } #Far { font-palette: --far; } #Up { font-palette: dark; }
#Rich { font-palette: light; text-decoration: underline; }"#,
    );
    assert!(
        drawn(&d, "L").is_empty(),
        "blank until the host made its font"
    );
    let made = file.take_instances();
    for (i, f) in made.iter().enumerate() {
        f.set_handle(FontHandle::from_index(100 + i));
    }
    d.step(InputFrame::default());
    // The palette each text is drawn in (`None`: the cooked face).
    let palette_of = |id: &str| -> Option<Vec<[u8; 4]>> {
        let t = drawn(&d, id);
        assert!(!t.is_empty(), "#{id} draws");
        let font = t[0].font;
        assert!(t.iter().all(|t| t.font == font), "#{id}: {t:?}");
        if font == base {
            return None;
        }
        let instance = made
            .iter()
            .find(|f| f.handle() == Some(font))
            .expect("an instance");
        Some(
            instance
                .palette()
                .expect("in a palette")
                .iter()
                .map(|c| c.map(|v| (v * 255.0).round() as u8))
                .collect(),
        )
    };
    let (red, green) = ([255, 0, 0, 255], [0, 255, 0, 255]);
    let (blue, yellow) = ([0, 0, 255, 255], [255, 255, 0, 255]);
    let (white, black) = ([255, 255, 255, 255], [0, 0, 0, 255]);
    assert_eq!(palette_of("N"), None);
    assert_eq!(palette_of("L"), Some(vec![blue, yellow]));
    assert_eq!(palette_of("D"), Some(vec![white, black]));
    assert_eq!(palette_of("Brand"), Some(vec![blue, [128, 128, 128, 255]]));
    assert_eq!(palette_of("Other"), None);
    assert_eq!(palette_of("Far"), Some(vec![red, green]));
    assert_eq!(palette_of("Inherit"), Some(vec![white, black]));
    assert_eq!(palette_of("Rich"), Some(vec![blue, yellow]));
    // Its colour glyphs paint in it: A's left in entry 0, its right in 1.
    let bytes = file.bytes();
    let light = made
        .iter()
        .find(|f| f.handle() == Some(drawn(&d, "L")[0].font))
        .unwrap();
    let a = og_ui::colr::render_runtime(
        bytes,
        og_draw::GlyphKey::Char('A'),
        32.0,
        light.palette().unwrap(),
    )
    .unwrap();
    assert!(near_rgb(
        pixel(&a.image, &a.image.rgba, 32.0, 0.25, 0.4),
        [0, 0, 255]
    ));
    assert!(near_rgb(
        pixel(&a.image, &a.image.rgba, 32.0, 0.75, 0.4),
        [255, 255, 0]
    ));
}

/// CSS Fonts 4 §2.1.3: `system-ui` and `ui-sans-serif` are the style's UI
/// face (body); `ui-monospace` and `ui-rounded` its own mono and rounded
/// faces; and where the style has no face that suits one (none ever for
/// `ui-serif` and `math`), the family matches nothing and the list goes
/// on, where `monospace`, which always matches, takes body's face.
#[test]
fn generic_families_take_the_style_faces_that_suit_them() {
    let fancy = || family("Fancy", (400, 400), face(GARAMOND, 9));
    let body = r#"<p id="System">Ab</p><p id="Sans">Ab</p><p id="Mono">Ab</p><p id="Round">Ab</p>
<p id="Serif">Ab</p><p id="Math">Ab</p><p id="SerifMono">Ab</p><p id="MathFancy">Ab</p>
<p id="MonoFancy">Ab</p><p id="RoundFancy">Ab</p><p id="Always">Ab</p>"#;
    let css = r#"#System { font-family: system-ui; } #Sans { font-family: ui-sans-serif; }
#Mono { font-family: ui-monospace; } #Round { font-family: ui-rounded; }
#Serif { font-family: ui-serif; } #Math { font-family: math; }
#SerifMono { font-family: ui-serif, ui-monospace; } #MathFancy { font-family: math, "Fancy"; }
#MonoFancy { font-family: ui-monospace, "Fancy"; } #RoundFancy { font-family: ui-rounded, "Fancy"; }
#Always { font-family: monospace, "Fancy"; }"#;
    let face_of = |d: &Doc, id: &str| {
        let t = drawn(d, id);
        assert!(!t.is_empty(), "#{id} draws");
        t[0].font
    };
    let h = FontHandle::from_index;
    // A style with its own mono and rounded faces.
    let styled = doc_in(
        Fonts {
            body: face(NOTO, 1),
            mono: Some(face(INTER, 2)),
            rounded: Some(face(GARAMOND, 3)),
            ..Fonts::default()
        },
        vec![fancy()],
        body,
        css,
    );
    for (id, want) in [
        ("System", 1),
        ("Sans", 1),
        ("Mono", 2),
        ("Round", 3),
        // Nothing suits them: the list ends, at the text's default face.
        ("Serif", 1),
        ("Math", 1),
        ("SerifMono", 2),
        ("MathFancy", 9),
        ("MonoFancy", 2),
        ("RoundFancy", 3),
        ("Always", 2),
    ] {
        assert_eq!(face_of(&styled, id), h(want), "#{id} in the styled fonts");
    }
    // A style with a body face only: the ui- families go on down the list.
    let plain = doc_in(
        Fonts {
            body: face(NOTO, 1),
            ..Fonts::default()
        },
        vec![fancy()],
        body,
        css,
    );
    for (id, want) in [
        ("System", 1),
        ("Mono", 1),
        ("MonoFancy", 9),
        ("RoundFancy", 9),
        ("Always", 1),
    ] {
        assert_eq!(face_of(&plain, id), h(want), "#{id} in body-only fonts");
    }
}

/// CSS Fonts 4 §9.1: `palette-mix()` mixes its palettes entry by entry as
/// `color-mix()` mixes colours (`normal` in it is the font's first
/// palette), and `font-palette` animates by such a mix in OKLab. A palette
/// that moves every frame makes an instance every frame; until the host
/// makes one's font, its text draws with the nearest one made.
#[test]
fn font_palettes_mix_and_animate_entry_by_entry() {
    let pal = face(PALETTE, 8);
    let file = pal.metrics.file.clone().expect("its file");
    let mut d = doc(
        vec![family("Pal", (400, 400), pal)],
        r#"<p id="Mix">AB</p><p id="Normal">AB</p><p id="Half">AB</p><p id="Run">AB</p>"#,
        r#"#Root { font-family: "Pal"; }
@keyframes swap { from { font-palette: light; } to { font-palette: dark; } }
#Mix { font-palette: palette-mix(in srgb, light 25%, dark); }
#Normal { font-palette: palette-mix(in srgb, normal, dark); }
#Half { animation: swap 1000ms linear -500ms paused; }
#Run { animation: swap 1000ms linear infinite; }"#,
    );
    let mut made: Vec<std::sync::Arc<og_ui::shape::FontFile>> = Vec::new();
    let make = |made: &mut Vec<_>| {
        for f in file.take_instances() {
            f.set_handle(FontHandle::from_index(100 + made.len()));
            made.push(f);
        }
    };
    make(&mut made);
    d.step(InputFrame::default());
    // The palette each text is drawn in, 8-bit.
    let palette_of = |d: &Doc, made: &[std::sync::Arc<og_ui::shape::FontFile>], id: &str| {
        let t = drawn(d, id);
        assert!(!t.is_empty(), "#{id} draws");
        let instance = made
            .iter()
            .find(|f| f.handle() == Some(t[0].font))
            .unwrap_or_else(|| panic!("#{id} in an instance"));
        instance
            .palette()
            .expect("in a palette")
            .iter()
            .map(|c| c.map(|v| (v * 255.0).round() as u8))
            .collect::<Vec<_>>()
    };
    let near = |got: Vec<[u8; 4]>, want: [[u8; 3]; 2]| {
        got.len() == 2 && got.iter().zip(want).all(|(g, w)| near_rgb(*g, w))
    };
    // light is blue and yellow, dark white and black, normal red and green.
    let mix = palette_of(&d, &made, "Mix");
    assert!(near(mix.clone(), [[191, 191, 255], [64, 64, 0]]), "{mix:?}");
    let normal = palette_of(&d, &made, "Normal");
    assert!(
        near(normal.clone(), [[255, 128, 128], [0, 128, 0]]),
        "{normal:?}"
    );
    // Halfway in OKLab: in sRGB it would be 128, 128, 255 and 128, 128, 0.
    let half = palette_of(&d, &made, "Half");
    assert!(
        near(half.clone(), [[116, 163, 255], [99, 99, 0]]),
        "{half:?}"
    );
    // A running one makes a new instance each frame; each frame draws
    // with one already made while the new one's font waits.
    for _ in 0..3 {
        make(&mut made);
        d.step(InputFrame::default());
        let before = made.len();
        let fresh = file.take_instances();
        assert!(!fresh.is_empty(), "the palette moved");
        let font = drawn(&d, "Run")[0].font;
        assert!(
            made[..before].iter().any(|f| f.handle() == Some(font)),
            "drawn with a made instance's font"
        );
        for f in fresh {
            f.set_handle(FontHandle::from_index(100 + made.len()));
            made.push(f);
        }
    }
}

/// The glyph ids drawn inside element `id`, left to right.
fn glyph_ids(d: &Doc, id: &str) -> Vec<Option<u16>> {
    let r = d.get(id);
    d.ui.frame()
        .items
        .iter()
        .filter_map(|it| match it.cmd.unstyled() {
            DrawCmd::Text {
                bounds, placements, ..
            } if (r.y..r.y + r.height).contains(&(bounds.y + bounds.height / 2.0)) => {
                Some(placements.iter().map(|g| g.glyph).collect::<Vec<_>>())
            }
            _ => None,
        })
        .next()
        .unwrap_or_default()
}

/// CSS Fonts 4 §6.8 and §6.9: `font-variant-alternates`' functions name
/// values an `@font-feature-values` rule for the text's family defines
/// (the last definition of a name), and set `salt`, one `ssNN` per value
/// and so on; a name no rule for the family defines is ignored, and
/// `font-feature-settings` applies after them (§7.2). Noto Sans's own
/// GSUB (fontTools): I is 904 under `salt` or `ss04`; ƒ is 319 and 1231
/// under `ss03`.
#[test]
fn font_variant_alternates_take_the_feature_values_named() {
    let d = doc(
        vec![family("Noto", (400, 400), face(NOTO, 9))],
        r#"<p id="Plain">Iƒ</p><p id="Alt">Iƒ</p><p id="Florin">Iƒ</p><p id="Both">Iƒ</p>
<p id="Late">Iƒ</p><p id="Other">Iƒ</p><p id="Nope">Iƒ</p><div id="Up"><p id="Inherit">Iƒ</p></div>
<p id="Short">Iƒ</p><p id="Off">Iƒ</p>"#,
        r#"#Root { font-family: Noto; }
@font-feature-values Noto, Other { @stylistic { alt: 1; } @styleset { florin: 3; both: 3 4; late: 3; } }
@font-feature-values "noto" { @styleset { late: 4; } }
@font-feature-values Other { @styleset { other: 4; } }
#Alt { font-variant-alternates: stylistic(alt); }
#Florin { font-variant-alternates: styleset(florin); }
#Both { font-variant-alternates: styleset(both); }
#Late { font-variant-alternates: styleset(late); }
#Other { font-variant-alternates: styleset(other); }
#Nope { font-variant-alternates: styleset(nope) stylistic(nope); }
#Up { font-variant-alternates: styleset(florin); }
#Short { font-variant: stylistic(alt) small-caps; }
#Off { font-variant-alternates: stylistic(alt); font-feature-settings: "salt" 0; }"#,
    );
    let ids = |id: &str| glyph_ids(&d, id);
    // An I no feature substitutes is drawn by its character.
    let (plain, alt, florin) = (None, Some(904), Some(1231));
    assert_eq!(ids("Plain"), [plain, Some(319)]);
    assert_eq!(ids("Alt"), [alt, Some(319)]);
    assert_eq!(ids("Florin"), [plain, florin]);
    assert_eq!(ids("Both"), [alt, florin]);
    assert_eq!(ids("Late"), [alt, Some(319)]);
    assert_eq!(ids("Other"), [plain, Some(319)]);
    assert_eq!(ids("Nope"), [plain, Some(319)]);
    assert_eq!(ids("Inherit"), [plain, florin]);
    assert_eq!(ids("Short")[0], alt);
    assert_eq!(ids("Off"), [plain, Some(319)]);
}

/// CSS Fonts 4 §6.13: text is shaped in its content language (`lang`,
/// inherited; empty is unknown), so a face's `locl` forms for it apply,
/// and `font-language-override` names the OpenType language system
/// instead. Noto Sans's own GSUB (fontTools): Romanian (ROM) draws Ş
/// (285) as Ș (326), Marshallese (MAH) Ļ (250) as 898.
#[test]
fn font_language_override_and_lang_pick_the_fonts_local_forms() {
    let d = doc(
        vec![family("Noto", (400, 400), face(NOTO, 9))],
        r#"<p id="Plain">ŞĻ</p><p id="Ro" lang="ro">ŞĻ</p><div lang="ro-RO"><p id="Inherit">ŞĻ</p>
<p id="Unknown" lang="">ŞĻ</p></div><p id="Tag">ŞĻ</p><p id="Over" lang="ro">ŞĻ</p>
<p id="Para" lang="ro">ŞĻ</p>"#,
        r#"#Root { font-family: Noto; }
#Tag { font-language-override: "ROM"; }
#Over { font-language-override: "MAH"; }
#Para { text-indent: 4px; }"#,
    );
    let ids = |id: &str| glyph_ids(&d, id);
    let (s, l) = (Some(285), Some(250));
    assert_eq!(ids("Plain"), [s, l]);
    assert_eq!(ids("Ro"), [Some(326), l]);
    assert_eq!(ids("Inherit"), [Some(326), l]);
    assert_eq!(ids("Unknown"), [s, l]);
    assert_eq!(ids("Tag"), [Some(326), l]);
    assert_eq!(ids("Over"), [s, Some(898)]);
    assert_eq!(ids("Para"), [Some(326), l]);
}
