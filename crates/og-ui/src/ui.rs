//! The immediate-mode context: build a tree every tick, read responses back.

use crate::draw::{DrawCmd, DrawItem, EmitDesc, assign_levels, emit};
use crate::id::{Id, child_key};
use crate::input::{InputFrame, Nav};
use crate::layout::{
    Align, Anchor, Content, Cross, Direction, Element, Float, FontRole, Insets, Metrics, NONE,
    Node, Rect, Shape, Size, TextAlign, TextContent, Widget, layout, preorder,
};
use crate::motion::Fx;
use crate::skin::{Ink, Ornament, Slice, SliceFill, StylePart};
use crate::style::{Breakpoint, Density, Srgba, Theme, TypeStep};
use crate::text::{FontMetrics, UiFont};
use og_core::glam::Vec2;
use og_probe::UiElement;
use og_probe::hooks::{find_by_id, find_by_text};
use og_render::{Overlay, TextureHandle};
use std::collections::BTreeMap;

/// The play surface the UI lays out in, in CSS pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
    /// Device pixels per CSS pixel; rectangles snap to device pixels.
    pub dpr: f32,
    /// The device's unsafe margins (notch, home indicator); the root pads by them.
    pub safe_area: Insets,
    /// A touch screen or other coarse pointer (raises controls to touch size).
    pub coarse_pointer: bool,
    /// The page prefers reduced motion (`prefers-reduced-motion: reduce`):
    /// the theme's `reduced_motion` is on whatever the game configured.
    pub reduced_motion: bool,
    /// The page prefers more contrast (`prefers-contrast: more`): the
    /// theme is its high-contrast variant whatever the game configured.
    pub high_contrast: bool,
    /// The page prefers less transparency (`prefers-reduced-transparency:
    /// reduce`): the theme's `reduced_transparency` is on whatever the game
    /// configured (glass parts draw as opaque surfaces, scrims and
    /// translucent fills flat: `Theme::to_reduced_transparency`).
    pub reduced_transparency: bool,
    /// Text controls take macOS's editing keys: Command for the shortcuts
    /// (A, C, X, Z) and with ←/→ for the line's ends (↑/↓ the text's),
    /// Option with ←/→, Backspace and Delete by words, Control+A and E the
    /// line's ends (emacs). Otherwise Control takes the shortcuts and moves
    /// by words (Alt too), Home and End go to the line's ends and Command
    /// only copies, cuts, selects all and undoes. The web shell sets it on
    /// Apple platforms (`navigator.platform`); native hosts leave it off, so
    /// a script edits the same on every OS.
    pub mac_keys: bool,
    /// Where the host's own control covers the surface (the web shell's
    /// round Back, top left of a published page), in CSS pixels; empty is
    /// none. Anchored groups keep out of it: a group it would overlap
    /// moves along the edge it sits on, past it.
    pub covered: Rect,
}

impl Default for Viewport {
    fn default() -> Viewport {
        Viewport {
            width: 1280.0,
            height: 720.0,
            dpr: 1.0,
            safe_area: Insets::default(),
            coarse_pointer: false,
            reduced_motion: false,
            high_contrast: false,
            reduced_transparency: false,
            mac_keys: false,
            covered: Rect::default(),
        }
    }
}

impl Viewport {
    pub fn new(width: f32, height: f32) -> Viewport {
        Viewport {
            width,
            height,
            ..Viewport::default()
        }
    }

    /// The size class of this surface.
    pub fn breakpoint(&self) -> Breakpoint {
        Breakpoint::of(self.width, self.height)
    }

    pub(crate) fn dpr(&self) -> f32 {
        if self.dpr.is_finite() && self.dpr > 0.0 {
            self.dpr
        } else {
            1.0
        }
    }
}

/// How to create a [`Ui`]. The zero value is the dark theme at 60 ticks per second.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiDesc {
    pub theme: Theme,
    /// The world's tick rate, for delays given in milliseconds (0: 60).
    pub ticks_per_second: u32,
    /// The fonts; without one, text is measured at 0.5 em per character and not drawn.
    pub fonts: Fonts,
}

/// The fonts of each role; `display` and `mono` (the numeric role) fall
/// back to `body`, `rounded` to `display`, then `body`. A text the role's
/// font cannot draw whole takes the first font of `chain` (the locale's,
/// og.i18n) that can; a text mixing what the role's font draws with what it
/// does not is laid out per run, each run from the first of the role's,
/// `chain`, `fallbacks` (the game's) and `fallback` (a skin's Latin display
/// face and Chinese text) that has it ([`Fonts::for_text`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fonts {
    pub body: UiFont,
    pub display: Option<UiFont>,
    pub mono: Option<UiFont>,
    /// Levels and scores (`FontRole::Rounded`).
    pub rounded: Option<UiFont>,
    pub fallback: Option<UiFont>,
    /// The active locale's fonts, first first.
    pub chain: Vec<UiFont>,
    /// The game's fonts for other scripts (`UiDesc.fallbackFonts`), tried
    /// after the locale's: a text the role's font cannot draw whole takes
    /// the first that can.
    pub fallbacks: Vec<UiFont>,
}

impl Fonts {
    pub fn get(&self, role: FontRole) -> &UiFont {
        match role {
            FontRole::Body => &self.body,
            FontRole::Display => self.display.as_ref().unwrap_or(&self.body),
            FontRole::Mono => self.mono.as_ref().unwrap_or(&self.body),
            FontRole::Rounded => self
                .rounded
                .as_ref()
                .or(self.display.as_ref())
                .unwrap_or(&self.body),
        }
    }

    /// The font a text of `role` is measured and drawn with: the role's when
    /// its atlas has every character; else the first of `chain` (the
    /// locale's faces, which set its strings whole) whose atlas does; else,
    /// for a text the role's font draws in part (a Latin name beside CJK or
    /// Arabic), the role's font mixed with the others: each run from the
    /// first of the role's, `chain`, `fallbacks` and `fallback` that has it
    /// ([`FontMetrics::is_mixed`]; once [`Fonts::link`] made it, as
    /// [`crate::Ui`] does); else the first of `fallbacks` whose atlas has
    /// every character; else the skin's `fallback` when it has a character
    /// the role's lacks; else the first font, in the same order, that can
    /// draw it with the characters its file renders at run time (`font {
    /// runtime true }`); else (no one font draws it all) the mixed font,
    /// so only the characters no font has are missing.
    pub fn for_text(&self, role: FontRole, text: &str) -> &UiFont {
        let f = self.get(role);
        let cooked = |font: &UiFont| {
            text.chars()
                .all(|c| c.is_whitespace() || font.metrics.in_atlas(c))
        };
        if cooked(f) {
            return f;
        }
        if let Some(c) = self.chain.iter().find(|c| cooked(c)) {
            return c;
        }
        let mixed = f.metrics.mixed.as_deref();
        if let Some(m) = mixed
            && text
                .chars()
                .any(|c| !c.is_whitespace() && f.metrics.in_atlas(c))
        {
            return m;
        }
        let others = || self.chain.iter().chain(&self.fallbacks);
        if let Some(c) = self.fallbacks.iter().find(|c| cooked(c)) {
            return c;
        }
        if let Some(fb) = &self.fallback
            && text
                .chars()
                .any(|c| !c.is_whitespace() && !f.metrics.in_atlas(c) && fb.metrics.has(c))
        {
            return fb;
        }
        let drawable = |font: &UiFont| {
            text.chars()
                .all(|c| c.is_whitespace() || font.metrics.has(c))
        };
        if drawable(f) {
            return f;
        }
        if let Some(c) = others().find(|c| drawable(c)) {
            return c;
        }
        // No one font draws it all: each run from the font that has it, so
        // only what no font has is missing.
        mixed.unwrap_or(f)
    }

    /// Makes each role's mixed font ([`Fonts::for_text`]): the role's font
    /// with `chain`, `fallbacks` and `fallback` after it, for a text that
    /// mixes scripts. [`crate::Ui`] links the fonts it is given; call it
    /// again after changing them by hand.
    pub fn link(&mut self) {
        let others: Vec<UiFont> = self
            .chain
            .iter()
            .chain(&self.fallbacks)
            .chain(self.fallback.as_ref())
            .map(UiFont::alone)
            .collect();
        let others: Option<std::sync::Arc<[UiFont]>> = (!others.is_empty()).then(|| others.into());
        let roles = [
            Some(&mut self.body),
            self.display.as_mut(),
            self.mono.as_mut(),
            self.rounded.as_mut(),
        ];
        for f in roles.into_iter().flatten() {
            f.metrics.mixed = others.as_ref().map(|o| {
                let mut m = f.alone();
                m.metrics.runs = Some(o.clone());
                std::sync::Arc::new(m)
            });
        }
    }
}

impl Metrics for Fonts {
    fn metrics_for(&self, role: FontRole, text: &str) -> &FontMetrics {
        &self.for_text(role, text).metrics
    }
}

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
}

pub(crate) const HIT_SCROLL: u8 = 1;
/// A press focuses it with a caret (text field).
pub(crate) const HIT_TEXT: u8 = 2;
/// Keyboard and gamepad focus can land on it.
pub(crate) const HIT_NAV: u8 = 4;
/// Left and right adjust it when focused.
pub(crate) const HIT_ADJUST: u8 = 8;
/// Outside its scroll container's view: navigation only, never the pointer.
pub(crate) const HIT_OFFSCREEN: u8 = 16;
/// Pointer-interactive (buttons, backdrops, controls).
pub(crate) const HIT_POINTER: u8 = 32;
/// A focus scope (dialog backdrop, open menu).
pub(crate) const HIT_SCOPE: u8 = 64;
/// ↑ and ↓ are its own while it is focused (a multi-line text control
/// edits with them, a zoom view pans).
pub(crate) const HIT_MULTILINE: u8 = 128;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Hit {
    pub key: u64,
    /// What is on screen (cut to the clip).
    pub rect: Rect,
    /// The whole element, for navigation and scrolling into view.
    pub full: Rect,
    pub flags: u8,
    /// The nearest focus scope around it (0: none).
    pub scope: u64,
    /// The scroll container it sits in (0: none).
    pub scroller: u64,
}

impl Hit {
    pub fn has(&self, flag: u8) -> bool {
        self.flags & flag != 0
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
    /// The world has had a typed-text stream (`og.TextInput`): key presses
    /// type nothing from then on.
    pub typed_stream: bool,
    /// The presses counted into double and triple clicks in text controls.
    pub click: crate::edit::Click,
    /// Each text control's undo history, by key.
    pub histories: BTreeMap<u64, crate::edit::History>,
    pub hover_ticks: u32,
    pub pointer: Vec2,
    /// The press that focused a field: where it landed, to place the caret.
    pub focus_click: Option<f32>,
    pub scroll: BTreeMap<u64, f32>,
    /// Horizontal scroll offsets.
    pub scroll_x: BTreeMap<u64, f32>,
    /// Each scroll container's ranges and scrollbar lengths, last frame.
    pub scroll_info: BTreeMap<u64, crate::scroll::ScrollInfo>,
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
    /// Where focus returns when that scope closes.
    pub focus_return: u64,
    /// Keyboard and gamepad navigation as the last frame declared it.
    pub nav_off: bool,
    /// The stick's held direction (0 none, 1 + Nav index) and since which tick.
    pub stick: (u8, u64),
    /// Intents queued by `Ui::navigate` for the next frame, each with
    /// whether the focus was on nothing when it was queued (then it only
    /// puts the focus on the first control: `Ui::navigate`).
    pub pending: Vec<(Nav, bool)>,
    pub anims: BTreeMap<u64, Anim>,
    pub toasts: Vec<Toast>,
    pub next_toast: u32,
    /// Screen transitions by key.
    pub screens: BTreeMap<u64, crate::juice::ScreenState>,
    /// The host's voice entity for each UI sound (bits; 0: none).
    pub voices: [u64; 6],
    /// Players the last frame allowed (`set_players`): pads 1 to players - 1
    /// each move a focus of their own.
    pub players: u8,
    /// Pads 1 to 3: the control each one's focus is on (0: none).
    pub pad_focus: [u64; 3],
    /// Pads 1 to 3: the stick's held direction and since which tick.
    pub pad_stick: [(u8, u64); 3],
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

/// The icon atlas `Ui::icon` draws from: a grid of equal cells of a
/// texture, or an icon set cooked like a font (`font`: an `.ogt` of
/// `type icons`, og-cook's `cook::icons`), whose icons are distance-field
/// glyphs drawn crisp at any size and tinted by their ink.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct IconAtlas {
    pub texture: Option<TextureHandle>,
    pub columns: u32,
    pub rows: u32,
    /// Optional names, one per cell in reading order, for `icon_named`;
    /// an icon set's names, glyph U+E000 + i being icon i.
    pub names: Vec<String>,
    /// An icon set: its MTSDF font (`texture` is then unused).
    pub font: Option<crate::text::UiFont>,
}

/// The code point of an icon set's first icon (og-cook's `ICON_PUA`).
pub const ICON_PUA: u32 = 0xE000;

impl IconAtlas {
    /// The UV rectangle of cell `index`.
    pub fn uv(&self, index: u32) -> [f32; 4] {
        let (c, r) = (self.columns.max(1), self.rows.max(1));
        let (x, y) = ((index % c) as f32, ((index / c) % r) as f32);
        let (w, h) = (1.0 / c as f32, 1.0 / r as f32);
        [x * w, y * h, (x + 1.0) * w, (y + 1.0) * h]
    }

    pub fn index_of(&self, name: &str) -> Option<u32> {
        self.names.iter().position(|n| n == name).map(|i| i as u32)
    }

    /// Whether it draws anything (a texture, or an icon set's font).
    pub fn is_drawn(&self) -> bool {
        self.texture.is_some() || self.font.as_ref().is_some_and(|f| f.handle.is_some())
    }

    /// An icon set's glyph for icon `index`: its font and character.
    pub fn glyph(&self, index: u32) -> Option<(og_render::FontHandle, char)> {
        let f = self.font.as_ref()?.handle?;
        Some((f, char::from_u32(ICON_PUA + index)?))
    }
}

/// One laid-out frame: what the probe reads and what the renderer draws.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiFrame {
    pub viewport: Viewport,
    /// Every visible element in draw order with its (clipped) bounds and text.
    pub elements: Vec<UiElement>,
    /// Draw commands with their painter levels.
    pub items: Vec<DrawItem>,
    /// Problems in this frame's tree (duplicate ids, unclosed elements), each naming the fix.
    pub warnings: Vec<String>,
    /// The path of the control each pad's focus is on (pad 0: the shared
    /// focus), or empty.
    pub focus_paths: [String; 4],
    /// The accessibility tree (`crate::a11y`).
    pub a11y: crate::a11y::A11yTree,
    /// The focused text control, for the host: where an input method's
    /// window goes, and the selection a copy takes (`None`: no text focus).
    pub text_input: Option<TextTarget>,
}

/// What [`Ui::edit_marks`] draws one line's marks from.
pub(crate) struct EditMarks<'a> {
    pub m: &'a crate::text::FontMetrics,
    /// The line's text as shown.
    pub text: &'a str,
    /// Where its first glyph's pen starts, and the line box's top and height.
    pub x: f32,
    pub top: f32,
    pub line_h: f32,
    pub size: f32,
    pub tracking: f32,
    /// The index of the line's first character in the whole text.
    pub first: usize,
    pub edit: &'a crate::layout::EditView,
    pub clip: Rect,
    pub alpha: f32,
    /// The selection (under the text), or the composition and caret (over it).
    pub under: bool,
    /// The caret is on this line (a caret at a soft break shows once).
    pub caret_here: bool,
}

/// The focused text control as a host sees it ([`UiFrame::text_input`]):
/// the browser shell puts its hidden text target over it (so an input
/// method's candidate window opens there) and copies `selection` on the
/// page's copy and cut.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextTarget {
    /// The element's path.
    pub path: String,
    /// Its rectangle, CSS px.
    pub rect: Rect,
    /// A multi-line text area.
    pub multiline: bool,
    /// The selected text (empty: none).
    pub selection: String,
    /// An input method is composing in it.
    pub composing: bool,
}

impl UiFrame {
    /// The element at an id or `Parent/Child` path (topmost first), as `touch_move_to` resolves it.
    pub fn element(&self, path: &str) -> Option<&UiElement> {
        find_by_id(&self.elements, path)
    }

    /// The element showing a text (`[=]text[@N]`), as `move_to_text` resolves it.
    pub fn element_by_text(&self, query: &str) -> Option<&UiElement> {
        find_by_text(&self.elements, query)
    }

    /// Painter levels used: the most sprite and text runs the frame can take.
    pub fn levels(&self) -> i32 {
        self.items.iter().map(|i| i.level + 1).max().unwrap_or(0)
    }

    /// Appends the frame's sprites, text and clip rectangles to an overlay
    /// (`RenderScene::overlay`), in CSS pixels: make it with the viewport's
    /// size and `dpr` (`og_render::Overlay::new(vp.width, vp.height, vp.dpr)`).
    pub fn emit(&self, desc: &EmitDesc, overlay: &mut Overlay) {
        emit(&self.items, desc, overlay);
    }
}

/// Button roles: how a style draws each.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    /// A clear lens: neutral fill (the default).
    #[default]
    Secondary,
    /// The primary action: one per screen.
    Primary,
    /// Text until hovered.
    Quiet,
    /// A destructive action: danger ink on the danger tint.
    Danger,
}

