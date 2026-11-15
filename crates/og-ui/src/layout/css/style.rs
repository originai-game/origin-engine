//! A node's used [`Element`] as Taffy reads CSS styles: a view over the
//! node, built where Taffy asks for it, so no style is copied or kept.
//! A `calc()` length reaches Taffy as a handle (`calc_ptr`) that the tree
//! resolves against its node again (`calc_of`).

use super::super::*;
use taffy as tf;

/// The lengths of a node that may be a `calc()`: their handle's slot.
pub(super) mod slot {
    pub const WIDTH: usize = 0;
    pub const HEIGHT: usize = 1;
    /// `bounds[axis][end]` at `BOUNDS + axis * 2 + end`.
    pub const BOUNDS: usize = 2;
    /// Margins in CSS order (top, right, bottom, left).
    pub const MARGIN: usize = 6;
    pub const BASIS: usize = 10;
    pub const COUNT: usize = 11;
}

/// The handle Taffy keeps for slot `s` of node `i`: never dereferenced,
/// an aligned address so Taffy's tag bits stay clear.
pub(super) fn calc_ptr(i: usize, s: usize) -> *const () {
    std::ptr::without_provenance((i * slot::COUNT + s + 1) << 3)
}

/// The CSS px a [`calc_ptr`] handle resolves to against `basis`.
pub(super) fn calc_of(nodes: &[Node], p: *const (), basis: f32) -> f32 {
    let v = (p.addr() >> 3).saturating_sub(1);
    let (i, s) = (v / slot::COUNT, v % slot::COUNT);
    let Some(n) = nodes.get(i) else { return 0.0 };
    let el = &n.layout.used;
    let size = |s: Size| match s {
        Size::Calc(c) => c.of(basis),
        Size::Percent(p) => p * basis,
        Size::Fixed(v) => v,
        // A content size a bound names: what layout measured last.
        Size::MinContent | Size::MaxContent => basis,
        Size::Fit { .. } | Size::Grow { .. } => 0.0,
    };
    match s {
        slot::WIDTH => size(el.width),
        slot::HEIGHT => size(el.height),
        b if (slot::BOUNDS..slot::MARGIN).contains(&b) => {
            let k = b - slot::BOUNDS;
            match el.bounds[k / 2][k % 2] {
                Some(Size::MinContent) => n.layout.min_content[k / 2],
                Some(Size::MaxContent) => [n.layout.pref_w, n.layout.pref_h][k / 2],
                Some(s) => size(s),
                None => 0.0,
            }
        }
        m if (slot::MARGIN..slot::BASIS).contains(&m) => el.margin[m - slot::MARGIN].len.of(basis),
        _ => el.flex.and_then(|f| f.basis).map_or(0.0, size),
    }
}

/// Whether node `n` lies in a line beside its siblings (CSS 2 §9.2.2): a
/// box displayed `inline-*`, a text that is not block-level, or an image.
pub(crate) fn inline_level(n: &Node) -> bool {
    n.layout.used.inline
        || match *n.content {
            Content::Text(_) | Content::Rich(_) => !n.layout.used.block,
            Content::Image { .. } => true,
            Content::None => false,
        }
}

/// What lays a node's children out (CSS's inner display).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    /// A text, rich text, image or widget: sized by its content.
    Leaf,
    /// Its children overlaid in its box (og-ui's stack).
    Stack,
    Flex,
    /// A CSS block container (block flow, margins collapsing, floats).
    Block,
    Grid,
    /// A CSS table: its cells laid out on its grid of slots.
    Table,
    /// A block holding inline-level children: they lie in lines.
    Lines,
    /// A multi-column container.
    Multicol,
    /// A masonry grid (CSS Grid 3): items packed into one axis's tracks.
    Masonry,
}

