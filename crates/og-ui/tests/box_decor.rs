//! CSS box decorations in documents: `border` and its longhands,
//! `outline`, `box-shadow`, `overflow` and `scrollbar-width`, built both
//! ways a document is (immediate and mounted).
//!
//! Ways these could fail, each covered below:
//! - a `border-radius` on a dressed control (a `<button>`) ignored for the
//!   style's corner → `border_radius_rounds_a_dressed_button`
//! - a border that takes no room (content under it), a side's width read
//!   for another side, a width counted with `border-style: none`, an
//!   outline that moves layout → `borders_pad_the_box_and_outlines_take_no_room`
//! - a border drawn at the wrong edge or colour, `currentColor` not the
//!   text colour, an outline not grown by its offset, a shadow drawn over
//!   the background instead of under it, the first shadow not on top, a
//!   bordered `<button>` losing its dressing →
//!   `decorations_paint_in_css_order_over_any_draw`
//! - a `<div>` that shrinks to its content instead of filling its block
//!   container → `borders_pad_the_box_and_outlines_take_no_room`
//! - `overflow: auto` not scrolling a box, or the shorthand's axes swapped
//!   → `overflow_scrolls_and_clips`
//! - a malformed value accepted → `bad_decorations_are_refused`
//! - `overscroll-behavior: contain` still chaining the wheel's rest to the
//!   container around it (or `auto` no longer chaining), and
//!   `scroll-padding` not insetting where a snap lines its area up →
//!   `overscroll_contains_and_scroll_padding_insets_snaps`
//! - a wheel settling past a `scroll-snap-stop: always` area it crossed,
//!   or `scrollbar-color` not inherited →
//!   `a_wheel_stops_at_snap_stop_always_and_scrollbar_color_inherits`

use crate::support::{Doc, both};
use og_core::glam::Vec2;
use og_render::{UiAlign, UiMode};
use og_ui::*;

fn lin(hex: &str) -> [f32; 4] {
    og_ui::style_file::parse_color(hex).unwrap().linear()
}

fn close(a: [f32; 4], b: [f32; 4]) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-3)
}

#[test]
fn borders_pad_the_box_and_outlines_take_no_room() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="A"><div id="AK"></div></div>
  <div id="B"><div id="BK"></div></div>
  <div id="C"><div id="CK"></div></div>
  <div id="D"><div id="DK"></div></div>
  <div id="E"><div id="EK"></div></div>
</div></div>
<style>
#Root { width: 400px; }
#Root div div { width: 10px; height: 10px; }
#A { border: 4px solid #ff0000; padding: 6px; }
#B { border-top: 2px solid; border-left: thick dashed var(--accent); }
#C { outline: 3px solid; outline-offset: 2px; }
#D { border-width: 9px; }
#E { border-inline: 1px solid; border-inline-end-width: 7px; border-block-start: 3px solid; border-block-style: solid none; }
</style>"#;
    both(text, &[], |d| {
        let off = |id: &str, kid: &str| {
            let (a, k) = (d.get(id), d.get(kid));
            (k.x - a.x, k.y - a.y, a.height)
        };
        assert_eq!(off("A", "AK"), (10.0, 10.0, 30.0));
        // A block box fills its block container's width (width: auto),
        // its border and padding inside it.
        assert_eq!(d.get("A").width, 400.0);
        // thick is 5px; only the top and left sides have a style.
        assert_eq!(off("B", "BK"), (5.0, 2.0, 12.0));
        assert_eq!(off("C", "CK"), (0.0, 0.0, 10.0));
        // A width without a style draws nothing and takes no room.
        assert_eq!(off("D", "DK"), (0.0, 0.0, 10.0));
        // Logical sides in horizontal-tb, left to right: inline-start is
        // the left (1px), inline-end the right (7px), block-start the top
        // (3px); the block end's style is none, so it takes no room.
        assert_eq!(off("E", "EK"), (1.0, 3.0, 13.0));
        assert_eq!(d.get("E").width, 400.0);
    });
}

