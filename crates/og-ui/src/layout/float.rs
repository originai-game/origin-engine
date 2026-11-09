//! CSS 2 §9.5 floats in a block formatting context.
//!
//! A block container's floated children leave its flow (`Node::floats`).
//! Once its subtree's heights are known, each formatting context root lays
//! its flow out again from the top, in tree order, through the blocks in
//! it that start no context of their own: each float is shifted to its
//! side of its containing block as high as §9.5.1 lets it, below earlier
//! floats it does not fit beside; a text's lines are shortened around the
//! floats beside them (a line no word fits moves down past them); a box
//! that starts a context of its own (a flow-root, a flex or grid box, an
//! image, a widget, a scroller) does not overlap them: it is narrowed to
//! the room beside them, or moved below them where it does not fit; and
//! `clear` puts a box below the earlier floats of its sides. The root's
//! height holds its floats (§10.6.7).
//!
//! Every kind of line box is shortened: a plain text's, a rich text's
//! (its spans broken one line at a time), and a block's lines of
//! inline-level boxes (inline-block, images, inline flex or grid, texts),
//! filled one at a time into the room beside the floats, a line no box
//! fits moved down past them. A block-level box among such lines takes a
//! line of its own, fitted to the room as well.

use super::flex::{
    column_spacing, cross_offset, cross_room, cross_size, css_line, distribute_w, inline_level,
    inline_rows, place_sides, row_bases, size_h, size_row, span,
};
use super::*;
use crate::rich::{RichContent, RichLine, RichSpan};

/// `clear: left` (and `both`).
pub const LEFT: u8 = 1;
/// `clear: right` (and `both`).
pub const RIGHT: u8 = 2;

/// A block container: its children lie in a block flow, or in lines, and
/// floats leave it (in a flex, grid, table or multi-column container
/// `float` does not apply).
pub(crate) fn container(n: &Node) -> bool {
    n.el.block
        && n.props.layout == LayoutMode::Flex
        && matches!(n.content, Content::None)
        && n.el.multicol.is_none()
}

/// Node `i` lays out in its parent's formatting context: a block in a
/// block's flow that starts none of its own (CSS 2 §9.4.1), whether its
/// children lie in a block flow or in lines.
fn joins(nodes: &[Node], i: usize) -> bool {
    let n = &nodes[i];
    let p = n.up;
    p != NONE
        && p < nodes.len()
        && container(n)
        && container(&nodes[p])
        && nodes[p].line_box.is_none()
        && n.line_box.map_or(n.el.direction, |l| l.0) == Direction::Column
        && n.el.float == FloatSide::None
        && !n.props.out_of_flow()
        && !n.el.bfc
        && !n.el.inline
        && !n.el.scroll
        && !n.el.scroll_x
}

/// A placed float's margin box, from its context root's content box.
#[derive(Clone, Copy, Debug)]
struct Placed {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    left: bool,
}

/// The room floats leave between `x0` and `x1` over `y0..y1`: its left and
/// right edges, and the highest bottom of the floats that narrow it
/// (`None`: none does).
fn room(floats: &[Placed], y0: f32, y1: f32, x0: f32, x1: f32) -> (f32, f32, Option<f32>) {
    let y1 = y1.max(y0 + 0.01);
    let (mut l, mut r, mut next) = (x0, x1, None::<f32>);
    for f in floats {
        if f.h <= 0.0 || f.y >= y1 || f.y + f.h <= y0 {
            continue;
        }
        let narrows = if f.left { f.x + f.w > x0 } else { f.x < x1 };
        if !narrows {
            continue;
        }
        if f.left {
            l = l.max(f.x + f.w);
        } else {
            r = r.min(f.x);
        }
        let b = f.y + f.h;
        next = Some(next.map_or(b, |n: f32| n.min(b)));
    }
    (l, r, next)
}

/// The bottom of the floats on the sides `clear` names (`-inf`: none).
fn cleared(floats: &[Placed], clear: u8) -> f32 {
    floats
        .iter()
        .filter(|f| clear & if f.left { LEFT } else { RIGHT } != 0)
        .map(|f| f.y + f.h)
        .fold(f32::NEG_INFINITY, f32::max)
}

struct Context<'m, 'a> {
    floats: Vec<Placed>,
    measure: &'m mut Measure<'a>,
    touched: &'m mut Vec<usize>,
}