/// How node `n` lays out its children.
pub(crate) fn kind(nodes: &[Node], i: usize) -> Kind {
    let n = &nodes[i];
    if !matches!(*n.content, Content::None) || n.layout.cv_skip {
        return Kind::Leaf;
    }
    let el = &n.layout.used;
    if el.table == TablePart::Table {
        return Kind::Table;
    }
    match n.layout_mode() {
        LayoutMode::Stack => return Kind::Stack,
        LayoutMode::Grid => {
            use crate::element::Template;
            let lines = &n.grid().lines;
            return if lines.iter().any(|l| l.kind == Template::Masonry) {
                Kind::Masonry
            } else {
                Kind::Grid
            };
        }
        LayoutMode::Flex => {}
    }
    if n.layout.line_box.is_some() {
        Kind::Lines
    } else if el.multicol.is_some() {
        Kind::Multicol
    } else if el.block && el.direction == Direction::Column && !grows_down(nodes, i) {
        Kind::Block
    } else {
        Kind::Flex
    }
}

/// A block whose flow holds a `stretch` box along its block axis lays
/// out as a column flex container, where that box shares the free room
/// (docs/ui.md, Sizes); one with floats stays a block.
fn grows_down(nodes: &[Node], i: usize) -> bool {
    let n = &nodes[i];
    let turned = turns(frame_of(nodes, i));
    n.floats.is_empty()
        && n.children.iter().any(|&c| {
            let u = &nodes[c].layout.used;
            !nodes[c].props.out_of_flow()
                && matches!(if turned { u.width } else { u.height }, Size::Grow { .. })
        })
}

/// The sides of a frame (CSS Writing Modes 4): frame side `k`
/// (block-start, inline-end, block-end, inline-start) is physical side
/// `s[k]` (0 top … 3 left). Layout space is `[0, 1, 2, 3]`.
pub(crate) type Sides = [u8; 4];
pub(crate) const LAYOUT_SPACE: Sides = [0, 1, 2, 3];

/// The frame node `i`'s box lays its children out in.
pub(crate) fn frame_of(nodes: &[Node], i: usize) -> Sides {
    if i == NONE || i >= nodes.len() {
        return LAYOUT_SPACE;
    }
    nodes[i].layout.used.vertical.unwrap_or(LAYOUT_SPACE)
}

/// Whether frame `s`'s axes cross layout space's.
pub(crate) fn turns(s: Sides) -> bool {
    s[0] % 2 == 1
}

/// Node `i` as Taffy reads its style, its box (kept physical in its
/// node) turned into the frame `frame` it is read in: its own as a
/// container, its parent's as an item.
#[derive(Clone, Copy)]
pub(crate) struct View<'a> {
    pub nodes: &'a [Node],
    pub i: usize,
    pub frame: Sides,
    /// It is measured for its contribution to its flex container's
    /// intrinsic main size: a scroller answers as if it did not scroll.
    pub measured: bool,
    /// It is one of its own lines (`Tree::lines`): a bare flex line its
    /// boxes lie on, its size, bounds and padding its box's.
    pub line: bool,
}

impl<'a> View<'a> {
    /// Node `i` as the container it is.
    pub fn new(nodes: &'a [Node], i: usize) -> Self {
        View {
            nodes,
            i,
            frame: frame_of(nodes, i),
            measured: false,
            line: false,
        }
    }

    /// Node `i` as an item of the box it lays out in.
    pub fn item(nodes: &'a [Node], i: usize) -> Self {
        View {
            nodes,
            i,
            frame: frame_of(nodes, nodes[i].up),
            measured: false,
            line: false,
        }
    }

