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

use og_core::glam::Vec2;
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

/// Builds `f` 100 px in from the screen's corner (`P/...`): the probe
/// clips bounds to the screen, so what moves stays in view.
fn padded(ui: &mut Ui, f: impl FnOnce(&mut Ui)) {
    ui.open(
        "P",
        Element {
            padding: Insets::all(100.0),
            ..Element::default()
        },
    );
    f(ui);
    ui.close();
}

fn rect(ui: &Ui, path: &str) -> [f32; 4] {
    let e = ui
        .frame()
        .element(path)
        .unwrap_or_else(|| panic!("no element {path}"));
    [e.x, e.y, e.width, e.height]
}

fn at(p: Vec2) -> InputFrame {
    InputFrame {
        pointer: p,
        pointer_inside: true,
        ..InputFrame::default()
    }
}

fn press(p: Vec2) -> InputFrame {
    InputFrame {
        down: true,
        pressed: true,
        ..at(p)
    }
}

fn hold(p: Vec2) -> InputFrame {
    InputFrame {
        down: true,
        ..at(p)
    }
}

fn release(p: Vec2) -> InputFrame {
    InputFrame {
        released: true,
        ..at(p)
    }
}

#[test]
fn a_transform_moves_the_hit_test_the_probe_rect_and_the_a11y_bounds() {
    let mut ui = Ui::new(UiDesc::default());
    // 200 px right, 100 down and a quarter turn about its centre: a wide
    // button stands up.
    let turned = Props {
        transform: Transform {
            x: 200.0,
            y: 100.0,
            rotate: std::f32::consts::FRAC_PI_2,
            ..Transform::IDENTITY
        },
        ..Props::default()
    };
    let step = |ui: &mut Ui, input: InputFrame, props: &Props| {
        ui.begin(input, vp());
        let r = ui.button(
            "Go",
            "Go",
            ButtonDesc {
                width: Some(Size::Fixed(120.0)),
                ..ButtonDesc::default()
            },
        );
        ui.set_props(props.clone());
        ui.end();
        r.clicked
    };
    step(&mut ui, InputFrame::default(), &Props::default());
    let [x, y, w, h] = rect(&ui, "Go");
    assert!(w == 120.0 && h < 100.0, "{w}×{h}");
    let (cx, cy) = (x + w / 2.0, y + h / 2.0);
    step(&mut ui, InputFrame::default(), &turned);
    let want = [cx + 200.0 - h / 2.0, cy + 100.0 - w / 2.0, h, w];
    let r = rect(&ui, "Go");
    for (got, want) in r.iter().zip(want) {
        assert!((got - want).abs() < 0.01, "probe rect follows it: {r:?}");
    }
    let node = ui
        .frame()
        .a11y
        .nodes
        .iter()
        .find(|n| n.path == "Go")
        .expect("in the a11y tree");
    assert!(
        (node.x - want[0]).abs() < 0.01 && (node.height - w).abs() < 0.01,
        "{node:?}"
    );
    // Where it was laid out is empty now; a point inside the turned button,
    // outside the unturned box moved by the translation, is on it.
    let mut clicked = Vec::new();
    for p in [
        Vec2::new(cx, cy),
        Vec2::new(cx + 200.0, cy + 100.0 + w / 2.0 - 5.0),
    ] {
        step(&mut ui, press(p), &turned);
        clicked.push(step(&mut ui, release(p), &turned));
    }
    assert_eq!(clicked, [false, true]);
}

#[test]
fn a_retargeted_spring_keeps_its_value_and_velocity() {
    let mut ui = Ui::new(UiDesc::default());
    let spring = Transition::Spring(SpringSpec::new(400.0, 0.0));
    let tick = |ui: &mut Ui, target: f32| {
        ui.begin(InputFrame::default(), vp());
        let v = ui.animate("v", target, spring);
        ui.end();
        v
    };
    tick(&mut ui, 0.0);
    let path: Vec<f32> = (0..6).map(|_| tick(&mut ui, 100.0)).collect();
    let (v, last_step) = (path[5], path[5] - path[4]);
    assert!(v > 10.0 && v < 90.0 && last_step > 0.0, "{path:?}");
    // Retargeted to 0 mid-flight: the next value is where its momentum
    // carries it (still rising, by about the last step), not a restart.
    let next = tick(&mut ui, 0.0);
    assert!(next > v, "momentum carries it past {v}: {next}");
    assert!(next - v < last_step * 1.01, "no jump: {v} → {next}");
    let settled = (0..60).map(|_| tick(&mut ui, 0.0)).last().unwrap();
    assert!(settled.abs() < 0.01, "{settled}");
}

/// A card with a `selection` group (off: scale 1, on: scale 1.5), off → on
/// on a 200 ms linear tween, on → off instant.
fn selectable(on: bool) -> Props {
    Props {
        states: vec![StateGroup {
            name: "selection".into(),
            states: vec![
                VisualState {
                    name: "off".into(),
                    values: vec![(Prop::Scale, 1.0)],
                    ..VisualState::default()
                },
                VisualState {
                    name: "on".into(),
                    values: vec![(Prop::Scale, 1.5)],
                    ..VisualState::default()
                },
            ],
            transitions: vec![
                StateTransition {
                    from: "off".into(),
                    to: "on".into(),
                    transition: Transition::Tween {
                        duration_ms: 200.0,
                        ease: Ease::Linear,
                    },
                },
                StateTransition {
                    from: "on".into(),
                    to: "off".into(),
                    transition: Transition::Instant,
                },
            ],
        }],
        state: vec![("selection".into(), if on { "on" } else { "off" }.into())],
        ..Props::default()
    }
}

