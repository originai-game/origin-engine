//! CSS Grid layout (CSS Grid 1 and 2, masonry from Grid 3): line
//! resolution (§8.3), auto-placement (§8.5), the track sizing algorithm
//! (§11–§12), content alignment of the tracks (§10.5), subgrids (§9) and
//! masonry packing.

use super::*;
use crate::element::{Breadth, GridLine, GridPlace, GridSpec, Template};
use crate::layout::TablePart;

/// A grid's resolved tracks and where each child in the flow sits.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct GridLayout {
    /// Per child in the flow: its column, row, column span, row span (0-based).
    pub cells: Vec<(usize, usize, usize, usize)>,
    pub ncols: usize,
    pub nrows: usize,
    pub cols: Vec<f32>,
    pub rows: Vec<f32>,
    /// Each track's start from the content box's edge, after the tracks'
    /// content alignment, per axis.
    pub at: [Vec<f32>; 2],
    pub col_gap: f32,
    pub row_gap: f32,
    /// Implicit tracks before the explicit grid, per axis.
    lead: [usize; 2],
    /// The explicit tracks, auto repetitions expanded, per axis.
    explicit: [Vec<Track>; 2],
    /// `auto-fit` tracks no item takes, per axis.
    collapsed: [Vec<bool>; 2],
    /// The repetitions placement used, per axis.
    reps: [usize; 2],
    /// Masonry: each item's start along the masonry axis.
    masonry: Vec<f32>,
}

impl GridLayout {
    fn sizes(&self, axis: usize) -> &[f32] {
        if axis == 0 { &self.cols } else { &self.rows }
    }

    fn gap(&self, axis: usize) -> f32 {
        if axis == 0 {
            self.col_gap
        } else {
            self.row_gap
        }
    }

    /// The start and length of `n` tracks from track `s` along `axis`.
    fn span(&self, axis: usize, s: usize, n: usize) -> (f32, f32) {
        let sizes = self.sizes(axis);
        let at = &self.at[axis];
        if sizes.is_empty() {
            return (0.0, 0.0);
        }
        let s = s.min(sizes.len() - 1);
        let e = (s + n.max(1)).min(sizes.len()) - 1;
        let start = at.get(s).copied().unwrap_or(0.0);
        let end = at.get(e).copied().unwrap_or(start) + sizes[e];
        (start, (end - start).max(0.0))
    }
}

/// The axis's template kind of node `i` (a subgrid only inside a grid).
fn kind(nodes: &[Node], i: usize, axis: usize) -> Template {
    let n = &nodes[i];
    if n.props.layout != LayoutMode::Grid {
        return Template::Tracks;
    }
    match n.props.grid.lines[axis].kind {
        Template::Subgrid if n.up == NONE || nodes[n.up].props.layout != LayoutMode::Grid => {
            Template::Tracks
        }
        k => k,
    }
}

/// The named grid areas: name, first column, first row, end column, end row.
fn areas(spec: &GridSpec) -> Vec<(&str, usize, usize, usize, usize)> {
    let mut out: Vec<(&str, usize, usize, usize, usize)> = Vec::new();
    for (r, line) in spec.areas.iter().enumerate() {
        for (c, name) in line.split_whitespace().enumerate() {
            if name.chars().all(|c| c == '.') {
                continue;
            }
            match out.iter_mut().find(|a| a.0 == name) {
                Some(a) => {
                    a.1 = a.1.min(c);
                    a.2 = a.2.min(r);
                    a.3 = a.3.max(c + 1);
                    a.4 = a.4.max(r + 1);
                }
                None => out.push((name, c, r, c + 1, r + 1)),
            }
        }
    }
    out
}

/// A track size's fixed px when it has one (a percentage of `basis`).
fn fixed(b: Breadth, basis: Option<f32>) -> Option<f32> {
    match b {
        Breadth::Px(v) => Some(v),
        Breadth::Percent(p) => basis.map(|a| p * a),
        _ => None,
    }
}

/// The repetitions of an axis's `auto-fill`/`auto-fit` (CSS Grid §7.2.3.2):
/// the most that fit `avail` (else at least `min`; else one).
fn repetitions(
    spec: &GridSpec,
    axis: usize,
    avail: Option<f32>,
    min: Option<f32>,
    gap: f32,
) -> usize {
    let Some(rep) = &spec.lines[axis].repeat else {
        return 0;
    };
    let tracks = if axis == 0 { &spec.columns } else { &spec.rows };
    let basis = avail.or(min);
    let size = |t: &Track| {
        let (lo, hi, _) = t.breadths();
        fixed(hi, basis)
            .or(fixed(lo, basis))
            .unwrap_or(0.0)
            .max(fixed(lo, basis).unwrap_or(0.0))
    };
    let base: f32 = tracks.iter().map(size).sum::<f32>() + gap * tracks.len() as f32;
    let one: f32 = rep.tracks.iter().map(size).sum::<f32>() + gap * rep.tracks.len() as f32;
    if one <= 0.0 {
        return 1;
    }
    // Every track but the last is followed by a gap.
    let room = |n: usize| base + one * n as f32 - gap;
    match (avail, min) {
        (Some(a), _) => {
            let n = ((a + gap - base) / one).floor().max(1.0) as usize;
            n.min(10_000)
        }
        (None, Some(m)) => {
            let mut n = 1;
            while room(n) < m && n < 10_000 {
                n += 1;
            }
            n
        }
        _ => 1,
    }
}

/// An axis's explicit tracks and line names with `reps` auto repetitions.
fn expand(spec: &GridSpec, axis: usize, reps: usize) -> (Vec<Track>, Vec<Vec<String>>) {
    let tracks = if axis == 0 { &spec.columns } else { &spec.rows };
    let lines = &spec.lines[axis];
    let named = |k: usize| lines.names.get(k).cloned().unwrap_or_default();
    let Some(rep) = &lines.repeat else {
        let names = (0..=tracks.len()).map(named).collect();
        return (tracks.clone(), names);
    };
    let at = rep.at.min(tracks.len());
    let mut out: Vec<Track> = tracks[..at].to_vec();
    let mut names: Vec<Vec<String>> = (0..at).map(named).collect();
    let last = rep.names.last().cloned().unwrap_or_default();
    for r in 0..reps.max(1) {
        let mut first = if r == 0 { Vec::new() } else { last.clone() };
        first.extend(rep.names.first().cloned().unwrap_or_default());
        names.push(first);
        for (k, t) in rep.tracks.iter().enumerate() {
            out.push(*t);
            if k + 1 < rep.tracks.len() {
                names.push(rep.names.get(k + 1).cloned().unwrap_or_default());
            }
        }
    }
    names.push([last, rep.after.clone()].concat());
    for k in at + 1..=tracks.len() {
        names.push(named(k));
    }
    out.extend_from_slice(&tracks[at..]);
    (out, names)
}

/// The index (0-based, explicit line 0 the grid's first) of the `n`th line
/// called `name` (`n` < 0: from the end), past the explicit grid every
/// implicit line counting as one.
fn nth_named(names: &[Vec<String>], name: &str, n: i32) -> i32 {
    let has = |k: usize| names[k].iter().any(|m| m == name);
    let last = names.len() as i32 - 1;
    if n > 0 {
        let mut left = n;
        for k in 0..names.len() {
            if has(k) {
                left -= 1;
                if left == 0 {
                    return k as i32;
                }
            }
        }
        last + left
    } else {
        let mut left = -n;
        for k in (0..names.len()).rev() {
            if has(k) {
                left -= 1;
                if left == 0 {
                    return k as i32;
                }
            }
        }
        -left
    }
}