    fn n(&self) -> &'a Node {
        &self.nodes[self.i]
    }

    fn el(&self) -> &'a Element {
        &self.nodes[self.i].layout.used
    }

    /// The physical axis (0 width, 1 height) its frame's axis `axis` runs on.
    fn phys(&self, axis: usize) -> usize {
        if turns(self.frame) { 1 - axis } else { axis }
    }

    /// The physical side (0 top … 3 left) its frame's side `k` (0 top …
    /// 3 left, in the frame) is.
    fn side(&self, k: usize) -> usize {
        self.frame[k] as usize
    }

    /// Its parent lays its flow out right to left (`direction: rtl`).
    pub fn parent_flips(&self) -> bool {
        let up = self.n().up;
        up != NONE
            && up < self.nodes.len()
            && self.nodes[up].layout.used.flip
            && !turns(frame_of(self.nodes, up))
    }

    /// The node laying it out, and how.
    fn parent(&self) -> Option<(&'a Node, Kind)> {
        let up = self.n().up;
        (up != NONE && up < self.nodes.len() && up != self.i)
            .then(|| (&self.nodes[up], kind(self.nodes, up)))
    }

    /// Its parent's main axis is horizontal (a flex row).
    fn main_across(&self) -> bool {
        self.parent()
            .is_none_or(|(p, _)| p.layout.used.direction == Direction::Row)
    }

    fn dim(&self, s: Size, sl: usize) -> tf::Dimension {
        match s {
            Size::Fixed(v) => tf::Dimension::length(v),
            Size::Percent(p) => tf::Dimension::percent(p),
            Size::Calc(_) => tf::Dimension::calc(calc_ptr(self.i, sl)),
            Size::MinContent => tf::Dimension::min_content(),
            Size::MaxContent => tf::Dimension::max_content(),
            Size::Fit { .. } | Size::Grow { .. } => tf::Dimension::auto(),
        }
    }

    /// Its min (`end` 0) or max bound along its frame's `axis`.
    fn bound(&self, axis: usize, end: usize) -> tf::LengthPercentageAuto {
        // A `stretch` size along a flex line is the room the parent leaves
        // (docs/ui.md, Sizes): down to its own minimum, not its content's.
        let along = self
            .parent()
            .is_some_and(|(_, k)| matches!(k, Kind::Flex | Kind::Lines))
            && (axis == 0) == self.main_across();
        let axis = self.phys(axis);
        let el = self.el();
        let size = [el.width, el.height][axis];
        let held = match (size, end) {
            (Size::Grow { min, .. }, 0) if along => Some(min),
            (Size::Fit { min, .. } | Size::Grow { min, .. }, 0) if min > 0.0 => Some(min),
            (Size::Fit { max, .. } | Size::Grow { max, .. }, 1) if max.is_finite() => Some(max),
            _ => None,
        };
        let sl = slot::BOUNDS + axis * 2 + end;
        match el.bounds[axis][end] {
            Some(Size::Fixed(v)) => tf::LengthPercentageAuto::length(v),
            Some(Size::Percent(p)) => tf::LengthPercentageAuto::percent(p),
            Some(Size::Calc(_) | Size::MinContent | Size::MaxContent) => {
                tf::LengthPercentageAuto::calc(calc_ptr(self.i, sl))
            }
            Some(Size::Fit { .. } | Size::Grow { .. }) | None => held.map_or(
                tf::LengthPercentageAuto::auto(),
                tf::LengthPercentageAuto::length,
            ),
        }
    }

    /// Its flex factors: its CSS ones, else its house kind's along its
    /// parent's main axis (docs/ui.md, Sizes): a `stretch` size shares the
    /// room (`flex: 1`), a fitting one shrinks, a definite one neither.
    fn factors(&self) -> (f32, f32, tf::Dimension) {
        let el = self.el();
        // A block-level box among lines lies on a line of its own.
        if self.parent().is_some_and(|(_, k)| k == Kind::Lines) && !inline_level(self.n()) {
            return (0.0, 1.0, tf::Dimension::percent(1.0));
        }
        if let Some(f) = el.flex {
            let basis = f
                .basis
                .map_or(tf::Dimension::auto(), |b| self.dim(b, slot::BASIS));
            return (f.grow.max(0.0), f.shrink.max(0.0), basis);
        }
        let main = if self.main_across() {
            self.sized(0)
        } else {
            self.sized(1)
        };
        match main {
            Size::Grow { .. } => (1.0, 1.0, tf::Dimension::length(0.0)),
            Size::Fit { .. } => (0.0, 1.0, tf::Dimension::auto()),
            _ => (0.0, 0.0, tf::Dimension::auto()),
        }
    }

    /// Its size along its frame's `axis`.
    fn sized(&self, axis: usize) -> Size {
        let el = self.el();
        [el.width, el.height][self.phys(axis)]
    }

    /// A `stretch` size across its parent's main axis stretches it there.
    fn grows_across(&self) -> bool {
        let cross = if self.main_across() {
            self.sized(1)
        } else {
            self.sized(0)
        };
        matches!(cross, Size::Grow { .. })
    }
}

