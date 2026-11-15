//! A pad's virtual cursor (docs/ui.md §2.1) and the player's HUD layout
//! (og-ui's README, §Gestures and the cursor), driven tick by tick.
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
//!   leaves it off the surface, the change is never reported, or a
//!   snapshot mid-drag continues otherwise (a locked move, which no
//!   document declares, is checked in `interaction::hud_layout`'s tests);
//! - Done leaves the edit mode on;
//! - a popup placed at a point (a drift score) covers the map, or moves it
//!   instead of stepping clear itself.

use crate::support::{Doc, engine_dir};
use og_core::glam::Vec2;
use og_ui::*;
use serde_json::Value;

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
    let e = ui.element(path).unwrap_or_else(|| {
        let all: Vec<&str> = ui.elements().iter().map(|e| e.path.as_str()).collect();
        panic!("{path} on screen: {all:?}")
    });
    (e.x, e.y, e.width, e.height)
}

/// The ids of the elements this tick's events clicked.
fn clicked(ui: &mut Ui) -> Vec<String> {
    ui.take_doc_events()
        .into_iter()
        .filter(|e| e.kind == "click")
        .map(|e| e.path.rsplit('/').next().unwrap_or_default().to_owned())
        .collect()
}

/// Two 100-wide buttons in a row at the top left, `A` then `B`.
const CURSOR_SCREEN: &str = r#"<meta name="component" content="pad">
<row id="Pad"><button id="A" onclick={a}>A</button><button id="B" onclick={b}>B</button></row>
<style>
#Pad { gap: 200px; align-items: start; }
#A, #B { width: 100px; }
</style>
"#;

/// A tick of [`CURSOR_SCREEN`] under a virtual cursor declared every tick;
/// answers which buttons were clicked.
fn cursor_screen(d: &mut Doc, input: InputFrame, desc: VirtualCursorDesc) -> Vec<String> {
    let ui = &mut d.ui;
    ui.begin(input, vp());
    ui.set_virtual_cursor(Some(desc));
    ui.build_retained_documents(&bind::NoEnv).expect("builds");
    ui.end();
    clicked(ui)
}

