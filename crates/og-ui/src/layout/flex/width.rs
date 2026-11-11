use super::*;

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

/// A text's extent along its line: across, or down a vertical one.
fn text_measure(t: &TextContent, m: &FontMetrics, text: &str) -> f32 {
    let (size, tr, ft) = (t.step.size, t.tracking(), t.features());
    match t.vertical {
        Some(v) => crate::text::vertical::measure(m, text, size, tr, ft, v),
        None => m.measure_with(text, size, tr, ft),
    }
}

/// A text's widest line along its line.
pub(super) fn text_widest(t: &TextContent, m: &FontMetrics) -> f32 {
    let (size, tr, ft) = (t.step.size, t.tracking(), t.features());
    match t.vertical {
        Some(v) => crate::text::vertical::widest_line(&t.text, m, size, tr, ft, v),
        None => widest_line(&t.text, m, size, tr, ft),
    }
}

/// A text's longest unbreakable unit along its line.
fn text_longest(t: &TextContent, m: &FontMetrics) -> f32 {
    let (size, tr, ft) = (t.step.size, t.tracking(), t.features());
    match t.vertical {
        Some(v) => crate::text::vertical::longest_word(&t.text, m, size, tr, ft, v),
        None => longest_word(&t.text, m, size, tr, ft),
    }
}

/// A text's lines at `width` along its line (`None`: only at `\n`).
pub(crate) fn text_lines(t: &TextContent, m: &FontMetrics, width: Option<f32>) -> Vec<Line> {
    let (size, tr, ft) = (t.step.size, t.tracking(), t.features());
    match t.vertical {
        Some(v) => crate::text::vertical::break_lines(&t.text, m, size, tr, ft, v, width),
        None => break_lines(&t.text, m, size, tr, ft, width),
    }
}

/// A text's preferred width (its widest line) and the least it can take:
/// its ellipsis, its longest word when it wraps, else all of it.
fn text_widths(t: &TextContent, fonts: &dyn Metrics) -> (f32, f32) {
    let m = t.metrics(fonts);
    let pref = text_widest(t, m);
    let min = if t.ellipsis {
        // It can shrink to its ellipsis.
        text_measure(t, m, t.cut.as_deref().unwrap_or_else(|| ellipsis_of(m))).min(pref)
    } else if t.wrap {
        text_longest(t, m)
    } else {
        pref
    };
    (pref, min)
}

/// A text shows whole at `width` at its step: one line no wider, or (wrapping)
/// no word wider and no more lines than its limit.
fn fits(t: &TextContent, fonts: &dyn Metrics, width: f32) -> bool {
    let m = t.metrics(fonts);
    if !t.wrap {
        return text_widest(t, m) <= width;
    }
    text_longest(t, m) <= width
        && (t.max_lines == 0 || text_lines(t, m, Some(width)).len() <= t.max_lines as usize)
}

