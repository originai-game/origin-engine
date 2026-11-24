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
pub(super) const DOUBLE_PRESS: f64 = 0.5;
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
pub(super) const TOUCH_SLOP: f32 = 8.0;
/// A finger held still this long (s) is a long press (Rune's
/// `LONG_PRESS`, Chromium's).
pub(super) const LONG_PRESS: f64 = 0.5;
/// A pan let go after resting this long (s) throws nothing (Rune's
/// `FLING_HOLD`).
const FLING_HOLD: f64 = 0.1;
/// A fling's speed falls by e every 1 / this seconds (Rune's
/// `SCROLL_DECAY`).
const SCROLL_DECAY: f32 = 2.002;

/// A pointer let go faster than this (px/s) along its main axis swipes,
/// once it has travelled [`SWIPE_DISTANCE`] (Hammer.js's swipe: 0.3 px/ms
/// and 10 px).
const SWIPE_VELOCITY: f32 = 300.0;
const SWIPE_DISTANCE: f32 = 10.0;

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
    /// The action whose binding its text shows (`<Glyph action>`), and the
    /// texture slot of the pack's art for that binding on the current
    /// device (0: none, the text shows).
    pub glyph: Box<str>,
    pub art: u32,
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
    pub floating_stick: bool,
    /// Floating stick's centre relative to the zone's centre, tree units.
    pub stick_centre: [f32; 2],
    /// An on-screen control's knob as og-input last read it (from the
    /// zone's middle, y down), and `knob`: this node moves by the knob of
    /// the control it is in, at paint.
    pub knob: [f32; 2],
    pub rides_knob: bool,
    /// `thumb`: the name of the scroller it is the scroll bar thumb of,
    /// and the length and axis the tree last fitted it to (0: none).
    pub thumb: Box<str>,
    pub thumb_len: f32,
    pub thumb_horizontal: bool,
    /// What its last `scroll` event said of its viewport and content
    /// (`onScroll` reports again when they change, not only its offset).
    pub reported: [f32; 4],
    /// `pinned`: in a scroller, it keeps its place while the content
    /// scrolls, and the scroller's extent leaves it out (a scroll bar).
    pub pinned: bool,
    /// An `Input`'s text, its `placeholder` (shown while the text is
    /// empty) and its caret, a byte offset at a grapheme cluster's edge.
    pub value: String,
    pub placeholder: Box<str>,
    pub caret: usize,
    /// The other selection edge (equal to the caret when collapsed).
    pub anchor: usize,
    /// Active IME preedit text (shown inline at the selection, committed on `type_text`).
    pub composing: String,
    /// HTML input hints forwarded to the host's hidden text element.
    pub input_type: Box<str>,
    pub input_mode: Box<str>,
    pub autocomplete: Box<str>,
    /// Password input: displays bullets and refuses copy/cut.
    pub secret: bool,
    /// Multi-line text area: permits newlines.
    pub multiline: bool,
    /// Maximum Unicode scalar count (0 = unlimited).
    pub max_length: usize,
    /// Read-only field: allows focus/selection/copy, refuses edits.
    pub read_only: bool,
    /// `rows`: a multi-line field at least that many lines tall (0: its text's).
    pub rows: u16,
    /// An `Input`'s class as the game gave it, which [`Tree::field_class`]
    /// puts its default field slot before.
    pub own_class: Box<str>,
    /// How far a one-line field's text is scrolled left to keep the caret
    /// in view, CSS px.
    pub scroll_x: f32,
    /// How far a multi-line field's text is scrolled up to keep the caret
    /// in view, CSS px.
    pub scroll_y: f32,
    /// Preferred caret x coordinate preserved across consecutive ↑/↓ steps
    /// in a multi-line field (`None` outside vertical movement).
    pub goal_x: Option<f32>,
    /// UI clock time when the caret last moved or edited (anchors blink phase).
    pub caret_at: f64,
    /// The states undo goes back to, oldest first, and redo forward to;
    /// the last edit's kind and time, which typing joins.
    pub undo: Vec<Snap>,
    pub redo: Vec<Snap>,
    pub last_edit: (Edit, f64),
    /// The pack slot its class names (`@Component.slot[.variant]`, from
    /// `packClass`, without the `@`): the scope of its sounds.
    pub slot: Box<str>,
    /// `sound`: the pack event the node sounds when it first paints
    /// (empty: none, or sounded already).
    pub cue: Box<str>,
    /// `exitSound`: the pack event the node sounds when removed/unmounted
    /// (e.g. `"close"` on dialogs, sheets, popovers, menus).
    pub exit_cue: Box<str>,
    /// `haptic`: preset override played with non-hover interactions and cues
    /// (empty: pack/default; `"none"`: silence).
    pub haptic: Box<str>,
    /// `draggable`: its data (non-empty: a drag source), and `dropTarget`:
    /// its id (non-empty: a drop target).
    pub drag: Box<str>,
    pub drop: Box<str>,
    /// `name`: what `anchor` and `spotlight` find it by (empty: none).
    pub name: Box<str>,
    /// `spotlight`: the name of the node it lights; its `bg-*` covers the
    /// surface outside that node's box, and it takes presses only there.
    pub spotlight: Box<str>,
    /// `focusGroup`: its focusable descendants are one group for the
    /// arrows and the d-pad (`Some(true)` wraps at its edge, `Some(false)`
    /// stops: the move leaves it); `focusMemory`: focus coming into it
    /// lands on `last`, the member it last had.
    pub group: Option<bool>,
    pub memory: bool,
    pub last: Option<u32>,
    /// `focusRoving`: with `focusGroup`, the group is one Tab stop
    /// (WAI-ARIA's roving tab index, NSH base's `roving.rs`).
    pub roving: bool,
}