/// Control heights from the density (`controlSm/md/lg`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ControlSize {
    Small,
    #[default]
    Medium,
    Large,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ButtonDesc {
    pub variant: ButtonVariant,
    pub size: ControlSize,
    pub disabled: bool,
    /// `None`: fit the label.
    pub width: Option<Size>,
    /// Take the focus when it appears (it was not on screen last frame)
    /// and no dialog is open: what opens takes the focus, so a title
    /// menu's first item answers Enter and the pad's A at once.
    pub autofocus: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LabelDesc {
    /// `None`: `text-body`.
    pub step: Option<TypeStep>,
    /// `None`: `ink`.
    pub color: Option<Srgba>,
    pub align: TextAlign,
    /// Keep on one line (break only at `\n`).
    pub nowrap: bool,
    pub width: Size,
    pub font: FontRole,
    /// A heading: a skin with a flourish frames it.
    pub heading: bool,
    /// Cut with "…" where the text does not fit its width: on its one line
    /// (with `nowrap`; the label can then shrink in a row) or on its last
    /// line (`max_lines`).
    pub ellipsis: bool,
    /// The most lines a wrapping label shows (0: no limit); with `ellipsis`
    /// the last one ends in "…" when text is left out.
    pub max_lines: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PanelDesc {
    /// Lay children left to right (default: top to bottom).
    pub row: bool,
    pub width: Size,
    pub height: Size,
    /// `None`: `space.lg`.
    pub padding: Option<Insets>,
    /// `None`: `space.md`.
    pub gap: Option<f32>,
    pub align: Align,
    pub cross: Cross,
    /// `None`: `panel`.
    pub background: Option<Srgba>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollDesc {
    pub width: Size,
    pub height: Size,
    pub padding: Insets,
    pub gap: f32,
    /// Which ways it scrolls (vertical by default).
    pub axes: ScrollAxes,
}

/// Which ways a scroll container scrolls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollAxes {
    #[default]
    Vertical,
    Horizontal,
    Both,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ImageDesc {
    /// `None` draws a solid rectangle of `tint`.
    pub texture: Option<TextureHandle>,
    /// `None`: the whole texture (`[u0, v0, u1, v1]`, v down).
    pub uv: Option<[f32; 4]>,
    /// `None`: white.
    pub tint: Option<Srgba>,
    pub width: Size,
    pub height: Size,
    /// Width over height (0: none): with the height left to fit, the image
    /// is as tall as its width over this (`Element::aspect`).
    pub aspect: f32,
    /// What a screen reader calls it (and the probe's text); empty is
    /// decoration.
    pub label: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ProgressDesc {
    /// `None`: grow.
    pub width: Option<Size>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SliderDesc {
    /// The range; `min == max` (the zero value) means 0 to 1.
    pub min: f32,
    pub max: f32,
    /// Snap to multiples of `step` from `min` (0: continuous).
    pub step: f32,
    /// `None`: grow.
    pub width: Option<Size>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FieldDesc {
    /// Shown, dimmed, while the field is empty.
    pub placeholder: String,
    /// `None`: grow.
    pub width: Option<Size>,
    /// The most characters it takes (0: no limit).
    pub max_chars: u32,
}

/// How a multi-line text area looks ([`Ui::text_area`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextAreaDesc {
    /// Shown, dimmed, while the area is empty.
    pub placeholder: String,
    /// `None`: grow.
    pub width: Option<Size>,
    /// Lines shown at once (0: 4); more scroll.
    pub rows: u32,
    /// The most characters it takes (0: no limit).
    pub max_chars: u32,
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
    /// UI events with a skin sound, not yet taken by the host.
    pub(crate) sounds: Vec<crate::skin::UiSound>,
    /// Text a copy or cut took, for the host's clipboard ([`Ui::take_clipboard`]).
    pub(crate) clipboard: Option<String>,
    /// The focused text control this frame (into [`UiFrame::text_input`]).
    pub(crate) text_target: Option<TextTarget>,
    /// This frame's screen-transition cover.
    pub(crate) cover: Option<crate::juice::Cover>,
    /// The configured skin's name (empty: the style).
    pub(crate) style_name: String,
    /// Two touches move a scroller or zoom view this tick: the primary
    /// pointer presses nothing.
    pub(crate) gesture: bool,
    /// The writing direction: right to left mirrors the layout and turns
    /// text alignment round (og.i18n's locale, or og.ui.configure).
    pub(crate) direction: og_i18n::Direction,
}

impl Default for Ui {
    fn default() -> Ui {
        Ui::new(UiDesc::default())
    }
}

const STATE_VERSION: u32 = 8;

/// `theme` with a viewport's preferences: the page's reduced motion turns
/// its flag on, its contrast preference takes the high-contrast variant,
/// and reduced transparency (the page's or the game's) the flat variant
/// (a game's own settings can only add to what the page asks).
fn effective_theme(theme: Theme, viewport: &Viewport) -> Theme {
    let mut t = theme;
    if viewport.high_contrast && t.contrast != crate::style::Contrast::High {
        t = t.to_high_contrast();
    }
    t.reduced_motion |= viewport.reduced_motion;
    if t.reduced_transparency || viewport.reduced_transparency {
        t = t.to_reduced_transparency();
    }
    t
}

/// A skin's `textGlow` size is how far the old four offset copies reached;
/// the distance-field glow fades to nothing over this many times it.
const TEXT_GLOW_REACH: f32 = 2.0;

impl Ui {
    pub fn new(desc: UiDesc) -> Ui {
        Ui {
            configured: desc.theme,
            theme: desc.theme,
            fonts: {
                let mut f = desc.fonts;
                f.link();
                f
            },
            ticks_per_second: if desc.ticks_per_second == 0 {
                60
            } else {
                desc.ticks_per_second
            },
            viewport: Viewport::default(),
            input: InputFrame::default(),
            nodes: Vec::new(),
            kinds: Vec::new(),
            stack: Vec::new(),
            floating: Vec::new(),
            last_closed: NONE,
            warnings: Vec::new(),
            building: false,
            state: State::default(),
            frame: UiFrame::default(),
            nav: NavFrame::default(),
            touched: Default::default(),
            nav_on: true,
            players: 1,
            request_focus: 0,
            scopes: Vec::new(),
            icons: IconAtlas::default(),
            toast_anchor: Anchor::Bottom,
            sounds: Vec::new(),
            clipboard: None,
            text_target: None,
            cover: None,
            style_name: String::new(),
            gesture: false,
            direction: og_i18n::Direction::Ltr,
        }
    }

    /// The icon atlas `icon` draws from.
    pub fn set_icons(&mut self, icons: IconAtlas) {
        self.icons = icons;
    }

    pub fn icons(&self) -> &IconAtlas {
        &self.icons
    }

    /// Where toasts stack (default: bottom centre of the safe area).
    pub fn set_toast_anchor(&mut self, anchor: Anchor) {
        self.toast_anchor = anchor;
    }

    /// The size class of the current play surface.
    pub fn breakpoint(&self) -> Breakpoint {
        self.viewport.breakpoint()
    }

    /// The play surface of the frame being built.
    pub fn viewport(&self) -> Viewport {
        self.viewport
    }

    /// Frames begun so far: the UI's clock.
    pub fn tick(&self) -> u64 {
        self.state.tick
    }

    /// The UI's clock in seconds (frames over the tick rate).
    pub fn seconds(&self) -> f64 {
        self.state.tick as f64 / f64::from(self.ticks_per_second)
    }

    pub fn ticks_per_second(&self) -> u32 {
        self.ticks_per_second
    }

    /// Ticks a duration in milliseconds takes (at least 1).
    pub fn ticks_for(&self, ms: f32) -> u32 {
        Theme::ticks(ms, self.ticks_per_second)
    }

    /// Whether keyboard and gamepad navigation moves focus on this screen.
    /// On by default; a gameplay HUD whose keys move the player turns it off
    /// (an open dialog or menu turns it on while it is open). Takes effect
    /// from the next tick.
    pub fn set_navigation(&mut self, on: bool) {
        self.nav_on = on;
    }

    /// Local multiplayer: how many players this screen takes. Pads 1 to
    /// `count - 1` each move a focus of their own (a ring in their player
    /// colour) among the same controls, and their A presses the control
    /// under it ([`Response::pad`] says which pad); pad 0 shares the one
    /// focus with the keyboard and the pointer. 1 by default; declared every
    /// tick, like [`Ui::set_navigation`], and read from the next.
    pub fn set_players(&mut self, count: u32) {
        self.players = count.clamp(1, 4);
    }

    /// The last frame's accessibility tree: each element with a role, its
    /// name, value, states and bounds, and the focus order (`crate::a11y`).
    pub fn a11y(&self) -> &crate::a11y::A11yTree {
        &self.frame.a11y
    }

    /// What element `i` tells the accessibility tree: role, name,
    /// description, value and its own states (the frame adds `focusable`
    /// and `focused`).
    pub(crate) fn a11y_info(
        &self,
        i: usize,
    ) -> (&'static str, String, String, String, Vec<String>) {
        let n = &self.nodes[i];
        let (kind, anonymous) = self.kinds.get(i).copied().unwrap_or(("", true));
        let role = crate::a11y::role_of(kind, n);
        let none = || ("", String::new(), String::new(), String::new(), Vec::new());
        if role.is_empty() {
            return none();
        }
        // A text box shows its value (or its placeholder), which is not
        // what it is: it is named by its label, the placeholder described.
        let textbox = role == "textbox";
        let mut name = n.a11y.name.clone().unwrap_or_default();
        let mut description = n.a11y.description.clone().unwrap_or_default();
        if name.is_empty() && !textbox {
            name = n.probe_text.clone();
        }
        // A picture nothing names is decoration (an icon names itself).
        if name.is_empty() && role == "image" && !n.nav {
            return none();
        }
        if name.is_empty() && (n.el.interactive || n.nav) && !textbox {
            name = crate::a11y::text_within(&self.nodes, i);
        }
        let labelled = role != "text" && role != "heading";
        if name.is_empty()
            && labelled
            && n.parent != NONE
            && self.nodes[n.parent].el.direction == crate::layout::Direction::Row
        {
            // A control without text of its own is named by the label just
            // before it in its row (a settings row's "Master" names its slider).
            let siblings = &self.nodes[n.parent].children;
            if let Some(at) = siblings.iter().position(|&c| c == i)
                && at > 0
                && self.kinds[siblings[at - 1]].0 == "label"
            {
                name = self.nodes[siblings[at - 1]].probe_text.clone();
            }
        }
        // Without a label, a field is named by what its placeholder asks.
        if name.is_empty() && !description.is_empty() {
            name = std::mem::take(&mut description);
        }
        if name.is_empty() && !anonymous && labelled {
            name = n.id.clone();
        }
        (
            role,
            name,
            description,
            crate::a11y::value_of(n),
            crate::a11y::states_of(n),
        )
    }

    /// The path of the control pad `pad`'s focus is on in the last frame
    /// (pad 0: the shared focus), or `""`.
    pub fn focus_path(&self, pad: u32) -> &str {
        self.frame
            .focus_paths
            .get(pad as usize)
            .map_or("", String::as_str)
    }

    /// Queues a navigation intent for the next tick (a gamepad the input
    /// layer does not read yet, a custom binding, a test). A move queued
    /// while the focus is on nothing puts it on the first control, whatever
    /// its direction, and moves no further: every such move queued before
    /// the focus lands (a game asking each tick until it sees the focus)
    /// lands on that same first control.
    pub fn navigate(&mut self, nav: Nav) {
        let entering = self.state.focus == 0 && !matches!(nav, Nav::Activate | Nav::Back);
        self.state.pending.push((nav, entering));
    }

    /// The key of the focused control (0: none).
    pub fn focused_key(&self) -> u64 {
        self.state.focus
    }

    /// Escape or gamepad B this tick, not taken by a dialog or menu: true
    /// once, then taken. A pause screen closes on it. While a dialog or menu
    /// Back closes is open (the topmost of the last frame), it answers false
    /// and leaves Back to that dialog, even when called before the dialog
    /// is built this tick (a HUD drawn under pause): og-ui's rule, the
    /// topmost dialog or menu closes. A locked dialog lets it through.
    pub fn back(&mut self) -> bool {
        if self.state.scope != 0 && !self.state.scope_locked {
            return false;
        }
        std::mem::take(&mut self.nav.back)
    }

    /// The widget memory of `key`, marked as used this frame.
    pub(crate) fn anim(&mut self, key: u64) -> Anim {
        self.touched.insert(key);
        self.state.anims.get(&key).copied().unwrap_or_default()
    }

    pub(crate) fn has_anim(&self, key: u64) -> bool {
        self.state.anims.contains_key(&key)
    }

    pub(crate) fn set_anim(&mut self, key: u64, a: Anim) {
        self.touched.insert(key);
        self.state.anims.insert(key, a);
    }

    /// The key the next element with this id would get under the open element.
    pub(crate) fn key_for(&self, id: &str) -> u64 {
        let parent = self.nodes.get(self.parent()).map_or(1, |n| n.key);
        child_key(parent, id)
    }

    /// The theme in effect: the configured one ([`Ui::set_theme`]) with the
    /// viewport's reduced-motion and contrast preferences applied.
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.configured = theme;
        self.theme = effective_theme(theme, &self.viewport);
    }

    /// The configured skin's name, for the theme the host reports (empty: the style).
    pub fn style_name(&self) -> &str {
        &self.style_name
    }

    pub fn set_style_name(&mut self, name: &str) {
        self.style_name = name.to_owned();
    }

    pub fn fonts(&self) -> &Fonts {
        &self.fonts
    }

    pub fn set_fonts(&mut self, fonts: Fonts) {
        self.fonts = fonts;
        self.fonts.link();
    }

    /// The writing direction (left to right by default).
    pub fn direction(&self) -> og_i18n::Direction {
        self.direction
    }

    /// Right to left mirrors every frame's layout from the next `end` on.
    pub fn set_direction(&mut self, direction: og_i18n::Direction) {
        self.direction = direction;
    }

    /// The control sizes in effect for the current viewport.
    pub fn sizes(&self) -> Density {
        self.theme.sizes(&self.viewport)
    }

    /// The last completed frame.
    pub fn frame(&self) -> &UiFrame {
        &self.frame
    }

    /// The input of the frame being built.
    pub fn input(&self) -> &InputFrame {
        &self.input
    }

    /// Starts a frame whose root is a column over the whole viewport, padded by the safe area.
    pub fn begin(&mut self, input: InputFrame, viewport: Viewport) {
        self.begin_with(input, viewport, Element::column());
    }

    /// Starts a frame with a root of your own (its size is the viewport; the
    /// safe area is added to its padding; its background covers the whole viewport).
    pub fn begin_with(&mut self, input: InputFrame, viewport: Viewport, root: Element) {
        // Scrolling first: a press on a scrollbar grabs it before anything
        // under it, two touches scroll or zoom what is under them.
        let grabbed = self.scroll_input(&input);
        let mut input = input;
        if self.gesture {
            input.pressed = false;
            input.released = false;
            input.down = false;
        }
        if self.building {
            self.warnings.push(
                "begin without end: the previous frame was dropped; call ui.end() once per tick"
                    .into(),
            );
        }
        self.viewport = viewport;
        self.theme = effective_theme(self.configured, &viewport);
        self.nodes.clear();
        self.kinds.clear();
        self.stack.clear();
        self.floating.clear();
        self.warnings.clear();
        self.last_closed = NONE;
        self.building = true;

        self.state.tick += 1;
        self.nav = NavFrame::default();
        self.touched.clear();
        self.request_focus = 0;
        self.scopes.clear();
        self.nav_on = true;
        self.players = 1;
        self.cover = None;

        // Hit test against last frame's rectangles, topmost first.
        let p = input.pointer;
        let hot = if input.pointer_inside {
            self.state
                .hits
                .iter()
                .rev()
                .find(|h| {
                    !h.has(HIT_OFFSCREEN)
                        && h.has(HIT_POINTER | HIT_SCROLL | HIT_TEXT)
                        && h.rect.contains(p.x, p.y)
                })
                .map_or(0, |h| h.key)
        } else {
            0
        };
        let hot = grabbed.unwrap_or(hot);
        self.state.hover_ticks = if hot != 0 && hot == self.state.hot {
            self.state.hover_ticks.saturating_add(1)
        } else {
            0
        };
        if hot != 0 && hot != self.state.hot && self.hit(hot).is_some_and(|h| h.has(HIT_NAV)) {
            self.sound(crate::skin::UiSound::Hover);
        }
        if input.pressed && grabbed.is_none() && self.hit(hot).is_some_and(|h| h.has(HIT_NAV)) {
            self.sound(crate::skin::UiSound::Press);
        }
        self.state.hot = hot;
        self.state.focus_click = None;
        if input.pressed {
            self.state.active = hot;
            let text = self.hit(hot).is_some_and(|h| h.has(HIT_TEXT));
            if text {
                self.state.focus = hot;
                self.state.focus_click = Some(p.x);
            } else if self.hit(hot).is_some_and(|h| h.has(HIT_NAV)) {
                // A pressed control takes focus without the ring (as :focus-visible).
                self.state.focus = hot;
            } else {
                self.state.focus = 0;
            }
            self.state.focus_visible = false;
        }
        let intents = self.intents(&input);
        self.apply_nav(&intents);
        self.apply_a11y(&input.a11y);
        self.apply_pads(&input);
        self.state.pointer = p;
        // Once a world has a typed-text stream, key presses type nothing.
        if input.typed_stream {
            self.state.typed_stream = true;
        } else if self.state.typed_stream {
            input.text.clear();
        }
        self.input = input;
        self.text_target = None;

        let mut el = root;
        el.width = Size::Fixed(viewport.width);
        el.height = Size::Fixed(viewport.height);
        el.padding = el.padding.add(&viewport.safe_area);
        el.scroll = false;
        self.nodes.push(Node::new(1, String::new(), NONE, el));
        self.kinds.push(("root", false));
        self.stack.push(0);
    }

    /// A further touch (pointers 1 to 4, `og.Pointers`) on the touch control
    /// `key`: the first pointer held, or released this tick, that started on
    /// its last-frame rectangle (`by_start`, a stick that keeps the finger
    /// that grabbed it) or is over it now (buttons, d-pad arms). Its
    /// position, and whether it is still held.
    pub(crate) fn touch(&self, key: u64, by_start: bool) -> Option<(Vec2, bool)> {
        let rect = self.hit(key)?.rect;
        let p = &self.input.pointers;
        (1..og_input::MAX_POINTERS as u32).find_map(|id| {
            let held = p.down(id);
            if !held && !p.is_released(id) {
                return None;
            }
            let at = if by_start {
                p.start(id)
            } else {
                p.position(id)
            };
            rect.contains(at.x, at.y).then(|| (p.position(id), held))
        })
    }

    pub(crate) fn hit(&self, key: u64) -> Option<&Hit> {
        if key == 0 {
            return None;
        }
        self.state.hits.iter().find(|h| h.key == key)
    }

    pub(crate) fn parent(&self) -> usize {
        self.stack.last().copied().unwrap_or(0)
    }

    /// Adds a node under the open element and returns its index.
    pub(crate) fn push(&mut self, kind: &'static str, id: Id, el: Element) -> usize {
        if !self.building {
            self.warnings
                .push("an element was added outside begin/end; call ui.begin() first".into());
            self.begin(InputFrame::default(), self.viewport);
        }
        let parent = self.parent();
        let anonymous = id.is_empty();
        let mut local = if anonymous {
            let n = self.nodes[parent]
                .children
                .iter()
                .filter(|&&c| self.kinds[c] == (kind, true))
                .count();
            format!("{kind}#{n}")
        } else {
            let name = id.as_str();
            if name.contains('/') {
                self.warnings.push(format!(
                    "id '{name}' contains '/', the path separator; it is used as '{}'",
                    name.replace('/', "_")
                ));
                name.replace('/', "_")
            } else {
                name.to_owned()
            }
        };
        if self.nodes[parent]
            .children
            .iter()
            .any(|&c| self.nodes[c].id == local)
        {
            let mut n = 1;
            while self.nodes[parent]
                .children
                .iter()
                .any(|&c| self.nodes[c].id == format!("{local}~{n}"))
            {
                n += 1;
            }
            let renamed = format!("{local}~{n}");
            self.warnings.push(format!(
                "duplicate id '{local}' under '{}'; this one is '{renamed}': give siblings distinct ids (ui_idi(name, index))",
                self.path_of(parent)
            ));
            local = renamed;
        }
        let key = child_key(self.nodes[parent].key, &local);
        let index = self.nodes.len();
        self.nodes.push(Node::new(key, local, parent, el));
        self.kinds.push((kind, anonymous));
        self.nodes[parent].children.push(index);
        index
    }

    pub(crate) fn path_of(&self, mut i: usize) -> String {
        let mut parts = Vec::new();
        while i != NONE && i != 0 {
            parts.push(self.nodes[i].id.clone());
            i = self.nodes[i].parent;
        }
        if parts.is_empty() {
            return "root".into();
        }
        parts.reverse();
        parts.join("/")
    }

    pub(crate) fn respond(&self, key: u64) -> Response {
        let s = &self.state;
        let hovered = s.hot == key && (s.active == 0 || s.active == key);
        let by_pad = (key != 0)
            .then(|| self.nav.pad_activate.iter().position(|&k| k == key))
            .flatten();
        let pad = match by_pad {
            Some(k) => Some(k as u32 + 1),
            None if self.nav.activate == key && key != 0 && self.nav.pad0 => Some(0),
            None => None,
        };
        let mut focused_by = u32::from(s.focus == key && key != 0);
        for (k, &f) in s.pad_focus.iter().enumerate() {
            if f == key && key != 0 && (k as u8) + 1 < s.players {
                focused_by |= 2 << k;
            }
        }
        Response {
            hovered,
            pressed: s.active == key && self.input.down,
            clicked: (s.active == key && self.input.released && s.hot == key)
                || (self.nav.activate == key && key != 0)
                || by_pad.is_some(),
            focused: s.focus == key,
            pad,
            focused_by,
            ..Response::default()
        }
    }

    /// Marks node `i` as a keyboard/gamepad focus target and returns its
    /// response; the ring shows while navigation put focus on it.
    pub(crate) fn nav_target(&mut self, i: usize, adjust: bool) -> Response {
        let key = self.nodes[i].key;
        let r = self.respond(key);
        let n = &mut self.nodes[i];
        n.nav = true;
        n.adjust = adjust;
        n.ring = self.state.focus == key && self.state.focus_visible;
        n.pad_rings = (r.focused_by >> 1) as u8;
        r
    }

    /// Opens a container; children follow until [`Ui::close`].
    pub fn open(&mut self, id: impl Into<Id>, el: Element) {
        let kind = match (el.scroll || el.scroll_x, el.direction) {
            (true, _) => "scroll",
            (false, Direction::Row) => "row",
            (false, Direction::Column) => "column",
        };
        let i = self.push(kind, id.into(), el);
        self.stack.push(i);
    }

    /// Closes the open container. For an `interactive` element, its response.
    pub fn close(&mut self) -> Response {
        if self.stack.len() <= 1 {
            self.warnings
                .push("close without open: every ui.open() takes one ui.close()".into());
            return Response::default();
        }
        let i = self.stack.pop().unwrap_or(0);
        self.last_closed = i;
        if self.nodes[i].el.interactive {
            self.respond(self.nodes[i].key)
        } else {
            Response::default()
        }
    }

    /// A container built by `f`.
    pub fn container(
        &mut self,
        id: impl Into<Id>,
        el: Element,
        f: impl FnOnce(&mut Ui),
    ) -> Response {
        self.open(id, el);
        f(self);
        self.close()
    }

    /// A row built by `f`.
    pub fn row(&mut self, id: impl Into<Id>, el: Element, f: impl FnOnce(&mut Ui)) {
        self.container(
            id,
            Element {
                direction: Direction::Row,
                ..el
            },
            f,
        );
    }

    /// A column built by `f`.
    pub fn column(&mut self, id: impl Into<Id>, el: Element, f: impl FnOnce(&mut Ui)) {
        self.container(
            id,
            Element {
                direction: Direction::Column,
                ..el
            },
            f,
        );
    }

    /// A card: `panel`, `radius.panel`, padded and spaced by the style.
    pub fn panel(&mut self, id: impl Into<Id>, desc: PanelDesc, f: impl FnOnce(&mut Ui)) {
        self.open_panel(id, desc);
        f(self);
        self.close();
    }

    /// Opens a card ([`Ui::panel`] without the closure); its children follow
    /// until [`Ui::close`]. For hosts whose build code cannot pass a closure
    /// (the TypeScript binding).
    pub fn open_panel(&mut self, id: impl Into<Id>, desc: PanelDesc) {
        let t = self.theme;
        let el = Element {
            direction: if desc.row {
                Direction::Row
            } else {
                Direction::Column
            },
            width: desc.width,
            height: desc.height,
            padding: desc.padding.unwrap_or(Insets::all(t.space.lg)),
            gap: desc.gap.unwrap_or(t.space.md),
            align: desc.align,
            cross: desc.cross,
            background: Some(desc.background.unwrap_or(t.palette.panel)),
            radius: t.radius.panel,
            skin: Some(crate::skin::StylePart::Card),
            ..Element::default()
        };
        let id = id.into();
        let i = self.push("panel", id, el);
        self.stack.push(i);
    }

    /// A vertically scrolling, clipping column built by `f`.
    pub fn scroll(&mut self, id: impl Into<Id>, desc: ScrollDesc, f: impl FnOnce(&mut Ui)) {
        self.open_scroll(id, desc);
        f(self);
        self.close();
    }

    /// Opens a scroll column ([`Ui::scroll`] without the closure); its
    /// children follow until [`Ui::close`].
    pub fn open_scroll(&mut self, id: impl Into<Id>, desc: ScrollDesc) {
        self.open(
            id,
            Element {
                direction: Direction::Column,
                width: desc.width,
                height: desc.height,
                padding: desc.padding,
                gap: desc.gap,
                scroll: desc.axes != ScrollAxes::Horizontal,
                scroll_x: desc.axes != ScrollAxes::Vertical,
                ..Element::default()
            },
        );
    }

    pub(crate) fn leaf(&mut self, kind: &'static str, id: Id, el: Element) -> usize {
        let i = self.push(kind, id, el);
        self.last_closed = i;
        i
    }

    pub(crate) fn text_content(&self, text: &str, step: TypeStep, color: Srgba) -> TextContent {
        TextContent {
            text: text.to_owned(),
            step,
            color,
            align: TextAlign::Start,
            wrap: true,
            font: FontRole::Body,
            ellipsis: false,
            max_lines: 0,
        }
    }

    /// Text in MSDF, wrapping at its width unless `nowrap`.
    pub fn label(&mut self, id: impl Into<Id>, text: &str, desc: LabelDesc) {
        let t = self.theme;
        let i = self.leaf(
            "label",
            id.into(),
            Element {
                width: desc.width,
                ..Element::default()
            },
        );
        let n = &mut self.nodes[i];
        n.content = Content::Text(TextContent {
            text: text.to_owned(),
            step: desc.step.unwrap_or(t.type_ramp.body),
            color: desc.color.unwrap_or(t.palette.ink),
            align: desc.align,
            wrap: !desc.nowrap,
            font: desc.font,
            ellipsis: desc.ellipsis,
            max_lines: desc.max_lines,
        });
        // The probe and screen readers read the whole text, cut or not.
        n.probe_text = text.to_owned();
        n.heading = desc.heading;
    }

    /// A capsule button with a label. `clicked` is a press and a release inside it.
    pub fn button(&mut self, id: impl Into<Id>, text: &str, desc: ButtonDesc) -> Response {
        let t = self.theme;
        let sizes = self.sizes();
        let h = match desc.size {
            ControlSize::Small => sizes.control_sm,
            ControlSize::Medium => sizes.control_md,
            ControlSize::Large => sizes.control_lg,
        };
        let pad = if desc.size == ControlSize::Small {
            t.space.md
        } else {
            t.space.lg
        };
        let i = self.leaf(
            "button",
            id.into(),
            Element {
                width: desc.width.unwrap_or_default(),
                height: Size::Fixed(h),
                padding: Insets::xy(pad, 0.0),
                // A skin that shakes a refused press hears presses on a disabled button.
                interactive: !desc.disabled || t.skin.motion.shake > 0.0,
                radius: t.radius.control,
                ..Element::default()
            },
        );
        self.nodes[i].disabled = desc.disabled;
        let refused =
            desc.disabled && t.skin.motion.shake > 0.0 && self.respond(self.nodes[i].key).clicked;
        let r = if desc.disabled {
            Response::default()
        } else {
            self.nav_target(i, false)
        };
        // What opens takes the focus (as a dialog's first control and a
        // dialogue box do): appearing, outside a dialog.
        let key = self.nodes[i].key;
        if desc.autofocus
            && !desc.disabled
            && self.state.scope == 0
            && !self.state.hits.iter().any(|h| h.key == key)
        {
            self.request_focus = key;
        }
        let p = &t.palette;
        let (bg, fg) = if desc.disabled {
            (Some(p.control), p.ink_faint)
        } else {
            match desc.variant {
                ButtonVariant::Primary => (
                    Some(if r.pressed {
                        p.primary_active
                    } else if r.hovered {
                        p.primary_hover
                    } else {
                        p.primary
                    }),
                    p.on_primary,
                ),
                ButtonVariant::Secondary => (
                    Some(if r.hovered || r.pressed {
                        p.control_strong
                    } else {
                        p.control
                    }),
                    p.ink,
                ),
                ButtonVariant::Quiet => ((r.hovered || r.pressed).then_some(p.control), p.ink),
                // Danger ink on the danger tint (10 % at rest, 15 %
                // hovered), composited in sRGB over what is behind it, so its label
                // reads at 4.5:1 on every ground.
                ButtonVariant::Danger => (
                    Some(
                        p.danger
                            .fade(if r.hovered || r.pressed { 0.15 } else { 0.10 })
                            .over(self.behind(i)),
                    ),
                    p.danger,
                ),
            }
        };
        let mut content = self.text_content(text, t.type_ramp.body, fg);
        content.align = TextAlign::Center;
        content.wrap = false;
        let part = if desc.disabled {
            StylePart::ButtonDisabled
        } else if r.pressed {
            StylePart::ButtonPressed
        } else if r.hovered {
            StylePart::ButtonHover
        } else {
            StylePart::Button
        };
        let press = self.button_motion(i, r, refused);
        let n = &mut self.nodes[i];
        n.el.background = bg;
        n.el.skin = Some(part);
        n.content = Content::Text(content);
        n.widget = Widget::Button {
            pressed: r.pressed,
            press,
        };
        n.probe_text = text.to_owned();
        r
    }

    /// A texture region, or a solid rectangle without a texture; with
    /// `aspect`, a fitting side follows the other.
    pub fn image(&mut self, id: impl Into<Id>, desc: ImageDesc) {
        let i = self.leaf(
            "image",
            id.into(),
            Element {
                width: desc.width,
                height: desc.height,
                aspect: desc.aspect.max(0.0),
                ..Element::default()
            },
        );
        self.nodes[i].content = Content::Image {
            texture: desc.texture,
            uv: desc.uv.unwrap_or([0.0, 0.0, 1.0, 1.0]),
            tint: desc.tint.unwrap_or(Srgba::WHITE),
        };
        self.nodes[i].probe_text = desc.label;
    }

    /// A capsule track filled to `value` (0–1).
    pub fn progress(&mut self, id: impl Into<Id>, value: f32, desc: ProgressDesc) {
        let t = self.theme;
        let i = self.leaf(
            "progress",
            id.into(),
            Element {
                width: desc.width.unwrap_or(Size::grow()),
                height: Size::Fixed(t.space.sm),
                ..Element::default()
            },
        );
        let mut v = if value.is_finite() {
            value.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let ms = t.skin.motion.progress_ms;
        if ms > 0.0 && !t.reduced_motion {
            // Eases toward the value: a constant share of the gap per tick.
            let key = child_key(self.nodes[i].key, "ease");
            let fresh = !self.has_anim(key);
            let mut a = self.anim(key);
            if fresh {
                a.a = v;
            } else {
                let k = (4.0 / self.ticks_for(ms) as f32).min(1.0);
                a.a += (v - a.a) * k;
                if (v - a.a).abs() < 0.002 {
                    a.a = v;
                }
            }
            self.set_anim(key, a);
            v = a.a;
        }
        self.nodes[i].widget = Widget::Progress { value: v };
    }

    /// A button's skin motion: the hover lift and the press squash on the
    /// skin's springs (`motion.hoverSpring`, `motion.pressSpring`), and the shake of a refused press (`motion.duration.slow`). Answers
    /// how far in the press is (0 at rest, 1 held; past either while the
    /// spring overshoots).
    fn button_motion(&mut self, i: usize, r: Response, refused: bool) -> f32 {
        let t = self.theme;
        let skin = t.skin;
        if skin.hover.is_none() && skin.press.is_none() && skin.motion.shake <= 0.0 {
            return if r.pressed { 1.0 } else { 0.0 };
        }
        let key = self.nodes[i].key;
        let hover_spring = skin
            .motion
            .hover_spring
            .unwrap_or(crate::theme::SPRING_DEFAULT);
        let press_spring = skin.motion.press_spring.unwrap_or(crate::theme::Spring {
            damping: 0.55,
            response_ms: t.durations_ms.instant,
        });
        let dt = 1.0 / self.ticks_per_second as f32;
        // Each spring keeps its value in `a` and its velocity in `b`.
        let step = |ui: &mut Ui, name: &str, spring: crate::theme::Spring, target: f32| -> f32 {
            let spring = og_tween::Spring {
                damping: spring.damping,
                response_ms: spring.response_ms,
            };
            let k = child_key(key, name);
            let fresh = !ui.has_anim(k);
            let mut a = ui.anim(k);
            if t.reduced_motion || (fresh && target == 0.0) {
                a.a = target;
                a.b = 0.0;
            } else {
                let s = spring.step(
                    og_tween::SpringState {
                        value: a.a,
                        velocity: a.b,
                    },
                    target,
                    dt,
                );
                a.a = s.value;
                a.b = s.velocity;
            }
            ui.set_anim(k, a);
            a.a
        };
        let hover = step(
            self,
            "hover",
            hover_spring,
            if r.hovered { 1.0 } else { 0.0 },
        );
        let press = step(
            self,
            "press",
            press_spring,
            if r.pressed { 1.0 } else { 0.0 },
        );
        let key = child_key(key, "motion");
        let mut a = self.anim(key);
        let now = self.state.tick;
        if refused {
            a.t = now;
            a.b = 1.0;
            self.sound(crate::skin::UiSound::Error);
        }
        self.set_anim(key, a);
        if t.reduced_motion {
            return if r.pressed { 1.0 } else { 0.0 };
        }
        let mut f = Fx::default();
        if let Some(h) = skin.hover {
            f.dy += h.dy * hover;
            f.scale = 1.0 + (h.scale - 1.0) * hover;
        }
        if let Some(p) = skin.press {
            f.dy += p.dy * press;
        }
        if a.b > 0.0 {
            let k = (now - a.t) as f32 / self.ticks_for(t.durations_ms.slow) as f32;
            if k < 1.0 {
                // Three decaying swings, arithmetic only (a triangle wave).
                let x = k * 3.0;
                let tri = 4.0 * (x - x.floor() - 0.5).abs() - 1.0;
                f.dx = skin.motion.shake * (1.0 - k) * tri;
            }
        }
        if f != Fx::default() {
            self.nodes[i].fx = Some(f);
        }
        press
    }

    /// Queues a UI event's sound when the skin has one for it.
    pub(crate) fn sound(&mut self, event: crate::skin::UiSound) {
        if self.theme.skin.sound(event) != 0 {
            self.sounds.push(event);
        }
    }

    /// Text a copy (Ctrl or Cmd with C) or cut (with X) took from a text
    /// control's selection since the last call: the host puts it on the
    /// clipboard (the browser shell does on the page's copy event).
    pub fn take_clipboard(&mut self) -> Option<String> {
        self.clipboard.take()
    }

    /// The UI events of the last frames that have a sound in the skin,
    /// oldest first; taking them empties the queue (the host plays them).
    pub fn take_sounds(&mut self) -> Vec<(crate::skin::UiSound, u64)> {
        let skin = self.theme.skin;
        std::mem::take(&mut self.sounds)
            .into_iter()
            .map(|e| (e, skin.sound(e)))
            .collect()
    }

    /// A horizontal slider over `value`; dragging anywhere on it sets the value.
    pub fn slider(&mut self, id: impl Into<Id>, value: &mut f32, desc: SliderDesc) -> Response {
        let sizes = self.sizes();
        let i = self.leaf(
            "slider",
            id.into(),
            Element {
                width: desc.width.unwrap_or(Size::grow()),
                height: Size::Fixed(sizes.control_sm),
                interactive: true,
                ..Element::default()
            },
        );
        let key = self.nodes[i].key;
        let mut r = self.nav_target(i, true);
        let (lo, hi) = if desc.min == desc.max {
            (0.0, 1.0)
        } else {
            (desc.min, desc.max)
        };
        if r.focused && self.nav.adjust != 0 {
            // Left and right step by `step`, or a twentieth of the range.
            let unit = if desc.step > 0.0 {
                desc.step
            } else {
                (hi - lo) / 20.0
            };
            let v = (*value + unit * self.nav.adjust as f32).clamp(lo.min(hi), lo.max(hi));
            if v != *value {
                *value = v;
                r.changed = true;
            }
        }
        let thumb = self.thumb();
        if self.state.active == key
            && (self.input.down || self.input.pressed)
            && let Some(h) = self.hit(key)
        {
            let travel = (h.rect.width - thumb).max(1.0);
            let t = ((self.input.pointer.x - h.rect.x - thumb / 2.0) / travel).clamp(0.0, 1.0);
            let mut v = lo + t * (hi - lo);
            if desc.step > 0.0 {
                v = lo + ((v - lo) / desc.step).round() * desc.step;
                v = v.clamp(lo.min(hi), lo.max(hi));
            }
            if v != *value {
                *value = v;
                r.changed = true;
            }
        }
        let t = if hi != lo {
            ((*value - lo) / (hi - lo)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.nodes[i].widget = Widget::Slider {
            t: if t.is_finite() { t } else { 0.0 },
            active: r.pressed,
        };
        // Its value as people read it: the step's decimals, at most two.
        let decimals = if desc.step >= 1.0 || (hi - lo).abs() > 20.0 {
            0
        } else {
            2
        };
        self.nodes[i].a11y.value = Some(format!("{:.*}", decimals, *value));
        self.nodes[i].a11y.range = Some(crate::a11y::A11yRange {
            min: Some(lo.min(hi)),
            max: Some(lo.max(hi)),
            now: *value,
        });
        r
    }

    pub(crate) fn thumb(&self) -> f32 {
        self.sizes().control_sm - 2.0 * self.theme.space.xs
    }

    pub(crate) fn switch_size(&self) -> (f32, f32) {
        let h = self.sizes().control_sm - self.theme.space.xs;
        // The switch's 51:31 proportion (geometry, not a design value).
        (h * 51.0 / 31.0, h)
    }

    /// A switch with a label; a click flips `on`.
    pub fn toggle(&mut self, id: impl Into<Id>, on: &mut bool, label: &str) -> Response {
        let t = self.theme;
        let sizes = self.sizes();
        let (sw, _) = self.switch_size();
        let i = self.leaf(
            "toggle",
            id.into(),
            Element {
                height: Size::Fixed(sizes.control_sm),
                padding: Insets {
                    left: sw + t.space.sm,
                    ..Insets::default()
                },
                interactive: true,
                ..Element::default()
            },
        );
        let mut r = self.nav_target(i, false);
        if r.clicked {
            *on = !*on;
            r.changed = true;
        }
        let mut content = self.text_content(label, t.type_ramp.body, t.palette.ink);
        content.wrap = false;
        let n = &mut self.nodes[i];
        n.content = Content::Text(content);
        n.widget = Widget::Toggle { on: *on };
        n.probe_text = label.to_owned();
        r
    }

    /// A one-line text field: a press focuses it (caret at the press, a
    /// drag selects and scrolls the text past the field's edge, a double
    /// click selects a word and a triple click everything), typed text
    /// replaces the selection at the caret, Backspace/Delete/←/→/Home/End
    /// edit (Shift extends the selection; Alt or Ctrl moves and deletes by
    /// words, Ctrl+Home/End goes to the ends; Ctrl or Cmd with A selects
    /// all, with C copies and with X cuts, for the host's clipboard, with Z
    /// undoes and with Shift+Z or Y redoes, from a history of the last 100
    /// steps with typing grouped by words), an input method's composition
    /// shows underlined at the caret until it commits, Enter submits and
    /// Escape leaves. The text scrolls to keep the caret in view; the caret
    /// blinks by the tick (steady under reduced motion).
    pub fn text_field(
        &mut self,
        id: impl Into<Id>,
        value: &mut String,
        desc: &FieldDesc,
    ) -> Response {
        let t = self.theme;
        let sizes = self.sizes();
        let i = self.leaf(
            "field",
            id.into(),
            Element {
                width: desc.width.unwrap_or(Size::grow()),
                height: Size::Fixed(sizes.control_md),
                padding: Insets::xy(t.space.md, 0.0),
                interactive: true,
                radius: t.radius.control,
                skin: Some(StylePart::Field),
                ..Element::default()
            },
        );
        self.nodes[i].focusable = true;
        self.nodes[i].nav = true;
        let key = self.nodes[i].key;
        let mut r = self.respond(key);
        let step = t.type_ramp.body;
        if r.focused {
            let inner = self
                .hit(key)
                .map(|h| h.full.inset(&Insets::xy(t.space.md, 0.0)));
            let e = self.edit(
                key,
                value,
                &crate::edit::EditDesc {
                    multiline: false,
                    max_chars: desc.max_chars,
                    role: FontRole::Body,
                    size: step.size,
                    tracking: crate::layout::step_tracking(&step, value),
                    line_height: step.line_height,
                    inner,
                },
            );
            r.changed = e.changed;
            r.submitted = e.submitted;
            r.focused = self.state.focus == key;
        }
        let focused = r.focused;
        let (shown, composition) = self.shown_text(value, focused);
        let placeholder = shown.is_empty();
        let shown = if placeholder {
            desc.placeholder.clone()
        } else {
            shown
        };
        let color = if placeholder {
            t.palette.ink_faint
        } else {
            t.palette.ink
        };
        let mut content = self.text_content(&shown, step, color);
        content.wrap = false;
        let edit = self.edit_view(key, focused, value, composition, false);
        let n = &mut self.nodes[i];
        n.el.background = Some(if focused {
            t.palette.panel
        } else {
            t.palette.control
        });
        n.content = Content::Text(content);
        n.widget = Widget::Field(edit);
        n.probe_text = if value.is_empty() {
            desc.placeholder.clone()
        } else {
            value.clone()
        };
        // A field is named by its label (the label before it in its row),
        // else its placeholder, else its id; the placeholder describes it,
        // and its value is its text.
        n.a11y.description = Some(desc.placeholder.clone());
        n.a11y.value = Some(value.clone());
        r
    }

    /// A multi-line text area: [`Ui::text_field`]'s editing, with Enter
    /// starting a new line, ↑ and ↓ moving between lines (and Home and End
    /// to a line's ends), a triple click selecting the line between its
    /// breaks, the text wrapped between words at the area's width and
    /// scrolled to keep the caret's line in view (a drag above or below it
    /// scrolls it, faster the further out). `rows` lines show at once.
    pub fn text_area(
        &mut self,
        id: impl Into<Id>,
        value: &mut String,
        desc: &TextAreaDesc,
    ) -> Response {
        let t = self.theme;
        let step = t.type_ramp.body;
        let rows = if desc.rows == 0 { 4 } else { desc.rows };
        let lh = if value.chars().any(crate::text::is_cjk) {
            step.cjk_line_height
        } else {
            step.line_height
        };
        let pad = Insets::xy(t.space.md, t.space.sm);
        let i = self.leaf(
            "textarea",
            id.into(),
            Element {
                width: desc.width.unwrap_or(Size::grow()),
                height: Size::Fixed(rows as f32 * lh + pad.vertical()),
                padding: pad,
                interactive: true,
                radius: t.radius.control,
                skin: Some(StylePart::Field),
                ..Element::default()
            },
        );
        self.nodes[i].focusable = true;
        self.nodes[i].nav = true;
        self.nodes[i].multiline = true;
        let key = self.nodes[i].key;
        let mut r = self.respond(key);
        if r.focused {
            let inner = self.hit(key).map(|h| h.full.inset(&pad));
            let e = self.edit(
                key,
                value,
                &crate::edit::EditDesc {
                    multiline: true,
                    max_chars: desc.max_chars,
                    role: FontRole::Body,
                    size: step.size,
                    tracking: crate::layout::step_tracking(&step, value),
                    line_height: lh,
                    inner,
                },
            );
            r.changed = e.changed;
            r.submitted = e.submitted;
            r.focused = self.state.focus == key;
        }
        let focused = r.focused;
        let (shown, composition) = self.shown_text(value, focused);
        let edit = self.edit_view(key, focused, value, composition, true);
        let n = &mut self.nodes[i];
        n.el.background = Some(if focused {
            t.palette.panel
        } else {
            t.palette.control
        });
        n.widget = Widget::TextArea {
            edit,
            text: shown,
            placeholder: desc.placeholder.clone(),
            line_height: lh,
        };
        n.probe_text = if value.is_empty() {
            desc.placeholder.clone()
        } else {
            value.clone()
        };
        // Named and described as a field is.
        n.a11y.description = Some(desc.placeholder.clone());
        n.a11y.value = Some(value.clone());
        r
    }

    /// A text control's text as shown: its value with the composition (if
    /// focused and composing) at the caret, and the composition's range.
    fn shown_text(&self, value: &str, focused: bool) -> (String, Option<(usize, usize)>) {
        if !focused || self.state.composition.is_empty() {
            return (value.to_string(), None);
        }
        let at = self.state.caret.min(value.chars().count());
        let mut out: String = value.chars().take(at).collect();
        out.push_str(&self.state.composition);
        out.extend(value.chars().skip(at));
        let n = self.state.composition.chars().count();
        (out, Some((at, at + n)))
    }

    /// The caret, selection and composition of text control `key` for
    /// drawing (in the shown text's characters), and the host's view of it.
    fn edit_view(
        &mut self,
        key: u64,
        focused: bool,
        value: &str,
        composition: Option<(usize, usize)>,
        multiline: bool,
    ) -> crate::layout::EditView {
        if !focused {
            return crate::layout::EditView::default();
        }
        let len = value.chars().count();
        let caret = self.state.caret.min(len);
        let anchor = self.state.anchor.min(len);
        let (a, b) = (caret.min(anchor), caret.max(anchor));
        let n = composition.map_or(0, |(s, e)| e - s);
        if let Some(h) = self.hit(key) {
            let path = self
                .nodes
                .iter()
                .position(|n| n.key == key)
                .map(|i| self.path_of(i))
                .unwrap_or_default();
            self.text_target = Some(TextTarget {
                path,
                rect: h.full,
                multiline,
                selection: value.chars().skip(a).take(b - a).collect(),
                composing: n > 0,
            });
        }
        crate::layout::EditView {
            focused: true,
            caret: caret + n,
            selection: (a, b),
            composition,
            scroll: self.state.edit_scroll,
            caret_on: self.caret_shown(),
        }
    }

    /// A tooltip for the element just built: shown below it (above when there
    /// is no room) once the pointer has rested on it for `motion.duration.slow`.
    /// Only interactive elements are hovered, so anchor it to a button or a control.
    pub fn tooltip(&mut self, text: &str) {
        let anchor = self.last_closed;
        if anchor == NONE {
            self.warnings.push(
                "tooltip with no element before it: call ui.tooltip() right after the control"
                    .into(),
            );
            return;
        }
        let delay =
            (self.theme.durations_ms.slow * self.ticks_per_second as f32 / 1000.0).ceil() as u32;
        if self.state.hot != self.nodes[anchor].key || self.state.hover_ticks < delay {
            return;
        }
        let t = self.theme;
        let key = child_key(self.nodes[anchor].key, "tooltip");
        let mut node = Node::new(
            key,
            "tooltip".into(),
            NONE,
            Element {
                padding: Insets::xy(t.space.sm, t.space.xs),
                background: Some(t.palette.panel_raised),
                radius: t.radius.control,
                skin: Some(StylePart::Tooltip),
                ..Element::default()
            },
        );
        let mut content = self.text_content(text, t.type_ramp.meta, t.palette.ink);
        content.wrap = false;
        node.content = Content::Text(content);
        node.widget = Widget::Tooltip;
        node.probe_text = text.to_owned();
        node.anchor = anchor;
        node.float = Float::Tooltip;
        let index = self.nodes.len();
        self.nodes.push(node);
        self.kinds.push(("tooltip", false));
        self.floating.push(index);
    }

    /// Opens a layer outside the tree (an anchored HUD group, a dialog's
    /// backdrop, a menu, the toast stack): it lays out and draws over the
    /// tree in its rank, and its children follow until [`Ui::close`]. Its
    /// path is its own id; `anchor` is the node it is placed against.
    pub(crate) fn open_layer(
        &mut self,
        kind: &'static str,
        id: Id,
        el: Element,
        float: Float,
        anchor: usize,
    ) -> usize {
        if !self.building {
            self.warnings
                .push("an element was added outside begin/end; call ui.begin() first".into());
            self.begin(InputFrame::default(), self.viewport);
        }
        let mut local = id.as_str().replace('/', "_");
        if local.is_empty() {
            local = format!("{kind}#{}", self.floating.len());
        }
        let taken = |s: &Ui, l: &str| s.floating.iter().any(|&f| s.nodes[f].id == l);
        if taken(self, &local) {
            let mut n = 1;
            while taken(self, &format!("{local}~{n}")) {
                n += 1;
            }
            let renamed = format!("{local}~{n}");
            self.warnings.push(format!(
                "duplicate layer id '{local}'; this one is '{renamed}': give each anchored group, dialog and menu its own id"
            ));
            local = renamed;
        }
        let key = child_key(self.nodes[0].key, &local);
        let mut node = Node::new(key, local, NONE, el);
        node.float = float;
        node.anchor = anchor;
        let index = self.nodes.len();
        self.nodes.push(node);
        self.kinds.push((kind, false));
        self.floating.push(index);
        self.stack.push(index);
        index
    }

    /// Lays out the frame, updates the interaction state and returns the frame.
    pub fn end(&mut self) -> &UiFrame {
        if !self.building {
            self.warnings
                .push("end without begin: call ui.begin() at the start of the tick".into());
            return &self.frame;
        }
        while self.stack.len() > 1 {
            let i = self.stack.pop().unwrap_or(0);
            self.warnings.push(format!(
                "'{}' was not closed; every ui.open() takes one ui.close()",
                self.path_of(i)
            ));
        }
        self.build_toasts();
        self.building = false;
        // Hidden slots (a closed dialog, a finished transition) take no room.
        for i in 0..self.nodes.len() {
            if self.nodes[i].hidden {
                continue;
            }
            let kids = std::mem::take(&mut self.nodes[i].children);
            let kept: Vec<usize> = kids
                .into_iter()
                .filter(|&c| !self.nodes[c].hidden)
                .collect();
            self.nodes[i].children = kept;
        }
        let mut floating: Vec<usize> = self
            .floating
            .iter()
            .copied()
            .filter(|&f| !self.nodes[f].hidden)
            .collect();
        floating.sort_by_key(|&f| self.nodes[f].float.rank());
        self.floating = floating;

        let vp = self.viewport;
        let dpr = vp.dpr();
        let screen = Rect::new(0.0, 0.0, vp.width, vp.height);
        let scroll = self.state.scroll.clone();
        let scroll_x = self.state.scroll_x.clone();
        let offset = |k: u64| {
            (
                scroll_x.get(&k).copied().unwrap_or(0.0),
                scroll.get(&k).copied().unwrap_or(0.0),
            )
        };
        layout(
            &mut self.nodes,
            0,
            screen,
            &self.fonts,
            &offset,
            dpr,
            screen,
        );
        let safe = Rect::new(
            vp.safe_area.left,
            vp.safe_area.top,
            vp.width - vp.safe_area.horizontal(),
            vp.height - vp.safe_area.vertical(),
        );
        // The host's control, where the layout stands before a right-to-left
        // mirror (which moves a top-left group to the top right).
        let covered = {
            let c = vp.covered;
            if self.direction == og_i18n::Direction::Rtl {
                Rect::new(vp.width - c.x - c.width, c.y, c.width, c.height)
            } else {
                c
            }
        };
        let gap = self.theme.space.xs;
        // The anchored groups first, each laid out in its area and placed
        // once all of them are known (so one the host's control moves keeps
        // off the others), then the other layers in their order (a popover
        // reads its anchor's final place).
        let (groups, rest): (Vec<usize>, Vec<usize>) = self
            .floating
            .iter()
            .partition(|&&f| matches!(self.nodes[f].float, Float::Anchored { .. }));
        let mut spots: Vec<(usize, Rect)> = Vec::new();
        for f in groups.into_iter().chain([NONE]).chain(rest) {
            // NONE marks the groups' end: place them, then go on to the rest.
            if f == NONE {
                if !covered.is_empty() {
                    for k in 0..spots.len() {
                        let (g, r) = spots[k];
                        let Float::Anchored { anchor, margin, .. } = self.nodes[g].float else {
                            continue;
                        };
                        let others: Vec<Rect> = spots
                            .iter()
                            .enumerate()
                            .filter(|&(j, _)| j != k)
                            .map(|(_, &(_, o))| o)
                            .collect();
                        spots[k].1 = keep_out(r, anchor, covered, margin.max(gap), safe, &others);
                    }
                }
                for &(g, placed) in &spots {
                    self.nodes[g].el.width = Size::Fixed(placed.width);
                    self.nodes[g].el.height = Size::Fixed(placed.height);
                    layout(
                        &mut self.nodes,
                        g,
                        placed,
                        &self.fonts,
                        &offset,
                        dpr,
                        screen,
                    );
                }
                continue;
            }
            let float = self.nodes[f].float;
            if float == Float::Modal {
                // The backdrop covers the surface; the safe area pads its content.
                self.nodes[f].el.width = Size::Fixed(vp.width);
                self.nodes[f].el.height = Size::Fixed(vp.height);
                if self.nodes[f].backdrop {
                    self.nodes[f].el.padding = self.nodes[f].el.padding.add(&vp.safe_area);
                }
                layout(
                    &mut self.nodes,
                    f,
                    screen,
                    &self.fonts,
                    &offset,
                    dpr,
                    screen,
                );
                continue;
            }
            let area = match float {
                Float::Anchored { margin, .. } | Float::Toasts { margin, .. } => Rect::new(
                    safe.x + margin,
                    safe.y + margin,
                    (safe.width - 2.0 * margin).max(0.0),
                    (safe.height - 2.0 * margin).max(0.0),
                ),
                _ => safe,
            };
            layout(&mut self.nodes, f, area, &self.fonts, &offset, dpr, screen);
            let (w, h) = (self.nodes[f].w, self.nodes[f].h);
            let (x, y) = match float {
                Float::Anchored { anchor, dx, dy, .. } => {
                    let (ax, ay) = anchor.fractions();
                    (
                        area.x + (area.width - w) * ax + dx,
                        area.y + (area.height - h) * ay + dy,
                    )
                }
                Float::Toasts { anchor, .. } => {
                    let (ax, ay) = anchor.fractions();
                    (
                        area.x + (area.width - w) * ax,
                        area.y + (area.height - h) * ay,
                    )
                }
                _ => {
                    let a = self
                        .nodes
                        .get(self.nodes[f].anchor)
                        .map_or(safe, |n| n.rect);
                    let x = if float == Float::Popover {
                        a.x.min(safe.right() - w).max(safe.x)
                    } else {
                        (a.x + a.width / 2.0 - w / 2.0)
                            .min(safe.right() - w)
                            .max(safe.x)
                    };
                    let mut y = a.bottom() + gap;
                    if y + h > safe.bottom() {
                        y = (a.y - gap - h).max(safe.y);
                    }
                    (x, y)
                }
            };
            let placed = Rect::new(x, y, w, h);
            if matches!(float, Float::Anchored { .. }) {
                spots.push((f, placed));
                continue;
            }
            self.nodes[f].el.width = Size::Fixed(w);
            self.nodes[f].el.height = Size::Fixed(h);
            layout(
                &mut self.nodes,
                f,
                placed,
                &self.fonts,
                &offset,
                dpr,
                screen,
            );
        }

        let mut order = preorder(&self.nodes, 0);
        for &f in &self.floating {
            order.extend(preorder(&self.nodes, f));
        }
        if self.direction == og_i18n::Direction::Rtl {
            // Right to left: the whole layout mirrored about the surface's centre.
            for &i in &order {
                let r = &mut self.nodes[i].rect;
                r.x = vp.width - r.x - r.width;
            }
        }
        self.apply_shakes(&order);
        self.apply_stagger(&order);
        self.apply_fx(&order);

        // Interaction state for the next tick.
        let mut present: BTreeMap<u64, usize> =
            order.iter().map(|&i| (self.nodes[i].key, i)).collect();
        // A rich text's links are controls of their own, inside its node.
        for &i in &order {
            for l in &self.nodes[i].links {
                present.insert(child_key(self.nodes[i].key, l), i);
            }
        }
        self.state
            .scroll
            .retain(|k, _| present.get(k).is_some_and(|&i| self.nodes[i].el.scroll));
        for (k, v) in self.state.scroll.iter_mut() {
            *v = v.clamp(0.0, self.nodes[present[k]].max_scroll());
        }
        self.state
            .scroll_x
            .retain(|k, _| present.get(k).is_some_and(|&i| self.nodes[i].el.scroll_x));
        for (k, v) in self.state.scroll_x.iter_mut() {
            *v = v.clamp(0.0, self.nodes[present[k]].max_scroll_x());
        }
        // Each scroll container's ranges and bar lengths, as drawn.
        let min_bar = self.theme.space.xl;
        self.state.scroll_info = order
            .iter()
            .filter(|&&i| self.nodes[i].scrolls())
            .map(|&i| {
                let n = &self.nodes[i];
                let range = Vec2::new(n.max_scroll_x(), n.max_scroll());
                let bar = Vec2::new(
                    crate::scroll::bar_len(
                        n.rect.width,
                        n.w - n.el.padding.horizontal(),
                        n.content_w,
                        min_bar,
                    ),
                    crate::scroll::bar_len(
                        n.rect.height,
                        n.h - n.el.padding.vertical(),
                        n.content_h,
                        min_bar,
                    ),
                );
                (n.key, crate::scroll::ScrollInfo { range, bar })
            })
            .collect();
        if let Some((k, _)) = self.state.fling
            && !present.contains_key(&k)
        {
            self.state.fling = None;
        }
        if !self.input.down || !present.contains_key(&self.state.active) {
            self.state.active = 0;
        }
        let touched = std::mem::take(&mut self.touched);
        self.state.anims.retain(|k, _| touched.contains(k));
        self.state.screens.retain(|k, _| touched.contains(k));
        self.touched = touched;

        // Scopes: the nearest scope around each node, by walking the order.
        let mut scope_of = vec![0u64; self.nodes.len()];
        let mut scroller_of = vec![0u64; self.nodes.len()];
        for &i in &order {
            let n = &self.nodes[i];
            let up = if n.parent == NONE { NONE } else { n.parent };
            let (ps, pr) = if up == NONE {
                (0, 0)
            } else {
                (scope_of[up], scroller_of[up])
            };
            scope_of[i] = if n.scope { n.key } else { ps };
            scroller_of[i] = if up != NONE && self.nodes[up].scrolls() {
                self.nodes[up].key
            } else {
                pr
            };
        }
        let mut hits = Vec::new();
        let mut elements = Vec::new();
        // The accessibility tree: each element's key (for its focus states)
        // and the nodes with a role, with the paths their children name.
        let mut element_keys: Vec<u64> = Vec::new();
        let mut tree: Vec<(u64, crate::a11y::A11yNode)> = Vec::new();
        let mut tree_path: BTreeMap<usize, String> = BTreeMap::new();
        let mut top_scope = 0u64;
        let mut top_locked = false;
        for &i in &order[1..] {
            let n = &self.nodes[i];
            let (role, name, description, value, states) = self.a11y_info(i);
            let parent_path = {
                let mut up = if n.parent == NONE { n.anchor } else { n.parent };
                loop {
                    if up == NONE || up == 0 {
                        break String::new();
                    }
                    if let Some(p) = tree_path.get(&up) {
                        break p.clone();
                    }
                    up = if self.nodes[up].parent == NONE {
                        self.nodes[up].anchor
                    } else {
                        self.nodes[up].parent
                    };
                }
            };
            let path = if role.is_empty() {
                String::new()
            } else {
                self.path_of(i)
            };
            if !role.is_empty() {
                tree_path.insert(i, path.clone());
            }
            if n.scope && !n.inert {
                top_scope = n.key;
                top_locked = n.locked;
            }
            let visible = n.rect.intersect(&n.clip);
            // Scrolled or clipped out of view: not on screen, not listed. A
            // zero-size element (an empty spacer) is listed where it sits.
            let (r, c) = (&n.rect, &n.clip);
            let outside =
                r.x > c.right() || r.right() < c.x || r.y > c.bottom() || r.bottom() < c.y;
            let mut flags = 0u8;
            if n.scrolls() && !n.el.interactive {
                flags |= HIT_SCROLL;
            }
            if n.el.interactive || n.backdrop {
                flags |= HIT_POINTER;
            }
            if n.focusable {
                flags |= HIT_TEXT;
            }
            if n.multiline {
                flags |= HIT_MULTILINE;
            }
            if n.nav {
                flags |= HIT_NAV;
            }
            if n.adjust {
                flags |= HIT_ADJUST;
            }
            if n.scope {
                flags |= HIT_SCOPE;
            }
            let hit = |rect: Rect, flags: u8| Hit {
                key: n.key,
                rect,
                full: *r,
                flags,
                scope: scope_of[i],
                scroller: scroller_of[i],
            };
            let range = crate::a11y::range_of(n).filter(|_| !role.is_empty());
            let node = |states: Vec<String>| crate::a11y::A11yNode {
                path: path.clone(),
                parent: parent_path.clone(),
                role: role.to_owned(),
                name: name.clone(),
                description: description.clone(),
                value: value.clone(),
                states,
                value_min: range.and_then(|r| r.min),
                value_max: range.and_then(|r| r.max),
                value_now: range.map(|r| r.now),
                x: r.x,
                y: r.y,
                width: r.width,
                height: r.height,
            };
            if outside || (visible.is_empty() && !r.is_empty()) {
                // Navigation can still reach a control just out of a list's view.
                if n.nav && !n.inert && scroller_of[i] != 0 {
                    hits.push(hit(*r, (flags & (HIT_NAV | HIT_ADJUST)) | HIT_OFFSCREEN));
                }
                if !role.is_empty() {
                    let mut st = states.clone();
                    st.push("offscreen".to_owned());
                    tree.push((n.key, node(st)));
                }
                continue;
            }
            if !role.is_empty() {
                tree.push((n.key, node(states.clone())));
            }
            if !visible.is_empty() && flags != 0 && !n.inert {
                hits.push(hit(visible, flags));
            }
            let parent = if n.parent == NONE {
                self.nodes
                    .get(n.anchor)
                    .map_or(String::new(), |a| a.id.clone())
            } else if n.parent == 0 {
                String::new()
            } else {
                self.nodes[n.parent].id.clone()
            };
            element_keys.push(n.key);
            elements.push(UiElement {
                id: n.id.clone(),
                parent,
                kind: self.kinds.get(i).map_or("", |k| k.0).to_string(),
                disabled: n.disabled,
                clipped: visible.width + 0.01 < r.width || visible.height + 0.01 < r.height,
                text: n.probe_text.clone(),
                role: role.to_owned(),
                name: name.clone(),
                value: value.clone(),
                states: states.clone(),
                x: visible.x,
                y: visible.y,
                width: visible.width,
                height: visible.height,
            });
            // A rich text's links: children for the probe, controls for input.
            if !n.links.is_empty() {
                let Content::Rich(rc) = &n.content else {
                    unreachable!("links come from rich text")
                };
                for (l, lr) in self.rich_links(i) {
                    let shown = lr.intersect(&n.clip);
                    if shown.is_empty() {
                        continue;
                    }
                    if !n.inert {
                        hits.push(Hit {
                            key: child_key(n.key, &l),
                            rect: shown,
                            full: lr,
                            flags: HIT_POINTER | HIT_NAV,
                            scope: scope_of[i],
                            scroller: scroller_of[i],
                        });
                    }
                    let text: String = rc
                        .spans
                        .iter()
                        .filter(|s| s.link.as_deref() == Some(l.as_str()))
                        .map(|s| s.text.as_str())
                        .collect();
                    let link_path = format!("{}/{l}", self.path_of(i));
                    tree.push((
                        child_key(n.key, &l),
                        crate::a11y::A11yNode {
                            path: link_path,
                            parent: path.clone(),
                            role: "link".into(),
                            name: text.clone(),
                            description: String::new(),
                            value: String::new(),
                            states: Vec::new(),
                            x: lr.x,
                            y: lr.y,
                            width: lr.width,
                            height: lr.height,
                            ..crate::a11y::A11yNode::default()
                        },
                    ));
                    element_keys.push(child_key(n.key, &l));
                    elements.push(UiElement {
                        id: l,
                        parent: n.id.clone(),
                        kind: "link".into(),
                        disabled: false,
                        clipped: shown.width + 0.01 < lr.width || shown.height + 0.01 < lr.height,
                        name: text.clone(),
                        text,
                        role: "link".into(),
                        value: String::new(),
                        states: Vec::new(),
                        x: shown.x,
                        y: shown.y,
                        width: shown.width,
                        height: shown.height,
                    });
                }
            }
        }
        // Focus: kept while its control is on screen (or reachable); a scope
        // that opened takes it (its first control, or the one a widget asked
        // for); a scope that closed gives it back.
        let alive = |k: u64| hits.iter().any(|h: &Hit| h.key == k);
        let first_in = |scope: u64| {
            hits.iter()
                .find(|h| h.scope == scope && h.has(HIT_NAV) && !h.has(HIT_OFFSCREEN))
                .map_or(0, |h| h.key)
        };
        if self.request_focus != 0 && alive(self.request_focus) {
            self.state.focus = self.request_focus;
        }
        if top_scope != self.state.scope {
            if top_scope != 0 {
                let inside = hits
                    .iter()
                    .any(|h| h.key == self.state.focus && h.scope == top_scope);
                if !inside {
                    if self.state.scope == 0 {
                        self.state.focus_return = self.state.focus;
                    }
                    let req = self.request_focus;
                    self.state.focus =
                        if req != 0 && hits.iter().any(|h| h.key == req && h.scope == top_scope) {
                            req
                        } else {
                            first_in(top_scope)
                        };
                }
            } else {
                if !alive(self.state.focus) || self.state.focus == 0 {
                    let back = self.state.focus_return;
                    self.state.focus = if alive(back) { back } else { 0 };
                }
                self.state.focus_return = 0;
            }
        }
        if !alive(self.state.focus) {
            self.state.focus = 0;
        }
        // Pads 1 to 3: a focus whose control left is dropped; one outside a
        // scope that opened moves to its first control.
        for k in 0..3 {
            let f = self.state.pad_focus[k];
            let inside =
                |f: u64| top_scope == 0 || hits.iter().any(|h| h.key == f && h.scope == top_scope);
            if (k as u32) + 1 >= self.players {
                self.state.pad_focus[k] = 0;
            } else if f != 0 && (!alive(f) || !inside(f)) {
                self.state.pad_focus[k] = if top_scope != 0 {
                    first_in(top_scope)
                } else {
                    0
                };
            }
        }
        self.state.players = self.players as u8;
        self.state.scope = top_scope;
        self.state.scope_locked = top_locked;
        self.state.nav_off = !self.nav_on;
        // Focus order: the controls keys and pads reach now, in the tree's
        // order (the order navigation walks: `nav::focusables`).
        let reach: Vec<u64> = hits
            .iter()
            .filter(|h| h.has(HIT_NAV | HIT_TEXT) && (top_scope == 0 || h.scope == top_scope))
            .map(|h| h.key)
            .collect();
        let mark = |key: u64, states: &mut Vec<String>| {
            if reach.contains(&key) {
                states.push("focusable".to_owned());
            }
            if key != 0 && key == self.state.focus {
                states.push("focused".to_owned());
            }
        };
        for (e, &key) in elements.iter_mut().zip(&element_keys) {
            if !e.role.is_empty() {
                mark(key, &mut e.states);
            }
        }
        for (key, node) in &mut tree {
            mark(*key, &mut node.states);
        }
        let focus_order: Vec<String> = reach
            .iter()
            .filter_map(|k| {
                tree.iter()
                    .find(|(t, _)| t == k)
                    .map(|(_, n)| n.path.clone())
            })
            .collect();
        let a11y = crate::a11y::A11yTree {
            focused: tree
                .iter()
                .find(|(k, _)| *k != 0 && *k == self.state.focus)
                .map_or(String::new(), |(_, n)| n.path.clone()),
            nodes: tree.into_iter().map(|(_, n)| n).collect(),
            focus_order,
        };
        self.state.hits = hits;

        let mut cmds = self.commands(&order);
        if let Some(c) = self.cover {
            self.cover_commands(&c, &mut cmds);
        }
        let mut focus_paths: [String; 4] = Default::default();
        let focused = [
            self.state.focus,
            self.state.pad_focus[0],
            self.state.pad_focus[1],
            self.state.pad_focus[2],
        ];
        for (slot, key) in focused.into_iter().enumerate() {
            if key != 0
                && let Some(&i) = present.get(&key)
            {
                focus_paths[slot] = self.path_of(i);
                if let Some(l) = self.nodes[i]
                    .links
                    .iter()
                    .find(|l| child_key(self.nodes[i].key, l) == key)
                {
                    focus_paths[slot] = format!("{}/{l}", focus_paths[slot]);
                }
            }
        }
        self.frame = UiFrame {
            viewport: vp,
            elements,
            items: assign_levels(cmds),
            warnings: std::mem::take(&mut self.warnings),
            focus_paths,
            a11y,
            text_input: self.text_target.take(),
        };
        &self.frame
    }

    /// Staggered entrances: under a node with a stagger, the k-th drawn
    /// leaf (a label, a control, an icon) enters `k` steps after the first;
    /// a long list shortens the step so the cascade ends within one more
    /// entrance. The cascade is only seen: a control takes input as soon as
    /// the panel's own entrance (inert while it moves) has played, where it
    /// is drawn, not when its place in the cascade has.
    fn apply_stagger(&mut self, order: &[usize]) {
        let now = self.state.tick as f32;
        let (reduced, distance) = (self.theme.reduced_motion, self.theme.space.lg);
        for &i in order {
            let Some(st) = self.nodes[i].stagger else {
                continue;
            };
            let leaves: Vec<usize> = preorder(&self.nodes, i)
                .into_iter()
                .skip(1)
                .filter(|&j| {
                    let n = &self.nodes[j];
                    n.children.is_empty()
                        && !matches!(n.widget, Widget::Spacer)
                        && (!matches!(n.content, Content::None)
                            || n.el.background.is_some()
                            || !matches!(n.widget, Widget::None)
                            || !n.shapes.is_empty())
                })
                .collect();
            // The whole cascade takes at most one entrance more.
            let step = if leaves.len() > 1 {
                st.step.min(st.ticks / (leaves.len() - 1) as f32)
            } else {
                st.step
            };
            for (k, &j) in leaves.iter().enumerate() {
                let start = st.since as f32 + step * k as f32;
                let p = ((now - start + 1.0) / st.ticks).clamp(0.0, 1.0);
                if p >= 1.0 || self.nodes[j].fx.is_some() {
                    continue;
                }
                self.nodes[j].fx = Some(crate::motion::fx(
                    st.kind,
                    crate::motion::ease_out(p),
                    distance,
                    reduced,
                ));
            }
        }
    }

    /// The tick `key` was first seen on (kept while it is asked every tick).
    pub(crate) fn since(&mut self, key: u64) -> u64 {
        let fresh = !self.has_anim(key);
        let mut a = self.anim(key);
        if fresh {
            a.t = self.state.tick;
        }
        self.set_anim(key, a);
        a.t
    }

    /// Applies each transition to its subtree: moves and scales the
    /// rectangles (and the clips its own scroll containers make), multiplies
    /// opacity, and makes the subtree inert while it moves.
    fn apply_fx(&mut self, order: &[usize]) {
        let pos: BTreeMap<usize, usize> = order.iter().enumerate().map(|(k, &i)| (i, k)).collect();
        for k in 0..order.len() {
            let i = order[k];
            let parent = self.nodes[i].parent;
            let (alpha, scale, inert) = if parent == NONE {
                (1.0, 1.0, false)
            } else {
                let p = &self.nodes[parent];
                (p.alpha, p.scale, p.inert)
            };
            let n = &mut self.nodes[i];
            n.alpha = alpha;
            n.scale = scale;
            n.inert = inert;
            let Some(fx) = n.fx else { continue };
            n.alpha *= fx.alpha;
            n.scale *= fx.scale;
            n.inert |= fx.inert;
            if fx == Fx::default() {
                continue;
            }
            // The subtree is the contiguous run of the preorder after i.
            let (px, py) = n.rect.center();
            let mut end = k + 1;
            let last = {
                let mut j = i;
                while let Some(&c) = self.nodes[j].children.last() {
                    j = c;
                }
                j
            };
            if last != i {
                end = pos[&last] + 1;
            }
            let tf = |r: Rect| {
                Rect::new(
                    px + (r.x - px) * fx.scale + fx.dx,
                    py + (r.y - py) * fx.scale + fx.dy,
                    r.width * fx.scale,
                    r.height * fx.scale,
                )
            };
            let inside: std::collections::BTreeSet<usize> = order[k..end].iter().copied().collect();
            for &j in &order[k..end] {
                let n = &mut self.nodes[j];
                n.rect = tf(n.rect);
                if n.clip_src != NONE && inside.contains(&n.clip_src) {
                    n.clip = tf(n.clip);
                }
            }
        }
    }

    fn commands(&self, order: &[usize]) -> Vec<DrawCmd> {
        let t = &self.theme;
        let p = &t.palette;
        let dpr = self.viewport.dpr();
        let snap = |v: f32| (v * dpr).round() / dpr;
        let mut cmds = Vec::new();
        // Enter/leave walk so scroll indicators draw over their content.
        let mut leave: Vec<(usize, usize)> = Vec::new(); // (node, depth of its subtree end)
        let depth_end = |i: usize| -> usize {
            // The last node of i's subtree in preorder.
            let mut j = i;
            while let Some(&last) = self.nodes[j].children.last() {
                j = last;
            }
            j
        };
        for &i in order {
            let n = &self.nodes[i];
            if i != 0 || n.el.background.is_some() {
                self.node_commands(i, &mut cmds, &snap);
            }
            if n.scrolls() {
                leave.push((i, depth_end(i)));
            }
            while let Some(&(s, end)) = leave.last() {
                if end != i {
                    break;
                }
                leave.pop();
                let sn = &self.nodes[s];
                let max_x = sn.max_scroll_x();
                if max_x > 0.0 {
                    let inner_w = sn.w - sn.el.padding.horizontal();
                    let track = sn.rect;
                    let bar_w =
                        crate::scroll::bar_len(track.width, inner_w, sn.content_w, t.space.xl);
                    let off = self.state.scroll_x.get(&sn.key).copied().unwrap_or(0.0);
                    let x = crate::scroll::bar_start(track.x, track.width, bar_w, off, max_x);
                    let h = t.space.xs;
                    cmds.push(DrawCmd::Quad {
                        rect: Rect::new(x, track.bottom() - h - t.space.xxs, bar_w, h).snapped(dpr),
                        radius: t.radius.bar,
                        color: p.control_strong.linear(),
                        clip: sn.clip.intersect(&sn.rect),
                    });
                }
                let max = sn.max_scroll();
                if max > 0.0 {
                    let inner_h = sn.h - sn.el.padding.vertical();
                    let track = sn.rect;
                    let bar_h = (track.height * inner_h / sn.content_h).max(t.space.xl);
                    let off = self
                        .state
                        .scroll
                        .get(&sn.key)
                        .copied()
                        .unwrap_or(0.0)
                        .clamp(0.0, max);
                    let y = track.y + (track.height - bar_h) * off / max;
                    let w = t.space.xs;
                    cmds.push(DrawCmd::Quad {
                        rect: Rect::new(track.right() - w - t.space.xxs, y, w, bar_h).snapped(dpr),
                        radius: t.radius.bar,
                        color: p.control_strong.linear(),
                        clip: sn.clip.intersect(&sn.rect),
                    });
                }
            }
        }
        cmds
    }

    fn node_commands(&self, i: usize, cmds: &mut Vec<DrawCmd>, snap: &dyn Fn(f32) -> f32) {
        let t = &self.theme;
        let p = &t.palette;
        let n = &self.nodes[i];
        let rect = n.rect;
        let clip = n.clip;
        let dpr = self.viewport.dpr();
        let alpha = n.alpha;
        let quad = |rect: Rect, radius: f32, color: Srgba| DrawCmd::Quad {
            rect,
            radius,
            color: color.fade(alpha).linear(),
            clip,
        };
        if n.pad_rings != 0 {
            // Pads 1 to 3: a ring each in the player's colour, outside where
            // pad 0's ring goes, outermost first (each cut from the next by
            // what is behind).
            let behind = self.behind(i);
            let w = t.space.xxs;
            for k in (0..3).rev().filter(|k| n.pad_rings & (1 << k) != 0) {
                let at = w * (2.0 * k as f32 + 3.0);
                let color = [p.info, p.success, p.warn][k];
                cmds.push(quad(rect.outset(at + w), n.el.radius + at + w, color));
                cmds.push(quad(rect.outset(at), n.el.radius + at, behind));
            }
        }
        let field_ring = match &n.widget {
            Widget::Field(e) | Widget::TextArea { edit: e, .. } => e.focused,
            _ => false,
        };
        let outline = t.outline && (n.nav || n.focusable) && n.el.background.is_some();
        if field_ring || n.ring || outline {
            // A ring, a gap of what is behind, then the control.
            let behind = self.behind(i);
            let w = if field_ring || n.ring {
                t.space.xxs
            } else {
                1.0
            };
            let color = if field_ring || n.ring {
                p.focus
            } else {
                p.line_strong
            };
            let gap = if field_ring || !n.ring {
                0.0
            } else {
                t.space.xxs
            };
            cmds.push(quad(rect.outset(w + gap), n.el.radius + w + gap, color));
            // Cut always: a field's skinned frame is not wholly opaque, and
            // the ring under it tinted its text's ground (the scifi pack's
            // focused placeholder read 3.5:1 on primary-tinted navy).
            cmds.push(quad(rect.outset(gap), n.el.radius + gap, behind));
        }
        if let Some(bg) = n.el.background {
            let (pressed, press) = match n.widget {
                Widget::Button { pressed, press } => (pressed, press),
                _ => (false, 0.0),
            };
            match n
                .el
                .skin
                .and_then(|part| t.skin.slice(part).map(|s| (part, s)))
            {
                Some((part, slice)) => {
                    let r = match (t.skin.press, pressed) {
                        // The squash follows the spring, in and back out.
                        (Some(pr), _) if press != 0.0 => scaled_xy(
                            rect,
                            1.0 + (pr.sx - 1.0) * press,
                            1.0 + (pr.sy - 1.0) * press,
                        ),
                        (None, true) => rect.scaled(t.press_scale),
                        _ => rect,
                    };
                    self.dress(part, slice, r, n.el.radius, bg, alpha, clip, cmds);
                }
                None => {
                    let r = if pressed {
                        rect.scaled(t.press_scale)
                    } else {
                        rect
                    };
                    cmds.push(quad(r, n.el.radius, bg));
                }
            }
        }
        if self.kinds[i].0 == "divider"
            && let Some(o) = t.skin.divider.filter(|_| t.skin.atlas.is_some())
        {
            let (cx, cy) = rect.center();
            self.ornament(
                o,
                Rect::new(cx - o.w / 2.0, cy - o.h / 2.0, o.w, o.h),
                false,
                false,
                alpha,
                clip,
                cmds,
            );
        }
        let track = t.skin.slice(StylePart::Track);
        match n.widget {
            Widget::Progress { value } if track.is_some() => {
                let fill = t.skin.slice(StylePart::Fill);
                if let Some(s) = track {
                    self.dress(
                        StylePart::Track,
                        s,
                        rect,
                        t.radius.bar,
                        p.control_strong,
                        alpha,
                        clip,
                        cmds,
                    );
                }
                if let (Some(s), true) = (fill, value > 0.0) {
                    let min = s.border[0] + s.border[2];
                    let w = (rect.width * value).max(min.min(rect.width));
                    let r = Rect::new(rect.x, rect.y, w, rect.height).snapped(dpr);
                    self.dress(
                        StylePart::Fill,
                        s,
                        r,
                        t.radius.bar,
                        p.primary,
                        alpha,
                        clip,
                        cmds,
                    );
                }
            }
            Widget::Progress { value } => {
                cmds.push(quad(rect, t.radius.bar, p.control_strong));
                if value > 0.0 {
                    let w = (rect.width * value).max(t.radius.bar * 2.0);
                    cmds.push(quad(
                        Rect::new(rect.x, rect.y, w, rect.height).snapped(dpr),
                        t.radius.bar,
                        p.primary,
                    ));
                }
            }
            Widget::Slider { t: v, active } => {
                let thumb = self.thumb();
                let track_h = t.space.xs;
                let cy = rect.y + rect.height / 2.0;
                let track = Rect::new(rect.x, cy - track_h / 2.0, rect.width, track_h).snapped(dpr);
                cmds.push(quad(track, t.radius.bar, p.control_strong));
                let tx = rect.x + v * (rect.width - thumb).max(0.0);
                cmds.push(quad(
                    Rect::new(rect.x, track.y, tx + thumb / 2.0 - rect.x, track_h).snapped(dpr),
                    t.radius.bar,
                    p.primary,
                ));
                let mut knob = Rect::new(tx, cy - thumb / 2.0, thumb, thumb);
                if active {
                    knob = knob.scaled(t.press_scale);
                }
                cmds.push(quad(knob.snapped(dpr), t.radius.control, p.knob));
            }
            Widget::Toggle { on } => {
                let (sw, sh) = self.switch_size();
                let track =
                    Rect::new(rect.x, rect.y + (rect.height - sh) / 2.0, sw, sh).snapped(dpr);
                cmds.push(quad(
                    track,
                    t.radius.control,
                    if on { p.primary } else { p.control_strong },
                ));
                let inset = t.space.xxs;
                let k = sh - 2.0 * inset;
                let kx = if on {
                    track.right() - inset - k
                } else {
                    track.x + inset
                };
                cmds.push(quad(
                    Rect::new(kx, track.y + inset, k, k).snapped(dpr),
                    (t.radius.control - inset).max(0.0),
                    p.knob,
                ));
            }
            _ => {}
        }
        if matches!(n.widget, Widget::TextArea { .. }) {
            self.text_area_commands(i, rect, clip, cmds);
        }
        match &n.content {
            Content::Text(tc) => {
                let font = self.fonts.for_text(tc.font, &tc.text);
                if font.handle.is_some() {
                    let m = &font.metrics;
                    let sk = n.scale;
                    let size = tc.step.size * sk;
                    let lh = tc.line_height() * sk;
                    let pad = n.el.padding;
                    let inner = Rect::new(
                        rect.x + pad.left * sk,
                        rect.y + pad.top * sk,
                        rect.width - pad.horizontal() * sk,
                        rect.height - pad.vertical() * sk,
                    );
                    let block = lh * n.lines.len() as f32;
                    let top = inner.y + ((inner.height - block) / 2.0).max(0.0);
                    // Text overflows its own box (a snapped edge must not eat a glyph)
                    // except in a field, which scrolls its text and clips.
                    let text_clip = match n.widget {
                        Widget::Field { .. } => clip.intersect(&rect),
                        _ => clip,
                    };
                    let field = match &n.widget {
                        Widget::Field(e) => Some(*e),
                        _ => None,
                    };
                    let mut field_x = inner.x;
                    for (k, line) in n.lines.iter().enumerate() {
                        let ly = top + k as f32 * lh;
                        let free = inner.width - line.width * sk;
                        // Right to left, start is the right edge.
                        let rtl = self.direction == og_i18n::Direction::Rtl;
                        let x = inner.x
                            + match (tc.align, rtl) {
                                (TextAlign::Start, false) | (TextAlign::End, true) => 0.0,
                                (TextAlign::Center, _) => free / 2.0,
                                (TextAlign::End, false) | (TextAlign::Start, true) => free,
                            };
                        // A focused field: scrolled, its selection under the text.
                        let x = match &field {
                            Some(e) if e.focused && tc.color != p.ink_faint => {
                                let x = x - e.scroll;
                                self.edit_marks(
                                    &EditMarks {
                                        m,
                                        text: &line.text,
                                        x,
                                        top: ly,
                                        line_h: lh,
                                        size,
                                        tracking: tc.tracking(),
                                        first: 0,
                                        edit: e,
                                        clip: text_clip,
                                        alpha,
                                        under: true,
                                        caret_here: true,
                                    },
                                    cmds,
                                );
                                field_x = x;
                                x
                            }
                            _ => x,
                        };
                        // Bidi runs in visual order, shaped runs shaped, one
                        // piece per font of a mixed text (crate::text).
                        let pieces =
                            crate::text::pieces(font, &line.text, self.direction, tc.tracking());
                        let baseline = m.baseline(&line.text, ly, lh, size);
                        let bounds = Rect::new(x, ly, line.width * sk, lh);
                        let role = tc.font.skin_index();
                        let tracking = tc.tracking();
                        // The skin's thickening for the role (`textWeight`, device
                        // pixels: what a stroke between two pixels lacks), shadow included.
                        let weight = t.skin.text_weights[role] * sk / dpr;
                        // A skin's text glow: the font's distance field, under the glyphs.
                        let glow = t.skin.text_glows[role].map(|g| {
                            let c = g.ink.color(p).fade(g.alpha * tc.color.alpha * alpha);
                            (c.linear(), TEXT_GLOW_REACH * g.size * sk)
                        });
                        let one = pieces.len() == 1;
                        let text =
                            |dx: f32, dy: f32, color: Srgba, glow, cmds: &mut Vec<DrawCmd>| {
                                for pc in &pieces {
                                    let Some(font) = pc.font else { continue };
                                    let ox = pc.dx * size;
                                    cmds.push(DrawCmd::Text {
                                        x: snap(x + dx) + ox,
                                        baseline: snap(baseline + dy),
                                        size,
                                        text: pc.text.clone(),
                                        color: color.fade(alpha).linear(),
                                        font,
                                        bounds: Rect::new(
                                            bounds.x + dx + ox,
                                            bounds.y + dy,
                                            if one { bounds.width } else { pc.width * size },
                                            bounds.height,
                                        ),
                                        clip: text_clip,
                                        tracking,
                                        weight,
                                        glow,
                                        placements: pc.placements.clone(),
                                    });
                                }
                            };
                        if let Some(sh) = t.skin.text_shadows[role] {
                            let c = sh.color.fade(tc.color.alpha);
                            text(sh.dx, sh.dy, c, None, cmds);
                            if sh.blur > 0.0 {
                                text(
                                    sh.dx + sh.blur / 2.0,
                                    sh.dy + sh.blur / 2.0,
                                    c.fade(0.5),
                                    None,
                                    cmds,
                                );
                            }
                        }
                        text(0.0, 0.0, tc.color, glow, cmds);
                        if n.heading
                            && k == 0
                            && let Some(o) = t.skin.flourish.filter(|_| t.skin.atlas.is_some())
                        {
                            let cy = ly + lh / 2.0;
                            let gap = o.offset + t.space.sm;
                            let (w, h) = (o.w * sk, o.h * sk);
                            let left = Rect::new(x - gap - w, cy - h / 2.0, w, h);
                            let right = Rect::new(x + line.width * sk + gap, cy - h / 2.0, w, h);
                            // Both sides of a centred heading; the side away from the edge otherwise.
                            if tc.align != TextAlign::Start {
                                self.ornament(o, left, false, false, alpha, clip, cmds);
                            }
                            if tc.align != TextAlign::End {
                                self.ornament(o, right, true, false, alpha, clip, cmds);
                            }
                        }
                    }
                    if let Some(e) = &field
                        && e.focused
                    {
                        // The composition's underline and the caret, over the text.
                        let text = if tc.color != p.ink_faint {
                            tc.text.as_str()
                        } else {
                            ""
                        };
                        // An empty field's caret sits at its start.
                        let x = if text.is_empty() {
                            if self.direction == og_i18n::Direction::Rtl {
                                inner.right()
                            } else {
                                inner.x
                            }
                        } else {
                            field_x
                        };
                        let ly = top;
                        self.edit_marks(
                            &EditMarks {
                                m,
                                text,
                                x,
                                top: ly,
                                line_h: lh,
                                size,
                                tracking: tc.tracking(),
                                first: 0,
                                edit: e,
                                clip: text_clip,
                                alpha,
                                under: false,
                                caret_here: true,
                            },
                            cmds,
                        );
                    }
                }
            }
            Content::Rich(rc) => self.rich_commands(i, rc, cmds),
            Content::Image { texture, uv, tint } => cmds.push(DrawCmd::Image {
                rect,
                texture: *texture,
                uv: *uv,
                color: tint.fade(alpha).linear(),
                clip,
            }),
            Content::None => {}
        }
        self.shape_commands(n, cmds);
    }

    pub(crate) fn edit_marks(&self, mk: &EditMarks<'_>, cmds: &mut Vec<DrawCmd>) {
        let p = &self.theme.palette;
        let dpr = self.viewport.dpr();
        let boxes = crate::edit::boxes(mk.m, mk.text, self.direction, mk.size, mk.tracking);
        let n = mk.text.chars().count();
        let span =
            |range: (usize, usize), cmds: &mut Vec<DrawCmd>, y: f32, h: f32, color: Srgba| {
                let (a, b) = (range.0.max(mk.first), range.1.min(mk.first + n));
                if a >= b {
                    return;
                }
                let mut runs: Vec<(f32, f32)> = Vec::new();
                for g in &boxes {
                    let k = g.index + mk.first;
                    if k < a || k >= b {
                        continue;
                    }
                    match runs.last_mut() {
                        Some(r) if (r.1 - g.x0).abs() < 0.01 => r.1 = g.x1,
                        _ => runs.push((g.x0, g.x1)),
                    }
                }
                for (x0, x1) in runs {
                    cmds.push(DrawCmd::Quad {
                        rect: Rect::new(mk.x + x0, y, x1 - x0, h).snapped(dpr),
                        radius: 0.0,
                        color: color.fade(mk.alpha).linear(),
                        clip: mk.clip,
                    });
                }
            };
        let e = mk.edit;
        let ch = mk.size + self.theme.space.xxs * 2.0;
        let cy = mk.top + (mk.line_h - ch) / 2.0;
        if mk.under {
            span(e.selection, cmds, cy, ch, p.control_strong);
            return;
        }
        if let Some(c) = e.composition {
            let th = (mk.size * 0.07).max(1.0 / dpr);
            span(c, cmds, cy + ch - th, th, p.ink);
        }
        if e.caret_on && mk.caret_here && e.caret >= mk.first && e.caret <= mk.first + n {
            let cx = mk.x + crate::edit::caret_x(&boxes, e.caret - mk.first);
            let caret_w = 1.0f32.max(1.0 / dpr);
            cmds.push(DrawCmd::Quad {
                rect: Rect::new(cx, cy, caret_w, ch).snapped(dpr),
                radius: 0.0,
                color: p.primary.fade(mk.alpha).linear(),
                clip: mk.clip,
            });
        }
    }

    /// A text area's lines: wrapped at its inner width, scrolled, clipped
    /// to it, each with its marks.
    fn text_area_commands(&self, i: usize, rect: Rect, clip: Rect, cmds: &mut Vec<DrawCmd>) {
        let n = &self.nodes[i];
        let Widget::TextArea {
            edit,
            text,
            placeholder,
            line_height,
        } = &n.widget
        else {
            return;
        };
        let t = &self.theme;
        let p = &t.palette;
        let step = t.type_ramp.body;
        let (shown, color) = if text.is_empty() {
            (placeholder.as_str(), p.ink_faint)
        } else {
            (text.as_str(), p.ink)
        };
        let font = self.fonts.for_text(FontRole::Body, shown);
        if font.handle.is_none() {
            return;
        }
        let m = &font.metrics;
        let sk = n.scale;
        let size = step.size * sk;
        let lh = line_height * sk;
        let tracking = crate::layout::step_tracking(&step, shown);
        let pad = n.el.padding;
        let inner = Rect::new(
            rect.x + pad.left * sk,
            rect.y + pad.top * sk,
            rect.width - pad.horizontal() * sk,
            rect.height - pad.vertical() * sk,
        );
        let clip = clip.intersect(&rect);
        let dpr = self.viewport.dpr();
        let snap = |v: f32| (v * dpr).round() / dpr;
        let chars: Vec<char> = shown.chars().collect();
        let lines = crate::edit::wrap(&chars, m, size, tracking, inner.width.max(1.0));
        let caret_line = lines
            .iter()
            .rposition(|l| l.0 <= edit.caret && edit.caret <= l.1)
            .unwrap_or(0);
        let rtl = self.direction == og_i18n::Direction::Rtl;
        let alpha = n.alpha;
        for (li, &(a, b)) in lines.iter().enumerate() {
            let top = inner.y + li as f32 * lh - edit.scroll * sk;
            if top + lh < inner.y - lh || top > inner.bottom() + lh {
                continue;
            }
            let line: String = chars[a..b].iter().collect();
            let w = m.measure_tracked(&line, size, tracking);
            let x = if rtl { inner.right() - w } else { inner.x };
            let marks = |under: bool, cmds: &mut Vec<DrawCmd>| {
                if edit.focused && !text.is_empty() || (edit.focused && !under) {
                    self.edit_marks(
                        &EditMarks {
                            m,
                            text: if text.is_empty() { "" } else { &line },
                            x,
                            top,
                            line_h: lh,
                            size,
                            tracking,
                            first: a,
                            edit,
                            clip,
                            alpha,
                            under,
                            caret_here: li == caret_line,
                        },
                        cmds,
                    );
                }
            };
            marks(true, cmds);
            let baseline = m.baseline(&line, top, lh, size);
            let pieces = crate::text::pieces(font, &line, self.direction, tracking);
            let one = pieces.len() == 1;
            for pc in pieces {
                let Some(handle) = pc.font.filter(|_| !pc.text.is_empty()) else {
                    continue;
                };
                let ox = pc.dx * size;
                cmds.push(DrawCmd::Text {
                    x: snap(x) + ox,
                    baseline: snap(baseline),
                    size,
                    text: pc.text,
                    color: color.fade(alpha).linear(),
                    font: handle,
                    bounds: Rect::new(x + ox, top, if one { w } else { pc.width * size }, lh),
                    clip,
                    tracking,
                    weight: t.skin.text_weights[0] * sk / dpr,
                    glow: None,
                    placements: pc.placements,
                });
            }
            marks(false, cmds);
        }
    }

    /// A skinned background: the part's shadow and glow, its nine-slice
    /// frame (tinted by `backdrop`), the tiled texture and the corner ornaments.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn dress(
        &self,
        part: StylePart,
        slice: Slice,
        rect: Rect,
        radius: f32,
        bg: Srgba,
        alpha: f32,
        clip: Rect,
        cmds: &mut Vec<DrawCmd>,
    ) {
        let skin = &self.theme.skin;
        let Some(atlas) = skin.atlas else { return };
        let p = &self.theme.palette;
        let fade = alpha * if slice.tint { 1.0 } else { bg.alpha };
        if let Some(sh) = skin.shadow(part) {
            let r = rect.outset(sh.spread);
            let r = Rect::new(r.x + sh.dx, r.y + sh.dy, r.width, r.height);
            match skin.soft.filter(|_| sh.blur > 0.0) {
                Some(soft) => nine(
                    atlas,
                    soft,
                    r.outset(sh.blur),
                    sh.color.fade(fade),
                    clip,
                    cmds,
                ),
                None => cmds.push(DrawCmd::Quad {
                    rect: r,
                    radius,
                    color: sh.color.fade(fade).linear(),
                    clip,
                }),
            }
        }
        if let (Some(g), Some(soft)) = (skin.glow(part), skin.soft) {
            let c = g.ink.color(p).fade(g.alpha * fade);
            nine(atlas, soft, rect.outset(g.size), c, clip, cmds);
        }
        // Glass: the blurred frame under the part, then its own frame
        // thinned to the glass's fill; under reduced transparency an opaque
        // surface instead, its frame whole.
        let solid = self.theme.reduced_transparency && skin.glass(part).is_some();
        let glass = skin.glass(part).filter(|_| !solid);
        let (bg, fade) = if solid {
            (Srgba { alpha: 1.0, ..bg }, alpha)
        } else {
            (bg, fade)
        };
        if let Some(g) = glass {
            let mut tint = bg.linear();
            tint[3] = g.tint;
            cmds.push(DrawCmd::Glass {
                rect,
                radius,
                blur: g.blur,
                tint,
                alpha,
                clip,
            });
        }
        let thin = glass.map_or(1.0, |g| g.fill);
        let tint = if slice.tint {
            bg.fade(alpha * thin)
        } else {
            Srgba::WHITE.fade(fade * thin)
        };
        nine(atlas, slice, rect, tint, clip, cmds);
        if let Some(tile) = skin.tile(part) {
            let inner = Rect::new(
                rect.x + tile.inset,
                rect.y + tile.inset,
                (rect.width - 2.0 * tile.inset).max(0.0),
                (rect.height - 2.0 * tile.inset).max(0.0),
            );
            let c = clip.intersect(&inner);
            let (tw, th) = (tile.w.max(4.0), tile.h.max(4.0));
            let mut y = inner.y;
            while y < inner.bottom() {
                let mut x = inner.x;
                while x < inner.right() {
                    cmds.push(DrawCmd::Image {
                        rect: Rect::new(x, y, tw, th),
                        texture: Some(atlas),
                        uv: tile.uv,
                        color: Srgba::WHITE.fade(tile.alpha * fade).linear(),
                        clip: c,
                    });
                    x += tw;
                }
                y += th;
            }
        }
        if let Some(o) = skin.corners(part) {
            let (w, h) = (o.w, o.h);
            let d = o.offset;
            let (l, t, r, b) = (
                rect.x - d,
                rect.y - d,
                rect.right() + d - w,
                rect.bottom() + d - h,
            );
            for (x, y, fx, fy) in [
                (l, t, false, false),
                (r, t, true, false),
                (l, b, false, true),
                (r, b, true, true),
            ] {
                self.ornament(o, Rect::new(x, y, w, h), fx, fy, alpha, clip, cmds);
            }
        }
    }

    /// An ornament in `rect`, mirrored across x and or y.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn ornament(
        &self,
        o: Ornament,
        rect: Rect,
        flip_x: bool,
        flip_y: bool,
        alpha: f32,
        clip: Rect,
        cmds: &mut Vec<DrawCmd>,
    ) {
        let Some(atlas) = self.theme.skin.atlas else {
            return;
        };
        let [u0, v0, u1, v1] = o.uv;
        let (u0, u1) = if flip_x { (u1, u0) } else { (u0, u1) };
        let (v0, v1) = if flip_y { (v1, v0) } else { (v0, v1) };
        let color = match o.ink {
            Ink::Authored => Srgba::WHITE,
            ink => ink.color(&self.theme.palette),
        };
        cmds.push(DrawCmd::Image {
            rect,
            texture: Some(atlas),
            uv: [u0, v0, u1, v1],
            color: color.fade(alpha).linear(),
            clip,
        });
    }

    /// What is behind node `i`: its nearest ancestor's fill over the theme's
    /// ground, opaque (a focus ring's gap).
    pub(crate) fn behind(&self, i: usize) -> Srgba {
        let mut fills = Vec::new();
        let mut j = self.nodes[i].parent;
        while j != NONE {
            if let Some(bg) = self.nodes[j].el.background {
                fills.push(bg);
                if bg.alpha >= 1.0 {
                    break;
                }
            }
            j = self.nodes[j].parent;
        }
        let mut c = self.theme.palette.backdrop;
        for f in fills.iter().rev() {
            c = f.over(c);
        }
        c
    }

    fn shape_commands(&self, n: &Node, cmds: &mut Vec<DrawCmd>) {
        let clip = n.clip;
        let alpha = n.alpha;
        let dpr = self.viewport.dpr();
        for shape in &n.shapes {
            match shape {
                Shape::Rect {
                    frame,
                    radius,
                    color,
                } => {
                    let r = frame.resolve(&n.rect);
                    if r.is_empty() {
                        continue;
                    }
                    cmds.push(DrawCmd::Quad {
                        rect: r.snapped(dpr),
                        radius: radius.min(r.width / 2.0).min(r.height / 2.0) * 1.0,
                        color: color.fade(alpha).linear(),
                        clip,
                    });
                }
                Shape::Bar {
                    frame,
                    angle,
                    color,
                } => {
                    let r = frame.resolve(&n.rect);
                    cmds.push(DrawCmd::Bar {
                        center: r.center(),
                        w: r.width,
                        h: r.height,
                        angle: *angle,
                        color: color.fade(alpha).linear(),
                        clip,
                    });
                }
                Shape::Half {
                    frame,
                    angle,
                    color,
                } => {
                    let r = frame.resolve(&n.rect);
                    cmds.push(DrawCmd::Half {
                        center: r.center(),
                        radius: r.width.min(r.height) / 2.0,
                        angle: *angle,
                        color: color.fade(alpha).linear(),
                        clip,
                    });
                }
                Shape::Image {
                    frame,
                    texture,
                    uv,
                    color,
                } => cmds.push(DrawCmd::Image {
                    rect: frame.resolve(&n.rect),
                    texture: Some(*texture),
                    uv: *uv,
                    color: color.fade(alpha).linear(),
                    clip,
                }),
                Shape::Text {
                    frame,
                    text,
                    step,
                    color,
                    font,
                    clip: within,
                } => {
                    let clip = clip.intersect(&within.resolve(&n.rect));
                    let f = self.fonts.for_text(*font, text);
                    if f.handle.is_none() {
                        continue;
                    }
                    let m = &f.metrics;
                    let r = frame.resolve(&n.rect);
                    let size = step.size * n.scale;
                    let tracking = crate::layout::step_tracking(step, text);
                    let w = m.measure_tracked(text, size, tracking);
                    let lh = step.line_height * n.scale;
                    let x = r.x + (r.width - w) / 2.0;
                    let ly = r.y + (r.height - lh) / 2.0;
                    let baseline = m.baseline(text, ly, lh, size);
                    let snap = |v: f32| (v * dpr).round() / dpr;
                    let pieces = crate::text::pieces(f, text, self.direction, tracking);
                    let one = pieces.len() == 1;
                    for pc in pieces {
                        let Some(handle) = pc.font else { continue };
                        let ox = pc.dx * size;
                        cmds.push(DrawCmd::Text {
                            x: snap(x) + ox,
                            baseline: snap(baseline),
                            size,
                            text: pc.text,
                            color: color.fade(alpha).linear(),
                            font: handle,
                            bounds: Rect::new(
                                x + ox,
                                ly,
                                if one { w } else { pc.width * size },
                                lh,
                            ),
                            clip,
                            tracking,
                            weight: 0.0,
                            glow: None,
                            placements: pc.placements,
                        });
                    }
                }
                Shape::Glyph {
                    frame,
                    font,
                    ch,
                    color,
                } => {
                    let r = frame.resolve(&n.rect);
                    let size = r.width.min(r.height);
                    if size <= 0.0 {
                        continue;
                    }
                    // The em square: ascender 1, descender 0 (cook::icons).
                    let (x, top) = (r.x + (r.width - size) / 2.0, r.y + (r.height - size) / 2.0);
                    cmds.push(DrawCmd::Text {
                        x,
                        baseline: top + size,
                        size,
                        text: ch.to_string(),
                        color: color.fade(alpha).linear(),
                        font: *font,
                        bounds: Rect::new(x, top, size, size),
                        clip,
                        tracking: 0.0,
                        weight: 0.0,
                        glow: None,
                        placements: Vec::new(),
                    });
                }
            }
        }
    }

    /// The interaction state (hover, press, focus, caret, scroll offsets,
    /// last frame's hit rectangles, the UI clock, widget memories, toasts and
    /// queued navigation) as bytes, for the host's world snapshot: restoring
    /// it with the world replays the UI exactly.
    pub fn save_state(&self) -> Vec<u8> {
        let s = &self.state;
        let mut b = Vec::new();
        let u32_ = |b: &mut Vec<u8>, v: u32| b.extend_from_slice(&v.to_le_bytes());
        let u64_ = |b: &mut Vec<u8>, v: u64| b.extend_from_slice(&v.to_le_bytes());
        let f32_ = |b: &mut Vec<u8>, v: f32| b.extend_from_slice(&v.to_le_bytes());
        let rect = |b: &mut Vec<u8>, r: &Rect| {
            for v in [r.x, r.y, r.width, r.height] {
                b.extend_from_slice(&v.to_le_bytes());
            }
        };
        u32_(&mut b, STATE_VERSION);
        u64_(&mut b, s.hot);
        u64_(&mut b, s.active);
        u64_(&mut b, s.focus);
        u32_(&mut b, s.caret as u32);
        u32_(&mut b, s.hover_ticks);
        f32_(&mut b, s.pointer.x);
        f32_(&mut b, s.pointer.y);
        u32_(&mut b, s.scroll.len() as u32);
        for (k, v) in &s.scroll {
            u64_(&mut b, *k);
            f32_(&mut b, *v);
        }
        u32_(&mut b, s.hits.len() as u32);
        for h in &s.hits {
            u64_(&mut b, h.key);
            rect(&mut b, &h.rect);
            rect(&mut b, &h.full);
            b.push(h.flags);
            u64_(&mut b, h.scope);
            u64_(&mut b, h.scroller);
        }
        u64_(&mut b, s.tick);
        b.push(
            u8::from(s.focus_visible) | u8::from(s.nav_off) << 1 | u8::from(s.scope_locked) << 2,
        );
        u64_(&mut b, s.scope);
        u64_(&mut b, s.focus_return);
        b.push(s.stick.0);
        u64_(&mut b, s.stick.1);
        u32_(&mut b, s.pending.len() as u32);
        for (n, entering) in &s.pending {
            b.push(n.index() | if *entering { 0x80 } else { 0 });
        }
        u32_(&mut b, s.anims.len() as u32);
        for (k, a) in &s.anims {
            u64_(&mut b, *k);
            f32_(&mut b, a.a);
            f32_(&mut b, a.b);
            u64_(&mut b, a.t);
            u64_(&mut b, a.h);
        }
        u32_(&mut b, s.toasts.len() as u32);
        for t in &s.toasts {
            u32_(&mut b, t.id);
            u32_(&mut b, t.text.len() as u32);
            b.extend_from_slice(t.text.as_bytes());
            b.push(t.tone);
            u64_(&mut b, t.born);
            u32_(&mut b, t.ticks);
            u64_(&mut b, t.leaving);
        }
        u32_(&mut b, s.next_toast);
        u32_(&mut b, s.screens.len() as u32);
        for (k, sc) in &s.screens {
            u64_(&mut b, *k);
            for text in [&sc.shown, &sc.wanted] {
                u32_(&mut b, text.len() as u32);
                b.extend_from_slice(text.as_bytes());
            }
            u64_(&mut b, sc.since);
        }
        for v in s.voices {
            u64_(&mut b, v);
        }
        b.push(s.players);
        for k in 0..3 {
            u64_(&mut b, s.pad_focus[k]);
            b.push(s.pad_stick[k].0);
            u64_(&mut b, s.pad_stick[k].1);
        }
        u32_(&mut b, s.scroll_x.len() as u32);
        for (k, v) in &s.scroll_x {
            u64_(&mut b, *k);
            f32_(&mut b, *v);
        }
        u32_(&mut b, s.scroll_info.len() as u32);
        for (k, v) in &s.scroll_info {
            u64_(&mut b, *k);
            for x in [v.range.x, v.range.y, v.bar.x, v.bar.y] {
                f32_(&mut b, x);
            }
        }
        let (fk, fv) = s.fling.unwrap_or((0, Vec2::ZERO));
        u64_(&mut b, fk);
        f32_(&mut b, fv.x);
        f32_(&mut b, fv.y);
        f32_(&mut b, s.drag_v.x);
        f32_(&mut b, s.drag_v.y);
        let t = s.thumb.unwrap_or_default();
        u64_(&mut b, t.key);
        b.push(t.axis);
        f32_(&mut b, t.grab);
        u32_(&mut b, s.anchor as u32);
        u64_(&mut b, s.edit_key);
        u64_(&mut b, s.edit_tick);
        u32_(&mut b, s.composition.len() as u32);
        b.extend_from_slice(s.composition.as_bytes());
        f32_(&mut b, s.edit_scroll);
        b.push(u8::from(s.typed_stream));
        let pinch = s.pinch.unwrap_or_default();
        b.push(u8::from(s.pinch.is_some()));
        u64_(&mut b, pinch.key);
        f32_(&mut b, pinch.center.x);
        f32_(&mut b, pinch.center.y);
        f32_(&mut b, pinch.spread);
        let c = &s.click;
        u64_(&mut b, c.tick);
        f32_(&mut b, c.at.x);
        f32_(&mut b, c.at.y);
        b.push(c.count);
        b.push(u8::from(c.moved));
        u32_(&mut b, c.origin.0 as u32);
        u32_(&mut b, c.origin.1 as u32);
        let text = |b: &mut Vec<u8>, t: &str| {
            u32_(b, t.len() as u32);
            b.extend_from_slice(t.as_bytes());
        };
        u32_(&mut b, s.histories.len() as u32);
        for (k, h) in &s.histories {
            u64_(&mut b, *k);
            u64_(&mut b, h.seen);
            u64_(&mut b, h.used);
            b.push(u8::from(h.open));
            for stack in [&h.undo, &h.redo] {
                u32_(&mut b, stack.len() as u32);
                for c in stack {
                    u32_(&mut b, c.at as u32);
                    text(&mut b, &c.removed);
                    text(&mut b, &c.inserted);
                    b.push(c.kind as u8);
                    for v in [c.before.0, c.before.1, c.after.0, c.after.1] {
                        u32_(&mut b, v as u32);
                    }
                }
            }
        }
        b
    }

    /// The host's voice entity for each UI sound (by `UiSound` index), kept
    /// in the UI state so snapshots carry it.
    pub fn voices_mut(&mut self) -> &mut [u64; 6] {
        &mut self.state.voices
    }

    /// Restores [`Ui::save_state`] bytes. Refused, with nothing changed, for
    /// bytes of another version or a truncated buffer.
    pub fn restore_state(&mut self, bytes: &[u8]) -> og_core::Result<()> {
        let bad = |why: String| {
            og_core::Error::new(
                og_core::ErrorKind::Parse,
                format!("UI state: {why}; pass the bytes Ui::save_state returned"),
            )
        };
        let mut r = Reader { bytes, at: 0 };
        let version = r.u32().ok_or_else(|| bad("truncated".into()))?;
        if version != STATE_VERSION {
            return Err(bad(format!("version {version}, expected {STATE_VERSION}")));
        }
        let s = r
            .state()
            .ok_or_else(|| bad("truncated or malformed".into()))?;
        if r.at != bytes.len() {
            return Err(bad("trailing bytes".into()));
        }
        self.state = s;
        Ok(())
    }
}

/// `r` scaled by `sx` and `sy` around its centre.
fn scaled_xy(r: Rect, sx: f32, sy: f32) -> Rect {
    let (cx, cy) = r.center();
    Rect::new(
        cx - r.width * sx / 2.0,
        cy - r.height * sy / 2.0,
        r.width * sx,
        r.height * sy,
    )
}

/// The nine pieces of a slice over `rect` (corners keep their size, shrunk
/// together when the rectangle is smaller than two of them); a tiling
/// slice cuts its edges and centre into tiles of the art's middle.
fn nine(
    atlas: TextureHandle,
    s: Slice,
    rect: Rect,
    tint: Srgba,
    clip: Rect,
    cmds: &mut Vec<DrawCmd>,
) {
    if rect.is_empty() {
        return;
    }
    let [l, t, r, b] = s.border;
    let kx = if l + r > rect.width {
        rect.width / (l + r)
    } else {
        1.0
    };
    let ky = if t + b > rect.height {
        rect.height / (t + b)
    } else {
        1.0
    };
    let (l, r, t, b) = (l * kx, r * kx, t * ky, b * ky);
    let xs = [rect.x, rect.x + l, rect.right() - r, rect.right()];
    let ys = [rect.y, rect.y + t, rect.bottom() - b, rect.bottom()];
    let [u0, v0, u1, v1] = s.uv;
    let [bl, bt, br, bb] = s.border_uv;
    let us = [u0, u0 + bl, u1 - br, u1];
    let vs = [v0, v0 + bt, v1 - bb, v1];
    let color = tint.linear();
    for row in 0..3 {
        for col in 0..3 {
            let piece = Rect::new(
                xs[col],
                ys[row],
                xs[col + 1] - xs[col],
                ys[row + 1] - ys[row],
            );
            if piece.width <= 0.0 || piece.height <= 0.0 {
                continue;
            }
            let uv = [us[col], vs[row], us[col + 1], vs[row + 1]];
            // Corners never tile; an edge tiles along its length, the centre both ways.
            let along_x = col == 1 && s.fill != SliceFill::Stretch;
            let along_y = row == 1 && s.fill != SliceFill::Stretch;
            let cols = tiles(piece.x, piece.width, s.middle[0], s.fill, along_x);
            let rows = tiles(piece.y, piece.height, s.middle[1], s.fill, along_y);
            for &(y, h, fv) in &rows {
                for &(x, w, fu) in &cols {
                    cmds.push(DrawCmd::Image {
                        rect: Rect::new(x, y, w, h),
                        texture: Some(atlas),
                        uv: [
                            uv[0],
                            uv[1],
                            uv[0] + (uv[2] - uv[0]) * fu,
                            uv[1] + (uv[3] - uv[1]) * fv,
                        ],
                        color,
                        clip,
                    });
                }
            }
        }
    }
}

/// The tiles along one axis of a piece starting at `at`, `len` long: each
/// `(start, length, the share of the art's middle it shows)`. Not tiling:
/// one piece showing all of it.
fn tiles(at: f32, len: f32, tile: f32, fill: SliceFill, tiled: bool) -> Vec<(f32, f32, f32)> {
    if !tiled || tile <= 0.0 {
        return vec![(at, len, 1.0)];
    }
    match fill {
        SliceFill::Stretch => vec![(at, len, 1.0)],
        SliceFill::Round => {
            let n = (len / tile).round().clamp(1.0, 4096.0) as usize;
            let w = len / n as f32;
            (0..n).map(|i| (at + i as f32 * w, w, 1.0)).collect()
        }
        SliceFill::Repeat => {
            let n = (len / tile).ceil().clamp(1.0, 4096.0) as usize;
            (0..n)
                .map(|i| {
                    let x = i as f32 * tile;
                    let w = (len - x).min(tile);
                    (at + x, w, w / tile)
                })
                .collect()
        }
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> Option<[u8; N]> {
        let s = self.bytes.get(self.at..self.at + N)?;
        self.at += N;
        s.try_into().ok()
    }
    fn u8(&mut self) -> Option<u8> {
        self.take::<1>().map(|b| b[0])
    }
    fn u32(&mut self) -> Option<u32> {
        self.take().map(u32::from_le_bytes)
    }
    fn u64(&mut self) -> Option<u64> {
        self.take().map(u64::from_le_bytes)
    }
    fn f32(&mut self) -> Option<f32> {
        self.take().map(f32::from_le_bytes)
    }

    fn string(&mut self) -> Option<String> {
        let len = self.u32()? as usize;
        let raw = self.bytes.get(self.at..self.at + len)?;
        self.at += len;
        String::from_utf8(raw.to_vec()).ok()
    }

    fn rect(&mut self) -> Option<Rect> {
        Some(Rect::new(
            self.f32()?,
            self.f32()?,
            self.f32()?,
            self.f32()?,
        ))
    }

    fn state(&mut self) -> Option<State> {
        let mut s = State {
            hot: self.u64()?,
            active: self.u64()?,
            focus: self.u64()?,
            caret: self.u32()? as usize,
            hover_ticks: self.u32()?,
            pointer: Vec2::new(self.f32()?, self.f32()?),
            ..State::default()
        };
        for _ in 0..self.u32()? {
            let k = self.u64()?;
            s.scroll.insert(k, self.f32()?);
        }
        for _ in 0..self.u32()? {
            s.hits.push(Hit {
                key: self.u64()?,
                rect: self.rect()?,
                full: self.rect()?,
                flags: self.u8()?,
                scope: self.u64()?,
                scroller: self.u64()?,
            });
        }
        s.tick = self.u64()?;
        let flags = self.u8()?;
        s.focus_visible = flags & 1 != 0;
        s.nav_off = flags & 2 != 0;
        s.scope_locked = flags & 4 != 0;
        s.scope = self.u64()?;
        s.focus_return = self.u64()?;
        s.stick = (self.u8()?, self.u64()?);
        for _ in 0..self.u32()? {
            let i = self.u8()?;
            s.pending.push((Nav::from_index(i & 0x7f)?, i & 0x80 != 0));
        }
        for _ in 0..self.u32()? {
            let k = self.u64()?;
            let a = Anim {
                a: self.f32()?,
                b: self.f32()?,
                t: self.u64()?,
                h: self.u64()?,
            };
            s.anims.insert(k, a);
        }
        for _ in 0..self.u32()? {
            let id = self.u32()?;
            let len = self.u32()? as usize;
            let raw = self.bytes.get(self.at..self.at + len)?;
            self.at += len;
            let text = String::from_utf8(raw.to_vec()).ok()?;
            s.toasts.push(Toast {
                id,
                text,
                tone: self.u8()?,
                born: self.u64()?,
                ticks: self.u32()?,
                leaving: self.u64()?,
            });
        }
        s.next_toast = self.u32()?;
        for _ in 0..self.u32()? {
            let k = self.u64()?;
            let shown = self.string()?;
            let wanted = self.string()?;
            let since = self.u64()?;
            s.screens.insert(
                k,
                crate::juice::ScreenState {
                    shown,
                    wanted,
                    since,
                },
            );
        }
        for v in s.voices.iter_mut() {
            *v = self.u64()?;
        }
        s.players = self.u8()?;
        for k in 0..3 {
            s.pad_focus[k] = self.u64()?;
            s.pad_stick[k] = (self.u8()?, self.u64()?);
        }
        for _ in 0..self.u32()? {
            let k = self.u64()?;
            s.scroll_x.insert(k, self.f32()?);
        }
        for _ in 0..self.u32()? {
            let k = self.u64()?;
            let (rx, ry, bx, by) = (self.f32()?, self.f32()?, self.f32()?, self.f32()?);
            s.scroll_info.insert(
                k,
                crate::scroll::ScrollInfo {
                    range: Vec2::new(rx, ry),
                    bar: Vec2::new(bx, by),
                },
            );
        }
        let fk = self.u64()?;
        let fv = Vec2::new(self.f32()?, self.f32()?);
        s.fling = (fk != 0).then_some((fk, fv));
        s.drag_v = Vec2::new(self.f32()?, self.f32()?);
        let t = crate::scroll::Thumb {
            key: self.u64()?,
            axis: self.u8()?,
            grab: self.f32()?,
        };
        s.thumb = (t.key != 0).then_some(t);
        s.anchor = self.u32()? as usize;
        s.edit_key = self.u64()?;
        s.edit_tick = self.u64()?;
        s.composition = self.string()?;
        s.edit_scroll = self.f32()?;
        s.typed_stream = self.u8()? != 0;
        let on = self.u8()? != 0;
        let pinch = crate::scroll::Pinch {
            key: self.u64()?,
            center: Vec2::new(self.f32()?, self.f32()?),
            spread: self.f32()?,
        };
        s.pinch = on.then_some(pinch);
        s.click = crate::edit::Click {
            tick: self.u64()?,
            at: Vec2::new(self.f32()?, self.f32()?),
            count: self.u8()?,
            moved: self.u8()? != 0,
            origin: (self.u32()? as usize, self.u32()? as usize),
        };
        for _ in 0..self.u32()? {
            let k = self.u64()?;
            let mut h = crate::edit::History {
                seen: self.u64()?,
                used: self.u64()?,
                open: self.u8()? != 0,
                ..Default::default()
            };
            for redo in [false, true] {
                let n = self.u32()?;
                let mut stack = Vec::new();
                for _ in 0..n {
                    stack.push(crate::edit::Change {
                        at: self.u32()? as usize,
                        removed: self.string()?,
                        inserted: self.string()?,
                        kind: crate::edit::EditKind::from_u8(self.u8()?),
                        before: (self.u32()? as usize, self.u32()? as usize),
                        after: (self.u32()? as usize, self.u32()? as usize),
                    });
                }
                if redo {
                    h.redo = stack;
                } else {
                    h.undo = stack;
                }
            }
            s.histories.insert(k, h);
        }
        Some(s)
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
