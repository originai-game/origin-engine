use super::*;
use crate::element::Position;

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
    og_core::sort::sort_by(&mut points, f32::total_cmp);
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
            // Its max never cuts under its content's least width (a wrapping
            // text's longest word): a word is never split between characters
            // because a box's max was narrower than it.
            let p = pref.clamp(lo, hi).max(min.min(pref));
            (p, min.max(lo).min(p))
        }
        Size::Percent(_) | Size::Calc(_) => (pref, min.min(pref)),
        Size::MinContent => (min, min),
        Size::MaxContent => (pref, pref),
    }
}

fn gaps(gap: f32, n: usize) -> f32 {
    gap * n.saturating_sub(1) as f32
}

/// A collapsing margin set (CSS 2 §8.3.1): its largest positive and its
/// most negative margin; it spans their sum.
pub(super) type MarginSet = (f32, f32);

fn margin_set(m: f32) -> MarginSet {
    (m.max(0.0), m.min(0.0))
}

fn joined(a: MarginSet, b: MarginSet) -> MarginSet {
    (a.0.max(b.0), a.1.min(b.1))
}

pub(super) fn span(m: MarginSet) -> f32 {
    m.0 + m.1
}

/// The width node `i`'s margin percentages read: its parent's content
/// width (CSS 2 §8.3: all four, even the vertical ones).
pub(super) fn margin_basis(nodes: &[Node], i: usize) -> f32 {
    let p = nodes[i].up;
    if p == NONE || p >= nodes.len() {
        return 0.0;
    }
    (nodes[p].w - nodes[p].el.padding.horizontal()).max(0.0)
}

/// Node `i`'s collapsed outer margins, and whether its first child's top
/// and last child's bottom margin collapse through its own edges: it is a
/// CSS block in a block parent's flow (so no new formatting context), with
/// no padding at that edge, and at the bottom no set height. An empty block
/// in a block's flow collapses through itself: its own top and bottom
/// margins, and those of its children, are one set (CSS 2 §8.3.1).
fn collapse_margins(nodes: &mut [Node], i: usize) {
    let n = &nodes[i];
    let p = n.up;
    let in_block_flow = p != NONE
        && p < nodes.len()
        && nodes[p].el.block
        && nodes[p].el.direction == Direction::Column
        && nodes[p].props.layout == LayoutMode::Flex
        && nodes[p].children.contains(&i);
    // A flow-root (or a box laying out lines) starts a formatting
    // context: no margin collapses through its edges (CSS 2 §8.3.1).
    let flows = in_block_flow
        && n.el.block
        && !n.el.bfc
        && n.el.direction == Direction::Column
        && n.props.layout == LayoutMode::Flex
        && matches!(n.content, Content::None)
        && !n.el.scroll
        && !n.el.scroll_x;
    let auto_height = matches!(n.el.height, Size::Fit { min, .. } if min <= 0.0);
    let empty_text = matches!(&n.content, Content::Text(t) if t.text.is_empty());
    let through = in_block_flow
        && !n.el.bfc
        && auto_height
        && n.el.padding.top == 0.0
        && n.el.padding.bottom == 0.0
        && n.out.is_empty()
        && ((flows && n.children.iter().all(|&c| nodes[c].through)) || empty_text);
    let basis = margin_basis(nodes, i);
    let edge = |m: crate::layout::Margin| margin_set(m.of(basis));
    let mut sets = [edge(n.el.margin[0]), edge(n.el.margin[2])];
    let collapse_in = if through {
        [true, true]
    } else {
        let kids = !n.children.is_empty();
        [
            flows && kids && n.el.padding.top == 0.0,
            flows && kids && n.el.padding.bottom == 0.0 && auto_height,
        ]
    };
    // The run of margins that meets an edge: each child that collapses
    // through itself passes on to the next.
    let chain = |kids: &mut dyn Iterator<Item = &usize>, side: usize| {
        let mut set = (0.0, 0.0);
        for &c in kids {
            set = joined(set, nodes[c].margin_sets[side]);
            if !nodes[c].through {
                break;
            }
        }
        set
    };
    if collapse_in[0] {
        sets[0] = joined(sets[0], chain(&mut n.children.iter(), 0));
    }
    if collapse_in[1] {
        sets[1] = joined(sets[1], chain(&mut n.children.iter().rev(), 1));
    }
    if through {
        sets = [joined(sets[0], sets[1]); 2];
    }
    nodes[i].margin_sets = sets;
    nodes[i].collapse_in = collapse_in;
    nodes[i].through = through;
}

/// The room a column's margins and gaps take around its children: before
/// the first, between each pair, and after the last. A block's adjacent
/// margins collapse, through any child that collapses through itself; a
/// flex column's add.
pub(super) fn column_spacing(nodes: &[Node], i: usize) -> (f32, Vec<f32>, f32) {
    let n = &nodes[i];
    let kids = &n.children;
    if kids.is_empty() {
        return (0.0, Vec::new(), 0.0);
    }
    // `run` is the margin set open after the last child, `used` the room
    // already given to it, and `outside` that the run collapses through
    // the column's own top edge.
    let mut run = (0.0, 0.0);
    let mut used = 0.0;
    let mut outside = n.collapse_in[0];
    let mut before = Vec::with_capacity(kids.len());
    for (j, &c) in kids.iter().enumerate() {
        let k = &nodes[c];
        let gap = if j > 0 { n.el.main_gap() } else { 0.0 };
        if n.el.block {
            let set = joined(run, k.margin_sets[0]);
            let total = if outside { 0.0 } else { span(set) };
            before.push(gap + total - used + k.clearance);
            if k.through {
                (run, used) = (set, total);
            } else {
                (run, used, outside) = (k.margin_sets[1], 0.0, false);
            }
        } else {
            before.push(gap + span(run) + span(k.margin_sets[0]));
            run = k.margin_sets[1];
        }
    }
    let trail = if n.collapse_in[1] {
        0.0
    } else if outside {
        // Every child collapsed through the top edge.
        0.0
    } else {
        span(run) - used
    };
    let lead = before.remove(0);
    (lead, before, trail)
}

fn column_room(nodes: &[Node], i: usize) -> f32 {
    let (lead, between, trail) = column_spacing(nodes, i);
    lead + between.iter().sum::<f32>() + trail
}

/// The room a child's left and right margins take in its parent's
/// intrinsic width: their px (a percentage and `auto` count 0, as CSS
/// counts them in a min- or max-content contribution).
pub(super) fn side_margins(n: &Node) -> f32 {
    n.el.margin[1].intrinsic() + n.el.margin[3].intrinsic()
}

/// A child's left and right margins against `basis`, with its `auto` ones
/// taking `free` (CSS 2 §10.3.3 in a block, CSS Flexbox §8.1 in a line):
/// split between two, all to one. Free space under 0 gives them none.
pub(super) fn place_sides(n: &Node, basis: f32, free: f32) -> [f32; 2] {
    let [l, r] = [n.el.margin[3], n.el.margin[1]];
    let free = free.max(0.0);
    match (l.auto, r.auto) {
        (true, true) => [free / 2.0, free / 2.0],
        (true, false) => [free, r.of(basis)],
        (false, true) => [l.of(basis), free],
        (false, false) => [l.of(basis), r.of(basis)],
    }
}

/// The room a child's own vertical margins take across a row.
fn cross_margins(n: &Node) -> f32 {
    span(n.margin_sets[0]) + span(n.margin_sets[1])
}

/// The room its own vertical margins take in a grid row.
pub(super) fn cross_room(n: &Node) -> f32 {
    cross_margins(n)
}

/// Greedy `lines` at `width` chosen again by `style` (CSS Text 4 §6.2):
/// `balance` breaks at the narrowest width that keeps their count, so the
/// lines come out as even as it allows; `pretty` breaks at the widest width
/// that keeps the count and gives the last line two words or more, when the
/// greedy last line is one word. `at` breaks at a width.
fn rewrap(
    lines: Vec<Line>,
    style: WrapStyle,
    width: f32,
    at: impl Fn(f32) -> Vec<Line>,
) -> Vec<Line> {
    let n = lines.len();
    let words = |l: &[Line]| l.last().map_or(0, |l| l.text.split_whitespace().count());
    if style == WrapStyle::Pretty && words(&lines) > 1 {
        return lines;
    }
    // The narrowest width keeping `n` lines, to half a CSS px.
    let (mut lo, mut hi) = (0.0f32, width);
    while hi - lo > 0.5 {
        let mid = (lo + hi) / 2.0;
        if at(mid).len() <= n {
            hi = mid
        } else {
            lo = mid
        }
    }
    if style == WrapStyle::Balance {
        return at(hi);
    }
    // Pretty: the widest width from there that still ends in two words.
    let ok = |w: f32| {
        let l = at(w);
        (l.len() == n && words(&l) > 1).then_some(l)
    };
    let Some(mut best) = ok(hi) else { return lines };
    let (mut lo, mut top) = (hi, width);
    while top - lo > 0.5 {
        let mid = (lo + top) / 2.0;
        match ok(mid) {
            Some(l) => {
                best = l;
                lo = mid;
            }
            None => top = mid,
        }
    }
    best
}