/// Element `id`'s paint commands from the last frame, in order: those whose
/// box or clip lies on its rectangle.
fn shapes(ui: &Ui) -> Vec<og_render::UiShape> {
    ui.frame()
        .items
        .iter()
        .filter_map(|i| match &i.cmd {
            DrawCmd::Shape { shape, .. } => Some((**shape).clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn decorations_paint_in_css_order_over_any_draw() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="A"></div>
  <p id="C">C</p>
  <div id="S"></div>
  <button id="Btn">Go</button>
  <div id="G"></div>
</div></div>
<style>
#Root { width: 400px; gap: 20px; }
#G { width: 40px; height: 20px; border: 4px groove #808080; }
#A { width: 50px; height: 30px; border: 4px solid #ff0000; }
#C { margin: 0; width: 40px; height: 20px; color: #00ff00; outline: 3px dashed; outline-offset: 2px; }
#S { width: 40px; height: 40px; background: #3366ff; box-shadow: 0 4px 12px #000000, inset 0 0 0 2px #ffffff; }
#Btn::part(base) { border: 2px solid #ffff00; }
</style>"#;
    both(text, &[], |d| {
        let s = shapes(&d.ui);
        let at = |id: &str| {
            let e = d.get(id);
            Vec2::new(e.x + e.width / 2.0, e.y + e.height / 2.0)
        };
        let stroke = |sh: &og_render::UiShape| match sh.mode {
            UiMode::Stroke(st) => Some(st),
            _ => None,
        };
        // A: a 4px red line inside its box.
        let a = s
            .iter()
            .find(|x| x.center == at("A") && stroke(x).is_some_and(|st| st.width == 4.0))
            .expect("A's border");
        assert_eq!(stroke(a).unwrap().align, UiAlign::Inside);
        assert_eq!(a.half, Vec2::new(25.0, 15.0));
        assert!(matches!(a.paint, og_render::UiPaint::Solid(c) if close(c, lin("#ff0000"))));
        // C: its outline outside a box grown by the offset, in its text
        // colour (currentColor), dashed: dashes and gaps about three
        // widths long, fitted to each side.
        let c = s
            .iter()
            .find(|x| x.center == at("C") && stroke(x).is_some_and(|st| st.width == 3.0))
            .expect("C's outline");
        let st = stroke(c).unwrap();
        assert_eq!(st.align, UiAlign::Outside);
        assert!(
            st.dash[0] == st.dash[1] && (st.dash[0] / 9.0 - 1.0).abs() < 0.5,
            "{:?}",
            st.dash
        );
        assert_eq!(c.half, Vec2::new(22.0, 12.0));
        assert!(matches!(c.paint, og_render::UiPaint::Solid(k) if close(k, lin("#00ff00"))));
        // S: outer shadow, then its background, then the inset shadow.
        let mine: Vec<&og_render::UiShape> = s.iter().filter(|x| x.center == at("S")).collect();
        let shadow = mine
            .iter()
            .position(|x| matches!(x.mode, UiMode::Shadow { offset, blur, .. } if offset == Vec2::new(0.0, 4.0) && blur == 12.0))
            .expect("S's drop shadow");
        let fill = mine
            .iter()
            .position(|x| {
                matches!(x.mode, UiMode::Fill)
                    && matches!(x.paint, og_render::UiPaint::Solid(k) if close(k, lin("#3366ff")))
            })
            .expect("S's background, drawn with its shadows");
        let inset = mine
            .iter()
            .position(|x| matches!(x.mode, UiMode::InnerShadow { spread, .. } if spread == 2.0))
            .expect("S's inset shadow");
        assert!(shadow < fill && fill < inset, "{shadow} {fill} {inset}");
        // The button keeps its dressed plate under its border.
        let btn: Vec<&og_render::UiShape> = s.iter().filter(|x| x.center == at("Btn")).collect();
        assert!(
            btn.iter()
                .any(|x| stroke(x).is_some_and(|st| st.width == 2.0))
        );
        assert!(
            btn.iter().any(|x| matches!(x.mode, UiMode::Fill))
                || d.ui.frame().items.iter().any(|i| matches!(&i.cmd, DrawCmd::Quad { rect, .. } if rect.width == d.get("Btn").width)),
            "the button's own plate"
        );
        // G: a groove is two half strips a side, the outer half of the top
        // dark and its inner half light, the bottom the reverse.
        let g = d.get("G");
        let quads: Vec<_> =
            d.ui.frame()
                .items
                .iter()
                .filter_map(|i| match &i.cmd {
                    DrawCmd::Quad { rect, color, .. } => Some((*rect, *color)),
                    _ => None,
                })
                .collect();
        let quad = |x: f32, y: f32, w: f32, h: f32| {
            quads
                .iter()
                .find(|(r, _)| (r.x, r.y, r.width, r.height) == (g.x + x, g.y + y, w, h))
                .map(|(_, c)| *c)
                .unwrap_or_else(|| panic!("G's strip at {x},{y} {w}x{h} in {quads:?}"))
        };
        let (light, dark) = (lin("#808080"), lin("#404040"));
        assert!(close(quad(0.0, 0.0, 40.0, 2.0), dark), "top outer half");
        assert!(close(quad(0.0, 2.0, 40.0, 2.0), light), "top inner half");
        assert!(
            close(quad(0.0, 18.0, 40.0, 2.0), light),
            "bottom outer half"
        );
        assert!(close(quad(0.0, 16.0, 40.0, 2.0), dark), "bottom inner half");
    });
}

#[test]
fn border_radius_rounds_a_dressed_button() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <button id="Round">Go</button>
  <button id="Plain">Go</button>
</div></div>
<style>
#Root { width: 400px; gap: 20px; }
#Round::part(base) { border-radius: 20px; }
</style>"#;
    both(text, &[], |d| {
        let s = shapes(&d.ui);
        let radii = |id: &str| {
            let e = d.get(id);
            let at = Vec2::new(e.x + e.width / 2.0, e.y + e.height / 2.0);
            s.iter()
                .filter(|x| x.center == at && matches!(x.mode, UiMode::Fill))
                .find_map(|x| match x.geometry {
                    og_render::UiGeometry::Rect { radii, .. } => Some(radii),
                    _ => None,
                })
                .or_else(|| {
                    d.ui.frame().items.iter().find_map(|item| match item.cmd {
                        DrawCmd::Quad { rect, radius, .. }
                            if Vec2::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0)
                                == at =>
                        {
                            Some([radius; 4])
                        }
                        _ => None,
                    })
                })
                .unwrap_or_else(|| panic!("{id}'s plate"))
        };
        // The written corner replaces the dressing's (the renderer clamps
        // overlapping radii, as CSS does).
        assert_eq!(radii("Round"), [20.0; 4]);
        assert_ne!(radii("Plain"), radii("Round"), "the style's corner differs");
    });
}

#[test]
fn corners_take_css_radii_and_shapes() {
    // 200 × 100: the top radii sum to 240, so every radius scales by
    // 200 / 240 (CSS Backgrounds 3 §5.5); the bottom corners are bevels.
    let text = r#"<meta name="component" content="t">
<div><div id="Root"><div id="Box"></div><div id="Pill"></div></div></div>
<style>
#Root { width: 400px; }
#Box { width: 200px; height: 100px; background: #224466;
  border: 2px solid #ffffff;
  border-top-left-radius: 160px 40px; border-top-right-radius: 80px;
  border-bottom-right-radius: 10%; corner-shape: round round bevel bevel; }
#Pill { width: 200px; height: 100px; background: #224466; border-radius: 50% / 20px 10px; }
</style>"#;
    both(text, &[], |d| {
        let s = shapes(&d.ui);
        let geo = |id: &str, fill: bool| {
            let e = d.get(id);
            let at = Vec2::new(e.x + e.width / 2.0, e.y + e.height / 2.0);
            s.iter()
                .filter(|x| x.center == at && matches!(x.mode, UiMode::Fill) == fill)
                .find_map(|x| match x.geometry {
                    og_render::UiGeometry::Rect {
                        radii,
                        corner: og_render::UiCorner::Css(c),
                    } => Some((radii, c)),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("{id}'s CSS corners"))
        };
        let near = |a: [f32; 4], b: [f32; 4]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 0.01);
        let f = 200.0 / 240.0;
        for fill in [true, false] {
            let (h, c) = geo("Box", fill);
            assert!(near(h, [160.0 * f, 80.0 * f, 20.0 * f, 0.0]), "{h:?}");
            assert!(
                near(c.vertical, [40.0 * f, 80.0 * f, 10.0 * f, 0.0]),
                "{c:?}"
            );
            assert_eq!(c.shape, [1.0, 1.0, 0.0, 0.0]);
        }
        let (h, c) = geo("Pill", true);
        assert!(near(h, [100.0; 4]), "{h:?}");
        assert!(near(c.vertical, [20.0, 10.0, 20.0, 10.0]), "{c:?}");
    });
}

