use super::*;

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
    ui.element(path).map(|e| Vec2::new(e.x, e.y))
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
    let view = rig.ui.element("Board").expect("board").clone();
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
        let r = ui.element("Board").expect("board").clone();
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
    let r = rig.ui.element("Board").expect("board").clone();
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
        rig.ui.element("Board/Row#0/Tile#0").is_none() || y < r.y - 100.0,
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

#[derive(Default)]
struct Gestures {
    clicks: u32,
    zoom: Option<ZoomResponse>,
}

fn gestures(ui: &mut Ui, s: &mut Gestures) {
    ui.row("Top", Element::row(), |ui| {
        ui.scroll(
            "List",
            ScrollDesc {
                width: Size::px(200.0),
                height: Size::px(200.0),
                ..ScrollDesc::default()
            },
            |ui| {
                for k in 0..20 {
                    if ui
                        .button(ui_idi("Row", k), &format!("Row {k}"), ButtonDesc::default())
                        .clicked
                    {
                        s.clicks += 1;
                    }
                }
            },
        );
        s.zoom = Some(ui.zoom_view(
            "Map",
            &ZoomViewDesc {
                width: Size::px(300.0),
                height: Size::px(200.0),
                ..ZoomViewDesc::default()
            },
        ));
    });
    ui.column(
        "Pictures",
        Element {
            width: Size::px(400.0),
            ..Element::column()
        },
        |ui| {
            ui.image(
                "Wide",
                ImageDesc {
                    width: Size::grow(),
                    aspect: 2.0,
                    ..ImageDesc::default()
                },
            );
            ui.image(
                "Tall",
                ImageDesc {
                    height: Size::px(90.0),
                    aspect: 0.5,
                    ..ImageDesc::default()
                },
            );
        },
    );
}