/// A text's preferred width (its widest line) and the least it can take:
/// its ellipsis, its longest word when it wraps, else all of it.
fn text_widths(t: &TextContent, fonts: &dyn Metrics) -> (f32, f32) {
    let m = t.metrics(fonts);
    let pref = widest_line(&t.text, m, t.step.size, t.tracking(), t.features());
    let min = if t.ellipsis {
        // It can shrink to its ellipsis.
        m.measure_with(
            t.cut.as_deref().unwrap_or_else(|| ellipsis_of(m)),
            t.step.size,
            t.tracking(),
            t.features(),
        )
        .min(pref)
    } else if t.wrap {
        longest_word(&t.text, m, t.step.size, t.tracking(), t.features())
    } else {
        pref
    };
    (pref, min)
}

/// A text shows whole at `width` at its step: one line no wider, or (wrapping)
/// no word wider and no more lines than its limit.
fn fits(t: &TextContent, fonts: &dyn Metrics, width: f32) -> bool {
    let m = t.metrics(fonts);
    let (size, tr, ft) = (t.step.size, t.tracking(), t.features());
    if !t.wrap {
        return widest_line(&t.text, m, size, tr, ft) <= width;
    }
    longest_word(&t.text, m, size, tr, ft) <= width
        && (t.max_lines == 0
            || break_lines(&t.text, m, size, tr, ft, Some(width)).len() <= t.max_lines as usize)
}

pub(super) fn size_w(nodes: &mut [Node], i: usize, measure: &mut Measure<'_>) {
    if nodes[i].props.layout == LayoutMode::Grid
        && matches!(nodes[i].content, Content::None)
        && !nodes[i].cv_skip
    {
        place_grid(nodes, i);
    }
    let n = &nodes[i];
    let fonts = measure.fonts;
    let (pref, min) = match &n.content {
        // Size containment: sized as if empty, or as its
        // `contain-intrinsic-width` says (CSS Containment 2 §3.1).
        _ if let Some(w) = n.contained_size(0) => (w, w),
        // Text on a path is as wide as its path.
        Content::Text(_) if let Some(tp) = &n.props.text_path => {
            let w = tp.extent().map_or(0.0, |e| e.x);
            (w, w)
        }
        Content::Text(t) => {
            let (pref, min) = measure.texts.widths(t, || text_widths(t, fonts));
            match t.fit.last() {
                // An auto-fit label can shrink to what its smallest step needs.
                Some(&small) => {
                    let s = TextContent {
                        step: small,
                        ..t.clone()
                    };
                    let (_, m) = measure.texts.widths(&s, || text_widths(&s, fonts));
                    (pref, min.min(m))
                }
                None => (pref, min),
            }
        }
        Content::Rich(rc) => crate::rich::rich_widths(rc, fonts),
        Content::Image { .. } => (0.0, 0.0),
        Content::None if n.props.layout == LayoutMode::Stack => {
            let kids = || n.children.iter().map(|&c| &nodes[c]);
            (
                kids().map(|c| c.pref_w).fold(0.0, f32::max),
                kids().map(|c| c.min_w).fold(0.0, f32::max),
            )
        }
        Content::None if n.props.layout == LayoutMode::Grid => grid_intrinsic(nodes, i, true),
        Content::None => {
            let k = n.children.len();
            let kids = n.children.iter().map(|&c| &nodes[c]);
            match n.el.direction {
                // Lines: its widest run of inline-level children, or its
                // widest block-level one (each on a line of its own).
                Direction::Row if n.line_box.is_some() => {
                    let gap = n.el.main_gap();
                    let (mut pref, mut run, mut count) = (0.0f32, 0.0f32, 0usize);
                    for &c in &n.children {
                        let k = &nodes[c];
                        let w = k.pref_w + side_margins(k);
                        if inline_level(k) {
                            run += w + if count > 0 { gap } else { 0.0 };
                            count += 1;
                        } else {
                            pref = pref.max(run).max(w);
                            (run, count) = (0.0, 0);
                        }
                    }
                    let min = kids.map(|c| c.min_w + side_margins(c)).fold(0.0, f32::max);
                    (pref.max(run), min)
                }
                Direction::Row => {
                    let gap = n.el.main_gap();
                    let pref = kids
                        .clone()
                        .map(|c| c.pref_w + side_margins(c))
                        .sum::<f32>()
                        + gaps(gap, k);
                    let min = if n.el.wrap {
                        kids.map(|c| c.min_w + side_margins(c)).fold(0.0, f32::max)
                    } else {
                        kids.map(|c| c.min_w + side_margins(c)).sum::<f32>() + gaps(gap, k)
                    };
                    (pref, min)
                }
                Direction::Column => {
                    let pref = kids
                        .clone()
                        .map(|c| c.pref_w + side_margins(c))
                        .fold(0.0, f32::max);
                    let min = kids.map(|c| c.min_w + side_margins(c)).fold(0.0, f32::max);
                    match n.el.multicol {
                        Some(m) => super::multicol::intrinsic(&m, pref, min),
                        None => (pref, min),
                    }
                }
            }
        }
    };
    // Floats lie beside the flow's lines (CSS 2 §9.5): at most all of
    // them on one line with it.
    let (pref, min) = if n.floats.is_empty() {
        (pref, min)
    } else {
        let f = n.floats.iter().map(|&c| &nodes[c]);
        (
            pref + f.clone().map(|c| c.pref_w + side_margins(c)).sum::<f32>(),
            f.map(|c| c.min_w + side_margins(c)).fold(min, f32::max),
        )
    };
    // A fixed height and an aspect ratio give the width.
    let (pref, min) = match (n.el.aspect > 0.0, n.el.height, n.el.width) {
        (true, Size::Fixed(h), Size::Fit { .. }) => {
            let w = (h - n.el.padding.vertical()).max(0.0) * n.el.aspect;
            (w, w)
        }
        _ => (pref, min),
    };
    let pad = n.el.padding.horizontal();
    // A horizontal scroller never shrinks its content: it overflows and
    // scrolls. A field scrolls its own text too: its placeholder or its value
    // never holds it wider than its row.
    let min = if n.el.scroll_x || matches!(n.widget, Widget::Field(_)) {
        0.0
    } else {
        min
    };
    let (p, m) = apply_size(n.el.width, pad + pref, pad + min);
    let (cp, cm) = (pad + pref, pad + min);
    let el = n.el;
    nodes[i].min_content[0] = cm;
    nodes[i].pref_w = el.bounded(0, None, cp, cm, p);
    nodes[i].min_w = el.bounded(0, None, cp, cm, m);
}

/// A child's width across a column (or height across a row), within its
/// bounds against `basis` (the parent's inner size, its percentages' basis).
pub(super) fn cross_size(n: &Node, axis: usize, inner: f32, basis: f32, stretch: bool) -> f32 {
    let (size, pref, min) = if axis == 0 {
        (n.el.width, n.pref_w, n.min_w)
    } else {
        (n.el.height, n.pref_h, n.min_h)
    };
    let v = match size {
        Size::Grow { .. } => {
            let (lo, hi) = size.bounds();
            inner.clamp(lo, hi)
        }
        Size::Fit { .. } if stretch => {
            let (lo, hi) = size.bounds();
            inner.clamp(lo, hi)
        }
        Size::Fit { .. } => pref.min(inner).max(min),
        _ => size.definite(pref, min, Some(basis)).unwrap_or(pref),
    };
    n.el.bounded(axis, Some(basis), pref, min, v)
}

/// Whether node `n` lies in a line beside its siblings (CSS 2 §9.2.2): a
/// box displayed `inline-*`, a text that is not block-level, or an image.
pub(super) fn inline_level(n: &Node) -> bool {
    n.el.inline
        || match n.content {
            Content::Text(_) | Content::Rich(_) => !n.el.block,
            Content::Image { .. } => true,
            Content::None => false,
        }
}

/// Node `c`'s size along `axis` held to its bounds against `basis`.
pub(super) fn clamp_to(n: &Node, axis: usize, basis: Option<f32>, v: f32) -> f32 {
    let (pref, min) = if axis == 0 {
        (n.pref_w, n.min_w)
    } else {
        (n.pref_h, n.min_h)
    };
    n.el.bounded(axis, basis, pref, min, v)
}