/// Whether the context under container `c` holds a float, or a box floats
/// moved, narrowed or shortened last layout.
fn needs(nodes: &[Node], c: usize) -> bool {
    let n = &nodes[c];
    !n.floats.is_empty()
        || !n.wrap_lines.is_empty()
        || n.children.iter().any(|&k| {
            let k_n = &nodes[k];
            k_n.beside
                || k_n.clearance != 0.0
                || !k_n.wrap_lines.is_empty()
                || (joins(nodes, k) && needs(nodes, k))
        })
}

/// Lays out the floats of the formatting context node `i` is the root of
/// (heights known, its own just measured), and the flow around them. What
/// it moves, narrows or shortens goes into `touched`.
pub(crate) fn flow(
    nodes: &mut [Node],
    i: usize,
    measure: &mut Measure<'_>,
    touched: &mut Vec<usize>,
) {
    let root = container(&nodes[i]) && !joins(nodes, i);
    if !root || !needs(nodes, i) {
        // A context it no longer roots holds no floats of its own.
        if nodes[i].float_h != 0.0 {
            nodes[i].float_h = 0.0;
            size_h(nodes, i);
        }
        return;
    }
    let mut cx = Context {
        floats: Vec::new(),
        measure,
        touched,
    };
    walk(nodes, i, 0.0, 0.0, &mut cx);
    let bottom = cx.floats.iter().map(|f| f.y + f.h).fold(0.0, f32::max);
    nodes[i].float_h = bottom;
    size_h(nodes, i);
    cx.touched.push(i);
}

/// Container `c`'s flow with its content box at `(ox, oy)` in the root's.
fn walk(nodes: &mut [Node], c: usize, ox: f32, oy: f32, cx: &mut Context<'_, '_>) {
    let n = &nodes[c];
    let inner = (n.w - n.el.padding.horizontal()).max(0.0);
    let (x0, x1) = (ox, ox + inner);
    let floats = n.floats.clone();
    if n.line_box.is_some() {
        // Its floats sit at its top, its lines beside them.
        for f in floats {
            place(nodes, f, x0, x1, oy, (ox, oy), cx);
        }
        lines(nodes, c, x0, x1, oy, cx);
        return;
    }
    let kids = nodes[c].children.clone();
    for &k in &kids {
        nodes[k].clearance = 0.0;
        baseline(nodes, c, k, inner, cx);
    }
    let (lead, between, _) = column_spacing(nodes, c);
    let order = nodes[c].ordered_children.clone();
    let mut pending = Vec::new();
    let (mut y, mut j) = (oy, 0);
    for k in order {
        if floats.contains(&k) {
            pending.push(k);
            continue;
        }
        if kids.get(j) != Some(&k) {
            continue;
        }
        let top = y + if j == 0 { lead } else { between[j - 1] };
        for f in pending.drain(..) {
            place(nodes, f, x0, x1, top, (ox, oy), cx);
        }
        let clear = (cleared(&cx.floats, nodes[k].el.clear) - top).max(0.0);
        let mut at = top + clear;
        let was = nodes[k].beside;
        nodes[k].beside = false;
        if joins(nodes, k) {
            let kx = child_x(nodes, c, k, x0, inner);
            let pad = nodes[k].el.padding;
            walk(nodes, k, kx + pad.left, at + pad.top, cx);
            size_h(nodes, k);
            nodes[k].beside = !cx.floats.is_empty();
        } else if wraps(&nodes[k]) {
            text(nodes, k, x0, inner, at, cx, was);
        } else if rich_wraps(&nodes[k]) {
            rich(nodes, k, x0, inner, at, cx, was);
        } else {
            at = beside(nodes, k, x0, x1, at, cx);
        }
        nodes[k].clearance = at - top;
        cx.touched.push(k);
        y = at + nodes[k].pref_h;
        j += 1;
    }
    for f in pending {
        place(nodes, f, x0, x1, y, (ox, oy), cx);
    }
    // Its children's spacing moved by their clearance.
    size_h(nodes, c);
    cx.touched.push(c);
}