/// An `Input`'s text and selection, as undo and redo restore them.
#[derive(Clone, Default)]
pub(super) struct Snap {
    value: String,
    caret: usize,
    anchor: usize,
}

/// What an `Input` edit was, for undo's grouping: typing joins the typing
/// before it and deleting the deleting, until the caret moves, the kind
/// changes or [`UNDO_PAUSE`] passes; anything else (a paste, a cut, typing
/// over a selection) is a step of its own.
#[derive(Clone, Copy, Default, PartialEq)]
pub(super) enum Edit {
    #[default]
    None,
    Type,
    Delete,
    Other,
}

/// Seconds without an edit that end an undo group.
const UNDO_PAUSE: f64 = 1.0;
/// The most undo steps a field keeps.
const UNDO_STEPS: usize = 100;

/// A sound the pack gives an event: one of `clips` (sound handles) at
/// random, at a volume and pitch from their ranges.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cue {
    pub clips: Vec<u64>,
    pub volume: [f32; 2],
    pub pitch: [f32; 2],
}

impl Cue {
    /// One play of it: a clip at random, its volume and pitch from their
    /// ranges (a pitch at `rise`, 0 to 1, of its range when given), drawn
    /// from `rng` (xorshift64*, as og-ui's feedback). None without a clip.
    pub fn pick(&self, rng: &mut u64, rise: Option<f32>) -> Option<Play> {
        if self.clips.is_empty() {
            return None;
        }
        let mut next = || {
            let mut x = *rng;
            if x == 0 {
                x = 0x9e37_79b9_7f4a_7c15;
            }
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            *rng = x;
            (x.wrapping_mul(0x2545_f491_4f6c_dd1d) >> 40) as f32 / (1u64 << 24) as f32
        };
        let n = self.clips.len();
        let sound = self.clips[((next() * n as f32) as usize).min(n - 1)];
        let range = |r: [f32; 2], t: f32| r[0] + (r[1] - r[0]) * t;
        let volume = range(self.volume, next());
        let pitch = range(
            self.pitch,
            rise.map_or_else(&mut next, |r| r.clamp(0.0, 1.0)),
        );
        (sound != 0).then_some(Play {
            sound,
            volume,
            pitch,
        })
    }
}

/// The pack's sounds with their clips loaded: by scope (`""` the pack's
/// generic ones, else `Component.slot[.variant]`), then by event.
pub type SoundTable = std::collections::BTreeMap<String, std::collections::BTreeMap<String, Cue>>;

/// The pack's haptic patterns: by scope (`""` pack-wide, else
/// `Component.slot[.variant]`), then by event.
pub type HapticTable =
    std::collections::BTreeMap<String, std::collections::BTreeMap<String, og_ui_model::Haptic>>;

