//! A grid container's tracks, areas and placement as Taffy's grid reads
//! them (CSS Grid 1 and 2). Line names, `repeat(auto-fill | auto-fit)`,
//! areas and auto-placement are Taffy's; `subgrid` takes its parent's
//! track sizes as fixed tracks ([`subgrid`]).

use super::super::*;
use crate::element::{Breadth, GridFlow, GridLine, GridPlace, GridSpec, Template, Track};
use taffy as tf;

/// The math functions grid `spec`'s tracks are sized by, each once, in
/// the order their handles number them ([`super::style::slot::TRACK`]).
pub(super) fn calcs(spec: &GridSpec) -> impl Iterator<Item = crate::Calc> {
    let repeat = |axis: usize| spec.lines[axis].repeat.iter().flat_map(|r| &r.tracks);
    crate::element::track_calcs(
        spec.columns
            .iter()
            .chain(repeat(0))
            .chain(&spec.rows)
            .chain(repeat(1))
            .chain(&spec.auto_columns)
            .chain(&spec.auto_rows)
            .chain([&spec.auto_track]),
    )
}

/// A grid's handle for each of its math functions ([`calcs`]).
type Handles<'a> = &'a dyn Fn(crate::Calc) -> *const ();

/// The handles of tracks sized by no math function (a table's).
const NO_CALC: Handles = &|_| std::ptr::null();

fn min_of(b: Breadth, calc: Handles) -> tf::MinTrackSizingFunction {
    match b {
        Breadth::Calc(c) => tf::MinTrackSizingFunction::calc(calc(c)),
        Breadth::Px(v) => tf::MinTrackSizingFunction::length(v),
        Breadth::Percent(p) => tf::MinTrackSizingFunction::percent(p),
        Breadth::MinContent => tf::MinTrackSizingFunction::min_content(),
        Breadth::MaxContent => tf::MinTrackSizingFunction::max_content(),
        // An `fr` is no min breadth (CSS Grid §7.2.3): `auto`.
        Breadth::Fr(_) | Breadth::Auto => tf::MinTrackSizingFunction::auto(),
    }
}

fn max_of(b: Breadth, calc: Handles) -> tf::MaxTrackSizingFunction {
    match b {
        Breadth::Calc(c) => tf::MaxTrackSizingFunction::calc(calc(c)),
        Breadth::Px(v) => tf::MaxTrackSizingFunction::length(v),
        Breadth::Percent(p) => tf::MaxTrackSizingFunction::percent(p),
        Breadth::Fr(f) => tf::MaxTrackSizingFunction::fr(f),
        Breadth::MinContent => tf::MaxTrackSizingFunction::min_content(),
        Breadth::MaxContent => tf::MaxTrackSizingFunction::max_content(),
        Breadth::Auto => tf::MaxTrackSizingFunction::auto(),
    }
}

/// A track size as Taffy's `minmax()`.
pub(crate) fn track(t: Track, calc: Handles) -> tf::TrackSizingFunction {
    let (lo, hi, fit) = t.breadths();
    let max = match fit {
        Some(Breadth::Px(v)) => tf::MaxTrackSizingFunction::fit_content_px(v),
        Some(Breadth::Percent(p)) => tf::MaxTrackSizingFunction::fit_content_percent(p),
        Some(Breadth::Calc(c)) => tf::MaxTrackSizingFunction::fit_content_calc(calc(c)),
        _ => max_of(hi, calc),
    };
    tf::MinMax {
        min: min_of(lo, calc),
        max,
    }
}

/// An axis's template: its tracks, its auto repetition in place, and the
/// names of the lines between them (one set per line).
fn template(
    spec: &GridSpec,
    axis: usize,
    calc: Handles,
) -> (Vec<tf::GridTemplateComponent<String>>, Vec<Vec<String>>) {
    let tracks = if axis == 0 { &spec.columns } else { &spec.rows };
    let lines = &spec.lines[axis];
    let named = |k: usize| lines.names.get(k).cloned().unwrap_or_default();
    let single = |t: &Track| tf::GridTemplateComponent::Single(track(*t, calc));
    let Some(rep) = &lines.repeat else {
        let names = if lines.names.is_empty() {
            Vec::new()
        } else {
            (0..=tracks.len()).map(named).collect()
        };
        return (tracks.iter().map(single).collect(), names);
    };
    let at = rep.at.min(tracks.len());
    let mut out: Vec<_> = tracks[..at].iter().map(single).collect();
    out.push(tf::GridTemplateComponent::Repeat(
        tf::GridTemplateRepetition {
            count: if rep.fit {
                tf::RepetitionCount::AutoFit
            } else {
                tf::RepetitionCount::AutoFill
            },
            tracks: rep.tracks.iter().map(|t| track(*t, calc)).collect(),
            line_names: if rep.names.len() == rep.tracks.len() + 1 {
                rep.names.clone()
            } else {
                Vec::new()
            },
        },
    ));
    out.extend(tracks[at..].iter().map(single));
    // The lines before the repetition, the one after it, then the rest.
    let mut names: Vec<Vec<String>> = (0..=at).map(named).collect();
    names.push(rep.after.clone());
    names.extend((at + 1..=tracks.len()).map(named));
    if names.iter().all(Vec::is_empty) {
        names.clear();
    }
    (out, names)
}