/// One axis's placement of an item (CSS Grid §8.3.1): its start and end
/// lines (explicit line 0 the first), or `Err(span)` to auto-place.
fn resolve(names: &[Vec<String>], start: &GridLine, end: &GridLine) -> Result<(i32, i32), usize> {
    let last = names.len() as i32 - 1;
    let line = |l: &GridLine, side: &str| -> Option<i32> {
        match l {
            GridLine::Line(0, name) => {
                let edge = format!("{name}-{side}");
                Some(if names.iter().any(|m| m.contains(&edge)) {
                    nth_named(names, &edge, 1)
                } else {
                    nth_named(names, name, 1)
                })
            }
            GridLine::Line(n, name) if name.is_empty() => {
                Some(if *n > 0 { n - 1 } else { last + 1 + n })
            }
            GridLine::Line(n, name) => Some(nth_named(names, name, *n)),
            _ => None,
        }
    };
    // A named span counts the lines called that name from the other end;
    // past the grid every implicit line does.
    let span_from = |from: i32, n: u32, name: &str, forward: bool| -> i32 {
        if name.is_empty() {
            return if forward {
                from + n as i32
            } else {
                from - n as i32
            };
        }
        let mut left = n as i32;
        let mut k = from;
        loop {
            k += if forward { 1 } else { -1 };
            let named = k >= 0 && k <= last && names[k as usize].iter().any(|m| m == name);
            if named || k < 0 || k > last {
                left -= 1;
            }
            if left == 0 {
                return k;
            }
        }
    };
    let (s, e) = (line(start, "start"), line(end, "end"));
    let (s, e) = match (s, e) {
        (Some(s), Some(e)) => (s, e),
        (Some(s), None) => match end {
            GridLine::Span(n, name) => (s, span_from(s, *n, name, true)),
            _ => (s, s + 1),
        },
        (None, Some(e)) => match start {
            GridLine::Span(n, name) => (span_from(e, *n, name, false), e),
            _ => (e - 1, e),
        },
        (None, None) => {
            // Auto-placed: a numbered span counts, a named one is 1.
            let span = |l: &GridLine| match l {
                GridLine::Span(n, name) if name.is_empty() => Some(*n as usize),
                GridLine::Span(..) => Some(1),
                _ => None,
            };
            return Err(span(start).or(span(end)).unwrap_or(1).max(1));
        }
    };
    if s == e {
        Ok((s, s + 1))
    } else {
        Ok((s.min(e), s.max(e)))
    }
}

/// An item's placement lines per axis, from its CSS lines, else its area,
/// else the house's numbers.
fn item_lines(p: &GridPlace) -> [[GridLine; 2]; 2] {
    let mut lines = p.lines.clone();
    let unset = |l: &[GridLine; 2]| l.iter().all(|l| *l == GridLine::Auto);
    if !p.area.is_empty() && lines.iter().all(unset) {
        let named = GridLine::Line(0, p.area.clone());
        return [[named.clone(), named.clone()], [named.clone(), named]];
    }
    for (axis, (at, span)) in [(p.column, p.column_span), (p.row, p.row_span)]
        .into_iter()
        .enumerate()
    {
        if unset(&lines[axis]) {
            if at > 0 {
                lines[axis] = [
                    GridLine::Line(i32::from(at), String::new()),
                    GridLine::Span(u32::from(span.max(1)), String::new()),
                ];
            } else if span > 1 {
                lines[axis] = [
                    GridLine::Span(u32::from(span), String::new()),
                    GridLine::Auto,
                ];
            }
        }
    }
    lines
}

/// The tracks a subgrid takes on `axis` in its parent: its span there.
fn subgrid_span(nodes: &[Node], i: usize, axis: usize) -> usize {
    let n = &nodes[i];
    let p = &n.props.place;
    let parent = &nodes[n.up].props.grid;
    if !p.area.is_empty()
        && p.lines.iter().flatten().all(|l| *l == GridLine::Auto)
        && let Some(a) = areas(parent).into_iter().find(|a| a.0 == p.area)
    {
        return if axis == 0 { a.3 - a.1 } else { a.4 - a.2 };
    }
    let lines = item_lines(p);
    let [s, e] = &lines[axis];
    match (s, e) {
        (GridLine::Line(a, x), GridLine::Line(b, y))
            if x.is_empty() && y.is_empty() && *a > 0 && *b > 0 =>
        {
            (a - b).unsigned_abs() as usize
        }
        (GridLine::Span(k, _), _) | (_, GridLine::Span(k, _)) => *k as usize,
        _ => {
            // Without a span, the tracks its own line names count.
            n.props.grid.lines[axis]
                .names
                .len()
                .saturating_sub(1)
                .max(1)
        }
    }
    .max(1)
}

/// The definite inner size of node `i` along `axis`, if it has one before
/// layout (a fixed size), else its definite max size (`.1`: its min).
fn definite_inner(n: &Node, axis: usize) -> (Option<f32>, Option<f32>) {
    let (size, pad) = if axis == 0 {
        (n.el.width, n.el.padding.horizontal())
    } else {
        (n.el.height, n.el.padding.vertical())
    };
    let inner = |v: f32| (v - pad).max(0.0);
    if let Size::Fixed(v) = size {
        return (Some(inner(v)), None);
    }
    let px = |b: Option<Size>| match b {
        Some(Size::Fixed(v)) => Some(inner(v)),
        _ => None,
    };
    let [lo, hi] = n.el.bounds[axis];
    let (min, max) = size.bounds();
    (
        px(hi).or((max.is_finite()).then(|| inner(max))),
        px(lo).or((min > 0.0).then(|| inner(min))),
    )
}

/// Places a grid's children in its cells (CSS Grid §8.5) for `avail`,
/// the inner size per axis (`None`: not yet known).
pub(super) fn place_grid(nodes: &mut [Node], i: usize) {
    if nodes[i].table.is_some() {
        place_table(nodes, i);
        return;
    }
    let avail = [0, 1].map(|axis| definite_inner(&nodes[i], axis));
    place_at(nodes, i, [avail[0].0, avail[1].0], [avail[0].1, avail[1].1]);
}

