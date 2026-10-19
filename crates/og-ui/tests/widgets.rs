//! The game widget set and focus navigation, driven tick by tick through
//! `InputFrame`s (the probe-level paths are in `tests/probe.rs` and the
//! TypeScript demo's playtests).
//!
//! Ways these could fail, each covered below:
//! - arrows pick a diagonal over the aligned neighbour, or stop at an edge
//!   instead of wrapping; Tab skips or does not wrap; Shift+Tab goes forward;
//! - Enter/Space do not press the focused control, or press it while a text
//!   field is being edited (Space must type);
//! - a dialog lets focus or presses reach what is under it, ignores Back,
//!   closes while locked, or does not give focus back when it closes;
//! - a select does not open, choose, or close on a press outside;
//! - a virtualised list builds rows out of view, cannot reach its last row,
//!   or does not scroll a keyboard-focused row into view;
//! - toasts pile up past three, never leave, or differ after a restore;
//! - transitions advance by anything but ticks, take input while moving, or
//!   keep drawing once gone;
//! - a meter's trail jumps instead of following, or differs between runs;
//! - the dialogue types at the wrong rate, a press does not finish the line,
//!   or `advanced` fires before the line is shown;
//! - anchors ignore the safe area; touch controls come out under 44 px;
//! - a game accent leaves the token palette's derived tones stale;
//! - the gamepad stick repeats at the wrong ticks.

mod support;

use og_core::glam::Vec2;
use og_ui::*;
use support::*;

/// A Ui driven by a build closure.
struct Rig<S> {
    ui: Ui,
    state: S,
    vp: Viewport,
}

impl<S> Rig<S> {
    fn new(state: S) -> Rig<S> {
        Rig {
            ui: ui_with_font(test_font_metrics()),
            state,
            vp: Viewport::new(800.0, 600.0),
        }
    }

    fn step(&mut self, input: InputFrame, build: &mut dyn FnMut(&mut Ui, &mut S)) {
        self.ui.begin(input, self.vp);
        build(&mut self.ui, &mut self.state);
        self.ui.end();
    }

    fn idle(&mut self, n: u32, build: &mut dyn FnMut(&mut Ui, &mut S)) {
        for _ in 0..n {
            self.step(InputFrame::default(), build);
        }
    }

    fn center(&self, path: &str) -> Vec2 {
        let e = self
            .ui
            .frame()
            .element(path)
            .unwrap_or_else(|| panic!("no element {path}: {:?}", ids(&self.ui)));
        let (x, y) = e.center();
        Vec2::new(x, y)
    }

    fn click(&mut self, path: &str, build: &mut dyn FnMut(&mut Ui, &mut S)) {
        let p = self.center(path);
        self.step(press(p), build);
        self.step(release(p), build);
    }

    fn key(&mut self, k: UiKey, build: &mut dyn FnMut(&mut Ui, &mut S)) {
        self.step(keys(&[k]), build);
    }
}

fn ids(ui: &Ui) -> Vec<String> {
    ui.frame().elements.iter().map(|e| e.id.clone()).collect()
}

fn press(p: Vec2) -> InputFrame {
    InputFrame {
        pointer: p,
        pointer_inside: true,
        down: true,
        pressed: true,
        ..InputFrame::default()
    }
}

fn release(p: Vec2) -> InputFrame {
    InputFrame {
        pointer: p,
        pointer_inside: true,
        released: true,
        ..InputFrame::default()
    }
}

fn keys(k: &[UiKey]) -> InputFrame {
    InputFrame {
        keys: k.to_vec(),
        ..InputFrame::default()
    }
}

/// A 3×3 grid of buttons; records which one reports focus and which clicked.
#[derive(Default)]
struct Grid {
    focus: Option<String>,
    clicks: Vec<String>,
}