fn items(c: Cross, logical: bool) -> tf::AlignItems {
    match c {
        Cross::Start if logical => tf::AlignItems::START,
        Cross::Start => tf::AlignItems::FLEX_START,
        Cross::Center => tf::AlignItems::CENTER,
        Cross::End if logical => tf::AlignItems::END,
        Cross::End => tf::AlignItems::FLEX_END,
        Cross::Stretch => tf::AlignItems::STRETCH,
        Cross::Baseline | Cross::LastBaseline => tf::AlignItems::BASELINE,
    }
}

fn content(a: Align, logical: bool) -> tf::AlignContent {
    match a {
        Align::Start if logical => tf::AlignContent::START,
        Align::Start => tf::AlignContent::FLEX_START,
        Align::Center => tf::AlignContent::CENTER,
        Align::End if logical => tf::AlignContent::END,
        Align::End => tf::AlignContent::FLEX_END,
        Align::SpaceBetween => tf::AlignContent::SPACE_BETWEEN,
        Align::SpaceAround => tf::AlignContent::SPACE_AROUND,
        Align::SpaceEvenly => tf::AlignContent::SPACE_EVENLY,
        Align::Stretch => tf::AlignContent::STRETCH,
    }
}

fn safe_items(mut a: tf::AlignItems, safe: bool) -> tf::AlignItems {
    if safe {
        a.safety = tf::AlignmentSafety::Safe;
    }
    a
}

fn safe_content(mut a: tf::AlignContent, safe: bool) -> tf::AlignContent {
    if safe {
        a.safety = tf::AlignmentSafety::Safe;
    }
    a
}

