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

pub mod anchor;
mod cache;
mod flex;
mod grid;
mod node;

pub(crate) use cache::{
    DrawSig, LayoutCache, Measure, Metrics, TextCache, content_layout_eq, draw_eq, el_layout_eq,
    layout, props_layout_eq,
};
pub(crate) use flex::on_screen;
use flex::*;
pub(crate) use grid::{GridLayout, Table, derive_table, shown_children};
use grid::{grid_columns, grid_intrinsic, grid_rows, place_grid};
use node::truncate_lines;
pub(crate) use node::{
    Content, EditView, NONE, Node, Shape, Stagger, TextContent, Widget, WrapStyle, ellipsis_of,
    snap_pixel_text, step_tracking, text_features,
};
pub use node::{Family, FontRole};

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

    /// Edges rounded to the device pixel grid ([`snap`]).
    pub fn snapped(&self, dpr: f32) -> Rect {
        let s = |v: f32| snap(v, dpr);
        let (x, y) = (s(self.x), s(self.y));
        Rect::new(x, y, s(self.right()) - x, s(self.bottom()) - y)
    }
}

/// `v` CSS px on the device pixel grid of `dpr`, held to a multiple of
/// 1/256 px (less than 1/512 px off it, unseen): at a ratio like 1.5 a
/// device pixel is no exact `f32`, and a grid of exact values keeps every
/// sum of edges and moves exact, so a scrolled row's commands moved by the
/// scroll are the ones painting it there gives, and overlap what they did.
pub(crate) fn snap(v: f32, dpr: f32) -> f32 {
    ((v * dpr).round() / dpr * 256.0).round() / 256.0
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
    /// CSS `calc()`, `min()`, `max()` or `clamp()` over lengths in more than
    /// one unit (`calc(100% - 12px)`), resolved against the size a
    /// percentage reads.
    Calc(Calc),
    /// CSS `min-content`: its least size (a wrapping text's longest word).
    MinContent,
    /// CSS `max-content`: its preferred size (a text unwrapped).
    MaxContent,
}

/// A CSS length as a linear sum: a fraction of a basis plus CSS px.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Length {
    /// A fraction (0–1) of the basis.
    pub pct: f32,
    pub px: f32,
}

impl Length {
    pub const fn px(v: f32) -> Length {
        Length { pct: 0.0, px: v }
    }

    /// Its CSS px against `basis`, the size a percentage reads.
    pub fn of(self, basis: f32) -> f32 {
        self.pct * basis + self.px
    }
}

/// How a [`Calc`] combines its lengths.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CalcOp {
    /// The first length (`calc()`).
    #[default]
    Sum,
    Min,
    Max,
    /// The second length between the first and the third.
    Clamp,
    /// `round()`: the first length to the nearest multiple of the second
    /// (a tie upwards), up, down or towards zero (CSS Values 4 §10.3).
    Round,
    RoundUp,
    RoundDown,
    RoundToZero,
    /// `mod()` (the second length's sign) and `rem()` (the first's).
    Mod,
    Rem,
    Abs,
    /// The square root of the sum of the squares.
    Hypot,
}

/// Up to three lengths a CSS math function combines at layout.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Calc {
    pub op: CalcOp,
    pub args: [Length; 3],
    /// How many of `args` it reads.
    pub count: u8,
}

