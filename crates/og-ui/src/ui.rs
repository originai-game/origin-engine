//! The immediate-mode context: build a tree every tick, read responses back.

#[cfg(debug_assertions)]
pub(crate) mod bounded;
mod build;
mod commands;
mod configure;
mod controls;
mod css_tokens;
mod debug;
mod descriptors;
mod dressing;
mod finish;
mod inspect;
mod node_draw;
pub(crate) use node_draw::{VerticalLine, VerticalRun, vertical_run};
pub(crate) mod resize;
pub(crate) mod select;
pub(crate) mod settlement;
mod snapshot;
mod text_controls;
mod tree;
pub(crate) mod visit;

pub(crate) use commands::{bake, fade_cmd};
pub(crate) use debug::DebugBindings;
pub use debug::{UiDebugElement, UiDebugStyle, debug_overlay};
pub use descriptors::*;
use node_draw::part_rect;
pub(crate) use text_controls::FieldInk;

use crate::draw::{DrawCmd, DrawItem, EmitDesc, emit};
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
use crate::style_file::{Ink, Ornament, StylePart};
use crate::text::{FontMetrics, UiFont};
use og_core::IdMap;
use og_core::glam::Vec2;
use og_draw::{Overlay, TextureHandle};
use og_probe::UiElement;
use og_probe::hooks::{find_by_id, find_by_text};

/// What happened to an element this tick. Edges (`clicked`, `changed`,
/// `submitted`) are true for exactly one tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
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
    /// The player whose split screen region it is in, on the frame it was
    /// pressed or clicked (`Props::player`); `None` outside every region.
    pub player: Option<u32>,
    /// Bit `k`: pad `k`'s focus is on it (pad 0's is `focused`; pads 1 to 3
    /// while [`Ui::set_players`] allows them).
    pub focused_by: u32,
    /// An adjustable element's Left and Right this tick while it has the
    /// focus: +1 for each press of Right, -1 for Left (the screen's
    /// directions, not the reading order); 0 for any other element.
    pub adjust: i32,
    /// An element claiming Up and Down ([`crate::Element::arrows`]): -1 for
    /// each press of Up, +1 for Down this tick while it has the focus.
    pub adjust_y: i32,
    /// The focus ring is up on it: the focus is on it and keys or a pad
    /// put it there. With [`crate::Element::own_ring`] the component draws
    /// that ring (and what keys show, a cursor's card lifted) itself.
    pub ring: bool,
    /// Its tap was recognised this tick ([`crate::GestureSpec`]): a press
    /// and release within the slop, and where it declares a double tap,
    /// after that double tap's window passed without a second one. Keys
    /// and pads tap it by activating it.
    pub tapped: bool,
    /// The second tap of a double tap landed this tick.
    pub double_tapped: bool,
    /// Its long press fired this tick (the press it took clicks nothing).
    pub long_pressed: bool,
    /// How far into its long press the held press is, 0 to 1 (0: none).
    pub long_press: f32,
}

/// [`crate::Element::arrows`]: Left is its own while it has the focus.
pub const ARROW_LEFT: u8 = 1;
/// Right is its own.
pub const ARROW_RIGHT: u8 = 2;
/// Up is its own.
pub const ARROW_UP: u8 = 4;
/// Down is its own.
pub const ARROW_DOWN: u8 = 8;

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
/// Its text can be selected and copied (`user-select: text | all`).
pub(crate) const HIT_SELECT: u16 = 1 << 13;
/// CSS `pointer-events: none`: the pointer passes through it (it is
/// still reached by keys and pads).
pub(crate) const HIT_PASS: u16 = 1 << 14;
/// An SVG shape whose `pointer-events` takes only its painted geometry
/// (its fill, its stroke), tested by [`Ui::shape_hit`].
pub(crate) const HIT_SHAPE: u16 = 1 << 15;
/// The arrows it claims while focused: [`ARROW_LEFT`] … shifted by 9.
pub(crate) const HIT_CLAIM_SHIFT: u16 = 9;

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
    /// The nearest focusable element around it (0: none): a press on a
    /// control that takes no focus of its own (an item in an inventory's
    /// grid) focuses that one, as a click inside a focusable box does on
    /// the web.
    pub owner: u64,
    /// A transformed element: the map from the screen to its layout box,
    /// and that box (with its hit slop), tested after the clip.
    pub inv: Affine,
    pub local: Rect,
    pub clip: Rect,
    /// The nearest focus group around it (0: none).
    pub group: u64,
    /// The nearest element at or around it that recognises gestures (0: none).
    pub gesture: u64,
    /// The split screen region it is in: player `k`'s is `k + 1` (0: none).
    pub player: u8,
}