/// Child `k`'s width and side margins in container `c`'s flow before
/// floats (as `distribute_w` gives them).
fn baseline(nodes: &mut [Node], c: usize, k: usize, inner: f32, cx: &mut Context<'_, '_>) {
    let el = nodes[c].el;
    let n = &nodes[k];
    let fill = (inner - n.el.margin[1].of(inner) - n.el.margin[3].of(inner)).max(0.0);
    let basis = match n.el.width {
        Size::Grow { .. } | Size::Fit { .. } => fill,
        _ => inner,
    };
    let stretch = !el.wrap && n.el.align_self.unwrap_or(el.cross) == Cross::Stretch;
    let w = cross_size(n, 0, basis, inner, stretch);
    let w = match n.el.width {
        Size::Fit { .. } if el.scroll_x => w.max(n.pref_w),
        _ => w,
    };
    if !wraps(n) && w != n.w {
        relayout(nodes, k, w, cx);
    }
    nodes[k].w = w;
    nodes[k].margin_x = place_sides(&nodes[k], inner, fill - w);
}

/// Where container `c` puts child `k`'s border box across (as
/// `place_children` does).
fn child_x(nodes: &[Node], c: usize, k: usize, x0: f32, inner: f32) -> f32 {
    let el = nodes[c].el;
    let n = &nodes[k];
    let [ml, mr] = n.margin_x;
    if n.el.margin[1].auto || n.el.margin[3].auto {
        return x0 + ml;
    }
    let safe = el.safe & 2 != 0 || n.el.safe & 2 != 0;
    x0 + ml
        + cross_offset(
            n.el.align_self.unwrap_or(el.cross),
            inner - ml - mr,
            n.w,
            safe,
        )
}

/// Float `f` of a container with content box `x0..x1` placed at or below
/// `y` (CSS 2 §9.5.1): `origin` is the container's content box.
fn place(
    nodes: &mut [Node],
    f: usize,
    x0: f32,
    x1: f32,
    y: f32,
    origin: (f32, f32),
    cx: &mut Context<'_, '_>,
) {
    let n = &nodes[f];
    let [ml, mr] = n.margin_x;
    let (mt, mb) = (span(n.margin_sets[0]), span(n.margin_sets[1]));
    let (w, h) = (n.w + ml + mr, n.pref_h + mt + mb);
    let left = n.el.float != FloatSide::Right;
    // No higher than an earlier float (rule 5), below those it clears.
    let mut y = y
        .max(cx.floats.last().map_or(f32::NEG_INFINITY, |p| p.y))
        .max(cleared(&cx.floats, n.el.clear));
    let x = loop {
        let (l, r, next) = room(&cx.floats, y, y + h, x0, x1);
        match next {
            Some(b) if r - l < w - 0.01 => y = b,
            _ => break if left { l } else { r - w },
        }
    };
    cx.floats.push(Placed { x, y, w, h, left });
    nodes[f].float_at = [x - origin.0 + ml, y - origin.1 + mt];
    cx.touched.push(f);
}

/// A text whose lines floats shorten (an auto-fit, cut or path text keeps
/// its lines).
fn wraps(n: &Node) -> bool {
    matches!(&n.content, Content::Text(t) if t.fit.is_empty() && !t.ellipsis && t.max_lines == 0)
        && n.props.text_path.is_none()
        && n.widget == Widget::None
}

