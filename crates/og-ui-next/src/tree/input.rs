//! Input on op nodes: hover, press, wheel, keyboard focus, the modal trap
//! and input actions, each answered by events to the renderer by id
//! ([`Fired`]) and by the interaction states their classes read
//! (`hover:`, `active:`, `focus:`, `focus-visible:`, `focus-within:`,
//! `disabled:`).
//!
//! What decides is ported: NSH's `input.rs` (vendored as `nsh_input`:
//! the press retention, hover's enter and leave order, the Tab stop, the
//! keys that press the focused control) and Rune's input machines as
//! logic (`focus_machine.rs`'s modality for `focus-visible`,
//! `modal_focus.rs`'s trap stack with focus return, `capture.rs`'s click
//! slop and double press, `focus.rs`'s Tab order). Hit testing is the
//! tree's own: boxes in paint order, clipped, the last match wins.

use super::ops::{Event as OpEvent, Fired};
use super::*;
use crate::classes::state::{FOCUS_VISIBLE, FOCUS_WITHIN, FOCUSED, HOVERED, PRESSED};
use nsh_input::input as nsh;

/// A press that travelled further than this (px) is no click of a double
/// press (Rune's `CLICK_SLOP`).
const CLICK_SLOP: f32 = 4.0;
/// Two presses this close (s) on one element are a double press (Rune's
/// `double_click_interval`).
const DOUBLE_PRESS: f64 = 0.5;
/// Whether two points are within [`CLICK_SLOP`] of each other.
fn near(a: (f32, f32), b: (f32, f32)) -> bool {
    let (dx, dy) = (a.0 - b.0, a.1 - b.1);
    dx * dx + dy * dy <= CLICK_SLOP * CLICK_SLOP
}

/// No `tabIndex` prop: a Tab stop when it takes presses or is an input.
pub(super) const NO_TAB: i32 = i32::MIN;

/// What an op element brings to input, by element index.
#[derive(Clone, Default)]
pub(super) struct Per {
    /// Its `tabIndex` ([`NO_TAB`] when unset): positive goes first, 0 in
    /// tree order, negative is focused only by a press.
    pub tab: i32,
    /// An `input` tag: a Tab stop without a listener.
    pub input: bool,
    /// `modal`: while it shows, focus and the pointer stay inside.
    pub modal: bool,
    /// The input actions it answers (`onAction`), space separated.
    pub actions: Box<str>,
    /// The action whose binding its text shows (`<Glyph action>`).
    pub glyph: Box<str>,
    /// Its interaction states ([`crate::classes::state`] bits).
    pub states: u16,
}

/// The press the pointer holds on an op element.
struct Held {
    e: u32,
    at: (f32, f32),
    /// It went further than the click slop.
    moved: bool,
    /// It is the second of a double press.
    second: bool,
}

/// Whether the keys moved focus last: then the focused element shows
/// `focus-visible` (Rune's `Modality`).
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Modality {
    #[default]
    Pointer,
    Keyboard,
}

/// The input state of a tree's op nodes.
#[derive(Default)]
pub struct Input {
    pub(super) per: Vec<Per>,
    /// The elements under the pointer, innermost first, and this move's.
    hover: Vec<u32>,
    next: Vec<u32>,
    held: Option<Held>,
    /// The last click: its element, time and place, for a double press.
    first: Option<(u32, f64, (f32, f32))>,
    focus: u32,
    modality: Modality,
    /// The open modals, innermost last, each with the focus it returns.
    modals: Vec<(u32, u32)>,
    /// The elements with `modal`.
    pub(super) modal_els: Vec<u32>,
    /// Tab stops, made on a Tab or a move, and a move's candidates.
    stops: Vec<u32>,
    cands: Vec<(u64, og_ui::Rect)>,
    /// An Enter or Space pressed the focused control this frame: the
    /// `confirm` it is bound to does not press it again.
    keyed: bool,
    /// A `<Glyph>` came or changed: the host gives the glyphs again.
    pub(super) glyphs_wanted: bool,
}

impl Input {
    pub(super) fn new() -> Input {
        Input {
            focus: NONE,
            ..Input::default()
        }
    }

    pub(super) fn per(&self, e: u32) -> Option<&Per> {
        self.per.get(e as usize)
    }
}

impl Tree {
    /// The op element `e` is in that `want` accepts, itself first.
    fn up(&self, mut e: u32, want: impl Fn(&El) -> bool) -> u32 {
        while e != NONE {
            let el = &self.els[e as usize];
            if !el.dead && el.js != NONE && want(el) {
                return e;
            }
            e = el.parent;
        }
        NONE
    }

    /// Whether `e` is `m` or inside it.
    fn within(&self, mut e: u32, m: u32) -> bool {
        while e != NONE {
            if e == m {
                return true;
            }
            e = self.els[e as usize].parent;
        }
        false
    }

