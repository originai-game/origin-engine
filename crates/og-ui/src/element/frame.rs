use super::*;

// ── The element model's part of the frame ──────────────────────────────────

use crate::layout::{Content, Float, NONE, Node, apply_patch, preorder, width_in};
use crate::ui::Ui;

/// Whether `path` is `decl` or inside it (an ancestor path covers its subtree).
fn covers(decl: &str, path: &str) -> bool {
    // `*`: over anything on purpose (a dragged card's targeting arc, a
    // ghost following the pointer across the screen).
    decl == "*"
        || path == decl
        || (path.starts_with(decl) && path.as_bytes().get(decl.len()) == Some(&b'/'))
}

/// It draws something of its own.
fn painted(n: &Node) -> bool {
    !n.placed.rect.is_empty()
        && (n.el.background.is_some()
            || !matches!(n.content, Content::None)
            || !matches!(
                n.widget,
                crate::layout::Widget::None | crate::layout::Widget::Spacer
            )
            || !n.shapes.is_empty())
}

impl crate::ui::Layout {
    /// Overlaps nothing declared (docs/ui.md §2): an absolute, anchored
    /// or floating-group element over another element's drawing, and
    /// siblings in the flow over each other.
    pub(crate) fn find_overlaps(
        &self,
        order: &[usize],
        env: &crate::ui::Env,
        paint: &crate::ui::Paint,
        tree: &crate::ui::Tree,
    ) -> Vec<Overlap> {
        const MAX: usize = 64;
        let mut out: Vec<Overlap> = Vec::new();
        let mut seen: std::collections::BTreeSet<(usize, usize)> = Default::default();
        let area = |r: &Rect| r.width > 0.5 && r.height > 0.5;
        // Each node's declarations, with its ancestors' (a declaration on a
        // placed group covers what is inside it).
        let declared = |a: usize, b_path: &str| -> bool {
            let mut j = a;
            while j != NONE {
                if tree.nodes[j]
                    .props
                    .overlaps
                    .iter()
                    .any(|d| covers(d, b_path))
                {
                    return true;
                }
                j = tree.nodes[j].parent;
            }
            false
        };
        // Where a node is drawn by its own and its ancestors' declared
        // transforms (a key cap scaled down to fit its corner covers its
        // scaled box, not the line it was laid out on); motion, which
        // comes later, is not part of the layout checked here.
        let posed = |i: usize| -> Rect {
            let mut m = Affine::IDENTITY;
            let mut j = i;
            while j != NONE {
                let n = &tree.nodes[j];
                let transform = if env.direction == og_i18n::Direction::Rtl {
                    n.props.transform.mirrored()
                } else {
                    n.props.transform
                };
                if !transform.is_identity() {
                    m = transform.matrix(&n.placed.rect).mul(&m);
                }
                j = n.parent;
            }
            if m.is_identity() {
                tree.nodes[i].placed.rect
            } else {
                m.map_rect(&tree.nodes[i].placed.rect)
            }
        };
        // What of a node a scroll leaves in view: its rectangle cut by every
        // scrolling ancestor's (an element scrolled out of a list overlaps
        // nothing drawn outside it), and by every ancestor that clips its
        // children to its shape (a map's view larger than its frame draws
        // only inside the frame).
        let clips = |j: usize| -> bool {
            paint.draw_of(j, tree).is_some_and(|d| {
                matches!(
                    d.clip,
                    crate::paint::Clip::Shape | crate::paint::Clip::Cut(_)
                )
            })
        };
        // Each node's is worked out once: the pairs below ask for the same
        // node's again and again (every sibling of a long list against every
        // other, every element of a placed group against every element), and
        // each answer walks its ancestors.
        let shown_at: std::cell::RefCell<Vec<Option<Rect>>> =
            std::cell::RefCell::new(vec![None; tree.nodes.len()]);
        let shown = |i: usize| -> Rect {
            if let Some(r) = shown_at.borrow()[i] {
                return r;
            }
            let mut r = posed(i);
            let mut j = tree.nodes[i].parent;
            while j != NONE {
                let n = &tree.nodes[j];
                if n.el.scroll || n.el.scroll_x || clips(j) {
                    r = r.intersect(&posed(j));
                }
                j = n.parent;
            }
            shown_at.borrow_mut()[i] = Some(r);
            r
        };
        // Drawn at all: an element its own or an ancestor's opacity takes to
        // nothing (a counter's invisible sizer, holding its widest value's
        // room) covers nothing.
        let seen_at: std::cell::RefCell<Vec<Option<bool>>> =
            std::cell::RefCell::new(vec![None; tree.nodes.len()]);
        let seen_at_all = |i: usize| -> bool {
            if let Some(s) = seen_at.borrow()[i] {
                return s;
            }
            let mut j = i;
            let mut s = true;
            while j != NONE {
                if tree.nodes[j].props.opacity <= 0.0 {
                    s = false;
                    break;
                }
                j = tree.nodes[j].parent;
            }
            seen_at.borrow_mut()[i] = Some(s);
            s
        };
        let ancestors = |mut j: usize| {
            let mut set = std::collections::BTreeSet::new();
            while j != NONE {
                set.insert(j);
                j = tree.nodes[j].parent;
            }
            set
        };
        // Layers meant to be over the rest (dialogs, menus, tips, toasts).
        let over_layers: std::collections::BTreeSet<usize> = tree
            .floating
            .iter()
            .copied()
            .filter(|&f| !matches!(tree.nodes[f].float, Float::Anchored { .. }))
            .flat_map(|f| preorder(&tree.nodes, f))
            .collect();
        let placed: Vec<usize> = order
            .iter()
            .copied()
            .filter(|&i| {
                i != 0
                    && !over_layers.contains(&i)
                    && (tree.nodes[i].props.out_of_flow()
                        || (tree.nodes[i].parent == NONE
                            && matches!(tree.nodes[i].float, Float::Anchored { .. })))
            })
            .collect();
        let others: Vec<usize> = order
            .iter()
            .copied()
            .filter(|&i| {
                i != 0 && !over_layers.contains(&i) && painted(&tree.nodes[i]) && seen_at_all(i)
            })
            .collect();
        for &p in &placed {
            let sub: std::collections::BTreeSet<usize> =
                preorder(&tree.nodes, p).into_iter().collect();
            let up = ancestors(p);
            for &a in sub
                .iter()
                .filter(|&&a| painted(&tree.nodes[a]) && seen_at_all(a))
            {
                for &o in &others {
                    if sub.contains(&o) || up.contains(&o) {
                        continue;
                    }
                    let r = shown(a).intersect(&shown(o));
                    if !area(&r) {
                        continue;
                    }
                    let pair = (a.min(o), a.max(o));
                    if !seen.insert(pair) || out.len() >= MAX {
                        continue;
                    }
                    let (pa, po) = (tree.path_of(a), tree.path_of(o));
                    if declared(a, &po) || declared(o, &pa) {
                        continue;
                    }
                    out.push(Overlap {
                        path: pa,
                        other: po,
                        rect: r,
                        in_flow: false,
                    });
                }
            }
        }
        // Siblings in the flow of a row or column (a stack overlays them on
        // purpose; a grid's cells are its own).
        for &i in order {
            let n = &tree.nodes[i];
            if n.props.layout != LayoutMode::Flex
                || n.children.len() < 2
                || over_layers.contains(&i)
            {
                continue;
            }
            // The pairs whose spans along the flow can meet, by a sweep
            // from the start edge (a long list's rows meet only their
            // neighbours, where a check of every pair is quadratic), then
            // in the children's order, as each pair is reported.
            let column = n.el.direction == Direction::Column;
            let span = |r: &Rect| {
                if column {
                    (r.y, r.y + r.height)
                } else {
                    (r.x, r.x + r.width)
                }
            };
            let mut by_start: Vec<(usize, (f32, f32))> = n
                .children
                .iter()
                .enumerate()
                .filter(|&(_, &c)| seen_at_all(c))
                .map(|(k, &c)| (k, span(&shown(c))))
                .collect();
            by_start.sort_by(|x, y| x.1.0.total_cmp(&y.1.0).then(x.0.cmp(&y.0)));
            let mut pairs: Vec<(usize, usize)> = Vec::new();
            for (s, &(ka, (_, end))) in by_start.iter().enumerate() {
                for &(kb, (start, _)) in &by_start[s + 1..] {
                    if start > end {
                        break;
                    }
                    pairs.push((ka.min(kb), ka.max(kb)));
                }
            }
            pairs.sort_unstable();
            for (ka, kb) in pairs {
                let (a, b) = (n.children[ka], n.children[kb]);
                {
                    let r = shown(a).intersect(&shown(b));
                    if !area(&r) || out.len() >= MAX {
                        continue;
                    }
                    let (pa, pb) = (tree.path_of(a), tree.path_of(b));
                    if declared(a, &pb) || declared(b, &pa) {
                        continue;
                    }
                    out.push(Overlap {
                        path: pb,
                        other: pa,
                        rect: r,
                        in_flow: true,
                    });
                }
            }
        }
        out
    }
}

