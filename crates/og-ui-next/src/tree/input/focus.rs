//! Keyboard and gamepad focus, Tab order, roving tabindex, spatial focus
//! navigation, modal focus traps, actions, and per-frame input timers.

use super::*;

impl Tree {
    /// Whether op element `e` takes focus: shown, enabled, reachable, and
    /// a press listener, an input or a `tabIndex`.
    pub(super) fn focusable(&self, e: u32) -> bool {
        let el = &self.els[e as usize];
        let Some(p) = self.input.per(e) else {
            return false;
        };
        let carrying = self.input.drag.is_some_and(|d| d.live && d.keys);
        let takes = p.tab != NO_TAB
            || p.input
            || el.clickable
            || !p.drag.is_empty()
            || (carrying && !p.drop.is_empty());
        takes
            && !el.dead
            && el.js != NONE
            && self.abs.get(e as usize).is_some_and(|a| a.2)
            && !self.shown.get(e as usize).is_some_and(|s| s.out)
            && !self.disabled(e)
            && self.reachable(e)
    }

    pub(super) fn focusable_up(&self, e: u32) -> u32 {
        let mut e = e;
        while e != NONE {
            if self.els[e as usize].js != NONE && self.focusable(e) {
                return e;
            }
            e = self.els[e as usize].parent;
        }
        NONE
    }

    /// The op element with the keyboard focus, as its arena node (what
    /// the accessibility tree reports).
    pub fn focused(&self) -> Option<Id> {
        (self.input.focus != NONE).then(|| self.els[self.input.focus as usize].node)
    }

    /// The renderer's id of the focused op node.
    pub fn focused_id(&self) -> Option<u32> {
        (self.input.focus != NONE).then(|| self.els[self.input.focus as usize].js)
    }

    /// Moves the keyboard focus to `e` (NONE: nowhere): the old one hears
    /// `blur`, the new one `focus`; `focus:` sits on it, `focus-visible:`
    /// too when the keys moved it, and `focus-within:` on it and around.
    pub(super) fn focus(&mut self, e: u32) {
        let old = self.input.focus;
        let visible = self.input.modality == Modality::Keyboard;
        if old == e {
            if e != NONE {
                self.set_state(e, FOCUS_VISIBLE, visible);
            }
            return;
        }
        if old != NONE {
            if self
                .input
                .per(old)
                .is_some_and(|p| p.input && !p.composing.is_empty())
            {
                self.input.per[old as usize].composing.clear();
                self.show_value(old);
            }
            self.set_state(old, FOCUSED, false);
            self.set_state(old, FOCUS_VISIBLE, false);
            let mut a = old;
            while a != NONE {
                self.set_state(a, FOCUS_WITHIN, false);
                a = self.els[a as usize].parent;
            }
            if !self.els[old as usize].dead {
                self.fire(old, OpEvent::Blur, 0.0, 0.0);
            }
        }
        self.input.focus = e;
        // An `Input`'s caret shows while it has the focus.
        self.paint = true;
        if e != NONE && visible {
            self.reveal(e);
        }
        if e != NONE {
            // Each group around it remembers it.
            let mut g = self.group_of(e);
            while g != NONE {
                self.input.per[g as usize].last = Some(e);
                g = self.group_of(g);
            }
            if let Some(p) = self.input.per.get_mut(e as usize)
                && p.input
            {
                p.caret_at = self.time;
                p.goal_x = None;
            }
            self.set_state(e, FOCUSED, true);
            self.set_state(e, FOCUS_VISIBLE, visible);
            let mut a = e;
            while a != NONE {
                self.set_state(a, FOCUS_WITHIN, true);
                a = self.els[a as usize].parent;
            }
            self.fire(e, OpEvent::Focus, 0.0, 0.0);
            if self.input.drag.is_some_and(|d| d.live && d.keys) {
                let t = self.drop_up(e);
                let (cx, cy) = self.centre(e);
                self.drag_over(t, cx, cy);
            }
        }
    }

