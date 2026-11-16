//! The element model and motion (docs/ui.md §2, §3.1), driven tick by tick.
//!
//! Ways these could fail, each covered below:
//! - a transform moves the drawing but not the hit test or the probe's
//!   rectangle (a rotated card that answers clicks where it used to be);
//! - a retargeted spring restarts from rest or jumps to a fresh curve;
//! - a state change snaps instead of moving, or uses the wrong pair's
//!   transition;
//! - a stagger over many targets spreads past its cap;
//! - a leaving element is still clickable or in the accessibility tree, or
//!   never stops drawing;
//! - a layout change jumps instead of moving from the old rectangle (FLIP);
//! - a shared element starts at its own rectangle instead of the one it
//!   left on the screen before;
//! - a flick settles on the nearest snap point instead of the projected one;
//! - a restored snapshot continues differently from the original run;
//! - grid tracks, stacks and the overlap report place things wrongly;
//! - an unchanged frame lays out or rebuilds commands again.

use crate::support::Doc;
use og_ui::*;

fn vp() -> Viewport {
    Viewport {
        width: 800.0,
        height: 600.0,
        dpr: 1.0,
        coarse_pointer: false,
        ..Viewport::default()
    }
}

/// `text` mounted and laid out at [`vp`]; its root's path is the mount's,
/// `test:t`.
fn doc(text: &str) -> Doc {
    let mut d = Doc::new(text, &[]);
    d.viewport = vp();
    d.step(InputFrame::default());
    d
}

fn rect(ui: &Ui, path: &str) -> [f32; 4] {
    let e = ui
        .element(path)
        .unwrap_or_else(|| panic!("no element {path}"));
    [e.x, e.y, e.width, e.height]
}
#[test]
fn shared_curves_drive_ui_channels_and_survive_restore() {
    use og_tween::StepPosition::*;
    let cases = [
        (Ease::QuadIn, 0.015625, 0.140625),
        (Ease::QuadOut, 0.234375, 0.609375),
        // x=u³, y=1−(1−u)³: x=1/8 gives u=1/2 and y=7/8.
        // At x=3/8, 1−(1−cbrt(3/8))³ = 0.9783114881721845.
        (Ease::CubicBezier([0.0, 1.0, 0.0, 1.0]), 0.875, 0.9783115),
        (
            Ease::Steps {
                count: 3,
                position: JumpEnd,
            },
            0.0,
            1.0 / 3.0,
        ),
        (
            Ease::Steps {
                count: 3,
                position: JumpStart,
            },
            1.0 / 3.0,
            2.0 / 3.0,
        ),
        (
            Ease::Steps {
                count: 3,
                position: JumpNone,
            },
            0.0,
            0.5,
        ),
        (
            Ease::Steps {
                count: 3,
                position: JumpBoth,
            },
            0.25,
            0.5,
        ),
    ];
    for (ease, eighth, three_eighths) in cases {
        let mut ui = Ui::new(UiDesc::default());
        let tick = |ui: &mut Ui, target| {
            ui.begin(InputFrame::default(), vp());
            let value = ui.animate(
                "value",
                target,
                Transition::Tween {
                    duration_ms: 800.0,
                    ease,
                },
            );
            ui.end();
            value
        };
        tick(&mut ui, 0.0);
        tick(&mut ui, 1.0);
        let mut value = 0.0;
        for _ in 0..6 {
            value = tick(&mut ui, 1.0);
        }
        assert!((value - eighth).abs() < 1e-6, "{ease:?}: {value} at 1/8");
        let saved = ui.save_state();
        let continued: Vec<_> = (0..12).map(|_| tick(&mut ui, 1.0)).collect();
        assert!(
            (continued[11] - three_eighths).abs() < 1e-6,
            "{ease:?}: {} at 3/8",
            continued[11]
        );
        ui.restore_state(&saved).unwrap();
        let replayed: Vec<_> = (0..12).map(|_| tick(&mut ui, 1.0)).collect();
        assert_eq!(continued, replayed, "{ease:?} resumes bit-identically");
    }
}

