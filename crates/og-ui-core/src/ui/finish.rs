use super::*;

/// The smallest touch target, CSS px: a control's reach on a touch screen
/// grows to it on each side it is shorter (Apple's HIG 44 pt).
pub(crate) const TOUCH_TARGET: f32 = 44.0;

/// Fixes node `i`'s used box at the size placement gave it, with `padding`
/// added, until the next layout pass measures it again
/// ([`crate::ui::Layout::unplace`]).
fn place_box(
    nodes: &mut [Node],
    placed: &mut Vec<usize>,
    i: usize,
    size: [f32; 2],
    padding: Insets,
) {
    let n = &mut nodes[i];
    if n.layout.placement.is_none() {
        placed.push(i);
    }
    n.layout.placement = Some(crate::layout::Placement { size, padding });
    n.derive_used();
}

/// Whether an element around node `i` skips its contents
/// (`content-visibility`).
fn skipped(nodes: &[Node], i: usize) -> bool {
    let mut at = nodes[i].up;
    while at != NONE && at < nodes.len() {
        if nodes[at].layout.cv_skip {
            return true;
        }
        at = nodes[at].up;
    }
    false
}

impl Ui {
    /// Lays out the frame, updates the interaction state and returns the frame.
    pub fn end(&mut self) -> &UiFrame {
        if let Err(error) = self.end_with_layout_queries(Ui::settle_document_containers) {
            self.out.warnings.push(error.to_string());
        }
        &self.out.frame
    }