#[test]
fn the_stick_moves_the_cursor_with_a_fine_start_and_a_magnet_and_a_presses_under_it() {
    let mut d = Doc::new(CURSOR_SCREEN, &[]);
    let d = &mut d;
    let v = VirtualCursorDesc::default();
    // Declared on the first frame, read from the next.
    cursor_screen(d, at(Vec2::new(400.0, 400.0)), v);
    cursor_screen(d, at(Vec2::new(400.0, 400.0)), v);
    assert_eq!(
        d.ui.virtual_cursor(),
        Some((Vec2::new(400.0, 400.0), false))
    );
    // Full tilt right: the first tick a third of 900 units a second.
    cursor_screen(d, pad(Vec2::new(1.0, 0.0), 0, 0), v);
    let (p1, shown) = d.ui.virtual_cursor().unwrap();
    assert!(shown, "the stick shows it");
    assert!((p1.x - 405.0).abs() < 0.01, "a fine start: {p1}");
    assert_eq!(d.ui.frame().cursor, Cursor::None, "the system cursor hides");
    // A quarter second on, full speed: 15 units a tick.
    for _ in 0..20 {
        cursor_screen(d, pad(Vec2::new(1.0, 0.0), 0, 0), v);
    }
    let (p2, _) = d.ui.virtual_cursor().unwrap();
    cursor_screen(d, pad(Vec2::new(1.0, 0.0), 0, 0), v);
    let (p3, _) = d.ui.virtual_cursor().unwrap();
    assert!(
        (p3.x - p2.x - 15.0).abs() < 0.01,
        "full speed: {}",
        p3.x - p2.x
    );
    // The stick moves no focus while the cursor is the pad's pointer.
    assert!(d.ui.frame().focus_paths[0].is_empty(), "no focus moved");

    // Up to B's row, then over B: the magnet slows it there.
    let (bx, by, bw, bh) = bounds(&d.ui, "B");
    let mut guard = 0;
    while d.ui.virtual_cursor().unwrap().0.y > by + bh / 2.0 && guard < 200 {
        cursor_screen(d, pad(Vec2::new(0.0, -1.0), 0, 0), v);
        guard += 1;
    }
    // Then left onto B: the magnet slows it over B.
    let mut free = 0.0f32;
    let mut slowed = 0.0f32;
    while d.ui.virtual_cursor().unwrap().0.x > bx + bw / 2.0 && guard < 400 {
        let before = d.ui.virtual_cursor().unwrap().0.x;
        cursor_screen(d, pad(Vec2::new(-1.0, 0.0), 0, 0), v);
        let step = before - d.ui.virtual_cursor().unwrap().0.x;
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
    let clicks = cursor_screen(d, pad(Vec2::ZERO, GamepadFrame::A, GamepadFrame::A), v);
    assert!(clicks.is_empty(), "pressed, not yet clicked");
    let clicks = cursor_screen(d, pad(Vec2::ZERO, 0, 0), v);
    assert_eq!(clicks, ["B"], "the release over B clicks it");

    // The real pointer moving hides it; the system cursor is back.
    cursor_screen(d, at(Vec2::new(10.0, 300.0)), v);
    assert_eq!(d.ui.virtual_cursor(), Some((Vec2::new(10.0, 300.0), false)));
    assert_ne!(d.ui.frame().cursor, Cursor::None);
}

/// A HUD with a customisable map at the top right (keyed by its path) and
/// a customisable button at the bottom right (key `jump`).
const HUD: &str = r#"<meta name="screen" content="hud" data-layer="hud">
<div id="Hud">
  <div id="Map" customize></div>
  <button id="Jump" customize customize-key="jump" onclick={jump}>Jump</button>
</div>
<style>
#Hud { width: stretch; height: stretch; }
#Map { position: absolute; left: 450px; top: 60px; width: 120px; height: 80px; }
#Jump { position: absolute; right: 40px; bottom: 40px; }
</style>
"#;

/// A UI that knows [`HUD`] and the built-in `hud-editor` screen.
fn hud_ui() -> Ui {
    let mut ui = Ui::new(UiDesc::default());
    ui.define_document(UiDocument::parse("hud.svelte", HUD).unwrap());
    let path = engine_dir("src/ui/screens/hud-editor.svelte");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
    ui.define_document(UiDocument::parse("hud-editor.svelte", &text).unwrap());
    ui
}

/// A tick of the mounted screens; answers whether Jump clicked.
fn hud(ui: &mut Ui, input: InputFrame) -> bool {
    ui.begin(input, vp());
    ui.build_retained_documents(&bind::NoEnv).expect("builds");
    ui.end();
    clicked(ui).iter().any(|id| id == "Jump")
}

#[test]
fn the_edit_mode_drags_without_pressing_keeps_it_on_the_surface_and_reports_the_change() {
    let mut ui = hud_ui();
    ui.push_screen("hud", Value::Null).unwrap();
    hud(&mut ui, InputFrame::default());
    let (x0, y0, w0, h0) = bounds(&ui, "Jump");
    let c = Vec2::new(x0 + w0 / 2.0, y0 + h0 / 2.0);
    let press = InputFrame {
        pressed: true,
        down: true,
        ..at(c)
    };
    let hold = |p: Vec2| InputFrame {
        down: true,
        ..at(p)
    };
    let release = |p: Vec2| InputFrame {
        released: true,
        ..at(p)
    };
    // Out of the edit mode it is a button.
    hud(&mut ui, press.clone());
    assert!(hud(&mut ui, release(c)), "Jump answers its press");

    // The edit mode takes the press itself: it selects, and the button
    // answers nothing (the editor screen, a modal, would also keep the
    // HUD under it inert).
    ui.set_hud_edit(true);
    hud(&mut ui, InputFrame::default());
    assert!(!hud(&mut ui, press.clone()));
    assert_eq!(ui.hud_edit().selected, "jump");
    assert!(!hud(&mut ui, release(c)), "the edit mode took the press");
    // The player opens the editor screen and drags.
    ui.push_screen("hud-editor", Value::Null).unwrap();
    hud(&mut ui, InputFrame::default());
    assert!(!hud(&mut ui, press));
    assert_eq!(ui.hud_edit().selected, "jump");
    // Halfway through the drag, a snapshot.
    hud(&mut ui, hold(c + Vec2::new(-100.0, -50.0)));
    let snap = ui.save_state();
    let finish = |ui: &mut Ui| {
        let mut changed = false;
        let mut clicked = false;
        for input in [
            hold(c + Vec2::new(-150.0, -60.0)),
            release(c + Vec2::new(-150.0, -60.0)),
        ] {
            clicked |= hud(ui, input);
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
    let mut back = hud_ui();
    back.restore_state(&snap).expect("restores");
    assert_eq!(
        finish(&mut back).2,
        layout,
        "the snapshot continues the drag"
    );

    // Far past the surface's corner: its centre stays on.
    let c = {
        hud(&mut ui, InputFrame::default());
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
        release(Vec2::new(5000.0, 5000.0)),
    ] {
        hud(&mut ui, input);
    }
    hud(&mut ui, InputFrame::default());
    let (x, y, w, h) = bounds(&ui, "Jump");
    assert!(x + w <= 800.5 && y + h <= 600.5, "inside: {x},{y} {w}x{h}");

    // A press on the map selects it, keyed by its path.
    let (mx, my, mw, mh) = bounds(&ui, "Map");
    let m = Vec2::new(mx + mw / 2.0, my + mh / 2.0);
    for input in [
        InputFrame {
            pressed: true,
            down: true,
            ..at(m)
        },
        release(m),
    ] {
        hud(&mut ui, input);
    }
    assert_eq!(
        ui.hud_edit().selected,
        "hud/Hud/Map",
        "selected by its path: {:?} {:?}",
        ui.hud_edit().keys,
        (mx, my, mw, mh)
    );
    assert_eq!(bounds(&ui, "Map"), (mx, my, mw, mh));
    // Its size steps up a tenth from the editor.
    ui.hud_step(1, 0);
    hud(&mut ui, InputFrame::default());
    let w = bounds(&ui, "Map").2;
    assert!(
        (w - mw * 1.1).abs() < 0.5,
        "{mw} → {w}: {}",
        ui.hud_layout(0)
    );
}
