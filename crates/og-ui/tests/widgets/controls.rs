use super::*;

#[derive(Default)]
struct Pick {
    chosen: usize,
    changes: u32,
}

fn select_screen(ui: &mut Ui, s: &mut Pick) {
    ui.label("Head", "Difficulty", LabelDesc::default());
    let r = ui.select(
        "Difficulty",
        &["Easy", "Normal", "Hard"],
        &mut s.chosen,
        &SelectDesc::default(),
    );
    if r.changed {
        s.changes += 1;
    }
    ui.button("Elsewhere", "Elsewhere", ButtonDesc::default());
}

#[test]
fn a_select_opens_chooses_and_closes_on_a_press_outside() {
    let mut rig = Rig::new(Pick::default());
    let b = &mut select_screen;
    rig.idle(1, b);
    rig.click("Difficulty", b);
    rig.idle(1, b);
    assert!(
        rig.ui.frame().element("Difficulty.menu/Option#2").is_some(),
        "the menu is open"
    );
    rig.click("Difficulty.menu/Option#2", b);
    rig.idle(1, b);
    assert_eq!((rig.state.chosen, rig.state.changes), (2, 1), "Hard chosen");
    assert!(
        rig.ui.frame().element("Difficulty.menu").is_none(),
        "choosing closes it"
    );
    assert_eq!(
        rig.ui
            .frame()
            .element("Difficulty")
            .map(|e| e.text.as_str()),
        Some("Hard")
    );
    rig.click("Difficulty", b);
    rig.idle(1, b);
    let out = rig.center("Head");
    rig.step(press(out), b);
    rig.step(release(out), b);
    rig.idle(1, b);
    assert!(
        rig.ui.frame().element("Difficulty.menu").is_none(),
        "a press outside closes it"
    );
    assert_eq!(rig.state.changes, 1);
    // From the keyboard: open, move down, choose.
    rig.key(UiKey::Tab, b);
    rig.key(UiKey::Enter, b);
    rig.idle(1, b);
    rig.key(UiKey::Up, b);
    rig.key(UiKey::Enter, b);
    rig.idle(1, b);
    assert_eq!(rig.state.chosen, 1, "Up from Hard is Normal");
}

#[derive(Default)]
struct Long {
    built: Vec<u32>,
    clicked: Option<u32>,
}

fn long_list(ui: &mut Ui, s: &mut Long) {
    s.built.clear();
    let r = ui.list(
        "Scores",
        ListDesc {
            count: 1000,
            row_height: 40.0,
            height: Some(Size::Fixed(400.0)),
            width: Size::Fixed(300.0),
            ..ListDesc::default()
        },
        s.clicked,
        |ui, i| {
            s.built.push(i);
            ui.label("Name", &format!("Player {i}"), LabelDesc::default());
        },
    );
    if r.is_some() {
        s.clicked = r;
    }
}

#[test]
fn a_long_list_builds_only_the_rows_in_view_and_scrolls_focus_into_view() {
    let mut rig = Rig::new(Long::default());
    let b = &mut long_list;
    rig.idle(2, b);
    assert!(
        rig.state.built.len() <= 12,
        "built {} of 1000 rows",
        rig.state.built.len()
    );
    assert_eq!(rig.state.built.first(), Some(&0));
    let list = rig.center("Scores");
    // Scroll far down with the wheel.
    rig.step(
        InputFrame {
            wheel: Vec2::new(0.0, 100_000.0),
            ..InputFrame::hover(list.x, list.y)
        },
        b,
    );
    rig.idle(1, b);
    assert_eq!(
        rig.state.built.last(),
        Some(&999),
        "the last row is reachable"
    );
    assert!(rig.state.built.len() <= 13);
    let last = rig
        .ui
        .frame()
        .element("Scores/Row#999")
        .expect("last row on screen")
        .clone();
    let view = rig.ui.frame().element("Scores").expect("list").clone();
    assert!(
        last.y + last.height <= view.y + view.height + 0.5,
        "the last row ends inside the list"
    );
    // Back to the top, then walk down past the view with the keyboard.
    rig.step(
        InputFrame {
            wheel: Vec2::new(0.0, -100_000.0),
            ..InputFrame::hover(list.x, list.y)
        },
        b,
    );
    rig.idle(1, b);
    rig.key(UiKey::Down, b);
    for _ in 0..14 {
        rig.key(UiKey::Down, b);
    }
    rig.idle(1, b);
    rig.key(UiKey::Enter, b);
    rig.idle(1, b);
    assert_eq!(
        rig.state.clicked,
        Some(14),
        "fifteen downs from row 0 land on row 14"
    );
    let row = rig
        .ui
        .frame()
        .element("Scores/Row#14")
        .expect("row 14 scrolled into view")
        .clone();
    let view = rig.ui.frame().element("Scores").expect("list").clone();
    assert!(row.y >= view.y - 0.5 && row.y + row.height <= view.y + view.height + 0.5);
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
        .frame()
        .elements
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
    let after: Vec<String> = rig
        .ui
        .frame()
        .elements
        .iter()
        .map(|e| e.id.clone())
        .collect();
    assert!(
        after.iter().all(|id| !id.starts_with("Toast")),
        "they left after 3.2 s: {after:?}"
    );
    rig.ui.restore_state(&saved).expect("restore");
    rig.idle(1, b);
    assert_eq!(
        rig.ui
            .frame()
            .elements
            .iter()
            .filter(|e| e.parent == "Toasts")
            .count(),
        3,
        "the restored state brings them back"
    );
}