    /// Restores focus state bits for checkpoint restore without firing JS
    /// `focus` or `blur` events or mutating focus-memory groups.
    pub(in crate::tree) fn set_focus_silent(&mut self, e: u32, visible: bool) {
        let old = self.input.focus;
        if old != NONE {
            self.set_state(old, FOCUSED, false);
            self.set_state(old, FOCUS_VISIBLE, false);
            let mut a = old;
            while a != NONE {
                self.set_state(a, FOCUS_WITHIN, false);
                a = self.els[a as usize].parent;
            }
        }
        self.input.focus = e;
        if e != NONE {
            self.set_state(e, FOCUSED, true);
            self.set_state(e, FOCUS_VISIBLE, visible);
            let mut a = e;
            while a != NONE {
                self.set_state(a, FOCUS_WITHIN, true);
                a = self.els[a as usize].parent;
            }
        }
        self.paint = true;
    }

    /// The Tab stops in Tab order (Rune's `tab_order`): positive
    /// `tabIndex` first by value, then the others in tree order; inside
    /// the top modal while one is open.
    pub(super) fn tab_stops(&mut self) {
        self.order();
        let mut stops = std::mem::take(&mut self.input.stops);
        stops.clear();
        let tab = |t: &Tree, e: u32| t.input.per(e).map_or(NO_TAB, |p| p.tab);
        // Positive `tabIndex` first, sorted (equal ones in tree order: a
        // stable sort, which allocates nothing for the few there are),
        // then the rest in tree order.
        for &e in &self.order {
            if tab(self, e) > 0 && self.focusable(e) {
                stops.push(e);
            }
        }
        stops.sort_by_key(|&e| tab(self, e));
        for &e in &self.order {
            let t = tab(self, e);
            if (t == NO_TAB || t == 0) && self.focusable(e) {
                stops.push(e);
            }
        }
        self.input.stops = stops;
    }

    /// Tab (Shift+Tab with `back`) moves the focus to the next stop,
    /// wrapping (NSH's `tab_stop`), and shows it.
    fn tab(&mut self, back: bool) {
        self.tab_stops();
        self.rove();
        let stops = &self.input.stops;
        let current = stops.iter().position(|&e| e == self.input.focus);
        let next = nsh::tab_stop(current, stops.len(), back, true);
        let e = next.map_or(NONE, |i| stops[i]);
        self.input.modality = Modality::Keyboard;
        self.focus(e);
    }

    /// Keeps one stop of each roving group (`focusRoving`) among the Tab
    /// stops, as NSH base's `Roving` makes only one item focusable: the
    /// member with the focus, else its selected, checked or pressed member
    /// (a tab list's, a radio group's), else the one it last had (a
    /// toolbar's), else its first. The arrows still move
    /// among all of them.
    fn rove(&mut self) {
        let mut stops = std::mem::take(&mut self.input.stops);
        let mut keep: Vec<(u32, u32)> = Vec::new();
        let outer = |t: &Tree, e: u32| {
            let (mut g, mut top) = (t.group_of(e), NONE);
            while g != NONE {
                if t.input.per(g).is_some_and(|p| p.roving) {
                    top = g;
                }
                g = t.group_of(g);
            }
            top
        };
        for &e in &stops {
            let g = outer(self, e);
            if g == NONE || keep.iter().any(|k| k.0 == g) {
                continue;
            }
            let members = || stops.iter().copied().filter(|&m| outer(self, m) == g);
            let current = crate::classes::state::SELECTED
                | crate::classes::state::CHECKED
                | crate::classes::state::ARIA_PRESSED;
            let focus = self.input.focus;
            let last = self.input.per(g).and_then(|p| p.last);
            let pick = members()
                .find(|&m| m == focus)
                .or_else(|| {
                    members().find(|&m| self.input.per(m).is_some_and(|p| p.states & current != 0))
                })
                .or_else(|| members().find(|&m| Some(m) == last))
                .unwrap_or(e);
            keep.push((g, pick));
        }
        stops.retain(|&e| {
            let g = outer(self, e);
            g == NONE || keep.iter().any(|k| k.1 == e)
        });
        self.input.stops = stops;
    }