/// A flex item's inputs along the main axis (CSS Flexbox §9.2, §9.3 and
/// §4.5): its flex base size, its hypothetical main size (the base held to
/// its bounds), its min and max, and its factors. An item without CSS flex
/// factors takes its house kind's: a stretch size grows and shrinks, a fit
/// one shrinks, and a definite one neither.
#[derive(Clone, Copy, Debug)]
struct FlexInputs {
    base: f32,
    hypo: f32,
    min: f32,
    max: f32,
    grow: f32,
    shrink: f32,
}

fn flex_inputs(n: &Node, horizontal: bool, inner: f32) -> FlexInputs {
    let axis = usize::from(!horizontal);
    let (size, pref, min) = if horizontal {
        (n.el.width, n.pref_w, n.min_w)
    } else {
        (n.el.height, n.pref_h, n.min_h)
    };
    let f = n.el.flex.unwrap_or(match size {
        Size::Grow { .. } => FlexItem {
            grow: 1.0,
            ..FlexItem::default()
        },
        Size::Fit { .. } => FlexItem::default(),
        _ => FlexItem {
            shrink: 0.0,
            ..FlexItem::default()
        },
    });
    // CSS Flexbox §4.4: a collapsed item keeps its cross size (its line's
    // strut) and gives up its main size.
    if n.props.interact.visibility == crate::element::Visibility::Collapse {
        return FlexInputs {
            base: 0.0,
            hypo: 0.0,
            min: 0.0,
            max: 0.0,
            grow: 0.0,
            shrink: 0.0,
        };
    }
    let specified = size.definite(pref, min, Some(inner));
    let base = f
        .basis
        .and_then(|b| b.definite(pref, min, Some(inner)))
        .or(specified)
        .unwrap_or(pref);
    let explicit = n.el.bounds[axis][0]
        .and_then(|b| b.definite(pref, min, Some(inner)))
        .or(match size {
            Size::Fit { min: m, .. } | Size::Grow { min: m, .. } if m > 0.0 => Some(m),
            _ => None,
        });
    // Its automatic minimum: its content's least size, at most its
    // specified one; a scroll container's is 0.
    let scrolls = if horizontal {
        n.el.scroll_x
    } else {
        n.el.scroll
    };
    let content = n.min_content[axis];
    let auto = if scrolls {
        0.0
    } else {
        specified.map_or(min, |s| content.min(s))
    };
    let lo = explicit.unwrap_or(auto);
    let hi = n.el.bounds[axis][1]
        .and_then(|b| b.definite(pref, min, Some(inner)))
        .or(match size {
            Size::Fit { max, .. } | Size::Grow { max, .. } => Some(max),
            _ => None,
        })
        .unwrap_or(f32::INFINITY)
        .max(lo);
    FlexInputs {
        base,
        hypo: base.clamp(lo, hi),
        min: lo,
        max: hi,
        grow: f.grow.max(0.0),
        shrink: f.shrink.max(0.0),
    }
}

/// CSS Flexbox §9.7, resolving flexible lengths: one line's main sizes in
/// `avail` (its room less gaps and margins), each item grown by its
/// share of the positive free space or shrunk by its scaled share of the
/// negative, the items its bounds clamp frozen and the rest resolved again.
fn css_flex(items: &[FlexInputs], avail: f32) -> Vec<f32> {
    let n = items.len();
    let growing = items.iter().map(|it| it.hypo).sum::<f32>() < avail;
    let factor = |it: &FlexInputs| if growing { it.grow } else { it.shrink };
    let mut target: Vec<f32> = items.iter().map(|it| it.hypo).collect();
    let mut frozen: Vec<bool> = items
        .iter()
        .map(|it| {
            factor(it) == 0.0 || (growing && it.base > it.hypo) || (!growing && it.base < it.hypo)
        })
        .collect();
    let room = |target: &[f32], frozen: &[bool]| {
        avail
            - (0..n)
                .map(|k| if frozen[k] { target[k] } else { items[k].base })
                .sum::<f32>()
    };
    let initial = room(&target, &frozen);
    for _ in 0..=n {
        if frozen.iter().all(|&f| f) {
            break;
        }
        let open: Vec<usize> = (0..n).filter(|&k| !frozen[k]).collect();
        let mut free = room(&target, &frozen);
        let factors: f32 = open.iter().map(|&k| factor(&items[k])).sum();
        if factors < 1.0 && (initial * factors).abs() < free.abs() {
            free = initial * factors;
        }
        if growing {
            for &k in &open {
                let share = if factors > 0.0 {
                    items[k].grow / factors
                } else {
                    0.0
                };
                target[k] = items[k].base + free * share;
            }
        } else {
            let scaled: f32 = open.iter().map(|&k| items[k].shrink * items[k].base).sum();
            for &k in &open {
                let share = if scaled > 0.0 {
                    items[k].shrink * items[k].base / scaled
                } else {
                    0.0
                };
                target[k] = items[k].base + free * share;
            }
        }
        let mut total = 0.0;
        let mut violation = vec![0.0f32; n];
        for &k in &open {
            let c = target[k].clamp(items[k].min, items[k].max);
            violation[k] = c - target[k];
            total += violation[k];
            target[k] = c;
        }
        for &k in &open {
            if total == 0.0
                || (total > 0.0 && violation[k] > 0.0)
                || (total < 0.0 && violation[k] < 0.0)
            {
                frozen[k] = true;
            }
        }
    }
    target
}

/// Whether a line of `kids` lays out by CSS Flexbox's algorithm: one of
/// them declares flex factors.
fn css_line(nodes: &[Node], kids: &[usize]) -> bool {
    kids.iter().any(|&c| nodes[c].el.flex.is_some())
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
    if css_line(nodes, kids) {
        let items: Vec<FlexInputs> = kids
            .iter()
            .map(|&c| flex_inputs(&nodes[c], horizontal, inner))
            .collect();
        return css_flex(&items, avail);
    }
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
        let base = size.definite(pref, min, Some(inner)).unwrap_or(pref);
        sizes.push(clamp_to(n, usize::from(!horizontal), Some(inner), base));
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
    for (s, &c) in sizes.iter_mut().zip(kids) {
        *s = clamp_to(&nodes[c], usize::from(!horizontal), Some(inner), *s);
    }
    sizes
}

