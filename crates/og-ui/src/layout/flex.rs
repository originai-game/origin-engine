use super::*;

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
    }
}

fn gaps(gap: f32, n: usize) -> f32 {
    gap * n.saturating_sub(1) as f32
}

/// A collapsing margin set (CSS 2 §8.3.1): its largest positive and its
/// most negative margin; it spans their sum.
type MarginSet = (f32, f32);

fn margin_set(m: f32) -> MarginSet {
    (m.max(0.0), m.min(0.0))
}

fn joined(a: MarginSet, b: MarginSet) -> MarginSet {
    (a.0.max(b.0), a.1.min(b.1))
}

fn span(m: MarginSet) -> f32 {
    m.0 + m.1
}

/// The width node `i`'s margin percentages read: its parent's content
/// width (CSS 2 §8.3: all four, even the vertical ones).
pub(super) fn margin_basis(nodes: &[Node], i: usize) -> f32 {
    let p = nodes[i].parent;
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
    let p = n.parent;
    let in_block_flow = p != NONE
        && p < nodes.len()
        && nodes[p].el.block
        && nodes[p].props.layout == LayoutMode::Flex
        && nodes[p].children.contains(&i);
    let flows = in_block_flow
        && n.el.block
        && n.props.layout == LayoutMode::Flex
        && matches!(n.content, Content::None)
        && !n.el.scroll
        && !n.el.scroll_x;
    let auto_height = matches!(n.el.height, Size::Fit { min, .. } if min <= 0.0);
    let empty_text = matches!(&n.content, Content::Text(t) if t.text.is_empty());
    let through = in_block_flow
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
fn column_spacing(nodes: &[Node], i: usize) -> (f32, Vec<f32>, f32) {
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
        let gap = if j > 0 { n.el.gap } else { 0.0 };
        if n.el.block {
            let set = joined(run, k.margin_sets[0]);
            let total = if outside { 0.0 } else { span(set) };
            before.push(gap + total - used);
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
fn place_sides(n: &Node, basis: f32, free: f32) -> [f32; 2] {
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

/// A text's preferred width (its widest line) and the least it can take:
/// its ellipsis, its longest word when it wraps, else all of it.
fn text_widths(t: &TextContent, fonts: &dyn Metrics) -> (f32, f32) {
    let m = t.metrics(fonts);
    let pref = widest_line(&t.text, m, t.step.size, t.tracking(), t.features());
    let min = if t.ellipsis {
        // It can shrink to its ellipsis.
        m.measure_with(ellipsis_of(m), t.step.size, t.tracking(), t.features())
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
    if nodes[i].props.layout == LayoutMode::Grid && matches!(nodes[i].content, Content::None) {
        place_grid(nodes, i);
    }
    let n = &nodes[i];
    let fonts = measure.fonts;
    let (pref, min) = match &n.content {
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
                Direction::Row => {
                    let pref = kids
                        .clone()
                        .map(|c| c.pref_w + side_margins(c))
                        .sum::<f32>()
                        + gaps(n.el.gap, k);
                    let min = if n.el.wrap {
                        kids.map(|c| c.min_w + side_margins(c)).fold(0.0, f32::max)
                    } else {
                        kids.map(|c| c.min_w + side_margins(c)).sum::<f32>() + gaps(n.el.gap, k)
                    };
                    (pref, min)
                }
                Direction::Column => (
                    kids.clone()
                        .map(|c| c.pref_w + side_margins(c))
                        .fold(0.0, f32::max),
                    kids.map(|c| c.min_w + side_margins(c)).fold(0.0, f32::max),
                ),
            }
        }
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
    nodes[i].pref_w = p;
    nodes[i].min_w = m;
}

/// A child's width across a column (or height across a row).
fn cross_size(size: Size, pref: f32, min: f32, inner: f32, stretch: bool) -> f32 {
    match size {
        Size::Fixed(v) => v,
        Size::Percent(p) => p * inner,
        Size::Calc(c) => c.of(inner),
        Size::Grow { .. } => {
            let (lo, hi) = size.bounds();
            inner.clamp(lo, hi)
        }
        Size::Fit { .. } if stretch => {
            let (lo, hi) = size.bounds();
            inner.clamp(lo, hi)
        }
        Size::Fit { .. } => pref.min(inner).max(min),
    }
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
        let base = match size {
            Size::Fixed(v) => v,
            Size::Percent(p) => p * inner,
            Size::Calc(c) => c.of(inner),
            _ => pref,
        };
        sizes.push(base);
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
        nodes[c].w = w;
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
                let mut lines = break_lines(
                    &t.text,
                    m,
                    t.step.size,
                    t.tracking(),
                    t.features(),
                    t.wrap.then_some(width),
                );
                if t.ellipsis {
                    truncate_lines(&mut lines, t, m, width);
                }
                lines
            });
        nodes[i].lines = lines;
        return;
    }
    if let Content::Rich(rc) = &nodes[i].content {
        let lines = crate::rich::rich_lines(rc, fonts, rc.wrap.then_some(inner + 0.5));
        nodes[i].rich_lines = lines;
        return;
    }
    let el = nodes[i].el;
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
                        (inner - s - e).max(0.0)
                    }
                    _ => cross_size(
                        n.el.width,
                        n.pref_w,
                        n.min_w,
                        inner,
                        el.cross == Cross::Stretch,
                    ),
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
            for &c in &kids {
                let n = &nodes[c];
                // A percentage, a calc() or a fixed width reads the column's
                // content width; a stretched or growing one fills what its
                // margins leave.
                let fill = (inner - n.el.margin[1].of(inner) - n.el.margin[3].of(inner)).max(0.0);
                let basis = match n.el.width {
                    Size::Grow { .. } | Size::Fit { .. } => fill,
                    _ => inner,
                };
                let w = cross_size(
                    n.el.width,
                    n.pref_w,
                    n.min_w,
                    basis,
                    el.cross == Cross::Stretch,
                );
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
            let avail = inner - gaps(el.gap, kids.len()) - sides;
            let sizes = main_sizes(nodes, &kids, true, inner, avail, !el.scroll_x);
            let free = avail - sizes.iter().sum::<f32>();
            for (&c, w) in kids.iter().zip(sizes) {
                nodes[c].w = w;
            }
            line_sides(nodes, &kids, inner, free);
        }
        Direction::Row => {
            // Each child at its preferred width, capped at the row; then lines.
            let mut base = Vec::with_capacity(kids.len());
            let mut sides = Vec::with_capacity(kids.len());
            for &c in &kids {
                let n = &nodes[c];
                base.push(match n.el.width {
                    Size::Fixed(v) => v,
                    Size::Percent(p) => p * inner,
                    Size::Calc(c) => c.of(inner),
                    _ => n.pref_w.min(inner).max(n.min_w),
                });
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
                if k > start && used + el.gap + b > inner + 0.5 {
                    rows.push((start, k));
                    start = k;
                    used = b;
                } else {
                    used += if k > start { el.gap + b } else { b };
                }
            }
            rows.push((start, kids.len()));
            for &(s, e) in &rows {
                let line = &kids[s..e];
                let mut sizes = base[s..e].to_vec();
                let grow: Vec<Option<f32>> = line
                    .iter()
                    .map(|&c| match nodes[c].el.width {
                        Size::Grow { .. } => Some(nodes[c].el.width.bounds().1),
                        _ => None,
                    })
                    .collect();
                let none = vec![None; line.len()];
                let avail = inner - gaps(el.gap, line.len()) - sides[s..e].iter().sum::<f32>();
                flex(&mut sizes, &grow, &none, avail);
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
            let h = n.lines.len().max(1) as f32 * t.line_height();
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
                    let pref = n
                        .rows
                        .iter()
                        .map(|&(s, e)| line_h(s, e, &|c| c.pref_h))
                        .sum::<f32>()
                        + gaps(n.el.gap, rows);
                    let min = n
                        .rows
                        .iter()
                        .map(|&(s, e)| line_h(s, e, &|c| c.min_h))
                        .sum::<f32>()
                        + gaps(n.el.gap, rows);
                    (pref, min)
                }
            }
        }
    };
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
    nodes[i].content_h = pref;
    nodes[i].pref_h = p;
    nodes[i].min_h = m;
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
        nodes[c].h = h;
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
                        (inner - t - b).max(0.0)
                    }
                    _ => cross_size(
                        n.el.height,
                        n.pref_h,
                        n.min_h,
                        inner,
                        el.cross == Cross::Stretch,
                    ),
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
                let h = cross_size(
                    n.el.height,
                    n.pref_h,
                    n.min_h,
                    (inner - cross_margins(n)).max(0.0),
                    el.cross == Cross::Stretch,
                );
                // A Fit child keeps its content height across a row.
                let h = match n.el.height {
                    Size::Fit { .. } if el.cross != Cross::Stretch => n.pref_h,
                    _ => h,
                };
                nodes[c].h = h;
            }
        }
        Direction::Row => {
            let rows = nodes[i].rows.clone();
            for (s, e) in rows {
                let line = &kids[s..e];
                let line_h = line
                    .iter()
                    .map(|&c| nodes[c].pref_h + cross_margins(&nodes[c]))
                    .fold(0.0, f32::max);
                for &c in line {
                    let n = &nodes[c];
                    let line_h = line_h - cross_margins(n);
                    let h = match n.el.height {
                        Size::Fixed(v) => v,
                        Size::Percent(p) => p * inner,
                        Size::Calc(c) => c.of(inner),
                        Size::Grow { .. } => {
                            let (lo, hi) = n.el.height.bounds();
                            line_h.clamp(lo, hi)
                        }
                        Size::Fit { .. } if el.cross == Cross::Stretch => line_h,
                        Size::Fit { .. } => n.pref_h,
                    };
                    nodes[c].h = h;
                }
            }
        }
    }
}

