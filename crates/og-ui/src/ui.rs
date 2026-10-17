//! The immediate-mode context: build a tree every tick, read responses back.

use crate::draw::{assign_levels, emit, DrawCmd, DrawItem, EmitDesc};
use crate::id::{child_key, Id};
use crate::input::{InputFrame, UiKey};
use crate::layout::{
    layout, preorder, Align, Content, Cross, Direction, Element, FontRole, Insets, Metrics, Node,
    Rect, Size, TextAlign, TextContent, Widget, NONE,
};
use crate::style::{Density, Srgba, Theme, TypeStep};
use crate::text::{FontMetrics, UiFont};
use og_core::glam::Vec2;
use og_probe::hooks::{find_by_id, find_by_text};
use og_probe::UiElement;
use og_render::{Sprite, Text, TextureHandle};
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

    fn dpr(&self) -> f32 {
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

/// The fonts of each role; `display` and `mono` fall back to `body`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fonts {
    pub body: UiFont,
    pub display: Option<UiFont>,
    pub mono: Option<UiFont>,
}

impl Fonts {
    pub fn get(&self, role: FontRole) -> &UiFont {
        match role {
            FontRole::Body => &self.body,
            FontRole::Display => self.display.as_ref().unwrap_or(&self.body),
            FontRole::Mono => self.mono.as_ref().unwrap_or(&self.body),
        }
    }
}

