use super::*;

/// A collapsing margin set (CSS 2 §8.3.1): its largest positive and its
/// most negative margin; it spans their sum.
pub(in crate::layout) type MarginSet = (f32, f32);

fn margin_set(m: f32) -> MarginSet {
    (m.max(0.0), m.min(0.0))
}

fn joined(a: MarginSet, b: MarginSet) -> MarginSet {
    (a.0.max(b.0), a.1.min(b.1))
}

pub(in crate::layout) fn span(m: MarginSet) -> f32 {
    m.0 + m.1
}

/// The width node `i`'s margin percentages read: its parent's content
/// width (CSS 2 §8.3: all four, even the vertical ones).
pub(in crate::layout) fn margin_basis(nodes: &[Node], i: usize) -> f32 {
    let p = nodes[i].up;
    if p == NONE || p >= nodes.len() {
        return 0.0;
    }
    (nodes[p].layout.w - nodes[p].layout.used.padding.horizontal()).max(0.0)
}

/// Node `i`'s collapsed outer margins, and whether its first child's top
/// and last child's bottom margin collapse through its own edges: it is a
/// CSS block in a block parent's flow (so no new formatting context), with
/// no padding at that edge, and at the bottom no set height. An empty block
/// in a block's flow collapses through itself: its own top and bottom
/// margins, and those of its children, are one set (CSS 2 §8.3.1).
pub(in crate::layout) fn collapse_margins(nodes: &mut [Node], i: usize) {
    let n = &nodes[i];
    let p = n.up;
    let in_block_flow = p != NONE
        && p < nodes.len()
        && nodes[p].layout.used.block
        && nodes[p].layout.used.direction == Direction::Column
        && nodes[p].layout_mode() == LayoutMode::Flex
        && nodes[p].children.contains(&i);
    // A flow-root (or a box laying out lines) starts a formatting
    // context: no margin collapses through its edges (CSS 2 §8.3.1).
    let flows = in_block_flow
        && n.layout.used.block
        && !n.layout.used.bfc
        && n.layout.used.direction == Direction::Column
        && n.layout_mode() == LayoutMode::Flex
        && matches!(*n.content, Content::None)
        && !n.layout.used.scroll
        && !n.layout.used.scroll_x;
    let auto_height = matches!(n.layout.used.height, Size::Fit { min, .. } if min <= 0.0);
    let empty_text = matches!(&*n.content, Content::Text(t) if t.text.is_empty());
    let through = in_block_flow
        && !n.layout.used.bfc
        && auto_height
        && n.layout.used.padding.top == 0.0
        && n.layout.used.padding.bottom == 0.0
        && n.out.is_empty()
        && ((flows && n.children.iter().all(|&c| nodes[c].layout.through)) || empty_text);
    let basis = margin_basis(nodes, i);
    let edge = |m: crate::layout::Margin| margin_set(m.of(basis));
    let mut sets = [edge(n.layout.used.margin[0]), edge(n.layout.used.margin[2])];
    let collapse_in = if through {
        [true, true]
    } else {
        let kids = !n.children.is_empty();
        [
            flows && kids && n.layout.used.padding.top == 0.0,
            flows && kids && n.layout.used.padding.bottom == 0.0 && auto_height,
        ]
    };
    // The run of margins that meets an edge: each child that collapses
    // through itself passes on to the next.
    let chain = |kids: &mut dyn Iterator<Item = &usize>, side: usize| {
        let mut set = (0.0, 0.0);
        for &c in kids {
            set = joined(set, nodes[c].layout.margin_sets[side]);
            if !nodes[c].layout.through {
                break;
            }
        }
        set
    };
    if collapse_in[0] {
        sets[0] = joined(sets[0], chain(&mut n.children.iter(), 0));
    }
    if collapse_in[1] {
        sets[1] = joined(sets[1], chain(&mut n.children.iter().rev(), 1));
    }
    if through {
        sets = [joined(sets[0], sets[1]); 2];
    }
    nodes[i].layout.margin_sets = sets;
    nodes[i].layout.collapse_in = collapse_in;
    nodes[i].layout.through = through;
}

/// The room a column's margins and gaps take around its children: before
/// the first, between each pair, and after the last. A block's adjacent
/// margins collapse, through any child that collapses through itself; a
/// flex column's add.
pub(in crate::layout) fn column_spacing(nodes: &[Node], i: usize) -> (f32, Vec<f32>, f32) {
    let n = &nodes[i];
    let kids = &n.children;
    if kids.is_empty() {
        return (0.0, Vec::new(), 0.0);
    }
    // `run` is the margin set open after the last child, `used` the room
    // already given to it, and `outside` that the run collapses through
    // the column's own top edge.
    let mut run = (0.0, 0.0);
    let mut used = 0.0;
    let mut outside = n.layout.collapse_in[0];
    let mut before = Vec::with_capacity(kids.len());
    for (j, &c) in kids.iter().enumerate() {
        let k = &nodes[c];
        let gap = if j > 0 { n.layout.used.main_gap() } else { 0.0 };
        if n.layout.used.block {
            let set = joined(run, k.layout.margin_sets[0]);
            let total = if outside { 0.0 } else { span(set) };
            before.push(gap + total - used + k.layout.clearance);
            if k.layout.through {
                (run, used) = (set, total);
            } else {
                (run, used, outside) = (k.layout.margin_sets[1], 0.0, false);
            }
        } else {
            before.push(gap + span(run) + span(k.layout.margin_sets[0]));
            run = k.layout.margin_sets[1];
        }
    }
    let trail = if n.layout.collapse_in[1] {
        0.0
    } else if outside {
        // Every child collapsed through the top edge.
        0.0
    } else {
        span(run) - used
    };
    let lead = before.remove(0);
    (lead, before, trail)
}

pub(super) fn column_room(nodes: &[Node], i: usize) -> f32 {
    let (lead, between, trail) = column_spacing(nodes, i);
    lead + between.iter().sum::<f32>() + trail
}

/// The room a child's left and right margins take in its parent's
/// intrinsic width: their px (a percentage and `auto` count 0, as CSS
/// counts them in a min- or max-content contribution).
pub(in crate::layout) fn side_margins(n: &Node) -> f32 {
    n.layout.used.margin[1].intrinsic() + n.layout.used.margin[3].intrinsic()
}

/// A child's left and right margins against `basis`, with its `auto` ones
/// taking `free` (CSS 2 §10.3.3 in a block, CSS Flexbox §8.1 in a line):
/// split between two, all to one. Free space under 0 gives them none.
pub(in crate::layout) fn place_sides(n: &Node, basis: f32, free: f32) -> [f32; 2] {
    let [l, r] = [n.layout.used.margin[3], n.layout.used.margin[1]];
    let free = free.max(0.0);
    match (l.auto, r.auto) {
        (true, true) => [free / 2.0, free / 2.0],
        (true, false) => [free, r.of(basis)],
        (false, true) => [l.of(basis), free],
        (false, false) => [l.of(basis), r.of(basis)],
    }
}

/// The room a child's own vertical margins take across a row.
pub(super) fn cross_margins(n: &Node) -> f32 {
    span(n.layout.margin_sets[0]) + span(n.layout.margin_sets[1])
}

/// The room its own vertical margins take in a grid row.
pub(in crate::layout) fn cross_room(n: &Node) -> f32 {
    cross_margins(n)
}