    /// [`Ui::end`], settling container queries with `resolve`. Under the
    /// bounded frame's check (`OG_UI_CHECK_BOUNDED`) a whole twin of the
    /// frame is finished too and `resolve` settles it as well: it is called
    /// once per finish, so effects outside the `Ui` it is given happen
    /// twice there.
    pub fn end_with_layout_queries(
        &mut self,
        mut resolve: impl FnMut(
            &mut Ui,
            &[settlement::ContainerMeasurement],
        ) -> Result<Vec<usize>, settlement::SettlementError>,
    ) -> Result<&UiFrame, settlement::SettlementError> {
        #[cfg(debug_assertions)]
        if self.tree.building && !self.tree.invalidation.whole && bounded::checking() {
            let mut twin = self.clone();
            twin.tree.invalidation.whole = true;
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
            return bounded.map(|()| &self.out.frame);
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
        if !self.tree.building {
            self.out
                .warnings
                .push("end without begin: call ui.begin() at the start of the tick".into());
            return Ok(&self.out.frame);
        }
        let whole = self.finish_build();

        let vp = self.env.viewport;
        let dpr = vp.dpr();
        let screen = Rect::new(0.0, 0.0, vp.width, vp.height);
        let rtl = self.env.direction == og_i18n::Direction::Rtl;
        self.lay_out(vp, dpr, screen, rtl, &mut *resolve)?;
        self.tree.place_popover_arrows();
        self.layout.queries.query_roots = None;
        if !self.tree.index.pending.is_empty() {
            self.tree.sync_tree(&[], &[], false);
        }
        self.layout.texts.sweep();

        // The trees' orders and features, kept where nothing changed shape.
        self.tree.update_orders(whole, &self.layout);
        let order = std::mem::take(&mut self.tree.orders.pre);
        self.out.stats.elements = order.len() as u32;
        let baselines = self.tree.orders.with(visit::feat::BASELINE).to_vec();
        self.layout
            .align_baselines(&baselines, &self.env, &mut self.tree);
        let anchored = self.tree.orders.with(visit::feat::ANCHORED).to_vec();
        self.layout
            .place_anchored(&anchored, &mut self.out, &mut self.tree);
        let (seeds, animated) = self.frame_motion(&order, whole, vp);
        let (order, mapped) = self.place_frame(order, &seeds, &animated);

        let (cursor, cursor_images, selected) =
            self.update_interaction(&order, &seeds, &mapped, screen);

        let (order, items, missing_glyphs) = self.record_paint(order, &seeds, mapped, selected);
        let focus_paths = self
            .layout
            .focus_paths(&self.docs, &self.interaction, &self.tree);
        self.tree.orders.draw = order;
        self.layout
            .note_relevance(&self.env, &self.interaction, &mut self.tree);
        self.out.frame = UiFrame {
            viewport: vp,
            items,
            warnings: std::mem::take(&mut self.out.warnings),
            missing_glyphs,
            focus_paths,
            text_input: self.interaction.text_target.take(),
            events: std::mem::take(&mut self.out.events),
            grounds: std::mem::take(&mut self.world.placement.grounds),
            stats: LayoutStats {
                laid_out: self.layout.cache.laid_out,
                ..self.out.stats
            },
            unit: self.env.unit,
            cursor,
            cursor_images,
        };
        self.paint.inspection = Default::default();
        Ok(&self.out.frame)
    }

    /// Closes the build: unclosed elements, layers, toasts and retired
    /// nodes, then variants and appearances, each node's lists, gutters,
    /// content visibility, layout reuse and the trees' index. Whether
    /// nothing is kept from last frame.
    fn finish_build(&mut self) -> bool {
        while self.tree.stack.len() > 1 {
            let i = self.tree.stack.pop().unwrap_or(0);
            self.out.warnings.push(format!(
                "'{}' was not closed; every ui.open() takes one ui.close()",
                self.tree.path_of(i)
            ));
        }
        if let Some(documents) = crate::doc::build::documents() {
            documents.build(self);
        }
        self.build_toasts();
        self.tree.retire_nodes();
        self.tree.building = false;
        // A retained node placement resized last frame lays out from its
        // own box again (a written node starts without a placement).
        let restored = self.layout.unplace(&mut self.tree);
        self.tree.invalidation.restored.extend(restored);
        // Breakpoint and orientation variants (docs/ui.md §2), before
        // anything is laid out: on the nodes written this frame (a retained
        // node keeps what they set; a new viewport emits it again).
        self.layout.apply_variants(&self.env, &mut self.tree);
        self.apply_part_appearances();
        self.tree.apply_resized(&mut self.interaction);
        // The written nodes' inputs are complete: what layout uses of them.
        for k in 0..self.tree.emitted.len() {
            let i = self.tree.emitted[k];
            self.tree.nodes[i].derive_used();
        }
        // Hidden slots (a closed dialog, a finished transition) take no room.
        // A node's lists follow from its built children and their own
        // props: derived again where those were written or a child left.
        let whole = self.tree.invalidation.whole
            || self.tree.invalidation.epoch != self.tree.invalidation.prev_epoch
            || self.tree.invalidation.sig
                != Some((self.env.theme, self.env.direction, self.env.viewport.dpr()));
        let derive: Vec<usize> = if whole {
            (0..self.tree.nodes.len()).collect()
        } else {
            let mut d: Vec<usize> = std::iter::once(0)
                .chain(
                    self.tree
                        .emitted
                        .iter()
                        .flat_map(|&i| [i, self.tree.nodes[i].parent]),
                )
                .chain(self.tree.invalidation.rederive.iter().copied())
                .filter(|&i| i < self.tree.nodes.len())
                .collect();
            // A box that hands its children on (`display: contents`, a
            // table's rows and groups) derives with the box it hands them to.
            let handed = |n: &crate::layout::Node| {
                use crate::layout::TablePart as T;
                n.el.contents
                    || !matches!(n.el.table, T::None | T::Table | T::Cell | T::Caption { .. })
            };
            for k in 0..d.len() {
                let mut at = d[k];
                while at < self.tree.nodes.len()
                    && self.tree.nodes[at].parent != NONE
                    && (handed(&self.tree.nodes[at])
                        || handed(&self.tree.nodes[self.tree.nodes[at].parent]))
                {
                    at = self.tree.nodes[at].parent;
                    d.push(at);
                }
            }
            d.sort_unstable();
            d.dedup();
            d
        };
        for &i in &derive {
            if self.tree.nodes[i].is_hidden() {
                continue;
            }
            use crate::layout::TablePart;
            // Its children are laid out by another box: it has none.
            let n = &self.tree.nodes[i];
            let in_table = |p: usize| {
                p != NONE
                    && (self.tree.nodes[p].el.table == TablePart::Table
                        || matches!(
                            self.tree.nodes[p].el.table,
                            TablePart::Header | TablePart::Body | TablePart::Footer
                        ) && self.tree.nodes[p].parent != NONE
                            && self.tree.nodes[self.tree.nodes[p].parent].el.table
                                == TablePart::Table)
            };
            let handed = i != 0
                && (n.el.contents
                    || matches!(
                        n.el.table,
                        TablePart::Header
                            | TablePart::Body
                            | TablePart::Footer
                            | TablePart::Row
                            | TablePart::Column
                            | TablePart::ColumnGroup
                    ) && in_table(n.parent));
            if handed {
                let n = &mut self.tree.nodes[i];
                n.ordered_children.clear();
                n.children.clear();
                n.out.clear();
                n.floats.clear();
                n.layout.table = None;
                continue;
            }
            let shown = crate::layout::shown_children(&self.tree.nodes, i);
            // Out of the flow (absolute, anchored): laid out against the box.
            let (out, inside): (Vec<usize>, Vec<usize>) = shown
                .iter()
                .partition(|&&c| self.tree.nodes[c].props.out_of_flow());
            // A table draws its rows and groups under their cells, places
            // them itself, and lays out its captions and cells by slot.
            let (kept, mut flow, table) = if self.tree.nodes[i].el.table == TablePart::Table {
                let (mut draw, flow, t) = crate::layout::derive_table(&self.tree.nodes, &inside);
                draw.extend(&out);
                (draw, flow, Some(t))
            } else {
                (shown, inside, None)
            };
            // A block's floated children leave its flow (CSS 2 §9.5); in
            // a flex, grid or multi-column container `float` does not apply.
            let floats: Vec<usize> = if crate::layout::float::container(&self.tree.nodes[i]) {
                let floats = flow
                    .iter()
                    .copied()
                    .filter(|&c| self.tree.nodes[c].el.float != crate::layout::FloatSide::None)
                    .collect();
                flow.retain(|&c| self.tree.nodes[c].el.float == crate::layout::FloatSide::None);
                floats
            } else {
                Vec::new()
            };
            for &c in &kept {
                self.tree.nodes[c].up = i;
            }
            // A block holding inline-level children lays them out in lines.
            self.tree.line_box(i, &flow);
            // Flex and grid items lay out in CSS `order`, then as built.
            if table.is_none() && flow.iter().any(|&c| self.tree.nodes[c].el.order != 0) {
                og_core::sort::sort_by_key(&mut flow, |&c| self.tree.nodes[c].el.order);
            }
            let baseline_items = flow.iter().any(|&c| {
                matches!(
                    self.tree.nodes[c].el.align_self,
                    Some(crate::layout::Cross::Baseline | crate::layout::Cross::LastBaseline)
                )
            });
            let n = &mut self.tree.nodes[i];
            n.ordered_children = kept;
            n.children = flow;
            n.out = out;
            n.floats = floats;
            n.layout.baseline_items = baseline_items;
            n.layout.table = table;
        }
        for k in 0..self.tree.emitted.len() {
            self.layout
                .prepare_layout_node(self.tree.emitted[k], &self.env, &mut self.tree);
        }
        // A gutter follows its node's props and last frame's scroll range.
        if whole {
            for i in 0..self.tree.nodes.len() {
                self.layout
                    .apply_gutter(i, &self.env, &self.interaction, &mut self.tree);
            }
        } else {
            for k in 0..self.tree.emitted.len() {
                self.layout.apply_gutter(
                    self.tree.emitted[k],
                    &self.env,
                    &self.interaction,
                    &mut self.tree,
                );
            }
            for k in 0..self.layout.cache.scrollers.len() {
                let i = self.layout.cache.scrollers[k];
                if i < self.tree.nodes.len() {
                    self.layout
                        .apply_gutter(i, &self.env, &self.interaction, &mut self.tree);
                }
            }
        }
        self.layout.content_visibility(&mut self.tree);
        self.layout
            .reuse_check(&self.env, &mut self.out, &self.docs, &mut self.tree);
        let mut floating: Vec<usize> = self
            .tree
            .floating
            .iter()
            .copied()
            .filter(|&f| !self.tree.nodes[f].is_hidden())
            .collect();
        og_core::sort::sort_by_key(&mut floating, |&f| self.tree.nodes[f].float.rank());
        self.tree.floating = floating;
        let written = std::mem::take(&mut self.tree.emitted);
        self.tree.index.restructured.clear();
        self.tree.index.all_restructured = false;
        self.tree.sync_tree(&derive, &written, whole);
        self.tree.emitted = written;
        whole
    }

    /// Lays the trees out, settling container queries until they hold.
    fn lay_out(
        &mut self,
        vp: Viewport,
        dpr: f32,
        screen: Rect,
        rtl: bool,
        resolve: &mut impl FnMut(
            &mut Ui,
            &[settlement::ContainerMeasurement],
        ) -> Result<Vec<usize>, settlement::SettlementError>,
    ) -> Result<(), settlement::SettlementError> {
        self.layout.queries.query_roots = None;
        let mut settlement = settlement::Settlement::default();
        loop {
            // A settlement's next pass measures the placed boxes again.
            self.layout.unplace(&mut self.tree);
            let scroll = self.interaction.state.scroll.clone();
            let scroll_x = self.interaction.state.scroll_x.clone();
            let offset = |k: u64| {
                (
                    scroll_x.get(&k).copied().unwrap_or(0.0),
                    scroll.get(&k).copied().unwrap_or(0.0),
                )
            };
            let mut measure = Measure {
                fonts: &self.env.fonts,
                texts: &mut self.layout.texts,
                body: self.env.theme.type_ramp.body,
            };
            layout(
                &mut self.tree.nodes,
                0,
                screen,
                &mut measure,
                &offset,
                dpr,
                screen,
                &mut self.layout.cache,
                &mut self.layout.queries,
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
            let hud = vp.hud_rect(self.env.hud_max_aspect);
            // The host's control, where the layout stands before a right-to-left
            // mirror (which moves a top-left group to the top right).
            let covered = {
                let c = vp.covered;
                if self.env.direction == og_i18n::Direction::Rtl {
                    Rect::new(vp.width - c.x - c.width, c.y, c.width, c.height)
                } else {
                    c
                }
            };
            let gap = self.env.theme.space.xs;
            // The anchored groups first, each laid out in its area and placed
            // once all of them are known (so one the host's control moves keeps
            // off the others), then the other layers in their order (a popover
            // reads its anchor's final place).
            let (groups, rest): (Vec<usize>, Vec<usize>) = self
                .tree
                .floating
                .iter()
                .partition(|&&f| matches!(self.tree.nodes[f].float, Float::Anchored { .. }));
            let mut spots: Vec<(usize, Rect)> = Vec::new();
            for f in groups.into_iter().chain([NONE]).chain(rest) {
                // NONE marks the groups' end: place them, then go on to the rest.
                if f == NONE {
                    if !covered.is_empty() {
                        for k in 0..spots.len() {
                            let (g, r) = spots[k];
                            let Float::Anchored { anchor, margin, .. } = self.tree.nodes[g].float
                            else {
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
                        if !matches!(
                            self.tree.nodes[g].float,
                            Float::Anchored { avoid: true, .. }
                        ) {
                            continue;
                        }
                        let mut others: Vec<Rect> = spots
                            .iter()
                            .filter(|&&(o, _)| {
                                !matches!(
                                    self.tree.nodes[o].float,
                                    Float::Anchored { avoid: true, .. }
                                )
                            })
                            .map(|&(_, o)| o)
                            .collect();
                        if !covered.is_empty() {
                            others.push(covered);
                        }
                        spots[k].1 = step_clear(r, &others, gap, safe);
                    }
                    for &(g, placed) in &spots {
                        place_box(
                            &mut self.tree.nodes,
                            &mut self.layout.placed,
                            g,
                            [placed.width, placed.height],
                            Insets::default(),
                        );
                        layout(
                            &mut self.tree.nodes,
                            g,
                            placed,
                            &mut measure,
                            &offset,
                            dpr,
                            screen,
                            &mut self.layout.cache,
                            &mut self.layout.queries,
                        );
                    }
                    continue;
                }
                let float = self.tree.nodes[f].float;
                if float == Float::World {
                    // At its own size: an anchor where it is projected later, a
                    // panel in its slot off the screen.
                    let (area, clip) = self.world.area(f, screen);
                    layout(
                        &mut self.tree.nodes,
                        f,
                        area,
                        &mut measure,
                        &offset,
                        dpr,
                        clip,
                        &mut self.layout.cache,
                        &mut self.layout.queries,
                    );
                    continue;
                }
                if float == Float::Modal {
                    // The backdrop covers the surface; the safe area pads its content.
                    let padding = if self.tree.nodes[f].backdrop {
                        vp.safe_area
                    } else {
                        Insets::default()
                    };
                    place_box(
                        &mut self.tree.nodes,
                        &mut self.layout.placed,
                        f,
                        [vp.width, vp.height],
                        padding,
                    );
                    layout(
                        &mut self.tree.nodes,
                        f,
                        screen,
                        &mut measure,
                        &offset,
                        dpr,
                        screen,
                        &mut self.layout.cache,
                        &mut self.layout.queries,
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
                    &mut self.tree.nodes,
                    f,
                    area,
                    &mut measure,
                    &offset,
                    dpr,
                    screen,
                    &mut self.layout.cache,
                    &mut self.layout.queries,
                );
                let (w, h) = (self.tree.nodes[f].layout.w, self.tree.nodes[f].layout.h);
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
                        let a = match self.tree.nodes[f].anchor_point {
                            Some([px, py]) => Rect::new(px, py, 0.0, 0.0),
                            None => self
                                .tree
                                .nodes
                                .get(self.tree.nodes[f].anchor)
                                .map_or(safe, |n| n.placed.rect),
                        };
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
                        // The side it sits on, for its arrow.
                        self.tree.nodes[f].layout.anchor_placed.side = if y >= a.bottom() - 0.5 {
                            Some(crate::anchor::PlacedSide::Bottom)
                        } else if y + h <= a.y + 0.5 {
                            Some(crate::anchor::PlacedSide::Top)
                        } else {
                            None
                        };
                        (x, y)
                    }
                };
                let placed = Rect::new(x, y, w, h);
                if matches!(float, Float::Anchored { .. }) {
                    spots.push((f, placed));
                    continue;
                }
                place_box(
                    &mut self.tree.nodes,
                    &mut self.layout.placed,
                    f,
                    [w, h],
                    Insets::default(),
                );
                layout(
                    &mut self.tree.nodes,
                    f,
                    placed,
                    &mut measure,
                    &offset,
                    dpr,
                    screen,
                    &mut self.layout.cache,
                    &mut self.layout.queries,
                );
            }
            // Elements placed on the screen (Position::Screen): laid out again in
            // the whole surface, under the safe area their layer keeps clear,
            // and clipped by nothing but the surface.
            let mut placed_order = preorder(&self.tree.nodes, 0);
            for &f in &self.tree.floating {
                placed_order.extend(preorder(&self.tree.nodes, f));
            }
            for &i in &placed_order {
                if self.tree.nodes[i].props.position != crate::element::Position::Screen
                    || skipped(&self.tree.nodes, i)
                {
                    continue;
                }
                let r = crate::layout::on_screen(&self.tree.nodes, i, &screen, &offset);
                place_box(
                    &mut self.tree.nodes,
                    &mut self.layout.placed,
                    i,
                    [r.width, r.height],
                    Insets::default(),
                );
                layout(
                    &mut self.tree.nodes,
                    i,
                    r,
                    &mut measure,
                    &offset,
                    dpr,
                    screen,
                    &mut self.layout.cache,
                    &mut self.layout.queries,
                );
            }
            // Anchor-positioned elements (CSS Anchor Positioning 1): placed
            // against their anchors' boxes as laid out this frame, and laid
            // out again where that moved or resized them.
            self.layout.cache.anchor_toggled.clear();
            if placed_order
                .iter()
                .any(|&i| self.tree.nodes[i].props.anchor.is_some())
            {
                let names: Vec<(usize, &[String])> = placed_order
                    .iter()
                    .filter(|&&i| !self.tree.nodes[i].props.anchor_names.is_empty())
                    .map(|&i| (i, self.tree.nodes[i].props.anchor_names.as_slice()))
                    .collect();
                let named: Vec<(usize, String)> = names
                    .iter()
                    .flat_map(|(i, n)| n.iter().map(move |n| (*i, n.clone())))
                    .collect();
                for &i in &placed_order {
                    let Some(spec) = self.tree.nodes[i].props.anchor.clone() else {
                        continue;
                    };
                    if skipped(&self.tree.nodes, i) {
                        continue;
                    }
                    let box_of = |j: usize| {
                        let n = &self.tree.nodes[j];
                        Rect::new(n.layout.x, n.layout.y, n.layout.w, n.layout.h)
                    };
                    // The last element in tree order by that name, not inside
                    // it (§2.3); no name: its default anchor, else its
                    // implicit one.
                    let inside = |j: usize| {
                        let mut at = j;
                        while at != NONE {
                            if at == i {
                                return true;
                            }
                            at = self.tree.nodes[at].parent;
                        }
                        false
                    };
                    let by_name = |name: &str| {
                        named
                            .iter()
                            .rev()
                            .find(|(j, n)| n == name && !inside(*j))
                            .map(|(j, _)| *j)
                    };
                    // A popover's implicit anchor (HTML): the control that
                    // opened it, or the point a context menu opened at.
                    let popover =
                        matches!(self.tree.nodes[i].float, Float::Popover | Float::Tooltip)
                            && self.tree.floating.contains(&i);
                    let point = self.tree.nodes[i]
                        .anchor_point
                        .filter(|_| popover && spec.anchor.is_empty())
                        .map(|[x, y]| Rect::new(x, y, 0.0, 0.0));
                    let default = if spec.anchor.is_empty() {
                        let to = &self.tree.nodes[i].props.anchor_to;
                        let implicit = self.tree.nodes[i].anchor;
                        (!to.is_empty())
                            .then(|| crate::ui::node_in(&self.tree.index, &self.tree.nodes, to, i))
                            .flatten()
                            .or(
                                (popover && point.is_none() && implicit < self.tree.nodes.len())
                                    .then_some(implicit),
                            )
                    } else {
                        by_name(&spec.anchor)
                    };
                    let anchor = |name: &str| {
                        if name.is_empty() {
                            point.or(default.map(box_of))
                        } else {
                            by_name(name).map(box_of)
                        }
                    };
                    let n = &self.tree.nodes[i];
                    let cb = match n.props.position {
                        // A popover keeps inside the safe area, as it does unanchored.
                        _ if popover => safe,
                        crate::element::Position::Screen => screen,
                        _ if n.up != NONE => box_of(n.up),
                        _ => screen,
                    };
                    // Its own sizes, not the box an earlier placement fixed.
                    let own = n.derived(false);
                    let authored = [own.width, own.height];
                    let frame = crate::anchor::Frame {
                        cb,
                        anchor: &anchor,
                        size: authored,
                        content: [
                            (n.layout.pref_w, n.layout.min_w),
                            (n.layout.pref_h, n.layout.min_h),
                        ],
                    };
                    let (r, mut placed) = crate::anchor::resolve(&spec, &frame);
                    // `anchors-visible`: hidden while its default anchor is
                    // clipped out of view or hidden.
                    if spec.visibility == crate::anchor::Visibility::AnchorsVisible
                        && let Some(a) = default
                    {
                        let a = &self.tree.nodes[a];
                        placed.hidden =
                            a.is_hidden() || !a.layout.laid_clip.overlaps(&a.layout.laid_rect);
                    }
                    if placed.hidden != self.tree.nodes[i].layout.anchor_placed.hidden {
                        self.layout.cache.anchor_toggled.push(i);
                    }
                    let here = box_of(i);
                    self.tree.nodes[i].layout.anchor_placed = placed;
                    if r == here {
                        continue;
                    }
                    place_box(
                        &mut self.tree.nodes,
                        &mut self.layout.placed,
                        i,
                        [r.width, r.height],
                        Insets::default(),
                    );
                    layout(
                        &mut self.tree.nodes,
                        i,
                        r,
                        &mut measure,
                        &offset,
                        dpr,
                        screen,
                        &mut self.layout.cache,
                        &mut self.layout.queries,
                    );
                }
            }
            match self.settle_layout_queries(&mut settlement, &mut *resolve) {
                Ok(true) => {}
                Ok(false) => break,
                Err(error) => {
                    self.layout.abort_layout_queries(&mut self.tree);
                    self.paint.inspection = inspect::Inspection::none();
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    /// Samples the style, mirrors right to left, then shakes, staggers and
    /// motion; what may change this frame (`None`: everything) and what
    /// motion moved.
    fn frame_motion(
        &mut self,
        order: &[usize],
        whole: bool,
        vp: Viewport,
    ) -> (Option<Vec<usize>>, Vec<usize>) {
        // What may change this frame (`visit`): every node on a fresh one.
        let (mut seeds, restyle) = match whole {
            true => (None, None),
            false => {
                let (seeds, restyle) =
                    self.tree
                        .frame_seeds(&self.style, &self.layout, &self.interaction);
                (Some(seeds), Some(restyle))
            }
        };
        let sampled: &[usize] = restyle.as_deref().unwrap_or(order);
        for &i in sampled {
            self.tree.nodes[i].sample_style();
        }
        if self.env.direction == og_i18n::Direction::Rtl {
            // Right to left: the whole layout mirrored about the surface's
            // centre, and with it each transform (moves and turns the other way).
            // A node layout left out is mirrored as it was last frame.
            for k in 0..self.layout.cache.laid.len() {
                let i = self.layout.cache.laid[k];
                let r = &mut self.tree.nodes[i].placed.rect;
                r.x = vp.width - r.x - r.width;
                // Its clip too (a scroll container's box, laid out left to right): unmirrored, it
                // would cut away the content of a scroller off the surface's centre line.
                let c = &mut self.tree.nodes[i].placed.clip;
                c.x = vp.width - c.x - c.width;
            }
            for &i in sampled {
                let t = self.tree.nodes[i].style.sampled.transform;
                // A document's transforms are physical: they turn as written.
                if !t.is_identity() && !self.tree.nodes[i].props.flow.doc {
                    self.tree.nodes[i].style.sampled.transform = t.mirrored();
                }
            }
        }
        self.apply_shakes(order);
        let staggered = self.tree.orders.with(visit::feat::STAGGER).to_vec();
        self.paint
            .apply_stagger(&staggered, &self.env, &self.interaction, &mut self.tree);
        let animated = self.run_motion(restyle.as_deref());
        self.out.stats.animated = animated.len() as u32;
        // What motion wrote is placed and painted again.
        if let Some(seeds) = seeds.as_mut() {
            seeds.extend_from_slice(&animated);
        }
        self.style.apply_bursts(
            order,
            &mut self.world,
            &self.env,
            &mut self.paint,
            &self.interaction,
            &self.tree,
        );
        self.world_place();
        (seeds, animated)
    }

    /// The drawing order, the HUD's placements, the maps and the world:
    /// the drawing order and the nodes whose maps were made again.
    fn place_frame(
        &mut self,
        order: Vec<usize>,
        seeds: &Option<Vec<usize>>,
        animated: &[usize],
    ) -> (Vec<usize>, Vec<usize>) {
        self.tree.orders.pre = order;
        // Drawn and hit in z order among siblings.
        self.tree
            .update_draw_order(animated, &self.world, &self.layout);
        #[cfg(debug_assertions)]
        if bounded::checking() {
            self.tree.check_orders(&self.world, &self.layout);
        }
        let order = std::mem::take(&mut self.tree.orders.draw);
        let custom = self.tree.orders.drawn(visit::feat::CUSTOM);
        self.tree.hud_apply(&custom, &self.interaction);
        // Maps, opacity and inertness where they may have changed, and
        // below wherever they did.
        let world_seeds = seeds.as_ref().map(|s| {
            let mut w = s.clone();
            w.extend_from_slice(&self.tree.orders.wrote);
            w.extend_from_slice(&custom);
            w
        });
        let mapped = self.apply_world(&order, world_seeds.as_deref());
        self.layout.plane_quads(&mut self.world, &self.tree);
        self.interaction
            .hud_collect(&custom, &self.tree, &self.front);
        self.tree.world_finish(
            &mut self.world,
            &self.env,
            &mut self.paint,
            &self.interaction,
        );
        let dnd = self.tree.orders.with(visit::feat::DND).to_vec();
        self.note_drag_and_drop(&dnd);
        (order, mapped)
    }

    /// Interaction state for the next tick (scroll ranges, scopes, input
    /// specs, hits, focus, pads, taps), the cursor, and the selection's
    /// ranges on its elements: those whose range changed, painted again.
    #[allow(clippy::ptr_arg)]
    fn update_interaction(
        &mut self,
        order: &Vec<usize>,
        seeds: &Option<Vec<usize>>,
        mapped: &Vec<usize>,
        screen: Rect,
    ) -> (crate::Cursor, Vec<crate::CursorImage>, Vec<usize>) {
        // Interaction state for the next tick: what is on screen is what
        // the trees' index holds (`tree`), a rich text's links with it.
        #[cfg(debug_assertions)]
        if bounded::checking() {
            bounded::compare_tree(self, order);
        }
        let present = |k: &u64| self.tree.index.find(*k);
        self.interaction
            .state
            .scroll
            .retain(|k, _| present(k).is_some_and(|i| self.tree.nodes[i].el.scroll));
        self.interaction
            .state
            .virtual_lists
            .retain(|k, _| present(k).is_some());
        for (k, v) in self.interaction.state.scroll.iter_mut() {
            *v = v.clamp(
                0.0,
                present(k).map_or(0.0, |i| self.tree.nodes[i].max_scroll()),
            );
        }
        self.interaction
            .state
            .scroll_x
            .retain(|k, _| present(k).is_some_and(|i| self.tree.nodes[i].el.scroll_x));
        for (k, v) in self.interaction.state.scroll_x.iter_mut() {
            *v = v.clamp(
                0.0,
                present(k).map_or(0.0, |i| self.tree.nodes[i].max_scroll_x()),
            );
        }
        // Each scroll container's ranges and bar lengths, as drawn.
        self.layout.cache.scrollers = self.tree.orders.drawn(visit::feat::SCROLLS);
        let min_bar = self.env.theme.space.xl;
        self.interaction.state.scroll_info =
            IdMap::from_iter(self.layout.cache.scrollers.iter().map(|&i| {
                let n = &self.tree.nodes[i];
                let range = Vec2::new(n.max_scroll_x(), n.max_scroll());
                let bar = Vec2::new(
                    crate::scroll::bar_len(
                        n.placed.rect.width,
                        n.layout.w - n.layout.used.padding.horizontal(),
                        n.layout.content_w,
                        min_bar,
                    ),
                    crate::scroll::bar_len(
                        n.placed.rect.height,
                        n.layout.h - n.layout.used.padding.vertical(),
                        n.layout.content_h,
                        min_bar,
                    ),
                );
                (n.key, crate::scroll::ScrollInfo { range, bar })
            }));
        if let Some((k, _)) = self.interaction.state.fling
            && self.tree.index.find(k).is_none()
        {
            self.interaction.state.fling = None;
        }
        if !self.interaction.input.down
            || self
                .tree
                .index
                .find(self.interaction.state.active)
                .is_none()
        {
            self.interaction.state.active = 0;
        }
        let touched = std::mem::take(&mut self.front.touched);
        let docs = &self.docs;
        self.interaction
            .state
            .anims
            .retain(|k, _| touched.contains(k) || docs.mount_keeps_memory(*k));
        let tree = &self.tree.index;
        self.interaction
            .state
            .clocks
            .retain(|_, (owner, _)| tree.find(*owner).is_some() || docs.mount_owns(*owner));
        self.docs
            .state
            .retain_element_memory(&|k| tree.find(k).is_some());
        self.front.touched = touched;

        // Scopes, scrollers, focus owners, groups and recognisers around
        // each node: again where they may have changed, and below wherever
        // they did (a layer takes its anchor's).
        let scope_seeds = seeds.as_ref().map(|s| {
            let mut w = mapped.clone();
            w.extend_from_slice(s);
            w.extend_from_slice(&self.tree.floating);
            w
        });
        let scoped = self.visit(order, scope_seeds.as_deref(), Self::scope_step);
        // Focus groups, explicit moves and gesture recognisers for the next frame.
        self.interaction
            .collect_input_specs(&mut self.out, &self.tree);
        // Hits where a node's place, map or scopes may have changed (every
        // node when the touch target or the screen did).
        let touch_min = self.layout.touch_min(&self.env);
        let hit_seeds = match &seeds {
            Some(_) if self.tree.orders.hit_inputs == Some((touch_min, screen)) => {
                let mut h = mapped.clone();
                h.extend(scoped);
                Some(h)
            }
            _ => None,
        };
        self.tree.orders.hit_inputs = Some((touch_min, screen));
        let mut hits_changed = hit_seeds.is_none()
            || self.tree.invalidation.fresh
            || std::mem::take(&mut self.tree.orders.hits_moved);
        let hit_seeds = hit_seeds.unwrap_or_else(|| order.clone());
        for i in hit_seeds {
            if self.tree.orders.pos[i] != u32::MAX {
                hits_changed |= self.hit_step(i, touch_min, screen);
            }
        }
        let o = &self.tree.orders;
        if hits_changed || self.interaction.state.hits.is_empty() {
            self.interaction.state.hits = o
                .hit_nodes
                .iter()
                .flat_map(|&i| o.hit_of[i].iter().copied())
                .collect();
        }
        self.out
            .warnings
            .extend(o.hit_nodes.iter().filter_map(|&i| o.warn_of[i].clone()));
        let hits = std::mem::take(&mut self.interaction.state.hits);
        // The topmost scope open (not leaving).
        let (top_scope, top_locked) = o
            .drawn(visit::feat::SCOPE)
            .into_iter()
            .rev()
            .filter(|&i| i != order[0] && !self.tree.nodes[i].placed.inert)
            .map(|i| (self.tree.nodes[i].key, self.tree.nodes[i].locked))
            .next()
            .unwrap_or((0, false));
        // Focus: kept while its control is on screen (or reachable); a scope
        // that opened takes it (its first control, or the one a widget asked
        // for); a scope that closed gives it back.
        let alive = |k: u64| {
            hits.iter().any(|h: &Hit| h.key == k) || self.docs.deferred_focus_path(k).is_some()
        };
        let first_in = |scope: u64| {
            hits.iter()
                .find(|h| h.scope == scope && h.has(HIT_NAV) && !h.has(HIT_OFFSCREEN))
                .map_or(0, |h| h.key)
        };
        if self.interaction.request_focus != 0
            && hits.iter().any(|h| h.key == self.interaction.request_focus)
        {
            self.interaction.state.focus = self.interaction.request_focus;
        }
        if top_scope != self.interaction.state.scope {
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
                .interaction
                .state
                .focus_returns
                .iter()
                .rposition(|&(s, _)| s == top_scope);
            if let Some(k) = back_to {
                let (_, back) = self.interaction.state.focus_returns[k];
                self.interaction.state.focus_returns.truncate(k);
                if !inside(self.interaction.state.focus) {
                    self.interaction.state.focus = if inside(back) {
                        back
                    } else if top_scope != 0 {
                        first_in(top_scope)
                    } else {
                        0
                    };
                }
            } else if top_scope != 0 {
                if !inside(self.interaction.state.focus) {
                    // Entries of scopes no longer on screen are dropped; one
                    // that replaced another (the old one gone) opened over
                    // what that one had.
                    let on_screen = |s: u64| s == 0 || hits.iter().any(|h| h.scope == s);
                    self.interaction
                        .state
                        .focus_returns
                        .retain(|&(s, _)| on_screen(s));
                    if on_screen(self.interaction.state.scope) {
                        self.interaction
                            .state
                            .focus_returns
                            .push((self.interaction.state.scope, self.interaction.state.focus));
                    }
                    let req = self.interaction.request_focus;
                    self.interaction.state.focus =
                        if req != 0 && hits.iter().any(|h| h.key == req && h.scope == top_scope) {
                            req
                        } else {
                            first_in(top_scope)
                        };
                }
            } else {
                self.interaction.state.focus_returns.clear();
            }
        }
        if !alive(self.interaction.state.focus) {
            self.interaction.state.focus = 0;
        }
        // Pads 1 to 3: a focus whose control left is dropped; one outside a
        // scope that opened moves to its first control.
        for k in 0..3 {
            let f = self.interaction.state.pad_focus[k];
            let inside =
                |f: u64| top_scope == 0 || hits.iter().any(|h| h.key == f && h.scope == top_scope);
            if (k as u32) + 1 >= self.interaction.players {
                self.interaction.state.pad_focus[k] = 0;
            } else if f != 0 && (!alive(f) || !inside(f)) {
                self.interaction.state.pad_focus[k] = if top_scope != 0 {
                    first_in(top_scope)
                } else {
                    0
                };
            }
        }
        self.interaction.state.players = self.interaction.players as u8;
        self.interaction.state.vcursor.desc = self.interaction.vcursor;
        self.interaction.state.scope = top_scope;
        self.interaction.state.scope_locked = top_locked;
        self.interaction.state.nav_off = !self.interaction.nav_on;
        self.interaction.state.hits = hits;
        self.interaction.note_groups();
        self.tree.note_taps(&mut self.out, &self.interaction);
        let (cursor, cursor_images) = if self.interaction.state.vcursor.shown
            && self.interaction.state.vcursor.desc.is_some()
        {
            // The pad's cursor is drawn; the system's is hidden.
            (crate::Cursor::None, Vec::new())
        } else {
            self.tree
                .resolve_cursor(order, &self.env, &self.paint, &self.interaction)
        };
        let selected = self.interaction.mark_selection(&mut self.tree);
        (cursor, cursor_images, selected)
    }

    /// Parts, decorations, selection marks, the trees' commands and the
    /// frame's items; the drawing order is handed back.
    #[allow(clippy::type_complexity)]
    fn record_paint(
        &mut self,
        order: Vec<usize>,
        seeds: &Option<Vec<usize>>,
        mapped: Vec<usize>,
        selected: Vec<usize>,
    ) -> (
        Vec<usize>,
        Vec<DrawItem>,
        Vec<(og_draw::FontHandle, Vec<char>)>,
    ) {
        // A button narrower than an ornament's span is `[narrow]`, laid
        // out: a style draws it plain (fantasy's hairline and bevel, no
        // ornament under 160 px, art review 2026-11-04). `Ui::button`
        // decides its own as it builds (its label's ink follows the art).
        for k in 0..self.tree.orders.with(visit::feat::BUTTON_PART).len() {
            let i = self.tree.orders.with(visit::feat::BUTTON_PART)[k];
            let n = &mut self.tree.nodes[i];
            if n.placed.rect.width < NARROW_BUTTON && n.el.part.is_some() {
                n.placed.narrow = true;
                n.layout.used.part = n.part();
            }
        }
        // Parts a style dresses, and game draws it may restyle.
        let mut parts = self.tree.orders.with(visit::feat::PART).to_vec();
        parts.extend(
            self.tree
                .draws
                .keys()
                .copied()
                .filter(|&i| self.tree.index.has(i)),
        );
        self.build_part_draws(&parts);
        self.paint.decor_draws.clear();
        for k in 0..self.tree.orders.with(visit::feat::DECOR).len() {
            let i = self.tree.orders.with(visit::feat::DECOR)[k];
            if let Some(decor) = &self.tree.nodes[i].decor {
                let n = &self.tree.nodes[i];
                let r = n.placed.rect;
                // A scroller's content as a `local` layer's area: its
                // scrolled padding box.
                let scroll = n.scrolls().then(|| {
                    let p = decor.content;
                    let b = [3, 0, 1, 2].map(|k| decor.borders[k].map_or(0.0, |b| b.width));
                    (
                        [
                            self.interaction
                                .state
                                .scroll_x
                                .get(&n.key)
                                .copied()
                                .unwrap_or(0.0)
                                .clamp(0.0, n.max_scroll_x()),
                            self.interaction
                                .state
                                .scroll
                                .get(&n.key)
                                .copied()
                                .unwrap_or(0.0)
                                .clamp(0.0, n.max_scroll()),
                        ],
                        [
                            (r.width - b[0] - b[2])
                                .max(n.layout.content_w + p[0] - b[0] + p[2] - b[2]),
                            (r.height - b[1] - b[3])
                                .max(n.layout.content_h + p[1] - b[1] + p[3] - b[3]),
                        ],
                    )
                });
                let d = decor.over(
                    self.tree
                        .draws
                        .get(&i)
                        .or_else(|| self.paint.part_draws.get(&i)),
                    crate::css::paint_css::LayerFrame {
                        size: [r.width, r.height],
                        viewport: [self.env.viewport.width, self.env.viewport.height],
                        scroll,
                        ..Default::default()
                    },
                );
                self.paint.decor_draws.insert(i, d);
            }
        }
        // Painted again where a node's drawing may have changed (every
        // node on a fresh frame).
        let painted = if selected.is_empty() {
            mapped
        } else {
            [mapped, selected].concat()
        };
        let trees = self.commands(&order, seeds.is_some().then_some(&painted[..]));
        // The part styles sampled while painting take their next state.
        for commit in self.style.part_commits.take().into_values() {
            self.style
                .commit_styles(&commit, &mut self.interaction, &mut self.tree);
        }
        #[cfg(debug_assertions)]
        if bounded::checking() {
            for &i in &order {
                let n = &self.tree.nodes[i];
                assert_eq!(
                    self.paint.cache.sigs.get(i).copied().flatten(),
                    Some((n.key, crate::layout::DrawSig::of(n))),
                    "'{}' paints with another signature than it was kept by",
                    self.tree.path_of(i)
                );
            }
        }
        let repainted = self.paint.cache.repainted.take();
        self.tree
            .note_semantics(repainted.as_deref(), &self.layout, &self.interaction);
        let matched = self.tree.orders.with(visit::feat::MATCH).to_vec();
        self.record_matches(&matched);
        // A view transition's groups draw over the trees (and a cover);
        // the live elements taking part are left out of the items.
        let view = self.view_transition_frame(&trees);
        let mut tail = Vec::new();
        if let Some(c) = self.front.cover {
            let mut cover = Vec::new();
            self.cover_commands(&c, &mut cover);
            tail.extend(cover.into_iter().map(|c| (c, Affine::IDENTITY)));
        }
        if let Some((groups, _)) = &view {
            tail.extend(groups.iter().cloned());
        }
        let mut over = Vec::new();
        self.hud_edit_commands(&mut over);
        self.interaction
            .virtual_cursor_commands(&mut over, &self.env);
        tail.extend(over.into_iter().map(|c| (c, Affine::IDENTITY)));
        // Every tree drew as last frame, and what is over them too: the
        // frame's items are last frame's.
        let kept_items = view.is_none()
            && self.paint.cache.reused_all
            && tail == self.paint.cache.last_tail
            && self.out.frame.unit == self.env.unit;
        let (items, missing_glyphs) = if kept_items {
            (std::mem::take(&mut self.out.frame.items), Vec::new())
        } else if let Some(relevelled) = view
            .is_none()
            .then(|| {
                self.layout
                    .relevel(&trees, &tail, &self.env, &mut self.out, &mut self.paint)
            })
            .flatten()
        {
            relevelled
        } else if let Some(spliced) = view
            .is_none()
            .then(|| {
                self.layout
                    .splice_levels(&trees, &tail, &self.env, &mut self.out, &mut self.paint)
            })
            .flatten()
        {
            spliced
        } else {
            let mut cmds = trees.clone();
            cmds.extend(tail.iter().cloned());
            let mut cmds: Vec<(DrawCmd, Affine)> =
                cmds.into_iter().map(|(c, m)| bake(c, m)).collect();
            let missing_glyphs =
                self.layout
                    .missing_glyphs(&cmds, &self.env, &mut self.out, &mut self.paint);
            if let Some((_, hidden)) = &view {
                let mut shown = vec![true; cmds.len()];
                for r in hidden {
                    shown[r.clone()].fill(false);
                }
                let mut k = 0;
                cmds.retain(|_| {
                    k += 1;
                    shown[k - 1]
                });
            }
            let (items, levels) = crate::draw::assign_levels_kept(cmds);
            // Levels of a list with elements left out are no base for the
            // next frame's.
            self.paint.cache.levels = view.is_none().then_some(levels);
            (
                crate::draw::zoom_items(items, self.env.unit),
                missing_glyphs,
            )
        };
        self.paint.cache.prev_cmds = Vec::new();
        self.paint.cache.last_cmds = trees;
        self.paint.cache.last_tail = tail;
        (order, items, missing_glyphs)
    }
}

impl crate::ui::Layout {
    /// A vertical scroller's bar gutter beside its content, as its last
    /// frame's range needs it: its used padding takes it on the right.
    pub(crate) fn apply_gutter(
        &mut self,
        i: usize,
        env: &crate::ui::Env,
        interaction: &crate::ui::Interaction,
        tree: &mut crate::ui::Tree,
    ) {
        let node = &tree.nodes[i];
        let gutter = if node.el.scroll && !node.el.scroll_x && !node.el.hide_bars {
            interaction.bar_gutter(node.key, node.el.scroll_css, env)
        } else {
            0.0
        };
        let node = &mut tree.nodes[i];
        if gutter != node.layout.gutter {
            node.layout.gutter = gutter;
            node.derive_used();
        }
    }

    /// Lets go of the boxes placement fixed, so the next layout pass
    /// measures them again; the nodes that had one.
    pub(crate) fn unplace(&mut self, tree: &mut crate::ui::Tree) -> Vec<usize> {
        let mut freed = std::mem::take(&mut self.placed);
        freed.retain(|&i| {
            let had = tree.nodes[i].layout.placement.take().is_some();
            if had {
                tree.nodes[i].derive_used();
            }
            had
        });
        freed
    }
}

impl crate::ui::Layout {
    /// The minimum touch target in og-ui units (44 CSS px) on a touch
    /// screen, zero with a fine pointer.
    pub(crate) fn touch_min(&self, env: &crate::ui::Env) -> f32 {
        if env.viewport.coarse_pointer {
            TOUCH_TARGET / env.unit.max(0.01)
        } else {
            0.0
        }
    }

    /// CSS `content-visibility` (CSS Containment 2 §4), before layout:
    /// which elements skip their contents this frame. `hidden` always does;
    /// `auto` does from the frame after it was found far from the screen
    /// with no focus inside it ([`Ui::note_relevance`]). An element that
    /// starts or stops skipping lays out again, its contents afresh.
    fn content_visibility(&mut self, tree: &mut crate::ui::Tree) {
        use crate::layout::ContentVisibility as Cv;
        // A node written again keeps what it had found.
        for k in 0..tree.emitted.len() {
            let i = tree.emitted[k];
            if let Some(p) = tree.replaced.get(&i) {
                let (skip, far, remembered) =
                    (p.layout.cv_skip, p.layout.cv_far, p.layout.remembered);
                let n = &mut tree.nodes[i];
                (n.layout.cv_skip, n.layout.cv_far, n.layout.remembered) = (skip, far, remembered);
            }
        }
        let mut at: Vec<usize> = tree.orders.with(visit::feat::SKIPS).to_vec();
        at.extend(tree.emitted.iter().copied());
        at.sort_unstable();
        at.dedup();
        for i in at {
            if i >= tree.nodes.len() {
                continue;
            }
            let n = &mut tree.nodes[i];
            let v = n.el.contain.visibility;
            if v != Cv::Auto {
                n.layout.cv_far = false;
            }
            let skip = v == Cv::Hidden || (v == Cv::Auto && n.layout.cv_far);
            if skip != n.layout.cv_skip {
                n.layout.cv_skip = skip;
                self.cache.relayout.push(i);
            }
        }
    }

    /// After the frame: whether each `content-visibility: auto` element is
    /// relevant to the user, near the screen (its box within half the
    /// viewport of what of the screen its clip shows) or holding the
    /// focus; one whose answer changed skips its contents, or lays them
    /// out, from the next frame. One laying its contents out remembers its
    /// content box for `contain-intrinsic-size: auto`.
    fn note_relevance(
        &mut self,
        env: &crate::ui::Env,
        interaction: &crate::ui::Interaction,
        tree: &mut crate::ui::Tree,
    ) {
        let vp = env.viewport;
        let screen = Rect::new(0.0, 0.0, vp.width, vp.height);
        let focus = tree.index.find(interaction.state.focus);
        for k in 0..tree.orders.with(visit::feat::SKIPS).len() {
            let i = tree.orders.with(visit::feat::SKIPS)[k];
            if i >= tree.nodes.len() || !tree.index.has(i) {
                continue;
            }
            let n = &tree.nodes[i];
            if !n.layout.cv_skip {
                let size = [
                    (n.layout.w - n.layout.used.padding.horizontal()).max(0.0),
                    (n.layout.h - n.layout.used.padding.vertical()).max(0.0),
                ];
                tree.nodes[i].layout.remembered = Some(size);
            }
            let n = &tree.nodes[i];
            if n.el.contain.visibility != crate::layout::ContentVisibility::Auto {
                continue;
            }
            let (mx, my) = (vp.width * 0.5, vp.height * 0.5);
            let near = Rect::new(
                n.placed.screen.x - mx,
                n.placed.screen.y - my,
                n.placed.screen.width + 2.0 * mx,
                n.placed.screen.height + 2.0 * my,
            );
            let mut holds_focus = false;
            let mut at = focus.unwrap_or(NONE);
            while at != NONE && at < tree.nodes.len() {
                if at == i {
                    holds_focus = true;
                    break;
                }
                at = tree.nodes[at].parent;
            }
            let far = !holds_focus && !near.overlaps(&n.placed.clip.intersect(&screen));
            tree.nodes[i].layout.cv_far = far;
        }
    }

    /// The paths of the focused controls, pads 1 to 3 after the first.
    fn focus_paths(
        &self,
        docs: &crate::ui::Docs,
        interaction: &crate::ui::Interaction,
        tree: &crate::ui::Tree,
    ) -> [String; 4] {
        let mut focus_paths: [String; 4] = Default::default();
        let focused = [
            interaction.state.focus,
            interaction.state.pad_focus[0],
            interaction.state.pad_focus[1],
            interaction.state.pad_focus[2],
        ];
        for (slot, key) in focused.into_iter().enumerate() {
            if key != 0
                && let Some(i) = tree.index.find(key)
            {
                focus_paths[slot] = tree.path_of(i);
                if let Some(l) = tree.nodes[i]
                    .layout
                    .links
                    .iter()
                    .find(|l| child_key(tree.nodes[i].key, l) == key)
                {
                    focus_paths[slot] = format!("{}/{l}", focus_paths[slot]);
                }
            } else if let Some(path) = docs.deferred_focus_path(key) {
                focus_paths[slot] = path.to_owned();
            }
        }
        focus_paths
    }
}

impl crate::ui::Tree {
    /// Node `i` lays its flow out in lines while one of it is inline-level
    /// (CSS 2 §9.4.2): a block is used as a wrapping row whose lines align
    /// on their baseline and by its `text-align`, a block-level child on a
    /// line of its own; its own layout is used again when none is.
    fn line_box(&mut self, i: usize, flow: &[usize]) {
        let n = &self.nodes[i];
        let wanted = (n.el.block
            && n.layout_mode() == crate::LayoutMode::Flex
            && matches!(*n.content, Content::None)
            && flow.iter().any(|&c| self.nodes[c].el.inline))
        .then_some(n.el.direction);
        let n = &mut self.nodes[i];
        if wanted != n.layout.line_box {
            n.layout.line_box = wanted;
            n.derive_used();
        }
    }

    fn place_popover_arrows(&mut self) {
        // Popovers' arrows (`::-og-arrow`): a square turned 45° across the
        // edge facing the anchor, over the anchor's middle where the box
        // allows, hidden while no side faces it. A context menu (opened at
        // a point) draws none: its corner is at the point, as a desktop's
        // and a game's context menus are.
        for k in 0..self.floating.len() {
            let f = self.floating[k];
            let n = &self.nodes[f];
            let Some((size, color)) = n.arrow else {
                continue;
            };
            let a = match n.anchor_point {
                Some([x, y]) => Rect::new(x, y, 0.0, 0.0),
                None => match self.nodes.get(n.anchor) {
                    Some(a) => a.placed.rect,
                    None => continue,
                },
            };
            let (b, placed, point) = (n.placed.rect, n.layout.anchor_placed, n.anchor_point);
            self.nodes[f].placed.arrow = None;
            let (Some(side), false, None) = (placed.side, placed.hidden, point) else {
                continue;
            };
            // Kept a side's length in from the corners.
            let along =
                |lo: f32, len: f32, at: f32| at.clamp(lo + size, (lo + len - size).max(lo + size));
            let (ax, ay) = a.center();
            let (cx, cy) = match side {
                crate::anchor::PlacedSide::Bottom => (along(b.x, b.width, ax), b.y),
                crate::anchor::PlacedSide::Top => (along(b.x, b.width, ax), b.bottom()),
                crate::anchor::PlacedSide::Right => (b.x, along(b.y, b.height, ay)),
                crate::anchor::PlacedSide::Left => (b.right(), along(b.y, b.height, ay)),
            };
            let (bx, by) = b.center();
            self.nodes[f].placed.arrow = Some(crate::layout::Shape::Bar {
                part: crate::dress::PartRef::of("Popover", "arrow", None),
                frame: crate::layout::Frame::centered(cx - bx, cy - by, size, size),
                angle: std::f32::consts::FRAC_PI_4,
                color,
            });
        }
    }
}

impl crate::ui::Layout {
    /// Prepares a written node's text for layout, as the build ends: a
    /// display text under the art's least size set in the body font,
    /// uppercase steps applied, a pixel font's sizes on its grid. A
    /// normalisation of the content the build wrote, so it writes it with
    /// the build's proof; layout proper never does.
    pub(crate) fn prepare_layout_node(
        &mut self,
        i: usize,
        env: &crate::ui::Env,
        tree: &mut crate::ui::Tree,
    ) {
        let build = tree.build();
        let node = &mut tree.nodes[i];
        if let Content::Text(text) = node.content.write(build)
            && env.theme.art.display_min > 0.0
            && text.font == crate::FontRole::Display
            && text.step.size < env.theme.art.display_min
        {
            text.font = crate::FontRole::Body;
        }
        if !matches!(node.widget, Widget::Field(_) | Widget::TextArea { .. }) {
            match node.content.write(build) {
                Content::Text(text) if text.step.uppercase => {
                    text.text = og_i18n::text::uppercase(&text.text, &env.locale);
                }
                Content::Rich(rich) => {
                    for span in &mut rich.spans {
                        if span.step.uppercase && span.atom.is_none() {
                            span.text = og_i18n::text::uppercase(&span.text, &env.locale);
                        }
                    }
                }
                _ => {}
            }
        }
        crate::layout::snap_pixel_text(&mut tree.nodes[i..i + 1], &env.fonts, build);
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
        env: &crate::ui::Env,
        out: &mut crate::ui::Out,
        paint: &mut crate::ui::Paint,
    ) -> Option<(Vec<DrawItem>, MissingGlyphs)> {
        let prev = std::mem::take(&mut paint.cache.prev_cmds);
        if paint.cache.levels.is_none()
            || paint.cache.aligned.len() != paint.cache.aligned_kept
            || paint.cache.shifted_out
            || out.frame.unit != env.unit
            || trees.len() != paint.cache.prev_len
            || tail.len() != paint.cache.last_tail.len()
        {
            return None;
        }
        let total = trees.len() + tail.len();
        let mut changed = Vec::new();
        let mut at = 0;
        let gaps = paint
            .cache
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
        for (k, (now, was)) in tail.iter().zip(&paint.cache.last_tail).enumerate() {
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
        paint.kept_spans.clear();
        let missing_glyphs = self.missing_glyphs(&baked, env, out, paint);
        let mut items = std::mem::take(&mut out.frame.items);
        let replaced = changed
            .into_iter()
            .zip(baked)
            .map(|(p, (c, m))| (p, c, m))
            .collect();
        let levels = paint.cache.levels.as_mut().expect("checked");
        if !levels.update(&mut items, replaced, env.unit) {
            out.frame.items = items;
            return None;
        }
        #[cfg(debug_assertions)]
        if bounded::checking() {
            let mut cmds: Vec<(DrawCmd, Affine)> = trees.to_vec();
            cmds.extend(tail.iter().cloned());
            let cmds: Vec<_> = cmds.into_iter().map(|(c, m)| bake(c, m)).collect();
            let fresh = crate::draw::zoom_items(crate::draw::assign_levels(cmds), env.unit);
            assert!(
                fresh == items,
                "the frame's items levelled again differ from levelling it afresh"
            );
        }
        Some((items, missing_glyphs))
    }

    /// Last frame's items with the commands this frame copied from last
    /// frame's kept, moved with the copy ([`crate::draw::Levels::splice`]),
    /// and only the others baked and levelled again: a scroll moves the
    /// rows it keeps and levels the rows it brings in. `None` when nothing
    /// was copied or the levels cannot follow.
    fn splice_levels(
        &mut self,
        trees: &[(DrawCmd, Affine)],
        tail: &[(DrawCmd, Affine)],
        env: &crate::ui::Env,
        out: &mut crate::ui::Out,
        paint: &mut crate::ui::Paint,
    ) -> Option<(Vec<DrawItem>, MissingGlyphs)> {
        if paint.cache.copied.is_empty() || out.frame.unit != env.unit {
            return None;
        }
        let levels = paint.cache.levels.as_mut()?;
        let mut from: Vec<Option<(usize, (f32, f32))>> = vec![None; trees.len()];
        for &(start, was, len, delta) in &paint.cache.copied {
            for k in 0..len {
                from[start + k] = Some((was + k, delta));
            }
        }
        let same_tail = tail.len() == paint.cache.last_tail.len();
        let mut fresh = Vec::new();
        let mut next = Vec::with_capacity(trees.len() + tail.len());
        for (p, (c, m)) in trees.iter().enumerate() {
            next.push(match from[p] {
                Some((was, (0.0, 0.0))) => crate::draw::Spliced {
                    from: Some(was),
                    delta: (0.0, 0.0),
                    cmd: None,
                },
                Some((was, delta)) => crate::draw::Spliced {
                    from: Some(was),
                    delta,
                    cmd: Some(bake(c.clone(), *m)),
                },
                None => {
                    let baked = bake(c.clone(), *m);
                    fresh.push(baked.clone());
                    crate::draw::Spliced {
                        from: None,
                        delta: (0.0, 0.0),
                        cmd: Some(baked),
                    }
                }
            });
        }
        for (k, c) in tail.iter().enumerate() {
            next.push(if same_tail && *c == paint.cache.last_tail[k] {
                crate::draw::Spliced {
                    from: Some(paint.cache.prev_len + k),
                    delta: (0.0, 0.0),
                    cmd: None,
                }
            } else {
                let baked = bake(c.0.clone(), c.1);
                fresh.push(baked.clone());
                crate::draw::Spliced {
                    from: None,
                    delta: (0.0, 0.0),
                    cmd: Some(baked),
                }
            });
        }
        let old = std::mem::take(&mut out.frame.items);
        let items = levels.splice(old, next, env.unit);
        if items.is_none() {
            paint.cache.levels = None;
        }
        let items = items?;
        paint.kept_spans.clear();
        let missing_glyphs = self.missing_glyphs(&fresh, env, out, paint);
        #[cfg(debug_assertions)]
        if bounded::checking() {
            let mut cmds: Vec<(DrawCmd, Affine)> = trees.to_vec();
            cmds.extend(tail.iter().cloned());
            let cmds: Vec<_> = cmds.into_iter().map(|(c, m)| bake(c, m)).collect();
            let fresh = crate::draw::zoom_items(crate::draw::assign_levels(cmds), env.unit);
            assert!(
                fresh == items,
                "the frame's items spliced from last frame's differ from levelling it afresh"
            );
        }
        Some((items, missing_glyphs))
    }

    fn missing_glyphs(
        &mut self,
        cmds: &[(DrawCmd, Affine)],
        env: &crate::ui::Env,
        ui_out: &mut crate::ui::Out,
        paint: &mut crate::ui::Paint,
    ) -> Vec<(og_draw::FontHandle, Vec<char>)> {
        let mut out: Vec<(og_draw::FontHandle, Vec<char>)> = Vec::new();
        let kept = std::mem::take(&mut paint.kept_spans);
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
            let Some(face) = env.fonts.face(*font) else {
                continue;
            };
            for ch in text.chars() {
                if ch.is_whitespace()
                    || ch.is_control()
                    || face.metrics.has(ch)
                    || !ui_out.glyphs_missed.insert((*font, ch))
                {
                    continue;
                }
                match out.iter_mut().find(|(f, _)| f == font) {
                    Some((_, chars)) => chars.push(ch),
                    None => out.push((*font, vec![ch])),
                }
            }
        }
        paint.kept_spans = kept;
        out
    }
}

/// The width (CSS px) under which a button is `[narrow]`.
pub const NARROW_BUTTON: f32 = 160.0;

impl crate::ui::Interaction {
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
type MissingGlyphs = Vec<(og_draw::FontHandle, Vec<char>)>;
