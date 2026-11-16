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

use crate::support::Doc;
use og_draw::FontHandle;
use og_ui::*;

const NOTO: &str = "templates/fonts/latin/NotoSans-Latin.ttf";
const INTER: &str = "tests/fixtures/ui-text/assets/fonts/Inter-Display.ttf";

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
