//! CSS Containment 2 in documents (docs/ui.md §2): `contain`,
//! `content-visibility` and `contain-intrinsic-size`, and CSS UI 4's
//! `resize`, built both ways.
//!
//! Ways these could fail, each covered below:
//! - size containment sizing by the contents anyway, or ignoring
//!   `contain-intrinsic-size`; `inline-size` containing the height too →
//!   `size_containment_sizes_as_if_empty`
//! - `content-visibility: hidden` laying out, hitting or exposing its
//!   contents, or `auto` laying out what is far off screen, never laying
//!   it out once it comes near, or forgetting its last size under
//!   `contain-intrinsic-size: auto` (or remembering without it) →
//!   `content_visibility_skips_contents_off_screen`
//! - a resize handle not taking the press, its drag not reaching the
//!   element's size (or lost when it builds again), an axis it does not
//!   resize moved, `max-width` ignored, or `resize` applied with overflow
//!   visible → `a_resize_handle_drags_the_size_onto_the_element`

use crate::support::{Doc, both};
use og_ui::InputFrame;
use serde_json::json;

#[test]
fn size_containment_sizes_as_if_empty() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="Sized"><div id="SizedIn"></div></div>
  <div id="Given"><div id="GivenIn"></div></div>
  <div id="Inline"><div id="InlineIn"></div></div>
</div></div>
<style>
#Root { display: flex; flex-direction: column; align-items: flex-start; }
#SizedIn, #GivenIn, #InlineIn { width: 120px; height: 80px; }
#Sized { contain: size; }
#Given { contain: strict; contain-intrinsic-size: 30px 40px; }
#Inline { contain: inline-size; }
</style>"#;
    both(text, &[], |d| {
        d.step(InputFrame::default());
        let wh = |d: &Doc, id: &str| {
            let e = d.get(id);
            (e.width, e.height)
        };
        assert_eq!(wh(d, "Sized"), (0.0, 0.0));
        assert_eq!(
            wh(d, "SizedIn"),
            (120.0, 80.0),
            "its contents still lay out"
        );
        assert_eq!(wh(d, "Given"), (30.0, 40.0));
        assert_eq!(wh(d, "Inline"), (0.0, 80.0));
    });
}

#[test]
fn content_visibility_skips_contents_off_screen() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="Gone"><button id="GoneButton">Gone</button></div>
  <div id="Far" class={$where}><div id="FarIn"></div></div>
  <div id="After"></div>
  <div id="Kept" class={$where}><div id="KeptIn"></div></div>
  <div id="Last"></div>
</div></div>
<style>
#Root { display: flex; flex-direction: column; align-items: flex-start; }
#Gone { content-visibility: hidden; contain-intrinsic-size: 50px 20px; }
#After, #Last { width: 10px; height: 10px; }
#Far { content-visibility: auto; contain-intrinsic-size: 10px; }
#Kept { content-visibility: auto; contain-intrinsic-size: auto 10px; }
.away { position: relative; top: -5000px; }
#FarIn, #KeptIn { width: 200px; height: 150px; }
</style>"#;
    both(text, &[("where", json!("away"))], |d| {
        for _ in 0..3 {
            d.step(InputFrame::default());
        }
        let gone = d.get("Gone");
        assert_eq!((gone.width, gone.height), (50.0, 20.0));
        assert!(
            !format!("{:?}", d.ui.a11y()).contains("GoneButton"),
            "hidden contents leave the accessibility tree"
        );
        // Far above the viewport: skipped from the frame after it was
        // seen there, sized by its intrinsic size; with `auto`, by the
        // size it had on that first frame. Its flow place shows in where
        // the next box starts.
        let far_h = |d: &Doc| d.get("After").y - d.get("Gone").y - 20.0;
        let kept_h = |d: &Doc| d.get("Last").y - d.get("After").y - 10.0;
        assert_eq!(far_h(d), 10.0);
        assert_eq!(kept_h(d), 150.0);
        // Brought near: it lays its contents out.
        d.set("where", json!(""));
        d.step(InputFrame::default());
        assert_eq!(far_h(d), 150.0);
        assert_eq!(d.get("FarIn").height, 150.0);
        // Away again: skipped again.
        d.set("where", json!("away"));
        d.step(InputFrame::default());
        assert_eq!(far_h(d), 10.0);
        assert_eq!(kept_h(d), 150.0);
    });
}

#[test]
fn a_resize_handle_drags_the_size_onto_the_element() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="Both"><div id="BothIn"></div></div>
  <div id="Across"></div>
  <div id="Visible"></div>
</div></div>
<style>
#Root { display: flex; flex-direction: column; align-items: flex-start; gap: 10px; }
#Both { width: 100px; height: 60px; overflow: auto; resize: both; max-width: 130px; }
#Across { width: 100px; height: 60px; overflow: hidden; resize: horizontal; }
#Visible { width: 100px; height: 60px; resize: both; }
</style>"#;
    both(text, &[], |d| {
        d.step(InputFrame::default());
        let corner = |d: &Doc, id: &str| {
            let e = d.get(id);
            og_core::glam::Vec2::new(e.x + e.width - 3.0, e.y + e.height - 3.0)
        };
        let drag = |d: &mut Doc, at: og_core::glam::Vec2, by: og_core::glam::Vec2| {
            let at_input = |pointer, pressed, down, released| InputFrame {
                pointer,
                pointer_inside: true,
                pressed,
                down,
                released,
                ..InputFrame::default()
            };
            d.step(at_input(at, false, false, false));
            d.step(at_input(at, true, true, false));
            d.step(at_input(at + by * 0.5, false, true, false));
            d.step(at_input(at + by, false, true, false));
            d.step(at_input(at + by, false, false, true));
            d.step(InputFrame::default());
        };
        let wh = |d: &Doc, id: &str| {
            let e = d.get(id);
            (e.width, e.height)
        };
        let at = corner(d, "Both");
        d.step(InputFrame {
            pointer: at,
            pointer_inside: true,
            ..InputFrame::default()
        });
        assert_eq!(d.ui.frame().cursor, og_ui::Cursor::ResizeNwse);
        drag(d, at, og_core::glam::Vec2::new(60.0, 25.0));
        // Its width held by max-width; its height as dragged.
        assert_eq!(wh(d, "Both"), (130.0, 85.0));
        // It stays when built again with nothing held.
        d.step(InputFrame::default());
        assert_eq!(wh(d, "Both"), (130.0, 85.0));
        let at = corner(d, "Across");
        drag(d, at, og_core::glam::Vec2::new(20.0, 30.0));
        assert_eq!(wh(d, "Across"), (120.0, 60.0), "across only");
        // With overflow visible, resize does not apply.
        let at = corner(d, "Visible");
        drag(d, at, og_core::glam::Vec2::new(20.0, 30.0));
        assert_eq!(wh(d, "Visible"), (100.0, 60.0));
    });
}
