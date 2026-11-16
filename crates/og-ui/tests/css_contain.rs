//! CSS Containment 2 in documents (docs/ui.md §2): `contain`,
//! `content-visibility` and `contain-intrinsic-size`, and CSS UI 4's
//! `resize`, built mounted.
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
//! - boxes painted in source order whatever their `z-index`, or `contain:
//!   paint`, `overflow: clip` and `strict`/`content` not clipping, or
//!   `overflow: overlay` not scrolling as `auto` →
//!   `z_index_orders_paint_and_clip_and_contain_cut_it`

use crate::support::{Doc, with_doc};
use og_ui::{DrawCmd, InputFrame};
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
    with_doc(text, &[], |d| {
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
    with_doc(text, &[("where", json!("away"))], |d| {
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
    with_doc(text, &[], |d| {
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

/// The paint rows of docs/css-coverage.md's CSS checklist: `z-index` orders
/// positioned boxes by stack level, negative under `auto` under positive
/// whatever their source order (CSS 2 §9.9.1, Appendix E); `contain:
/// paint` and `overflow: clip` clip what their subtree draws to their box
/// (CSS Containment 2 §3.4, CSS Overflow 3 §3.1), and `overflow: clip`
/// makes no scroll container (a wheel moves nothing); `contain: strict` is
/// size and paint containment, `content` paint without size, `none`
/// neither (CSS Containment 2 §2).
///
/// Ways it could fail: boxes painted in source order; a clip missing or
/// at the content's size; `overflow: clip` scrolling like `hidden` does
/// by script; `strict` or `content` read as one of their parts only;
/// `overflow: overlay` refused or read as visible.
#[test]
fn z_index_orders_paint_and_clip_and_contain_cut_it() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="Zs"><div id="Za"></div><div id="Zb"></div><div id="Zc"></div></div>
  <div id="Cp"><div class="big"></div></div>
  <div id="Oc"><div class="big"></div></div>
  <div id="Strict"><div class="big"></div></div>
  <div id="Content"><div class="big"></div></div>
  <div id="None"><div class="big"></div></div>
  <div id="Ov"><div class="big"></div></div>
  <div id="OvY"><div class="big"></div></div>
</div></div>
<style>
#Root { width: 400px; }
#Zs { position: relative; height: 40px; }
#Za, #Zb, #Zc { position: absolute; left: 0; top: 0; width: 30px; height: 30px; }
#Za { z-index: 2; background: #ff0000; }
#Zb { z-index: -1; background: #00ff00; }
#Zc { z-index: auto; background: #0000ff; }
#Cp { contain: paint; width: 50px; height: 20px; }
#Oc { overflow: clip; width: 60px; height: 20px; }
#Strict { contain: strict; width: 70px; }
#Content { contain: content; width: 80px; }
#None { contain: none; width: 90px; }
#Ov { overflow: overlay; width: 40px; height: 40px; }
#OvY { overflow-y: overlay; width: 40px; height: 40px; }
.big { width: 300px; height: 300px; background: #123456; }
</style>"#;
    with_doc(text, &[], |d| {
        let rgb = |c: [f32; 4]| [c[0], c[1], c[2]];
        let order: Vec<_> = d
            .ui
            .frame()
            .items
            .iter()
            .filter_map(|i| i.cmd.filled())
            .map(|(_, c)| rgb(c))
            .filter(|c| c.iter().all(|v| *v == 0.0 || *v == 1.0) && c.iter().sum::<f32>() == 1.0)
            .collect();
        assert_eq!(
            order,
            [[0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]],
            "z-index -1, then auto, then 2"
        );
        let clips: Vec<(f32, f32, f32, f32)> =
            d.ui.frame()
                .items
                .iter()
                .filter_map(|i| match &i.cmd {
                    DrawCmd::PushClip { center, half, .. } => Some((
                        center.0 - half.0,
                        center.1 - half.1,
                        half.0 * 2.0,
                        half.1 * 2.0,
                    )),
                    _ => None,
                })
                .collect();
        let clips_to = |id: &str| {
            let e = d.get(id);
            clips.contains(&(e.x, e.y, e.width, e.height))
        };
        for id in ["Cp", "Oc", "Strict", "Content"] {
            assert!(clips_to(id), "{id} clips to its box: {clips:?}");
        }
        assert!(!clips_to("None"), "contain: none clips nothing");
        assert_eq!(
            (d.get("Strict").width, d.get("Strict").height),
            (70.0, 0.0),
            "strict sizes as if empty"
        );
        assert_eq!(
            d.get("Content").height,
            300.0,
            "content keeps its content's height"
        );
        assert_eq!(d.get("None").height, 300.0);
        let oc = d.get("Oc");
        assert!(
            d.ui.scroll_state(&oc.path).is_none(),
            "overflow: clip is no scroll container"
        );
    });
}

/// Stacking contexts (CSS 2 §9.9.1, Appendix E; CSS Positioned Layout 3
/// §4.1): a box with a stack level paints in its nearest ancestor that
/// makes a stacking context, so `z-index: 1` lifts it over its parent's
/// later sibling and `z-index: -1` puts it under its parent's own
/// background; a parent that makes a context keeps both inside it
/// (positive over its own paint, negative between its background and its
/// in-flow content), under that later sibling. The parent makes one with
/// a stack level, opacity under 1, a transform, a filter, a mask, a clip
/// path, a blend mode, `isolation: isolate`, `will-change` naming one of
/// those, or paint containment.
///
/// Ways it could fail: children ordered among their siblings only (a
/// lifted child under the parent's later sibling); a negative child drawn
/// over its parent's background; a trigger that lets its children out.
#[test]
fn z_index_paints_in_the_nearest_stacking_context() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="P" class={$p}><div id="K" class={$k} onpointerdown={k}></div></div>
  <div id="Q" onpointerdown={q}></div>
</div></div>
<style>
#Root { position: relative; width: 200px; height: 100px; }
#P, #K, #Q { position: absolute; left: 0; top: 0; width: 40px; height: 40px; }
#P { background: #0000ff; }
#K { background: #ff0000; }
#Q { background: #00ff00; }
.up { z-index: 1; }
.down { z-index: -1; }
.level { z-index: 2; }
.opacity { opacity: 0.5; }
.transform { transform: translateX(1px); }
.filter { filter: blur(1px); }
.mask { mask-image: linear-gradient(#000, #000); }
.clip { clip-path: inset(1px); }
.blend { mix-blend-mode: multiply; }
.isolate { isolation: isolate; }
.will { will-change: opacity; }
.paint { contain: paint; }
</style>"#;
    // Each case from the last one's frame: the drawing order is made again
    // where a level or a context comes or goes.
    fn order(d: &mut Doc, p: &str, k: &str) -> Vec<&'static str> {
        d.set("p", json!(p));
        d.set("k", json!(k));
        let name = |c: [f32; 4]| match [c[0], c[1], c[2]] {
            [1.0, 0.0, 0.0] => Some("K"),
            [0.0, 0.0, 1.0] => Some("P"),
            [0.0, 1.0, 0.0] => Some("Q"),
            _ => None,
        };
        d.ui.frame()
            .items
            .iter()
            .filter_map(|i| i.cmd.filled())
            .filter_map(|(_, c)| name(c))
            .collect()
    }
    // The box a click over all three reaches: the one drawn on top.
    fn hit(d: &mut Doc) -> Vec<String> {
        let events = d.click_at(og_core::glam::Vec2::new(20.0, 20.0));
        events.into_iter().map(|e| e.name).collect()
    }
    let stores = [("p", json!("")), ("k", json!("up"))];
    with_doc(text, &stores, |d| {
        assert_eq!(
            order(d, "", "up"),
            ["P", "Q", "K"],
            "lifted over P's later sibling"
        );
        assert_eq!(hit(d), ["k"], "and hit first");
        assert_eq!(
            order(d, "", "down"),
            ["K", "P", "Q"],
            "under P's own background"
        );
        assert_eq!(hit(d), ["q"]);
        // With a level of its own P paints over Q, K still inside it.
        assert_eq!(order(d, "level", "up"), ["Q", "P", "K"]);
        assert_eq!(order(d, "level", "down"), ["Q", "P", "K"]);
        assert_eq!(hit(d), ["k"]);
        for context in [
            "opacity",
            "transform",
            "filter",
            "mask",
            "clip",
            "blend",
            "isolate",
            "will",
            "paint",
        ] {
            assert_eq!(
                order(d, context, "up"),
                ["P", "K", "Q"],
                "{context} keeps K in"
            );
            assert_eq!(
                order(d, context, "down"),
                ["P", "K", "Q"],
                "{context} keeps K over P"
            );
            assert_eq!(
                order(d, "", "up"),
                ["P", "Q", "K"],
                "{context} gone lets K out"
            );
        }
        order(d, "isolate", "up");
        assert_eq!(hit(d), ["q"], "kept in, K is under Q");
    });
}
