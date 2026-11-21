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
use crate::classes::state::{DISABLED, FOCUS_VISIBLE, FOCUS_WITHIN, FOCUSED, HOVERED, PRESSED};
use nsh_input::input as nsh;

/// A press that travelled further than this (px) is no click of a double
/// press (Rune's `CLICK_SLOP`).
const CLICK_SLOP: f32 = 4.0;
/// Two presses this close (s) on one element are a double press (Rune's
/// `double_click_interval`).
const DOUBLE_PRESS: f64 = 0.5;
/// A hold's progress as text, "0.42", written into `buf` (no allocation).
fn progress(t: f32, buf: &mut [u8; 8]) -> &str {
    let hundredths = (t.clamp(0.0, 1.0) * 100.0) as u32;
    buf[..2].copy_from_slice(b"0.");
    buf[2] = b'0' + (hundredths / 10) as u8;
    buf[3] = b'0' + (hundredths % 10) as u8;
    std::str::from_utf8(&buf[..4]).expect("ascii")
}

/// Whether two points are within [`CLICK_SLOP`] of each other.
fn near(a: (f32, f32), b: (f32, f32)) -> bool {
    let (dx, dy) = (a.0 - b.0, a.1 - b.1);
    dx * dx + dy * dy <= CLICK_SLOP * CLICK_SLOP
}

/// A finger that travels further than this (px) pans what scrolls under
/// it, and is no tap (Rune's `TOUCH_SLOP`).
const TOUCH_SLOP: f32 = 8.0;
/// A finger held still this long (s) is a long press (Rune's
/// `LONG_PRESS`, Chromium's).
const LONG_PRESS: f64 = 0.5;
/// A pan let go after resting this long (s) throws nothing (Rune's
/// `FLING_HOLD`).
const FLING_HOLD: f64 = 0.1;
/// A fling's speed falls by e every 1 / this seconds (Rune's
/// `SCROLL_DECAY`).
const SCROLL_DECAY: f32 = 2.002;

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
    /// A `Scroll`: its children move by `scroll` (x, y), within what
    /// runs past its box.
    pub scroller: bool,
    pub scroll: [f32; 2],
    /// `hold`: seconds a press or `confirm` is held on it before it
    /// presses (0: it presses on release).
    pub hold: f32,
    /// `autofocus`: it takes the focus once it shows.
    pub autofocus: bool,
    /// `touchStick`, `touchPad` or `touchButton`: an on-screen control,
    /// its box an input source of the action named (og-input's zone),
    /// with `deadZone` a stick's dead zone.
    pub zone: Option<(og_input::ZoneKind, Box<str>)>,
    pub dead_zone: f32,
    /// An `Input`'s text, its `placeholder` (shown while the text is
    /// empty) and its caret, a byte offset at a grapheme cluster's edge.
    pub value: String,
    pub placeholder: Box<str>,
    pub caret: usize,
    /// The pack slot its class names (`@Component.slot[.variant]`, from
    /// `packClass`, without the `@`): the scope of its sounds.
    pub slot: Box<str>,
    /// `draggable`: its data (non-empty: a drag source), and `dropTarget`:
    /// its id (non-empty: a drop target).
    pub drag: Box<str>,
    pub drop: Box<str>,
    /// `focusGroup`: its focusable descendants are one group for the
    /// arrows and the d-pad (`Some(true)` wraps at its edge, `Some(false)`
    /// stops: the move leaves it); `focusMemory`: focus coming into it
    /// lands on `last`, the member it last had.
    pub group: Option<bool>,
    pub memory: bool,
    pub last: Option<u32>,
}

/// A node being dragged, or a press on one not yet past the slop.
#[derive(Clone, Copy)]
struct Drag {
    src: u32,
    /// Where the press went down.
    from: (f32, f32),
    /// Picked up: past the slop, or by a key.
    live: bool,
    /// Carried by the keys (or an assistive technology): focus moves it.
    keys: bool,
}

/// How far a mouse press moves before it drags (a finger's is the pan's).
const DRAG_SLOP: f32 = 4.0;

/// A sound the pack gives an event: one of `clips` (sound handles) at
/// random, at a volume and pitch from their ranges.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cue {
    pub clips: Vec<u64>,
    pub volume: [f32; 2],
    pub pitch: [f32; 2],
}

/// The pack's sounds with their clips loaded: by scope (`""` the pack's
/// generic ones, else `Component.slot[.variant]`), then by event.
pub type SoundTable = std::collections::BTreeMap<String, std::collections::BTreeMap<String, Cue>>;

