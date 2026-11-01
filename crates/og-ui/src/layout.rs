//! Clay-style layout: one layout per frame in five walks over the tree.
//!
//! 1. **Widths, bottom-up**: every element's preferred (fit) and minimum
//!    width from its content: text unwrapped and its longest word, children
//!    summed along a row and maxed across a column, plus padding and gaps.
//! 2. **Widths, top-down**: a row gives each child its preferred width, then
//!    grows `Grow` children into the free space (the smallest first, until
//!    they level or reach their max) or shrinks `Fit`/`Grow` children down to
//!    their minimum (the widest first) when it overflows; a wrapping row
//!    breaks its children into lines first and grows within each line. A
//!    column gives `Grow` (or `Cross::Stretch`) children its inner width and
//!    caps the others at it. Text now wraps at its final width.
//! 3. **Heights, bottom-up**, from the wrapped text and the line breaks.
//! 4. **Heights, top-down**: the same grow/shrink along a column; a scroll
//!    container never shrinks its children (they overflow and scroll).
//! 5. **Positions**: main-axis `Align`, cross-axis `Cross`, scroll offsets,
//!    clip rectangles; rectangles snap to device pixels (`Viewport::dpr`).

use crate::FontMetrics;
use crate::element::{Edges, GridFlow, LayoutMode, Patch, Query, Track};
use crate::motion::Fx;
use crate::style::{Srgba, TypeStep};
use crate::text::{Line, break_lines, is_cjk, longest_word, widest_line};
use og_render::TextureHandle;

/// A rectangle in CSS pixels, origin at the top left, y down.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    pub fn intersect(&self, o: &Rect) -> Rect {
        let x = self.x.max(o.x);
        let y = self.y.max(o.y);
        let r = self.right().min(o.right());
        let b = self.bottom().min(o.bottom());
        Rect::new(x, y, (r - x).max(0.0), (b - y).max(0.0))
    }

    pub fn overlaps(&self, o: &Rect) -> bool {
        self.x < o.right() && o.x < self.right() && self.y < o.bottom() && o.y < self.bottom()
    }

    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Grown by `d` on every side.
    /// The rectangle less `pad` on each side (never negative).
    pub fn inset(&self, pad: &Insets) -> Rect {
        Rect::new(
            self.x + pad.left,
            self.y + pad.top,
            (self.width - pad.horizontal()).max(0.0),
            (self.height - pad.vertical()).max(0.0),
        )
    }

    pub fn outset(&self, d: f32) -> Rect {
        Rect::new(
            self.x - d,
            self.y - d,
            self.width + 2.0 * d,
            self.height + 2.0 * d,
        )
    }

    /// Scaled by `k` around its centre.
    pub fn scaled(&self, k: f32) -> Rect {
        let (cx, cy) = self.center();
        Rect::new(
            cx - self.width * k / 2.0,
            cy - self.height * k / 2.0,
            self.width * k,
            self.height * k,
        )
    }

    /// Edges rounded to the device pixel grid.
    pub fn snapped(&self, dpr: f32) -> Rect {
        let s = |v: f32| (v * dpr).round() / dpr;
        let (x, y) = (s(self.x), s(self.y));
        Rect::new(x, y, s(self.right()) - x, s(self.bottom()) - y)
    }
}

/// Space inside an element's edges (padding) or a device's unsafe margins.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Insets {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Insets {
    pub const fn all(v: f32) -> Insets {
        Insets {
            left: v,
            top: v,
            right: v,
            bottom: v,
        }
    }

    /// `x` left and right, `y` top and bottom.
    pub const fn xy(x: f32, y: f32) -> Insets {
        Insets {
            left: x,
            top: y,
            right: x,
            bottom: y,
        }
    }

    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }

    pub fn add(&self, o: &Insets) -> Insets {
        Insets {
            left: self.left + o.left,
            top: self.top + o.top,
            right: self.right + o.right,
            bottom: self.bottom + o.bottom,
        }
    }
}

/// How an element is sized along one axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Size {
    /// Its content, between `min` and `max` (the default: 0 to unbounded).
    Fit { min: f32, max: f32 },
    /// Its content, then whatever its parent has left, between `min` and `max`.
    Grow { min: f32, max: f32 },
    /// Exactly this many CSS pixels.
    Fixed(f32),
    /// A fraction (0–1) of the parent's inner size.
    Percent(f32),
}

impl Default for Size {
    fn default() -> Size {
        Size::fit()
    }
}

impl Size {
    pub const fn fit() -> Size {
        Size::Fit {
            min: 0.0,
            max: f32::INFINITY,
        }
    }

    pub const fn grow() -> Size {
        Size::Grow {
            min: 0.0,
            max: f32::INFINITY,
        }
    }

    pub const fn px(v: f32) -> Size {
        Size::Fixed(v)
    }

    pub const fn percent(f: f32) -> Size {
        Size::Percent(f)
    }

    fn bounds(&self) -> (f32, f32) {
        match *self {
            Size::Fit { min, max } | Size::Grow { min, max } => (min, max.max(min)),
            Size::Fixed(v) => (v, v),
            Size::Percent(_) => (0.0, f32::INFINITY),
        }
    }
}

/// The main axis: children left to right (`Row`) or top to bottom (`Column`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Direction {
    #[default]
    Row,
    Column,
}

/// Where children sit along the main axis when there is room left.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
    /// The free space split evenly between children.
    SpaceBetween,
}

/// Where children sit across the main axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Cross {
    #[default]
    Start,
    Center,
    End,
    /// Fill the cross axis (a `Fit` child takes the parent's inner size).
    Stretch,
    /// A row's children share their first text baseline (rows of mixed
    /// type sizes); a column treats it as `Start`.
    Baseline,
}

/// Horizontal alignment of text lines inside their element.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextAlign {
    #[default]
    Start,
    Center,
    End,
}

/// A container: its layout and its look. The zero value is a transparent
/// row that fits its content.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Element {
    pub direction: Direction,
    pub width: Size,
    pub height: Size,
    pub padding: Insets,
    /// Space between children (and between wrapped lines).
    pub gap: f32,
    pub align: Align,
    pub cross: Cross,
    /// A row breaks its children into lines at its width.
    pub wrap: bool,
    /// Scrolls its content vertically and clips it.
    pub scroll: bool,
    /// Scrolls its content horizontally and clips it (with `scroll`: both ways).
    pub scroll_x: bool,
    /// Reports hover, press and click through `Ui::close`.
    pub interactive: bool,
    pub background: Option<Srgba>,
    /// Corner radius in CSS pixels (clamped to half the shorter side).
    pub radius: f32,
    /// The frame its background takes (`None`: a plain fill). Without
    /// a style, or a style without that part, the background is the fill.
    pub frame: Option<crate::style_file::StylePart>,
    /// Width over height (0: none). With the height left to fit, the
    /// element is as tall as its laid-out width over this; with a fixed
    /// height and the width left to fit, as wide as the height times it
    /// (an image keeps its picture's proportions in a growing column).
    pub aspect: f32,
}

impl Element {
    pub fn row() -> Element {
        Element::default()
    }

    pub fn column() -> Element {
        Element {
            direction: Direction::Column,
            ..Element::default()
        }
    }
}

/// Where an anchored layer sits in the safe area of the play surface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Anchor {
    #[default]
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl Anchor {
    /// The anchor's point as fractions of the area (0 start, 0.5 centre, 1 end).
    pub fn fractions(self) -> (f32, f32) {
        match self {
            Anchor::TopLeft => (0.0, 0.0),
            Anchor::Top => (0.5, 0.0),
            Anchor::TopRight => (1.0, 0.0),
            Anchor::Left => (0.0, 0.5),
            Anchor::Center => (0.5, 0.5),
            Anchor::Right => (1.0, 0.5),
            Anchor::BottomLeft => (0.0, 1.0),
            Anchor::Bottom => (0.5, 1.0),
            Anchor::BottomRight => (1.0, 1.0),
        }
    }
}

/// How a layer outside the tree is placed, and its drawing rank.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Float {
    /// In the tree.
    None,
    /// At an anchor of the safe area, inset by `margin`, moved by `offset`.
    Anchored {
        anchor: Anchor,
        margin: f32,
        dx: f32,
        dy: f32,
        /// Over dialogs and spotlights (a tutorial callout, a dragged item).
        over: bool,
    },
    /// A full-surface backdrop over the tree; its child is the dialog.
    Modal,
    /// Under (or over) its anchor node, at least as wide.
    Popover,
    /// The toast stack, at an anchor.
    Toasts { anchor: Anchor, margin: f32 },
    /// Under (or over) its anchor node.
    Tooltip,
    /// Placed by [`crate::world`]: an element anchored in the world, a
    /// world panel or a meta layer, laid out at its own size.
    World,
}

impl Float {
    /// Layers draw in rank order, then in the order they were built.
    pub fn rank(&self) -> u8 {
        match self {
            Float::None => 0,
            Float::Anchored { over: false, .. } => 1,
            Float::Anchored { over: true, .. } => 3,
            Float::Modal => 2,
            Float::Popover => 3,
            Float::Toasts { .. } => 4,
            Float::Tooltip => 5,
            Float::World => 0,
        }
    }
}

/// A rectangle relative to an element: `f*` are fractions of its size, the
/// rest CSS px added to them (x, y from its top left).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Frame {
    pub fx: f32,
    pub fy: f32,
    pub fw: f32,
    pub fh: f32,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Frame {
    /// The whole element.
    pub const FULL: Frame = Frame {
        fx: 0.0,
        fy: 0.0,
        fw: 1.0,
        fh: 1.0,
        x: 0.0,
        y: 0.0,
        w: 0.0,
        h: 0.0,
    };

    /// A `w`×`h` box whose centre is the element's centre moved by (dx, dy).
    pub fn centered(dx: f32, dy: f32, w: f32, h: f32) -> Frame {
        Frame {
            fx: 0.5,
            fy: 0.5,
            x: dx - w / 2.0,
            y: dy - h / 2.0,
            w,
            h,
            ..Frame::default()
        }
    }

    /// A fraction of the element's width from its left, full height.
    pub fn left(from: f32, to: f32) -> Frame {
        Frame {
            fx: from,
            fw: (to - from).max(0.0),
            fh: 1.0,
            ..Frame::default()
        }
    }

    pub fn resolve(&self, r: &Rect) -> Rect {
        Rect::new(
            r.x + self.fx * r.width + self.x,
            r.y + self.fy * r.height + self.y,
            self.fw * r.width + self.w,
            self.fh * r.height + self.h,
        )
    }
}

