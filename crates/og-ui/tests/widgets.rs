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
//! - toasts pile up past three, never leave, or differ after a restore;
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

mod appearance;
mod controls;
mod navigation;
mod scrolling;
