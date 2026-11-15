//! og-ui's layout core: CSS Flexbox, Grid and block flow (margins
//! collapsing, floats and clearance) laid out by Taffy over the node array
//! itself ([`Tree`]). og-ui keeps what CSS leaves to the box's content or
//! to the engine: text and its lines (a leaf Taffy measures), stacks,
//! tables' slots, masonry, multi-column, writing modes, out-of-flow and
//! anchored boxes, and the cache that says which nodes lay out again.

mod grid;
mod inline;
mod run;
mod style;

pub(crate) use run::{layout, on_screen};

use style::{GridView, View, calc_of, turns};
pub(crate) use style::{Kind, kind};

use super::*;
use taffy as tf;
use tf::{
    CacheTree, LayoutBlockContainer, LayoutFlexboxContainer, LayoutGridContainer,
    LayoutPartialTree, TraversePartialTree,
};

/// The node array as Taffy's tree: a node's id is its index.
pub(super) struct Tree<'a, 'm> {
    pub nodes: &'a mut [Node],
    pub measure: &'a mut Measure<'m>,
    /// Boxes laid out (not answered from the cache) this call.
    pub laid: u32,
    /// Per node: laid out in full this call.
    pub performed: Vec<bool>,
    /// The grid and table containers' tracks and areas, built where they
    /// first lay out this call ([`grid::style`]).
    grids: std::collections::HashMap<usize, tf::Style<String>>,
    /// The style a grid container answers before its own is built.
    empty: tf::Style<String>,
    /// The column flex container being measured for its inline size, if
    /// any ([`Tree::through_ratio`]).
    measuring: usize,
    /// The flex container being measured for its intrinsic main size, if
    /// any (`View::measured`).
    intrinsic: usize,
    /// The block formatting contexts being laid out, innermost last.
    bfcs: Vec<inline::Bfc>,
    /// Per context root holding a float whose contour narrows lines: its
    /// floats and line boxes as its last pass placed them.
    shaped: std::collections::HashMap<usize, inline::Shaped>,
    /// A block's line laid out as one flex line: the block and the range
    /// of its children on it ([`Tree::lines`]).
    line: Option<(usize, usize, usize)>,
}

fn id(i: usize) -> tf::NodeId {
    tf::NodeId::from(i)
}

/// A node's children as Taffy lays them out: its in-flow children in
/// CSS `order`; a block with floats its flow and its floats in document
/// order, as block flow places them; none while it skips its contents.
pub(super) enum Kids<'a> {
    Plain(std::slice::Iter<'a, usize>),
    Flow(std::slice::Iter<'a, usize>, &'a [Node]),
}

impl Iterator for Kids<'_> {
    type Item = tf::NodeId;
    fn next(&mut self) -> Option<tf::NodeId> {
        match self {
            Kids::Plain(it) => it.next().map(|&c| id(c)),
            Kids::Flow(it, nodes) => it.find(|&&c| !nodes[c].props.out_of_flow()).map(|&c| id(c)),
        }
    }
}

/// Whether node `n` hands Taffy its floats among its flow.
fn floats_in_flow(nodes: &[Node], i: usize) -> bool {
    !nodes[i].floats.is_empty() && matches!(kind(nodes, i), Kind::Block)
}

pub(super) fn kids<'a>(nodes: &'a [Node], i: usize) -> Kids<'a> {
    let n = &nodes[i];
    if n.layout.cv_skip {
        return Kids::Plain([].iter());
    }
    if floats_in_flow(nodes, i) {
        Kids::Flow(n.ordered_children.iter(), nodes)
    } else {
        Kids::Plain(n.children.iter())
    }
}

impl<'a, 'm> Tree<'a, 'm> {
    pub fn new(nodes: &'a mut [Node], measure: &'a mut Measure<'m>) -> Self {
        let performed = vec![false; nodes.len()];
        let mut empty = tf::Style::<String>::DEFAULT;
        empty.display = tf::Display::Grid;
        Tree {
            nodes,
            measure,
            laid: 0,
            performed,
            grids: Default::default(),
            empty,
            measuring: NONE,
            intrinsic: NONE,
            bfcs: Vec::new(),
            shaped: Default::default(),
            line: None,
        }
    }

