//! The immediate-mode context: build a tree every tick, read responses back.

mod build;
mod commands;
mod configure;
mod controls;
mod descriptors;
mod dressing;
mod finish;
mod node_draw;
mod snapshot;
mod text_controls;

pub(crate) use commands::bake;
pub use descriptors::*;
use dressing::scaled_xy;
use node_draw::part_rect;

use crate::draw::{DrawCmd, DrawItem, EmitDesc, assign_levels, emit};
use crate::element::{Affine, LayoutStats, Overlap};
use crate::id::{Id, child_key};
use crate::input::{InputFrame, Nav};
use crate::layout::{
    Align, Anchor, Content, Cross, Direction, Element, Float, FontRole, Insets, Measure, Metrics,
    NONE, Node, Rect, Shape, Size, TextAlign, TextContent, Widget, layout, preorder,
};
use crate::motion::Fx;
use crate::motion::{Timeline, UiEvent};
use crate::style::{Breakpoint, Density, Srgba, Theme, TypeStep};
use crate::style_file::{Ink, Ornament, Slice, SliceFill, StylePart};
use crate::text::{FontMetrics, UiFont};
use og_core::IdMap;
use og_core::glam::Vec2;
use og_probe::UiElement;
use og_probe::hooks::{find_by_id, find_by_text};
use og_render::{Overlay, TextureHandle};

/// What happened to an element this tick. Edges (`clicked`, `changed`,
/// `submitted`) are true for exactly one tick.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Response {
    /// The pointer is over it (and no other element holds the press).
    pub hovered: bool,
    /// The press that started on it is still held.
    pub pressed: bool,
    /// Pressed and released inside it.
    pub clicked: bool,
    /// Its value changed (slider, toggle, text field).
    pub changed: bool,
    /// It has focus: a text field being edited, or the control keyboard
    /// and gamepad navigation landed on.
    pub focused: bool,
    /// Enter was pressed in it (text field).
    pub submitted: bool,
    /// A list row or grid cell a press started on was released over this
    /// one: the index of that row or cell (drag and drop).
    pub dropped: Option<u32>,
    /// The gamepad whose A pressed it this tick (0 to 3); `None` when the
    /// pointer or the keyboard did, or nothing.
    pub pad: Option<u32>,
    /// Bit `k`: pad `k`'s focus is on it (pad 0's is `focused`; pads 1 to 3
    /// while [`Ui::set_players`] allows them).
    pub focused_by: u32,
    /// An adjustable element's Left and Right this tick while it has the
    /// focus: +1 for each press of Right, -1 for Left (the screen's
    /// directions, not the reading order); 0 for any other element.
    pub adjust: i32,
}

pub(crate) const HIT_SCROLL: u16 = 1;
/// A press focuses it with a caret (text field).
pub(crate) const HIT_TEXT: u16 = 2;
/// Keyboard and gamepad focus can land on it.
pub(crate) const HIT_NAV: u16 = 4;
/// Left and right adjust it when focused.
pub(crate) const HIT_ADJUST: u16 = 8;
/// Outside its scroll container's view: navigation only, never the pointer.
pub(crate) const HIT_OFFSCREEN: u16 = 16;
/// Pointer-interactive (buttons, backdrops, controls).
pub(crate) const HIT_POINTER: u16 = 32;
/// A focus scope (dialog backdrop, open menu).
pub(crate) const HIT_SCOPE: u16 = 64;
/// ↑ and ↓ are its own while it is focused (a multi-line text control
/// edits with them, a zoom view pans).
pub(crate) const HIT_MULTILINE: u16 = 128;
/// It takes drops of dragged elements (`Props::drop`).
pub(crate) const HIT_DROP: u16 = 256;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Hit {
    pub key: u64,
    /// What is on screen (cut to the clip): the bounds of its transformed box.
    pub rect: Rect,
    /// The whole element, for navigation and scrolling into view.
    pub full: Rect,
    pub flags: u16,
    /// The nearest focus scope around it (0: none).
    pub scope: u64,
    /// The scroll container it sits in (0: none).
    pub scroller: u64,
    /// A transformed element: the map from the screen to its layout box,
    /// and that box (with its hit slop), tested after the clip.
    pub inv: Affine,
    pub local: Rect,
    pub clip: Rect,
}