/// Text `k` of container `c` with its border box's top at `top`: its lines
/// broken one at a time at the room the floats beside each leave, one no
/// word fits moved down past them. Its box is its line boxes' width.
#[allow(clippy::too_many_arguments)]
fn text(
    nodes: &mut [Node],
    k: usize,
    x0: f32,
    inner: f32,
    top: f32,
    cx: &mut Context<'_, '_>,
    was: bool,
) {
    let n = &nodes[k];
    let Content::Text(t) = &n.content else { return };
    let [ml, mr] = n.margin_x;
    let pad = n.el.padding;
    let fill = (inner - ml - mr).max(0.0);
    let (tx0, tx1) = (x0 + ml + pad.left, x0 + ml + fill - pad.right);
    let ty = top + pad.top;
    let lh = t.line_height();
    let lowest = cx
        .floats
        .iter()
        .map(|f| f.y + f.h)
        .fold(f32::NEG_INFINITY, f32::max);
    let fonts = cx.measure.fonts;
    let m = t.metrics(fonts);
    // Down a vertical line, its lines measure their advances down it.
    let at = |s: &str, w: f32| {
        let part = TextContent {
            text: s.to_string(),
            ..t.clone()
        };
        super::flex::text_lines(&part, m, t.wrap.then_some(w + 0.5))
    };
    let full = (tx1 - tx0).max(0.0);
    let (mut lines, mut shifts) = (Vec::new(), Vec::new());
    let mut narrowed = false;
    let mut rest: &str = &t.text;
    let mut drop = 0.0f32;
    loop {
        let y = ty + lines.len() as f32 * lh + drop;
        if y >= lowest {
            for l in at(rest, full) {
                shifts.push([0.0, drop, full]);
                lines.push(l);
            }
            break;
        }
        let (l, r, next) = room(&cx.floats, y, y + lh, tx0, tx1);
        let avail = (r - l).max(0.0);
        let first = at(rest, avail).into_iter().next().unwrap_or(Line {
            text: String::new(),
            width: 0.0,
        });
        if let Some(b) = next
            && first.width > avail + 0.5
        {
            drop += b - y;
            continue;
        }
        narrowed |= next.is_some();
        let p = rest.find(first.text.as_str()).unwrap_or(0);
        rest = rest[p + first.text.len()..].trim_start_matches([' ', '\t']);
        let newline = rest.starts_with('\n') || rest.starts_with("\r\n");
        rest = rest
            .strip_prefix("\r\n")
            .or_else(|| rest.strip_prefix('\n'))
            .unwrap_or(rest);
        shifts.push([l - tx0, drop, avail]);
        lines.push(first);
        if rest.is_empty() {
            if newline {
                shifts.push([0.0, drop, full]);
                lines.push(Line {
                    text: String::new(),
                    width: 0.0,
                });
            }
            break;
        }
    }
    if narrowed || drop > 0.0 {
        nodes[k].w = fill;
        nodes[k].margin_x = [ml, mr];
        nodes[k].lines = lines;
        nodes[k].wrap_lines = shifts;
        nodes[k].beside = true;
        size_h(nodes, k);
    } else if was || !nodes[k].wrap_lines.is_empty() {
        // Clear of floats: its lines as its width breaks them.
        nodes[k].wrap_lines.clear();
        distribute_w(nodes, k, cx.measure);
        size_h(nodes, k);
    }
}

/// A rich text whose lines floats shorten.
fn rich_wraps(n: &Node) -> bool {
    matches!(&n.content, Content::Rich(rc) if rc.wrap)
        && n.props.text_path.is_none()
        && n.widget == Widget::None
}