impl tf::CoreStyle for View<'_> {
    type CustomIdent = String;

    fn is_block(&self) -> bool {
        let n = self.n();
        let el = self.el();
        // A flow-root, a multi-column box and an orthogonal flow start a
        // context of their own (CSS Display 3 §2.3, CSS Writing Modes 4
        // §7.3): Taffy lays them out as boxes of their own, not in their
        // parent's flow.
        let orthogonal = turns(frame_of(self.nodes, self.i)) != turns(frame_of(self.nodes, n.up));
        match kind(self.nodes, self.i) {
            Kind::Block | Kind::Lines => !el.bfc && !orthogonal,
            Kind::Leaf => el.block && !el.inline && !orthogonal,
            _ => false,
        }
    }

    fn box_sizing(&self) -> tf::BoxSizing {
        tf::BoxSizing::BorderBox
    }

    /// CSS `direction`: a box whose inline direction runs against its
    /// layout space's lays its flex lines, grid columns and block flow out
    /// from the right (a vertical one's flip is its block axis, which its
    /// frame turns).
    fn direction(&self) -> tf::Direction {
        if self.el().flip && !turns(self.frame) {
            tf::Direction::Rtl
        } else {
            tf::Direction::Ltr
        }
    }

    fn overflow(&self) -> tf::Point<tf::Overflow> {
        let el = self.el();
        // CSS Flexbox 1 §9.9.1: an item's intrinsic contribution is its
        // content's whether or not it scrolls (Taffy answers a scroller's
        // basis alone).
        let of = |s: bool| {
            if s && !self.measured && !self.line {
                tf::Overflow::Scroll
            } else {
                tf::Overflow::Visible
            }
        };
        let (x, y) = if turns(self.frame) {
            (el.scroll, el.scroll_x)
        } else {
            (el.scroll_x, el.scroll)
        };
        tf::Point { x: of(x), y: of(y) }
    }

    fn position(&self) -> tf::Position {
        tf::Position::Relative
    }

    fn size(&self) -> tf::Size<tf::Dimension> {
        if self.line {
            return tf::Size::auto();
        }
        let slots = [slot::WIDTH, slot::HEIGHT];
        // A fitting box in a block flow shrinks to fit its line (CSS 2
        // §10.3.5): the house's `Fit` is CSS's `fit-content` there, where
        // `auto` would fill it.
        let in_flow = self
            .parent()
            .is_some_and(|(_, k)| matches!(k, Kind::Block | Kind::Multicol));
        if in_flow && matches!(self.sized(0), Size::Fit { .. }) {
            return tf::Size {
                width: tf::Dimension::fit_content(),
                height: self.dim(self.sized(1), slots[self.phys(1)]),
            };
        }
        tf::Size {
            width: self.dim(self.sized(0), slots[self.phys(0)]),
            height: self.dim(self.sized(1), slots[self.phys(1)]),
        }
    }

    fn min_size(&self) -> tf::Size<tf::LengthPercentageAuto> {
        if self.line {
            return tf::Size::auto();
        }
        tf::Size {
            width: self.bound(0, 0),
            height: self.bound(1, 0),
        }
    }

    fn max_size(&self) -> tf::Size<tf::LengthPercentageAuto> {
        if self.line {
            return tf::Size::auto();
        }
        tf::Size {
            width: self.bound(0, 1),
            height: self.bound(1, 1),
        }
    }

    fn aspect_ratio(&self) -> Option<f32> {
        if self.line {
            return None;
        }
        let a = self.el().aspect;
        (a > 0.0).then(|| if turns(self.frame) { 1.0 / a } else { a })
    }

    fn margin(&self) -> tf::Rect<tf::LengthPercentageAuto> {
        let el = self.el();
        // A stack places its children without their margins.
        if self.parent().is_some_and(|(_, k)| k == Kind::Stack) {
            return tf::Rect {
                left: tf::LengthPercentageAuto::length(0.0),
                right: tf::LengthPercentageAuto::length(0.0),
                top: tf::LengthPercentageAuto::length(0.0),
                bottom: tf::LengthPercentageAuto::length(0.0),
            };
        }
        let side = |k: usize| {
            let k = self.side(k);
            let m = el.margin[k];
            if m.auto {
                tf::LengthPercentageAuto::auto()
            } else if m.len.pct == 0.0 {
                tf::LengthPercentageAuto::length(m.len.px)
            } else if m.len.px == 0.0 {
                tf::LengthPercentageAuto::percent(m.len.pct)
            } else {
                tf::LengthPercentageAuto::calc(calc_ptr(self.i, slot::MARGIN + k))
            }
        };
        tf::Rect {
            top: side(0),
            right: side(1),
            bottom: side(2),
            left: side(3),
        }
    }

    fn padding(&self) -> tf::Rect<tf::LengthPercentage> {
        if self.line {
            return tf::Rect::zero();
        }
        let p = self.el().padding;
        let v = [p.top, p.right, p.bottom, p.left];
        let at = |k: usize| tf::LengthPercentage::length(v[self.side(k)]);
        tf::Rect {
            top: at(0),
            right: at(1),
            bottom: at(2),
            left: at(3),
        }
    }

    fn contain(&self) -> tf::Contain {
        let el = self.el();
        let mut c = tf::Contain::NONE;
        if el.contain.has(Contain::LAYOUT) {
            c = c.union(tf::Contain::LAYOUT);
        }
        if el.contain.has(Contain::PAINT) {
            c = c.union(tf::Contain::PAINT);
        }
        c
    }
}

impl tf::FlexboxContainerStyle for View<'_> {
    fn flex_direction(&self) -> tf::FlexDirection {
        let el = self.el();
        match (el.direction, el.reverse) {
            (Direction::Row, false) => tf::FlexDirection::Row,
            (Direction::Row, true) => tf::FlexDirection::RowReverse,
            (Direction::Column, false) => tf::FlexDirection::Column,
            (Direction::Column, true) => tf::FlexDirection::ColumnReverse,
        }
    }

    fn flex_wrap(&self) -> tf::FlexWrap {
        if self.line {
            return tf::FlexWrap::NoWrap;
        }
        let el = self.el();
        match (el.wrap, el.wrap_reverse, el.wrap_balance) {
            (false, ..) => tf::FlexWrap::NoWrap,
            (true, false, true) => tf::FlexWrap::Balance,
            (true, false, false) => tf::FlexWrap::Wrap,
            (true, true, _) => tf::FlexWrap::WrapReverse,
        }
    }

    fn gap(&self) -> tf::Size<tf::LengthPercentage> {
        let el = self.el();
        tf::Size {
            width: tf::LengthPercentage::length(el.gaps[0].unwrap_or(el.gap)),
            height: tf::LengthPercentage::length(el.gaps[1].unwrap_or(el.gap)),
        }
    }

    fn align_content(&self) -> tf::AlignContent {
        let el = self.el();
        el.align_content
            .map_or(tf::AlignContent::FLEX_START, |a| content(a, false))
    }

    fn align_items(&self) -> tf::AlignItems {
        let el = self.el();
        safe_items(items(el.cross, el.cross_logical & 1 != 0), el.safe & 2 != 0)
    }

    fn justify_content(&self) -> tf::JustifyContent {
        let el = self.el();
        // `stretch` distributes as `flex-start` in a flex line.
        let a = match el.align {
            Align::Stretch => Align::Start,
            a => a,
        };
        safe_content(content(a, el.align_logical), el.safe & 1 != 0)
    }
}