    /// Node `i` as the container it is, or as one of its lines.
    fn container(&self, i: usize) -> View<'_> {
        let mut v = View::new(self.nodes, i);
        v.line = self.line.is_some_and(|l| l.0 == i);
        v
    }

    /// Whether node `i` was laid out in full this call.
    pub fn performed(&self, i: usize) -> bool {
        self.performed.get(i).copied().unwrap_or(false)
    }

    /// Lays node `i` out again at `input`, an input it was given in its
    /// own frame (`LayoutBox::css_input`).
    pub fn relay(&mut self, i: usize, input: tf::LayoutInput) {
        self.lay_turned(id(i), input, None);
    }

    /// The box that lays out again for a change in block `p`'s children:
    /// `p`, or, where `p` lies in a block flow that holds floats, the
    /// outermost block of that flow (the floats it places reach `p`).
    pub fn flow_of(&self, p: usize) -> usize {
        if p == NONE {
            return p;
        }
        let mut top = p;
        let mut floats = !self.nodes[p].floats.is_empty();
        while kind(self.nodes, top) == Kind::Block {
            let up = self.nodes[top].up;
            if up == NONE || kind(self.nodes, up) != Kind::Block {
                break;
            }
            top = up;
            floats |= !self.nodes[top].floats.is_empty();
        }
        if floats { top } else { p }
    }

    /// Lays node `i` out at `input` (its whole subtree, as Taffy asks).
    pub fn lay(&mut self, i: usize, input: tf::LayoutInput) -> tf::LayoutOutput {
        self.compute_child_layout(id(i), input)
    }

    fn lay_in(
        &mut self,
        node: tf::NodeId,
        input: tf::LayoutInput,
        ctx: Option<&mut tf::BlockContext<'_>>,
    ) -> tf::LayoutOutput {
        if input.run_mode == tf::RunMode::PerformHiddenLayout {
            return tf::compute_hidden_layout(self, node);
        }
        let i = usize::from(node);
        // An orthogonal flow (CSS Writing Modes 4 §7.3): its frame's axes
        // cross its parent's, so it lays out at the input turned into its
        // frame and answers its size turned back, in a context of its own.
        let own = style::frame_of(self.nodes, i);
        let turned =
            style::turns(own) != style::turns(style::frame_of(self.nodes, self.nodes[i].up));
        // A box laid out without its parent's context starts one: a float
        // among them is one of its parent's context's floats.
        let starts = ctx.is_none() || turned;
        if starts {
            if self.nodes[i].layout.used.float != FloatSide::None
                && let Some(b) = self.bfcs.last_mut()
                && !b.floats.contains(&i)
            {
                b.floats.push(i);
            }
            self.bfcs.push(inline::Bfc {
                root: i,
                ..Default::default()
            });
        }
        let out = if turned {
            flip_output(self.lay_turned(node, flip_input(input), None))
        } else {
            self.lay_turned(node, input, ctx)
        };
        if starts {
            self.bfcs.pop();
        }
        out
    }

    fn lay_turned(
        &mut self,
        node: tf::NodeId,
        input: tf::LayoutInput,
        ctx: Option<&mut tf::BlockContext<'_>>,
    ) -> tf::LayoutOutput {
        let i = usize::from(node);
        let input = self.contained(i, input);
        let input = self.through_ratio(i, input);
        let input = self.fit_content(i, input);
        let input = self.block_size_from_content(i, input);
        // A text among floats breaks its lines around them, where they
        // stand this time: not a size to keep.
        let around = matches!(kind(self.nodes, i), Kind::Leaf | Kind::Lines)
            && ctx.as_ref().is_some_and(|c| c.has_floats());
        if around {
            return self.uncached(i, input, ctx);
        }
        tf::compute_cached_layout(self, node, input, |t, node, input| {
            t.uncached(usize::from(node), input, ctx)
        })
    }

    /// CSS Sizing 4 §5.1: a box with a preferred aspect ratio whose size
    /// is automatic in one axis takes that axis from the other's known
    /// size, through the ratio, on the box `box-sizing` names, clamped by
    /// its own min and max. Taffy transfers only a box's styled sizes; a
    /// size its parent decided (a flexed main size, a stretched cross
    /// size, a block's fill) reaches it here.
    fn through_ratio(&self, i: usize, mut input: tf::LayoutInput) -> tf::LayoutInput {
        use tf::{CoreStyle, MaybeMath, MaybeResolve, ResolveOrZero};
        let v = View::new(self.nodes, i);
        let Some(ratio) = v.aspect_ratio() else {
            return input;
        };
        let known = input.known_dimensions;
        let size = v.size();
        let parent = input.parent_size;
        let calc = |p, b| self.resolve_calc_value(p, b);
        let pb = v.padding().resolve_or_zero(parent.width, calc).sum_axes()
            + v.border().resolve_or_zero(parent.width, calc).sum_axes();
        let adj = if v.box_sizing() == tf::BoxSizing::ContentBox {
            pb
        } else {
            tf::Size::ZERO
        };
        let min = v.min_size().maybe_resolve(parent, calc);
        let max = v.max_size().maybe_resolve(parent, calc);
        // A column flex container measured for its inline size takes its
        // items' max-content contributions (CSS Flexbox 1 §9.9.1): an
        // item's flexed height, decided later, does not reach its width.
        if self.measuring != NONE
            && self.nodes[i].up == self.measuring
            && known.width.is_none()
            && known.height.is_some()
        {
            input.known_dimensions.height = None;
            input.available_space.width = tf::AvailableSpace::MaxContent;
            return input;
        }
        match (known.width, known.height) {
            (Some(w), None) if size.height.is_auto() => {
                let h = (w - adj.width).max(0.0) / ratio + adj.height;
                input.known_dimensions.height =
                    Some(h.maybe_clamp(min.height, max.height).max(pb.height));
            }
            (None, Some(h)) if size.width.is_auto() => {
                let w = (h - adj.height).max(0.0) * ratio + adj.width;
                input.known_dimensions.width =
                    Some(w.maybe_clamp(min.width, max.width).max(pb.width));
            }
            _ => {}
        }
        input
    }

    /// CSS Containment 2 §3.1: a box with size containment along an axis
    /// is sized there as if it were empty, its content the length its
    /// `contain-intrinsic-size` gives (none: 0); its contents still lay
    /// out in the box that makes.
    fn contained(&self, i: usize, mut input: tf::LayoutInput) -> tf::LayoutInput {
        let c = &self.nodes[i].layout.used.contain;
        let axes = [c.has(Contain::SIZE_W), c.has(Contain::SIZE_H)];
        let k = input.known_dimensions;
        if !(axes[0] && k.width.is_none() || axes[1] && k.height.is_none()) {
            return input;
        }
        let content = [0, 1].map(|a| c.intrinsic[a].1.unwrap_or(0.0));
        let view = View::new(self.nodes, i);
        let empty = tf::compute_leaf_layout(
            tf::LayoutInput {
                run_mode: tf::RunMode::ComputeSize,
                ..input
            },
            &view,
            |p, b| calc_of(self.nodes, p, b),
            |known, _| tf::Size {
                width: known.width.unwrap_or(content[0]),
                height: known.height.unwrap_or(content[1]),
            },
        );
        if axes[0] && k.width.is_none() {
            input.known_dimensions.width = Some(empty.size.width);
        }
        if axes[1] && k.height.is_none() {
            input.known_dimensions.height = Some(empty.size.height);
        }
        input
    }

    /// CSS Sizing 3 §5.1: a box's `auto` block size is its content's, the
    /// room below it notwithstanding. A column flex container measured
    /// into definite room sums its items' flex bases instead (Taffy), which
    /// loses an item whose basis is 0; measured into max-content room it
    /// sums their contributions (CSS Flexbox 1 §9.9.1).
    fn block_size_from_content(&self, i: usize, mut input: tf::LayoutInput) -> tf::LayoutInput {
        use tf::FlexboxContainerStyle;
        if input.known_dimensions.height.is_some()
            || !input.available_space.height.is_definite()
            || !matches!(kind(self.nodes, i), Kind::Flex | Kind::Lines)
        {
            return input;
        }
        if matches!(
            View::new(self.nodes, i).flex_direction(),
            tf::FlexDirection::Column | tf::FlexDirection::ColumnReverse
        ) {
            input.available_space.height = tf::AvailableSpace::MaxContent;
        }
        input
    }

    /// CSS Sizing 3 §5.1: a row flex container (or lines) whose width is
    /// `auto` or `fit-content` and not set by its parent takes the fit-content width of
    /// the room it is given, its max-content width capped by the room and
    /// floored by its min-content width. Taffy sizes it by its items'
    /// flex bases instead, which loses an item whose basis is 0.
    fn fit_content(&mut self, i: usize, mut input: tf::LayoutInput) -> tf::LayoutInput {
        use tf::{CoreStyle, FlexboxContainerStyle, MaybeMath, MaybeResolve};
        let tf::AvailableSpace::Definite(room) = input.available_space.width else {
            return input;
        };
        if input.known_dimensions.width.is_some()
            || !matches!(kind(self.nodes, i), Kind::Flex | Kind::Lines)
        {
            return input;
        }
        let v = View::new(self.nodes, i);
        let row = matches!(
            v.flex_direction(),
            tf::FlexDirection::Row | tf::FlexDirection::RowReverse
        );
        let width = v.size().width;
        if !row || !(width.is_auto() || width == tf::Dimension::fit_content()) {
            return input;
        }
        let parent = input.parent_size;
        let calc = |p, b| self.resolve_calc_value(p, b);
        let (min, max) = (
            v.min_size().maybe_resolve(parent, calc).width,
            v.max_size().maybe_resolve(parent, calc).width,
        );
        let measure = |t: &mut Self, a: tf::AvailableSpace| {
            let probe = tf::LayoutInput {
                run_mode: tf::RunMode::ComputeSize,
                axis: tf::RequestedAxis::Horizontal,
                available_space: tf::Size {
                    width: a,
                    height: input.available_space.height,
                },
                ..input
            };
            t.lay_turned(id(i), probe, None).size.width
        };
        let most = measure(self, tf::AvailableSpace::MaxContent);
        let least = measure(self, tf::AvailableSpace::MinContent);
        let w = most.min(room).max(least).maybe_clamp(min, max);
        input.known_dimensions.width = Some(w);
        input
    }

    fn uncached(
        &mut self,
        i: usize,
        input: tf::LayoutInput,
        ctx: Option<&mut tf::BlockContext<'_>>,
    ) -> tf::LayoutOutput {
        let perform = input.run_mode == tf::RunMode::PerformLayout;
        if perform {
            self.laid += 1;
            self.performed[i] = true;
            self.nodes[i].layout.css_input = Some(input);
        }
        let mut ctx = ctx;
        let mut out = self.dispatch(i, input, ctx.as_deref_mut());
        // A context with a shaped float lays out again with its floats
        // where this pass put them, until they stay (twice at most).
        if perform && ctx.is_none() {
            for _ in 0..2 {
                if !self.settle_shapes(i) {
                    break;
                }
                self.clear_below(i);
                if let Some(b) = self.bfcs.last_mut() {
                    (b.floats, b.wrappers) = Default::default();
                }
                out = self.dispatch(i, input, None);
            }
        }
        // A container query reads the element's own inline size, now
        // known: where its patch changes its box, it lays out again so.
        if perform
            && self.nodes[i]
                .props
                .variants
                .iter()
                .any(|v| v.when.container)
        {
            let before = (
                self.nodes[i].layout.used,
                self.nodes[i].layout.varied.clone(),
            );
            let n = &mut self.nodes[i];
            n.layout.query_w = Some(out.size.width);
            n.derive_used();
            if (n.layout.used, n.layout.varied.clone()) != before {
                self.grids.remove(&i);
                out = self.dispatch(i, input, ctx);
            }
        }
        if perform {
            self.nodes[i].layout.css.scrollable_overflow_rect = out.scrollable_overflow_rect;
            // Its size in layout space, for what it places there itself.
            let mut size = [out.size.width, out.size.height];
            if turns(style::frame_of(self.nodes, i)) {
                size.swap(0, 1);
            }
            self.out_of_flow(i, size);
        }
        out
    }

    /// Node `i` laid out by its kind's algorithm.
    fn dispatch(
        &mut self,
        i: usize,
        input: tf::LayoutInput,
        ctx: Option<&mut tf::BlockContext<'_>>,
    ) -> tf::LayoutOutput {
        let perform = input.run_mode == tf::RunMode::PerformLayout;
        let node = id(i);
        match kind(self.nodes, i) {
            Kind::Leaf => self.leaf(i, input, ctx),
            Kind::Lines => match self.lines(i, input, ctx) {
                Some(out) => out,
                None => {
                    if perform {
                        self.nodes[i].layout.inline_rows.clear();
                    }
                    self.flex(i, input)
                }
            },
            Kind::Flex => self.flex(i, input),
            Kind::Block => tf::compute_block_layout(self, node, input, ctx),
            Kind::Multicol => self.multicol(i, input),
            k @ (Kind::Grid | Kind::Masonry) => {
                if !self.grids.contains_key(&i) {
                    let s = grid::style(self.nodes, i, [None, None]);
                    self.grids.insert(i, s);
                }
                let out = tf::compute_grid_layout(self, node, input);
                if perform {
                    self.subgrids(i);
                }
                if perform && k == Kind::Masonry {
                    self.masonry(i, out)
                } else {
                    out
                }
            }
            Kind::Table => {
                if !self.grids.contains_key(&i) {
                    let s = grid::table_style(self.nodes, i);
                    self.grids.insert(i, s);
                }
                tf::compute_grid_layout(self, node, input)
            }
            Kind::Stack => self.stack(i, input),
        }
    }

    /// Every cached size under node `i` (its descendants' and floats').
    fn clear_below(&mut self, i: usize) {
        let mut stack = vec![i];
        while let Some(k) = stack.pop() {
            self.nodes[k].layout.css_cache.clear();
            let n = &self.nodes[k];
            stack.extend(n.children.iter().chain(&n.floats).chain(&n.out).copied());
        }
    }

    /// Flex container `i` laid out by Taffy's flexbox.
    fn flex(&mut self, i: usize, input: tf::LayoutInput) -> tf::LayoutOutput {
        let node = id(i);
        {
            use tf::FlexboxContainerStyle;
            let column = matches!(
                View::new(self.nodes, i).flex_direction(),
                tf::FlexDirection::Column | tf::FlexDirection::ColumnReverse
            );
            let measured = column
                && input.run_mode == tf::RunMode::ComputeSize
                && input.known_dimensions.width.is_none()
                && !input.available_space.width.is_definite();
            let (main_known, main_room) = if column {
                (input.known_dimensions.height, input.available_space.height)
            } else {
                (input.known_dimensions.width, input.available_space.width)
            };
            let was = (self.measuring, self.intrinsic);
            if measured {
                self.measuring = i;
            }
            if main_known.is_none() && !main_room.is_definite() {
                self.intrinsic = i;
            }
            let out = tf::compute_flexbox_layout(self, node, input);
            (self.measuring, self.intrinsic) = was;
            out
        }
    }

    /// A text, rich text or image: its content measured as CSS sizes a
    /// replaced or inline-level box, its lines broken at the width it is
    /// given.
    fn leaf(
        &mut self,
        i: usize,
        input: tf::LayoutInput,
        ctx: Option<&mut tf::BlockContext<'_>>,
    ) -> tf::LayoutOutput {
        if let Some(ctx) = ctx
            && ctx.has_active_floats(0.0)
            && let Some(out) = self.text_around(i, input, ctx)
        {
            return out;
        }
        let nodes: &[Node] = self.nodes;
        let measure = &mut *self.measure;
        let view = View::new(nodes, i);
        let mut lines = None;
        let mut out = tf::compute_leaf_layout(
            input,
            &view,
            |p, b| calc_of(nodes, p, b),
            |known, avail| {
                let (size, l) = text::content(nodes, i, measure, known, avail);
                lines = l;
                size
            },
        );
        let n = &nodes[i];
        let pad = n.layout.used.padding;
        if let Some(l) = &lines {
            out.baselines.first = text::first_baseline(n, l, measure.fonts).map(|b| b + pad.top);
        }
        if input.run_mode == tf::RunMode::PerformLayout {
            self.nodes[i].layout.beside = false;
            if let Some(l) = lines {
                l.store(&mut self.nodes[i]);
            }
        }
        out
    }

    /// A stack: each child laid out in its content box and placed by its
    /// insets there, else by the stack's alignment; it is as large as its
    /// largest child.
    fn stack(&mut self, i: usize, input: tf::LayoutInput) -> tf::LayoutOutput {
        let nodes: &[Node] = self.nodes;
        let n = &nodes[i];
        let el = n.layout.used;
        let pad = el.padding;
        let parent = input.parent_size;
        let own = |axis: usize| {
            let size = [el.width, el.height][axis];
            let basis = [parent.width, parent.height][axis];
            [input.known_dimensions.width, input.known_dimensions.height][axis].or(match size {
                Size::Grow { .. } => None,
                s => s
                    .definite(0.0, 0.0, basis)
                    .filter(|_| !matches!(s, Size::MinContent | Size::MaxContent)),
            })
        };
        let (kw, kh) = (own(0), own(1));
        let inner = |v: Option<f32>, p: f32| v.map(|v| (v - p).max(0.0));
        let (iw, ih) = (inner(kw, pad.horizontal()), inner(kh, pad.vertical()));
        let avail = tf::Size {
            width: iw.map_or(
                input
                    .available_space
                    .width
                    .map_definite_value(|v| (v - pad.horizontal()).max(0.0)),
                tf::AvailableSpace::Definite,
            ),
            height: ih.map_or(
                input
                    .available_space
                    .height
                    .map_definite_value(|v| (v - pad.vertical()).max(0.0)),
                tf::AvailableSpace::Definite,
            ),
        };
        let kids: Vec<usize> = n.children.clone();
        let mut sizes = Vec::with_capacity(kids.len());
        let (mut cw, mut ch) = (0.0f32, 0.0f32);
        for &c in &kids {
            let k = &self.nodes[c];
            let ins = k.layout.insets;
            let fills = |size: Size, a: Option<f32>, b: Option<f32>, stretch: bool| {
                matches!(size, Size::Grow { .. })
                    || (a.is_some() && b.is_some() && !matches!(size, Size::Fixed(_)))
                    || (stretch && matches!(size, Size::Fit { .. }))
            };
            let kw_ = iw
                .filter(|_| {
                    fills(
                        k.layout.used.width,
                        ins.start,
                        ins.end,
                        el.cross == Cross::Stretch,
                    )
                })
                .map(|w| (w - ins.start.unwrap_or(0.0) - ins.end.unwrap_or(0.0)).max(0.0));
            let kh_ = ih
                .filter(|_| fills(k.layout.used.height, ins.top, ins.bottom, false))
                .map(|h| (h - ins.top.unwrap_or(0.0) - ins.bottom.unwrap_or(0.0)).max(0.0));
            let out = self.compute_child_layout(
                id(c),
                tf::LayoutInput {
                    run_mode: if input.run_mode == tf::RunMode::PerformLayout {
                        tf::RunMode::PerformLayout
                    } else {
                        tf::RunMode::ComputeSize
                    },
                    sizing_mode: tf::SizingMode::InherentSize,
                    axis: tf::RequestedAxis::Both,
                    known_dimensions: tf::Size {
                        width: kw_,
                        height: kh_,
                    },
                    known_dimensions_are_definite: tf::Size {
                        width: true,
                        height: true,
                    },
                    parent_size: tf::Size {
                        width: iw,
                        height: ih,
                    },
                    available_space: avail,
                    vertical_margins_are_collapsible: tf::Line::FALSE,
                },
            );
            cw = cw.max(out.size.width);
            ch = ch.max(out.size.height);
            sizes.push(out.size);
        }
        let el = self.nodes[i].layout.used;
        let w = kw.unwrap_or_else(|| {
            self.nodes[i]
                .layout
                .used
                .bounded(0, parent.width, cw, cw, cw + pad.horizontal())
        });
        let h = kh.unwrap_or_else(|| {
            self.nodes[i]
                .layout
                .used
                .bounded(1, parent.height, ch, ch, ch + pad.vertical())
        });
        if input.run_mode == tf::RunMode::PerformLayout {
            let bx = Rect::new(
                pad.left,
                pad.top,
                (w - pad.horizontal()).max(0.0),
                (h - pad.vertical()).max(0.0),
            );
            for (k, &c) in kids.iter().enumerate() {
                let s = sizes[k];
                let ins = flow_insets(&self.nodes[c]);
                let (x, y) = place_in(&bx, s.width, s.height, &ins, el.align, el.cross);
                let mut l = tf::Layout::with_order(k as u32);
                l.location = tf::Point { x, y };
                l.size = s;
                self.set_unrounded_layout(id(c), &l);
            }
            self.nodes[i].layout.content_w = cw;
        }
        let mut out = tf::LayoutOutput::from_outer_size(tf::Size {
            width: w,
            height: h,
        });
        out.scrollable_overflow_rect = tf::Rect {
            left: 0.0,
            top: 0.0,
            right: cw + pad.horizontal(),
            bottom: ch + pad.vertical(),
        };
        out
    }

    /// The subgrids among grid `i`'s items take the sizes of the tracks
    /// their area spans, as fixed tracks, and lay out again in it.
    fn subgrids(&mut self, i: usize) {
        let kids = self.nodes[i].children.clone();
        for (k, &c) in kids.iter().enumerate() {
            let sub = [0, 1].map(|axis| grid::subgrid(self.nodes, c, axis));
            if !sub[0] && !sub[1] {
                continue;
            }
            let Some(area) = self.nodes[i].layout.grid_items.get(k).copied() else {
                continue;
            };
            let tracks = |axis: usize| -> Option<Vec<f32>> {
                if !sub[axis] {
                    return None;
                }
                let at = &self.nodes[i].layout.grid_tracks[axis];
                let (s, e) = (area[axis * 2] as usize, area[axis * 2 + 1] as usize);
                let r = s.saturating_sub(1)..e.saturating_sub(1).min(at.len());
                Some(at.get(r)?.iter().map(|t| (t[1] - t[0]).max(0.0)).collect())
            };
            let (cols, rows) = (tracks(0), tracks(1));
            let s = grid::style(self.nodes, c, [cols.as_deref(), rows.as_deref()]);
            self.grids.insert(c, s);
            let Some(input) = self.nodes[c].layout.css_input else {
                continue;
            };
            self.nodes[c].layout.css_cache.clear();
            self.lay(c, input);
        }
    }
}