impl Metrics for Fonts {
    fn metrics(&self, role: FontRole) -> &FontMetrics {
        &self.get(role).metrics
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
    /// It has keyboard focus (text field).
    pub focused: bool,
    /// Enter was pressed in it (text field).
    pub submitted: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Hit {
    key: u64,
    rect: Rect,
    scroll: bool,
    focusable: bool,
}

/// What persists between ticks (and in [`Ui::save_state`]).
#[derive(Clone, Debug, Default, PartialEq)]
struct State {
    hot: u64,
    active: u64,
    focus: u64,
    caret: usize,
    hover_ticks: u32,
    pointer: Vec2,
    /// The press that focused a field: where it landed, to place the caret.
    focus_click: Option<f32>,
    scroll: BTreeMap<u64, f32>,
    /// Last frame's interactive rectangles in draw order, for hit testing.
    hits: Vec<Hit>,
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

    /// Appends the frame's sprites and text to a render scene's lists.
    pub fn emit(&self, desc: &EmitDesc, sprites: &mut Vec<Sprite>, texts: &mut Vec<Text>) {
        emit(&self.items, desc, sprites, texts);
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
    theme: Theme,
    fonts: Fonts,
    ticks_per_second: u32,
    viewport: Viewport,
    input: InputFrame,
    nodes: Vec<Node>,
    kinds: Vec<(&'static str, bool)>,
    stack: Vec<usize>,
    floating: Vec<usize>,
    last_closed: usize,
    warnings: Vec<String>,
    building: bool,
    state: State,
    frame: UiFrame,
}

impl Default for Ui {
    fn default() -> Ui {
        Ui::new(UiDesc::default())
    }
}

const STATE_VERSION: u32 = 1;

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
        }
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
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

        // Hit test against last frame's rectangles, topmost first.
        let p = input.pointer;
        let hot = if input.pointer_inside {
            self.state
                .hits
                .iter()
                .rev()
                .find(|h| h.rect.contains(p.x, p.y))
                .map_or(0, |h| h.key)
        } else {
            0
        };
        self.state.hover_ticks = if hot != 0 && hot == self.state.hot {
            self.state.hover_ticks.saturating_add(1)
        } else {
            0
        };
        self.state.hot = hot;
        self.state.focus_click = None;
        if input.pressed {
            self.state.active = hot;
            let focusable = self.hit(hot).is_some_and(|h| h.focusable);
            if focusable {
                if self.state.focus != hot {
                    self.state.focus = hot;
                }
                self.state.focus_click = Some(p.x);
            } else {
                self.state.focus = 0;
            }
        }
        // Wheel over a scroll container; drag on one that holds the press.
        if input.wheel.y != 0.0 {
            if let Some(h) = self
                .state
                .hits
                .iter()
                .rev()
                .find(|h| h.scroll && h.rect.contains(p.x, p.y))
            {
                *self.state.scroll.entry(h.key).or_insert(0.0) += input.wheel.y;
            }
        }
        if input.down && !input.pressed && self.hit(self.state.active).is_some_and(|h| h.scroll) {
            let dy = self.state.pointer.y - p.y;
            *self.state.scroll.entry(self.state.active).or_insert(0.0) += dy;
        }
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

    fn hit(&self, key: u64) -> Option<&Hit> {
        if key == 0 {
            return None;
        }
        self.state.hits.iter().find(|h| h.key == key)
    }

    fn parent(&self) -> usize {
        self.stack.last().copied().unwrap_or(0)
    }

    /// Adds a node under the open element and returns its index.
    fn push(&mut self, kind: &'static str, id: Id, el: Element) -> usize {
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

    fn path_of(&self, mut i: usize) -> String {
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

    fn respond(&self, key: u64) -> Response {
        let s = &self.state;
        let hovered = s.hot == key && (s.active == 0 || s.active == key);
        Response {
            hovered,
            pressed: s.active == key && self.input.down,
            clicked: s.active == key && self.input.released && s.hot == key,
            focused: s.focus == key,
            ..Response::default()
        }
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

    fn leaf(&mut self, kind: &'static str, id: Id, el: Element) -> usize {
        let i = self.push(kind, id, el);
        self.last_closed = i;
        i
    }

    fn text_content(&self, text: &str, step: TypeStep, color: Srgba) -> TextContent {
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
                interactive: !desc.disabled,
                radius: t.radius.button,
                ..Element::default()
            },
        );
        let key = self.nodes[i].key;
        let r = if desc.disabled {
            Response::default()
        } else {
            self.respond(key)
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
        let n = &mut self.nodes[i];
        n.el.background = bg;
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
        let v = if value.is_finite() {
            value.clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.nodes[i].widget = Widget::Progress { value: v };
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
        let mut r = self.respond(key);
        let (lo, hi) = if desc.min == desc.max {
            (0.0, 1.0)
        } else {
            (desc.min, desc.max)
        };
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

    fn thumb(&self) -> f32 {
        self.sizes().control_sm - 2.0 * self.theme.space.xs
    }

    fn switch_size(&self) -> (f32, f32) {
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
        let key = self.nodes[i].key;
        let mut r = self.respond(key);
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
                ..Element::default()
            },
        );
        self.nodes[i].focusable = true;
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
                ..Element::default()
            },
        );
        let mut content = self.text_content(text, t.type_ramp.meta, t.palette.fg);
        content.wrap = false;
        node.content = Content::Text(content);
        node.widget = Widget::Tooltip;
        node.probe_text = text.to_owned();
        node.anchor = anchor;
        let index = self.nodes.len();
        self.nodes.push(node);
        self.kinds.push(("tooltip", false));
        self.floating.push(index);
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
        self.building = false;
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
            layout(
                &mut self.nodes,
                f,
                screen,
                &self.fonts,
                &offset,
                dpr,
                screen,
            );
            let a = self.nodes[self.nodes[f].anchor].rect;
            let (w, h) = (self.nodes[f].w, self.nodes[f].h);
            let gap = self.theme.space.xs;
            let x = (a.x + a.width / 2.0 - w / 2.0)
                .min(safe.right() - w)
                .max(safe.x);
            let mut y = a.bottom() + gap;
            if y + h > safe.bottom() {
                y = a.y - gap - h;
            }
            let area = Rect::new(x, y, w, h);
            self.nodes[f].el.width = Size::Fixed(w);
            self.nodes[f].el.height = Size::Fixed(h);
            layout(&mut self.nodes, f, area, &self.fonts, &offset, dpr, screen);
        }

        let mut order = preorder(&self.nodes, 0);
        for &f in &self.floating {
            order.extend(preorder(&self.nodes, f));
        }

        // Interaction state for the next tick.
        let present: BTreeMap<u64, usize> = order.iter().map(|&i| (self.nodes[i].key, i)).collect();
        self.state
            .scroll
            .retain(|k, _| present.get(k).is_some_and(|&i| self.nodes[i].el.scroll));
        for (k, v) in self.state.scroll.iter_mut() {
            *v = v.clamp(0.0, self.nodes[present[k]].max_scroll());
        }
        if !present.contains_key(&self.state.focus) {
            self.state.focus = 0;
        }
        if !self.input.down || !present.contains_key(&self.state.active) {
            self.state.active = 0;
        }
        let mut hits = Vec::new();
        let mut elements = Vec::new();
        for &i in &order[1..] {
            let n = &self.nodes[i];
            let visible = n.rect.intersect(&n.clip);
            // Scrolled or clipped out of view: not on screen, not listed. A
            // zero-size element (an empty spacer) is listed where it sits.
            let (r, c) = (&n.rect, &n.clip);
            let outside =
                r.x > c.right() || r.right() < c.x || r.y > c.bottom() || r.bottom() < c.y;
            if outside || (visible.is_empty() && !r.is_empty()) {
                continue;
            }
            if !visible.is_empty() && (n.el.interactive || n.el.scroll || n.focusable) {
                hits.push(Hit {
                    key: n.key,
                    rect: visible,
                    scroll: n.el.scroll && !n.el.interactive,
                    focusable: n.focusable,
                });
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
        self.state.hits = hits;

        let cmds = self.commands(&order);
        self.frame = UiFrame {
            viewport: vp,
            elements,
            items: assign_levels(cmds),
            warnings: std::mem::take(&mut self.warnings),
        };
        &self.frame
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
        let quad = |rect: Rect, radius: f32, color: Srgba| DrawCmd::Quad {
            rect,
            radius,
            color: color.linear(),
            clip,
        };
        if let Widget::Field { focused: true, .. } = n.widget {
            let ring = t.space.xxs;
            cmds.push(quad(rect.outset(ring), n.el.radius + ring, p.focus));
        }
        if let Some(bg) = n.el.background {
            let r = match n.widget {
                Widget::Button { pressed: true } => rect.scaled(t.press_scale),
                _ => rect,
            };
            cmds.push(quad(r, n.el.radius, bg));
        }
        match n.widget {
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
                let font = self.fonts.get(tc.font);
                if let Some(handle) = font.handle {
                    let m = &font.metrics;
                    let size = tc.step.size;
                    let lh = tc.line_height();
                    let inner = Rect::new(
                        rect.x + n.el.padding.left,
                        rect.y + n.el.padding.top,
                        rect.width - n.el.padding.horizontal(),
                        rect.height - n.el.padding.vertical(),
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
                        let free = inner.width - line.width;
                        let x = inner.x
                            + match tc.align {
                                TextAlign::Start => 0.0,
                                TextAlign::Center => free / 2.0,
                                TextAlign::End => free,
                            };
                        let baseline =
                            ly + (lh - (m.ascender - m.descender) * size) / 2.0 + m.ascender * size;
                        cmds.push(DrawCmd::Text {
                            x: snap(x),
                            baseline: snap(baseline),
                            size,
                            text: line.text.clone(),
                            color: tc.color.linear(),
                            font: handle,
                            bounds: Rect::new(x, ly, line.width, lh),
                            clip: text_clip,
                            advances: line.text.chars().map(|c| m.advance(c) * size).collect(),
                        });
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
                            color: p.accent.linear(),
                            clip: text_clip,
                        });
                    }
                }
            }
            Content::Image { texture, uv, tint } => cmds.push(DrawCmd::Image {
                rect,
                texture: *texture,
                uv: *uv,
                color: tint.linear(),
                clip,
            }),
            Content::None => {}
        }
    }