fn grid(ui: &mut Ui, s: &mut Grid) {
    s.focus = None;
    ui.column(
        "Grid",
        Element {
            gap: 8.0,
            ..Element::default()
        },
        |ui| {
            for r in 0..3u32 {
                ui.row(
                    ui_idi("R", r),
                    Element {
                        gap: 8.0,
                        ..Element::default()
                    },
                    |ui| {
                        for c in 0..3u32 {
                            let name = format!("B{r}{c}");
                            let resp = ui.button(name.as_str(), &name, ButtonDesc::default());
                            if resp.focused {
                                s.focus = Some(name.clone());
                            }
                            if resp.clicked {
                                s.clicks.push(name);
                            }
                        }
                    },
                );
            }
        },
    );
}

#[test]
fn arrows_move_to_the_aligned_neighbour_and_wrap_at_the_edges() {
    let mut rig = Rig::new(Grid::default());
    let b = &mut grid;
    rig.idle(1, b);
    rig.key(UiKey::Down, b);
    rig.idle(1, b);
    assert_eq!(
        rig.state.focus.as_deref(),
        Some("B00"),
        "the first arrow focuses the first control"
    );
    rig.key(UiKey::Right, b);
    rig.idle(1, b);
    assert_eq!(rig.state.focus.as_deref(), Some("B01"));
    rig.key(UiKey::Down, b);
    rig.idle(1, b);
    assert_eq!(
        rig.state.focus.as_deref(),
        Some("B11"),
        "down stays in the column"
    );
    rig.key(UiKey::Down, b);
    rig.key(UiKey::Down, b);
    rig.idle(1, b);
    assert_eq!(
        rig.state.focus.as_deref(),
        Some("B01"),
        "down from the last row wraps to the top of the column"
    );
    rig.key(UiKey::Left, b);
    rig.key(UiKey::Left, b);
    rig.idle(1, b);
    assert_eq!(
        rig.state.focus.as_deref(),
        Some("B02"),
        "left from the first column wraps to the end of the row"
    );
    rig.key(UiKey::Enter, b);
    assert_eq!(
        rig.state.clicks,
        vec!["B02".to_string()],
        "Enter presses the focused control"
    );
    rig.key(UiKey::Space, b);
    assert_eq!(rig.state.clicks.len(), 2, "Space presses it too");
}

#[test]
fn tab_walks_reading_order_and_shift_tab_walks_back() {
    let mut rig = Rig::new(Grid::default());
    let b = &mut grid;
    rig.idle(1, b);
    let mut seen = Vec::new();
    for _ in 0..10 {
        rig.key(UiKey::Tab, b);
        rig.idle(1, b);
        seen.push(rig.state.focus.clone().unwrap_or_default());
    }
    assert_eq!(
        seen,
        [
            "B00", "B01", "B02", "B10", "B11", "B12", "B20", "B21", "B22", "B00"
        ],
        "Tab in reading order, wrapping"
    );
    rig.step(
        InputFrame {
            keys: vec![UiKey::Tab],
            shift: true,
            ..InputFrame::default()
        },
        b,
    );
    rig.idle(1, b);
    assert_eq!(
        rig.state.focus.as_deref(),
        Some("B22"),
        "Shift+Tab wraps backwards"
    );
}

#[test]
fn the_gamepad_dpad_and_stick_navigate_with_repeat_by_tick() {
    let mut rig = Rig::new(Grid::default());
    let b = &mut grid;
    rig.idle(1, b);
    let pad = |pressed: u32, stick: Vec2| InputFrame {
        gamepad: GamepadFrame { pressed, stick },
        ..InputFrame::default()
    };
    rig.step(pad(GamepadFrame::RIGHT, Vec2::ZERO), b);
    rig.idle(1, b);
    assert_eq!(rig.state.focus.as_deref(), Some("B00"));
    rig.step(pad(GamepadFrame::RIGHT, Vec2::ZERO), b);
    rig.idle(1, b);
    assert_eq!(rig.state.focus.as_deref(), Some("B01"), "d-pad right");
    // Hold the stick right: one move now, then after 400 ms (24 ticks) one
    // every 180 ms (11 ticks).
    let mut moves = Vec::new();
    let mut last = rig.state.focus.clone();
    for tick in 0..48 {
        rig.step(pad(0, Vec2::new(0.9, 0.1)), b);
        if rig.state.focus != last {
            moves.push(tick);
            last = rig.state.focus.clone();
        }
    }
    assert_eq!(
        moves,
        vec![0, 24, 35, 46],
        "stick: at once, after the slow delay, then every fast step"
    );
    rig.step(pad(GamepadFrame::A, Vec2::ZERO), b);
    assert_eq!(rig.state.clicks.len(), 1, "A presses the focused control");
}