fn flip_size<T>(s: tf::Size<T>) -> tf::Size<T> {
    tf::Size {
        width: s.height,
        height: s.width,
    }
}

/// A layout input turned across (an orthogonal flow's).
fn flip_input(mut input: tf::LayoutInput) -> tf::LayoutInput {
    input.known_dimensions = flip_size(input.known_dimensions);
    input.known_dimensions_are_definite = flip_size(input.known_dimensions_are_definite);
    input.parent_size = flip_size(input.parent_size);
    input.available_space = flip_size(input.available_space);
    input.axis = match input.axis {
        tf::RequestedAxis::Horizontal => tf::RequestedAxis::Vertical,
        tf::RequestedAxis::Vertical => tf::RequestedAxis::Horizontal,
        tf::RequestedAxis::Both => tf::RequestedAxis::Both,
    };
    input.vertical_margins_are_collapsible = tf::Line::FALSE;
    input
}

/// An orthogonal flow's output turned back: its size; no baseline and no
/// margin collapses through it.
fn flip_output(out: tf::LayoutOutput) -> tf::LayoutOutput {
    let r = out.scrollable_overflow_rect;
    let mut o = tf::LayoutOutput::from_outer_size(flip_size(out.size));
    o.scrollable_overflow_rect = tf::Rect {
        left: r.top,
        right: r.bottom,
        top: r.left,
        bottom: r.right,
    };
    o.oof_candidates = out.oof_candidates;
    o
}

