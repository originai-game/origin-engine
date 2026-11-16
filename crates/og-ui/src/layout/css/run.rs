//! One layout call: the subtree under `root` laid out by Taffy inside an
//! area, then placed. Taffy keeps each box's size and its place in its
//! layout parent's frame; the place pass turns those into rectangles on
//! the surface: scrolled, mirrored for a right-to-left box, turned back
//! out of a vertical frame, moved by `position: relative` and `sticky`,
//! split into columns, and snapped to device pixels.

use super::style::{Sides, frame_of, turns};
use super::*;
use crate::element::Position;

fn input(
    run: tf::RunMode,
    known: [Option<f32>; 2],
    parent: [Option<f32>; 2],
    avail: [tf::AvailableSpace; 2],
) -> tf::LayoutInput {
    tf::LayoutInput {
        run_mode: run,
        sizing_mode: tf::SizingMode::InherentSize,
        axis: tf::RequestedAxis::Both,
        known_dimensions: tf::Size {
            width: known[0],
            height: known[1],
        },
        known_dimensions_are_definite: tf::Size {
            width: true,
            height: true,
        },
        parent_size: tf::Size {
            width: parent[0],
            height: parent[1],
        },
        available_space: tf::Size {
            width: avail[0],
            height: avail[1],
        },
        vertical_margins_are_collapsible: tf::Line::FALSE,
    }
}

impl Tree<'_, '_> {
    /// Node `c`'s size laid out in layout space (physical axes), whatever
    /// frame the box it lays out in has.
    pub(super) fn lay_physical(
        &mut self,
        c: usize,
        run: tf::RunMode,
        known: [Option<f32>; 2],
        parent: [Option<f32>; 2],
        avail: [tf::AvailableSpace; 2],
    ) -> tf::Size<f32> {
        let across = turns(frame_of(self.nodes, self.nodes[c].up));
        fn sw<T: Copy>(v: [T; 2], across: bool) -> [T; 2] {
            if across { [v[1], v[0]] } else { v }
        }
        let out = self.lay(
            c,
            input(
                run,
                sw(known, across),
                sw(parent, across),
                sw(avail, across),
            ),
        );
        if across {
            tf::Size {
                width: out.size.height,
                height: out.size.width,
            }
        } else {
            out.size
        }
    }

    /// Node `c`'s preferred and least widths and its height at each,
    /// stored for anchor positioning and the screen placement.
    pub(super) fn intrinsic(&mut self, c: usize, parent: [Option<f32>; 2]) -> [f32; 4] {
        use tf::AvailableSpace as A;
        let max = self.lay_physical(
            c,
            tf::RunMode::ComputeSize,
            [None, None],
            parent,
            [A::MaxContent, A::MaxContent],
        );
        let min = self.lay_physical(
            c,
            tf::RunMode::ComputeSize,
            [None, None],
            parent,
            [A::MinContent, A::MaxContent],
        );
        let l = &mut self.nodes[c].layout;
        l.pref_w = max.width;
        l.min_w = min.width.min(max.width);
        l.pref_h = max.height;
        l.min_h = min.height;
        l.min_content = [l.min_w, l.min_h];
        [l.pref_w, l.min_w, l.pref_h, l.min_h]
    }

    /// Lays out node `i`'s out-of-flow children against its border box,
    /// `size` (layout space): each sized as CSS sizes an absolutely
    /// positioned box and placed by its insets, its place kept as its
    /// layout from `i`'s top left in layout space.
    pub(super) fn out_of_flow(&mut self, i: usize, size: [f32; 2]) {
        let out = self.nodes[i].out.clone();
        if out.is_empty() {
            return;
        }
        let bx = Rect::new(0.0, 0.0, size[0], size[1]);
        for c in out {
            if self.nodes[c].is_hidden() {
                continue;
            }
            self.intrinsic(c, [Some(size[0]), Some(size[1])]);
            let n = &self.nodes[c];
            let w = out_size(n, 0, size[0], size[0]);
            let h0 = out_size(n, 1, size[1], size[0]);
            // Its height is its content's at its width unless set.
            let h_set = !auto_height(n);
            let got = self.lay_physical(
                c,
                tf::RunMode::PerformLayout,
                [Some(w), h_set.then_some(h0)],
                [Some(size[0]), Some(size[1])],
                [
                    tf::AvailableSpace::Definite(w),
                    tf::AvailableSpace::Definite(size[1]),
                ],
            );
            let h = clamp_to(&self.nodes[c], 1, Some(size[1]), got.height);
            let n = &self.nodes[c];
            let (x, y) = if let Some(v) = n.props.view_place {
                // Its anchor on the point the view box shows `at` at
                // (meet, centred), scaled about it by the fit when drawn.
                let [vx, vy, vw, vh] = v.view_box;
                let k = (bx.width / vw.max(1e-6)).min(bx.height / vh.max(1e-6));
                let off = (v.at[0] - (vx + vw / 2.0)) * k;
                let x = bx.width / 2.0 + off - v.anchor_x * w;
                let y = bx.height / 2.0 + (v.at[1] - (vy + vh / 2.0)) * k - v.anchor_y;
                self.nodes[c].layout.fit = [k, v.anchor_x, v.anchor_y, off];
                (x, y)
            } else {
                let (mut x, mut y) = place_out(&bx, n, w, h);
                let ins = &n.layout.insets;
                let free_x = ins.start.is_none()
                    && ins.end.is_none()
                    && ins.across.is_none()
                    && !ins.center_x;
                let free_y = ins.top.is_none() && ins.bottom.is_none() && !ins.center_y;
                if free_x || free_y {
                    let (sx, sy) = static_position(self.nodes, i, c, w, h, size);
                    if free_x {
                        x = sx;
                    }
                    if free_y {
                        y = sy;
                    }
                }
                (x, y)
            };
            let m = self.nodes[c].layout.used.margin.map(|m| m.of(size[0]));
            let mut l = tf::Layout::with_order(0);
            l.location = tf::Point { x, y };
            l.size = tf::Size {
                width: w,
                height: h,
            };
            l.margin = tf::Rect {
                top: m[0],
                right: m[1],
                bottom: m[2],
                left: m[3],
            };
            l.scrollable_overflow_rect = self.nodes[c].layout.css.scrollable_overflow_rect;
            self.nodes[c].layout.css = l;
        }
    }

