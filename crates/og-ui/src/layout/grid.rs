use super::*;

/// A grid's resolved tracks and where each child in the flow sits.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct GridLayout {
    /// Per child in the flow: its column, row, column span, row span (0-based).
    pub cells: Vec<(usize, usize, usize, usize)>,
    pub ncols: usize,
    pub nrows: usize,
    pub cols: Vec<f32>,
    pub rows: Vec<f32>,
    pub col_gap: f32,
    pub row_gap: f32,
}

/// Places a grid's children in its cells: named areas and explicit lines
/// first, then the rest in flow order (`dense` back-fills holes).
pub(super) fn place_grid(nodes: &mut [Node], i: usize) {
    let n = &nodes[i];
    let spec = &n.props.grid;
    let mut areas: Vec<(&str, usize, usize, usize, usize)> = Vec::new();
    for (r, line) in spec.areas.iter().enumerate() {
        for (c, name) in line.split_whitespace().enumerate() {
            if name == "." {
                continue;
            }
            match areas.iter_mut().find(|a| a.0 == name) {
                Some(a) => {
                    a.1 = a.1.min(c);
                    a.2 = a.2.min(r);
                    a.3 = a.3.max(c + 1);
                    a.4 = a.4.max(r + 1);
                }
                None => areas.push((name, c, r, c + 1, r + 1)),
            }
        }
    }
    let row_flow = spec.flow == GridFlow::Row;
    let area_cols = spec
        .areas
        .iter()
        .map(|l| l.split_whitespace().count())
        .max()
        .unwrap_or(0);
    let mut ncols = spec.columns.len().max(area_cols);
    let mut nrows = spec.rows.len().max(spec.areas.len());
    let kids = n.children.clone();
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
    let mut auto = Vec::new();
    for (k, &c) in kids.iter().enumerate() {
        let p = &nodes[c].props.place;
        let (cs, rs) = (p.column_span.max(1) as usize, p.row_span.max(1) as usize);
        if !p.area.is_empty()
            && let Some(a) = areas.iter().find(|a| a.0 == p.area)
        {
            cells[k] = (a.1, a.2, a.3 - a.1, a.4 - a.2);
            mark(&mut occ, a.1, a.2, a.3 - a.1, a.4 - a.2);
            continue;
        }
        if p.column > 0 && p.row > 0 {
            let (col, row) = (p.column as usize - 1, p.row as usize - 1);
            cells[k] = (col, row, cs, rs);
            mark(&mut occ, col, row, cs, rs);
            ncols = ncols.max(col + cs);
            nrows = nrows.max(row + rs);
            continue;
        }
        auto.push(k);
    }
    // The fixed axis (columns in a row flow) and the cursor along it.
    let major = if row_flow { ncols.max(1) } else { nrows.max(1) };
    let (mut cur_line, mut cur_at) = (0usize, 0usize);
    for k in auto {
        let p = &nodes[kids[k]].props.place;
        let (cs, rs) = (p.column_span.max(1) as usize, p.row_span.max(1) as usize);
        let (along, across) = if row_flow { (cs, rs) } else { (rs, cs) };
        let along = along.min(major);
        let fixed = if row_flow { p.column } else { p.row } as usize;
        if spec.dense {
            cur_line = 0;
            cur_at = 0;
        }
        let (mut line, mut at) = (cur_line, cur_at);
        if fixed > 0 {
            at = (fixed - 1).min(major - along);
            if at < cur_at && !spec.dense {
                line += 1;
            }
        }
        loop {
            if at + along > major {
                at = if fixed > 0 {
                    (fixed - 1).min(major - along)
                } else {
                    0
                };
                line += 1;
                continue;
            }
            let (c, r, w, h) = if row_flow {
                (at, line, along, across)
            } else {
                (line, at, across, along)
            };
            if free(&occ, c, r, w, h) {
                cells[k] = (c, r, w, h);
                mark(&mut occ, c, r, w, h);
                cur_line = line;
                cur_at = at + along;
                break;
            }
            if fixed > 0 {
                line += 1;
            } else {
                at += 1;
            }
        }
    }
    for &(c, r, w, h) in &cells {
        ncols = ncols.max(c + w);
        nrows = nrows.max(r + h);
    }
    if row_flow {
        ncols = ncols.max(major);
    } else {
        nrows = nrows.max(major);
    }
    let gap = n.el.gap;
    let layout = GridLayout {
        cells,
        ncols,
        nrows,
        cols: Vec::new(),
        rows: Vec::new(),
        col_gap: spec.column_gap.unwrap_or(gap),
        row_gap: spec.row_gap.unwrap_or(gap),
    };
    nodes[i].grid = Some(layout);
}