#[test]
fn overscroll_contains_and_scroll_padding_insets_snaps() {
    for (behaviour, outer) in [("auto", 30.0), ("contain", 0.0)] {
        let text = format!(
            r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="Out"><div id="In"><div></div><div></div><div></div></div><div id="Fill"></div></div>
  <div id="Snap"><div></div><div></div><div></div><div></div></div>
</div></div>
<style>
#Root {{ width: 400px; }}
#Out {{ height: 100px; overflow-y: auto; scrollbar-width: none; }}
#In {{ height: 50px; overflow-y: auto; scrollbar-width: none; overscroll-behavior: {behaviour}; }}
#In div {{ height: 40px; }}
#Fill {{ height: 300px; }}
#Snap {{ height: 100px; overflow-y: auto; scrollbar-width: none; scroll-snap-type: y mandatory; scroll-padding: 20px 0 0; }}
#Snap div {{ height: 60px; scroll-snap-align: start; }}
</style>"#
        );
        both(&text, &[], |d| {
            // The inner box's range is 70: a wheel of 100 leaves 30.
            let b = d.get("In");
            d.step(InputFrame {
                wheel: Vec2::new(0.0, 100.0),
                ..InputFrame::hover(b.x + 10.0, b.y + 10.0)
            });
            d.step(InputFrame::hover(b.x + 10.0, b.y + 10.0));
            let at = |d: &mut Doc, id: &str| d.ui.scroll_state(&d.get(id).path).unwrap().offset_y;
            assert_eq!((at(d, "In"), at(d, "Out")), (70.0, outer), "{behaviour}");
            // Snap positions sit 20 px down the view: 0, 40, 100…; 50
            // settles on 40 (without the padding 0, 60…: on 60).
            let s = d.get("Snap");
            d.step(InputFrame {
                wheel: Vec2::new(0.0, 50.0),
                ..InputFrame::hover(s.x + 10.0, s.y + 10.0)
            });
            for _ in 0..80 {
                d.step(InputFrame::hover(s.x + 10.0, s.y + 10.0));
            }
            assert!((at(d, "Snap") - 40.0).abs() < 0.5, "{}", at(d, "Snap"));
        });
    }
}

