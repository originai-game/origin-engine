//! The frame's orders and the nodes that carry what a pass reads every
//! frame. Kept across frames and made again only where the trees changed
//! shape or a node was written, so a pass that reads one feature (a drag,
//! a match key, a focus group) reads the nodes that have it, never every
//! node.

use super::*;

/// What a node carries that a pass of the frame reads every frame.
pub(crate) mod feat {
    /// A row whose children share a baseline (`Ui::align_baselines`).
    pub const BASELINE: u32 = 1 << 0;
    /// Placed onto another node (`Position::Anchored`).
    pub const ANCHORED: u32 = 1 << 1;
    /// A staggered entrance over its leaves.
    pub const STAGGER: u32 = 1 << 2;
    /// A drag or a pinch.
    pub const GESTURE: u32 = 1 << 3;
    /// States, transitions, channels or layout motion.
    pub const ANIMATED: u32 = 1 << 4;
    /// A shared element's match key.
    pub const MATCH: u32 = 1 << 5;
    /// A customisable HUD element.
    pub const CUSTOM: u32 = 1 << 6;
    /// A drop target or a dragged element.
    pub const DND: u32 = 1 << 7;
    /// A scroll container.
    pub const SCROLLS: u32 = 1 << 8;
    /// A focus group.
    pub const GROUP: u32 = 1 << 9;
    /// A gesture recogniser.
    pub const RECOGNIZER: u32 = 1 << 10;
    /// Explicit focus moves.
    pub const NAV_LINKS: u32 = 1 << 11;
    /// A focus scope.
    pub const SCOPE: u32 = 1 << 12;
    /// A styled button part (`[narrow]` follows its width).
    pub const BUTTON_PART: u32 = 1 << 13;
    /// A part or a frame a style may dress (`Ui::build_part_draws`).
    pub const PART: u32 = 1 << 14;
    /// Laid out in the world (`Float::World`): placed every frame.
    pub const WORLD: u32 = 1 << 15;
    /// Rich text: painted afresh every frame.
    pub const RICH: u32 = 1 << 16;
    pub const COUNT: usize = 17;
}

/// The features of node `n`.
pub(crate) fn features(n: &Node) -> u32 {
    use crate::element::Position;
    let p = &n.props;
    let mut f = 0;
    if n.el.cross == crate::layout::Cross::Baseline
        && n.el.direction == crate::layout::Direction::Row
        && p.layout == crate::element::LayoutMode::Flex
    {
        f |= feat::BASELINE;
    }
    if p.position == Position::Anchored {
        f |= feat::ANCHORED;
    }
    if n.stagger.is_some() {
        f |= feat::STAGGER;
    }
    if p.drag.is_some() || p.pinch.is_some() {
        f |= feat::GESTURE;
    }
    if p.animated() || p.layout_motion.is_some() {
        f |= feat::ANIMATED;
    }
    if !p.match_key.is_empty() {
        f |= feat::MATCH;
    }
    if p.customize.is_some() {
        f |= feat::CUSTOM;
    }
    if p.drop.is_some() || p.drag.is_some() {
        f |= feat::DND;
    }
    if n.scrolls() {
        f |= feat::SCROLLS;
    }
    if p.focus_group.is_some() {
        f |= feat::GROUP;
    }
    if p.gestures.is_some_and(|g| g.any()) {
        f |= feat::RECOGNIZER;
    }
    if !p.nav.is_empty() {
        f |= feat::NAV_LINKS;
    }
    if n.scope {
        f |= feat::SCOPE;
    }
    if !matches!(n.widget, crate::layout::Widget::Button { .. })
        && n.el
            .part
            .is_some_and(|p| p.class == Some(crate::tokens::ElementClass::Button))
    {
        f |= feat::BUTTON_PART;
    }
    if n.el.part.is_some() || n.el.frame.is_some() {
        f |= feat::PART;
    }
    if n.float == Float::World {
        f |= feat::WORLD;
    }
    if matches!(n.content, Content::Rich(_)) {
        f |= feat::RICH;
    }
    f
}