/// `grid-template-areas`: each name's rectangle of cells, 1-based lines.
fn areas(spec: &GridSpec) -> Option<tf::GridTemplateAreas<String>> {
    if spec.areas.is_empty() {
        return None;
    }
    let mut out: Vec<tf::GridTemplateArea<String>> = Vec::new();
    let mut cols = 0u16;
    for (r, line) in spec.areas.iter().enumerate() {
        let r = r as u16 + 1;
        let mut c = 0u16;
        for name in line.split_whitespace() {
            c += 1;
            if name.chars().all(|ch| ch == '.') {
                continue;
            }
            match out.iter_mut().find(|a| a.name == name) {
                Some(a) => {
                    a.column_start = a.column_start.min(c);
                    a.row_start = a.row_start.min(r);
                    a.column_end = a.column_end.max(c + 1);
                    a.row_end = a.row_end.max(r + 1);
                }
                None => out.push(tf::GridTemplateArea {
                    name: name.to_string(),
                    row_start: r,
                    row_end: r + 1,
                    column_start: c,
                    column_end: c + 1,
                }),
            }
        }
        cols = cols.max(c);
    }
    Some(tf::GridTemplateAreas {
        areas: out,
        row_count: spec.areas.len() as u16,
        column_count: cols,
    })
}

/// Grid `i`'s tracks, areas and alignment as a Taffy style (its own box
/// stays in its node: [`super::style::GridView`]). `sub` gives a subgrid
/// axis its parent's track sizes, px.
pub(crate) fn style(nodes: &[Node], i: usize, sub: [Option<&[f32]>; 2]) -> tf::Style<String> {
    let n = &nodes[i];
    let el = &n.layout.used;
    let spec = n.grid();
    let numbered: Vec<_> = calcs(spec).collect();
    let calc = &|c| {
        let k = numbered.iter().position(|n| *n == c).unwrap_or(0);
        super::style::calc_ptr(i, super::style::slot::TRACK + k)
    };
    let mut s = tf::Style::<String>::DEFAULT;
    s.display = tf::Display::Grid;
    for (axis, sub) in sub.into_iter().enumerate() {
        let (tracks, names) = match sub {
            Some(sizes) => (
                sizes
                    .iter()
                    .map(|&v| tf::GridTemplateComponent::Single(track(Track::Px(v), calc)))
                    .collect(),
                Vec::new(),
            ),
            None => template(spec, axis, calc),
        };
        let auto = if axis == 0 {
            &spec.auto_columns
        } else {
            &spec.auto_rows
        };
        let auto: Vec<_> = if auto.is_empty() {
            vec![track(spec.auto_track, calc)]
        } else {
            auto.iter().map(|t| track(*t, calc)).collect()
        };
        if axis == 0 {
            s.grid_template_columns = tracks;
            s.grid_template_column_names = names;
            s.grid_auto_columns = auto;
        } else {
            s.grid_template_rows = tracks;
            s.grid_template_row_names = names;
            s.grid_auto_rows = auto;
        }
    }
    s.grid_template_areas = areas(spec);
    s.grid_auto_flow = match (spec.flow, spec.dense) {
        (GridFlow::Row, false) => tf::GridAutoFlow::Row,
        (GridFlow::Row, true) => tf::GridAutoFlow::RowDense,
        (GridFlow::Column, false) => tf::GridAutoFlow::Column,
        (GridFlow::Column, true) => tf::GridAutoFlow::ColumnDense,
    };
    let gap =
        |g: Option<f32>, k: usize| tf::LengthPercentage::length(g.or(el.gaps[k]).unwrap_or(el.gap));
    s.gap = tf::Size {
        width: gap(spec.column_gap, 0),
        height: gap(spec.row_gap, 1),
    };
    let content = |a: Option<Align>| match a {
        None => tf::AlignContent::NORMAL,
        Some(Align::Start) => tf::AlignContent::START,
        Some(Align::Center) => tf::AlignContent::CENTER,
        Some(Align::End) => tf::AlignContent::END,
        Some(Align::SpaceBetween) => tf::AlignContent::SPACE_BETWEEN,
        Some(Align::SpaceAround) => tf::AlignContent::SPACE_AROUND,
        Some(Align::SpaceEvenly) => tf::AlignContent::SPACE_EVENLY,
        Some(Align::Stretch) => tf::AlignContent::STRETCH,
    };
    s.justify_content = content(spec.content[0]);
    s.align_content = content(spec.content[1]);
    let items = |c: Option<Cross>| match c {
        None => tf::AlignItems::NORMAL,
        Some(Cross::Start) => tf::AlignItems::START,
        Some(Cross::Center) => tf::AlignItems::CENTER,
        Some(Cross::End) => tf::AlignItems::END,
        Some(Cross::Stretch) => tf::AlignItems::STRETCH,
        Some(Cross::Baseline | Cross::LastBaseline) => tf::AlignItems::BASELINE,
    };
    s.align_items = items(spec.align_items);
    s.justify_items = items(el.justify_items);
    s
}