/// The insets a child is placed by in a stack: none for a relative or
/// sticky one, which moves after.
fn flow_insets(n: &Node) -> crate::element::Edges {
    use crate::element::Position;
    match n.props.position {
        Position::Relative | Position::Sticky => crate::element::Edges::default(),
        _ => n.layout.insets,
    }
}

/// Where a `w`×`h` box sits in `bx`: by its insets where they are set,
/// else by `align` across and `cross` down.
pub(super) fn place_in(
    bx: &Rect,
    w: f32,
    h: f32,
    ins: &crate::element::Edges,
    align: Align,
    cross: Cross,
) -> (f32, f32) {
    let x = if let Some(f) = ins.across {
        bx.x + ((bx.width - w) * f).round() + ins.start.unwrap_or(0.0)
    } else if ins.center_x {
        bx.x + (bx.width - w) / 2.0 + ins.start.unwrap_or(0.0)
    } else if let Some(s) = ins.start {
        bx.x + s
    } else if let Some(e) = ins.end {
        bx.right() - e - w
    } else {
        match align {
            Align::Center => bx.x + (bx.width - w) / 2.0,
            Align::End => bx.right() - w,
            _ => bx.x,
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
            Cross::Center => bx.y + (bx.height - h) / 2.0,
            Cross::End => bx.bottom() - h,
            _ => bx.y,
        }
    };
    (x, y)
}