fn place_at(nodes: &mut [Node], i: usize, avail: [Option<f32>; 2], min: [Option<f32>; 2]) {
    let n = &nodes[i];
    let spec = &n.props.grid;
    let gap = [
        spec.column_gap.or(n.el.gaps[0]).unwrap_or(n.el.gap),
        spec.row_gap.or(n.el.gaps[1]).unwrap_or(n.el.gap),
    ];
    let kinds = [kind(nodes, i, 0), kind(nodes, i, 1)];
    let reps = [0, 1].map(|axis| repetitions(spec, axis, avail[axis], min[axis], gap[axis]));
    let mut explicit: [Vec<Track>; 2] = Default::default();
    let mut names: [Vec<Vec<String>>; 2] = Default::default();
    let named_areas = areas(spec);
    let area_size = [
        spec.areas
            .iter()
            .map(|l| l.split_whitespace().count())
            .max()
            .unwrap_or(0),
        spec.areas.len(),
    ];
    for axis in 0..2 {
        if kinds[axis] == Template::Subgrid {
            let count = subgrid_span(nodes, i, axis);
            explicit[axis] = vec![Track::Auto; count];
            let own = &spec.lines[axis].names;
            names[axis] = (0..=count)
                .map(|k| own.get(k).cloned().unwrap_or_default())
                .collect();
            continue;
        }
        if kinds[axis] == Template::Masonry {
            continue;
        }
        let (t, mut nm) = expand(spec, axis, reps[axis]);
        explicit[axis] = t;
        // Areas past the tracks add explicit tracks of the auto size.
        let count = explicit[axis].len().max(area_size[axis]);
        nm.resize(count + 1, Vec::new());
        // An area's edges are lines called `<area>-start` and `-end` (§7.3.2).
        for a in &named_areas {
            let (s, e) = if axis == 0 { (a.1, a.3) } else { (a.2, a.4) };
            nm[s].push(format!("{}-start", a.0));
            nm[e].push(format!("{}-end", a.0));
        }
        names[axis] = nm;
    }
    let kids = n.children.clone();
    // Each item's resolved lines per axis.
    let mut pos: Vec<[Result<(i32, i32), usize>; 2]> = Vec::with_capacity(kids.len());
    for &c in &kids {
        let lines = item_lines(&nodes[c].props.place);
        pos.push([0, 1].map(|axis| {
            if kinds[axis] == Template::Masonry {
                return Err(
                    match resolve(&[Vec::new()], &lines[axis][0], &lines[axis][1]) {
                        Ok((s, e)) => (e - s) as usize,
                        Err(n) => n,
                    },
                );
            }
            let r = resolve(&names[axis], &lines[axis][0], &lines[axis][1]);
            // A subgrid has no implicit tracks: its items stay inside.
            if kinds[axis] == Template::Subgrid {
                let count = explicit[axis].len() as i32;
                return match r {
                    Ok((s, e)) => {
                        let s = s.clamp(0, count - 1);
                        Ok((s, e.clamp(s + 1, count)))
                    }
                    Err(n) => Err(n.min(count as usize)),
                };
            }
            r
        }));
    }
    let lead = [0, 1].map(|axis| {
        pos.iter()
            .filter_map(|p| p[axis].as_ref().ok().map(|&(s, _)| s))
            .min()
            .map_or(0, |s| (-s).max(0) as usize)
    });
    let count = [0, 1].map(|axis| {
        names[axis]
            .len()
            .saturating_sub(1)
            .max(explicit[axis].len())
    });
    let mut ends = [0, 1].map(|axis| lead[axis] + count[axis]);
    let mut cells = vec![(0usize, 0usize, 1usize, 1usize); kids.len()];
    // Occupied cells, [row][col].
    let mut occ: Vec<Vec<bool>> = Vec::new();
    let mark = |occ: &mut Vec<Vec<bool>>, c: usize, r: usize, cs: usize, rs: usize| {
        for rr in r..r + rs {
            if occ.len() <= rr {
                occ.resize(rr + 1, Vec::new());
            }
            let row = &mut occ[rr];
            if row.len() < c + cs {
                row.resize(c + cs, false);
            }
            for cell in &mut row[c..c + cs] {
                *cell = true;
            }
        }
    };
    let free = |occ: &Vec<Vec<bool>>, c: usize, r: usize, cs: usize, rs: usize| {
        (r..r + rs).all(|rr| {
            (c..c + cs).all(|cc| {
                !occ.get(rr)
                    .and_then(|row| row.get(cc))
                    .copied()
                    .unwrap_or(false)
            })
        })
    };
    let shift = |axis: usize, (s, e): (i32, i32)| {
        let s = (s + lead[axis] as i32).max(0) as usize;
        (s, ((e + lead[axis] as i32).max(0) as usize).max(s + 1) - s)
    };
    let masonry = kinds.iter().position(|k| *k == Template::Masonry);
    // The axis items advance along (columns in a row flow).
    let a = if let Some(m) = masonry {
        1 - m
    } else if spec.flow == GridFlow::Row {
        0
    } else {
        1
    };
    let b = 1 - a;
    let cell = |pa: (usize, usize), pb: (usize, usize)| {
        if a == 0 {
            (pa.0, pb.0, pa.1, pb.1)
        } else {
            (pb.0, pa.0, pb.1, pa.1)
        }
    };
    let occupy =
        |occ: &mut Vec<Vec<bool>>, c: (usize, usize, usize, usize)| mark(occ, c.0, c.1, c.2, c.3);
    let fits =
        |occ: &Vec<Vec<bool>>, c: (usize, usize, usize, usize)| free(occ, c.0, c.1, c.2, c.3);
    if let Some(m) = masonry {
        // Masonry: the grid axis only; packing along the other waits for
        // the items' sizes ([`masonry_pack`]). Auto items take tracks in
        // turn until then.
        let tracks = ends[a].max(1);
        let mut at = 0;
        for (k, p) in pos.iter().enumerate() {
            let (s, span) = match &p[a] {
                Ok(l) => shift(a, *l),
                Err(span) => {
                    let span = (*span).min(tracks);
                    if at + span > tracks {
                        at = 0;
                    }
                    let s = at;
                    at = (at + span) % tracks;
                    (s, span)
                }
            };
            ends[a] = ends[a].max(s + span);
            cells[k] = cell((s, span), (0, 1));
        }
        ends[m] = 1;
    } else {
        // 1. Items placed on both axes.
        for (k, p) in pos.iter().enumerate() {
            if let [Ok(x), Ok(y)] = p {
                let (x, y) = (shift(0, *x), shift(1, *y));
                cells[k] = (x.0, y.0, x.1, y.1);
                occupy(&mut occ, cells[k]);
                ends[0] = ends[0].max(x.0 + x.1);
                ends[1] = ends[1].max(y.0 + y.1);
            }
        }
        // 2. Items locked to a track of `b`, auto along `a`.
        let mut cursors: Vec<usize> = Vec::new();
        for (k, p) in pos.iter().enumerate() {
            let (Err(span), Ok(lb)) = (&p[a], &p[b]) else {
                continue;
            };
            let pb = shift(b, *lb);
            if cursors.len() < pb.0 + 1 {
                cursors.resize(pb.0 + 1, 0);
            }
            let mut at = if spec.dense { 0 } else { cursors[pb.0] };
            while !fits(&occ, cell((at, *span), pb)) {
                at += 1;
            }
            cells[k] = cell((at, *span), pb);
            occupy(&mut occ, cells[k]);
            cursors[pb.0] = at + span;
            ends[a] = ends[a].max(at + span);
            ends[b] = ends[b].max(pb.0 + pb.1);
        }
        // 3. The tracks along `a`: enough for every auto item's span.
        for p in &pos {
            match &p[a] {
                Err(span) if p[b].is_err() => ends[a] = ends[a].max(*span),
                Ok(l) if p[b].is_err() => {
                    let l = shift(a, *l);
                    ends[a] = ends[a].max(l.0 + l.1);
                }
                _ => {}
            }
        }
        let major = ends[a].max(1);
        // 4. The rest, along `a` from the cursor (`dense`: from the start).
        let (mut ca, mut cb) = (0usize, 0usize);
        for (k, p) in pos.iter().enumerate() {
            if p[b].is_ok() {
                continue;
            }
            let Err(span_b) = p[b] else { continue };
            if spec.dense {
                (ca, cb) = (0, 0);
            }
            match &p[a] {
                Ok(l) => {
                    let la = shift(a, *l);
                    if !spec.dense && la.0 < ca {
                        cb += 1;
                    }
                    ca = la.0;
                    while !fits(&occ, cell(la, (cb, span_b))) {
                        cb += 1;
                    }
                    cells[k] = cell(la, (cb, span_b));
                }
                Err(span) => {
                    let span = (*span).min(major);
                    loop {
                        if ca + span > major {
                            ca = 0;
                            cb += 1;
                            continue;
                        }
                        if fits(&occ, cell((ca, span), (cb, span_b))) {
                            break;
                        }
                        ca += 1;
                    }
                    cells[k] = cell((ca, span), (cb, span_b));
                    ca += span;
                }
            }
            occupy(&mut occ, cells[k]);
            let c = cells[k];
            ends[0] = ends[0].max(c.0 + c.2);
            ends[1] = ends[1].max(c.1 + c.3);
        }
        // A subgrid's track count is its span in its parent.
        for axis in 0..2 {
            if kinds[axis] == Template::Subgrid {
                ends[axis] = explicit[axis].len().max(1);
            }
        }
    }
    // `auto-fit` repeated tracks no item takes collapse (§7.2.3.2).
    let mut collapsed = [vec![false; ends[0]], vec![false; ends[1]]];
    for axis in 0..2 {
        let Some(rep) = spec.lines[axis].repeat.as_ref().filter(|r| r.fit) else {
            continue;
        };
        if matches!(kinds[axis], Template::Subgrid | Template::Masonry) {
            continue;
        }
        let from = lead[axis] + rep.at;
        let to = (from + rep.tracks.len() * reps[axis].max(1)).min(ends[axis]);
        for (t, slot) in collapsed[axis].iter_mut().enumerate().take(to).skip(from) {
            *slot = !cells.iter().any(|c| {
                let (s, n) = if axis == 0 { (c.0, c.2) } else { (c.1, c.3) };
                (s..s + n).contains(&t)
            });
        }
    }
    let layout = GridLayout {
        cells,
        ncols: ends[0].max(1),
        nrows: ends[1].max(1),
        col_gap: gap[0],
        row_gap: gap[1],
        lead,
        explicit,
        collapsed,
        reps,
        ..GridLayout::default()
    };
    nodes[i].grid = Some(layout);
}