    /// A multi-column container `i`: its children stacked in one column
    /// of the used column width (a spanning one at the whole width), its
    /// height the balanced columns' ([`super::super::multicol`]); the place
    /// pass moves them into their columns.
    pub(super) fn multicol(&mut self, i: usize, input: tf::LayoutInput) -> tf::LayoutOutput {
        let el = self.nodes[i].layout.used;
        let m = el.multicol.unwrap_or_default();
        let pad = el.padding;
        let parent = [input.parent_size.width, input.parent_size.height];
        let kids = self.nodes[i].children.clone();
        let w = match input.known_dimensions.width {
            Some(w) => w,
            None => {
                // Its intrinsic widths from its children's.
                let (mut pref, mut min) = (0.0f32, 0.0f32);
                for &c in &kids {
                    let [p, l, _, _] = self.intrinsic(c, [None, None]);
                    pref = pref.max(p);
                    min = min.max(l);
                }
                let (p, l) = multicol::intrinsic(&m, pref, min);
                let (p, l) = (p + pad.horizontal(), l + pad.horizontal());
                let w = match input.available_space.width {
                    tf::AvailableSpace::Definite(a) => match el.width {
                        Size::Fit { .. } => p.min(a).max(l),
                        s => s.definite(p, l, parent[0]).unwrap_or(a),
                    },
                    tf::AvailableSpace::MinContent => l,
                    tf::AvailableSpace::MaxContent => p,
                };
                el.bounded(0, parent[0], p, l, w)
            }
        };
        let inner = (w - pad.horizontal()).max(0.0);
        let cols = multicol::used(&m, inner);
        self.nodes[i].layout.cols = cols;
        let perform = input.run_mode == tf::RunMode::PerformLayout;
        let run = if perform {
            tf::RunMode::PerformLayout
        } else {
            tf::RunMode::ComputeSize
        };
        let (mut tops, mut hs) = (Vec::new(), Vec::new());
        let mut y = 0.0f32;
        let mut prev = 0.0f32;
        for (k, &c) in kids.iter().enumerate() {
            let cw = if multicol::in_column(&self.nodes[c]) {
                cols.1
            } else {
                inner
            };
            let mg = self.nodes[c].layout.used.margin.map(|g| g.of(inner));
            let room = (cw - mg[1] - mg[3]).max(0.0);
            let fills = !matches!(
                self.nodes[c].layout.used.width,
                Size::Fixed(_) | Size::Percent(_) | Size::Calc(_) | Size::CalcSize { .. }
            );
            let size = self.lay_physical(
                c,
                run,
                [fills.then_some(room), None],
                [Some(cw), None],
                [
                    tf::AvailableSpace::Definite(room),
                    tf::AvailableSpace::MaxContent,
                ],
            );
            // Adjoining vertical margins collapse (CSS 2 §8.3.1).
            y += if k == 0 {
                mg[0]
            } else {
                mg[0].max(prev) - prev
            };
            tops.push(y);
            hs.push(size.height);
            if perform {
                let mut l = tf::Layout::with_order(k as u32);
                l.location = tf::Point {
                    x: pad.left + mg[3],
                    y: pad.top + y,
                };
                l.size = size;
                l.margin = tf::Rect {
                    left: mg[3],
                    right: mg[1],
                    top: mg[0],
                    bottom: mg[2],
                };
                l.scrollable_overflow_rect = self.nodes[c].layout.css.scrollable_overflow_rect;
                self.nodes[c].layout.css = l;
            }
            y += size.height + mg[2];
            prev = mg[2];
        }
        let content = multicol::balanced_height(self.nodes, i, &tops, &hs);
        let h = input.known_dimensions.height.unwrap_or_else(|| {
            let ch = content + pad.vertical();
            el.height
                .definite(ch, ch, parent[1])
                .unwrap_or(ch)
                .max(pad.vertical())
        });
        let h = el.bounded(1, parent[1], h, h, h);
        let mut out = tf::LayoutOutput::from_outer_size(tf::Size {
            width: w,
            height: h,
        });
        out.scrollable_overflow_rect = tf::Rect {
            left: 0.0,
            top: 0.0,
            right: w - pad.right,
            bottom: pad.top + content,
        };
        out
    }