/// Track sizes along one axis: `items` are (first track, span, preferred,
/// minimum); `avail` the inner size (`None`: intrinsic, every `fr` track
/// as large as its content asks).
fn track_sizes(
    defs: &[Track],
    auto_track: Track,
    n: usize,
    items: &[(usize, usize, f32, f32)],
    avail: Option<f32>,
    gap: f32,
) -> Vec<f32> {
    let def = |k: usize| defs.get(k).copied().unwrap_or(auto_track);
    let mut content = vec![0.0f32; n];
    let mut least = vec![0.0f32; n];
    for &(s, span, pref, min) in items {
        if span == 1 && s < n {
            content[s] = content[s].max(pref);
            least[s] = least[s].max(min);
        }
    }
    let mut sizes: Vec<f32> = (0..n)
        .map(|k| match def(k) {
            Track::Px(v) => v,
            Track::Percent(p) => avail.map_or(content[k], |a| p * a),
            Track::Auto => content[k],
            Track::MinMax { min, max } => content[k].clamp(min, max.max(min)),
            Track::Fr(_) => {
                if avail.is_some() {
                    least[k]
                } else {
                    content[k]
                }
            }
        })
        .collect();
    // A spanning item wider than its tracks grows their flexible ones.
    for &(s, span, pref, _) in items {
        if span <= 1 || s >= n {
            continue;
        }
        let e = (s + span).min(n);
        let have: f32 = sizes[s..e].iter().sum::<f32>() + gap * (e - s - 1) as f32;
        if pref <= have {
            continue;
        }
        let flex: Vec<usize> = (s..e)
            .filter(|&k| matches!(def(k), Track::Auto | Track::Fr(_) | Track::MinMax { .. }))
            .collect();
        let grow = if flex.is_empty() {
            (s..e).collect()
        } else {
            flex
        };
        let each = (pref - have) / grow.len() as f32;
        for k in grow {
            sizes[k] += each;
        }
    }
    let fr: f32 = (0..n)
        .map(|k| match def(k) {
            Track::Fr(f) => f.max(0.0),
            _ => 0.0,
        })
        .sum();
    if fr > 0.0 {
        match avail {
            Some(a) => {
                let fixed: f32 = (0..n)
                    .filter(|&k| !matches!(def(k), Track::Fr(_)))
                    .map(|k| sizes[k])
                    .sum();
                let free = (a - fixed - gap * n.saturating_sub(1) as f32).max(0.0);
                let unit = free / fr;
                for k in 0..n {
                    if let Track::Fr(f) = def(k) {
                        sizes[k] = (f.max(0.0) * unit).max(least[k]);
                    }
                }
            }
            None => {
                // Every fr track as wide as the largest content per unit asks.
                let unit = (0..n)
                    .filter_map(|k| match def(k) {
                        Track::Fr(f) if f > 0.0 => Some(sizes[k] / f),
                        _ => None,
                    })
                    .fold(0.0, f32::max);
                for (k, size) in sizes.iter_mut().enumerate() {
                    if let Track::Fr(f) = def(k) {
                        *size = f.max(0.0) * unit;
                    }
                }
            }
        }
    }
    sizes
}

