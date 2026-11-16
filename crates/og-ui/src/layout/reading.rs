//! CSS Display 4 §3: `reading-flow` and `reading-order`. A flex or grid
//! container whose `reading-flow` is not `normal` sets the order sequential
//! focus moves through its items in (Tab and Shift+Tab, a list's Home and
//! End, the accessibility tree's focus order): their places on screen
//! (`flex-visual`, `grid-rows`, `grid-columns`), their `order`
//! (`flex-flow`, `grid-order`) or the source (`source-order`), each item's
//! `reading-order` first. Pointer hits and arrow moves keep the painting
//! order; in a `normal` container whose `order` moved an item, which
//! paints its items in that order, focus keeps the document's.

use super::css::{Kind, kind};
use super::*;

/// CSS `reading-flow` (CSS Display 4 §3.1).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReadingFlow {
    #[default]
    Normal,
    FlexVisual,
    FlexFlow,
    GridRows,
    GridColumns,
    GridOrder,
    SourceOrder,
}

impl ReadingFlow {
    /// From its lowered word (`flexVisual`, …); `None`: not one.
    pub fn from_word(w: &str) -> Option<ReadingFlow> {
        Some(match w {
            "normal" => ReadingFlow::Normal,
            "flexVisual" => ReadingFlow::FlexVisual,
            "flexFlow" => ReadingFlow::FlexFlow,
            "gridRows" => ReadingFlow::GridRows,
            "gridColumns" => ReadingFlow::GridColumns,
            "gridOrder" => ReadingFlow::GridOrder,
            "sourceOrder" => ReadingFlow::SourceOrder,
            _ => return None,
        })
    }

    /// Whether it orders container `i`'s items: the flex values in a flex
    /// container, the grid values in a grid, `source-order` in either, and
    /// `normal` in either when `order` moved an item: its items are hit in
    /// the order they paint ([`super::paint_children`]), and focus keeps
    /// the document's.
    fn applies(self, nodes: &[Node], i: usize) -> bool {
        match (self, kind(nodes, i)) {
            (ReadingFlow::Normal, _) => super::order_moves(nodes, i),
            (ReadingFlow::FlexVisual | ReadingFlow::FlexFlow, k) => k == Kind::Flex,
            (ReadingFlow::GridRows | ReadingFlow::GridColumns | ReadingFlow::GridOrder, k) => {
                k == Kind::Grid
            }
            (ReadingFlow::SourceOrder, k) => matches!(k, Kind::Flex | Kind::Grid),
        }
    }
}

/// `hits`, the nodes focus reaches in the tree's order, in reading order:
/// the hits inside each reading-flow container's items move among the
/// places those hits held, item by item in the container's reading order,
/// innermost containers first (an item's own hits keep their order).
/// `rtl`: the screen reads right to left. Whether any container did.
pub(crate) fn reorder(nodes: &[Node], hits: &mut [usize], rtl: bool) -> bool {
    // Each container's places in `hits`, and how deep it is.
    let mut held: Vec<(usize, usize, Vec<usize>)> = Vec::new();
    for (s, &h) in hits.iter().enumerate() {
        let mut at = h;
        while nodes[at].parent != NONE {
            let p = nodes[at].parent;
            if nodes[p].layout.used.reading_flow.applies(nodes, p) {
                match held.iter_mut().find(|(c, _, _)| *c == p) {
                    Some((_, _, slots)) => slots.push(s),
                    None => {
                        let mut depth = 0;
                        let mut up = p;
                        while nodes[up].parent != NONE {
                            up = nodes[up].parent;
                            depth += 1;
                        }
                        held.push((p, depth, vec![s]));
                    }
                }
            }
            at = p;
        }
    }
    if held.is_empty() {
        return false;
    }
    held.sort_by_key(|&(_, depth, _)| std::cmp::Reverse(depth));
    for (c, _, slots) in held {
        let rank = ranks(nodes, c, rtl);
        let item = |h: usize| {
            let mut at = h;
            while nodes[at].parent != c {
                at = nodes[at].parent;
            }
            nodes[c].children.iter().position(|&k| k == at).unwrap_or(0)
        };
        let mut moved: Vec<(i32, usize, usize, usize)> = slots
            .iter()
            .map(|&s| {
                let k = item(hits[s]);
                let order = nodes[nodes[c].children[k]].layout.used.reading_order;
                (order, rank[k], s, hits[s])
            })
            .collect();
        moved.sort_unstable();
        for (&s, &(_, _, _, h)) in slots.iter().zip(&moved) {
            hits[s] = h;
        }
    }
    true
}