impl Hit {
    pub fn has(&self, flag: u16) -> bool {
        self.flags & flag != 0
    }

    /// The point is on it: inside its clip and, mapped back through its
    /// transform, inside its box (with its hit slop).
    pub fn contains(&self, x: f32, y: f32) -> bool {
        if self.inv.is_identity() {
            return self.rect.contains(x, y);
        }
        if !self.clip.contains(x, y) {
            return false;
        }
        let (lx, ly) = self.inv.apply(x, y);
        self.local.contains(lx, ly)
    }
}

/// Per-widget memory between ticks: what each widget's animation or open
/// state needs (`a`, `b` values, `t` a tick, `h` a hash or flags).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Anim {
    pub a: f32,
    pub b: f32,
    pub t: u64,
    pub h: u64,
}

/// A queued notification.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Toast {
    pub id: u32,
    pub text: String,
    pub tone: u8,
    /// The tick it was queued on.
    pub born: u64,
    /// Ticks it stays before leaving.
    pub ticks: u32,
    /// The tick it started leaving (0: not leaving).
    pub leaving: u64,
}

/// What persists between ticks (and in [`Ui::save_state`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct State {
    pub hot: u64,
    pub active: u64,
    pub focus: u64,
    pub caret: usize,
    /// The selection's other end (equal to `caret`: no selection).
    pub anchor: usize,
    /// The text control the caret belongs to (0: none yet).
    pub edit_key: u64,
    /// The tick of the last edit or caret move: the blink's phase.
    pub edit_tick: u64,
    /// An input method's composition in the focused text control.
    pub composition: String,
    /// The focused text control's scroll: a field's across, an area's down.
    pub edit_scroll: f32,
    /// The caret is drawn at the trailing edge of the glyph before it, not
    /// the leading edge of the glyph after: one of the two places a caret
    /// at a bidi run's edge has, where ← or → put it on screen.
    pub caret_upstream: bool,
    /// The emacs kill buffer (Control+K, U and W under mac keys put what
    /// they delete in it, Control+Y yanks it), shared by every text control.
    pub kill: String,
    /// The last edit was a kill: the next kill adds to the buffer (after it
    /// for K, before it for U and W) instead of replacing it.
    pub killing: bool,
    /// The world has had a typed-text stream (`og.TextInput`): key presses
    /// type nothing from then on.
    pub typed_stream: bool,
    /// The presses counted into double and triple clicks in text controls.
    pub click: crate::edit::Click,
    /// Each text control's undo history, by key.
    pub histories: IdMap<u64, crate::edit::History>,
    pub hover_ticks: u32,
    pub pointer: Vec2,
    /// The press that focused a field: where it landed, to place the caret.
    pub focus_click: Option<f32>,
    pub scroll: IdMap<u64, f32>,
    /// Horizontal scroll offsets.
    pub scroll_x: IdMap<u64, f32>,
    /// Each scroll container's ranges and scrollbar lengths, last frame.
    pub scroll_info: IdMap<u64, crate::scroll::ScrollInfo>,
    /// A released drag's momentum: the container and its velocity per tick.
    pub fling: Option<(u64, Vec2)>,
    /// Two touches held at once (og.Pointers): the scroll container or
    /// zoom view they move, and their centre and spread last tick.
    pub pinch: Option<crate::scroll::Pinch>,
    /// The drag's velocity so far, CSS px per tick.
    pub drag_v: Vec2,
    /// A scrollbar the press grabbed.
    pub thumb: Option<crate::scroll::Thumb>,
    /// Last frame's interactive rectangles in draw order, for hit testing.
    pub hits: Vec<Hit>,
    /// Frames begun: the clock of every animation.
    pub tick: u64,
    /// Focus moved by keyboard or gamepad: the ring shows.
    pub focus_visible: bool,
    /// The topmost focus scope of the last frame (0: none).
    pub scope: u64,
    /// That scope is a locked dialog: Back does not close it, so
    /// `Ui::back` answers it.
    pub scope_locked: bool,
    /// Where focus returns as scopes close, innermost last: each entry is
    /// the scope (0: none) a scope opened over and the focus it had there,
    /// so a submenu closing gives the focus back to its row and the menu
    /// closing then to what opened it.
    pub focus_returns: Vec<(u64, u64)>,
    /// Keyboard and gamepad navigation as the last frame declared it.
    pub nav_off: bool,
    /// The stick's held direction (0 none, 1 + Nav index) and since which tick.
    pub stick: (u8, u64),
    /// Intents queued by `Ui::navigate` for the next frame, each with
    /// whether the focus was on nothing when it was queued (then it only
    /// puts the focus on the first control: `Ui::navigate`).
    pub pending: Vec<(Nav, bool)>,
    pub anims: IdMap<u64, Anim>,
    pub toasts: Vec<Toast>,
    pub next_toast: u32,
    /// Screen transitions by key.
    pub screens: IdMap<u64, crate::juice::ScreenState>,
    /// The host's voice entity for each UI event's sound (bits; 0: none),
    /// by [`crate::tokens::SoundEvent`] index.
    pub voices: [u64; crate::tokens::SoundEvent::COUNT],
    /// The feedback sets' random stream (a xorshift): which sound of a set,
    /// and where in its volume and pitch ranges, a play takes.
    pub feedback_rng: u64,
    /// Per event, the sound of its set it played last, so a set of more
    /// than one never plays the same sound twice in a row.
    pub feedback_last: [u8; crate::tokens::SoundEvent::COUNT],
    /// Players the last frame allowed (`set_players`): pads 1 to players - 1
    /// each move a focus of their own.
    pub players: u8,
    /// Pads 1 to 3: the control each one's focus is on (0: none).
    pub pad_focus: [u64; 3],
    /// Pads 1 to 3: the stick's held direction and since which tick.
    pub pad_stick: [(u8, u64); 3],
    /// Motion in flight: channels, states, plays, ghosts, gestures.
    pub motion: crate::motion::state::MotionState,
}