    /// Whether the pointer and the keys may reach `e`: inside the top
    /// modal while one is open.
    pub(super) fn reachable(&self, e: u32) -> bool {
        self.input
            .modals
            .last()
            .is_none_or(|&(m, _)| self.within(e, m))
    }

    fn disabled(&self, e: u32) -> bool {
        let el = &self.els[e as usize];
        el.disabled != NONE && og_reactive::bind::truthy(&queue::get(el.disabled))
    }

    fn fire(&mut self, e: u32, event: OpEvent, x: f32, y: f32) {
        let el = &self.els[e as usize];
        if el.listens & event.bit() != 0 {
            self.events.push(Fired::at(el.js, event, x, y));
        }
    }

    /// Sets (or clears) interaction state `bit` on op element `e`, and
    /// restyles it when its classes read that state.
    pub(super) fn set_state(&mut self, e: u32, bit: u16, on: bool) {
        let Some(p) = self.input.per.get_mut(e as usize) else {
            return;
        };
        let states = if on { p.states | bit } else { p.states & !bit };
        if states == p.states {
            return;
        }
        p.states = states;
        if self.classes.tracked(self.els[e as usize].class) & bit != 0 {
            self.restyle(e);
        }
    }

    /// The pointer moved to (x, y): the op elements it left lose `hover:`
    /// from the inside out, those it came over gain it from the outside
    /// in (NSH's `hover_changes`), and the innermost that listens hears
    /// the move. A held press is inside its element while the pointer is
    /// within NSH's press retention around it.
    fn hover_to(&mut self, x: f32, y: f32) {
        let top = self.hit_where(x, y, |el| el.js != NONE);
        let mut next = std::mem::take(&mut self.input.next);
        next.clear();
        let mut e = top;
        while e != NONE {
            if self.els[e as usize].js != NONE {
                next.push(e);
            }
            e = self.els[e as usize].parent;
        }
        let hover = std::mem::take(&mut self.input.hover);
        // Both lists are out of `self`, so the states can restyle.
        let (left, came) = nsh::hover_changes(&hover, &next);
        for &e in left {
            self.set_state(e, HOVERED, false);
        }
        for &e in came {
            self.set_state(e, HOVERED, true);
        }
        self.input.hover = next;
        self.input.next = hover;
        if let Some(h) = &mut self.input.held {
            h.moved |= !near(h.at, (x, y));
            let e = h.e;
            let inside = self.inside(e, x, y);
            self.set_state(e, PRESSED, inside);
        }
    }

    /// Whether (x, y) is on op element `e`'s box or within NSH's press
    /// retention around it.
    fn inside(&self, e: u32, x: f32, y: f32) -> bool {
        let Some(&(bx, by, shown, _)) = self.abs.get(e as usize) else {
            return false;
        };
        let b = self.arena.box_of(self.els[e as usize].node);
        let local = nsh_input::Point {
            x: nsh_input::Points(x - bx),
            y: nsh_input::Points(y - by),
        };
        let size = nsh_input::Size {
            width: nsh_input::Points(b.w),
            height: nsh_input::Points(b.h),
        };
        shown && nsh::released_inside(local, size, nsh_input::EdgeInsets::default())
    }

    /// The pointer on op nodes this frame, at (x, y): it hovers, a press
    /// takes the innermost element that answers one (and focuses the
    /// nearest focusable around it), and a release within the press's
    /// retention clicks it: `press` and `tap`, with `value` "2" for the
    /// second of a double press. Whether a press went to an op node.
    pub(super) fn pointer_ops(
        &mut self,
        x: f32,
        y: f32,
        moved: bool,
        pressed: bool,
        released: bool,
    ) -> bool {
        self.sync_modals();
        self.hover_to(x, y);
        let under = self.input.hover.first().copied().unwrap_or(NONE);
        if moved && under != NONE {
            let l = self.up(under, |el| el.listens & OpEvent::PointerMove.bit() != 0);
            if l != NONE {
                self.fire(l, OpEvent::PointerMove, x, y);
            }
        }
        let mut took = false;
        if pressed {
            if under != NONE {
                let l = self.up(under, |el| el.listens & OpEvent::PointerDown.bit() != 0);
                if l != NONE {
                    self.fire(l, OpEvent::PointerDown, x, y);
                }
            }
            let e = self.hit(x, y);
            if e != NONE && self.els[e as usize].js != NONE {
                took = true;
                let second = self.input.first.is_some_and(|(f, at, p)| {
                    f == e && self.time - at <= DOUBLE_PRESS && near(p, (x, y))
                });
                self.input.held = Some(Held {
                    e,
                    at: (x, y),
                    moved: false,
                    second,
                });
                self.set_state(e, PRESSED, true);
            }
            // A press focuses what it lands in, or blurs (Rune: a press
            // on content that takes no focus).
            if under != NONE || e != NONE {
                let from = if e != NONE { e } else { under };
                let f = self.focusable_up(from);
                self.input.modality = Modality::Pointer;
                self.focus(f);
            }
        }
        if released {
            if under != NONE {
                let l = self.up(under, |el| el.listens & OpEvent::PointerUp.bit() != 0);
                if l != NONE {
                    self.fire(l, OpEvent::PointerUp, x, y);
                }
            }
            if let Some(h) = self.input.held.take() {
                took = true;
                self.set_state(h.e, PRESSED, false);
                let el = &self.els[h.e as usize];
                if !el.dead && el.js != NONE && self.inside(h.e, x, y) && !self.disabled(h.e) {
                    self.click(h.e, x, y, h.second);
                    self.input.first = (!h.moved && !h.second).then_some((h.e, self.time, h.at));
                } else {
                    self.input.first = None;
                }
            }
        }
        took
    }