impl Calc {
    /// Its CSS px against `basis`.
    pub fn of(&self, basis: f32) -> f32 {
        let v = self.args[..self.count.max(1) as usize]
            .iter()
            .map(|l| l.of(basis));
        match self.op {
            CalcOp::Sum => self.args[0].of(basis),
            CalcOp::Min => v.fold(f32::INFINITY, f32::min),
            CalcOp::Max => v.fold(f32::NEG_INFINITY, f32::max),
            CalcOp::Clamp => {
                let [lo, v, hi] = self.args.map(|l| l.of(basis));
                v.min(hi).max(lo)
            }
            CalcOp::Round | CalcOp::RoundUp | CalcOp::RoundDown | CalcOp::RoundToZero => {
                let [a, b, _] = self.args.map(|l| l.of(basis));
                let b = if self.count < 2 { 1.0 } else { b.abs() };
                if b == 0.0 {
                    return 0.0;
                }
                let q = a / b;
                b * match self.op {
                    CalcOp::RoundUp => q.ceil(),
                    CalcOp::RoundDown => q.floor(),
                    CalcOp::RoundToZero => q.trunc(),
                    _ => (q + 0.5).floor(),
                }
            }
            CalcOp::Mod | CalcOp::Rem => {
                let [a, b, _] = self.args.map(|l| l.of(basis));
                if b == 0.0 {
                    return 0.0;
                }
                let q = a / b;
                a - b * if self.op == CalcOp::Mod {
                    q.floor()
                } else {
                    q.trunc()
                }
            }
            CalcOp::Abs => self.args[0].of(basis).abs(),
            CalcOp::Hypot => v.map(|x| x * x).sum::<f32>().sqrt(),
        }
    }

    /// Its px with no basis (a percentage of an unknown size is 0, as CSS
    /// reads one for an intrinsic size).
    pub fn intrinsic(&self) -> f32 {
        let mut c = *self;
        for a in &mut c.args {
            a.pct = 0.0;
        }
        c.of(0.0).max(0.0)
    }
}

/// One margin edge: a length, or `auto` (the free space).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Margin {
    pub len: Length,
    pub auto: bool,
}

impl Margin {
    pub const fn px(v: f32) -> Margin {
        Margin {
            len: Length::px(v),
            auto: false,
        }
    }

    /// Its CSS px against `basis` (its parent's content width); `auto` is 0.
    pub fn of(self, basis: f32) -> f32 {
        if self.auto { 0.0 } else { self.len.of(basis) }
    }

    /// Its CSS px for an intrinsic size: a percentage and `auto` are 0.
    pub fn intrinsic(self) -> f32 {
        if self.auto { 0.0 } else { self.len.px }
    }
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
            Size::Percent(_) | Size::Calc(_) | Size::MinContent | Size::MaxContent => {
                (0.0, f32::INFINITY)
            }
        }
    }

    /// Its CSS px when it names one: a length, a percentage or calc() of
    /// `basis` (`None`: an indefinite basis, where a percentage behaves as
    /// `auto`, CSS Sizing 3 §5.2.1), or a content size from `pref` and `min`.
    pub(crate) fn definite(&self, pref: f32, min: f32, basis: Option<f32>) -> Option<f32> {
        match *self {
            Size::Fixed(v) => Some(v),
            Size::Percent(p) => basis.map(|b| p * b),
            Size::Calc(c) => basis.map(|b| c.of(b)),
            Size::MinContent => Some(min),
            Size::MaxContent => Some(pref),
            Size::Fit { .. } | Size::Grow { .. } => None,
        }
    }
}

impl Element {
    /// `v` held between its CSS min- and max- bounds along `axis` (0:
    /// width, 1: height), resolved against `basis` (`None`: a percentage
    /// bound does not apply). A min- bound wins over a max- one (CSS 2
    /// §10.4).
    pub(crate) fn bounded(
        &self,
        axis: usize,
        basis: Option<f32>,
        pref: f32,
        min: f32,
        v: f32,
    ) -> f32 {
        let [lo, hi] = self.bounds[axis];
        let at = |b: Option<Size>| b.and_then(|b| b.definite(pref, min, basis));
        let mut v = v;
        if let Some(h) = at(hi) {
            v = v.min(h);
        }
        if let Some(l) = at(lo) {
            v = v.max(l);
        }
        v
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
    /// Half a share of the free space at each end, a share between.
    SpaceAround,
    /// The free space split evenly before, between and after children.
    SpaceEvenly,
    /// CSS `stretch`: in a flex line as `Start`; a grid's auto tracks grow
    /// into the free space, and lines of a multi-line container share it.
    Stretch,
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
    /// A row's children share their last text baseline.
    LastBaseline,
}

/// A CSS flex item's factors (CSS Flexbox §7): it takes part in the line's
/// free space as the flex algorithm (§9.7) resolves it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlexItem {
    pub grow: f32,
    pub shrink: f32,
    /// Its flex basis (`None`: `auto`, its width or height, else its content).
    pub basis: Option<Size>,
}