/// The size of implicit or explicit track `k` along `axis`.
fn track_def(spec: &GridSpec, g: &GridLayout, axis: usize, k: usize) -> Track {
    let auto = if axis == 0 {
        &spec.auto_columns
    } else {
        &spec.auto_rows
    };
    let from_auto = |j: isize| -> Track {
        if auto.is_empty() {
            return spec.auto_track;
        }
        let n = auto.len() as isize;
        auto[j.rem_euclid(n) as usize]
    };
    let lead = g.lead[axis];
    let explicit = &g.explicit[axis];
    if k < lead {
        // Implicit tracks before the grid take the list backwards.
        return from_auto(k as isize - lead as isize);
    }
    match explicit.get(k - lead) {
        Some(t) => *t,
        None => from_auto((k - lead - explicit.len()) as isize),
    }
}

/// One item's part in a track size: its first track, span and its
/// minimum, min-content and max-content contributions (CSS Grid §6.6).
#[derive(Clone, Copy, Debug)]
struct Item {
    s: usize,
    span: usize,
    least: f32,
    min_c: f32,
    max_c: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Definite(f32),
    MinContent,
    MaxContent,
}

/// The track sizing algorithm (CSS Grid §12.3–§12.8): each track's size.
fn size_tracks(
    defs: &[Track],
    collapsed: &[bool],
    items: &[Item],
    mode: Mode,
    gap: f32,
    stretch: bool,
) -> Vec<f32> {
    let n = defs.len();
    let basis = match mode {
        Mode::Definite(a) => Some(a),
        _ => None,
    };
    let is_collapsed = |k: usize| collapsed.get(k).copied().unwrap_or(false);
    // A percentage of an indefinite size is `auto` (§7.2.1).
    let fns: Vec<(Breadth, Breadth, Option<f32>)> = defs
        .iter()
        .map(|t| {
            let (lo, hi, fit) = t.breadths();
            let def = |b: Breadth| match b {
                Breadth::Percent(_) if basis.is_none() => Breadth::Auto,
                b => b,
            };
            (def(lo), def(hi), fit.and_then(|f| fixed(f, basis)))
        })
        .collect();
    let flex = |k: usize| match fns[k].1 {
        Breadth::Fr(f) => Some(f),
        _ => None,
    };
    let intrinsic_min = |k: usize| {
        matches!(
            fns[k].0,
            Breadth::Auto | Breadth::MinContent | Breadth::MaxContent
        )
    };
    let intrinsic_max = |k: usize| {
        matches!(
            fns[k].1,
            Breadth::Auto | Breadth::MinContent | Breadth::MaxContent
        )
    };
    let max_content_max = |k: usize| matches!(fns[k].1, Breadth::Auto | Breadth::MaxContent);
    let mut base: Vec<f32> = (0..n)
        .map(|k| {
            if is_collapsed(k) {
                0.0
            } else {
                fixed(fns[k].0, basis).unwrap_or(0.0)
            }
        })
        .collect();
    let mut limit: Vec<f32> = (0..n)
        .map(|k| {
            if is_collapsed(k) {
                0.0
            } else {
                fixed(fns[k].1, basis).map_or(f32::INFINITY, |v| v.max(base[k]))
            }
        })
        .collect();
    let gaps_in = |s: usize, e: usize| {
        gap * (s..e)
            .filter(|&k| !is_collapsed(k))
            .count()
            .saturating_sub(1) as f32
    };
    let crosses_flex = |it: &Item| (it.s..(it.s + it.span).min(n)).any(|k| flex(k).is_some());
    let under_min = mode == Mode::MinContent;
    // Under a min-content constraint an `auto` minimum reads the item's
    // min-content contribution (§12.5 step 2).
    let least = |it: &Item| if under_min { it.min_c } else { it.least };
    // 12.5 step 2: items of one track.
    let mut acc = vec![None::<f32>; n];
    for it in items
        .iter()
        .filter(|it| it.span == 1 && it.s < n && !crosses_flex(it))
    {
        let k = it.s;
        if is_collapsed(k) {
            continue;
        }
        match fns[k].0 {
            Breadth::Auto => base[k] = base[k].max(least(it)),
            Breadth::MinContent => base[k] = base[k].max(it.min_c),
            Breadth::MaxContent => base[k] = base[k].max(it.max_c),
            _ => {}
        }
        let c = match fns[k].1 {
            Breadth::MinContent => Some(it.min_c),
            Breadth::MaxContent | Breadth::Auto => Some(it.max_c),
            _ => None,
        };
        let c = match (c, fns[k].2) {
            (Some(c), Some(l)) => Some(c.min(l.max(it.min_c))),
            (c, _) => c,
        };
        if let Some(c) = c {
            acc[k] = Some(acc[k].map_or(c, |a: f32| a.max(c)));
        }
    }
    for k in 0..n {
        if let Some(a) = acc[k]
            && limit[k].is_infinite()
        {
            limit[k] = a;
        }
        if limit[k] < base[k] {
            limit[k] = base[k];
        }
    }
    // 12.5 step 3: items spanning several tracks, fewest first.
    let mut spans: Vec<usize> = items
        .iter()
        .filter(|it| it.span > 1 && !crosses_flex(it))
        .map(|it| it.span)
        .collect();
    spans.sort_unstable();
    spans.dedup();
    // §12.5.1: distributes the room an item needs over the affected tracks
    // of its span, each up to its limit, the rest over `beyond`.
    #[allow(clippy::too_many_arguments)]
    fn distribute(
        sizes: &mut [f32],
        caps: &[f32],
        items: &[(usize, usize, f32)],
        affected: &dyn Fn(usize) -> bool,
        beyond: &dyn Fn(usize) -> bool,
        gaps: &dyn Fn(usize, usize) -> f32,
    ) {
        let mut planned = vec![0.0f32; sizes.len()];
        for &(s, span, need) in items {
            let e = (s + span).min(sizes.len());
            let tracks: Vec<usize> = (s..e).filter(|&k| affected(k)).collect();
            if tracks.is_empty() {
                continue;
            }
            let have: f32 = (s..e).map(|k| sizes[k]).sum::<f32>() + gaps(s, e);
            let mut room = need - have;
            if room <= 0.0 {
                continue;
            }
            let mut inc = vec![0.0f32; sizes.len()];
            // Equally, each up to its cap.
            let mut open: Vec<usize> = tracks.clone();
            while room > 1e-4 && !open.is_empty() {
                let each = room / open.len() as f32;
                let mut next = Vec::new();
                for &k in &open {
                    let headroom = caps[k] - sizes[k] - inc[k];
                    let d = each.min(headroom.max(0.0));
                    inc[k] += d;
                    room -= d;
                    if headroom - d > 1e-4 {
                        next.push(k);
                    }
                }
                if next.len() == open.len() {
                    break;
                }
                open = next;
            }
            if room > 1e-4 {
                let over: Vec<usize> = tracks.iter().copied().filter(|&k| beyond(k)).collect();
                let over = if over.is_empty() {
                    tracks.clone()
                } else {
                    over
                };
                let each = room / over.len() as f32;
                for k in over {
                    inc[k] += each;
                }
            }
            for k in s..e {
                planned[k] = planned[k].max(inc[k]);
            }
        }
        for (s, p) in sizes.iter_mut().zip(planned) {
            *s += p;
        }
    }
    for span in spans {
        let group: Vec<&Item> = items
            .iter()
            .filter(|it| it.span == span && it.s < n && !crosses_flex(it))
            .collect();
        let pick = |f: &dyn Fn(&Item) -> f32| -> Vec<(usize, usize, f32)> {
            group.iter().map(|it| (it.s, it.span, f(it))).collect()
        };
        let live = |k: usize| !is_collapsed(k);
        // Base sizes: intrinsic minimums, then content ones, then max-content.
        let caps = limit.clone();
        distribute(
            &mut base,
            &caps,
            &pick(&|it| least(it)),
            &|k| live(k) && intrinsic_min(k),
            &|k| intrinsic_max(k),
            &gaps_in,
        );
        distribute(
            &mut base,
            &caps,
            &pick(&|it| it.min_c),
            &|k| live(k) && matches!(fns[k].0, Breadth::MinContent | Breadth::MaxContent),
            &|k| intrinsic_max(k),
            &gaps_in,
        );
        distribute(
            &mut base,
            &caps,
            &pick(&|it| it.max_c),
            &|k| live(k) && fns[k].0 == Breadth::MaxContent,
            &|k| max_content_max(k),
            &gaps_in,
        );
        for k in 0..n {
            if limit[k] < base[k] {
                limit[k] = base[k];
            }
        }
        // Growth limits: an infinite one counts as its base size.
        let mut lim: Vec<f32> = (0..n)
            .map(|k| {
                if limit[k].is_infinite() {
                    base[k]
                } else {
                    limit[k]
                }
            })
            .collect();
        let fit_cap: Vec<f32> = (0..n).map(|k| fns[k].2.unwrap_or(f32::INFINITY)).collect();
        let before = lim.clone();
        distribute(
            &mut lim,
            &fit_cap,
            &pick(&|it| it.min_c),
            &|k| live(k) && intrinsic_max(k),
            &|k| fns[k].2.is_none(),
            &gaps_in,
        );
        distribute(
            &mut lim,
            &fit_cap,
            &pick(&|it| it.max_c),
            &|k| live(k) && (max_content_max(k) || fns[k].2.is_some()),
            &|k| fns[k].2.is_none(),
            &gaps_in,
        );
        for k in 0..n {
            if lim[k] != before[k] || !limit[k].is_infinite() {
                limit[k] = lim[k].max(base[k]);
            }
        }
    }
    // 12.5 step 4: items crossing flexible tracks grow their base sizes
    // in proportion to their flex factors.
    {
        let mut planned = vec![0.0f32; n];
        for it in items.iter().filter(|it| it.s < n && crosses_flex(it)) {
            let e = (it.s + it.span).min(n);
            let have: f32 = (it.s..e).map(|k| base[k]).sum::<f32>() + gaps_in(it.s, e);
            let need = it.least;
            let room = need - have;
            if room <= 0.0 {
                continue;
            }
            let tracks: Vec<usize> = (it.s..e)
                .filter(|&k| flex(k).is_some() && intrinsic_min(k) && !is_collapsed(k))
                .collect();
            let sum: f32 = tracks.iter().filter_map(|&k| flex(k)).sum();
            for &k in &tracks {
                let share = if sum > 0.0 {
                    flex(k).unwrap() / sum
                } else {
                    1.0 / tracks.len() as f32
                };
                planned[k] = planned[k].max(room * share);
            }
        }
        for k in 0..n {
            base[k] += planned[k];
        }
    }
    for k in 0..n {
        if limit[k].is_infinite() {
            limit[k] = base[k];
        }
        limit[k] = limit[k].max(base[k]);
    }
    let gaps_all = gaps_in(0, n);
    // 12.6: maximize the tracks.
    match mode {
        Mode::Definite(a) => {
            let mut room = a - base.iter().sum::<f32>() - gaps_all;
            let mut open: Vec<usize> = (0..n).filter(|&k| limit[k] > base[k]).collect();
            while room > 1e-4 && !open.is_empty() {
                let each = room / open.len() as f32;
                let mut next = Vec::new();
                for &k in &open {
                    let d = each.min(limit[k] - base[k]);
                    base[k] += d;
                    room -= d;
                    if limit[k] - base[k] > 1e-4 {
                        next.push(k);
                    }
                }
                open = next;
            }
        }
        Mode::MaxContent => {
            for k in 0..n {
                if flex(k).is_none() {
                    base[k] = limit[k];
                }
            }
        }
        Mode::MinContent => {}
    }
    // 12.7: flexible tracks.
    let flexes: Vec<usize> = (0..n)
        .filter(|&k| flex(k).is_some() && !is_collapsed(k))
        .collect();
    if !flexes.is_empty() {
        let fr = match mode {
            Mode::Definite(a) => {
                // §12.7.1: the fr size, leaving out flexible tracks whose
                // base is larger than their share.
                let mut fixed_out: Vec<bool> = vec![false; n];
                loop {
                    let leftover = a
                        - gaps_all
                        - (0..n)
                            .filter(|&k| flex(k).is_none() || fixed_out[k])
                            .map(|k| base[k])
                            .sum::<f32>();
                    let sum: f32 = flexes
                        .iter()
                        .filter(|&&k| !fixed_out[k])
                        .filter_map(|&k| flex(k))
                        .sum();
                    let fr = leftover.max(0.0) / sum.max(1.0);
                    let over: Vec<usize> = flexes
                        .iter()
                        .copied()
                        .filter(|&k| !fixed_out[k] && base[k] > fr * flex(k).unwrap())
                        .collect();
                    if over.is_empty() {
                        break fr;
                    }
                    for k in over {
                        fixed_out[k] = true;
                    }
                }
            }
            Mode::MaxContent => {
                let mut fr = flexes
                    .iter()
                    .map(|&k| {
                        let f = flex(k).unwrap();
                        if f > 1.0 { base[k] / f } else { base[k] }
                    })
                    .fold(0.0f32, f32::max);
                for it in items.iter().filter(|it| it.s < n && crosses_flex(it)) {
                    let e = (it.s + it.span).min(n);
                    let rest: f32 = (it.s..e)
                        .filter(|&k| flex(k).is_none())
                        .map(|k| base[k])
                        .sum();
                    let sum: f32 = (it.s..e).filter_map(flex).sum();
                    let need = it.max_c - rest - gaps_in(it.s, e);
                    fr = fr.max(need / sum.max(1.0));
                }
                fr
            }
            Mode::MinContent => 0.0,
        };
        for &k in &flexes {
            base[k] = base[k].max(fr * flex(k).unwrap());
        }
    }
    // 12.8: `auto` tracks share the room left.
    if let Mode::Definite(a) = mode
        && stretch
    {
        let room = a - base.iter().sum::<f32>() - gaps_all;
        let autos: Vec<usize> = (0..n)
            .filter(|&k| fns[k].1 == Breadth::Auto && !is_collapsed(k))
            .collect();
        if room > 0.0 && !autos.is_empty() {
            let each = room / autos.len() as f32;
            for k in autos {
                base[k] += each;
            }
        }
    }
    base
}