#[test]
fn a_wheel_stops_at_snap_stop_always_and_scrollbar_color_inherits() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="Snap"><div></div><div class="stop"></div><div></div><div></div><div></div></div>
</div></div>
<style>
#Root { width: 400px; scrollbar-color: red blue; }
#Snap { height: 100px; overflow-y: auto; scroll-snap-type: y mandatory; }
#Snap div { height: 60px; scroll-snap-align: start; }
.stop { scroll-snap-stop: always; }
</style>"#;
    both(text, &[], |d| {
        // Snap positions 0, 60, 120…: a wheel of 150 in three ticks would
        // settle on 120, but it may not pass the area at 60, even though
        // the wheel has carried it past that area by the time it settles.
        let s = d.get("Snap");
        for _ in 0..3 {
            d.step(InputFrame {
                wheel: Vec2::new(0.0, 50.0),
                ..InputFrame::hover(s.x + 10.0, s.y + 10.0)
            });
        }
        for _ in 0..300 {
            d.step(InputFrame::hover(s.x + 10.0, s.y + 10.0));
        }
        let at = d.ui.scroll_state(&d.get("Snap").path).unwrap().offset_y;
        assert!((at - 60.0).abs() < 0.5, "{at}");
        // The scroller inherits its ancestor's thumb and track colours.
        let red = Srgba::rgb(255, 0, 0).linear();
        let painted = d.ui.frame().items.iter().any(|i| match &i.cmd {
            DrawCmd::Quad { color, .. } => {
                (color[0] - red[0]).abs() < 1e-3 && color[1] < 1e-3 && color[3] > 0.9
            }
            _ => false,
        });
        assert!(painted, "no red thumb");
    });
}

#[test]
fn overflow_scrolls_and_clips() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="Box"><div id="K1"></div><div id="K2"></div><div id="K3"></div></div>
</div></div>
<style>
#Root { width: 400px; }
#Box { height: 50px; overflow: hidden auto; scrollbar-width: none; }
#Box div { height: 40px; }
</style>"#;
    both(text, &[], |d| {
        let b = d.get("Box");
        for wheel in [Vec2::new(0.0, 30.0), Vec2::new(30.0, 0.0)] {
            d.step(InputFrame {
                wheel,
                ..InputFrame::hover(b.x + 10.0, b.y + 10.0)
            });
            for _ in 0..40 {
                d.step(InputFrame::hover(b.x + 10.0, b.y + 10.0));
            }
        }
        // overflow-y auto scrolls; overflow-x hidden does not.
        let s = d.ui.scroll_state(&b.path).expect("a scroll container");
        assert_eq!((s.offset_x, s.offset_y), (0.0, 30.0));
        assert_eq!((s.view_height, s.content_height), (50.0, 120.0));
    });
}