fn card(ui: &mut Ui, on: bool) {
    ui.begin(InputFrame::default(), vp());
    padded(ui, |ui| boxed(ui, "Card", 100.0, 100.0, selectable(on)));
    ui.end();
}

#[test]
fn a_state_change_moves_on_its_pairs_transition() {
    let mut ui = Ui::new(UiDesc::default());
    for _ in 0..3 {
        card(&mut ui, false);
    }
    assert_eq!(rect(&ui, "P/Card")[2], 100.0);
    // off → on: 200 ms is 12 ticks at 60 Hz; the first tick starts it,
    // six more are halfway.
    let widths: Vec<f32> = (0..14)
        .map(|_| {
            card(&mut ui, true);
            rect(&ui, "P/Card")[2]
        })
        .collect();
    assert_eq!(widths[0], 100.0, "{widths:?}");
    assert!((widths[6] - 125.0).abs() < 0.01, "halfway: {widths:?}");
    assert!(widths.windows(2).all(|w| w[1] >= w[0]), "{widths:?}");
    assert_eq!(widths[12], 150.0, "{widths:?}");
    assert_eq!(
        ui.visual_state("P/Card", "selection").as_deref(),
        Some("on")
    );
    // on → off is instant.
    card(&mut ui, false);
    assert_eq!(rect(&ui, "P/Card")[2], 100.0);
    // The probe can force a state without the game asking.
    ui.force_state("P/Card", "selection", Some("on"));
    for _ in 0..14 {
        card(&mut ui, false);
    }
    assert_eq!(rect(&ui, "P/Card")[2], 150.0);
}

/// `Row` with `n` 20 px boxes, each 30 px apart.
fn row_of(ui: &mut Ui, n: usize) {
    ui.open(
        "Row",
        Element {
            direction: Direction::Row,
            gap: 10.0,
            ..Element::default()
        },
    );
    for k in 0..n {
        boxed(ui, &format!("C{k}"), 20.0, 20.0, Props::default());
    }
    ui.close();
}

#[test]
fn a_stagger_spreads_within_its_cap() {
    let mut ui = Ui::new(UiDesc::default());
    // Each target drops 50 px down to its place over 100 ms.
    ui.define_timeline(
        "drop",
        Timeline {
            tracks: vec![TimelineTrack {
                target: String::new(),
                prop: Prop::Y,
                keys: vec![
                    Key {
                        at_ms: 0.0,
                        value: -50.0,
                        ease: KeyEase::Hold,
                    },
                    Key {
                        at_ms: 100.0,
                        value: 0.0,
                        ease: KeyEase::Linear,
                    },
                ],
            }],
            ..Timeline::default()
        },
    );
    let frame = |ui: &mut Ui| {
        ui.begin(InputFrame::default(), vp());
        padded(ui, |ui| row_of(ui, 10));
        ui.end();
    };
    frame(&mut ui);
    ui.begin(InputFrame::default(), vp());
    padded(&mut ui, |ui| row_of(ui, 10));
    // 100 ms apart would put the last one 900 ms late; the cap is 300.
    let play = ui.play(
        "P/Row",
        Choreo::Stagger {
            timeline: "drop".into(),
            targets: Vec::new(),
            step_ms: 100.0,
            cap_ms: 300.0,
            from: StaggerFrom::Start,
        },
        PlayDesc::default(),
    );
    ui.end();
    assert_ne!(play, 0);
    let y = |ui: &Ui, k: usize| rect(ui, &format!("P/Row/C{k}"))[1];
    let rest = rect(&ui, "P/Row")[1];
    // The first one starts at once; the last, 300 ms (18 ticks) in.
    frame(&mut ui);
    assert!(y(&ui, 0) > rest - 50.0, "the first moves at once");
    assert_eq!(y(&ui, 9), rest - 50.0, "the last waits");
    for _ in 0..17 {
        frame(&mut ui);
    }
    assert_eq!(y(&ui, 9), rest - 50.0, "still waiting at 17 ticks");
    // 300 + 100 ms: everything has landed (well short of 900 + 100).
    for _ in 0..8 {
        frame(&mut ui);
    }
    for k in 0..10 {
        assert_eq!(y(&ui, k), rest, "C{k} landed within the cap");
    }
    assert!(
        ui.events()
            .iter()
            .any(|e| e.kind == "end" && e.play == play)
            || ui.play_info(play).is_none_or(|p| p.ended),
        "the play ended"
    );
}

