use super::*;

/// The smallest touch target, CSS px: a control's reach on a touch screen
/// grows to it on each side it is shorter (Apple's HIG 44 pt).
pub(crate) const TOUCH_TARGET: f32 = 44.0;

/// Keeps node `i`'s authored box before placement replaces it for this
/// frame (restored at the next `end`).
fn authored_box(nodes: &mut [Node], mutated: &mut Vec<usize>, i: usize) {
    let n = &mut nodes[i];
    if n.authored_box.is_none() {
        n.authored_box = Some((n.el.width, n.el.height, n.el.padding));
        mutated.push(i);
    }
}

impl Ui {
    /// Normalize freshly assigned presentation before measurement. Call once
    /// after replacing fields, not on already prepared scroll padding.
    pub(crate) fn prepare_layout_node(&mut self, i: usize) {
        let node = &mut self.nodes[i];
        if let Content::Text(text) = &mut node.content
            && self.theme.art.display_min > 0.0
            && text.font == crate::FontRole::Display
            && text.step.size < self.theme.art.display_min
        {
            text.font = crate::FontRole::Body;
        }
        if !matches!(node.widget, Widget::Field(_) | Widget::TextArea { .. }) {
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
        crate::layout::snap_pixel_text(&mut self.nodes[i..i + 1], &self.fonts);
    }

    /// A vertical scroller's bar gutter on its right padding, as its last
    /// frame's range needs it: replaced, never added again, so a retained
    /// scroller that is not built again keeps one gutter.
    pub(crate) fn apply_gutter(&mut self, i: usize) {
        let node = &self.nodes[i];
        let gutter = if node.el.scroll && !node.el.scroll_x && !node.el.hide_bars {
            self.bar_gutter(node.key)
        } else {
            0.0
        };
        let node = &mut self.nodes[i];
        node.el.padding.right += gutter - node.gutter;
        node.gutter = gutter;
    }

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
        if let Err(error) = self.end_with_layout_queries(Ui::settle_document_containers) {
            self.warnings.push(error.to_string());
        }
        &self.frame
    }

    pub(crate) fn end_with_layout_queries(
        &mut self,
        mut resolve: impl FnMut(
            &mut Ui,
            &[settlement::ContainerMeasurement],
        ) -> Result<Vec<usize>, settlement::SettlementError>,
    ) -> Result<&UiFrame, settlement::SettlementError> {
        #[cfg(debug_assertions)]
        if self.building && !self.kept.whole && bounded::checking() {
            let mut twin = self.clone();
            twin.kept.whole = true;
            let whole = twin.end_frame(&mut resolve).is_ok();
            let bounded = self.end_frame(&mut resolve).map(|_| ());
            assert_eq!(
                whole,
                bounded.is_ok(),
                "the bounded frame settled otherwise"
            );
            if bounded.is_ok() {
                bounded::compare(self, &twin);
            }
            return bounded.map(|()| &self.frame);
        }
        self.end_frame(&mut resolve)
    }