impl tf::FlexboxItemStyle for View<'_> {
    fn flex_basis(&self) -> tf::Dimension {
        self.factors().2
    }

    fn flex_grow(&self) -> f32 {
        self.factors().0
    }

    fn flex_shrink(&self) -> f32 {
        self.factors().1
    }

    fn align_self(&self) -> Option<tf::AlignSelf> {
        let el = self.el();
        match el.align_self {
            Some(c) => Some(safe_items(
                items(c, el.cross_logical & 2 != 0),
                el.safe & 2 != 0,
            )),
            None if self.grows_across() => Some(tf::AlignItems::STRETCH),
            None => None,
        }
    }
}

impl tf::BlockContainerStyle for View<'_> {
    /// CSS Box Alignment 3 §5.1.1: a block's content shifted as one in its
    /// block axis; any value but `normal` starts a context of its own.
    fn align_content(&self) -> tf::AlignContent {
        let el = self.el();
        match el.align_content {
            None | Some(Align::Stretch) => tf::AlignContent::NORMAL,
            Some(a) => safe_content(content(a, true), el.safe & 1 != 0),
        }
    }
}

impl tf::BlockItemStyle for View<'_> {
    fn is_table(&self) -> bool {
        self.el().table == TablePart::Table
    }

    /// Its float side, physical: the lowering gives it before its
    /// parent's right-to-left mirror, which Taffy's `direction` replaces.
    fn float(&self) -> tf::Float {
        let swap = self.parent_flips();
        match (self.el().float, swap) {
            (FloatSide::None, _) => tf::Float::None,
            (FloatSide::Left, false) | (FloatSide::Right, true) => tf::Float::Left,
            (FloatSide::Right, false) | (FloatSide::Left, true) => tf::Float::Right,
        }
    }

    fn clear(&self) -> tf::Clear {
        let c = self.el().clear & (float::LEFT | float::RIGHT);
        // One side swaps; none and both stay.
        let both = float::LEFT | float::RIGHT;
        let c = if self.parent_flips() && c != 0 && c != both {
            c ^ both
        } else {
            c
        };
        match c {
            0 => tf::Clear::None,
            c if c == float::LEFT => tf::Clear::Left,
            c if c == float::RIGHT => tf::Clear::Right,
            _ => tf::Clear::Both,
        }
    }
}

impl tf::GridItemStyle for View<'_> {
    fn grid_row(&self) -> tf::Line<tf::GridPlacement<String>> {
        super::grid::placement(self.nodes, self.i, 1)
    }

    fn grid_column(&self) -> tf::Line<tf::GridPlacement<String>> {
        super::grid::placement(self.nodes, self.i, 0)
    }

    fn align_self(&self) -> Option<tf::AlignSelf> {
        let el = self.el();
        el.align_self.map(|c| items(c, true)).or_else(|| {
            matches!(self.sized(1), Size::Grow { .. }).then_some(tf::AlignItems::STRETCH)
        })
    }

    fn justify_self(&self) -> Option<tf::AlignSelf> {
        let el = self.el();
        el.justify_self.map(|c| items(c, true)).or_else(|| {
            matches!(self.sized(0), Size::Grow { .. }).then_some(tf::AlignItems::STRETCH)
        })
    }
}