impl Default for FlexItem {
    fn default() -> FlexItem {
        FlexItem {
            grow: 0.0,
            shrink: 1.0,
            basis: None,
        }
    }
}

/// Horizontal alignment of text lines inside their element.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextAlign {
    #[default]
    Start,
    Center,
    End,
    /// CSS `text-align: justify`: every line but the last (and a line a
    /// forced break ends) stretched to the box at its spaces, from start.
    Justify,
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
    /// A scroll container that draws no scrollbars (a row of chips or tabs
    /// that shows its overflow by its cut edge, where a resting thumb read
    /// as a selection underline).
    pub hide_bars: bool,
    /// CSS `scroll-snap-type` of a scroll container: bit 0 x, bit 1 y,
    /// bit 2 proximity (else mandatory); 0 does not snap.
    pub snap: u8,
    /// CSS `scroll-snap-align` of a snap area: x in bits 0–1, y in bits
    /// 2–3, each 0 none, 1 start, 2 center, 3 end.
    pub snap_align: u8,
    /// Its CSS overscroll, scroll behaviour, scroll margin and padding,
    /// snap stop and scrollbar properties.
    pub scroll_css: crate::scroll::ScrollCss,
    /// A scroll container's edge fade (CSS px, 0: none) where more content
    /// lies beyond, into `fade_into` (`None`: `panel`).
    pub fade: f32,
    pub fade_into: Option<Srgba>,
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
    /// With focusable: the arrows it claims while it has the focus, bits of
    /// [`crate::ARROW_LEFT`], [`crate::ARROW_RIGHT`], [`crate::ARROW_UP`]
    /// and [`crate::ARROW_DOWN`]: each steps it ([`crate::Response::adjust`]
    /// for Left and Right, [`crate::Response::adjust_y`] for Up and Down)
    /// instead of moving the focus. A grid's own cursor claims the arrows
    /// that keep it inside, so the focus leaves it at its edges.
    pub arrows: u8,
    /// With focusable: the component draws its own focus (a hand rings its
    /// focused card, not the whole fan); og-ui draws no ring around it.
    pub own_ring: bool,
    pub background: Option<Srgba>,
    /// Corner radius in CSS pixels (clamped to half the shorter side).
    pub radius: f32,
    /// The frame its background takes (`None`: a plain fill). Without
    /// a style, or a style without that part, the background is the fill.
    pub frame: Option<crate::style_file::StylePart>,
    /// What it is to the style (docs/ui.md §3.2): the component, part,
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
    /// Its margins in CSS order (top, right, bottom, left): room outside
    /// its box. A percentage reads its parent's content width, as CSS's do;
    /// an `auto` one takes the free space a block's width or a flex line
    /// leaves (docs/ui.md §2, HTML blocks). A stack places its children
    /// without them.
    pub margin: [Margin; 4],
    /// A CSS block box (`display: block`): its in-flow children's vertical
    /// margins collapse with each other and, where no padding separates
    /// them, with its own (CSS 2 §8.3.1).
    pub block: bool,
    /// CSS `row-reverse`/`column-reverse`: its children run from the main
    /// axis's end.
    pub reverse: bool,
    /// CSS `wrap-reverse`: its lines stack from the cross axis's end.
    pub wrap_reverse: bool,
    /// Its `align` was written `start`, `end`, `left` or `right` (the writing
    /// mode's sides), not `flex-start`/`flex-end`: a reversed line keeps it
    /// on that side.
    pub align_logical: bool,
    /// CSS `align-content` of a multi-line flex container (`None`: lines at
    /// its start, each its own size).
    pub align_content: Option<Align>,
    /// CSS `justify-items` (a grid's items across their areas; `None`: by
    /// `align`), and an item's `align-self` and `justify-self` over its
    /// parent's.
    pub justify_items: Option<Cross>,
    pub align_self: Option<Cross>,
    pub justify_self: Option<Cross>,
    /// CSS `column-gap` and `row-gap` over `gap`.
    pub gaps: [Option<f32>; 2],
    /// Its CSS flex factors in a flex line (`None`: its `width`/`height`
    /// kind sizes it).
    pub flex: Option<FlexItem>,
    /// CSS `order`: flex and grid items lay out by it, then as built.
    pub order: i32,
    /// CSS `safe` alignment: bit 0 its `justify-content`, bit 1 its
    /// `align-items` (an item's `align-self`): overflow stays on the start
    /// side.
    pub safe: u8,
    /// Its CSS `min-`/`max-` bounds beyond those a `Fit` or `Grow` size
    /// holds, per axis (width, height), as (min, max): a length, a
    /// percentage or calc() of the size its parent offers, or a content
    /// size (`MinContent`, `MaxContent`). `None`: unbounded.
    pub bounds: [[Option<Size>; 2]; 2],
    /// CSS outer display `inline` (`inline-block`, `inline-flex`…): in a
    /// block that holds one, it and the texts beside it lie in lines.
    pub inline: bool,
    /// It establishes a block formatting context (`display: flow-root`):
    /// its children's margins do not collapse through its edges.
    pub bfc: bool,
    /// CSS `display: contents`: it generates no box; its children lay out
    /// in its parent's as its own would.
    pub contents: bool,
    /// Its part of a CSS table (`display: table`, `table-row`…).
    pub table: TablePart,
    /// CSS `text-align` of the lines it lays its inline-level children in.
    pub line_align: Align,
}