pub(super) fn distribute_w(nodes: &mut [Node], i: usize, measure: &mut Measure<'_>) {
    // A container query reads the element's own width, now known.
    if nodes[i].props.variants.iter().any(|v| v.when.container) {
        let w = nodes[i].w;
        let variants = nodes[i].props.variants.clone();
        for v in variants.iter().filter(|v| v.when.container) {
            if width_in(w, &v.when) {
                apply_patch(&mut nodes[i], &v.set);
            }
        }
    }
    let inner = (nodes[i].w - nodes[i].el.padding.horizontal()).max(0.0);
    // Out of the flow: sized against the parent's box.
    let outer = nodes[i].w;
    for c in nodes[i].out.clone() {
        let w = out_size(
            nodes[c].el.width,
            nodes[c].pref_w,
            outer,
            nodes[c].props.insets.start,
            nodes[c].props.insets.end,
        );
        nodes[c].w = clamp_to(&nodes[c], 0, Some(outer), w);
    }
    let fonts = measure.fonts;
    // An auto-fit label: the largest of its steps its text fits at this width.
    if let Content::Text(t) = &nodes[i].content
        && !t.fit.is_empty()
    {
        let width = inner + 0.5;
        let step = t
            .fit
            .iter()
            .copied()
            .find(|&step| fits(&TextContent { step, ..t.clone() }, fonts, width))
            .unwrap_or(t.fit[t.fit.len() - 1]);
        if let Content::Text(t) = &mut nodes[i].content {
            t.step = step;
        }
    }
    if let Content::Text(t) = &nodes[i].content {
        // A fit width snapped to the device pixel may land up to half a
        // pixel under the text it was measured from: that is no reason to wrap.
        let width = inner + 0.5;
        let lines = measure
            .texts
            .lines(t, (t.wrap || t.ellipsis).then_some(width), || {
                let m = t.metrics(fonts);
                let at = |w: f32| {
                    break_lines(&t.text, m, t.step.size, t.tracking(), t.features(), Some(w))
                };
                let mut lines = break_lines(
                    &t.text,
                    m,
                    t.step.size,
                    t.tracking(),
                    t.features(),
                    t.wrap.then_some(width),
                );
                if t.wrap && lines.len() > 1 && t.wrap_style != WrapStyle::Auto {
                    lines = rewrap(lines, t.wrap_style, width, at);
                }
                if t.ellipsis {
                    truncate_lines(&mut lines, t, m, width);
                }
                lines
            });
        nodes[i].lines = lines;
        nodes[i].wrap_lines.clear();
        return;
    }
    if let Content::Rich(rc) = &nodes[i].content {
        let lines = crate::rich::rich_lines(rc, fonts, rc.wrap.then_some(inner + 0.5));
        nodes[i].rich_lines = lines;
        return;
    }
    let el = nodes[i].el;
    // A float shrinks to fit (CSS 2 §10.3.5); its `auto` margins are 0.
    for c in nodes[i].floats.clone() {
        let n = &nodes[c];
        let fill = (inner - n.el.margin[1].of(inner) - n.el.margin[3].of(inner)).max(0.0);
        nodes[c].w = cross_size(n, 0, fill, inner, false);
        nodes[c].margin_x = place_sides(&nodes[c], inner, 0.0);
    }
    let kids = nodes[i].children.clone();
    if kids.is_empty() {
        return;
    }
    match nodes[i].props.layout {
        LayoutMode::Stack => {
            for &c in &kids {
                let n = &nodes[c];
                let ins = n.props.insets;
                let w = match (ins.start, ins.end) {
                    (Some(s), Some(e)) if !matches!(n.el.width, Size::Fixed(_)) => {
                        clamp_to(n, 0, Some(inner), (inner - s - e).max(0.0))
                    }
                    _ => cross_size(n, 0, inner, inner, el.cross == Cross::Stretch),
                };
                nodes[c].w = w;
            }
            return;
        }
        LayoutMode::Grid => {
            grid_columns(nodes, i, inner);
            return;
        }
        LayoutMode::Flex => {}
    }
    match el.direction {
        Direction::Column => {
            // A multi-column container's children lay out at the column
            // width, a spanning one at the whole.
            let col = el.multicol.map(|m| super::multicol::used(&m, inner));
            if let Some(cols) = col {
                nodes[i].cols = cols;
            }
            for &c in &kids {
                let n = &nodes[c];
                let inner = match col {
                    Some((_, w)) if super::multicol::in_column(n) => w,
                    _ => inner,
                };
                // A percentage, a calc() or a fixed width reads the column's
                // content width; a stretched or growing one fills what its
                // margins leave.
                let fill = (inner - n.el.margin[1].of(inner) - n.el.margin[3].of(inner)).max(0.0);
                let basis = match n.el.width {
                    Size::Grow { .. } | Size::Fit { .. } => fill,
                    _ => inner,
                };
                let stretch = !el.wrap && n.el.align_self.unwrap_or(el.cross) == Cross::Stretch;
                let w = cross_size(n, 0, basis, inner, stretch);
                // In a horizontal scroller a fitting child keeps its width.
                let w = match n.el.width {
                    Size::Fit { .. } if el.scroll_x => w.max(n.pref_w),
                    _ => w,
                };
                nodes[c].w = w;
                nodes[c].margin_x = place_sides(&nodes[c], inner, fill - w);
            }
        }
        Direction::Row if !el.wrap => {
            let sides: f32 = kids
                .iter()
                .map(|&c| nodes[c].el.margin[1].of(inner) + nodes[c].el.margin[3].of(inner))
                .sum();
            let avail = inner - gaps(el.main_gap(), kids.len()) - sides;
            let sizes = main_sizes(nodes, &kids, true, inner, avail, !el.scroll_x);
            let free = avail - sizes.iter().sum::<f32>();
            for (&c, w) in kids.iter().zip(sizes) {
                nodes[c].w = w;
            }
            line_sides(nodes, &kids, inner, free);
        }
        Direction::Row => {
            // Each child at its preferred width, capped at the row (its
            // hypothetical size, by CSS flex), then lines.
            let css = css_line(nodes, &kids);
            let gap = el.main_gap();
            let mut base = Vec::with_capacity(kids.len());
            let mut sides = Vec::with_capacity(kids.len());
            for &c in &kids {
                let n = &nodes[c];
                let w = if css {
                    flex_inputs(n, true, inner).hypo
                } else {
                    let w =
                        n.el.width
                            .definite(n.pref_w, n.min_w, Some(inner))
                            .unwrap_or_else(|| n.pref_w.min(inner).max(n.min_w));
                    clamp_to(n, 0, Some(inner), w)
                };
                base.push(w);
                sides.push(n.el.margin[1].of(inner) + n.el.margin[3].of(inner));
            }
            let mut rows = Vec::new();
            let mut start = 0;
            let mut used = 0.0f32;
            for (k, (&b, &m)) in base.iter().zip(&sides).enumerate() {
                let b = b + m;
                // Half a pixel of slack, as text wrapping has: a fit row
                // snapped a fraction under its children's sum keeps them
                // on one line, over by less than half a pixel.
                // In lines, a block-level child is a line of its own.
                let forced = nodes[i].line_box.is_some()
                    && k > start
                    && (!inline_level(&nodes[kids[k]]) || !inline_level(&nodes[kids[k - 1]]));
                if k > start && (forced || used + gap + b > inner + 0.5) {
                    rows.push((start, k));
                    start = k;
                    used = b;
                } else {
                    used += if k > start { gap + b } else { b };
                }
            }
            rows.push((start, kids.len()));
            for &(s, e) in &rows {
                let line = &kids[s..e];
                let avail = inner - gaps(gap, line.len()) - sides[s..e].iter().sum::<f32>();
                let sizes = if css {
                    let items: Vec<FlexInputs> = line
                        .iter()
                        .map(|&c| flex_inputs(&nodes[c], true, inner))
                        .collect();
                    css_flex(&items, avail)
                } else {
                    let mut sizes = base[s..e].to_vec();
                    let grow: Vec<Option<f32>> = line
                        .iter()
                        .map(|&c| match nodes[c].el.width {
                            Size::Grow { .. } => Some(nodes[c].el.width.bounds().1),
                            _ => None,
                        })
                        .collect();
                    let none = vec![None; line.len()];
                    flex(&mut sizes, &grow, &none, avail);
                    for (s, &c) in sizes.iter_mut().zip(line) {
                        *s = clamp_to(&nodes[c], 0, Some(inner), *s);
                    }
                    sizes
                };
                let free = avail - sizes.iter().sum::<f32>();
                for (&c, w) in line.iter().zip(sizes) {
                    nodes[c].w = w;
                }
                line_sides(nodes, line, inner, free);
            }
            nodes[i].rows = rows;
        }
    }
}

/// A flex line's left and right margins: `free` split evenly between its
/// `auto` ones (CSS Flexbox §8.1), each other one against `basis`.
fn line_sides(nodes: &mut [Node], line: &[usize], basis: f32, free: f32) {
    let autos = line
        .iter()
        .map(|&c| nodes[c].el.margin[1].auto as usize + nodes[c].el.margin[3].auto as usize)
        .sum::<usize>();
    let each = if autos > 0 {
        free.max(0.0) / autos as f32
    } else {
        0.0
    };
    for &c in line {
        let [l, r] = [nodes[c].el.margin[3], nodes[c].el.margin[1]];
        let side = |m: crate::layout::Margin| if m.auto { each } else { m.of(basis) };
        nodes[c].margin_x = [side(l), side(r)];
    }
}