/// Rich text `k` of container `c` with its border box's top at `top`, as
/// [`text`] lays out a plain one: each line broken from the spans left at
/// the room the floats beside it leave, one no word fits moved down past
/// them.
#[allow(clippy::too_many_arguments)]
fn rich(
    nodes: &mut [Node],
    k: usize,
    x0: f32,
    inner: f32,
    top: f32,
    cx: &mut Context<'_, '_>,
    was: bool,
) {
    let n = &nodes[k];
    let Content::Rich(rc) = &n.content else {
        return;
    };
    let [ml, mr] = n.margin_x;
    let pad = n.el.padding;
    let fill = (inner - ml - mr).max(0.0);
    let (tx0, tx1) = (x0 + ml + pad.left, x0 + ml + fill - pad.right);
    let full = (tx1 - tx0).max(0.0);
    let lowest = cx
        .floats
        .iter()
        .map(|f| f.y + f.h)
        .fold(f32::NEG_INFINITY, f32::max);
    let fonts = cx.measure.fonts;
    let break_at = |rest: &[(usize, RichSpan)], w: f32| {
        let part = RichContent {
            spans: rest.iter().map(|s| s.1.clone()).collect(),
            ..rc.clone()
        };
        crate::rich::rich_lines(&part, fonts, Some(w + 0.5))
    };
    // A line of the spans left, its spans renumbered to the paragraph's.
    let renumber = |rest: &[(usize, RichSpan)], mut l: RichLine| {
        for p in &mut l.pieces {
            p.span = rest[p.span].0;
        }
        for s in l.shaped.iter_mut().flatten() {
            s.span = rest[s.span].0;
        }
        l
    };
    let guess = rc.base.0.line_height;
    let mut rest: Vec<(usize, RichSpan)> = rc.spans.iter().cloned().enumerate().collect();
    let (mut lines, mut shifts) = (Vec::new(), Vec::new());
    let (mut y, mut drop, mut narrowed) = (top + pad.top, 0.0f32, false);
    while !rest.is_empty() {
        if y >= lowest {
            for l in break_at(&rest, full) {
                shifts.push([0.0, drop, full]);
                lines.push(renumber(&rest, l));
            }
            break;
        }
        // The room beside the line as tall as the line it holds.
        let mut h = guess;
        let (line, l, avail, next) = loop {
            let (l, r, next) = room(&cx.floats, y, y + h, tx0, tx1);
            let avail = (r - l).max(0.0);
            let line = break_at(&rest, avail)
                .into_iter()
                .next()
                .unwrap_or_default();
            if line.height <= h + 0.01 {
                break (line, l, avail, next);
            }
            h = line.height;
        };
        if let Some(b) = next
            && line.width > avail + 0.5
        {
            drop += b - y;
            y = b;
            continue;
        }
        narrowed |= next.is_some();
        // What the line took from each span (an atom whole).
        let mut used = vec![(0usize, false); rest.len()];
        let mut newline = false;
        for p in &line.pieces {
            used[p.span].0 += p.text.chars().count();
            used[p.span].1 = true;
            newline |= p.text.ends_with('\n');
        }
        let lh = line.height;
        lines.push(renumber(&rest, line));
        shifts.push([l - tx0, drop, avail]);
        let mut moved = false;
        let mut left = Vec::with_capacity(rest.len());
        for ((id, mut s), (chars, took)) in rest.into_iter().zip(used) {
            if s.atom.is_some() {
                moved |= took;
                if !took {
                    left.push((id, s));
                }
                continue;
            }
            let cut = s
                .text
                .char_indices()
                .nth(chars)
                .map_or(s.text.len(), |(b, _)| b);
            moved |= cut > 0;
            s.text.drain(..cut);
            if !s.text.is_empty() {
                left.push((id, s));
            }
        }
        // The spaces at the break go, and a forced break with them.
        while let Some((_, s)) = left.first_mut()
            && s.atom.is_none()
        {
            let t = s.text.trim_start_matches([' ', '\t']);
            let t = match t.strip_prefix("\r\n").or_else(|| t.strip_prefix('\n')) {
                Some(t) if !newline => {
                    newline = true;
                    t
                }
                _ => t,
            };
            moved |= t.len() != s.text.len();
            s.text = t.to_owned();
            if !s.text.is_empty() {
                break;
            }
            left.remove(0);
        }
        rest = left;
        y += lh;
        if !moved {
            // Nothing more fits a line (cannot happen with text left).
            break;
        }
        if rest.is_empty() && newline {
            shifts.push([0.0, drop, full]);
            lines.push(RichLine {
                height: guess,
                ..RichLine::default()
            });
        }
    }
    if narrowed || drop > 0.0 {
        nodes[k].w = fill;
        nodes[k].margin_x = [ml, mr];
        nodes[k].rich_lines = lines;
        nodes[k].wrap_lines = shifts;
        nodes[k].beside = true;
        size_h(nodes, k);
    } else if was || !nodes[k].wrap_lines.is_empty() {
        // Clear of floats: its lines as its width breaks them.
        nodes[k].wrap_lines.clear();
        distribute_w(nodes, k, cx.measure);
        size_h(nodes, k);
    }
}

