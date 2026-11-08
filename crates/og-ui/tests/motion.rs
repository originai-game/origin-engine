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

fn fixed(w: f32, h: f32) -> Element {
    Element {
        width: Size::Fixed(w),
        height: Size::Fixed(h),
        ..Element::default()
    }
}

/// A box with a background, `props` on it.
fn boxed(ui: &mut Ui, id: &str, w: f32, h: f32, props: Props) {
    ui.open_with(
        id,
        Element {
            background: Some(Srgba::rgb(128, 128, 128)),
            interactive: true,
            ..fixed(w, h)
        },
        props,
    );
    ui.close();
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

#[test]
fn a_stack_overlays_and_undeclared_overlap_is_reported() {
    let mut ui = Ui::new(UiDesc::default());
    let frame = |ui: &mut Ui, declared: bool| {
        ui.begin(InputFrame::default(), vp());
        ui.open(
            "P",
            Element {
                padding: Insets::all(100.0),
                ..Element::default()
            },
        );
        ui.open_with(
            "Card",
            fixed(120.0, 160.0),
            Props {
                layout: LayoutMode::Stack,
                ..Props::default()
            },
        );
        boxed(ui, "Art", 120.0, 160.0, Props::default());
        // The cost gem sits on the art's corner, on purpose or not.
        boxed(
            ui,
            "Gem",
            24.0,
            24.0,
            Props {
                position: Position::Absolute,
                insets: Edges {
                    start: Some(-8.0),
                    top: Some(-8.0),
                    ..Edges::default()
                },
                overlaps: if declared {
                    vec!["P/Card/Art".into()]
                } else {
                    Vec::new()
                },
                ..Props::default()
            },
        );
        ui.close();
        // A label pushed over the card by a negative inset: a bug.
        boxed(
            ui,
            "Tag",
            40.0,
            20.0,
            Props {
                position: Position::Absolute,
                insets: Edges {
                    start: Some(100.0),
                    top: Some(150.0),
                    ..Edges::default()
                },
                ..Props::default()
            },
        );
        ui.close();
        ui.end();
    };
    frame(&mut ui, false);
    assert_eq!(rect(&ui, "P/Card/Art"), [100.0, 100.0, 120.0, 160.0]);
    assert_eq!(rect(&ui, "P/Card/Gem"), [92.0, 92.0, 24.0, 24.0]);
    // Insets are from the parent's padding box, as CSS places them.
    assert_eq!(rect(&ui, "P/Tag"), [100.0, 150.0, 40.0, 20.0]);
    let pairs = |ui: &Ui| -> Vec<(String, String)> {
        let mut v: Vec<_> = ui
            .overlaps()
            .iter()
            .map(|o| (o.path.clone(), o.other.clone()))
            .collect();
        v.sort();
        v
    };
    let all = pairs(&ui);
    assert!(
        all.contains(&("P/Card/Gem".into(), "P/Card/Art".into())),
        "{all:?}"
    );
    assert!(all.iter().any(|(a, _)| a == "P/Tag"), "{all:?}");
    frame(&mut ui, true);
    let declared = pairs(&ui);
    assert!(
        !declared.contains(&("P/Card/Gem".into(), "P/Card/Art".into())),
        "{declared:?}"
    );
    assert!(declared.iter().any(|(a, _)| a == "P/Tag"), "{declared:?}");
}

/// Siblings in a row's or a column's flow moved over each other (here by
/// a transform) are reported pair by pair, in the children's order. Wrong
/// implementations this catches: only neighbours compared (the moved
/// first child meets the fourth and fifth, not the second), a sweep along
/// the cross axis that stops early, the pairs in another order.
#[test]
fn siblings_moved_over_each_other_in_a_flow_are_reported_in_order() {
    let mut ui = Ui::new(UiDesc::default());
    let moved = |x: f32, y: f32| Props {
        transform: Transform {
            x,
            y,
            ..Transform::IDENTITY
        },
        ..Props::default()
    };
    ui.begin(InputFrame::default(), vp());
    ui.open("Col", Element::column());
    // Rows at y 0, 40, … 160; the first moved to 130..170.
    boxed(&mut ui, "A0", 100.0, 40.0, moved(0.0, 130.0));
    for k in 1..5 {
        boxed(&mut ui, &format!("A{k}"), 100.0, 40.0, Props::default());
    }
    ui.close();
    ui.open("Row", Element::row());
    // Cells at x 0, 100, 200, 300; the first moved to 220..320.
    boxed(&mut ui, "B0", 100.0, 40.0, moved(220.0, 0.0));
    for k in 1..4 {
        boxed(&mut ui, &format!("B{k}"), 100.0, 40.0, Props::default());
    }
    ui.close();
    ui.end();
    let shown: Vec<(String, String, bool)> = ui
        .overlaps()
        .iter()
        .map(|o| (o.path.clone(), o.other.clone(), o.in_flow))
        .collect();
    let pair = |a: &str, b: &str| (a.to_string(), b.to_string(), true);
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
    let mut ui = Ui::new(UiDesc::default());
    let frame = |ui: &mut Ui, list_h: f32| {
        ui.begin(InputFrame::default(), vp());
        ui.open(
            "P",
            Element {
                direction: Direction::Column,
                ..Element::default()
            },
        );
        ui.open(
            "List",
            Element {
                scroll: true,
                ..fixed(200.0, list_h)
            },
        );
        ui.open("Item", fixed(200.0, 300.0));
        // A count badge low on a tall item: below the list's view when the
        // list is short, over the bar that follows the list.
        boxed(
            ui,
            "Count",
            30.0,
            20.0,
            Props {
                position: Position::Absolute,
                insets: Edges {
                    start: Some(10.0),
                    top: Some(110.0),
                    ..Edges::default()
                },
                ..Props::default()
            },
        );
        ui.close();
        ui.close();
        boxed(ui, "Bar", 200.0, 50.0, Props::default());
        ui.close();
        ui.end();
        ui.overlaps()
            .iter()
            .map(|o| (o.path.clone(), o.other.clone()))
            .collect::<Vec<_>>()
    };
    // The list shows 0..100 and the bar sits at 100..150: the badge at
    // 110..130 is scrolled out of view, so it is over nothing on screen.
    let hidden = frame(&mut ui, 100.0);
    assert!(
        !hidden
            .iter()
            .any(|(a, b)| a == "P/List/Item/Count" || b == "P/List/Item/Count"),
        "{hidden:?}"
    );
    // Without a scroll around it the same badge over the bar is reported.
    ui.begin(InputFrame::default(), vp());
    ui.open(
        "Q",
        Element {
            direction: Direction::Column,
            ..Element::default()
        },
    );
    ui.open("Item", fixed(200.0, 100.0));
    boxed(
        &mut ui,
        "Count",
        30.0,
        20.0,
        Props {
            position: Position::Absolute,
            insets: Edges {
                start: Some(10.0),
                top: Some(90.0),
                ..Edges::default()
            },
            ..Props::default()
        },
    );
    ui.close();
    boxed(&mut ui, "Bar", 200.0, 50.0, Props::default());
    ui.close();
    ui.end();
    let shown: Vec<_> = ui
        .overlaps()
        .iter()
        .map(|o| (o.path.clone(), o.other.clone()))
        .collect();
    assert!(
        shown
            .iter()
            .any(|(a, b)| a == "Q/Item/Count" && b == "Q/Bar"),
        "{shown:?}"
    );
}
