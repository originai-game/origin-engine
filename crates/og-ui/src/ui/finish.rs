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

    /// [`Ui::end`], settling container queries with `resolve`. Under the
    /// bounded frame's check (`OG_UI_CHECK_BOUNDED`) a whole twin of the
    /// frame is finished too and `resolve` settles it as well: it is called
    /// once per finish, so effects outside the `Ui` it is given happen
    /// twice there.
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

        // The trees' orders and features, kept where nothing changed shape.
        self.update_orders(whole);
        let order = std::mem::take(&mut self.orders.pre);
        self.stats.elements = order.len() as u32;
        let baselines = self.orders.with(visit::feat::BASELINE).to_vec();
        self.align_baselines(&baselines);
        let anchored = self.orders.with(visit::feat::ANCHORED).to_vec();
        self.place_anchored(&anchored);
        // What may change this frame (`visit`): every node on a fresh one.
        let (mut seeds, restyle) = match whole {
            true => (None, None),
            false => {
                let (seeds, restyle) = self.frame_seeds();
                (Some(seeds), Some(restyle))
            }
        };
        let sampled: &[usize] = restyle.as_deref().unwrap_or(&order);
        for &i in sampled {
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
            for &i in sampled {
                let t = self.nodes[i].sampled.transform;
                if !t.is_identity() {
                    self.nodes[i].sampled.transform = t.mirrored();
                }
            }
        }
        self.apply_shakes(&order);
        let staggered = self.orders.with(visit::feat::STAGGER).to_vec();
        self.apply_stagger(&staggered);
        let animated = self.run_motion(restyle.as_deref());
        self.stats.animated = animated.len() as u32;
        // What motion wrote is placed and painted again.
        if let Some(seeds) = seeds.as_mut() {
            seeds.extend_from_slice(&animated);
        }
        self.apply_bursts(&order);
        self.world_place();
        self.orders.pre = order;
        // Drawn and hit in z order among siblings.
        self.update_draw_order(&animated);
        #[cfg(debug_assertions)]
        if bounded::checking() {
            self.check_orders();
        }
        let order = std::mem::take(&mut self.orders.draw);
        let custom = self.orders.drawn(visit::feat::CUSTOM);
        self.hud_apply(&custom);
        // Maps, opacity and inertness where they may have changed, and
        // below wherever they did.
        let world_seeds = seeds.as_ref().map(|s| {
            let mut w = s.clone();
            w.extend_from_slice(&self.orders.wrote);
            w.extend_from_slice(&custom);
            w
        });
        let mapped = self.apply_world(&order, world_seeds.as_deref());
        self.hud_collect(&custom);
        self.world_finish();
        let dnd = self.orders.with(visit::feat::DND).to_vec();
        self.note_drag_and_drop(&dnd);

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
        self.kept.scrollers = self.orders.drawn(visit::feat::SCROLLS);
        let min_bar = self.theme.space.xl;
        self.state.scroll_info = IdMap::from_iter(self.kept.scrollers.iter().map(|&i| {
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
        }));
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

        // Scopes, scrollers, focus owners, groups and recognisers around
        // each node: again where they may have changed, and below wherever
        // they did (a layer takes its anchor's).
        let scope_seeds = seeds.as_ref().map(|s| {
            let mut w = mapped.clone();
            w.extend_from_slice(s);
            w.extend_from_slice(&self.floating);
            w
        });
        let scoped = self.visit(&order, scope_seeds.as_deref(), Self::scope_step);
        // Focus groups, explicit moves and gesture recognisers for the next frame.
        self.collect_input_specs();
        // Hits where a node's place, map or scopes may have changed (every
        // node when the touch target or the screen did).
        let touch_min = self.touch_min();
        let hit_seeds = match &seeds {
            Some(_) if self.orders.hit_inputs == Some((touch_min, screen)) => {
                let mut h = mapped.clone();
                h.extend(scoped);
                Some(h)
            }
            _ => None,
        };
        self.orders.hit_inputs = Some((touch_min, screen));
        let mut hits_changed =
            hit_seeds.is_none() || self.kept.fresh || std::mem::take(&mut self.orders.hits_moved);
        let hit_seeds = hit_seeds.unwrap_or_else(|| order.clone());
        for i in hit_seeds {
            if self.orders.pos[i] != u32::MAX {
                hits_changed |= self.hit_step(i, touch_min, screen);
            }
        }
        let o = &self.orders;
        if hits_changed || self.state.hits.is_empty() {
            self.state.hits = o
                .hit_nodes
                .iter()
                .flat_map(|&i| o.hit_of[i].iter().copied())
                .collect();
        }
        self.warnings
            .extend(o.hit_nodes.iter().filter_map(|&i| o.warn_of[i].clone()));
        let hits = std::mem::take(&mut self.state.hits);
        // The topmost scope open (not leaving).
        let (top_scope, top_locked) = o
            .drawn(visit::feat::SCOPE)
            .into_iter()
            .rev()
            .filter(|&i| i != order[0] && !self.nodes[i].inert)
            .map(|i| (self.nodes[i].key, self.nodes[i].locked))
            .next()
            .unwrap_or((0, false));
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
        for k in 0..self.orders.with(visit::feat::BUTTON_PART).len() {
            let i = self.orders.with(visit::feat::BUTTON_PART)[k];
            let n = &mut self.nodes[i];
            if let Some(p) = n.el.part.as_mut()
                && n.rect.width < NARROW_BUTTON
            {
                *p = p.flag("narrow");
            }
        }
        // Parts a style dresses, and game draws it may restyle.
        let mut parts = self.orders.with(visit::feat::PART).to_vec();
        parts.extend(self.draws.keys().copied().filter(|&i| self.tree.has(i)));
        self.build_part_draws(&parts);
        // Painted again where a node's drawing may have changed (every
        // node on a fresh frame).
        // The selection's ranges on its elements: those whose range
        // changed are painted again.
        let selected = self.mark_selection();
        let painted = if selected.is_empty() {
            mapped
        } else {
            [mapped, selected].concat()
        };
        let trees = self.commands(&order, seeds.is_some().then_some(&painted[..]));
        #[cfg(debug_assertions)]
        if bounded::checking() {
            for &i in &order {
                let n = &self.nodes[i];
                assert_eq!(
                    self.kept.sigs.get(i).copied().flatten(),
                    Some((n.key, crate::layout::DrawSig::of(n))),
                    "'{}' paints with another signature than it was kept by",
                    self.path_of(i)
                );
            }
        }
        let repainted = self.kept.repainted.take();
        self.note_semantics(repainted.as_deref());
        let matched = self.orders.with(visit::feat::MATCH).to_vec();
        self.record_matches(&matched);
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
        } else if let Some(relevelled) = self.relevel(&trees, &tail) {
            relevelled
        } else {
            let mut cmds = trees.clone();
            cmds.extend(tail.iter().cloned());
            let cmds: Vec<(DrawCmd, Affine)> = cmds.into_iter().map(|(c, m)| bake(c, m)).collect();
            let missing_glyphs = self.missing_glyphs(&cmds);
            let (items, levels) = crate::draw::assign_levels_kept(cmds);
            self.kept.levels = Some(levels);
            (crate::draw::zoom_items(items, self.unit), missing_glyphs)
        };
        self.kept.prev_cmds = Vec::new();
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
        self.orders.draw = order;
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
    /// Last frame's items with only the commands that differ from last
    /// frame's replaced and levelled again ([`crate::draw::Levels`]): the
    /// commands outside the ranges kept in place are compared, not every
    /// command. `None` when the list's shape changed or much of it did.
    fn relevel(
        &mut self,
        trees: &[(DrawCmd, Affine)],
        tail: &[(DrawCmd, Affine)],
    ) -> Option<(Vec<DrawItem>, MissingGlyphs)> {
        let prev = std::mem::take(&mut self.kept.prev_cmds);
        if self.kept.levels.is_none()
            || self.kept.aligned.len() != self.kept.aligned_kept
            || self.frame.unit != self.unit
            || trees.len() != self.kept.prev_len
            || tail.len() != self.kept.last_tail.len()
        {
            return None;
        }
        let total = trees.len() + tail.len();
        let mut changed = Vec::new();
        let mut at = 0;
        let gaps = self
            .kept
            .aligned
            .iter()
            .map(|r| (r.start, r.end))
            .chain([(trees.len(), trees.len())]);
        for (start, end) in gaps {
            if start > at {
                let was = prev.get(at..start)?;
                for (k, (now, was)) in trees[at..start].iter().zip(was).enumerate() {
                    if now != was {
                        changed.push(at + k);
                    }
                }
            }
            at = at.max(end);
        }
        for (k, (now, was)) in tail.iter().zip(&self.kept.last_tail).enumerate() {
            if now != was {
                changed.push(trees.len() + k);
            }
        }
        if changed.len() * 8 > total {
            return None;
        }
        let baked: Vec<(DrawCmd, Affine)> = changed
            .iter()
            .map(|&p| {
                let (c, m) = trees.get(p).unwrap_or_else(|| &tail[p - trees.len()]);
                bake(c.clone(), *m)
            })
            .collect();
        self.kept_spans.clear();
        let missing_glyphs = self.missing_glyphs(&baked);
        let mut items = std::mem::take(&mut self.frame.items);
        let replaced = changed
            .into_iter()
            .zip(baked)
            .map(|(p, (c, m))| (p, c, m))
            .collect();
        let levels = self.kept.levels.as_mut().expect("checked");
        if !levels.update(&mut items, replaced, self.unit) {
            self.frame.items = items;
            return None;
        }
        #[cfg(debug_assertions)]
        if bounded::checking() {
            let mut cmds: Vec<(DrawCmd, Affine)> = trees.to_vec();
            cmds.extend(tail.iter().cloned());
            let cmds: Vec<_> = cmds.into_iter().map(|(c, m)| bake(c, m)).collect();
            let fresh = crate::draw::zoom_items(crate::draw::assign_levels(cmds), self.unit);
            assert!(
                fresh == items,
                "the frame's items levelled again differ from levelling it afresh"
            );
        }
        Some((items, missing_glyphs))
    }

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

/// Per font, the characters it lacks that a frame drew for the first time.
type MissingGlyphs = Vec<(og_render::FontHandle, Vec<char>)>;