impl crate::ui::Layout {
    /// Where node `i`'s first line of text sits (its baseline, CSS px), if
    /// it or its first descendant in the flow has text.
    fn first_baseline(
        &self,
        i: usize,
        env: &crate::ui::Env,
        tree: &crate::ui::Tree,
    ) -> Option<f32> {
        let n = &tree.nodes[i];
        match &n.content {
            Content::Text(tc) => {
                let line = n.layout.lines.first()?;
                let m = &env
                    .fonts
                    .for_styled(tc.font, tc.step.weight, false, &tc.text)
                    .0
                    .metrics;
                let lh = tc.line_height();
                let inner = n.placed.rect.inset(&n.el.padding);
                let block = lh * n.layout.lines.len() as f32;
                let top = inner.y + ((inner.height - block) / 2.0).max(0.0);
                Some(m.baseline(&line.text, top, lh, tc.step.size))
            }
            Content::Rich(_) => {
                let l = n.layout.rich_lines.first()?;
                // A first line floats moved down (`Node::wrap_lines`).
                let drop = n.layout.wrap_lines.first().map_or(0.0, |w| w[1]);
                Some(n.placed.rect.y + n.el.padding.top + drop + l.baseline)
            }
            _ => n
                .children
                .iter()
                .find_map(|&c| self.first_baseline(c, env, tree)),
        }
    }

