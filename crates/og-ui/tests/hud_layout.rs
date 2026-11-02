//! A pad's virtual cursor (docs/ui.md §8.3) and the player's HUD layout
//! (§8.4), driven tick by tick.
//!
//! Ways these could fail, each covered below:
//! - the stick moves the focus as well as the cursor, or A both presses
//!   under the cursor and activates the focused control;
//! - the cursor starts at full speed (no fine aim) or never reaches it, or
//!   crosses a control as fast as empty space (no magnet);
//! - the real pointer moving leaves the cursor shown (two pointers), or
//!   the system cursor stays up over the drawn one;
//! - the right stick scrolls nothing, or scrolls the wrong way;
//! - a saved layout moves the element but not where its touches land, or
//!   applies to another player;
//! - in the edit mode the element still answers its own press, a drag
//!   leaves it off the surface or past a locked move, the change is never
//!   reported, or a snapshot mid-drag continues otherwise;
//! - Done leaves the edit mode on.

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

fn at(p: Vec2) -> InputFrame {
    InputFrame {
        pointer: p,
        pointer_inside: true,
        ..InputFrame::default()
    }
}

fn pad(stick: Vec2, pressed: u32, held: u32) -> InputFrame {
    InputFrame {
        gamepad: GamepadFrame {
            stick,
            pressed,
            held,
            ..GamepadFrame::default()
        },
        ..InputFrame::default()
    }
}

fn bounds(ui: &Ui, path: &str) -> (f32, f32, f32, f32) {
    let e = ui.frame().element(path).unwrap_or_else(|| {
        let all: Vec<&str> = ui
            .frame()
            .elements
            .iter()
            .map(|e| e.path.as_str())
            .collect();
        panic!("{path} on screen: {all:?}")
    });
    (e.x, e.y, e.width, e.height)
}

/// Two buttons in a row at the top left, `A` then `B`, under a virtual
/// cursor declared every tick; answers which were clicked.
fn cursor_screen(ui: &mut Ui, input: InputFrame, desc: VirtualCursorDesc) -> Vec<&'static str> {
    ui.begin_with(
        input,
        vp(),
        Element {
            direction: Direction::Row,
            gap: 200.0,
            ..Element::column()
        },
    );
    ui.set_virtual_cursor(Some(desc));
    let mut out = Vec::new();
    for id in ["A", "B"] {
        if ui
            .button(
                id,
                id,
                ButtonDesc {
                    width: Some(Size::Fixed(100.0)),
                    ..ButtonDesc::default()
                },
            )
            .clicked
        {
            out.push(id);
        }
    }
    ui.end();
    out
}

#[test]
fn the_stick_moves_the_cursor_with_a_fine_start_and_a_magnet_and_a_presses_under_it() {
    let mut ui = Ui::new(UiDesc::default());
    let d = VirtualCursorDesc::default();
    // Declared on the first frame, read from the next.
    cursor_screen(&mut ui, at(Vec2::new(400.0, 400.0)), d);
    cursor_screen(&mut ui, at(Vec2::new(400.0, 400.0)), d);
    assert_eq!(ui.virtual_cursor(), Some((Vec2::new(400.0, 400.0), false)));
    // Full tilt right: the first tick a third of 900 units a second.
    cursor_screen(&mut ui, pad(Vec2::new(1.0, 0.0), 0, 0), d);
    let (p1, shown) = ui.virtual_cursor().unwrap();
    assert!(shown, "the stick shows it");
    assert!((p1.x - 405.0).abs() < 0.01, "a fine start: {p1}");
    assert_eq!(ui.frame().cursor, Cursor::None, "the system cursor hides");
    // A quarter second on, full speed: 15 units a tick.
    for _ in 0..20 {
        cursor_screen(&mut ui, pad(Vec2::new(1.0, 0.0), 0, 0), d);
    }
    let (p2, _) = ui.virtual_cursor().unwrap();
    cursor_screen(&mut ui, pad(Vec2::new(1.0, 0.0), 0, 0), d);
    let (p3, _) = ui.virtual_cursor().unwrap();
    assert!(
        (p3.x - p2.x - 15.0).abs() < 0.01,
        "full speed: {}",
        p3.x - p2.x
    );
    // The stick moves no focus while the cursor is the pad's pointer.
    assert!(ui.frame().focus_paths[0].is_empty(), "no focus moved");

    // Up to B's row, then over B: the magnet slows it there.
    let (bx, by, bw, bh) = bounds(&ui, "B");
    let mut guard = 0;
    while ui.virtual_cursor().unwrap().0.y > by + bh / 2.0 && guard < 200 {
        cursor_screen(&mut ui, pad(Vec2::new(0.0, -1.0), 0, 0), d);
        guard += 1;
    }
    // Then left onto B: the magnet slows it over B.
    let mut free = 0.0f32;
    let mut slowed = 0.0f32;
    while ui.virtual_cursor().unwrap().0.x > bx + bw / 2.0 && guard < 400 {
        let before = ui.virtual_cursor().unwrap().0.x;
        cursor_screen(&mut ui, pad(Vec2::new(-1.0, 0.0), 0, 0), d);
        let step = before - ui.virtual_cursor().unwrap().0.x;
        if before > bx + bw + 20.0 {
            free = step;
        } else if before < bx + bw {
            slowed = step;
        }
        guard += 1;
    }
    assert!(
        slowed > 0.0 && slowed < free * 0.75,
        "slower over B ({slowed}) than before it ({free})"
    );
    // A presses B; its release clicks it; A activates nothing else.
    let clicked = cursor_screen(
        &mut ui,
        pad(Vec2::ZERO, GamepadFrame::A, GamepadFrame::A),
        d,
    );
    assert!(clicked.is_empty(), "pressed, not yet clicked");
    let clicked = cursor_screen(&mut ui, pad(Vec2::ZERO, 0, 0), d);
    assert_eq!(clicked, vec!["B"], "the release over B clicks it");

    // The real pointer moving hides it; the system cursor is back.
    cursor_screen(&mut ui, at(Vec2::new(10.0, 300.0)), d);
    assert_eq!(ui.virtual_cursor(), Some((Vec2::new(10.0, 300.0), false)));
    assert_ne!(ui.frame().cursor, Cursor::None);
}

