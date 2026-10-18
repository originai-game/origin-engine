//! Focus navigation: keyboard (arrows, Tab, Shift+Tab, Enter, Space,
//! Escape), gamepad (d-pad, left stick with repeat, A, B) and queued
//! intents move one focus between last frame's focusable controls.
//!
//! - **Next/Prev** walk the controls in reading (draw) order and wrap.
//! - **Directions** pick the nearest control whose centre lies that way,
//!   scoring the distance along the direction plus three times the gap
//!   across it (so the control in the same row or column wins); with none,
//!   focus wraps to the far end of the same row or column.
//! - A dialog or open menu is a **scope**: while it is the topmost, focus
//!   stays inside it, and it takes Back.
//! - Navigation that lands on a control outside its scroll container's view
//!   scrolls it into view.

use crate::input::{GamepadFrame, InputFrame, Nav, UiKey};
use crate::layout::Rect;
use crate::ui::{Hit, Ui, HIT_ADJUST, HIT_NAV, HIT_TEXT};

/// The control nearest `from` in direction `dir` among `cands`, wrapping.
pub(crate) fn neighbour(from: &Rect, cands: &[(u64, Rect)], dir: Nav) -> Option<u64> {
    let (fx, fy) = from.center();
    // (along, across): along > 0 is in the direction.
    let axes = |r: &Rect| -> (f32, f32) {
        let (cx, cy) = r.center();
        let gap = |a0: f32, a1: f32, b0: f32, b1: f32| (b0 - a1).max(a0 - b1).max(0.0);
        match dir {
            Nav::Down => (cy - fy, gap(from.x, from.right(), r.x, r.right())),
            Nav::Up => (fy - cy, gap(from.x, from.right(), r.x, r.right())),
            Nav::Right => (cx - fx, gap(from.y, from.bottom(), r.y, r.bottom())),
            _ => (fx - cx, gap(from.y, from.bottom(), r.y, r.bottom())),
        }
    };
    let mut best: Option<(f32, u64)> = None;
    for (k, r) in cands {
        let (along, across) = axes(r);
        if along > 0.5 {
            let score = along + 3.0 * across;
            if best.is_none_or(|(s, _)| score < s) {
                best = Some((score, *k));
            }
        }
    }
    if best.is_some() {
        return best.map(|b| b.1);
    }
    // Wrap: the farthest the other way, in the same row or column first.
    for (k, r) in cands {
        let (along, across) = axes(r);
        if along < -0.5 {
            let score = along + 3.0 * across;
            if best.is_none_or(|(s, _)| score < s) {
                best = Some((score, *k));
            }
        }
    }
    best.map(|b| b.1)
}

impl Ui {
    /// This tick's intents: queued ones, then the keyboard, then the gamepad.
    pub(crate) fn intents(&mut self, input: &InputFrame) -> Vec<Nav> {
        let mut out = std::mem::take(&mut self.state.pending);
        let editing = self.hit(self.state.focus).is_some_and(|h| h.has(HIT_TEXT));
        for k in &input.keys {
            let nav = match k {
                UiKey::Up => Some(Nav::Up),
                UiKey::Down => Some(Nav::Down),
                UiKey::Left if !editing => Some(Nav::Left),
                UiKey::Right if !editing => Some(Nav::Right),
                UiKey::Tab if input.shift => Some(Nav::Prev),
                UiKey::Tab => Some(Nav::Next),
                UiKey::Enter | UiKey::NumpadEnter | UiKey::Space if !editing => Some(Nav::Activate),
                UiKey::Escape if !editing => Some(Nav::Back),
                _ => None,
            };
            out.extend(nav);
        }
        let g = input.gamepad;
        for (bit, nav) in [
            (GamepadFrame::UP, Nav::Up),
            (GamepadFrame::DOWN, Nav::Down),
            (GamepadFrame::LEFT, Nav::Left),
            (GamepadFrame::RIGHT, Nav::Right),
            (GamepadFrame::A, Nav::Activate),
            (GamepadFrame::B, Nav::Back),
        ] {
            if g.pressed & bit != 0 {
                out.push(nav);
            }
        }
        // The stick: past half-way on its dominant axis, once, then after the
        // slow duration every fast duration while held.
        let s = g.stick;
        let dir = if s.x.abs().max(s.y.abs()) < 0.5 {
            0
        } else if s.x.abs() > s.y.abs() {
            1 + if s.x > 0.0 { Nav::Right } else { Nav::Left }.index()
        } else {
            1 + if s.y > 0.0 { Nav::Down } else { Nav::Up }.index()
        };
        let now = self.state.tick;
        if dir == 0 {
            self.state.stick = (0, 0);
        } else if self.state.stick.0 != dir {
            self.state.stick = (dir, now);
            out.extend(Nav::from_index(dir - 1));
        } else {
            let held = now - self.state.stick.1;
            let delay = u64::from(self.ticks_for(self.theme().durations_ms.slow));
            let every = u64::from(self.ticks_for(self.theme().durations_ms.fast));
            if held >= delay && (held - delay).is_multiple_of(every) {
                out.extend(Nav::from_index(dir - 1));
            }
        }
        out
    }

