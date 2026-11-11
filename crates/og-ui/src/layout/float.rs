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
//! (its paragraph shaped once and broken one line at a time), and a
//! block's lines of inline-level boxes (inline-block, images, inline flex
//! or grid, texts), filled one at a time into the room beside the floats,
//! a line no box fits moved down past them. A float among those lines is
//! placed at the top of the line it comes on, or of the next one when it
//! does not fit there. A block-level box among them is a block of its own
//! that spans the line box, its content around the floats (one that
//! starts a context of its own is fitted beside them). All of it runs in
//! the frame of the context's writing mode (`layout::writing`): `left` is
//! line-left and `y` the block axis.

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
    n.layout.used.block
        && n.props.layout == LayoutMode::Flex
        && matches!(n.content, Content::None)
        && n.layout.used.multicol.is_none()
}

/// Node `i` lays out in its parent's formatting context: a block in a
/// block's flow, or among its lines (in an anonymous block of its own,
/// CSS 2 §9.2.1.1), that starts none of its own (CSS 2 §9.4.1), whether
/// its children lie in a block flow or in lines.
fn joins(nodes: &[Node], i: usize) -> bool {
    let n = &nodes[i];
    let p = n.up;
    p != NONE
        && p < nodes.len()
        && container(n)
        && container(&nodes[p])
        && n.layout.line_box.unwrap_or(n.layout.used.direction) == Direction::Column
        && n.layout.used.float == FloatSide::None
        && !n.props.out_of_flow()
        && !n.layout.used.bfc
        && !n.layout.used.inline
        && !n.layout.used.scroll
        && !n.layout.used.scroll_x
}

/// An image's alpha channel on the CPU: one byte per pixel, row-major,
/// `width` x `height` ([`crate::Ui::define_texture_alpha`]). A
/// `shape-outside: url()` float's contour is read from it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImageAlpha {
    pub width: u32,
    pub height: u32,
    pub alpha: Vec<u8>,
}

/// `shape-outside: <image>` with its `shape-image-threshold`.
#[derive(Clone, Debug)]
pub(crate) struct ShapeImage {
    pub alpha: std::sync::Arc<ImageAlpha>,
    pub threshold: f32,
}

impl PartialEq for ShapeImage {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.alpha, &other.alpha) && self.threshold == other.threshold
    }
}

impl ShapeImage {
    /// The area of the pixels whose alpha is greater than the threshold
    /// (CSS Shapes 1 §6.2), the image stretched over the `w` x `h` box at
    /// `at`: each run of them in a row as its two vertical sides, runs
    /// repeated by the rows below merged into one.
    fn segments(&self, at: [f32; 2], w: f32, h: f32, out: &mut Vec<[[f32; 2]; 2]>) {
        let a = &*self.alpha;
        let (iw, ih) = (a.width as usize, a.height as usize);
        if iw == 0 || ih == 0 || a.alpha.len() < iw * ih {
            return;
        }
        let cut = self.threshold * 255.0;
        let runs = |j: usize| {
            let row = &a.alpha[j * iw..(j + 1) * iw];
            let mut runs = Vec::new();
            let mut k = 0;
            while k < iw {
                if f32::from(row[k]) > cut {
                    let start = k;
                    while k < iw && f32::from(row[k]) > cut {
                        k += 1;
                    }
                    runs.push((start, k));
                } else {
                    k += 1;
                }
            }
            runs
        };
        let (sx, sy) = (w / iw as f32, h / ih as f32);
        let mut j = 0;
        let mut row = runs(0);
        while j < ih {
            let mut end = j + 1;
            let mut next = Vec::new();
            while end < ih {
                next = runs(end);
                if next != row {
                    break;
                }
                end += 1;
            }
            // The rows cover [y0, y1): a line starting at y1 is below them.
            let y0 = at[1] + j as f32 * sy;
            let y1 = (at[1] + end as f32 * sy - 1.0 / 256.0).max(y0);
            for &(s, e) in &row {
                for x in [at[0] + s as f32 * sx, at[0] + e as f32 * sx] {
                    out.push([[x, y0], [x, y1]]);
                }
            }
            row = next;
            j = end;
        }
    }
}