#[test]
fn a_leaving_element_plays_its_exit_inert_and_out_of_the_tree() {
    let mut ui = Ui::new(UiDesc::default());
    let exit = PresenceSpec {
        values: vec![(Prop::Opacity, 0.0)],
        transition: Transition::Tween {
            duration_ms: 200.0,
            ease: Ease::Linear,
        },
        delay_ms: 0.0,
    };
    let props = Props {
        exit: Some(exit),
        ..Props::default()
    };
    let mut clicks = 0;
    let mut frame = |ui: &mut Ui, shown: bool, input: InputFrame| {
        ui.begin(input, vp());
        if shown {
            ui.open_with(
                "Toast",
                Element {
                    background: Some(Srgba::rgb(0, 0, 255)),
                    interactive: true,
                    ..fixed(100.0, 40.0)
                },
                props.clone(),
            );
            if ui.close().clicked {
                clicks += 1;
            }
        }
        ui.end();
    };
    frame(&mut ui, true, InputFrame::default());
    let blue = |ui: &Ui| {
        ui.frame()
            .items
            .iter()
            .filter_map(|it| match &it.cmd {
                DrawCmd::Quad { color, .. } if color[2] > 0.9 && color[0] < 0.1 => Some(color[3]),
                _ => None,
            })
            .next()
    };
    assert_eq!(blue(&ui), Some(1.0));
    // Gone from the build: still drawn, fading, but not an element, not in
    // the accessibility tree, and a press on it reaches nothing.
    let p = Vec2::new(50.0, 20.0);
    frame(&mut ui, false, press(p));
    let a = blue(&ui).expect("still drawn while it leaves");
    assert!(a > 0.8, "{a}");
    assert!(ui.frame().element("Toast").is_none());
    assert!(!ui.frame().a11y.nodes.iter().any(|n| n.path == "Toast"));
    assert!(
        ui.frame()
            .motion
            .iter()
            .any(|m| m.path == "Toast" && m.leaving)
    );
    frame(&mut ui, false, release(p));
    let half: Vec<f32> = (0..4)
        .map(|_| {
            frame(&mut ui, false, InputFrame::default());
            blue(&ui).unwrap_or(0.0)
        })
        .collect();
    assert!(half.windows(2).all(|w| w[1] < w[0]), "{half:?}");
    for _ in 0..8 {
        frame(&mut ui, false, InputFrame::default());
    }
    assert_eq!(blue(&ui), None, "done: no longer drawn");
    assert_eq!(clicks, 0);
}

#[test]
fn a_layout_change_moves_from_the_old_rectangle() {
    let mut ui = Ui::new(UiDesc::default());
    let props = Props {
        layout_motion: Some(Transition::Tween {
            duration_ms: 200.0,
            ease: Ease::Linear,
        }),
        ..Props::default()
    };
    let frame = |ui: &mut Ui, first: bool| {
        ui.begin(InputFrame::default(), vp());
        ui.open(
            "Col",
            Element {
                direction: Direction::Column,
                ..Element::default()
            },
        );
        if first {
            boxed(ui, "Spacer", 50.0, 100.0, Props::default());
        }
        boxed(ui, "Mover", 50.0, 50.0, props.clone());
        ui.close();
        ui.end();
    };
    for _ in 0..3 {
        frame(&mut ui, true);
    }
    let before = rect(&ui, "Col/Mover")[1];
    // The spacer goes: the mover's layout jumps 100 px up, its drawing
    // follows over 200 ms (12 ticks).
    frame(&mut ui, false);
    let first = rect(&ui, "Col/Mover")[1];
    assert!(
        first > before - 20.0,
        "starts from where it was: {before} → {first}"
    );
    let ys: Vec<f32> = (0..12)
        .map(|_| {
            frame(&mut ui, false);
            rect(&ui, "Col/Mover")[1]
        })
        .collect();
    assert!((ys[4] - (before - 50.0)).abs() < 10.0, "halfway: {ys:?}");
    assert_eq!(ys[11], before - 100.0, "{ys:?}");
}

#[test]
fn a_shared_element_moves_from_where_it_was_on_the_screen_before() {
    let mut ui = Ui::new(UiDesc::default());
    let hero = |ui: &mut Ui, id: &str, x: f32, size: f32| {
        ui.open(
            id,
            Element {
                padding: Insets {
                    left: x,
                    top: x,
                    ..Insets::default()
                },
                ..Element::default()
            },
        );
        boxed(
            ui,
            "Hero",
            size,
            size,
            Props {
                match_key: "hero".into(),
                layout_motion: Some(Transition::Tween {
                    duration_ms: 200.0,
                    ease: Ease::Linear,
                }),
                ..Props::default()
            },
        );
        ui.close();
    };
    let frame = |ui: &mut Ui, list: bool| {
        ui.begin(InputFrame::default(), vp());
        if list {
            hero(ui, "List", 0.0, 50.0);
        } else {
            hero(ui, "Detail", 300.0, 100.0);
        }
        ui.end();
    };
    for _ in 0..3 {
        frame(&mut ui, true);
    }
    assert_eq!(rect(&ui, "List/Hero"), [0.0, 0.0, 50.0, 50.0]);
    // The detail screen: its hero starts on the list's and grows into place.
    frame(&mut ui, false);
    let r = rect(&ui, "Detail/Hero");
    assert!(
        r[0] < 60.0 && r[2] < 60.0,
        "starts on the list's hero: {r:?}"
    );
    for _ in 0..12 {
        frame(&mut ui, false);
    }
    assert_eq!(rect(&ui, "Detail/Hero"), [300.0, 300.0, 100.0, 100.0]);
}

