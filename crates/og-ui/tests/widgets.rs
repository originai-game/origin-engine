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
//! - a HUD drawn before a dialog takes Back from it (`Ui::back` answering
//!   while a dialog Back closes is open), so Escape never closes pause;
//! - a select does not open, choose, or close on a press outside;
//! - a virtualised list builds rows out of view, cannot reach its last row,
//!   or does not scroll a keyboard-focused row into view;
//! - transitions advance by anything but ticks, take input while moving, or
//!   keep drawing once gone;
//! - a meter's trail jumps instead of following, or differs between runs;
//! - a meter's label takes one ink over a fill and a track it cannot read
//!   on both of (a light pack's deep green under a dark ink);
//! - the dialogue types at the wrong rate, a press does not finish the line,
//!   or `advanced` fires before the line is shown;
//! - anchors ignore the safe area; touch controls come out under 44 px;
//! - an anchored group sits under the host's own control (the web shell's
//!   Back, `Viewport::covered`) instead of beside it;
//! - a game accent leaves the style palette's derived tones stale;
//! - the gamepad stick repeats at the wrong ticks.

use crate::support;

use og_core::glam::Vec2;
use og_ui::*;
use support::*;

/// Document `text` (component `t`, mounted as `test:t`) over `stores`, in
/// an 800 × 600 viewport (`viewport` another), built twice.
fn mount_at(text: &str, stores: &[(&str, serde_json::Value)], viewport: Viewport) -> Doc {
    let mut d = Doc::new(text, stores);
    d.viewport = viewport;
    idle(&mut d, 2);
    d
}

fn mount(text: &str, stores: &[(&str, serde_json::Value)]) -> Doc {
    mount_at(text, stores, Viewport::new(800.0, 600.0))
}

fn idle(d: &mut Doc, n: u32) {
    for _ in 0..n {
        d.step(InputFrame::default());
    }
}

/// The centre of the element whose path ends with `id`.
fn center(d: &Doc, id: &str) -> Vec2 {
    let (x, y) = d.get(id).center();
    Vec2::new(x, y)
}

/// A press and its release on `id`, one tick each (no settling after).
fn tap(d: &mut Doc, id: &str) {
    let p = center(d, id);
    d.step(press(p));
    d.step(release(p));
}

/// Pad `pad`'s focus, without the mount's prefix.
fn focus(d: &Doc, pad: u32) -> String {
    let path = d.ui.focus_path(pad);
    path.strip_prefix("test:t/").unwrap_or(path).to_string()
}

/// The names of the `click` events `d` collected since the last call.
fn clicks(d: &mut Doc) -> Vec<String> {
    std::mem::take(&mut d.events)
        .into_iter()
        .filter(|e| e.kind == "click")
        .map(|e| e.path.rsplit('/').next().unwrap_or_default().to_string())
        .collect()
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

#[path = "widgets/appearance.rs"]
mod appearance;
#[path = "widgets/controls.rs"]
mod controls;
#[path = "widgets/navigation.rs"]
mod navigation;
#[path = "widgets/scrolling.rs"]
mod scrolling;
#[path = "widgets/touch.rs"]
mod touch;

/// Touches as og-input's `og.Pointers` holds them, tick by tick: each
/// `(id, at)` is down at `at` (landing this tick when it was not held).
struct Fingers {
    p: og_input::Pointers,
}

impl Fingers {
    fn new() -> Fingers {
        Fingers {
            p: og_input::Pointers::default(),
        }
    }

    fn event(&mut self, action: og_input::PointerAction, id: u32, at: Vec2) {
        self.p
            .apply(&og_input::InjectedInput::Pointer {
                action,
                x: at.x,
                y: at.y,
                button: og_input::PointerButton::Left,
                id,
                movement_x: 0.0,
                movement_y: 0.0,
            })
            .expect("a pointer id under five");
    }

    /// This tick's frame with the fingers at `at` (each id down there).
    fn at(&mut self, at: &[(u32, Vec2)]) -> InputFrame {
        self.p.end_tick();
        for &(id, pos) in at {
            let action = if self.p.down(id) {
                og_input::PointerAction::Move
            } else {
                og_input::PointerAction::Down
            };
            self.event(action, id, pos);
        }
        self.frame()
    }

    /// This tick's frame with finger `id` lifted.
    fn lift(&mut self, id: u32) -> InputFrame {
        self.p.end_tick();
        let pos = self.p.position(id);
        self.event(og_input::PointerAction::Up, id, pos);
        self.frame()
    }

    /// The UI's frame: pointer 0 is og.Input's pointer.
    fn frame(&self) -> InputFrame {
        let p0 = self.p.position(0);
        InputFrame {
            pointer: p0,
            pointer_inside: true,
            down: self.p.down(0),
            pressed: self.p.is_pressed(0),
            released: self.p.is_released(0),
            pointers: self.p,
            ..InputFrame::default()
        }
    }
}
