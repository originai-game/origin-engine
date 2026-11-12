use super::*;

/// Node `i`'s line boxes (CSS 2.1 §10.8) where they hold a box with a
/// `vertical-align` other than `baseline`, `top` or `bottom`: each such
/// line as tall as its strut, its baseline-aligned boxes and its shifted
/// boxes reach (`Node::inline_rows`), each box's place in it
/// (`Node::inline_y`). Other lines keep their tallest child's height.
pub(in crate::layout) fn inline_rows(nodes: &mut [Node], i: usize, measure: &Measure<'_>) {
    use crate::VerticalAlign as V;
    let n = &nodes[i];
    let shifted = |nodes: &[Node], c: usize| {
        !matches!(
            nodes[c].layout.used.valign,
            V::Baseline | V::Top | V::Bottom
        )
    };
    if n.layout.line_box.is_none() || !n.children.iter().any(|&c| shifted(nodes, c)) {
        if !nodes[i].layout.inline_rows.is_empty() {
            nodes[i].layout.inline_rows.clear();
        }
        return;
    }
    let fonts = measure.fonts;
    // The parent's text: the block's first text (an anonymous inline box
    // carries the block's font), else the body step.
    let (step, font) = n
        .children
        .iter()
        .find_map(|&c| match &*nodes[c].content {
            Content::Text(t) => Some((nodes[c].used_text(t).step, t.font)),
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
    let rows = n.layout.rows.clone();
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
            let (mt, mb) = (span(k.layout.margin_sets[0]), span(k.layout.margin_sets[1]));
            let b = layout_baseline(nodes, c, fonts).unwrap_or(k.layout.pref_h);
            let (above, below) = (mt + b, k.layout.pref_h - b + mb);
            let place = match k.layout.used.valign {
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
            nodes[c].layout.inline_y = at - above + span(nodes[c].layout.margin_sets[0]);
        }
        heights.push(Some(height));
    }
    nodes[i].layout.inline_rows = heights;
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

pub(in crate::layout) fn cross_offset(cross: Cross, room: f32, size: f32, safe: bool) -> f32 {
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
    let el = nodes[i].layout.used;
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
            let [ml, mr] = k.layout.margin_x;
            nodes[c].layout.x = 2.0 * x0 + inner_w - k.layout.x - k.layout.w - mr + ml;
        }
        let k = &nodes[c];
        if flip_y {
            let (mt, mb) = (span(k.layout.margin_sets[0]), span(k.layout.margin_sets[1]));
            nodes[c].layout.y = 2.0 * y0 + inner_h - k.layout.y - k.layout.h - mb + mt;
        }
    }
}