/// The frame's orders (`Ui::orders`), with each feature's nodes.
#[derive(Clone, Debug, Default)]
pub(crate) struct Orders {
    /// The trees' shape (`TreeIndex::shape`) the preorder was made for.
    shape: Option<u64>,
    /// Every node in the trees, the screen's tree first and then each
    /// layer's, parents before children.
    pub pre: Vec<usize>,
    /// Per slot: its position in the preorder.
    pub pre_pos: Vec<u32>,
    /// The same nodes in drawing order (children by `z`, world layers
    /// under the screen's own), each subtree's end, and each slot's
    /// position (`u32::MAX`: not in a tree).
    pub draw: Vec<usize>,
    pub ends: Vec<usize>,
    pub pos: Vec<u32>,
    /// The trees' roots, in drawing order.
    pub roots: Vec<u64>,
    /// Per slot: the `z` the drawing order sorted it by, and the world
    /// panels it was made with.
    z: Vec<i32>,
    panels: Vec<usize>,
    draw_ok: bool,
    /// Per slot: its features (`feat`); and per feature its nodes, in
    /// preorder.
    pub feat: Vec<u32>,
    lists: Vec<Vec<usize>>,
    lists_ok: bool,
    /// Nodes the frame's own passes wrote (a stagger's leaves, a shake,
    /// a subtree moved onto its anchor, a world placement), this frame
    /// and last: their places and maps are made again.
    pub wrote: Vec<usize>,
    wrote_last: Vec<usize>,
    /// Per slot: the visit that last reached it.
    seen: Vec<u32>,
    stamp: u32,
    /// Per slot: the focus scope, scroll container and focus owner
    /// around it, and its focus group and gesture recogniser (0: none),
    /// kept where nothing above it changed.
    pub scope_of: Vec<u64>,
    pub scroller_of: Vec<u64>,
    pub owner_of: Vec<u64>,
    pub group_of: Vec<u64>,
    pub gesture_of: Vec<u64>,
    /// Per slot: whether its text can be selected (`user-select` as used:
    /// its own, else its parent's; never `Auto`).
    pub select_of: Vec<crate::element::UserSelect>,
    /// Per slot: its hits and the warning its place gives; the nodes
    /// with either, in drawing order.
    pub hit_of: Vec<Vec<Hit>>,
    pub warn_of: Vec<Option<String>>,
    pub hit_nodes: Vec<usize>,
    /// What the hits were last read with (`Ui::touch_min`, the screen).
    pub hit_inputs: Option<(f32, Rect)>,
    /// The drawing order was made again since the hits were listed.
    pub hits_moved: bool,
    /// Per slot: written this frame where stamped (`written_stamp`).
    pub written_at: Vec<u32>,
    pub written_stamp: u32,
    /// The nodes drawn again on their own every frame, and those a focus
    /// ring or outline is drawn over (`Ui::paint_reuse`); and the paint's
    /// verdicts, kept for their buffers.
    pub volatile: std::collections::BTreeSet<usize>,
    pub ringed: std::collections::BTreeSet<usize>,
    pub reuse: super::commands::Reuse,
}

impl Orders {
    /// The nodes that carry feature `f`, in preorder.
    pub(crate) fn with(&self, f: u32) -> &[usize] {
        &self.lists[f.trailing_zeros() as usize]
    }

    /// The nodes that carry feature `f`, in drawing order.
    pub(crate) fn drawn(&self, f: u32) -> Vec<usize> {
        let mut out = self.with(f).to_vec();
        out.sort_unstable_by_key(|&i| self.pos[i]);
        out
    }
}