/// One sound to play ([`Tree::take_sounds`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Play {
    pub sound: u64,
    pub volume: f32,
    pub pitch: f32,
}

/// The grapheme cluster edge before (`back`) or after byte `at` in `text`
/// (og-i18n's UAX #29 segmenter): one step of the caret, one Backspace.
fn cluster_edge(text: &str, at: usize, back: bool) -> usize {
    // The segmenter counts in characters; the caret is in bytes.
    let bytes: Vec<usize> = text
        .char_indices()
        .map(|(b, _)| b)
        .chain([text.len()])
        .collect();
    let edges = og_i18n::graphemes(text)
        .into_iter()
        .map(|(_, end)| bytes[end]);
    let edges: Vec<usize> = std::iter::once(0).chain(edges).collect();
    if back {
        edges.iter().rev().copied().find(|&b| b < at).unwrap_or(0)
    } else {
        edges
            .iter()
            .copied()
            .find(|&b| b > at)
            .unwrap_or(text.len())
    }
}

/// What a finger on a touch screen is doing.
#[derive(Clone, Copy, PartialEq)]
enum Touch {
    /// Still within the slop: a tap, or a long press when it stays.
    Pending,
    /// It pans scroller `e` along one axis, from its offset `from`.
    Pan { e: u32, horizontal: bool, from: f32 },
    /// It drags what heard its `pointerdown`, or nothing pans.
    Drag,
    /// A long press fired: it lifts on nothing.
    Long,
}

/// A finger on a touch screen (Rune's `touch.rs`).
struct Finger {
    origin: (f32, f32),
    /// Travel along the pan, its time and its speed (px/s).
    last: f32,
    at: f64,
    velocity: f32,
    mode: Touch,
    /// When, still pending, it is a long press.
    due: f64,
}

/// A press held on a node with a `hold` duration.
struct Holding {
    e: u32,
    start: f64,
    /// Held by `confirm` (Enter, Space, a pad's A), else by the pointer.
    keys: bool,
    /// It completed and pressed: it waits for the release.
    done: bool,
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
    /// The element that heard `pointerdown`: the pointer's moves and its
    /// release go to it until the release, wherever the pointer is (the
    /// DOM's implicit capture of a touch), so a drag follows the finger.
    capture: u32,
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
    /// The op elements under a point, innermost first (scratch).
    chain: Vec<u32>,
    /// The scroller wheel notches went to, along which axis and when (s).
    latch: Option<(u32, bool, f64)>,
    /// The pointer is a finger, a finger is down, and a fling coasts
    /// (scroller, axis, speed px/s, last step time).
    touch: bool,
    finger: Option<Finger>,
    fling: Option<(u32, bool, f32, f64)>,
    holding: Option<Holding>,
    /// The player used a pad last: focus a node takes by itself
    /// (`autofocus`, a modal opening) shows its ring.
    pad: bool,
    /// `autofocus` nodes waiting to show.
    pub(super) autofocus: Vec<u32>,
    /// The elements that are on-screen controls, and the zones last
    /// handed out ([`Tree::take_zones`]).
    pub(super) zone_els: Vec<u32>,
    zones: Vec<og_input::ControlZone>,
    /// The pack's sounds, what this frame plays, and the stream that
    /// picks a clip and its volume and pitch (seeded: a replay plays the
    /// same).
    sounds: std::rc::Rc<SoundTable>,
    plays: Vec<Play>,
    sound_rng: u64,
    /// A press `confirm` made: it sounds `confirm`, not `press`.
    confirming: bool,
    drag: Option<Drag>,
}

impl Input {
    pub(super) fn new() -> Input {
        Input {
            focus: NONE,
            capture: NONE,
            ..Input::default()
        }
    }

    pub(super) fn per(&self, e: u32) -> Option<&Per> {
        self.per.get(e as usize)
    }

    /// The element with the keyboard focus (NONE: none).
    pub(super) fn focus_el(&self) -> u32 {
        self.focus
    }
}

impl Tree {
    /// The `draggable` node `e` is in, itself first (NONE: none).
    fn drag_up(&self, mut e: u32) -> u32 {
        while e != NONE {
            if self.input.per(e).is_some_and(|p| !p.drag.is_empty()) && !self.disabled(e) {
                return e;
            }
            e = self.els[e as usize].parent;
        }
        NONE
    }

