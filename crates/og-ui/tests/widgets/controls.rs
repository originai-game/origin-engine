use super::*;

#[test]
fn keyed_rows_measure_anchor_navigate_and_restore() {
    let mut rig = Rig::new((0..100).map(|n| format!("item{n}")).collect::<Vec<_>>());
    let b = &mut |ui: &mut Ui, keys: &mut Vec<String>| {
        let range = ui.open_list_inner(
            "Rows".into(),
            ListDesc {
                row_height: 40.0,
                width: Size::Fixed(240.0),
                height: Some(Size::Fixed(150.0)),
                gap: 3.0,
                ..Default::default()
            },
            Some(og_ui::virtualize::Order::Untracked(keys)),
        );
        assert!(range.last - range.first < 15, "only a window is built");
        for i in range.first..range.last {
            ui.list_item(i, false);
            let n: usize = keys[i as usize]
                .trim_start_matches("item")
                .parse()
                .unwrap_or(0);
            ui.spacer("Content", Some(Size::Fixed(31.0 + (n % 3) as f32 * 23.0)));
            ui.close();
        }
        ui.close_list();
    };
    rig.idle(4, b);
    assert_ne!(
        rig.ui.element("Rows/Row#item0").unwrap().height,
        rig.ui.element("Rows/Row#item1").unwrap().height
    );
    rig.ui.navigate(Nav::Next);
    rig.idle(1, b);
    for i in 1..25 {
        let input = if i % 2 == 0 {
            InputFrame {
                keys: vec![UiKey::Down],
                ..Default::default()
            }
        } else {
            InputFrame {
                gamepad: GamepadFrame {
                    pressed: GamepadFrame::DOWN,
                    ..Default::default()
                },
                ..Default::default()
            }
        };
        rig.step(input, b);
        assert_eq!(rig.ui.focus_path(0), format!("Rows/Row#item{i}"));
    }
    rig.idle(3, b);
    let anchor = rig
        .ui
        .elements()
        .iter()
        .filter(|e| e.parent == "Rows" && e.id.starts_with("Row#") && e.y >= 0.0)
        .min_by(|a, b| a.y.total_cmp(&b.y))
        .unwrap()
        .clone();
    rig.state.insert(0, "inserted".into());
    rig.state.swap(1, 3);
    rig.state.remove(5);
    rig.idle(3, b);
    assert!(
        (rig.ui.element(&anchor.path).unwrap().y - anchor.y).abs() < 0.01,
        "edits above the viewport preserve the visible item's position"
    );
    assert_eq!(
        rig.ui.focus_path(0),
        "Rows/Row#item24",
        "focus belongs to the key"
    );
    let saved = rig.ui.save_state();
    rig.idle(1, b);
    let expected = rig.ui.frame().items.clone();
    let mut restored = Rig::new(rig.state.clone());
    restored.ui.restore_state(&saved).unwrap();
    restored.idle(1, b);
    assert_eq!(
        restored.ui.frame().items,
        expected,
        "fresh UI restores measured extents"
    );
}

#[test]
fn keyed_grid_uses_tallest_cells_and_reaches_unbuilt_end() {
    let mut rig = Rig::new((0..41).map(|n| n.to_string()).collect::<Vec<_>>());
    let b = &mut |ui: &mut Ui, keys: &mut Vec<String>| {
        let range = ui.open_grid_inner(
            "Cells".into(),
            GridDesc {
                columns: 2,
                cell_width: 100.0,
                cell_height: 60.0,
                width: Size::fit(),
                height: Some(Size::Fixed(180.0)),
                gap: 4.0,
                ..Default::default()
            },
            Some(og_ui::virtualize::Order::Untracked(keys)),
        );
        for i in range.first..range.last {
            ui.grid_item(i, false);
            ui.spacer(
                "Content",
                Some(Size::Fixed(if i % 2 == 0 { 35.0 } else { 81.0 })),
            );
            ui.close();
        }
        ui.close_list();
    };
    rig.idle(4, b);
    let short = rig.ui.element("Cells/Cell#0").unwrap();
    let tall = rig.ui.element("Cells/Cell#1").unwrap();
    let next = rig.ui.element("Cells/Cell#2").unwrap();
    assert_eq!(short.y, tall.y);
    assert!((next.y - (tall.y + tall.height + 4.0)).abs() < 0.01);
    assert!(rig.ui.element("Cells/Cell#40").is_none());
    rig.ui.navigate(Nav::Next);
    rig.idle(1, b);
    rig.step(
        InputFrame {
            keys: vec![UiKey::End],
            ..Default::default()
        },
        b,
    );
    rig.idle(3, b);
    assert_eq!(rig.ui.focus_path(0), "Cells/Cell#40");
    assert!(rig.ui.element("Cells/Cell#40").unwrap().y < 180.0);
    rig.step(
        InputFrame {
            keys: vec![UiKey::Home],
            ..Default::default()
        },
        b,
    );
    assert_eq!(rig.ui.focus_path(0), "Cells/Cell#0");
}

