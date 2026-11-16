//! CSS Multi-column Layout 1: a block's flow in columns. The column count
//! and width follow §3.4 from `column-count`, `column-width` and the
//! column gap; the flow is balanced across them (`column-fill: balance`),
//! or filled in turn to a definite height (`column-fill: auto`), and
//! columns past the count overflow in the inline direction (§8.2). A
//! `column-span: all` child spans every column and starts a new set of
//! columns below it; `column-rule` is drawn in the middle of each gap
//! between two columns that hold content.
//!
//! Fragmentation (CSS Fragmentation 3): the flow breaks between children
//! and between the lines of a text, keeping `orphans` lines of a
//! paragraph before a break and `widows` after it (2 each by default),
//! and never inside a text with `break-inside: avoid`. `break-before` and
//! `break-after` `column` (or `page`, `always`) force a break. Any other
//! box is monolithic: it moves to the next column whole.

use super::*;

/// A multi-column container's authored columns.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Multicol {
    /// `column-count` (0: auto).
    pub count: u16,
    /// `column-width` in px (0: auto).
    pub width: f32,
    /// The used `column-gap` in px (`normal`: 1em).
    pub gap: f32,
    /// `column-fill: auto`: columns fill in turn, not balanced.
    pub fill_auto: bool,
    /// `column-rule` (`None`: style `none` or width 0).
    pub rule: Option<crate::decor::Line>,
    /// The rule's colour is the theme's ink (`currentcolor` with no colour
    /// of its own).
    pub current: bool,
}

/// A box's breaks in a fragmented flow (CSS Fragmentation 3 §3).
pub const BREAK_BEFORE: u8 = 1;
pub const BREAK_AFTER: u8 = 2;
pub const AVOID_INSIDE: u8 = 4;

/// Lines a break inside a paragraph leaves before and after it by
/// default (`orphans` and `widows`' initial value).
const KEEP: usize = 2;

/// The used column count and width (CSS Multicol §3.4) in `inner` with
/// gap `gap`.
pub(crate) fn used(m: &Multicol, inner: f32) -> (usize, f32) {
    let gap = m.gap;
    let fit = |w: f32| (((inner + gap) / (w + gap)).floor() as usize).max(1);
    let n = match (m.count, m.width > 0.0) {
        (0, false) => 1,
        (c, false) => c as usize,
        (0, true) => fit(m.width),
        (c, true) => (c as usize).min(fit(m.width)),
    };
    (n, ((inner - gap * (n - 1) as f32) / n as f32).max(0.0))
}

/// Its intrinsic widths (pref, min) from its children's (`col`: their
/// widest pref, `least`: their widest min): `column-width` is each
/// column's least, and the count of columns sets the preferred width.
pub(crate) fn intrinsic(m: &Multicol, col: f32, least: f32) -> (f32, f32) {
    let one = if m.width > 0.0 {
        m.width.max(least)
    } else {
        col
    };
    let n = m.count.max(1) as f32;
    let min = if m.width > 0.0 {
        m.width.min(one).max(least)
    } else {
        least
    };
    (n * one + (n - 1.0) * m.gap, min)
}

/// One unbreakable piece of a run of the flow: its top and bottom from the
/// run's top, the child it belongs to (its index in the run), the first
/// line it holds (0 for a whole box) and whether a break comes before it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Piece {
    pub top: f32,
    pub bottom: f32,
    pub kid: usize,
    pub line: usize,
    pub forced: bool,
}