    /// Rows with `Cross::Baseline`: each child moved down so their first
    /// baselines meet (per line of a wrapping row). A vertical element's
    /// rows are its lines down the page: their items move across, from
    /// its block-start side, so their central baselines meet.
    pub(crate) fn align_baselines(
        &mut self,
        order: &[usize],
        env: &crate::ui::Env,
        tree: &mut crate::ui::Tree,
    ) {
        use crate::layout::Cross;
        for &i in order {
            let n = &tree.nodes[i];
            if !crate::ui::visit::baseline_row(n)
                || n.el.direction != crate::layout::Direction::Row
                || n.props.layout != LayoutMode::Flex
                || n.children.is_empty()
            {
                continue;
            }
            let parent = n.el.cross;
            // Across a vertical line: +1 from a left block-start, -1 from
            // a right one (`Element::vertical`'s sides).
            let across =
                n.el.vertical
                    .filter(|s| s[0] % 2 == 1)
                    .map(|s| if s[0] == 3 { 1.0 } else { -1.0 });
            let kids = n.children.clone();
            let lines = if n.el.wrap && !n.layout.rows.is_empty() {
                n.layout.rows.clone()
            } else {
                vec![(0, kids.len())]
            };
            // A line's first-baseline items share their first baselines, and
            // its last-baseline ones their last (CSS Box Alignment §9.3).
            // A line whose line box holds a `vertical-align` was placed by
            // layout (`Node::inline_rows`), its baselines with it.
            let held: Vec<bool> = (0..lines.len())
                .map(|r| n.layout.inline_rows.get(r).copied().flatten().is_some())
                .collect();
            for (s, e, last) in lines
                .into_iter()
                .zip(held)
                .filter(|&(_, held)| !held)
                .flat_map(|((s, e), _)| [(s, e, false), (s, e, true)])
            {
                let want = if last {
                    Cross::LastBaseline
                } else {
                    Cross::Baseline
                };
                let bs: Vec<Option<f32>> = kids[s..e]
                    .iter()
                    .map(|&c| {
                        if tree.nodes[c].el.align_self.unwrap_or(parent) != want {
                            None
                        } else if let Some(dir) = across {
                            tree.central_baseline(c, last).map(|x| dir * x)
                        } else if last {
                            self.last_baseline(c, env, tree)
                        } else {
                            self.first_baseline(c, env, tree)
                        }
                    })
                    .collect();
                let Some(max) = bs.iter().flatten().copied().reduce(f32::max) else {
                    continue;
                };
                // Where it draws only: a clean row keeps its children's
                // laid-out places into the next frame, and a shift kept
                // there was made again on top of itself every frame (a
                // slider's label and its value in two faces, 0.38 px
                // apart, sank 14 px a second).
                for (&c, b) in kids[s..e].iter().zip(bs) {
                    if let Some(b) = b {
                        match across {
                            Some(dir) => tree.shift_subtree(c, dir * (max - b), 0.0, false),
                            None => tree.shift_subtree(c, 0.0, max - b, false),
                        }
                    }
                }
            }
        }
    }

