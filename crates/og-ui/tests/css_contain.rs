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
//!   paint`, `overflow: clip` and `strict`/`content` not clipping →
//!   `z_index_orders_paint_and_clip_and_contain_cut_it`
//! - `overflow: overlay` not scrolling as `auto` →
//!   `overflow_overlay_scrolls_as_auto`

use crate::support::{Doc, with_doc};
use og_core::glam::Vec2;
use og_ui::{DrawCmd, InputFrame};
use serde_json::json;

/// A content-based flex basis must not read the container's descendants.
/// Chromium gives Cell its 36px padding plus half the remaining 322px;
/// without containment the cqi-sized text makes its basis grow each pass.
#[test]
fn inline_size_container_in_content_sized_flex_item() {
    let text = r#"<meta name="component" content="t">
<div id="Root"><div id="Top"><div id="Cell"><span id="Score">1234567</span></div><div id="Rank">Rank</div></div></div>
<style>
#Root { width: 358px; container-type: inline-size; }
#Top { display: flex; }
#Rank { flex: 1 1 0; min-width: 0; }
#Cell { container-type: inline-size; flex: 1 1 auto; max-width: stretch; min-width: 0; padding: 12px 18px; }
#Score { --score-size: min(72px, 22cqi); font-size: var(--score-size); white-space: nowrap; }
@container (width < 560px) { #Score { --score-size: min(64px, 20cqi); } }
</style>"#;
    with_doc(text, &[], |d| {
        for _ in 0..3 {
            d.step(InputFrame::default());
            let cell = d.get("Cell");
            let score = d.get("Score");
            assert_eq!(cell.width, 197.0, "{cell:?}");
            assert!(score.height > 0.0 && score.height < 100.0, "{score:?}");
        }
    });
}

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
/// by script; `strict` or `content` read as one of their parts only.
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

/// `overflow: overlay` is a legacy alias of `auto` (CSS Overflow 3 §3.1):
/// `Ov` scrolls both ways, and `OvY`'s `overflow-y: overlay` makes its
/// `visible` x compute to `auto`, so a wheel moves both. `Hidden` beside
/// them, `overflow: hidden`, takes no wheel.
///
/// Ways it could fail: `overlay` refused, read as `visible` (no scroll
/// container) or `hidden` (the wheel moves nothing), or `overflow-y`
/// leaving x visible.
#[test]
fn overflow_overlay_scrolls_as_auto() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="Ov"><div class="big"></div></div>
  <div id="OvY"><div class="big"></div></div>
  <div id="Hidden"><div class="big"></div></div>
</div></div>
<style>
#Root { width: 400px; display: flex; gap: 10px; }
#Ov { overflow: overlay; width: 40px; height: 40px; }
#OvY { overflow-y: overlay; width: 40px; height: 40px; }
#Hidden { overflow: hidden; width: 40px; height: 40px; }
.big { width: 300px; height: 300px; background: #123456; flex: none; }
</style>"#;
    with_doc(text, &[], |d| {
        for id in ["Ov", "OvY", "Hidden"] {
            let e = d.get(id);
            let over = InputFrame::hover(e.x + 10.0, e.y + 10.0);
            d.step(InputFrame {
                wheel: Vec2::new(20.0, 30.0),
                ..over.clone()
            });
            for _ in 0..40 {
                d.step(over.clone());
            }
        }
        for id in ["Ov", "OvY"] {
            let s =
                d.ui.scroll_state(&d.get(id).path)
                    .unwrap_or_else(|| panic!("{id} is a scroll container"));
            // (OvY, scrolling down only by what it wrote, keeps a
            // vertical scroller's bar gutter beside its view.)
            assert_eq!(
                (s.view_height, s.content_width, s.content_height),
                (40.0, 300.0, 300.0),
                "{id}'s view and content"
            );
            assert_eq!(
                (s.offset_x, s.offset_y),
                (20.0, 30.0),
                "{id} took the wheel"
            );
        }
        let h = d.ui.scroll_state(&d.get("Hidden").path);
        assert!(
            h.is_none_or(|s| (s.offset_x, s.offset_y) == (0.0, 0.0)),
            "hidden takes no wheel: {h:?}"
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

/// CSS Overflow 4 §3.2: `overflow-clip-margin` moves the cut of an
/// `overflow: clip` or paint-contained box out from its padding edge, or
/// from the `border-box` or `content-box` it names; `overflow: hidden`
/// keeps its cut where it is.
///
/// Ways it could fail: the margin ignored (every cut on the box), applied
/// to `hidden` too, or measured from the border box whatever box is named
/// (the content-box cut 3px outside the box instead of 5px inside it).
#[test]
fn overflow_clip_margin_moves_the_clip_edge() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="Om" class="o"><div class="big"></div></div>
  <div id="Ob" class="o"><div class="big"></div></div>
  <div id="Oc" class="o"><div class="big"></div></div>
  <div id="Op" class="o"><div class="big"></div></div>
  <div id="Oh" class="o"><div class="big"></div></div>
</div></div>
<style>
#Root { width: 400px; }
.o { width: 60px; height: 20px; margin-bottom: 40px; overflow: clip; }
#Om { overflow-clip-margin: 10px; }
#Ob { border: 4px solid #000; overflow-clip-margin: border-box 2px; }
#Oc { padding: 8px; overflow-clip-margin: 3px content-box; }
#Op { overflow: visible; contain: paint; overflow-clip-margin: 6px; }
#Oh { overflow: hidden; overflow-clip-margin: 10px; }
.big { width: 300px; height: 300px; background: #123456; }
</style>"#;
    with_doc(text, &[], |d| {
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
        let cut = |id: &str, out: f32| {
            let e = d.get(id);
            let want = (
                e.x - out,
                e.y - out,
                e.width + 2.0 * out,
                e.height + 2.0 * out,
            );
            assert!(clips.contains(&want), "{id} cut at {want:?}: {clips:?}");
        };
        cut("Om", 10.0);
        cut("Ob", 2.0);
        cut("Oc", -5.0);
        cut("Op", 6.0);
        cut("Oh", 0.0);
    });
}

/// CSS Will Change 1 §2: `will-change` makes now what a non-initial value
/// of each property it names would make, and nothing more. `transform`
/// (and `filter`, even listed after a property that makes nothing) makes
/// both a stacking context, keeping a z-index: 1 child in, and a
/// containing block that holds a fixed box; `opacity` makes the stacking
/// context but holds no fixed box; `left`, `scroll-position`, `contents`
/// and `auto` make neither. Ways it could fail: every named property read
/// as a layer, a stacking property read as a containing block, a list read
/// by its first entry only, or `auto` read as a property.
#[test]
fn will_change_makes_what_its_properties_would_and_nothing_else() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="P" class={$will}><div id="K"></div><div id="F"></div></div>
  <div id="Q"></div>
</div></div>
<style>
#Root { position: relative; width: 200px; height: 100px; }
#P, #K, #Q { position: absolute; left: 50px; top: 20px; width: 40px; height: 40px; }
#K { left: 0; top: 0; z-index: 1; background: #ff0000; }
#P { background: #0000ff; }
#Q { background: #00ff00; }
#F { position: fixed; left: 3px; top: 4px; width: 5px; height: 5px; }
.transform { will-change: transform; }
.list { will-change: left, filter; }
.opacity { will-change: opacity; }
.left { will-change: left; }
.scroll { will-change: scroll-position, contents; }
.auto { will-change: auto; }
</style>"#;
    let stores = [("will", json!(""))];
    with_doc(text, &stores, |d| {
        for (will, stacks, holds) in [
            ("", false, false),
            ("transform", true, true),
            ("list", true, true),
            ("opacity", true, false),
            ("left", false, false),
            ("scroll", false, false),
            ("auto", false, false),
        ] {
            d.set("will", json!(will));
            let name = |c: [f32; 4]| match [c[0], c[1], c[2]] {
                [1.0, 0.0, 0.0] => Some("K"),
                [0.0, 0.0, 1.0] => Some("P"),
                [0.0, 1.0, 0.0] => Some("Q"),
                _ => None,
            };
            let order: Vec<_> =
                d.ui.frame()
                    .items
                    .iter()
                    .filter_map(|i| i.cmd.filled())
                    .filter_map(|(_, c)| name(c))
                    .collect();
            let want = if stacks {
                ["P", "K", "Q"]
            } else {
                ["P", "Q", "K"]
            };
            assert_eq!(order, want, "{will:?}: stacking context {stacks}");
            let (f, p) = (d.get("F"), d.get("P"));
            let at = if holds {
                (f.x - p.x, f.y - p.y)
            } else {
                (f.x, f.y)
            };
            assert_eq!(at, (3.0, 4.0), "{will:?}: holds the fixed box {holds}");
        }
    });
}

/// CSS Flexbox 1 §5.4 and CSS Grid 2 §9: flex and grid items paint in
/// order-modified document order, so an item `order` moves ahead paints
/// under the siblings it now precedes, and the one drawn on top is hit
/// first. B comes after A in the document; with `order: -1` it paints
/// first. In the flex row each item's `margin-right: -20px` overlaps the
/// next by 20px; in the grid both sit in one cell.
///
/// Ways it could fail: painting in document order (B over A); the order
/// kept from an earlier frame after `order` changes (A over B once B's
/// order is 1, or B over A once it is -1 again); hits following the
/// document instead of the paint.
#[test]
fn order_modified_document_order_paints_and_hits() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root" class={$c}>
  <div id="A" onpointerdown={a}></div>
  <div id="B" class={$b} onpointerdown={b}></div>
</div></div>
<style>
#Root { width: 200px; height: 40px; }
.flex { display: flex; }
.grid { display: grid; grid-template-columns: 40px; }
#A, #B { width: 40px; height: 40px; flex: none; }
.flex #A, .flex #B { margin-right: -20px; }
.grid #A, .grid #B { grid-area: 1 / 1; }
#A { background: #ff0000; }
#B { background: #0000ff; }
.first { order: -1; }
.last { order: 1; }
</style>"#;
    fn top(d: &mut Doc, c: &str, b: &str) -> (Vec<&'static str>, Vec<String>) {
        d.set("c", json!(c));
        d.set("b", json!(b));
        let name = |c: [f32; 4]| match [c[0], c[1], c[2]] {
            [1.0, 0.0, 0.0] => Some("A"),
            [0.0, 0.0, 1.0] => Some("B"),
            _ => None,
        };
        let painted =
            d.ui.frame()
                .items
                .iter()
                .filter_map(|i| i.cmd.filled())
                .filter_map(|(_, c)| name(c))
                .collect();
        // Over the overlap in both layouts.
        let hit = d.click_at(og_core::glam::Vec2::new(30.0, 20.0));
        (painted, hit.into_iter().map(|e| e.name).collect())
    }
    let stores = [("c", json!("flex")), ("b", json!(""))];
    with_doc(text, &stores, |d| {
        for c in ["flex", "grid"] {
            assert_eq!(
                top(d, c, ""),
                (vec!["A", "B"], vec!["b".into()]),
                "{c}: document order"
            );
            assert_eq!(
                top(d, c, "first"),
                (vec!["B", "A"], vec!["a".into()]),
                "{c}: B moved first paints under A, which is hit"
            );
            assert_eq!(
                top(d, c, "last"),
                (vec!["A", "B"], vec!["b".into()]),
                "{c}: and back"
            );
            assert_eq!(
                top(d, c, "first"),
                (vec!["B", "A"], vec!["a".into()]),
                "{c}: again"
            );
        }
    });
}