    /// The `dropTarget` node `e` is in, itself first (NONE: none).
    fn drop_up(&self, mut e: u32) -> u32 {
        while e != NONE {
            if self.input.per(e).is_some_and(|p| !p.drop.is_empty()) && !self.disabled(e) {
                return e;
            }
            e = self.els[e as usize].parent;
        }
        NONE
    }

    fn drop_at(&self, x: f32, y: f32) -> u32 {
        self.drop_up(self.hit_where(x, y, |el| el.js != NONE))
    }

    fn centre(&self, e: u32) -> (f32, f32) {
        let Some(&(x, y, ..)) = self.abs.get(e as usize) else {
            return (0.0, 0.0);
        };
        let b = self.arena.box_of(self.els[e as usize].node);
        (x + b.w / 2.0, y + b.h / 2.0)
    }

    fn box_text(&self, e: u32) -> Box<str> {
        let Some(&(x, y, ..)) = self.abs.get(e as usize) else {
            return Box::default();
        };
        let b = self.arena.box_of(self.els[e as usize].node);
        format!("{x} {y} {} {}", b.w, b.h).into()
    }

    /// Fires a drag event on `e` when it listens.
    fn fire_drag(&mut self, e: u32, event: OpEvent, x: f32, y: f32, key: &str, value: Box<str>) {
        let el = &self.els[e as usize];
        if el.listens & event.bit() != 0 {
            let mut f = Fired::at(el.js, event, x, y);
            f.key = key.into();
            f.value = value;
            self.events.push(f);
        }
    }

    fn data_of(&self, e: u32) -> Box<str> {
        self.input
            .per(e)
            .map_or_else(Box::default, |p| p.drag.clone())
    }

    fn target_id(&self, t: u32) -> Box<str> {
        match t {
            NONE => Box::default(),
            t => self
                .input
                .per(t)
                .map_or_else(Box::default, |p| p.drop.clone()),
        }
    }

    /// A carried node is over target `t` (NONE: none): its source hears
    /// `dragmove` with the target's id.
    fn drag_over(&mut self, t: u32, x: f32, y: f32) {
        let Some(d) = self.input.drag else { return };
        let src = d.src;
        let (data, id) = (self.data_of(src), self.target_id(t));
        self.fire_drag(src, OpEvent::DragMove, x, y, &data, id);
    }

    fn pick_up(&mut self, src: u32, x: f32, y: f32) {
        self.set_state(src, PRESSED, true);
        let (data, b) = (self.data_of(src), self.box_text(src));
        self.fire_drag(src, OpEvent::DragStart, x, y, &data, b);
        self.sound(src, "select");
    }

    /// The pointer moved while a press holds a `draggable`: past the slop
    /// it picks it up (the press then clicks nothing); once carried, each
    /// move reports the target under it.
    fn drag_moved(&mut self, x: f32, y: f32) {
        let Some(d) = self.input.drag else { return };
        if d.keys {
            return;
        }
        if !d.live {
            let slop = if self.input.touch {
                TOUCH_SLOP
            } else {
                DRAG_SLOP
            };
            let (dx, dy) = (x - d.from.0, y - d.from.1);
            if dx * dx + dy * dy <= slop * slop {
                return;
            }
            self.cancel_press();
            if let Some(d) = &mut self.input.drag {
                d.live = true;
            }
            self.pick_up(d.src, x, y);
        }
        let t = self.drop_at(x, y);
        self.drag_over(t, x, y);
    }

    /// Puts the carried node down on target `t` (NONE: back where it
    /// was): the target hears `drop` with the node's data, the node
    /// `dragend` with the target's id ("" when put back).
    fn put_down(&mut self, t: u32, x: f32, y: f32) {
        let Some(d) = self.input.drag.take() else {
            return;
        };
        let src = d.src;
        self.set_state(src, PRESSED, false);
        let (data, id) = (self.data_of(src), self.target_id(t));
        if t != NONE {
            let b = self.box_text(t);
            self.fire_drag(t, OpEvent::Drop, x, y, &data, b);
            self.sound(t, "confirm");
        } else {
            self.sound(src, "cancel");
        }
        self.fire_drag(src, OpEvent::DragEnd, x, y, &data, id);
        // Drop targets are focus stops only while a node is carried.
        if d.keys && self.input.focus != NONE && !self.focusable(self.input.focus) {
            self.focus(src);
        }
    }

