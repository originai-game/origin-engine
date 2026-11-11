//! Vertical writing modes in layout (CSS Writing Modes 4 §7). An element
//! whose writing mode is not its parent's starts a frame: its subtree's
//! boxes, up to the next element that changes the writing mode again, are
//! turned so the frame's top is their block-start and its left their
//! inline-start ([`Element::vertical`]), laid out by the horizontal flow,
//! flex, grid, table and multi-column passes, and turned back. To the
//! frame around it the element is one box, sized by its own layout (an
//! orthogonal flow, §7.3): its inline size fits the containing block's
//! size along that axis where its layout fixes one, else the initial
//! containing block's; its block size is its content's. When the frame
//! around it then gives it another size (a stretch, a flex line), it lays
//! out again at that size.

use super::*;

/// The sides of a frame: frame side `k` (block-start, inline-end,
/// block-end, inline-start) is side `s[k]` (0 top … 3 left) of the space
/// around it.
type Sides = [u8; 4];

const LAYOUT_SPACE: Sides = [0, 1, 2, 3];

fn sides_of(n: &Node) -> Sides {
    n.layout.used.vertical.unwrap_or(LAYOUT_SPACE)
}

/// The frame's axes cross the space's (a vertical frame in a horizontal
/// space, or the other way round).
fn turns(s: Sides) -> bool {
    s[0] % 2 == 1
}

/// Node `i` starts a frame of its own: its writing mode is not that of the
/// box it lays out in.
pub(super) fn island(nodes: &[Node], i: usize) -> bool {
    let up = nodes[i].up;
    up != NONE
        && up < nodes.len()
        && nodes[up].layout.used.vertical != nodes[i].layout.used.vertical
}

/// Node `i`'s subtree, `i` first, in preorder.
pub(super) fn subtree(nodes: &[Node], i: usize) -> Vec<usize> {
    let mut out = Vec::new();
    let mut stack = vec![i];
    while let Some(j) = stack.pop() {
        out.push(j);
        stack.extend(nodes[j].ordered_children.iter().rev());
    }
    out
}

/// The boxes of the frame `r` starts, `r` first, in preorder: its subtree
/// down to and including the elements that start frames of their own,
/// without their subtrees.
fn members(nodes: &[Node], r: usize) -> Vec<usize> {
    let mut out = Vec::new();
    let mut stack = vec![r];
    while let Some(j) = stack.pop() {
        out.push(j);
        if j == r || !island(nodes, j) {
            stack.extend(nodes[j].ordered_children.iter().rev());
        }
    }
    out
}

/// Turns a side array (top, right, bottom, left) into frame `s`
/// (`into`), or back.
fn sides4<T: Copy>(v: [T; 4], s: Sides, into: bool) -> [T; 4] {
    let mut out = v;
    for k in 0..4 {
        if into {
            out[k] = v[s[k] as usize];
        } else {
            out[s[k] as usize] = v[k];
        }
    }
    out
}

/// Turns node `n`'s used box (its sizes, bounds, padding, margins, insets,
/// overflow and snapping) into frame `s`, or back.
fn turn_box(n: &mut Node, s: Sides, into: bool) {
    let el = &mut n.layout.used;
    let p = el.padding;
    let [top, right, bottom, left] = sides4([p.top, p.right, p.bottom, p.left], s, into);
    el.padding = Insets {
        left,
        top,
        right,
        bottom,
    };
    el.margin = sides4(el.margin, s, into);
    let e = &mut n.layout.insets;
    [e.top, e.end, e.bottom, e.start] = sides4([e.top, e.end, e.bottom, e.start], s, into);
    if turns(s) {
        std::mem::swap(&mut el.width, &mut el.height);
        el.bounds.swap(0, 1);
        if el.aspect > 0.0 {
            el.aspect = 1.0 / el.aspect;
        }
        std::mem::swap(&mut el.scroll, &mut el.scroll_x);
        el.snap = (el.snap & !3) | ((el.snap & 1) << 1) | ((el.snap >> 1) & 1);
        el.snap_align =
            (el.snap_align & !15) | ((el.snap_align & 3) << 2) | ((el.snap_align >> 2) & 3);
        std::mem::swap(&mut e.center_x, &mut e.center_y);
    }
}

/// Matches node `n`'s container variants at inline size `w` (its frame's
/// width) and makes its used element again with their patches, which set
/// the physical sides and axes: made in layout space, then turned into the
/// frame its box is laid out in.
pub(super) fn query(n: &mut Node, w: f32) {
    let bfc = n.layout.used.bfc;
    n.layout.query_w = Some(w);
    n.derive_used();
    if let Some(s) = n.layout.used.vertical {
        turn_box(n, s, true);
    }
    // A frame's root is a formatting context while it lays out (`nest`).
    n.layout.used.bfc = bfc;
}

