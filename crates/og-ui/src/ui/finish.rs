use super::*;

/// The smallest touch target, CSS px: a control's reach on a touch screen
/// grows to it on each side it is shorter (Apple's HIG 44 pt).
pub(crate) const TOUCH_TARGET: f32 = 44.0;

impl Ui {
    /// The minimum touch target in og-ui units (44 CSS px) on a touch
    /// screen, zero with a fine pointer.
    pub(crate) fn touch_min(&self) -> f32 {
        if self.viewport.coarse_pointer {
            TOUCH_TARGET / self.unit.max(0.01)
        } else {
            0.0
        }
    }

    /// Lays out the frame, updates the interaction state and returns the frame.
    pub fn end(&mut self) -> &UiFrame {
        if !self.building {
            self.warnings
                .push("end without begin: call ui.begin() at the start of the tick".into());
            return &self.frame;
        }
        while self.stack.len() > 1 {
            let i = self.stack.pop().unwrap_or(0);
            self.warnings.push(format!(
                "'{}' was not closed; every ui.open() takes one ui.close()",
                self.path_of(i)
            ));
        }
        crate::layers::build_without_host(self);
        self.build_toasts();
        self.building = false;
        // Breakpoint and orientation variants (docs/ui.md §6.2), before
        // anything is laid out.
        self.apply_variants();
        self.apply_part_appearances();
        // Hidden slots (a closed dialog, a finished transition) take no room.
        for i in 0..self.nodes.len() {
            if self.nodes[i].hidden {
                continue;
            }
            let kids = std::mem::take(&mut self.nodes[i].children);
            let kept: Vec<usize> = kids
                .into_iter()
                .filter(|&c| !self.nodes[c].hidden)
                .collect();
            self.nodes[i].children = kept;
            // Out of the flow (absolute, anchored): laid out against the box.
            if self.nodes[i]
                .children
                .iter()
                .any(|&c| self.nodes[c].props.out_of_flow())
            {
                let kids = std::mem::take(&mut self.nodes[i].children);
                let (out, flow): (Vec<usize>, Vec<usize>) = kids
                    .into_iter()
                    .partition(|&c| self.nodes[c].props.out_of_flow());
                self.nodes[i].children = flow;
                self.nodes[i].out = out;
            }
        }
        // A style's display face stops at its `displayMin`: smaller text in
        // the display role takes the body face, before it is measured.
        let min = self.theme.art.display_min;
        if min > 0.0 {
            for n in &mut self.nodes {
                if let Content::Text(tc) = &mut n.content
                    && tc.font == crate::FontRole::Display
                    && tc.step.size < min
                {
                    tc.font = crate::FontRole::Body;
                }
            }
        }
        for node in &mut self.nodes {
            if matches!(node.widget, Widget::Field(_) | Widget::TextArea { .. }) {
                continue;
            }
            match &mut node.content {
                Content::Text(text) if text.step.uppercase => {
                    text.text = og_i18n::text::uppercase(&text.text, &self.locale);
                }
                Content::Rich(rich) => {
                    for span in &mut rich.spans {
                        if span.step.uppercase && span.atom.is_none() {
                            span.text = og_i18n::text::uppercase(&span.text, &self.locale);
                        }
                    }
                }
                _ => {}
            }
        }
        // A vertical scroller whose content ran past it last frame keeps a
        // gutter at its end for its bar (Ui::bar_gutter).
        let gutters: Vec<(usize, f32)> = (0..self.nodes.len())
            .filter(|&i| {
                let el = &self.nodes[i].el;
                el.scroll && !el.scroll_x && !el.hide_bars
            })
            .map(|i| (i, self.bar_gutter(self.nodes[i].key)))
            .filter(|g| g.1 > 0.0)
            .collect();
        for (i, g) in gutters {
            self.nodes[i].el.padding.right += g;
        }
        crate::layout::snap_pixel_text(&mut self.nodes, &self.fonts);
        self.reuse_check();
        let mut floating: Vec<usize> = self
            .floating
            .iter()
            .copied()
            .filter(|&f| !self.nodes[f].hidden)
            .collect();
        og_core::sort::sort_by_key(&mut floating, |&f| self.nodes[f].float.rank());
        self.floating = floating;

        let vp = self.viewport;
        let dpr = vp.dpr();
        let screen = Rect::new(0.0, 0.0, vp.width, vp.height);
        let scroll = self.state.scroll.clone();
        let scroll_x = self.state.scroll_x.clone();
        let rtl = self.direction == og_i18n::Direction::Rtl;
        let offset = |k: u64| {
            (
                scroll_x.get(&k).copied().unwrap_or(0.0),
                scroll.get(&k).copied().unwrap_or(0.0),
            )
        };
        let mut measure = Measure {
            fonts: &self.fonts,
            texts: &mut self.texts,
        };
        layout(
            &mut self.nodes,
            0,
            screen,
            &mut measure,
            &offset,
            dpr,
            screen,
            &mut self.kept,
        );
        let safe = Rect::new(
            vp.safe_area.left,
            vp.safe_area.top,
            vp.width - vp.safe_area.horizontal(),
            vp.height - vp.safe_area.vertical(),
        );
        // The HUD band: the safe area at most `hud_max_aspect` wide,
        // centred (an ultrawide's corners stay near the play), where the
        // anchored groups and the toasts lay out.
        let hud = vp.hud_rect(self.hud_max_aspect);
        // The host's control, where the layout stands before a right-to-left
        // mirror (which moves a top-left group to the top right).
        let covered = {
            let c = vp.covered;
            if self.direction == og_i18n::Direction::Rtl {
                Rect::new(vp.width - c.x - c.width, c.y, c.width, c.height)
            } else {
                c
            }
        };
        let gap = self.theme.space.xs;
        // The anchored groups first, each laid out in its area and placed
        // once all of them are known (so one the host's control moves keeps
        // off the others), then the other layers in their order (a popover
        // reads its anchor's final place).
        let (groups, rest): (Vec<usize>, Vec<usize>) = self
            .floating
            .iter()
            .partition(|&&f| matches!(self.nodes[f].float, Float::Anchored { .. }));
        let mut spots: Vec<(usize, Rect)> = Vec::new();
        for f in groups.into_iter().chain([NONE]).chain(rest) {
            // NONE marks the groups' end: place them, then go on to the rest.
            if f == NONE {
                if !covered.is_empty() {
                    for k in 0..spots.len() {
                        let (g, r) = spots[k];
                        let Float::Anchored { anchor, margin, .. } = self.nodes[g].float else {
                            continue;
                        };
                        let others: Vec<Rect> = spots
                            .iter()
                            .enumerate()
                            .filter(|&(j, _)| j != k)
                            .map(|(_, &(_, o))| o)
                            .collect();
                        spots[k].1 = keep_out(r, anchor, covered, margin.max(gap), hud, &others);
                    }
                }
                // A group placed at a point steps clear of the others
                // (which keep their places) and of the host's control.
                for k in 0..spots.len() {
                    let (g, r) = spots[k];
                    if !matches!(self.nodes[g].float, Float::Anchored { avoid: true, .. }) {
                        continue;
                    }
                    let mut others: Vec<Rect> = spots
                        .iter()
                        .filter(|&&(o, _)| {
                            !matches!(self.nodes[o].float, Float::Anchored { avoid: true, .. })
                        })
                        .map(|&(_, o)| o)
                        .collect();
                    if !covered.is_empty() {
                        others.push(covered);
                    }
                    spots[k].1 = step_clear(r, &others, gap, safe);
                }
                for &(g, placed) in &spots {
                    self.nodes[g].el.width = Size::Fixed(placed.width);
                    self.nodes[g].el.height = Size::Fixed(placed.height);
                    layout(
                        &mut self.nodes,
                        g,
                        placed,
                        &mut measure,
                        &offset,
                        dpr,
                        screen,
                        &mut self.kept,
                    );
                }
                continue;
            }
            let float = self.nodes[f].float;
            if float == Float::World {
                // At its own size: an anchor where it is projected later, a
                // panel in its slot off the screen.
                let (area, clip) = self.world.area(f, screen);
                layout(
                    &mut self.nodes,
                    f,
                    area,
                    &mut measure,
                    &offset,
                    dpr,
                    clip,
                    &mut self.kept,
                );
                continue;
            }
            if float == Float::Modal {
                // The backdrop covers the surface; the safe area pads its content.
                self.nodes[f].el.width = Size::Fixed(vp.width);
                self.nodes[f].el.height = Size::Fixed(vp.height);
                if self.nodes[f].backdrop {
                    self.nodes[f].el.padding = self.nodes[f].el.padding.add(&vp.safe_area);
                }
                layout(
                    &mut self.nodes,
                    f,
                    screen,
                    &mut measure,
                    &offset,
                    dpr,
                    screen,
                    &mut self.kept,
                );
                continue;
            }
            let area = match float {
                Float::Anchored { margin, .. } | Float::Toasts { margin, .. } => Rect::new(
                    hud.x + margin,
                    hud.y + margin,
                    (hud.width - 2.0 * margin).max(0.0),
                    (hud.height - 2.0 * margin).max(0.0),
                ),
                _ => safe,
            };
            layout(
                &mut self.nodes,
                f,
                area,
                &mut measure,
                &offset,
                dpr,
                screen,
                &mut self.kept,
            );
            let (w, h) = (self.nodes[f].w, self.nodes[f].h);
            let (x, y) = match float {
                Float::Anchored { anchor, dx, dy, .. } => {
                    let (ax, ay) = anchor.fractions();
                    (
                        area.x + (area.width - w) * ax + dx,
                        area.y + (area.height - h) * ay + dy,
                    )
                }
                Float::Toasts { anchor, .. } => {
                    let (ax, ay) = anchor.fractions();
                    (
                        area.x + (area.width - w) * ax,
                        area.y + (area.height - h) * ay,
                    )
                }
                _ => {
                    let a = self
                        .nodes
                        .get(self.nodes[f].anchor)
                        .map_or(safe, |n| n.rect);
                    let popover = float == Float::Popover;
                    let x = if popover {
                        // Aligned to the anchor's start in the reading direction.
                        if rtl { a.right() - w } else { a.x }
                    } else {
                        a.x + a.width / 2.0 - w / 2.0
                    };
                    let x = x.min(safe.right() - w).max(safe.x);
                    let mut y = a.bottom() + gap;
                    // A popover goes above only where there is more room
                    // (a tooltip whenever it does not fit), then stays on screen.
                    let above = a.y - safe.y > safe.bottom() - a.bottom();
                    if y + h > safe.bottom() && (above || !popover) {
                        y = (a.y - gap - h).max(safe.y);
                    }
                    if popover {
                        y = y.min(safe.bottom() - h).max(safe.y);
                    }
                    (x, y)
                }
            };
            let placed = Rect::new(x, y, w, h);
            if matches!(float, Float::Anchored { .. }) {
                spots.push((f, placed));
                continue;
            }
            self.nodes[f].el.width = Size::Fixed(w);
            self.nodes[f].el.height = Size::Fixed(h);
            layout(
                &mut self.nodes,
                f,
                placed,
                &mut measure,
                &offset,
                dpr,
                screen,
                &mut self.kept,
            );
        }
        // Elements placed on the screen (Position::Screen): laid out again in
        // the whole surface, under the safe area their layer keeps clear,
        // and clipped by nothing but the surface.
        let mut placed_order = preorder(&self.nodes, 0);
        for &f in &self.floating {
            placed_order.extend(preorder(&self.nodes, f));
        }
        for i in placed_order {
            if self.nodes[i].props.position != crate::element::Position::Screen {
                continue;
            }
            let r = crate::layout::on_screen(&self.nodes[i], &screen);
            self.nodes[i].el.width = Size::Fixed(r.width);
            self.nodes[i].el.height = Size::Fixed(r.height);
            layout(
                &mut self.nodes,
                i,
                r,
                &mut measure,
                &offset,
                dpr,
                screen,
                &mut self.kept,
            );
        }
        self.texts.sweep();

        let mut order = preorder(&self.nodes, 0);
        for &f in &self.floating {
            order.extend(preorder(&self.nodes, f));
        }
        self.align_baselines(&order);
        self.place_anchored(&order);
        if self.direction == og_i18n::Direction::Rtl {
            // Right to left: the whole layout mirrored about the surface's
            // centre, and with it each transform (moves and turns the other way).
            for &i in &order {
                let r = &mut self.nodes[i].rect;
                r.x = vp.width - r.x - r.width;
                // Its clip too (a scroll container's box, laid out left to right): unmirrored, it
                // would cut away the content of a scroller off the surface's centre line.
                let c = &mut self.nodes[i].clip;
                c.x = vp.width - c.x - c.width;
                let t = self.nodes[i].props.transform;
                if !t.is_identity() {
                    self.nodes[i].props.transform = t.mirrored();
                }
            }
        }
        let overlaps = self.find_overlaps(&order);
        let dense: Vec<String> = order
            .iter()
            .filter(|&&i| self.nodes[i].props.dense)
            .map(|&i| self.path_of(i))
            .collect();
        // A carousel's track, dragged sideways, scrolls as one does: the
        // pages peeking at its edges move into view.
        let sideways: Vec<String> = order
            .iter()
            .filter(|&&i| {
                let n = &self.nodes[i];
                n.el.scroll_x
                    || n.props
                        .drag
                        .as_ref()
                        .is_some_and(|d| d.axis == crate::DragAxis::X)
            })
            .map(|&i| self.path_of(i))
            .collect();
        self.apply_shakes(&order);
        self.apply_stagger(&order);
        self.run_motion(&order);
        self.apply_bursts(&order);
        self.world_place();
        // Drawn and hit in z order among siblings.
        let mut order = crate::layout::draw_order(&self.nodes, 0);
        for &f in &self.floating {
            order.extend(crate::layout::draw_order(&self.nodes, f));
        }
        self.world_order(&mut order);
        self.hud_apply(&order);
        self.apply_world(&order);
        self.hud_collect(&order);
        self.world_finish();
        self.note_drag_and_drop(&order);

        // Interaction state for the next tick.
        let mut present: crate::id::KeyMap<usize> =
            crate::id::KeyMap::from_iter(order.iter().map(|&i| (self.nodes[i].key, i)));
        // A rich text's links are controls of their own, inside its node.
        for &i in &order {
            for l in &self.nodes[i].links {
                present.insert(child_key(self.nodes[i].key, l), i);
            }
        }
        self.state
            .scroll
            .retain(|k, _| present.get(k).is_some_and(|&i| self.nodes[i].el.scroll));
        for (k, v) in self.state.scroll.iter_mut() {
            *v = v.clamp(0.0, self.nodes[present[k]].max_scroll());
        }
        self.state
            .scroll_x
            .retain(|k, _| present.get(k).is_some_and(|&i| self.nodes[i].el.scroll_x));
        for (k, v) in self.state.scroll_x.iter_mut() {
            *v = v.clamp(0.0, self.nodes[present[k]].max_scroll_x());
        }
        // Each scroll container's ranges and bar lengths, as drawn.
        let min_bar = self.theme.space.xl;
        self.state.scroll_info = IdMap::from_iter(
            order
                .iter()
                .filter(|&&i| self.nodes[i].scrolls())
                .map(|&i| {
                    let n = &self.nodes[i];
                    let range = Vec2::new(n.max_scroll_x(), n.max_scroll());
                    let bar = Vec2::new(
                        crate::scroll::bar_len(
                            n.rect.width,
                            n.w - n.el.padding.horizontal(),
                            n.content_w,
                            min_bar,
                        ),
                        crate::scroll::bar_len(
                            n.rect.height,
                            n.h - n.el.padding.vertical(),
                            n.content_h,
                            min_bar,
                        ),
                    );
                    (n.key, crate::scroll::ScrollInfo { range, bar })
                }),
        );
        if let Some((k, _)) = self.state.fling
            && !present.contains_key(&k)
        {
            self.state.fling = None;
        }
        if !self.input.down || !present.contains_key(&self.state.active) {
            self.state.active = 0;
        }
        let touched = std::mem::take(&mut self.touched);
        self.state.anims.retain(|k, _| touched.contains(k));
        self.state.screens.retain(|k, _| touched.contains(k));
        self.touched = touched;

        // Scopes: the nearest scope around each node, by walking the order.
        let mut scope_of = vec![0u64; self.nodes.len()];
        let mut scroller_of = vec![0u64; self.nodes.len()];
        let mut owner_of = vec![0u64; self.nodes.len()];
        for &i in &order {
            let n = &self.nodes[i];
            let up = if n.parent == NONE { NONE } else { n.parent };
            let (ps, pr) = if up == NONE {
                (0, 0)
            } else {
                (scope_of[up], scroller_of[up])
            };
            owner_of[i] = match up {
                NONE => 0,
                u if self.nodes[u].nav && !self.nodes[u].inert => self.nodes[u].key,
                u => owner_of[u],
            };
            scope_of[i] = if n.scope { n.key } else { ps };
            scroller_of[i] = if up != NONE && self.nodes[up].scrolls() {
                self.nodes[up].key
            } else {
                pr
            };
        }
        // Focus groups, explicit moves and gesture recognisers for the next frame.
        let mut group_of = vec![0u64; self.nodes.len()];
        let mut gesture_of = vec![0u64; self.nodes.len()];
        self.collect_input_specs(&order, &mut group_of, &mut gesture_of);
        let mut hits = Vec::new();
        let mut elements = Vec::new();
        let mut parts = Vec::new();
        // The accessibility tree: each element's key (for its focus states)
        // and the nodes with a role, with the paths their children name.
        let mut element_keys: Vec<u64> = Vec::new();
        let mut tree: Vec<(u64, crate::a11y::A11yNode)> = Vec::new();
        // The path of each node with a role, by node: read for every node's
        // nearest named ancestor, so an array, not a map.
        let mut tree_path: Vec<Option<String>> = vec![None; self.nodes.len()];
        let mut top_scope = 0u64;
        let mut top_locked = false;
        // The minimum touch target, in og-ui units (44 CSS px), on a touch screen.
        let touch_min = self.touch_min();
        for &i in &order[1..] {
            let n = &self.nodes[i];
            let (role, name, description, value, states) = self.a11y_info(i);
            let parent_path = {
                let mut up = if n.parent == NONE { n.anchor } else { n.parent };
                loop {
                    if up == NONE || up == 0 {
                        break String::new();
                    }
                    if let Some(p) = &tree_path[up] {
                        break p.clone();
                    }
                    up = if self.nodes[up].parent == NONE {
                        self.nodes[up].anchor
                    } else {
                        self.nodes[up].parent
                    };
                }
            };
            let path = if role.is_empty() {
                String::new()
            } else {
                self.path_of(i)
            };
            if !role.is_empty() {
                tree_path[i] = Some(path.clone());
            }
            if n.scope && !n.inert {
                top_scope = n.key;
                top_locked = n.locked;
            }
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
                && scroller_of[i] == 0
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
                    self.warnings.push(format!(
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
            if n.adjust {
                flags |= HIT_ADJUST;
            }
            flags |= u16::from(n.claim) << crate::ui::HIT_CLAIM_SHIFT;
            if n.scope {
                flags |= HIT_SCOPE;
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
                    scope: scope_of[i],
                    scroller: scroller_of[i],
                    owner: owner_of[i],
                    inv,
                    local: reach,
                    clip: n.clip,
                    group: group_of[i],
                    gesture: gesture_of[i],
                }
            };
            let range = crate::a11y::range_of(n).filter(|_| !role.is_empty());
            let node = |states: Vec<String>| crate::a11y::A11yNode {
                path: path.clone(),
                parent: parent_path.clone(),
                role: role.to_owned(),
                name: name.clone(),
                description: description.clone(),
                value: value.clone(),
                states,
                value_min: range.and_then(|r| r.min),
                value_max: range.and_then(|r| r.max),
                value_now: range.map(|r| r.now),
                x: r.x,
                y: r.y,
                width: r.width,
                height: r.height,
            };
            if outside || (visible.is_empty() && !r.is_empty()) {
                // Navigation can still reach a control just out of a list's view.
                if n.nav && !n.inert && scroller_of[i] != 0 {
                    hits.push(hit(*r, (flags & (HIT_NAV | HIT_ADJUST)) | HIT_OFFSCREEN));
                }
                if !role.is_empty() {
                    let mut st = states.clone();
                    st.push("offscreen".to_owned());
                    tree.push((n.key, node(st)));
                }
                continue;
            }
            if !role.is_empty() {
                tree.push((n.key, node(states.clone())));
            }
            if (!visible.is_empty() || reaches) && flags != 0 && !n.inert {
                hits.push(hit(visible, flags));
            }
            let parent = if n.parent == NONE {
                self.nodes
                    .get(n.anchor)
                    .map_or(String::new(), |a| a.id.clone())
            } else if n.parent == 0 {
                String::new()
            } else {
                self.nodes[n.parent].id.clone()
            };
            element_keys.push(n.key);
            parts.push(n.el.part.or_else(|| n.el.frame.map(StylePart::part_ref)));
            elements.push(UiElement {
                id: n.id.clone(),
                parent,
                path: self.probe_path(i),
                kind: self.kinds.get(i).map_or("", |k| k.0).to_string(),
                disabled: n.disabled,
                clipped: visible.width + 0.01 < r.width || visible.height + 0.01 < r.height,
                // As read: bidi marks and isolates steer the drawing only.
                text: n
                    .probe_text
                    .chars()
                    .filter(|&c| !og_i18n::is_bidi_control(c))
                    .collect(),
                role: role.to_owned(),
                name: name.clone(),
                value: value.clone(),
                states: states.clone(),
                // Its fill, solid and at full opacity, covers its box (a
                // rounded corner's few pixels aside).
                opaque: n.el.background.is_some_and(|bg| bg.alpha >= 0.999) && n.alpha >= 0.999,
                x: visible.x,
                y: visible.y,
                width: visible.width,
                height: visible.height,
                detail: Default::default(),
            });
            // A Rive frame's drawn objects: children for dump_ui, describe
            // and pick, their source the artboard's id and the object.
            if let Some(crate::paint::RiveDraw(player)) =
                self.draws.get(&i).and_then(|d| d.rive.as_ref())
            {
                let board = self.state.rive.get(&n.key).map_or("", |s| s.board.as_str());
                let file = board.split_once('#').map_or(board, |(f, _)| f);
                let at = n.world.map_rect(&n.rect);
                for e in player.elements([at.x, at.y, at.width, at.height]) {
                    let b = Rect::new(e.bounds[0], e.bounds[1], e.bounds[2], e.bounds[3]);
                    let shown = b.intersect(&n.clip);
                    element_keys.push(child_key(n.key, &e.id));
                    parts.push(None);
                    let mut detail = serde_json::Map::new();
                    detail.insert(
                        "source".into(),
                        serde_json::json!(format!("{file}#{}", e.id)),
                    );
                    detail.insert(
                        "rive".into(),
                        serde_json::json!({ "node": e.node, "opacity": e.opacity }),
                    );
                    elements.push(UiElement {
                        path: format!("{}/{}", self.probe_path(i), e.id),
                        id: e.id,
                        parent: n.id.clone(),
                        kind: "rive".into(),
                        disabled: false,
                        clipped: shown.width + 0.01 < b.width || shown.height + 0.01 < b.height,
                        text: e.text.clone(),
                        role: String::new(),
                        name: String::new(),
                        value: String::new(),
                        states: Vec::new(),
                        opaque: false,
                        x: shown.x,
                        y: shown.y,
                        width: shown.width,
                        height: shown.height,
                        detail,
                    });
                }
            }
            // A rich text's links: children for the probe, controls for input.
            if !n.links.is_empty() {
                let Content::Rich(rc) = &n.content else {
                    unreachable!("links come from rich text")
                };
                for (l, lr) in self.rich_links(i) {
                    let lr = n.world.map_rect(&lr);
                    let shown = lr.intersect(&n.clip);
                    if shown.is_empty() {
                        continue;
                    }
                    if !n.inert {
                        hits.push(Hit {
                            key: child_key(n.key, &l),
                            rect: shown,
                            full: lr,
                            flags: HIT_POINTER | HIT_NAV,
                            scope: scope_of[i],
                            scroller: scroller_of[i],
                            owner: owner_of[i],
                            inv: Affine::IDENTITY,
                            local: shown,
                            clip: n.clip,
                            group: group_of[i],
                            gesture: gesture_of[i],
                        });
                    }
                    let text: String = rc
                        .spans
                        .iter()
                        .filter(|s| s.link.as_deref() == Some(l.as_str()))
                        .map(|s| s.text.as_str())
                        .collect();
                    let link_path = format!("{}/{l}", self.path_of(i));
                    tree.push((
                        child_key(n.key, &l),
                        crate::a11y::A11yNode {
                            path: link_path,
                            parent: path.clone(),
                            role: "link".into(),
                            name: text.clone(),
                            description: String::new(),
                            value: String::new(),
                            states: Vec::new(),
                            x: lr.x,
                            y: lr.y,
                            width: lr.width,
                            height: lr.height,
                            ..crate::a11y::A11yNode::default()
                        },
                    ));
                    element_keys.push(child_key(n.key, &l));
                    parts.push(None);
                    elements.push(UiElement {
                        path: format!("{}/{l}", self.probe_path(i)),
                        id: l,
                        parent: n.id.clone(),
                        kind: "link".into(),
                        disabled: false,
                        clipped: shown.width + 0.01 < lr.width || shown.height + 0.01 < lr.height,
                        name: text.clone(),
                        text,
                        role: "link".into(),
                        value: String::new(),
                        states: Vec::new(),
                        opaque: false,
                        x: shown.x,
                        y: shown.y,
                        width: shown.width,
                        height: shown.height,
                        detail: Default::default(),
                    });
                }
            }
        }
        // Focus: kept while its control is on screen (or reachable); a scope
        // that opened takes it (its first control, or the one a widget asked
        // for); a scope that closed gives it back.
        let alive = |k: u64| hits.iter().any(|h: &Hit| h.key == k);
        let first_in = |scope: u64| {
            hits.iter()
                .find(|h| h.scope == scope && h.has(HIT_NAV) && !h.has(HIT_OFFSCREEN))
                .map_or(0, |h| h.key)
        };
        if self.request_focus != 0 && alive(self.request_focus) {
            self.state.focus = self.request_focus;
        }
        if top_scope != self.state.scope {
            let inside = |f: u64| {
                if top_scope == 0 {
                    f != 0 && alive(f)
                } else {
                    hits.iter().any(|h| h.key == f && h.scope == top_scope)
                }
            };
            // Back to a scope it opened over (the base, a menu under its
            // submenu): the focus it had there, else that scope's first.
            let back_to = self
                .state
                .focus_returns
                .iter()
                .rposition(|&(s, _)| s == top_scope);
            if let Some(k) = back_to {
                let (_, back) = self.state.focus_returns[k];
                self.state.focus_returns.truncate(k);
                if !inside(self.state.focus) {
                    self.state.focus = if inside(back) {
                        back
                    } else if top_scope != 0 {
                        first_in(top_scope)
                    } else {
                        0
                    };
                }
            } else if top_scope != 0 {
                if !inside(self.state.focus) {
                    // Entries of scopes no longer on screen are dropped; one
                    // that replaced another (the old one gone) opened over
                    // what that one had.
                    let on_screen = |s: u64| s == 0 || hits.iter().any(|h| h.scope == s);
                    self.state.focus_returns.retain(|&(s, _)| on_screen(s));
                    if on_screen(self.state.scope) {
                        self.state
                            .focus_returns
                            .push((self.state.scope, self.state.focus));
                    }
                    let req = self.request_focus;
                    self.state.focus =
                        if req != 0 && hits.iter().any(|h| h.key == req && h.scope == top_scope) {
                            req
                        } else {
                            first_in(top_scope)
                        };
                }
            } else {
                self.state.focus_returns.clear();
            }
        }
        if !alive(self.state.focus) {
            self.state.focus = 0;
        }
        // Pads 1 to 3: a focus whose control left is dropped; one outside a
        // scope that opened moves to its first control.
        for k in 0..3 {
            let f = self.state.pad_focus[k];
            let inside =
                |f: u64| top_scope == 0 || hits.iter().any(|h| h.key == f && h.scope == top_scope);
            if (k as u32) + 1 >= self.players {
                self.state.pad_focus[k] = 0;
            } else if f != 0 && (!alive(f) || !inside(f)) {
                self.state.pad_focus[k] = if top_scope != 0 {
                    first_in(top_scope)
                } else {
                    0
                };
            }
        }
        self.state.players = self.players as u8;
        self.state.vcursor.desc = self.vcursor;
        self.state.scope = top_scope;
        self.state.scope_locked = top_locked;
        self.state.nav_off = !self.nav_on;
        // Focus order: the controls keys and pads reach now, in the tree's
        // order (the order navigation walks: `nav::focusables`).
        let reach: Vec<u64> = hits
            .iter()
            .filter(|h| h.has(HIT_NAV | HIT_TEXT) && (top_scope == 0 || h.scope == top_scope))
            .map(|h| h.key)
            .collect();
        let mark = |key: u64, states: &mut Vec<String>| {
            if reach.contains(&key) {
                states.push("focusable".to_owned());
            }
            if key != 0 && key == self.state.focus {
                states.push("focused".to_owned());
            }
        };
        for (e, &key) in elements.iter_mut().zip(&element_keys) {
            if !e.role.is_empty() {
                mark(key, &mut e.states);
            }
        }
        for (key, node) in &mut tree {
            mark(*key, &mut node.states);
        }
        let focus_order: Vec<String> = reach
            .iter()
            .filter_map(|k| {
                tree.iter()
                    .find(|(t, _)| t == k)
                    .map(|(_, n)| n.path.clone())
            })
            .collect();
        let a11y = crate::a11y::A11yTree {
            focused: tree
                .iter()
                .find(|(k, _)| *k != 0 && *k == self.state.focus)
                .map_or(String::new(), |(_, n)| n.path.clone()),
            nodes: tree.into_iter().map(|(_, n)| n).collect(),
            focus_order,
        };
        self.state.hits = hits;
        self.note_groups();
        self.note_taps(&present);
        let cursor = if self.state.vcursor.shown && self.state.vcursor.desc.is_some() {
            // The pad's cursor is drawn; the system's is hidden.
            crate::Cursor::None
        } else {
            self.resolve_cursor(&present)
        };

        // A button narrower than an ornament's span is `[narrow]`, laid
        // out: a style draws it plain (fantasy's hairline and bevel, no
        // ornament under 160 px, art review 2026-11-04). `Ui::button`
        // decides its own as it builds (its label's ink follows the art).
        for &i in &order {
            let n = &mut self.nodes[i];
            if !matches!(n.widget, crate::layout::Widget::Button { .. })
                && let Some(p) = n.el.part.as_mut()
                && p.class == Some(crate::tokens::ElementClass::Button)
                && n.rect.width < NARROW_BUTTON
            {
                *p = p.flag("narrow");
            }
        }
        self.build_part_draws(&order);
        let mut cmds = self.commands(&order);
        self.record_matches(&order);
        if let Some(c) = self.cover {
            let mut cover = Vec::new();
            self.cover_commands(&c, &mut cover);
            cmds.extend(cover.into_iter().map(|c| (c, Affine::IDENTITY)));
        }
        let mut over = Vec::new();
        self.hud_edit_commands(&mut over);
        self.virtual_cursor_commands(&mut over);
        cmds.extend(over.into_iter().map(|c| (c, Affine::IDENTITY)));
        let cmds: Vec<(DrawCmd, Affine)> = cmds.into_iter().map(|(c, m)| bake(c, m)).collect();
        let missing_glyphs = self.missing_glyphs(&cmds);
        let motion_dump = self.motion_dump(&order);
        let mut focus_paths: [String; 4] = Default::default();
        let focused = [
            self.state.focus,
            self.state.pad_focus[0],
            self.state.pad_focus[1],
            self.state.pad_focus[2],
        ];
        for (slot, key) in focused.into_iter().enumerate() {
            if key != 0
                && let Some(&i) = present.get(&key)
            {
                focus_paths[slot] = self.path_of(i);
                if let Some(l) = self.nodes[i]
                    .links
                    .iter()
                    .find(|l| child_key(self.nodes[i].key, l) == key)
                {
                    focus_paths[slot] = format!("{}/{l}", focus_paths[slot]);
                }
            }
        }
        self.frame = UiFrame {
            viewport: vp,
            elements,
            parts,
            items: crate::draw::zoom_items(assign_levels(cmds), self.unit),
            warnings: std::mem::take(&mut self.warnings),
            missing_glyphs,
            focus_paths,
            a11y,
            text_input: self.text_target.take(),
            overlaps,
            dense,
            sideways,
            events: std::mem::take(&mut self.events),
            stats: LayoutStats {
                laid_out: self.kept.laid_out,
                ..self.stats
            },
            motion: motion_dump,
            unit: self.unit,
            cursor,
        };
        &self.frame
    }

    /// [`UiFrame::missing_glyphs`] of a frame's commands: read from each
    /// face's metrics on the CPU, so a run with no renderer warns as one
    /// with a GPU does.
    fn missing_glyphs(
        &mut self,
        cmds: &[(DrawCmd, Affine)],
    ) -> Vec<(og_render::FontHandle, Vec<char>)> {
        let mut out: Vec<(og_render::FontHandle, Vec<char>)> = Vec::new();
        for (c, _) in cmds {
            let c = match c {
                DrawCmd::StyledText { text, .. } => &**text,
                c => c,
            };
            let DrawCmd::Text { font, text, .. } = c else {
                continue;
            };
            let Some(face) = self.fonts.face(*font) else {
                continue;
            };
            for ch in text.chars() {
                if ch.is_whitespace()
                    || ch.is_control()
                    || face.metrics.has(ch)
                    || !self.glyphs_missed.insert((*font, ch))
                {
                    continue;
                }
                match out.iter_mut().find(|(f, _)| f == font) {
                    Some((_, chars)) => chars.push(ch),
                    None => out.push((*font, vec![ch])),
                }
            }
        }
        out
    }
}

/// The width (CSS px) under which a button is `[narrow]`.
pub const NARROW_BUTTON: f32 = 160.0;

impl Ui {
    /// Whether a button of `width` is `[narrow]`: its fixed width, else its
    /// width as laid out last tick (none yet: not narrow).
    pub(crate) fn narrow_button(&self, key: u64, width: Size) -> bool {
        match width {
            Size::Fixed(w) => w < NARROW_BUTTON,
            _ => self.hit(key).is_some_and(|h| h.full.width < NARROW_BUTTON),
        }
    }
}