pub(in crate::layout) fn size_w(nodes: &mut [Node], i: usize, measure: &mut Measure<'_>) {
    if nodes[i].props.layout == LayoutMode::Grid
        && matches!(nodes[i].content, Content::None)
        && !nodes[i].layout.cv_skip
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
        Content::Rich(rc) => crate::rich::rich_widths(rc, fonts, n.rich_vertical()),
        Content::Image { .. } => (0.0, 0.0),
        Content::None if n.props.layout == LayoutMode::Stack => {
            let kids = || n.children.iter().map(|&c| &nodes[c]);
            (
                kids().map(|c| c.layout.pref_w).fold(0.0, f32::max),
                kids().map(|c| c.layout.min_w).fold(0.0, f32::max),
            )
        }
        Content::None if n.props.layout == LayoutMode::Grid => grid_intrinsic(nodes, i, true),
        Content::None => {
            let k = n.children.len();
            let kids = n.children.iter().map(|&c| &nodes[c]);
            match n.layout.used.direction {
                // Lines: its widest run of inline-level children, or its
                // widest block-level one (each on a line of its own).
                Direction::Row if n.layout.line_box.is_some() => {
                    let gap = n.layout.used.main_gap();
                    let (mut pref, mut run, mut count) = (0.0f32, 0.0f32, 0usize);
                    for &c in &n.children {
                        let k = &nodes[c];
                        let w = k.layout.pref_w + side_margins(k);
                        if inline_level(k) {
                            run += w + if count > 0 { gap } else { 0.0 };
                            count += 1;
                        } else {
                            pref = pref.max(run).max(w);
                            (run, count) = (0.0, 0);
                        }
                    }
                    let min = kids
                        .map(|c| c.layout.min_w + side_margins(c))
                        .fold(0.0, f32::max);
                    (pref.max(run), min)
                }
                Direction::Row => {
                    let gap = n.layout.used.main_gap();
                    let pref = kids
                        .clone()
                        .map(|c| c.layout.pref_w + side_margins(c))
                        .sum::<f32>()
                        + gaps(gap, k);
                    let min = if n.layout.used.wrap {
                        kids.map(|c| c.layout.min_w + side_margins(c))
                            .fold(0.0, f32::max)
                    } else {
                        kids.map(|c| c.layout.min_w + side_margins(c)).sum::<f32>() + gaps(gap, k)
                    };
                    (pref, min)
                }
                Direction::Column => {
                    let pref = kids
                        .clone()
                        .map(|c| c.layout.pref_w + side_margins(c))
                        .fold(0.0, f32::max);
                    let min = kids
                        .map(|c| c.layout.min_w + side_margins(c))
                        .fold(0.0, f32::max);
                    match n.layout.used.multicol {
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
            pref + f
                .clone()
                .map(|c| c.layout.pref_w + side_margins(c))
                .sum::<f32>(),
            f.map(|c| c.layout.min_w + side_margins(c))
                .fold(min, f32::max),
        )
    };
    // A fixed height and an aspect ratio give the width.
    let (pref, min) = match (
        n.layout.used.aspect > 0.0,
        n.layout.used.height,
        n.layout.used.width,
    ) {
        (true, Size::Fixed(h), Size::Fit { .. }) => {
            let w = (h - n.layout.used.padding.vertical()).max(0.0) * n.layout.used.aspect;
            (w, w)
        }
        _ => (pref, min),
    };
    let pad = n.layout.used.padding.horizontal();
    // A horizontal scroller never shrinks its content: it overflows and
    // scrolls. A field scrolls its own text too: its placeholder or its value
    // never holds it wider than its row.
    let min = if n.layout.used.scroll_x || matches!(n.widget, Widget::Field(_)) {
        0.0
    } else {
        min
    };
    let (p, m) = apply_size(n.layout.used.width, pad + pref, pad + min);
    let (cp, cm) = (pad + pref, pad + min);
    let el = n.layout.used;
    nodes[i].layout.min_content[0] = cm;
    nodes[i].layout.pref_w = el.bounded(0, None, cp, cm, p);
    nodes[i].layout.min_w = el.bounded(0, None, cp, cm, m);
}

/// A child's width across a column (or height across a row), within its
/// bounds against `basis` (the parent's inner size, its percentages' basis).
pub(crate) fn cross_size(n: &Node, axis: usize, inner: f32, basis: f32, stretch: bool) -> f32 {
    let (size, pref, min) = if axis == 0 {
        (n.layout.used.width, n.layout.pref_w, n.layout.min_w)
    } else {
        (n.layout.used.height, n.layout.pref_h, n.layout.min_h)
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
    n.layout.used.bounded(axis, Some(basis), pref, min, v)
}

/// Whether node `n` lies in a line beside its siblings (CSS 2 §9.2.2): a
/// box displayed `inline-*`, a text that is not block-level, or an image.
pub(in crate::layout) fn inline_level(n: &Node) -> bool {
    n.layout.used.inline
        || match n.content {
            Content::Text(_) | Content::Rich(_) => !n.layout.used.block,
            Content::Image { .. } => true,
            Content::None => false,
        }
}

/// Node `c`'s size along `axis` held to its bounds against `basis`.
pub(in crate::layout) fn clamp_to(n: &Node, axis: usize, basis: Option<f32>, v: f32) -> f32 {
    let (pref, min) = if axis == 0 {
        (n.layout.pref_w, n.layout.min_w)
    } else {
        (n.layout.pref_h, n.layout.min_h)
    };
    n.layout.used.bounded(axis, basis, pref, min, v)
}

pub(in crate::layout) fn distribute_w(nodes: &mut [Node], i: usize, measure: &mut Measure<'_>) {
    // A container query reads the element's own inline size, now known:
    // its frame's width (a vertical element's height). Its patch sets the
    // physical sides and axes.
    if nodes[i].props.variants.iter().any(|v| v.when.container) {
        let w = nodes[i].layout.w;
        super::writing::query(&mut nodes[i], w);
    }
    let inner = (nodes[i].layout.w - nodes[i].layout.used.padding.horizontal()).max(0.0);
    // Out of the flow: sized against the parent's box.
    let outer = nodes[i].layout.w;
    for c in nodes[i].out.clone() {
        nodes[c].layout.w = out_size(&nodes[c], 0, outer, outer);
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
                let at = |w: f32| text_lines(t, m, Some(w));
                let mut lines = text_lines(t, m, t.wrap.then_some(width));
                if t.wrap && lines.len() > 1 && t.wrap_style != WrapStyle::Auto {
                    lines = rewrap(lines, t.wrap_style, width, at);
                }
                if t.ellipsis {
                    truncate_lines(&mut lines, t, m, width);
                }
                lines
            });
        nodes[i].layout.lines = lines;
        nodes[i].layout.wrap_lines.clear();
        return;
    }
    if let Content::Rich(rc) = &nodes[i].content {
        let v = nodes[i].rich_vertical();
        let lines = crate::rich::rich_lines(rc, fonts, rc.wrap.then_some(inner + 0.5), v);
        nodes[i].layout.rich_lines = lines;
        nodes[i].layout.wrap_lines.clear();
        return;
    }
    let el = nodes[i].layout.used;
    // A float shrinks to fit (CSS 2 §10.3.5); its `auto` margins are 0.
    for c in nodes[i].floats.clone() {
        let n = &nodes[c];
        let fill = (inner - n.layout.used.margin[1].of(inner) - n.layout.used.margin[3].of(inner))
            .max(0.0);
        nodes[c].layout.w = cross_size(n, 0, fill, inner, false);
        nodes[c].layout.margin_x = place_sides(&nodes[c], inner, 0.0);
    }
    let kids = nodes[i].children.clone();
    if kids.is_empty() {
        return;
    }
    match nodes[i].props.layout {
        LayoutMode::Stack => {
            for &c in &kids {
                let n = &nodes[c];
                let ins = n.layout.insets;
                let w = match (ins.start, ins.end) {
                    (Some(s), Some(e)) if !matches!(n.layout.used.width, Size::Fixed(_)) => {
                        clamp_to(n, 0, Some(inner), (inner - s - e).max(0.0))
                    }
                    _ => cross_size(n, 0, inner, inner, el.cross == Cross::Stretch),
                };
                nodes[c].layout.w = w;
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
                nodes[i].layout.cols = cols;
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
                let fill =
                    (inner - n.layout.used.margin[1].of(inner) - n.layout.used.margin[3].of(inner))
                        .max(0.0);
                let basis = match n.layout.used.width {
                    Size::Grow { .. } | Size::Fit { .. } => fill,
                    _ => inner,
                };
                let stretch =
                    !el.wrap && n.layout.used.align_self.unwrap_or(el.cross) == Cross::Stretch;
                let w = cross_size(n, 0, basis, inner, stretch);
                // In a horizontal scroller a fitting child keeps its width.
                let w = match n.layout.used.width {
                    Size::Fit { .. } if el.scroll_x => w.max(n.layout.pref_w),
                    _ => w,
                };
                nodes[c].layout.w = w;
                nodes[c].layout.margin_x = place_sides(&nodes[c], inner, fill - w);
            }
        }
        Direction::Row if !el.wrap => {
            let sides: f32 = kids
                .iter()
                .map(|&c| {
                    nodes[c].layout.used.margin[1].of(inner)
                        + nodes[c].layout.used.margin[3].of(inner)
                })
                .sum();
            let avail = inner - gaps(el.main_gap(), kids.len()) - sides;
            let sizes = main_sizes(nodes, &kids, true, inner, avail, !el.scroll_x);
            let free = avail - sizes.iter().sum::<f32>();
            for (&c, w) in kids.iter().zip(sizes) {
                nodes[c].layout.w = w;
            }
            line_sides(nodes, &kids, inner, free);
        }
        Direction::Row => {
            // Each child at its preferred width, capped at the row (its
            // hypothetical size, by CSS flex), then lines.
            let gap = el.main_gap();
            let (base, sides) = row_bases(nodes, &kids, inner);
            let mut rows = Vec::new();
            let mut start = 0;
            let mut used = 0.0f32;
            for (k, (&b, &m)) in base.iter().zip(&sides).enumerate() {
                let b = b + m;
                // Half a pixel of slack, as text wrapping has: a fit row
                // snapped a fraction under its children's sum keeps them
                // on one line, over by less than half a pixel.
                // In lines, a block-level child is a line of its own.
                let forced = nodes[i].layout.line_box.is_some()
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
            let css = css_line(nodes, &kids);
            for &(s, e) in &rows {
                size_row(
                    nodes,
                    &kids[s..e],
                    css,
                    &base[s..e],
                    &sides[s..e],
                    gap,
                    inner,
                    inner,
                );
            }
            // Its lines as its width breaks them: floats shorten them again.
            nodes[i].layout.wrap_lines.clear();
            nodes[i].layout.rows = rows;
        }
    }
}

/// A wrapping row's children before lines: each one's hypothetical width
/// (its preferred width capped at the row, by CSS flex) and its left and
/// right margins against `inner`.
pub(crate) fn row_bases(nodes: &[Node], kids: &[usize], inner: f32) -> (Vec<f32>, Vec<f32>) {
    let css = css_line(nodes, kids);
    kids.iter()
        .map(|&c| {
            let n = &nodes[c];
            let w = if css {
                flex_inputs(n, true, inner).hypo
            } else {
                let w = n
                    .layout
                    .used
                    .width
                    .definite(n.layout.pref_w, n.layout.min_w, Some(inner))
                    .unwrap_or_else(|| n.layout.pref_w.min(inner).max(n.layout.min_w));
                clamp_to(n, 0, Some(inner), w)
            };
            (
                w,
                n.layout.used.margin[1].of(inner) + n.layout.used.margin[3].of(inner),
            )
        })
        .unzip()
}

/// One line of a wrapping row flexed into `room` across (its line box:
/// the row's content width, or what floats leave of it), from its
/// children's `base` widths and `sides`; `inner` is what percentages and
/// bounds read.
#[allow(clippy::too_many_arguments)]
pub(in crate::layout) fn size_row(
    nodes: &mut [Node],
    line: &[usize],
    css: bool,
    base: &[f32],
    sides: &[f32],
    gap: f32,
    inner: f32,
    room: f32,
) {
    let avail = room - gaps(gap, line.len()) - sides.iter().sum::<f32>();
    let sizes = if css {
        let items: Vec<FlexInputs> = line
            .iter()
            .map(|&c| flex_inputs(&nodes[c], true, inner))
            .collect();
        css_flex(&items, avail)
    } else {
        let mut sizes = base.to_vec();
        let grow: Vec<Option<f32>> = line
            .iter()
            .map(|&c| match nodes[c].layout.used.width {
                Size::Grow { .. } => Some(nodes[c].layout.used.width.bounds().1),
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
        nodes[c].layout.w = w;
    }
    line_sides(nodes, line, inner, free);
}

/// A flex line's left and right margins: `free` split evenly between its
/// `auto` ones (CSS Flexbox §8.1), each other one against `basis`.
fn line_sides(nodes: &mut [Node], line: &[usize], basis: f32, free: f32) {
    let autos = line
        .iter()
        .map(|&c| {
            nodes[c].layout.used.margin[1].auto as usize
                + nodes[c].layout.used.margin[3].auto as usize
        })
        .sum::<usize>();
    let each = if autos > 0 {
        free.max(0.0) / autos as f32
    } else {
        0.0
    };
    for &c in line {
        let [l, r] = [
            nodes[c].layout.used.margin[3],
            nodes[c].layout.used.margin[1],
        ];
        let side = |m: crate::layout::Margin| if m.auto { each } else { m.of(basis) };
        nodes[c].layout.margin_x = [side(l), side(r)];
    }
}