/// Where an interval `[at, at + len]` along a frame axis lies along the
/// space's axis it runs on: the axis (0 x, 1 y) and its start. The axis
/// runs from side `from` of a box `size` wide (x, y).
fn along(from: u8, at: f32, len: f32, size: [f32; 2]) -> (usize, f32) {
    match from {
        0 => (1, at),
        1 => (0, size[0] - at - len),
        2 => (1, size[1] - at - len),
        _ => (0, at),
    }
}

/// The frame's layout of `members` turned back into the space around it,
/// from the frame root's top left: each box's place and size, its content
/// extent and intrinsic sizes.
fn turn_back(nodes: &mut [Node], members: &[usize], s: Sides) {
    let r = members[0];
    let (fw, fh) = (nodes[r].layout.w, nodes[r].layout.h);
    let size = if turns(s) { [fh, fw] } else { [fw, fh] };
    for &m in members {
        let n = &mut nodes[m];
        let mut xy = [0.0; 2];
        let (ax, sx) = along(s[3], n.layout.x, n.layout.w, size);
        xy[ax] = sx;
        let (ay, sy) = along(s[0], n.layout.y, n.layout.h, size);
        xy[ay] = sy;
        [n.layout.x, n.layout.y] = xy;
        if turns(s) {
            std::mem::swap(&mut n.layout.w, &mut n.layout.h);
            std::mem::swap(&mut n.layout.content_w, &mut n.layout.content_h);
            std::mem::swap(&mut n.layout.pref_w, &mut n.layout.pref_h);
            std::mem::swap(&mut n.layout.min_w, &mut n.layout.min_h);
            n.layout.min_content.swap(0, 1);
        }
    }
}

/// Moves node `i`'s descendants by `d`.
fn shift_inside(nodes: &mut [Node], i: usize, d: [f32; 2]) {
    if d == [0.0; 2] {
        return;
    }
    for j in subtree(nodes, i).into_iter().skip(1) {
        nodes[j].layout.x += d[0];
        nodes[j].layout.y += d[1];
    }
}

/// What a nested layout reads besides the nodes.
pub(super) struct Cx<'a, 'b> {
    pub measure: &'a mut Measure<'b>,
    pub scroll: &'a dyn Fn(u64) -> (f32, f32),
    /// The initial containing block (the area the layout call fills).
    pub icb: [f32; 2],
    /// Boxes laid out (`LayoutCache::laid_out`).
    pub laid: u32,
}

/// The margins of `r`, which starts a frame, in the frame around it: its
/// own, none collapsing through it (it is a formatting context).
pub(super) fn island_margins(nodes: &mut [Node], r: usize) {
    let bfc = nodes[r].layout.used.bfc;
    nodes[r].layout.used.bfc = true;
    collapse_margins(nodes, r);
    nodes[r].layout.used.bfc = bfc;
}