pub(super) fn size_h(nodes: &mut [Node], i: usize) {
    collapse_margins(nodes, i);
    let n = &nodes[i];
    let (pref, min) = match &n.content {
        _ if let Some(h) = n.contained_size(1) => (h, h),
        Content::None if n.props.layout == LayoutMode::Stack => {
            let kids = || n.children.iter().map(|&c| &nodes[c]);
            (
                kids().map(|c| c.pref_h).fold(0.0, f32::max),
                kids().map(|c| c.min_h).fold(0.0, f32::max),
            )
        }
        Content::None if n.props.layout == LayoutMode::Grid => grid_intrinsic(nodes, i, false),
        Content::Text(_) if let Some(tp) = &n.props.text_path => {
            let h = tp.extent().map_or(0.0, |e| e.y);
            (h, h)
        }
        // An empty text that collapses through itself has no line.
        Content::Text(_) if n.through => (0.0, 0.0),
        Content::Text(t) => {
            // Lines moved down past floats add their drop.
            let drop = n.wrap_lines.last().map_or(0.0, |l| l[1]);
            let h = n.lines.len().max(1) as f32 * t.line_height() + drop;
            (h, h)
        }
        Content::Rich(_) => {
            let h: f32 = n.rich_lines.iter().map(|l| l.height).sum();
            (h, h)
        }
        Content::Image { .. } => (0.0, 0.0),
        Content::None => {
            let kids = || n.children.iter().map(|&c| &nodes[c]);
            match n.el.direction {
                Direction::Column if n.el.multicol.is_some() => {
                    let (lead, between, trail) = column_spacing(nodes, i);
                    let at = |f: &dyn Fn(&Node) -> f32| {
                        super::multicol::height(nodes, i, f, (lead, &between, trail))
                    };
                    (at(&|c| c.pref_h), at(&|c| c.min_h))
                }
                Direction::Column => {
                    let room = column_room(nodes, i);
                    (
                        kids().map(|c| c.pref_h).sum::<f32>() + room,
                        kids().map(|c| c.min_h).sum::<f32>() + room,
                    )
                }
                Direction::Row if !n.el.wrap || n.rows.is_empty() => (
                    kids()
                        .map(|c| c.pref_h + cross_margins(c))
                        .fold(0.0, f32::max),
                    kids()
                        .map(|c| c.min_h + cross_margins(c))
                        .fold(0.0, f32::max),
                ),
                Direction::Row => {
                    let line_h = |s: usize, e: usize, f: &dyn Fn(&Node) -> f32| {
                        n.children[s..e]
                            .iter()
                            .map(|&c| f(&nodes[c]) + cross_margins(&nodes[c]))
                            .fold(0.0, f32::max)
                    };
                    let rows = n.rows.len();
                    let gap = n.el.line_gap();
                    // A line box that holds a `vertical-align` is as tall
                    // as `inline_rows` says.
                    let held = |r: usize| n.inline_rows.get(r).copied().flatten();
                    let pref = n
                        .rows
                        .iter()
                        .enumerate()
                        .map(|(r, &(s, e))| held(r).unwrap_or_else(|| line_h(s, e, &|c| c.pref_h)))
                        .sum::<f32>()
                        + gaps(gap, rows);
                    let min = n
                        .rows
                        .iter()
                        .enumerate()
                        .map(|(r, &(s, e))| held(r).unwrap_or_else(|| line_h(s, e, &|c| c.min_h)))
                        .sum::<f32>()
                        + gaps(gap, rows);
                    (pref, min)
                }
            }
        }
    };
    // A formatting context's height holds its floats (CSS 2 §10.6.7).
    let (pref, min) = (pref.max(n.float_h), min.max(n.float_h));
    // An aspect ratio gives a fitting height from the laid-out width.
    let (pref, min) = match (n.el.aspect > 0.0, n.el.height) {
        (true, Size::Fit { .. }) => {
            let h = (n.w - n.el.padding.horizontal()).max(0.0) / n.el.aspect;
            (h, h)
        }
        _ => (pref, min),
    };
    let scroll = n.el.scroll && n.el.aspect <= 0.0;
    let pad = n.el.padding.vertical();
    let content_min = if scroll { 0.0 } else { min };
    let (p, m) = apply_size(n.el.height, pad + pref, pad + content_min);
    let (cp, cm) = (pad + pref, pad + content_min);
    let el = n.el;
    nodes[i].min_content[1] = cm;
    nodes[i].content_h = pref;
    nodes[i].pref_h = el.bounded(1, None, cp, cm, p);
    nodes[i].min_h = el.bounded(1, None, cp, cm, m);
}

pub(super) fn distribute_h(nodes: &mut [Node], i: usize) {
    let el = nodes[i].el;
    let inner = (nodes[i].h - el.padding.vertical()).max(0.0);
    let outer = nodes[i].h;
    for c in nodes[i].out.clone() {
        let h = out_size(
            nodes[c].el.height,
            nodes[c].pref_h,
            outer,
            nodes[c].props.insets.top,
            nodes[c].props.insets.bottom,
        );
        nodes[c].h = clamp_to(&nodes[c], 1, Some(outer), h);
    }
    for c in nodes[i].floats.clone() {
        let n = &nodes[c];
        let h = match n.el.height {
            Size::Fit { .. } | Size::Grow { .. } => n.pref_h,
            size => size
                .definite(n.pref_h, n.min_h, Some(inner))
                .unwrap_or(n.pref_h),
        };
        nodes[c].h = clamp_to(n, 1, Some(inner), h);
    }
    let kids = nodes[i].children.clone();
    if kids.is_empty() {
        return;
    }
    match nodes[i].props.layout {
        LayoutMode::Stack => {
            for &c in &kids {
                let n = &nodes[c];
                let ins = n.props.insets;
                let h = match (ins.top, ins.bottom) {
                    (Some(t), Some(b)) if !matches!(n.el.height, Size::Fixed(_)) => {
                        clamp_to(n, 1, Some(inner), (inner - t - b).max(0.0))
                    }
                    _ => cross_size(n, 1, inner, inner, el.cross == Cross::Stretch),
                };
                nodes[c].h = h;
            }
            return;
        }
        LayoutMode::Grid => {
            grid_rows(nodes, i, inner);
            return;
        }
        LayoutMode::Flex => {}
    }
    match el.direction {
        Direction::Column if el.wrap => {
            // CSS flex-wrap down a column: lines of the children that fit
            // its height, each sized along it on its own.
            let gap = el.main_gap();
            let outer = |c: usize| nodes[c].pref_h + cross_margins(&nodes[c]);
            let mut rows = Vec::new();
            let (mut start, mut used) = (0, 0.0f32);
            for (k, &kid) in kids.iter().enumerate() {
                let b = outer(kid);
                if k > start && used + gap + b > inner + 0.5 {
                    rows.push((start, k));
                    (start, used) = (k, b);
                } else {
                    used += if k > start { gap + b } else { b };
                }
            }
            rows.push((start, kids.len()));
            for &(s, e) in &rows {
                let line = &kids[s..e];
                let margins: f32 = line.iter().map(|&c| cross_margins(&nodes[c])).sum();
                let avail = inner - gaps(gap, line.len()) - margins;
                let sizes = main_sizes(nodes, line, false, inner, avail, !el.scroll);
                for (&c, h) in line.iter().zip(sizes) {
                    nodes[c].h = h;
                }
            }
            nodes[i].rows = rows;
        }
        // A column's flow is as tall as its content: columns hold it.
        Direction::Column if el.multicol.is_some() => {
            for &c in &kids {
                let h = clamp_to(&nodes[c], 1, Some(inner), nodes[c].pref_h);
                nodes[c].h = h;
            }
        }
        Direction::Column => {
            let avail = inner - column_room(nodes, i);
            let sizes = main_sizes(nodes, &kids, false, inner, avail, !el.scroll);
            for (&c, h) in kids.iter().zip(sizes) {
                nodes[c].h = h;
            }
        }
        Direction::Row if !el.wrap => {
            for &c in &kids {
                let n = &nodes[c];
                let stretch = n.el.align_self.unwrap_or(el.cross) == Cross::Stretch;
                let h = cross_size(n, 1, (inner - cross_margins(n)).max(0.0), inner, stretch);
                // A Fit child keeps its content height across a row.
                let h = match n.el.height {
                    Size::Fit { .. } if !stretch => n.pref_h,
                    _ => h,
                };
                nodes[c].h = h;
            }
        }
        Direction::Row => {
            let rows = nodes[i].rows.clone();
            let extra = line_extra(nodes, i, inner);
            for (r, (s, e)) in rows.into_iter().enumerate() {
                let line = &kids[s..e];
                let line_h = match nodes[i].inline_rows.get(r).copied().flatten() {
                    Some(h) => h,
                    None => line
                        .iter()
                        .map(|&c| nodes[c].pref_h + cross_margins(&nodes[c]))
                        .fold(0.0, f32::max),
                } + extra;
                for &c in line {
                    let n = &nodes[c];
                    let line_h = line_h - cross_margins(n);
                    let stretch = n.el.align_self.unwrap_or(el.cross) == Cross::Stretch;
                    let h = match n.el.height {
                        Size::Grow { .. } => {
                            let (lo, hi) = n.el.height.bounds();
                            line_h.clamp(lo, hi)
                        }
                        Size::Fit { .. } if stretch => line_h,
                        Size::Fit { .. } => n.pref_h,
                        size => size
                            .definite(n.pref_h, n.min_h, Some(inner))
                            .unwrap_or(n.pref_h),
                    };
                    nodes[c].h = clamp_to(n, 1, Some(inner), h);
                }
            }
        }
    }
}

/// The room each line of a wrapping row gains from `align-content:
/// stretch` (CSS Flexbox §9.4): the free space down `inner` its lines
/// leave, shared evenly.
fn line_extra(nodes: &[Node], i: usize, inner: f32) -> f32 {
    let n = &nodes[i];
    if n.el.align_content != Some(Align::Stretch) || n.rows.is_empty() {
        return 0.0;
    }
    let used: f32 = n
        .rows
        .iter()
        .map(|&(s, e)| {
            n.children[s..e]
                .iter()
                .map(|&c| nodes[c].pref_h + cross_margins(&nodes[c]))
                .fold(0.0, f32::max)
        })
        .sum::<f32>()
        + gaps(n.el.line_gap(), n.rows.len());
    ((inner - used) / n.rows.len() as f32).max(0.0)
}