/// Unresolved CSS shape geometry; layout resolves percentages after sizing.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Outside {
    pub shape: crate::doc::paint_css::CssClip,
    pub reference: crate::doc::paint_css::GeoBox,
    /// An image's contour, in place of `shape`, over the content box.
    pub image: Option<ShapeImage>,
    pub margin: crate::doc::paint_css::Pos,
    pub rtl: bool,
}

impl Outside {
    fn edges(&self, n: &Node, offset: [f32; 2], margin: f32) -> Vec<[[f32; 2]; 2]> {
        use crate::doc::paint_css::GeoBox;
        use crate::paint::ShapeKind;
        use og_core::glam::Vec2;
        let vertical = n.layout.used.vertical;
        let physical = |values: [f32; 4]| {
            let mut out = values;
            if let Some(sides) = vertical {
                for k in 0..4 {
                    out[sides[k] as usize] = values[k];
                }
            }
            out
        };
        let p = n.layout.used.padding;
        // An image lies over the content box as a replaced element's
        // picture would, stretched to it (CSS Shapes 1 §6.1).
        let reference = if self.image.is_some() {
            GeoBox::Content
        } else {
            self.reference
        };
        let insets = match reference {
            GeoBox::Border => [0.0; 4],
            GeoBox::Padding => n
                .decor
                .as_ref()
                .map_or([0.0; 4], |d| d.borders.map(|b| b.map_or(0.0, |b| b.width))),
            GeoBox::Content => physical([p.top, p.right, p.bottom, p.left]),
            GeoBox::Margin => physical([
                -span(n.layout.margin_sets[0]),
                -n.layout.margin_x[1],
                -span(n.layout.margin_sets[1]),
                -n.layout.margin_x[0],
            ]),
        };
        let radius = if n.layout.used.radius > 0.0 {
            (n.layout.used.radius - insets.iter().copied().fold(f32::INFINITY, f32::min)).max(0.0)
        } else {
            0.0
        };
        let (w, h) = if vertical.is_some() {
            (n.layout.pref_h, n.layout.w)
        } else {
            (n.layout.w, n.layout.pref_h)
        };
        let map = |p: Vec2| {
            let (mut x, y) = match vertical {
                Some([1, _, _, _]) => (p.y, w - p.x),
                Some([3, 0, _, _]) => (h - p.y, p.x),
                Some(_) => (p.y, p.x),
                None => (p.x, p.y),
            };
            if self.rtl {
                x = n.layout.w - x;
            }
            [x + offset[0], y + offset[1]]
        };
        let mut edges = Vec::new();
        if let Some(image) = &self.image {
            let [t, r, b, l] = insets;
            let mut sides = Vec::new();
            image.segments(
                [l, t],
                (w - l - r).max(0.0),
                (h - t - b).max(0.0),
                &mut sides,
            );
            edges.extend(
                sides
                    .into_iter()
                    .map(|[a, b]| [map(Vec2::from(a)), map(Vec2::from(b))]),
            );
        }
        let (points, contours) = match self.image {
            Some(_) => (Vec::new(), Vec::new()),
            None => match self.shape.resolve(w, h, (insets, radius)) {
                ShapeKind::Path {
                    points, contours, ..
                } => (points, contours),
                _ => return Vec::new(),
            },
        };
        let mut start = 0;
        for count in contours {
            let contour = &points[start..start + count as usize];
            start += count as usize;
            if contour.is_empty() {
                continue;
            }
            let mut at = Vec2::from(contour[0]);
            for c in contour[1..].as_chunks::<3>().0 {
                let line = og_draw::flatten_cubic(at, c[0].into(), c[1].into(), c[2].into(), 0.01);
                edges.extend(line.windows(2).map(|p| [map(p[0]), map(p[1])]));
                at = c[2].into();
            }
            edges.push([map(at), map(contour[0].into())]);
        }
        let (top, bottom) = edges
            .iter()
            .flatten()
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(top, bottom), p| {
                (top.min(p[1]), bottom.max(p[1]))
            });
        if top == bottom && margin == 0.0 {
            edges.clear();
        }
        edges
    }
}

