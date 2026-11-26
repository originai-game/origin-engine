//! Flowchart layout (`og.flow.chart`) with 2-pass barycenter crossing
//! minimization and rich node/link metadata.

use super::{Node, Prepared};
use crate::host::wire::{FlowChart, FlowChartLink, FlowChartNode};
use std::collections::BTreeMap;

/// Graph units between columns and between rows, and around the chart.
const CHART_COLUMN: f32 = 220.0;
const CHART_ROW: f32 = 130.0;
const CHART_MARGIN: f32 = 60.0;

/// A link from (x, y) to (x2, y2) leaving and arriving level: the cubic
/// with both handles at the middle column, in 16 segments.
fn level_curve(x: f32, y: f32, x2: f32, y2: f32) -> Vec<f32> {
    let mid = (x + x2) * 0.5;
    (0..=16)
        .flat_map(|k| {
            let u = k as f32 / 16.0;
            let w = 1.0 - u;
            let (a, b, c, d) = (w * w * w, 3.0 * w * w * u, 3.0 * w * u * u, u * u * u);
            [a * x + (b + c) * mid + d * x2, (a + b) * y + (c + d) * y2]
        })
        .collect()
}

fn node_kind(n: &Node) -> &'static str {
    match (n.first.is_some(), n.live) {
        (true, true) => "both",
        (false, true) => "live",
        _ => "footage",
    }
}

/// Columns by the fewest flow nodes from the start (breadth first, so a
/// loop back draws as a link to an earlier column); rows ordered by a 2-pass
/// Sugiyama barycenter sweep to minimize link crossings. Nodes the start never
/// reaches follow the deepest column.
pub(super) fn build(engine_id: u32, p: &Prepared) -> FlowChart {
    let leads = |n: &Node| -> Vec<u32> {
        let mut out: Vec<u32> = Vec::new();
        for to in n
            .choices_list
            .iter()
            .flat_map(|c| c.to.iter())
            .chain(&n.next)
        {
            if !out.contains(to) {
                out.push(*to);
            }
        }
        out
    };
    let mut depth: BTreeMap<u32, u32> = BTreeMap::new();
    let mut columns: Vec<Vec<u32>> = Vec::new();
    let mut frontier: Vec<u32> = Vec::new();
    for r in &p.roots {
        if p.nodes.contains_key(r) && !depth.contains_key(r) {
            depth.insert(*r, 0);
            frontier.push(*r);
        }
    }
    while !frontier.is_empty() {
        let d = columns.len() as u32;
        let mut next = Vec::new();
        for id in &frontier {
            for to in leads(&p.nodes[id]) {
                if p.nodes.contains_key(&to) && !depth.contains_key(&to) {
                    depth.insert(to, d + 1);
                    next.push(to);
                }
            }
        }
        columns.push(std::mem::replace(&mut frontier, next));
    }
    let unreached: Vec<u32> = p
        .order
        .iter()
        .copied()
        .filter(|id| !depth.contains_key(id))
        .collect();
    if !unreached.is_empty() {
        for id in &unreached {
            depth.insert(*id, columns.len() as u32);
        }
        columns.push(unreached);
    }

    // Build incoming and outgoing adjacency maps for barycenter ordering.
    let mut outgoing: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    let mut incoming: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for (&id, n) in &p.nodes {
        let tos = leads(n);
        for &to in &tos {
            if p.nodes.contains_key(&to) {
                incoming.entry(to).or_default().push(id);
            }
        }
        outgoing.insert(id, tos);
    }

    // Two forward + backward barycenter sweeps over adjacent layers.
    let order_by_neighbors =
        |col: &mut Vec<u32>, adj: &BTreeMap<u32, Vec<u32>>, neighbor_pos: &BTreeMap<u32, usize>| {
            let indexed: Vec<(usize, u32)> = col.iter().copied().enumerate().collect();
            let mut scored: Vec<(f32, usize, u32)> = indexed
                .into_iter()
                .map(|(orig_idx, id)| {
                    let (sum, cnt) = adj
                        .get(&id)
                        .into_iter()
                        .flatten()
                        .filter_map(|nb| neighbor_pos.get(nb))
                        .fold((0usize, 0usize), |(s, c), &pos| (s + pos, c + 1));
                    let score = if cnt > 0 {
                        sum as f32 / cnt as f32
                    } else {
                        orig_idx as f32
                    };
                    (score, orig_idx, id)
                })
                .collect();
            scored.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            *col = scored.into_iter().map(|(_, _, id)| id).collect();
        };
    if columns.len() > 1 {
        for _ in 0..2 {
            for c in 1..columns.len() {
                let prev_pos: BTreeMap<u32, usize> = columns[c - 1]
                    .iter()
                    .enumerate()
                    .map(|(i, &id)| (id, i))
                    .collect();
                order_by_neighbors(&mut columns[c], &incoming, &prev_pos);
            }
            for c in (0..columns.len() - 1).rev() {
                let next_pos: BTreeMap<u32, usize> = columns[c + 1]
                    .iter()
                    .enumerate()
                    .map(|(i, &id)| (id, i))
                    .collect();
                order_by_neighbors(&mut columns[c], &outgoing, &next_pos);
            }
        }
    }

    let mut at: BTreeMap<u32, (f32, f32)> = BTreeMap::new();
    for (c, col) in columns.iter().enumerate() {
        for (r, id) in col.iter().enumerate() {
            at.insert(
                *id,
                (
                    CHART_MARGIN + c as f32 * CHART_COLUMN,
                    CHART_MARGIN + r as f32 * CHART_ROW,
                ),
            );
        }
    }
    let rows = columns.iter().map(Vec::len).max().unwrap_or(0);
    let mut out = FlowChart {
        width: 2.0 * CHART_MARGIN + columns.len().saturating_sub(1) as f32 * CHART_COLUMN,
        height: 2.0 * CHART_MARGIN + rows.saturating_sub(1) as f32 * CHART_ROW,
        ..FlowChart::default()
    };
    for id in &p.order {
        let n = &p.nodes[id];
        let (x, y) = at[id];
        out.nodes.push(FlowChartNode {
            node: n.title.clone(),
            title: if n.display_title.is_empty() {
                n.title.clone()
            } else {
                crate::bind::i18n::label(engine_id, &n.display_title)
            },
            segment: n.segment.clone(),
            poster: n.poster,
            kind: node_kind(n).to_string(),
            chapter: n.chapter.clone(),
            ending: n.ending.clone(),
            depth: depth[id],
            x,
            y,
        });
        let mut push_link =
            |to: u32, choice: Option<u32>, label: String, outcome: String, conditional: bool| {
                let Some(t) = p.nodes.get(&to) else { return };
                let (x2, y2) = at[&to];
                out.links.push(FlowChartLink {
                    from: n.title.clone(),
                    to: t.title.clone(),
                    choice,
                    label,
                    outcome,
                    conditional,
                    points: level_curve(x, y, x2, y2),
                });
            };
        for c in &n.choices_list {
            let label = if c.key.is_empty() {
                String::new()
            } else {
                crate::bind::story::localize_key(engine_id, &c.key)
            };
            let outcome = c.outcome.clone().unwrap_or_default();
            let conditional = c.conditional || c.to.len() > 1;
            for &to in &c.to {
                push_link(
                    to,
                    Some(c.index),
                    label.clone(),
                    outcome.clone(),
                    conditional,
                );
            }
        }
        if n.choices_list.is_empty() {
            let conditional = n.next.len() > 1;
            for &to in &n.next {
                push_link(to, None, String::new(), String::new(), conditional);
            }
        }
    }
    out
}