/// Extra drawing of a widget, in its own frame, after its background.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Shape {
    /// A rounded rectangle (`radius` ≥ half the short side: a capsule or disc).
    Rect {
        frame: Frame,
        radius: f32,
        color: Srgba,
    },
    /// A solid bar rotated by `angle` (radians, clockwise on screen) about its centre.
    Bar {
        frame: Frame,
        angle: f32,
        color: Srgba,
    },
    /// The half of a disc on the side `angle` points to (0: the right half,
    /// clockwise on screen).
    Half {
        frame: Frame,
        angle: f32,
        color: Srgba,
    },
    /// A texture region.
    Image {
        frame: Frame,
        texture: TextureHandle,
        uv: [f32; 4],
        color: Srgba,
    },
    /// One line of text centred in the frame, drawn only inside `clip`
    /// (`Frame::FULL`: the whole element; a meter's label is two of these,
    /// one ink over the fill and one over the track).
    Text {
        frame: Frame,
        text: String,
        step: TypeStep,
        color: Srgba,
        font: FontRole,
        clip: Frame,
    },
    /// An icon of an icon set (an MTSDF font, `IconAtlas::font`): the
    /// glyph's em square fitted to the frame's shorter side, centred.
    Glyph {
        frame: Frame,
        font: og_render::FontHandle,
        ch: char,
        color: Srgba,
    },
}

/// Which font a text uses.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FontRole {
    #[default]
    Body,
    /// Display titles and key numbers (falls back to `Body`).
    Display,
    /// Code, ids and prices (falls back to `Body`).
    Mono,
    /// Levels and scores (falls back
    /// to `Display`, then `Body`).
    Rounded,
}

impl FontRole {
    /// The role a style's text effects are keyed by (`body`, `display`,
    /// `numeric`): a rounded text takes the display role's.
    pub fn effect_index(self) -> usize {
        match self {
            FontRole::Body => 0,
            FontRole::Display | FontRole::Rounded => 1,
            FontRole::Mono => 2,
        }
    }

    /// The role an IDL name names (`body`, `display`, `mono`, `rounded`).
    pub fn from_name(name: &str) -> Option<FontRole> {
        Some(match name {
            "body" => FontRole::Body,
            "display" => FontRole::Display,
            "mono" => FontRole::Mono,
            "rounded" => FontRole::Rounded,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextContent {
    pub text: String,
    pub step: TypeStep,
    pub color: Srgba,
    pub align: TextAlign,
    pub wrap: bool,
    pub font: FontRole,
    /// Cut with "…" where it does not fit: a one-line text at its width, a
    /// wrapping one at its last line (`max_lines`).
    pub ellipsis: bool,
    /// The most lines a wrapping text shows (0: no limit).
    pub max_lines: u32,
}

impl TextContent {
    /// The type step's tracking, in em: the CJK tracking for text with CJK
    /// characters (0 unless a skin sets it), none for a shaped script.
    pub fn tracking(&self) -> f32 {
        step_tracking(&self.step, &self.text)
    }

    pub fn line_height(&self) -> f32 {
        if self.text.chars().any(is_cjk) {
            self.step.cjk_line_height
        } else {
            self.step.line_height
        }
    }
}

/// The ellipsis a font draws: "…" when it has the character, else "...".
pub(crate) fn ellipsis_of(m: &FontMetrics) -> &'static str {
    if m.has('…') { "…" } else { "..." }
}

/// Ellipsis truncation: lines past `max_lines` go, and the last line kept
/// (or a one-line text) wider than `width` loses characters from its end
/// until it fits with the ellipsis after it (a cut between words drops the
/// space before the ellipsis).
fn truncate_lines(lines: &mut Vec<Line>, t: &TextContent, m: &FontMetrics, width: f32) {
    let (size, tracking) = (t.step.size, t.tracking());
    let cut = t.max_lines > 0 && lines.len() > t.max_lines as usize;
    if cut {
        lines.truncate(t.max_lines as usize);
    }
    let Some(last) = lines.last_mut() else {
        return;
    };
    if !cut && last.width <= width {
        return;
    }
    let dots = ellipsis_of(m);
    let mut chars: Vec<char> = last.text.chars().collect();
    loop {
        let body: String = chars.iter().collect();
        let text = format!("{}{dots}", body.trim_end());
        let w = m.measure_tracked(&text, size, tracking);
        if w <= width || chars.is_empty() {
            last.text = text;
            last.width = w;
            return;
        }
        chars.pop();
    }
}

/// A step's tracking for `text`: none with a shaped script (its joins would break), the
/// step's CJK tracking with CJK characters (0 unless a skin's ramp sets it).
pub(crate) fn step_tracking(step: &TypeStep, text: &str) -> f32 {
    if text.chars().any(og_i18n::needs_shaping) {
        0.0
    } else if text.chars().any(is_cjk) {
        step.cjk_tracking_em
    } else {
        step.tracking_em
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Content {
    None,
    Text(TextContent),
    /// A paragraph of rich text spans (`crate::rich`).
    Rich(crate::rich::RichContent),
    Image {
        texture: Option<TextureHandle>,
        uv: [f32; 4],
        tint: Srgba,
    },
}

/// What a widget draws besides its background and content.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Widget {
    None,
    Button {
        pressed: bool,
        /// How far in the press is on the style's spring (0 rest, 1 held).
        press: f32,
    },
    Progress {
        value: f32,
    },
    Slider {
        t: f32,
        active: bool,
    },
    Toggle {
        on: bool,
    },
    Field(EditView),
    /// A multi-line text area: its text (the value with any composition),
    /// wrapped at draw time, and its placeholder.
    TextArea {
        edit: EditView,
        text: String,
        placeholder: String,
        line_height: f32,
    },
    Tooltip,
    /// A virtualised list or grid's filler: no drawing.
    Spacer,
}

/// A focused text control's caret, selection and composition, in
/// characters of the text it shows, and its scroll.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct EditView {
    pub focused: bool,
    pub caret: usize,
    /// The selection `[start, end)` (empty: none).
    pub selection: (usize, usize),
    /// The composition's range, drawn underlined.
    pub composition: Option<(usize, usize)>,
    pub scroll: f32,
    /// The caret shows this tick (its blink).
    pub caret_on: bool,
    /// The caret is drawn after the glyph before it (`caret_x_at`).
    pub upstream: bool,
}

pub(crate) const NONE: usize = usize::MAX;

/// A staggered entrance: descendant k starts `k × step` ticks after `since`
/// and takes `ticks`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Stagger {
    pub since: u64,
    pub step: f32,
    pub ticks: f32,
    pub kind: crate::motion::TransitionKind,
}

#[derive(Clone, Debug)]
pub(crate) struct Node {
    pub key: u64,
    pub id: String,
    pub parent: usize,
    pub children: Vec<usize>,
    pub el: Element,
    pub content: Content,
    pub widget: Widget,
    /// The text the probe reports for it.
    pub probe_text: String,
    /// A disabled control (the probe reports it; contrast rules exempt it).
    pub disabled: bool,
    pub focusable: bool,
    /// A multi-line text control: ↑ and ↓ belong to it while it is edited.
    pub multiline: bool,
    /// For a floating element: the node it is placed against.
    pub anchor: usize,
    /// How it is placed when it is a layer of its own.
    pub float: Float,
    /// A transition's effect on its subtree.
    pub fx: Option<Fx>,
    /// Keyboard and gamepad focus can land on it.
    pub nav: bool,
    /// Left and right adjust it when focused (slider, stepper).
    pub adjust: bool,
    /// Draw the focus ring (focused by keyboard or gamepad).
    pub ring: bool,
    /// Bit k: pad k + 1's focus is on it (its ring in that player's colour).
    pub pad_rings: u8,
    /// Built but not shown (a closed dialog's slot): no layout, drawing or input.
    pub hidden: bool,
    /// A modal backdrop: takes the pointer, never focus.
    pub backdrop: bool,
    /// A focus scope (dialog, open menu): focus stays inside while it is the topmost.
    pub scope: bool,
    /// A scope Back does not close (a locked dialog): while it is the
    /// topmost, Back reaches `Ui::back`.
    pub locked: bool,
    /// Drawn after the background.
    pub shapes: Vec<Shape>,
    /// Widget data its children read (a list's row height, a grid's cell size).
    pub aux: [f32; 3],
    /// A heading label: the style's flourish frames it.
    pub heading: bool,
    /// What it tells the accessibility tree beyond its kind.
    pub a11y: crate::a11y::A11y,
    /// Its descendants enter one after another (a style's stagger).
    pub stagger: Option<Stagger>,
    /// The element model beyond its box (`crate::element`).
    pub props: crate::element::Props,
    /// Its children out of the flow (absolute, anchored), in build order;
    /// `children` holds the ones in the flow once the frame ends.
    pub out: Vec<usize>,
    /// A grid's resolved tracks and its children's cells.
    pub grid: Option<GridLayout>,
    /// Where it is drawn: its map from layout to screen (its transforms and
    /// its ancestors'), and its box's bounds there.
    pub world: crate::element::Affine,
    pub screen: Rect,
    /// Layout animation's offset and scale this tick (x, y, sx, sy).
    pub flip: [f32; 4],
    /// A drag's offset and a pinch's scale and turn (x, y, scale, radians).
    pub gesture: [f32; 4],
    // Computed by the frame.
    pub alpha: f32,
    pub scale: f32,
    pub inert: bool,
    /// The scroll container whose clip applies (NONE: the screen).
    pub clip_src: usize,
    // Computed.
    pub pref_w: f32,
    pub min_w: f32,
    pub pref_h: f32,
    pub min_h: f32,
    pub w: f32,
    pub h: f32,
    pub x: f32,
    pub y: f32,
    pub content_h: f32,
    /// The content's width (its children's extent), for horizontal scrolling.
    pub content_w: f32,
    pub lines: Vec<Line>,
    /// A rich text's lines.
    pub rich_lines: Vec<crate::rich::RichLine>,
    /// A rich text's links, by id, in order.
    pub links: Vec<String>,
    /// A wrapping row's lines: ranges of `children`.
    pub rows: Vec<(usize, usize)>,
    pub clip: Rect,
    pub rect: Rect,
}

impl Node {
    pub fn new(key: u64, id: String, parent: usize, el: Element) -> Node {
        Node {
            key,
            id,
            parent,
            children: Vec::new(),
            el,
            content: Content::None,
            widget: Widget::None,
            probe_text: String::new(),
            disabled: false,
            focusable: false,
            multiline: false,
            anchor: NONE,
            float: Float::None,
            fx: None,
            nav: false,
            adjust: false,
            ring: false,
            pad_rings: 0,
            hidden: false,
            backdrop: false,
            scope: false,
            locked: false,
            shapes: Vec::new(),
            aux: [0.0; 3],
            heading: false,
            a11y: crate::a11y::A11y::default(),
            stagger: None,
            props: crate::element::Props::default(),
            out: Vec::new(),
            grid: None,
            world: crate::element::Affine::IDENTITY,
            screen: Rect::default(),
            flip: [0.0, 0.0, 1.0, 1.0],
            gesture: [0.0, 0.0, 1.0, 0.0],
            alpha: 1.0,
            scale: 1.0,
            inert: false,
            clip_src: NONE,
            pref_w: 0.0,
            min_w: 0.0,
            pref_h: 0.0,
            min_h: 0.0,
            w: 0.0,
            h: 0.0,
            x: 0.0,
            y: 0.0,
            content_h: 0.0,
            content_w: 0.0,
            lines: Vec::new(),
            rich_lines: Vec::new(),
            links: Vec::new(),
            rows: Vec::new(),
            clip: Rect::default(),
            rect: Rect::default(),
        }
    }

    /// The scroll range: how far the content overflows the inner height.
    pub fn max_scroll(&self) -> f32 {
        if !self.el.scroll {
            return 0.0;
        }
        (self.content_h - (self.h - self.el.padding.vertical())).max(0.0)
    }

    /// The horizontal scroll range: how far the content overflows the inner width.
    pub fn max_scroll_x(&self) -> f32 {
        if !self.el.scroll_x {
            return 0.0;
        }
        (self.content_w - (self.w - self.el.padding.horizontal())).max(0.0)
    }

    /// Scrolls either way.
    pub fn scrolls(&self) -> bool {
        self.el.scroll || self.el.scroll_x
    }
}

/// The fonts layout measures with, by role.
pub(crate) trait Metrics {
    fn metrics_for(&self, role: FontRole, text: &str) -> &FontMetrics;
}

/// What layout measures text with: the fonts, and the measurements kept
/// from the frames before ([`TextCache`]).
pub(crate) struct Measure<'a> {
    pub fonts: &'a dyn Metrics,
    pub texts: &'a mut TextCache,
}

/// Everything a text's measurement depends on besides its string and the
/// fonts: the font role, the type size and tracking, wrapping, the
/// ellipsis and its line limit, and (for its lines) the width it breaks at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TextKey {
    font: FontRole,
    size: u32,
    tracking: u32,
    cjk_tracking: u32,
    wrap: bool,
    ellipsis: bool,
    max_lines: u32,
    /// The width the lines break or cut at (`None`: only at `\n`).
    width: Option<u32>,
}