/// A 100×100 card on a carousel: three snap points 200 px apart.
fn carousel(ui: &mut Ui, input: InputFrame) {
    ui.begin(input, vp());
    boxed(
        ui,
        "Card",
        100.0,
        100.0,
        Props {
            drag: Some(DragSpec {
                axis: DragAxis::X,
                snap: vec![(0.0, 0.0), (200.0, 0.0), (400.0, 0.0)],
                momentum: true,
                ..DragSpec::default()
            }),
            ..Props::default()
        },
    );
    ui.end();
}

/// Drags the card `ticks` times by `dx`, holds it still `still` ticks,
/// releases it and lets it settle: the snap point it chose.
fn flick(dx: f32, ticks: usize, still: usize) -> GestureInfo {
    let mut ui = Ui::new(UiDesc::default());
    carousel(&mut ui, InputFrame::default());
    let mut p = Vec2::new(50.0, 50.0);
    carousel(&mut ui, press(p));
    for _ in 0..ticks {
        p.x += dx;
        // Diagonal pointer travel stays on the x axis.
        p.y += dx / 3.0;
        carousel(&mut ui, hold(p));
    }
    assert!(ui.gesture_at("Card").dragging);
    let dragged = ui.gesture_at("Card").offset_x;
    assert!((dragged - dx * ticks as f32).abs() < 0.01, "{dragged}");
    assert_eq!(rect(&ui, "Card")[1], 0.0, "x only");
    for _ in 0..still {
        carousel(&mut ui, hold(p));
    }
    carousel(&mut ui, release(p));
    for _ in 0..90 {
        carousel(&mut ui, at(p));
    }
    let g = ui.gesture_at("Card");
    let want = [0.0, 200.0, 400.0][g.snapped.max(0) as usize];
    assert!((g.offset_x - want).abs() < 0.01, "settled on it: {g:?}");
    assert!((rect(&ui, "Card")[0] - want).abs() < 0.01, "{g:?}");
    g
}

#[test]
fn a_flick_settles_on_the_snap_point_its_momentum_reaches() {
    // Each drag ends 90 px out, nearest the point at 0; a release projects
    // 0.3 s of its velocity. 30 px a tick (1800 px/s) reaches past 400; 3 px
    // a tick (180 px/s) reaches 144, nearest 200; held still, it stays at 90.
    assert_eq!(flick(30.0, 3, 0).snapped, 2);
    assert_eq!(flick(3.0, 30, 0).snapped, 1);
    assert_eq!(flick(3.0, 30, 20).snapped, 0);
}

#[test]
fn a_snapshot_taken_mid_animation_continues_the_same() {
    let tl = Timeline {
        tracks: vec![TimelineTrack {
            target: String::new(),
            prop: Prop::Rotate,
            keys: vec![
                Key {
                    at_ms: 0.0,
                    value: 0.0,
                    ease: KeyEase::Linear,
                },
                Key {
                    at_ms: 500.0,
                    value: 1.0,
                    ease: KeyEase::Spring(SpringSpec::new(300.0, 0.4)),
                },
            ],
        }],
        ..Timeline::default()
    };
    let make = || {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_timeline("spin", tl.clone());
        ui
    };
    let mut a = make();
    for _ in 0..2 {
        card(&mut a, false);
    }
    a.begin(InputFrame::default(), vp());
    padded(&mut a, |ui| {
        boxed(ui, "Card", 100.0, 100.0, selectable(true))
    });
    a.play_timeline("P/Card", "spin", PlayDesc::default());
    a.end();
    for _ in 0..7 {
        card(&mut a, true);
    }
    let snap = a.save_state();
    let mut b = make();
    b.restore_state(&snap).expect("restores");
    let run = |ui: &mut Ui| -> Vec<[f32; 4]> {
        (0..30)
            .map(|_| {
                card(ui, true);
                rect(ui, "P/Card")
            })
            .collect()
    };
    let (ra, rb) = (run(&mut a), run(&mut b));
    assert_ne!(ra[0], ra[29], "it was moving");
    assert_eq!(ra, rb);
    assert_eq!(a.save_state(), b.save_state());
}