#[derive(Default)]
struct Shown {
    show: bool,
    presses: u32,
    progress: Vec<f32>,
}

fn fading(ui: &mut Ui, s: &mut Shown) {
    let p = ui.transition(
        "Panel",
        s.show,
        TransitionDesc {
            kind: TransitionKind::SlideUp,
            ..TransitionDesc::default()
        },
        |ui| {
            if ui.button("Go", "Go", ButtonDesc::default()).clicked {
                s.presses += 1;
            }
        },
    );
    s.progress.push(p.progress);
}

/// A transition of `kind` over `ms` (0: the style's) around a button,
/// shown from the second tick; the frame's draw items `at` ticks in.
fn torn(kind: TransitionKind, ms: f32, reduced: bool, at: u32) -> Vec<DrawItem> {
    let mut rig = Rig::new(Shown::default());
    if reduced {
        let mut t = *rig.ui.theme();
        t.reduced_motion = true;
        rig.ui.set_theme(t);
    }
    let b = &mut |ui: &mut Ui, s: &mut Shown| {
        ui.transition(
            "Panel",
            s.show,
            TransitionDesc {
                kind,
                duration_ms: ms,
                ..TransitionDesc::default()
            },
            |ui| {
                ui.button("Go", "Go", ButtonDesc::default());
            },
        );
    };
    rig.idle(1, b);
    rig.state.show = true;
    rig.idle(at, b);
    rig.ui.frame().items.clone()
}

/// The strength of the glitch material a frame draws through, if any.
fn glitch_strength(items: &[DrawItem]) -> Option<f32> {
    items.iter().find_map(|i| match &i.cmd {
        DrawCmd::BeginLayer {
            material: og_render::UiMaterial::Builtin(og_render::UiBuiltin::Glitch),
            params,
            ..
        } => Some(params[0][0]),
        _ => None,
    })
}

/// A glitch is a frame like any other: two runs draw one frame (the
/// material's slices come from the element's id and the tick-counted UI
/// clock, never a wall clock or an unseeded rng); it draws its subtree
/// through og-render's glitch material, weaker as it settles; settled it
/// is exactly the fade's frame; under reduced motion it is that fade
/// throughout.
///
/// Ways it could fail: parameters from wall-clock time (two runs differ);
/// no glitch layer at all, or one that never weakens; the layer left
/// behind once settled; reduced motion still tearing.
#[test]
fn a_glitch_draws_the_same_every_run_settles_and_only_fades_under_reduced_motion() {
    let mid = 4;
    let a = torn(TransitionKind::Glitch, 0.0, false, mid);
    assert_eq!(
        a,
        torn(TransitionKind::Glitch, 0.0, false, mid),
        "two runs, one frame"
    );
    let now = glitch_strength(&a).expect("the subtree draws through the glitch material");
    let later = glitch_strength(&torn(TransitionKind::Glitch, 0.0, false, mid + 4))
        .expect("still glitching four ticks on");
    assert!(now > later && later > 0.0, "it settles: {now} then {later}");
    // The default base (240 ms) is held to 180 ms: settled by tick 11.
    assert_eq!(
        torn(TransitionKind::Glitch, 0.0, false, 12),
        torn(TransitionKind::Fade, 180.0, false, 12),
        "settled, nothing of the glitch is left"
    );
    for at in [2, mid, 8] {
        assert_eq!(
            torn(TransitionKind::Glitch, 180.0, true, at),
            torn(TransitionKind::Fade, 180.0, true, at),
            "reduced motion: a plain fade at tick {at}"
        );
    }
}