impl TextKey {
    fn of(t: &TextContent, width: Option<f32>) -> TextKey {
        TextKey {
            font: t.font,
            size: t.step.size.to_bits(),
            tracking: t.step.tracking_em.to_bits(),
            cjk_tracking: t.step.cjk_tracking_em.to_bits(),
            wrap: t.wrap,
            ellipsis: t.ellipsis,
            max_lines: t.max_lines,
            width: width.map(f32::to_bits),
        }
    }
}

/// One kept measurement and the frame that last used it.
#[derive(Clone, Debug)]
struct Kept<V> {
    key: TextKey,
    value: V,
    frame: u64,
}

/// Text measurements kept across frames while a text's string, font, size
/// and width are unchanged: a HUD lays out the same texts every tick, and
/// measuring each again (its widest line and longest word, its line breaks
/// and ellipsis) was about a third of `og.ui.end`. What a frame did not use
/// goes at its [`TextCache::sweep`]; new fonts clear it
/// ([`crate::Ui::set_fonts`]), the only other input a measurement has.
#[derive(Clone, Debug, Default)]
pub(crate) struct TextCache {
    /// A text's preferred and minimum width ([`size_w`]).
    widths: std::collections::HashMap<String, Vec<Kept<(f32, f32)>>>,
    /// A text's lines at a width ([`distribute_w`]).
    lines: std::collections::HashMap<String, Vec<Kept<Vec<Line>>>>,
    frame: u64,
    /// Measurements this frame made (widths and lines each count one).
    made: u32,
    /// What the last swept frame made ([`crate::Ui::text_measurements`]).
    measured: u32,
}

/// The kept value of `text` under `key`, or `measure`'s, kept.
fn kept<V: Clone>(
    map: &mut std::collections::HashMap<String, Vec<Kept<V>>>,
    frame: u64,
    measured: &mut u32,
    text: &str,
    key: TextKey,
    measure: impl FnOnce() -> V,
) -> V {
    let measure = || {
        *measured += 1;
        measure()
    };
    if let Some(list) = map.get_mut(text) {
        if let Some(k) = list.iter_mut().find(|k| k.key == key) {
            k.frame = frame;
            return k.value.clone();
        }
        let value = measure();
        list.push(Kept {
            key,
            value: value.clone(),
            frame,
        });
        return value;
    }
    let value = measure();
    map.insert(
        text.to_owned(),
        vec![Kept {
            key,
            value: value.clone(),
            frame,
        }],
    );
    value
}

impl TextCache {
    fn widths(&mut self, t: &TextContent, measure: impl FnOnce() -> (f32, f32)) -> (f32, f32) {
        kept(
            &mut self.widths,
            self.frame,
            &mut self.made,
            &t.text,
            TextKey::of(t, None),
            measure,
        )
    }

    fn lines(
        &mut self,
        t: &TextContent,
        width: Option<f32>,
        measure: impl FnOnce() -> Vec<Line>,
    ) -> Vec<Line> {
        kept(
            &mut self.lines,
            self.frame,
            &mut self.made,
            &t.text,
            TextKey::of(t, width),
            measure,
        )
    }

    /// Keeps `t`'s measurements through this frame's sweep: a node that
    /// kept its layout (docs/ui.md §6.3) did not measure, but still uses them.
    fn touch(&mut self, t: &TextContent) {
        let frame = self.frame;
        if let Some(list) = self.widths.get_mut(&t.text) {
            list.iter_mut().for_each(|k| k.frame = frame);
        }
        if let Some(list) = self.lines.get_mut(&t.text) {
            list.iter_mut().for_each(|k| k.frame = frame);
        }
    }

    /// Ends a frame: what it did not measure goes.
    pub(crate) fn sweep(&mut self) {
        let frame = self.frame;
        for map in [&mut self.widths as &mut dyn Sweep, &mut self.lines] {
            map.sweep(frame);
        }
        self.frame += 1;
        self.measured = std::mem::take(&mut self.made);
    }

    /// The measurements the last laid-out frame made.
    pub(crate) fn measured(&self) -> u32 {
        self.measured
    }

    /// Forgets every measurement (the fonts changed).
    pub(crate) fn clear(&mut self) {
        self.widths.clear();
        self.lines.clear();
    }

    /// How many measurements are kept (tests).
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.widths.values().map(Vec::len).sum::<usize>()
            + self.lines.values().map(Vec::len).sum::<usize>()
    }
}

trait Sweep {
    fn sweep(&mut self, frame: u64);
}

impl<V> Sweep for std::collections::HashMap<String, Vec<Kept<V>>> {
    fn sweep(&mut self, frame: u64) {
        self.retain(|_, list| {
            list.retain(|k| k.frame == frame);
            !list.is_empty()
        });
    }
}

pub(crate) fn preorder(nodes: &[Node], root: usize) -> Vec<usize> {
    let mut order = Vec::new();
    let mut stack = vec![root];
    while let Some(i) = stack.pop() {
        order.push(i);
        let n = &nodes[i];
        if n.out.is_empty() {
            stack.extend(n.children.iter().rev());
        } else {
            stack.extend(all_children(n).into_iter().rev());
        }
    }
    order
}

/// A node's children in and out of the flow, in build order (node indices
/// grow in build order).
pub(crate) fn all_children(n: &Node) -> Vec<usize> {
    let mut all: Vec<usize> = n.children.iter().chain(&n.out).copied().collect();
    all.sort_unstable();
    all
}

/// The preorder with each node's children in drawing order: by `z`, then
/// as built (a lifted card draws over its neighbours and is hit first).
pub(crate) fn draw_order(nodes: &[Node], root: usize) -> Vec<usize> {
    let mut order = Vec::new();
    let mut stack = vec![root];
    while let Some(i) = stack.pop() {
        order.push(i);
        let mut kids = all_children(&nodes[i]);
        if kids.iter().any(|&c| nodes[c].props.z != 0) {
            og_core::sort::sort_by_key(&mut kids, |&c| nodes[c].props.z);
        }
        stack.extend(kids.into_iter().rev());
    }
    order
}

/// For each position of `order` (a preorder), the position just past its
/// subtree.
pub(crate) fn subtree_ends(nodes: &[Node], order: &[usize]) -> Vec<usize> {
    let mut ends = vec![order.len(); order.len()];
    let mut open: Vec<usize> = Vec::new();
    for (k, &i) in order.iter().enumerate() {
        let parent = nodes[i].parent;
        while let Some(&top) = open.last() {
            if order[top] == parent {
                break;
            }
            ends[top] = k;
            open.pop();
        }
        open.push(k);
    }
    ends
}

/// Raises `items` (by index into `w`) towards a common level so their sum
/// grows by `extra`, each capped at its own max: the smallest grow first.
fn fill_up(w: &mut [f32], idx: &[usize], max: &[f32], extra: f32) {
    if extra <= 0.0 || idx.is_empty() {
        return;
    }
    let base: Vec<f32> = idx.iter().map(|&i| w[i]).collect();
    let cap: Vec<f32> = idx
        .iter()
        .zip(&base)
        .map(|(&i, &b)| max[i].max(b))
        .collect();
    let f = |level: f32| -> f32 {
        base.iter()
            .zip(&cap)
            .map(|(&b, &c)| level.clamp(b, c) - b)
            .sum()
    };
    let mut points: Vec<f32> = base
        .iter()
        .chain(cap.iter())
        .copied()
        .filter(|v| v.is_finite())
        .collect();
    og_core::sort::sort_by(&mut points, f32::total_cmp);
    points.dedup();
    let mut lo = base.iter().copied().fold(f32::INFINITY, f32::min);
    let mut level = f32::INFINITY;
    let mut solved = false;
    for &p in &points {
        if p <= lo {
            continue;
        }
        if f(p) >= extra {
            let active = base
                .iter()
                .zip(&cap)
                .filter(|&(&b, &c)| b <= lo && c > lo)
                .count();
            level = lo + (extra - f(lo)) / active.max(1) as f32;
            solved = true;
            break;
        }
        lo = p;
    }
    if !solved {
        let active = base
            .iter()
            .zip(&cap)
            .filter(|&(&b, &c)| b <= lo && c > lo)
            .count();
        if active > 0 {
            level = lo + (extra - f(lo)) / active as f32;
        }
    }
    for (k, &i) in idx.iter().enumerate() {
        w[i] = level.clamp(base[k], cap[k]);
    }
}

/// Lowers `items` towards a common level so their sum shrinks by `deficit`,
/// none below its min: the widest shrink first.
fn fill_down(w: &mut [f32], idx: &[usize], min: &[f32], deficit: f32) {
    let mut neg: Vec<f32> = w.iter().map(|v| -v).collect();
    let neg_cap: Vec<f32> = min.iter().map(|v| -v).collect();
    fill_up(&mut neg, idx, &neg_cap, deficit);
    for &i in idx {
        w[i] = -neg[i];
    }
}