#[test]
fn toasts_stack_three_leave_on_time_and_replay_after_a_restore() {
    let mut rig = Rig::new(());
    let b = &mut |_: &mut Ui, _: &mut ()| {};
    rig.idle(1, b);
    for n in 0..5 {
        rig.ui.toast(&format!("Saved {n}"), ToastDesc::default());
    }
    rig.idle(30, b);
    let up: Vec<String> = rig
        .ui
        .elements()
        .iter()
        .filter(|e| e.parent == "Toasts")
        .map(|e| e.text.clone())
        .collect();
    assert_eq!(
        up,
        ["Saved 2", "Saved 3", "Saved 4"],
        "three at most; the oldest left"
    );
    let saved = rig.ui.save_state();
    rig.idle(200, b);
    let after: Vec<String> = rig.ui.elements().iter().map(|e| e.id.clone()).collect();
    assert!(
        after.iter().all(|id| !id.starts_with("Toast")),
        "they left after 3.2 s: {after:?}"
    );
    rig.ui.restore_state(&saved).expect("restore");
    rig.idle(1, b);
    assert_eq!(
        rig.ui
            .elements()
            .iter()
            .filter(|e| e.parent == "Toasts")
            .count(),
        3,
        "the restored state brings them back"
    );
}

/// A grid below a scroll view's fold builds the rows of its own box, not a
/// screenful: out of view it has no hit, so it reads last frame's layout for
/// its height and width (it guessed the viewport's: a dozen columns of rows,
/// laid out four across, three times its height).
#[test]
fn a_grid_out_of_view_builds_only_its_own_rows() {
    let mut rig = Rig::new(Vec::<(u32, u32)>::new());
    let b = &mut |ui: &mut Ui, built: &mut Vec<(u32, u32)>| {
        ui.scroll(
            "Page",
            ScrollDesc {
                width: Size::Fixed(400.0),
                height: Size::Fixed(300.0),
                ..ScrollDesc::default()
            },
            |ui| {
                ui.open(
                    "Spacer",
                    Element {
                        height: Size::Fixed(900.0),
                        ..Element::default()
                    },
                );
                ui.close();
                // As a data grid builds it: a frame of a fixed height, the
                // grid growing to fill it.
                ui.open(
                    "Frame",
                    Element {
                        direction: Direction::Column,
                        width: Size::Fixed(300.0),
                        height: Size::Fixed(150.0),
                        ..Element::default()
                    },
                );
                let range = ui.open_grid(
                    "Bag",
                    GridDesc {
                        count: 120,
                        cell_width: 64.0,
                        width: Size::grow(),
                        height: Some(Size::grow()),
                        ..GridDesc::default()
                    },
                );
                for i in range.first..range.last {
                    ui.grid_item(i, false);
                    ui.close();
                }
                ui.close_list();
                ui.close();
                built.push((range.first, range.last));
            },
        );
    };
    rig.idle(3, b);
    let (first, last) = *rig.state.last().expect("built");
    // 300 px holds four 64 px cells across; 150 px shows two or three rows,
    // plus one past each edge: at most five rows of four.
    assert!(last - first <= 20, "built cells {first}..{last}");
}

/// Small controls' reach (docs/ui.md §2.1): on a touch screen a control's
/// box grows to the 44 px touch target, and where two reaches overlap the
/// nearest box wins; a box around them (a panel behind) never takes a
/// press from a reach, a sibling's own box does; `hitSlop` reaches on any
/// pointer, the touch minimum only on a coarse one.
#[test]
fn a_small_controls_reach_grows_to_the_touch_target_and_the_nearest_box_wins() {
    let build = |ui: &mut Ui, hits: &mut Vec<&'static str>| {
        let place = |x: f32, y: f32, w: f32, h: f32, slop: f32| {
            (
                Element {
                    width: Size::Fixed(w),
                    height: Size::Fixed(h),
                    interactive: true,
                    ..Element::default()
                },
                Props {
                    position: Position::Absolute,
                    insets: Edges {
                        start: Some(x),
                        top: Some(y),
                        ..Edges::default()
                    },
                    hit_slop: slop,
                    ..Props::default()
                },
            )
        };
        for (id, (el, props)) in [
            ("Panel", place(40.0, 40.0, 400.0, 300.0, 0.0)),
            ("A", place(100.0, 100.0, 20.0, 20.0, 0.0)),
            ("B", place(126.0, 100.0, 20.0, 20.0, 0.0)),
            ("C", place(300.0, 100.0, 20.0, 20.0, 8.0)),
        ] {
            ui.open_with(id, el, props);
            if ui.close().clicked {
                hits.push(id);
            }
        }
    };
    let tap = |touch: bool, x: f32, y: f32| {
        let mut rig = Rig::new(Vec::new());
        rig.vp.coarse_pointer = touch;
        rig.idle(2, &mut |ui, s| build(ui, s));
        let p = Vec2::new(x, y);
        rig.step(press(p), &mut |ui, s| build(ui, s));
        rig.step(release(p), &mut |ui, s| build(ui, s));
        rig.state.first().copied().unwrap_or("")
    };
    // Between A (100–120) and B (126–146): the nearer box.
    assert_eq!(tap(true, 124.0, 110.0), "B");
    assert_eq!(tap(true, 121.0, 110.0), "A");
    // 10 px over A: in its 44 px reach on touch (the panel behind does not take it), the panel's with a mouse.
    assert_eq!(tap(true, 110.0, 90.0), "A");
    assert_eq!(tap(false, 110.0, 90.0), "Panel");
    // Past the 44 px reach: the panel.
    assert_eq!(tap(true, 110.0, 86.0), "Panel");
    // C's hitSlop reaches with a mouse too.
    assert_eq!(tap(false, 310.0, 94.0), "C");
}