    /// A masonry grid `i` (rows `masonry`): its items, laid out on the
    /// grid's columns, packed down them, each where its columns are
    /// shortest; its height what they reach.
    pub(super) fn masonry(&mut self, i: usize, mut out: tf::LayoutOutput) -> tf::LayoutOutput {
        let n = &self.nodes[i];
        let el = n.layout.used;
        let gap = n.grid().row_gap.or(el.gaps[1]).unwrap_or(el.gap);
        let ncols = n.layout.grid_tracks[0].len().max(1);
        let kids = n.children.clone();
        let mut bottom = vec![0.0f32; ncols];
        for (k, &c) in kids.iter().enumerate() {
            let Some(a) = self.nodes[i].layout.grid_items.get(k).copied() else {
                continue;
            };
            let mut s = (a[0] as usize).saturating_sub(1).min(ncols - 1);
            let span = (a[1] as usize).saturating_sub(1).clamp(s + 1, ncols) - s;
            let reach = |s: usize| bottom[s..s + span].iter().copied().fold(0.0, f32::max);
            // An auto-placed item goes to the tracks that end highest, the
            // first of them (CSS Grid 3 §2.4); a placed one keeps its own.
            let from = s;
            if super::grid::placement(self.nodes, c, 0).start == tf::GridPlacement::Auto {
                s = (0..=ncols - span)
                    .min_by(|&a, &b| reach(a).total_cmp(&reach(b)))
                    .unwrap_or(s);
            }
            let top = reach(s);
            let tracks = &self.nodes[i].layout.grid_tracks[0];
            let shift = match (tracks.get(s), tracks.get(from)) {
                (Some(to), Some(at)) => to[0] - at[0],
                _ => 0.0,
            };
            let l = &mut self.nodes[c].layout.css;
            l.location.x += shift;
            l.location.y = el.padding.top + top;
            for b in &mut bottom[s..s + span] {
                *b = top + l.size.height + gap;
            }
        }
        let content = (bottom.iter().copied().fold(0.0, f32::max) - gap).max(0.0);
        if matches!(
            el.height,
            Size::Fit { .. } | Size::Grow { .. } | Size::Stretch { .. }
        ) {
            out.size.height = out
                .size
                .height
                .min(content + el.padding.vertical())
                .max(content + el.padding.vertical());
        }
        out
    }
}

/// An out-of-flow element's size along `axis` of its containing block
/// `outer` long, its margins' percentages against `basis` (CSS 2.1
/// §10.3.7, §10.6.4): a definite size as written; `auto` between two set
/// insets fills the room between them less its margins, and otherwise
/// shrinks to fit that room; `stretch` fills it either way.
pub(super) fn out_size(n: &Node, axis: usize, outer: f32, basis: f32) -> f32 {
    let ins = &n.layout.insets;
    let (size, pref, min, a, b, [lead, trail]) = if axis == 0 {
        let l = &n.layout;
        (l.used.width, l.pref_w, l.min_w, ins.start, ins.end, [3, 1])
    } else {
        let l = &n.layout;
        (
            l.used.height,
            l.pref_h,
            l.min_h,
            ins.top,
            ins.bottom,
            [0, 2],
        )
    };
    let margins = n.layout.used.margin[lead].of(basis) + n.layout.used.margin[trail].of(basis);
    let room = (outer - a.unwrap_or(0.0) - b.unwrap_or(0.0) - margins).max(0.0);
    let (lo, hi) = match size {
        Size::Fit { min, max } | Size::Grow { min, max } | Size::Stretch { min, max } => {
            (min, max.max(min))
        }
        _ => (0.0, f32::INFINITY),
    };
    let v = match (a, b, size) {
        (_, _, Size::Fixed(v)) => v,
        (_, _, Size::Percent(p)) => p * outer,
        (_, _, Size::Calc(c)) => c.of(outer),
        (Some(_), Some(_), Size::Fit { .. }) | (_, _, Size::Grow { .. } | Size::Stretch { .. }) => {
            room.clamp(lo, hi)
        }
        (_, _, Size::Fit { .. }) => pref.min(room).max(min).clamp(lo, hi),
        (_, _, Size::MinContent) => min,
        (_, _, Size::MaxContent) => pref,
        (
            a,
            b,
            Size::CalcSize {
                basis: k,
                scale,
                add,
            },
        ) => scale * k.of(pref, min, room, a.is_some() && b.is_some()) + add.of(outer),
    };
    clamp_to(n, axis, Some(outer), v)
}

/// Node `n`'s size along `axis` held to its bounds against `basis`.
pub(super) fn clamp_to(n: &Node, axis: usize, basis: Option<f32>, v: f32) -> f32 {
    let (pref, min) = if axis == 0 {
        (n.layout.pref_w, n.layout.min_w)
    } else {
        (n.layout.pref_h, n.layout.min_h)
    };
    n.layout.used.bounded(axis, basis, pref, min, v)
}