#[test]
fn grid_tracks_spans_and_areas() {
    let mut ui = Ui::new(UiDesc::default());
    ui.begin(InputFrame::default(), vp());
    ui.open_with(
        "G",
        Element {
            gap: 0.0,
            ..fixed(400.0, 200.0)
        },
        Props {
            layout: LayoutMode::Grid,
            grid: GridSpec {
                // 100 px, then 300 shared 1:2.
                columns: vec![Track::Px(100.0), Track::Fr(1.0), Track::Fr(2.0)],
                rows: vec![Track::Px(50.0), Track::Fr(1.0)],
                areas: vec!["head head head".into(), "side main main".into()],
                ..GridSpec::default()
            },
            ..Props::default()
        },
    );
    let place = |area: &str| Props {
        place: GridPlace {
            area: area.into(),
            ..GridPlace::default()
        },
        ..Props::default()
    };
    let grow = Element {
        width: Size::grow(),
        height: Size::grow(),
        ..Element::default()
    };
    for (id, area) in [("Head", "head"), ("Side", "side"), ("Main", "main")] {
        ui.open_with(id, grow, place(area));
        ui.close();
    }
    ui.close();
    ui.end();
    assert_eq!(rect(&ui, "G/Head"), [0.0, 0.0, 400.0, 50.0]);
    assert_eq!(rect(&ui, "G/Side"), [0.0, 50.0, 100.0, 150.0]);
    assert_eq!(rect(&ui, "G/Main"), [100.0, 50.0, 300.0, 150.0]);
    // Auto-flow with a span: [A A] [B], then C wraps to the next row.
    ui.begin(InputFrame::default(), vp());
    ui.open_with(
        "F",
        fixed(300.0, 100.0),
        Props {
            layout: LayoutMode::Grid,
            grid: GridSpec {
                columns: Track::repeat(3, Track::Fr(1.0)),
                ..GridSpec::default()
            },
            ..Props::default()
        },
    );
    let span = Props {
        place: GridPlace {
            column_span: 2,
            ..GridPlace::default()
        },
        ..Props::default()
    };
    ui.open_with("A", fixed(10.0, 20.0), span);
    ui.close();
    boxed(&mut ui, "B", 10.0, 20.0, Props::default());
    boxed(&mut ui, "C", 10.0, 20.0, Props::default());
    ui.close();
    ui.end();
    assert_eq!(rect(&ui, "F/A")[0], 0.0);
    assert_eq!(rect(&ui, "F/B")[0], 200.0);
    assert_eq!(rect(&ui, "F/C")[..2], [0.0, 20.0]);
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
fn an_element_declared_over_everything_is_never_reported() {
    let mut ui = Ui::new(UiDesc::default());
    let frame = |ui: &mut Ui, decl: Vec<String>| {
        ui.begin(InputFrame::default(), vp());
        ui.open("P", Element::default());
        boxed(ui, "Enemy", 100.0, 100.0, Props::default());
        // An aiming arc drawn across the enemy.
        boxed(
            ui,
            "Arc",
            200.0,
            40.0,
            Props {
                position: Position::Absolute,
                insets: Edges {
                    start: Some(20.0),
                    top: Some(30.0),
                    ..Edges::default()
                },
                overlaps: decl,
                ..Props::default()
            },
        );
        ui.close();
        ui.end();
        ui.frame()
            .overlaps
            .iter()
            .any(|o| o.path == "P/Arc" || o.other == "P/Arc")
    };
    assert!(frame(&mut ui, Vec::new()), "undeclared, it is reported");
    assert!(
        !frame(&mut ui, vec!["*".into()]),
        "`*` declares it over anything"
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

#[test]
fn an_overlap_is_measured_where_a_transform_draws_the_element() {
    // A command cell's key cap laid out on a 32 px line and scaled to half
    // about its top-left corner, with the seconds placed below the scaled
    // cap: they meet only on the unscaled line. A badge scaled up past its
    // box, the other way, is reported where it is drawn.
    let mut ui = Ui::new(UiDesc::default());
    let at = |left: f32, top: f32, transform: Transform| Props {
        position: Position::Absolute,
        insets: Edges {
            start: Some(left),
            top: Some(top),
            ..Edges::default()
        },
        transform,
        ..Props::default()
    };
    let half = Transform {
        scale_x: 0.5,
        scale_y: 0.5,
        pivot_x: 0.0,
        pivot_y: 0.0,
        ..Transform::IDENTITY
    };
    let double = Transform {
        scale_x: 2.0,
        scale_y: 2.0,
        ..Transform::IDENTITY
    };
    ui.begin(InputFrame::default(), vp());
    ui.open("Cell", fixed(200.0, 200.0));
    boxed(&mut ui, "Key", 20.0, 32.0, at(0.0, 0.0, half));
    boxed(
        &mut ui,
        "Time",
        30.0,
        20.0,
        at(0.0, 16.0, Transform::IDENTITY),
    );
    boxed(&mut ui, "Badge", 20.0, 20.0, at(100.0, 100.0, double));
    boxed(
        &mut ui,
        "Tag",
        20.0,
        20.0,
        at(122.0, 100.0, Transform::IDENTITY),
    );
    ui.close();
    ui.end();
    let pairs: Vec<_> = ui
        .frame()
        .overlaps
        .iter()
        .map(|o| (o.path.clone(), o.other.clone()))
        .collect();
    let meet = |a: &str, b: &str| {
        pairs
            .iter()
            .any(|(x, y)| (x == a && y == b) || (x == b && y == a))
    };
    assert!(!meet("Cell/Key", "Cell/Time"), "{pairs:?}");
    assert!(meet("Cell/Badge", "Cell/Tag"), "{pairs:?}");
}

#[test]
fn an_unchanged_frame_keeps_its_layout_and_commands() {
    let mut ui = Ui::new(UiDesc::default());
    let frame = |ui: &mut Ui, n: usize| {
        ui.begin(InputFrame::default(), vp());
        row_of(ui, n);
        ui.end();
        ui.frame().stats
    };
    let first = frame(&mut ui, 5);
    assert_eq!(first.rebuilt, first.elements);
    let same = frame(&mut ui, 5);
    assert_eq!(same.rebuilt, 0, "{same:?}");
    assert_eq!(same.laid_out, 0, "{same:?}");
    // Every element but the root (which draws nothing): the row and its
    // five boxes.
    assert_eq!(same.kept_commands, 6, "{same:?}");
    let items = ui.frame().items.clone();
    // One more box: the row and the new box are rebuilt and laid out.
    let grown = frame(&mut ui, 6);
    assert!(grown.rebuilt >= 1 && grown.laid_out >= 1, "{grown:?}");
    let back = frame(&mut ui, 5);
    assert!(back.laid_out >= 1, "{back:?}");
    frame(&mut ui, 5);
    assert_eq!(ui.frame().items, items, "the kept commands draw the same");
}

#[test]
fn start_and_end_insets_and_transforms_mirror_right_to_left() {
    let frame = |dir: og_i18n::Direction| {
        let mut ui = Ui::new(UiDesc::default());
        ui.set_direction(dir);
        ui.begin(InputFrame::default(), vp());
        ui.open("Bar", fixed(200.0, 50.0));
        boxed(
            &mut ui,
            "Badge",
            20.0,
            20.0,
            Props {
                position: Position::Absolute,
                insets: Edges {
                    start: Some(10.0),
                    top: Some(5.0),
                    ..Edges::default()
                },
                ..Props::default()
            },
        );
        boxed(
            &mut ui,
            "Nudge",
            20.0,
            20.0,
            Props {
                transform: Transform {
                    x: 30.0,
                    ..Transform::IDENTITY
                },
                ..Props::default()
            },
        );
        ui.close();
        ui.end();
        let bar = rect(&ui, "Bar");
        let local = |r: [f32; 4]| [r[0] - bar[0], r[1] - bar[1], r[2], r[3]];
        (local(rect(&ui, "Bar/Badge")), local(rect(&ui, "Bar/Nudge")))
    };
    let (badge, nudge) = frame(og_i18n::Direction::Ltr);
    assert_eq!(badge, [10.0, 5.0, 20.0, 20.0]);
    assert_eq!(nudge[0], 30.0);
    // 10 px in from the bar's right edge; the flow starts at the right, and
    // the nudge moves towards the end (left).
    let (badge, nudge) = frame(og_i18n::Direction::Rtl);
    assert_eq!(badge, [170.0, 5.0, 20.0, 20.0]);
    assert_eq!(nudge[0], 180.0 - 30.0);
}

#[test]
fn a_container_query_reads_the_elements_own_width() {
    // The same card, a row when it is at least 300 px wide, a column below.
    let mut ui = Ui::new(UiDesc::default());
    ui.begin(InputFrame::default(), vp());
    for (slot, w) in [("Wide", 400.0), ("Narrow", 200.0)] {
        ui.open(slot, fixed(w, 200.0));
        // Unique ids: the probe resolves a path's ancestors by id.
        ui.open_with(
            format!("{slot}Card").as_str(),
            Element {
                width: Size::grow(),
                direction: Direction::Column,
                ..Element::default()
            },
            Props {
                container: true,
                variants: vec![Variant {
                    when: Query {
                        min_width: 300.0,
                        container: true,
                        ..Query::default()
                    },
                    set: Patch {
                        direction: Some(Direction::Row),
                        ..Patch::default()
                    },
                }],
                ..Props::default()
            },
        );
        boxed(&mut ui, "A", 40.0, 40.0, Props::default());
        boxed(&mut ui, "B", 40.0, 40.0, Props::default());
        ui.close();
        ui.close();
    }
    ui.end();
    let b = |slot: &str| rect(&ui, &format!("{slot}/{slot}Card/B"));
    assert_eq!(b("Wide")[..2], [40.0, 0.0], "a row in the wide slot");
    assert_eq!(b("Narrow")[..2], [0.0, 240.0], "a column in the narrow one");
}

#[test]
fn an_anchored_element_sits_on_its_targets_point() {
    let mut ui = Ui::new(UiDesc::default());
    for _ in 0..2 {
        ui.begin(InputFrame::default(), vp());
        padded(&mut ui, |ui| {
            boxed(ui, "Card", 100.0, 60.0, Props::default());
            boxed(
                ui,
                "Tip",
                80.0,
                30.0,
                Props {
                    position: Position::Anchored,
                    anchor_to: "P/Card".into(),
                    anchor_at: Anchor::Bottom,
                    anchor_self: Anchor::Top,
                    insets: Edges {
                        top: Some(4.0),
                        ..Edges::default()
                    },
                    ..Props::default()
                },
            );
        });
        ui.end();
    }
    // The card's bottom centre is (150, 160); the tip's top centre sits 4 px under it.
    assert_eq!(rect(&ui, "P/Tip"), [110.0, 164.0, 80.0, 30.0]);
}

#[test]
fn reduced_motion_jumps_movement_and_fades_opacity() {
    let mut ui = Ui::new(UiDesc::default());
    let mut t = *ui.theme();
    t.reduced_motion = true;
    ui.set_theme(t);
    for _ in 0..3 {
        card(&mut ui, false);
    }
    // off → on is a 200 ms tween; under reduced motion the scale jumps.
    card(&mut ui, true);
    assert_eq!(rect(&ui, "P/Card")[2], 150.0);
    // A spring's number jumps as well.
    ui.begin(InputFrame::default(), vp());
    ui.animate("v", 0.0, Transition::Spring(SpringSpec::new(400.0, 0.0)));
    ui.end();
    ui.begin(InputFrame::default(), vp());
    let v = ui.animate("v", 100.0, Transition::Spring(SpringSpec::new(400.0, 0.0)));
    ui.end();
    assert_eq!(v, 100.0);
}

#[test]
fn a_swipe_past_its_distance_dismisses() {
    let sheet = |ui: &mut Ui, input: InputFrame| {
        ui.begin(input, vp());
        padded(ui, |ui| {
            boxed(
                ui,
                "Toast",
                200.0,
                60.0,
                Props {
                    drag: Some(DragSpec {
                        axis: DragAxis::X,
                        return_home: true,
                        dismiss: Some(Dismiss::default()),
                        ..DragSpec::default()
                    }),
                    ..Props::default()
                },
            );
        });
        ui.end();
    };
    // 40 % of 200 is 80: a slow 60 px drag goes home, a slow 100 px one is thrown off.
    for (dx, thrown) in [(2.0, false), (4.0, true)] {
        let mut ui = Ui::new(UiDesc::default());
        sheet(&mut ui, InputFrame::default());
        let mut p = Vec2::new(150.0, 130.0);
        sheet(&mut ui, press(p));
        for _ in 0..25 {
            p.x += dx;
            sheet(&mut ui, hold(p));
        }
        for _ in 0..10 {
            sheet(&mut ui, hold(p));
        }
        sheet(&mut ui, release(p));
        let mut events: Vec<String> = ui.events().iter().map(|e| e.kind.clone()).collect();
        for _ in 0..60 {
            sheet(&mut ui, at(p));
            events.extend(ui.events().iter().map(|e| e.kind.clone()));
        }
        let g = ui.gesture_at("P/Toast");
        assert_eq!(g.dismissed, thrown, "{dx}: {g:?}");
        assert_eq!(events.iter().any(|k| k == "dismiss"), thrown, "{events:?}");
        if !thrown {
            assert!(g.offset_x.abs() < 0.01, "home: {g:?}");
        }
    }
}

#[test]
fn a_cascade_starts_each_step_at_the_callback_of_the_one_before() {
    let mut ui = Ui::new(UiDesc::default());
    // Each count lasts 300 ms and calls `next` 100 ms in.
    ui.define_timeline(
        "count",
        Timeline {
            tracks: vec![TimelineTrack {
                target: String::new(),
                prop: Prop::Y,
                keys: vec![
                    Key {
                        at_ms: 0.0,
                        value: 10.0,
                        ease: KeyEase::Hold,
                    },
                    Key {
                        at_ms: 300.0,
                        value: 0.0,
                        ease: KeyEase::Linear,
                    },
                ],
            }],
            events: vec![
                TimelineEvent {
                    at_ms: 100.0,
                    target: String::new(),
                    kind: EventKind::Callback("next".into()),
                },
                TimelineEvent {
                    at_ms: 0.0,
                    target: String::new(),
                    kind: EventKind::Sound {
                        name: "press".into(),
                        pitch: 1.0,
                    },
                },
            ],
            ..Timeline::default()
        },
    );
    let frame = |ui: &mut Ui| {
        ui.begin(InputFrame::default(), vp());
        padded(ui, |ui| row_of(ui, 3));
        ui.end();
    };
    frame(&mut ui);
    ui.begin(InputFrame::default(), vp());
    padded(&mut ui, |ui| row_of(ui, 3));
    let step = |k: usize| Choreo::Play {
        timeline: "count".into(),
        target: format!("C{k}"),
        delay_ms: 0.0,
    };
    ui.play(
        "P/Row",
        Choreo::Cascade {
            steps: vec![step(0), step(1), step(2)],
            on: "next".into(),
        },
        PlayDesc::default(),
    );
    ui.end();
    // The sounds land 100 ms (6 ticks) apart, each pitched a step higher;
    // the first in the frame the play starts.
    let mut sounds: Vec<_> = ui
        .events()
        .iter()
        .filter(|e| e.kind == "sound")
        .map(|e| (-1, e.target.clone(), e.index, e.pitch))
        .collect();
    for t in 0..40 {
        frame(&mut ui);
        for e in ui.events().iter().filter(|e| e.kind == "sound") {
            sounds.push((t, e.target.clone(), e.index, e.pitch));
        }
    }
    assert_eq!(sounds.len(), 3, "{sounds:?}");
    assert_eq!(sounds[1].0 - sounds[0].0, 6, "{sounds:?}");
    assert_eq!(sounds[2].0 - sounds[1].0, 6, "{sounds:?}");
    assert!(sounds[2].1.ends_with("C2"), "{sounds:?}");
    assert!(
        sounds[2].3 > sounds[1].3 && sounds[1].3 > sounds[0].3,
        "{sounds:?}"
    );
}

/// A list's rows are options with §7.3 groups: `selection` from the game,
/// `interaction` from input, so a hover fades in on the style's spring
/// instead of jumping (and the checker sees their visual states).
#[test]
fn a_list_rows_hover_and_selection_move_through_state_groups() {
    let mut ui = Ui::new(UiDesc::default());
    let frame = |ui: &mut Ui, input: InputFrame| {
        ui.begin(input, vp());
        ui.list(
            "L",
            ListDesc {
                count: 3,
                row_height: 40.0,
                height: Some(Size::Fixed(200.0)),
                ..ListDesc::default()
            },
            Some(1),
            |ui, k| {
                ui.label(ui_idi("Text", k), "Row", LabelDesc::default());
            },
        );
        ui.end();
    };
    frame(&mut ui, InputFrame::default());
    frame(&mut ui, InputFrame::default());
    assert_eq!(
        ui.visual_state("L/Row#1", "selection").as_deref(),
        Some("on")
    );
    assert_eq!(
        ui.visual_state("L/Row#0", "selection").as_deref(),
        Some("off")
    );
    let r = rect(&ui, "L/Row#0");
    let at = (r[0] + r[2] / 2.0, r[1] + r[3] / 2.0);
    let fill = |ui: &Ui| {
        ui.frame()
            .items
            .iter()
            .filter_map(|it| it.cmd.filled())
            .filter(|(q, _)| {
                (q.x - r[0]).abs() < 0.5
                    && (q.width - r[2]).abs() < 0.5
                    && at.1 >= q.y
                    && at.1 <= q.y + q.height
            })
            .map(|(_, c)| c[3])
            .next_back()
            .unwrap_or(0.0)
    };
    assert_eq!(fill(&ui), 0.0, "clear at rest");
    let hover = InputFrame {
        pointer: Vec2::new(at.0, at.1),
        pointer_inside: true,
        ..InputFrame::default()
    };
    let alphas: Vec<f32> = (0..30)
        .map(|_| {
            frame(&mut ui, hover.clone());
            fill(&ui)
        })
        .collect();
    assert_eq!(
        ui.visual_state("L/Row#0", "interaction").as_deref(),
        Some("hover")
    );
    // Hovered, it reaches the palette's `control` wash, through values between.
    let want = Theme::default().palette.control.alpha;
    let last = *alphas.last().unwrap();
    assert!((last - want).abs() < 0.005, "{alphas:?}");
    assert!(
        alphas.iter().any(|a| *a > 0.2 * want && *a < 0.8 * want),
        "it moves, not jumps: {alphas:?}"
    );
}

#[test]
fn a_part_inside_a_moved_container_keeps_inside_its_clip() {
    // A sheet slides up by a transform; the checkbox inside it draws its
    // box through the UI pass, whose clip is the screen's: a clip left in
    // the layout box cut the box away while the check over it drew.
    let mut ui = Ui::new(UiDesc::default());
    let step = |ui: &mut Ui| {
        ui.begin(InputFrame::default(), vp());
        ui.open("Sheet", fixed(300.0, 200.0));
        let mut on = true;
        ui.checkbox("Weapons", &mut on, "Weapons");
        ui.close();
        ui.set_props(Props {
            transform: Transform {
                y: 350.0,
                ..Transform::IDENTITY
            },
            ..Props::default()
        });
        ui.end();
    };
    step(&mut ui);
    step(&mut ui);
    let boxes: Vec<_> = ui
        .frame()
        .items
        .iter()
        .filter_map(|i| match &i.cmd {
            DrawCmd::Shape { shape, clip } if shape.mode == og_render::UiMode::Fill => {
                Some((i.transform.apply(shape.center.x, shape.center.y), *clip))
            }
            _ => None,
        })
        .collect();
    assert!(!boxes.is_empty(), "the box is a part drawn as a shape");
    for ((x, y), clip) in boxes {
        assert!(y > 350.0, "drawn where the sheet moved it: {y}");
        assert!(
            x >= clip.x && x <= clip.x + clip.width && y >= clip.y && y <= clip.y + clip.height,
            "its clip holds it: ({x}, {y}) in {clip:?}"
        );
    }
}

/// An entrance in flight is in the frame's motion (what dump_ui shows and
/// the style playtests wait on before they capture): `presence` while it
/// enters, `glitch` while a glitch tears it, and nothing once it settles.
#[test]
fn an_entrance_in_flight_is_reported_until_it_settles() {
    let mut ui = Ui::new(UiDesc::default());
    let entering = |ui: &Ui| -> Vec<String> {
        ui.frame()
            .motion
            .iter()
            .filter(|m| m.path == "Card")
            .flat_map(|m| m.moving.clone())
            .collect()
    };
    let mut seen = Vec::new();
    for _ in 0..30 {
        ui.begin(InputFrame::default(), vp());
        ui.open_transition(
            "Card",
            true,
            TransitionDesc {
                kind: TransitionKind::Glitch,
                ..TransitionDesc::default()
            },
        );
        ui.label("Text", "Hello", LabelDesc::default());
        ui.close();
        ui.end();
        seen.push(entering(&ui));
    }
    assert_eq!(seen[0], vec!["presence", "glitch"], "a glitch entering");
    let settled = seen.iter().position(|m| m.is_empty()).expect("it settles");
    assert!(
        settled <= 12,
        "a glitch keeps to 180 ms: settled at {settled}"
    );
    assert!(seen[settled..].iter().all(|m| m.is_empty()), "{seen:?}");
}