    /// Anchored elements onto their targets' rectangles.
    pub(crate) fn place_anchored(
        &mut self,
        order: &[usize],
        out: &mut crate::ui::Out,
        tree: &mut crate::ui::Tree,
    ) {
        for &i in order {
            if tree.nodes[i].props.position != Position::Anchored {
                continue;
            }
            let p = tree.nodes[i].props.clone();
            let Some(t) = tree.node_at(&p.anchor_to, i) else {
                let at = tree.path_of(i);
                out.warnings.push(format!(
                    "'{at}' is anchored to '{}', which is not on screen: anchorTo takes a full path (Hud/Coins) or a sibling's id",
                    p.anchor_to
                ));
                continue;
            };
            // Its target's box before a right-to-left mirror, as laid out
            // this frame or kept from the last.
            let tr = tree.nodes[t].layout.laid_rect;
            let n = &tree.nodes[i];
            let (ax, ay) = p.anchor_at.fractions();
            let (ox, oy) = p.anchor_self.fractions();
            let x = tr.x + ax * tr.width - ox * n.placed.rect.width + p.insets.start.unwrap_or(0.0);
            let y = tr.y + ay * tr.height - oy * n.placed.rect.height + p.insets.top.unwrap_or(0.0);
            let (dx, dy) = (x - n.placed.rect.x, y - n.placed.rect.y);
            tree.shift_subtree(i, dx, dy, true);
        }
    }
}

impl crate::ui::Tree {
    /// The node a path names: a full path, else one relative to `from`'s parent.
    pub(crate) fn node_at(&self, path: &str, from: usize) -> Option<usize> {
        crate::ui::node_in(&self.index, &self.nodes, path, from)
    }
}

impl crate::ui::Layout {
    /// Applies each element's breakpoint and orientation variants (the
    /// container ones wait for its width, in layout).
    pub(crate) fn apply_variants(&mut self, env: &crate::ui::Env, tree: &mut crate::ui::Tree) {
        let vp = env.viewport;
        let bp = vp.breakpoint();
        let portrait = vp.height >= vp.width;
        for k in 0..tree.emitted.len() {
            let i = tree.emitted[k];
            if tree.nodes[i].props.variants.is_empty() {
                continue;
            }
            let vs = tree.nodes[i].props.variants.clone();
            for v in vs.iter().filter(|v| !v.when.container) {
                let q = &v.when;
                let ok = (q.breakpoints.is_empty() || q.breakpoints.contains(&bp))
                    && q.orientation
                        .is_none_or(|o| (o == Orientation::Portrait) == portrait)
                    && width_in(vp.width, q);
                if ok {
                    apply_patch(&mut tree.nodes[i], &v.set);
                }
            }
        }
    }