/// The pieces of the run of children `run`, each at `tops[k]` with height
/// `h(child)`: a child whole, or a text's lines in groups a break may
/// separate.
fn pieces(nodes: &[Node], run: &[usize], tops: &[f32], h: &dyn Fn(&Node) -> f32) -> Vec<Piece> {
    let mut out = Vec::new();
    let mut after = false;
    for (k, &c) in run.iter().enumerate() {
        let n = &nodes[c];
        let (top, bottom) = (tops[k], tops[k] + h(n));
        let forced = k > 0 && (after || n.layout.used.breaks & BREAK_BEFORE != 0);
        after = n.layout.used.breaks & BREAK_AFTER != 0;
        let lines = n.layout.lines.len();
        let lh = match &*n.content {
            Content::Text(t)
                if n.layout.used.breaks & AVOID_INSIDE == 0 && !n.layout.used.scroll =>
            {
                n.used_text(t).line_height()
            }
            _ => 0.0,
        };
        // `orphans` lines before a break, `widows` after it (CSS
        // Fragmentation 3 §4.4).
        let [orphans, widows] = n.layout.used.keep.map(|k| match k {
            0 => KEEP,
            k => k as usize,
        });
        if lh <= 0.0 || lines < orphans + widows {
            out.push(Piece {
                top,
                bottom,
                kid: k,
                line: 0,
                forced,
            });
            continue;
        }
        // Breaks between lines `orphans..=lines - widows`.
        let y0 = top + n.layout.used.padding.top;
        let mut starts = vec![0];
        starts.extend(orphans..=lines - widows);
        for (j, &s) in starts.iter().enumerate() {
            let e = starts.get(j + 1).copied().unwrap_or(lines);
            out.push(Piece {
                top: if s == 0 { top } else { y0 + s as f32 * lh },
                bottom: if e == lines {
                    bottom
                } else {
                    y0 + e as f32 * lh
                },
                kid: k,
                line: s,
                forced: forced && s == 0,
            });
        }
    }
    out
}

/// Where `pieces` break into columns no taller than they must be for `n`
/// of them (balanced), or no taller than `limit` (filled in turn, or the
/// container's definite height a balanced set may not pass): the piece
/// each column starts at, and the tallest column.
pub(crate) fn balance(
    pieces: &[Piece],
    n: usize,
    limit: Option<f32>,
    fill: bool,
) -> (Vec<usize>, f32) {
    if pieces.is_empty() {
        return (Vec::new(), 0.0);
    }
    // Columns of at most `h` (a piece taller than that has one alone).
    let pack = |h: f32| {
        let mut starts = vec![0];
        let mut top = pieces[0].top;
        let mut tallest = 0.0f32;
        let mut over = f32::INFINITY;
        for (k, p) in pieces.iter().enumerate() {
            let fresh = k == starts[starts.len() - 1];
            if !fresh && (p.forced || p.bottom - top > h + 0.01) {
                if !p.forced {
                    over = over.min(p.bottom - top - h);
                }
                starts.push(k);
                top = p.top;
            }
            tallest = tallest.max(p.bottom - top);
        }
        (starts, tallest, over)
    };
    let capped = |(starts, tallest): (Vec<usize>, f32)| match limit {
        Some(cap) if tallest > cap + 0.01 => {
            let (s, t, _) = pack(cap);
            (s, t)
        }
        _ => (starts, tallest),
    };
    if let (true, Some(h)) = (fill, limit) {
        let (starts, tallest, _) = pack(h);
        return (starts, tallest);
    }
    let total = pieces[pieces.len() - 1].bottom - pieces[0].top;
    let mut h = pieces
        .iter()
        .map(|p| p.bottom - p.top)
        .fold(total / n.max(1) as f32, f32::max);
    // Raise the height by the least overflow until the pieces fit in `n`.
    for _ in 0..=pieces.len() {
        let (starts, tallest, over) = pack(h);
        if starts.len() <= n || !over.is_finite() {
            return capped((starts, tallest));
        }
        h += over;
    }
    let (starts, tallest, _) = pack(f32::INFINITY);
    capped((starts, tallest))
}

/// Its children's runs: the stretches between `column-span: all`
/// children (`Ok`), and the spanning children (`Err`), as indices of
/// `kids`.
fn runs(nodes: &[Node], kids: &[usize]) -> Vec<Result<std::ops::Range<usize>, usize>> {
    let mut out = Vec::new();
    let mut start = 0;
    for (k, &c) in kids.iter().enumerate() {
        if nodes[c].layout.used.span_all {
            if k > start {
                out.push(Ok(start..k));
            }
            out.push(Err(k));
            start = k + 1;
        }
    }
    if start < kids.len() {
        out.push(Ok(start..kids.len()));
    }
    out
}

/// Whether child `c` of a multi-column container lays out in a column (a
/// spanning one takes the whole width).
pub(crate) fn in_column(n: &Node) -> bool {
    !n.layout.used.span_all
}

/// The height columns may not pass: a definite height's content box.
fn limit(n: &Node) -> Option<f32> {
    match n.layout.used.height {
        Size::Fixed(v) => Some((v - n.layout.used.padding.vertical()).max(0.0)),
        _ => None,
    }
}