/// One sound to play ([`Tree::take_sounds`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Play {
    pub sound: u64,
    pub volume: f32,
    pub pitch: f32,
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
pub(super) enum Modality {
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
    /// The cursor the element under the pointer asks for.
    cursor: nsh_tailwind::effects::Cursor,
    next: Vec<u32>,
    held: Option<Held>,
    /// The element that heard `pointerdown`: the pointer's moves and its
    /// release go to it until the release, wherever the pointer is (the
    /// DOM's implicit capture of a touch), so a drag follows the finger.
    capture: u32,
    /// The last click: its element, time and place, for a double press.
    first: Option<(u32, f64, (f32, f32))>,
    /// The last press in a field's text: its element, time, place and how
    /// many presses in a row (1 places the caret, 2 selects a word, 3 all).
    field_press: Option<(u32, f64, (f32, f32), u8)>,
    /// The field a held press is selecting in, as the pointer drags.
    selecting: Option<u32>,
    /// The field whose text was last scrolled to its caret.
    caret_scrolled: Option<u32>,
    pub(super) focus: u32,
    pub(super) modality: Modality,
    /// The open modals, innermost last, each with the focus it returns.
    pub(super) modals: Vec<(u32, u32)>,
    /// The elements with `modal`.
    pub(super) modal_els: Vec<u32>,
    /// Tab stops, made on a Tab or a move, and a move's candidates.
    stops: Vec<u32>,
    cands: Vec<(u64, og_ui_model::Rect)>,
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
    pub(super) touch: bool,
    finger: Option<Finger>,
    fling: Option<(u32, bool, f32, f64)>,
    holding: Option<Holding>,
    /// The player used a pad last: focus a node takes by itself
    /// (`autofocus`, a modal opening) shows its ring.
    pub(super) pad: bool,
    /// `autofocus` nodes waiting to show.
    pub(super) autofocus: Vec<u32>,
    /// The elements that are on-screen controls, and the zones last
    /// handed out ([`Tree::take_zones`]).
    pub(super) zone_els: Vec<u32>,
    zones: Vec<og_input::ControlZone>,
    /// The element each zone handed out came from, in their order, and
    /// the ones of the call before ([`Tree::take_zones`]'s scratch).
    zone_from: Vec<u32>,
    zone_was: Vec<u32>,
    /// Pointers 0 to 4 on elements that hear a gesture, and the two
    /// fingers that pinch and turn one ([`Tree::touches`]).
    gestures: [Option<Gesturing>; 5],
    pinch: Option<Pinching>,
    /// The pack's sounds, what this frame plays, and the stream that
    /// picks a clip and its volume and pitch (seeded: a replay plays the
    /// same).
    sounds: std::rc::Rc<SoundTable>,
    plays: Vec<Play>,
    pub(super) sound_rng: u64,
    /// The pack's haptics and what this frame's UI interactions played.
    haptics_table: std::rc::Rc<HapticTable>,
    haptics: Vec<og_ui_model::Haptic>,
    /// A press `confirm` made: it sounds `confirm`, not `press`.
    confirming: bool,
    drag: Option<Drag>,
    /// Text copied or cut from the focused `Input`, or copied from a
    /// selected text, this frame (`Tree::take_copy`).
    pub(super) copied: Option<String>,
    /// The scroll bar thumbs, the one the pointer holds and where on it,
    /// and the scrollers that moved since the events were last taken.
    pub(super) thumbs: Vec<u32>,
    pub(super) grab: Option<(u32, f32)>,
    pub(super) scrolled: Vec<u32>,
    /// The arena nodes of `pinned` elements (few: scroll bars).
    pub(super) pinned: Vec<Id>,
}

impl Input {
    /// Clears transient pointer, fling, drag, gesture and hold interactions
    /// on checkpoint restore.
    pub(super) fn clear_transient(&mut self) {
        self.fling = None;
        self.latch = None;
        self.grab = None;
        self.selecting = None;
        self.field_press = None;
        self.holding = None;
        self.held = None;
        self.drag = None;
        self.finger = None;
        self.gestures = [None; 5];
        self.pinch = None;
        self.first = None;
        self.keyed = false;
        self.hover.clear();
        self.capture = NONE;
    }
}

/// The focused `Input` reported to the host shell (`packages/engine/src/web/text-input.ts`)
/// so the page can position its hidden `<input>`/`<textarea>` for mobile keyboards,
/// IME candidate windows, autofill, and clipboard events.
#[derive(Clone, Debug, PartialEq)]
pub struct TextTarget {
    pub path: String,
    /// The field's box and its caret bar, in surface (CSS) px: an input
    /// method's candidate window opens at the caret.
    pub rect: [f32; 4],
    pub caret: [f32; 4],
    pub multiline: bool,
    pub selection: String,
    pub composing: bool,
    pub input_type: String,
    pub input_mode: String,
    pub autocomplete: String,
    pub secret: bool,
}