    /// The element's box on screen, as og-ui's navigation reads it.
    pub(super) fn rect(&self, e: u32) -> og_ui_model::Rect {
        let (x, y, _, _) = self.abs[e as usize];
        let node = self.els[e as usize].node;
        let b = self
            .motion
            .box_of(node)
            .unwrap_or_else(|| self.arena.box_of(node));
        let [x0, y0, x1, y1] = self.screen_bounds(e, x, y, b.w, b.h);
        og_ui_model::Rect {
            x: x0,
            y: y0,
            width: x1 - x0,
            height: y1 - y0,
        }
    }

    /// Whether element `e` is a horizontal range control (`slider` / `spinbutton`)
    /// that consumes Left/Right for value adjustment while allowing Up/Down
    /// focus navigation.
    fn is_range_control(&self, e: u32) -> bool {
        e != NONE
            && self.els[e as usize].listens & OpEvent::Key.bit() != 0
            && !self.disabled(e)
            && matches!(
                self.sem.get(self.els[e as usize].node).role,
                Some(Some(accesskit::Role::Slider | accesskit::Role::SpinButton))
            )
    }

    /// A d-pad, stick or arrow move (`Up`, `Down`, `Left`, `Right`): the
    /// focus goes to the nearest Tab stop that way, wrapping to the far
    /// end of its row or column (og-ui's FocusFinder rule), and shows.
    /// When a `slider` or `spinbutton` is focused, Left/Right adjusts its
    /// value via `key` so gamepad players can operate sliders and steppers.
    /// With nothing focused it lands on the first stop. Whether a stop
    /// took it.
    pub fn nav(&mut self, dir: og_input::frame::Nav) -> bool {
        self.sync_modals();
        self.tab_stops();
        let Some(&first) = self.input.stops.first() else {
            return false;
        };
        self.input.modality = Modality::Keyboard;
        let f = self.input.focus;
        if f == NONE || !self.input.stops.contains(&f) {
            self.focus(first);
            return true;
        }
        if matches!(
            dir,
            og_input::frame::Nav::Left | og_input::frame::Nav::Right
        ) && self.is_range_control(f)
        {
            self.set_state(f, FOCUS_VISIBLE, true);
            let code = if dir == og_input::frame::Nav::Left {
                "ArrowLeft"
            } else {
                "ArrowRight"
            };
            let mut ev = Fired::at(self.els[f as usize].js, OpEvent::Key, 0.0, 0.0);
            ev.key = code.into();
            self.events.push(ev);
            return true;
        }
        let to = self.step(f, dir);
        match to {
            Some(e) => self.focus(e),
            None => self.set_state(f, FOCUS_VISIBLE, true),
        }
        true
    }

    /// The focus group `e` is in (its nearest `focusGroup` ancestor; NONE
    /// none).
    fn group_of(&self, e: u32) -> u32 {
        let mut g = self.els[e as usize].parent;
        while g != NONE {
            if self.input.per(g).is_some_and(|p| p.group.is_some()) {
                return g;
            }
            g = self.els[g as usize].parent;
        }
        NONE
    }