    /// Confirm (a key, the action or an assistive technology) on `e`:
    /// with a node carried by the keys, puts it down on the target `e` is
    /// in (else back); else picks `e` up when it is `draggable`. Whether
    /// it did either.
    fn carry(&mut self, e: u32) -> bool {
        if e == NONE {
            return false;
        }
        match self.input.drag {
            Some(d) if d.live && d.keys => {
                let t = self.drop_up(e);
                let (x, y) = self.centre(e);
                self.put_down(t, x, y);
                true
            }
            Some(_) => false,
            None if self.input.per(e).is_some_and(|p| !p.drag.is_empty()) && !self.disabled(e) => {
                self.input.drag = Some(Drag {
                    src: e,
                    from: self.centre(e),
                    live: true,
                    keys: true,
                });
                let (x, y) = self.centre(e);
                self.pick_up(e, x, y);
                true
            }
            None => false,
        }
    }

    /// `e` is freed: a drag of it ends without events.
    pub(super) fn forget_drag(&mut self, e: u32) {
        if self.input.drag.is_some_and(|d| d.src == e) {
            self.input.drag = None;
        }
    }

    /// The pack's sounds, their clips loaded (the host's).
    pub fn set_sounds(&mut self, sounds: std::rc::Rc<SoundTable>) {
        self.input.sounds = sounds;
    }

    /// The sounds this frame's events played, oldest first.
    pub fn take_sounds(&mut self) -> Vec<Play> {
        std::mem::take(&mut self.input.plays)
    }

    /// `e` plays `event`: its slot's sound (with its variant, then
    /// without), else the pack's generic one; `confirm` and `back` fall
    /// back to `press`. Nothing when the pack has none.
    fn sound(&mut self, e: u32, event: &str) {
        let table = &self.input.sounds;
        if table.is_empty() {
            return;
        }
        let slot = self.input.per(e).map_or("", |p| &*p.slot);
        let bare = slot
            .rsplit_once('.')
            .map(|x| x.0)
            .filter(|b| b.contains('.'));
        let scopes = [Some(slot).filter(|s| !s.is_empty()), bare, Some("")];
        let fallback = matches!(event, "confirm" | "back").then_some("press");
        let cue = [Some(event), fallback]
            .into_iter()
            .flatten()
            .find_map(|ev| scopes.iter().flatten().find_map(|s| table.get(*s)?.get(ev)))
            .filter(|c| !c.clips.is_empty());
        let Some(cue) = cue else {
            return;
        };
        let mut next = || {
            // xorshift64*, as og-ui's feedback.
            let mut x = self.input.sound_rng;
            if x == 0 {
                x = 0x9e37_79b9_7f4a_7c15;
            }
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            self.input.sound_rng = x;
            (x.wrapping_mul(0x2545_f491_4f6c_dd1d) >> 40) as f32 / (1u64 << 24) as f32
        };
        let n = cue.clips.len();
        let sound = cue.clips[((next() * n as f32) as usize).min(n - 1)];
        let range = |r: [f32; 2], t: f32| r[0] + (r[1] - r[0]) * t;
        let volume = range(cue.volume, next());
        let pitch = range(cue.pitch, next());
        if sound != 0 {
            self.input.plays.push(Play {
                sound,
                volume,
                pitch,
            });
        }
    }

    /// Shows an `Input`'s text, or its placeholder while it is empty, and
    /// paints its caret again.
    pub(super) fn show_value(&mut self, e: u32) {
        let p = &self.input.per[e as usize];
        let shown = if p.value.is_empty() {
            &*p.placeholder
        } else {
            &*p.value
        };
        let shown = shown.to_owned();
        self.set_text(e, &shown);
        self.paint = true;
    }

    /// The focused `Input` that takes edits: shown and enabled.
    fn editing(&self) -> Option<u32> {
        let f = self.input.focus;
        (f != NONE && self.input.per(f).is_some_and(|p| p.input) && !self.disabled(f)).then_some(f)
    }

    /// Text typed this frame (og-input's `og.TextInput` stream, or the
    /// printable keys) goes into the focused `Input` at its caret, which
    /// hears `change` with its new text. Whether one took it.
    pub fn type_text(&mut self, text: &str) -> bool {
        let Some(e) = self.editing() else {
            return false;
        };
        let p = &mut self.input.per[e as usize];
        let at = p.caret.min(p.value.len());
        let before = p.value.len();
        for ch in text.chars().filter(|c| !c.is_control()) {
            p.value.insert(at + (p.value.len() - before), ch);
        }
        if p.value.len() == before {
            return false;
        }
        p.caret = at + (p.value.len() - before);
        self.edited(e);
        true
    }