/// Node `c`'s contributions along `axis`, margins included.
fn contribution(k: &Node, axis: usize) -> (f32, f32, f32) {
    let (pref, min, m, scrolls) = if axis == 0 {
        (
            k.pref_w,
            k.min_w,
            super::flex::side_margins(k),
            k.el.scroll_x,
        )
    } else {
        (k.pref_h, k.min_h, super::flex::cross_room(k), k.el.scroll)
    };
    let least = if scrolls { 0.0 } else { min };
    (least + m, min + m, pref.max(min) + m)
}

/// The items sizing node `i`'s tracks along `axis`: its children, a
/// subgrid's children in its tracks instead (§9), and a masonry grid's
/// auto items in every track they could take.
fn grid_items(nodes: &[Node], i: usize, axis: usize) -> Vec<Item> {
    let n = &nodes[i];
    let Some(g) = &n.grid else {
        return Vec::new();
    };
    let masonry = (0..2).any(|a| kind(nodes, i, a) == Template::Masonry);
    let tracks = if axis == 0 { g.ncols } else { g.nrows };
    let mut out = Vec::new();
    for (&c, &(col, row, cs, rs)) in n.children.iter().zip(&g.cells) {
        let (s, span) = if axis == 0 { (col, cs) } else { (row, rs) };
        if kind(nodes, c, axis) == Template::Subgrid && nodes[c].grid.is_some() {
            let pad = &nodes[c].el.padding;
            let (lo, hi) = if axis == 0 {
                (pad.left, pad.right)
            } else {
                (pad.top, pad.bottom)
            };
            let edge = if axis == 0 {
                [
                    nodes[c].el.margin[3].intrinsic(),
                    nodes[c].el.margin[1].intrinsic(),
                ]
            } else {
                [
                    nodes[c].el.margin[0].intrinsic(),
                    nodes[c].el.margin[2].intrinsic(),
                ]
            };
            for mut it in grid_items(nodes, c, axis) {
                let add = if it.s == 0 { lo + edge[0] } else { 0.0 }
                    + if it.s + it.span >= span {
                        hi + edge[1]
                    } else {
                        0.0
                    };
                it.s += s;
                it.least += add;
                it.min_c += add;
                it.max_c += add;
                out.push(it);
            }
            continue;
        }
        let (least, min_c, max_c) = contribution(&nodes[c], axis);
        let auto = masonry
            && item_lines(&nodes[c].props.place)[axis]
                .iter()
                .all(|l| matches!(l, GridLine::Auto | GridLine::Span(..)));
        if auto && kind(nodes, i, axis) != Template::Masonry {
            for start in 0..=tracks.saturating_sub(span) {
                out.push(Item {
                    s: start,
                    span,
                    least,
                    min_c,
                    max_c,
                });
            }
        } else {
            out.push(Item {
                s,
                span,
                least,
                min_c,
                max_c,
            });
        }
    }
    out
}