impl Ui {
    /// Brings the preorder and the features up to date with the trees:
    /// made again where their shape changed, a feature read again on the
    /// nodes written or laid out this frame.
    pub(super) fn update_orders(&mut self, whole: bool) {
        let n = self.nodes.len();
        let o = &mut self.orders;
        if o.feat.len() < n {
            o.feat.resize(n, 0);
            o.z.resize(n, 0);
            o.seen.resize(n, 0);
            o.scope_of.resize(n, 0);
            o.scroller_of.resize(n, 0);
            o.owner_of.resize(n, 0);
            o.group_of.resize(n, 0);
            o.gesture_of.resize(n, 0);
            o.select_of.resize(n, crate::element::UserSelect::None);
            o.hit_of.resize(n, Vec::new());
            o.warn_of.resize(n, None);
            o.written_at.resize(n, 0);
        }
        o.written_stamp = o.written_stamp.wrapping_add(1);
        if o.written_stamp == 0 {
            o.written_at.fill(0);
            o.written_stamp = 1;
        }
        for &i in &self.emitted {
            if i < n {
                o.written_at[i] = o.written_stamp;
            }
        }
        o.wrote_last = std::mem::take(&mut o.wrote);
        let reshaped = whole || o.shape != Some(self.tree.shape);
        if reshaped {
            o.shape = Some(self.tree.shape);
            o.pre = crate::layout::preorder(&self.nodes, 0);
            for &f in &self.floating {
                o.pre.extend(crate::layout::preorder(&self.nodes, f));
            }
            o.pre_pos.clear();
            o.pre_pos.resize(n, u32::MAX);
            for (k, &i) in o.pre.iter().enumerate() {
                o.pre_pos[i] = k as u32;
            }
            o.draw_ok = false;
            o.lists_ok = false;
            if whole {
                for &i in &o.pre {
                    o.feat[i] = features(&self.nodes[i]);
                }
            }
        }
        let read = self
            .emitted
            .iter()
            .chain(&self.kept.laid)
            .chain(&self.kept.restored);
        for &i in read {
            if i < n {
                let f = features(&self.nodes[i]);
                if f != o.feat[i] {
                    o.feat[i] = f;
                    o.lists_ok = false;
                }
            }
        }
        if reshaped && !whole {
            // A node that entered a tree reads its features as it is.
            for &i in &o.pre {
                o.feat[i] = features(&self.nodes[i]);
            }
        }
        if !o.lists_ok {
            o.lists_ok = true;
            o.lists = vec![Vec::new(); feat::COUNT];
            for &i in &o.pre {
                let mut f = o.feat[i];
                while f != 0 {
                    let b = f.trailing_zeros() as usize;
                    o.lists[b].push(i);
                    f &= f - 1;
                }
            }
        }
    }

    /// What this frame may change, before its passes: the nodes written,
    /// laid out or put back, those that entered a tree or whose parent's
    /// children changed, those the frame's own passes wrote last frame,
    /// those still moving after last frame's motion, and those a pass
    /// moves every frame (drags, shared elements, world anchors, layers);
    /// and of them, those whose style may change (`restyle`): sampled
    /// again and moved by motion. A node only laid out or moved among its
    /// siblings keeps its sampled style (made from what was written).
    pub(super) fn frame_seeds(&self) -> (Vec<usize>, Vec<usize>) {
        let o = &self.orders;
        let mut restyle: Vec<usize> = self
            .emitted
            .iter()
            .chain(&o.wrote_last)
            .chain(&self.kept.motion_moving)
            .chain(o.with(feat::GESTURE))
            .chain(o.with(feat::MATCH))
            .chain(o.with(feat::WORLD))
            .copied()
            .collect();
        for &k in &self.tree.moved_keys {
            restyle.extend(self.tree.by_key.get(&k).copied());
        }
        let mut seeds = restyle.clone();
        seeds.extend(self.kept.laid.iter().chain(&self.floating));
        for &p in &self.tree.restructured {
            seeds.extend_from_slice(&self.nodes[p].ordered_children);
        }
        for list in [&mut restyle, &mut seeds] {
            list.retain(|&i| i < self.nodes.len() && self.tree.has(i));
            list.sort_unstable();
            list.dedup();
        }
        (seeds, restyle)
    }

    /// Visits the nodes from `seeds` down in drawing order: `step` makes
    /// a node's frame state again and answers whether its children's
    /// changes with it. `None` visits every node in `order`.
    pub(super) fn visit(
        &mut self,
        order: &[usize],
        seeds: Option<&[usize]>,
        mut step: impl FnMut(&mut Ui, usize) -> bool,
    ) -> Vec<usize> {
        let Some(seeds) = seeds else {
            for &i in order {
                step(self, i);
            }
            return order.to_vec();
        };
        let o = &mut self.orders;
        o.stamp = o.stamp.wrapping_add(1);
        if o.stamp == 0 {
            o.seen.fill(0);
            o.stamp = 1;
        }
        let stamp = o.stamp;
        let mut heap: std::collections::BinaryHeap<std::cmp::Reverse<(u32, usize)>> = seeds
            .iter()
            .filter(|&&i| o.pos[i] != u32::MAX)
            .map(|&i| std::cmp::Reverse((o.pos[i], i)))
            .collect();
        let mut out = Vec::new();
        while let Some(std::cmp::Reverse((_, i))) = heap.pop() {
            if self.orders.seen[i] == stamp {
                continue;
            }
            self.orders.seen[i] = stamp;
            out.push(i);
            if step(self, i) {
                for k in 0..self.nodes[i].ordered_children.len() {
                    let c = self.nodes[i].ordered_children[k];
                    let p = self.orders.pos[c];
                    if p != u32::MAX && self.orders.seen[c] != stamp {
                        heap.push(std::cmp::Reverse((p, c)));
                    }
                }
            }
        }
        out
    }