/// Grow and shrink one line of children along the main axis.
fn flex(sizes: &mut [f32], grow_max: &[Option<f32>], shrink_min: &[Option<f32>], avail: f32) {
    let total: f32 = sizes.iter().sum();
    if total < avail {
        let idx: Vec<usize> = (0..sizes.len())
            .filter(|&i| grow_max[i].is_some())
            .collect();
        let max: Vec<f32> = grow_max.iter().map(|m| m.unwrap_or(0.0)).collect();
        fill_up(sizes, &idx, &max, avail - total);
    } else if total > avail {
        let idx: Vec<usize> = (0..sizes.len())
            .filter(|&i| shrink_min[i].is_some_and(|m| m < sizes[i]))
            .collect();
        let min: Vec<f32> = shrink_min.iter().map(|m| m.unwrap_or(0.0)).collect();
        fill_down(sizes, &idx, &min, total - avail);
    }
}

fn apply_size(size: Size, pref: f32, min: f32) -> (f32, f32) {
    match size {
        Size::Fixed(v) => (v, v),
        Size::Fit { .. } | Size::Grow { .. } => {
            let (lo, hi) = size.bounds();
            let p = pref.clamp(lo, hi);
            (p, min.clamp(lo, hi).min(p))
        }
        Size::Percent(_) => (pref, min.min(pref)),
    }
}

fn gaps(gap: f32, n: usize) -> f32 {
    gap * n.saturating_sub(1) as f32
}

fn size_w(nodes: &mut [Node], i: usize, measure: &mut Measure<'_>) {
    if nodes[i].props.layout == LayoutMode::Grid && matches!(nodes[i].content, Content::None) {
        place_grid(nodes, i);
    }
    let n = &nodes[i];
    let fonts = measure.fonts;
    let (pref, min) = match &n.content {
        Content::Text(t) => measure.texts.widths(t, || {
            let m = fonts.metrics_for(t.font, &t.text);
            let pref = widest_line(&t.text, m, t.step.size, t.tracking());
            let min = if t.ellipsis {
                // It can shrink to its ellipsis.
                m.measure_tracked(ellipsis_of(m), t.step.size, t.tracking())
                    .min(pref)
            } else if t.wrap {
                longest_word(&t.text, m, t.step.size, t.tracking())
            } else {
                pref
            };
            (pref, min)
        }),
        Content::Rich(rc) => crate::rich::rich_widths(rc, fonts),
        Content::Image { .. } => (0.0, 0.0),
        Content::None if n.props.layout == LayoutMode::Stack => {
            let kids = || n.children.iter().map(|&c| &nodes[c]);
            (
                kids().map(|c| c.pref_w).fold(0.0, f32::max),
                kids().map(|c| c.min_w).fold(0.0, f32::max),
            )
        }
        Content::None if n.props.layout == LayoutMode::Grid => grid_intrinsic(nodes, i, true),
        Content::None => {
            let k = n.children.len();
            let kids = n.children.iter().map(|&c| &nodes[c]);
            match n.el.direction {
                Direction::Row => {
                    let pref = kids.clone().map(|c| c.pref_w).sum::<f32>() + gaps(n.el.gap, k);
                    let min = if n.el.wrap {
                        kids.map(|c| c.min_w).fold(0.0, f32::max)
                    } else {
                        kids.map(|c| c.min_w).sum::<f32>() + gaps(n.el.gap, k)
                    };
                    (pref, min)
                }
                Direction::Column => (
                    kids.clone().map(|c| c.pref_w).fold(0.0, f32::max),
                    kids.map(|c| c.min_w).fold(0.0, f32::max),
                ),
            }
        }
    };
    // A fixed height and an aspect ratio give the width.
    let (pref, min) = match (n.el.aspect > 0.0, n.el.height, n.el.width) {
        (true, Size::Fixed(h), Size::Fit { .. }) => {
            let w = (h - n.el.padding.vertical()).max(0.0) * n.el.aspect;
            (w, w)
        }
        _ => (pref, min),
    };
    let pad = n.el.padding.horizontal();
    // A horizontal scroller never shrinks its content: it overflows and scrolls.
    let min = if n.el.scroll_x { 0.0 } else { min };
    let (p, m) = apply_size(n.el.width, pad + pref, pad + min);
    nodes[i].pref_w = p;
    nodes[i].min_w = m;
}

/// A child's width across a column (or height across a row).
fn cross_size(size: Size, pref: f32, min: f32, inner: f32, stretch: bool) -> f32 {
    match size {
        Size::Fixed(v) => v,
        Size::Percent(p) => p * inner,
        Size::Grow { .. } => {
            let (lo, hi) = size.bounds();
            inner.clamp(lo, hi)
        }
        Size::Fit { .. } if stretch => {
            let (lo, hi) = size.bounds();
            inner.clamp(lo, hi)
        }
        Size::Fit { .. } => pref.min(inner).max(min),
    }
}

/// Main-axis sizes of a line of children: base, then grow or shrink.
fn main_sizes(
    nodes: &[Node],
    kids: &[usize],
    horizontal: bool,
    inner: f32,
    avail: f32,
    allow_shrink: bool,
) -> Vec<f32> {
    let mut sizes = Vec::with_capacity(kids.len());
    let mut grow = Vec::with_capacity(kids.len());
    let mut shrink = Vec::with_capacity(kids.len());
    for &c in kids {
        let n = &nodes[c];
        let (size, pref, min) = if horizontal {
            (n.el.width, n.pref_w, n.min_w)
        } else {
            (n.el.height, n.pref_h, n.min_h)
        };
        let base = match size {
            Size::Fixed(v) => v,
            Size::Percent(p) => p * inner,
            _ => pref,
        };
        sizes.push(base);
        grow.push(match size {
            Size::Grow { .. } => Some(size.bounds().1),
            _ => None,
        });
        shrink.push(match size {
            Size::Fit { .. } | Size::Grow { .. } if allow_shrink => Some(min),
            _ => None,
        });
    }
    flex(&mut sizes, &grow, &shrink, avail);
    sizes
}

fn distribute_w(nodes: &mut [Node], i: usize, measure: &mut Measure<'_>) {
    // A container query reads the element's own width, now known.
    if nodes[i].props.variants.iter().any(|v| v.when.container) {
        let w = nodes[i].w;
        let variants = nodes[i].props.variants.clone();
        for v in variants.iter().filter(|v| v.when.container) {
            if width_in(w, &v.when) {
                apply_patch(&mut nodes[i], &v.set);
            }
        }
    }
    let inner = (nodes[i].w - nodes[i].el.padding.horizontal()).max(0.0);
    // Out of the flow: sized against the parent's box.
    let outer = nodes[i].w;
    for c in nodes[i].out.clone() {
        let w = out_size(
            nodes[c].el.width,
            nodes[c].pref_w,
            outer,
            nodes[c].props.insets.start,
            nodes[c].props.insets.end,
        );
        nodes[c].w = w;
    }
    let fonts = measure.fonts;
    if let Content::Text(t) = &nodes[i].content {
        // A fit width snapped to the device pixel may land up to half a
        // pixel under the text it was measured from: that is no reason to wrap.
        let width = inner + 0.5;
        let lines = measure
            .texts
            .lines(t, (t.wrap || t.ellipsis).then_some(width), || {
                let m = fonts.metrics_for(t.font, &t.text);
                let mut lines = break_lines(
                    &t.text,
                    m,
                    t.step.size,
                    t.tracking(),
                    t.wrap.then_some(width),
                );
                if t.ellipsis {
                    truncate_lines(&mut lines, t, m, width);
                }
                lines
            });
        nodes[i].lines = lines;
        return;
    }
    if let Content::Rich(rc) = &nodes[i].content {
        let lines = crate::rich::rich_lines(rc, fonts, rc.wrap.then_some(inner + 0.5));
        nodes[i].rich_lines = lines;
        return;
    }
    let el = nodes[i].el;
    let kids = nodes[i].children.clone();
    if kids.is_empty() {
        return;
    }
    match nodes[i].props.layout {
        LayoutMode::Stack => {
            for &c in &kids {
                let n = &nodes[c];
                let ins = n.props.insets;
                let w = match (ins.start, ins.end) {
                    (Some(s), Some(e)) if !matches!(n.el.width, Size::Fixed(_)) => {
                        (inner - s - e).max(0.0)
                    }
                    _ => cross_size(
                        n.el.width,
                        n.pref_w,
                        n.min_w,
                        inner,
                        el.cross == Cross::Stretch,
                    ),
                };
                nodes[c].w = w;
            }
            return;
        }
        LayoutMode::Grid => {
            grid_columns(nodes, i, inner);
            return;
        }
        LayoutMode::Flex => {}
    }
    match el.direction {
        Direction::Column => {
            for &c in &kids {
                let n = &nodes[c];
                let w = cross_size(
                    n.el.width,
                    n.pref_w,
                    n.min_w,
                    inner,
                    el.cross == Cross::Stretch,
                );
                // In a horizontal scroller a fitting child keeps its width.
                let w = match n.el.width {
                    Size::Fit { .. } if el.scroll_x => w.max(n.pref_w),
                    _ => w,
                };
                nodes[c].w = w;
            }
        }
        Direction::Row if !el.wrap => {
            let avail = inner - gaps(el.gap, kids.len());
            let sizes = main_sizes(nodes, &kids, true, inner, avail, !el.scroll_x);
            for (&c, w) in kids.iter().zip(sizes) {
                nodes[c].w = w;
            }
        }
        Direction::Row => {
            // Each child at its preferred width, capped at the row; then lines.
            let mut base = Vec::with_capacity(kids.len());
            for &c in &kids {
                let n = &nodes[c];
                base.push(match n.el.width {
                    Size::Fixed(v) => v,
                    Size::Percent(p) => p * inner,
                    _ => n.pref_w.min(inner).max(n.min_w),
                });
            }
            let mut rows = Vec::new();
            let mut start = 0;
            let mut used = 0.0f32;
            for (k, &b) in base.iter().enumerate() {
                if k > start && used + el.gap + b > inner {
                    rows.push((start, k));
                    start = k;
                    used = b;
                } else {
                    used += if k > start { el.gap + b } else { b };
                }
            }
            rows.push((start, kids.len()));
            for &(s, e) in &rows {
                let line = &kids[s..e];
                let mut sizes = base[s..e].to_vec();
                let grow: Vec<Option<f32>> = line
                    .iter()
                    .map(|&c| match nodes[c].el.width {
                        Size::Grow { .. } => Some(nodes[c].el.width.bounds().1),
                        _ => None,
                    })
                    .collect();
                let none = vec![None; line.len()];
                flex(&mut sizes, &grow, &none, inner - gaps(el.gap, line.len()));
                for (&c, w) in line.iter().zip(sizes) {
                    nodes[c].w = w;
                }
            }
            nodes[i].rows = rows;
        }
    }
}