/// Line-box container `c` with its content box from `x0` to `x1` and its
/// top at `top`: its inline-level children filled into lines one at a
/// time, each into the room the floats beside it leave (CSS 2 §9.5), one
/// whose first box does not fit moved down past them.
fn lines(nodes: &mut [Node], c: usize, x0: f32, x1: f32, top: f32, cx: &mut Context<'_, '_>) {
    let kids = nodes[c].children.clone();
    let inner = (x1 - x0).max(0.0);
    let el = nodes[c].el;
    let (gap, line_gap) = (el.main_gap(), el.line_gap());
    let (base, sides) = row_bases(nodes, &kids, inner);
    let css = css_line(nodes, &kids);
    // A block-level box (or the box after one) starts a line.
    let starts: Vec<bool> = (0..kids.len())
        .map(|k| k > 0 && !(inline_level(&nodes[kids[k]]) && inline_level(&nodes[kids[k - 1]])))
        .collect();
    let outer = |k: usize| base[k] + sides[k];
    let tall = |nodes: &[Node], k: usize| nodes[kids[k]].pref_h + cross_room(&nodes[kids[k]]);
    let (mut rows, mut shifts) = (Vec::new(), Vec::new());
    let (mut s, mut y, mut drop, mut narrowed) = (0, top, 0.0f32, false);
    while s < kids.len() {
        // The room beside the line as tall as the boxes it holds.
        let mut h = tall(nodes, s);
        let (e, l, avail, next) = loop {
            let (l, r, next) = room(&cx.floats, y, y + h, x0, x1);
            let avail = (r - l).max(0.0);
            let (mut e, mut used) = (s + 1, outer(s));
            while e < kids.len() && !starts[e] && used + gap + outer(e) <= avail + 0.5 {
                used += gap + outer(e);
                e += 1;
            }
            let held = (s..e).map(|k| tall(nodes, k)).fold(0.0, f32::max);
            if held <= h + 0.01 {
                break (e, l, avail, next);
            }
            h = held;
        };
        if let Some(b) = next
            && outer(s) > avail + 0.5
        {
            drop += b - y;
            y = b;
            continue;
        }
        narrowed |= next.is_some();
        let line = &kids[s..e];
        let was: Vec<f32> = line.iter().map(|&k| nodes[k].w).collect();
        size_row(
            nodes,
            line,
            css,
            &base[s..e],
            &sides[s..e],
            gap,
            inner,
            avail,
        );
        for (&k, w0) in line.iter().zip(was) {
            if nodes[k].w != w0 {
                let w = nodes[k].w;
                relayout(nodes, k, w, cx);
            }
        }
        rows.push((s, e));
        shifts.push([l - x0, drop, avail]);
        // Its height as its line box holds it (`inline_rows`).
        nodes[c].rows.clone_from(&rows);
        inline_rows(nodes, c, cx.measure);
        let lh = nodes[c]
            .inline_rows
            .get(rows.len() - 1)
            .copied()
            .flatten()
            .unwrap_or_else(|| (s..e).map(|k| tall(nodes, k)).fold(0.0, f32::max));
        y += lh + line_gap;
        s = e;
    }
    nodes[c].rows = rows;
    inline_rows(nodes, c, cx.measure);
    if narrowed || drop > 0.0 {
        nodes[c].wrap_lines = shifts;
        nodes[c].beside = true;
    } else {
        nodes[c].wrap_lines.clear();
    }
    size_h(nodes, c);
    cx.touched.push(c);
    cx.touched.extend(kids);
}

/// Box `k` that starts a formatting context of its own, at `top` or
/// lower: its border box beside the floats, narrowed to the room they
/// leave when its width fills the line (CSS 2 §9.5), moved down past them
/// where it does not fit. Returns its top.
#[allow(clippy::too_many_arguments)]
fn beside(
    nodes: &mut [Node],
    k: usize,
    x0: f32,
    x1: f32,
    top: f32,
    cx: &mut Context<'_, '_>,
) -> f32 {
    let n = &nodes[k];
    let [ml, mr] = n.margin_x;
    let w0 = n.w;
    let fill = (x1 - x0 - ml - mr).max(0.0);
    // A box that fills the line narrows to the room; another keeps its width.
    let fills = w0 >= fill - 0.01;
    let need = if fills { n.min_w } else { w0 };
    let h = n.pref_h;
    let mut y = top;
    loop {
        let (l, r, next) = room(&cx.floats, y, y + h, x0, x1);
        let Some(b) = next else { break };
        let (bl, br) = ((x0 + ml).max(l), (x1 - mr).min(r));
        if br - bl >= need - 0.01 {
            let w = if fills { (br - bl).min(w0) } else { w0 };
            if w != w0 {
                relayout(nodes, k, w, cx);
            }
            // A flipped container (`layout::flow`) keeps margins on their
            // sides as it mirrors its content: the room is swapped so the
            // box mirrors whole.
            let up = nodes[k].up;
            nodes[k].margin_x = if up != NONE && nodes[up].el.flip {
                [x1 - br, bl - x0]
            } else {
                [bl - x0, x1 - br]
            };
            nodes[k].beside = true;
            return y;
        }
        y = b;
    }
    nodes[k].beside = y != top;
    y
}

/// Lays subtree `k` out again at width `w`: its widths, then its heights
/// (with the floats of the contexts in it).
fn relayout(nodes: &mut [Node], k: usize, w: f32, cx: &mut Context<'_, '_>) {
    nodes[k].w = w;
    let order = preorder(nodes, k);
    for &j in &order {
        distribute_w(nodes, j, cx.measure);
    }
    for &j in order.iter().rev() {
        super::flex::inline_rows(nodes, j, cx.measure);
        size_h(nodes, j);
        flow(nodes, j, cx.measure, cx.touched);
    }
    cx.touched.extend(order);
}