/// Where an out-of-flow `w`×`h` element stands in its containing block
/// `bx` (CSS 2.1 §10.3.7, §10.6.4, CSS Position 3 §4): its margin edge on
/// each set inset; between two, `auto` margins share what is left.
fn place_out(bx: &Rect, n: &Node, w: f32, h: f32) -> (f32, f32) {
    let ins = &n.layout.insets;
    let (x, y) = place_in(bx, w, h, ins, Align::Start, Cross::Start);
    let m = &n.layout.used.margin;
    let at = |lo: f32, len: f32, size: f32, a: Option<f32>, b: Option<f32>, edges: [usize; 2]| {
        let [lead, trail] = edges.map(|e| m[e]);
        let (ml, mr) = (lead.of(bx.width), trail.of(bx.width));
        match (a, b) {
            (Some(a), Some(b)) => {
                let free = len - a - b - size - ml - mr;
                lo + a
                    + ml
                    + match (lead.auto, trail.auto) {
                        (true, true) if edges[0] == 3 => (free / 2.0).max(0.0),
                        (true, true) => free / 2.0,
                        (true, false) => free,
                        _ => 0.0,
                    }
            }
            (None, Some(b)) => lo + len - b - mr - size,
            (a, None) => lo + a.unwrap_or(0.0) + ml,
        }
    };
    let x = if ins.across.is_some() || ins.center_x {
        x
    } else {
        at(bx.x, bx.width, w, ins.start, ins.end, [3, 1])
    };
    let y = if ins.center_y {
        y
    } else {
        at(bx.y, bx.height, h, ins.top, ins.bottom, [0, 2])
    };
    (x, y)
}

/// The static position of out-of-flow child `c` of `i`, `w`×`h`, from
/// `i`'s border box `size` (CSS 2.1 §10.3.7, §10.6.4): in a block after
/// the box before it in the flow; in a flex container where the only item
/// would stand (CSS Flexbox §4.1); in a stack by its alignment; else at
/// the content box's start.
fn static_position(
    nodes: &[Node],
    i: usize,
    c: usize,
    w: f32,
    h: f32,
    size: [f32; 2],
) -> (f32, f32) {
    let p = &nodes[i];
    let el = p.layout.used;
    let pad = el.padding;
    let [mt, mr, mb, ml] = nodes[c].layout.used.margin.map(|m| m.of(size[0]));
    let inner = Rect::new(
        pad.left,
        pad.top,
        (size[0] - pad.horizontal()).max(0.0),
        (size[1] - pad.vertical()).max(0.0),
    );
    let (room_w, room_h) = (inner.width - ml - mr, inner.height - mt - mb);
    let offset = |a: Align, free: f32| match a {
        Align::Center => free / 2.0,
        Align::End => free,
        _ => 0.0,
    };
    let cross = |c: Cross, free: f32| match c {
        Cross::Center => free / 2.0,
        Cross::End => free,
        _ => 0.0,
    };
    let item = nodes[c].layout.used.align_self.unwrap_or(el.cross);
    let (x, y) = match kind(nodes, i) {
        Kind::Stack => place_in(&inner, w, h, &Default::default(), el.align, el.cross),
        Kind::Flex if el.direction == Direction::Row => (
            inner.x + ml + offset(el.align, room_w - w),
            inner.y + mt + cross(item, room_h - h),
        ),
        Kind::Flex => (
            inner.x + ml + cross(item, room_w - w),
            inner.y + mt + offset(el.align, room_h - h),
        ),
        Kind::Block | Kind::Lines | Kind::Multicol => {
            let siblings = &nodes[c].parent;
            let order = &nodes[*siblings].ordered_children;
            let before = order
                .iter()
                .position(|&k| k == c)
                .and_then(|at| order[..at].iter().rfind(|k| p.children.contains(k)));
            let y = before.map_or(inner.y + mt, |&b| {
                let l = &nodes[b].layout.css;
                l.location.y + l.size.height + l.margin.bottom + mt
            });
            (inner.x + ml, y)
        }
        _ => (inner.x + ml, inner.y + mt),
    };
    // A right-to-left parent's mirror (`mirror`) turns it to the right.
    let _ = (mb, mr);
    (x, y)
}

/// Whether out-of-flow node `n`'s height is its content's at its width
/// (CSS 2.1 §10.6.7): `auto`, and not between a top and a bottom inset.
pub(crate) fn auto_height(n: &Node) -> bool {
    matches!(n.layout.used.height, Size::Fit { .. })
        && !(n.layout.insets.top.is_some() && n.layout.insets.bottom.is_some())
}

/// The box a document's absolute or fixed node is placed against when it
/// is not the one the main pass places it in (CSS Position 3 §2.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Holder {
    /// Its parent's box (an absolute node's), or the screen (a fixed
    /// node's): where the main pass and the screen pass place it already.
    Own,
    /// An ancestor's box.
    Node(usize),
}