    /// Node `i`'s scope, scroller and focus owner from its parent's, its
    /// focus group and gesture recogniser from its parent's (a layer's
    /// anchor's); answers whether any changed.
    pub(super) fn scope_step(&mut self, i: usize) -> bool {
        let nodes = &self.nodes;
        let o = &self.orders;
        let n = &nodes[i];
        let up = n.parent;
        let (ps, pr) = if up == NONE {
            (0, 0)
        } else {
            (o.scope_of[up], o.scroller_of[up])
        };
        let owner = match up {
            NONE => 0,
            u if nodes[u].nav && !nodes[u].inert => nodes[u].key,
            u => o.owner_of[u],
        };
        let scope = if n.scope { n.key } else { ps };
        let scroller = if up != NONE && nodes[up].scrolls() {
            nodes[up].key
        } else {
            pr
        };
        // A layer takes its anchor's group and recogniser where the
        // anchor is drawn before it (one drawn after has none yet).
        let up = if n.parent == NONE { n.anchor } else { n.parent };
        let (pg, pr) = if up == NONE || up >= nodes.len() || o.pos[up] > o.pos[i] {
            (0, 0)
        } else {
            (o.group_of[up], o.gesture_of[up])
        };
        let group = if n.props.focus_group.is_some() {
            n.key
        } else {
            pg
        };
        let gesture = match n.props.gestures.filter(|g| g.any()) {
            Some(_) if !n.inert => n.key,
            _ => pr,
        };
        // `user-select`: its own, else its parent's; a control's subtree
        // (a button's label) is pressed, not selected, unless it says so.
        let select = {
            use crate::element::UserSelect;
            let ps = if up == NONE {
                UserSelect::None
            } else {
                o.select_of[up]
            };
            match n.props.select {
                UserSelect::Auto if n.el.interactive || n.focusable => UserSelect::None,
                UserSelect::Auto => ps,
                s => s,
            }
        };
        let o = &mut self.orders;
        let old = (
            o.scope_of[i],
            o.scroller_of[i],
            o.owner_of[i],
            o.group_of[i],
            o.gesture_of[i],
            o.select_of[i],
        );
        let new = (scope, scroller, owner, group, gesture, select);
        (
            o.scope_of[i],
            o.scroller_of[i],
            o.owner_of[i],
            o.group_of[i],
            o.gesture_of[i],
            o.select_of[i],
        ) = new;
        old != new
    }