    /// The interaction state (hover, press, focus, caret, scroll offsets and
    /// last frame's hit rectangles) as bytes, for the host's world snapshot:
    /// restoring it with the world replays the UI exactly.
    pub fn save_state(&self) -> Vec<u8> {
        let s = &self.state;
        let mut b = Vec::new();
        let u32_ = |b: &mut Vec<u8>, v: u32| b.extend_from_slice(&v.to_le_bytes());
        let u64_ = |b: &mut Vec<u8>, v: u64| b.extend_from_slice(&v.to_le_bytes());
        let f32_ = |b: &mut Vec<u8>, v: f32| b.extend_from_slice(&v.to_le_bytes());
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
            for v in [h.rect.x, h.rect.y, h.rect.width, h.rect.height] {
                f32_(&mut b, v);
            }
            b.push(u8::from(h.scroll) | u8::from(h.focusable) << 1);
        }
        b
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
        let s = r.state().ok_or_else(|| bad("truncated".into()))?;
        let (version, s) = s;
        if version != STATE_VERSION {
            return Err(bad(format!("version {version}, expected {STATE_VERSION}")));
        }
        if r.at != bytes.len() {
            return Err(bad("trailing bytes".into()));
        }
        self.state = s;
        Ok(())
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

    fn state(&mut self) -> Option<(u32, State)> {
        let version = self.u32()?;
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
            let key = self.u64()?;
            let rect = Rect::new(self.f32()?, self.f32()?, self.f32()?, self.f32()?);
            let flags = self.u8()?;
            s.hits.push(Hit {
                key,
                rect,
                scroll: flags & 1 != 0,
                focusable: flags & 2 != 0,
            });
        }
        Some((version, s))
    }
}
