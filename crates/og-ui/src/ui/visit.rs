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
    /// Rich text.
    pub const RICH: u32 = 1 << 16;
    /// CSS box decorations (`Ui::build_part_draws` merges them).
    pub const DECOR: u32 = 1 << 17;
    /// CSS `content-visibility` other than `visible`: it may skip its
    /// contents (`Ui::content_visibility`).
    pub const SKIPS: u32 = 1 << 18;
    /// CSS `resize`: a grab handle in its corner.
    pub const RESIZE: u32 = 1 << 19;
    pub const COUNT: usize = 20;
}

/// A row whose items (all, or those with an `align-self` of their own)
/// align by a baseline.
pub(crate) fn baseline_row(n: &Node) -> bool {
    use crate::layout::Cross;
    matches!(n.layout.used.cross, Cross::Baseline | Cross::LastBaseline) || n.layout.baseline_items
}

/// The features of node `n`.
pub(crate) fn features(n: &Node) -> u32 {
    use crate::element::Position;
    let p = &n.props;
    let mut f = 0;
    if baseline_row(n)
        && n.layout.used.direction == crate::layout::Direction::Row
        && n.layout_mode() == crate::element::LayoutMode::Flex
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
    if n.el.contain.visibility != crate::layout::ContentVisibility::Visible || n.layout.cv_skip {
        f |= feat::SKIPS;
    }
    if n.el.resize != 0 {
        f |= feat::RESIZE;
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
    if matches!(*n.content, Content::Rich(_)) {
        f |= feat::RICH;
    }
    if n.decor.is_some() {
        f |= feat::DECOR;
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
    /// Per slot: the `z` the drawing order sorted it by and whether it
    /// made a stacking context then (`layout::stacking_context`), and the
    /// world panels the order was made with.
    stack: Vec<(i32, bool)>,
    panels: Vec<usize>,
    /// Per slot: whether a stacking context lifts it out of its parent's
    /// run of the drawing order (drawn before its parent, or past the
    /// parent's `ends`); and those nodes, in drawing order.
    pub lifted_of: Vec<bool>,
    pub lifted: Vec<usize>,
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
    /// Per drawing position: the node a visit has still to take there
    /// (`usize::MAX`: none); empty between visits.
    pending: Vec<usize>,
    /// Per slot: the focus scope, scroll container and focus owner
    /// around it, and its focus group and gesture recogniser (0: none),
    /// kept where nothing above it changed.
    pub scope_of: Vec<u64>,
    pub scroller_of: Vec<u64>,
    pub owner_of: Vec<u64>,
    pub group_of: Vec<u64>,
    pub gesture_of: Vec<u64>,
    /// Per slot: the split screen region around it (`Hit::player`).
    pub player_of: Vec<u8>,
    /// Per slot: whether its text can be selected (`user-select` as used:
    /// its own, else its parent's; never `Auto`).
    pub select_of: Vec<crate::element::UserSelect>,
    /// Per slot: its used CSS `pointer-events` (never `Inherit`),
    /// `visibility` (whether hidden) and `touch-action` (the gestures a
    /// touch on it may make, its ancestors' intersected; `u8::MAX`: auto).
    pub pointer_of: Vec<crate::element::PointerEvents>,
    pub hidden_of: Vec<bool>,
    /// Per slot: whether an element around it skips its contents
    /// (`content-visibility`): [`SKIP_AUTO`] off the screen, [`SKIP_HIDDEN`]
    /// always; it is then not drawn or hit, and with `SKIP_HIDDEN` not in
    /// the accessibility tree. `skip_in` is the same for its own contents.
    pub skip_of: Vec<u8>,
    pub skip_in: Vec<u8>,
    pub touch_of: Vec<u8>,
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
}

/// [`Orders::skip_of`]: skipped by `content-visibility: auto`, or `hidden`.
pub(crate) const SKIP_AUTO: u8 = 1;
pub(crate) const SKIP_HIDDEN: u8 = 2;

impl Orders {
    /// The nodes that carry feature `f`, in preorder.
    pub(crate) fn with(&self, f: u32) -> &[usize] {
        self.lists
            .get(f.trailing_zeros() as usize)
            .map_or(&[], Vec::as_slice)
    }

    /// Whether node `i` made a stacking context when the drawing order
    /// was made.
    pub(crate) fn stacks(&self, i: usize) -> bool {
        self.stack.get(i).is_some_and(|s| s.1)
    }

    /// The nodes that carry feature `f`, in drawing order.
    pub(crate) fn drawn(&self, f: u32) -> Vec<usize> {
        let mut out = self.with(f).to_vec();
        out.sort_unstable_by_key(|&i| self.pos[i]);
        out
    }
}

impl Ui {
    /// Visits the drawn nodes from `seeds` down, parents before children
    /// (in preorder: a stacking context draws some children before their
    /// parent): `step` makes a node's frame state again and answers
    /// whether its children's changes with it. `None` visits every node in
    /// `order`, the drawing order.
    pub(super) fn visit(
        &mut self,
        order: &[usize],
        seeds: Option<&[usize]>,
        mut step: impl FnMut(&mut Ui, usize) -> bool,
    ) -> Vec<usize> {
        let Some(seeds) = seeds else {
            if self.tree.orders.lifted.is_empty() {
                for &i in order {
                    step(self, i);
                }
            } else {
                let pre = std::mem::take(&mut self.tree.orders.pre);
                for &i in &pre {
                    if self.tree.orders.pos[i] != u32::MAX {
                        step(self, i);
                    }
                }
                self.tree.orders.pre = pre;
            }
            return order.to_vec();
        };
        let o = &mut self.tree.orders;
        o.stamp = o.stamp.wrapping_add(1);
        if o.stamp == 0 {
            o.seen.fill(0);
            o.stamp = 1;
        }
        let stamp = o.stamp;
        // The nodes still to visit by preorder position, taken lowest
        // first: what a visit reaches is mostly whole subtrees, so a scan
        // up the positions finds them without ordering each one.
        let mut pending = std::mem::take(&mut o.pending);
        let mut at = usize::MAX;
        let mark = |pending: &mut Vec<usize>, p: u32, i: usize| {
            let p = p as usize;
            if p >= pending.len() {
                pending.resize(p + 1, usize::MAX);
            }
            pending[p] = i;
            p
        };
        for &i in seeds {
            if o.pos[i] != u32::MAX && o.pre_pos[i] != u32::MAX {
                at = at.min(mark(&mut pending, o.pre_pos[i], i));
            }
        }
        let mut out = Vec::new();
        let mut p = at;
        while p < pending.len() {
            let i = std::mem::replace(&mut pending[p], usize::MAX);
            p += 1;
            if i == usize::MAX || self.tree.orders.seen[i] == stamp {
                continue;
            }
            self.tree.orders.seen[i] = stamp;
            out.push(i);
            if step(self, i) {
                for k in 0..self.tree.nodes[i].ordered_children.len() {
                    let c = self.tree.nodes[i].ordered_children[k];
                    let o = &self.tree.orders;
                    if o.pos[c] != u32::MAX && o.pre_pos[c] != u32::MAX && o.seen[c] != stamp {
                        // One before it (a layer's root) is the next to take.
                        p = p.min(mark(&mut pending, o.pre_pos[c], c));
                    }
                }
            }
        }
        pending.clear();
        self.tree.orders.pending = pending;
        out
    }

    /// Node `i`'s scope, scroller and focus owner from its parent's, its
    /// focus group and gesture recogniser from its parent's (a layer's
    /// anchor's); answers whether any changed.
    pub(super) fn scope_step(&mut self, i: usize) -> bool {
        let nodes = &self.tree.nodes;
        let o = &self.tree.orders;
        let n = &nodes[i];
        let up = n.parent;
        let (ps, pr) = if up == NONE {
            (0, 0)
        } else {
            (o.scope_of[up], o.scroller_of[up])
        };
        let owner = match up {
            NONE => 0,
            u if nodes[u].nav && !nodes[u].placed.inert => nodes[u].key,
            u => o.owner_of[u],
        };
        let scope = if n.scope { n.key } else { ps };
        let scroller = if up != NONE && nodes[up].scrolls() {
            nodes[up].key
        } else {
            pr
        };
        // A node takes its parent's group and recogniser, a layer its
        // anchor's where the anchor is drawn and visited before it (one
        // after has none yet).
        let up = if n.parent == NONE { n.anchor } else { n.parent };
        let later = |up: usize| o.pos[up] > o.pos[i] || o.pre_pos[up] > o.pre_pos[i];
        let (pg, pr, pp) = if up == NONE || up >= nodes.len() || (n.parent == NONE && later(up)) {
            (0, 0, 0)
        } else {
            (o.group_of[up], o.gesture_of[up], o.player_of[up])
        };
        let player = n.props.player.map_or(pp, |k| k.min(3) + 1);
        let group = if n.props.focus_group.is_some() {
            n.key
        } else {
            pg
        };
        let gesture = match n.props.gestures.filter(|g| g.any()) {
            Some(_) if !n.placed.inert => n.key,
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
        // `pointer-events` and `visibility` inherit; `touch-action`'s used
        // value is its own intersected with its ancestors'.
        let (pointer, hidden, touch) = {
            use crate::element::{PointerEvents, Visibility};
            let parent = (up != NONE).then_some(up);
            let i = &n.props.interact;
            let pointer = match i.pointer_events {
                PointerEvents::Inherit => parent.map_or(PointerEvents::Auto, |u| o.pointer_of[u]),
                v => v,
            };
            let hidden = match i.visibility {
                Visibility::Inherit => parent.is_some_and(|u| o.hidden_of[u]),
                v => v != Visibility::Visible,
            };
            let touch =
                parent.map_or(u8::MAX, |u| o.touch_of[u]) & i.touch_action.unwrap_or(u8::MAX);
            (pointer, hidden, touch)
        };
        // `content-visibility`: skipped where an element around it skips
        // its contents.
        let skip = if up == NONE || up >= nodes.len() {
            0
        } else {
            o.skip_in[up]
        };
        let skip_in = skip.max(match (n.layout.cv_skip, n.el.contain.visibility) {
            (false, _) => 0,
            (true, crate::layout::ContentVisibility::Hidden) => SKIP_HIDDEN,
            (true, _) => SKIP_AUTO,
        });
        let o = &mut self.tree.orders;
        let old = (
            o.scope_of[i],
            o.scroller_of[i],
            o.owner_of[i],
            o.group_of[i],
            o.gesture_of[i],
            o.select_of[i],
            o.player_of[i],
            (o.pointer_of[i], o.hidden_of[i], o.touch_of[i]),
            (o.skip_of[i], o.skip_in[i]),
        );
        let new = (
            scope,
            scroller,
            owner,
            group,
            gesture,
            select,
            player,
            (pointer, hidden, touch),
            (skip, skip_in),
        );
        (
            o.scope_of[i],
            o.scroller_of[i],
            o.owner_of[i],
            o.group_of[i],
            o.gesture_of[i],
            o.select_of[i],
            o.player_of[i],
            (o.pointer_of[i], o.hidden_of[i], o.touch_of[i]),
            (o.skip_of[i], o.skip_in[i]),
        ) = new;
        old != new
    }

    /// Node `i`'s hits (none for the screen's root) and the warning its
    /// place gives, from its frame state; answers whether they changed.
    pub(super) fn hit_step(&mut self, i: usize, touch_min: f32, screen: Rect) -> bool {
        let o = &self.tree.orders;
        let mut warn = None;
        let mut hits = Vec::new();
        'node: {
            if i == 0 {
                break 'node;
            }
            let n = &self.tree.nodes[i];
            if self.tree.backface_hidden(i) {
                break 'node;
            }
            // CSS `pointer-events` and `visibility` (an SVG value without
            // `visible` in it takes the pointer while hidden too).
            let (pass, shape) = match o.pointer_of[i] {
                crate::element::PointerEvents::None => (true, None),
                crate::element::PointerEvents::Svg(s)
                    if self
                        .paint
                        .draw_of(i, &self.tree)
                        .is_some_and(|d| d.view_box.is_some()) =>
                {
                    (false, Some(s.0))
                }
                _ => (false, None),
            };
            let shown_or_svg = shape.is_some_and(|s| s & crate::element::SvgPointer::VISIBLE == 0);
            if (o.hidden_of[i] && !shown_or_svg) || o.skip_of[i] != 0 {
                break 'node;
            }
            let visible = n.placed.screen.intersect(&n.placed.clip);
            // Scrolled or clipped out of view: not on screen, not listed. A
            // zero-size element (an empty spacer) is listed where it sits.
            let (r, c) = (&n.placed.screen, &n.placed.clip);
            let outside =
                r.x > c.right() || r.right() < c.x || r.y > c.bottom() || r.bottom() < c.y;
            // A control laid out past the screen's edge with no scroller to
            // bring it back (a menu taller than a landscape phone): the
            // player cannot reach it, so the frame says which and by how much.
            if n.el.interactive
                && o.scroller_of[i] == 0
                && !r.is_empty()
                && !n.placed.inert
                && !crate::world::off_screen(&n.placed.rect)
            {
                // Where layout put it: a transform's move (a card thrown
                // off by a swipe) is not a layout that does not fit.
                let r = &n.placed.rect;
                let cut = (screen.x - r.x)
                    .max(r.right() - screen.right())
                    .max(screen.y - r.y)
                    .max(r.bottom() - screen.bottom());
                if cut > 1.0 {
                    warn = Some(format!(
                        "'{}' is laid out {:.0} px past the {}x{} screen and nothing scrolls to it: fit the screen (a smaller type step, fewer rows, a row instead of a column) or put it in a scroll container",
                        self.tree.path_of(i),
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
            if (matches!(*n.content, Content::Text(_))
                || matches!(*n.content, Content::Rich(_)) && n.rich_vertical().is_some())
                && n.props.text_path.is_none()
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
            if flags == 0 && n.layout.links.is_empty() && n.el.resize == 0 {
                break 'node;
            }
            if pass {
                // Keys and pads still reach it; the pointer passes.
                flags = (flags & !(HIT_SCROLL | HIT_DROP | HIT_SELECT)) | HIT_PASS;
            } else if shape.is_some_and(|s| s & crate::element::SvgPointer::BOX == 0) {
                flags |= HIT_SHAPE;
            }
            // Its reach past its box: its hit slop, and on a touch screen a
            // control's box grown to the minimum touch target (§15.2).
            let slop = n.props.hit_slop.max(0.0);
            let control = flags & (HIT_POINTER | HIT_TEXT) != 0 && !n.backdrop;
            let (sx, sy) = if control && touch_min > 0.0 {
                (
                    slop.max((touch_min - n.placed.rect.width) / 2.0),
                    slop.max((touch_min - n.placed.rect.height) / 2.0),
                )
            } else {
                (slop, slop)
            };
            let reach = Rect::new(
                n.placed.rect.x - sx,
                n.placed.rect.y - sy,
                n.placed.rect.width + 2.0 * sx,
                n.placed.rect.height + 2.0 * sy,
            );
            let reaches = sx > 0.0 || sy > 0.0;
            let transformed = !n.placed.world.is_identity();
            let inv = if transformed {
                n.placed.world.invert().unwrap_or(Affine::IDENTITY)
            } else {
                Affine::IDENTITY
            };
            let hit = |rect: Rect, flags: u16| {
                let rect = if reaches {
                    n.placed.world.map_rect(&reach).intersect(&n.placed.clip)
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
                    clip: n.placed.clip,
                    group: o.group_of[i],
                    gesture: o.gesture_of[i],
                    player: o.player_of[i],
                }
            };
            if outside || (visible.is_empty() && !r.is_empty()) {
                // Navigation can still reach a control just out of a list's view.
                if n.nav && !n.placed.inert && o.scroller_of[i] != 0 {
                    hits.push(hit(*r, (flags & (HIT_NAV | HIT_ADJUST)) | HIT_OFFSCREEN));
                }
                break 'node;
            }
            if (!visible.is_empty() || reaches) && flags != 0 && !n.placed.inert {
                hits.push(hit(visible, flags));
            }
            // CSS `resize`: its handle, a hit of its own over it.
            if n.el.resize != 0 && !n.placed.inert && !pass && !transformed {
                let rtl = self.env.direction == og_i18n::Direction::Rtl;
                let g = super::resize::grip_rect(n, rtl).intersect(&n.placed.clip);
                if !g.is_empty() {
                    hits.push(Hit {
                        key: super::resize::grip_key(n.key),
                        rect: g,
                        full: g,
                        flags: HIT_POINTER,
                        local: g,
                        ..hit(g, HIT_POINTER)
                    });
                }
            }
            // A rich text's links are controls of their own, hit on each
            // line's fragment (not across the lines between), reached by
            // the pad on their first.
            if !n.layout.links.is_empty() && !n.placed.inert {
                for (l, frags) in self.rich_link_frags(i) {
                    for (k, lr) in frags.iter().enumerate() {
                        let lr = n.placed.world.map_rect(lr);
                        let shown = lr.intersect(&n.placed.clip);
                        if shown.is_empty() {
                            continue;
                        }
                        let nav = if k == 0 { HIT_NAV } else { 0 };
                        hits.push(Hit {
                            key: child_key(n.key, &l),
                            rect: shown,
                            full: lr,
                            flags: HIT_POINTER | nav | if pass { HIT_PASS } else { 0 },
                            scope: o.scope_of[i],
                            scroller: o.scroller_of[i],
                            owner: o.owner_of[i],
                            inv: Affine::IDENTITY,
                            local: shown,
                            clip: n.placed.clip,
                            group: o.group_of[i],
                            gesture: o.gesture_of[i],
                            player: o.player_of[i],
                        });
                    }
                }
            }
        }
        let o = &mut self.tree.orders;
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
}

impl crate::ui::Tree {
    /// A layer culled for its backface cannot take the pointer, including
    /// children composited into it. Follow layers, not ancestry:
    /// preserve-3d planes draw separately.
    pub(crate) fn backface_hidden(&self, mut i: usize) -> bool {
        while let Some(plane) = self.nodes[i].placed.plane.as_deref() {
            if plane.hidden {
                return true;
            }
            if plane.into == NONE {
                break;
            }
            i = plane.into;
        }
        false
    }

    /// What this frame may change, before its passes: the nodes written,
    /// laid out or put back, those that entered a tree or whose parent's
    /// children changed, those the frame's own passes wrote last frame,
    /// those still moving after last frame's motion, and those a pass
    /// moves every frame (drags, shared elements, world anchors, layers);
    /// and of them, those whose style may change (`restyle`): sampled
    /// again and moved by motion. A node only laid out or moved among its
    /// siblings keeps its sampled style (made from what was written).
    pub(super) fn frame_seeds(
        &self,
        style: &crate::ui::Style,
        layout: &crate::ui::Layout,
        interaction: &crate::ui::Interaction,
    ) -> (Vec<usize>, Vec<usize>) {
        let o = &self.orders;
        let mut restyle: Vec<usize> = self
            .emitted
            .iter()
            .chain(&o.wrote_last)
            .chain(&style.motion.motion_moving)
            .chain(&layout.cache.anchor_toggled)
            .chain(o.with(feat::GESTURE))
            .chain(o.with(feat::MATCH))
            .chain(o.with(feat::WORLD))
            .copied()
            .collect();
        for &k in &self.index.moved_keys {
            restyle.extend(self.index.by_key.get(&k).copied());
        }
        // Scroll-driven animations whose timeline moved (a scroll, a
        // layout change of the scroller or the subject).
        for (k, last) in &style.motion.timelines {
            let Some(&i) = self.index.by_key.get(k) else {
                continue;
            };
            let moved = self.nodes[i]
                .props
                .animations
                .iter()
                .enumerate()
                .any(|(x, a)| {
                    !a.timeline.ticks()
                        && style.timeline_progress(i, a, interaction, self)
                            != last.get(x).copied().flatten()
                });
            if moved {
                restyle.push(i);
            }
        }
        let mut seeds = restyle.clone();
        seeds.extend(layout.cache.laid.iter().chain(&self.floating));
        for &p in &self.index.restructured {
            seeds.extend_from_slice(&self.nodes[p].ordered_children);
        }
        for list in [&mut restyle, &mut seeds] {
            list.retain(|&i| i < self.nodes.len() && self.index.has(i));
            list.sort_unstable();
            list.dedup();
        }
        (seeds, restyle)
    }

    /// Whether a point on an SVG shape's hit ([`HIT_SHAPE`]) is on the
    /// geometry its `pointer-events` names.
    pub(crate) fn shape_hit(&self, h: &Hit, x: f32, y: f32, paint: &crate::ui::Paint) -> bool {
        let Some(i) = self.index.find(h.key) else {
            return false;
        };
        let (crate::element::PointerEvents::Svg(bits), Some(d)) =
            (self.orders.pointer_of[i], paint.draw_of(i, self))
        else {
            return true;
        };
        let r = &self.nodes[i].placed.rect;
        let (lx, ly) = if h.inv.is_identity() {
            (x, y)
        } else {
            h.inv.apply(x, y)
        };
        let local = og_core::glam::Vec2::new(lx - r.x, ly - r.y);
        d.covers([r.width, r.height], local, bits.0)
    }
}

impl crate::ui::Tree {
    /// Node `i`'s stack level and whether it makes a stacking context.
    fn stack(&self, i: usize) -> (i32, bool) {
        let n = &self.nodes[i];
        let drawn = self.draws.contains_key(&i);
        (n.style.sampled.z, crate::layout::stacking_context(n, drawn))
    }

    /// The trees in drawing order: the screen's, then each layer's, with
    /// the world panels in their places (`Layout::world_order`).
    pub(crate) fn drawing_order(
        &self,
        world: &crate::world::WorldUi,
        layout: &crate::ui::Layout,
    ) -> Vec<usize> {
        let drawn = |i: usize| self.draws.contains_key(&i);
        let mut draw = crate::layout::draw_order(&self.nodes, 0, &drawn);
        for &f in &self.floating {
            draw.extend(crate::layout::draw_order(&self.nodes, f, &drawn));
        }
        layout.world_order(&mut draw, world, self);
        draw
    }

    /// Brings the drawing order up to date once motion has set this
    /// frame's `z`: made again where the shape, a `z`, a stacking context
    /// or the world panels changed.
    pub(super) fn update_draw_order(
        &mut self,
        moved: &[usize],
        world: &crate::world::WorldUi,
        layout: &crate::ui::Layout,
    ) {
        let n = self.nodes.len();
        let o = &self.orders;
        let restacked = moved
            .iter()
            .chain(&self.emitted)
            .any(|&i| i < n && o.stack[i] != self.stack(i));
        let panels = world.panel_nodes();
        if o.draw_ok && !restacked && panels == o.panels {
            return;
        }
        let draw = self.drawing_order(world, layout);
        let stack: Vec<_> = draw.iter().map(|&i| (i, self.stack(i))).collect();
        let o = &mut self.orders;
        o.draw_ok = true;
        o.panels = panels;
        o.pos.clear();
        o.pos.resize(n, u32::MAX);
        for (k, &i) in draw.iter().enumerate() {
            o.pos[i] = k as u32;
        }
        for (i, s) in stack {
            o.stack[i] = s;
        }
        o.ends = crate::layout::subtree_ends(&self.nodes, &draw);
        o.lifted.clear();
        for (k, &i) in draw.iter().enumerate() {
            let up = self.nodes[i].parent;
            let lifted = up != NONE && {
                let at = o.pos[up] as usize;
                at > k || o.ends[at] <= k
            };
            o.lifted_of[i] = lifted;
            if lifted {
                o.lifted.push(i);
            }
        }
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
    pub(crate) fn check_orders(&self, world: &crate::world::WorldUi, layout: &crate::ui::Layout) {
        let mut pre = crate::layout::preorder(&self.nodes, 0);
        for &f in &self.floating {
            pre.extend(crate::layout::preorder(&self.nodes, f));
        }
        assert_eq!(pre, self.orders.pre, "the kept preorder differs");
        let draw = self.drawing_order(world, layout);
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

impl crate::ui::Tree {
    /// Brings the preorder and the features up to date with the trees:
    /// made again where their shape changed, a feature read again on the
    /// nodes written or laid out this frame.
    pub(super) fn update_orders(&mut self, whole: bool, layout: &crate::ui::Layout) {
        let n = self.nodes.len();
        let o = &mut self.orders;
        if o.feat.len() < n {
            o.feat.resize(n, 0);
            o.stack.resize(n, (0, false));
            o.lifted_of.resize(n, false);
            o.seen.resize(n, 0);
            o.scope_of.resize(n, 0);
            o.scroller_of.resize(n, 0);
            o.owner_of.resize(n, 0);
            o.group_of.resize(n, 0);
            o.gesture_of.resize(n, 0);
            o.player_of.resize(n, 0);
            o.select_of.resize(n, crate::element::UserSelect::None);
            o.pointer_of.resize(n, crate::element::PointerEvents::Auto);
            o.hidden_of.resize(n, false);
            o.skip_of.resize(n, 0);
            o.skip_in.resize(n, 0);
            o.touch_of.resize(n, u8::MAX);
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
        let reshaped = whole || o.shape != Some(self.index.shape);
        if reshaped {
            o.shape = Some(self.index.shape);
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
            .chain(&layout.cache.laid)
            .chain(&self.invalidation.restored);
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
}