/// A grid container: its own box from its node, its tracks and areas from
/// the grid style built for this layout ([`super::grid::style`]).
#[derive(Clone, Copy)]
pub(crate) struct GridView<'a> {
    pub view: View<'a>,
    pub grid: &'a tf::Style<String>,
}

impl tf::CoreStyle for GridView<'_> {
    type CustomIdent = String;
    fn is_block(&self) -> bool {
        false
    }
    fn box_sizing(&self) -> tf::BoxSizing {
        tf::BoxSizing::BorderBox
    }
    fn overflow(&self) -> tf::Point<tf::Overflow> {
        self.view.overflow()
    }
    fn position(&self) -> tf::Position {
        tf::Position::Relative
    }
    fn size(&self) -> tf::Size<tf::Dimension> {
        self.view.size()
    }
    fn min_size(&self) -> tf::Size<tf::LengthPercentageAuto> {
        self.view.min_size()
    }
    fn max_size(&self) -> tf::Size<tf::LengthPercentageAuto> {
        self.view.max_size()
    }
    fn aspect_ratio(&self) -> Option<f32> {
        self.view.aspect_ratio()
    }
    fn margin(&self) -> tf::Rect<tf::LengthPercentageAuto> {
        self.view.margin()
    }
    fn padding(&self) -> tf::Rect<tf::LengthPercentage> {
        self.view.padding()
    }
    fn contain(&self) -> tf::Contain {
        tf::Contain::LAYOUT
    }
}

impl tf::GridContainerStyle for GridView<'_> {
    type Repetition<'b>
        = <tf::Style<String> as tf::GridContainerStyle>::Repetition<'b>
    where
        Self: 'b;
    type TemplateTrackList<'b>
        = <tf::Style<String> as tf::GridContainerStyle>::TemplateTrackList<'b>
    where
        Self: 'b;
    type AutoTrackList<'b>
        = <tf::Style<String> as tf::GridContainerStyle>::AutoTrackList<'b>
    where
        Self: 'b;
    type TemplateLineNames<'b>
        = <tf::Style<String> as tf::GridContainerStyle>::TemplateLineNames<'b>
    where
        Self: 'b;
    type GridTemplateAreas<'b>
        = <tf::Style<String> as tf::GridContainerStyle>::GridTemplateAreas<'b>
    where
        Self: 'b;

    fn grid_template_rows(&self) -> Option<Self::TemplateTrackList<'_>> {
        self.grid.grid_template_rows()
    }
    fn grid_template_columns(&self) -> Option<Self::TemplateTrackList<'_>> {
        self.grid.grid_template_columns()
    }
    fn grid_auto_rows(&self) -> Self::AutoTrackList<'_> {
        self.grid.grid_auto_rows()
    }
    fn grid_auto_columns(&self) -> Self::AutoTrackList<'_> {
        self.grid.grid_auto_columns()
    }
    fn grid_template_areas(&self) -> Option<Self::GridTemplateAreas<'_>> {
        self.grid.grid_template_areas()
    }
    fn grid_template_area_row_count(&self) -> u16 {
        self.grid.grid_template_area_row_count()
    }
    fn grid_template_area_column_count(&self) -> u16 {
        self.grid.grid_template_area_column_count()
    }
    fn grid_template_column_names(&self) -> Option<Self::TemplateLineNames<'_>> {
        self.grid.grid_template_column_names()
    }
    fn grid_template_row_names(&self) -> Option<Self::TemplateLineNames<'_>> {
        self.grid.grid_template_row_names()
    }
    fn grid_auto_flow(&self) -> tf::GridAutoFlow {
        self.grid.grid_auto_flow
    }
    fn gap(&self) -> tf::Size<tf::LengthPercentage> {
        self.grid.gap
    }
    fn align_content(&self) -> tf::AlignContent {
        self.grid.align_content
    }
    fn justify_content(&self) -> tf::JustifyContent {
        self.grid.justify_content
    }
    fn align_items(&self) -> tf::AlignItems {
        self.grid.align_items
    }
    fn justify_items(&self) -> tf::AlignItems {
        self.grid.justify_items
    }
}