/// Node `i`'s containing block, when it is not its own ([`Holder`]): an
/// absolute node's is its nearest ancestor that is positioned, transformed
/// or contained, else its tree's root (the initial containing block); a
/// fixed node's is its nearest transformed or contained ancestor (CSS
/// Transforms 1 §3, CSS Containment 2 §3.2, §3.4), else the screen.
pub(crate) fn containing_block(nodes: &[Node], i: usize) -> Holder {
    let n = &nodes[i];
    let fixed = match n.props.position {
        Position::Screen => true,
        Position::Absolute => false,
        _ => return Holder::Own,
    };
    if !n.props.css_position || n.props.anchor.is_some() || n.up == NONE {
        return Holder::Own;
    }
    // A stack overlays its children: it holds them as a positioned box does.
    let holds = |a: usize| {
        let m = &nodes[a];
        let c = m.layout.used.contain;
        m.props.holds_fixed
            || c.has(Contain::LAYOUT)
            || c.has(Contain::PAINT)
            || (!fixed && (m.props.position != Position::Flow || kind(nodes, a) == Kind::Stack))
    };
    let top = |a: usize| {
        let up = nodes[a].up;
        up == NONE || up >= nodes.len() || nodes[up].up == NONE || nodes[up].up >= nodes.len()
    };
    let mut a = n.up;
    loop {
        if holds(a) {
            break;
        }
        if top(a) {
            // The outermost box under the UI's root (its layer, inside the
            // safe area): an absolute node's initial containing block; a
            // fixed node stays on the screen.
            if fixed {
                return Holder::Own;
            }
            break;
        }
        a = nodes[a].up;
    }
    if !fixed && a == n.up {
        Holder::Own
    } else {
        Holder::Node(a)
    }
}

/// Where an out-of-flow element placed on the screen ([`Position::Screen`])
/// stands: its size and place by its insets in the whole surface; an axis
/// with no inset keeps its static position. An [`auto_height`] is
/// `content`, its content's height laid out at the width this gives; its
/// height at its least width, which `out_size` would take, is no bound.
pub(crate) fn on_screen(
    nodes: &[Node],
    i: usize,
    screen: &Rect,
    _scroll: &dyn Fn(u64) -> (f32, f32),
    content: Option<f32>,
) -> Rect {
    let n = &nodes[i];
    let w = out_size(n, 0, screen.width, screen.width);
    let h = match content {
        Some(h) if auto_height(n) => clamp_to(n, 1, Some(screen.height), h),
        _ => out_size(n, 1, screen.height, screen.width),
    };
    let (mut x, mut y) = place_out(screen, n, w, h);
    let ins = &n.layout.insets;
    let free_x = ins.start.is_none() && ins.end.is_none() && ins.across.is_none() && !ins.center_x;
    let free_y = ins.top.is_none() && ins.bottom.is_none() && !ins.center_y;
    if (free_x || free_y) && n.up != NONE {
        let p = &nodes[n.up];
        let (sx, sy) = static_position(nodes, n.up, i, w, h, [p.layout.w, p.layout.h]);
        if free_x {
            x = p.layout.x + sx;
        }
        if free_y {
            y = p.layout.y + sy;
        }
    }
    Rect::new(x, y, w, h)
}

/// Where an interval `[at, at + len]` along a frame axis lies on the
/// layout space's axis it runs on: the axis (0 x, 1 y) and its start. The
/// axis runs from side `from` of a box `size` (x, y) large.
fn along(from: u8, at: f32, len: f32, size: [f32; 2]) -> (usize, f32) {
    match from {
        0 => (1, at),
        1 => (0, size[0] - at - len),
        2 => (1, size[1] - at - len),
        _ => (0, at),
    }
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
    queries: &mut crate::ui::settlement::ContainerQueries,
) {
    if let Some(roots) = &queries.query_roots {
        let mut at = root;
        while nodes[at].parent != NONE {
            at = nodes[at].parent;
        }
        if !roots.contains(&at) {
            return;
        }
    }
    // What changed lays out again: its kept sizes go.
    let order = kept.walk(nodes, root);
    let dirty: Vec<usize> = order
        .iter()
        .copied()
        .filter(|&i| !kept.is_clean(i))
        .collect();
    for &i in &dirty {
        nodes[i].layout.css_cache.clear();
    }
    let mut tree = Tree::new(nodes, measure);
    let parent = [Some(area.width), Some(area.height)];
    let avail = [
        tf::AvailableSpace::Definite(area.width),
        tf::AvailableSpace::Definite(area.height),
    ];
    // The root: a fitting width its content's between the area and its
    // least, a stretching one the area's.
    let [pref, min, _, _] = tree.intrinsic(root, parent);
    let el = tree.nodes[root].layout.used;
    let w = match el.width {
        Size::Fit { .. } => Some(pref.min(area.width).max(min)),
        Size::Grow { min: lo, max: hi } | Size::Stretch { min: lo, max: hi } => {
            Some(area.width.clamp(lo, hi.max(lo)))
        }
        _ => None,
    }
    .map(|w| el.bounded(0, Some(area.width), pref, min, w));
    let h = match el.height {
        Size::Grow { min: lo, max: hi } | Size::Stretch { min: lo, max: hi } => {
            Some(area.height.clamp(lo, hi.max(lo)))
        }
        _ => None,
    };
    let got = tree.lay_physical(root, tf::RunMode::PerformLayout, [w, h], parent, avail);
    {
        let n = &mut tree.nodes[root];
        let mut l = tf::Layout::with_order(0);
        l.size = got;
        l.scrollable_overflow_rect = n.layout.css.scrollable_overflow_rect;
        n.layout.css = l;
    }
    // A changed box under one that kept its size (a fixed box: the cache
    // keeps its parent clean) lays out again at the input it was last
    // given. A box in a block flow with floats is laid out again with the
    // whole flow they lie in. An out-of-flow one is sized and placed again
    // by its containing box.
    for &i in &dirty {
        if tree.performed(i) || i == root {
            continue;
        }
        let mut owner = tree.nodes[i].parent;
        while owner != NONE && !tree.nodes[owner].out.contains(&i) {
            owner = tree.nodes[owner].parent;
        }
        if owner != NONE {
            if tree.performed(owner) {
                continue;
            }
            let l = &tree.nodes[owner].layout;
            let size = [l.w, l.h];
            tree.out_of_flow(owner, size);
            continue;
        }
        // One written again keeps no input: its layout parent lays it out.
        let mut p = i;
        while p != NONE && tree.nodes[p].layout.css_input.is_none() {
            p = tree.nodes[p].up;
        }
        let p = tree.flow_of(p);
        if p == NONE || tree.performed(p) {
            continue;
        }
        if let Some(input) = tree.nodes[p].layout.css_input {
            tree.nodes[p].layout.css_cache.clear();
            tree.relay(p, input);
        }
    }
    kept.laid_out += tree.laid;
    let performed = std::mem::take(&mut tree.performed);
    let nodes = tree.nodes;
    nodes[root].layout.x = area.x;
    nodes[root].layout.y = area.y;
    place(nodes, root, scroll, dpr, clip, kept, queries, &performed);
}