#[test]
fn a_transition_advances_by_tick_takes_no_input_while_moving_and_leaves() {
    let mut rig = Rig::new(Shown::default());
    let b = &mut fading;
    rig.idle(1, b);
    assert!(
        rig.ui.frame().element("Go").is_none(),
        "hidden: nothing built"
    );
    rig.state.show = true;
    rig.idle(2, b);
    let go = rig.center("Go");
    rig.step(press(go), b);
    rig.step(release(go), b);
    assert_eq!(rig.state.presses, 0, "inert while it enters");
    rig.idle(20, b);
    let n = rig.state.progress.len();
    // 240 ms at 60 ticks/s is 15 ticks: 1/15 per tick.
    assert!(
        (rig.state.progress[2] - 2.0 / 15.0).abs() < 1e-4,
        "{:?}",
        &rig.state.progress[..4]
    );
    assert_eq!(rig.state.progress[n - 1], 1.0);
    let go = rig.center("Go");
    rig.click("Go", b);
    assert_eq!(rig.state.presses, 1, "settled: it takes input");
    rig.state.show = false;
    rig.idle(1, b);
    assert!(
        rig.ui.frame().element("Go").is_some(),
        "still drawn while it leaves"
    );
    rig.idle(16, b);
    assert!(
        rig.ui.frame().element("Go").is_none(),
        "gone after its exit"
    );
    let _ = go;
}

#[test]
fn a_meter_trail_follows_a_drop_by_tick_and_replays_identically() {
    let run = || {
        let mut rig = Rig::new(100.0f32);
        let b = &mut |ui: &mut Ui, hp: &mut f32| {
            ui.meter("Hp", *hp, 100.0, &MeterDesc::default());
        };
        rig.idle(2, b);
        rig.state = 40.0;
        let mut frames = Vec::new();
        for _ in 0..40 {
            rig.idle(1, b);
            frames.push(rig.ui.frame().items.clone());
        }
        frames
    };
    let a = run();
    assert_eq!(a, run(), "the same ticks draw the same frames");
    let widths = |items: &Vec<DrawItem>| -> Vec<f32> {
        items
            .iter()
            .filter_map(|i| match &i.cmd {
                DrawCmd::Quad { rect, .. } => Some(rect.width),
                // The track and the fill are parts drawn through the UI
                // pass: their fills (not their shadows or strokes).
                DrawCmd::Shape { shape, .. } if shape.mode == og_render::UiMode::Fill => {
                    Some(shape.half.x * 2.0)
                }
                _ => None,
            })
            .collect()
    };
    // The track, the trail, the fill: the trail starts at 100 % and ends at 40 %.
    let first = widths(&a[0]);
    let last = widths(&a[39]);
    assert!(first.len() >= 3 && last.len() >= 2, "{first:?} {last:?}");
    assert!(
        first[1] > first[2] * 2.0,
        "right after the drop the trail still shows the old 100 %: {first:?}"
    );
    let mid = widths(&a[15]);
    assert!(
        mid[1] < first[1] && mid[1] > first[2],
        "half-way it is between: {mid:?}"
    );
    assert!(
        last.len() == 2 || (last[1] - last[2]).abs() < 0.6,
        "it caught up: {last:?}"
    );
}