#[test]
fn the_right_stick_scrolls_under_the_cursor() {
    let mut ui = Ui::new(UiDesc::default());
    let d = VirtualCursorDesc::default();
    let build = |ui: &mut Ui, input: InputFrame| {
        ui.begin(input, vp());
        ui.set_virtual_cursor(Some(d));
        ui.scroll(
            "List",
            ScrollDesc {
                width: Size::Fixed(300.0),
                height: Size::Fixed(200.0),
                ..ScrollDesc::default()
            },
            |ui| {
                for k in 0..30 {
                    ui.label(Id::from(format!("Row{k}")), "row", LabelDesc::default());
                }
            },
        );
        ui.end();
    };
    build(&mut ui, at(Vec2::new(100.0, 100.0)));
    build(&mut ui, at(Vec2::new(100.0, 100.0)));
    let y0 = bounds(&ui, "List/Row9").1;
    for _ in 0..3 {
        let mut i = pad(Vec2::ZERO, 0, 0);
        i.gamepad.stick_r = Vec2::new(0.0, 1.0);
        build(&mut ui, i);
    }
    let y1 = bounds(&ui, "List/Row9").1;
    assert!(
        y1 < y0 - 40.0,
        "down on the right stick scrolls down: {y0} → {y1}"
    );
}

/// A HUD with a customisable stick button at the bottom right (key
/// `jump`), a locked map at the top right of the editor and the editor; answers whether jump clicked.
fn hud(ui: &mut Ui, input: InputFrame, editor: bool) -> (bool, HudEditorResponse) {
    ui.begin(input, vp());
    let mut out = (false, HudEditorResponse::default());
    ui.open(
        "Map",
        Element {
            width: Size::Fixed(120.0),
            height: Size::Fixed(80.0),
            ..Element::default()
        },
    );
    ui.close();
    ui.set_props(Props {
        position: Position::Absolute,
        insets: Edges {
            start: Some(450.0),
            top: Some(60.0),
            ..Edges::default()
        },
        customize: Some(Customize {
            lock_move: true,
            ..Customize::default()
        }),
        ..Props::default()
    });
    out.0 = ui
        .action_button("Jump", "Jump", &ActionButtonDesc::default())
        .clicked;
    ui.set_props(Props {
        position: Position::Absolute,
        insets: Edges {
            end: Some(40.0),
            bottom: Some(40.0),
            ..Edges::default()
        },
        customize: Some(Customize {
            key: "jump".into(),
            ..Customize::default()
        }),
        ..Props::default()
    });
    if editor {
        out.1 = ui.hud_layout_editor("Editor", &HudEditorDesc::default());
    }
    ui.end();
    out
}

