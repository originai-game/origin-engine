//! Focus navigation: keyboard (arrows, Tab, Shift+Tab, Enter, Space,
//! Escape), gamepad (d-pad, left stick with repeat, A, B) and queued
//! intents move one focus between last frame's focusable controls.
//!
//! - An element's **explicit moves** ([`crate::NavLinks`]) come first.
//! - **Next/Prev** walk the controls in reading (draw) order and wrap.
//! - **Directions** pick the nearest control past the focus that way at
//!   both its edges, as Android's FocusFinder does: one in the focus's row or column (its
//!   beam) first, then by the edge-to-edge distance along the move,
//!   weighted, and the centres' offset across it; with none, focus wraps
//!   to the far end of the same row or column.
//! - Inside a **focus group** ([`crate::FocusGroup`]) a direction picks
//!   among the group's members first; at its edge a wrapping group goes
//!   round inside itself, any other leaves it for the nearest control
//!   outside that way (the group around it first) and stays put with none.
//!   Focus coming into a group that remembers lands on its last member.
//! - A dialog or open menu is a **scope**: while it is the topmost, focus
//!   stays inside it, and it takes Back.
//! - Navigation that lands on a control outside its scroll container's view
//!   scrolls it into view.

use crate::input::{GamepadFrame, InputFrame, Nav, UiKey};
use crate::layout::Rect;
use crate::ui::{HIT_ADJUST, HIT_MULTILINE, HIT_NAV, HIT_TEXT, Hit, Ui};

/// The control nearest `from` in direction `dir` among `cands`, wrapping.
pub(crate) fn neighbour(from: &Rect, cands: &[(u64, Rect)], dir: Nav) -> Option<u64> {
    neighbour_in(from, cands, dir, false).or_else(|| neighbour_in(from, cands, dir, true))
}