    /// Where a move `dir` from stop `f` lands (og-ui's focus groups): in a
    /// group, among its members first; at its edge a wrapping group goes
    /// round inside itself, any other leaves for the group around it (the
    /// whole surface last, which does not wrap). Outside any group the
    /// move wraps to the far end of its row or column. Coming into a group
    /// that remembers lands on its last member.
    fn step(&mut self, f: u32, dir: og_input::frame::Nav) -> Option<u32> {
        let from = self.rect(f);
        let (fx, fy) = from.center();
        // A move that went back the other way is a wrap.
        let ahead = |r: &og_ui_model::Rect| {
            let (cx, cy) = r.center();
            match dir {
                og_input::frame::Nav::Down => cy > fy,
                og_input::frame::Nav::Up => cy < fy,
                og_input::frame::Nav::Right => cx > fx,
                _ => cx < fx,
            }
        };
        let mut level = self.group_of(f);
        let grouped = level != NONE;
        let mut cands = std::mem::take(&mut self.input.cands);
        let to = loop {
            cands.clear();
            for &e in &self.input.stops {
                if e != f && (level == NONE || self.within(e, level)) {
                    cands.push((u64::from(e), self.rect(e)));
                }
            }
            let hit = crate::nav::neighbour(&from, &cands, dir).map(|k| k as u32);
            let rect = |e: u32| cands.iter().find(|c| c.0 == u64::from(e)).map(|c| c.1);
            match hit {
                Some(e) if rect(e).is_some_and(|r| ahead(&r)) => break Some(e),
                Some(e) if level == NONE && !grouped => break Some(e),
                Some(e)
                    if level != NONE
                        && self.input.per(level).is_some_and(|p| p.group == Some(true)) =>
                {
                    break Some(e);
                }
                _ if level == NONE => break None,
                _ => level = self.group_of(level),
            }
        };
        self.input.cands = cands;
        to.map(|k| self.enter_groups(k, f))
    }

    /// A move's landing `k` from `f`: coming into a group that remembers
    /// (the outermost one entered) goes to its last member instead.
    fn enter_groups(&self, k: u32, f: u32) -> u32 {
        let mut pick = k;
        let mut g = self.group_of(k);
        while g != NONE && !self.within(f, g) {
            if let Some(p) = self.input.per(g)
                && p.memory
                && let Some(last) = p.last
                && self.input.stops.contains(&last)
                && self.within(last, g)
            {
                pick = last;
            }
            g = self.group_of(g);
        }
        pick
    }