/// Each of container `i`'s children's place in its reading flow (by the
/// index in its `children`), before `reading-order`.
fn ranks(nodes: &[Node], i: usize, rtl: bool) -> Vec<usize> {
    let n = &nodes[i];
    let kids = &n.children;
    let used = &n.layout.used;
    let by = |key: &dyn Fn(usize) -> (i64, i64)| {
        let mut seq: Vec<usize> = (0..kids.len()).collect();
        seq.sort_by_key(|&k| (key(k), k));
        let mut rank = vec![0; kids.len()];
        for (r, k) in seq.into_iter().enumerate() {
            rank[k] = r;
        }
        rank
    };
    let order = |k: usize| i64::from(nodes[kids[k]].layout.used.order);
    // `children` are in `order`; the document's is `ordered_children`.
    let source = || {
        let mut at = vec![0i64; nodes.len()];
        for (p, &c) in n.ordered_children.iter().enumerate() {
            at[c] = p as i64;
        }
        at
    };
    let area = |k: usize| n.layout.grid_items.get(k).copied().unwrap_or([u16::MAX; 4]);
    match used.reading_flow {
        ReadingFlow::SourceOrder | ReadingFlow::Normal => {
            let at = source();
            by(&|k| (at[kids[k]], 0))
        }
        ReadingFlow::GridOrder => by(&|k| (order(k), 0)),
        ReadingFlow::FlexFlow => {
            let rank = by(&|k| (order(k), 0));
            if used.reverse {
                rank.iter().map(|r| kids.len() - 1 - r).collect()
            } else {
                rank
            }
        }
        ReadingFlow::GridRows => by(&|k| (i64::from(area(k)[2]), i64::from(area(k)[0]))),
        ReadingFlow::GridColumns => by(&|k| (i64::from(area(k)[0]), i64::from(area(k)[2]))),
        ReadingFlow::FlexVisual => visual(nodes, i, rtl),
    }
}

/// A flex container's items in visual reading order (CSS Display 4
/// §3.1 `flex-visual`): line by line from the top (a column flex: column
/// by column in the inline direction), each line in the inline direction
/// (a column: from the top).
fn visual(nodes: &[Node], i: usize, rtl: bool) -> Vec<usize> {
    let n = &nodes[i];
    let kids = &n.children;
    let row = n.layout.used.direction == Direction::Row;
    let at = |k: usize| {
        let l = &nodes[kids[k]].layout;
        // (main, cross) from the screen's reading start.
        let x = if rtl { -(l.x + l.w) } else { l.x };
        if row { (x, l.y) } else { (l.y, x) }
    };
    // The flow's order, and its lines: a new line where the main position
    // stops moving the flow's way.
    let mut flow: Vec<usize> = (0..kids.len()).collect();
    flow.sort_by_key(|&k| (nodes[kids[k]].layout.used.order, k));
    if n.layout.used.reverse {
        flow.reverse();
    }
    // Right to left, a row's flow runs from the right: still ahead in
    // the mirrored positions above.
    let ahead = !n.layout.used.reverse;
    let mut line = vec![0usize; kids.len()];
    let mut l = 0;
    for w in flow.windows(2) {
        let (a, b) = (at(w[0]).0, at(w[1]).0);
        if if ahead { b < a - 0.5 } else { b > a + 0.5 } {
            l += 1;
        }
        line[w[1]] = l;
    }
    // Lines from the cross start, items along the main axis.
    let mut start = vec![f32::INFINITY; l + 1];
    for k in 0..kids.len() {
        start[line[k]] = start[line[k]].min(at(k).1);
    }
    let mut seq: Vec<usize> = (0..kids.len()).collect();
    seq.sort_by(|&a, &b| {
        (start[line[a]], at(a).0, a)
            .partial_cmp(&(start[line[b]], at(b).0, b))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut rank = vec![0; kids.len()];
    for (r, k) in seq.into_iter().enumerate() {
        rank[k] = r;
    }
    rank
}