/// Node `i`'s content height in columns, its children laid out in one
/// column at `tops` (from its content box's top, in `children` order)
/// with their border-box heights `hs`: as [`place`] will lay them out.
pub(crate) fn balanced_height(nodes: &[Node], i: usize, tops: &[f32], hs: &[f32]) -> f32 {
    let Some(m) = nodes[i].layout.used.multicol else {
        return 0.0;
    };
    let cols = nodes[i].layout.cols.0.max(1);
    let kids = &nodes[i].children;
    let mut saved = 0.0f32;
    let mut bottom = 0.0f32;
    for run in runs(nodes, kids) {
        match run {
            Err(c) => bottom = bottom.max(tops[c] + hs[c] - saved),
            Ok(r) => {
                let top = tops[r.start];
                let rel: Vec<f32> = r.clone().map(|k| tops[k] - top).collect();
                let at = |n: &Node| {
                    let k = kids
                        .iter()
                        .position(|&c| std::ptr::eq(&nodes[c], n))
                        .unwrap_or(0);
                    hs[k]
                };
                let p = pieces(nodes, &kids[r.clone()], &rel, &at);
                let (_, tallest) = balance(&p, cols, limit(&nodes[i]), m.fill_auto);
                bottom = bottom.max(top - saved + tallest);
                let flow = p.last().map_or(0.0, |q| q.bottom);
                saved += flow - tallest;
            }
        }
    }
    bottom
}

/// Moves node `i`'s children, laid out in one column of the column width
/// by the flow, into its columns, splits a text across a break
/// ([`Node::frag`]) and keeps the rules between the columns.
pub(crate) fn place(nodes: &mut [Node], i: usize) {
    let Some(m) = nodes[i].layout.used.multicol else {
        return;
    };
    let (cols, w) = nodes[i].layout.cols;
    let kids = nodes[i].children.clone();
    let x0 = nodes[i].layout.x + nodes[i].layout.used.padding.left;
    let limit = limit(&nodes[i]);
    let mut rules = Vec::new();
    for &c in &kids {
        nodes[c].layout.frag.clear();
    }
    // Each run starts where the flow put its first child, moved up by what
    // the runs above it saved.
    let mut saved = 0.0f32;
    for run in runs(nodes, &kids) {
        let r = match run {
            Err(c) => {
                nodes[kids[c]].layout.y -= saved;
                continue;
            }
            Ok(r) => r,
        };
        let run = &kids[r];
        let top = nodes[run[0]].layout.y;
        let tops: Vec<f32> = run.iter().map(|&c| nodes[c].layout.y - top).collect();
        let p = pieces(nodes, run, &tops, &|n: &Node| n.layout.h);
        let (starts, tallest) = balance(&p, cols.max(1), limit, m.fill_auto);
        // Each child's move (dx, dy), from the column its first piece is in.
        let mut moved: Vec<Option<[f32; 2]>> = vec![None; run.len()];
        for (k, &s) in starts.iter().enumerate() {
            let end = starts.get(k + 1).copied().unwrap_or(p.len());
            let shift = [k as f32 * (w + m.gap), -p[s].top - saved];
            for (j, piece) in p[s..end].iter().enumerate() {
                let c = run[piece.kid];
                match moved[piece.kid] {
                    None => moved[piece.kid] = Some(shift),
                    // A text's lines from the top of this column on.
                    Some(first) if j == 0 && piece.line > 0 => {
                        let d = [shift[0] - first[0], shift[1] - first[1]];
                        nodes[c].layout.frag.push((piece.line as u32, d));
                    }
                    Some(_) => {}
                }
            }
            if k > 0 {
                // A rule in the middle of the gap, from the box's origin.
                let x = x0 + k as f32 * (w + m.gap) - m.gap / 2.0 - nodes[i].layout.x;
                rules.push([x, top - saved - nodes[i].layout.y, tallest]);
            }
        }
        for (k, &c) in run.iter().enumerate() {
            let [dx, dy] = moved[k].unwrap_or([0.0, -saved]);
            nodes[c].layout.x += dx;
            nodes[c].layout.y += dy;
            // A split text's box is its first fragment.
            if let Some(&(line, _)) = nodes[c].layout.frag.first()
                && let Some(q) = p.iter().find(|q| q.kid == k && q.line == line as usize)
            {
                nodes[c].layout.h = q.top - tops[k];
            }
        }
        let flow = p.last().map_or(0.0, |q| q.bottom);
        saved += flow - tallest;
    }
    nodes[i].layout.col_rules = rules;
}
