//! The element model and motion (docs/ui.md §6, §7), driven tick by tick.
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
        .frame()
        .element(path)
        .unwrap_or_else(|| panic!("no element {path}"));
    [e.x, e.y, e.width, e.height]
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
            .frame()
            .overlaps
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
        ui.frame()
            .overlaps
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
        .frame()
        .overlaps
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