    /// The focusable controls of last frame in reading order, inside the open scope.
    fn focusables(&self) -> Vec<Hit> {
        let scope = self.state.scope;
        self.state
            .hits
            .iter()
            .filter(|h| h.has(HIT_NAV | HIT_TEXT) && (scope == 0 || h.scope == scope))
            .copied()
            .collect()
    }

    pub(crate) fn apply_nav(&mut self, intents: &[Nav]) {
        if intents.is_empty() {
            return;
        }
        let enabled = !self.state.nav_off || self.state.scope != 0;
        let mut moved = false;
        for &nav in intents {
            match nav {
                Nav::Back => {
                    self.nav.back = true;
                    continue;
                }
                _ if !enabled => continue,
                Nav::Activate => {
                    if self.hit(self.state.focus).is_some_and(|h| h.has(HIT_NAV)) {
                        self.nav.activate = self.state.focus;
                        self.state.focus_visible = true;
                    }
                    continue;
                }
                _ => {}
            }
            let cands = self.focusables();
            if cands.is_empty() {
                continue;
            }
            self.state.focus_visible = true;
            let current = cands.iter().position(|h| h.key == self.state.focus);
            let adjustable = current.is_some_and(|i| cands[i].has(HIT_ADJUST));
            let next = match (nav, current) {
                (Nav::Left | Nav::Right, Some(_)) if adjustable => {
                    self.nav.adjust += if nav == Nav::Right { 1 } else { -1 };
                    None
                }
                (Nav::Next, None) => Some(cands[0].key),
                (Nav::Prev, None) => Some(cands[cands.len() - 1].key),
                (Nav::Next, Some(i)) => Some(cands[(i + 1) % cands.len()].key),
                (Nav::Prev, Some(i)) => Some(cands[(i + cands.len() - 1) % cands.len()].key),
                (_, None) => Some(cands[0].key),
                (dir, Some(i)) => {
                    let from = cands[i].full;
                    let others: Vec<(u64, Rect)> = cands
                        .iter()
                        .filter(|h| h.key != cands[i].key)
                        .map(|h| (h.key, h.full))
                        .collect();
                    neighbour(&from, &others, dir)
                }
            };
            if let Some(k) = next {
                self.state.focus = k;
                moved = true;
            }
        }
        if moved {
            self.scroll_into_view(self.state.focus);
        }
    }

    /// Scrolls the containers around `key` so it is in their view.
    fn scroll_into_view(&mut self, key: u64) {
        let Some(h) = self.hit(key).copied() else {
            return;
        };
        if h.scroller == 0 {
            return;
        }
        let Some(view) = self.hit(h.scroller).map(|s| s.rect) else {
            return;
        };
        let pad = self.theme().space.xs;
        let off = self.state.scroll.entry(h.scroller).or_insert(0.0);
        if h.full.y < view.y {
            *off -= view.y - h.full.y + pad;
        } else if h.full.bottom() > view.bottom() {
            *off += h.full.bottom() - view.bottom() + pad;
        }
        *off = off.max(0.0);
    }
}