#[test]
fn bad_decorations_are_refused() {
    let err = |body: &str| {
        UiDocument::parse(
            "ui/t.svelte",
            &format!("<meta name=\"component\" content=\"t\"><div id=\"A\"></div><style>#A {{ {body} }}</style>"),
        )
        .unwrap_err()
        .join("\n")
    };
    assert!(err("border: 2px wavy red").contains("not a border width, style"));
    assert!(err("box-shadow: 4px").contains("two to four lengths"));
    assert!(err("box-shadow: 0 0 -2px #000").contains("blur is not negative"));
    assert!(err("overflow: sideways").contains("not an overflow"));
    assert!(err("outline: 1px hidden").contains("hidden"));
}

const PAGE: &str = r#"<meta name="component" content="t">
<main id="Page">
  <h2 i18n="h">Box decorations</h2>
  <div class="grid">
    <div class="card solid"><p i18n="a">Solid</p></div>
    <div class="card dashed"><p i18n="b">Dashed</p></div>
    <div class="card dotted"><p i18n="c">Dotted</p></div>
    <div class="card double"><p i18n="d">Double</p></div>
    <div class="card inset"><p i18n="e">Inset</p></div>
    <div class="card groove"><p i18n="k">Groove</p></div>
    <div class="card sides"><p i18n="f">Sides</p></div>
    <div class="card raised"><p i18n="g">Shadow</p></div>
    <div class="card outlined"><p i18n="h">Outline</p></div>
    <div class="card clipped"><div class="spill"></div></div>
    <div class="card corners"><div class="spill"></div></div>
    <div class="card shaped"><p i18n="s">Shapes</p></div>
  </div>
  <button class="go" i18n="go">Continue</button>