/// The baseline of node `c` below its border box's top as layout sizes it
/// (its first line's, as `Ui::first_baseline` reads it once drawn), if it
/// or its first descendant in the flow has text.
fn layout_baseline(nodes: &[Node], c: usize, fonts: &dyn Metrics) -> Option<f32> {
    let n = &nodes[c];
    match &n.content {
        Content::Text(t) => {
            let line = n.lines.first()?;
            let lh = t.line_height();
            let inner = (n.pref_h - n.el.padding.vertical()).max(0.0);
            let block = lh * n.lines.len() as f32;
            let top = n.el.padding.top + ((inner - block) / 2.0).max(0.0);
            Some(t.metrics(fonts).baseline(&line.text, top, lh, t.step.size))
        }
        Content::Rich(_) => Some(n.el.padding.top + n.rich_lines.first()?.baseline),
        Content::Image { .. } => None,
        Content::None => {
            let &k = n.children.first()?;
            let lead = if n.el.direction == Direction::Column && n.props.layout == LayoutMode::Flex
            {
                column_spacing(nodes, c).0
            } else {
                0.0
            };
            Some(n.el.padding.top + lead + layout_baseline(nodes, k, fonts)?)
        }
    }
}

/// Node `i`'s line boxes (CSS 2.1 §10.8) where they hold a box with a
/// `vertical-align` other than `baseline`, `top` or `bottom`: each such
/// line as tall as its strut, its baseline-aligned boxes and its shifted
/// boxes reach (`Node::inline_rows`), each box's place in it
/// (`Node::inline_y`). Other lines keep their tallest child's height.
pub(super) fn inline_rows(nodes: &mut [Node], i: usize, measure: &Measure<'_>) {
    use crate::VerticalAlign as V;
    let n = &nodes[i];
    let shifted =
        |nodes: &[Node], c: usize| !matches!(nodes[c].el.valign, V::Baseline | V::Top | V::Bottom);
    if n.line_box.is_none() || !n.children.iter().any(|&c| shifted(nodes, c)) {
        if !nodes[i].inline_rows.is_empty() {
            nodes[i].inline_rows.clear();
        }
        return;
    }
    let fonts = measure.fonts;
    // The parent's text: the block's first text (an anonymous inline box
    // carries the block's font), else the body step.
    let (step, font) = n
        .children
        .iter()
        .find_map(|&c| match &nodes[c].content {
            Content::Text(t) => Some((t.step, t.font)),
            Content::Rich(rc) => Some(rc.base),
            _ => None,
        })
        .unwrap_or((measure.body, FontRole::Body));
    let m = fonts.metrics_styled(font, step.weight, false, "");
    let strut = Strut {
        size: step.size,
        ascent: m.ascender * step.size,
        descent: -m.descender * step.size,
        x_height: m.x_height() * step.size,
    };
    let half = (step.line_height - strut.ascent - strut.descent) / 2.0;
    let strut_reach = (half + strut.ascent, step.line_height - half - strut.ascent);
    let kids = n.children.clone();
    let rows = n.rows.clone();
    let mut heights = Vec::with_capacity(rows.len());
    for (s, e) in rows {
        let line = &kids[s..e];
        if !line.iter().any(|&c| shifted(nodes, c)) {
            heights.push(None);
            continue;
        }
        let mut base = strut_reach;
        let mut items = Vec::with_capacity(line.len());
        for &c in line {
            let k = &nodes[c];
            let (mt, mb) = (span(k.margin_sets[0]), span(k.margin_sets[1]));
            let b = layout_baseline(nodes, c, fonts).unwrap_or(k.pref_h);
            let (above, below) = (mt + b, k.pref_h - b + mb);
            let place = match k.el.valign {
                V::Top => InlinePlace::Top,
                V::Bottom => InlinePlace::Bottom,
                va => va.place(&strut, step.size, step.line_height, above, below),
            };
            if place == InlinePlace::Raised(0.0) {
                base = (base.0.max(above), base.1.max(below));
            }
            items.push((place, above, below));
        }
        let (height, _, at) = line_box(base, &items);
        for ((&c, &(_, above, _)), at) in line.iter().zip(&items).zip(at) {
            nodes[c].inline_y = at - above + span(nodes[c].margin_sets[0]);
        }
        heights.push(Some(height));
    }
    nodes[i].inline_rows = heights;
}

/// Where `k` items (or lines) start along an axis with `free` room left,
/// and the extra room between each pair (CSS Box Alignment §5.3). `safe`:
/// an overflow stays at the start. The distributed values fall back to the
/// start when nothing is free.
fn main_offsets(align: Align, free: f32, k: usize, safe: bool) -> (f32, f32) {
    if free < 0.0 && safe {
        return (0.0, 0.0);
    }
    let k = k.max(1) as f32;
    match align {
        Align::Start | Align::Stretch => (0.0, 0.0),
        Align::Center => (free / 2.0, 0.0),
        Align::End => (free, 0.0),
        _ if free <= 0.0 => (0.0, 0.0),
        Align::SpaceBetween if k > 1.0 => (0.0, free / (k - 1.0)),
        Align::SpaceBetween => (0.0, 0.0),
        Align::SpaceAround => (free / (2.0 * k), free / k),
        Align::SpaceEvenly => (free / (k + 1.0), free / (k + 1.0)),
    }
}

pub(super) fn cross_offset(cross: Cross, room: f32, size: f32, safe: bool) -> f32 {
    if safe && room < size {
        return 0.0;
    }
    match cross {
        Cross::Start | Cross::Stretch | Cross::Baseline | Cross::LastBaseline => 0.0,
        Cross::Center => (room - size) / 2.0,
        Cross::End => room - size,
    }
}

/// Its justify-content as laid out: a writing-mode `start`/`end` in a
/// reversed line names the other flex side (CSS Box Alignment §6.1).
fn line_align(el: &Element) -> Align {
    match el.align {
        Align::Start if el.reverse && el.align_logical => Align::End,
        Align::End if el.reverse && el.align_logical => Align::Start,
        a => a,
    }
}

/// Node `i`'s children mirrored along its main axis (`row-reverse`,
/// `column-reverse`) and its lines across it (`wrap-reverse`), inside its
/// content box: each margin box flips, its own margins kept on their sides.
fn mirror_lines(nodes: &mut [Node], i: usize, x0: f32, y0: f32, inner_w: f32, inner_h: f32) {
    let el = nodes[i].el;
    let row = el.direction == Direction::Row;
    let (flip_x, flip_y) = if row {
        (el.reverse, el.wrap_reverse && el.wrap)
    } else {
        (el.wrap_reverse && el.wrap, el.reverse)
    };
    if !flip_x && !flip_y {
        return;
    }
    for c in nodes[i].children.clone() {
        let k = &nodes[c];
        if flip_x {
            let [ml, mr] = k.margin_x;
            nodes[c].x = 2.0 * x0 + inner_w - k.x - k.w - mr + ml;
        }
        let k = &nodes[c];
        if flip_y {
            let (mt, mb) = (span(k.margin_sets[0]), span(k.margin_sets[1]));
            nodes[c].y = 2.0 * y0 + inner_h - k.y - k.h - mb + mt;
        }
    }
}