fn size_h(nodes: &mut [Node], i: usize) {
    let n = &nodes[i];
    let (pref, min) = match &n.content {
        Content::None if n.props.layout == LayoutMode::Stack => {
            let kids = || n.children.iter().map(|&c| &nodes[c]);
            (
                kids().map(|c| c.pref_h).fold(0.0, f32::max),
                kids().map(|c| c.min_h).fold(0.0, f32::max),
            )
        }
        Content::None if n.props.layout == LayoutMode::Grid => grid_intrinsic(nodes, i, false),
        Content::Text(t) => {
            let h = n.lines.len().max(1) as f32 * t.line_height();
            (h, h)
        }
        Content::Rich(_) => {
            let h: f32 = n.rich_lines.iter().map(|l| l.height).sum();
            (h, h)
        }
        Content::Image { .. } => (0.0, 0.0),
        Content::None => {
            let k = n.children.len();
            let kids = || n.children.iter().map(|&c| &nodes[c]);
            match n.el.direction {
                Direction::Column => (
                    kids().map(|c| c.pref_h).sum::<f32>() + gaps(n.el.gap, k),
                    kids().map(|c| c.min_h).sum::<f32>() + gaps(n.el.gap, k),
                ),
                Direction::Row if !n.el.wrap || n.rows.is_empty() => (
                    kids().map(|c| c.pref_h).fold(0.0, f32::max),
                    kids().map(|c| c.min_h).fold(0.0, f32::max),
                ),
                Direction::Row => {
                    let line_h = |s: usize, e: usize, f: &dyn Fn(&Node) -> f32| {
                        n.children[s..e]
                            .iter()
                            .map(|&c| f(&nodes[c]))
                            .fold(0.0, f32::max)
                    };
                    let rows = n.rows.len();
                    let pref = n
                        .rows
                        .iter()
                        .map(|&(s, e)| line_h(s, e, &|c| c.pref_h))
                        .sum::<f32>()
                        + gaps(n.el.gap, rows);
                    let min = n
                        .rows
                        .iter()
                        .map(|&(s, e)| line_h(s, e, &|c| c.min_h))
                        .sum::<f32>()
                        + gaps(n.el.gap, rows);
                    (pref, min)
                }
            }
        }
    };
    // An aspect ratio gives a fitting height from the laid-out width.
    let (pref, min) = match (n.el.aspect > 0.0, n.el.height) {
        (true, Size::Fit { .. }) => {
            let h = (n.w - n.el.padding.horizontal()).max(0.0) / n.el.aspect;
            (h, h)
        }
        _ => (pref, min),
    };
    let scroll = n.el.scroll && n.el.aspect <= 0.0;
    let pad = n.el.padding.vertical();
    let content_min = if scroll { 0.0 } else { min };
    let (p, m) = apply_size(n.el.height, pad + pref, pad + content_min);
    nodes[i].content_h = pref;
    nodes[i].pref_h = p;
    nodes[i].min_h = m;
}

fn distribute_h(nodes: &mut [Node], i: usize) {
    let el = nodes[i].el;
    let inner = (nodes[i].h - el.padding.vertical()).max(0.0);
    let outer = nodes[i].h;
    for c in nodes[i].out.clone() {
        let h = out_size(
            nodes[c].el.height,
            nodes[c].pref_h,
            outer,
            nodes[c].props.insets.top,
            nodes[c].props.insets.bottom,
        );
        nodes[c].h = h;
    }
    let kids = nodes[i].children.clone();
    if kids.is_empty() {
        return;
    }
    match nodes[i].props.layout {
        LayoutMode::Stack => {
            for &c in &kids {
                let n = &nodes[c];
                let ins = n.props.insets;
                let h = match (ins.top, ins.bottom) {
                    (Some(t), Some(b)) if !matches!(n.el.height, Size::Fixed(_)) => {
                        (inner - t - b).max(0.0)
                    }
                    _ => cross_size(
                        n.el.height,
                        n.pref_h,
                        n.min_h,
                        inner,
                        el.cross == Cross::Stretch,
                    ),
                };
                nodes[c].h = h;
            }
            return;
        }
        LayoutMode::Grid => {
            grid_rows(nodes, i, inner);
            return;
        }
        LayoutMode::Flex => {}
    }
    match el.direction {
        Direction::Column => {
            let avail = inner - gaps(el.gap, kids.len());
            let sizes = main_sizes(nodes, &kids, false, inner, avail, !el.scroll);
            for (&c, h) in kids.iter().zip(sizes) {
                nodes[c].h = h;
            }
        }
        Direction::Row if !el.wrap => {
            for &c in &kids {
                let n = &nodes[c];
                let h = cross_size(
                    n.el.height,
                    n.pref_h,
                    n.min_h,
                    inner,
                    el.cross == Cross::Stretch,
                );
                // A Fit child keeps its content height across a row.
                let h = match n.el.height {
                    Size::Fit { .. } if el.cross != Cross::Stretch => n.pref_h,
                    _ => h,
                };
                nodes[c].h = h;
            }
        }
        Direction::Row => {
            let rows = nodes[i].rows.clone();
            for (s, e) in rows {
                let line = &kids[s..e];
                let line_h = line.iter().map(|&c| nodes[c].pref_h).fold(0.0, f32::max);
                for &c in line {
                    let n = &nodes[c];
                    let h = match n.el.height {
                        Size::Fixed(v) => v,
                        Size::Percent(p) => p * inner,
                        Size::Grow { .. } => {
                            let (lo, hi) = n.el.height.bounds();
                            line_h.clamp(lo, hi)
                        }
                        Size::Fit { .. } if el.cross == Cross::Stretch => line_h,
                        Size::Fit { .. } => n.pref_h,
                    };
                    nodes[c].h = h;
                }
            }
        }
    }
}

fn main_offsets(align: Align, free: f32, k: usize) -> (f32, f32) {
    match align {
        Align::Start => (0.0, 0.0),
        Align::Center => (free / 2.0, 0.0),
        Align::End => (free, 0.0),
        Align::SpaceBetween if k > 1 && free > 0.0 => (0.0, free / (k - 1) as f32),
        Align::SpaceBetween => (0.0, 0.0),
    }
}

fn cross_offset(cross: Cross, room: f32, size: f32) -> f32 {
    match cross {
        Cross::Start | Cross::Stretch | Cross::Baseline => 0.0,
        Cross::Center => (room - size) / 2.0,
        Cross::End => room - size,
    }
}

fn position(nodes: &mut [Node], i: usize, scroll: &dyn Fn(u64) -> (f32, f32)) {
    // Out of the flow: placed by its insets in the parent's box (an
    // anchored one is moved onto its target once every layer is placed).
    let bx = Rect::new(nodes[i].x, nodes[i].y, nodes[i].w, nodes[i].h);
    for c in nodes[i].out.clone() {
        let (w, h) = (nodes[c].w, nodes[c].h);
        let (x, y) = place_in(
            &bx,
            w,
            h,
            &nodes[c].props.insets,
            Align::Start,
            Cross::Start,
        );
        nodes[c].x = x;
        nodes[c].y = y;
    }
    if nodes[i].props.layout != LayoutMode::Flex && matches!(nodes[i].content, Content::None) {
        let n = &nodes[i];
        let (sx, sy) = if n.el.scroll || n.el.scroll_x {
            scroll(n.key)
        } else {
            (0.0, 0.0)
        };
        let inner = Rect::new(
            n.x + n.el.padding.left
                - if n.el.scroll_x {
                    sx.clamp(0.0, n.max_scroll_x())
                } else {
                    0.0
                },
            n.y + n.el.padding.top
                - if n.el.scroll {
                    sy.clamp(0.0, n.max_scroll())
                } else {
                    0.0
                },
            (n.w - n.el.padding.horizontal()).max(0.0),
            (n.h - n.el.padding.vertical()).max(0.0),
        );
        let (align, cross) = (n.el.align, n.el.cross);
        let kids = n.children.clone();
        if n.props.layout == LayoutMode::Stack {
            let mut extent = 0.0f32;
            for &c in &kids {
                let (w, h) = (nodes[c].w, nodes[c].h);
                let (x, y) = place_in(&inner, w, h, &nodes[c].props.insets, align, cross);
                nodes[c].x = x;
                nodes[c].y = y;
                extent = extent.max(w);
            }
            nodes[i].content_w = extent;
        } else if let Some(g) = nodes[i].grid.clone() {
            let offsets = |sizes: &[f32], gap: f32| {
                let mut at = Vec::with_capacity(sizes.len() + 1);
                let mut v = 0.0;
                for s in sizes {
                    at.push(v);
                    v += s + gap;
                }
                at.push(v - gap);
                at
            };
            let (cx, ry) = (offsets(&g.cols, g.col_gap), offsets(&g.rows, g.row_gap));
            for (k, &c) in kids.iter().enumerate() {
                let Some(&(col, row, cs, rs)) = g.cells.get(k) else {
                    continue;
                };
                let span = |at: &[f32], sizes: &[f32], s: usize, n: usize, gap: f32| {
                    let s = s.min(sizes.len());
                    let e = (s + n).min(sizes.len());
                    let len: f32 =
                        sizes[s..e].iter().sum::<f32>() + gap * (e - s).saturating_sub(1) as f32;
                    (at.get(s).copied().unwrap_or(0.0), len)
                };
                let (x0, cw) = span(&cx, &g.cols, col, cs, g.col_gap);
                let (y0, rh) = span(&ry, &g.rows, row, rs, g.row_gap);
                let cell = Rect::new(inner.x + x0, inner.y + y0, cw, rh);
                let (w, h) = (nodes[c].w, nodes[c].h);
                let (x, y) = place_in(&cell, w, h, &nodes[c].props.insets, align, cross);
                nodes[c].x = x;
                nodes[c].y = y;
            }
            nodes[i].content_w = cx.last().copied().unwrap_or(0.0);
        }
        return;
    }
    // The content's extent along x, before the offsets (the horizontal range).
    let content_w = {
        let n = &nodes[i];
        let kids = &n.children;
        match n.el.direction {
            Direction::Column => kids.iter().map(|&c| nodes[c].w).fold(0.0, f32::max),
            Direction::Row if n.el.wrap && !n.rows.is_empty() => n
                .rows
                .iter()
                .map(|&(s, e)| {
                    kids[s..e].iter().map(|&c| nodes[c].w).sum::<f32>() + gaps(n.el.gap, e - s)
                })
                .fold(0.0, f32::max),
            Direction::Row => {
                kids.iter().map(|&c| nodes[c].w).sum::<f32>() + gaps(n.el.gap, kids.len())
            }
        }
    };
    nodes[i].content_w = content_w;
    let n = &nodes[i];
    let el = n.el;
    let inner_w = n.w - el.padding.horizontal();
    let inner_h = n.h - el.padding.vertical();
    let mut x0 = n.x + el.padding.left;
    let mut y0 = n.y + el.padding.top;
    let (sx, sy) = if el.scroll || el.scroll_x {
        scroll(n.key)
    } else {
        (0.0, 0.0)
    };
    if el.scroll {
        y0 -= sy.clamp(0.0, n.max_scroll());
    }
    if el.scroll_x {
        x0 -= sx.clamp(0.0, n.max_scroll_x());
    }
    let kids = n.children.clone();
    let rows = if el.direction == Direction::Row && el.wrap && !n.rows.is_empty() {
        n.rows.clone()
    } else {
        vec![(0, kids.len())]
    };
    let mut line_y = 0.0;
    for (s, e) in rows {
        let line = &kids[s..e];
        let k = line.len();
        match el.direction {
            Direction::Row => {
                let used: f32 = line.iter().map(|&c| nodes[c].w).sum::<f32>() + gaps(el.gap, k);
                let mut free = inner_w - used;
                if el.scroll_x {
                    free = free.max(0.0);
                }
                let (start, between) = main_offsets(el.align, free, k);
                let room = if el.wrap {
                    line.iter().map(|&c| nodes[c].h).fold(0.0, f32::max)
                } else {
                    inner_h
                };
                let mut x = x0 + start;
                for &c in line {
                    let (w, h) = (nodes[c].w, nodes[c].h);
                    nodes[c].x = x;
                    nodes[c].y = y0 + line_y + cross_offset(el.cross, room, h);
                    x += w + el.gap + between;
                }
                line_y += room + el.gap;
            }
            Direction::Column => {
                let used: f32 = line.iter().map(|&c| nodes[c].h).sum::<f32>() + gaps(el.gap, k);
                let mut free = inner_h - used;
                if el.scroll {
                    free = free.max(0.0);
                }
                let (start, between) = main_offsets(el.align, free, k);
                let mut y = y0 + start;
                for &c in line {
                    let (w, h) = (nodes[c].w, nodes[c].h);
                    nodes[c].y = y;
                    nodes[c].x = x0
                        + cross_offset(el.cross, inner_w.max(if el.scroll_x { w } else { 0.0 }), w);
                    y += h + el.gap + between;
                }
            }
        }
    }
}

