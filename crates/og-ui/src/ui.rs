//! The immediate-mode context: build a tree every tick, read responses back.

use crate::draw::{assign_levels, emit, DrawCmd, DrawItem, EmitDesc};
use crate::id::{child_key, Id};
use crate::input::{InputFrame, Nav, UiKey};
use crate::layout::{
    layout, preorder, Align, Anchor, Content, Cross, Direction, Element, Float, FontRole, Insets,
    Metrics, Node, Rect, Shape, Size, TextAlign, TextContent, Widget, NONE,
};
use crate::motion::Fx;
use crate::skin::{Ink, Ornament, SkinPart, Slice};
use crate::style::{Breakpoint, Density, Srgba, Theme, TypeStep};
use crate::text::{FontMetrics, UiFont};
use og_core::glam::Vec2;
use og_probe::hooks::{find_by_id, find_by_text};
use og_probe::UiElement;
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
}

impl Default for Viewport {
    fn default() -> Viewport {
        Viewport {
            width: 1280.0,
            height: 720.0,
            dpr: 1.0,
            safe_area: Insets::default(),
            coarse_pointer: false,
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
/// back to `body`. `fallback` draws a text whose role font lacks one of its
/// characters (a skin's Latin display face and Chinese text).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fonts {
    pub body: UiFont,
    pub display: Option<UiFont>,
    pub mono: Option<UiFont>,
    pub fallback: Option<UiFont>,
}

impl Fonts {
    pub fn get(&self, role: FontRole) -> &UiFont {
        match role {
            FontRole::Body => &self.body,
            FontRole::Display => self.display.as_ref().unwrap_or(&self.body),
            FontRole::Mono => self.mono.as_ref().unwrap_or(&self.body),
        }
    }

    /// The font a text of `role` is measured and drawn with: the role's,
    /// or the fallback when the role's lacks a character the fallback has.
    pub fn for_text(&self, role: FontRole, text: &str) -> &UiFont {
        let f = self.get(role);
        match &self.fallback {
            Some(fb)
                if text
                    .chars()
                    .any(|c| !c.is_whitespace() && !f.metrics.has(c) && fb.metrics.has(c)) =>
            {
                fb
            }
            _ => f,
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
    pub hover_ticks: u32,
    pub pointer: Vec2,
    /// The press that focused a field: where it landed, to place the caret.
    pub focus_click: Option<f32>,
    pub scroll: BTreeMap<u64, f32>,
    /// Last frame's interactive rectangles in draw order, for hit testing.
    pub hits: Vec<Hit>,
    /// Frames begun: the clock of every animation.
    pub tick: u64,
    /// Focus moved by keyboard or gamepad: the ring shows.
    pub focus_visible: bool,
    /// The topmost focus scope of the last frame (0: none).
    pub scope: u64,
    /// Where focus returns when that scope closes.
    pub focus_return: u64,
    /// Keyboard and gamepad navigation as the last frame declared it.
    pub nav_off: bool,
    /// The stick's held direction (0 none, 1 + Nav index) and since which tick.
    pub stick: (u8, u64),
    /// Intents queued by `Ui::navigate` for the next frame.
    pub pending: Vec<Nav>,
    pub anims: BTreeMap<u64, Anim>,
    pub toasts: Vec<Toast>,
    pub next_toast: u32,
    /// Screen transitions by key.
    pub screens: BTreeMap<u64, crate::juice::ScreenState>,
    /// The host's voice entity for each UI sound (bits; 0: none).
    pub voices: [u64; 6],
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
}

/// The icon atlas `Ui::icon` draws from: a grid of equal cells.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct IconAtlas {
    pub texture: Option<TextureHandle>,
    pub columns: u32,
    pub rows: u32,
    /// Optional names, one per cell in reading order, for `icon_named`.
    pub names: Vec<String>,
}

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

/// Button looks (`docs/design-language.md` §Button).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    /// A clear lens: neutral fill (the default).
    #[default]
    Secondary,
    /// The amber action: one per screen.
    Primary,
    /// Text until hovered.
    Quiet,
    /// A destructive action: danger ink on a neutral fill.
    Danger,
}

/// Control heights from the density (`h-control-sm/md/lg`).
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
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LabelDesc {
    /// `None`: `text-body`.
    pub step: Option<TypeStep>,
    /// `None`: `fg`.
    pub color: Option<Srgba>,
    pub align: TextAlign,
    /// Keep on one line (break only at `\n`).
    pub nowrap: bool,
    pub width: Size,
    pub font: FontRole,
    /// A heading: a skin with a flourish frames it.
    pub heading: bool,
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
    /// `None`: `surface`.
    pub background: Option<Srgba>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollDesc {
    pub width: Size,
    pub height: Size,
    pub padding: Insets,
    pub gap: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ImageDesc {
    /// `None` draws a solid rectangle of `tint`.
    pub texture: Option<TextureHandle>,
    /// `None`: the whole texture (`[u0, v0, u1, v1]`, v down).
    pub uv: Option<[f32; 4]>,
    /// `None`: white.
    pub tint: Option<Srgba>,
    pub width: Size,
    pub height: Size,
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

/// The immediate-mode UI. Call [`Ui::begin`], build, [`Ui::end`] once per tick.
#[derive(Clone, Debug)]
pub struct Ui {
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
    /// The control a widget asked to focus this frame.
    pub(crate) request_focus: u64,
    /// Open focus scopes this frame, in build order.
    pub(crate) scopes: Vec<usize>,
    pub(crate) icons: IconAtlas,
    pub(crate) toast_anchor: Anchor,
    /// UI events with a skin sound, not yet taken by the host.
    pub(crate) sounds: Vec<crate::skin::UiSound>,
    /// This frame's screen-transition cover.
    pub(crate) cover: Option<crate::juice::Cover>,
    /// The configured skin's name (empty: the tokens).
    pub(crate) skin_name: String,
}

impl Default for Ui {
    fn default() -> Ui {
        Ui::new(UiDesc::default())
    }
}

const STATE_VERSION: u32 = 3;

impl Ui {
    pub fn new(desc: UiDesc) -> Ui {
        Ui {
            theme: desc.theme,
            fonts: desc.fonts,
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
            request_focus: 0,
            scopes: Vec::new(),
            icons: IconAtlas::default(),
            toast_anchor: Anchor::Bottom,
            sounds: Vec::new(),
            cover: None,
            skin_name: String::new(),
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

    /// Queues a navigation intent for the next tick (a gamepad the input
    /// layer does not read yet, a custom binding, a test).
    pub fn navigate(&mut self, nav: Nav) {
        self.state.pending.push(nav);
    }

    /// The key of the focused control (0: none).
    pub fn focused_key(&self) -> u64 {
        self.state.focus
    }

    /// Escape or gamepad B this tick, not taken by a dialog or menu: true
    /// once, then taken. A pause screen closes on it.
    pub fn back(&mut self) -> bool {
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

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    /// The configured skin's name, for the theme the host reports (empty: the tokens).
    pub fn skin_name(&self) -> &str {
        &self.skin_name
    }

    pub fn set_skin_name(&mut self, name: &str) {
        self.skin_name = name.to_owned();
    }

    pub fn fonts(&self) -> &Fonts {
        &self.fonts
    }

    pub fn set_fonts(&mut self, fonts: Fonts) {
        self.fonts = fonts;
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
        if self.building {
            self.warnings.push(
                "begin without end: the previous frame was dropped; call ui.end() once per tick"
                    .into(),
            );
        }
        self.viewport = viewport;
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
        self.state.hover_ticks = if hot != 0 && hot == self.state.hot {
            self.state.hover_ticks.saturating_add(1)
        } else {
            0
        };
        if hot != 0 && hot != self.state.hot && self.hit(hot).is_some_and(|h| h.has(HIT_NAV)) {
            self.sound(crate::skin::UiSound::Hover);
        }
        if input.pressed && self.hit(hot).is_some_and(|h| h.has(HIT_NAV)) {
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
        // Wheel over a scroll container; drag on one that holds the press.
        if input.wheel.y != 0.0 {
            if let Some(h) = self
                .state
                .hits
                .iter()
                .rev()
                .find(|h| h.has(HIT_SCROLL) && h.rect.contains(p.x, p.y))
            {
                *self.state.scroll.entry(h.key).or_insert(0.0) += input.wheel.y;
            }
        }
        if input.down
            && !input.pressed
            && self
                .hit(self.state.active)
                .is_some_and(|h| h.has(HIT_SCROLL))
        {
            let dy = self.state.pointer.y - p.y;
            *self.state.scroll.entry(self.state.active).or_insert(0.0) += dy;
        }
        let intents = self.intents(&input);
        self.apply_nav(&intents);
        self.state.pointer = p;
        self.input = input;

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
        Response {
            hovered,
            pressed: s.active == key && self.input.down,
            clicked: (s.active == key && self.input.released && s.hot == key)
                || (self.nav.activate == key && key != 0),
            focused: s.focus == key,
            ..Response::default()
        }
    }

    /// Marks node `i` as a keyboard/gamepad focus target and returns its
    /// response; the ring shows while navigation put focus on it.
    pub(crate) fn nav_target(&mut self, i: usize, adjust: bool) -> Response {
        let key = self.nodes[i].key;
        let n = &mut self.nodes[i];
        n.nav = true;
        n.adjust = adjust;
        n.ring = self.state.focus == key && self.state.focus_visible;
        self.respond(key)
    }

    /// Opens a container; children follow until [`Ui::close`].
    pub fn open(&mut self, id: impl Into<Id>, el: Element) {
        let kind = match (el.scroll, el.direction) {
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

    /// A card: `surface`, `radius.card`, padded and spaced by the tokens.
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
            background: Some(desc.background.unwrap_or(t.palette.surface)),
            radius: t.radius.card,
            skin: Some(crate::skin::SkinPart::Card),
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
                scroll: true,
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
            color: desc.color.unwrap_or(t.palette.fg),
            align: desc.align,
            wrap: !desc.nowrap,
            font: desc.font,
        });
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
                radius: t.radius.button,
                ..Element::default()
            },
        );
        let refused =
            desc.disabled && t.skin.motion.shake > 0.0 && self.respond(self.nodes[i].key).clicked;
        let r = if desc.disabled {
            Response::default()
        } else {
            self.nav_target(i, false)
        };
        let p = &t.palette;
        let (bg, fg) = if desc.disabled {
            (Some(p.fill), p.fg_subtle)
        } else {
            match desc.variant {
                ButtonVariant::Primary => (
                    Some(if r.pressed {
                        p.accent_active
                    } else if r.hovered {
                        p.accent_hover
                    } else {
                        p.accent
                    }),
                    p.accent_fg,
                ),
                ButtonVariant::Secondary => (
                    Some(if r.hovered || r.pressed {
                        p.fill_strong
                    } else {
                        p.fill
                    }),
                    p.fg,
                ),
                ButtonVariant::Quiet => ((r.hovered || r.pressed).then_some(p.fill), p.fg),
                ButtonVariant::Danger => (
                    Some(if r.hovered || r.pressed {
                        p.fill_strong
                    } else {
                        p.fill
                    }),
                    p.danger,
                ),
            }
        };
        let mut content = self.text_content(text, t.type_ramp.body, fg);
        content.align = TextAlign::Center;
        content.wrap = false;
        let part = if desc.disabled {
            SkinPart::ButtonDisabled
        } else if r.pressed {
            SkinPart::ButtonPressed
        } else if r.hovered {
            SkinPart::ButtonHover
        } else {
            SkinPart::Button
        };
        self.button_motion(i, r, refused);
        let n = &mut self.nodes[i];
        n.el.background = bg;
        n.el.skin = Some(part);
        n.content = Content::Text(content);
        n.widget = Widget::Button { pressed: r.pressed };
        n.probe_text = text.to_owned();
        r
    }

    /// A texture region, or a solid rectangle without a texture.
    pub fn image(&mut self, id: impl Into<Id>, desc: ImageDesc) {
        let i = self.leaf(
            "image",
            id.into(),
            Element {
                width: desc.width,
                height: desc.height,
                ..Element::default()
            },
        );
        self.nodes[i].content = Content::Image {
            texture: desc.texture,
            uv: desc.uv.unwrap_or([0.0, 0.0, 1.0, 1.0]),
            tint: desc.tint.unwrap_or(Srgba::WHITE),
        };
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

    /// A button's skin motion: the hover lift eased in and out, the press
    /// sink, and the shake of a refused press (`motion.duration.slow`).
    fn button_motion(&mut self, i: usize, r: Response, refused: bool) {
        let t = self.theme;
        let skin = t.skin;
        if skin.hover.is_none() && skin.press.is_none() && skin.motion.shake <= 0.0 {
            return;
        }
        let key = child_key(self.nodes[i].key, "motion");
        let mut a = self.anim(key);
        let step = 1.0 / self.ticks_for(t.durations_ms.instant) as f32;
        a.a = if r.hovered {
            (a.a + step).min(1.0)
        } else {
            (a.a - step).max(0.0)
        };
        let now = self.state.tick;
        if refused {
            a.t = now;
            a.b = 1.0;
            self.sound(crate::skin::UiSound::Error);
        }
        self.set_anim(key, a);
        if t.reduced_motion {
            return;
        }
        let e = crate::motion::ease_out(a.a);
        let mut f = Fx::default();
        if let Some(h) = skin.hover {
            f.dy += h.dy * e;
            f.scale = 1.0 + (h.scale - 1.0) * e;
        }
        if r.pressed {
            if let Some(p) = skin.press {
                f.dy += p.dy;
            }
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
    }

    /// Queues a UI event's sound when the skin has one for it.
    pub(crate) fn sound(&mut self, event: crate::skin::UiSound) {
        if self.theme.skin.sound(event) != 0 {
            self.sounds.push(event);
        }
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
        if self.state.active == key && (self.input.down || self.input.pressed) {
            if let Some(h) = self.hit(key) {
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
        let mut content = self.text_content(label, t.type_ramp.body, t.palette.fg);
        content.wrap = false;
        let n = &mut self.nodes[i];
        n.content = Content::Text(content);
        n.widget = Widget::Toggle { on: *on };
        n.probe_text = label.to_owned();
        r
    }

    /// A one-line text field: a press focuses it (caret at the press), typed
    /// text inserts at the caret, Backspace/Delete/arrows/Home/End edit,
    /// Enter submits and Escape leaves.
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
                radius: t.radius.field,
                skin: Some(SkinPart::Field),
                ..Element::default()
            },
        );
        self.nodes[i].focusable = true;
        self.nodes[i].nav = true;
        let key = self.nodes[i].key;
        let mut r = self.respond(key);
        let len = value.chars().count();
        if r.focused {
            let metrics = &self.fonts.body.metrics;
            let size = t.type_ramp.body.size;
            if let (Some(px), Some(h)) = (self.state.focus_click, self.hit(key)) {
                // The caret goes to the character boundary nearest the press.
                let mut pen = h.rect.x + t.space.md;
                let mut best = (0usize, (px - pen).abs());
                for (k, c) in value.chars().enumerate() {
                    pen += metrics.advance(c) * size;
                    let d = (px - pen).abs();
                    if d < best.1 {
                        best = (k + 1, d);
                    }
                }
                self.state.caret = best.0;
            }
            let mut caret = self.state.caret.min(len);
            let mut chars: Vec<char> = value.chars().collect();
            let before = chars.clone();
            for c in self.input.text.chars() {
                if desc.max_chars == 0 || chars.len() < desc.max_chars as usize {
                    chars.insert(caret, c);
                    caret += 1;
                }
            }
            for key_pressed in self.input.keys.clone() {
                match key_pressed {
                    UiKey::Backspace if caret > 0 => {
                        chars.remove(caret - 1);
                        caret -= 1;
                    }
                    UiKey::Delete if caret < chars.len() => {
                        chars.remove(caret);
                    }
                    UiKey::Left => caret = caret.saturating_sub(1),
                    UiKey::Right => caret = (caret + 1).min(chars.len()),
                    UiKey::Home => caret = 0,
                    UiKey::End => caret = chars.len(),
                    UiKey::Enter => {
                        r.submitted = true;
                        self.state.focus = 0;
                    }
                    UiKey::Escape => self.state.focus = 0,
                    _ => {}
                }
            }
            if chars != before {
                *value = chars.into_iter().collect();
                r.changed = true;
            }
            self.state.caret = caret;
            r.focused = self.state.focus == key;
        }
        let focused = r.focused;
        let shown = if value.is_empty() {
            desc.placeholder.as_str()
        } else {
            value.as_str()
        };
        let color = if value.is_empty() {
            t.palette.fg_subtle
        } else {
            t.palette.fg
        };
        let mut content = self.text_content(shown, t.type_ramp.body, color);
        content.wrap = false;
        let caret = focused.then_some(self.state.caret.min(value.chars().count()));
        let n = &mut self.nodes[i];
        n.el.background = Some(if focused {
            t.palette.surface
        } else {
            t.palette.fill
        });
        n.content = Content::Text(content);
        n.widget = Widget::Field { focused, caret };
        n.probe_text = shown.to_owned();
        r
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
                background: Some(t.palette.surface_raised),
                radius: t.radius.control,
                skin: Some(SkinPart::Tooltip),
                ..Element::default()
            },
        );
        let mut content = self.text_content(text, t.type_ramp.meta, t.palette.fg);
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
        let offset = |k: u64| scroll.get(&k).copied().unwrap_or(0.0);
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
        for &f in &self.floating.clone() {
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
            let gap = self.theme.space.xs;
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
        self.apply_stagger(&order);
        self.apply_fx(&order);

        // Interaction state for the next tick.
        let present: BTreeMap<u64, usize> = order.iter().map(|&i| (self.nodes[i].key, i)).collect();
        self.state
            .scroll
            .retain(|k, _| present.get(k).is_some_and(|&i| self.nodes[i].el.scroll));
        for (k, v) in self.state.scroll.iter_mut() {
            *v = v.clamp(0.0, self.nodes[present[k]].max_scroll());
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
            scroller_of[i] = if up != NONE && self.nodes[up].el.scroll {
                self.nodes[up].key
            } else {
                pr
            };
        }
        let mut hits = Vec::new();
        let mut elements = Vec::new();
        let mut top_scope = 0u64;
        for &i in &order[1..] {
            let n = &self.nodes[i];
            if n.scope && !n.inert {
                top_scope = n.key;
            }
            let visible = n.rect.intersect(&n.clip);
            // Scrolled or clipped out of view: not on screen, not listed. A
            // zero-size element (an empty spacer) is listed where it sits.
            let (r, c) = (&n.rect, &n.clip);
            let outside =
                r.x > c.right() || r.right() < c.x || r.y > c.bottom() || r.bottom() < c.y;
            let mut flags = 0u8;
            if n.el.scroll && !n.el.interactive {
                flags |= HIT_SCROLL;
            }
            if n.el.interactive || n.backdrop {
                flags |= HIT_POINTER;
            }
            if n.focusable {
                flags |= HIT_TEXT;
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
            if outside || (visible.is_empty() && !r.is_empty()) {
                // Navigation can still reach a control just out of a list's view.
                if n.nav && !n.inert && scroller_of[i] != 0 {
                    hits.push(hit(*r, (flags & (HIT_NAV | HIT_ADJUST)) | HIT_OFFSCREEN));
                }
                continue;
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
            elements.push(UiElement {
                id: n.id.clone(),
                parent,
                text: n.probe_text.clone(),
                x: visible.x,
                y: visible.y,
                width: visible.width,
                height: visible.height,
            });
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
        self.state.scope = top_scope;
        self.state.nav_off = !self.nav_on;
        self.state.hits = hits;

        let mut cmds = self.commands(&order);
        if let Some(c) = self.cover {
            self.cover_commands(&c, &mut cmds);
        }
        self.frame = UiFrame {
            viewport: vp,
            elements,
            items: assign_levels(cmds),
            warnings: std::mem::take(&mut self.warnings),
        };
        &self.frame
    }

    /// Staggered entrances: under a node with a stagger, the k-th drawn
    /// leaf (a label, a control, an icon) enters `k` steps after the first.
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
            for (k, &j) in leaves.iter().enumerate() {
                let start = st.since as f32 + st.step * k.min(12) as f32;
                let p = ((now - start + 1.0) / st.ticks).clamp(0.0, 1.0);
                if p >= 1.0 || self.nodes[j].fx.is_some() {
                    continue;
                }
                let mut f =
                    crate::motion::fx(st.kind, crate::motion::ease_out(p), distance, reduced);
                f.inert = true;
                self.nodes[j].fx = Some(f);
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
            if n.el.scroll {
                leave.push((i, depth_end(i)));
            }
            while let Some(&(s, end)) = leave.last() {
                if end != i {
                    break;
                }
                leave.pop();
                let sn = &self.nodes[s];
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
                        radius: w / 2.0,
                        color: p.fill_strong.linear(),
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
        let field_ring = matches!(n.widget, Widget::Field { focused: true, .. });
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
                p.border_strong
            };
            let gap = if field_ring || !n.ring {
                0.0
            } else {
                t.space.xxs
            };
            cmds.push(quad(rect.outset(w + gap), n.el.radius + w + gap, color));
            if gap > 0.0 || (outline && !field_ring) {
                cmds.push(quad(rect.outset(gap), n.el.radius + gap, behind));
            }
        }
        if let Some(bg) = n.el.background {
            let pressed = matches!(n.widget, Widget::Button { pressed: true });
            match n
                .el
                .skin
                .and_then(|part| t.skin.slice(part).map(|s| (part, s)))
            {
                Some((part, slice)) => {
                    let r = match (pressed, t.skin.press) {
                        (true, Some(pr)) => scaled_xy(rect, pr.sx, pr.sy),
                        (true, None) => rect.scaled(t.press_scale),
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
        if self.kinds[i].0 == "divider" {
            if let Some(o) = t.skin.divider.filter(|_| t.skin.atlas.is_some()) {
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
        }
        let track = t.skin.slice(SkinPart::Track);
        match n.widget {
            Widget::Progress { value } if track.is_some() => {
                let fill = t.skin.slice(SkinPart::Fill);
                if let Some(s) = track {
                    self.dress(
                        SkinPart::Track,
                        s,
                        rect,
                        rect.height / 2.0,
                        p.fill_strong,
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
                        SkinPart::Fill,
                        s,
                        r,
                        rect.height / 2.0,
                        p.accent,
                        alpha,
                        clip,
                        cmds,
                    );
                }
            }
            Widget::Progress { value } => {
                cmds.push(quad(rect, rect.height / 2.0, p.fill_strong));
                if value > 0.0 {
                    let w = (rect.width * value).max(rect.height);
                    cmds.push(quad(
                        Rect::new(rect.x, rect.y, w, rect.height).snapped(dpr),
                        rect.height / 2.0,
                        p.accent,
                    ));
                }
            }
            Widget::Slider { t: v, active } => {
                let thumb = self.thumb();
                let track_h = t.space.xs;
                let cy = rect.y + rect.height / 2.0;
                let track = Rect::new(rect.x, cy - track_h / 2.0, rect.width, track_h).snapped(dpr);
                cmds.push(quad(track, track_h / 2.0, p.fill_strong));
                let tx = rect.x + v * (rect.width - thumb).max(0.0);
                cmds.push(quad(
                    Rect::new(rect.x, track.y, tx + thumb / 2.0 - rect.x, track_h).snapped(dpr),
                    track_h / 2.0,
                    p.accent,
                ));
                let mut knob = Rect::new(tx, cy - thumb / 2.0, thumb, thumb);
                if active {
                    knob = knob.scaled(t.press_scale);
                }
                cmds.push(quad(knob.snapped(dpr), thumb / 2.0, p.thumb));
            }
            Widget::Toggle { on } => {
                let (sw, sh) = self.switch_size();
                let track =
                    Rect::new(rect.x, rect.y + (rect.height - sh) / 2.0, sw, sh).snapped(dpr);
                cmds.push(quad(
                    track,
                    sh / 2.0,
                    if on { p.accent } else { p.fill_strong },
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
                    k / 2.0,
                    p.thumb,
                ));
            }
            _ => {}
        }
        match &n.content {
            Content::Text(tc) => {
                let font = self.fonts.for_text(tc.font, &tc.text);
                if let Some(handle) = font.handle {
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
                    // except in a field, which scrolls nothing and clips.
                    let text_clip = match n.widget {
                        Widget::Field { .. } => clip.intersect(&rect),
                        _ => clip,
                    };
                    for (k, line) in n.lines.iter().enumerate() {
                        let ly = top + k as f32 * lh;
                        let free = inner.width - line.width * sk;
                        let x = inner.x
                            + match tc.align {
                                TextAlign::Start => 0.0,
                                TextAlign::Center => free / 2.0,
                                TextAlign::End => free,
                            };
                        let baseline =
                            ly + (lh - (m.ascender - m.descender) * size) / 2.0 + m.ascender * size;
                        let bounds = Rect::new(x, ly, line.width * sk, lh);
                        let role = tc.font as usize;
                        let text = |dx: f32, dy: f32, color: Srgba| DrawCmd::Text {
                            x: snap(x + dx),
                            baseline: snap(baseline + dy),
                            size,
                            text: line.text.clone(),
                            color: color.fade(alpha).linear(),
                            font: handle,
                            bounds: Rect::new(
                                bounds.x + dx,
                                bounds.y + dy,
                                bounds.width,
                                bounds.height,
                            ),
                            clip: text_clip,
                        };
                        if let Some(g) = t.skin.text_glows[role] {
                            let c = g.ink.color(p).fade(g.alpha * tc.color.alpha / 4.0);
                            for (dx, dy) in
                                [(-g.size, 0.0), (g.size, 0.0), (0.0, -g.size), (0.0, g.size)]
                            {
                                cmds.push(text(dx, dy, c));
                            }
                        }
                        if let Some(sh) = t.skin.text_shadows[role] {
                            let c = sh.color.fade(tc.color.alpha);
                            cmds.push(text(sh.dx, sh.dy, c));
                            if sh.blur > 0.0 {
                                cmds.push(text(
                                    sh.dx + sh.blur / 2.0,
                                    sh.dy + sh.blur / 2.0,
                                    c.fade(0.5),
                                ));
                            }
                        }
                        cmds.push(text(0.0, 0.0, tc.color));
                        if n.heading && k == 0 {
                            if let Some(o) = t.skin.flourish.filter(|_| t.skin.atlas.is_some()) {
                                let cy = ly + lh / 2.0;
                                let gap = o.offset + t.space.sm;
                                let (w, h) = (o.w * sk, o.h * sk);
                                let left = Rect::new(x - gap - w, cy - h / 2.0, w, h);
                                let right =
                                    Rect::new(x + line.width * sk + gap, cy - h / 2.0, w, h);
                                // Both sides of a centred heading; the side away from the edge otherwise.
                                if tc.align != TextAlign::Start {
                                    self.ornament(o, left, false, false, alpha, clip, cmds);
                                }
                                if tc.align != TextAlign::End {
                                    self.ornament(o, right, true, false, alpha, clip, cmds);
                                }
                            }
                        }
                    }
                    if let Widget::Field { caret: Some(c), .. } = n.widget {
                        let text = match &n.content {
                            Content::Text(tc) if tc.color != p.fg_subtle => tc.text.as_str(),
                            _ => "",
                        };
                        let prefix: String = text.chars().take(c).collect();
                        let cx = inner.x + m.measure(&prefix, size);
                        let ch = size + t.space.xxs * 2.0;
                        let caret_w = 1.0f32.max(1.0 / dpr);
                        cmds.push(DrawCmd::Quad {
                            rect: Rect::new(cx, rect.y + (rect.height - ch) / 2.0, caret_w, ch)
                                .snapped(dpr),
                            radius: 0.0,
                            color: p.accent.fade(alpha).linear(),
                            clip: text_clip,
                        });
                    }
                }
            }
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

    /// A skinned background: the part's shadow and glow, its nine-slice
    /// frame (tinted by `bg`), the tiled texture and the corner ornaments.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn dress(
        &self,
        part: SkinPart,
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
        let tint = if slice.tint {
            bg.fade(alpha)
        } else {
            Srgba::WHITE.fade(fade)
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
        let mut c = self.theme.palette.bg;
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
                } => {
                    let f = self.fonts.for_text(*font, text);
                    let Some(handle) = f.handle else { continue };
                    let m = &f.metrics;
                    let r = frame.resolve(&n.rect);
                    let size = step.size * n.scale;
                    let w = m.measure(text, size);
                    let lh = step.line_height * n.scale;
                    let x = r.x + (r.width - w) / 2.0;
                    let ly = r.y + (r.height - lh) / 2.0;
                    let baseline =
                        ly + (lh - (m.ascender - m.descender) * size) / 2.0 + m.ascender * size;
                    let snap = |v: f32| (v * dpr).round() / dpr;
                    cmds.push(DrawCmd::Text {
                        x: snap(x),
                        baseline: snap(baseline),
                        size,
                        text: text.clone(),
                        color: color.fade(alpha).linear(),
                        font: handle,
                        bounds: Rect::new(x, ly, w, lh),
                        clip,
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
        b.push(u8::from(s.focus_visible) | u8::from(s.nav_off) << 1);
        u64_(&mut b, s.scope);
        u64_(&mut b, s.focus_return);
        b.push(s.stick.0);
        u64_(&mut b, s.stick.1);
        u32_(&mut b, s.pending.len() as u32);
        for n in &s.pending {
            b.push(n.index());
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
/// together when the rectangle is smaller than two of them).
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
            cmds.push(DrawCmd::Image {
                rect: piece,
                texture: Some(atlas),
                uv: [us[col], vs[row], us[col + 1], vs[row + 1]],
                color,
                clip,
            });
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
        s.scope = self.u64()?;
        s.focus_return = self.u64()?;
        s.stick = (self.u8()?, self.u64()?);
        for _ in 0..self.u32()? {
            s.pending.push(Nav::from_index(self.u8()?)?);
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
        Some(s)
    }
}