#[derive(Default)]
struct Dialog {
    open: bool,
    under: u32,
    ok: u32,
    dismissed: u32,
    locked: bool,
    focus: Vec<String>,
}

fn dialog_screen(ui: &mut Ui, s: &mut Dialog) {
    s.focus.clear();
    if ui.button("Open", "Open", ButtonDesc::default()).clicked {
        s.open = true;
    }
    let r = ui.button("Under", "Under", ButtonDesc::default());
    if r.clicked {
        s.under += 1;
    }
    let desc = ModalDesc {
        title: "Quit?".into(),
        locked: s.locked,
        ..ModalDesc::default()
    };
    let m = ui.modal("Confirm", s.open, &desc, |ui| {
        for name in ["Cancel", "Ok"] {
            let r = ui.button(name, name, ButtonDesc::default());
            if r.focused {
                s.focus.push(name.into());
            }
            if r.clicked {
                if name == "Ok" {
                    s.ok += 1;
                }
                s.open = false;
            }
        }
    });
    if m.dismissed {
        s.dismissed += 1;
        s.open = false;
    }
}

#[test]
fn a_dialog_traps_focus_blocks_what_is_under_it_and_closes_on_back() {
    let mut rig = Rig::new(Dialog::default());
    let b = &mut dialog_screen;
    rig.idle(1, b);
    rig.key(UiKey::Tab, b);
    rig.key(UiKey::Enter, b);
    assert!(rig.state.open, "Enter on Open opens the dialog");
    rig.idle(30, b);
    assert_eq!(
        rig.state.focus,
        vec!["Cancel".to_string()],
        "focus moved into the dialog"
    );
    for _ in 0..5 {
        rig.key(UiKey::Tab, b);
    }
    rig.idle(1, b);
    assert!(
        rig.state.focus == vec!["Ok".to_string()] || rig.state.focus == vec!["Cancel".to_string()],
        "Tab stays inside: {:?}",
        rig.state.focus
    );
    // The Under button sits under the backdrop: a press there hits the
    // backdrop, which dismisses.
    let under = rig.center("Under");
    rig.step(press(under), b);
    rig.step(release(under), b);
    assert_eq!(
        rig.state.under, 0,
        "the press did not reach the control under the dialog"
    );
    assert_eq!(rig.state.dismissed, 1, "a press on the backdrop dismisses");
    rig.idle(30, b);
    assert!(
        rig.ui.frame().element("Confirm").is_none(),
        "gone after its exit"
    );

    // Locked: neither Back nor the backdrop closes it.
    rig.state.locked = true;
    rig.state.open = true;
    rig.idle(30, b);
    rig.key(UiKey::Escape, b);
    rig.step(press(under), b);
    rig.step(release(under), b);
    assert_eq!(rig.state.dismissed, 1, "locked");
    rig.state.locked = false;
    rig.idle(1, b);
    rig.key(UiKey::Escape, b);
    assert_eq!(rig.state.dismissed, 2, "Back dismisses an unlocked dialog");
    rig.idle(30, b);
    rig.key(UiKey::Enter, b);
    assert!(
        rig.state.open,
        "focus came back to Open, so Enter opens it again"
    );
}

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