/// An item's placement lines per axis (columns, rows): its CSS lines,
/// else its area, else the house's numbers (`column`, `row`, spans).
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

fn line(l: &GridLine) -> tf::GridPlacement<String> {
    let int = |v: i64| v.clamp(i64::from(i16::MIN), i64::from(i16::MAX)) as i16;
    let span = |v: u32| v.clamp(1, u32::from(u16::MAX)) as u16;
    match l {
        GridLine::Auto => tf::GridPlacement::Auto,
        GridLine::Line(n, name) if name.is_empty() => {
            tf::GridPlacement::Line(int(i64::from(*n)).into())
        }
        GridLine::Line(n, name) => tf::GridPlacement::NamedLine(name.clone(), int(i64::from(*n))),
        GridLine::Span(n, name) if name.is_empty() => tf::GridPlacement::Span(span(*n)),
        GridLine::Span(n, name) => tf::GridPlacement::NamedSpan(name.clone(), span(*n)),
    }
}

/// Node `i`'s placement on `axis` (0: columns) of the grid it is in; a
/// table's cells sit at the slots the table gave them.
pub(crate) fn placement(
    nodes: &[Node],
    i: usize,
    axis: usize,
) -> tf::Line<tf::GridPlacement<String>> {
    let n = &nodes[i];
    if n.up != NONE
        && let Some(t) = &nodes[n.up].layout.table
        && let Some(k) = nodes[n.up].children.iter().position(|&c| c == i)
        && let Some(&(col, row, cs, rs)) = t.cells.get(k)
    {
        let (at, span) = if axis == 0 { (col, cs) } else { (row, rs) };
        return tf::Line {
            start: tf::GridPlacement::Line((at as i16 + 1).into()),
            end: tf::GridPlacement::Span(span.max(1) as u16),
        };
    }
    let [start, end] = &item_lines(&n.props.place)[axis];
    tf::Line {
        start: line(start),
        end: line(end),
    }
}

/// A table `i` as a grid of its slots: auto tracks, a column at least its
/// column box's width.
pub(crate) fn table_style(nodes: &[Node], i: usize) -> tf::Style<String> {
    let n = &nodes[i];
    let el = &n.layout.used;
    let mut s = tf::Style::<String>::DEFAULT;
    s.display = tf::Display::Grid;
    let Some(t) = &n.layout.table else { return s };
    // A column a cell sizes by a percentage takes the width the others
    // leave (CSS 2 §17.5.2.2's automatic layout, as browsers resolve a
    // `width: 100%` cell); the others fit their content, and stretch
    // where none does.
    let kids = &n.children;
    let percent = |k: usize| {
        t.cells.iter().zip(kids).any(|(&(col, _, span, _), &c)| {
            col == k && span <= 1 && matches!(nodes[c].layout.used.width, Size::Percent(_))
        })
    };
    s.grid_template_columns = (0..t.ncols)
        .map(|k| {
            let w = t.widths.get(k).copied().unwrap_or(0.0);
            let min = if w > 0.0 {
                Breadth::Px(w)
            } else {
                Breadth::Auto
            };
            let max = if percent(k) {
                Breadth::Fr(1.0)
            } else {
                Breadth::Auto
            };
            tf::GridTemplateComponent::Single(track(Track::Range { min, max }, NO_CALC))
        })
        .collect();
    s.grid_template_rows = (0..t.nrows.max(1))
        .map(|_| tf::GridTemplateComponent::Single(track(Track::Auto, NO_CALC)))
        .collect();
    s.gap = tf::Size {
        width: tf::LengthPercentage::length(el.gaps[0].unwrap_or(el.gap)),
        height: tf::LengthPercentage::length(el.gaps[1].unwrap_or(el.gap)),
    };
    s.align_content = tf::AlignContent::START;
    s
}

/// Whether grid `i`'s `axis` is a subgrid of the grid it is in.
pub(crate) fn subgrid(nodes: &[Node], i: usize, axis: usize) -> bool {
    let n = &nodes[i];
    n.layout_mode() == LayoutMode::Grid
        && n.grid().lines[axis].kind == Template::Subgrid
        && n.up != NONE
        && nodes[n.up].layout_mode() == LayoutMode::Grid
}