    fn end_frame(
        &mut self,
        resolve: &mut impl FnMut(
            &mut Ui,
            &[settlement::ContainerMeasurement],
        ) -> Result<Vec<usize>, settlement::SettlementError>,
    ) -> Result<&UiFrame, settlement::SettlementError> {
        if !self.building {
            self.warnings
                .push("end without begin: call ui.begin() at the start of the tick".into());
            return Ok(&self.frame);
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
        self.retire_nodes();
        self.building = false;
        // A retained node placement resized last frame lays out from its
        // authored box again (a written node is authored afresh).
        for i in std::mem::take(&mut self.layout_mutated) {
            if let Some((width, height, padding)) = self.nodes[i].authored_box.take() {
                let el = &mut self.nodes[i].el;
                (el.width, el.height, el.padding) = (width, height, padding);
                self.kept.restored.push(i);
            }
        }
        // Breakpoint and orientation variants (docs/ui.md §2), before
        // anything is laid out: on the nodes written this frame (a retained
        // node keeps what they set; a new viewport emits it again).
        self.apply_variants();
        self.apply_part_appearances();
        // Hidden slots (a closed dialog, a finished transition) take no room.
        // A node's lists follow from its built children and their own
        // props: derived again where those were written or a child left.
        let whole = self.kept.whole
            || self.kept.epoch != self.kept.prev_epoch
            || self.kept.sig != Some((self.theme, self.direction, self.viewport.dpr()));
        let derive: Vec<usize> = if whole {
            (0..self.nodes.len()).collect()
        } else {
            let mut d: Vec<usize> = std::iter::once(0)
                .chain(self.emitted.iter().flat_map(|&i| [i, self.nodes[i].parent]))
                .chain(self.kept.rederive.iter().copied())
                .filter(|&i| i < self.nodes.len())
                .collect();
            d.sort_unstable();
            d.dedup();
            d
        };
        for &i in &derive {
            if self.nodes[i].hidden {
                continue;
            }
            let kept: Vec<usize> = self.nodes[i]
                .built_children
                .iter()
                .copied()
                .filter(|&c| !self.nodes[c].hidden && self.nodes[c].parent == i)
                .collect();
            // Out of the flow (absolute, anchored): laid out against the box.
            let (out, flow): (Vec<usize>, Vec<usize>) = kept
                .iter()
                .partition(|&&c| self.nodes[c].props.out_of_flow());
            let n = &mut self.nodes[i];
            n.ordered_children = kept;
            n.children = flow;
            n.out = out;
        }
        for k in 0..self.emitted.len() {
            self.prepare_layout_node(self.emitted[k]);
        }
        // A gutter follows its node's props and last frame's scroll range.
        if whole {
            for i in 0..self.nodes.len() {
                self.apply_gutter(i);
            }
        } else {
            for k in 0..self.emitted.len() {
                self.apply_gutter(self.emitted[k]);
            }
            for k in 0..self.kept.scrollers.len() {
                let i = self.kept.scrollers[k];
                if i < self.nodes.len() {
                    self.apply_gutter(i);
                }
            }
        }
        self.reuse_check();
        let mut floating: Vec<usize> = self
            .floating
            .iter()
            .copied()
            .filter(|&f| !self.nodes[f].hidden)
            .collect();
        og_core::sort::sort_by_key(&mut floating, |&f| self.nodes[f].float.rank());
        self.floating = floating;
        let written = std::mem::take(&mut self.emitted);
        self.tree.restructured.clear();
        self.tree.all_restructured = false;
        self.sync_tree(&derive, &written, whole);
        self.emitted = written;

        let vp = self.viewport;
        let dpr = vp.dpr();
        let screen = Rect::new(0.0, 0.0, vp.width, vp.height);
        let rtl = self.direction == og_i18n::Direction::Rtl;
        self.kept.query_roots = None;
        let mut settlement = settlement::Settlement::default();
        loop {
            let mut layout_inputs = settlement::LayoutInputs::new(&self.kept.measurements);
            let scroll = self.state.scroll.clone();
            let scroll_x = self.state.scroll_x.clone();
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
                            spots[k].1 =
                                keep_out(r, anchor, covered, margin.max(gap), hud, &others);
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
                        layout_inputs.remember(g, &self.nodes[g]);
                        authored_box(&mut self.nodes, &mut self.layout_mutated, g);
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
                    layout_inputs.remember(f, &self.nodes[f]);
                    authored_box(&mut self.nodes, &mut self.layout_mutated, f);
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
                layout_inputs.remember(f, &self.nodes[f]);
                authored_box(&mut self.nodes, &mut self.layout_mutated, f);
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
                layout_inputs.remember(i, &self.nodes[i]);
                authored_box(&mut self.nodes, &mut self.layout_mutated, i);
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
            match self.settle_layout_queries(&mut settlement, layout_inputs, &mut *resolve) {
                Ok(true) => {}
                Ok(false) => break,
                Err(error) => {
                    self.abort_layout_queries();
                    self.inspection = inspect::Inspection::none();
                    return Err(error);
                }
            }
        }
        self.kept.query_roots = None;
        if !self.tree.pending.is_empty() {
            self.sync_tree(&[], &[], false);
        }
        self.texts.sweep();

        let mut order = preorder(&self.nodes, 0);
        for &f in &self.floating {
            order.extend(preorder(&self.nodes, f));
        }
        self.stats.elements = order.len() as u32;
        self.align_baselines(&order);
        self.place_anchored(&order);
        for &i in &order {
            self.nodes[i].sample_style();
        }
        if self.direction == og_i18n::Direction::Rtl {
            // Right to left: the whole layout mirrored about the surface's
            // centre, and with it each transform (moves and turns the other way).
            // A node layout left out is mirrored as it was last frame.
            for k in 0..self.kept.laid.len() {
                let i = self.kept.laid[k];
                let r = &mut self.nodes[i].rect;
                r.x = vp.width - r.x - r.width;
                // Its clip too (a scroll container's box, laid out left to right): unmirrored, it
                // would cut away the content of a scroller off the surface's centre line.
                let c = &mut self.nodes[i].clip;
                c.x = vp.width - c.x - c.width;
            }
            for &i in &order {
                let t = self.nodes[i].sampled.transform;
                if !t.is_identity() {
                    self.nodes[i].sampled.transform = t.mirrored();
                }
            }
        }
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

        // Interaction state for the next tick: what is on screen is what
        // the trees' index holds (`tree`), a rich text's links with it.
        #[cfg(debug_assertions)]
        if bounded::checking() {
            bounded::compare_tree(self, &order);
        }
        let present = |k: &u64| self.tree.find(*k);
        self.state
            .scroll
            .retain(|k, _| present(k).is_some_and(|i| self.nodes[i].el.scroll));
        self.state.virtual_lists.retain(|k, _| present(k).is_some());
        for (k, v) in self.state.scroll.iter_mut() {
            *v = v.clamp(0.0, present(k).map_or(0.0, |i| self.nodes[i].max_scroll()));
        }
        self.state
            .scroll_x
            .retain(|k, _| present(k).is_some_and(|i| self.nodes[i].el.scroll_x));
        for (k, v) in self.state.scroll_x.iter_mut() {
            *v = v.clamp(
                0.0,
                present(k).map_or(0.0, |i| self.nodes[i].max_scroll_x()),
            );
        }
        // Each scroll container's ranges and bar lengths, as drawn.
        self.kept.scrollers = order
            .iter()
            .copied()
            .filter(|&i| self.nodes[i].scrolls())
            .collect();
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
            && self.tree.find(k).is_none()
        {
            self.state.fling = None;
        }
        if !self.input.down || self.tree.find(self.state.active).is_none() {
            self.state.active = 0;
        }
        let touched = std::mem::take(&mut self.touched);
        let retained = &self.docs.retained;
        self.state
            .anims
            .retain(|k, _| touched.contains(k) || retained.keeps_memory(*k));
        self.state
            .screens
            .retain(|k, _| touched.contains(k) || retained.keeps_memory(*k));
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
        let mut top_scope = 0u64;
        let mut top_locked = false;
        // The minimum touch target, in og-ui units (44 CSS px), on a touch screen.
        let touch_min = self.touch_min();
        for &i in &order[1..] {
            let n = &self.nodes[i];
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
            if flags == 0 && n.links.is_empty() {
                continue;
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
            if outside || (visible.is_empty() && !r.is_empty()) {
                // Navigation can still reach a control just out of a list's view.
                if n.nav && !n.inert && scroller_of[i] != 0 {
                    hits.push(hit(*r, (flags & (HIT_NAV | HIT_ADJUST)) | HIT_OFFSCREEN));
                }
                continue;
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
            }
        }
        // Focus: kept while its control is on screen (or reachable); a scope
        // that opened takes it (its first control, or the one a widget asked
        // for); a scope that closed gives it back.
        let alive = |k: u64| {
            hits.iter().any(|h: &Hit| h.key == k)
                || self.docs.retained.deferred_focus_path(k).is_some()
        };
        let first_in = |scope: u64| {
            hits.iter()
                .find(|h| h.scope == scope && h.has(HIT_NAV) && !h.has(HIT_OFFSCREEN))
                .map_or(0, |h| h.key)
        };
        if self.request_focus != 0 && hits.iter().any(|h| h.key == self.request_focus) {
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
        self.state.hits = hits;
        self.note_groups();
        self.note_taps();
        let cursor = if self.state.vcursor.shown && self.state.vcursor.desc.is_some() {
            // The pad's cursor is drawn; the system's is hidden.
            crate::Cursor::None
        } else {
            self.resolve_cursor()
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
        let trees = self.commands(&order);
        self.record_matches(&order);
        let mut tail = Vec::new();
        if let Some(c) = self.cover {
            let mut cover = Vec::new();
            self.cover_commands(&c, &mut cover);
            tail.extend(cover.into_iter().map(|c| (c, Affine::IDENTITY)));
        }
        let mut over = Vec::new();
        self.hud_edit_commands(&mut over);
        self.virtual_cursor_commands(&mut over);
        tail.extend(over.into_iter().map(|c| (c, Affine::IDENTITY)));
        // Every tree drew as last frame, and what is over them too: the
        // frame's items are last frame's.
        let kept_items =
            self.kept.reused_all && tail == self.kept.last_tail && self.frame.unit == self.unit;
        let (items, missing_glyphs) = if kept_items {
            (std::mem::take(&mut self.frame.items), Vec::new())
        } else {
            let mut cmds = trees.clone();
            cmds.extend(tail.iter().cloned());
            let cmds: Vec<(DrawCmd, Affine)> = cmds.into_iter().map(|(c, m)| bake(c, m)).collect();
            let missing_glyphs = self.missing_glyphs(&cmds);
            (
                crate::draw::zoom_items(assign_levels(cmds), self.unit),
                missing_glyphs,
            )
        };
        self.kept.last_cmds = trees;
        self.kept.last_tail = tail;
        let mut focus_paths: [String; 4] = Default::default();
        let focused = [
            self.state.focus,
            self.state.pad_focus[0],
            self.state.pad_focus[1],
            self.state.pad_focus[2],
        ];
        for (slot, key) in focused.into_iter().enumerate() {
            if key != 0
                && let Some(i) = self.tree.find(key)
            {
                focus_paths[slot] = self.path_of(i);
                if let Some(l) = self.nodes[i]
                    .links
                    .iter()
                    .find(|l| child_key(self.nodes[i].key, l) == key)
                {
                    focus_paths[slot] = format!("{}/{l}", focus_paths[slot]);
                }
            } else if let Some(path) = self.docs.retained.deferred_focus_path(key) {
                focus_paths[slot] = path.to_owned();
            }
        }
        self.frame = UiFrame {
            viewport: vp,
            items,
            warnings: std::mem::take(&mut self.warnings),
            missing_glyphs,
            focus_paths,
            text_input: self.text_target.take(),
            events: std::mem::take(&mut self.events),
            stats: LayoutStats {
                laid_out: self.kept.laid_out,
                ..self.stats
            },
            unit: self.unit,
            cursor,
        };
        self.inspection = Default::default();
        Ok(&self.frame)
    }

    /// [`UiFrame::missing_glyphs`] of a frame's commands: read from each
    /// face's metrics on the CPU, so a run with no renderer warns as one
    /// with a GPU does. Commands kept from last frame (`kept_spans`) were
    /// read when they were painted.
    fn missing_glyphs(
        &mut self,
        cmds: &[(DrawCmd, Affine)],
    ) -> Vec<(og_render::FontHandle, Vec<char>)> {
        let mut out: Vec<(og_render::FontHandle, Vec<char>)> = Vec::new();
        let kept = std::mem::take(&mut self.kept_spans);
        let mut spans = kept.iter().peekable();
        for (at, (c, _)) in cmds.iter().enumerate() {
            while spans.next_if(|r| r.end <= at).is_some() {}
            if spans.peek().is_some_and(|r| r.contains(&at)) {
                continue;
            }
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
        self.kept_spans = kept;
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