    /// An editing key on the focused `Input`: Backspace and Delete remove
    /// the grapheme cluster before and after the caret, ←/→ step over one,
    /// Home and End go to the ends. Whether it was one.
    fn edit_key(&mut self, e: u32, code: &str) -> bool {
        let p = &mut self.input.per[e as usize];
        let at = p.caret.min(p.value.len());
        match code {
            "Backspace" | "Delete" => {
                let (a, b) = if code == "Backspace" {
                    (cluster_edge(&p.value, at, true), at)
                } else {
                    (at, cluster_edge(&p.value, at, false))
                };
                p.caret = a;
                if a < b {
                    p.value.replace_range(a..b, "");
                    self.edited(e);
                }
            }
            "ArrowLeft" => p.caret = cluster_edge(&p.value, at, true),
            "ArrowRight" => p.caret = cluster_edge(&p.value, at, false),
            "Home" => p.caret = 0,
            "End" => p.caret = p.value.len(),
            _ => return false,
        }
        self.paint = true;
        true
    }

    fn edited(&mut self, e: u32) {
        self.show_value(e);
        let value = self.input.per[e as usize].value.clone();
        self.fire_value(e, OpEvent::Change, &value);
    }

    /// The on-screen controls' zones for og-input's action map
    /// ([`og_input::ActionMap::set_zones`]), after [`Tree::present`]: each
    /// shown, enabled control's box in surface px. `None` when none came,
    /// went, moved or changed since the last call; such a frame allocates
    /// nothing.
    pub fn take_zones(&mut self) -> Option<Vec<og_input::ControlZone>> {
        let mut k = 0;
        let mut changed = false;
        for i in 0..self.input.zone_els.len() {
            let e = self.input.zone_els[i] as usize;
            let p = &self.input.per[e];
            let Some((kind, action)) = &p.zone else {
                continue;
            };
            let Some(&(x, y, true, _)) = self.abs.get(e) else {
                continue;
            };
            let b = self.arena.box_of(self.els[e].node);
            if self.els[e].dead || p.states & DISABLED != 0 || b.w <= 0.0 || b.h <= 0.0 {
                continue;
            }
            let same = |z: &og_input::ControlZone| {
                z.kind == *kind
                    && *z.action == **action
                    && [z.x, z.y, z.width, z.height, z.dead_zone] == [x, y, b.w, b.h, p.dead_zone]
            };
            let next = || og_input::ControlZone {
                action: action.to_string(),
                kind: *kind,
                x,
                y,
                width: b.w,
                height: b.h,
                dead_zone: p.dead_zone,
            };
            match self.input.zones.get_mut(k) {
                Some(z) if same(z) => {}
                Some(z) => {
                    *z = next();
                    changed = true;
                }
                None => {
                    self.input.zones.push(next());
                    changed = true;
                }
            }
            k += 1;
        }
        if self.input.zones.len() != k {
            self.input.zones.truncate(k);
            changed = true;
        }
        changed.then(|| self.input.zones.clone())
    }

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