impl TextTarget {
    pub fn to_json(&self) -> serde_json::Value {
        let mut obj = serde_json::Map::new();
        obj.insert("path".into(), serde_json::json!(self.path));
        obj.insert("rect".into(), serde_json::json!(self.rect));
        obj.insert("caret".into(), serde_json::json!(self.caret));
        obj.insert("multiline".into(), serde_json::json!(self.multiline));
        obj.insert("selection".into(), serde_json::json!(self.selection));
        obj.insert("composing".into(), serde_json::json!(self.composing));
        if !self.input_type.is_empty() {
            obj.insert("type".into(), serde_json::json!(self.input_type));
        }
        if !self.input_mode.is_empty() {
            obj.insert("inputmode".into(), serde_json::json!(self.input_mode));
        }
        if !self.autocomplete.is_empty() {
            obj.insert("autocomplete".into(), serde_json::json!(self.autocomplete));
        }
        if self.secret {
            obj.insert("secret".into(), serde_json::json!(true));
        }
        serde_json::Value::Object(obj)
    }
}

impl Per {
    /// The byte offset in `self.value` at byte `shown` of the displayed
    /// text (its bullets when `secret`): [`Per::display_offset`]'s inverse.
    pub(crate) fn value_offset(&self, shown: usize) -> usize {
        if self.secret {
            let n = shown / '•'.len_utf8();
            self.value
                .char_indices()
                .nth(n)
                .map_or(self.value.len(), |(b, _)| b)
        } else {
            let mut at = shown.min(self.value.len());
            while !self.value.is_char_boundary(at) {
                at -= 1;
            }
            at
        }
    }

    /// Byte offset in the displayed text corresponding to a byte offset in `self.value`.
    pub(crate) fn display_offset(&self, byte: usize) -> usize {
        let clamped = byte.min(self.value.len());
        if self.secret {
            self.value[..clamped].chars().count() * '•'.len_utf8()
        } else {
            clamped
        }
    }

    /// The selection range in `self.value` bytes, ordered `start <= end`.
    pub(crate) fn selection_range(&self) -> (usize, usize) {
        let at = self.caret.min(self.value.len());
        let anc = self.anchor.min(self.value.len());
        (anc.min(at), anc.max(at))
    }

    /// The active IME composition range in displayed text bytes, if composing.
    pub(crate) fn composing_display_range(&self) -> Option<(usize, usize)> {
        if self.composing.is_empty() {
            return None;
        }
        let (start, _) = self.selection_range();
        let d_start = self.display_offset(start);
        let d_len = if self.secret {
            self.composing.chars().count() * '•'.len_utf8()
        } else {
            self.composing.len()
        };
        Some((d_start, d_start + d_len))
    }
}

/// A pointer that went down on an element hearing `onPinch`, `onRotate`
/// or `onSwipe`.
#[derive(Clone, Copy)]
struct Gesturing {
    e: u32,
    from: (f32, f32),
    tracker: nsh::VelocityTracker,
    /// It is one of a pinch's fingers: its release swipes nothing.
    pinched: bool,
}