/// What a box is to the CSS table layout (CSS 2 §17.2).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TablePart {
    #[default]
    None,
    /// `display: table` (or `inline-table`): it lays its rows out.
    Table,
    /// `table-header-group`, `table-row-group`, `table-footer-group`.
    Header,
    Body,
    Footer,
    Row,
    Cell,
    /// `table-caption`: above the table (`below`: under it).
    Caption {
        below: bool,
    },
    /// `table-column` and `table-column-group`: no box; a column's width
    /// is the least its column takes.
    Column,
    ColumnGroup,
}

impl Element {
    /// The gap between items along its main axis.
    pub(crate) fn main_gap(&self) -> f32 {
        let k = usize::from(self.direction == Direction::Column);
        self.gaps[k].unwrap_or(self.gap)
    }

    /// The gap between its lines (a wrapping row's rows, a wrapping
    /// column's columns).
    pub(crate) fn line_gap(&self) -> f32 {
        let k = usize::from(self.direction == Direction::Row);
        self.gaps[k].unwrap_or(self.gap)
    }

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
        /// Steps clear of the other anchored groups (a popup at a point).
        avoid: bool,
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
    /// Fit a centred square inside the resolved rectangle, capped in UI units.
    pub square_max: Option<f32>,
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
        square_max: None,
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
        let rect = Rect::new(
            r.x + self.fx * r.width + self.x,
            r.y + self.fy * r.height + self.y,
            self.fw * r.width + self.w,
            self.fh * r.height + self.h,
        );
        if let Some(max) = self.square_max {
            let side = rect.width.min(rect.height).min(max);
            Rect::new(
                rect.x + (rect.width - side) / 2.0,
                rect.y + (rect.height - side) / 2.0,
                side,
                side,
            )
        } else {
            rect
        }
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

/// A node's children in and out of the flow, in sibling order.
pub(crate) fn all_children(n: &Node) -> Vec<usize> {
    n.ordered_children.clone()
}

/// The preorder with each node's children in drawing order: by `z`, then
/// as built (a lifted card draws over its neighbours and is hit first).
pub(crate) fn draw_order(nodes: &[Node], root: usize) -> Vec<usize> {
    let mut order = Vec::new();
    let mut stack = vec![root];
    while let Some(i) = stack.pop() {
        order.push(i);
        let mut kids = all_children(&nodes[i]);
        if kids.iter().any(|&c| nodes[c].sampled.z != 0) {
            og_core::sort::sort_by_key(&mut kids, |&c| nodes[c].sampled.z);
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
        n.sample_style();
        let mut row = read_values(n);
        for &(prop, v) in &p.values {
            for &c in prop.channels() {
                row[c] = v;
            }
        }
        write_values(n, &row);
        n.props.transform = n.sampled.transform;
        n.props.opacity = n.sampled.opacity;
        n.props.z = n.sampled.z;
        n.props.inert = n.sampled.inert;
        n.el.radius = n.sampled.radius;
        n.el.background = n.sampled.background;
        if let Content::Text(text) = &mut n.content {
            text.color = n.sampled.ink;
        }
    }
}

/// A node's animatable values, one per channel (`Prop::channels`).
pub(crate) fn read_values(n: &Node) -> [f32; crate::element::CHANNELS] {
    let t = &n.sampled.transform;
    let bg = n.sampled.background.unwrap_or(Srgba {
        rgb: [0, 0, 0],
        alpha: 0.0,
    });
    let ink = n.sampled.ink;
    let c = |v: u8| f32::from(v) / 255.0;
    [
        t.x,
        t.y,
        t.scale_x,
        t.scale_y,
        t.rotate,
        t.skew_x,
        t.skew_y,
        n.sampled.opacity,
        n.sampled.radius,
        c(bg.rgb[0]),
        c(bg.rgb[1]),
        c(bg.rgb[2]),
        bg.alpha,
        c(ink.rgb[0]),
        c(ink.rgb[1]),
        c(ink.rgb[2]),
        ink.alpha,
        n.sampled.z as f32,
        f32::from(u8::from(n.sampled.inert)),
        n.sampled.blur,
        n.sampled.draw,
    ]
}

/// Writes frame output without changing the next tick's authored inputs.
pub(crate) fn write_values(n: &mut Node, v: &[f32; crate::element::CHANNELS]) {
    let t = &mut n.sampled.transform;
    t.x = v[0];
    t.y = v[1];
    t.scale_x = v[2];
    t.scale_y = v[3];
    t.rotate = v[4];
    t.skew_x = v[5];
    t.skew_y = v[6];
    n.sampled.opacity = v[7].clamp(0.0, 1.0);
    n.sampled.radius = v[8].max(0.0);
    let u = |x: f32| (x.clamp(0.0, 1.0) * 255.0).round() as u8;
    let bg = Srgba {
        rgb: [u(v[9]), u(v[10]), u(v[11])],
        alpha: v[12].clamp(0.0, 1.0),
    };
    if n.sampled.background.is_some() || bg.alpha > 0.0 {
        n.sampled.background = Some(bg);
    }
    n.sampled.ink = Srgba {
        rgb: [u(v[13]), u(v[14]), u(v[15])],
        alpha: v[16].clamp(0.0, 1.0),
    };
    n.sampled.z = v[17].round() as i32;
    n.sampled.inert = v[18] >= 0.5;
    n.sampled.blur = v[19].max(0.0);
    n.sampled.draw = v[20].clamp(0.0, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A snapped edge is on the device pixel grid to within 1/512 px and a
    /// multiple of 1/256 px at every ratio, and snapping it again keeps it:
    /// so a move between snapped edges is exact in `f32`, at 1.5 and 3 as
    /// at 1 and 2 (where nothing moved).
    #[test]
    fn snapped_edges_are_exact_on_every_device_grid() {
        for dpr in [1.0f32, 1.25, 1.5, 2.0, 2.625, 3.0] {
            for k in 0..20_000 {
                let v = k as f32 * 0.731 - 4000.0;
                let s = snap(v, dpr);
                let device = (v * dpr).round() / dpr;
                assert!(
                    (s - device).abs() <= 1.0 / 512.0,
                    "{v} at {dpr}: {s} vs {device}"
                );
                assert_eq!((s * 256.0).fract(), 0.0, "{v} at {dpr}: {s}");
                assert_eq!(snap(s, dpr), s, "{v} at {dpr}");
                if dpr == 1.0 || dpr == 2.0 {
                    assert_eq!(s, device, "{v} at {dpr}");
                }
            }
        }
    }
}