    fn click(&mut self, e: u32, x: f32, y: f32, second: bool) {
        let js = self.els[e as usize].js;
        let listens = self.els[e as usize].listens;
        for ev in [OpEvent::Press, OpEvent::Tap] {
            if listens & ev.bit() != 0 {
                let mut f = Fired::at(js, ev, x, y);
                if second {
                    f.value = "2".into();
                }
                self.events.push(f);
            }
        }
    }

    /// A wheel by (dx, dy) at (x, y): the innermost op element under it
    /// that listens hears it, with the deltas in `x` and `y`.
    pub fn wheel(&mut self, x: f32, y: f32, dx: f32, dy: f32) {
        if dx == 0.0 && dy == 0.0 {
            return;
        }
        let under = self.hit_where(x, y, |el| el.js != NONE);
        let l = self.up(under, |el| el.listens & OpEvent::Wheel.bit() != 0);
        if l != NONE {
            self.fire(l, OpEvent::Wheel, dx, dy);
        }
    }

    /// Whether op element `e` takes focus: shown, enabled, reachable, and
    /// a press listener, an input or a `tabIndex`.
    fn focusable(&self, e: u32) -> bool {
        let el = &self.els[e as usize];
        let Some(p) = self.input.per(e) else {
            return false;
        };
        let takes = p.tab != NO_TAB || p.input || el.clickable;
        takes
            && !el.dead
            && el.js != NONE
            && self.abs.get(e as usize).is_some_and(|a| a.2)
            && !self.shown.get(e as usize).is_some_and(|s| s.out)
            && !self.disabled(e)
            && self.reachable(e)
    }

    fn focusable_up(&self, e: u32) -> u32 {
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
    fn focus(&mut self, e: u32) {
        let old = self.input.focus;
        let visible = self.input.modality == Modality::Keyboard;
        if old == e {
            if e != NONE {
                self.set_state(e, FOCUS_VISIBLE, visible);
            }
            return;
        }
        if old != NONE {
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
        if e != NONE {
            self.set_state(e, FOCUSED, true);
            self.set_state(e, FOCUS_VISIBLE, visible);
            let mut a = e;
            while a != NONE {
                self.set_state(a, FOCUS_WITHIN, true);
                a = self.els[a as usize].parent;
            }
            self.fire(e, OpEvent::Focus, 0.0, 0.0);
        }
    }

    /// The Tab stops in Tab order (Rune's `tab_order`): positive
    /// `tabIndex` first by value, then the others in tree order; inside
    /// the top modal while one is open.
    fn tab_stops(&mut self) {
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
        let stops = &self.input.stops;
        let current = stops.iter().position(|&e| e == self.input.focus);
        let next = nsh::tab_stop(current, stops.len(), back, true);
        let e = next.map_or(NONE, |i| stops[i]);
        self.input.modality = Modality::Keyboard;
        self.focus(e);
    }

    /// The element's box on screen, as og-ui's navigation reads it.
    fn rect(&self, e: u32) -> og_ui::Rect {
        let (x, y, _, _) = self.abs[e as usize];
        let b = self.arena.box_of(self.els[e as usize].node);
        og_ui::Rect {
            x,
            y,
            width: b.w,
            height: b.h,
        }
    }

    /// A d-pad, stick or arrow move (`Up`, `Down`, `Left`, `Right`): the
    /// focus goes to the nearest Tab stop that way, wrapping to the far
    /// end of its row or column (og-ui's FocusFinder rule), and shows.
    /// With nothing focused it lands on the first stop. Whether a stop
    /// took it.
    pub fn nav(&mut self, dir: og_ui::Nav) -> bool {
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
        let mut cands = std::mem::take(&mut self.input.cands);
        cands.clear();
        for &e in &self.input.stops {
            if e != f {
                cands.push((u64::from(e), self.rect(e)));
            }
        }
        let to = og_ui::nav_neighbour(&self.rect(f), &cands, dir);
        self.input.cands = cands;
        match to {
            Some(e) => self.focus(e as u32),
            None => self.set_state(f, FOCUS_VISIBLE, true),
        }
        true
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
        let dir = match code {
            "ArrowUp" => Some(og_ui::Nav::Up),
            "ArrowDown" => Some(og_ui::Nav::Down),
            "ArrowLeft" => Some(og_ui::Nav::Left),
            "ArrowRight" => Some(og_ui::Nav::Right),
            _ => None,
        };
        // An arrow moves the focus, unless the focused node takes keys
        // itself (a slider, a list): then it hears the arrow instead.
        let own = self.input.focus != NONE
            && self.els[self.input.focus as usize].listens & OpEvent::Key.bit() != 0;
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
            self.click(f, 0.0, 0.0, false);
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
                return true;
            }
            e = el.parent;
        }
        let f = self.input.focus;
        if name == "confirm" && f != NONE && self.els[f as usize].clickable {
            if !self.input.keyed {
                self.input.modality = Modality::Keyboard;
                self.set_state(f, FOCUS_VISIBLE, true);
                self.click(f, 0.0, 0.0, false);
            }
            return true;
        }
        false
    }