/// Lays out the frame node `r` starts, in place, and returns its size in
/// the frame around it (`parent`; `None`: layout space). `avail` (layout
/// space) is the room its inline size fits in; `size` (layout space) is
/// its border box when the frame around it fixed one. Its subtree ends
/// placed from its own top left; [`settle`] moves it where `r` lands.
pub(super) fn nest(
    nodes: &mut [Node],
    r: usize,
    parent: Option<Sides>,
    avail: [f32; 2],
    size: Option<[f32; 2]>,
    cx: &mut Cx<'_, '_>,
) -> [f32; 2] {
    if let Some(ps) = parent {
        turn_box(&mut nodes[r], ps, false);
    }
    let s = sides_of(&nodes[r]);
    let members = members(nodes, r);
    for &m in &members {
        turn_box(&mut nodes[m], s, true);
    }
    let bfc = nodes[r].layout.used.bfc;
    nodes[r].layout.used.bfc = true;
    let flip = |v: [f32; 2]| if turns(s) { [v[1], v[0]] } else { v };
    let (avail, size) = (flip(avail), size.map(flip));
    let inner = |nodes: &[Node], m: usize| m != r && island(nodes, m);
    let mut produced: Vec<(usize, [f32; 2])> = Vec::new();
    for &m in members.iter().rev() {
        if inner(nodes, m) {
            let p = nest(nodes, m, Some(s), cx.icb, None, cx);
            produced.push((m, p));
        } else {
            size_w(nodes, m, cx.measure);
        }
    }
    let n = &nodes[r];
    let w = match (size, n.layout.used.width) {
        (Some(sz), _) => sz[0],
        (None, Size::Fit { .. }) => n.layout.pref_w.min(avail[0]).max(n.layout.min_w),
        (None, Size::Grow { .. }) => {
            let (lo, hi) = n.layout.used.width.bounds();
            avail[0].clamp(lo, hi)
        }
        (None, sz) => sz
            .definite(n.layout.pref_w, n.layout.min_w, Some(avail[0]))
            .unwrap_or(n.layout.pref_w),
    };
    nodes[r].layout.w = if size.is_some() {
        w
    } else {
        clamp_to(&nodes[r], 0, Some(avail[0]), w)
    };
    for &m in &members {
        if !inner(nodes, m) {
            if !nodes[m].layout.cv_skip {
                distribute_w(nodes, m, cx.measure);
            }
            cx.laid += 1;
        }
    }
    // Floats and their clearance in the frame: its line-left is the
    // frame's left (CSS Writing Modes 4 §7.5, `float: left`).
    let mut touched = Vec::new();
    for &m in members.iter().rev() {
        if inner(nodes, m) {
            island_margins(nodes, m);
        } else {
            inline_rows(nodes, m, cx.measure);
            size_h(nodes, m);
            super::float::flow(nodes, m, cx.measure, &mut touched);
        }
    }
    let n = &nodes[r];
    let h = match (size, n.layout.used.height) {
        (Some(sz), _) => sz[1],
        (None, Size::Grow { .. }) => {
            let (lo, hi) = n.layout.used.height.bounds();
            avail[1].clamp(lo, hi).max(n.layout.pref_h)
        }
        (None, Size::Fit { .. }) => n.layout.pref_h,
        (None, sz) => sz
            .definite(n.layout.pref_h, n.layout.min_h, Some(avail[1]))
            .unwrap_or(n.layout.pref_h),
    };
    nodes[r].layout.h = if size.is_some() {
        h
    } else {
        clamp_to(&nodes[r], 1, Some(avail[1]), h)
    };
    for &m in &members {
        if !inner(nodes, m) && !nodes[m].layout.cv_skip {
            distribute_h(nodes, m, cx.measure);
        }
    }
    // An inner frame its boxes gave another size lays out again at it.
    for &(m, p) in &produced {
        let got = [nodes[m].layout.w, nodes[m].layout.h];
        if got != p {
            let fixed = if turns(s) { [got[1], got[0]] } else { got };
            nest(nodes, m, Some(s), cx.icb, Some(fixed), cx);
            [nodes[m].layout.w, nodes[m].layout.h] = got;
        }
    }
    // Scroll offsets in the frame: an axis that runs against the space's
    // counts from its far end, so a scroller keeps its physical offset.
    let mut offsets = std::collections::HashMap::new();
    for &m in &members {
        let n = &nodes[m];
        if !n.scrolls() || inner(nodes, m) {
            continue;
        }
        let (px, py) = (cx.scroll)(n.key);
        let on = |from: u8, max: f32| {
            let v = if from % 2 == 1 { px } else { py };
            if from == 1 || from == 2 {
                max - v.clamp(0.0, max)
            } else {
                v
            }
        };
        offsets.insert(
            n.key,
            (on(s[3], n.max_scroll_x()), on(s[0], n.max_scroll())),
        );
    }
    let outer = cx.scroll;
    let scroll = |key: u64| offsets.get(&key).copied().unwrap_or_else(|| outer(key));
    nodes[r].layout.x = 0.0;
    nodes[r].layout.y = 0.0;
    for &m in &members {
        if !inner(nodes, m) && !nodes[m].layout.cv_skip {
            position(nodes, m, &scroll);
        }
    }
    // Its intrinsic sizes to the frame around it: the inline size it laid
    // out at (no less than its least) and its block size.
    let n = &nodes[r];
    let inline = (n.layout.w, n.layout.min_w.min(n.layout.w));
    let block = n.layout.h;
    nodes[r].layout.used.bfc = bfc;
    turn_back(nodes, &members, s);
    for &m in &members[1..] {
        if inner(nodes, m) {
            let d = [nodes[m].layout.x, nodes[m].layout.y];
            shift_inside(nodes, m, d);
        }
    }
    for &m in &members {
        turn_box(&mut nodes[m], s, false);
    }
    // Layout space: (pref, min) across, then down.
    let space = if turns(s) {
        [(block, block), inline]
    } else {
        [inline, (block, block)]
    };
    let ps = parent.unwrap_or(LAYOUT_SPACE);
    if parent.is_some() {
        turn_box(&mut nodes[r], ps, true);
    }
    let [a, b] = if turns(ps) {
        [space[1], space[0]]
    } else {
        space
    };
    let n = &mut nodes[r];
    (
        n.layout.pref_w,
        n.layout.min_w,
        n.layout.pref_h,
        n.layout.min_h,
    ) = (a.0, a.1, b.0, b.1);
    n.layout.min_content = [a.1, b.1];
    n.layout.w = a.0;
    n.layout.h = b.0;
    [a.0, b.0]
}