    /// A key went down (its `KeyboardEvent.code`): Tab moves the focus,
    /// Enter or Space presses the focused control (NSH's
    /// `presses_focused_control`), and the focused element, or the
    /// nearest around it that listens, hears `key` with the code. Whether
    /// an op node took it.
    pub fn key(&mut self, code: &str, shift: bool, ctrl: bool, alt: bool, meta: bool) -> bool {
        self.sync_modals();
        if code == "Tab" && !(ctrl || alt || meta) {
            self.tab(shift);
            return true;
        }
        if ((code == "F10" && shift) || code == "ContextMenu")
            && !(ctrl || alt || meta)
            && self.context_menu_key()
        {
            return true;
        }
        // A selected text copies, or selects all of itself.
        if self.editing().is_none() && self.select_key(code, (ctrl || meta) && !alt) {
            return true;
        }
        // The focused `Input` edits with its keys (arrows across its text
        // included); the game's shortcuts with Control, Alt or Command
        // pass on.
        if let Some(e) = self.editing() {
            let cmd = (ctrl || meta) && !alt;
            if cmd && code == "KeyA" {
                let p = &mut self.input.per[e as usize];
                p.anchor = 0;
                p.caret = p.value.len();
                self.paint = true;
                return true;
            }
            if cmd && matches!(code, "KeyC" | "KeyX") {
                let p = &mut self.input.per[e as usize];
                let (start, end) = p.selection_range();
                if start < end {
                    if !p.secret {
                        self.input.copied = Some(p.value[start..end].to_owned());
                    }
                    if code == "KeyX" && !p.read_only {
                        p.composing.clear();
                        self.remember(e, Edit::Other);
                        let p = &mut self.input.per[e as usize];
                        p.value.replace_range(start..end, "");
                        p.caret = start;
                        p.anchor = start;
                        self.edited(e);
                    }
                }
                return true;
            }
            // Undo, and redo with Shift (or Ctrl+Y); the text's own.
            if cmd && (code == "KeyZ" || code == "KeyY" && ctrl) {
                self.undo(e, shift || code == "KeyY");
                return true;
            }
            if !(ctrl || alt || meta)
                && matches!(code, "ArrowUp" | "ArrowDown")
                && self.input.per[e as usize].multiline
            {
                self.line_key(e, code == "ArrowDown", shift);
                return true;
            }
            if !(ctrl || alt || meta)
                && matches!(code, "Enter" | "NumpadEnter")
                && self.input.per[e as usize].multiline
            {
                self.type_text("\n");
                return true;
            }
            let word = (ctrl || alt) && !meta;
            if (!ctrl && !alt && !meta || word) && self.edit_key(e, code, shift, word) {
                return true;
            }
        }
        let dir = match code {
            "ArrowUp" => Some(og_input::frame::Nav::Up),
            "ArrowDown" => Some(og_input::frame::Nav::Down),
            "ArrowLeft" => Some(og_input::frame::Nav::Left),
            "ArrowRight" => Some(og_input::frame::Nav::Right),
            _ => None,
        };
        // An arrow moves the focus, unless the focused node takes keys
        // itself (a slider, a list): then it hears the arrow instead. For
        // horizontal range controls (`slider` / `spinbutton`), Up/Down still
        // move focus between rows while Left/Right adjust the value.
        let own = self.input.focus != NONE
            && self.els[self.input.focus as usize].listens & OpEvent::Key.bit() != 0
            && !(matches!(
                dir,
                Some(og_input::frame::Nav::Up | og_input::frame::Nav::Down)
            ) && self.is_range_control(self.input.focus));
        if let Some(dir) = dir
            && !own
            && !(shift || ctrl || alt || meta)
        {
            return self.nav(dir);
        }
        let f = self.input.focus;
        if f == NONE {
            return false;
        }
        let mods = nsh_input::keymap::Modifiers {
            shift,
            ctrl,
            alt,
            cmd: meta,
        };
        // A carried node: Escape puts it back, Enter or Space puts it down
        // (or picks the focused one up).
        let plain = !(shift || ctrl || alt || meta);
        if plain && code == "Escape" && self.input.drag.is_some_and(|d| d.live) {
            self.put_down(NONE, 0.0, 0.0);
            return true;
        }
        if plain && matches!(code, "Enter" | "NumpadEnter" | "Space") && self.carry(f) {
            self.input.keyed = true;
            return true;
        }
        let press = match code {
            "Enter" | "NumpadEnter" => Some(nsh_input::keymap::Key::Enter),
            "Space" => Some(nsh_input::keymap::Key::Char(' ')),
            _ => None,
        };
        if let Some(k) = press
            && nsh::presses_focused_control(k, mods)
            && self.els[f as usize].clickable
            && !self.input.per(f).is_some_and(|p| p.input)
        {
            if self.input.modality != Modality::Keyboard {
                self.input.modality = Modality::Keyboard;
                self.set_state(f, FOCUS_VISIBLE, true);
            }
            self.press_or_hold(f, true);
            self.input.keyed = true;
            return true;
        }
        let l = self.up(f, |el| el.listens & OpEvent::Key.bit() != 0);
        if l == NONE {
            return false;
        }
        let mut ev = Fired::at(self.els[l as usize].js, OpEvent::Key, 0.0, 0.0);
        ev.key = code.into();
        self.events.push(ev);
        true
    }