pub(super) fn position(nodes: &mut [Node], i: usize, scroll: &dyn Fn(u64) -> (f32, f32)) {
    place_children(nodes, i, scroll);
    super::multicol::place(nodes, i);
    if nodes[i].el.flip {
        let n = &nodes[i];
        let sx = if n.el.scroll_x {
            scroll(n.key).0.clamp(0.0, n.max_scroll_x())
        } else {
            0.0
        };
        super::flow::flip_children(nodes, i, sx);
    }
    // Relative and sticky children (CSS Position 3 §3.3–3.4): moved from
    // where the flow put them, their siblings unmoved. Each placement
    // above starts them from the flow again, so this never adds up.
    let kids = nodes[i].children.clone();
    if !kids.iter().any(|&c| {
        matches!(
            nodes[c].props.position,
            Position::Relative | Position::Sticky
        )
    }) {
        return;
    }
    // Their containing block: its content box, scrolled with its content
    // where it is the scroller.
    let n = &nodes[i];
    let (sx, sy) = match (n.el.scroll_x, n.el.scroll) {
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
    let (iw, ih) = (
        (n.w - n.el.padding.horizontal()).max(0.0),
        (n.h - n.el.padding.vertical()).max(0.0),
    );
    let cb = Rect::new(
        n.x + n.el.padding.left - sx,
        n.y + n.el.padding.top - sy,
        if sx > 0.0 { iw.max(n.content_w) } else { iw },
        if n.el.scroll { ih.max(n.content_h) } else { ih },
    );
    // The nearest scroller's scrollport, at or above the parent.
    let mut at = i;
    while at != NONE && !(nodes[at].el.scroll || nodes[at].el.scroll_x) {
        at = nodes[at].up;
    }
    let port = (at != NONE).then(|| Rect::new(nodes[at].x, nodes[at].y, nodes[at].w, nodes[at].h));
    for c in kids {
        let ins = nodes[c].props.insets;
        let (x, y, w, h) = (nodes[c].x, nodes[c].y, nodes[c].w, nodes[c].h);
        let (dx, dy) = match nodes[c].props.position {
            Position::Relative => (
                ins.start.or(ins.end.map(|e| -e)).unwrap_or(0.0),
                ins.top.or(ins.bottom.map(|b| -b)).unwrap_or(0.0),
            ),
            Position::Sticky => {
                let Some(p) = port else { continue };
                // Toward the scrollport's inset edge, never out of its
                // containing block (§3.4).
                let stick = |at: f32,
                             len: f32,
                             lo: Option<f32>,
                             hi: Option<f32>,
                             p0: f32,
                             p1: f32,
                             c0: f32,
                             c1: f32| {
                    let mut v = at;
                    if let Some(lo) = lo {
                        v = v.max(p0 + lo).min((c1 - len).max(at));
                    }
                    if let Some(hi) = hi {
                        v = v.min(p1 - hi - len).max(c0.min(at));
                    }
                    v - at
                };
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
        nodes[c].x += dx;
        nodes[c].y += dy;
        // The edges a sticky box is held to (`scroll-state(stuck)`).
        use crate::doc::media::scroll::{BOTTOM, LEFT, RIGHT, TOP};
        nodes[c].stuck = if nodes[c].props.position == Position::Sticky {
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

/// The insets an in-flow child is placed by in a stack or a grid cell:
/// none for a relative or sticky one, which moves after.
fn flow_insets(n: &Node) -> Edges {
    match n.props.position {
        Position::Relative | Position::Sticky => Edges::default(),
        _ => n.props.insets,
    }
}

fn place_children(nodes: &mut [Node], i: usize, scroll: &dyn Fn(u64) -> (f32, f32)) {
    // Out of the flow: placed by its insets in the parent's box (an
    // anchored one is moved onto its target once every layer is placed).
    let bx = Rect::new(nodes[i].x, nodes[i].y, nodes[i].w, nodes[i].h);
    for c in nodes[i].out.clone() {
        let (w, h) = (nodes[c].w, nodes[c].h);
        if let Some(v) = nodes[c].props.view_place {
            // Its anchor on the point the view box shows `at` at (meet,
            // centred), scaled about it by the fit when drawn.
            let [vx, vy, vw, vh] = v.view_box;
            let k = (bx.width / vw.max(1e-6)).min(bx.height / vh.max(1e-6));
            let off = (v.at[0] - (vx + vw / 2.0)) * k;
            let ax = v.anchor_x * w;
            nodes[c].x = bx.x + bx.width / 2.0 + off - ax;
            nodes[c].y = bx.y + bx.height / 2.0 + (v.at[1] - (vy + vh / 2.0)) * k - v.anchor_y;
            // Right to left the layout is mirrored whole; the draw moves
            // the mirrored box back by its offset `off` from the centre.
            nodes[c].fit = [k, v.anchor_x, v.anchor_y, off];
            continue;
        }
        let (x, y) = place_in(
            &bx,
            w,
            h,
            &nodes[c].props.insets,
            Align::Start,
            Cross::Start,
        );
        nodes[c].x = x;
        nodes[c].y = y;
    }
    if nodes[i].props.layout != LayoutMode::Flex && matches!(nodes[i].content, Content::None) {
        let n = &nodes[i];
        let (sx, sy) = if n.el.scroll || n.el.scroll_x {
            scroll(n.key)
        } else {
            (0.0, 0.0)
        };
        let inner = Rect::new(
            n.x + n.el.padding.left
                - if n.el.scroll_x {
                    sx.clamp(0.0, n.max_scroll_x())
                } else {
                    0.0
                },
            n.y + n.el.padding.top
                - if n.el.scroll {
                    sy.clamp(0.0, n.max_scroll())
                } else {
                    0.0
                },
            (n.w - n.el.padding.horizontal()).max(0.0),
            (n.h - n.el.padding.vertical()).max(0.0),
        );
        let (align, cross) = (n.el.align, n.el.cross);
        let kids = n.children.clone();
        if n.props.layout == LayoutMode::Stack {
            let mut extent = 0.0f32;
            for &c in &kids {
                let (w, h) = (nodes[c].w, nodes[c].h);
                let (x, y) = place_in(&inner, w, h, &flow_insets(&nodes[c]), align, cross);
                nodes[c].x = x;
                nodes[c].y = y;
                extent = extent.max(w);
            }
            nodes[i].content_w = extent;
        } else if nodes[i].grid.is_some() {
            let areas = super::grid::areas_of(nodes, i);
            let mut extent = 0.0f32;
            for (k, &c) in kids.iter().enumerate() {
                let Some(Some((a, justify, align))) = areas.get(k).copied() else {
                    continue;
                };
                // Its margins inset its grid area; `auto` ones align it there
                // (CSS Grid §11.2), else its `justify-self` and `align-self`.
                let k = &nodes[c];
                let [ml, mr] = k.margin_x;
                let [mt, mb] = k.margin_sets.map(|(pos, neg)| pos + neg);
                let cell = Rect::new(
                    inner.x + a.x + ml,
                    inner.y + a.y + mt,
                    (a.width - ml - mr).max(0.0),
                    (a.height - mt - mb).max(0.0),
                );
                let (w, h) = (k.w, k.h);
                let justify = match justify {
                    Cross::Center => Align::Center,
                    Cross::End => Align::End,
                    _ => Align::Start,
                };
                let (x, y) = place_in(&cell, w, h, &flow_insets(k), justify, align);
                let y = auto_offset(k, 0, cell.height - h).map_or(y, |o| cell.y + o);
                nodes[c].x = x;
                nodes[c].y = y;
                extent = extent.max(a.x + a.width);
            }
            nodes[i].content_w = extent;
            super::grid::place_table_parts(nodes, i, inner.x, inner.y);
        }
        return;
    }
    // The content's extent along x, before the offsets (the horizontal range).
    let content_w = {
        let n = &nodes[i];
        let kids = &n.children;
        let gap = n.el.main_gap();
        match n.el.direction {
            Direction::Column if n.el.wrap && !n.rows.is_empty() => {
                n.rows
                    .iter()
                    .map(|&(s, e)| {
                        kids[s..e]
                            .iter()
                            .map(|&c| outer_w(&nodes[c]))
                            .fold(0.0, f32::max)
                    })
                    .sum::<f32>()
                    + gaps(n.el.line_gap(), n.rows.len())
            }
            Direction::Column => kids.iter().map(|&c| outer_w(&nodes[c])).fold(0.0, f32::max),
            Direction::Row if n.el.wrap && !n.rows.is_empty() => n
                .rows
                .iter()
                .map(|&(s, e)| {
                    kids[s..e].iter().map(|&c| outer_w(&nodes[c])).sum::<f32>() + gaps(gap, e - s)
                })
                .fold(0.0, f32::max),
            Direction::Row => {
                kids.iter().map(|&c| outer_w(&nodes[c])).sum::<f32>() + gaps(gap, kids.len())
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
    let wrapped = el.wrap && !n.rows.is_empty();
    let rows = if wrapped {
        n.rows.clone()
    } else {
        vec![(0, kids.len())]
    };
    let (safe_main, safe_cross) = (el.safe & 1 != 0, el.safe & 2 != 0);
    let align = line_align(&el);
    // Each line's size across the axis, and where the lines start and
    // what lies between them (`align-content`, CSS Flexbox §9.4).
    let across = |c: usize, row: bool| {
        if row {
            nodes[c].h + cross_margins(&nodes[c])
        } else {
            outer_w(&nodes[c])
        }
    };
    let row = el.direction == Direction::Row;
    let extra = if row {
        line_extra(nodes, i, inner_h)
    } else {
        0.0
    };
    let held: Vec<Option<f32>> = (0..rows.len())
        .map(|r| {
            if wrapped && row {
                n.inline_rows.get(r).copied().flatten()
            } else {
                None
            }
        })
        .collect();
    let rooms: Vec<f32> = rows
        .iter()
        .zip(&held)
        .map(|(&(s, e), h)| match (wrapped, row) {
            (false, true) => inner_h,
            (false, false) => inner_w,
            (true, _) => {
                h.unwrap_or_else(|| {
                    kids[s..e]
                        .iter()
                        .map(|&c| across(c, row))
                        .fold(0.0, f32::max)
                }) + extra
            }
        })
        .collect();
    let (mut line_at, line_between) = match (wrapped, el.align_content) {
        (true, Some(a)) => {
            let room = if row { inner_h } else { inner_w };
            let free = room - rooms.iter().sum::<f32>() - gaps(el.line_gap(), rooms.len());
            main_offsets(a, free, rooms.len(), false)
        }
        _ => (0.0, 0.0),
    };
    for (line_k, (s, e)) in rows.into_iter().enumerate() {
        let line = &kids[s..e];
        let k = line.len();
        let room = rooms[line_k];
        match el.direction {
            Direction::Row => {
                let gap = el.main_gap();
                let used: f32 =
                    line.iter().map(|&c| outer_w(&nodes[c])).sum::<f32>() + gaps(gap, k);
                let mut free = inner_w - used;
                if el.scroll_x {
                    free = free.max(0.0);
                }
                let (start, between) = main_offsets(align, free, k, safe_main);
                let mut x = x0 + start;
                for &c in line {
                    let (w, h) = (nodes[c].w, nodes[c].h);
                    let (top, around) = (span(nodes[c].margin_sets[0]), cross_margins(&nodes[c]));
                    let [ml, mr] = nodes[c].margin_x;
                    let cross = nodes[c].el.align_self.unwrap_or(el.cross);
                    let safe = safe_cross || nodes[c].el.safe & 2 != 0;
                    nodes[c].x = x + ml;
                    nodes[c].y = y0
                        + line_at
                        + if held[line_k].is_some() {
                            // Where its line box put it (`inline_rows`).
                            nodes[c].inline_y
                        } else {
                            top + auto_offset(&nodes[c], 0, room - around - h)
                                .unwrap_or_else(|| cross_offset(cross, room - around, h, safe))
                        };
                    x += ml + w + mr + gap + between;
                }
                line_at += room + el.line_gap() + line_between;
            }
            Direction::Column if wrapped => {
                // One column of a wrapping column: its items down it, each
                // aligned across the column's width.
                let gap = el.main_gap();
                let used: f32 = line
                    .iter()
                    .map(|&c| nodes[c].h + cross_margins(&nodes[c]))
                    .sum::<f32>()
                    + gaps(gap, k);
                let free = inner_h - used;
                let (start, between) = main_offsets(align, free, k, safe_main);
                let mut y = y0 + start;
                for &c in line {
                    let (w, h) = (nodes[c].w, nodes[c].h);
                    let [ml, mr] = nodes[c].margin_x;
                    let (top, bottom) =
                        (span(nodes[c].margin_sets[0]), span(nodes[c].margin_sets[1]));
                    let cross = nodes[c].el.align_self.unwrap_or(el.cross);
                    let safe = safe_cross || nodes[c].el.safe & 2 != 0;
                    nodes[c].y = y + top;
                    nodes[c].x = x0 + line_at + ml + cross_offset(cross, room - ml - mr, w, safe);
                    y += top + h + bottom + gap + between;
                }
                line_at += room + el.line_gap() + line_between;
            }
            Direction::Column => {
                let (lead, spacing, trail) = column_spacing(nodes, i);
                let used: f32 = line.iter().map(|&c| nodes[c].h).sum::<f32>()
                    + lead
                    + spacing.iter().sum::<f32>()
                    + trail;
                let mut free = inner_h - used;
                if el.scroll {
                    free = free.max(0.0);
                }
                // A flex column's `auto` top and bottom margins take its free
                // space before `justify-content` does (CSS Flexbox §8.1); a
                // block's are 0 (CSS 2 §10.6.3).
                let autos = if el.block {
                    0
                } else {
                    line.iter()
                        .map(|&c| {
                            nodes[c].el.margin[0].auto as usize
                                + nodes[c].el.margin[2].auto as usize
                        })
                        .sum()
                };
                let each = if autos > 0 {
                    free.max(0.0) / autos as f32
                } else {
                    0.0
                };
                let (start, between) = if autos > 0 {
                    (0.0, 0.0)
                } else {
                    main_offsets(align, free, k, safe_main)
                };
                let mut y = y0 + start + lead;
                for (j, &c) in line.iter().enumerate() {
                    let (w, h) = (nodes[c].w, nodes[c].h);
                    let [top, bottom] = [0, 2].map(|e| {
                        if nodes[c].el.margin[e].auto {
                            each
                        } else {
                            0.0
                        }
                    });
                    let [ml, mr] = nodes[c].margin_x;
                    let room = inner_w.max(if el.scroll_x { ml + w + mr } else { 0.0 }) - ml - mr;
                    let sideways = nodes[c].el.margin[1].auto || nodes[c].el.margin[3].auto;
                    let cross = nodes[c].el.align_self.unwrap_or(el.cross);
                    let safe = safe_cross || nodes[c].el.safe & 2 != 0;
                    nodes[c].y = y + top;
                    nodes[c].x = x0
                        + ml
                        + if sideways {
                            0.0
                        } else {
                            cross_offset(cross, room, w, safe)
                        };
                    y += top + h + bottom + spacing.get(j).copied().unwrap_or(0.0) + between;
                }
            }
        }
    }
    mirror_lines(nodes, i, x0, y0, inner_w, inner_h);
    for c in nodes[i].floats.clone() {
        let [fx, fy] = nodes[c].float_at;
        nodes[c].x = x0 + fx;
        nodes[c].y = y0 + fy;
    }
}

/// Its width with its left and right margins as laid out.
fn outer_w(n: &Node) -> f32 {
    n.w + n.margin_x[0] + n.margin_x[1]
}

/// Where its `auto` margins on edges `edge` and `edge + 2` (top and bottom,
/// or right and left) put it in `free` room: centred between two, at the
/// far end after a leading one, at the start before a trailing one. `None`
/// with neither: its parent's alignment places it.
pub(super) fn auto_offset(n: &Node, edge: usize, free: f32) -> Option<f32> {
    let free = free.max(0.0);
    let (lead, trail) = if edge == 0 {
        (n.el.margin[0].auto, n.el.margin[2].auto)
    } else {
        (n.el.margin[3].auto, n.el.margin[1].auto)
    };
    match (lead, trail) {
        (true, true) => Some(free / 2.0),
        (true, false) => Some(free),
        (false, true) => Some(0.0),
        (false, false) => None,
    }
}

/// Where an out-of-flow element placed on the screen ([`Position::Screen`])
/// stands: its size and place by its insets in the whole surface `screen`.
pub(crate) fn on_screen(n: &Node, screen: &Rect) -> Rect {
    let ins = &n.props.insets;
    let w = out_size(n.el.width, n.pref_w, screen.width, ins.start, ins.end);
    let w = clamp_to(n, 0, Some(screen.width), w);
    let h = out_size(n.el.height, n.pref_h, screen.height, ins.top, ins.bottom);
    let h = clamp_to(n, 1, Some(screen.height), h);
    let (x, y) = place_in(screen, w, h, ins, Align::Start, Cross::Start);
    Rect::new(x, y, w, h)
}

/// Where a `w`×`h` box sits in `bx`: by its insets where they are set,
/// else by `align` across and `cross` down.
fn place_in(bx: &Rect, w: f32, h: f32, ins: &Edges, align: Align, cross: Cross) -> (f32, f32) {
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
            Align::Start
            | Align::SpaceBetween
            | Align::SpaceAround
            | Align::SpaceEvenly
            | Align::Stretch => bx.x,
            Align::Center => bx.x + (bx.width - w) / 2.0,
            Align::End => bx.right() - w,
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
            Cross::Start | Cross::Stretch | Cross::Baseline | Cross::LastBaseline => bx.y,
            Cross::Center => bx.y + (bx.height - h) / 2.0,
            Cross::End => bx.bottom() - h,
        }
    };
    (x, y)
}

/// An out-of-flow element's size along one axis of its parent's box
/// (`outer`), between insets `a` and `b`.
fn out_size(size: Size, pref: f32, outer: f32, a: Option<f32>, b: Option<f32>) -> f32 {
    match (a, b, size) {
        (_, _, Size::Fixed(v)) => v,
        (Some(a), Some(b), _) => (outer - a - b).max(0.0),
        (_, _, Size::Percent(p)) => p * outer,
        (_, _, Size::Calc(c)) => c.of(outer),
        (_, _, Size::Grow { .. }) => {
            let (lo, hi) = size.bounds();
            (outer - a.unwrap_or(0.0) - b.unwrap_or(0.0)).clamp(lo, hi)
        }
        (_, _, Size::Fit { .. }) => {
            let (lo, hi) = size.bounds();
            pref.clamp(lo, hi)
        }
        // A content size is its preferred size, which layout already holds.
        (_, _, Size::MinContent | Size::MaxContent) => pref,
    }
}
