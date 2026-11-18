use super::*;

/// `p`'s level of detail.
pub(super) fn lod_of(lod: &[f32], distance: f32) -> u32 {
    lod.iter().take_while(|&&d| distance >= d).count() as u32
}

/// The scale an anchor takes at `distance`.
fn scale_of(s: AnchorScale, distance: f32, ortho: bool) -> f32 {
    match s {
        AnchorScale::Constant => 1.0,
        AnchorScale::Distance { .. } if ortho => 1.0,
        AnchorScale::Distance {
            reference,
            min,
            max,
        } => (reference / distance.max(1e-3)).clamp(min.min(max), max.max(min)),
    }
}

/// Where a box of half size `half` stands clamped inside `area` along the
/// ray from `area`'s centre in `dir`.
fn clamp_to_edge(area: Rect, half: Vec2, dir: Vec2) -> Vec2 {
    let (cx, cy) = area.center();
    let hx = (area.width / 2.0 - half.x).max(0.0);
    let hy = (area.height / 2.0 - half.y).max(0.0);
    let tx = if dir.x.abs() > 1e-6 {
        hx / dir.x.abs()
    } else {
        f32::INFINITY
    };
    let ty = if dir.y.abs() > 1e-6 {
        hy / dir.y.abs()
    } else {
        f32::INFINITY
    };
    let t = tx.min(ty);
    Vec2::new(cx + dir.x * t, cy + dir.y * t)
}

/// The top left and bottom right, in layout px, of what node `i` draws:
/// its box and every shown box under it, out-of-flow ones included, each
/// mapped by its own transform and those between it and `i` (a targeted
/// nameplate's frame scaled up), as it is drawn.
fn drawn_box(nodes: &[crate::layout::Node], i: usize) -> (Vec2, Vec2) {
    let r = nodes[i].placed.rect;
    let (mut lo, mut hi) = (Vec2::new(r.x, r.y), Vec2::new(r.right(), r.bottom()));
    let kids = |n: &crate::layout::Node, m: Affine| {
        n.children
            .iter()
            .chain(&n.out)
            .map(|&j| (j, m))
            .collect::<Vec<_>>()
    };
    let mut stack = kids(&nodes[i], Affine::IDENTITY);
    while let Some((j, m)) = stack.pop() {
        let b = nodes[j].placed.rect;
        let t = nodes[j].style.sampled.transform;
        let m = if t.is_identity() {
            m
        } else {
            m.mul(&t.matrix(&b))
        };
        if !b.is_empty() {
            let b = m.map_rect(&b);
            lo = lo.min(Vec2::new(b.x, b.y));
            hi = hi.max(Vec2::new(b.right(), b.bottom()));
        }
        stack.extend(kids(&nodes[j], m));
    }
    (lo, hi)
}

/// The shown nodes under ground `i` that show text (a badge, a label), the
/// outermost of each branch with what is under it: the ground's labels
/// rather than its marks on the ground.
fn ground_labels(nodes: &[crate::layout::Node], i: usize) -> Vec<usize> {
    let mut out = Vec::new();
    let mut stack: Vec<usize> = nodes[i]
        .children
        .iter()
        .chain(&nodes[i].out)
        .copied()
        .collect();
    while let Some(j) = stack.pop() {
        let n = &nodes[j];
        if n.hidden {
            continue;
        }
        if matches!(
            &*n.content,
            crate::layout::Content::Text(_) | crate::layout::Content::Rich(_)
        ) {
            out.push(j);
        } else {
            stack.extend(n.children.iter().chain(&n.out).copied());
        }
    }
    out
}