    /// Where node `i`'s last line of text sits, if it or its last
    /// descendant in the flow with text has one.
    fn last_baseline(&self, i: usize, env: &crate::ui::Env, tree: &crate::ui::Tree) -> Option<f32> {
        let n = &tree.nodes[i];
        match &n.content {
            Content::Text(tc) => {
                let line = n.layout.lines.last()?;
                let m = &env
                    .fonts
                    .for_styled(tc.font, tc.step.weight, false, &tc.text)
                    .0
                    .metrics;
                let lh = tc.line_height();
                let inner = n.placed.rect.inset(&n.el.padding);
                let block = lh * n.layout.lines.len() as f32;
                let top = inner.y + ((inner.height - block) / 2.0).max(0.0);
                let top = top + lh * (n.layout.lines.len() - 1) as f32;
                Some(m.baseline(&line.text, top, lh, tc.step.size))
            }
            Content::Rich(_) => {
                let (last, before) = n.layout.rich_lines.split_last()?;
                let above: f32 = before.iter().map(|l| l.height).sum();
                let drop = n.layout.wrap_lines.last().map_or(0.0, |w| w[1]);
                Some(n.placed.rect.y + n.el.padding.top + above + drop + last.baseline)
            }
            _ => n
                .children
                .iter()
                .rev()
                .find_map(|&c| self.last_baseline(c, env, tree)),
        }
    }
}

impl crate::ui::Tree {
    /// Moves node `i` and its subtree by `(dx, dy)` (with the clips its own
    /// scroll containers make). With `layout` its laid-out position moves
    /// too, which a clean subtree keeps into the next frame; without it only
    /// where it draws, so a move made again each frame does not add up.
    pub(crate) fn shift_subtree(&mut self, i: usize, dx: f32, dy: f32, layout: bool) {
        if dx == 0.0 && dy == 0.0 {
            return;
        }
        let sub = preorder(&self.nodes, i);
        let inside: std::collections::BTreeSet<usize> = sub.iter().copied().collect();
        for &j in &sub {
            let n = &mut self.nodes[j];
            if layout {
                n.layout.x += dx;
                n.layout.y += dy;
            }
            n.placed.rect.x += dx;
            n.placed.rect.y += dy;
            n.layout.laid_rect.x += dx;
            n.layout.laid_rect.y += dy;
            if n.placed.clip_src != NONE && inside.contains(&n.placed.clip_src) {
                n.placed.clip.x += dx;
                n.placed.clip.y += dy;
                n.layout.laid_clip.x += dx;
                n.layout.laid_clip.y += dy;
            }
        }
    }

    /// Where node `i`'s first (`last`) line of text has its central
    /// baseline across a vertical line: the middle of its first (last)
    /// column, physical x. A vertical line's dominant baseline is central
    /// (CSS Writing Modes 4 §4.2); a horizontal text in it (an orthogonal
    /// flow) has none.
    fn central_baseline(&self, i: usize, last: bool) -> Option<f32> {
        let n = &self.nodes[i];
        match &n.content {
            Content::Text(tc) => {
                let v = tc.vertical?;
                let k = if last {
                    n.layout.lines.len().checked_sub(1)?
                } else {
                    0
                };
                n.layout.lines.first()?;
                let lh = tc.line_height();
                let inner = n.placed.rect.inset(&n.el.padding);
                let off = ((inner.width - lh * n.layout.lines.len() as f32) / 2.0).max(0.0);
                Some(if v.mode.block_rl() {
                    inner.right() - off - (k as f32 + 0.5) * lh
                } else {
                    inner.x + off + (k as f32 + 0.5) * lh
                })
            }
            Content::None if last => n
                .children
                .iter()
                .rev()
                .find_map(|&c| self.central_baseline(c, last)),
            Content::None => n
                .children
                .iter()
                .find_map(|&c| self.central_baseline(c, last)),
            _ => None,
        }
    }
}

