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
//! Not shortened: the lines of inline-level boxes and of rich text, which
//! are laid out whole beside floats as a box with a context of its own.

use super::flex::{
    column_spacing, cross_offset, cross_size, distribute_w, place_sides, size_h, span,
};
use super::*;

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
/// block's flow that starts none of its own (CSS 2 §9.4.1).
fn joins(nodes: &[Node], i: usize) -> bool {
    let n = &nodes[i];
    let p = n.up;
    p != NONE
        && p < nodes.len()
        && container(n)
        && container(&nodes[p])
        && nodes[p].line_box.is_none()
        && n.line_box.is_none()
        && n.el.direction == Direction::Column
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
        // Its lines are laid out whole: its floats sit at its top.
        for f in floats {
            place(nodes, f, x0, x1, oy, (ox, oy), cx);
        }
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
    let (size, tr, ft) = (t.step.size, t.tracking(), t.features());
    let at = |s: &str, w: f32| break_lines(s, m, size, tr, ft, t.wrap.then_some(w + 0.5));
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
        size_h(nodes, j);
        flow(nodes, j, cx.measure, cx.touched);
    }
    cx.touched.extend(order);
}