/// The place pass from `root`: each box's rectangle from its parent's and
/// its own layout, and the boxes under it that moved, resized or laid out.
#[allow(clippy::too_many_arguments)]
fn place(
    nodes: &mut [Node],
    root: usize,
    scroll: &dyn Fn(u64) -> (f32, f32),
    dpr: f32,
    clip: Rect,
    kept: &mut LayoutCache,
    queries: &mut crate::ui::settlement::ContainerQueries,
    performed: &[bool],
) {
    let laid = |i: usize| performed.get(i).copied().unwrap_or(false);
    {
        let n = &mut nodes[root];
        let s = n.layout.css.size;
        (n.layout.w, n.layout.h) = (s.width, s.height);
    }
    nodes[root].placed.clip = clip;
    let mut stack = vec![root];
    while let Some(i) = stack.pop() {
        content_extent(nodes, i);
        let n = &nodes[i];
        let rect = Rect::new(n.layout.x, n.layout.y, n.layout.w, n.layout.h);
        let (parent_clip, src) = if n.up == NONE || i == root {
            (clip, NONE)
        } else {
            let p = &nodes[n.up];
            if p.layout.used.scroll || p.layout.used.scroll_x {
                (p.layout.laid_clip.intersect(&p.layout.laid_rect), n.up)
            } else {
                (p.layout.laid_clip, p.placed.clip_src)
            }
        };
        let open = laid(i)
            || kept.visits(i)
            || nodes[i].scrolls()
            || parent_clip != nodes[i].layout.laid_clip
            || src != nodes[i].placed.clip_src
            || [nodes[i].layout.x, nodes[i].layout.y] != nodes[i].layout.laid_xy
            || nodes[i].layout.layout_size != [nodes[i].layout.w, nodes[i].layout.h];
        nodes[i].placed.clip = parent_clip;
        nodes[i].layout.laid_clip = parent_clip;
        nodes[i].placed.clip_src = src;
        nodes[i].placed.rect = rect.snapped(dpr);
        nodes[i].layout.laid_rect = nodes[i].placed.rect;
        nodes[i].layout.laid_xy = [nodes[i].layout.x, nodes[i].layout.y];
        kept.lay(i);
        if nodes[i].layout.layout_size != [nodes[i].layout.w, nodes[i].layout.h] {
            kept.resized.push(i);
        }
        nodes[i].layout.layout_size = [nodes[i].layout.w, nodes[i].layout.h];
        nodes[i].layout.layout_offset = if nodes[i].up == NONE || i == root {
            [0.0; 2]
        } else {
            let p = &nodes[nodes[i].up];
            [
                nodes[i].layout.x - p.layout.x,
                nodes[i].layout.y - p.layout.y,
            ]
        };
        queries.measurements.observe(
            nodes[i].key,
            crate::ui::settlement::ContentBox {
                width: (nodes[i].layout.w - nodes[i].layout.used.padding.horizontal()).max(0.0),
                height: (nodes[i].layout.h - nodes[i].layout.used.padding.vertical()).max(0.0),
            },
        );
        if !open || nodes[i].layout.cv_skip {
            continue;
        }
        place_children(nodes, i, scroll);
        let kids: Vec<usize> = nodes[i].ordered_children.clone();
        stack.extend(kids.into_iter().rev());
    }
}

/// Node `i`'s content extent from Taffy's overflow, in layout space, for
/// scrolling: its content box's.
fn content_extent(nodes: &mut [Node], i: usize) {
    let n = &mut nodes[i];
    let r = n.layout.css.scrollable_overflow_rect;
    let (mut w, mut h) = (r.right, r.bottom);
    if turns(frame_of(nodes, i)) {
        std::mem::swap(&mut w, &mut h);
    }
    let n = &mut nodes[i];
    let pad = n.layout.used.padding;
    let end = |s: bool, p: f32| if s { p } else { 0.0 };
    n.layout.content_w = (w - pad.left - end(n.layout.used.scroll_x, pad.right)).max(0.0);
    n.layout.content_h = (h - pad.top - end(n.layout.used.scroll, pad.bottom)).max(0.0);
}

