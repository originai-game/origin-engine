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
    /// The skin frame its background takes (`None`: a plain fill). Without
    /// a skin, or a skin without that part, the background is the fill.
    pub skin: Option<crate::skin::SkinPart>,
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
    /// One line of text centred in the frame.
    Text {
        frame: Frame,
        text: String,
        step: TypeStep,
        color: Srgba,
        font: FontRole,
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
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextContent {
    pub text: String,
    pub step: TypeStep,
    pub color: Srgba,
    pub align: TextAlign,
    pub wrap: bool,
    pub font: FontRole,
}

impl TextContent {
    /// The type step's tracking, in em: Latin tracking, so none for text
    /// with CJK characters or a shaped script (whose joins it would break).
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

/// A step's tracking for `text`: none with CJK or a shaped script.
pub(crate) fn step_tracking(step: &TypeStep, text: &str) -> f32 {
    if text.chars().any(|c| is_cjk(c) || og_i18n::needs_shaping(c)) {
        0.0
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
        /// How far in the press is on the skin's spring (0 rest, 1 held).
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
    Field {
        focused: bool,
        caret: Option<usize>,
    },
    Tooltip,
    /// A virtualised list or grid's filler: no drawing.
    Spacer,
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
    /// Drawn after the background.
    pub shapes: Vec<Shape>,
    /// Widget data its children read (a list's row height, a grid's cell size).
    pub aux: [f32; 3],
    /// A heading label: the skin's flourish frames it.
    pub heading: bool,
    /// Its descendants enter one after another (a skin's stagger).
    pub stagger: Option<Stagger>,
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
            shapes: Vec::new(),
            aux: [0.0; 3],
            heading: false,
            stagger: None,
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

pub(crate) fn preorder(nodes: &[Node], root: usize) -> Vec<usize> {
    let mut order = Vec::new();
    let mut stack = vec![root];
    while let Some(i) = stack.pop() {
        order.push(i);
        stack.extend(nodes[i].children.iter().rev());
    }
    order
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
    points.sort_by(f32::total_cmp);
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

fn size_w(nodes: &mut [Node], i: usize, fonts: &dyn Metrics) {
    let n = &nodes[i];
    let (pref, min) = match &n.content {
        Content::Text(t) => {
            let m = fonts.metrics_for(t.font, &t.text);
            let pref = widest_line(&t.text, m, t.step.size, t.tracking());
            let min = if t.wrap {
                longest_word(&t.text, m, t.step.size, t.tracking())
            } else {
                pref
            };
            (pref, min)
        }
        Content::Rich(rc) => crate::rich::rich_widths(rc, fonts),
        Content::Image { .. } => (0.0, 0.0),
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

fn distribute_w(nodes: &mut [Node], i: usize, fonts: &dyn Metrics) {
    let inner = (nodes[i].w - nodes[i].el.padding.horizontal()).max(0.0);
    if let Content::Text(t) = &nodes[i].content {
        let m = fonts.metrics_for(t.font, &t.text);
        // A fit width snapped to the device pixel may land up to half a
        // pixel under the text it was measured from: that is no reason to wrap.
        let lines = break_lines(
            &t.text,
            m,
            t.step.size,
            t.tracking(),
            t.wrap.then_some(inner + 0.5),
        );
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
    let scroll = n.el.scroll;
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
    let kids = nodes[i].children.clone();
    if kids.is_empty() {
        return;
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
        Cross::Start | Cross::Stretch => 0.0,
        Cross::Center => (room - size) / 2.0,
        Cross::End => room - size,
    }
}

fn position(nodes: &mut [Node], i: usize, scroll: &dyn Fn(u64) -> (f32, f32)) {
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

/// Lays out the tree under `root` inside `area`. `scroll(key)` is a scroll
/// container's offset; `dpr` snaps the final rectangles.
pub(crate) fn layout(
    nodes: &mut [Node],
    root: usize,
    area: Rect,
    fonts: &dyn Metrics,
    scroll: &dyn Fn(u64) -> (f32, f32),
    dpr: f32,
    clip: Rect,
) {
    let order = preorder(nodes, root);
    for &i in order.iter().rev() {
        size_w(nodes, i, fonts);
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
    for &i in &order {
        distribute_w(nodes, i, fonts);
    }
    for &i in order.iter().rev() {
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
    for &i in &order {
        distribute_h(nodes, i);
    }
    nodes[root].x = area.x;
    nodes[root].y = area.y;
    for &i in &order {
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