impl crate::ui::Layout {
    /// Which nodes can reuse layout, stopping propagation at unchanged fixed
    /// boxes, and the frame's counts: elements built and rebuilt.
    pub(crate) fn reuse_check(
        &mut self,
        env: &crate::ui::Env,
        out: &mut crate::ui::Out,
        docs: &crate::ui::Docs,
        tree: &mut crate::ui::Tree,
    ) {
        let k = &mut self.cache;
        let inv = &mut tree.invalidation;
        let nodes = &tree.nodes;
        let n = nodes.len();
        let sig = (env.theme, env.direction, env.viewport.dpr());
        let fresh = inv.epoch != inv.prev_epoch || inv.sig != Some(sig);
        inv.prev_epoch = inv.epoch;
        inv.sig = Some(sig);
        inv.fresh = fresh;
        k.laid_out = 0;
        k.full = fresh || inv.whole;
        k.unreach();
        // A retained node not written this frame keeps its own inputs.
        if k.full {
            k.clean = vec![!fresh; n];
            k.unclean.clear();
        } else {
            k.clean.resize(n, true);
            for i in std::mem::take(&mut k.unclean) {
                if let Some(c) = k.clean.get_mut(i) {
                    *c = true;
                }
            }
        }
        let mut written = vec![false; n];
        let mut rebuilt = 0u32;
        for &i in &tree.emitted {
            written[i] = true;
        }
        // What may lay out otherwise than last frame: the nodes written
        // again, the parents of written and removed ones, every layer and
        // scroll container, a container query's reader, and the subtrees
        // layout moves after it places them; with their ancestors. Every
        // other node keeps its layout, so the walk leaves it out.
        let roots: Vec<usize> = std::iter::once(0)
            .chain(tree.floating.iter().copied())
            .collect();
        if !k.full {
            let mut sources: Vec<usize> = Vec::with_capacity(tree.emitted.len() * 2);
            for &i in &tree.emitted {
                sources.push(i);
                sources.push(nodes[i].parent);
            }
            sources.extend(std::mem::take(&mut inv.rederive));
            sources.extend(roots.iter().copied());
            sources.extend(k.scrollers.iter().copied());
            for &i in &tree.emitted {
                let a = &nodes[i];
                let moved = a.props.position == Position::Anchored
                    || a.props.position == Position::Screen
                    || a.props.variants.iter().any(|v| v.when.container)
                    || (a.el.cross == crate::layout::Cross::Baseline
                        && a.el.direction == crate::layout::Direction::Row);
                if moved {
                    inv.forced.insert(i);
                } else {
                    inv.forced.remove(&i);
                }
            }
            let restored = std::mem::take(&mut inv.restored);
            for i in sources {
                if i < n {
                    k.reach(nodes, i, false);
                }
            }
            let forced: Vec<usize> = inv.forced.iter().copied().collect();
            for i in forced.into_iter().chain(restored) {
                if i < n && nodes[i].key != 0 {
                    k.reach(nodes, i, true);
                }
            }
        }
        // Subtrees laid out afresh (`LayoutCache::relayout`).
        let mut afresh = vec![false; if k.relayout.is_empty() { 0 } else { n }];
        for r in std::mem::take(&mut k.relayout) {
            if r >= n {
                continue;
            }
            k.reach(nodes, r, true);
            let mut stack = vec![r];
            while let Some(j) = stack.pop() {
                afresh[j] = true;
                stack.extend_from_slice(&nodes[j].ordered_children);
            }
        }
        let mut reflowed = Vec::new();
        // Children before parents: the reverse of a preorder of every tree.
        let mut order = Vec::new();
        for &r in &roots {
            order.extend(k.walk(nodes, r));
        }
        for &i in order.iter().rev() {
            let a = &nodes[i];
            let same_layout = if written[i] {
                let Some(b) = k.prev.get(&i) else {
                    rebuilt += 1;
                    k.clean[i] = false;
                    k.unclean.push(i);
                    continue;
                };
                // Its children's margins collapse through their edges only
                // in a block's flow: a change there is theirs too.
                if a.el.block != b.el.block || a.props.layout != b.props.layout {
                    reflowed.push(i);
                }
                let same_layout = crate::layout::el_layout_eq(&a.el, &b.el)
                    && crate::layout::content_layout_eq(&a.content, &b.content)
                    && crate::layout::props_layout_eq(&a.props, &b.props)
                    && a.shape_outside == b.shape_outside
                    && a.float == b.float
                    && a.float == Float::None
                    && a.children == b.children
                    && a.out == b.out
                    && a.floats == b.floats;
                let same = same_layout
                    && a.el == b.el
                    && a.content == b.content
                    && a.widget == b.widget
                    && a.shapes == b.shapes
                    && a.props == b.props;
                if !same {
                    rebuilt += 1;
                }
                same_layout
            } else {
                a.float == Float::None && !a.props.variants.iter().any(|v| v.when.container)
            };
            k.clean[i] = !fresh
                && same_layout
                && !afresh.get(i).copied().unwrap_or(false)
                && a.children.iter().chain(&a.out).chain(&a.floats).all(|&c| {
                    let child = &nodes[c];
                    // A box among floats changes what they shorten and
                    // move; its size alone is not all it gives its parent.
                    if child.layout.beside || !child.floats.is_empty() {
                        return k.clean[c];
                    }
                    // A fixed box still lays out its dirty contents, but
                    // those contents cannot change its parent's sizing.
                    k.clean[c]
                        || matches!(child.el.width, Size::Fixed(_))
                            && matches!(child.el.height, Size::Fixed(_))
                            // Nor can they through its edge, unless a
                            // child's top margin collapses through it.
                            && (written[c] || !child.layout.collapse_in[0])
                            && (!written[c]
                                || k.prev.get(&c).is_some_and(|old| {
                                    child.el.width == old.el.width
                                        && child.el.height == old.el.height
                                        && child.el.margin == old.el.margin
                                        && !old.layout.collapse_in[0]
                                        && child.props.place == old.props.place
                                        && child.props.position == old.props.position
                                        && child.props.insets == old.props.insets
                                        && child.props.view_place == old.props.view_place
                                        && child.float == old.float
                                        && child.shape_outside == old.shape_outside
                                }))
                });
            if !k.clean[i] {
                k.unclean.push(i);
            }
        }
        for i in reflowed {
            for &c in &nodes[i].children {
                if k.clean[c] {
                    k.clean[c] = false;
                    k.unclean.push(c);
                }
            }
        }
        for &i in &tree.emitted {
            if let Some(previous) = k.prev.get(&i) {
                tree.nodes[i].inherit_layout(previous, k.clean[i]);
            }
        }
        out.stats = LayoutStats {
            rebuilt,
            bindings: docs.host.retained.evaluated as u32,
            reconciled: docs.host.retained.reconciled as u32,
            ..LayoutStats::default()
        };
    }
}

impl Ui {
    /// Opens a container with the element model's props (docs/ui.md §2):
    /// its children follow until [`Ui::close`].
    pub fn open_with(&mut self, id: impl Into<crate::Id>, el: crate::Element, props: Props) {
        self.open(id, el);
        let i = self.tree.parent();
        self.tree.nodes[i].props = props;
    }

    /// Sets the props of the element built just before (any widget, or a
    /// container just closed). Answers false when there is none.
    pub fn set_props(&mut self, props: Props) -> bool {
        if let Some(node) = self.tree.nodes.get_mut(self.tree.last_closed)
            && let Some(part) = node.part_box.as_mut()
        {
            part.intrinsic_props = props.clone();
        }
        match self.props_mut() {
            Some(p) => {
                *p = props;
                true
            }
            None => false,
        }
    }

    /// The props of the element built just before, to change in place.
    pub fn props_mut(&mut self) -> Option<&mut Props> {
        let i = self.tree.last_closed;
        if i == NONE || i >= self.tree.nodes.len() {
            self.out.warnings.push(
                "props with no element before it: call ui.props() right after the element".into(),
            );
            return None;
        }
        Some(&mut self.tree.nodes[i].props)
    }
}