/// The nearest control that way (`wrap` false, [`toward`]), or the
/// farthest the other way, in the same row or column first (`wrap` true).
fn neighbour_in(from: &Rect, cands: &[(u64, Rect)], dir: Nav, wrap: bool) -> Option<u64> {
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
    if !wrap {
        return toward(from, cands, dir);
    }
    let mut best: Option<(f32, u64)> = None;
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

/// Where a candidate stands from the focus, for one direction.
struct Toward {
    /// It overlaps the focus across the move (its row or column).
    beam: bool,
    /// The gap from the focus's leading edge to its near edge (0 touching).
    major: f32,
    /// From the focus's leading edge to its far edge.
    far: f32,
    /// Its centre's offset from the focus's across the move.
    minor: f32,
}

/// The nearest control that lies in direction `dir` from `from` at both of
/// its edges, by Android's FocusFinder (the W3C css-nav rule is the same in
/// substance): a control in the focus's beam (overlapping it across the
/// move) beats one outside it (going up or down, unless that one is wholly
/// nearer), then the least `13·major² + minor²` with `major` measured edge
/// to edge along the move and `minor` centre to centre across it. Scored
/// from centres, a tab diagonally below beat the button beside the focus in
/// its own row, and nothing led to that button.
fn toward(from: &Rect, cands: &[(u64, Rect)], dir: Nav) -> Option<u64> {
    let (fx, fy) = from.center();
    let horizontal = matches!(dir, Nav::Left | Nav::Right);
    let at = |r: &Rect| -> Option<Toward> {
        let (cx, cy) = r.center();
        let overlap = |a0: f32, a1: f32, b0: f32, b1: f32| a1.min(b1) - a0.max(b0) > 0.5;
        // That way at both of its edges (FocusFinder's isCandidate): a row
        // around the focus, whose centre lies past it, is no move out of
        // it (it won at no distance, and a link's neighbour in the same
        // row was never reached).
        let (that_way, major, far) = match dir {
            Nav::Down => (
                (from.y < r.y || from.bottom() <= r.y) && from.bottom() < r.bottom(),
                r.y - from.bottom(),
                r.bottom() - from.bottom(),
            ),
            Nav::Up => (
                (from.bottom() > r.bottom() || from.y >= r.bottom()) && from.y > r.y,
                from.y - r.bottom(),
                from.y - r.y,
            ),
            Nav::Right => (
                (from.x < r.x || from.right() <= r.x) && from.right() < r.right(),
                r.x - from.right(),
                r.right() - from.right(),
            ),
            _ => (
                (from.right() > r.right() || from.x >= r.right()) && from.x > r.x,
                from.x - r.right(),
                from.x - r.x,
            ),
        };
        // A control inside the focus (a row's own Remove button, a chat
        // line's link) whose centre lies that way from the focus's: the move
        // steps into it, at no distance. No edge of it lies past the focus,
        // so nothing led to it at all.
        let inside = r.x >= from.x - 0.5
            && r.y >= from.y - 0.5
            && r.right() <= from.right() + 0.5
            && r.bottom() <= from.bottom() + 0.5
            && match dir {
                Nav::Down => cy > fy + 0.5,
                Nav::Up => cy < fy - 0.5,
                Nav::Right => cx > fx + 0.5,
                _ => cx < fx - 0.5,
            };
        if inside {
            return Some(Toward {
                beam: true,
                major: 0.0,
                far: 0.0,
                minor: if horizontal {
                    (cy - fy).abs()
                } else {
                    (cx - fx).abs()
                },
            });
        }
        that_way.then(|| Toward {
            beam: if horizontal {
                overlap(from.y, from.bottom(), r.y, r.bottom())
            } else {
                overlap(from.x, from.right(), r.x, r.right())
            },
            major: major.max(0.0),
            far: far.max(0.0),
            minor: if horizontal {
                (cy - fy).abs()
            } else {
                (cx - fx).abs()
            },
        })
    };
    let weighted = |t: &Toward| 13.0 * t.major * t.major + t.minor * t.minor;
    // Whether `a` beats `b` (FocusFinder's isBetterCandidate and beamBeats).
    let beats = |a: &Toward, b: &Toward| {
        if a.beam != b.beam {
            let (inside, outside) = if a.beam { (a, b) } else { (b, a) };
            if horizontal || inside.major < outside.far {
                return a.beam;
            }
        }
        weighted(a) < weighted(b)
    };
    let mut best: Option<(Toward, u64)> = None;
    for (k, r) in cands {
        if let Some(t) = at(r)
            && best.as_ref().is_none_or(|(b, _)| beats(&t, b))
        {
            best = Some((t, *k));
        }
    }
    best.map(|b| b.1)
}

/// A move's slot in [`crate::NavLinks`] and the UI state's links.
fn link_slot(nav: Nav) -> Option<usize> {
    match nav {
        Nav::Up => Some(0),
        Nav::Down => Some(1),
        Nav::Left => Some(2),
        Nav::Right => Some(3),
        Nav::Next => Some(4),
        Nav::Prev => Some(5),
        _ => None,
    }
}

impl Ui {
    /// Where an explicit move to `target` (a control's or a group's key)
    /// lands among `cands`: the control, or a group's remembered member
    /// (with memory), else its first member on screen.
    fn land(&self, target: u64, cands: &[Hit]) -> Option<u64> {
        if cands.iter().any(|h| h.key == target) {
            return Some(target);
        }
        let g = self.state.groups.get(&target)?;
        if g.spec.memory && g.last != 0 && cands.iter().any(|h| h.key == g.last) {
            return Some(g.last);
        }
        cands
            .iter()
            .find(|h| self.in_group(h.group, target))
            .map(|h| h.key)
    }

    /// A spatial move's landing `k`, from `from`: focus coming into a
    /// group that remembers (the outermost one entered) goes to its last
    /// member instead.
    fn enter_groups(&self, k: u64, from: &Hit, cands: &[Hit]) -> u64 {
        let Some(to) = cands.iter().find(|h| h.key == k) else {
            return k;
        };
        let mut g = to.group;
        let mut pick = k;
        while g != 0 {
            let Some(gs) = self.state.groups.get(&g) else {
                break;
            };
            if self.in_group(from.group, g) {
                break;
            }
            if gs.spec.memory && gs.last != 0 && cands.iter().any(|h| h.key == gs.last) {
                pick = gs.last;
            }
            g = gs.parent;
        }
        pick
    }

    /// The control a move `nav` from `cands[i]` lands on (`None`: it stays):
    /// its explicit move, else within its focus groups, else spatially.
    pub(crate) fn step_focus(&self, cands: &[Hit], i: usize, nav: Nav) -> Option<u64> {
        let from = cands[i];
        if let Some(slot) = link_slot(nav)
            && let Some(links) = self.state.nav_links.get(&from.key)
            && links[slot] != 0
            && let Some(k) = self.land(links[slot], cands)
        {
            return Some(k);
        }
        if matches!(nav, Nav::Next | Nav::Prev) {
            let n = cands.len();
            return Some(if nav == Nav::Next {
                cands[(i + 1) % n].key
            } else {
                cands[(i + n - 1) % n].key
            });
        }
        // A control in a scroll is scored by its part in view, the focus as
        // its candidates: its whole box, past the scroll's edge, reached
        // under a button floating over the scroll's foot (a tall shop tile
        // under the Gallery button) and took every move meant for that
        // button's neighbour. Wholly scrolled out, it keeps its whole box.
        let placed = |h: &Hit| -> Rect {
            if h.scroller != 0 && h.rect.width > 0.5 && h.rect.height > 0.5 {
                h.rect
            } else {
                h.full
            }
        };
        let pick = |level: u64| -> Vec<(u64, Rect)> {
            cands
                .iter()
                .filter(|h| h.key != from.key && self.in_group(h.group, level))
                .map(|h| (h.key, placed(h)))
                .collect()
        };
        let from = Hit {
            full: placed(&from),
            ..from
        };
        if from.group == 0 {
            return neighbour(&from.full, &pick(0), nav)
                .map(|k| self.enter_groups(k, &from, cands));
        }
        // Inside groups: each level outwards, the root last; a wrapping
        // group goes round inside itself, the root does not.
        let mut level = from.group;
        loop {
            let set = pick(level);
            if let Some(k) = neighbour_in(&from.full, &set, nav, false) {
                return Some(self.enter_groups(k, &from, cands));
            }
            if level == 0 {
                return None;
            }
            let g = self.state.groups.get(&level).copied().unwrap_or_default();
            if g.spec.wrap
                && let Some(k) = neighbour_in(&from.full, &set, nav, true)
            {
                return Some(self.enter_groups(k, &from, cands));
            }
            level = g.parent;
        }
    }

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
            // The arrows the focused element claims (Element::arrows).
            let claim = current.map_or(0, |i| (cands[i].flags >> crate::ui::HIT_CLAIM_SHIFT) as u8);
            let claimed = |bit: u8| claim & bit != 0;
            let next = match (nav, current) {
                (Nav::Left | Nav::Right, Some(_)) if adjustable => {
                    self.nav.adjust += if nav == Nav::Right { 1 } else { -1 };
                    None
                }
                (Nav::Left, Some(_)) if claimed(crate::ui::ARROW_LEFT) => {
                    self.nav.adjust -= 1;
                    None
                }
                (Nav::Right, Some(_)) if claimed(crate::ui::ARROW_RIGHT) => {
                    self.nav.adjust += 1;
                    None
                }
                (Nav::Up, Some(_)) if claimed(crate::ui::ARROW_UP) => {
                    self.nav.adjust_y -= 1;
                    None
                }
                (Nav::Down, Some(_)) if claimed(crate::ui::ARROW_DOWN) => {
                    self.nav.adjust_y += 1;
                    None
                }
                (Nav::Next, None) => Some(cands[0].key),
                (Nav::Prev, None) => Some(cands[cands.len() - 1].key),
                (_, None) => Some(cands[0].key),
                (dir, Some(i)) => self.step_focus(&cands, i, dir),
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
                    (dir, Some(i)) => self.step_focus(&cands, i, dir),
                };
                if let Some(next) = next.filter(|&n| n != focus) {
                    self.state.pad_focus[k - 1] = next;
                    self.sound(crate::style_file::UiSound::Hover);
                    self.scroll_into_view(next);
                }
            }
        }
    }

    /// Scrolls the scroll container around the control at `path` (a full
    /// path, as [`Ui::focus_path`] names it; an interactive or focusable
    /// element) so the control is in its view,
    /// by where both were in the last frame: by the least that shows it
    /// whole, or with `center` so it stands in the view's middle (a saga
    /// map opening on the current level). The offset is clamped to the
    /// container's range; it stops a fling. Nothing for a path the last
    /// frame had no control at or that is in no scroll container.
    pub fn scroll_into_view_path(&mut self, path: &str, center: bool) {
        let key = crate::id::key_of_path(path);
        if !center {
            self.scroll_into_view(key);
            return;
        }
        let Some(h) = self.hit(key).copied() else {
            return;
        };
        if h.scroller == 0 {
            return;
        }
        let Some(view) = self.hit(h.scroller).map(|s| s.rect) else {
            return;
        };
        let range = self
            .state
            .scroll_info
            .get(&h.scroller)
            .map_or(og_core::glam::Vec2::ZERO, |i| i.range);
        let at = self.offset(h.scroller);
        let d = og_core::glam::Vec2::new(
            (h.full.x + h.full.width / 2.0) - (view.x + view.width / 2.0),
            (h.full.y + h.full.height / 2.0) - (view.y + view.height / 2.0),
        );
        let to = (at + d).clamp(
            og_core::glam::Vec2::ZERO,
            range.max(og_core::glam::Vec2::ZERO),
        );
        if range.y > 0.0 {
            self.state.scroll.insert(h.scroller, to.y);
        }
        if range.x > 0.0 {
            self.state.scroll_x.insert(h.scroller, to.x);
        }
        if self.state.fling.is_some_and(|(k, _)| k == h.scroller) {
            self.state.fling = None;
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