</main>
<style>
main { width: 100%; height: 100%; padding: 24px; gap: 16px; background: #4a5060; }
.grid { display: grid; grid-template-columns: 110px 110px 110px; gap: 24px; }
.card { height: 64px; padding: 8px; background: var(--panel); border-radius: 10px; }
.card p { margin: 0; }
.solid { border: 2px solid var(--accent); }
.dashed { border: 3px dashed var(--danger); }
.dotted { border: 2px dotted; }
.double { border: 6px double var(--accent); }
.inset { border: 4px inset var(--control-strong); border-radius: 0; }
.groove { border: 6px groove var(--control-strong); border-radius: 0; }
.sides { border-top: 6px solid var(--accent); border-left: 2px solid var(--danger); border-radius: 0; }
.raised { box-shadow: 0 6px 16px #000000aa, inset 0 0 0 1px #ffffff33; }
.outlined { outline: 2px solid var(--accent); outline-offset: 4px; }
.clipped { overflow: hidden; }
.spill { width: 160px; height: 90px; background: var(--danger); }
.corners { overflow: hidden; border: 3px solid var(--accent); border-radius: 40px 8px / 24px 8px; box-shadow: 0 6px 12px #000000aa; }
.shaped { border: 2px solid var(--accent); border-radius: 50% 20px 20px 20px; corner-shape: round squircle bevel scoop; }
.go { border: 2px solid var(--accent); border-radius: 20px; margin: 0 auto; }
@media (width < 600px) { .grid { grid-template-columns: 110px 110px; } }
</style>"#;

#[test]
#[ignore = "requires GPU"]
fn box_decor_matches_its_reference_frames() {
    frames(PAGE, "box-decor", &[(1280, 720), (390, 844)]);
}

/// Background, mask and border image layers.
const LAYERS: &str = r#"<meta name="component" content="t">
<main id="Page">
  <div class="grid">
    <div class="card tiles"><p i18n="a">Tiles</p></div>
    <div class="card origin"><p i18n="b">Content</p></div>
    <div class="card blend"><p i18n="c">Blend</p></div>
    <p class="clipped" i18n="d">Clipped text</p>
    <div class="card framed"><p i18n="e">Frame</p></div>
    <div class="card cut"><p i18n="f">Masks</p></div>
    <div class="card notched"><p i18n="g">Mask border</p></div>
    <div class="card fixed"></div>
  </div>
</main>
<style>
main { width: 100%; height: 100%; padding: 24px; background: #4a5060; }
.grid { display: grid; grid-template-columns: 150px 150px 150px 150px; gap: 24px; }
.card { height: 90px; padding: 12px; background: var(--panel); border-radius: 12px; }
.card p { margin: 0; }
.tiles { background: radial-gradient(circle, #ffd166 30%, transparent 32%) 0 0 / 20px 20px, #264653; }
.origin { border: 8px dashed #ffffff66; padding: 16px; background: linear-gradient(90deg, #e76f51, #2a9d8f) content-box no-repeat; }
.blend { background: linear-gradient(#ff0000, #0000ff), linear-gradient(90deg, #ffffff, #000000); background-blend-mode: multiply; }
.clipped { width: max-content; font: var(--text-display); background: linear-gradient(90deg, #ffd166, #ef476f); background-clip: text; color: transparent; }
.framed { border: 14px solid; border-image: repeating-linear-gradient(45deg, #ffd166 0 6px, #264653 6px 12px) 14 round; }
.cut { background: #2a9d8f; mask: linear-gradient(#000, transparent), radial-gradient(circle at 70% 50%, #000 30%, transparent 32%) add; mask-composite: exclude, add; }
.notched { padding: 1px 12px; background: #ef476f; mask-border: linear-gradient(#000, #000) 20 / 20px; mask-border-mode: alpha; }
.fixed { background: linear-gradient(#ffffff22, #00000055), repeating-linear-gradient(90deg, #e9c46a 0 10px, #f4a261 10px 20px) fixed; }
</style>"#;

#[test]
#[ignore = "requires GPU"]
fn box_layers_match_their_reference_frames() {
    frames(LAYERS, "box-layers", &[(1280, 720)]);
}

/// Renders `page` at each size and compares it with `references/vulkan/<name>-<width>.png`.
fn frames(page: &str, stem: &str, sizes: &[(u32, u32)]) {
    use crate::support::*;
    use og_render::*;
    for &(width, height) in sizes {
        let mut scene = RenderScene::new();
        let font = add_test_font(&mut scene);
        let mut ui = ui_with_font(font);
        ui.define_document(UiDocument::parse("ui/t.svelte", page).unwrap());
        ui.mount_document(
            og_ui::reactive::SourceId::new("test", "t"),
            "t",
            Default::default(),
            Default::default(),
            &bind::NoEnv,
        )
        .unwrap();
        let vp = Viewport::new(width as f32, height as f32);
        for _ in 0..3 {
            ui.begin(InputFrame::default(), vp);
            ui.build_retained_documents(&bind::NoEnv).unwrap();
            ui.end();
        }
        let mut overlay = Overlay::new(width as f32, height as f32, 1.0);
        ui.frame().emit(&EmitDesc::default(), &mut overlay);
        scene.ui.overlay = Some(overlay);
        let bg = ui.theme().palette.backdrop.linear();
        scene.lighting.environment.background =
            Background::Color(og_core::glam::Vec3::new(bg[0], bg[1], bg[2]));
        scene.look.post.tonemap = Tonemap::None;
        let mut renderer = Renderer::new_blocking(&RenderDesc {
            width,
            height,
            ..Default::default()
        })
        .expect("a headless adapter");
        let stats = renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
        assert_eq!(stats.missing_glyphs, 0);
        let image = renderer.read_offscreen().unwrap();
        let name = format!("{stem}-{width}.png");
        image::save_buffer(
            out_dir().join(&name),
            &image.pixels,
            width,
            height,
            image::ColorType::Rgba8,
        )
        .unwrap();
        let own = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("references")
            .join(og_tests::compare::RECORDING_TIER)
            .join(&name);
        if std::env::var_os("OG_UI_CAPTURE").is_some() {
            record_reference(&own, width, height, &image.pixels);
        }
        let expected = image::open(&own)
            .unwrap_or_else(|e| panic!("{}: {e} (OG_UI_CAPTURE=1 records it)", own.display()))
            .to_rgba8();
        let (rmse, _) = compare(&image.pixels, expected.as_raw(), renderer.info(), &own)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        println!("{name}: RMSE {rmse:.6}");
    }
}