/// Two fingers (og.Pointers): moved together over a list of buttons they
/// scroll it (one finger would press a row) and the row the first finger
/// landed on is not clicked; spread apart over a zoom view they zoom it
/// about their centre, moved together they pan it; the wheel zooms about
/// the pointer; the zoom stays in its limits and the view on its picture;
/// a pinch in progress replays after a restore. An image with an aspect
/// ratio is as tall as its width over it, or as wide as its height times it.
///
/// Ways it could fail: the two touches fight the press (the row clicks,
/// the list does not move), the pinch zooms about the view's corner (the
/// centre moves), the zoom runs past its limits or the view off its
/// picture, the gesture is outside the UI state, or the aspect is ignored.
#[test]
fn two_fingers_scroll_lists_and_pinch_zoom_views() {
    let mut rig = Rig::new(Gestures::default());
    let b = &mut gestures;
    rig.idle(2, b);
    // The aspect ratios.
    let wide = rig.ui.element("Pictures/Wide").expect("wide").clone();
    assert_eq!(
        (wide.width, wide.height),
        (400.0, 200.0),
        "grow 400 wide at 2:1"
    );
    let tall = rig.ui.element("Pictures/Tall").expect("tall").clone();
    assert_eq!((tall.width, tall.height), (45.0, 90.0), "90 tall at 1:2");
    // Two-finger scroll over the list, starting on a row.
    let row3 = rig.center("Top/List/Row#3");
    let y3 = pos(&rig.ui, "Top/List/Row#3").expect("row 3").y;
    let mut f = Fingers::new();
    rig.step(f.at(&[(0, row3)]), b);
    rig.step(f.at(&[(0, row3), (1, row3 + Vec2::new(40.0, 0.0))]), b);
    for k in 1..=3 {
        let up = Vec2::new(0.0, -20.0 * k as f32);
        rig.step(
            f.at(&[(0, row3 + up), (1, row3 + Vec2::new(40.0, 0.0) + up)]),
            b,
        );
    }
    // Held still a tick (lifted, they would fling it on).
    let up = Vec2::new(0.0, -60.0);
    rig.step(
        f.at(&[(0, row3 + up), (1, row3 + Vec2::new(40.0, 0.0) + up)]),
        b,
    );
    assert_eq!(
        pos(&rig.ui, "Top/List/Row#3").expect("row 3").y,
        y3 - 60.0,
        "the list followed the two fingers up by 60"
    );
    rig.step(f.lift(1), b);
    rig.step(f.lift(0), b);
    rig.idle(1, b);
    assert_eq!(
        rig.state.clicks, 0,
        "the row the first finger landed on is not clicked"
    );
    // A pinch over the map: 100 apart to 200 apart about its centre.
    let map = rig.center("Top/Map");
    let mut f = Fingers::new();
    let apart = |d: f32| {
        [
            (0, map - Vec2::new(d / 2.0, 0.0)),
            (1, map + Vec2::new(d / 2.0, 0.0)),
        ]
    };
    rig.step(f.at(&apart(100.0)[..1]), b);
    rig.step(f.at(&apart(100.0)), b);
    for d in [125.0, 150.0, 175.0, 200.0] {
        rig.step(f.at(&apart(d)), b);
    }
    let z = rig.state.zoom.expect("zoom");
    assert!(
        (z.zoom - 2.0).abs() < 1e-4,
        "doubling the spread doubles the zoom ({})",
        z.zoom
    );
    assert!(
        (z.center - Vec2::splat(0.5)).length() < 1e-4,
        "about the centre: the middle stays ({:?})",
        z.center
    );
    // Both fingers right by 60: the picture follows them.
    let saved = rig.ui.save_state();
    let moved = |d: f32| {
        let a = apart(200.0);
        [
            (0, a[0].1 + Vec2::new(d, 0.0)),
            (1, a[1].1 + Vec2::new(d, 0.0)),
        ]
    };
    rig.step(f.at(&moved(30.0)), b);
    rig.step(f.at(&moved(60.0)), b);
    let panned = rig.state.zoom.expect("zoom");
    let want = 0.5 - 60.0 / (300.0 * 2.0);
    assert!(
        (panned.center.x - want).abs() < 1e-4 && (panned.center.y - 0.5).abs() < 1e-4,
        "panned left by 60 px of a 2x view: {:?} (want x {want})",
        panned.center
    );
    // A restore in the middle of the gesture replays it.
    rig.ui.restore_state(&saved).expect("restore");
    let mut g = Fingers { p: f.p };
    g.p = {
        let mut p = og_input::Pointers::default();
        for (id, at) in apart(200.0) {
            p.apply(&og_input::InjectedInput::Pointer {
                action: og_input::PointerAction::Down,
                x: at.x,
                y: at.y,
                button: og_input::PointerButton::Left,
                id,
                movement_x: 0.0,
                movement_y: 0.0,
            });
        }
        p
    };
    rig.step(g.at(&moved(30.0)), b);
    rig.step(g.at(&moved(60.0)), b);
    assert_eq!(rig.state.zoom, Some(panned), "the restored pinch replays");
    rig.step(g.lift(0), b);
    rig.step(g.lift(1), b);
    // The limits: spread far apart, the zoom stops at 4; the view stays on the picture.
    let mut f = Fingers::new();
    rig.step(f.at(&apart(40.0)[..1]), b);
    rig.step(f.at(&apart(40.0)), b);
    rig.step(f.at(&apart(400.0)), b);
    rig.step(
        f.at(&apart(400.0).map(|(id, p)| (id, p + Vec2::new(500.0, 0.0)))),
        b,
    );
    rig.idle(1, b);
    let z = rig.state.zoom.expect("zoom");
    assert_eq!(z.zoom, 4.0, "the zoom stops at its limit");
    assert!(
        z.center.x >= 0.125 - 1e-6,
        "the view stays on the picture ({:?})",
        z.center
    );
    rig.step(f.lift(1), b);
    rig.step(f.lift(0), b);
    // The wheel zooms out about the pointer.
    rig.step(wheel(map, Vec2::new(0.0, 300.0)), b);
    rig.idle(1, b);
    let out = rig.state.zoom.expect("zoom");
    assert!(out.zoom < 4.0, "a wheel down zooms out ({})", out.zoom);
}