    /// The frame's input ended: what a key consumed is forgotten.
    pub fn input_done(&mut self) {
        self.input.keyed = false;
    }

    /// The modals that opened and closed since the last input (Rune's
    /// `sync_modals`): a closed top hands focus back to what had it
    /// before it opened; one that opened takes the focus to its first
    /// Tab stop, else itself, and drops the press the pointer held.
    pub(super) fn sync_modals(&mut self) {
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
        // Inner modals left under the top one's close are dropped too.
        let mut k = 0;
        while k < self.input.modals.len() {
            if live(self, self.input.modals[k].0) {
                k += 1;
            } else {
                self.input.modals.remove(k);
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
            self.tab_stops();
            let first = self.input.stops.first().copied().unwrap_or(m);
            self.focus(first);
        }
    }

    /// The actions the `<Glyph>` nodes show, when one came or changed
    /// since the last [`Tree::set_glyphs`].
    pub fn glyphs_wanted(&self) -> bool {
        self.input.glyphs_wanted
    }

    /// Writes each `<Glyph action>`'s text: the action's binding on the
    /// current device, as `glyph` answers it (empty for none). The host
    /// calls it when the device or the bindings change, or
    /// [`Tree::glyphs_wanted`].
    pub fn set_glyphs(&mut self, glyph: &dyn Fn(&str) -> Option<String>) {
        self.input.glyphs_wanted = false;
        for e in 0..self.els.len() as u32 {
            let Some(p) = self.input.per(e) else { continue };
            if p.glyph.is_empty() || self.els[e as usize].dead {
                continue;
            }
            let text = glyph(&p.glyph).unwrap_or_default();
            if text != self.els[e as usize].text {
                self.set_text(e, &text);
            }
        }
    }

    /// The op element of arena node `node`, for an assistive
    /// technology's request ([`crate::a11y::target`]).
    fn of_node(&self, node: Id) -> u32 {
        self.els
            .iter()
            .position(|el| !el.dead && el.js != NONE && el.node == node)
            .map_or(NONE, |e| e as u32)
    }

    /// An assistive technology moved the focus to `node`: with the ring,
    /// as the keys would.
    pub fn a11y_focus(&mut self, node: Id) {
        let e = self.of_node(node);
        if e != NONE && self.focusable(e) {
            self.input.modality = Modality::Keyboard;
            self.focus(e);
        }
    }

    /// An assistive technology pressed `node`, as Enter on it would; or
    /// stepped it (`up`: Increment, else Decrement), which it hears as
    /// the arrow key a slider steps by. Whether it answered.
    pub fn a11y_act(&mut self, node: Id, step: Option<bool>) -> bool {
        let e = self.of_node(node);
        if e == NONE || self.disabled(e) || !self.reachable(e) {
            return false;
        }
        match step {
            None if self.els[e as usize].clickable => {
                self.click(e, 0.0, 0.0, false);
                true
            }
            None => false,
            Some(up) => {
                let el = &self.els[e as usize];
                if el.listens & OpEvent::Key.bit() == 0 {
                    return false;
                }
                let mut ev = Fired::at(el.js, OpEvent::Key, 0.0, 0.0);
                ev.key = if up { "ArrowUp" } else { "ArrowDown" }.into();
                self.events.push(ev);
                true
            }
        }
    }
}
