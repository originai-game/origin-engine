//! CSS tables (CSS 2 §17, HTML's table processing model): a table's
//! boxes and the slots its cells, rows, row groups and columns take.

use super::*;
use crate::layout::TablePart;

/// A CSS table's grid (CSS 2 §17.5, HTML's table processing model): each
/// cell's slots, and the slots of the row, row group and column boxes that
/// paint behind them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Table {
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
        if n.is_hidden() || n.parent != i {
            continue;
        }
        if n.layout.used.contents {
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
    let part = |c: usize| nodes[c].layout.used.table;
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
        if let Size::Fixed(w) = nodes[c].layout.used.width {
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