impl Hit {
    pub fn has(&self, flag: u16) -> bool {
        self.flags & flag != 0
    }

    /// The point is on it: inside its clip and, mapped back through its
    /// transform, inside its box (with its hit slop).
    pub fn contains(&self, x: f32, y: f32) -> bool {
        if self.has(HIT_PASS) {
            return false;
        }
        if self.inv.is_identity() {
            return self.rect.contains(x, y);
        }
        if !self.clip.contains(x, y) {
            return false;
        }
        let (lx, ly) = self.inv.apply(x, y);
        self.local.contains(lx, ly)
    }

    /// How far the point is from its own box (0 inside): a point in its
    /// reach but outside its box (hit slop, the touch minimum) is this far.
    pub fn distance(&self, x: f32, y: f32) -> f32 {
        let r = &self.full;
        let dx = (r.x - x).max(x - r.right()).max(0.0);
        let dy = (r.y - y).max(y - r.bottom()).max(0.0);
        dx.hypot(dy)
    }
}

/// The element a pointer at (`x`, `y`) is on among `hits` (drawing order,
/// topmost last) that `eligible` accepts: the topmost one whose box holds
/// the point, unless points it holds only by its reach (hit slop, the touch
/// minimum) lie over it, where the nearest box wins. A box around those
/// reaches (a dialog's backdrop, a panel, a scroller) never takes the
/// point from them; a sibling's own box does (docs/ui.md §2.1).
pub(crate) fn pick_hit(hits: &[Hit], x: f32, y: f32, eligible: impl Fn(&Hit) -> bool) -> u64 {
    let mut reached: Vec<(&Hit, f32)> = Vec::new();
    for h in hits
        .iter()
        .rev()
        .filter(|h| eligible(h) && h.contains(x, y))
    {
        let d = h.distance(x, y);
        if d > 0.0 {
            reached.push((h, d));
            continue;
        }
        let around = |s: &Hit| {
            let (a, b) = (&h.full, &s.full);
            a.x <= b.x && a.y <= b.y && a.right() >= b.right() && a.bottom() >= b.bottom()
        };
        if reached.is_empty() || !reached.iter().all(|(s, _)| around(s)) {
            return h.key;
        }
        break;
    }
    // The nearest box; the topmost of equals.
    reached
        .iter()
        .fold(None::<(&Hit, f32)>, |best, &(h, d)| match best {
            Some((_, bd)) if bd <= d => best,
            _ => Some((h, d)),
        })
        .map_or(0, |(h, _)| h.key)
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

/// A Rive artboard an element plays: its id, its player (`None` once
/// restored: rebuilt from `saved` on the element's next build, refused
/// bytes kept and reported, never reset), and the tick it was last built
/// (one not built for a tick is dropped).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RiveSlot {
    pub board: String,
    pub player: Option<crate::rive::Player>,
    pub saved: Vec<u8>,
    pub built: u64,
}