/// A placed float's margin box, from its context root's content box.
#[derive(Clone, Debug)]
struct Placed {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    left: bool,
    contour: Option<Vec<[[f32; 2]; 2]>>,
    margin: f32,
}

/// The horizontal extent of a segment swept by a disk, intersected with
/// a line band. Endpoints, band crossings and circle tangencies are the
/// only extrema; expanding its bounding box would give square joins.
fn expanded_edge(a: [f32; 2], b: [f32; 2], lo: f32, hi: f32, margin: f32) -> (f32, f32) {
    let (mut left, mut right) = (f32::INFINITY, f32::NEG_INFINITY);
    let mut include = |p: [f32; 2]| {
        let distance = (lo - p[1]).max(p[1] - hi).max(0.0);
        if distance <= margin {
            let radius = (margin * margin - distance * distance).max(0.0).sqrt();
            left = left.min(p[0] - radius);
            right = right.max(p[0] + radius);
        }
    };
    include(a);
    include(b);
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    if dy != 0.0 {
        let tangent = margin * dx * dy.signum() / dx.hypot(dy);
        for y in [
            lo - margin,
            lo,
            lo + margin,
            hi - margin,
            hi,
            hi + margin,
            lo - tangent,
            lo + tangent,
            hi - tangent,
            hi + tangent,
        ] {
            let t = (y - a[1]) / dy;
            if (0.0..=1.0).contains(&t) {
                include([a[0] + t * dx, y]);
            }
        }
    }
    (left, right)
}