mod text {
    //! A text leaf's content size and lines.
    use super::super::*;
    use super::tf;

    /// The lines a text leaf broke at the width it was measured at.
    pub(crate) enum Lines {
        Text {
            lines: Vec<crate::text::Line>,
            step: Option<TypeStep>,
        },
        Rich(Vec<crate::rich::RichLine>),
    }

    impl Lines {
        pub fn store(self, n: &mut Node) {
            n.layout.wrap_lines.clear();
            match self {
                Lines::Text { lines, step } => {
                    n.layout.lines = lines;
                    n.layout.fit_step = step;
                }
                Lines::Rich(lines) => n.layout.rich_lines = lines,
            }
        }
    }

    /// Text node `i`'s content box at `known` within `avail` (CSS
    /// `fit-content` across: its widest line, no wider than the room, no
    /// narrower than its longest word), with the lines it breaks into.
    pub(crate) fn content(
        nodes: &[Node],
        i: usize,
        measure: &mut Measure<'_>,
        known: tf::Size<Option<f32>>,
        avail: tf::Size<tf::AvailableSpace>,
    ) -> (tf::Size<f32>, Option<Lines>) {
        let n = &nodes[i];
        let fonts = measure.fonts;
        let fit = |pref: f32, min: f32| {
            known.width.unwrap_or(match avail.width {
                tf::AvailableSpace::Definite(a) => pref.min(a.max(min)),
                tf::AvailableSpace::MinContent => min,
                tf::AvailableSpace::MaxContent => pref,
            })
        };
        let size = |w: f32, h: f32| tf::Size {
            width: w,
            height: known.height.unwrap_or(h),
        };
        let contained = [n.contained_size(0), n.contained_size(1)];
        if let [Some(w), Some(h)] = contained {
            return (size(w, h), None);
        }
        match &*n.content {
            Content::Text(_) if let Some(tp) = &n.props.text_path => {
                let e = tp.extent().unwrap_or_default();
                (size(e.x, e.y), None)
            }
            Content::Text(t) => {
                let t = n.used_text(t);
                let (pref, min) = measure.texts.widths(t, || text_widths(t, fonts));
                let min = match t.fit.last() {
                    Some(&small) => {
                        let s = t.at(small);
                        min.min(measure.texts.widths(s, || text_widths(s, fonts)).1)
                    }
                    None => min,
                };
                let w = fit(pref, min);
                // An auto-fit label: the largest of its steps its text fits at.
                let step = (!t.fit.is_empty()).then(|| {
                    t.fit
                        .iter()
                        .copied()
                        .find(|&s| fits(t.at(s), fonts, w + 0.5))
                        .unwrap_or(t.fit[t.fit.len() - 1])
                });
                let t = step.map_or(t, |s| t.at(s));
                // A fit width snapped to the device pixel may land up to half
                // a pixel under the text it was measured from.
                let width = w + 0.5;
                let lines = measure
                    .texts
                    .lines(t, (t.wrap || t.ellipsis).then_some(width), || {
                        super::super::text::text_lines_at(t, fonts, width)
                    });
                let h = if t.text.is_empty() && n.layout.used.block {
                    0.0
                } else {
                    lines.len().max(1) as f32 * t.line_height()
                };
                (size(w, h), Some(Lines::Text { lines, step }))
            }
            Content::Rich(rc) => {
                let v = n.rich_vertical();
                let (pref, min) = crate::rich::rich_widths(rc, fonts, v);
                let w = fit(pref, min);
                let lines = crate::rich::rich_lines(rc, fonts, rc.wrap.then_some(w + 0.5), v);
                let h = lines.iter().map(|l| l.height).sum();
                (size(w, h), Some(Lines::Rich(lines)))
            }
            Content::Image { .. } | Content::None => (
                size(contained[0].unwrap_or(0.0), contained[1].unwrap_or(0.0)),
                None,
            ),
        }
    }