fn defs(nodes: &[Node], i: usize, axis: usize) -> Vec<Track> {
    let n = &nodes[i];
    let g = n.grid.as_ref().unwrap();
    let count = if axis == 0 { g.ncols } else { g.nrows };
    (0..count)
        .map(|k| track_def(&n.props.grid, g, axis, k))
        .collect()
}

/// Packs a masonry grid's items along `m`, each where its tracks are
/// shortest (`size`: an item's outer size along `m`): each item's track
/// and start, and the extent.
fn masonry_pack(
    nodes: &[Node],
    i: usize,
    m: usize,
    size: &dyn Fn(&Node) -> f32,
) -> (Vec<(usize, f32)>, f32) {
    let n = &nodes[i];
    let g = n.grid.as_ref().unwrap();
    let a = 1 - m;
    let tracks = if a == 0 { g.ncols } else { g.nrows }.max(1);
    let gap = g.gap(m);
    let mut running = vec![0.0f32; tracks];
    let mut out = Vec::with_capacity(n.children.len());
    for (&c, &(col, row, cs, rs)) in n.children.iter().zip(&g.cells) {
        let (s, span) = if a == 0 { (col, cs) } else { (row, rs) };
        let span = span.min(tracks);
        let auto = item_lines(&nodes[c].props.place)[a]
            .iter()
            .all(|l| matches!(l, GridLine::Auto | GridLine::Span(..)));
        let start = if auto {
            (0..=tracks - span)
                .min_by(|&x, &y| {
                    let top = |s: usize| running[s..s + span].iter().copied().fold(0.0, f32::max);
                    top(x)
                        .partial_cmp(&top(y))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap_or(0)
        } else {
            s.min(tracks - span)
        };
        let top = running[start..start + span]
            .iter()
            .copied()
            .fold(0.0, f32::max);
        let end = top + size(&nodes[c]);
        for r in &mut running[start..start + span] {
            *r = end + gap;
        }
        out.push((start, top));
    }
    let extent = running.iter().copied().fold(0.0f32, f32::max);
    (
        out,
        if n.children.is_empty() {
            0.0
        } else {
            (extent - gap).max(0.0)
        },
    )
}

/// A grid's preferred and least size along one axis.
pub(super) fn grid_intrinsic(nodes: &[Node], i: usize, horizontal: bool) -> (f32, f32) {
    let n = &nodes[i];
    let Some(g) = &n.grid else {
        return (0.0, 0.0);
    };
    let axis = usize::from(!horizontal);
    if kind(nodes, i, axis) == Template::Masonry {
        let outer = |k: &Node, pref: bool| {
            let (_, min, max) = contribution(k, axis);
            if pref { max } else { min }
        };
        let (_, pref) = masonry_pack(nodes, i, axis, &|k| outer(k, true));
        let (_, min) = masonry_pack(nodes, i, axis, &|k| outer(k, false));
        return (pref, min.min(pref));
    }
    let gap = g.gap(axis);
    let d = defs(nodes, i, axis);
    let items = grid_items(nodes, i, axis);
    let collapsed = &g.collapsed[axis];
    let gaps = gap
        * (0..d.len())
            .filter(|&k| !collapsed.get(k).copied().unwrap_or(false))
            .count()
            .saturating_sub(1) as f32;
    let pref: f32 = size_tracks(&d, collapsed, &items, Mode::MaxContent, gap, false)
        .iter()
        .sum::<f32>()
        + gaps;
    let min: f32 = size_tracks(&d, collapsed, &items, Mode::MinContent, gap, false)
        .iter()
        .sum::<f32>()
        + gaps;
    (pref, min.min(pref))
}

/// Track starts along an axis, the tracks aligned in `inner` by `align`
/// (CSS Box Alignment §5.1; `None`: `normal`, at the start).
fn track_starts(
    sizes: &[f32],
    collapsed: &[bool],
    gap: f32,
    inner: f32,
    align: Option<Align>,
) -> Vec<f32> {
    let live: Vec<usize> = (0..sizes.len())
        .filter(|&k| !collapsed.get(k).copied().unwrap_or(false))
        .collect();
    let used: f32 = sizes.iter().sum::<f32>() + gap * live.len().saturating_sub(1) as f32;
    let free = inner - used;
    let k = live.len().max(1) as f32;
    let (lead, between) = match align.unwrap_or(Align::Start) {
        Align::Start | Align::Stretch => (0.0, 0.0),
        Align::Center => (free / 2.0, 0.0),
        Align::End => (free, 0.0),
        // A distribution that cannot spread falls back (§4.3).
        Align::SpaceBetween if free > 0.0 && live.len() > 1 => (0.0, free / (k - 1.0)),
        Align::SpaceBetween => (0.0, 0.0),
        Align::SpaceAround if free > 0.0 => (free / k / 2.0, free / k),
        Align::SpaceEvenly if free > 0.0 => (free / (k + 1.0), free / (k + 1.0)),
        Align::SpaceAround | Align::SpaceEvenly => (free / 2.0, 0.0),
    };
    let mut at = Vec::with_capacity(sizes.len());
    let mut v = lead;
    for (t, s) in sizes.iter().enumerate() {
        at.push(v);
        if !collapsed.get(t).copied().unwrap_or(false) {
            v += s + gap + between;
        }
    }
    at
}

/// The sizes of a subgrid's tracks along `axis`: those of the tracks its
/// area spans in its parent, its own padding and margins taken from the
/// edge ones.
fn subgrid_tracks(
    nodes: &[Node],
    i: usize,
    axis: usize,
    count: usize,
) -> Option<(Vec<f32>, Vec<f32>)> {
    let n = &nodes[i];
    let p = n.up;
    let pg = nodes[p].grid.as_ref()?;
    let at = nodes[p].children.iter().position(|&c| c == i)?;
    let &(col, row, cs, rs) = pg.cells.get(at)?;
    let (s, span) = if axis == 0 { (col, cs) } else { (row, rs) };
    let sizes = pg.sizes(axis);
    let starts = &pg.at[axis];
    if sizes.len() < s + span || starts.len() < s + span {
        return None;
    }
    let (pad_lo, m_lo) = if axis == 0 {
        (n.el.padding.left, n.margin_x[0])
    } else {
        (n.el.padding.top, n.margin_sets[0].0 + n.margin_sets[0].1)
    };
    let origin = starts[s] + m_lo + pad_lo;
    let mut sz: Vec<f32> = sizes[s..s + span].to_vec();
    let mut st: Vec<f32> = starts[s..s + span].iter().map(|v| v - origin).collect();
    let inner = if axis == 0 {
        n.w - n.el.padding.horizontal()
    } else {
        n.h - n.el.padding.vertical()
    };
    // The edge tracks lose what its padding and margins take.
    if let Some(f) = sz.first_mut() {
        *f = (*f + st[0]).max(0.0);
        st[0] = 0.0;
    }
    if let (Some(l), Some(ls)) = (sz.last_mut(), st.last()) {
        *l = (inner - ls).max(0.0).min(*l);
    }
    sz.resize(count, 0.0);
    st.resize(count, inner.max(0.0));
    Some((sz, st))
}

/// Resolves one axis's tracks of grid `i` at `inner`, its track starts and
/// its children's sizes along it.
fn resolve_axis(nodes: &mut [Node], i: usize, axis: usize, inner: f32) {
    // An auto repetition count that depended on this size places again.
    let changed = {
        let n = &nodes[i];
        let g = n.grid.as_ref().unwrap();
        let gap = g.gap(axis);
        n.props.grid.lines[axis].repeat.is_some()
            && kind(nodes, i, axis) == Template::Tracks
            && repetitions(&n.props.grid, axis, Some(inner), None, gap) != g.reps[axis]
    };
    if changed {
        let other = definite_inner(&nodes[i], 1 - axis).0;
        let mut avail = [None, None];
        avail[axis] = Some(inner);
        avail[1 - axis] = other;
        let keep = nodes[i].grid.clone().unwrap();
        place_at(nodes, i, avail, [None, None]);
        // The other axis keeps the sizes it already resolved.
        if let Some(g) = nodes[i].grid.as_mut()
            && axis == 1
        {
            g.cols = keep.cols;
            g.at[0] = keep.at[0].clone();
        }
    }
    let kind_here = kind(nodes, i, axis);
    let n = &nodes[i];
    let g = n.grid.as_ref().unwrap();
    let gap = g.gap(axis);
    let collapsed = g.collapsed[axis].clone();
    let count = if axis == 0 { g.ncols } else { g.nrows };
    let spec = &n.props.grid;
    let (sizes, starts) = match kind_here {
        Template::Subgrid => subgrid_tracks(nodes, i, axis, count).unwrap_or_else(|| {
            let s = vec![inner / count.max(1) as f32; count];
            let at = track_starts(&s, &collapsed, gap, inner, None);
            (s, at)
        }),
        Template::Masonry => (vec![inner], vec![0.0]),
        Template::Tracks => {
            let d = defs(nodes, i, axis);
            let items = grid_items(nodes, i, axis);
            let stretch = spec.content[axis].is_none();
            let s = size_tracks(&d, &collapsed, &items, Mode::Definite(inner), gap, stretch);
            let at = track_starts(&s, &collapsed, gap, inner, spec.content[axis]);
            (s, at)
        }
    };
    {
        let g = nodes[i].grid.as_mut().unwrap();
        if axis == 0 {
            g.cols = sizes;
        } else {
            g.rows = sizes;
        }
        g.at[axis] = starts;
    }
    // Each child's size in its area (CSS Grid §11.3, Box Alignment §6).
    let kids = nodes[i].children.clone();
    let g = nodes[i].grid.clone().unwrap();
    let justify_items = nodes[i].el.justify_items;
    let align_items = nodes[i].props.grid.align_items;
    for (k, &c) in kids.iter().enumerate() {
        let Some(&(col, row, cs, rs)) = g.cells.get(k) else {
            continue;
        };
        let (s, span) = if axis == 0 { (col, cs) } else { (row, rs) };
        let area = if kind_here == Template::Masonry {
            inner
        } else {
            g.span(axis, s, span).1
        };
        let kn = &nodes[c];
        let alignment = if axis == 0 {
            kn.el.justify_self.or(justify_items)
        } else {
            kn.el.align_self.or(align_items)
        };
        let stretch = matches!(alignment, None | Some(Cross::Stretch));
        if axis == 0 {
            // Its margins inset its area (a percentage reads the area's
            // width); `auto` ones take what its width leaves (§11.2).
            let [l, r] = [kn.el.margin[3], kn.el.margin[1]];
            let fill = (area - l.of(area) - r.of(area)).max(0.0);
            let w = match kn.el.width {
                _ if kind_here == Template::Masonry => kn.pref_w.min(fill).max(kn.min_w),
                Size::Fit { .. } if l.auto || r.auto || !stretch => {
                    kn.pref_w.min(fill).max(kn.min_w)
                }
                Size::Fit { .. } | Size::Grow { .. } => {
                    let (lo, hi) = kn.el.width.bounds();
                    fill.clamp(lo, hi)
                }
                size => size
                    .definite(kn.pref_w, kn.min_w, Some(area))
                    .unwrap_or(fill),
            };
            let w = super::flex::clamp_to(kn, 0, Some(area), w);
            let used =
                if l.auto { 0.0 } else { l.of(area) } + if r.auto { 0.0 } else { r.of(area) };
            let free = (area - w - used).max(0.0);
            let sides = match (l.auto, r.auto) {
                (true, true) => [free / 2.0, free / 2.0],
                (true, false) => [free, r.of(area)],
                (false, true) => [l.of(area), free],
                (false, false) => [l.of(area), r.of(area)],
            };
            nodes[c].w = w;
            nodes[c].margin_x = sides;
        } else {
            let around = super::flex::cross_room(kn);
            let fill = (area - around).max(0.0);
            let auto_margin = kn.el.margin[0].auto || kn.el.margin[2].auto;
            let h = match kn.el.height {
                _ if kind_here == Template::Masonry => kn.pref_h,
                Size::Fit { .. } if auto_margin || !stretch => {
                    kn.pref_h.min(fill).max(kn.min_h.min(fill))
                }
                Size::Fit { .. } | Size::Grow { .. } => {
                    let (lo, hi) = kn.el.height.bounds();
                    fill.clamp(lo, hi)
                }
                size => size
                    .definite(kn.pref_h, kn.min_h, Some(area))
                    .unwrap_or(fill),
            };
            nodes[c].h = super::flex::clamp_to(kn, 1, Some(area), h);
        }
        // A subgrid child sizes its tracks again from these.
        if kind(nodes, c, axis) == Template::Subgrid {
            nodes[c].layout_size[axis] = f32::NAN;
        }
    }
    // Masonry: pack along this axis now its items have their sizes.
    if kind_here == Template::Masonry {
        let outer = |k: &Node| {
            if axis == 0 {
                k.w + k.margin_x[0] + k.margin_x[1]
            } else {
                k.h + super::flex::cross_room(k)
            }
        };
        let (packed, _) = masonry_pack(nodes, i, axis, &outer);
        let g = nodes[i].grid.as_mut().unwrap();
        g.masonry = packed.iter().map(|p| p.1).collect();
        for (cell, (t, _)) in g.cells.iter_mut().zip(packed) {
            if axis == 0 {
                cell.1 = t;
            } else {
                cell.0 = t;
            }
        }
    }
}

/// Resolves a grid's columns at its inner width and sizes its children.
pub(super) fn grid_columns(nodes: &mut [Node], i: usize, inner: f32) {
    if nodes[i].grid.is_some() {
        resolve_axis(nodes, i, 0, inner);
    }
}

/// Resolves a grid's rows at its inner height and sizes its children.
pub(super) fn grid_rows(nodes: &mut [Node], i: usize, inner: f32) {
    if nodes[i].grid.is_some() {
        resolve_axis(nodes, i, 1, inner);
    }
}

/// Each child's area in grid `i`'s content box, and how it aligns there:
/// (x, y, width, height, justify, align); `None` past the cells.
pub(super) fn areas_of(nodes: &[Node], i: usize) -> Vec<Option<(Rect, Cross, Cross)>> {
    let n = &nodes[i];
    let Some(g) = &n.grid else {
        return Vec::new();
    };
    let masonry = [kind(nodes, i, 0), kind(nodes, i, 1)].map(|k| k == Template::Masonry);
    n.children
        .iter()
        .enumerate()
        .map(|(k, &c)| {
            let &(col, row, cs, rs) = g.cells.get(k)?;
            let kn = &nodes[c];
            let (x0, cw) = if masonry[0] {
                (
                    g.masonry.get(k).copied().unwrap_or(0.0),
                    kn.w + kn.margin_x[0] + kn.margin_x[1],
                )
            } else {
                g.span(0, col, cs)
            };
            let (y0, rh) = if masonry[1] {
                (
                    g.masonry.get(k).copied().unwrap_or(0.0),
                    kn.h + super::flex::cross_room(kn),
                )
            } else {
                g.span(1, row, rs)
            };
            let justify = kn
                .el
                .justify_self
                .or(n.el.justify_items)
                .unwrap_or(Cross::Start);
            let align = kn
                .el
                .align_self
                .or(n.props.grid.align_items)
                .unwrap_or(Cross::Start);
            Some((Rect::new(x0, y0, cw, rh), justify, align))
        })
        .collect()
}

/// A CSS table's grid (CSS 2 §17.5, HTML's table processing model): each
/// cell's slots, and the slots of the row, row group and column boxes that
/// paint behind them.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Table {
    /// Per child in the flow (captions and cells): column, row, column
    /// span, row span.
    pub cells: Vec<(usize, usize, usize, usize)>,
    pub ncols: usize,
    pub nrows: usize,
    /// A column's least width from its `table-column` box's px width.
    pub widths: Vec<f32>,
    /// The row, group and column boxes: node and its slots.
    pub parts: Vec<(usize, (usize, usize, usize, usize))>,
}

/// Node `i`'s shown children, each `display: contents` one replaced by its
/// own (CSS Display §2.5), in build order.
pub(crate) fn shown_children(nodes: &[Node], i: usize) -> Vec<usize> {
    let mut out = Vec::new();
    for &c in &nodes[i].built_children {
        let n = &nodes[c];
        if n.hidden || n.parent != i {
            continue;
        }
        if n.el.contents {
            out.extend(shown_children(nodes, c));
        } else {
            out.push(c);
        }
    }
    out
}

/// A table `i`'s layout from its shown children `kids`: the order its
/// boxes draw in (each row and group under its cells), its flow (captions
/// and cells), the boxes it places itself (rows, groups, columns) and
/// their slots. Children that are no table part sit in anonymous rows and
/// cells (CSS 2 §17.2.1); the first header group goes first, the first
/// footer group last; a `rowspan` of 0 runs to its group's end.
pub(crate) fn derive_table(nodes: &[Node], kids: &[usize]) -> (Vec<usize>, Vec<usize>, Table) {
    let part = |c: usize| nodes[c].el.table;
    let is_group = |c: usize| {
        matches!(
            part(c),
            TablePart::Header | TablePart::Body | TablePart::Footer
        )
    };
    // Rows of a group (or of the table's loose rows): a row's cells, an
    // anonymous row a run of loose cells.
    type Row = (Option<usize>, Vec<usize>);
    let rows_of = |items: &[usize]| -> Vec<Row> {
        let mut rows: Vec<Row> = Vec::new();
        let mut loose: Vec<usize> = Vec::new();
        for &c in items {
            if part(c) == TablePart::Row {
                if !loose.is_empty() {
                    rows.push((None, std::mem::take(&mut loose)));
                }
                rows.push((Some(c), shown_children(nodes, c)));
            } else {
                loose.push(c);
            }
        }
        if !loose.is_empty() {
            rows.push((None, loose));
        }
        rows
    };
    let mut captions: [Vec<usize>; 2] = Default::default();
    let mut columns: Vec<usize> = Vec::new();
    let mut groups: Vec<(Option<usize>, Vec<Row>)> = Vec::new();
    let (mut head, mut foot) = (None, None);
    let mut loose: Vec<usize> = Vec::new();
    let flush = |loose: &mut Vec<usize>, groups: &mut Vec<(Option<usize>, Vec<Row>)>| {
        if !loose.is_empty() {
            groups.push((None, rows_of(loose)));
            loose.clear();
        }
    };
    for &c in kids {
        match part(c) {
            TablePart::Caption { below } => captions[usize::from(below)].push(c),
            TablePart::Column | TablePart::ColumnGroup => columns.push(c),
            TablePart::Header if head.is_none() => head = Some(c),
            TablePart::Footer if foot.is_none() => foot = Some(c),
            _ if is_group(c) => {
                flush(&mut loose, &mut groups);
                groups.push((Some(c), rows_of(&shown_children(nodes, c))));
            }
            _ => loose.push(c),
        }
    }
    flush(&mut loose, &mut groups);
    if let Some(h) = head {
        groups.insert(0, (Some(h), rows_of(&shown_children(nodes, h))));
    }
    if let Some(f) = foot {
        groups.push((Some(f), rows_of(&shown_children(nodes, f))));
    }
    let mut t = Table::default();
    let mut order: Vec<usize> = Vec::new();
    let mut flow: Vec<usize> = Vec::new();
    // Columns paint under everything else.
    order.extend(&columns);
    let top = captions[0].len();
    // Occupied slots, [row][col], rows counted from the first after the
    // top captions.
    let mut occ: Vec<Vec<bool>> = Vec::new();
    let mut cells: Vec<(usize, (usize, usize, usize, usize))> = Vec::new();
    let mut row = 0usize;
    let mut group_parts: Vec<(usize, usize, usize)> = Vec::new();
    let mut row_parts: Vec<(usize, usize)> = Vec::new();
    for (g, rows) in &groups {
        let start = row;
        let end = start + rows.len();
        if let Some(g) = g {
            order.push(*g);
        }
        for (r, items) in rows {
            if let Some(r) = r {
                order.push(*r);
                row_parts.push((*r, row));
            }
            let mut col = 0usize;
            for &c in items {
                let p = &nodes[c].props.place;
                let cs = usize::from(p.column_span.max(1)).min(1000);
                let rs = match p.row_span {
                    u16::MAX => end - row,
                    n => usize::from(n.max(1)),
                }
                .min(end - row)
                .max(1);
                while occ
                    .get(row)
                    .and_then(|r| r.get(col))
                    .copied()
                    .unwrap_or(false)
                {
                    col += 1;
                }
                for rr in row..row + rs {
                    if occ.len() <= rr {
                        occ.resize(rr + 1, Vec::new());
                    }
                    if occ[rr].len() < col + cs {
                        occ[rr].resize(col + cs, false);
                    }
                    occ[rr][col..col + cs].iter_mut().for_each(|s| *s = true);
                }
                cells.push((c, (col, row + top, cs, rs)));
                order.push(c);
                col += cs;
            }
            row += 1;
        }
        if let Some(g) = g {
            group_parts.push((*g, start, end));
        }
    }
    let ncols = occ
        .iter()
        .map(Vec::len)
        .max()
        .unwrap_or(0)
        .max(columns.len())
        .max(1);
    let body = row;
    t.ncols = ncols;
    t.nrows = top + body + captions[1].len();
    for (k, &c) in captions[0].iter().enumerate() {
        flow.push(c);
        t.cells.push((0, k, ncols, 1));
    }
    for (c, slot) in &cells {
        flow.push(*c);
        t.cells.push(*slot);
    }
    for (k, &c) in captions[1].iter().enumerate() {
        flow.push(c);
        t.cells.push((0, top + body + k, ncols, 1));
    }
    let mut draw = captions[0].clone();
    draw.extend(order);
    draw.extend(&captions[1]);
    let rows_span = (top, body.max(1));
    t.widths = vec![0.0; ncols];
    for (k, &c) in columns.iter().enumerate().take(ncols) {
        t.parts.push((c, (k, rows_span.0, 1, rows_span.1)));
        if let Size::Fixed(w) = nodes[c].el.width {
            t.widths[k] = w;
        }
    }
    for (g, s, e) in group_parts {
        t.parts.push((g, (0, top + s, ncols, (e - s).max(1))));
    }
    for (r, at) in row_parts {
        t.parts.push((r, (0, top + at, ncols, 1)));
    }
    (draw, flow, t)
}

/// A table's grid layout from its [`Table`]: auto tracks (a column at
/// least its column box's width), the cells where the table put them.
fn place_table(nodes: &mut [Node], i: usize) {
    let n = &nodes[i];
    let Some(t) = &n.table else { return };
    let gap = [
        n.el.gaps[0].unwrap_or(n.el.gap),
        n.el.gaps[1].unwrap_or(n.el.gap),
    ];
    let cols = t
        .widths
        .iter()
        .map(|&w| {
            if w > 0.0 {
                Track::Range {
                    min: Breadth::Px(w),
                    max: Breadth::Auto,
                }
            } else {
                Track::Auto
            }
        })
        .collect();
    let layout = GridLayout {
        cells: t.cells.clone(),
        ncols: t.ncols,
        nrows: t.nrows.max(1),
        col_gap: gap[0],
        row_gap: gap[1],
        explicit: [cols, vec![Track::Auto; t.nrows.max(1)]],
        collapsed: [vec![false; t.ncols], vec![false; t.nrows.max(1)]],
        ..GridLayout::default()
    };
    nodes[i].grid = Some(layout);
}

/// Places a table's row, group and column boxes over their slots, in its
/// content box at `(x, y)`.
pub(super) fn place_table_parts(nodes: &mut [Node], i: usize, x: f32, y: f32) {
    let (Some(t), Some(g)) = (&nodes[i].table, &nodes[i].grid) else {
        return;
    };
    let placed: Vec<(usize, Rect)> = t
        .parts
        .iter()
        .map(|&(c, (col, row, cs, rs))| {
            let (x0, w) = g.span(0, col, cs);
            let (y0, h) = g.span(1, row, rs);
            (c, Rect::new(x + x0, y + y0, w, h))
        })
        .collect();
    for (c, r) in placed {
        let n = &mut nodes[c];
        (n.x, n.y, n.w, n.h) = (r.x, r.y, r.width, r.height);
    }
}