/// Only inline content uses the contour. Float placement, BFC avoidance
/// and clearance keep the rectangular margin box (CSS Shapes 1 §2).
fn line_room(floats: &[Placed], y0: f32, y1: f32, x0: f32, x1: f32) -> (f32, f32, Option<f32>) {
    let (mut l, mut r, mut next) = (x0, x1, None::<f32>);
    for f in floats {
        let (lo, hi) = (y0.max(f.y), y1.min(f.y + f.h));
        if hi <= lo {
            continue;
        }
        let (mut left, mut right) = (f.x, f.x + f.w);
        if let Some(edges) = &f.contour {
            (left, right) = (f32::INFINITY, f32::NEG_INFINITY);
            for &[a, b] in edges {
                if a[1].max(b[1]) + f.margin < lo || a[1].min(b[1]) - f.margin > hi {
                    continue;
                }
                let (a, b) = expanded_edge(a, b, lo, hi, f.margin);
                left = left.min(a);
                right = right.max(b);
            }
        }
        if left > right {
            continue;
        }
        let left = left.clamp(f.x, f.x + f.w.max(0.0));
        let right = right.clamp(f.x, f.x + f.w.max(0.0));
        if !(if f.left { right > x0 } else { left < x1 }) {
            continue;
        }
        if f.left {
            l = l.max(right);
        } else {
            r = r.min(left);
        }
        let bottom = f.y + f.h;
        let b = if f.contour.is_some() {
            (y0 + 1.0 / 64.0).min(bottom)
        } else {
            bottom
        };
        next = Some(next.map_or(b, |n| n.min(b)));
    }
    (l, r, next)
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
        || !n.layout.wrap_lines.is_empty()
        || n.children.iter().any(|&k| {
            let k_n = &nodes[k];
            k_n.layout.beside
                || k_n.layout.clearance != 0.0
                || !k_n.layout.wrap_lines.is_empty()
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
        if nodes[i].layout.float_h != 0.0 {
            nodes[i].layout.float_h = 0.0;
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
    nodes[i].layout.float_h = bottom;
    size_h(nodes, i);
    cx.touched.push(i);
}

/// Container `c`'s flow with its content box at `(ox, oy)` in the root's.
fn walk(nodes: &mut [Node], c: usize, ox: f32, oy: f32, cx: &mut Context<'_, '_>) {
    let n = &nodes[c];
    let inner = (n.layout.w - n.layout.used.padding.horizontal()).max(0.0);
    let (x0, x1) = (ox, ox + inner);
    let floats = n.floats.clone();
    if n.layout.line_box.is_some() {
        // Each float where it is in the content: before which of the
        // boxes in lines it comes.
        let (kids, order) = (&n.children, &n.ordered_children);
        let at = floats
            .iter()
            .map(|&f| {
                let before = order.iter().position(|&k| k == f).map_or(0, |p| {
                    order[..p].iter().filter(|k| kids.contains(k)).count()
                });
                (before, f)
            })
            .collect();
        lines(nodes, c, x0, x1, oy, at, cx);
        return;
    }
    let kids = nodes[c].children.clone();
    for &k in &kids {
        nodes[k].layout.clearance = 0.0;
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
        let clear = (cleared(&cx.floats, nodes[k].layout.used.clear) - top).max(0.0);
        let mut at = top + clear;
        let was = nodes[k].layout.beside;
        nodes[k].layout.beside = false;
        if joins(nodes, k) {
            let kx = child_x(nodes, c, k, x0, inner);
            let pad = nodes[k].layout.used.padding;
            walk(nodes, k, kx + pad.left, at + pad.top, cx);
            size_h(nodes, k);
            nodes[k].layout.beside = !cx.floats.is_empty();
        } else if wraps(&nodes[k]) {
            text(nodes, k, x0, inner, at, cx, was);
        } else if rich_wraps(&nodes[k]) {
            rich(nodes, k, x0, inner, at, cx, was);
        } else {
            at = beside(nodes, k, x0, x1, at, cx);
        }
        nodes[k].layout.clearance = at - top;
        cx.touched.push(k);
        y = at + nodes[k].layout.pref_h;
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
    let el = nodes[c].layout.used;
    let n = &nodes[k];
    let fill =
        (inner - n.layout.used.margin[1].of(inner) - n.layout.used.margin[3].of(inner)).max(0.0);
    let basis = match n.layout.used.width {
        Size::Grow { .. } | Size::Fit { .. } => fill,
        _ => inner,
    };
    let stretch = !el.wrap && n.layout.used.align_self.unwrap_or(el.cross) == Cross::Stretch;
    let w = cross_size(n, 0, basis, inner, stretch);
    let w = match n.layout.used.width {
        Size::Fit { .. } if el.scroll_x => w.max(n.layout.pref_w),
        _ => w,
    };
    if !wraps(n) && w != n.layout.w {
        relayout(nodes, k, w, cx);
    }
    nodes[k].layout.w = w;
    nodes[k].layout.margin_x = place_sides(&nodes[k], inner, fill - w);
}

/// Where container `c` puts child `k`'s border box across (as
/// `place_children` does).
fn child_x(nodes: &[Node], c: usize, k: usize, x0: f32, inner: f32) -> f32 {
    let el = nodes[c].layout.used;
    let n = &nodes[k];
    let [ml, mr] = n.layout.margin_x;
    if n.layout.used.margin[1].auto || n.layout.used.margin[3].auto {
        return x0 + ml;
    }
    let safe = el.safe & 2 != 0 || n.layout.used.safe & 2 != 0;
    x0 + ml
        + cross_offset(
            n.layout.used.align_self.unwrap_or(el.cross),
            inner - ml - mr,
            n.layout.w,
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
    let [ml, mr] = n.layout.margin_x;
    let (mt, mb) = (span(n.layout.margin_sets[0]), span(n.layout.margin_sets[1]));
    let (w, h) = (n.layout.w + ml + mr, n.layout.pref_h + mt + mb);
    let left = n.layout.used.float != FloatSide::Right;
    // No higher than an earlier float (rule 5), below those it clears.
    let mut y = y
        .max(cx.floats.last().map_or(f32::NEG_INFINITY, |p| p.y))
        .max(cleared(&cx.floats, n.layout.used.clear));
    let x = loop {
        let (l, r, next) = room(&cx.floats, y, y + h, x0, x1);
        match next {
            Some(b) if r - l < w - 0.01 => y = b,
            _ => break if left { l } else { r - w },
        }
    };
    let margin = n
        .shape_outside
        .as_ref()
        .map_or(0.0, |s| s.margin.at(x1 - x0));
    let contour = n
        .shape_outside
        .as_ref()
        .map(|s| s.edges(n, [x + ml, y + mt], margin));
    cx.floats.push(Placed {
        x,
        y,
        w,
        h,
        left,
        contour,
        margin,
    });
    nodes[f].layout.float_at = [x - origin.0 + ml, y - origin.1 + mt];
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
    let [ml, mr] = n.layout.margin_x;
    let pad = n.layout.used.padding;
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
        let (l, r, next) = line_room(&cx.floats, y, y + lh, tx0, tx1);
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
        nodes[k].layout.w = fill;
        nodes[k].layout.margin_x = [ml, mr];
        nodes[k].layout.lines = lines;
        nodes[k].layout.wrap_lines = shifts;
        nodes[k].layout.beside = true;
        size_h(nodes, k);
    } else if was || !nodes[k].layout.wrap_lines.is_empty() {
        // Clear of floats: its lines as its width breaks them.
        nodes[k].layout.wrap_lines.clear();
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
/// [`text`] lays out a plain one: its paragraph shaped once and broken one
/// line at a time at the room the floats beside each line leave, one no
/// word fits moved down past them.
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
    let [ml, mr] = n.layout.margin_x;
    let pad = n.layout.used.padding;
    let fill = (inner - ml - mr).max(0.0);
    let (tx0, tx1) = (x0 + ml + pad.left, x0 + ml + fill - pad.right);
    let lowest = cx
        .floats
        .iter()
        .map(|f| f.y + f.h)
        .fold(f32::NEG_INFINITY, f32::max);
    let fonts = cx.measure.fonts;
    // The room beside a line at `y` as tall as `h`: its left, its width,
    // and the highest bottom of the floats that narrow it.
    let floats = &cx.floats;
    let room_at = |y: f32, h: f32| {
        if y >= lowest {
            return (tx0, (tx1 - tx0).max(0.0), None);
        }
        let (l, r, next) = line_room(floats, y, y + h, tx0, tx1);
        (l, (r - l).max(0.0), next)
    };
    let y = top + pad.top;
    // Down a vertical line its units take their vertical lengths, which the
    // shaper does not lay out.
    let v = n.rich_vertical();
    #[cfg(feature = "shaping")]
    let broken = v
        .is_none()
        .then(|| shaped_lines(rc, fonts, y, room_at))
        .flatten()
        .unwrap_or_else(|| span_lines(rc, fonts, y, lowest, room_at, v));
    #[cfg(not(feature = "shaping"))]
    let broken = span_lines(rc, fonts, y, lowest, room_at, v);
    let (lines, shifts, narrowed, drop) = broken;
    let shifts = shifts
        .into_iter()
        .map(|[l, d, w]| [l - tx0, d, w])
        .collect();
    if narrowed || drop > 0.0 {
        nodes[k].layout.w = fill;
        nodes[k].layout.margin_x = [ml, mr];
        nodes[k].layout.rich_lines = lines;
        nodes[k].layout.wrap_lines = shifts;
        nodes[k].layout.beside = true;
        size_h(nodes, k);
    } else if was || !nodes[k].layout.wrap_lines.is_empty() {
        // Clear of floats: its lines as its width breaks them.
        nodes[k].layout.wrap_lines.clear();
        distribute_w(nodes, k, cx.measure);
        size_h(nodes, k);
    }
}

/// A rich text's lines beside floats from its top `y` (`room_at`: a
/// line's left, width and the floats' next bottom): the lines, each one's
/// left, drop and width, whether a float narrowed one, and the drop.
type Broken = (Vec<RichLine>, Vec<[f32; 3]>, bool, f32);

/// [`rich`]'s lines from its paragraph shaped once
/// ([`crate::text::paragraph::Breaker`]): bidi and soft hyphens see the
/// whole paragraph. `None` when it cannot be shaped.
#[cfg(feature = "shaping")]
fn shaped_lines(
    rc: &RichContent,
    fonts: &dyn Metrics,
    mut y: f32,
    room_at: impl Fn(f32, f32) -> (f32, f32, Option<f32>),
) -> Option<Broken> {
    let mut breaker = crate::text::paragraph::Breaker::new(rc, fonts)?;
    let guess = rc.base.0.line_height;
    let (mut lines, mut shifts) = (Vec::new(), Vec::new());
    let (mut drop, mut narrowed) = (0.0f32, false);
    loop {
        // The room beside the line as tall as the line it holds.
        let mut h = guess;
        let (step, got, l, avail, next) = loop {
            let (l, avail, next) = room_at(y, h);
            let step = breaker.next(avail);
            let got = breaker.lines(&step, fonts)?;
            let height: f32 = got.iter().map(|l| l.height).sum();
            if height <= h + 0.01 || next.is_none() {
                break (step, got, l, avail, next);
            }
            h = height;
        };
        let width = got.iter().map(|l| l.width).fold(0.0, f32::max);
        if let Some(b) = next
            && width > avail + 0.5
        {
            drop += b - y;
            y = b;
            continue;
        }
        narrowed |= next.is_some();
        for line in got {
            y += line.height;
            shifts.push([l, drop, avail]);
            lines.push(line);
        }
        if breaker.take(&step) {
            break;
        }
    }
    Some((lines, shifts, narrowed, drop))
}

/// [`rich`]'s lines where its faces cannot be shaped (fixed or atlas-only
/// metrics) or run down a vertical line `v`: each line broken from the
/// spans left; below `lowest` (the floats' bottom) the rest at once.
fn span_lines(
    rc: &RichContent,
    fonts: &dyn Metrics,
    mut y: f32,
    lowest: f32,
    room_at: impl Fn(f32, f32) -> (f32, f32, Option<f32>),
    v: Option<crate::layout::flow::VerticalText>,
) -> Broken {
    let break_at = |rest: &[(usize, RichSpan)], w: f32| {
        let part = RichContent {
            spans: rest.iter().map(|s| s.1.clone()).collect(),
            ..rc.clone()
        };
        crate::rich::rich_lines(&part, fonts, Some(w + 0.5), v)
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
    let (mut drop, mut narrowed) = (0.0f32, false);
    while !rest.is_empty() {
        if y >= lowest {
            let (l0, full, _) = room_at(y, guess);
            for l in break_at(&rest, full) {
                shifts.push([l0, drop, full]);
                lines.push(renumber(&rest, l));
            }
            break;
        }
        // The room beside the line as tall as the line it holds.
        let mut h = guess;
        let (line, l, avail, next) = loop {
            let (l, avail, next) = room_at(y, h);
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
        shifts.push([l, drop, avail]);
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
            let (l, w, _) = room_at(y, guess);
            shifts.push([l, drop, w]);
            lines.push(RichLine {
                height: guess,
                ..RichLine::default()
            });
        }
    }
    (lines, shifts, narrowed, drop)
}

/// A float's margin box's width.
fn float_w(n: &Node) -> f32 {
    n.layout.w + n.layout.margin_x[0] + n.layout.margin_x[1]
}

/// Line-box container `c` with its content box from `x0` to `x1` and its
/// top at `top`: its inline-level children filled into lines one at a
/// time, each into the room the floats beside it leave (CSS 2 §9.5), one
/// whose first box does not fit moved down past them. Its floats
/// (`floats`: each with the number of boxes before it) are placed where
/// they come: at the top of the line they are on when they fit beside
/// what it already holds, else at the next line's (§9.5.1 rules 6 and 8).
/// A block-level box among the lines is a block of its own (CSS 2
/// §9.2.1.1): it spans the line box and its content flows around the
/// floats; one that starts a context of its own is fitted beside them.
#[allow(clippy::too_many_arguments)]
fn lines(
    nodes: &mut [Node],
    c: usize,
    x0: f32,
    x1: f32,
    top: f32,
    mut floats: std::collections::VecDeque<(usize, usize)>,
    cx: &mut Context<'_, '_>,
) {
    let origin = (x0, top);
    let kids = nodes[c].children.clone();
    let inner = (x1 - x0).max(0.0);
    let el = nodes[c].layout.used;
    let (gap, line_gap) = (el.main_gap(), el.line_gap());
    let (base, sides) = row_bases(nodes, &kids, inner);
    let css = css_line(nodes, &kids);
    // A block-level box (or the box after one) starts a line.
    let starts: Vec<bool> = (0..kids.len())
        .map(|k| k > 0 && !(inline_level(&nodes[kids[k]]) && inline_level(&nodes[kids[k - 1]])))
        .collect();
    let outer = |k: usize| base[k] + sides[k];
    let tall =
        |nodes: &[Node], k: usize| nodes[kids[k]].layout.pref_h + cross_room(&nodes[kids[k]]);
    let (mut rows, mut shifts) = (Vec::new(), Vec::new());
    let (mut s, mut y, mut drop, mut narrowed) = (0, top, 0.0f32, false);
    // Floats that did not fit on the line they came on: at the next one's top.
    let mut later: Vec<usize> = Vec::new();
    while s < kids.len() {
        for f in later.drain(..) {
            place(nodes, f, x0, x1, y, origin, cx);
        }
        while let Some(&(b, f)) = floats.front()
            && b <= s
        {
            floats.pop_front();
            place(nodes, f, x0, x1, y, origin, cx);
        }
        if !inline_level(&nodes[kids[s]]) {
            let k = kids[s];
            // Below the earlier floats it clears.
            let clear = (cleared(&cx.floats, nodes[k].layout.used.clear) - y).max(0.0);
            (drop, y) = (drop + clear, y + clear);
            let w0 = nodes[k].layout.w;
            size_row(
                nodes,
                &kids[s..s + 1],
                css,
                &base[s..s + 1],
                &sides[s..s + 1],
                gap,
                inner,
                inner,
            );
            let was = nodes[k].layout.beside;
            nodes[k].layout.beside = false;
            let at = y + span(nodes[k].layout.margin_sets[0]);
            if joins(nodes, k) || wraps(&nodes[k]) || rich_wraps(&nodes[k]) {
                if nodes[k].layout.w != w0 {
                    let w = nodes[k].layout.w;
                    relayout(nodes, k, w, cx);
                }
                let [ml, _] = nodes[k].layout.margin_x;
                let pad = nodes[k].layout.used.padding;
                if joins(nodes, k) {
                    walk(nodes, k, x0 + ml + pad.left, at + pad.top, cx);
                    size_h(nodes, k);
                    nodes[k].layout.beside = !cx.floats.is_empty();
                } else {
                    // `text` and `rich` lay it out at its margins in the line box.
                    let margins = nodes[k].layout.margin_x;
                    if wraps(&nodes[k]) {
                        text(nodes, k, x0, inner, at, cx, was);
                    } else {
                        rich(nodes, k, x0, inner, at, cx, was);
                    }
                    nodes[k].layout.margin_x = margins;
                }
                narrowed |= nodes[k].layout.beside;
                rows.push((s, s + 1));
                shifts.push([0.0, drop, inner]);
                nodes[c].layout.rows.clone_from(&rows);
                inline_rows(nodes, c, cx.measure);
                let lh = nodes[c]
                    .layout
                    .inline_rows
                    .get(rows.len() - 1)
                    .copied()
                    .flatten()
                    .unwrap_or_else(|| tall(nodes, s));
                y += lh + line_gap;
                s += 1;
                continue;
            }
            nodes[k].layout.w = w0;
        }
        // The room beside the line as tall as the boxes it holds; a float
        // that comes on it and fits beside what it holds so far is placed
        // at its top, and the line filled again.
        let mut h = tall(nodes, s);
        let (e, l, avail, next) = 'fit: loop {
            let (l, r, next) = line_room(&cx.floats, y, y + h, x0, x1);
            let avail = (r - l).max(0.0);
            let (mut e, mut used) = (s + 1, outer(s));
            loop {
                if let Some(&(b, f)) = floats.front()
                    && b == e
                {
                    floats.pop_front();
                    if used + float_w(&nodes[f]) <= avail + 0.5 {
                        place(nodes, f, x0, x1, y, origin, cx);
                        continue 'fit;
                    }
                    later.push(f);
                    continue;
                }
                if e < kids.len() && !starts[e] && used + gap + outer(e) <= avail + 0.5 {
                    used += gap + outer(e);
                    e += 1;
                } else {
                    break;
                }
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
        let was: Vec<f32> = line.iter().map(|&k| nodes[k].layout.w).collect();
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
            if nodes[k].layout.w != w0 {
                let w = nodes[k].layout.w;
                relayout(nodes, k, w, cx);
            }
        }
        rows.push((s, e));
        shifts.push([l - x0, drop, avail]);
        // Its height as its line box holds it (`inline_rows`).
        nodes[c].layout.rows.clone_from(&rows);
        inline_rows(nodes, c, cx.measure);
        let lh = nodes[c]
            .layout
            .inline_rows
            .get(rows.len() - 1)
            .copied()
            .flatten()
            .unwrap_or_else(|| (s..e).map(|k| tall(nodes, k)).fold(0.0, f32::max));
        y += lh + line_gap;
        s = e;
    }
    // What comes after the last box: at the last line's top when it fit
    // there (placed above), else below it.
    for f in later.into_iter().chain(floats.into_iter().map(|(_, f)| f)) {
        place(
            nodes,
            f,
            x0,
            x1,
            y - if rows.is_empty() { 0.0 } else { line_gap },
            origin,
            cx,
        );
    }
    nodes[c].layout.rows = rows;
    inline_rows(nodes, c, cx.measure);
    if narrowed || drop > 0.0 {
        nodes[c].layout.wrap_lines = shifts;
        nodes[c].layout.beside = true;
    } else {
        nodes[c].layout.wrap_lines.clear();
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
    let [ml, mr] = n.layout.margin_x;
    let w0 = n.layout.w;
    let fill = (x1 - x0 - ml - mr).max(0.0);
    // A box that fills the line narrows to the room; another keeps its width.
    let fills = w0 >= fill - 0.01;
    let need = if fills { n.layout.min_w } else { w0 };
    let h = n.layout.pref_h;
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
            nodes[k].layout.margin_x = if up != NONE && nodes[up].layout.used.flip {
                [x1 - br, bl - x0]
            } else {
                [bl - x0, x1 - br]
            };
            nodes[k].layout.beside = true;
            return y;
        }
        y = b;
    }
    nodes[k].layout.beside = y != top;
    y
}

/// Lays subtree `k` out again at width `w`: its widths, then its heights
/// (with the floats of the contexts in it).
fn relayout(nodes: &mut [Node], k: usize, w: f32, cx: &mut Context<'_, '_>) {
    nodes[k].layout.w = w;
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