/// A card stacking its art and a cost gem, and a tag placed over the card
/// by its insets. The engine's own declared overlaps (a list's marker on
/// its item), which a document cannot write, are checked where they are
/// found (`element::frame`'s tests).
#[test]
fn a_stack_overlays_and_undeclared_overlap_is_reported() {
    let d = doc(r#"<meta name="component" content="stack">
<div id="P">
  <stack id="Card"><div id="Art"></div><div id="Gem"></div></stack>
  <div id="Tag"></div>
</div>
<style>
#P { padding: 100px; }
#Card { width: 120px; height: 160px; }
#Art, #Gem, #Tag { background: #808080; }
#Art { width: 120px; height: 160px; }
/* The cost gem sits on the art's corner. */
#Gem { position: absolute; left: -8px; top: -8px; width: 24px; height: 24px; }
/* A label pushed over the card by its inset: a bug. */
#Tag { position: absolute; left: 100px; top: 150px; width: 40px; height: 20px; }
</style>
"#);
    let ui = &d.ui;
    assert_eq!(rect(ui, "test:t/Card/Art"), [100.0, 100.0, 120.0, 160.0]);
    assert_eq!(rect(ui, "test:t/Card/Gem"), [92.0, 92.0, 24.0, 24.0]);
    // Insets are from the parent's padding box, as CSS places them.
    assert_eq!(rect(ui, "test:t/Tag"), [100.0, 150.0, 40.0, 20.0]);
    let mut all: Vec<(String, String)> = ui
        .overlaps()
        .iter()
        .map(|o| (o.path.clone(), o.other.clone()))
        .collect();
    all.sort();
    assert!(
        all.contains(&("test:t/Card/Gem".into(), "test:t/Card/Art".into())),
        "{all:?}"
    );
    assert!(all.iter().any(|(a, _)| a == "test:t/Tag"), "{all:?}");
}

/// Siblings in a row's or a column's flow moved over each other (here by
/// a transform) are reported pair by pair, in the children's order. Wrong
/// implementations this catches: only neighbours compared (the moved
/// first child meets the fourth and fifth, not the second), a sweep along
/// the cross axis that stops early, the pairs in another order.
#[test]
fn siblings_moved_over_each_other_in_a_flow_are_reported_in_order() {
    let d = doc(r#"<meta name="component" content="flow">
<column id="Root">
  <column id="Col"><div id="A0"></div><div id="A1"></div><div id="A2"></div><div id="A3"></div><div id="A4"></div></column>
  <row id="Row"><div id="B0"></div><div id="B1"></div><div id="B2"></div><div id="B3"></div></row>
</column>
<style>
#Root { align-items: start; }
#Col > div, #Row > div { width: 100px; height: 40px; background: #808080; }
/* Rows at y 0, 40, … 160; the first moved to 130..170. */
#A0 { translate: 0 130px; }
/* Cells at x 0, 100, 200, 300; the first moved to 220..320. */
#B0 { translate: 220px 0; }
</style>
"#);
    let shown: Vec<(String, String, bool)> =
        d.ui.overlaps()
            .iter()
            .map(|o| (o.path.clone(), o.other.clone(), o.in_flow))
            .collect();
    let pair = |a: &str, b: &str| (format!("test:t/{a}"), format!("test:t/{b}"), true);
    assert_eq!(
        shown,
        [
            pair("Col/A3", "Col/A0"),
            pair("Col/A4", "Col/A0"),
            pair("Row/B2", "Row/B0"),
            pair("Row/B3", "Row/B0"),
        ]
    );
}

#[test]
fn a_badge_scrolled_out_of_its_list_overlaps_nothing_outside_it() {
    // A count badge low on a tall item: below the list's view when the
    // list is short, over the bar that follows the list.
    let pairs = |d: &Doc| {
        d.ui.overlaps()
            .iter()
            .map(|o| (o.path.clone(), o.other.clone()))
            .collect::<Vec<_>>()
    };
    // The list shows 0..100 and the bar sits at 100..150: the badge at
    // 110..130 is scrolled out of view, so it is over nothing on screen.
    let d = doc(r#"<meta name="component" content="scrolled">
<column id="P">
  <div id="List"><div id="Item"><div id="Count"></div></div></div>
  <div id="Bar"></div>
</column>
<style>
#P { align-items: start; }
#List { width: 200px; height: 100px; overflow-y: auto; }
#Item { width: 200px; height: 300px; }
#Count, #Bar { background: #808080; }
#Count { position: absolute; left: 10px; top: 110px; width: 30px; height: 20px; }
#Bar { width: 200px; height: 50px; }
</style>
"#);
    let count = "test:t/List/Item/Count";
    let hidden = pairs(&d);
    assert!(
        !hidden.iter().any(|(a, b)| a == count || b == count),
        "{hidden:?}"
    );
    // Without a scroll around it the same badge over the bar is reported.
    let d = doc(r#"<meta name="component" content="unscrolled">
<column id="Q">
  <div id="Item"><div id="Count"></div></div>
  <div id="Bar"></div>
</column>
<style>
#Q { align-items: start; }
#Item { width: 200px; height: 100px; }
#Count, #Bar { background: #808080; }
#Count { position: absolute; left: 10px; top: 90px; width: 30px; height: 20px; }
#Bar { width: 200px; height: 50px; }
</style>
"#);
    let shown = pairs(&d);
    assert!(
        shown
            .iter()
            .any(|(a, b)| a == "test:t/Item/Count" && b == "test:t/Bar"),
        "{shown:?}"
    );
}

/// CSS Easing 2's `ease` (`cubic-bezier(0.25, 0.1, 0.25, 1)`) at `x`,
/// solved by bisection on the curve's x: independent of og-tween's solver.
fn css_ease(x: f64) -> f64 {
    let (x1, y1, x2, y2) = (0.25, 0.1, 0.25, 1.0);
    let bez = |a: f64, b: f64, u: f64| {
        3.0 * a * u * (1.0 - u).powi(2) + 3.0 * b * u * u * (1.0 - u) + u.powi(3)
    };
    let (mut lo, mut hi) = (0.0, 1.0);
    for _ in 0..60 {
        let mid = (lo + hi) / 2.0;
        if bez(x1, x2, mid) < x {
            lo = mid
        } else {
            hi = mid
        }
    }
    bez(y1, y2, (lo + hi) / 2.0)
}

/// CSS Transitions 1 and 2 and CSS Easing 1 and 2, sampled tick by tick
/// (60 a second) against the spec's curves worked out here: `linear`,
/// `ease` (solved above, not by og-tween), `steps(4, jump-start)`,
/// `steps(2, jump-both)` (three levels), `linear()` with a placed point,
/// a delay; `transition-behavior: allow-discrete` holding `display` and
/// `content-visibility` shown to the end where `normal` flips at once;
/// `@starting-style` as an entrance's before-change style; and
/// `animation-composition` combining a keyframe `scale(3)` with the
/// underlying `scale(2)` (Web Animations 1 §5.4, CSS Transforms 2 §16:
/// replace 3, add 2·3, accumulate 2 + 3 − 1). A checkpoint taken mid-way
/// replays exactly. Widths are laid out on whole pixels, hence ±0.5.
/// Ways it could fail: a curve's ends swapped or its control points read
/// in another order, a jump at the wrong edge of a step, a `linear()`
/// point's input ignored, the delay counted into the duration, a discrete
/// property flipping half-way (or holding without allow-discrete), the
/// entrance starting from the final style, composite modes conflated, or
/// motion that reads a clock.
#[test]
fn transitions_and_easings_sample_the_specs_curves_by_the_tick() {
    let text = r#"<meta name="component" content="t">
<div id="Page" class={$mode}>
  <div id="Lin"></div><div id="Ease"></div><div id="Steps"></div><div id="Both"></div><div id="Points"></div><div id="Late"></div>
  <div id="Gone"><div id="GoneKid"></div></div><div id="Snap"><div id="SnapKid"></div></div>
  <div id="Fold"><button id="FoldKid">Kid</button></div>
  <div id="Rep" class="grow"></div><div id="Add" class="grow"></div><div id="Acc" class="grow"></div>
  {#if $shown}<div id="In"></div>{/if}
</div>
<style>
#Page { display: flex; flex-direction: column; align-items: start; }
#Page > div { height: 10px; width: 0px; }
#Lin { transition: width 1000ms linear; }
#Ease { transition: width 1000ms; }
#Steps { transition: width 400ms steps(4, jump-start); }
#Both { transition: width 600ms steps(2, jump-both); }
#Points { transition: width 1000ms linear(0, 0.25 75%, 1); }
#Late { transition: width 500ms linear 100ms; }
#Page.on > div { width: 600px; }
#Page > #Gone, #Page > #Snap { display: block; }
#Gone { transition: display 200ms allow-discrete; }
#Snap { transition: display 200ms; }
#Page.on > #Gone, #Page.on > #Snap { display: none; }
#Page > #Fold { content-visibility: visible; transition: content-visibility 200ms allow-discrete; }
#Page.on > #Fold { content-visibility: hidden; }
#GoneKid, #SnapKid, #FoldKid { width: 10px; height: 10px; }
#Page > .grow, #Page.on > .grow { margin: 20px 100px; width: 10px; transform: scale(2); animation: grow 1s linear infinite; }
#Page > #Add { animation-composition: add; }
#Page > #Acc { animation-composition: accumulate; }
@keyframes grow { from, to { transform: scale(3); } }
#Page > #In { width: 100px; transition: width 1000ms linear; }
@starting-style { #Page > #In { width: 0px; } }
</style>"#;
    let mut d = Doc::new(
        text,
        &[
            ("mode", serde_json::json!("off")),
            ("shown", serde_json::json!(false)),
        ],
    );
    type Curve = (&'static str, fn(f64) -> f64);
    let curves: [Curve; 6] = [
        ("Lin", |t| (t / 1000.0).min(1.0)),
        ("Ease", |t| css_ease((t / 1000.0).min(1.0))),
        // jump-start: floor(4p) + 1 levels of 4 from the first instant.
        ("Steps", |t| {
            ((4.0 * (t / 400.0)).floor() + 1.0).min(4.0) / 4.0
        }),
        // jump-both: floor(2p) + 1 of 3 levels, 1 at the end.
        ("Both", |t| {
            if t >= 600.0 {
                1.0
            } else {
                ((2.0 * t / 600.0).floor() + 1.0) / 3.0
            }
        }),
        ("Points", |t| {
            let p = (t / 1000.0).min(1.0);
            if p < 0.75 {
                p / 3.0
            } else {
                0.25 + (p - 0.75) / 0.25 * 0.75
            }
        }),
        ("Late", |t| ((t - 100.0) / 500.0).clamp(0.0, 1.0)),
    ];
    // `set` steps twice: the change is seen on the first (progress 0),
    // so the next read is one tick in.
    d.set("mode", serde_json::json!("on"));
    for (id, scaled) in [("Rep", 30.0), ("Add", 60.0), ("Acc", 40.0)] {
        assert_eq!(d.get(id).width, scaled, "{id}'s composite");
    }
    let mut saved = None;
    let mut sampled = Vec::new();
    for tick in 1..=64 {
        let ms = tick as f64 * 1000.0 / 60.0;
        for (id, curve) in curves {
            let (w, want) = (d.get(id).width as f64, 600.0 * curve(ms));
            assert!(
                (w - want).abs() <= 0.5 + 1e-3,
                "{id} at tick {tick}: {w} vs {want}"
            );
        }
        // 200 ms is 12 ticks: allow-discrete keeps the shown value until
        // the end; `normal` flipped on the change.
        let shown = tick < 12;
        assert_eq!(d.el("GoneKid").is_some(), shown, "display at tick {tick}");
        let folded = !d
            .ui
            .a11y()
            .nodes
            .iter()
            .any(|n| n.path.ends_with("/FoldKid"));
        assert_eq!(folded, !shown, "content-visibility at tick {tick}");
        assert!(d.el("SnapKid").is_none(), "normal display flips at once");
        if tick == 20 {
            saved = Some(d.ui.save_state());
        }
        if tick > 20 {
            sampled.push(d.ui.frame().items.clone());
        }
        d.step(InputFrame::default());
    }
    d.ui.restore_state(&saved.unwrap()).unwrap();
    for (k, expected) in sampled.iter().enumerate() {
        d.step(InputFrame::default());
        assert_eq!(&d.ui.frame().items, expected, "replay {k}");
    }
    // An entrance starts from its @starting-style.
    d.set("shown", serde_json::json!(true));
    for tick in 1..=6 {
        let want = 100.0 * tick as f32 / 60.0;
        let w = d.get("In").width;
        assert!(
            (w - want).abs() <= 0.5 + 1e-3,
            "entrance at tick {tick}: {w} vs {want}"
        );
        d.step(InputFrame::default());
    }
}

/// CSS View Transitions 1 and 2 across a navigation (a layer replacing
/// another): a named element's group moves from its old border box to its
/// new one, every tick on the box the spec's interpolation gives
/// (`lerp(old, new, easing(t / duration))`), both images drawn there
/// until it ends. `view-transition-class: card` picks up
/// `::view-transition-group(*.card)`'s 400 ms linear timing; without the
/// class the group takes the UA's 0.25s `ease`; without the
/// `@view-transition { navigation: auto }` opt-in nothing transitions.
/// Ways it could fail: the group starting on the new box, a class rule
/// applied to an unclassed group (or not to a classed one), the default
/// timing not the UA's, a transition without the opt-in, or one image left
/// behind after it ends.
#[test]
fn view_transitions_morph_a_named_element_by_its_groups_timing() {
    let red = |ui: &Ui| -> Vec<[f32; 4]> {
        ui.frame()
            .items
            .iter()
            .filter_map(|it| match &it.cmd {
                DrawCmd::Quad { rect, color, .. } if color[0] > 0.9 && color[1] < 0.05 => {
                    Some([rect.x, rect.y, rect.width, rect.height])
                }
                _ => None,
            })
            .collect()
    };
    let (old, new) = ([40.0, 40.0, 100.0, 50.0], [300.0, 200.0, 200.0, 100.0]);
    let run = |opt_in: bool, class: &str| {
        let at = if opt_in {
            "@view-transition { navigation: auto; }"
        } else {
            ""
        };
        let list = format!(
            r#"<meta name="screen" content="list"><column id="Page"><column id="Card"></column></column>
<style>{at} #Page {{ padding: 40px; }}
#Card {{ width: 100px; height: 50px; background: #ff0000; view-transition-name: hero; }}</style>"#
        );
        let detail = format!(
            r#"<meta name="screen" content="detail"><column id="Page"><column id="Card"></column></column>
<style>{at} #Page {{ padding: 200px 0 0 300px; }}
#Card {{ width: 200px; height: 100px; background: #ff0000; view-transition-name: hero; view-transition-class: {class}; }}
::view-transition-group(*.card) {{ animation-duration: 400ms; animation-timing-function: linear; }}</style>"#
        );
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(UiDocument::parse("list.svelte", &list).unwrap());
        ui.define_document(UiDocument::parse("detail.svelte", &detail).unwrap());
        let step = |ui: &mut Ui| {
            ui.begin(InputFrame::default(), Viewport::new(640.0, 480.0));
            ui.end();
        };
        ui.push_screen("list", serde_json::Value::Null).unwrap();
        for _ in 0..30 {
            step(&mut ui);
        }
        ui.replace_screen("detail", serde_json::Value::Null)
            .unwrap();
        (0..30)
            .map(|_| {
                step(&mut ui);
                red(&ui)
            })
            .collect::<Vec<_>>()
    };
    let check = |seen: &[Vec<[f32; 4]>], ms: f64, ease: fn(f64) -> f64| {
        for (k, boxes) in seen.iter().enumerate() {
            let t = (k as f64 * 1000.0 / 60.0 / ms).min(1.0);
            let p = ease(t) as f32;
            let want: [f32; 4] = std::array::from_fn(|i| old[i] + (new[i] - old[i]) * p);
            let images = if t > 0.0 && t < 1.0 { 2 } else { 1 };
            assert_eq!(boxes.len(), images, "tick {k}: {boxes:?}");
            for b in boxes {
                let close = b.iter().zip(want).all(|(a, w)| (a - w).abs() < 1e-3);
                assert!(close, "tick {k}: {b:?} vs {want:?}");
            }
        }
    };
    check(&run(true, "card"), 400.0, |t| t);
    check(&run(true, "none"), 250.0, css_ease);
    for (k, boxes) in run(false, "card").iter().enumerate() {
        assert_eq!(boxes, &[new], "no opt-in: tick {k}");
    }
}

/// CSS Transitions over two value types the spec makes animatable, each
/// checked every tick against its own interpolation (60 ticks a second,
/// 600 ms linear, so t = tick / 36): `grid-template-columns` track by
/// track (CSS Grid 2 §7.2.5: 100px 200px → 200px 100px), `clip-path`
/// `inset()` by its offsets (CSS Masking 1 §5.1: 0 → 50px, read through
/// where the cursor applies). Ways it could fail: tracks swapped or
/// jumping at 50%, or the clip interpolating its box instead of its insets
/// (or flipping discretely).
#[test]
fn grid_tracks_and_clip_paths_transition_by_their_values() {
    let text = r#"<meta name="component" content="t">
<div id="Page" class={$mode}>
  <div id="Grid"><div id="G1"></div><div id="G2"></div></div>
  <div id="Clip"></div>
</div>
<style>
#Page { position: relative; width: 800px; height: 600px; cursor: help; }
#Grid { display: grid; width: 300px; grid-template-columns: 100px 200px; transition: grid-template-columns 600ms linear; }
#Page.on > #Grid { grid-template-columns: 200px 100px; }
#G1, #G2 { height: 10px; }
#Clip { position: absolute; left: 100px; top: 100px; width: 100px; height: 100px; cursor: copy; clip-path: inset(0px); transition: clip-path 600ms linear; }
#Page.on > #Clip { clip-path: inset(50px); }
</style>"#;
    let mut d = Doc::new(text, &[("mode", serde_json::json!("off"))]);
    d.set("mode", serde_json::json!("on"));
    // The clip's edge at tick k is 50·t in from each side: a point 20 px in
    // is inside it until t = 0.4 (tick 14.4). The cursor hit-tests the
    // frame drawn before the step, as every og-ui hit does.
    let probe = og_core::glam::Vec2::new(120.0, 150.0);
    for tick in 1..=40 {
        let t = (tick as f32 / 36.0).min(1.0);
        let g1 = d.get("G1").width;
        assert!(
            (g1 - (100.0 + 100.0 * t)).abs() <= 0.5 + 1e-3,
            "track at {tick}: {g1}"
        );
        d.step(InputFrame::hover(probe.x, probe.y));
        let inside = 50.0 * tick as f32 / 36.0 < 20.0;
        let edge = (50.0 * tick as f32 / 36.0 - 20.0).abs() < 0.5;
        if !edge {
            let want = if inside { Cursor::Copy } else { Cursor::Help };
            assert_eq!(d.ui.frame().cursor, want, "clip at {tick}");
        }
    }
}

/// CSS Anchor Positioning 1 §2.4: `anchor()` interpolates by the length it
/// resolves to. A tag transitioning (600 ms linear) from `left:
/// anchor(--a right)` (x 140) to `anchor(--b right)` (x 440) is at
/// 140 + 300t every tick, not at one anchor then the other; on the
/// right edge (`right: anchor(--a left)` to `anchor(--b left)`) it moves
/// the same way from the far side; `anchor-size()` does the same for a
/// width (40 → 100 px); and it keeps following an anchor that moves
/// mid-way. Ways it could fail: the pair flipping at 50% (or at
/// once), the weights swapped, the far side's distance not mirrored, or
/// the mix frozen at the anchors' first boxes.
#[test]
fn anchor_insets_transition_by_their_resolved_lengths() {
    let text = r#"<meta name="component" content="t">
<div id="Page" class={$mode}>
  <div id="A" class={$moved}></div><div id="B"></div><div id="C"></div><div id="Tag"></div><div id="Far"></div><div id="Wide"></div>
</div>
<style>
#Page { position: relative; width: 800px; height: 600px; }
#A, #B { position: absolute; top: 300px; width: 40px; height: 20px; }
#A { left: 100px; anchor-name: --a; }
#A.yes { left: 160px; }
#B { left: 400px; anchor-name: --b; }
#C { position: absolute; top: 350px; left: 600px; width: 100px; height: 20px; anchor-name: --c; }
#Wide { position: absolute; top: 500px; left: 0; height: 10px; width: anchor-size(--a width); transition: width 600ms linear; }
#Page.on > #Wide { width: anchor-size(--c width); }
#Tag, #Far { position: absolute; width: 10px; height: 10px; transition: left 600ms linear, right 600ms linear; }
#Tag { top: 400px; left: anchor(--a right); }
#Far { top: 450px; right: anchor(--a left); }
#Page.on > #Tag { left: anchor(--b right); }
#Page.on > #Far { right: anchor(--b left); }
</style>"#;
    let mut d = Doc::new(
        text,
        &[
            ("mode", serde_json::json!("off")),
            ("moved", serde_json::json!("no")),
        ],
    );
    assert_eq!(d.get("Tag").x, 140.0);
    // `right: anchor(--a left)` puts its right edge on x 100.
    assert_eq!(d.get("Far").x + 10.0, 100.0);
    d.set("mode", serde_json::json!("on"));
    for tick in 1..=18 {
        let t = tick as f32 / 36.0;
        let (tag, far) = (d.get("Tag").x, d.get("Far").x + 10.0);
        let wide = d.get("Wide").width;
        assert!(
            (wide - (40.0 + 60.0 * t)).abs() <= 0.5 + 1e-3,
            "anchor-size at {tick}: {wide}"
        );
        assert!(
            (tag - (140.0 + 300.0 * t)).abs() <= 0.5 + 1e-3,
            "left at {tick}: {tag}"
        );
        assert!(
            (far - (100.0 + 300.0 * t)).abs() <= 0.5 + 1e-3,
            "right at {tick}: {far}"
        );
        d.step(InputFrame::default());
    }
    // Half-way, --a moves 60 px right: the mix follows its new edge
    // (weight 1 − t on it), as the resolved length does.
    d.set("moved", serde_json::json!("yes"));
    for tick in 21..=40 {
        let t = (tick as f32 / 36.0).min(1.0);
        let tag = d.get("Tag").x;
        let want = (1.0 - t) * 200.0 + t * 440.0;
        assert!(
            (tag - want).abs() <= 0.5 + 1e-3,
            "moved anchor at {tick}: {tag} vs {want}"
        );
        d.step(InputFrame::default());
    }
}