fn grid_items(nodes: &[Node], i: usize, horizontal: bool) -> Vec<(usize, usize, f32, f32)> {
    let n = &nodes[i];
    let Some(g) = &n.grid else {
        return Vec::new();
    };
    n.children
        .iter()
        .zip(&g.cells)
        .map(|(&c, &(col, row, cs, rs))| {
            let k = &nodes[c];
            if horizontal {
                (col, cs, k.pref_w, k.min_w)
            } else {
                (row, rs, k.pref_h, k.min_h)
            }
        })
        .collect()
}

/// A grid's preferred and least size along one axis.
pub(super) fn grid_intrinsic(nodes: &[Node], i: usize, horizontal: bool) -> (f32, f32) {
    let n = &nodes[i];
    let Some(g) = &n.grid else {
        return (0.0, 0.0);
    };
    let spec = &n.props.grid;
    let (defs, count, gap) = if horizontal {
        (&spec.columns, g.ncols, g.col_gap)
    } else {
        (&spec.rows, g.nrows, g.row_gap)
    };
    let items = grid_items(nodes, i, horizontal);
    let gaps = gap * count.saturating_sub(1) as f32;
    let pref: f32 = track_sizes(defs, spec.auto_track, count, &items, None, gap)
        .iter()
        .sum::<f32>()
        + gaps;
    let mins: Vec<(usize, usize, f32, f32)> =
        items.iter().map(|&(s, n, _, m)| (s, n, m, m)).collect();
    let min: f32 = track_sizes(defs, spec.auto_track, count, &mins, None, gap)
        .iter()
        .sum::<f32>()
        + gaps;
    (pref, min.min(pref))
}

/// Resolves a grid's columns at its inner width and sizes its children.
pub(super) fn grid_columns(nodes: &mut [Node], i: usize, inner: f32) {
    let items = grid_items(nodes, i, true);
    let n = &nodes[i];
    let Some(g) = &n.grid else { return };
    let spec = &n.props.grid;
    let cols = track_sizes(
        &spec.columns,
        spec.auto_track,
        g.ncols,
        &items,
        Some(inner),
        g.col_gap,
    );
    let gap = g.col_gap;
    let kids = n.children.clone();
    let cells = g.cells.clone();
    for (&c, &(col, _, cs, _)) in kids.iter().zip(&cells) {
        let e = (col + cs).min(cols.len());
        let span: f32 = cols[col.min(e)..e].iter().sum::<f32>()
            + gap * (e.saturating_sub(col)).saturating_sub(1) as f32;
        let k = &nodes[c];
        nodes[c].w = match k.el.width {
            Size::Fixed(v) => v,
            Size::Percent(p) => p * span,
            Size::Fit { .. } | Size::Grow { .. } => {
                let (lo, hi) = k.el.width.bounds();
                span.clamp(lo, hi)
            }
        };
    }
    if let Some(g) = nodes[i].grid.as_mut() {
        g.cols = cols;
    }
}

/// Resolves a grid's rows at its inner height and sizes its children.
pub(super) fn grid_rows(nodes: &mut [Node], i: usize, inner: f32) {
    let items = grid_items(nodes, i, false);
    let n = &nodes[i];
    let Some(g) = &n.grid else { return };
    let spec = &n.props.grid;
    let rows = track_sizes(
        &spec.rows,
        spec.auto_track,
        g.nrows,
        &items,
        Some(inner),
        g.row_gap,
    );
    let gap = g.row_gap;
    let kids = n.children.clone();
    let cells = g.cells.clone();
    for (&c, &(_, row, _, rs)) in kids.iter().zip(&cells) {
        let e = (row + rs).min(rows.len());
        let span: f32 = rows[row.min(e)..e].iter().sum::<f32>()
            + gap * (e.saturating_sub(row)).saturating_sub(1) as f32;
        let k = &nodes[c];
        nodes[c].h = match k.el.height {
            Size::Fixed(v) => v,
            Size::Percent(p) => p * span,
            Size::Fit { .. } | Size::Grow { .. } => {
                let (lo, hi) = k.el.height.bounds();
                span.clamp(lo, hi)
            }
        };
    }
    if let Some(g) = nodes[i].grid.as_mut() {
        g.rows = rows;
    }
}