    /// Node `i`'s hits (none for the screen's root) and the warning its
    /// place gives, from its frame state; answers whether they changed.
    pub(super) fn hit_step(&mut self, i: usize, touch_min: f32, screen: Rect) -> bool {
        let o = &self.orders;
        let mut warn = None;
        let mut hits = Vec::new();
        'node: {
            if i == 0 {
                break 'node;
            }
            let n = &self.nodes[i];
            let visible = n.screen.intersect(&n.clip);
            // Scrolled or clipped out of view: not on screen, not listed. A
            // zero-size element (an empty spacer) is listed where it sits.
            let (r, c) = (&n.screen, &n.clip);
            let outside =
                r.x > c.right() || r.right() < c.x || r.y > c.bottom() || r.bottom() < c.y;
            // A control laid out past the screen's edge with no scroller to
            // bring it back (a menu taller than a landscape phone): the
            // player cannot reach it, so the frame says which and by how much.
            if n.el.interactive
                && o.scroller_of[i] == 0
                && !r.is_empty()
                && !n.inert
                && !crate::world::off_screen(&n.rect)
            {
                // Where layout put it: a transform's move (a card thrown
                // off by a swipe) is not a layout that does not fit.
                let r = &n.rect;
                let cut = (screen.x - r.x)
                    .max(r.right() - screen.right())
                    .max(screen.y - r.y)
                    .max(r.bottom() - screen.bottom());
                if cut > 1.0 {
                    warn = Some(format!(
                        "'{}' is laid out {:.0} px past the {}x{} screen and nothing scrolls to it: fit the screen (a smaller type step, fewer rows, a row instead of a column) or put it in a scroll container",
                        self.path_of(i),
                        cut,
                        screen.width,
                        screen.height
                    ));
                }
            }
            let mut flags = 0u16;
            if n.scrolls() && !n.el.interactive {
                flags |= HIT_SCROLL;
            }
            if n.el.interactive
                || n.backdrop
                || n.props.drag.is_some()
                || n.props.pinch.is_some()
                || n.props.gestures.is_some_and(|g| g.any())
                || n.props.pointer
            {
                flags |= HIT_POINTER;
            }
            if n.props.drop.is_some() {
                flags |= HIT_DROP;
            }
            if n.focusable {
                flags |= HIT_TEXT;
            }
            if n.multiline {
                flags |= HIT_MULTILINE;
            }
            if n.nav {
                flags |= HIT_NAV;
            }
            if matches!(n.content, Content::Text(_))
                && !matches!(n.widget, Widget::Field(_))
                && matches!(
                    o.select_of[i],
                    crate::element::UserSelect::Text | crate::element::UserSelect::All
                )
            {
                flags |= HIT_SELECT;
            }
            if n.adjust {
                flags |= HIT_ADJUST;
            }
            flags |= u16::from(n.claim) << crate::ui::HIT_CLAIM_SHIFT;
            if n.scope {
                flags |= HIT_SCOPE;
            }
            if flags == 0 && n.links.is_empty() {
                break 'node;
            }
            // Its reach past its box: its hit slop, and on a touch screen a
            // control's box grown to the minimum touch target (§15.2).
            let slop = n.props.hit_slop.max(0.0);
            let control = flags & (HIT_POINTER | HIT_TEXT) != 0 && !n.backdrop;
            let (sx, sy) = if control && touch_min > 0.0 {
                (
                    slop.max((touch_min - n.rect.width) / 2.0),
                    slop.max((touch_min - n.rect.height) / 2.0),
                )
            } else {
                (slop, slop)
            };
            let reach = Rect::new(
                n.rect.x - sx,
                n.rect.y - sy,
                n.rect.width + 2.0 * sx,
                n.rect.height + 2.0 * sy,
            );
            let reaches = sx > 0.0 || sy > 0.0;
            let transformed = !n.world.is_identity();
            let inv = if transformed {
                n.world.invert().unwrap_or(Affine::IDENTITY)
            } else {
                Affine::IDENTITY
            };
            let hit = |rect: Rect, flags: u16| {
                let rect = if reaches {
                    n.world.map_rect(&reach).intersect(&n.clip)
                } else {
                    rect
                };
                Hit {
                    key: n.key,
                    rect,
                    full: *r,
                    flags,
                    scope: o.scope_of[i],
                    scroller: o.scroller_of[i],
                    owner: o.owner_of[i],
                    inv,
                    local: reach,
                    clip: n.clip,
                    group: o.group_of[i],
                    gesture: o.gesture_of[i],
                }
            };
            if outside || (visible.is_empty() && !r.is_empty()) {
                // Navigation can still reach a control just out of a list's view.
                if n.nav && !n.inert && o.scroller_of[i] != 0 {
                    hits.push(hit(*r, (flags & (HIT_NAV | HIT_ADJUST)) | HIT_OFFSCREEN));
                }
                break 'node;
            }
            if (!visible.is_empty() || reaches) && flags != 0 && !n.inert {
                hits.push(hit(visible, flags));
            }
            // A rich text's links are controls of their own.
            if !n.links.is_empty() && !n.inert {
                for (l, lr) in self.rich_links(i) {
                    let lr = n.world.map_rect(&lr);
                    let shown = lr.intersect(&n.clip);
                    if shown.is_empty() {
                        continue;
                    }
                    hits.push(Hit {
                        key: child_key(n.key, &l),
                        rect: shown,
                        full: lr,
                        flags: HIT_POINTER | HIT_NAV,
                        scope: o.scope_of[i],
                        scroller: o.scroller_of[i],
                        owner: o.owner_of[i],
                        inv: Affine::IDENTITY,
                        local: shown,
                        clip: n.clip,
                        group: o.group_of[i],
                        gesture: o.gesture_of[i],
                    });
                }
            }
        }
        let o = &mut self.orders;
        if o.hit_of[i] == hits && o.warn_of[i] == warn {
            return false;
        }
        let had = !o.hit_of[i].is_empty() || o.warn_of[i].is_some();
        let has = !hits.is_empty() || warn.is_some();
        o.hit_of[i] = hits;
        o.warn_of[i] = warn;
        if had != has {
            let at = o.hit_nodes.partition_point(|&j| o.pos[j] < o.pos[i]);
            if has {
                o.hit_nodes.insert(at, i);
            } else if o.hit_nodes.get(at) == Some(&i) {
                o.hit_nodes.remove(at);
            }
        }
        true
    }

    /// Brings the drawing order up to date once motion has set this
    /// frame's `z`: made again where the shape, a `z` or the world panels
    /// changed.
    pub(super) fn update_draw_order(&mut self, moved: &[usize]) {
        let n = self.nodes.len();
        let o = &mut self.orders;
        let nodes = &self.nodes;
        let z_moved = moved
            .iter()
            .chain(&self.emitted)
            .any(|&i| i < n && o.z[i] != nodes[i].sampled.z);
        let panels = self.world.panel_nodes();
        if o.draw_ok && !z_moved && panels == o.panels {
            return;
        }
        o.draw_ok = true;
        o.panels = panels;
        let mut draw = crate::layout::draw_order(nodes, 0);
        for &f in &self.floating {
            draw.extend(crate::layout::draw_order(nodes, f));
        }
        self.world_order(&mut draw);
        let o = &mut self.orders;
        o.pos.clear();
        o.pos.resize(n, u32::MAX);
        for (k, &i) in draw.iter().enumerate() {
            o.pos[i] = k as u32;
            o.z[i] = self.nodes[i].sampled.z;
        }
        o.ends = crate::layout::subtree_ends(&self.nodes, &draw);
        o.roots = draw
            .iter()
            .filter(|&&i| self.nodes[i].parent == NONE)
            .map(|&i| self.nodes[i].key)
            .collect();
        o.draw = draw;
        // The hits follow the drawing order; a node that left has none.
        let mut kept = Vec::with_capacity(o.hit_nodes.len());
        for &i in &o.hit_nodes {
            if o.pos[i] == u32::MAX {
                o.hit_of[i].clear();
                o.warn_of[i] = None;
            } else {
                kept.push(i);
            }
        }
        kept.sort_unstable_by_key(|&i| o.pos[i]);
        o.hit_nodes = kept;
        o.hits_moved = true;
    }

    /// The bounded frame's check: the kept orders and features are those
    /// the frame makes afresh.
    #[cfg(debug_assertions)]
    pub(super) fn check_orders(&self) {
        let mut pre = crate::layout::preorder(&self.nodes, 0);
        for &f in &self.floating {
            pre.extend(crate::layout::preorder(&self.nodes, f));
        }
        assert_eq!(pre, self.orders.pre, "the kept preorder differs");
        let mut draw = crate::layout::draw_order(&self.nodes, 0);
        for &f in &self.floating {
            draw.extend(crate::layout::draw_order(&self.nodes, f));
        }
        self.world_order(&mut draw);
        assert_eq!(draw, self.orders.draw, "the kept drawing order differs");
        for &i in &pre {
            assert_eq!(
                features(&self.nodes[i]),
                self.orders.feat[i],
                "the kept features of '{}' differ",
                self.path_of(i)
            );
        }
        for b in 0..feat::COUNT {
            let want: Vec<usize> = pre
                .iter()
                .copied()
                .filter(|&i| self.orders.feat[i] & (1 << b) != 0)
                .collect();
            assert_eq!(
                want, self.orders.lists[b],
                "the kept list of feature {b} differs"
            );
        }
    }
}