/// What the grounds' text covers on the screen, og-ui px: a path's badge,
/// a placement's label. A ground lies pinned on the scene, so its text
/// is in place before any tag is decluttered, and the tags keep clear of
/// it (a prompt drawn over "Out of reach" hid it).
fn ground_text(
    world: &crate::world::WorldUi,
    nodes: &[crate::layout::Node],
    vp: Vec2,
) -> Vec<Rect> {
    let Some(view) = world.declared.view else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for p in world.declared.panels.iter().filter(|p| p.desc.projected) {
        let Some((t, rotation, size)) = p.desc.quad else {
            continue;
        };
        let Some((center, q)) = world.pose_of(t) else {
            continue;
        };
        let q = q * rotation;
        let r = nodes[p.node].placed.rect;
        // A layout point's screen point, through the plane (as drawn).
        let seen = |x: f32, y: f32| {
            let on = Vec3::new(
                (x - r.x) / r.width.max(1e-6) * size.x - size.x / 2.0,
                size.y / 2.0 - (y - r.y) / r.height.max(1e-6) * size.y,
                0.0,
            );
            let (sx, sy, _) = view.project(center + q * on, vp.x, vp.y)?;
            Some(Vec2::new(sx, sy))
        };
        let mut stack: Vec<usize> = nodes[p.node]
            .children
            .iter()
            .chain(&nodes[p.node].out)
            .copied()
            .collect();
        while let Some(j) = stack.pop() {
            let n = &nodes[j];
            stack.extend(n.children.iter().chain(&n.out).copied());
            let b = n.placed.rect;
            if n.probe_text.trim().is_empty() || b.is_empty() {
                continue;
            }
            let corners = [
                (b.x, b.y),
                (b.right(), b.y),
                (b.right(), b.bottom()),
                (b.x, b.bottom()),
            ]
            .map(|(x, y)| seen(x, y));
            if corners.iter().any(Option::is_none) {
                continue;
            }
            let c = corners.map(Option::unwrap);
            let (lo, hi) = c[1..]
                .iter()
                .fold((c[0], c[0]), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
            out.push(Rect::new(lo.x, lo.y, hi.x - lo.x, hi.y - lo.y));
        }
    }
    out
}

/// Collision counts over sorted candidate coordinates. In bounded sweeps,
/// each bit plane holds one binary digit of 64 independent counters: a range
/// update advances whole words rather than walking individual coordinates.
/// Large domains retain the logarithmic range-update tree.
enum Coverage {
    Bits {
        len: usize,
        blocks: Vec<CoverageBlock>,
        free_blocks: u64,
    },
    Tree(CoverageTree),
}

struct CoverageBlock {
    planes: [u64; 32],
    used: usize,
    free: u64,
}

fn low_bits(len: usize) -> u64 {
    if len == 64 {
        u64::MAX
    } else {
        (1u64 << len) - 1
    }
}

impl Coverage {
    fn new(len: usize) -> Self {
        if len > 4096 {
            return Self::Tree(CoverageTree::new(len));
        }
        let blocks = (0..len.div_ceil(64))
            .map(|b| CoverageBlock {
                planes: [0; 32],
                used: 1,
                free: low_bits((len - b * 64).min(64)),
            })
            .collect();
        Self::Bits {
            len,
            blocks,
            free_blocks: low_bits(len.div_ceil(64)),
        }
    }

    fn add(&mut self, range: (usize, usize), delta: i32) {
        let (blocks, free_blocks) = match self {
            Self::Bits {
                blocks,
                free_blocks,
                ..
            } => (blocks, free_blocks),
            Self::Tree(tree) => return tree.add(range, delta),
        };
        debug_assert!(delta == 1 || delta == -1);
        if range.0 >= range.1 {
            return;
        }
        let first = range.0 / 64;
        for (offset, block) in blocks[first..=(range.1 - 1) / 64].iter_mut().enumerate() {
            let b = first + offset;
            let mask =
                low_bits((range.1 - b * 64).min(64)) & !low_bits(range.0.saturating_sub(b * 64));
            let mut carry = mask;
            let mut plane = 0;
            while carry != 0 {
                let old = block.planes[plane];
                block.planes[plane] ^= carry;
                carry &= if delta > 0 { old } else { !old };
                plane += 1;
            }
            block.used = block.used.max(plane);
            if delta > 0 {
                block.free &= !mask;
            } else {
                let occupied = block.planes[..block.used].iter().fold(0, |a, p| a | p);
                block.free |= mask & !occupied;
            }
            if block.free == 0 {
                *free_blocks &= !(1u64 << b);
            } else {
                *free_blocks |= 1u64 << b;
            }
        }
    }

    fn first(&self, range: (usize, usize), reverse: bool) -> Option<usize> {
        let (len, blocks, free_blocks) = match self {
            Self::Bits {
                len,
                blocks,
                free_blocks,
            } => (*len, blocks, *free_blocks),
            Self::Tree(tree) => return tree.first(range, reverse),
        };
        let (lo, hi) = (range.0.min(len), range.1.min(len));
        if lo >= hi {
            return None;
        }
        let mut available = free_blocks & low_bits((hi - 1) / 64 + 1) & !low_bits(lo / 64);
        while available != 0 {
            let b = if reverse {
                63 - available.leading_zeros() as usize
            } else {
                available.trailing_zeros() as usize
            };
            let free = blocks[b].free
                & low_bits((hi - b * 64).min(64))
                & !low_bits(lo.saturating_sub(b * 64));
            if free != 0 {
                let bit = if reverse {
                    63 - free.leading_zeros()
                } else {
                    free.trailing_zeros()
                };
                return Some(b * 64 + bit as usize);
            }
            available &= !(1u64 << b);
        }
        None
    }

    fn gaps(&self, out: &mut Vec<(usize, usize)>) {
        let (blocks, mut available) = match self {
            Self::Bits {
                blocks,
                free_blocks,
                ..
            } => (blocks, *free_blocks),
            Self::Tree(tree) => return tree.gaps(out),
        };
        out.clear();
        while available != 0 {
            let b = available.trailing_zeros() as usize;
            let mut free = blocks[b].free;
            while free != 0 {
                let start = free.trailing_zeros() as usize;
                let end = start + (free >> start).trailing_ones() as usize;
                let span = (b * 64 + start, b * 64 + end);
                if let Some(last) = out.last_mut().filter(|last| last.1 == span.0) {
                    last.1 = span.1;
                } else {
                    out.push(span);
                }
                free &= !low_bits(end);
            }
            available &= available - 1;
        }
    }
}

struct CoverageTree {
    len: usize,
    minimum: Vec<i32>,
    maximum: Vec<i32>,
    lazy: Vec<i32>,
}

impl CoverageTree {
    fn new(len: usize) -> Self {
        Self {
            len,
            minimum: vec![0; len * 4],
            maximum: vec![0; len * 4],
            lazy: vec![0; len * 4],
        }
    }

    fn add(&mut self, range: (usize, usize), delta: i32) {
        fn apply(
            tree: &mut CoverageTree,
            node: usize,
            span: (usize, usize),
            range: (usize, usize),
            delta: i32,
        ) {
            if span.0 >= range.1 || span.1 <= range.0 {
                return;
            }
            if range.0 <= span.0 && span.1 <= range.1 {
                tree.minimum[node] += delta;
                tree.maximum[node] += delta;
                tree.lazy[node] += delta;
                return;
            }
            let mid = (span.0 + span.1) / 2;
            apply(tree, node * 2, (span.0, mid), range, delta);
            apply(tree, node * 2 + 1, (mid, span.1), range, delta);
            tree.minimum[node] =
                tree.lazy[node] + tree.minimum[node * 2].min(tree.minimum[node * 2 + 1]);
            tree.maximum[node] =
                tree.lazy[node] + tree.maximum[node * 2].max(tree.maximum[node * 2 + 1]);
        }
        apply(self, 1, (0, self.len), range, delta);
    }

    fn first(&self, range: (usize, usize), reverse: bool) -> Option<usize> {
        fn find(
            tree: &CoverageTree,
            node: usize,
            span: (usize, usize),
            range: (usize, usize),
            reverse: bool,
            inherited: i32,
        ) -> Option<usize> {
            if span.0 >= range.1 || span.1 <= range.0 || tree.minimum[node] + inherited > 0 {
                return None;
            }
            if span.1 - span.0 == 1 {
                return Some(span.0);
            }
            let mid = (span.0 + span.1) / 2;
            let mut children = [(node * 2, (span.0, mid)), (node * 2 + 1, (mid, span.1))];
            if reverse {
                children.swap(0, 1);
            }
            children.into_iter().find_map(|(child, span)| {
                find(
                    tree,
                    child,
                    span,
                    range,
                    reverse,
                    inherited + tree.lazy[node],
                )
            })
        }
        find(self, 1, (0, self.len), range, reverse, 0)
    }

    fn gaps(&self, out: &mut Vec<(usize, usize)>) {
        fn visit(
            tree: &CoverageTree,
            node: usize,
            span: (usize, usize),
            inherited: i32,
            out: &mut Vec<(usize, usize)>,
        ) {
            if tree.minimum[node] + inherited > 0 {
                return;
            }
            if tree.maximum[node] + inherited == 0 {
                if let Some(last) = out.last_mut().filter(|last| last.1 == span.0) {
                    last.1 = span.1;
                } else {
                    out.push(span);
                }
                return;
            }
            let mid = (span.0 + span.1) / 2;
            let inherited = inherited + tree.lazy[node];
            visit(tree, node * 2, (span.0, mid), inherited, out);
            visit(tree, node * 2 + 1, (mid, span.1), inherited, out);
        }
        out.clear();
        visit(self, 1, (0, self.len), 0, out);
    }
}

/// The spot nearest `b`'s own that is clear of `placed`, as the move
/// there, or `None`: nothing fits and it is hidden. As map labels are
/// placed (MapLibre's collision index): by rank, each in the first of its
/// candidate spots clear of those placed before it and inside the screen.
/// A spot is clear only against a side of a box near it, so the
/// candidates are `b` a `gap` off the sides of the boxes within `reach`,
/// on either axis or both. `Stack` first takes the nearest straight above,
/// then straight below (nameplates over a crowd); `Shift` and a stack with
/// no room either way take the nearest of all. A moved box stays inside
/// `inside` (the screen) and within `reach` of its spot.
fn declutter(
    mode: Declutter,
    b: Rect,
    placed: &[Rect],
    gap: f32,
    reach: f32,
    inside: Rect,
    available: Option<&[[f32; 4]]>,
) -> Option<Vec2> {
    // Clear: a gap from every placed box (touching is not clear).
    let clear = |r: &Rect| {
        let r = r.outset(gap - 0.01);
        !placed.iter().any(|p| p.overlaps(&r))
    };
    if clear(&b) {
        return Some(Vec2::ZERO);
    }
    if !matches!(mode, Declutter::Shift | Declutter::Stack) {
        return None;
    }
    let moved = |d: Vec2| Rect::new(b.x + d.x, b.y + d.y, b.width, b.height);
    let mut ys = Vec::with_capacity(placed.len() * 2 + 1);
    ys.push(0.0);
    let nearby = b.outset(reach);
    for p in placed.iter().filter(|p| p.overlaps(&nearby)) {
        ys.extend([p.y - gap - b.bottom(), p.bottom() + gap - b.y]);
    }
    // Keep the same coordinates, but sweep their free y intervals instead
    // of materialising and sorting the Cartesian product. Each obstacle
    // excludes one contiguous interval of sorted candidate ordinates.
    ys.retain(|&y| {
        Vec2::new(0.0, y).length() <= reach
            && b.y + y >= inside.y - 0.5
            && b.y + y + b.height <= inside.bottom() + 0.5
            && available
                .is_none_or(|holes| holes.iter().any(|h| b.y + y >= h[1] && b.y + y <= h[3]))
    });
    // Coordinates are deduplicated, and equal-start intervals commute:
    // these sorts have no meaningful tie order to preserve.
    ys.sort_unstable_by(f32::total_cmp);
    ys.dedup();
    if ys.is_empty() {
        return None;
    }
    let zero = ys.partition_point(|&y| y < 0.0);
    if mode == Declutter::Stack
        && 0.0 <= reach
        && b.x + 0.0 >= inside.x - 0.5
        && b.x + 0.0 + b.width <= inside.right() + 0.5
    {
        // Stacks usually have room on their own vertical. Test it before
        // building the two-dimensional collision index. A collision rules
        // out a whole run of ordinates, not just the one tested.
        let free = |at: usize| clear(&moved(Vec2::new(0.0, ys[at])));
        let mut end = zero;
        while end > 0 {
            let at = end - 1;
            let r = moved(Vec2::new(0.0, ys[at])).outset(gap - 0.01);
            if let Some(p) = placed.iter().find(|p| p.overlaps(&r)) {
                end = ys[..at].partition_point(|&y| {
                    moved(Vec2::new(0.0, y)).outset(gap - 0.01).bottom() <= p.y
                });
            } else {
                let length = Vec2::new(0.0, ys[at]).length();
                let lo = ys[..=at].partition_point(|&y| Vec2::new(0.0, y).length() > length);
                let at = (lo..=at).find(|&at| free(at)).unwrap();
                return Some(Vec2::new(0.0, ys[at]));
            }
        }
        let mut at = ys.partition_point(|&y| y <= 0.0);
        while at < ys.len() {
            let r = moved(Vec2::new(0.0, ys[at])).outset(gap - 0.01);
            if let Some(p) = placed.iter().find(|p| p.overlaps(&r)) {
                at += 1 + ys[at + 1..].partition_point(|&y| {
                    moved(Vec2::new(0.0, y)).outset(gap - 0.01).y < p.bottom()
                });
            } else {
                return Some(Vec2::new(0.0, ys[at]));
            }
        }
    }
    // A successful vertical stack needs no horizontal candidates. Keep every
    // original ordinate above: even a non-crossing obstacle can supply the
    // winning f32 coordinate.
    let mut xs = Vec::with_capacity(placed.len() * 2 + 1);
    xs.push(0.0);
    for p in placed.iter().filter(|p| p.overlaps(&nearby)) {
        xs.extend([p.x - gap - b.right(), p.right() + gap - b.x]);
    }
    xs.retain(|&x| {
        Vec2::new(x, 0.0).length() <= reach
            && b.x + x >= inside.x - 0.5
            && b.x + x + b.width <= inside.right() + 0.5
            && available
                .is_none_or(|holes| holes.iter().any(|h| b.x + x >= h[0] && b.x + x <= h[2]))
    });
    if xs.is_empty() {
        return None;
    }
    xs.sort_unstable_by(f32::total_cmp);
    xs.dedup();
    let leftmost = moved(Vec2::new(xs[0], 0.0)).outset(gap - 0.01).x;
    let rightmost = moved(Vec2::new(xs[xs.len() - 1], 0.0))
        .outset(gap - 0.01)
        .right();
    let edges: Vec<_> = ys
        .iter()
        .map(|&y| {
            let r = moved(Vec2::new(0.0, y)).outset(gap - 0.01);
            (r.y, r.bottom())
        })
        .collect();
    let topmost = edges[0].0;
    let bottommost = edges[edges.len() - 1].1;
    let x_edges: Vec<_> = xs
        .iter()
        .map(|&x| {
            let r = moved(Vec2::new(x, 0.0)).outset(gap - 0.01);
            (r.x, r.right())
        })
        .collect();
    // Horizontal bit runs make wide, shallow labels cheap to mark. Prepare
    // each obstacle's mask once; the row union is a contiguous bitwise loop.
    let words = xs.len().div_ceil(64);
    if ys.len().saturating_mul(words) <= 4096 {
        let mut blocked = vec![0u64; ys.len() * words];
        let mut mask = vec![0u64; words];
        for p in placed {
            if p.x >= rightmost
                || p.right() <= leftmost
                || p.y >= bottommost
                || p.bottom() <= topmost
            {
                continue;
            }
            let top = edges.partition_point(|&(_, bottom)| bottom <= p.y);
            let bottom = edges.partition_point(|&(top, _)| top < p.bottom());
            let left = x_edges.partition_point(|&(_, right)| right <= p.x);
            let right = x_edges.partition_point(|&(left, _)| left < p.right());
            if top >= bottom || left >= right {
                continue;
            }
            let first = left / 64;
            let last = (right - 1) / 64;
            let head = u64::MAX << (left % 64);
            let tail = u64::MAX >> (63 - (right - 1) % 64);
            mask.fill(0);
            if first == last {
                mask[first] = head & tail;
            } else {
                mask[first] = head;
                mask[first + 1..last].fill(u64::MAX);
                mask[last] = tail;
            }
            for row in blocked[top * words..bottom * words].chunks_exact_mut(words) {
                for (cell, bits) in row.iter_mut().zip(&mask) {
                    *cell |= bits;
                }
            }
        }
        let free = |row: &[u64], lo: usize, hi: usize, reverse: bool| -> Option<usize> {
            if lo >= hi {
                return None;
            }
            let mut words = lo / 64..=(hi - 1) / 64;
            while let Some(word) = if reverse {
                words.next_back()
            } else {
                words.next()
            } {
                let start = lo.saturating_sub(word * 64);
                let end = (hi - word * 64).min(64);
                let bits = !row[word] & (u64::MAX << start) & (u64::MAX >> (64 - end));
                if bits != 0 {
                    let bit = if reverse {
                        63 - bits.leading_zeros()
                    } else {
                        bits.trailing_zeros()
                    };
                    return Some(word * 64 + bit as usize);
                }
            }
            None
        };
        let order = |d: Vec2| (d.length(), d.x.abs(), d.y, d.x);
        let mut best: Option<Vec2> = None;
        let zero = xs.partition_point(|&x| x < 0.0);
        for (y, row) in ys.iter().copied().zip(blocked.chunks_exact(words)) {
            if best.is_some_and(|d| Vec2::new(0.0, y).length() > d.length()) {
                continue;
            }
            // At fixed y, the nearest free x on either side wins even when
            // lengths round equal: absolute x is the next ordering key.
            let left = free(row, 0, zero, true);
            let right = free(row, zero, xs.len(), false);
            for at in left.into_iter().chain(right) {
                let d = Vec2::new(xs[at], y);
                if d.length() <= reach && best.is_none_or(|old| order(d) < order(old)) {
                    best = Some(d);
                }
            }
        }
        return best;
    }
    let mut events = vec![Vec::new(); xs.len() + 1];
    for p in placed {
        if p.x >= rightmost || p.right() <= leftmost || p.y >= bottommost || p.bottom() <= topmost {
            continue;
        }
        let top = edges.partition_point(|&(_, bottom)| bottom <= p.y);
        let bottom = edges.partition_point(|&(top, _)| top < p.bottom());
        let left = x_edges.partition_point(|&(_, right)| right <= p.x);
        let right = x_edges.partition_point(|&(left, _)| left < p.right());
        if top < bottom && left < right {
            events[left].push((top, bottom, 1));
            events[right].push((top, bottom, -1));
        }
    }
    let mut coverage = Coverage::new(ys.len());
    let order = |d: Vec2| (d.length(), d.x.abs(), d.y, d.x);
    let mut best: Option<Vec2> = None;
    for (column, x) in xs.into_iter().enumerate() {
        for &(top, bottom, delta) in &events[column] {
            coverage.add((top, bottom), delta);
        }
        // The sweep still updates coverage where no y can improve best.
        if best.is_some_and(|d| Vec2::new(x, 0.0).length() > d.length()) {
            continue;
        }
        let above = coverage.first((0, zero), true).map(|at| {
            let length = Vec2::new(x, ys[at]).length();
            // Preserve the up-first tie when f32 lengths round equal,
            // even across separate free intervals.
            let lo = ys[..=at].partition_point(|&y| Vec2::new(x, y).length() > length);
            coverage.first((lo, at + 1), false).unwrap()
        });
        let below = coverage.first((zero, ys.len()), false);
        for at in above.into_iter().chain(below) {
            let d = Vec2::new(x, ys[at]);
            if d.length() <= reach && best.is_none_or(|old| order(d) < order(old)) {
                best = Some(d);
            }
        }
    }
    best
}

/// The most numbers a template lays apart on one tick; the rest stand on
/// their points.
const SPAWNED_PLACED: usize = 64;

/// A conservative continuous free-space certificate for one spawn batch.
/// Unlike a failed discrete search, an empty reachable region remains empty
/// when another obstacle is added. Possible positions still use `declutter`
/// unchanged, including its candidate order and floating-point ties.
struct Vacancies {
    holes: Vec<[f32; 4]>,
    scratch: Vec<[f32; 4]>,
    size: Vec2,
    pad: f32,
    active: bool,
}

impl Vacancies {
    fn new(pending: &[(usize, Rect)], taken: &[Rect], gap: f32) -> Self {
        let mut bounds = [
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
        ];
        let mut size = Vec2::splat(f32::INFINITY);
        let mut scale = gap.abs().max(1.0);
        for &(_, b) in pending {
            size = size.min(Vec2::new(b.width, b.height));
            scale = scale.max(b.width.abs()).max(b.height.abs());
            let reach = 3.0 * b.height.max(b.width.min(160.0));
            bounds[0] = bounds[0].min(b.x - reach);
            bounds[1] = bounds[1].min(b.y - reach);
            bounds[2] = bounds[2].max(b.x + reach);
            bounds[3] = bounds[3].max(b.y + reach);
        }
        for p in taken {
            scale = scale
                .max(p.x.abs())
                .max(p.y.abs())
                .max(p.width.abs())
                .max(p.height.abs());
        }
        for v in bounds {
            scale = scale.max(v.abs());
        }
        // Shrink blockers and enlarge query domains: roundoff can only leave
        // extra possible space, never turn a valid placement into a rejection.
        let pad = scale * f32::EPSILON * 32.0;
        bounds[0] -= pad;
        bounds[1] -= pad;
        bounds[2] += pad;
        bounds[3] += pad;
        let mut space = Self {
            holes: Vec::new(),
            scratch: Vec::new(),
            size,
            pad,
            active: true,
        };
        // Build the initial union by a sweep, rather than repeatedly splitting
        // every free rectangle for every older number in the pool.
        let mut xs = vec![bounds[0], bounds[2]];
        let covers: Vec<_> = taken
            .iter()
            .filter_map(|&p| {
                let c = space.cover(p, gap);
                let c = [
                    c[0].max(bounds[0]),
                    c[1].max(bounds[1]),
                    c[2].min(bounds[2]),
                    c[3].min(bounds[3]),
                ];
                if c[0] >= c[2] || c[1] >= c[3] {
                    None
                } else {
                    xs.extend([c[0], c[2]]);
                    Some(c)
                }
            })
            .collect();
        xs.sort_unstable_by(f32::total_cmp);
        xs.dedup();
        let mut events = Vec::with_capacity(covers.len() * 2 + 1);
        for c in covers {
            let range = (
                xs.partition_point(|&x| x < c[0]),
                xs.partition_point(|&x| x < c[2]),
            );
            events.extend([(c[1], range, 1), (c[3], range, -1)]);
        }
        events.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
        events.push((bounds[3], (0, 0), 0));
        let mut coverage = Coverage::new(xs.len() - 1);
        let mut last = bounds[1];
        let mut gaps = Vec::new();
        let mut previous: Vec<usize> = Vec::new();
        let mut next = Vec::new();
        for (y, range, delta) in events {
            if y > last {
                coverage.gaps(&mut gaps);
                next.clear();
                let mut old = 0;
                for &(a, b) in &gaps {
                    while old < previous.len() && space.holes[previous[old]][0] < xs[a] {
                        old += 1;
                    }
                    if old < previous.len()
                        && space.holes[previous[old]][0] == xs[a]
                        && space.holes[previous[old]][2] == xs[b]
                    {
                        let index = previous[old];
                        space.holes[index][3] = y;
                        next.push(index);
                    } else {
                        next.push(space.holes.len());
                        space.holes.push([xs[a], last, xs[b], y]);
                    }
                }
                std::mem::swap(&mut previous, &mut next);
                if space.holes.len() > 512 {
                    space.active = false;
                    break;
                }
                last = y;
            }
            if delta != 0 {
                coverage.add(range, delta);
            }
        }
        space
    }

    fn cover(&self, p: Rect, gap: f32) -> [f32; 4] {
        let g = gap - 0.01;
        [
            p.x - self.size.x - g + self.pad,
            p.y - self.size.y - g + self.pad,
            p.right() + g - self.pad,
            p.bottom() + g - self.pad,
        ]
    }

    fn block(&mut self, p: Rect, gap: f32) {
        if !self.active {
            return;
        }
        let cover = self.cover(p, gap);
        if cover[0] >= cover[2] || cover[1] >= cover[3] {
            return;
        }
        self.scratch.clear();
        for h in self.holes.drain(..) {
            let a = [
                h[0].max(cover[0]),
                h[1].max(cover[1]),
                h[2].min(cover[2]),
                h[3].min(cover[3]),
            ];
            if a[0] >= a[2] || a[1] >= a[3] {
                self.scratch.push(h);
                continue;
            }
            if h[0] < a[0] {
                self.scratch.push([h[0], h[1], a[0], h[3]]);
            }
            if a[2] < h[2] {
                self.scratch.push([a[2], h[1], h[2], h[3]]);
            }
            if h[1] < a[1] {
                self.scratch.push([a[0], h[1], a[2], a[1]]);
            }
            if a[3] < h[3] {
                self.scratch.push([a[0], a[3], a[2], h[3]]);
            }
        }
        // Bound certificate work in fragmented scenes; the exact search is
        // always available and has its own bounded bitset fast path.
        if self.scratch.len() > 512 {
            self.active = false;
        }
        std::mem::swap(&mut self.holes, &mut self.scratch);
    }

    fn reachable(&self, b: Rect, reach: f32, inside: Rect) -> Option<Vec<[f32; 4]>> {
        if !self.active {
            return None;
        }
        let mut reachable = Vec::new();
        for h in &self.holes {
            let x0 = h[0].max(inside.x - 0.5 - self.pad);
            let y0 = h[1].max(inside.y - 0.5 - self.pad);
            let x1 = h[2].min(inside.right() + 0.5 - b.width + self.pad);
            let y1 = h[3].min(inside.bottom() + 0.5 - b.height + self.pad);
            if x0 > x1 || y0 > y1 {
                continue;
            }
            let dx = b.x.clamp(x0, x1) - b.x;
            let dy = b.y.clamp(y0, y1) - b.y;
            if Vec2::new(dx, dy).length() <= reach + self.pad * 2.0 {
                reachable.push([x0, y0, x1, y1]);
            }
        }
        Some(reachable)
    }
}

impl crate::world::WorldUi {
    /// Lays the numbers and texts spawned in the world this tick (damage
    /// numbers) apart, as games stack a target's hits: each clear of the
    /// tags placed (`taken`, the screen boxes `world_anchors` returns), of
    /// those still rising and of those spawned before it (each over the
    /// path it rises along while it shows), at the nearest
    /// spot a stack takes (above, then below, then any way), moved once
    /// by its offset (`aux`) for its whole life. One with no room in reach
    /// is dropped rather than drawn over another, where neither reads.
    pub(super) fn place_spawned(&mut self, env: &crate::ui::Env, tick: u32, mut taken: Vec<Rect>) {
        use og_draw::UiMassKind;
        let Some(view) = self.declared.view else {
            return;
        };
        let vp = env.viewport;
        let unit = env.unit.max(1e-6);
        let safe = Rect::new(
            vp.safe_area.left,
            vp.safe_area.top,
            vp.width - vp.safe_area.horizontal(),
            vp.height - vp.safe_area.vertical(),
        );
        let gap = env.theme.space.xs;
        let metrics = &env.fonts.body.metrics;
        for (k, (_, t, space)) in self.declared.templates.iter().enumerate() {
            let words = matches!(t.kind, UiMassKind::Number { .. } | UiMassKind::Text { .. });
            if *space != UiMassSpace::World || t.lifetime == 0 || !words {
                continue;
            }
            let Some(pool) = self.state.pools.get_mut(&(k as u32)) else {
                continue;
            };
            // Timeline stops belong to the template, not each instance.
            let stops: Vec<_> = t
                .keys_drawn()
                .iter()
                .map(|k| k.t)
                .chain((1..=8).map(|k| k as f32 / 8.0))
                .filter_map(|age| {
                    let (offset, scale, alpha) = t.sample(age);
                    (alpha >= 0.5).then_some((age, offset, scale))
                })
                .collect();
            // Its remaining rise: project and measure once, then union the
            // current box with the still-visible future timeline samples.
            let swept = |i: &UiMassInstance, age: f32| -> Option<Rect> {
                let text = match &t.kind {
                    UiMassKind::Number {
                        decimals,
                        plus,
                        compact,
                    } => og_draw::format_number(i.value, *decimals, *plus, *compact),
                    UiMassKind::Text { strings } => strings.get(i.value.max(0.0) as usize)?.clone(),
                    _ => return None,
                };
                let (x, y, depth) = view.project(Vec3::from(i.position), vp.width, vp.height)?;
                let d = if t.distance.x > 0.0 {
                    (t.distance.x / depth.max(1e-4)).clamp(t.distance.y, t.distance.z)
                } else {
                    1.0
                };
                let width = metrics.measure(&text, 1.0);
                let boxed = |off: Vec2, scale: f32| {
                    let em = t.size.x * i.scale * scale.min(1.0) * d;
                    let w = width * em;
                    let at = Vec2::new(x, y) + (off + Vec2::new(i.aux[0], i.aux[1]) / unit) * d;
                    Rect::new(at.x - t.pivot.x * w, at.y - t.pivot.y * em, w, em)
                };
                let (off, scale, _) = t.sample(age);
                let mut r = boxed(off, scale);
                for &(_, off, scale) in stops.iter().filter(|&&(at, _, _)| at > age) {
                    let b = boxed(off, scale);
                    let (x, y) = (r.x.min(b.x), r.y.min(b.y));
                    r = Rect::new(
                        x,
                        y,
                        r.right().max(b.right()) - x,
                        r.bottom().max(b.bottom()) - y,
                    );
                }
                Some(r)
            };
            // Those of earlier ticks over the rest of their rise.
            for i in pool.iter().filter(|i| i.start != tick && i.color[3] > 0.0) {
                let age = tick.wrapping_sub(i.start) as f32 / t.lifetime as f32;
                taken.extend(swept(i, age));
            }
            let pending: Vec<_> = pool
                .iter()
                .enumerate()
                .filter(|(_, i)| i.start == tick)
                .take(SPAWNED_PLACED)
                .filter_map(|(index, i)| swept(i, 0.0).map(|b| (index, b)))
                .collect();
            if pending.is_empty() {
                continue;
            }
            let mut vacancies = Vacancies::new(&pending, &taken, gap);
            for (index, b) in pending {
                let i = &mut pool[index];
                let reach = 3.0 * b.height.max(b.width.min(160.0));
                let (x, y) = (safe.x.min(b.x), safe.y.min(b.y));
                let inside = Rect::new(
                    x,
                    y,
                    safe.right().max(b.right()) - x,
                    safe.bottom().max(b.bottom()) - y,
                );
                let available = vacancies.reachable(b, reach, inside);
                if available.as_ref().is_some_and(Vec::is_empty) {
                    i.color[3] = 0.0;
                    continue;
                }
                match declutter(
                    Declutter::Stack,
                    b,
                    &taken,
                    gap,
                    reach,
                    inside,
                    available.as_deref(),
                ) {
                    Some(d) => {
                        i.aux[0] += d.x * unit;
                        i.aux[1] += d.y * unit;
                        let placed = Rect::new(b.x + d.x, b.y + d.y, b.width, b.height);
                        taken.push(placed);
                        vacancies.block(placed, gap);
                    }
                    None => i.color[3] = 0.0,
                }
            }
        }
    }
}

impl Ui {
    pub(super) fn world_metas(&mut self) {
        let metas = std::mem::take(&mut self.world.declared.metas);
        let vp = self.env.viewport;
        let seconds = self.seconds() as f32;
        for (i, m) in &metas {
            let pulse = if m.pulse_hz > 0.0 {
                let s = og_core::math::sin(seconds * m.pulse_hz * std::f32::consts::TAU);
                1.0 - m.pulse_depth.clamp(0.0, 1.0) * (0.5 - 0.5 * s)
            } else {
                1.0
            };
            let alpha = (m.intensity * pulse).clamp(0.0, 1.0);
            let screen = Rect::new(0.0, 0.0, vp.width, vp.height);
            let cmds = meta_cmds(&m.kind, screen, alpha);
            let key = self.tree.nodes[*i].key;
            self.world.placement.marks.insert(
                key,
                Marks {
                    enter: cmds,
                    leave: Vec::new(),
                },
            );
        }
        self.world.declared.metas = metas;
    }
}

/// Only the pure placement solution is retained. Projection, drawn bounds,
/// priority order, pinned labels and fading are still evaluated every tick.
/// The key contains every solver input as bits, not a lossy hash.
#[derive(Clone, Debug, Default)]
pub(super) struct AnchorCache {
    key: Vec<u32>,
    moves: Vec<(Vec2, f32)>,
    taken: Vec<Rect>,
}

impl crate::ui::Layout {
    pub(super) fn world_anchors(
        &mut self,
        world: &mut crate::world::WorldUi,
        env: &crate::ui::Env,
        tree: &mut crate::ui::Tree,
    ) -> Vec<Rect> {
        let vp = env.viewport;
        let screen = Rect::new(0.0, 0.0, vp.width, vp.height);
        let safe = Rect::new(
            vp.safe_area.left,
            vp.safe_area.top,
            vp.width - vp.safe_area.horizontal(),
            vp.height - vp.safe_area.vertical(),
        );
        let anchors = std::mem::take(&mut world.declared.anchors);
        world.placement.placed.clear();
        // Where each lands before decluttering.
        struct Placed {
            entry: usize,
            key: u64,
            at: Vec2,
            pivot: (f32, f32),
            scale: f32,
            alpha: f32,
            depth: f32,
            info: AnchorInfo,
            /// What it draws on the screen (`drawn_box`), which declutters.
            drawn: Rect,
            arrow: Option<(f32, Srgba, f32)>,
        }
        let mut placed: Vec<Placed> = Vec::new();
        let view = world.declared.view;
        for (e, a) in anchors.iter().enumerate() {
            let n = &tree.nodes[a.node];
            let key = n.key;
            let r = n.placed.rect;
            let d = &a.desc;
            let mut info = AnchorInfo {
                scale: 1.0,
                ..AnchorInfo::default()
            };
            let (Some(view), Some((p, _))) = (view, world.pose_of(d.target)) else {
                placed.push(Placed {
                    entry: e,
                    key,
                    at: Vec2::ZERO,
                    pivot: d.pivot.fractions(),
                    scale: 1.0,
                    alpha: 0.0,
                    depth: 0.0,
                    info,
                    drawn: Rect::default(),
                    arrow: None,
                });
                continue;
            };
            let p = p + d.offset;
            let distance = (p - view.eye).length();
            info.distance = distance;
            info.lod = lod_of(&d.lod, distance);
            let mut alpha = 1.0;
            if d.max_distance > 0.0 {
                let fade = d.max_distance * 0.1;
                alpha = ((d.max_distance - distance) / fade.max(1e-3)).clamp(0.0, 1.0);
            }
            let scale = scale_of(d.scale, distance, view.orthographic);
            info.scale = scale;
            let projected = view.project(p, vp.width, vp.height);
            let half = Vec2::new(r.width, r.height) * scale / 2.0;
            let (lo, hi) = drawn_box(&tree.nodes, a.node);
            let mut pivot = d.pivot.fractions();
            let mut arrow = None;
            let (at, depth) = match projected {
                Some((x, y, depth)) => {
                    let at = Vec2::new(x, y) + d.screen_offset;
                    info.on_screen = screen.contains(x, y);
                    info.in_view = alpha > 0.0;
                    (at, depth)
                }
                None => {
                    info.in_view = false;
                    (Vec2::new(-1e5, -1e5), 0.0)
                }
            };
            let mut at = at;
            if let Some(c) = d.clamp {
                let area = Rect::new(
                    safe.x + c.margin,
                    safe.y + c.margin,
                    (safe.width - 2.0 * c.margin).max(0.0),
                    (safe.height - 2.0 * c.margin).max(0.0),
                );
                let inside = projected.is_some()
                    && at.x - half.x >= area.x
                    && at.x + half.x <= area.right()
                    && at.y - half.y >= area.y
                    && at.y + half.y <= area.bottom();
                if !inside {
                    let dir = view.direction(p, vp.width, vp.height);
                    let reach = c.arrow.map_or(0.0, |_| c.arrow_size);
                    at = clamp_to_edge(area, half + Vec2::splat(reach), dir);
                    pivot = (0.5, 0.5);
                    info.clamped = true;
                    info.angle = og_core::math::atan2(dir.y, dir.x);
                    let room =
                        (Vec2::new(area.width, area.height) / 2.0 - half - Vec2::splat(reach))
                            .max(Vec2::ZERO);
                    // The first edge the ray hits, not its dominant axis:
                    // a portrait surface can hit a side even on a steep ray.
                    info.edge = Some(if room.x * dir.y.abs() <= room.y * dir.x.abs() {
                        if dir.x < 0.0 { "left" } else { "right" }
                    } else if dir.y < 0.0 {
                        "top"
                    } else {
                        "bottom"
                    });
                    if let Some(color) = c.arrow {
                        arrow = Some((info.angle, color, c.arrow_size));
                    }
                }
                if alpha > 0.0 {
                    info.in_view = true;
                }
            } else if projected.is_none() {
                alpha = 0.0;
            } else if d.contain {
                // Slid inside the safe area less a gutter, so it never
                // touches the edge, and fading across a band at the edge:
                // whole while its point is a half width inside, half at
                // the edge, gone a half width past it.
                let (x, y) = (at.x - d.screen_offset.x, at.y - d.screen_offset.y);
                let inside_by = (x - safe.x)
                    .min(safe.right() - x)
                    .min(y - safe.y)
                    .min(safe.bottom() - y);
                let band = half.x.max(16.0);
                let k = ((inside_by + band) / (2.0 * band)).clamp(0.0, 1.0);
                if d.contain_fade {
                    alpha *= k * k * (3.0 - 2.0 * k);
                }
                let g = d.contain_margin.unwrap_or(env.theme.space.sm);
                let safe = Rect::new(
                    safe.x + g,
                    safe.y + g,
                    (safe.width - 2.0 * g).max(0.0),
                    (safe.height - 2.0 * g).max(0.0),
                );
                let b = Rect::new(
                    at.x + (lo.x - r.x - pivot.0 * r.width) * scale,
                    at.y + (lo.y - r.y - pivot.1 * r.height) * scale,
                    (hi.x - lo.x) * scale,
                    (hi.y - lo.y) * scale,
                );
                let into = |lo: f32, len: f32, min: f32, room: f32| {
                    if len >= room || lo < min {
                        min - lo
                    } else {
                        (min + room - (lo + len)).min(0.0)
                    }
                };
                at.x += into(b.x, b.width, safe.x, safe.width);
                at.y += into(b.y, b.height, safe.y, safe.height);
            } else {
                // Culled once its whole box is off the screen.
                let b = Rect::new(
                    at.x - pivot.0 * half.x * 2.0,
                    at.y - pivot.1 * half.y * 2.0,
                    half.x * 2.0,
                    half.y * 2.0,
                );
                if !b.overlaps(&screen) {
                    alpha = 0.0;
                }
            }
            let (px, py) = (r.x + pivot.0 * r.width, r.y + pivot.1 * r.height);
            let drawn = Rect::new(
                at.x + (lo.x - px) * scale,
                at.y + (lo.y - py) * scale,
                (hi.x - lo.x) * scale,
                (hi.y - lo.y) * scale,
            );
            placed.push(Placed {
                entry: e,
                key,
                at,
                pivot,
                scale,
                alpha,
                depth,
                info,
                drawn,
                arrow,
            });
        }
        // Declutter by priority, then nearness, then build order.
        let mut by_rank: Vec<usize> = (0..placed.len()).collect();
        by_rank.sort_by(|&x, &y| {
            let (a, b) = (&placed[x], &placed[y]);
            anchors[b.entry]
                .desc
                .priority
                .cmp(&anchors[a.entry].desc.priority)
                .then(a.info.distance.total_cmp(&b.info.distance))
                .then(x.cmp(&y))
        });
        let gap = env.theme.space.xs;
        // The grounds' text first: pinned, it never moves for a tag.
        let mut taken = ground_text(world, &tree.nodes, Vec2::new(vp.width, vp.height));
        let mut moves = vec![(Vec2::ZERO, 1.0f32); placed.len()];
        let mut requests = Vec::with_capacity(by_rank.len());
        let mut key = vec![gap.to_bits(), placed.len() as u32, taken.len() as u32];
        for r in &taken {
            key.extend([r.x, r.y, r.width, r.height].map(f32::to_bits));
        }
        for &k in &by_rank {
            let pl = &placed[k];
            if pl.alpha <= 0.0 || pl.drawn.is_empty() {
                continue;
            }
            let mode = anchors[pl.entry].desc.declutter;
            let a = &anchors[pl.entry].desc;
            // A move never takes it off the safe area, nor further off
            // than its own spot is (a tag at the edge stays where it is).
            let b = pl.drawn;
            let inside = if a.contain && a.clamp.is_none() {
                safe
            } else {
                let (x, y) = (safe.x.min(b.x), safe.y.min(b.y));
                Rect::new(
                    x,
                    y,
                    safe.right().max(b.right()) - x,
                    safe.bottom().max(b.bottom()) - y,
                )
            };
            key.extend([k as u32, mode as u32]);
            for r in [b, inside] {
                key.extend([r.x, r.y, r.width, r.height].map(f32::to_bits));
            }
            requests.push((k, mode, b, inside));
        }
        let cache = &mut world.placement.anchor_cache;
        if cache.key == key {
            moves.clone_from(&cache.moves);
            taken.clone_from(&cache.taken);
        } else {
            for (k, mode, b, inside) in requests {
                if mode == Declutter::None {
                    taken.push(b);
                    continue;
                }
                let reach = 3.0 * b.height.max(b.width.min(160.0));
                match declutter(mode, b, &taken, gap, reach, inside, None) {
                    Some(d) => {
                        taken.push(Rect::new(b.x + d.x, b.y + d.y, b.width, b.height));
                        moves[k] = (d, 1.0);
                    }
                    None => moves[k] = (Vec2::ZERO, 0.0),
                }
            }
            cache.key = key;
            cache.moves.clone_from(&moves);
            cache.taken.clone_from(&taken);
        }
        // Eased toward this tick's values (world state), applied.
        let mut seen = Vec::new();
        for (k, pl) in placed.iter().enumerate() {
            let d = &anchors[pl.entry].desc;
            let (shift, keep) = moves[k];
            let target_alpha = pl.alpha * keep;
            let fresh = !world.state.smooth.contains_key(&pl.key);
            let s = world.state.smooth.entry(pl.key).or_default();
            let step = if d.fade_ticks == 0 {
                1.0 / 6.0
            } else {
                1.0 / d.fade_ticks.clamp(1, 120) as f32
            };
            if fresh {
                s.dx = shift.x;
                s.dy = shift.y;
                s.alpha = 0.0;
            }
            s.alpha = if s.alpha < target_alpha {
                (s.alpha + step).min(target_alpha)
            } else {
                (s.alpha - step).max(target_alpha)
            };
            // Shifts glide (a third of the way a tick), so a crowd settles.
            s.dx += (shift.x - s.dx) * 0.34;
            s.dy += (shift.y - s.dy) * 0.34;
            if (shift.x - s.dx).abs() < 0.05 {
                s.dx = shift.x;
            }
            if (shift.y - s.dy).abs() < 0.05 {
                s.dy = shift.y;
            }
            let s = *s;
            seen.push(pl.key);
            let node = anchors[pl.entry].node;
            let r = tree.nodes[node].placed.rect;
            let px = r.x + pl.pivot.0 * r.width;
            let py = r.y + pl.pivot.1 * r.height;
            let at = pl.at + Vec2::new(s.dx, s.dy);
            let n = &mut tree.nodes[node];
            n.style.sampled.transform = Transform {
                x: at.x - px,
                y: at.y - py,
                scale_x: pl.scale,
                scale_y: pl.scale,
                pivot_x: pl.pivot.0,
                pivot_y: pl.pivot.1,
                ..n.style.sampled.transform
            };
            n.style.sampled.opacity *= s.alpha;
            if s.alpha < 0.05 {
                n.style.sampled.inert = true;
            }
            let mut info = pl.info;
            info.x = at.x;
            info.y = at.y;
            info.alpha = s.alpha;
            world.placement.last.insert(pl.key, info);
            // Bracketed by its key, so a frame drawn between ticks moves it
            // with its point (`UiFrame::emit_shifted`).
            let mut marks = Marks::default();
            marks
                .enter
                .push(DrawCmd::World(Box::new(WorldCmd::Anchor { key: pl.key })));
            world.placement.placed.push(super::AnchorPoint {
                key: pl.key,
                target: d.target,
                offset: d.offset,
                clamped: pl.info.clamped,
            });
            if let Some(view) = world.declared.view
                && d.occlusion != WorldOcclusion::None
                && !pl.info.clamped
                && pl.depth > 0.0
            {
                let _ = view;
                let alpha = match d.occlusion {
                    WorldOcclusion::Fade { alpha } => alpha,
                    _ => 0.0,
                };
                marks.enter.push(DrawCmd::World(Box::new(WorldCmd::Occlude {
                    at: (pl.at.x, pl.at.y),
                    depth: pl.depth,
                    alpha,
                })));
                marks
                    .leave
                    .push(DrawCmd::World(Box::new(WorldCmd::EndOcclude)));
            }
            if let Some((angle, color, size)) = pl.arrow
                && s.alpha > 0.0
            {
                // On the box's edge toward the point, pointing out.
                let half = Vec2::new(r.width, r.height) * pl.scale / 2.0;
                let dir = Vec2::new(og_core::math::cos(angle), og_core::math::sin(angle));
                let tx = if dir.x.abs() > 1e-6 {
                    half.x / dir.x.abs()
                } else {
                    f32::INFINITY
                };
                let ty = if dir.y.abs() > 1e-6 {
                    half.y / dir.y.abs()
                } else {
                    f32::INFINITY
                };
                let c = at + dir * (tx.min(ty) + size * 0.6);
                let mut shape = UiShape::rect(
                    [c.x - size / 2.0, c.y - size / 2.0, size, size],
                    0.0,
                    color.fade(s.alpha).linear(),
                );
                // A triangle whose first vertex points along `angle`.
                shape.geometry = UiGeometry::Polygon {
                    sides: 3,
                    inner: 1.0,
                    rounding: 1.5,
                    rotation: angle + std::f32::consts::FRAC_PI_2,
                };
                marks.leave.push(DrawCmd::Shape {
                    shape: Box::new(shape),
                    clip: screen,
                });
            }
            marks
                .leave
                .push(DrawCmd::World(Box::new(WorldCmd::EndAnchor)));
            world.placement.marks.insert(pl.key, marks);
        }
        world.state.smooth.retain(|k, _| seen.contains(k));
        world.placement.last.retain(|k, _| seen.contains(k));
        world.declared.anchors = anchors;
        taken
    }

    pub(super) fn world_panels(
        &mut self,
        ui_world: &mut crate::world::WorldUi,
        env: &crate::ui::Env,
        tree: &mut crate::ui::Tree,
    ) {
        let mut hits = Vec::new();
        let mut grounds = Vec::new();
        for p in &ui_world.declared.panels {
            let key = tree.nodes[p.node].key;
            if p.desc.projected {
                // Drawn by the UI onto where its quad is seen this tick: the
                // brackets are the same every tick (the quad goes in the
                // frame, `UiFrame::grounds`), so a moving camera relevels
                // and rebuilds nothing. Inert: no ray, no focus.
                tree.nodes[p.node].style.sampled.inert = true;
                let all = Rect::new(-1e9, -1e9, 2e9, 2e9);
                let begin = || DrawCmd::BeginLayer {
                    material: og_draw::UiMaterial::Plain,
                    params: [[0.0; 4]; og_draw::UI_MATERIAL_PARAMS],
                    mask: None,
                    expand: 0.0,
                    rect: Rect::default(),
                    filter: None,
                    role: Default::default(),
                };
                let end = |rect: Rect, ground| DrawCmd::EndLayer {
                    geometry: None,
                    rect,
                    opacity: 1.0,
                    blend: og_draw::UiBlend::Normal,
                    clip: all,
                    ground: Some(ground),
                };
                ui_world.placement.marks.insert(
                    key,
                    Marks {
                        enter: vec![begin()],
                        leave: vec![end(
                            tree.nodes[p.node].placed.rect,
                            crate::draw::Ground::Panel(key),
                        )],
                    },
                );
                // Its texts and badges: a layer each, laid with it over
                // every ground's other drawing (`Ground::Label`).
                for j in ground_labels(&tree.nodes, p.node) {
                    let m = ui_world
                        .placement
                        .marks
                        .entry(tree.nodes[j].key)
                        .or_default();
                    m.enter.insert(0, begin());
                    m.leave.push(end(
                        tree.nodes[j].placed.rect,
                        crate::draw::Ground::Label(key),
                    ));
                }
                if let (Some(view), Some((t, rotation, size))) =
                    (ui_world.declared.view, p.desc.quad)
                    && let Some((center, q)) = ui_world.pose_of(t)
                {
                    let (vp, unit) = (env.viewport, env.unit.max(1e-6));
                    let q = q * rotation;
                    let (hx, hy) = (size.x / 2.0, size.y / 2.0);
                    let r = tree.nodes[p.node].placed.rect;
                    let rect = Rect::new(r.x * unit, r.y * unit, r.width * unit, r.height * unit);
                    let world = |x: f32, y: f32, z: f32| center + q * Vec3::new(x, y, z);
                    let seen = |p: Vec3| {
                        let (sx, sy, _) = view.project(p, vp.width, vp.height)?;
                        Some(Vec2::new(sx, sy) * unit)
                    };
                    // Top left, top right, bottom right, bottom left; none
                    // when one is behind the eye (its projection would wrap).
                    let quad = [(-hx, hy), (hx, hy), (hx, -hy), (-hx, -hy)];
                    let corners: Option<Vec<Vec2>> =
                        quad.iter().map(|&(x, y)| seen(world(x, y, 0.0))).collect();
                    // A decal reaches a quarter of its shorter side off its
                    // plane: terrain under a 2 m ring rises and falls 0.5 m.
                    let reach = 0.25 * size.x.min(size.y);
                    let screen = Rect::new(0.0, 0.0, vp.width * unit, vp.height * unit);
                    // What it draws (its box and what overflows it, such as
                    // a badge past a path's end), on its plane.
                    let (lo, hi) = drawn_box(&tree.nodes, p.node);
                    let on_plane = |x: f32, y: f32| {
                        (
                            (x - r.x) / r.width.max(1e-6) * size.x - hx,
                            hy - (y - r.y) / r.height.max(1e-6) * size.y,
                        )
                    };
                    let drawn = [(lo.x, lo.y), (hi.x, lo.y), (hi.x, hi.y), (lo.x, hi.y)]
                        .map(|(x, y)| on_plane(x, y));
                    // What that covers on the screen, `reach` either side
                    // of the plane; all of it while a corner is behind the eye.
                    let boxed: Vec<Option<Vec2>> = drawn
                        .iter()
                        .flat_map(|&(x, y)| [world(x, y, -reach), world(x, y, reach)])
                        .map(seen)
                        .collect();
                    let bounds = if boxed.iter().all(Option::is_none) {
                        // Wholly behind the eye: nothing to draw.
                        Rect::default()
                    } else if boxed.iter().any(Option::is_none) {
                        screen
                    } else {
                        let b: Vec<Vec2> = boxed.into_iter().flatten().collect();
                        let (lo, hi) = b
                            .iter()
                            .fold((b[0], b[0]), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
                        Rect::new(lo.x, lo.y, hi.x - lo.x, hi.y - lo.y)
                            .outset(1.0)
                            .intersect(&screen)
                    };
                    let drape = og_draw::UiDrape::new(
                        view.view_proj,
                        view.eye,
                        view.forward,
                        Vec2::new(screen.width, screen.height),
                        [
                            world(-hx, hy, 0.0),
                            world(hx, hy, 0.0),
                            world(-hx, -hy, 0.0),
                        ],
                        [rect.x, rect.y, rect.width, rect.height],
                        reach,
                    );
                    if let Some(drape) = drape
                        && !bounds.is_empty()
                    {
                        grounds.push(crate::GroundQuad {
                            key,
                            rect,
                            corners: corners.map(|c| [c[0], c[1], c[2], c[3]]),
                            bounds,
                            drape,
                            context: 0,
                            depth: 0.0,
                        });
                    }
                }
                continue;
            }
            if let Some(target) = p.desc.target {
                ui_world.placement.marks.insert(
                    key,
                    Marks {
                        enter: vec![DrawCmd::World(Box::new(WorldCmd::BeginPanel {
                            target,
                            rect: tree.nodes[p.node].placed.rect,
                        }))],
                        leave: vec![DrawCmd::World(Box::new(WorldCmd::EndPanel))],
                    },
                );
            }
            if let Some((t, rotation, size)) = p.desc.quad
                && let Some((center, q)) = ui_world.pose_of(t)
            {
                hits.push(PanelHit {
                    key,
                    rect: tree.nodes[p.node].placed.rect,
                    center,
                    rotation: q * rotation,
                    size,
                });
            }
        }
        ui_world.placement.last_panels = hits;
        ui_world.placement.grounds = grounds;
    }

    /// The order with world anchors and meta layers drawn (and hit) under
    /// the screen's own elements: right after the root, before its children.
    pub(crate) fn world_order(
        &self,
        order: &mut Vec<usize>,
        world: &crate::world::WorldUi,
        tree: &crate::ui::Tree,
    ) {
        if order.is_empty() {
            return;
        }
        let under = |i: usize| -> bool {
            let n = &tree.nodes[i];
            n.float == Float::World && !world.is_panel_node(i)
        };
        // Subtrees of world layers below the root (each a run of the order).
        let mut moved = Vec::new();
        let mut rest = Vec::with_capacity(order.len());
        let mut k = 0;
        while k < order.len() {
            let i = order[k];
            if under(i) {
                let end = crate::layout::subtree_ends(&tree.nodes, &order[k..])[0] + k;
                moved.extend_from_slice(&order[k..end]);
                k = end;
            } else {
                rest.push(i);
                k += 1;
            }
        }
        if moved.is_empty() {
            return;
        }
        let mut out = Vec::with_capacity(order.len());
        out.push(rest[0]);
        out.extend(moved);
        out.extend_from_slice(&rest[1..]);
        *order = out;
    }
}

impl crate::world::WorldUi {
    /// What a node's commands are wrapped in (an occlusion test, a panel's
    /// target, an edge arrow, a meta layer's paint, particles).
    pub(crate) fn world_marks(&self, key: u64) -> Option<&Marks> {
        self.placement.marks.get(&key)
    }
}

/// A meta layer's commands over `screen` at `alpha`.
fn meta_cmds(kind: &MetaKind, screen: Rect, alpha: f32) -> Vec<DrawCmd> {
    let r = [screen.x, screen.y, screen.width, screen.height];
    match kind {
        MetaKind::Tint { color } => vec![DrawCmd::Shape {
            shape: Box::new(UiShape::rect(r, 0.0, color.fade(alpha).linear())),
            clip: screen,
        }],
        MetaKind::Vignette {
            color,
            inner,
            outer,
        } => {
            let c = color.linear();
            let mut shape = UiShape::rect(r, 0.0, c);
            // A radial gradient from the centre out to the corners.
            shape.paint = og_draw::UiPaint::Gradient(og_draw::UiGradient {
                kind: og_draw::UiGradientKind::Radial,
                from: Vec2::new(0.5, 0.5),
                to: Vec2::new(1.0, 1.0),
                stops: vec![
                    og_draw::UiStop {
                        offset: inner.clamp(0.0, 1.0),
                        color: [c[0], c[1], c[2], 0.0],
                    },
                    og_draw::UiStop {
                        offset: outer.clamp(0.0, 1.0).max(*inner + 1e-3),
                        color: [c[0], c[1], c[2], c[3]],
                    },
                ],
                ..Default::default()
            });
            shape.opacity = alpha;
            vec![DrawCmd::Shape {
                shape: Box::new(shape),
                clip: screen,
            }]
        }
        MetaKind::Material { material, params } => {
            // The frame under it as a backdrop, in a layer composited
            // through the material.
            let mut back = UiShape::rect(r, 0.0, [1.0; 4]);
            back.mode = og_draw::UiMode::Backdrop {
                blur: 0.0,
                saturation: 1.0,
                brightness: 1.0,
                glass: og_draw::UiGlass::default(),
            };
            // No tint over the scene: the backdrop alone.
            back.paint = og_draw::UiPaint::Solid([1.0, 1.0, 1.0, 0.0]);
            vec![
                DrawCmd::BeginLayer {
                    material: *material,
                    params: *params,
                    mask: None,
                    expand: 0.0,
                    rect: screen,
                    filter: None,
                    role: Default::default(),
                },
                DrawCmd::Shape {
                    shape: Box::new(back),
                    clip: screen,
                },
                DrawCmd::EndLayer {
                    geometry: None,
                    rect: screen,
                    opacity: alpha,
                    blend: og_draw::UiBlend::Normal,
                    clip: screen,
                    ground: None,
                },
            ]
        }
    }
}

/// Maps a point by an element's map (particles spawn where it is drawn).
pub(crate) fn mapped(m: &Affine, p: Vec2) -> Vec2 {
    let (x, y) = m.apply(p.x, p.y);
    Vec2::new(x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captured_world_placements_keep_their_exact_results() {
        struct Query {
            mass: bool,
            mode: Declutter,
            b: Rect,
            gap: f32,
            reach: f32,
            inside: Rect,
            placed: Vec<Rect>,
            expected: Option<[u32; 2]>,
        }
        fn word(bytes: &mut &[u8]) -> u32 {
            let n = u32::from_le_bytes(bytes[..4].try_into().unwrap());
            *bytes = &bytes[4..];
            n
        }
        fn number(bytes: &mut &[u8]) -> f32 {
            f32::from_bits(word(bytes))
        }
        fn rect(bytes: &mut &[u8]) -> Rect {
            Rect::new(number(bytes), number(bytes), number(bytes), number(bytes))
        }
        let decoded = miniz_oxide::inflate::decompress_to_vec_zlib(include_bytes!(
            "../../tests/fixtures/world-placement.zlib"
        ))
        .unwrap();
        let mut bytes = decoded.as_slice();
        let mut queries = Vec::new();
        while !bytes.is_empty() {
            let tag = bytes[0];
            bytes = &bytes[1..];
            let mode = [
                Declutter::None,
                Declutter::Shift,
                Declutter::Stack,
                Declutter::Hide,
            ][(tag & 15) as usize];
            let b = rect(&mut bytes);
            let gap = number(&mut bytes);
            let reach = number(&mut bytes);
            let inside = rect(&mut bytes);
            let placed: Vec<_> = (0..word(&mut bytes)).map(|_| rect(&mut bytes)).collect();
            let present = bytes[0];
            bytes = &bytes[1..];
            let expected = (present != 0).then(|| [word(&mut bytes), word(&mut bytes)]);
            let actual = declutter(mode, b, &placed, gap, reach, inside, None)
                .map(|v| [v.x.to_bits(), v.y.to_bits()]);
            assert_eq!(actual, expected, "captured query {}", queries.len() + 6000);
            queries.push(Query {
                mass: tag & 16 != 0,
                mode,
                b,
                gap,
                reach,
                inside,
                placed,
                expected,
            });
        }
        assert_eq!(queries.len(), 12000);
        // Replay append-only mass batches too: continuous certificates must
        // never reject a position the discrete solver would have accepted.
        let mut start = 0;
        while start < queries.len() {
            if !queries[start].mass {
                start += 1;
                continue;
            }
            let end = start + queries[start..].iter().take_while(|q| q.mass).count();
            let batch = &queries[start..end];
            let pending: Vec<_> = batch.iter().enumerate().map(|(i, q)| (i, q.b)).collect();
            let mut vacancies = Vacancies::new(&pending, &batch[0].placed, batch[0].gap);
            let mut previous: &[Rect] = &batch[0].placed;
            for q in batch {
                assert_eq!(q.mode, Declutter::Stack);
                assert!(q.placed.starts_with(previous));
                for &p in &q.placed[previous.len()..] {
                    vacancies.block(p, q.gap);
                }
                previous = &q.placed;
                let available = vacancies.reachable(q.b, q.reach, q.inside);
                let actual = if available.as_ref().is_some_and(Vec::is_empty) {
                    None
                } else {
                    declutter(
                        q.mode,
                        q.b,
                        &q.placed,
                        q.gap,
                        q.reach,
                        q.inside,
                        available.as_deref(),
                    )
                    .map(|v| [v.x.to_bits(), v.y.to_bits()])
                };
                assert_eq!(
                    actual,
                    q.expected,
                    "certified batch query at {b:?}",
                    b = q.b
                );
            }
            start = end;
        }
    }

    #[test]
    fn free_space_certificates_preserve_zero_gap_clearance() {
        let b = Rect::new(0.0, 0.0, 1.0, 1.0);
        let blockers = [
            Rect::new(-100.0, -100.0, 100.005, 200.0),
            Rect::new(0.995, -100.0, 100.0, 200.0),
            Rect::new(-100.0, -100.0, 200.0, 100.005),
            Rect::new(-100.0, 0.995, 200.0, 100.0),
        ];
        let mut vacancies = Vacancies::new(&[(0, b)], &blockers, 0.0);
        for p in blockers {
            vacancies.block(p, 0.0);
        }
        assert_eq!(
            declutter(Declutter::Stack, b, &blockers, 0.0, 3.0, b, None),
            Some(Vec2::ZERO)
        );
        assert!(!vacancies.reachable(b, 3.0, b).unwrap().is_empty());
    }

    #[test]
    fn edge_clamp_meets_the_side_its_direction_points_to() {
        let area = Rect::new(0.0, 0.0, 200.0, 100.0);
        // Right: x at the right edge less the half width.
        let p = clamp_to_edge(area, Vec2::new(10.0, 5.0), Vec2::new(1.0, 0.0));
        assert_eq!(p, Vec2::new(190.0, 50.0));
        // Steeply down-left: the bottom edge first.
        let d = Vec2::new(-0.2, 1.0).normalize();
        let p = clamp_to_edge(area, Vec2::new(10.0, 5.0), d);
        assert!(
            (p.y - 95.0).abs() < 1e-3 && p.x < 100.0 && p.x > 80.0,
            "{p:?}"
        );
    }

    const ANY: Rect = Rect {
        x: -1e6,
        y: -1e6,
        width: 2e6,
        height: 2e6,
    };

    #[test]
    fn declutter_shifts_the_short_way_and_stacks_when_boxed_in() {
        let a = Rect::new(0.0, 0.0, 100.0, 20.0);
        // Overlapping a by 4 px at its bottom: down by 4 + gap is shortest.
        let b = Rect::new(0.0, 16.0, 100.0, 20.0);
        assert_eq!(
            declutter(Declutter::Shift, b, &[a], 2.0, 100.0, ANY, None),
            Some(Vec2::new(0.0, 6.0))
        );
        // Stack: always above.
        assert_eq!(
            declutter(Declutter::Stack, b, &[a], 2.0, 100.0, ANY, None),
            Some(Vec2::new(0.0, -38.0))
        );
        assert_eq!(
            declutter(Declutter::Hide, b, &[a], 2.0, 100.0, ANY, None),
            None
        );
        // Touching a's bottom is not clear: it keeps a gap (a bubble's
        // tail on a cast bar).
        let t = Rect::new(0.0, 20.0, 100.0, 20.0);
        assert_eq!(
            declutter(Declutter::Shift, t, &[a], 2.0, 100.0, ANY, None),
            Some(Vec2::new(0.0, 2.0))
        );
        // Clear: no move.
        let c = Rect::new(0.0, 40.0, 100.0, 20.0);
        assert_eq!(
            declutter(Declutter::Hide, c, &[a], 2.0, 100.0, ANY, None),
            Some(Vec2::ZERO)
        );
        // Stacking up would leave the area: it stacks down under instead,
        // and with no room either way it is hidden.
        let top = Rect::new(0.0, 0.0, 100.0, 40.0);
        let a = Rect::new(0.0, 4.0, 100.0, 20.0);
        let b = Rect::new(0.0, 10.0, 100.0, 20.0);
        assert_eq!(
            declutter(
                Declutter::Stack,
                b,
                &[a],
                2.0,
                100.0,
                Rect::new(0.0, 0.0, 100.0, 100.0),
                None
            ),
            Some(Vec2::new(0.0, 16.0))
        );
        assert_eq!(
            declutter(Declutter::Stack, b, &[a], 2.0, 100.0, top, None),
            None
        );
        // With no room above or below but room beside, it goes beside.
        let wide = Rect::new(0.0, 0.0, 300.0, 40.0);
        assert_eq!(
            declutter(Declutter::Stack, b, &[a], 2.0, 120.0, wide, None),
            Some(Vec2::new(102.0, 0.0))
        );
    }

    #[test]
    fn a_point_behind_the_eye_points_the_way_to_turn() {
        let v = WorldView::look_at(
            Vec3::ZERO,
            Vec3::NEG_Z,
            Vec3::Y,
            60.0,
            16.0 / 9.0,
            0.1,
            100.0,
        );
        // Behind and to the right: not projected, and the arrow points right.
        let p = Vec3::new(3.0, 0.0, 5.0);
        assert!(v.project(p, 1280.0, 720.0).is_none());
        assert!(v.direction(p, 1280.0, 720.0).x > 0.9);
        // In front and up: y on the screen is up (negative).
        let q = Vec3::new(0.0, 3.0, -5.0);
        let (_, y, depth) = v.project(q, 1280.0, 720.0).unwrap();
        assert!(y < 360.0 && (depth - 5.0).abs() < 1e-4);
    }
}