    /// An input action fired (`confirm`, `back`, a game's own): the
    /// focused element, or the nearest around it whose `actions` list it,
    /// hears `action` with its name; else, inside the top modal, the
    /// modal or what in it lists it. `confirm` on a focused control that
    /// no one around lists presses it, unless its key did this frame.
    /// Whether an op node took it.
    pub fn action(&mut self, name: &str) -> bool {
        self.sync_modals();
        // `confirm` picks the focused draggable up or puts a carried node
        // down; `back` puts it back.
        let f = self.input.focus;
        if name == "confirm" && f != NONE && !self.input.keyed && self.carry(f) {
            return true;
        }
        if name == "back" && self.input.drag.is_some_and(|d| d.live) {
            self.put_down(NONE, 0.0, 0.0);
            return true;
        }
        let lists = |p: Option<&Per>| p.is_some_and(|p| p.actions.split(' ').any(|a| a == name));
        let from = match self.input.focus {
            NONE => self.input.modals.last().map_or(NONE, |m| m.0),
            f => f,
        };
        let mut e = from;
        while e != NONE {
            let el = &self.els[e as usize];
            if !el.dead && el.js != NONE && lists(self.input.per(e)) {
                let mut ev = Fired::at(el.js, OpEvent::Action, 0.0, 0.0);
                ev.key = name.into();
                self.events.push(ev);
                if matches!(name, "confirm" | "back") {
                    self.sound(e, name);
                }
                return true;
            }
            e = el.parent;
        }
        let f = self.input.focus;
        if name == "confirm" && f != NONE && self.els[f as usize].clickable {
            if !self.input.keyed {
                self.input.modality = Modality::Keyboard;
                self.set_state(f, FOCUS_VISIBLE, true);
                self.input.confirming = self.input.per(f).is_none_or(|p| p.hold <= 0.0);
                self.press_or_hold(f, true);
            }
            return true;
        }
        false
    }

    /// The frame's input ended: what a key consumed is forgotten.
    ///
    /// `confirm`: the `confirm` action is down (a hold it began goes on
    /// while it is). Here a still finger's long press comes due, a hold
    /// moves on, completes or lets go, and a fling coasts.
    pub fn input_done(&mut self, confirm: bool) {
        self.input.keyed = false;
        let now = self.time;
        // An `autofocus` node takes the focus once it shows.
        let mut k = 0;
        while k < self.input.autofocus.len() {
            let e = self.input.autofocus[k];
            if self.els[e as usize].dead {
                self.input.autofocus.swap_remove(k);
            } else if self.focusable(e) {
                self.input.autofocus.swap_remove(k);
                if self.input.pad {
                    self.input.modality = Modality::Keyboard;
                }
                self.focus(e);
            } else {
                k += 1;
            }
        }
        // A long press: the finger stayed within the slop.
        if let Some(f) = &self.input.finger
            && f.mode == Touch::Pending
            && now >= f.due
        {
            let (x, y) = f.origin;
            self.chain_at(x, y);
            let l = self
                .input
                .chain
                .iter()
                .copied()
                .find(|&e| self.els[e as usize].listens & OpEvent::LongPress.bit() != 0);
            if let Some(l) = l {
                self.fire(l, OpEvent::LongPress, x, y);
                self.cancel_press();
                if let Some(f) = &mut self.input.finger {
                    f.mode = Touch::Long;
                }
            }
        }
        // A hold: held by its press while inside it, or by `confirm` on
        // the focus.
        if let Some(h) = &self.input.holding {
            let (e, start, keys, done) = (h.e, h.start, h.keys, h.done);
            let held = if keys {
                confirm && self.input.focus == e
            } else {
                self.input.held.as_ref().is_some_and(|p| p.e == e)
                    && self.input.per(e).is_some_and(|p| p.states & PRESSED != 0)
            };
            let dur = self.input.per(e).map_or(0.0, |p| p.hold);
            if !held || self.els[e as usize].dead {
                self.input.holding = None;
                if !done && !self.els[e as usize].dead {
                    self.fire_value(e, OpEvent::Hold, "0");
                }
            } else if !done {
                let t = ((now - start) as f32 / dur.max(f32::EPSILON)).min(1.0);
                if t >= 1.0 {
                    self.fire_value(e, OpEvent::Hold, "1");
                    self.click(e, 0.0, 0.0, false);
                    if let Some(h) = &mut self.input.holding {
                        h.done = true;
                    }
                } else {
                    let mut buf = [0u8; 8];
                    let text = progress(t, &mut buf);
                    self.fire_value(e, OpEvent::Hold, text);
                }
            }
        }
        // A fling coasts (Rune's exact exponential decay) until it stops
        // or meets its scroller's end.
        if let Some((e, h, v, last)) = self.input.fling {
            let dt = (now - last) as f32;
            if dt > 0.0 {
                let decay = og_core::math::exp(-SCROLL_DECAY * dt);
                let by = v * (1.0 - decay) / SCROLL_DECAY;
                let moved = self.scroller(e) && self.scroll_to(e, h, self.scroll_at(e, h) + by);
                let v = v * decay;
                self.input.fling = (moved && v.abs() > 1.0).then_some((e, h, v, now));
            }
        }
    }