#[test]
fn anchors_sit_in_the_safe_area_and_touch_controls_keep_44px() {
    let mut rig = Rig::new(JoystickResponse::default());
    rig.vp = Viewport {
        width: 390.0,
        height: 844.0,
        dpr: 3.0,
        safe_area: Insets {
            left: 0.0,
            top: 47.0,
            right: 0.0,
            bottom: 34.0,
        },
        coarse_pointer: true,
    };
    let b = &mut |ui: &mut Ui, s: &mut JoystickResponse| {
        ui.anchor(
            "Score",
            AnchorDesc {
                anchor: Anchor::TopRight,
                ..AnchorDesc::default()
            },
            |ui| {
                ui.label("Points", "1200", LabelDesc::default());
            },
        );
        ui.anchor(
            "Pad",
            AnchorDesc {
                anchor: Anchor::BottomLeft,
                ..AnchorDesc::default()
            },
            |ui| {
                *s = ui.joystick(
                    "Stick",
                    JoystickDesc {
                        size: 60.0,
                        ..JoystickDesc::default()
                    },
                );
            },
        );
        ui.anchor(
            "Buttons",
            AnchorDesc {
                anchor: Anchor::BottomRight,
                ..AnchorDesc::default()
            },
            |ui| {
                ui.action_button(
                    "Jump",
                    "Jump",
                    &ActionButtonDesc {
                        size: 20.0,
                        ..ActionButtonDesc::default()
                    },
                );
                ui.dpad("Dpad", DpadDesc { size: 60.0 });
            },
        );
    };
    rig.idle(1, b);
    assert_eq!(rig.ui.breakpoint(), Breakpoint::PhonePortrait);
    let gutter = rig.ui.sizes().gutter;
    let score = rig.ui.frame().element("Score").expect("score").clone();
    assert!(
        (score.y - (47.0 + gutter)).abs() < 0.5,
        "under the notch: {score:?}"
    );
    assert!((score.x + score.width - (390.0 - gutter)).abs() < 0.5);
    let pad = rig.ui.frame().element("Pad").expect("pad").clone();
    assert!(
        (pad.y + pad.height - (844.0 - 34.0 - gutter)).abs() < 0.5,
        "above the home indicator: {pad:?}"
    );
    for id in ["Stick", "Jump", "Dpad/Up", "Dpad/Left"] {
        let e = rig.ui.frame().element(id).expect(id);
        assert!(
            e.width >= 44.0 && e.height >= 44.0,
            "{id} is {}×{}",
            e.width,
            e.height
        );
    }
    // Drag the stick to the right edge.
    let c = rig.center("Stick");
    let r = rig.ui.frame().element("Stick").unwrap().width / 2.0;
    rig.step(press(c), b);
    rig.step(
        InputFrame {
            pointer: c + Vec2::new(r * 2.0, 0.0),
            pointer_inside: true,
            down: true,
            ..InputFrame::default()
        },
        b,
    );
    assert!(
        rig.state.active && (rig.state.x - 1.0).abs() < 1e-4 && rig.state.y.abs() < 1e-4,
        "{:?}",
        rig.state
    );
    rig.step(release(c), b);
    assert!(!rig.state.active && rig.state.x == 0.0);
}