    /// Fires `event` on `e` when it listens. A pointer event's `value` is
    /// `e`'s box on the surface, "x y width height", so a slider turns
    /// the pointer into a value on its own track.
    fn fire(&mut self, e: u32, event: OpEvent, x: f32, y: f32) {
        let el = &self.els[e as usize];
        if el.listens & event.bit() == 0 {
            return;
        }
        let mut f = Fired::at(el.js, event, x, y);
        if matches!(
            event,
            OpEvent::PointerDown | OpEvent::PointerMove | OpEvent::PointerUp | OpEvent::LongPress
        ) && let Some(&(bx, by, ..)) = self.abs.get(e as usize)
        {
            let b = self.arena.box_of(el.node);
            f.value = format!("{bx} {by} {} {}", b.w, b.h).into();
        }
        self.events.push(f);
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
        // The innermost control the pointer came over (they come from the
        // outside in) sounds `hover`; a finger has no hover.
        let mut control = NONE;
        for &e in came {
            self.set_state(e, HOVERED, true);
            if self.els[e as usize].clickable && !self.disabled(e) {
                control = e;
            }
        }
        if control != NONE && !self.input.touch {
            self.sound(control, "hover");
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
        // While a pointerdown holds it, the moves go to its listener.
        let to = match self.input.capture {
            c if c != NONE && !self.els[c as usize].dead => c,
            _ => under,
        };
        if moved && to != NONE {
            let l = self.up(to, |el| el.listens & OpEvent::PointerMove.bit() != 0);
            if l != NONE {
                self.fire(l, OpEvent::PointerMove, x, y);
            }
        }
        if moved && !pressed {
            self.drag_moved(x, y);
            self.finger_moved(x, y);
        }
        let mut took = false;
        if pressed {
            if under != NONE {
                let l = self.up(under, |el| el.listens & OpEvent::PointerDown.bit() != 0);
                if l != NONE {
                    self.fire(l, OpEvent::PointerDown, x, y);
                }
                self.input.capture = l;
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
                if self.input.per(e).is_some_and(|p| p.hold > 0.0) {
                    self.input.holding = Some(Holding {
                        e,
                        start: self.time,
                        keys: false,
                        done: false,
                    });
                }
            }
            // A `draggable` takes the press whether or not it answers one:
            // from the pressed element, else from what is under the pointer.
            let src = self.drag_up(if took { e } else { under });
            if src != NONE && self.input.drag.is_none() {
                took = true;
                self.input.drag = Some(Drag {
                    src,
                    from: (x, y),
                    live: false,
                    keys: false,
                });
            }
            // A finger stops a fling under it, and may pan or long-press.
            if self.input.touch {
                self.input.fling = None;
                self.input.finger = Some(Finger {
                    origin: (x, y),
                    last: 0.0,
                    at: self.time,
                    velocity: 0.0,
                    mode: Touch::Pending,
                    due: self.time + LONG_PRESS,
                });
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
            match self.input.drag {
                Some(d) if d.live && !d.keys => {
                    let t = self.drop_at(x, y);
                    self.put_down(t, x, y);
                }
                Some(d) if !d.live => self.input.drag = None,
                _ => {}
            }
            let c = std::mem::replace(&mut self.input.capture, NONE);
            let to = if c != NONE && !self.els[c as usize].dead {
                c
            } else {
                under
            };
            if to != NONE {
                let l = self.up(to, |el| el.listens & OpEvent::PointerUp.bit() != 0);
                if l != NONE {
                    self.fire(l, OpEvent::PointerUp, x, y);
                }
            }
            if let Some(f) = self.input.finger.take()
                && let Touch::Pan { e, horizontal, .. } = f.mode
                && self.time - f.at < FLING_HOLD
                && f.velocity.abs() > 1.0
            {
                self.input.fling = Some((e, horizontal, f.velocity, self.time));
            }
            if let Some(h) = self.input.held.take() {
                took = true;
                self.set_state(h.e, PRESSED, false);
                let el = &self.els[h.e as usize];
                let holds = self.input.per(h.e).is_some_and(|p| p.hold > 0.0);
                if holds {
                    // A hold presses when it completes, not on release.
                    self.input.first = None;
                } else if !el.dead && el.js != NONE && self.inside(h.e, x, y) && !self.disabled(h.e)
                {
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
        let event = if std::mem::take(&mut self.input.confirming) {
            "confirm"
        } else {
            "press"
        };
        self.sound(e, event);
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

    /// The op elements at (x, y), innermost first, into `chain`.
    fn chain_at(&mut self, x: f32, y: f32) {
        let mut chain = std::mem::take(&mut self.input.chain);
        chain.clear();
        let mut e = self.hit_where(x, y, |el| el.js != NONE);
        while e != NONE {
            if self.els[e as usize].js != NONE {
                chain.push(e);
            }
            e = self.els[e as usize].parent;
        }
        self.input.chain = chain;
    }

    /// How far a scroller's children may move it: what runs past its box,
    /// right and down.
    fn extent(&self, e: u32) -> [f32; 2] {
        let node = self.els[e as usize].node;
        let b = self.arena.box_of(node);
        let (mut right, mut bottom) = (0.0f32, 0.0f32);
        for &c in self.arena.children(node) {
            let k = self.arena.box_of(c);
            right = right.max(k.x + k.w);
            bottom = bottom.max(k.y + k.h);
        }
        [
            (right + b.inset[2] - b.w).max(0.0),
            (bottom + b.inset[3] - b.h).max(0.0),
        ]
    }

    fn scroller(&self, e: u32) -> bool {
        self.input.per(e).is_some_and(|p| p.scroller) && !self.els[e as usize].dead
    }

    /// A scroller's axis: horizontal when only its width runs over.
    fn axis(&self, e: u32) -> Option<bool> {
        if !self.scroller(e) {
            return None;
        }
        let [w, h] = self.extent(e);
        Some(h <= 0.0 && w > 0.0)
    }

    /// How far op element `e`'s children are moved: its scroll.
    pub(super) fn scroll_of(&self, e: u32) -> [f32; 2] {
        match e {
            NONE => [0.0; 2],
            e => self.input.per(e).map_or([0.0; 2], |p| p.scroll),
        }
    }

    /// Op element `e`'s scroll offset along `horizontal` (else down).
    pub(super) fn scroll_at(&self, e: u32, horizontal: bool) -> f32 {
        self.scroll_of(e)[usize::from(!horizontal)]
    }

    /// Sets scroller `e`'s offset along an axis, within its extent;
    /// whether it moved.
    fn scroll_to(&mut self, e: u32, horizontal: bool, to: f32) -> bool {
        let k = usize::from(!horizontal);
        let to = to.clamp(0.0, self.extent(e)[k]);
        let p = &mut self.input.per[e as usize];
        if p.scroll[k] == to {
            return false;
        }
        p.scroll[k] = to;
        self.paint = true;
        true
    }

    /// After a layout: each scroller's offset stays within its extent.
    pub(super) fn clamp_scrolls(&mut self) {
        for e in 0..self.input.per.len() as u32 {
            if self.scroller(e) {
                let [x, y] = self.scroll_of(e);
                self.scroll_to(e, true, x);
                self.scroll_to(e, false, y);
            }
        }
    }

    /// A wheel by (dx, dy) at (x, y). The nearest op element under it
    /// that listens hears it, with the deltas in `x` and `y`, and takes it
    /// unless a scroller is nearer (NSH's `wheel_listener`); else the
    /// innermost scroller along the wheel's axis that can move scrolls
    /// (NSH's `scroll_target`), and notches that keep coming stay on it
    /// (NSH's `latch_holds`).
    pub fn wheel(&mut self, x: f32, y: f32, dx: f32, dy: f32) {
        if dx == 0.0 && dy == 0.0 {
            return;
        }
        self.chain_at(x, y);
        let chain = std::mem::take(&mut self.input.chain);
        let heard = nsh::wheel_listener(
            chain.iter().copied(),
            |&e| self.els[e as usize].listens & OpEvent::Wheel.bit() != 0,
            |&e| self.scroller(e),
        );
        if let Some((l, _)) = heard {
            self.fire(l, OpEvent::Wheel, dx, dy);
        }
        let taken = heard.is_some_and(|(_, takes)| takes);
        let horizontal = dx.abs() > dy.abs();
        let delta = |axis_h: bool| if axis_h && dx != 0.0 { dx } else { dy };
        let now = self.time * 1000.0;
        let latched = self
            .input
            .latch
            .filter(|&(e, h, last)| self.scroller(e) && nsh::latch_holds(h, last, horizontal, now));
        let target = match latched {
            Some((e, ..)) => Some(e),
            None if taken => None,
            None => nsh::scroll_target(
                chain
                    .iter()
                    .copied()
                    .filter(|&e| self.scroller(e))
                    .collect(),
                nsh_input::Points(dx),
                nsh_input::Points(dy),
                |&e| self.axis(e),
                |&e| {
                    let h = self.axis(e) == Some(true);
                    let (at, max) = (self.scroll_at(e, h), self.extent(e)[usize::from(!h)]);
                    let d = delta(h);
                    (d > 0.0 && at < max) || (d < 0.0 && at > 0.0)
                },
            ),
        };
        self.input.chain = chain;
        if let Some(e) = target {
            let h = self.axis(e) == Some(true);
            self.scroll_to(e, h, self.scroll_at(e, h) + delta(h));
            self.input.fling = None;
            self.input.latch = Some((e, horizontal, now));
        }
    }

    /// The device the player used last: a touch screen (`touch`: a drag
    /// pans what scrolls, a still finger long-presses) or a pad (`pad`:
    /// focus a node takes by itself shows its ring).
    pub fn set_device(&mut self, touch: bool, pad: bool) {
        self.input.touch = touch;
        self.input.pad = pad;
    }

    /// Scrolls the scrollers around op element `e` so its box shows, as
    /// focus moved by the keys does (CSS's `scrollIntoView` nearest).
    fn reveal(&mut self, e: u32) {
        let Some(&(mut ex, mut ey, ..)) = self.abs.get(e as usize) else {
            return;
        };
        let b = self.arena.box_of(self.els[e as usize].node);
        let mut s = self.els[e as usize].parent;
        while s != NONE {
            if self.scroller(s)
                && let Some(&(sx, sy, ..)) = self.abs.get(s as usize)
            {
                let sb = self.arena.box_of(self.els[s as usize].node);
                for (h, at, size, from, span) in
                    [(true, ex, b.w, sx, sb.w), (false, ey, b.h, sy, sb.h)]
                {
                    let by = if at < from {
                        at - from
                    } else if at + size > from + span {
                        (at + size - from - span).min(at - from)
                    } else {
                        0.0
                    };
                    let old = self.scroll_at(s, h);
                    if by != 0.0 && self.scroll_to(s, h, old + by) {
                        let moved = self.scroll_at(s, h) - old;
                        if h {
                            ex -= moved;
                        } else {
                            ey -= moved;
                        }
                    }
                }
            }
            s = self.els[s as usize].parent;
        }
    }

    /// A press on a node with a `hold` starts it; elsewhere it presses.
    fn press_or_hold(&mut self, e: u32, keys: bool) {
        if self.input.per(e).is_some_and(|p| p.hold > 0.0) {
            if self.input.holding.as_ref().is_none_or(|h| h.e != e) {
                self.input.holding = Some(Holding {
                    e,
                    start: self.time,
                    keys,
                    done: false,
                });
            }
        } else {
            self.click(e, 0.0, 0.0, false);
        }
    }

    fn fire_value(&mut self, e: u32, event: OpEvent, value: &str) {
        let el = &self.els[e as usize];
        if el.listens & event.bit() != 0 {
            let mut f = Fired::at(el.js, event, 0.0, 0.0);
            f.value = value.into();
            self.events.push(f);
        }
    }

    /// Whether op element `e` takes focus: shown, enabled, reachable, and
    /// a press listener, an input or a `tabIndex`.
    fn focusable(&self, e: u32) -> bool {
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
    fn step(&mut self, f: u32, dir: og_ui::Nav) -> Option<u32> {
        let from = self.rect(f);
        let (fx, fy) = from.center();
        // A move that went back the other way is a wrap.
        let ahead = |r: &og_ui::Rect| {
            let (cx, cy) = r.center();
            match dir {
                og_ui::Nav::Down => cy > fy,
                og_ui::Nav::Up => cy < fy,
                og_ui::Nav::Right => cx > fx,
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
            let hit = og_ui::nav_neighbour(&from, &cands, dir).map(|k| k as u32);
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
        // The focused `Input` edits with its keys (arrows across its text
        // included); the game's shortcuts with Control, Alt or Command
        // pass on.
        if let Some(e) = self.editing()
            && !(ctrl || alt || meta)
            && self.edit_key(e, code)
        {
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

    /// The press the pointer holds lets go without a press.
    fn cancel_press(&mut self) {
        if let Some(h) = self.input.held.take() {
            self.set_state(h.e, PRESSED, false);
        }
        self.input.first = None;
    }

    /// A finger moved to (x, y): past the slop it pans the scroller under
    /// where it went down along the way it went, unless what heard its
    /// `pointerdown` drags; a pan lets the press go.
    fn finger_moved(&mut self, x: f32, y: f32) {
        let Some(f) = &self.input.finger else { return };
        let (ox, oy) = f.origin;
        let (dx, dy) = (x - ox, y - oy);
        if f.mode == Touch::Pending && dx * dx + dy * dy > TOUCH_SLOP * TOUCH_SLOP {
            let horizontal = dx.abs() > dy.abs();
            let mode = if self.input.capture != NONE || self.input.drag.is_some_and(|d| d.live) {
                Touch::Drag
            } else {
                self.chain_at(ox, oy);
                let e =
                    self.input.chain.iter().copied().find(|&e| {
                        self.scroller(e) && self.extent(e)[usize::from(!horizontal)] > 0.0
                    });
                match e {
                    Some(e) => Touch::Pan {
                        e,
                        horizontal,
                        from: self.scroll_at(e, horizontal),
                    },
                    None => Touch::Drag,
                }
            };
            if matches!(mode, Touch::Pan { .. }) {
                self.cancel_press();
            }
            if let Some(f) = &mut self.input.finger {
                f.mode = mode;
            }
        }
        let now = self.time;
        let Some(f) = &mut self.input.finger else {
            return;
        };
        if let Touch::Pan {
            e,
            horizontal,
            from,
        } = f.mode
        {
            let along = if horizontal { dx } else { dy };
            let dt = (now - f.at) as f32;
            if dt > 0.0 {
                f.velocity = -(along - f.last) / dt;
            }
            (f.last, f.at) = (along, now);
            self.scroll_to(e, horizontal, from - along);
        }
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
            None if self.carry(e) => true,
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