    /// A text's first baseline below its content box's top.
    pub(crate) fn first_baseline(n: &Node, l: &Lines, fonts: &dyn Metrics) -> Option<f32> {
        match (l, &*n.content) {
            (Lines::Text { lines, step }, Content::Text(t)) => {
                let t = n.used_text(t);
                let t = step.map_or(t, |s| t.at(s));
                let line = lines.first()?;
                let lh = t.line_height();
                Some(t.metrics(fonts).baseline(&line.text, 0.0, lh, t.step.size))
            }
            (Lines::Rich(lines), _) => Some(lines.first()?.baseline),
            _ => None,
        }
    }
}

impl TraversePartialTree for Tree<'_, '_> {
    type ChildIter<'b>
        = Kids<'b>
    where
        Self: 'b;

    fn child_ids(&self, parent: tf::NodeId) -> Kids<'_> {
        let p = usize::from(parent);
        match self.line {
            Some((l, s, e)) if l == p => Kids::Plain(self.nodes[p].children[s..e].iter()),
            _ => kids(self.nodes, p),
        }
    }

    fn child_count(&self, parent: tf::NodeId) -> usize {
        self.child_ids(parent).count()
    }

    fn get_child_id(&self, parent: tf::NodeId, index: usize) -> tf::NodeId {
        self.child_ids(parent)
            .nth(index)
            .expect("a child within the count")
    }
}