fn main_offsets(align: Align, free: f32, k: usize) -> (f32, f32) {
    match align {
        Align::Start => (0.0, 0.0),
        Align::Center => (free / 2.0, 0.0),
        Align::End => (free, 0.0),
        Align::SpaceBetween if k > 1 && free > 0.0 => (0.0, free / (k - 1) as f32),
        Align::SpaceBetween => (0.0, 0.0),
    }
}

fn cross_offset(cross: Cross, room: f32, size: f32) -> f32 {
    match cross {
        Cross::Start | Cross::Stretch | Cross::Baseline => 0.0,
        Cross::Center => (room - size) / 2.0,
        Cross::End => room - size,
    }
}

pub(super) fn position(nodes: &mut [Node], i: usize, scroll: &dyn Fn(u64) -> (f32, f32)) {
    // Out of the flow: placed by its insets in the parent's box (an
    // anchored one is moved onto its target once every layer is placed).
    let bx = Rect::new(nodes[i].x, nodes[i].y, nodes[i].w, nodes[i].h);
    for c in nodes[i].out.clone() {
        let (w, h) = (nodes[c].w, nodes[c].h);
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
                let (x, y) = place_in(&inner, w, h, &nodes[c].props.insets, align, cross);
                nodes[c].x = x;
                nodes[c].y = y;
                extent = extent.max(w);
            }
            nodes[i].content_w = extent;
        } else if let Some(g) = nodes[i].grid.clone() {
            let offsets = |sizes: &[f32], gap: f32| {
                let mut at = Vec::with_capacity(sizes.len() + 1);
                let mut v = 0.0;
                for s in sizes {
                    at.push(v);
                    v += s + gap;
                }
                at.push(v - gap);
                at
            };
            let (cx, ry) = (offsets(&g.cols, g.col_gap), offsets(&g.rows, g.row_gap));
            for (k, &c) in kids.iter().enumerate() {
                let Some(&(col, row, cs, rs)) = g.cells.get(k) else {
                    continue;
                };
                let span = |at: &[f32], sizes: &[f32], s: usize, n: usize, gap: f32| {
                    let s = s.min(sizes.len());
                    let e = (s + n).min(sizes.len());
                    let len: f32 =
                        sizes[s..e].iter().sum::<f32>() + gap * (e - s).saturating_sub(1) as f32;
                    (at.get(s).copied().unwrap_or(0.0), len)
                };
                let (x0, cw) = span(&cx, &g.cols, col, cs, g.col_gap);
                let (y0, rh) = span(&ry, &g.rows, row, rs, g.row_gap);
                // Its margins inset its grid area; `auto` ones align it there
                // (CSS Grid §11.2).
                let k = &nodes[c];
                let [ml, mr] = k.margin_x;
                let [mt, mb] = k.margin_sets.map(|(pos, neg)| pos + neg);
                let cell = Rect::new(
                    inner.x + x0 + ml,
                    inner.y + y0 + mt,
                    (cw - ml - mr).max(0.0),
                    (rh - mt - mb).max(0.0),
                );
                let (w, h) = (k.w, k.h);
                let (x, y) = place_in(&cell, w, h, &k.props.insets, align, cross);
                let y = auto_offset(k, 0, cell.height - h).map_or(y, |o| cell.y + o);
                nodes[c].x = x;
                nodes[c].y = y;
            }
            nodes[i].content_w = cx.last().copied().unwrap_or(0.0);
        }
        return;
    }
    // The content's extent along x, before the offsets (the horizontal range).
    let content_w = {
        let n = &nodes[i];
        let kids = &n.children;
        match n.el.direction {
            Direction::Column => kids.iter().map(|&c| outer_w(&nodes[c])).fold(0.0, f32::max),
            Direction::Row if n.el.wrap && !n.rows.is_empty() => n
                .rows
                .iter()
                .map(|&(s, e)| {
                    kids[s..e].iter().map(|&c| outer_w(&nodes[c])).sum::<f32>()
                        + gaps(n.el.gap, e - s)
                })
                .fold(0.0, f32::max),
            Direction::Row => {
                kids.iter().map(|&c| outer_w(&nodes[c])).sum::<f32>() + gaps(n.el.gap, kids.len())
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
    let rows = if el.direction == Direction::Row && el.wrap && !n.rows.is_empty() {
        n.rows.clone()
    } else {
        vec![(0, kids.len())]
    };
    let mut line_y = 0.0;
    for (s, e) in rows {
        let line = &kids[s..e];
        let k = line.len();
        match el.direction {
            Direction::Row => {
                let used: f32 =
                    line.iter().map(|&c| outer_w(&nodes[c])).sum::<f32>() + gaps(el.gap, k);
                let mut free = inner_w - used;
                if el.scroll_x {
                    free = free.max(0.0);
                }
                let (start, between) = main_offsets(el.align, free, k);
                let room = if el.wrap {
                    line.iter()
                        .map(|&c| nodes[c].h + cross_margins(&nodes[c]))
                        .fold(0.0, f32::max)
                } else {
                    inner_h
                };
                let mut x = x0 + start;
                for &c in line {
                    let (w, h) = (nodes[c].w, nodes[c].h);
                    let (top, around) = (span(nodes[c].margin_sets[0]), cross_margins(&nodes[c]));
                    let [ml, mr] = nodes[c].margin_x;
                    nodes[c].x = x + ml;
                    nodes[c].y = y0
                        + line_y
                        + top
                        + auto_offset(&nodes[c], 0, room - around - h)
                            .unwrap_or_else(|| cross_offset(el.cross, room - around, h));
                    x += ml + w + mr + el.gap + between;
                }
                line_y += room + el.gap;
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
                    main_offsets(el.align, free, k)
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
                    nodes[c].y = y + top;
                    nodes[c].x = x0
                        + ml
                        + if sideways {
                            0.0
                        } else {
                            cross_offset(el.cross, room, w)
                        };
                    y += top + h + bottom + spacing.get(j).copied().unwrap_or(0.0) + between;
                }
            }
        }
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
    let h = out_size(n.el.height, n.pref_h, screen.height, ins.top, ins.bottom);
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
            Align::Start | Align::SpaceBetween => bx.x,
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
            Cross::Start | Cross::Stretch | Cross::Baseline => bx.y,
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
    }
}