/// Where a `w`×`h` box sits in `bx`: by its insets where they are set,
/// else by `align` across and `cross` down.
fn place_in(bx: &Rect, w: f32, h: f32, ins: &Edges, align: Align, cross: Cross) -> (f32, f32) {
    let x = if ins.center_x {
        bx.x + (bx.width - w) / 2.0 + ins.start.unwrap_or(0.0)
    } else if let Some(s) = ins.start {
        bx.x + s
    } else if let Some(e) = ins.end {
        bx.right() - e - w
    } else {
        match align {
            Align::Start | Align::SpaceBetween => bx.x,
            Align::Center => bx.x + (bx.width - w) / 2.0,
            Align::End => bx.right() - w,
        }
    };
    let y = if ins.center_y {
        bx.y + (bx.height - h) / 2.0 + ins.top.unwrap_or(0.0)
    } else if let Some(t) = ins.top {
        bx.y + t
    } else if let Some(b) = ins.bottom {
        bx.bottom() - b - h
    } else {
        match cross {
            Cross::Start | Cross::Stretch | Cross::Baseline => bx.y,
            Cross::Center => bx.y + (bx.height - h) / 2.0,
            Cross::End => bx.bottom() - h,
        }
    };
    (x, y)
}

/// An out-of-flow element's size along one axis of its parent's box
/// (`outer`), between insets `a` and `b`.
fn out_size(size: Size, pref: f32, outer: f32, a: Option<f32>, b: Option<f32>) -> f32 {
    match (a, b, size) {
        (_, _, Size::Fixed(v)) => v,
        (Some(a), Some(b), _) => (outer - a - b).max(0.0),
        (_, _, Size::Percent(p)) => p * outer,
        (_, _, Size::Grow { .. }) => {
            let (lo, hi) = size.bounds();
            (outer - a.unwrap_or(0.0) - b.unwrap_or(0.0)).clamp(lo, hi)
        }
        (_, _, Size::Fit { .. }) => {
            let (lo, hi) = size.bounds();
            pref.clamp(lo, hi)
        }
    }
}

/// `w` is in the query's width range.
pub(crate) fn width_in(w: f32, q: &Query) -> bool {
    (q.min_width <= 0.0 || w >= q.min_width) && (q.max_width <= 0.0 || w <= q.max_width)
}

/// Applies a variant's patch to a node.
pub(crate) fn apply_patch(n: &mut Node, p: &Patch) {
    if let Some(v) = p.direction {
        n.el.direction = v;
    }
    if let Some(v) = p.width {
        n.el.width = v;
    }
    if let Some(v) = p.height {
        n.el.height = v;
    }
    if let Some(v) = p.padding {
        n.el.padding = v;
    }
    if let Some(v) = p.gap {
        n.el.gap = v;
    }
    if let Some(v) = p.align {
        n.el.align = v;
    }
    if let Some(v) = p.cross {
        n.el.cross = v;
    }
    if let Some(v) = p.wrap {
        n.el.wrap = v;
    }
    if let Some(v) = p.layout {
        n.props.layout = v;
    }
    if let Some(v) = &p.columns {
        n.props.grid.columns = v.clone();
    }
    if let Some(v) = p.hidden {
        n.hidden = v;
    }
    if !p.values.is_empty() {
        let mut row = read_values(n);
        for &(prop, v) in &p.values {
            for &c in prop.channels() {
                row[c] = v;
            }
        }
        write_values(n, &row);
    }
}

/// A node's animatable values, one per channel (`Prop::channels`).
pub(crate) fn read_values(n: &Node) -> [f32; crate::element::CHANNELS] {
    let t = &n.props.transform;
    let bg = n.el.background.unwrap_or(Srgba {
        rgb: [0, 0, 0],
        alpha: 0.0,
    });
    let ink = match &n.content {
        Content::Text(tc) => tc.color,
        _ => Srgba::WHITE,
    };
    let c = |v: u8| f32::from(v) / 255.0;
    [
        t.x,
        t.y,
        t.scale_x,
        t.scale_y,
        t.rotate,
        t.skew_x,
        t.skew_y,
        n.props.opacity,
        n.el.radius,
        c(bg.rgb[0]),
        c(bg.rgb[1]),
        c(bg.rgb[2]),
        bg.alpha,
        c(ink.rgb[0]),
        c(ink.rgb[1]),
        c(ink.rgb[2]),
        ink.alpha,
        n.props.z as f32,
        f32::from(u8::from(n.props.inert)),
    ]
}

/// Writes a row of channel values back into a node.
pub(crate) fn write_values(n: &mut Node, v: &[f32; crate::element::CHANNELS]) {
    let t = &mut n.props.transform;
    t.x = v[0];
    t.y = v[1];
    t.scale_x = v[2];
    t.scale_y = v[3];
    t.rotate = v[4];
    t.skew_x = v[5];
    t.skew_y = v[6];
    n.props.opacity = v[7].clamp(0.0, 1.0);
    n.el.radius = v[8].max(0.0);
    let u = |x: f32| (x.clamp(0.0, 1.0) * 255.0).round() as u8;
    let bg = Srgba {
        rgb: [u(v[9]), u(v[10]), u(v[11])],
        alpha: v[12].clamp(0.0, 1.0),
    };
    if n.el.background.is_some() || bg.alpha > 0.0 {
        n.el.background = Some(bg);
    }
    if let Content::Text(tc) = &mut n.content {
        tc.color = Srgba {
            rgb: [u(v[13]), u(v[14]), u(v[15])],
            alpha: v[16].clamp(0.0, 1.0),
        };
    }
    n.props.z = v[17].round() as i32;
    n.props.inert = v[18] >= 0.5;
}

/// A grid's resolved tracks and where each child in the flow sits.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct GridLayout {
    /// Per child in the flow: its column, row, column span, row span (0-based).
    pub cells: Vec<(usize, usize, usize, usize)>,
    pub ncols: usize,
    pub nrows: usize,
    pub cols: Vec<f32>,
    pub rows: Vec<f32>,
    pub col_gap: f32,
    pub row_gap: f32,
}

/// Places a grid's children in its cells: named areas and explicit lines
/// first, then the rest in flow order (`dense` back-fills holes).
fn place_grid(nodes: &mut [Node], i: usize) {
    let n = &nodes[i];
    let spec = &n.props.grid;
    let mut areas: Vec<(&str, usize, usize, usize, usize)> = Vec::new();
    for (r, line) in spec.areas.iter().enumerate() {
        for (c, name) in line.split_whitespace().enumerate() {
            if name == "." {
                continue;
            }
            match areas.iter_mut().find(|a| a.0 == name) {
                Some(a) => {
                    a.1 = a.1.min(c);
                    a.2 = a.2.min(r);
                    a.3 = a.3.max(c + 1);
                    a.4 = a.4.max(r + 1);
                }
                None => areas.push((name, c, r, c + 1, r + 1)),
            }
        }
    }
    let row_flow = spec.flow == GridFlow::Row;
    let area_cols = spec
        .areas
        .iter()
        .map(|l| l.split_whitespace().count())
        .max()
        .unwrap_or(0);
    let mut ncols = spec.columns.len().max(area_cols);
    let mut nrows = spec.rows.len().max(spec.areas.len());
    let kids = n.children.clone();
    let mut cells = vec![(0usize, 0usize, 1usize, 1usize); kids.len()];
    // Occupied cells, [row][col].
    let mut occ: Vec<Vec<bool>> = Vec::new();
    let mark = |occ: &mut Vec<Vec<bool>>, c: usize, r: usize, cs: usize, rs: usize| {
        for rr in r..r + rs {
            if occ.len() <= rr {
                occ.resize(rr + 1, Vec::new());
            }
            let row = &mut occ[rr];
            if row.len() < c + cs {
                row.resize(c + cs, false);
            }
            for cell in &mut row[c..c + cs] {
                *cell = true;
            }
        }
    };
    let free = |occ: &Vec<Vec<bool>>, c: usize, r: usize, cs: usize, rs: usize| {
        (r..r + rs).all(|rr| {
            (c..c + cs).all(|cc| {
                !occ.get(rr)
                    .and_then(|row| row.get(cc))
                    .copied()
                    .unwrap_or(false)
            })
        })
    };
    let mut auto = Vec::new();
    for (k, &c) in kids.iter().enumerate() {
        let p = &nodes[c].props.place;
        let (cs, rs) = (p.column_span.max(1) as usize, p.row_span.max(1) as usize);
        if !p.area.is_empty()
            && let Some(a) = areas.iter().find(|a| a.0 == p.area)
        {
            cells[k] = (a.1, a.2, a.3 - a.1, a.4 - a.2);
            mark(&mut occ, a.1, a.2, a.3 - a.1, a.4 - a.2);
            continue;
        }
        if p.column > 0 && p.row > 0 {
            let (col, row) = (p.column as usize - 1, p.row as usize - 1);
            cells[k] = (col, row, cs, rs);
            mark(&mut occ, col, row, cs, rs);
            ncols = ncols.max(col + cs);
            nrows = nrows.max(row + rs);
            continue;
        }
        auto.push(k);
    }
    // The fixed axis (columns in a row flow) and the cursor along it.
    let major = if row_flow { ncols.max(1) } else { nrows.max(1) };
    let (mut cur_line, mut cur_at) = (0usize, 0usize);
    for k in auto {
        let p = &nodes[kids[k]].props.place;
        let (cs, rs) = (p.column_span.max(1) as usize, p.row_span.max(1) as usize);
        let (along, across) = if row_flow { (cs, rs) } else { (rs, cs) };
        let along = along.min(major);
        let fixed = if row_flow { p.column } else { p.row } as usize;
        if spec.dense {
            cur_line = 0;
            cur_at = 0;
        }
        let (mut line, mut at) = (cur_line, cur_at);
        if fixed > 0 {
            at = (fixed - 1).min(major - along);
            if at < cur_at && !spec.dense {
                line += 1;
            }
        }
        loop {
            if at + along > major {
                at = if fixed > 0 {
                    (fixed - 1).min(major - along)
                } else {
                    0
                };
                line += 1;
                continue;
            }
            let (c, r, w, h) = if row_flow {
                (at, line, along, across)
            } else {
                (line, at, across, along)
            };
            if free(&occ, c, r, w, h) {
                cells[k] = (c, r, w, h);
                mark(&mut occ, c, r, w, h);
                cur_line = line;
                cur_at = at + along;
                break;
            }
            if fixed > 0 {
                line += 1;
            } else {
                at += 1;
            }
        }
    }
    for &(c, r, w, h) in &cells {
        ncols = ncols.max(c + w);
        nrows = nrows.max(r + h);
    }
    if row_flow {
        ncols = ncols.max(major);
    } else {
        nrows = nrows.max(major);
    }
    let gap = n.el.gap;
    let layout = GridLayout {
        cells,
        ncols,
        nrows,
        cols: Vec::new(),
        rows: Vec::new(),
        col_gap: spec.column_gap.unwrap_or(gap),
        row_gap: spec.row_gap.unwrap_or(gap),
    };
    nodes[i].grid = Some(layout);
}