impl LayoutPartialTree for Tree<'_, '_> {
    type CoreContainerStyle<'b>
        = View<'b>
    where
        Self: 'b;
    type CustomIdent = String;

    fn get_core_container_style(&self, node: tf::NodeId) -> View<'_> {
        self.container(usize::from(node))
    }

    fn resolve_calc_value(&self, val: *const (), basis: f32) -> f32 {
        calc_of(self.nodes, val, basis)
    }

    fn set_unrounded_layout(&mut self, node: tf::NodeId, layout: &tf::Layout) {
        // Its scrollable overflow is its own, in its own frame, as it last
        // laid out ([`Tree::uncached`]): a parent answered from its cache
        // passes on what it saw then.
        let css = &mut self.nodes[usize::from(node)].layout.css;
        let own = css.scrollable_overflow_rect;
        *css = *layout;
        css.scrollable_overflow_rect = own;
    }

    fn compute_child_layout(
        &mut self,
        node: tf::NodeId,
        input: tf::LayoutInput,
    ) -> tf::LayoutOutput {
        self.lay_in(node, input, None)
    }
}

impl CacheTree for Tree<'_, '_> {
    fn cache_get(&mut self, node: tf::NodeId, input: &tf::LayoutInput) -> Option<tf::LayoutOutput> {
        self.nodes[usize::from(node)].layout.css_cache.get(input)
    }

