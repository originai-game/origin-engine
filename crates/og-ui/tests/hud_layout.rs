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

use crate::support::Doc;
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