#[test]
fn a_meter_label_takes_the_ink_that_reads_over_the_fill_and_over_the_track() {
    // The light theme: a deep green fill under the dark ink, a pale track.
    let mut scene = og_render::RenderScene::new();
    let mut ui = ui_with_font(add_test_font(&mut scene));
    ui.set_theme(Theme::light());
    let p = ui.theme().palette;
    for _ in 0..30 {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.meter(
            "Hp",
            55.0,
            100.0,
            &MeterDesc {
                label: "55 / 100".into(),
                width: Some(Size::Fixed(300.0)),
                ..MeterDesc::default()
            },
        );
        ui.end();
    }
    let bar = ui.frame().element("Hp").expect("the meter");
    let edge = bar.x + bar.width * 0.55;
    let texts: Vec<_> = ui
        .frame()
        .items
        .iter()
        .filter_map(|i| match &i.cmd {
            DrawCmd::Text {
                text, color, clip, ..
            } if text == "55 / 100" => Some((*color, *clip)),
            _ => None,
        })
        .collect();
    assert_eq!(texts.len(), 2, "one label per side: {texts:?}");
    let (over_fill, over_track) = (texts[0], texts[1]);
    assert!(
        (over_fill.1.x + over_fill.1.width - edge).abs() < 0.5
            && (over_track.1.x - edge).abs() < 0.5,
        "split at the fill's edge, {edge}: {texts:?}"
    );
    let ink = |c: Srgba| c.linear();
    assert_eq!(
        over_fill.0,
        ink(p.ink_inverse),
        "the inverse ink on the deep green"
    );
    assert_eq!(over_track.0, ink(p.ink), "the ink on the pale track");
    assert!(p.ink_inverse.contrast(p.health) > p.ink.contrast(p.health));
}

#[test]
fn the_dialogue_types_by_tick_a_press_finishes_and_the_next_press_advances() {
    let mut rig = Rig::new((DialogueResponse::default(), 0u32));
    let b = &mut |ui: &mut Ui, s: &mut (DialogueResponse, u32)| {
        s.0 = ui.dialogue(
            "Talk",
            &DialogueDesc {
                speaker: "Mira".into(),
                text: "The gate opens at dawn.".into(),
                chars_per_second: 30.0,
                ..DialogueDesc::default()
            },
        );
        if s.0.advanced {
            s.1 += 1;
        }
    };
    rig.idle(31, b);
    // 30 ticks at 30 chars/s and 60 ticks/s: 15 characters.
    assert_eq!(rig.state.0.shown, 15);
    assert_eq!(
        rig.ui
            .frame()
            .element("Talk/Body/Line")
            .map(|e| e.text.as_str()),
        Some("The gate opens ")
    );
    rig.click("Talk", b);
    assert!(
        rig.state.0.done && rig.state.1 == 0,
        "a press shows the whole line, nothing more"
    );
    rig.click("Talk", b);
    assert_eq!(rig.state.1, 1, "the next press advances");
}

#[derive(Default)]
struct Slots {
    drops: Vec<(u32, u32)>,
}

fn slots(ui: &mut Ui, s: &mut Slots) {
    let range = ui.open_grid(
        "Slots",
        GridDesc {
            count: 12,
            cell_width: 64.0,
            columns: 4,
            height: Some(Size::Fixed(300.0)),
            ..GridDesc::default()
        },
    );
    for i in range.first..range.last {
        if let Some(from) = ui.grid_item(i, false).dropped {
            s.drops.push((from, i));
        }
        ui.close();
    }
    ui.close_list();
}

#[test]
fn a_press_dragged_from_one_cell_to_another_drops_there_once() {
    let mut rig = Rig::new(Slots::default());
    let b = &mut slots;
    rig.idle(2, b);
    let (a, c) = (rig.center("Slots/Cell#0"), rig.center("Slots/Cell#6"));
    rig.step(press(a), b);
    let held = |p: Vec2| InputFrame {
        pointer: p,
        pointer_inside: true,
        down: true,
        ..InputFrame::default()
    };
    rig.step(held((a + c) / 2.0), b);
    rig.step(held(c), b);
    assert_eq!(rig.ui.dragging("Slots"), Some(0), "cell 0 is being dragged");
    rig.step(release(c), b);
    rig.idle(3, b);
    assert_eq!(
        rig.state.drops,
        vec![(0, 6)],
        "one drop, from cell 0 onto cell 6"
    );
    // A click on one cell is no drop.
    rig.click("Slots/Cell#3", b);
    assert_eq!(rig.state.drops.len(), 1);
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
                // As og.ui.dataGrid builds it: a frame of a fixed height, the
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

/// Small controls' reach (docs/ui.md §8.1): on a touch screen a control's
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