/// Vertical frames in one layout call: which boxes start one, and which of
/// them lay out this call.
pub(super) struct Frames {
    /// Inside a frame laid out this call (its root excluded): the layout
    /// space's passes leave it to [`nest`].
    pub inside: Vec<bool>,
    /// The frame roots in the layout space, in order, with whether each
    /// lays out this call and the size it gave.
    pub roots: Vec<(usize, Option<[f32; 2]>)>,
}

impl Frames {
    pub fn inside(&self, i: usize) -> bool {
        self.inside.get(i).copied().unwrap_or(false)
    }

    pub fn root(&self, i: usize) -> bool {
        self.roots.iter().any(|&(r, _)| r == i)
    }

    /// A frame root that laid out this call.
    pub fn laid(&self, i: usize) -> bool {
        self.roots.iter().any(|&(r, p)| r == i && p.is_some())
    }
}

/// The frames `order` (the layout call's walk from `root`) reaches; `None`
/// when no box in it is vertical. A frame lays out this call when a box in
/// it changed or scrolls (its whole subtree then joins `order`); the others
/// keep their layout unless the space gives them another size ([`refit`]).
pub(super) fn frames(
    nodes: &[Node],
    root: usize,
    order: &mut Vec<usize>,
    clean: &dyn Fn(usize) -> bool,
) -> Option<Frames> {
    if !order
        .iter()
        .any(|&i| nodes[i].layout.used.vertical.is_some())
    {
        return None;
    }
    let mut f = Frames {
        inside: vec![false; nodes.len()],
        roots: Vec::new(),
    };
    let mut k = 0;
    while k < order.len() {
        let i = order[k];
        let starts = if i == root {
            nodes[i].layout.used.vertical.is_some()
        } else {
            island(nodes, i)
        };
        if f.inside(i) || !starts {
            k += 1;
            continue;
        }
        let all = subtree(nodes, i);
        let lays = all.iter().any(|&j| !clean(j) || nodes[j].scrolls());
        f.roots.push((i, lays.then_some([0.0; 2])));
        if lays {
            join(&mut f, order, k, &all);
        }
        k += 1;
    }
    Some(f)
}

/// Puts frame root `order[k]`'s whole subtree (`all`) in `order` after it,
/// in place of what the walk reached of it, and marks it inside.
fn join(f: &mut Frames, order: &mut Vec<usize>, k: usize, all: &[usize]) {
    let mut end = k + 1;
    while end < order.len() && all.contains(&order[end]) {
        end += 1;
    }
    order.splice(k + 1..end, all[1..].iter().copied());
    for &j in &all[1..] {
        f.inside[j] = true;
    }
}

/// Lays out the frames that changed (the layout space's widths pass, from
/// the leaves up, reaches each root).
pub(super) fn lay(nodes: &mut [Node], f: &mut Frames, i: usize, cx: &mut Cx<'_, '_>) -> bool {
    let Some(at) = f.roots.iter().position(|&(r, _)| r == i) else {
        return false;
    };
    if f.roots[at].1.is_some() {
        let icb = cx.icb;
        f.roots[at].1 = Some(nest(nodes, i, None, icb, None, cx));
    }
    true
}

/// After the layout space's heights: each frame root the space gave
/// another size than it laid out at lays out again at it (one that kept
/// its layout joins `order` to).
pub(super) fn refit(
    nodes: &mut [Node],
    f: &mut Frames,
    order: &mut Vec<usize>,
    laid_size: &dyn Fn(&Node) -> [f32; 2],
    cx: &mut Cx<'_, '_>,
) {
    for at in 0..f.roots.len() {
        let (r, p) = f.roots[at];
        let got = [nodes[r].layout.w, nodes[r].layout.h];
        let was = p.unwrap_or_else(|| laid_size(&nodes[r]));
        if got == was {
            continue;
        }
        if p.is_none() {
            let Some(k) = order.iter().position(|&j| j == r) else {
                continue;
            };
            let all = subtree(nodes, r);
            join(f, order, k, &all);
        }
        let icb = cx.icb;
        nest(nodes, r, None, icb, Some(got), cx);
        [nodes[r].layout.w, nodes[r].layout.h] = got;
        f.roots[at].1 = Some(got);
    }
}

/// Once the layout space placed them, moves the subtree of each frame root
/// that laid out this call from its root's top left to where it landed.
pub(super) fn settle(nodes: &mut [Node], f: &Frames) {
    for &(r, p) in &f.roots {
        if p.is_some() {
            let d = [nodes[r].layout.x, nodes[r].layout.y];
            shift_inside(nodes, r, d);
        }
    }
}