#[test]
fn a_game_accent_derives_its_tones_the_way_the_tokens_derive_amber() {
    let dark = Theme::dark();
    let same = dark.with_accent(dark.palette.accent);
    assert_eq!(
        same.palette, dark.palette,
        "amber in, the token palette out"
    );
    let light = Theme::light();
    assert_eq!(
        light.with_accent(light.palette.accent).palette,
        light.palette
    );
    let blue = dark.with_hue(HueName::Blue);
    assert_eq!(blue.palette.accent, dark.hues.blue);
    assert!(
        blue.palette.accent_hover.luminance() > blue.palette.accent.luminance(),
        "dark hover is lighter, as amber's"
    );
    let hc = Theme::high_contrast(Scheme::Dark);
    assert_eq!(hc.palette.fg_muted, dark.palette.fg, "muted ink takes fg");
    assert!(hc.outline);
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

/// A type step's tracking spaces Latin glyphs (each advance plus tracking
/// em: the label is wider by that per character and its draw command
/// carries it to og-render), and leaves CJK text as the font sets it.
/// Ways it could fail: the ramp's tracking is ignored (equal widths),
/// measured but not drawn (the text overruns its box), or applied to CJK.
#[test]
fn a_steps_tracking_spaces_latin_glyphs_and_not_cjk() {
    let width = |tracking: f32, text: &str| {
        let mut scene = og_render::RenderScene::new();
        let mut ui = ui_with_font(add_test_font(&mut scene));
        let mut t = *ui.theme();
        t.type_ramp.xl.tracking_em = tracking;
        ui.set_theme(t);
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.label(
            "L",
            text,
            LabelDesc {
                step: Some(t.type_ramp.xl),
                nowrap: true,
                ..LabelDesc::default()
            },
        );
        ui.end();
        let drawn = ui
            .frame()
            .items
            .iter()
            .find_map(|i| match &i.cmd {
                DrawCmd::Text { tracking, .. } => Some(*tracking),
                _ => None,
            })
            .unwrap_or(f32::NAN);
        (ui.frame().element("L").expect("label").width, drawn)
    };
    let size = Theme::default().type_ramp.xl.size;
    let (plain, _) = width(0.0, "Origin");
    let (wide, drawn) = width(0.1, "Origin");
    assert!(
        (wide - plain - 6.0 * 0.1 * size).abs() < 0.75,
        "six glyphs, each 0.1 em further: {plain} then {wide}"
    );
    assert_eq!(drawn, 0.1, "the draw command carries the tracking");
    let (cjk_plain, _) = width(0.0, "原点引擎");
    let (cjk_wide, cjk_drawn) = width(0.1, "原点引擎");
    assert_eq!(cjk_plain, cjk_wide, "CJK keeps the font's spacing");
    assert_eq!(cjk_drawn, 0.0);
}

/// A 3×3 grid of buttons for local multiplayer: who pressed what.
#[derive(Default)]
struct Lobby {
    players: u32,
    presses: Vec<(String, Option<u32>)>,
    rings: Vec<(String, u32)>,
}

fn lobby(ui: &mut Ui, s: &mut Lobby) {
    ui.set_players(s.players);
    s.rings.clear();
    ui.column(
        "Lobby",
        Element {
            gap: 8.0,
            ..Element::column()
        },
        |ui| {
            for r in 0..3u32 {
                ui.row(
                    ui_idi("R", r),
                    Element {
                        gap: 8.0,
                        ..Element::default()
                    },
                    |ui| {
                        for c in 0..3u32 {
                            let name = format!("B{r}{c}");
                            let resp = ui.button(name.as_str(), &name, ButtonDesc::default());
                            if resp.clicked {
                                s.presses.push((name.clone(), resp.pad));
                            }
                            if resp.focused_by != 0 {
                                s.rings.push((name, resp.focused_by));
                            }
                        }
                    },
                );
            }
        },
    );
}

fn pad(n: usize, bit: u32) -> InputFrame {
    let mut f = InputFrame::default();
    if n == 0 {
        f.gamepad.pressed = bit;
    } else {
        f.extra_pads[n - 1].pressed = bit;
    }
    f
}

/// Pads beyond the first: with `set_players(3)`, pads 1 and 2 each move a
/// focus of their own and press with A (the response names the pad); pad 0
/// keeps the shared focus; `focus_path` answers each; their rings are
/// drawn; the state replays after a restore. Ways it could fail: an extra
/// pad moves the shared focus, presses the shared focus's control, is heard
/// while the screen takes one player, or loses its place after a restore.
#[test]
fn pads_beyond_the_first_move_their_own_focus_and_press() {
    let mut rig = Rig::new(Lobby {
        players: 1,
        ..Lobby::default()
    });
    let b = &mut lobby;
    rig.idle(2, b);
    // One player: pad 1 does nothing.
    rig.step(pad(1, GamepadFrame::RIGHT), b);
    rig.step(pad(1, GamepadFrame::A), b);
    rig.idle(1, b);
    assert!(rig.state.presses.is_empty(), "pad 1 is not a player yet");
    assert_eq!(rig.ui.focus_path(1), "");
    rig.state.players = 3;
    rig.idle(2, b);
    // Pad 1: its first move lands on the control nearest the surface's
    // centre (B22, the grid sits top left), then left to B21.
    rig.step(pad(1, GamepadFrame::RIGHT), b);
    rig.step(pad(1, GamepadFrame::LEFT), b);
    // Pad 2: onto B22 the same way, then up to B12.
    rig.step(pad(2, GamepadFrame::DOWN), b);
    rig.step(pad(2, GamepadFrame::UP), b);
    // Pad 0: the shared focus, its own place.
    rig.step(pad(0, GamepadFrame::DOWN), b);
    rig.idle(1, b);
    assert_eq!(rig.ui.focus_path(1), "Lobby/R#2/B21");
    assert_eq!(rig.ui.focus_path(2), "Lobby/R#1/B12");
    assert_eq!(rig.ui.focus_path(0), "Lobby/R#0/B00");
    assert_eq!(rig.ui.focus_path(3), "", "pad 3 has not moved");
    let saved = rig.ui.save_state();
    rig.step(pad(1, GamepadFrame::A), b);
    rig.step(pad(2, GamepadFrame::A), b);
    rig.step(pad(0, GamepadFrame::A), b);
    rig.idle(1, b);
    assert_eq!(
        rig.state.presses,
        vec![
            ("B21".to_string(), Some(1)),
            ("B12".to_string(), Some(2)),
            ("B00".to_string(), Some(0)),
        ],
        "each pad pressed the control under its own focus"
    );
    let ring = |name: &str| rig.state.rings.iter().find(|r| r.0 == name).map(|r| r.1);
    assert_eq!(ring("B21"), Some(2), "B21 shows pad 1's ring");
    assert_eq!(ring("B12"), Some(4), "B12 shows pad 2's ring");
    let info = rig.ui.theme().palette.info.linear();
    assert!(
        rig.ui
            .frame()
            .items
            .iter()
            .any(|i| matches!(&i.cmd, DrawCmd::Quad { color, .. } if *color == info)),
        "pad 1's ring is drawn in its player colour"
    );
    // A restore before the presses replays them identically.
    let before = rig.state.presses.clone();
    rig.state.presses.clear();
    rig.ui.restore_state(&saved).expect("restore");
    rig.step(pad(1, GamepadFrame::A), b);
    rig.step(pad(2, GamepadFrame::A), b);
    rig.step(pad(0, GamepadFrame::A), b);
    rig.idle(1, b);
    assert_eq!(rig.state.presses, before, "the pads' focus is UI state");
    // Back to one player: the extra focus is dropped.
    rig.state.players = 1;
    rig.idle(2, b);
    assert_eq!(rig.ui.focus_path(1), "");
}

/// Two scrollers: a page (vertical) holding a horizontal strip, and a
/// board scrolling both ways. Tiles are plain (non-interactive) so a press
/// on them holds the scroller.
fn scrollers(ui: &mut Ui, _: &mut ()) {
    let tile = |ui: &mut Ui, id: Id, w: f32, h: f32| {
        ui.container(
            id,
            Element {
                width: Size::Fixed(w),
                height: Size::Fixed(h),
                background: Some(Srgba::rgb(80, 80, 90)),
                ..Element::default()
            },
            |_| {},
        );
    };
    ui.row(
        "Root",
        Element {
            gap: 20.0,
            ..Element::default()
        },
        |ui| {
            ui.scroll(
                "Page",
                ScrollDesc {
                    width: Size::Fixed(220.0),
                    height: Size::Fixed(200.0),
                    gap: 10.0,
                    ..ScrollDesc::default()
                },
                |ui| {
                    ui.scroll(
                        "Strip",
                        ScrollDesc {
                            width: Size::grow(),
                            height: Size::Fixed(60.0),
                            axes: ScrollAxes::Horizontal,
                            ..ScrollDesc::default()
                        },
                        |ui| {
                            ui.row(
                                "Cards",
                                Element {
                                    gap: 10.0,
                                    ..Element::default()
                                },
                                |ui| {
                                    for k in 0..10 {
                                        tile(ui, ui_idi("Card", k), 80.0, 50.0);
                                    }
                                },
                            );
                        },
                    );
                    for k in 0..10 {
                        tile(ui, ui_idi("Line", k), 200.0, 40.0);
                    }
                },
            );
            ui.scroll(
                "Board",
                ScrollDesc {
                    width: Size::Fixed(200.0),
                    height: Size::Fixed(200.0),
                    axes: ScrollAxes::Both,
                    ..ScrollDesc::default()
                },
                |ui| {
                    for r in 0..10 {
                        ui.row(ui_idi("Row", r), Element::default(), |ui| {
                            for c in 0..10 {
                                tile(ui, ui_idi("Tile", r * 10 + c), 50.0, 50.0);
                            }
                        });
                    }
                },
            );
        },
    );
}

fn wheel(at: Vec2, d: Vec2) -> InputFrame {
    InputFrame {
        wheel: d,
        ..InputFrame::hover(at.x, at.y)
    }
}

/// Where an element is now, or None when it is scrolled out of view.
fn pos(ui: &Ui, path: &str) -> Option<Vec2> {
    ui.frame().element(path).map(|e| Vec2::new(e.x, e.y))
}

/// Scrolling both ways: the wheel moves a board on both axes and stops at
/// its content's ends; a wheel the inner strip cannot take (it scrolls
/// sideways only) scrolls the page around it; a drag moves both axes and,
/// released while moving, keeps going and slows to rest (a press stops it);
/// a press on a scrollbar grabs it; the offsets replay after a restore.
/// Ways it could fail: one axis only, no clamp at the end, a nested
/// container swallowing what it cannot use, a fling that never stops or
/// does not decay, a scrollbar that cannot be grabbed, or offsets outside
/// the UI state.
#[test]
fn scroll_views_scroll_both_ways_hand_off_fling_and_grab() {
    let mut rig = Rig::new(());
    let b = &mut scrollers;
    rig.idle(2, b);
    let board = rig.center("Board");
    let t0 = pos(&rig.ui, "Board/Row#0/Tile#0").expect("the first tile shows");
    // Both axes by the wheel.
    // (The probe reports what shows: a tile well inside the view is whole.)
    let t22 = pos(&rig.ui, "Board/Row#2/Tile#22").expect("a middle tile");
    rig.step(wheel(board, Vec2::new(30.0, 40.0)), b);
    rig.idle(1, b);
    let t = pos(&rig.ui, "Board/Row#2/Tile#22").expect("still in view");
    assert_eq!(
        t,
        t22 - Vec2::new(30.0, 40.0),
        "the wheel moved the board on both axes"
    );
    // Past the end: the content is 500×500 in a 200×200 view, so 300 each way.
    rig.step(wheel(board, Vec2::new(5000.0, 5000.0)), b);
    rig.idle(1, b);
    let last = pos(&rig.ui, "Board/Row#9/Tile#99").expect("the last tile shows");
    let view = rig.ui.frame().element("Board").expect("board").clone();
    assert!(
        (last.x + 50.0 - (view.x + view.width)).abs() < 0.5
            && (last.y + 50.0 - (view.y + view.height)).abs() < 0.5,
        "it stops with the last tile at the view's corner: {last:?} in {view:?}"
    );
    // Handoff: a vertical wheel over the strip (horizontal only) scrolls the page.
    let strip = rig.center("Strip");
    let line0 = pos(&rig.ui, "Page/Line#0").expect("a line");
    rig.step(wheel(strip, Vec2::new(0.0, 25.0)), b);
    rig.idle(1, b);
    assert_eq!(
        pos(&rig.ui, "Page/Line#0").expect("a line").y,
        line0.y - 25.0,
        "what the strip cannot take scrolls the page"
    );
    let card0 = pos(&rig.ui, "Strip/Cards/Card#1").expect("a card");
    let strip_now = rig.center("Strip");
    rig.step(wheel(strip_now, Vec2::new(45.0, 0.0)), b);
    rig.idle(1, b);
    assert_eq!(
        pos(&rig.ui, "Strip/Cards/Card#1").expect("a card").x,
        card0.x - 45.0,
        "a sideways wheel scrolls the strip"
    );
    // A drag on the board back towards the start, released while moving.
    rig.step(wheel(board, Vec2::new(-5000.0, -5000.0)), b);
    rig.idle(1, b);
    let start = rig.center("Board");
    let mut p = start;
    rig.step(
        InputFrame {
            pointer: p,
            pointer_inside: true,
            down: true,
            pressed: true,
            ..InputFrame::default()
        },
        b,
    );
    for _ in 0..4 {
        p -= Vec2::new(12.0, 9.0);
        rig.step(
            InputFrame {
                pointer: p,
                pointer_inside: true,
                down: true,
                ..InputFrame::default()
            },
            b,
        );
    }
    let dragged = pos(&rig.ui, "Board/Row#0/Tile#0");
    rig.step(
        InputFrame {
            pointer: p,
            pointer_inside: true,
            released: true,
            ..InputFrame::default()
        },
        b,
    );
    // The horizontal scrollbar's position is the board's horizontal offset.
    let bar_x = |ui: &Ui| -> f32 {
        let r = ui.frame().element("Board").expect("board").clone();
        ui.frame()
            .items
            .iter()
            .find_map(|i| match &i.cmd {
                DrawCmd::Quad { rect, .. }
                    if rect.y > r.y + r.height - 12.0
                        && rect.width > 20.0
                        && rect.height < 8.0
                        && rect.x >= r.x - 0.5 =>
                {
                    Some(rect.x)
                }
                _ => None,
            })
            .unwrap_or(f32::NAN)
    };
    let mut xs = Vec::new();
    for _ in 0..120 {
        rig.step(InputFrame::hover(p.x, p.y), b);
        xs.push(bar_x(&rig.ui));
    }
    assert!(dragged.is_some(), "the drag scrolled the board");
    let steps: Vec<f32> = xs.windows(2).map(|w| w[0] - w[1]).collect();
    let steps: Vec<f32> = steps.iter().map(|d| -d).collect();
    assert!(
        steps[0] > 0.3,
        "released while moving, it keeps going: {xs:?}"
    );
    assert!(
        steps[0] > steps[20] && steps[20] >= 0.0,
        "and slows: {steps:?}"
    );
    assert_eq!(xs[118], xs[119], "and comes to rest");
    // Back to the start.
    rig.step(wheel(board, Vec2::new(-5000.0, -5000.0)), b);
    let saved = rig.ui.save_state();
    // The vertical scrollbar: a press near the bottom of the board's right edge grabs it.
    let r = rig.ui.frame().element("Board").expect("board").clone();
    let grab = Vec2::new(r.x + r.width - 3.0, r.y + r.height - 6.0);
    rig.step(
        InputFrame {
            pointer: grab,
            pointer_inside: true,
            down: true,
            pressed: true,
            ..InputFrame::default()
        },
        b,
    );
    rig.idle(1, b);
    let y = pos(&rig.ui, "Board/Row#0/Tile#0").map_or(f32::NEG_INFINITY, |p| p.y);
    assert!(
        rig.ui.frame().element("Board/Row#0/Tile#0").is_none() || y < r.y - 100.0,
        "pressing the track near its end moved the board far down ({y})"
    );
    // A restore replays the offsets.
    rig.ui.restore_state(&saved).expect("restore");
    rig.idle(1, b);
    assert_eq!(
        pos(&rig.ui, "Board/Row#0/Tile#0"),
        Some(t0),
        "the offsets are UI state"
    );
}
