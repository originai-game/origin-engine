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
use og_draw::{UiAlign, UiMode};
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
fn shapes(ui: &Ui) -> Vec<og_draw::UiShape> {
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
        let stroke = |sh: &og_draw::UiShape| match sh.mode {
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
        assert!(matches!(a.paint, og_draw::UiPaint::Solid(c) if close(c, lin("#ff0000"))));
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
        assert!(matches!(c.paint, og_draw::UiPaint::Solid(k) if close(k, lin("#00ff00"))));
        // S: outer shadow, then its background, then the inset shadow.
        let mine: Vec<&og_draw::UiShape> = s.iter().filter(|x| x.center == at("S")).collect();
        let shadow = mine
            .iter()
            .position(|x| matches!(x.mode, UiMode::Shadow { offset, blur, .. } if offset == Vec2::new(0.0, 4.0) && blur == 12.0))
            .expect("S's drop shadow");
        let fill = mine
            .iter()
            .position(|x| {
                matches!(x.mode, UiMode::Fill)
                    && matches!(x.paint, og_draw::UiPaint::Solid(k) if close(k, lin("#3366ff")))
            })
            .expect("S's background, drawn with its shadows");
        let inset = mine
            .iter()
            .position(|x| matches!(x.mode, UiMode::InnerShadow { spread, .. } if spread == 2.0))
            .expect("S's inset shadow");
        assert!(shadow < fill && fill < inset, "{shadow} {fill} {inset}");
        // The button keeps its dressed plate under its border.
        let btn: Vec<&og_draw::UiShape> = s.iter().filter(|x| x.center == at("Btn")).collect();
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
                    og_draw::UiGeometry::Rect { radii, .. } => Some(radii),
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
                    og_draw::UiGeometry::Rect {
                        radii,
                        corner: og_draw::UiCorner::Css(c),
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