#[test]
fn a_saved_layout_moves_and_scales_the_element_and_where_it_is_pressed() {
    let mut ui = Ui::new(UiDesc::default());
    hud(&mut ui, InputFrame::default(), false);
    let (x0, y0, w0, h0) = bounds(&ui, "Jump");
    ui.set_hud_layout(1, r#"{"jump":{"x":-300,"y":-200,"scale":1.5}}"#)
        .expect("reads");
    hud(&mut ui, InputFrame::default(), false);
    assert_eq!(bounds(&ui, "Jump"), (x0, y0, w0, h0), "player 1's, not 0's");
    ui.set_hud_player(1);
    hud(&mut ui, InputFrame::default(), false);
    let (x1, y1, w1, h1) = bounds(&ui, "Jump");
    let c0 = (x0 + w0 / 2.0, y0 + h0 / 2.0);
    let c1 = (x1 + w1 / 2.0, y1 + h1 / 2.0);
    assert!((c1.0 - (c0.0 - 300.0)).abs() < 0.5 && (c1.1 - (c0.1 - 200.0)).abs() < 0.5);
    assert!((w1 - w0 * 1.5).abs() < 0.5, "scaled about its centre");
    // A press where it is drawn now clicks it; where it was, nothing.
    let mut clicked = false;
    for input in [
        InputFrame {
            pressed: true,
            down: true,
            ..at(Vec2::new(c1.0, c1.1))
        },
        InputFrame {
            released: true,
            ..at(Vec2::new(c1.0, c1.1))
        },
    ] {
        clicked |= hud(&mut ui, input, false).0;
    }
    assert!(clicked, "its touches follow it");
    assert!(ui.set_hud_layout(1, "[1]").is_err());
    assert!(
        ui.hud_layout(1).contains("\"jump\""),
        "a refused layout keeps the old"
    );
    assert_eq!(ui.hud_layout(0), "{}");
}

#[test]
fn the_edit_mode_drags_without_pressing_keeps_it_on_the_surface_and_reports_the_change() {
    let mut ui = Ui::new(UiDesc::default());
    hud(&mut ui, InputFrame::default(), true);
    ui.set_hud_edit(true);
    hud(&mut ui, InputFrame::default(), true);
    let (x0, y0, w0, h0) = bounds(&ui, "Jump");
    let c = Vec2::new(x0 + w0 / 2.0, y0 + h0 / 2.0);
    let press = InputFrame {
        pressed: true,
        down: true,
        ..at(c)
    };
    let (clicked, _) = hud(&mut ui, press, true);
    assert!(!clicked);
    assert_eq!(ui.hud_edit().selected, "jump");
    // Halfway through the drag, a snapshot.
    let hold = |p: Vec2| InputFrame {
        down: true,
        ..at(p)
    };
    hud(&mut ui, hold(c + Vec2::new(-100.0, -50.0)), true);
    let snap = ui.save_state();
    let finish = |ui: &mut Ui| {
        let mut changed = false;
        let mut clicked = false;
        for input in [
            hold(c + Vec2::new(-150.0, -60.0)),
            InputFrame {
                released: true,
                ..at(c + Vec2::new(-150.0, -60.0))
            },
        ] {
            clicked |= hud(ui, input, true).0;
            changed |= ui.hud_edit().changed;
        }
        (clicked, changed, ui.hud_layout(0))
    };
    let (clicked, changed, layout) = finish(&mut ui);
    assert!(!clicked, "an edited element answers nothing");
    assert!(changed, "the move's end is reported");
    assert_eq!(
        layout,
        r#"{"jump":{"opacity":1.0,"scale":1.0,"x":-150.0,"y":-60.0}}"#
    );
    let mut back = Ui::new(UiDesc::default());
    back.restore_state(&snap).expect("restores");
    assert_eq!(
        finish(&mut back).2,
        layout,
        "the snapshot continues the drag"
    );

    // Far past the surface's corner: its centre stays on.
    let c = {
        hud(&mut ui, InputFrame::default(), true);
        let (x, y, w, h) = bounds(&ui, "Jump");
        Vec2::new(x + w / 2.0, y + h / 2.0)
    };
    for input in [
        InputFrame {
            pressed: true,
            down: true,
            ..at(c)
        },
        hold(Vec2::new(5000.0, 5000.0)),
        InputFrame {
            released: true,
            ..at(Vec2::new(5000.0, 5000.0))
        },
    ] {
        hud(&mut ui, input, true);
    }
    hud(&mut ui, InputFrame::default(), true);
    let (x, y, w, h) = bounds(&ui, "Jump");
    assert!(x + w <= 800.5 && y + h <= 600.5, "inside: {x},{y} {w}x{h}");

    // The locked map does not move.
    let (mx, my, mw, mh) = bounds(&ui, "Map");
    for input in [
        InputFrame {
            pressed: true,
            down: true,
            ..at(Vec2::new(mx + mw / 2.0, my + mh / 2.0))
        },
        hold(Vec2::new(400.0, 300.0)),
        InputFrame {
            released: true,
            ..at(Vec2::new(400.0, 300.0))
        },
    ] {
        hud(&mut ui, input, true);
    }
    assert_eq!(
        ui.hud_edit().selected,
        "Map",
        "selected by its path: {:?} {:?}",
        ui.hud_edit().keys,
        (mx, my, mw, mh)
    );
    assert_eq!(bounds(&ui, "Map"), (mx, my, mw, mh));
    // Its size steps up a tenth from the editor.
    ui.hud_step(1, 0);
    hud(&mut ui, InputFrame::default(), true);
    let w = bounds(&ui, "Map").2;
    assert!(
        (w - mw * 1.1).abs() < 0.5,
        "{mw} → {w}: {}",
        ui.hud_layout(0)
    );
}

#[test]
fn done_on_the_editor_ends_the_edit_mode() {
    let mut ui = Ui::new(UiDesc::default());
    ui.set_hud_edit(true);
    hud(&mut ui, InputFrame::default(), true);
    let (x, y, w, h) = bounds(&ui, "Editor/Actions/Done");
    let c = Vec2::new(x + w / 2.0, y + h / 2.0);
    hud(
        &mut ui,
        InputFrame {
            pressed: true,
            down: true,
            ..at(c)
        },
        true,
    );
    let (_, r) = hud(
        &mut ui,
        InputFrame {
            released: true,
            ..at(c)
        },
        true,
    );
    assert!(r.done);
    assert!(!ui.hud_edit().on);
}