/// Node `i`'s children placed from its rectangle: their layouts in its
/// frame turned into layout space, its scroll offset, a right-to-left
/// mirror, columns, table parts, and relative and sticky moves.
fn place_children(nodes: &mut [Node], i: usize, scroll: &dyn Fn(u64) -> (f32, f32)) {
    let n = &nodes[i];
    let s: Sides = frame_of(nodes, i);
    let size = [n.layout.w, n.layout.h];
    let (sx, sy) = match (n.layout.used.scroll_x, n.layout.used.scroll) {
        (false, false) => (0.0, 0.0),
        (x, y) => {
            let (sx, sy) = scroll(n.key);
            (
                if x {
                    sx.clamp(0.0, n.max_scroll_x())
                } else {
                    0.0
                },
                if y {
                    sy.clamp(0.0, n.max_scroll())
                } else {
                    0.0
                },
            )
        }
    };
    let (x0, y0) = (n.layout.x, n.layout.y);
    let flow: Vec<usize> = n
        .children
        .iter()
        .chain(&n.floats)
        .copied()
        .filter(|&c| !nodes[c].props.out_of_flow())
        .collect();
    let out = n.out.clone();
    for &c in &flow {
        let l = nodes[c].layout.css;
        let (mut w, mut h) = (l.size.width, l.size.height);
        if turns(s) {
            std::mem::swap(&mut w, &mut h);
        }
        let mut xy = [0.0; 2];
        let (ax, px) = along(s[3], l.location.x, l.size.width, size);
        xy[ax] = px;
        let (ay, py) = along(s[0], l.location.y, l.size.height, size);
        xy[ay] = py;
        let k = &mut nodes[c].layout;
        (k.w, k.h) = (w, h);
        k.x = x0 + xy[0] - sx;
        k.y = y0 + xy[1] - sy;
        k.margin_x = [l.margin.left, l.margin.right];
    }
    // Out of the flow: placed against its box in layout space.
    for &c in &out {
        let l = nodes[c].layout.css;
        let k = &mut nodes[c].layout;
        (k.w, k.h) = (l.size.width, l.size.height);
        k.x = x0 + l.location.x;
        k.y = y0 + l.location.y;
        k.margin_x = [l.margin.left, l.margin.right];
    }
    if nodes[i].layout.used.multicol.is_some() {
        multicol::place(nodes, i);
    }
    if nodes[i].layout.table.is_some() {
        table_parts(nodes, i);
    }
    lines_of(nodes, i);
    if nodes[i].layout.used.flip {
        // Taffy lays a right-to-left box's flow out from the right
        // (`View::direction`); what og-ui places itself flips here: a
        // stack's and multi-column box's children, a vertical box's
        // block axis, and out-of-flow boxes at their static position.
        let own = matches!(kind(nodes, i), Kind::Stack | Kind::Multicol) || turns(s);
        mirror(nodes, i, sx, if own { &flow } else { &[] }, &out);
    }
    relative(nodes, i, &flow, scroll, (sx, sy));
}

/// A wrapping row's lines as Taffy broke them: ranges of `children`.
fn lines_of(nodes: &mut [Node], i: usize) {
    let n = &nodes[i];
    // Lines laid out one at a time keep the rows they were laid out in.
    if !n.layout.inline_rows.is_empty() {
        return;
    }
    if !n.layout.used.wrap || n.layout.used.direction != Direction::Row {
        nodes[i].layout.rows.clear();
        return;
    }
    let mut rows = Vec::new();
    let (mut start, mut bottom) = (0, f32::NEG_INFINITY);
    for (k, &c) in n.children.iter().enumerate() {
        let l = &nodes[c].layout.css;
        let (top, end) = (l.location.y, l.location.y + l.size.height);
        if k > start && top >= bottom - 0.5 {
            rows.push((start, k));
            start = k;
            bottom = end;
        } else {
            bottom = bottom.max(end);
        }
    }
    if !n.children.is_empty() {
        rows.push((start, n.children.len()));
    }
    nodes[i].layout.rows = rows;
}

/// A table's row, group and column boxes over their slots.
fn table_parts(nodes: &mut [Node], i: usize) {
    let n = &nodes[i];
    let Some(t) = &n.layout.table else { return };
    let [cols, rows] = &n.layout.grid_tracks;
    let span = |at: &[[f32; 2]], s: usize, k: usize| -> Option<(f32, f32)> {
        let a = at.get(s)?;
        let b = at.get((s + k.max(1) - 1).min(at.len().saturating_sub(1)))?;
        Some((a[0], b[1] - a[0]))
    };
    let placed: Vec<(usize, Rect)> = t
        .parts
        .iter()
        .filter_map(|&(c, (col, row, cs, rs))| {
            let (x, w) = span(cols, col, cs)?;
            let (y, h) = span(rows, row, rs)?;
            Some((c, Rect::new(n.layout.x + x, n.layout.y + y, w, h)))
        })
        .collect();
    for (c, r) in placed {
        let k = &mut nodes[c].layout;
        (k.x, k.y, k.w, k.h) = (r.x, r.y, r.width, r.height);
    }
}