/// Two fingers pinching and turning an element: NSH's recogniser, the
/// pointers' ids, and the last scale and turn reported.
#[derive(Clone, Copy)]
struct Pinching {
    e: u32,
    a: u32,
    b: u32,
    p: nsh::Pinch,
    last: (f32, f32),
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

mod drag;
mod edit;
mod focus;
mod pointer;
mod scroll;
mod zones;
use drag::Drag;
pub(in crate::tree) use edit::caret_bar;

impl Tree {
    /// `e` (arena node `node`) is freed and its slot may be reused: scrub
    /// every input reference to `e` or `node` and reset `per[e]`.
    pub(super) fn forget_el(&mut self, e: u32, node: Id) {
        self.forget_drag(e);
        let inp = &mut self.input;
        inp.hover.retain(|&x| x != e);
        inp.next.retain(|&x| x != e);
        if inp.held.as_ref().is_some_and(|h| h.e == e) {
            inp.held = None;
        }
        if inp.capture == e {
            inp.capture = NONE;
        }
        if inp.first.is_some_and(|f| f.0 == e) {
            inp.first = None;
        }
        if inp.field_press.is_some_and(|f| f.0 == e) {
            inp.field_press = None;
        }
        if inp.selecting == Some(e) {
            inp.selecting = None;
        }
        if inp.caret_scrolled == Some(e) {
            inp.caret_scrolled = None;
        }
        if inp.focus == e {
            inp.focus = NONE;
        }
        inp.modals.retain(|&(m, _)| m != e);
        for (_, ret) in &mut inp.modals {
            if *ret == e {
                *ret = NONE;
            }
        }
        inp.modal_els.retain(|&m| m != e);
        inp.stops.retain(|&s| s != e);
        if inp.latch.is_some_and(|l| l.0 == e) {
            inp.latch = None;
        }
        if inp.fling.is_some_and(|f| f.0 == e) {
            inp.fling = None;
        }
        if let Some(f) = &mut inp.finger
            && matches!(f.mode, Touch::Pan { e: pe, .. } if pe == e)
        {
            f.mode = Touch::Drag;
        }
        if inp.holding.as_ref().is_some_and(|h| h.e == e) {
            inp.holding = None;
        }
        inp.autofocus.retain(|&a| a != e);
        inp.zone_els.retain(|&z| z != e);
        inp.zone_from.retain(|&z| z != e);
        inp.zone_was.retain(|&z| z != e);
        for g in &mut inp.gestures {
            if g.as_ref().is_some_and(|g| g.e == e) {
                *g = None;
            }
        }
        if inp.pinch.as_ref().is_some_and(|p| p.e == e) {
            inp.pinch = None;
        }
        inp.thumbs.retain(|&t| t != e);
        if inp.grab.is_some_and(|g| g.0 == e) {
            inp.grab = None;
        }
        inp.scrolled.retain(|&s| s != e);
        inp.pinned.retain(|&p| p != node);
        let mut p = self.els[e as usize].parent;
        while p != NONE {
            if let Some(per) = inp.per.get_mut(p as usize)
                && per.last == Some(e)
            {
                per.last = None;
            }
            p = self.els[p as usize].parent;
        }
        if let Some(per) = inp.per.get_mut(e as usize) {
            *per = Per::default();
            per.tab = NO_TAB;
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

    /// The pack's haptics by scope and event (the host's).
    pub fn set_haptics(&mut self, haptics: std::rc::Rc<HapticTable>) {
        self.input.haptics_table = haptics;
    }

    /// The haptic patterns this frame's UI interactions played, oldest first.
    pub fn take_haptics(&mut self) -> Vec<og_ui_model::Haptic> {
        std::mem::take(&mut self.input.haptics)
    }

    /// `e` plays `event`: its slot's sound (with its variant, then
    /// without), else the pack's generic one; an event no scope has falls
    /// back along [`crate::pack::sound_fallback`] (`purchase` to
    /// `confirm`, then `press`). Also emits its haptic pattern (explicit
    /// `haptic` prop, else pack `haptics` by scope, else
    /// [`og_ui_model::event_haptic`], skipping `hover` and `tick` defaults).
    pub(super) fn sound(&mut self, e: u32, event: &str) {
        let per = self.input.per.get(e as usize);
        let slot = per.map_or("", |p| &*p.slot);
        let bare = slot
            .rsplit_once('.')
            .map(|x| x.0)
            .filter(|b| b.contains('.'));
        let scopes = [Some(slot).filter(|s| !s.is_empty()), bare, Some("")];
        if !self.input.sounds.is_empty() {
            let table = &self.input.sounds;
            let cue = std::iter::successors(Some(event), |ev| crate::pack::sound_fallback(ev))
                .find_map(|ev| scopes.iter().flatten().find_map(|s| table.get(*s)?.get(ev)));
            if let Some(play) = cue.and_then(|c| c.pick(&mut self.input.sound_rng, None)) {
                self.input.plays.push(play);
            }
        }
        let override_haptic = if event != "hover" {
            per.map(|p| &*p.haptic)
                .filter(|h| !h.is_empty())
                .and_then(crate::pack::parse_haptic)
        } else {
            None
        };
        let haptic = override_haptic.unwrap_or_else(|| {
            let h_table = &self.input.haptics_table;
            let from_pack = if h_table.is_empty() {
                None
            } else {
                std::iter::successors(Some(event), |ev| {
                    (*ev != "tick")
                        .then(|| crate::pack::sound_fallback(ev))
                        .flatten()
                })
                .find_map(|ev| {
                    scopes
                        .iter()
                        .flatten()
                        .find_map(|s| h_table.get(*s)?.get(ev).copied())
                })
            };
            from_pack.unwrap_or_else(|| {
                if matches!(event, "hover" | "tick") {
                    og_ui_model::Haptic::NONE
                } else {
                    std::iter::successors(Some(event), |ev| crate::pack::sound_fallback(ev))
                        .map(og_ui_model::event_haptic)
                        .find(|h| h.count > 0)
                        .unwrap_or(og_ui_model::Haptic::NONE)
                }
            })
        });
        if haptic.count > 0 && self.input.haptics.len() < 16 {
            self.input.haptics.push(haptic);
        }
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
    /// modal while one is open, or a `spotlight` scrim targeting that modal.
    pub(super) fn reachable(&self, e: u32) -> bool {
        self.input.modals.last().is_none_or(|&(m, _)| {
            self.within(e, m)
                || self
                    .input
                    .per(e)
                    .is_some_and(|p| !p.spotlight.is_empty() && self.named(&p.spotlight) == Some(m))
        })
    }

    /// Whether the last active pointer device was a touchscreen (`pointer-coarse:`).
    pub fn is_touch(&self) -> bool {
        self.input.touch
    }

    /// Plays `exitSound` cues declared on `top` or any live descendant when
    /// a subtree is removed/unmounted (e.g. `"close"` on overlays).
    pub(super) fn play_exit_sounds(&mut self, top: u32) {
        let mut stack = vec![top];
        while let Some(e) = stack.pop() {
            let el = &self.els[e as usize];
            if el.dead {
                continue;
            }
            stack.extend_from_slice(&el.children);
            if let Some(p) = self.input.per.get_mut(e as usize)
                && !p.exit_cue.is_empty()
            {
                let ev = std::mem::take(&mut p.exit_cue);
                self.sound(e, &ev);
            }
        }
    }

    fn disabled(&self, e: u32) -> bool {
        self.els[e as usize].disabled
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
        // A group's descendants read its states (`group-checked:`).
        if bit & super::ops::GROUP_STATES != 0
            && self
                .classes
                .groups(self.els[e as usize].class)
                .next()
                .is_some()
        {
            self.data_dirty = true;
        }
        // Focus-visible restyles every element, and a press every control:
        // the ring and the squash are the pack's when its own classes do
        // not read them (`restyle`).
        if bit == FOCUS_VISIBLE
            || (bit == PRESSED && self.presses(e))
            || self.classes.tracked(self.els[e as usize].class) & bit != 0
        {
            self.restyle(e);
        }
    }

    /// The device the player used last: a touch screen (`touch`: a drag
    /// pans what scrolls, a still finger long-presses) or a pad (`pad`:
    /// focus a node takes by itself shows its ring).
    pub fn set_device(&mut self, touch: bool, pad: bool) {
        if touch && !self.input.touch {
            // What the mouse lit goes out: a finger hovers nothing.
            for e in std::mem::take(&mut self.input.hover) {
                self.set_state(e, HOVERED, false);
            }
        }
        self.input.touch = touch;
        self.input.pad = pad;
    }

    /// The cursor the element under the pointer names (`cursor-*`, its
    /// own or its nearest ancestor's), as of the last pointer frame;
    /// `Auto` when none does.
    pub fn cursor(&self) -> nsh_tailwind::effects::Cursor {
        self.input.cursor
    }

    /// The actions the `<Glyph>` nodes show, when one came or changed
    /// since the last [`Tree::set_glyphs`].
    pub fn glyphs_wanted(&self) -> bool {
        self.input.glyphs_wanted
    }

    /// Writes each `<Glyph action>`: the action's binding on the current
    /// device, as `glyph` answers it: its text, and the texture slot of
    /// the pack's art for it (0 none). Art shows in the text's place, an
    /// ink mask one em square (the text leaf is empty and holds that
    /// square, whatever the font);
    /// without art the text shows; with neither, nothing. The host calls
    /// it when the device, the bindings or the pack's art change, or
    /// [`Tree::glyphs_wanted`].
    pub fn set_glyphs(&mut self, glyph: &dyn Fn(&str) -> Option<(String, u32)>) {
        self.input.glyphs_wanted = false;
        for e in 0..self.els.len() as u32 {
            let Some(p) = self.input.per(e) else { continue };
            if p.glyph.is_empty() || self.els[e as usize].dead {
                continue;
            }
            let (text, art) = glyph(&p.glyph).unwrap_or_default();
            let text = if art != 0 { "" } else { text.as_str() };
            if art != p.art {
                self.input.per[e as usize].art = art;
                self.paint = true;
                // Its leaf holds the art's em square ([`Tree::inherit`]).
                self.inherit(e);
            }
            if text != self.els[e as usize].text {
                self.set_text(e, text);
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