    /// Whether a hold, a fling or a finger's long press is under way:
    /// the host runs [`Tree::input_done`] each frame while it is.
    pub fn input_busy(&self) -> bool {
        self.input.holding.is_some() || self.input.fling.is_some() || self.input.finger.is_some()
    }

    /// The modals that opened and closed since the last input (Rune's
    /// `sync_modals`): a closed top hands focus back to what had it
    /// before it opened; one that opened takes the focus to its first
    /// Tab stop, else itself, and drops the press the pointer held.
    pub(in crate::tree) fn sync_modals(&mut self) {
        let live = |t: &Tree, m: u32| {
            let el = &t.els[m as usize];
            !el.dead
                && t.abs.get(m as usize).is_some_and(|a| a.2)
                && !t.shown.get(m as usize).is_some_and(|s| s.out)
                && t.input.per(m).is_some_and(|p| p.modal)
        };
        while let Some(&(m, restore)) = self.input.modals.last() {
            if live(self, m) {
                break;
            }
            self.input.modals.pop();
            let inside = self.input.focus != NONE && self.within(self.input.focus, m);
            if inside || self.input.focus == NONE {
                let back = if restore != NONE && self.focusable(restore) {
                    restore
                } else if self.input.modality == Modality::Keyboard {
                    self.tab_stops();
                    self.input.stops.first().copied().unwrap_or(NONE)
                } else {
                    NONE
                };
                self.focus(back);
            }
        }
        // A focus that went (released, hidden, behind a modal) is gone.
        if self.input.focus != NONE && !self.focusable(self.input.focus) {
            self.focus(NONE);
        }
        // Inner modals left under the top one's close are dropped too; pass
        // their return-focus target upward so closing the top modal still
        // restores focus to what preceded the dropped inner modal.
        let mut k = 0;
        while k < self.input.modals.len() {
            if live(self, self.input.modals[k].0) {
                k += 1;
            } else {
                let (dead_m, dead_restore) = self.input.modals.remove(k);
                if let Some(&(_, next_restore)) = self.input.modals.get(k)
                    && (next_restore == NONE || self.within(next_restore, dead_m))
                {
                    self.input.modals[k].1 = dead_restore;
                }
            }
        }
        for i in 0..self.input.modal_els.len() {
            let m = self.input.modal_els[i];
            if !live(self, m) || self.input.modals.iter().any(|l| l.0 == m) {
                continue;
            }
            self.input.modals.push((m, self.input.focus));
            if let Some(h) = self.input.held.take() {
                self.set_state(h.e, PRESSED, false);
            }
            self.input.capture = NONE;
            self.tab_stops();
            let first = self.input.stops.first().copied().unwrap_or(m);
            if self.input.pad {
                self.input.modality = Modality::Keyboard;
            }
            // An `autofocus` node in it waits to take it instead.
            let wants = self.input.autofocus.iter().any(|&a| self.within(a, m));
            if !wants {
                self.focus(first);
            }
        }
    }
}
