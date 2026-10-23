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
use crate::ui::{HIT_ADJUST, HIT_MULTILINE, HIT_NAV, HIT_TEXT, Hit, Ui};

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
        // Moves queued while the focus was on nothing: the first puts it on
        // the first control (a move from nothing does); the rest, and all of
        // them once the focus is somewhere, are spent.
        let mut entered = self.state.focus != 0;
        let mut out = Vec::new();
        for (nav, entering) in std::mem::take(&mut self.state.pending) {
            if entering {
                if entered {
                    continue;
                }
                entered = true;
            }
            // From nothing, Next lands on the first control (Prev would on the last).
            out.push(if entering { Nav::Next } else { nav });
        }
        let editing = self.hit(self.state.focus).is_some_and(|h| h.has(HIT_TEXT));
        let lines = self
            .hit(self.state.focus)
            .is_some_and(|h| h.has(HIT_MULTILINE));
        for k in &input.keys {
            let nav = match k {
                UiKey::Up if !lines => Some(Nav::Up),
                UiKey::Down if !lines => Some(Nav::Down),
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
        // A pad intent the shell also delivered as its mapped key this tick
        // (the web's pad-to-keys fallback) counts once.
        let from_keys = out.clone();
        let g = input.gamepad;
        for (bit, nav) in [
            (GamepadFrame::UP, Nav::Up),
            (GamepadFrame::DOWN, Nav::Down),
            (GamepadFrame::LEFT, Nav::Left),
            (GamepadFrame::RIGHT, Nav::Right),
            (GamepadFrame::A, Nav::Activate),
            (GamepadFrame::B, Nav::Back),
        ] {
            if g.pressed & bit != 0 && !from_keys.contains(&nav) {
                out.push(nav);
                if nav == Nav::Activate {
                    self.nav.pad0 = true;
                }
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
            out.extend(Nav::from_index(dir - 1).filter(|n| !from_keys.contains(n)));
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

    /// An assistive technology's requests (og-input's `og.A11yAction`,
    /// [`InputFrame::a11y`]): each names a control by its path in the last
    /// frame's accessibility tree. Focus moves there (with the ring, as
    /// keyboard navigation shows it), Activate presses it as Enter would,
    /// Increment and Decrement step an adjustable one, and a stepper's
    /// spinbutton as a whole (its − and + are the controls focus reaches).
    /// A path naming no focusable control of the topmost scope is ignored,
    /// as a press outside an open dialog is; a gameplay screen's
    /// `set_navigation(false)` does not stop them.
    pub(crate) fn apply_a11y(&mut self, acts: &[(og_input::A11yAct, String)]) {
        use og_input::A11yAct;
        for (act, path) in acts {
            let key = crate::id::key_of_path(path);
            let Some(hit) = self.focusables().into_iter().find(|h| h.key == key) else {
                let step = match act {
                    A11yAct::Increment => 1,
                    A11yAct::Decrement => -1,
                    _ => continue,
                };
                // A stepper's spinbutton: one of its buttons is in reach.
                let spin = self
                    .frame
                    .a11y
                    .nodes
                    .iter()
                    .any(|n| n.path == *path && n.role == "spinbutton");
                let buttons = [format!("{path}/Inc"), format!("{path}/Dec")]
                    .map(|p| crate::id::key_of_path(&p));
                if spin && self.focusables().iter().any(|h| buttons.contains(&h.key)) {
                    let (k, n) = self.nav.spin;
                    self.nav.spin = (key, if k == key { n + step } else { step });
                }
                continue;
            };
            if self.state.focus != key {
                self.sound(crate::style_file::UiSound::Hover);
            }
            self.state.focus = key;
            self.state.focus_visible = true;
            match act {
                A11yAct::Focus => {}
                A11yAct::Activate if hit.has(HIT_NAV) => {
                    self.nav.activate = key;
                    self.sound(crate::style_file::UiSound::Press);
                }
                // A text control's activation is its focus: it edits.
                A11yAct::Activate => {}
                A11yAct::Increment if hit.has(HIT_ADJUST) => self.nav.adjust += 1,
                A11yAct::Decrement if hit.has(HIT_ADJUST) => self.nav.adjust -= 1,
                A11yAct::Increment | A11yAct::Decrement => {}
            }
            self.scroll_into_view(key);
        }
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
                        // A key or the pad's A presses as a pointer does, and sounds the same.
                        self.sound(crate::style_file::UiSound::Press);
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
            let before = self.state.focus;
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
                if k != before {
                    // Focus reaching a control by key or pad sounds as the pointer reaching it.
                    self.sound(crate::style_file::UiSound::Hover);
                }
            }
        }
        if moved {
            self.scroll_into_view(self.state.focus);
        }
    }

    /// One extra pad's intents: its d-pad, A and B edges, and its stick
    /// (past half-way once, then after the slow duration every fast one).
    fn pad_intents(&self, g: crate::input::GamepadFrame, stick: &mut (u8, u64)) -> Vec<Nav> {
        let mut out = Vec::new();
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
            *stick = (0, 0);
        } else if stick.0 != dir {
            *stick = (dir, now);
            out.extend(Nav::from_index(dir - 1));
        } else {
            let held = now - stick.1;
            let delay = u64::from(self.ticks_for(self.theme().durations_ms.slow));
            let every = u64::from(self.ticks_for(self.theme().durations_ms.fast));
            if held >= delay && (held - delay).is_multiple_of(every) {
                out.extend(Nav::from_index(dir - 1));
            }
        }
        out
    }

    /// Pads 1 to 3, while the last frame allowed their players
    /// (`set_players`): each moves its own focus among the controls pad 0
    /// can reach (inside the open scope), starting at the one nearest the
    /// surface's centre, and presses the one under it with A. Back and
    /// adjusting (a slider's left and right) are pad 0's.
    pub(crate) fn apply_pads(&mut self, input: &InputFrame) {
        let players = usize::from(self.state.players.clamp(1, 4));
        for k in 1..players {
            let mut stick = self.state.pad_stick[k - 1];
            let intents = self.pad_intents(input.extra_pads[k - 1], &mut stick);
            self.state.pad_stick[k - 1] = stick;
            for nav in intents {
                let focus = self.state.pad_focus[k - 1];
                match nav {
                    Nav::Back => continue,
                    Nav::Activate => {
                        if self.hit(focus).is_some_and(|h| h.has(HIT_NAV)) {
                            self.nav.pad_activate[k - 1] = focus;
                            self.sound(crate::style_file::UiSound::Press);
                        }
                        continue;
                    }
                    _ => {}
                }
                let cands = self.focusables();
                if cands.is_empty() {
                    continue;
                }
                let current = cands.iter().position(|h| h.key == focus);
                let next = match (nav, current) {
                    // A pad's first move lands on the control nearest the
                    // surface's centre (where a shared screen's choices are),
                    // not on the first control of the reading order.
                    (_, None) => {
                        let (cx, cy) = (self.viewport.width / 2.0, self.viewport.height / 2.0);
                        let d = |h: &Hit| {
                            let (x, y) = h.full.center();
                            (x - cx) * (x - cx) + (y - cy) * (y - cy)
                        };
                        cands
                            .iter()
                            .min_by(|a, b| d(a).total_cmp(&d(b)))
                            .map(|h| h.key)
                    }
                    (Nav::Next, Some(i)) => Some(cands[(i + 1) % cands.len()].key),
                    (Nav::Prev, Some(i)) => Some(cands[(i + cands.len() - 1) % cands.len()].key),
                    (dir, Some(i)) => {
                        let others: Vec<(u64, Rect)> = cands
                            .iter()
                            .filter(|h| h.key != cands[i].key)
                            .map(|h| (h.key, h.full))
                            .collect();
                        neighbour(&cands[i].full, &others, dir)
                    }
                };
                if let Some(next) = next.filter(|&n| n != focus) {
                    self.state.pad_focus[k - 1] = next;
                    self.sound(crate::style_file::UiSound::Hover);
                    self.scroll_into_view(next);
                }
            }
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
        let scrolls_x = self
            .state
            .scroll_info
            .get(&h.scroller)
            .is_some_and(|i| i.range.x > 0.0);
        if scrolls_x {
            let off = self.state.scroll_x.entry(h.scroller).or_insert(0.0);
            if h.full.x < view.x {
                *off -= view.x - h.full.x + pad;
            } else if h.full.right() > view.right() {
                *off += h.full.right() - view.right() + pad;
            }
            *off = off.max(0.0);
        }
    }
}