    fn cache_store(&mut self, node: tf::NodeId, input: &tf::LayoutInput, out: tf::LayoutOutput) {
        self.nodes[usize::from(node)]
            .layout
            .css_cache
            .store(input, out);
    }

    fn cache_clear(&mut self, node: tf::NodeId) {
        self.nodes[usize::from(node)].layout.css_cache.clear();
    }
}

impl LayoutFlexboxContainer for Tree<'_, '_> {
    type FlexboxContainerStyle<'b>
        = View<'b>
    where
        Self: 'b;
    type FlexboxItemStyle<'b>
        = View<'b>
    where
        Self: 'b;

    fn get_flexbox_container_style(&self, node: tf::NodeId) -> View<'_> {
        self.container(usize::from(node))
    }

    fn get_flexbox_child_style(&self, child: tf::NodeId) -> View<'_> {
        let c = usize::from(child);
        let mut v = View::item(self.nodes, c);
        v.measured = self.intrinsic == self.nodes[c].up;
        v
    }
}

impl LayoutGridContainer for Tree<'_, '_> {
    type GridContainerStyle<'b>
        = GridView<'b>
    where
        Self: 'b;
    type GridItemStyle<'b>
        = View<'b>
    where
        Self: 'b;

    fn get_grid_container_style(&self, node: tf::NodeId) -> GridView<'_> {
        let i = usize::from(node);
        GridView {
            view: View::new(self.nodes, i),
            grid: self.grids.get(&i).unwrap_or(&self.empty),
        }
    }

    fn get_grid_child_style(&self, child: tf::NodeId) -> View<'_> {
        View::item(self.nodes, usize::from(child))
    }

    fn set_detailed_grid_info(&mut self, node: tf::NodeId, info: tf::DetailedGridInfo<String>) {
        let n = &mut self.nodes[usize::from(node)].layout;
        let at = |t: &tf::DetailedGridTracksInfo<String>| {
            t.positions
                .iter()
                .map(|l| [l.start, l.end])
                .collect::<Vec<_>>()
        };
        n.grid_tracks = [at(&info.columns), at(&info.rows)];
        n.grid_items = info
            .items
            .iter()
            .map(|a| [a.column_start, a.column_end, a.row_start, a.row_end])
            .collect();
    }
}

impl LayoutBlockContainer for Tree<'_, '_> {
    type BlockContainerStyle<'b>
        = View<'b>
    where
        Self: 'b;
    type BlockItemStyle<'b>
        = View<'b>
    where
        Self: 'b;

    fn get_block_container_style(&self, node: tf::NodeId) -> View<'_> {
        View::new(self.nodes, usize::from(node))
    }

    fn get_block_child_style(&self, child: tf::NodeId) -> View<'_> {
        View::item(self.nodes, usize::from(child))
    }

    fn compute_block_child_layout(
        &mut self,
        node: tf::NodeId,
        input: tf::LayoutInput,
        ctx: Option<&mut tf::BlockContext<'_>>,
    ) -> tf::LayoutOutput {
        self.lay_in(node, input, ctx)
    }
}
