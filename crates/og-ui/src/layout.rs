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

mod cache;
mod flex;
mod grid;
mod node;

pub(crate) use cache::{
    LayoutCache, Measure, Metrics, TextCache, content_layout_eq, draw_eq, el_layout_eq, layout,
    props_layout_eq,
};
use flex::*;
pub(crate) use grid::GridLayout;
use grid::{grid_columns, grid_intrinsic, grid_rows, place_grid};
pub use node::FontRole;
use node::truncate_lines;
pub(crate) use node::{
    Content, EditView, NONE, Node, Shape, Stagger, TextContent, Widget, ellipsis_of, step_tracking,
};

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
    /// An interactive container keys and pads reach (a card, a menu item,
    /// a tab of a component): it takes the focus ring and Enter, Space or
    /// the pad's A press it, as a button.
    pub focusable: bool,
    /// With focusable: Left and Right (a pad's left and right) are its
    /// own while it has the focus: they step it ([`crate::Response::adjust`])
    /// instead of moving the focus (a menu row opening its submenu, a
    /// carousel, a divider).
    pub adjustable: bool,
    pub background: Option<Srgba>,
    /// Corner radius in CSS pixels (clamped to half the shorter side).
    pub radius: f32,
    /// The frame its background takes (`None`: a plain fill). Without
    /// a style, or a style without that part, the background is the fill.
    pub frame: Option<crate::style_file::StylePart>,
    /// What it is to the style (docs/ui.md §12.3): the component, part,
    /// variant and state a dressing's selector matches, the element class
    /// whose corner and the level whose shadows it takes. With one, its
    /// background is drawn as the style dresses the part, or from the
    /// tokens where no dressing does; `frame` names a version 1 part.
    pub part: Option<crate::dress::PartRef>,
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
        /// In the tooltip layer, over every other (a card the player opened).
        tooltip: bool,
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
            Float::Anchored { tooltip: true, .. } => 5,
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