/// Track sizes along one axis: `items` are (first track, span, preferred,
/// minimum); `avail` the inner size (`None`: intrinsic, every `fr` track
/// as large as its content asks).
fn track_sizes(
    defs: &[Track],
    auto_track: Track,
    n: usize,
    items: &[(usize, usize, f32, f32)],
    avail: Option<f32>,
    gap: f32,
) -> Vec<f32> {
    let def = |k: usize| defs.get(k).copied().unwrap_or(auto_track);
    let mut content = vec![0.0f32; n];
    let mut least = vec![0.0f32; n];
    for &(s, span, pref, min) in items {
        if span == 1 && s < n {
            content[s] = content[s].max(pref);
            least[s] = least[s].max(min);
        }
    }
    let mut sizes: Vec<f32> = (0..n)
        .map(|k| match def(k) {
            Track::Px(v) => v,
            Track::Percent(p) => avail.map_or(content[k], |a| p * a),
            Track::Auto => content[k],
            Track::MinMax { min, max } => content[k].clamp(min, max.max(min)),
            Track::Fr(_) => {
                if avail.is_some() {
                    least[k]
                } else {
                    content[k]
                }
            }
        })
        .collect();
    // A spanning item wider than its tracks grows their flexible ones.
    for &(s, span, pref, _) in items {
        if span <= 1 || s >= n {
            continue;
        }
        let e = (s + span).min(n);
        let have: f32 = sizes[s..e].iter().sum::<f32>() + gap * (e - s - 1) as f32;
        if pref <= have {
            continue;
        }
        let flex: Vec<usize> = (s..e)
            .filter(|&k| matches!(def(k), Track::Auto | Track::Fr(_) | Track::MinMax { .. }))
            .collect();
        let grow = if flex.is_empty() {
            (s..e).collect()
        } else {
            flex
        };
        let each = (pref - have) / grow.len() as f32;
        for k in grow {
            sizes[k] += each;
        }
    }
    let fr: f32 = (0..n)
        .map(|k| match def(k) {
            Track::Fr(f) => f.max(0.0),
            _ => 0.0,
        })
        .sum();
    if fr > 0.0 {
        match avail {
            Some(a) => {
                let fixed: f32 = (0..n)
                    .filter(|&k| !matches!(def(k), Track::Fr(_)))
                    .map(|k| sizes[k])
                    .sum();
                let free = (a - fixed - gap * n.saturating_sub(1) as f32).max(0.0);
                let unit = free / fr;
                for k in 0..n {
                    if let Track::Fr(f) = def(k) {
                        sizes[k] = (f.max(0.0) * unit).max(least[k]);
                    }
                }
            }
            None => {
                // Every fr track as wide as the largest content per unit asks.
                let unit = (0..n)
                    .filter_map(|k| match def(k) {
                        Track::Fr(f) if f > 0.0 => Some(sizes[k] / f),
                        _ => None,
                    })
                    .fold(0.0, f32::max);
                for (k, size) in sizes.iter_mut().enumerate() {
                    if let Track::Fr(f) = def(k) {
                        *size = f.max(0.0) * unit;
                    }
                }
            }
        }
    }
    sizes
}

fn grid_items(nodes: &[Node], i: usize, horizontal: bool) -> Vec<(usize, usize, f32, f32)> {
    let n = &nodes[i];
    let Some(g) = &n.grid else {
        return Vec::new();
    };
    n.children
        .iter()
        .zip(&g.cells)
        .map(|(&c, &(col, row, cs, rs))| {
            let k = &nodes[c];
            if horizontal {
                (col, cs, k.pref_w, k.min_w)
            } else {
                (row, rs, k.pref_h, k.min_h)
            }
        })
        .collect()
}

/// A grid's preferred and least size along one axis.
fn grid_intrinsic(nodes: &[Node], i: usize, horizontal: bool) -> (f32, f32) {
    let n = &nodes[i];
    let Some(g) = &n.grid else {
        return (0.0, 0.0);
    };
    let spec = &n.props.grid;
    let (defs, count, gap) = if horizontal {
        (&spec.columns, g.ncols, g.col_gap)
    } else {
        (&spec.rows, g.nrows, g.row_gap)
    };
    let items = grid_items(nodes, i, horizontal);
    let gaps = gap * count.saturating_sub(1) as f32;
    let pref: f32 = track_sizes(defs, spec.auto_track, count, &items, None, gap)
        .iter()
        .sum::<f32>()
        + gaps;
    let mins: Vec<(usize, usize, f32, f32)> =
        items.iter().map(|&(s, n, _, m)| (s, n, m, m)).collect();
    let min: f32 = track_sizes(defs, spec.auto_track, count, &mins, None, gap)
        .iter()
        .sum::<f32>()
        + gaps;
    (pref, min.min(pref))
}

/// Resolves a grid's columns at its inner width and sizes its children.
fn grid_columns(nodes: &mut [Node], i: usize, inner: f32) {
    let items = grid_items(nodes, i, true);
    let n = &nodes[i];
    let Some(g) = &n.grid else { return };
    let spec = &n.props.grid;
    let cols = track_sizes(
        &spec.columns,
        spec.auto_track,
        g.ncols,
        &items,
        Some(inner),
        g.col_gap,
    );
    let gap = g.col_gap;
    let kids = n.children.clone();
    let cells = g.cells.clone();
    for (&c, &(col, _, cs, _)) in kids.iter().zip(&cells) {
        let e = (col + cs).min(cols.len());
        let span: f32 = cols[col.min(e)..e].iter().sum::<f32>()
            + gap * (e.saturating_sub(col)).saturating_sub(1) as f32;
        let k = &nodes[c];
        nodes[c].w = match k.el.width {
            Size::Fixed(v) => v,
            Size::Percent(p) => p * span,
            Size::Fit { .. } | Size::Grow { .. } => {
                let (lo, hi) = k.el.width.bounds();
                span.clamp(lo, hi)
            }
        };
    }
    if let Some(g) = nodes[i].grid.as_mut() {
        g.cols = cols;
    }
}

/// Resolves a grid's rows at its inner height and sizes its children.
fn grid_rows(nodes: &mut [Node], i: usize, inner: f32) {
    let items = grid_items(nodes, i, false);
    let n = &nodes[i];
    let Some(g) = &n.grid else { return };
    let spec = &n.props.grid;
    let rows = track_sizes(
        &spec.rows,
        spec.auto_track,
        g.nrows,
        &items,
        Some(inner),
        g.row_gap,
    );
    let gap = g.row_gap;
    let kids = n.children.clone();
    let cells = g.cells.clone();
    for (&c, &(_, row, _, rs)) in kids.iter().zip(&cells) {
        let e = (row + rs).min(rows.len());
        let span: f32 = rows[row.min(e)..e].iter().sum::<f32>()
            + gap * (e.saturating_sub(row)).saturating_sub(1) as f32;
        let k = &nodes[c];
        nodes[c].h = match k.el.height {
            Size::Fixed(v) => v,
            Size::Percent(p) => p * span,
            Size::Fit { .. } | Size::Grow { .. } => {
                let (lo, hi) = k.el.height.bounds();
                span.clamp(lo, hi)
            }
        };
    }
    if let Some(g) = nodes[i].grid.as_mut() {
        g.rows = rows;
    }
}

/// Last frame's nodes and which of this frame's subtrees have the same
/// layout inputs (docs/ui.md §6.3): a clean subtree given the same size
/// keeps its layout (copied from last frame) instead of being laid out.
#[derive(Clone, Debug, Default)]
pub(crate) struct LayoutCache {
    /// Last frame's nodes, with their layout.
    pub prev: Vec<Node>,
    /// Per node of this frame: its twin in `prev` (NONE: new).
    pub twin: Vec<usize>,
    /// Per node: its subtree's layout inputs are unchanged.
    pub clean: Vec<bool>,
    /// Fonts, theme, direction or pixel ratio changed: nothing is clean.
    pub epoch: u64,
    pub prev_epoch: u64,
    /// Nodes laid out this frame (the rest kept their layout).
    pub laid_out: u32,
    /// The theme, direction and pixel ratio last frame drew with: a change
    /// makes nothing clean.
    pub sig: Option<(crate::Theme, og_i18n::Direction, f32)>,
    /// This frame keeps nothing (`epoch` or `sig` changed).
    pub fresh: bool,
    /// Each element's own draw commands last frame, by key, reused while
    /// everything it draws from is unchanged.
    pub cmds: og_core::IdMap<u64, Vec<crate::DrawCmd>>,
}

/// Everything an element's own commands are made from is as last frame
/// (its ancestors' fills are checked by the caller, up the tree).
pub(crate) fn draw_eq(a: &Node, b: &Node) -> bool {
    a.rect == b.rect
        && a.clip == b.clip
        && a.alpha == b.alpha
        && a.world == b.world
        && a.ring == b.ring
        && a.pad_rings == b.pad_rings
        && a.heading == b.heading
        && a.disabled == b.disabled
        && a.aux == b.aux
        && a.scale == b.scale
        && a.el == b.el
        && a.content == b.content
        && a.widget == b.widget
        && a.shapes == b.shapes
        && a.lines == b.lines
        && a.rich_lines == b.rich_lines
}