/// What persists between ticks (and in [`Ui::save_state`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct State {
    /// The view transition running (its shared elements' groups).
    pub view_transition: Option<crate::view_transition::Running>,
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
    /// The player's selection of read-only text (`user-select`), and the
    /// presses on such text.
    pub selection: Option<select::Selection>,
    pub select_press: select::SelectPress,
    /// Each text control's undo history, by key.
    pub histories: IdMap<u64, crate::edit::History>,
    pub hover_ticks: u32,
    pub pointer: Vec2,
    /// The press that focused a field: where it landed, to place the caret.
    pub focus_click: Option<f32>,
    pub scroll: IdMap<u64, f32>,
    pub virtual_lists: std::collections::BTreeMap<u64, crate::virtualize::Virtualizer>,
    /// Horizontal scroll offsets.
    pub scroll_x: IdMap<u64, f32>,
    /// Each scroll container's ranges and scrollbar lengths, last frame.
    pub scroll_info: IdMap<u64, crate::scroll::ScrollInfo>,
    /// A released drag's momentum: the container and its velocity per tick.
    pub fling: Option<(u64, Vec2)>,
    /// A snapping container the wheel moves and its offset before the
    /// wheel did, until it settles: `scroll-snap-stop: always` areas
    /// between there and where it settles stop it.
    pub wheel_from: Option<(u64, Vec2)>,
    /// Two touches held at once (og.Pointers): the scroll container or
    /// zoom view they move, and their centre and spread last tick.
    pub pinch: Option<crate::scroll::Pinch>,
    /// The drag's velocity so far, CSS px per tick.
    pub drag_v: Vec2,
    /// A scrollbar the press grabbed.
    pub thumb: Option<crate::scroll::Thumb>,
    /// A resize handle the press grabbed (CSS `resize`), and the sizes
    /// the user gave resizable elements, by key (CSS px, per axis).
    pub resizing: Option<resize::Resizing>,
    pub resized: crate::id::KeyMap<[Option<f32>; 2]>,
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
    /// The further touch (an og.Pointers id) each touch control holds, by
    /// key: the finger that went down on it, kept while it is down, so
    /// another landing on the control never takes it over.
    pub touch_owners: IdMap<u64, u32>,
    /// What was typed to jump in the focused list (lowercase) and the tick
    /// of its last letter: it starts again after a second's pause.
    pub typeahead: (String, u64),
    /// The Rive artboards elements play (`<og-rive src>`), by key.
    pub rive: IdMap<u64, RiveSlot>,
    /// Intents queued by `Ui::navigate` for the next frame, each with
    /// whether the focus was on nothing when it was queued (then it only
    /// puts the focus on the first control: `Ui::navigate`).
    pub pending: Vec<(Nav, bool)>,
    pub anims: IdMap<u64, Anim>,
    /// Clock key to (semantic owner key, first tick). Lifetime follows the
    /// owner, including ticks when a restored mount has no render node yet.
    pub clocks: std::collections::BTreeMap<u64, (u64, u64)>,
    pub toasts: Vec<Toast>,
    pub next_toast: u32,
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
    /// Last frame's focus groups by key (docs/ui.md §2.1).
    pub groups: IdMap<u64, crate::recognize::GroupState>,
    /// Last frame's explicit focus moves by control key: the key each of
    /// up, down, left, right, next and prev lands on (a control's or a
    /// group's; 0: the layout's).
    pub nav_links: IdMap<u64, [u64; 6]>,
    /// Last frame's gesture recognisers by key: each one's spec and the
    /// nearest recognising element around it (0: none).
    pub recognizers: IdMap<u64, (crate::element::GestureSpec, u64)>,
    /// The press being recognised and a tap waiting for its double.
    pub tap: crate::recognize::TapState,
    /// A pad's virtual cursor (docs/ui.md §2.1).
    pub vcursor: crate::vcursor::VCursorState,
    /// The players' HUD layouts and the edit mode (docs/ui.md §2.1).
    pub hud: crate::hud_layout::HudState,
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
    /// Up (-1) and down (+1) on a control that claims them.
    pub adjust_y: i32,
    /// `activate` came from pad 0's A.
    pub pad0: bool,
    /// The control pads 1 to 3 pressed with A (0: none).
    pub pad_activate: [u64; 3],
    /// A stepper a screen reader adjusted as a whole (its spinbutton's
    /// key) and by how many steps.
    pub spin: (u64, i32),
    /// The elements tapped, double tapped and long pressed this tick.
    pub tapped: u64,
    pub double_tapped: u64,
    pub long_pressed: u64,
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
    /// The og-ui unit of the frame being built, CSS px (`begin`).
    pub(crate) unit: f32,
    pub(crate) input: InputFrame,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) native_input: crate::native_a11y::Input,
    pub(crate) nodes: Vec<Node>,
    pub(crate) kinds: Vec<(&'static str, bool)>,
    /// Last frame's `kinds`, beside `kept.prev`.
    pub(crate) prev_kinds: std::collections::HashMap<usize, (&'static str, bool)>,
    render_slots: std::collections::HashMap<(u64, usize), usize>,
    free_slots: Vec<usize>,
    pub(crate) emitted: Vec<usize>,
    retiring: Vec<usize>,
    slot_present: Vec<bool>,
    /// Per slot: owned by a retained element, so it lives across frames
    /// until its owner retires it (`retire_owned`), not for one frame.
    pub(crate) persistent: Vec<bool>,
    /// The slots each retained element being emitted writes: its own node
    /// and its widget's internal nodes, innermost element last.
    pub(crate) owning: Vec<build::Owned>,
    /// A retained element re-emitted into the slot it already holds in its
    /// parent's children: `(key, parent)` of the push that must not append.
    pub(crate) in_place: Option<(u64, usize)>,
    /// Slots whose box placement replaced (floating sizes, a modal's safe
    /// padding): their authored box comes back before the next layout.
    pub(crate) layout_mutated: Vec<usize>,
    /// This frame's element keys under a parent, which a repeated id
    /// would take again (`Ui::push` renames it), and the anonymous
    /// elements of each kind under each parent: kept as built, so a
    /// long list pays no scan of its siblings for each element.
    built: crate::id::KeySet,
    anonymous: crate::id::KeyMap<u32>,
    pub(crate) stack: Vec<usize>,
    pub(crate) floating: Vec<usize>,
    pub(crate) last_closed: usize,
    pub(crate) warnings: Vec<String>,
    /// The (face, character) pairs [`UiFrame::missing_glyphs`] named.
    pub(crate) glyphs_missed: std::collections::BTreeSet<(og_draw::FontHandle, char)>,
    /// The trees' membership, keys and features (`tree`).
    pub(crate) tree: tree::TreeIndex,
    /// The frame's orders and each feature's nodes (`visit`).
    pub(crate) orders: visit::Orders,
    /// What may have changed for screen readers, by frame (`tree`).
    pub(crate) semantics: tree::SemanticLog,
    /// This frame's command ranges kept from last frame's paint, in order.
    pub(crate) kept_spans: Vec<std::ops::Range<usize>>,
    pub(crate) building: bool,
    pub(crate) state: State,
    pub(crate) frame: UiFrame,
    /// The last frame's inspection, made when asked for (`inspect`).
    pub(crate) inspection: inspect::Inspection,
    /// Last frame's layers and draws while this frame is built, for
    /// inspecting the last frame mid-build.
    pub(crate) prev_floating: Vec<usize>,
    pub(crate) prev_draws: std::collections::HashMap<usize, crate::paint::ElementDraw>,
    pub(crate) nav: NavFrame,
    /// Widget memories used this frame (others are dropped at `end`).
    pub(crate) touched: std::collections::BTreeSet<u64>,
    /// Navigation declared for this frame (`set_navigation`).
    pub(crate) nav_on: bool,
    /// Players declared for this frame (`set_players`).
    pub(crate) players: u32,
    /// The virtual cursor declared for this frame (`set_virtual_cursor`).
    pub(crate) vcursor: Option<crate::vcursor::VirtualCursorDesc>,
    /// The HUD layout editor's panel this frame (its key; 0: none).
    pub(crate) hud_editor_key: u64,
    /// The control a widget asked to focus this frame.
    pub(crate) request_focus: u64,
    /// The host opens a file picker for `<input type="file">`
    /// ([`Ui::set_file_picker`]).
    pub(crate) file_picker: bool,
    /// The content buttons open now and their ink (the innermost last).
    pub(crate) button_ink: Vec<(usize, Srgba)>,
    /// Open focus scopes this frame, in build order.
    pub(crate) scopes: Vec<usize>,
    pub(crate) icons: IconAtlas,
    /// The style's icon names, which its colour tokens' icon cues index
    /// (`Ui::set_cue_icons`); empty: the atlas's own names.
    pub(crate) cue_icons: Vec<String>,
    /// Text a control keeps between ticks while it is open (a dropdown's
    /// filter), by key: names of the frame's UI, dropped when it closes.
    pub(crate) kept_texts: crate::id::KeyMap<String>,
    /// Pictures rich text draws by name (`[img=…]`).
    pub(crate) images: std::collections::BTreeMap<String, InlineImage>,
    pub(crate) toast_anchor: Anchor,
    /// The widest the HUD band is, over its height ([`Ui::hud_area`]).
    pub(crate) hud_max_aspect: f32,
    /// UI events the style gives a sound or a haptic, not yet taken by the
    /// host ([`Ui::take_feedback`]), each with where it is in a rising
    /// cascade (0–1; NaN: a random pitch in the event's range).
    pub(crate) feedback: Vec<(crate::tokens::SoundEvent, f32, crate::tokens::Feedback)>,
    pub(crate) css_sounds: std::collections::BTreeMap<String, u64>,
    /// The player's haptic strength (0: off, 1: as the style sets it).
    pub(crate) haptics: f32,
    /// The haptic patterns played since the host last took them
    /// ([`Ui::take_haptics`]).
    pub(crate) haptic_out: Vec<crate::tokens::Haptic>,
    /// Global CSS part rules in cascade order and their at-rule descriptors.
    pub(crate) part_rules: Vec<crate::doc::StyleRule>,
    pub(crate) part_sheet: crate::doc::at_rules::Sheet,
    pub(crate) root_tokens: Vec<(String, String)>,
    pub(crate) root_type_tokens: [Vec<(String, String)>; 4],
    /// Part samples read one before-change state throughout a frame. Commit
    /// their next state together after rendering the owning nodes.
    pub(crate) part_commits: std::cell::RefCell<
        std::collections::BTreeMap<
            (u64, &'static str),
            std::sync::Arc<crate::doc::build::StyleCommit>,
        >,
    >,
    /// What each part of this frame draws through the UI pass, by node
    /// (made at `end` from its dressing, or the tokens).
    pub(crate) part_draws: std::collections::HashMap<usize, crate::paint::ElementDraw>,
    /// Parts whose computed paint changed since the preceding frame.
    pub(crate) part_changed: std::collections::HashSet<usize>,
    /// This frame's draws of elements with CSS box decorations: their
    /// draw or dressing with the decorations over it.
    pub(crate) decor_draws: std::collections::HashMap<usize, crate::paint::ElementDraw>,
    /// Text a copy or cut took, for the host's clipboard ([`Ui::take_clipboard`]).
    pub(crate) clipboard: Option<String>,
    /// The keys of the text elements last painted with a selection.
    pub(crate) selected: Vec<u64>,
    /// The focused text control this frame (into [`UiFrame::text_input`]).
    pub(crate) text_target: Option<TextTarget>,
    /// This frame's screen-transition cover.
    pub(crate) cover: Option<crate::juice::Cover>,
    /// The configured style's name (empty: the default style).
    pub(crate) style_name: String,
    /// Two touches move a scroller or zoom view this tick: the primary
    /// pointer presses nothing.
    pub(crate) gesture: bool,
    /// The element a resize drag moved this tick (written again).
    pub(crate) resize_moved: Option<u64>,
    /// The writing direction: right to left mirrors the layout and turns
    /// text alignment round (og.i18n's locale, or og.ui.configure).
    pub(crate) direction: og_i18n::Direction,
    /// Active BCP-47 locale, for display text casing.
    pub(crate) locale: String,
    /// CSS `accent-color` of the document control being built: its
    /// primary ink (`widgets::controls`' roles).
    pub(crate) accent: Option<crate::Srgba>,
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
    /// layout, for reuse (docs/ui.md §2).
    pub(crate) kept: crate::layout::LayoutCache,
    /// What this frame may keep from the last.
    pub(crate) invalidation: crate::layout::Invalidation,
    /// Last frame's paint, for reuse.
    pub(crate) paint_cache: crate::layout::PaintCache,
    /// The motion pass's memory between ticks.
    pub(crate) motion_cache: crate::layout::MotionCache,
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
    /// Documents, their mounts and the layer stack (docs/ui.md §2.1, §2.1).
    pub(crate) docs: crate::doc::build::DocHost,
    pub(crate) reactive: crate::reactive::session::Session,
    /// Named UI actions and the last device (docs/ui.md §2.1).
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

const STATE_VERSION: u32 = 23;

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
    t.ramp_scale = 1.0;
    t.to_text_scale()
}

/// The og-ui unit for a frame on the CSS `viewport`: the game's pin, else
/// the surface's ([`Theme::auto_unit`]).
pub(crate) fn surface_unit(theme: &Theme, viewport: &Viewport) -> f32 {
    let u = if theme.unit_pin > 0.0 {
        theme.unit_pin
    } else {
        Theme::auto_unit(viewport)
    };
    if u.is_finite() {
        u.clamp(1.0, 2.0)
    } else {
        1.0
    }
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

/// A group's place `r` stepped clear of `others` by `gap`: the nearest of
/// the places just above, below, left or right of one of them that
/// overlaps none and stays in `bounds`; `r` when it overlaps none, or when
/// no such place exists.
fn step_clear(r: Rect, others: &[Rect], gap: f32, bounds: Rect) -> Rect {
    let hit = |p: &Rect| others.iter().any(|o| !o.is_empty() && p.overlaps(o));
    if !hit(&r) {
        return r;
    }
    let inside = |p: &Rect| {
        p.x >= bounds.x - 0.5
            && p.y >= bounds.y - 0.5
            && p.right() <= bounds.right() + 0.5
            && p.bottom() <= bounds.bottom() + 0.5
    };
    others
        .iter()
        .filter(|o| !o.is_empty())
        .flat_map(|o| {
            [
                Rect {
                    y: o.y - gap - r.height,
                    ..r
                },
                Rect {
                    y: o.bottom() + gap,
                    ..r
                },
                Rect {
                    x: o.x - gap - r.width,
                    ..r
                },
                Rect {
                    x: o.right() + gap,
                    ..r
                },
            ]
        })
        .filter(|p| inside(p) && !hit(p))
        .min_by(|a, b| {
            let d = |p: &Rect| (p.x - r.x).abs() + (p.y - r.y).abs();
            d(a).total_cmp(&d(b))
        })
        .unwrap_or(r)
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

/// The element `path` names from node `from`: a full path, else a
/// sibling's id (`Ui::node_at`).
pub(crate) fn node_in(
    tree: &tree::TreeIndex,
    nodes: &[crate::layout::Node],
    path: &str,
    from: usize,
) -> Option<usize> {
    let full = crate::id::key_of_path(path);
    let parent = nodes[from].parent;
    let sibling = if parent == NONE {
        0
    } else {
        path.split('/')
            .fold(nodes[parent].key, crate::id::child_key)
    };
    tree.by_key
        .get(&full)
        .or_else(|| tree.by_key.get(&sibling))
        .copied()
}