/// What navigation decided at the start of this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct NavFrame {
    /// The control pressed by Enter, Space or gamepad A.
    pub activate: u64,
    /// Escape or gamepad B, not yet taken.
    pub back: bool,
    /// Left (-1) and right (+1) on an adjustable control.
    pub adjust: i32,
    /// `activate` came from pad 0's A.
    pub pad0: bool,
    /// The control pads 1 to 3 pressed with A (0: none).
    pub pad_activate: [u64; 3],
    /// A stepper a screen reader adjusted as a whole (its spinbutton's
    /// key) and by how many steps.
    pub spin: (u64, i32),
}

/// The immediate-mode UI. Call [`Ui::begin`], build, [`Ui::end`] once per tick.
#[derive(Clone, Debug)]
pub struct Ui {
    /// The theme as configured ([`Ui::set_theme`]); `theme` is it with the
    /// viewport's preferences applied.
    pub(crate) configured: Theme,
    pub(crate) theme: Theme,
    pub(crate) fonts: Fonts,
    pub(crate) ticks_per_second: u32,
    pub(crate) viewport: Viewport,
    pub(crate) input: InputFrame,
    pub(crate) nodes: Vec<Node>,
    pub(crate) kinds: Vec<(&'static str, bool)>,
    /// Last frame's `kinds`, beside `kept.prev`.
    prev_kinds: Vec<(&'static str, bool)>,
    pub(crate) stack: Vec<usize>,
    pub(crate) floating: Vec<usize>,
    pub(crate) last_closed: usize,
    pub(crate) warnings: Vec<String>,
    pub(crate) building: bool,
    pub(crate) state: State,
    pub(crate) frame: UiFrame,
    pub(crate) nav: NavFrame,
    /// Widget memories used this frame (others are dropped at `end`).
    pub(crate) touched: std::collections::BTreeSet<u64>,
    /// Navigation declared for this frame (`set_navigation`).
    pub(crate) nav_on: bool,
    /// Players declared for this frame (`set_players`).
    pub(crate) players: u32,
    /// The control a widget asked to focus this frame.
    pub(crate) request_focus: u64,
    /// Open focus scopes this frame, in build order.
    pub(crate) scopes: Vec<usize>,
    pub(crate) icons: IconAtlas,
    pub(crate) toast_anchor: Anchor,
    /// The widest the HUD band is, over its height ([`Ui::hud_area`]).
    pub(crate) hud_max_aspect: f32,
    /// UI events the style gives a sound or a haptic, not yet taken by the
    /// host ([`Ui::take_feedback`]), each with where it is in a rising
    /// cascade (0–1; NaN: a random pitch in the event's range).
    pub(crate) feedback: Vec<(crate::tokens::SoundEvent, f32)>,
    /// The player's haptic strength (0: off, 1: as the style sets it).
    pub(crate) haptics: f32,
    /// The haptic patterns played since the host last took them
    /// ([`Ui::take_haptics`]).
    pub(crate) haptic_out: Vec<crate::tokens::Haptic>,
    /// The style's part dressings ([`Ui::set_dressings`]); without them, the
    /// ones its theme's version 1 art makes.
    pub(crate) dress: std::sync::Arc<crate::dress::DressSet>,
    pub(crate) dress_set: bool,
    /// What each part of this frame draws through the UI pass, by node
    /// (made at `end` from its dressing, or the tokens).
    pub(crate) part_draws: std::collections::HashMap<usize, crate::paint::ElementDraw>,
    /// Text a copy or cut took, for the host's clipboard ([`Ui::take_clipboard`]).
    pub(crate) clipboard: Option<String>,
    /// The focused text control this frame (into [`UiFrame::text_input`]).
    pub(crate) text_target: Option<TextTarget>,
    /// This frame's screen-transition cover.
    pub(crate) cover: Option<crate::juice::Cover>,
    /// The configured style's name (empty: the default style).
    pub(crate) style_name: String,
    /// Two touches move a scroller or zoom view this tick: the primary
    /// pointer presses nothing.
    pub(crate) gesture: bool,
    /// The writing direction: right to left mirrors the layout and turns
    /// text alignment round (og.i18n's locale, or og.ui.configure).
    pub(crate) direction: og_i18n::Direction,
    /// Text measurements kept from the frames before (layout's).
    pub(crate) texts: crate::layout::TextCache,
    /// Timelines by name hash (`define_timeline`): configuration, like the theme.
    pub(crate) timelines: std::collections::BTreeMap<u64, (String, Timeline)>,
    /// Plays asked for this tick, flattened at `end`.
    pub(crate) pending_plays: Vec<crate::motion::run::PendingPlay>,
    /// This frame's motion events (into `UiFrame::events`).
    pub(crate) events: Vec<UiEvent>,
    /// Last frame's drop targets and dragged elements: their paths, and the
    /// tags each target takes.
    pub(crate) drop_paths: IdMap<u64, String>,
    pub(crate) drag_paths: IdMap<u64, String>,
    pub(crate) drop_accepts: IdMap<u64, Vec<String>>,
    /// Last frame's input hash of each element and its subtree, with its
    /// layout, for reuse (docs/ui.md §6.3).
    pub(crate) kept: crate::layout::LayoutCache,
    /// This frame's counts (into `UiFrame::stats`).
    pub(crate) stats: LayoutStats,
    /// The map of the element whose commands are being made: a nine-slice
    /// is cut at its size on the screen, its corners kept crisp.
    drawing: Affine,
    /// What elements of this frame draw with ([`Ui::set_draw`]), by node.
    pub(crate) draws: std::collections::HashMap<usize, crate::paint::ElementDraw>,
    /// The draws of [`Ui::set_draw_dressable`]: a component's own look for
    /// a part, which a style that dresses the part restyles.
    pub(crate) dressable: std::collections::HashSet<usize>,
    /// Documents, their mounts and the layer stack (docs/ui.md §8.1, §10).
    pub(crate) docs: crate::doc::build::DocHost,
    /// Named UI actions and the last device (docs/ui.md §8.2).
    pub(crate) actions: crate::actions::Actions,
    /// World-space UI (`crate::world`): anchors, panels, mass elements,
    /// particles, meta layers.
    pub(crate) world: crate::world::WorldUi,
}

/// One UI event's feedback as the host plays it ([`Ui::take_feedback`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FeedbackPlay {
    pub event: crate::tokens::SoundEvent,
    /// The sound handle (0: the event has a haptic only, or its sound is
    /// not loaded).
    pub sound: u64,
    /// Linear gain, from the set's volume range.
    pub volume: f32,
    /// Playback rate, from the set's pitch range.
    pub pitch: f32,
    /// The pattern at the player's strength (no pulses: none).
    pub haptic: crate::tokens::Haptic,
}

impl Default for Ui {
    fn default() -> Ui {
        Ui::new(UiDesc::default())
    }
}

const STATE_VERSION: u32 = 12;

/// `theme` with a viewport's preferences: the page's reduced motion turns
/// its flag on, its contrast preference takes the high-contrast variant,
/// and reduced transparency (the page's or the game's) the flat variant
/// (a game's own settings can only add to what the page asks); the type
/// ramp takes the player's text scale.
fn effective_theme(theme: Theme, viewport: &Viewport) -> Theme {
    let mut t = theme;
    if viewport.high_contrast && t.contrast != crate::style::Contrast::High {
        t = t.to_high_contrast();
    }
    t.reduced_motion |= viewport.reduced_motion;
    if t.reduced_transparency || viewport.reduced_transparency {
        t = t.to_reduced_transparency();
    }
    t.to_text_scale()
}

/// A style's `textGlow` size is how far the old four offset copies reached;
/// the distance-field glow fades to nothing over this many times it.
const TEXT_GLOW_REACH: f32 = 2.0;

/// `a` moved `t` of the way to `b` in sRGB, alpha included.
/// `a` towards `b` by `t`, premultiplied (a translucent white mixing into an
/// opaque dark passes through greys, not a bright half-opaque white), in
/// sRGB bytes as a CSS transition mixes.
pub(crate) fn mix_srgba(a: Srgba, b: Srgba, t: f32) -> Srgba {
    let t = t.clamp(0.0, 1.0);
    if t <= 0.0 {
        return a;
    }
    if t >= 1.0 {
        return b;
    }
    let alpha = a.alpha + (b.alpha - a.alpha) * t;
    if alpha <= 0.0 {
        return Srgba { alpha: 0.0, ..a };
    }
    let ch = |k: usize| {
        let pa = f32::from(a.rgb[k]) * a.alpha;
        let pb = f32::from(b.rgb[k]) * b.alpha;
        ((pa + (pb - pa) * t) / alpha).round().clamp(0.0, 255.0) as u8
    };
    Srgba {
        rgb: [ch(0), ch(1), ch(2)],
        alpha,
    }
}

/// An anchored group's place moved out of the host's control `c`: along
/// the edge it sits on, past `c` by `gap` (a corner or side group sideways,
/// a top or bottom centred group up or down), or for a corner group past it
/// vertically when that lands on less of the `others` and of the surface
/// outside `bounds`; a centred group stays.
fn keep_out(r: Rect, anchor: Anchor, c: Rect, gap: f32, bounds: Rect, others: &[Rect]) -> Rect {
    if c.is_empty() || !r.overlaps(&c) {
        return r;
    }
    let (ax, ay) = anchor.fractions();
    let along = if ax < 0.5 {
        Rect {
            x: r.x.max(c.right() + gap),
            ..r
        }
    } else if ax > 0.5 {
        Rect {
            x: r.x.min(c.x - gap - r.width),
            ..r
        }
    } else if ay < 0.5 {
        Rect {
            y: r.y.max(c.bottom() + gap),
            ..r
        }
    } else if ay > 0.5 {
        Rect {
            y: r.y.min(c.y - gap - r.height),
            ..r
        }
    } else {
        return r;
    };
    // A corner group may also step past it the other way (under a top-left
    // control, down): taken when moving along the edge would land on more
    // of the other groups, or off the safe area, than that does.
    let across = if ax == 0.5 || ay == 0.5 {
        None
    } else if ay < 0.5 {
        Some(Rect {
            y: r.y.max(c.bottom() + gap),
            ..r
        })
    } else {
        Some(Rect {
            y: r.y.min(c.y - gap - r.height),
            ..r
        })
    };
    let area = |a: Rect| a.width * a.height;
    let cost = |p: Rect| {
        let off = area(p) - area(p.intersect(&bounds));
        off + others.iter().map(|o| area(p.intersect(o))).sum::<f32>()
    };
    match across {
        Some(a) if cost(a) < cost(along) => a,
        _ => along,
    }
}