/// A right-to-left element's children mirrored across its content box
/// (its scrolled content's): each margin box flips, its own margins kept
/// on their sides. An out-of-flow child placed by a horizontal inset keeps
/// its place.
fn mirror(nodes: &mut [Node], i: usize, sx: f32, flow: &[usize], out: &[usize]) {
    let n = &nodes[i];
    let inner = (n.layout.w - n.layout.used.padding.horizontal()).max(0.0);
    let width = if n.layout.used.scroll_x {
        inner.max(n.layout.content_w)
    } else {
        inner
    };
    let x0 = n.layout.x + n.layout.used.padding.left - sx;
    let free = |c: &Node| {
        !c.props.out_of_flow()
            || (c.layout.insets.start.is_none()
                && c.layout.insets.end.is_none()
                && !c.layout.insets.center_x
                && c.layout.insets.across.is_none()
                && c.props.anchor.is_none())
    };
    for &c in flow.iter().chain(out) {
        let k = &nodes[c];
        if !free(k) {
            continue;
        }
        let [ml, mr] = k.layout.margin_x;
        nodes[c].layout.x = 2.0 * x0 + width - k.layout.x - k.layout.w - mr + ml;
    }
    let pl = nodes[i].layout.used.padding.left;
    for r in &mut nodes[i].layout.col_rules {
        r[0] = 2.0 * pl + width - r[0];
    }
}

/// Relative and sticky children (CSS Position 3 §3.3–3.4): moved from
/// where the flow put them, their siblings unmoved.
fn relative(
    nodes: &mut [Node],
    i: usize,
    kids: &[usize],
    scroll: &dyn Fn(u64) -> (f32, f32),
    (sx, sy): (f32, f32),
) {
    if !kids.iter().any(|&c| {
        matches!(
            nodes[c].props.position,
            Position::Relative | Position::Sticky
        )
    }) {
        return;
    }
    let _ = scroll;
    let n = &nodes[i];
    let (iw, ih) = (
        (n.layout.w - n.layout.used.padding.horizontal()).max(0.0),
        (n.layout.h - n.layout.used.padding.vertical()).max(0.0),
    );
    let cb = Rect::new(
        n.layout.x + n.layout.used.padding.left - sx,
        n.layout.y + n.layout.used.padding.top - sy,
        if sx > 0.0 {
            iw.max(n.layout.content_w)
        } else {
            iw
        },
        if n.layout.used.scroll {
            ih.max(n.layout.content_h)
        } else {
            ih
        },
    );
    // The nearest scroller's scrollport, at or above the parent.
    let mut at = i;
    while at != NONE && !(nodes[at].layout.used.scroll || nodes[at].layout.used.scroll_x) {
        at = nodes[at].up;
    }
    let port = (at != NONE).then(|| {
        let l = &nodes[at].layout;
        Rect::new(l.x, l.y, l.w, l.h)
    });
    for &c in kids {
        let ins = nodes[c].layout.insets;
        let l = &nodes[c].layout;
        let (x, y, w, h) = (l.x, l.y, l.w, l.h);
        let (dx, dy) = match nodes[c].props.position {
            Position::Relative => (
                ins.start.or(ins.end.map(|e| -e)).unwrap_or(0.0),
                ins.top.or(ins.bottom.map(|b| -b)).unwrap_or(0.0),
            ),
            Position::Sticky => {
                let Some(p) = port else { continue };
                // Toward the scrollport's inset edge, never out of its
                // containing block (§3.4).
                #[allow(clippy::too_many_arguments)]
                fn stick(
                    at: f32,
                    len: f32,
                    lo: Option<f32>,
                    hi: Option<f32>,
                    p0: f32,
                    p1: f32,
                    c0: f32,
                    c1: f32,
                ) -> f32 {
                    let mut v = at;
                    if let Some(lo) = lo {
                        v = v.max(p0 + lo).min((c1 - len).max(at));
                    }
                    if let Some(hi) = hi {
                        v = v.min(p1 - hi - len).max(c0.min(at));
                    }
                    v - at
                }
                (
                    stick(x, w, ins.start, ins.end, p.x, p.right(), cb.x, cb.right()),
                    stick(
                        y,
                        h,
                        ins.top,
                        ins.bottom,
                        p.y,
                        p.bottom(),
                        cb.y,
                        cb.bottom(),
                    ),
                )
            }
            _ => continue,
        };
        nodes[c].layout.x += dx;
        nodes[c].layout.y += dy;
        use crate::css::media::scroll::{BOTTOM, LEFT, RIGHT, TOP};
        nodes[c].layout.stuck = if nodes[c].props.position == Position::Sticky {
            let edge = |d: f32, lo: u16, hi: u16| match d {
                d if d > 0.01 => lo,
                d if d < -0.01 => hi,
                _ => 0,
            };
            (edge(dy, TOP, BOTTOM) | edge(dx, LEFT, RIGHT)) as u8
        } else {
            0
        };
    }
}