impl LayoutCache {
    /// Node `i`'s twin when its subtree is clean.
    fn clean_twin(&self, i: usize) -> Option<&Node> {
        if *self.clean.get(i)? {
            self.prev.get(*self.twin.get(i)?)
        } else {
            None
        }
    }
}

/// The box props layout reads are equal.
pub(crate) fn el_layout_eq(a: &Element, b: &Element) -> bool {
    a.direction == b.direction
        && a.width == b.width
        && a.height == b.height
        && a.padding == b.padding
        && a.gap == b.gap
        && a.align == b.align
        && a.cross == b.cross
        && a.wrap == b.wrap
        && a.scroll == b.scroll
        && a.scroll_x == b.scroll_x
        && a.aspect == b.aspect
}

/// The content layout measures is equal (an ink may differ).
pub(crate) fn content_layout_eq(a: &Content, b: &Content) -> bool {
    match (a, b) {
        (Content::Text(x), Content::Text(y)) => {
            x.text == y.text
                && x.step == y.step
                && x.font == y.font
                && x.wrap == y.wrap
                && x.ellipsis == y.ellipsis
                && x.max_lines == y.max_lines
        }
        (Content::Image { .. }, Content::Image { .. }) => true,
        _ => a == b,
    }
}

/// The element-model props layout reads are equal.
pub(crate) fn props_layout_eq(a: &crate::element::Props, b: &crate::element::Props) -> bool {
    a.position == b.position
        && a.insets == b.insets
        && a.layout == b.layout
        && a.grid == b.grid
        && a.place == b.place
        && !a.variants.iter().any(|v| v.when.container)
        && !b.variants.iter().any(|v| v.when.container)
}

/// Lays out the tree under `root` inside `area`. `scroll(key)` is a scroll
/// container's offset; `dpr` snaps the final rectangles.
#[allow(clippy::too_many_arguments)]
pub(crate) fn layout(
    nodes: &mut [Node],
    root: usize,
    area: Rect,
    measure: &mut Measure<'_>,
    scroll: &dyn Fn(u64) -> (f32, f32),
    dpr: f32,
    clip: Rect,
    kept: &mut LayoutCache,
) {
    let order = preorder(nodes, root);
    let twin_of = |kept: &LayoutCache, i: usize| -> Option<usize> {
        kept.clean_twin(i)?;
        kept.twin.get(i).copied()
    };
    for &i in order.iter().rev() {
        if let Some(t) = twin_of(kept, i) {
            let p = &kept.prev[t];
            nodes[i].pref_w = p.pref_w;
            nodes[i].min_w = p.min_w;
            nodes[i].grid = p.grid.clone();
            if let Content::Text(t) = &nodes[i].content {
                measure.texts.touch(t);
            }
            continue;
        }
        size_w(nodes, i, measure);
    }
    nodes[root].w = match nodes[root].el.width {
        Size::Fixed(v) => v,
        Size::Fit { .. } => nodes[root].pref_w.min(area.width).max(nodes[root].min_w),
        Size::Percent(p) => p * area.width,
        Size::Grow { .. } => {
            let (lo, hi) = nodes[root].el.width.bounds();
            area.width.clamp(lo, hi)
        }
    };
    // A clean node at the width it had keeps its children's widths, its
    // lines and its tracks.
    let same_w = |nodes: &[Node], kept: &LayoutCache, i: usize| {
        twin_of(kept, i).filter(|&t| kept.prev[t].w == nodes[i].w)
    };
    let kids_of = |n: &Node| all_children(n);
    for &i in &order {
        if let Some(t) = same_w(nodes, kept, i) {
            let p = &kept.prev[t];
            nodes[i].lines = p.lines.clone();
            nodes[i].rich_lines = p.rich_lines.clone();
            nodes[i].rows = p.rows.clone();
            nodes[i].grid = p.grid.clone();
            for c in kids_of(&nodes[i]) {
                if let Some(&tc) = kept.twin.get(c) {
                    nodes[c].w = kept.prev[tc].w;
                }
            }
            continue;
        }
        kept.laid_out += 1;
        distribute_w(nodes, i, measure);
    }
    for &i in order.iter().rev() {
        if let Some(t) = same_w(nodes, kept, i) {
            let p = &kept.prev[t];
            nodes[i].pref_h = p.pref_h;
            nodes[i].min_h = p.min_h;
            nodes[i].content_h = p.content_h;
            continue;
        }
        size_h(nodes, i);
    }
    nodes[root].h = match nodes[root].el.height {
        Size::Fixed(v) => v,
        Size::Fit { .. } => nodes[root].pref_h,
        Size::Percent(p) => p * area.height,
        Size::Grow { .. } => {
            let (lo, hi) = nodes[root].el.height.bounds();
            area.height.clamp(lo, hi)
        }
    };
    let same_wh = |nodes: &[Node], kept: &LayoutCache, i: usize| {
        same_w(nodes, kept, i).filter(|&t| kept.prev[t].h == nodes[i].h)
    };
    for &i in &order {
        if let Some(t) = same_wh(nodes, kept, i) {
            if let (Some(g), Some(pg)) = (nodes[i].grid.as_mut(), kept.prev[t].grid.as_ref()) {
                g.rows = pg.rows.clone();
            }
            for c in kids_of(&nodes[i]) {
                if let Some(&tc) = kept.twin.get(c) {
                    nodes[c].h = kept.prev[tc].h;
                }
            }
            continue;
        }
        distribute_h(nodes, i);
    }
    nodes[root].x = area.x;
    nodes[root].y = area.y;
    for &i in &order {
        if !nodes[i].scrolls()
            && let Some(t) = same_wh(nodes, kept, i)
        {
            let p = &kept.prev[t];
            let (px, py) = (p.x, p.y);
            nodes[i].content_w = p.content_w;
            for c in kids_of(&nodes[i]) {
                if let Some(&tc) = kept.twin.get(c) {
                    let q = &kept.prev[tc];
                    nodes[c].x = nodes[i].x + (q.x - px);
                    nodes[c].y = nodes[i].y + (q.y - py);
                }
            }
            continue;
        }
        position(nodes, i, scroll);
    }
    nodes[root].clip = clip;
    for &i in &order {
        let n = &nodes[i];
        let rect = Rect::new(n.x, n.y, n.w, n.h);
        let (parent_clip, src) = if n.parent == NONE || i == root {
            (clip, NONE)
        } else {
            let p = &nodes[n.parent];
            if p.el.scroll || p.el.scroll_x {
                (p.clip.intersect(&p.rect), n.parent)
            } else {
                (p.clip, p.clip_src)
            }
        };
        nodes[i].clip = parent_clip;
        nodes[i].clip_src = src;
        nodes[i].rect = rect.snapped(dpr);
    }
}

#[cfg(test)]
mod text_cache {
    use super::{Content, Rect};
    use crate::{Fonts, InputFrame, LabelDesc, Size, Ui, UiDesc, UiFont, Viewport};

    /// One frame of three labels (a wrapping one at `width`, one cut with
    /// an ellipsis, one unwrapped), `count` in the last: each text's lines,
    /// their widths and its rectangle.
    fn frame(ui: &mut Ui, width: f32, count: u32) -> Vec<(Vec<(String, f32)>, Rect)> {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.label(
            "wrap",
            "the quick brown fox jumps over the lazy dog",
            LabelDesc {
                width: Size::Fixed(width),
                ..LabelDesc::default()
            },
        );
        ui.label(
            "cut",
            "a title too long for its slot",
            LabelDesc {
                width: Size::Fixed(width / 2.0),
                ellipsis: true,
                nowrap: true,
                ..LabelDesc::default()
            },
        );
        ui.label("count", &format!("score {count}"), LabelDesc::default());
        ui.end();
        ui.nodes
            .iter()
            .filter(|n| matches!(n.content, Content::Text(_)))
            .map(|n| {
                let lines = n.lines.iter().map(|l| (l.text.clone(), l.width));
                (lines.collect(), n.rect)
            })
            .collect()
    }

    fn fresh() -> Ui {
        Ui::new(UiDesc {
            fonts: Fonts {
                body: UiFont::unrendered(0.55),
                ..Fonts::default()
            },
            ..UiDesc::default()
        })
    }

    #[test]
    fn a_kept_measurement_lays_out_as_a_fresh_one() {
        // Frames whose widths and counts change and come back: each laid
        // out by one Ui that keeps its measurements equals a Ui laying it
        // out for the first time.
        let mut kept = fresh();
        for (width, count) in [(120.0, 1), (120.0, 1), (200.0, 1), (120.0, 2), (90.0, 3)] {
            let laid = frame(&mut kept, width, count);
            assert_eq!(laid, frame(&mut fresh(), width, count), "width {width}");
            // The wrapping text wraps and the cut one is cut.
            assert!(laid[0].0.len() > 1, "{laid:?}");
            assert!(laid[1].0[0].0.ends_with('…') || laid[1].0[0].0.ends_with("..."));
        }
    }

    #[test]
    fn a_frame_measures_only_the_texts_that_changed() {
        // Three labels: a wrapping one, a cut one and a counter. Each text
        // is measured for its widths and for its lines.
        let mut ui = fresh();
        frame(&mut ui, 120.0, 1);
        assert_eq!(ui.text_measurements(), 6, "the first frame measures all");
        frame(&mut ui, 120.0, 1);
        assert_eq!(
            ui.text_measurements(),
            0,
            "an unchanged frame measures none"
        );
        frame(&mut ui, 120.0, 2);
        assert_eq!(
            ui.text_measurements(),
            2,
            "the new count's widths and lines"
        );
        // A new width breaks the wrapping text and cuts the cut one again;
        // their widths and the counter (which breaks only at \n) are kept.
        frame(&mut ui, 200.0, 2);
        assert_eq!(ui.text_measurements(), 2);
        let fonts = ui.fonts().clone();
        ui.set_fonts(fonts);
        frame(&mut ui, 200.0, 2);
        assert_eq!(ui.text_measurements(), 6, "new fonts measure everything");
    }

    #[test]
    fn an_unchanged_text_is_measured_once_and_a_gone_one_is_dropped() {
        let mut ui = fresh();
        frame(&mut ui, 120.0, 1);
        // Each label's widths and lines.
        let first = ui.texts.len();
        assert_eq!(first, 6, "{:?}", ui.texts);
        frame(&mut ui, 120.0, 1);
        assert_eq!(ui.texts.len(), first, "the same frame keeps what it had");
        // A new count measures the new text; the old one goes at the sweep.
        frame(&mut ui, 120.0, 2);
        assert_eq!(ui.texts.len(), first);
        assert!(ui.texts.lines.contains_key("score 2"));
        assert!(!ui.texts.lines.contains_key("score 1"));
        // New fonts forget every measurement.
        let fonts = ui.fonts().clone();
        ui.set_fonts(fonts);
        assert_eq!(ui.texts.len(), 0);
    }
}
