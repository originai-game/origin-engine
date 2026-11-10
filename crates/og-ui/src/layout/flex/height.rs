use super::*;

pub(in crate::layout) fn size_h(nodes: &mut [Node], i: usize) {
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
            let drop = n.wrap_lines.last().map_or(0.0, |l| l[1]);
            let h: f32 = n.rich_lines.iter().map(|l| l.height).sum::<f32>() + drop;
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
                    // Lines moved down past floats add their drop.
                    let drop = n.wrap_lines.last().map_or(0.0, |l| l[1]);
                    (pref + drop, min + drop)
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

pub(in crate::layout) fn distribute_h(nodes: &mut [Node], i: usize, measure: &mut Measure<'_>) {
    let el = nodes[i].el;
    let inner = (nodes[i].h - el.padding.vertical()).max(0.0);
    let outer = nodes[i].h;
    for c in nodes[i].out.clone() {
        nodes[c].h = out_size(&nodes[c], 1, outer, nodes[i].w);
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
    // CSS Flexbox §9.4: the hypothetical cross size uses the final main
    // size. A column's flexed height was not known in the widths pass.
    if el.direction == Direction::Column && el.multicol.is_none() {
        let basis = (nodes[i].w - el.padding.horizontal()).max(0.0);
        for c in kids {
            let n = &nodes[c];
            if n.el.aspect <= 0.0
                || !matches!(n.el.width, Size::Fit { .. })
                || n.el.align_self.unwrap_or(el.cross) == Cross::Stretch
            {
                continue;
            }
            let w =
                (n.h - n.el.padding.vertical()).max(0.0) * n.el.aspect + n.el.padding.horizontal();
            let (lo, hi) = n.el.width.bounds();
            let w = clamp_to(n, 0, Some(basis), w.clamp(lo, hi));
            if w == n.w {
                continue;
            }
            nodes[c].w = w;
            let order = preorder(nodes, c);
            for &j in &order {
                distribute_w(nodes, j, measure);
            }
            let mut touched = Vec::new();
            for &j in order.iter().rev() {
                inline_rows(nodes, j, measure);
                size_h(nodes, j);
                super::float::flow(nodes, j, measure, &mut touched);
            }
        }
    }
}

/// The room each line of a wrapping row gains from `align-content:
/// stretch` (CSS Flexbox §9.4): the free space down `inner` its lines
/// leave, shared evenly.
pub(super) fn line_extra(nodes: &[Node], i: usize, inner: f32) -> f32 {
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
pub(super) fn layout_baseline(nodes: &[Node], c: usize, fonts: &dyn Metrics) -> Option<f32> {
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
        Content::Rich(_) => {
            let drop = n.wrap_lines.first().map_or(0.0, |l| l[1]);
            Some(n.el.padding.top + drop + n.rich_lines.first()?.baseline)
        }
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