pub(crate) fn position(nodes: &mut [Node], i: usize, scroll: &dyn Fn(u64) -> (f32, f32)) {
    place_children(nodes, i, scroll);
    super::multicol::place(nodes, i);
    if nodes[i].layout.used.flip {
        let n = &nodes[i];
        let sx = if n.layout.used.scroll_x {
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
        Rect::new(
            nodes[at].layout.x,
            nodes[at].layout.y,
            nodes[at].layout.w,
            nodes[at].layout.h,
        )
    });
    for c in kids {
        let ins = nodes[c].layout.insets;
        let (x, y, w, h) = (
            nodes[c].layout.x,
            nodes[c].layout.y,
            nodes[c].layout.w,
            nodes[c].layout.h,
        );
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
        nodes[c].layout.x += dx;
        nodes[c].layout.y += dy;
        // The edges a sticky box is held to (`scroll-state(stuck)`).
        use crate::doc::media::scroll::{BOTTOM, LEFT, RIGHT, TOP};
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

/// The insets an in-flow child is placed by in a stack or a grid cell:
/// none for a relative or sticky one, which moves after.
fn flow_insets(n: &Node) -> Edges {
    match n.props.position {
        Position::Relative | Position::Sticky => Edges::default(),
        _ => n.layout.insets,
    }
}

fn place_children(nodes: &mut [Node], i: usize, scroll: &dyn Fn(u64) -> (f32, f32)) {
    // Out of the flow: placed by its insets in the parent's box (an
    // anchored one is moved onto its target once every layer is placed).
    let bx = Rect::new(
        nodes[i].layout.x,
        nodes[i].layout.y,
        nodes[i].layout.w,
        nodes[i].layout.h,
    );
    for c in nodes[i].out.clone() {
        let (w, h) = (nodes[c].layout.w, nodes[c].layout.h);
        if let Some(v) = nodes[c].props.view_place {
            // Its anchor on the point the view box shows `at` at (meet,
            // centred), scaled about it by the fit when drawn.
            let [vx, vy, vw, vh] = v.view_box;
            let k = (bx.width / vw.max(1e-6)).min(bx.height / vh.max(1e-6));
            let off = (v.at[0] - (vx + vw / 2.0)) * k;
            let ax = v.anchor_x * w;
            nodes[c].layout.x = bx.x + bx.width / 2.0 + off - ax;
            nodes[c].layout.y =
                bx.y + bx.height / 2.0 + (v.at[1] - (vy + vh / 2.0)) * k - v.anchor_y;
            // Right to left the layout is mirrored whole; the draw moves
            // the mirrored box back by its offset `off` from the centre.
            nodes[c].layout.fit = [k, v.anchor_x, v.anchor_y, off];
            continue;
        }
        let (x, y) = place_out(&bx, &nodes[c], w, h);
        nodes[c].layout.x = x;
        nodes[c].layout.y = y;
    }
    if nodes[i].layout_mode() != LayoutMode::Flex && matches!(*nodes[i].content, Content::None) {
        let n = &nodes[i];
        let (sx, sy) = if n.layout.used.scroll || n.layout.used.scroll_x {
            scroll(n.key)
        } else {
            (0.0, 0.0)
        };
        let inner = Rect::new(
            n.layout.x + n.layout.used.padding.left
                - if n.layout.used.scroll_x {
                    sx.clamp(0.0, n.max_scroll_x())
                } else {
                    0.0
                },
            n.layout.y + n.layout.used.padding.top
                - if n.layout.used.scroll {
                    sy.clamp(0.0, n.max_scroll())
                } else {
                    0.0
                },
            (n.layout.w - n.layout.used.padding.horizontal()).max(0.0),
            (n.layout.h - n.layout.used.padding.vertical()).max(0.0),
        );
        let (align, cross) = (n.layout.used.align, n.layout.used.cross);
        let kids = n.children.clone();
        if n.layout_mode() == LayoutMode::Stack {
            let mut extent = 0.0f32;
            for &c in &kids {
                let (w, h) = (nodes[c].layout.w, nodes[c].layout.h);
                let (x, y) = place_in(&inner, w, h, &flow_insets(&nodes[c]), align, cross);
                nodes[c].layout.x = x;
                nodes[c].layout.y = y;
                extent = extent.max(w);
            }
            nodes[i].layout.content_w = extent;
        } else if nodes[i].layout.grid.is_some() {
            let areas = super::grid::areas_of(nodes, i);
            let mut extent = 0.0f32;
            for (k, &c) in kids.iter().enumerate() {
                let Some(Some((a, justify, align))) = areas.get(k).copied() else {
                    continue;
                };
                // Its margins inset its grid area; `auto` ones align it there
                // (CSS Grid §11.2), else its `justify-self` and `align-self`.
                let k = &nodes[c];
                let [ml, mr] = k.layout.margin_x;
                let [mt, mb] = k.layout.margin_sets.map(|(pos, neg)| pos + neg);
                let cell = Rect::new(
                    inner.x + a.x + ml,
                    inner.y + a.y + mt,
                    (a.width - ml - mr).max(0.0),
                    (a.height - mt - mb).max(0.0),
                );
                let (w, h) = (k.layout.w, k.layout.h);
                let justify = match justify {
                    Cross::Center => Align::Center,
                    Cross::End => Align::End,
                    _ => Align::Start,
                };
                let (x, y) = place_in(&cell, w, h, &flow_insets(k), justify, align);
                let y = auto_offset(k, 0, cell.height - h).map_or(y, |o| cell.y + o);
                nodes[c].layout.x = x;
                nodes[c].layout.y = y;
                extent = extent.max(a.x + a.width);
            }
            nodes[i].layout.content_w = extent;
            super::grid::place_table_parts(nodes, i, inner.x, inner.y);
        }
        return;
    }
    // The content's extent along x, before the offsets (the horizontal range).
    let content_w = {
        let n = &nodes[i];
        let kids = &n.children;
        let gap = n.layout.used.main_gap();
        match n.layout.used.direction {
            Direction::Column if n.layout.used.wrap && !n.layout.rows.is_empty() => {
                n.layout
                    .rows
                    .iter()
                    .map(|&(s, e)| {
                        kids[s..e]
                            .iter()
                            .map(|&c| outer_w(&nodes[c]))
                            .fold(0.0, f32::max)
                    })
                    .sum::<f32>()
                    + gaps(n.layout.used.line_gap(), n.layout.rows.len())
            }
            Direction::Column => kids.iter().map(|&c| outer_w(&nodes[c])).fold(0.0, f32::max),
            Direction::Row if n.layout.used.wrap && !n.layout.rows.is_empty() => n
                .layout
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
    nodes[i].layout.content_w = content_w;
    let n = &nodes[i];
    let el = n.layout.used;
    let inner_w = n.layout.w - el.padding.horizontal();
    let inner_h = n.layout.h - el.padding.vertical();
    let mut x0 = n.layout.x + el.padding.left;
    let mut y0 = n.layout.y + el.padding.top;
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
    let wrapped = el.wrap && !n.layout.rows.is_empty();
    let rows = if wrapped {
        n.layout.rows.clone()
    } else {
        vec![(0, kids.len())]
    };
    let (safe_main, safe_cross) = (el.safe & 1 != 0, el.safe & 2 != 0);
    let align = line_align(&el);
    // Each line's size across the axis, and where the lines start and
    // what lies between them (`align-content`, CSS Flexbox §9.4).
    let across = |c: usize, row: bool| {
        if row {
            nodes[c].layout.h + cross_margins(&nodes[c])
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
                n.layout.inline_rows.get(r).copied().flatten()
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
                // A line box floats shorten (`Node::wrap_lines`): its
                // shift right and down, and its width.
                let [dx, dy, room_w] = match nodes[i].layout.wrap_lines.get(line_k) {
                    Some(&l) if wrapped => l,
                    _ => [0.0, 0.0, inner_w],
                };
                let mut free = room_w - used;
                if el.scroll_x {
                    free = free.max(0.0);
                }
                let (start, between) = main_offsets(align, free, k, safe_main);
                let mut x = x0 + dx + start;
                for &c in line {
                    let (w, h) = (nodes[c].layout.w, nodes[c].layout.h);
                    let (top, around) = (
                        span(nodes[c].layout.margin_sets[0]),
                        cross_margins(&nodes[c]),
                    );
                    let [ml, mr] = nodes[c].layout.margin_x;
                    let cross = nodes[c].layout.used.align_self.unwrap_or(el.cross);
                    let safe = safe_cross || nodes[c].layout.used.safe & 2 != 0;
                    nodes[c].layout.x = x + ml;
                    nodes[c].layout.y = y0
                        + line_at
                        + dy
                        + if held[line_k].is_some() {
                            // Where its line box put it (`inline_rows`).
                            nodes[c].layout.inline_y
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
                    .map(|&c| nodes[c].layout.h + cross_margins(&nodes[c]))
                    .sum::<f32>()
                    + gaps(gap, k);
                let free = inner_h - used;
                let (start, between) = main_offsets(align, free, k, safe_main);
                let mut y = y0 + start;
                for &c in line {
                    let (w, h) = (nodes[c].layout.w, nodes[c].layout.h);
                    let [ml, mr] = nodes[c].layout.margin_x;
                    let (top, bottom) = (
                        span(nodes[c].layout.margin_sets[0]),
                        span(nodes[c].layout.margin_sets[1]),
                    );
                    let cross = nodes[c].layout.used.align_self.unwrap_or(el.cross);
                    let safe = safe_cross || nodes[c].layout.used.safe & 2 != 0;
                    nodes[c].layout.y = y + top;
                    nodes[c].layout.x =
                        x0 + line_at + ml + cross_offset(cross, room - ml - mr, w, safe);
                    y += top + h + bottom + gap + between;
                }
                line_at += room + el.line_gap() + line_between;
            }
            Direction::Column => {
                let (lead, spacing, trail) = column_spacing(nodes, i);
                let used: f32 = line.iter().map(|&c| nodes[c].layout.h).sum::<f32>()
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
                            nodes[c].layout.used.margin[0].auto as usize
                                + nodes[c].layout.used.margin[2].auto as usize
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
                    let (w, h) = (nodes[c].layout.w, nodes[c].layout.h);
                    let [top, bottom] = [0, 2].map(|e| {
                        if nodes[c].layout.used.margin[e].auto {
                            each
                        } else {
                            0.0
                        }
                    });
                    let [ml, mr] = nodes[c].layout.margin_x;
                    let room = inner_w.max(if el.scroll_x { ml + w + mr } else { 0.0 }) - ml - mr;
                    let sideways =
                        nodes[c].layout.used.margin[1].auto || nodes[c].layout.used.margin[3].auto;
                    let cross = nodes[c].layout.used.align_self.unwrap_or(el.cross);
                    let safe = safe_cross || nodes[c].layout.used.safe & 2 != 0;
                    nodes[c].layout.y = y + top;
                    nodes[c].layout.x = x0
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
        let [fx, fy] = nodes[c].layout.float_at;
        nodes[c].layout.x = x0 + fx;
        nodes[c].layout.y = y0 + fy;
    }
}

/// Its width with its left and right margins as laid out.
fn outer_w(n: &Node) -> f32 {
    n.layout.w + n.layout.margin_x[0] + n.layout.margin_x[1]
}

/// Where its `auto` margins on edges `edge` and `edge + 2` (top and bottom,
/// or right and left) put it in `free` room: centred between two, at the
/// far end after a leading one, at the start before a trailing one. `None`
/// with neither: its parent's alignment places it.
pub(in crate::layout) fn auto_offset(n: &Node, edge: usize, free: f32) -> Option<f32> {
    let free = free.max(0.0);
    let (lead, trail) = if edge == 0 {
        (n.layout.used.margin[0].auto, n.layout.used.margin[2].auto)
    } else {
        (n.layout.used.margin[3].auto, n.layout.used.margin[1].auto)
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
/// An axis with no inset keeps its static position ([`static_position`]).
pub(crate) fn on_screen(
    nodes: &[Node],
    i: usize,
    screen: &Rect,
    scroll: &dyn Fn(u64) -> (f32, f32),
) -> Rect {
    let n = &nodes[i];
    let w = out_size(n, 0, screen.width, screen.width);
    let h = out_size(n, 1, screen.height, screen.width);
    let (mut x, mut y) = place_out(screen, n, w, h);
    let ins = &n.layout.insets;
    let free_x = ins.start.is_none() && ins.end.is_none() && ins.across.is_none() && !ins.center_x;
    let free_y = ins.top.is_none() && ins.bottom.is_none() && !ins.center_y;
    if (free_x || free_y) && n.up != NONE {
        let (sx, sy) = static_position(nodes, i, w, h, screen.width, scroll);
        if free_x {
            x = sx;
        }
        if free_y {
            y = sy;
        }
    }
    Rect::new(x, y, w, h)
}

/// The static position of out-of-flow node `i`, `w`×`h` (CSS 2.1 §10.3.7
/// and §10.6.4): where it would have stood in the flow of the box it lays
/// out in, which an axis with no inset keeps. In a block it follows the
/// sibling before it in the flow; in a flex container it is placed as the
/// only item would be (CSS Flexbox §4.1), by `justify-content` and its
/// `align-self`; in a stack by the stack's alignment; in a grid at the
/// content box's start. Its margins' percentages read `basis`.
fn static_position(
    nodes: &[Node],
    i: usize,
    w: f32,
    h: f32,
    basis: f32,
    scroll: &dyn Fn(u64) -> (f32, f32),
) -> (f32, f32) {
    let n = &nodes[i];
    let p = &nodes[n.up];
    let el = p.layout.used;
    let (sx, sy) = if el.scroll || el.scroll_x {
        scroll(p.key)
    } else {
        (0.0, 0.0)
    };
    let inner = Rect::new(
        p.layout.x + el.padding.left - sx.clamp(0.0, p.max_scroll_x()),
        p.layout.y + el.padding.top - sy.clamp(0.0, p.max_scroll()),
        (p.layout.w - el.padding.horizontal()).max(0.0),
        (p.layout.h - el.padding.vertical()).max(0.0),
    );
    let [mt, mr, mb, ml] = n.layout.used.margin.map(|m| m.of(basis));
    let (room_w, room_h) = (inner.width - ml - mr, inner.height - mt - mb);
    let (mut x, y) = if p.layout_mode() == LayoutMode::Stack {
        // A stack places its children without their margins.
        place_in(&inner, w, h, &Edges::default(), el.align, el.cross)
    } else if p.layout_mode() != LayoutMode::Flex && matches!(*p.content, Content::None) {
        (inner.x + ml, inner.y + mt)
    } else {
        // The sibling before it in the flow of the same box.
        let siblings = &nodes[n.parent].ordered_children;
        let before = siblings
            .iter()
            .position(|&c| c == i)
            .and_then(|at| siblings[..at].iter().rfind(|c| p.children.contains(c)))
            .map(|&c| &nodes[c]);
        let cross = n.layout.used.align_self.unwrap_or(el.cross);
        let safe = (el.safe | n.layout.used.safe) & 2 != 0;
        let main = |free: f32| main_offsets(line_align(&el), free, 1, el.safe & 1 != 0).0;
        let row = el.direction == Direction::Row;
        let (x, y) = match (row, before) {
            // On a line, after the box before it.
            (true, Some(b)) if p.layout.line_box.is_some() => (
                b.layout.x + b.layout.w + b.layout.margin_x[1] + ml,
                b.layout.y,
            ),
            (true, _) if p.layout.line_box.is_some() => (inner.x + ml, inner.y + mt),
            (true, _) => (
                inner.x + main(room_w - w) + ml,
                inner.y + mt + cross_offset(cross, room_h, h, safe),
            ),
            (false, before) => (
                inner.x + ml + cross_offset(cross, room_w, w, safe),
                match before {
                    Some(b) if el.block => {
                        b.layout.y + b.layout.h + span(b.layout.margin_sets[1]) + mt
                    }
                    _ if el.block => inner.y + mt,
                    _ => inner.y + main(room_h - h) + mt,
                },
            ),
        };
        // A reversed axis runs from its other end, as its flow does.
        let (flip_x, flip_y) = if row {
            (el.reverse, el.wrap_reverse && el.wrap)
        } else {
            (el.wrap_reverse && el.wrap, el.reverse)
        };
        (
            if flip_x {
                2.0 * inner.x + inner.width - x - w - mr + ml
            } else {
                x
            },
            if flip_y {
                2.0 * inner.y + inner.height - y - h - mb + mt
            } else {
                y
            },
        )
    };
    if el.flip {
        x = 2.0 * inner.x + inner.width - x - w - mr + ml;
    }
    (x, y)
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

/// An out-of-flow element's size along `axis` of its containing block
/// `outer` long, its margins' percentages against `basis` (the block's
/// width), as CSS 2.1 §10.3.7 and §10.6.4 size an absolutely positioned
/// box: a definite size as written; `auto` between two set insets fills
/// the room between them less its margins, and with an inset or none it
/// shrinks to fit that room (its max-content, at most the room, at least
/// its min-content). `stretch` fills the room either way.
pub(super) fn out_size(n: &Node, axis: usize, outer: f32, basis: f32) -> f32 {
    let ins = &n.layout.insets;
    let (size, pref, min, a, b, [lead, trail]) = if axis == 0 {
        (
            n.layout.used.width,
            n.layout.pref_w,
            n.layout.min_w,
            ins.start,
            ins.end,
            [3, 1],
        )
    } else {
        (
            n.layout.used.height,
            n.layout.pref_h,
            n.layout.min_h,
            ins.top,
            ins.bottom,
            [0, 2],
        )
    };
    let margins = n.layout.used.margin[lead].of(basis) + n.layout.used.margin[trail].of(basis);
    let room = (outer - a.unwrap_or(0.0) - b.unwrap_or(0.0) - margins).max(0.0);
    let (lo, hi) = size.bounds();
    let v = match (a, b, size) {
        (_, _, Size::Fixed(v)) => v,
        (_, _, Size::Percent(p)) => p * outer,
        (_, _, Size::Calc(c)) => c.of(outer),
        (Some(_), Some(_), Size::Fit { .. }) | (_, _, Size::Grow { .. }) => room.clamp(lo, hi),
        (_, _, Size::Fit { .. }) => pref.min(room).max(min).clamp(lo, hi),
        // A content size is its preferred size, which layout already holds.
        (_, _, Size::MinContent | Size::MaxContent) => pref,
    };
    clamp_to(n, axis, Some(outer), v)
}

/// Where an out-of-flow `w`×`h` element stands in its containing block
/// `bx` (CSS 2.1 §10.3.7 and §10.6.4, CSS Position 3 §4): its margin
/// edge on each set inset; between two, `auto` margins share what is left
/// (across, a start margin never goes under 0), and with none `auto` an
/// over-constrained box keeps its start inset. With no inset on an axis it
/// stands at the block's start, past its margin. Centring and fractional
/// insets place it as [`place_in`] does.
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
