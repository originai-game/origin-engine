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
