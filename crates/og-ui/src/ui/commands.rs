use super::*;

impl Ui {
    /// Each element's map onto the screen (docs/ui.md §2): its layout
    /// animation, its drag and pinch, its transition's movement and its own
    /// transform, about its box, composed with its parent's; its opacity
    /// multiplied down the tree; inert where a transition moves it, a state
    /// or timeline holds it, or its parent is. Clips made by a transformed
    /// scroll container move with it.
    pub(super) fn apply_world(&mut self, order: &[usize], seeds: Option<&[usize]>) -> Vec<usize> {
        self.visit(order, seeds, Self::world_step)
    }

    /// Node `i`'s map, opacity and inertness from its parent's (see
    /// `apply_world`); answers whether its children's change with them.
    fn world_step(&mut self, i: usize) -> bool {
        let before = {
            let n = &self.tree.nodes[i];
            (
                n.placed.world,
                n.placed.alpha,
                n.placed.inert,
                n.placed.plane.clone(),
            )
        };
        let parent = self.tree.nodes[i].parent;
        let (alpha, inert, pw) = if parent == NONE {
            (1.0, false, Affine::IDENTITY)
        } else {
            let p = &self.tree.nodes[parent];
            (p.placed.alpha, p.placed.inert, p.placed.world)
        };
        // The 3D space its parent gives it: the parent's plane, and the
        // parent's `perspective` (about its perspective-origin).
        let (pplane, ppersp) = if parent == NONE {
            (None, None)
        } else {
            let p = &self.tree.nodes[parent];
            let persp = p.props.space3d.as_ref().and_then(|s| {
                s.perspective
                    .map(|_| s.perspective_matrix(&reference_box(p, s.transform_box)))
            });
            (p.placed.plane.as_deref().cloned(), persp)
        };

        let rtl = self.env.direction == og_i18n::Direction::Rtl;
        let n = &mut self.tree.nodes[i];
        let shown = !n.layout.anchor_placed.hidden;
        n.placed.alpha =
            alpha * n.style.sampled.opacity.clamp(0.0, 1.0) * f32::from(u8::from(shown));
        n.placed.scale = 1.0;
        n.placed.inert = inert || n.style.sampled.inert || !shown;
        let r = n.placed.rect;
        let (cx, cy) = r.center();
        let mut local = Affine::IDENTITY;
        let [fx, fy, fsx, fsy] = n.style.flip;
        if fx != 0.0 || fy != 0.0 || fsx != 1.0 || fsy != 1.0 {
            local = Affine::translate(fx, fy).mul(&Affine::scale(fsx, fsy).about(r.x, r.y));
        }
        let [hx, hy, hs, ho] = n.style.custom;
        n.placed.alpha *= ho.clamp(0.0, 1.0);
        if hx != 0.0 || hy != 0.0 || hs != 1.0 {
            local = local
                .mul(&Affine::translate(hx, hy))
                .mul(&Affine::scale(hs, hs).about(cx, cy));
        }
        let [gx, gy, gs, gr] = n.style.gesture;
        if gx != 0.0 || gy != 0.0 || gs != 1.0 || gr != 0.0 {
            let turn = Affine::rotate(gr).mul(&Affine::scale(gs, gs)).about(cx, cy);
            local = local.mul(&Affine::translate(gx, gy)).mul(&turn);
        }
        if let Some(f) = n.fx {
            n.placed.alpha *= f.alpha;
            n.placed.inert |= f.inert;
            if f.dx != 0.0 || f.dy != 0.0 || f.scale != 1.0 {
                local = local
                    .mul(&Affine::translate(f.dx, f.dy))
                    .mul(&Affine::scale(f.scale, f.scale).about(cx, cy));
            }
        }
        if n.props.view_place.is_some() {
            // Laid out unscaled at its anchor: scaled by the view box's fit
            // about it (moved back into place first, right to left).
            let [k, fx, ay, off] = n.layout.fit;
            let ax = fx * r.width;
            let dx = if rtl { 2.0 * (off - ax) + r.width } else { 0.0 };
            let fit = Affine::scale(k, k)
                .about(r.x + ax + dx, r.y + ay)
                .mul(&Affine::translate(dx, 0.0));
            if !fit.is_identity() {
                local = local.mul(&fit);
            }
        }
        let space = n.props.space3d.clone();
        let tbox = space
            .as_ref()
            .map_or(r, |s| reference_box(n, s.transform_box));
        let mut t = n.style.sampled.transform;
        if let Some(s) = space.as_ref().filter(|s| s.origin.is_some()) {
            // `transform-origin` pivots the individual properties too.
            let (ox, oy, _) = s.origin_at(&tbox, (t.pivot_x, t.pivot_y));
            t.pivot_x = (ox - r.x) / r.width.max(1e-6);
            t.pivot_y = (oy - r.y) / r.height.max(1e-6);
        }
        if !t.is_identity() {
            local = local.mul(&t.matrix(&r));
        }
        if space.is_none() && ppersp.is_none() && pplane.is_none() {
            n.placed.plane = None;
            n.placed.world = if local.is_identity() {
                pw
            } else {
                pw.mul(&local)
            };
        } else {
            use crate::transform3d as t3;
            let mut local4 = t3::from_affine(&local);
            if let Some(s) = &space {
                let pivot = (
                    n.style.sampled.transform.pivot_x,
                    n.style.sampled.transform.pivot_y,
                );
                local4 = t3::mul(&local4, &s.matrix(&tbox, pivot));
            }
            if let Some(pm) = &ppersp {
                local4 = t3::mul(pm, &local4);
            }
            // Into the parent's space: its 3D context unflattened, a
            // root's own layer, or the parent's plane flattened.
            let (into, base, shared) = match &pplane {
                Some(pp) if pp.preserve => (pp.into, pp.m, pp.context),
                Some(pp) if pp.root => (parent, t3::IDENTITY, 0),
                Some(pp) => (pp.into, t3::flatten(&pp.m), 0),
                None => (NONE, t3::from_affine(&pw), 0),
            };
            let m = t3::mul(&base, &local4);
            let preserve = space.as_ref().is_some_and(|s| s.preserve_3d);
            let root = !t3::is_flat(&m);
            n.placed.plane = (into != NONE || root || preserve).then(|| {
                Box::new(t3::Plane {
                    into,
                    m,
                    root,
                    preserve,
                    context: if shared != 0 {
                        shared
                    } else if preserve {
                        n.key
                    } else {
                        0
                    },
                    hidden: false,
                    clip: INFINITE,
                })
            });
            let onto = if into == NONE {
                Affine::IDENTITY
            } else {
                self.tree.nodes[into].placed.world
            };
            let n = &mut self.tree.nodes[i];
            n.placed.world = onto.mul(&t3::project(&m));
            if let Some(p) = n.placed.plane.as_deref_mut()
                && p.root
                && space.as_ref().is_some_and(|s| s.backface_hidden)
            {
                // Seen from behind: its corners wind the other way on
                // the screen (CSS Transforms 2 §6.3).
                let w = n.placed.world;
                let c = [
                    (r.x, r.y),
                    (r.right(), r.y),
                    (r.right(), r.bottom()),
                    (r.x, r.bottom()),
                ];
                let ws: Vec<f32> = c.iter().map(|&(x, y)| w.g * x + w.h * y + w.i).collect();
                if ws.iter().all(|&v| v > 0.0) {
                    let q: Vec<(f32, f32)> = c.iter().map(|&(x, y)| w.apply(x, y)).collect();
                    let area: f32 = (0..4)
                        .map(|k| {
                            let (a, b) = (q[k], q[(k + 1) % 4]);
                            a.0 * b.1 - b.0 * a.1
                        })
                        .sum();
                    p.hidden = area < 0.0;
                }
            }
        }
        let n = &mut self.tree.nodes[i];
        n.placed.screen = n.placed.world.map_rect(&r);
        // A transformed scroll container's clip moves with it: made from
        // the clip layout gave (mirrored right to left), so a node layout
        // left out is not moved twice.
        let src = self.tree.nodes[i].placed.clip_src;
        if src != NONE && !self.tree.nodes[src].placed.world.is_identity() {
            let w = self.tree.nodes[src].placed.world;
            let mut c = self.tree.nodes[i].layout.laid_clip;
            if self.env.direction == og_i18n::Direction::Rtl {
                c.x = self.env.viewport.width - c.x - c.width;
            }
            self.tree.nodes[i].placed.clip = w.map_rect(&c);
        }
        // Its clip in the space its layer draws in: a scroll container's
        // in the same layer, else none (an outer clip cuts the composite).
        if self.tree.nodes[i].placed.plane.is_some() {
            let into = self.tree.nodes[i]
                .placed
                .plane
                .as_ref()
                .map_or(NONE, |p| p.into);
            let clip = if src == NONE {
                INFINITE
            } else if into == NONE {
                self.tree.nodes[i].placed.clip
            } else if src == into {
                self.tree.nodes[src].layout.laid_clip
            } else {
                match self.tree.nodes[src].placed.plane.as_deref() {
                    Some(sp) if sp.into == into && !sp.root => crate::transform3d::project(&sp.m)
                        .map_rect(&self.tree.nodes[src].layout.laid_clip),
                    _ => INFINITE,
                }
            };
            if let Some(p) = self.tree.nodes[i].placed.plane.as_deref_mut() {
                p.clip = clip;
            }
        }
        let n = &self.tree.nodes[i];
        if n.placed.plane.as_ref().is_some_and(|p| p.root) {
            self.world.placement.planes.insert(n.key);
        }
        let n = &self.tree.nodes[i];
        before
            != (
                n.placed.world,
                n.placed.alpha,
                n.placed.inert,
                n.placed.plane.clone(),
            )
    }

    /// The frame's draw commands with their maps (docs/ui.md §2): each
    /// element's in its layout box, mapped by its world (an axis-aligned
    /// map is baked into the command, a turn or skew is left for the
    /// overlay); a scroll container's bars after its content; leaving
    /// elements (ghosts) after their parent's subtree, or last; and the
    /// drawing of every element with an exit or a match key frozen for
    /// next tick.
    pub(super) fn commands(
        &mut self,
        order: &[usize],
        changed: Option<&[usize]>,
    ) -> Vec<(DrawCmd, Affine)> {
        let dpr = self.env.viewport.dpr();
        let snap = |v: f32| crate::layout::snap(v, dpr);
        let ends = std::mem::take(&mut self.tree.orders.ends);
        let mut last = std::mem::take(&mut self.paint.cache.last_cmds);
        self.paint.cache.prev_len = last.len();
        self.paint.cache.aligned.clear();
        self.paint.cache.aligned_kept = 0;
        self.paint.cache.shifted_out = false;
        self.paint.cache.copied.clear();
        let mut cmds: Vec<(DrawCmd, Affine)> = Vec::with_capacity(last.len());
        let ghosts = std::mem::take(&mut self.interaction.state.motion.ghosts);
        let mut drawn = vec![false; ghosts.len()];
        self.paint.cache.named.clear();
        let mut frozen: IdMap<u64, crate::motion::state::Frozen> = IdMap::new();
        // Open subtrees: (node, its first command, the order position past
        // it, where its commands began in last frame's list).
        let mut open: Vec<(usize, usize, usize, Option<usize>, usize)> = Vec::new();
        // Focus rings waiting for the subtree they draw over (`ring_over`).
        let mut rings: Vec<(usize, Vec<(DrawCmd, Affine)>)> = Vec::new();
        let prev_kinds = std::mem::take(&mut self.tree.prev_kinds);
        self.paint.kept_spans.clear();
        let reuse = self.paint_reuse(order, &ends, changed, &ghosts, &prev_kinds);
        self.paint.cache.repainted = reuse.repainted.clone();
        let mut reused_roots = true;
        let theme = self.env.theme;
        // An element's draw (crate::paint) in its layout box, at its
        // opacity; its clip is the screen's (bake leaves draw commands be).
        let press_scale = theme.press_scale;
        let marked = |n: &crate::layout::Node| {
            n.el.part
                .is_some_and(|p| p.has("selected") || p.has("focused"))
        };
        let placed = |n: &crate::layout::Node| crate::paint::Placed {
            rect: part_rect(n, press_scale),
            radius: n.style.sampled.radius,
            clip: n.placed.clip,
            alpha: n.placed.alpha,
        };
        let mut k = 0;
        while k < order.len() {
            let i = order[k];
            // Where its subtree's commands began in last frame's list.
            let up = open.last().map(|o| o.3);
            let at_last = match self.paint.cache.ranges.get(&self.tree.nodes[i].key) {
                Some(&(rel, len, count)) => match up {
                    Some(Some(base)) => Some((base + rel, len, count)),
                    Some(None) => None,
                    None => Some((rel, len, count)),
                },
                None => None,
            };
            // A subtree that paints as last frame: its commands copied whole,
            // moved as far as it moved (a scrolled row).
            let (dx, dy) = reuse.delta(i);
            let moved = (dx, dy) != (0.0, 0.0);
            let copied = match at_last {
                Some((from, len, _)) if reuse.clean(i) && from + len <= last.len() => {
                    !moved || last[from..from + len].iter().all(|(c, _)| shiftable(c))
                }
                _ => false,
            };
            if copied && let Some((from, len, count)) = at_last {
                let start = cmds.len();
                if moved {
                    // Moved out of last frame's list (no other subtree
                    // copies its range) and shifted where it lies.
                    let blank = || (DrawCmd::PopClip, Affine::IDENTITY);
                    cmds.extend(last[from..from + len].iter_mut().map(|c| {
                        let mut c = std::mem::replace(c, blank());
                        shift(&mut c.0, dx, dy);
                        c
                    }));
                    self.paint
                        .moved_subtree(&order[k..ends[k]], dx, dy, &self.tree);
                    self.paint.cache.shifted_out = true;
                } else if cmds.is_empty() && from == 0 && len == last.len() {
                    // Last frame's whole list, as it was: taken, not copied.
                    cmds = std::mem::take(&mut last);
                } else if from == start {
                    // Where it stood last frame: moved out of last frame's
                    // list, which is not read there again (`relevel`
                    // compares only what is not kept in place).
                    #[cfg(debug_assertions)]
                    if bounded::checking() {
                        assert!(
                            self.paint
                                .cache
                                .aligned
                                .last()
                                .is_none_or(|r| r.end <= from),
                            "a range of last frame's commands is kept twice"
                        );
                    }
                    let blank = || (DrawCmd::PopClip, Affine::IDENTITY);
                    cmds.extend(
                        last[from..from + len]
                            .iter_mut()
                            .map(|c| std::mem::replace(c, blank())),
                    );
                } else {
                    cmds.extend_from_slice(&last[from..from + len]);
                }
                self.paint.kept_spans.push(start..cmds.len());
                self.paint.cache.copied.push((start, from, len, (dx, dy)));
                if !moved && from == start {
                    self.paint.cache.aligned.push(start..cmds.len());
                    self.paint.cache.aligned_kept += 1;
                }
                self.out.stats.kept_commands += count;
                let base = open.last().map_or(0, |o| o.1);
                self.paint
                    .cache
                    .ranges
                    .insert(self.tree.nodes[i].key, (start - base, len, count));
                k = ends[k];
                self.close_subtrees(
                    &mut open,
                    &mut cmds,
                    k,
                    &ghosts,
                    &mut drawn,
                    &mut rings,
                    &mut frozen,
                );
                continue;
            }
            if self.tree.nodes[i].parent == NONE {
                reused_roots = false;
            }
            let n = &self.tree.nodes[i];
            let world = n.placed.plane.as_ref().map_or(n.placed.world, |p| p.draw());
            // Its clip where its commands draw: a 3D plane's in its layer.
            let draw_clip = match n.placed.plane.as_deref() {
                Some(p) if p.root => INFINITE,
                Some(p) => p.clip,
                None => n.placed.clip,
            };
            let plane = n
                .placed
                .plane
                .as_deref()
                .filter(|p| p.root)
                .map(|p| (p.preserve, p.clip));
            let start = cmds.len();
            if plane.is_some() {
                // A 3D plane draws into a layer of its own, laid onto it.
                cmds.push((plane_begin(), Affine::IDENTITY));
            }
            let draw = self.paint.draw_of(i, &self.tree).cloned();
            let n = &self.tree.nodes[i];
            if let Some(m) = self.world.world_marks(n.key) {
                cmds.extend(m.enter.iter().map(|c| (c.clone(), Affine::IDENTITY)));
            }
            // One order for game and style draws, over the element's
            // background (`set_draw`): ground, rim, content, then children
            // and raised badges; a selected or focused part's rim (its
            // mark) after its children (`marked`), so the item's art never
            // covers it.
            let over_background = |ui: &Self| -> Vec<(DrawCmd, Affine)> {
                let Some(d) = &draw else { return Vec::new() };
                let n = &ui.tree.nodes[i];
                // CSS accent-color: the primary ink of its control's parts.
                let mut theme = theme;
                if let Some(a) = ui.tree.nearest(i, |n| n.accent) {
                    theme.palette.primary = a;
                    theme.palette.primary_hover = a;
                    theme.palette.primary_active = a;
                }
                let mut out = d.enter(&theme, &placed(n));
                if !marked(n) {
                    out.extend(d.strokes(&theme, &placed(n)));
                }
                undraw(&mut out, n.style.sampled.draw);
                if let Some(decor) = &n.decor {
                    out.extend(decor.lines(&placed(n)));
                }
                out.into_iter().map(|c| (c, world)).collect()
            };
            let sig = crate::layout::DrawSig::of(n);
            // CSS `visibility: hidden`: nothing of its own is drawn; its
            // children that say `visible` are.
            let hidden = self.tree.orders.hidden_of.get(i) == Some(&true)
                || self.tree.orders.skip_of.get(i).is_some_and(|&s| s != 0);
            if hidden {
                if let Some(d) = &draw {
                    cmds.extend(d.begin(&theme, &placed(n)).into_iter().map(|c| (c, world)));
                }
            } else if i != 0 || n.style.sampled.background.is_some() {
                let mut local = Vec::new();
                let mut background = 0;
                let mut kept_now = false;
                if reuse.same(i)
                    && let Some((b, c, _)) = self.paint.cache.cmds.get(&n.key)
                {
                    (background, local) = (*b, c.clone());
                    self.out.stats.kept_commands += 1;
                    kept_now = true;
                } else {
                    // CSS accent-color: its controls' primary ink.
                    let mut ink = self.env.theme;
                    if let Some(a) = self.tree.nearest(i, |n| n.accent) {
                        ink.palette.primary = a;
                        ink.palette.primary_hover = a;
                        ink.palette.primary_active = a;
                    }
                    if world.is_identity() {
                        self.out.stats.painted += 1;
                        self.paint.ctx = Some(PaintCtx {
                            node: i,
                            clip: draw_clip,
                            map: Affine::IDENTITY,
                            theme: ink,
                        });
                        background = self.node_commands(i, &mut local, &snap);
                        self.paint.ctx = None;
                    } else if let Some(inv) = world.invert() {
                        self.out.stats.painted += 1;
                        // Drawn in its layout box: its screen clip taken there.
                        self.paint.ctx = Some(PaintCtx {
                            node: i,
                            clip: inv.map_rect(&draw_clip),
                            map: world,
                            theme: ink,
                        });
                        background = self.node_commands(i, &mut local, &snap);
                        self.paint.ctx = None;
                    }
                }
                if let Some(d) = &draw {
                    let grid = match &self.tree.nodes[i].content {
                        Content::Text(t) => t.metrics(&self.env.fonts).pixel_grid(),
                        _ => 0,
                    };
                    d.style_text(&theme, &mut local, grid);
                }
                let key = self.tree.nodes[i].key;
                if !kept_now {
                    self.paint
                        .cache
                        .cmds
                        .insert(key, (background, local.clone(), sig));
                }
                let kept_at = kept_now.then_some(local.len());
                let mut local = local.into_iter().map(|c| (c, world));
                let from = cmds.len();
                cmds.extend(local.by_ref().take(background));
                let behind = cmds.len();
                cmds.extend(over_background(self));
                let over = cmds.len();
                // Its text, with the backgrounds cut to it when it has them.
                let text: Vec<DrawCmd> = local.map(|(c, _)| c).collect();
                let text = match &draw {
                    Some(d) => d.clip_to_text(&theme, &placed(&self.tree.nodes[i]), text),
                    None => text,
                };
                cmds.extend(text.into_iter().map(|c| (c, world)));
                // Kept commands were read for missing glyphs when painted.
                if kept_at.is_some() {
                    self.paint.kept_spans.push(from..behind);
                    self.paint.kept_spans.push(over..cmds.len());
                }
            } else {
                cmds.extend(over_background(self));
            }
            let mut ring = Vec::new();
            if hidden {
            } else if world.is_identity() {
                self.paint.ctx = Some(PaintCtx {
                    node: i,
                    clip: draw_clip,
                    map: Affine::IDENTITY,
                    theme: self.env.theme,
                });
                self.focus_ring_commands(i, &mut ring);
                self.paint.ctx = None;
            } else if let Some(inv) = world.invert() {
                self.paint.ctx = Some(PaintCtx {
                    node: i,
                    clip: inv.map_rect(&draw_clip),
                    map: world,
                    theme: self.env.theme,
                });
                self.focus_ring_commands(i, &mut ring);
                self.paint.ctx = None;
            }
            if !ring.is_empty() {
                rings.push((
                    self.paint.ring_over(i, &self.world, &self.tree),
                    ring.into_iter().map(|c| (c, world)).collect(),
                ));
            }
            if let Some(d) = &draw {
                let at = placed(&self.tree.nodes[i]);
                cmds.extend(d.children(&at).into_iter().map(|c| (c, world)));
            }
            if let Some((true, clip)) = plane {
                // Its children are planes of its 3D space, not drawn onto
                // its own: its layer ends with its own drawing.
                cmds.push((plane_end(self.tree.nodes[i].key, clip), Affine::IDENTITY));
            }
            self.paint.cache.sigs_set(i, self.tree.nodes[i].key, sig);
            open.push((i, start, ends[k], at_last.map(|a| a.0), k));
            k += 1;
            self.close_subtrees(
                &mut open,
                &mut cmds,
                k,
                &ghosts,
                &mut drawn,
                &mut rings,
                &mut frozen,
            );
        }
        for (g, ghost) in ghosts.iter().enumerate() {
            if !drawn[g] {
                reused_roots = false;
                self.tree.ghost_commands(ghost, &mut cmds);
            }
        }
        self.interaction.state.motion.ghosts = ghosts;
        self.interaction.state.motion.frozen = frozen;
        self.style.nest_named(&mut self.paint, &self.tree);
        // The same trees as last frame, each drawn whole as it was.
        let same_roots = self.tree.orders.roots == self.paint.cache.last_roots;
        self.paint
            .cache
            .last_roots
            .clone_from(&self.tree.orders.roots);
        self.tree.orders.ends = ends;
        self.paint.cache.reuse = reuse;
        self.paint.cache.reused_all = reused_roots
            && same_roots
            && !self.tree.invalidation.fresh
            && !self.tree.invalidation.whole;
        self.paint.cache.prev_cmds = last;
        cmds
    }

    /// Closes the open subtrees that end at order position `k`: what each
    /// draws after its children (ghosts, a scroll's fade and bars, its
    /// draw's end, focus rings), its frozen drawing, and its range in this
    /// frame's commands for the next.
    #[allow(clippy::too_many_arguments)]
    fn close_subtrees(
        &mut self,
        open: &mut Vec<(usize, usize, usize, Option<usize>, usize)>,
        cmds: &mut Vec<(DrawCmd, Affine)>,
        k: usize,
        ghosts: &[crate::motion::state::Ghost],
        drawn: &mut [bool],
        rings: &mut Vec<(usize, Vec<(DrawCmd, Affine)>)>,
        frozen: &mut IdMap<u64, crate::motion::state::Frozen>,
    ) {
        let theme = self.env.theme;
        let marked = |n: &crate::layout::Node| {
            n.el.part
                .is_some_and(|p| p.has("selected") || p.has("focused"))
        };
        let press_scale = theme.press_scale;
        let placed = |n: &crate::layout::Node| crate::paint::Placed {
            rect: part_rect(n, press_scale),
            radius: n.style.sampled.radius,
            clip: n.placed.clip,
            alpha: n.placed.alpha,
        };
        {
            while let Some(&(j, first, end, _, pos)) = open.last() {
                if end != k {
                    break;
                }
                open.pop();
                let key = self.tree.nodes[j].key;
                for (g, ghost) in ghosts.iter().enumerate() {
                    if !drawn[g] && ghost.frozen.parent == key {
                        drawn[g] = true;
                        self.tree.ghost_commands(ghost, cmds);
                    }
                }
                if self.tree.nodes[j].scrolls() && self.tree.nodes[j].el.fade > 0.0 {
                    let mut fades = Vec::new();
                    self.paint.scroll_fade_commands(
                        j,
                        &mut fades,
                        &self.env,
                        &self.interaction,
                        &self.tree,
                    );
                    let w = self.tree.nodes[j].placed.world;
                    cmds.extend(fades.into_iter().map(|c| (c, w)));
                }
                if self.tree.nodes[j].scrolls() && !self.tree.nodes[j].el.hide_bars {
                    let mut bars = Vec::new();
                    self.scroll_bar_commands(j, &mut bars);
                    let w = self.tree.nodes[j].placed.world;
                    cmds.extend(bars.into_iter().map(|c| (c, w)));
                }
                // CSS `resize`: its handle over everything it drew.
                if self.tree.nodes[j].el.resize != 0
                    && self.tree.orders.skip_of.get(j).is_none_or(|&s| s == 0)
                {
                    let mut grip = Vec::new();
                    self.tree.resizer_commands(j, &mut grip, &self.env);
                    let w = self.tree.nodes[j].placed.world;
                    cmds.extend(grip.into_iter().map(|c| (c, w)));
                }
                let w = self.tree.nodes[j].placed.world;
                if let Some(d) = self.paint.draw_of(j, &self.tree) {
                    let at = placed(&self.tree.nodes[j]);
                    let hidden = self.tree.orders.hidden_of.get(j) == Some(&true)
                        || self.tree.orders.skip_of.get(j).is_some_and(|&s| s != 0);
                    let mut leave = if marked(&self.tree.nodes[j]) && !hidden {
                        d.leave(&theme, &at)
                    } else {
                        d.leave_stroked(&theme, &at)
                    };
                    undraw(&mut leave, self.tree.nodes[j].style.sampled.draw);
                    cmds.extend(leave.into_iter().map(|c| (c, w)));
                }
                // Focus rings over everything their subtree drew: later
                // siblings, a scroll's fade and bars (an outline's paint
                // order in CSS).
                rings.retain_mut(|(over, ring)| {
                    if *over != j {
                        return true;
                    }
                    cmds.append(ring);
                    false
                });
                if let Some(p) = self.tree.nodes[j].placed.plane.as_deref()
                    && p.root
                    && !p.preserve
                {
                    cmds.push((plane_end(key, p.clip), Affine::IDENTITY));
                }
                if let Some(m) = self.world.world_marks(key) {
                    cmds.extend(m.leave.iter().map(|c| (c.clone(), Affine::IDENTITY)));
                }
                let nj = &self.tree.nodes[j];
                if nj.props.exit.is_some() || !nj.props.match_key.is_empty() {
                    let inv = nj.placed.world.invert().unwrap_or(Affine::IDENTITY);
                    let parent = if nj.parent == NONE {
                        0
                    } else {
                        self.tree.nodes[nj.parent].key
                    };
                    frozen.insert(
                        key,
                        crate::motion::state::Frozen {
                            cmds: cmds[first..]
                                .iter()
                                .filter(|(c, _)| !matches!(c, DrawCmd::World(_)))
                                .map(|(c, m)| (c.clone(), inv.mul(m)))
                                .collect(),
                            rect: nj.placed.rect,
                            world: nj.placed.world,
                            alpha: nj.placed.alpha,
                            parent: if self.tree.index.by_key.contains_key(&parent) {
                                parent
                            } else {
                                0
                            },
                            path: self.tree.path_of(j),
                            exit: nj.props.exit.clone(),
                            match_key: if nj.props.match_key.is_empty() {
                                0
                            } else {
                                crate::motion::state::name_hash(&nj.props.match_key)
                            },
                            radius: nj.style.sampled.radius,
                        },
                    );
                }
                // A glitching subtree draws through the glitch material
                // (TransitionKind::Glitch): a layer from its first command
                // to its last.
                if let Some(g) = self.tree.nodes[j].fx.map(|f| f.glitch).filter(|&g| g > 0.0) {
                    let nj = &self.tree.nodes[j];
                    let strength = match theme.art.motion.glitch {
                        s if s > 0.0 => s,
                        _ => 1.0,
                    };
                    let (begin, end) = glitch_layer(
                        key,
                        g * strength,
                        nj.placed.screen.height,
                        self.env.ticks_per_second,
                        nj.placed.rect,
                        nj.placed.clip,
                    );
                    // Every later command moves one place: no span is kept.
                    self.paint.kept_spans.retain(|r| r.end <= first);
                    self.paint.cache.aligned.retain(|r| r.end <= first);
                    shift_named(&mut self.paint.cache.named, first);
                    cmds.insert(first, (begin, nj.placed.world));
                    cmds.push((end, nj.placed.world));
                }
                // A blurring subtree (Svelte's `blur` transition) draws
                // through a blur layer, as CSS `filter: blur()` does.
                let nj = &self.tree.nodes[j];
                if nj.style.sampled.blur > 0.0 {
                    let (begin, end) =
                        blur_layer(nj.style.sampled.blur, nj.placed.rect, nj.placed.clip);
                    self.paint.kept_spans.retain(|r| r.end <= first);
                    self.paint.cache.aligned.retain(|r| r.end <= first);
                    shift_named(&mut self.paint.cache.named, first);
                    cmds.insert(first, (begin, nj.placed.world));
                    cmds.push((end, nj.placed.world));
                }
                // A shared element of a view transition: where its drawing
                // is in this frame's list (crate::view_transition).
                let nj = &self.tree.nodes[j];
                if !nj.props.view_transition_name.is_empty() {
                    let name = match nj.props.view_transition_name.as_str() {
                        "auto" | "match-element" => crate::view_transition::element_name(key),
                        n => n.to_string(),
                    };
                    self.paint.cache.named.push(crate::view_transition::Named {
                        node: j,
                        key,
                        name,
                        classes: nj.props.view_transition_class.clone(),
                        group: nj.props.view_transition_group.clone(),
                        parent: None,
                        range: first..cmds.len(),
                        screen: nj.placed.screen,
                        inset: crate::view_transition::border_inset(nj),
                        inherited: if nj.parent == NONE {
                            1.0
                        } else {
                            self.tree.nodes[nj.parent].placed.alpha
                        },
                    });
                }
                // Its range for the next frame, from its parent's start.
                let base = open.last().map_or(0, |o| o.1);
                let count = (k - pos) as u32;
                self.paint.cache.ranges.insert(
                    self.tree.nodes[j].key,
                    (first - base, cmds.len() - first, count),
                );
            }
        }
    }

    /// Which nodes paint as last frame (`same`: their own commands and
    /// their ancestors' fills are unchanged) and which subtrees do whole
    /// (`clean`: every node in them is `same`, and none draws what the
    /// frame makes again on its own: a focus ring, a ghost, world marks, a
    /// frozen or glitching drawing, a scroll's fade and bars). `changed`
    /// holds the nodes whose drawing may differ from last frame's (`None`:
    /// any); every other node, and the subtree it heads, paints as it did
    /// unless one below it changed.
    fn paint_reuse(
        &mut self,
        order: &[usize],
        ends: &[usize],
        changed: Option<&[usize]>,
        ghosts: &[crate::motion::state::Ghost],
        prev_kinds: &std::collections::HashMap<usize, (&'static str, bool)>,
    ) -> Reuse {
        let n = self.tree.nodes.len();
        let mut r = std::mem::take(&mut self.paint.cache.reuse);
        r.begin(n);
        // The bounded frame's check paints its whole twin afresh.
        if self.tree.invalidation.fresh
            || self.tree.invalidation.whole
            || self.tree.index.all_restructured
        {
            let p = &mut self.paint.cache;
            p.volatile.clear();
            p.ringed.clear();
            for &i in order {
                r.set(i, false, false, (f32::NAN, f32::NAN));
                let (v, ring) = self.paints_alone(i);
                if v {
                    self.paint.cache.volatile.insert(i);
                }
                if ring {
                    self.paint.cache.ringed.insert(i);
                }
            }
            r.repainted = None;
            return r;
        }
        let (o, p) = (&self.tree.orders, &mut self.paint.cache);
        // What the frame draws again on its own, kept until it does not.
        p.volatile.retain(|&i| o.pos[i] != u32::MAX);
        p.ringed.retain(|&i| o.pos[i] != u32::MAX);
        let mut volatile: std::collections::HashSet<usize> = self
            .tree
            .draws
            .keys()
            .chain(&self.tree.index.restructured)
            .chain(&self.paint.part_changed)
            .copied()
            .collect();
        let keyed = ghosts
            .iter()
            .map(|g| g.frozen.parent)
            .chain(self.world.placement.marks.keys().copied())
            .chain(self.interaction.state.motion.part_styles.iter().filter_map(
                |(&key, styles)| {
                    styles
                        .values()
                        .any(|s| s.running || !s.runs.is_empty())
                        .then_some(key)
                },
            ));
        for key in keyed {
            if let Some(&i) = self.tree.index.by_key.get(&key) {
                volatile.insert(i);
            }
        }
        // The nodes to judge, in drawing order: those that may have
        // changed, those drawn again on their own, and below any that
        // does not paint as last frame.
        let mut at: Vec<u32> = changed
            .unwrap_or(order)
            .iter()
            .chain(&self.tree.emitted)
            .chain(&volatile)
            .chain(&p.volatile)
            .chain(&p.ringed)
            .chain(o.with(visit::feat::RICH))
            .map(|&i| o.pos.get(i).copied().unwrap_or(u32::MAX))
            .filter(|&p| p != u32::MAX)
            .collect();
        at.sort_unstable();
        at.dedup();
        let written_at = std::mem::take(&mut self.tree.orders.written_at);
        let stamp = self.tree.orders.written_stamp;
        let written = |i: usize| written_at[i] == stamp;
        let mut judged = Vec::new();
        let (mut next, mut cover, mut p) = (0usize, 0usize, None::<usize>);
        loop {
            let q = match p {
                Some(p) if p + 1 < cover => p + 1,
                _ => {
                    while next < at.len() && p.is_some_and(|p| at[next] as usize <= p) {
                        next += 1;
                    }
                    let Some(&q) = at.get(next) else { break };
                    q as usize
                }
            };
            p = Some(q);
            let i = order[q];
            let node = &self.tree.nodes[i];
            let up = node.parent;
            let own = !volatile.contains(&i)
                && !self.tree.draws.contains_key(&i)
                && !matches!(node.content, Content::Rich(_))
                && if written(i) {
                    // A node written again: as it was drawn last frame.
                    self.layout.cache.prev.get(&i).is_some_and(|b| {
                        crate::layout::draw_eq(node, b)
                            && Some(&self.tree.kinds[i]) == prev_kinds.get(&i)
                    })
                } else {
                    // A retained node not written again: its own inputs are
                    // as they were; what the frame moves is in its signature.
                    self.paint.cache.sigs.get(i).is_some_and(|s| {
                        s.is_some_and(|(k, sig)| {
                            k == node.key && sig == crate::layout::DrawSig::of(node)
                        })
                    })
                };
            let same = own && (up == NONE || r.same(up));
            let parent = (up != NONE).then(|| r.delta(up));
            let delta = if same {
                (0.0, 0.0)
            } else if !written(i)
                && !self.tree.draws.contains_key(&i)
                && !matches!(node.content, Content::Rich(_))
                && let Some(Some((k, old))) = self.paint.cache.sigs.get(i)
                && *k == node.key
                && let Some(d) = crate::layout::DrawSig::of(node).moved_from(old)
                && parent.is_none_or(|p| p == (0.0, 0.0) || p == d)
            {
                d
            } else {
                (f32::NAN, f32::NAN)
            };
            let (own_volatile, ring) = self.paints_alone(i);
            let p = &mut self.paint.cache;
            if own_volatile {
                p.volatile.insert(i);
            } else {
                p.volatile.remove(&i);
            }
            if ring {
                p.ringed.insert(i);
            } else {
                p.ringed.remove(&i);
            }
            let clean = !delta.0.is_nan() && !own_volatile && !volatile.contains(&i);
            r.set(i, same, clean, delta);
            judged.push(i);
            if !same {
                cover = cover.max(ends[q]);
            }
        }
        // A subtree is copied whole when every node in it moved as its
        // root: a node that is not, or moved otherwise, unsettles each
        // node above it.
        for &i in judged.iter().rev() {
            let mut c = i;
            loop {
                let up = self.tree.nodes[c].parent;
                if up == NONE || (r.clean(c) && r.delta(c) == r.delta(up)) || !r.clean(up) {
                    break;
                }
                r.unclean(up);
                c = up;
            }
        }
        // A focus ring is drawn at the end of the subtree it draws over
        // (`ring_over`): that subtree is copied whole with it or not at
        // all, never a part of it below.
        let ringed: Vec<usize> = self.paint.cache.ringed.iter().copied().collect();
        for i in ringed {
            let over = self.paint.ring_over(i, &self.world, &self.tree);
            let mut j = i;
            while j != over && j != NONE {
                r.unclean(j);
                j = self.tree.nodes[j].parent;
            }
        }
        // The nodes not written that draw otherwise than last frame.
        r.repainted = Some(
            judged
                .into_iter()
                .filter(|&i| !r.same(i) && !written(i))
                .collect(),
        );
        self.tree.orders.written_at = written_at;
        r
    }

    /// Whether node `i` draws again on its own every frame (an exit, a
    /// match key, links, a glitch, a scroll's fade and bars), and whether
    /// a focus ring or outline is drawn over it.
    fn paints_alone(&self, i: usize) -> (bool, bool) {
        let node = &self.tree.nodes[i];
        let volatile = node.props.exit.is_some()
            || !node.props.match_key.is_empty()
            || !node.props.view_transition_name.is_empty()
            || !node.layout.links.is_empty()
            || node.fx.is_some_and(|f| f.glitch > 0.0)
            || node.style.sampled.blur > 0.0
            || node.decor.as_ref().is_some_and(|d| d.fixed())
            || (node.scrolls() && (node.el.fade > 0.0 || !node.el.hide_bars))
            || node.el.resize != 0;
        let (framed, field_ring) = self.field_rings(i);
        let outline = self.env.theme.outline && (node.nav || node.focusable) && framed;
        (
            volatile,
            node.marks.ring || node.marks.pad_rings != 0 || outline || field_ring,
        )
    }

    fn scroll_bar_commands(&self, s: usize, cmds: &mut Vec<DrawCmd>) {
        let t = &self.env.theme;
        let p = &t.palette;
        let dpr = self.env.viewport.dpr();
        let sn = &self.tree.nodes[s];
        let clip = self.tree.scroll_clip(s);
        // CSS `scrollbar-width: thin` halves the bar; `scrollbar-color`
        // paints its thumb and a track along the whole edge under it.
        let css = sn.el.scroll_css;
        let thick = if css.thin {
            t.space.xs / 2.0
        } else {
            t.space.xs
        };
        let bar = |axis, rect: Rect, track: Rect, cmds: &mut Vec<DrawCmd>| {
            if let Some([thumb, ground]) = css.colors {
                cmds.push(DrawCmd::Quad {
                    rect: track.snapped(dpr),
                    radius: t.radius.bar,
                    color: ground.linear(),
                    clip,
                });
                cmds.push(DrawCmd::Quad {
                    rect: rect.snapped(dpr),
                    radius: t.radius.bar,
                    color: thumb.linear(),
                    clip,
                });
                return;
            }
            let part = crate::dress::PartRef::of("Scroll", "bar", None).attr("axis", axis);
            let values = self.part_values(&part, Some(sn.key));
            self.part_commands_at(
                Some(sn.key),
                &part,
                values.part_rect(rect, sn.placed.rect).snapped(dpr),
                Some(values.color("color").unwrap_or(p.control_strong)),
                t.radius.bar,
                1.0,
                clip,
                cmds,
            );
        };
        let max_x = sn.max_scroll_x();
        if max_x > 0.0 {
            let inner_w = sn.layout.w - sn.el.padding.horizontal();
            let track = sn.placed.rect;
            let bar_w =
                crate::scroll::bar_len(track.width, inner_w, sn.layout.content_w, t.space.xl);
            let off = self
                .interaction
                .state
                .scroll_x
                .get(&sn.key)
                .copied()
                .unwrap_or(0.0);
            let x = crate::scroll::bar_start(track.x, track.width, bar_w, off, max_x);
            let h = thick;
            let y = track.bottom() - h - t.space.xxs;
            bar(
                "horizontal",
                Rect::new(x, y, bar_w, h),
                Rect::new(track.x, y, track.width, h),
                cmds,
            );
        }
        let max = sn.max_scroll();
        if max > 0.0 {
            let inner_h = sn.layout.h - sn.el.padding.vertical();
            let track = sn.placed.rect;
            let bar_h = (track.height * inner_h / sn.layout.content_h).max(t.space.xl);
            let off = self
                .interaction
                .state
                .scroll
                .get(&sn.key)
                .copied()
                .unwrap_or(0.0)
                .clamp(0.0, max);
            let y = track.y + (track.height - bar_h) * off / max;
            let w = thick;
            let x = track.right() - w - t.space.xxs;
            bar(
                "vertical",
                Rect::new(x, y, w, bar_h),
                Rect::new(x, track.y, w, track.height),
                cmds,
            );
        }
    }
}

impl crate::ui::Paint {
    /// Staggered entrances: under a node with a stagger, the k-th drawn
    /// leaf (a label, a control, an icon) enters `k` steps after the first;
    /// a long list shortens the step so the cascade ends within one more
    /// entrance. The cascade is only seen: a control takes input as soon as
    /// the panel's own entrance (inert while it moves) has played, where it
    /// is drawn, not when its place in the cascade has.
    pub(super) fn apply_stagger(
        &mut self,
        order: &[usize],
        env: &crate::ui::Env,
        interaction: &crate::ui::Interaction,
        tree: &mut crate::ui::Tree,
    ) {
        let now = interaction.state.tick as f32;
        let (reduced, distance) = (env.theme.reduced_motion, env.theme.space.lg);
        for &i in order {
            let Some(st) = tree.nodes[i].stagger else {
                continue;
            };
            let leaves: Vec<usize> = preorder(&tree.nodes, i)
                .into_iter()
                .skip(1)
                .filter(|&j| {
                    let n = &tree.nodes[j];
                    n.children.is_empty()
                        && !matches!(n.widget, Widget::Spacer)
                        && (!matches!(n.content, Content::None)
                            || n.el.background.is_some()
                            || !matches!(n.widget, Widget::None)
                            || !n.shapes.is_empty())
                })
                .collect();
            // The whole cascade takes at most one entrance more.
            let step = if leaves.len() > 1 {
                st.step.min(st.ticks / (leaves.len() - 1) as f32)
            } else {
                st.step
            };
            for (k, &j) in leaves.iter().enumerate() {
                let start = st.since as f32 + step * k as f32;
                let p = ((now - start + 1.0) / st.ticks).clamp(0.0, 1.0);
                if p >= 1.0 || tree.nodes[j].fx.is_some() {
                    continue;
                }
                tree.nodes[j].fx = Some(crate::motion::fx(
                    st.kind,
                    env.theme
                        .tokens
                        .ease(crate::tokens::EaseRole::WidgetEnter)
                        .apply(p),
                    distance,
                    reduced,
                ));
            }
        }
    }

    /// The nodes of a subtree copied moved by `(dx, dy)`: their signatures
    /// and kept commands are where they are drawn now.
    fn moved_subtree(&mut self, nodes: &[usize], dx: f32, dy: f32, tree: &crate::ui::Tree) {
        for &j in nodes {
            let key = tree.nodes[j].key;
            self.cache
                .sigs_set(j, key, crate::layout::DrawSig::of(&tree.nodes[j]));
            if let Some((_, local, sig)) = self.cache.cmds.get_mut(&key) {
                if local.iter().all(shiftable) {
                    for c in local.iter_mut() {
                        shift(c, dx, dy);
                    }
                    *sig = crate::layout::DrawSig::of(&tree.nodes[j]);
                } else {
                    self.cache.cmds.remove(&key);
                }
            }
        }
    }

    /// A scroll container's bars, over its content.
    /// A scroll container's clip in its own space.
    /// The subtree node `i`'s focus ring draws after: its screen's (the
    /// root's child it is in), or the world-placed element's it is in, so
    /// a ring stays in that element's space.
    fn ring_over(&self, i: usize, world: &crate::world::WorldUi, tree: &crate::ui::Tree) -> usize {
        let mut j = i;
        loop {
            let parent = tree.nodes[j].parent;
            if parent == NONE || parent == 0 || world.world_marks(tree.nodes[j].key).is_some() {
                return j;
            }
            j = parent;
        }
    }

    /// The fade over each edge of a scroll container where more of its
    /// content lies beyond, as strong as the content past that edge is
    /// over the fade's reach: none at the top of a list, full a fade's
    /// length into it, so a cut row reads as "there is more".
    fn scroll_fade_commands(
        &self,
        s: usize,
        cmds: &mut Vec<DrawCmd>,
        env: &crate::ui::Env,
        interaction: &crate::ui::Interaction,
        tree: &crate::ui::Tree,
    ) {
        use crate::paint::{ColorRef, ElementDraw, Gradient, Paint, Placed, Stop};
        let sn = &tree.nodes[s];
        let reach = sn.el.fade;
        let into = sn.el.fade_into.unwrap_or(env.theme.palette.panel);
        let clip = tree.scroll_clip(s);
        let r = sn.placed.rect;
        let max_y = sn.max_scroll();
        let off_y = interaction
            .state
            .scroll
            .get(&sn.key)
            .copied()
            .unwrap_or(0.0)
            .clamp(0.0, max_y);
        let max_x = sn.max_scroll_x();
        let off_x = interaction
            .state
            .scroll_x
            .get(&sn.key)
            .copied()
            .unwrap_or(0.0)
            .clamp(0.0, max_x);
        // (strength, the strip, the gradient's from and to: opaque at the outer edge.)
        let edges = [
            (
                off_y,
                Rect::new(r.x, r.y, r.width, reach.min(r.height / 2.0)),
                [0.5, 0.0],
                [0.5, 1.0],
            ),
            (
                max_y - off_y,
                Rect::new(
                    r.x,
                    r.bottom() - reach.min(r.height / 2.0),
                    r.width,
                    reach.min(r.height / 2.0),
                ),
                [0.5, 1.0],
                [0.5, 0.0],
            ),
            (
                off_x,
                Rect::new(r.x, r.y, reach.min(r.width / 2.0), r.height),
                [0.0, 0.5],
                [1.0, 0.5],
            ),
            (
                max_x - off_x,
                Rect::new(
                    r.right() - reach.min(r.width / 2.0),
                    r.y,
                    reach.min(r.width / 2.0),
                    r.height,
                ),
                [1.0, 0.5],
                [0.0, 0.5],
            ),
        ];
        for (beyond, rect, from, to) in edges {
            let k = (beyond / reach).min(1.0);
            if k < 0.01 {
                continue;
            }
            let draw = ElementDraw {
                fills: vec![Paint::Gradient(Gradient {
                    from,
                    to,
                    stops: vec![
                        Stop {
                            offset: 0.0,
                            color: ColorRef::Literal(into),
                        },
                        Stop {
                            offset: 1.0,
                            color: ColorRef::Literal(into.fade(0.0)),
                        },
                    ],
                    ..Gradient::default()
                })],
                ..ElementDraw::default()
            };
            let at = Placed {
                rect,
                radius: 0.0,
                clip,
                alpha: sn.placed.alpha * k,
            };
            cmds.extend(draw.enter(&env.theme, &at));
        }
    }
}

impl crate::ui::Interaction {
    /// The clock's first tick, kept for the lifetime of its semantic owner.
    pub(crate) fn since(&mut self, owner: u64, key: u64) -> u64 {
        self.state
            .clocks
            .entry(key)
            .or_insert((owner, self.state.tick))
            .1
    }
}

impl crate::ui::Tree {
    /// A leaving element's frozen drawing, moved by its exit (about its
    /// box's centre) or onto the element it morphs into, and faded.
    fn ghost_commands(&self, g: &crate::motion::state::Ghost, cmds: &mut Vec<(DrawCmd, Affine)>) {
        let f = &g.frozen;
        let mut v = [0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        for (c, ch) in &g.channels {
            if let Some(slot) = v.get_mut(*c as usize) {
                *slot = ch.value;
            }
        }
        let world = if g.morph_to != 0 {
            let to = self
                .nodes
                .iter()
                .find(|n| n.key == g.morph_to)
                .map(|n| n.placed.screen);
            match to {
                Some(to) => Affine::rect_to_rect(&f.world.map_rect(&f.rect), &to).mul(&f.world),
                None => f.world,
            }
        } else {
            let t = crate::element::Transform {
                x: v[0],
                y: v[1],
                scale_x: v[2],
                scale_y: v[3],
                rotate: v[4],
                skew_x: v[5],
                skew_y: v[6],
                ..crate::element::Transform::IDENTITY
            };
            f.world.mul(&t.matrix(&f.rect))
        };
        let fade = v[7].clamp(0.0, 1.0);
        if fade <= 0.0 {
            return;
        }
        let exit = |c: usize| g.channels.iter().find(|(k, _)| *k as usize == c);
        let blur = exit(19).map_or(0.0, |(_, ch)| ch.value.max(0.0));
        let drawn = exit(20).map_or(0.0, |(_, ch)| ch.value.clamp(0.0, 1.0));
        let layer = (blur > 0.0).then(|| blur_layer(blur, f.rect, Rect::new(-1e9, -1e9, 2e9, 2e9)));
        if let Some((begin, _)) = &layer {
            cmds.push((begin.clone(), world));
        }
        for (c, rel) in &f.cmds {
            let mut c = c.clone();
            fade_cmd(&mut c, fade);
            undraw(std::slice::from_mut(&mut c), drawn);
            cmds.push((c, world.mul(rel)));
        }
        if let Some((_, end)) = layer {
            cmds.push((end, world));
        }
    }

    fn scroll_clip(&self, s: usize) -> Rect {
        let sn = &self.nodes[s];
        if sn.placed.world.is_identity() {
            sn.placed.clip.intersect(&sn.placed.rect)
        } else {
            sn.placed
                .world
                .invert()
                .map_or(sn.placed.clip, |inv| inv.map_rect(&sn.placed.clip))
                .intersect(&sn.placed.rect)
        }
    }
}

/// A command's colours faded by `k` (a leaving element's opacity).
pub(crate) fn fade_cmd(c: &mut DrawCmd, k: f32) {
    match c {
        DrawCmd::Glass { alpha, .. } => *alpha *= k,
        DrawCmd::Quad { color, .. }
        | DrawCmd::Image { color, .. }
        | DrawCmd::Bar { color, .. }
        | DrawCmd::Half { color, .. } => color[3] *= k,
        DrawCmd::Text { color, glow, .. } => {
            color[3] *= k;
            if let Some((g, _)) = glow {
                g[3] *= k;
            }
        }
        // The UI pass's commands (crate::paint).
        DrawCmd::StyledText { text, .. } => fade_cmd(text, k),
        DrawCmd::Shape { shape, .. } => shape.opacity *= k,
        DrawCmd::Vector { tint, .. } => tint[3] *= k,
        DrawCmd::EndLayer { opacity, .. } => *opacity *= k,
        DrawCmd::World(w) => {
            if let crate::world::WorldCmd::Mass { batch, .. } = &mut **w {
                for i in &mut batch.instances {
                    i.color[3] *= k;
                }
            }
        }
        DrawCmd::PushClip { .. } | DrawCmd::PopClip | DrawCmd::BeginLayer { .. } => {}
    }
}

/// The layer a glitching subtree draws into and its composite: og-render's
/// glitch material (`UiBuiltin::Glitch`) at `g` (0 settled, 1 torn; a
/// stronger style past 1), its slices about a sixth of the element's
/// height and varied by its key, shifted up to 10 CSS px with a 3.5 px
/// colour split, re-cut every two ticks of the UI clock (`Ui::seconds`,
/// counted in ticks), so a captured frame is the same every run.
fn glitch_layer(
    key: u64,
    g: f32,
    height: f32,
    ticks_per_second: u32,
    rect: Rect,
    clip: Rect,
) -> (DrawCmd, DrawCmd) {
    // 0.8–1.2 from the key (splitmix64's finaliser).
    let mut x = key;
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^= x >> 31;
    let vary = 0.8 + 0.4 * ((x >> 40) as f32 / (1u64 << 24) as f32);
    let slice = (height / 6.0 * vary).clamp(4.0, 48.0);
    let split = 10.0 * g.min(1.5);
    let mut params = [[0.0; 4]; og_draw::UI_MATERIAL_PARAMS];
    params[0] = [
        g.clamp(0.0, 1.0),
        slice,
        split,
        ticks_per_second as f32 / 2.0,
    ];
    let begin = DrawCmd::BeginLayer {
        material: og_draw::UiMaterial::Builtin(og_draw::UiBuiltin::Glitch),
        params,
        mask: None,
        expand: split + 2.0,
        rect: Rect::default(),
        filter: None,
        role: Default::default(),
    };
    let end = DrawCmd::EndLayer {
        geometry: None,
        rect,
        opacity: 1.0,
        blend: og_draw::UiBlend::Normal,
        clip,
        ground: None,
    };
    (begin, end)
}

/// The layer a blurring subtree draws into and its composite: og-render's
/// blur material at `sigma` CSS px (CSS `blur()`'s standard deviation;
/// the material's radius is twice it), reaching past the box by the
/// Gaussian's tail.
fn blur_layer(sigma: f32, rect: Rect, clip: Rect) -> (DrawCmd, DrawCmd) {
    let mut params = [[0.0; 4]; og_draw::UI_MATERIAL_PARAMS];
    params[0][0] = 2.0 * sigma;
    let begin = DrawCmd::BeginLayer {
        material: og_draw::UiMaterial::Builtin(og_draw::UiBuiltin::Blur),
        params,
        mask: None,
        expand: 3.0 * sigma,
        rect: Rect::default(),
        filter: None,
        role: Default::default(),
    };
    let end = DrawCmd::EndLayer {
        geometry: None,
        rect,
        opacity: 1.0,
        blend: og_draw::UiBlend::Normal,
        clip,
        ground: None,
    };
    (begin, end)
}

/// Hides `hidden` (0 none … 1 all) of each stroke in `cmds` from its
/// end, as Svelte's `draw` does with `stroke-dasharray` over the
/// outline's length: one dash as long as the outline, then a gap as long.
fn undraw(cmds: &mut [DrawCmd], hidden: f32) {
    if hidden <= 0.0 {
        return;
    }
    for c in cmds {
        let DrawCmd::Shape { shape, .. } = c else {
            continue;
        };
        let og_draw::UiMode::Stroke(stroke) = &mut shape.mode else {
            continue;
        };
        let length = outline_length(&shape.geometry, shape.half);
        if length <= 0.0 {
            continue;
        }
        stroke.dash = [length, length];
        stroke.dash_offset = hidden * length;
    }
}

/// The length of a shape's outline in its local space, CSS px.
fn outline_length(geometry: &og_draw::UiGeometry, half: og_core::glam::Vec2) -> f32 {
    use og_draw::UiGeometry as G;
    let polyline = |points: &[og_core::glam::Vec2], closed: bool| {
        let open: f32 = points.windows(2).map(|w| w[0].distance(w[1])).sum();
        match (closed, points.first(), points.last()) {
            (true, Some(a), Some(b)) => open + a.distance(*b),
            _ => open,
        }
    };
    match geometry {
        G::Rect { radii, .. } => {
            let r = radii.iter().map(|r| r.min(half.x.min(half.y))).sum::<f32>();
            4.0 * (half.x + half.y) - r * (2.0 - std::f32::consts::FRAC_PI_2)
        }
        G::Ellipse => {
            // Ramanujan's second approximation.
            let (a, b) = (half.x, half.y);
            let h = ((a - b) / (a + b).max(1e-6)).powi(2);
            std::f32::consts::PI * (a + b) * (1.0 + 3.0 * h / (10.0 + (4.0 - 3.0 * h).sqrt()))
        }
        G::Arc { sweep, .. } => sweep.abs().min(std::f32::consts::TAU) * half.x.min(half.y),
        G::Polygon { .. } => 0.0,
        G::Polyline { points, closed, .. } => polyline(points, *closed),
        G::Path(path) => path
            .flatten(0.25)
            .iter()
            .map(|(points, closed)| polyline(points, *closed))
            .sum(),
    }
}

/// A command with its map applied where the map keeps rectangles upright
/// (translation and scale): the identity left over. A turn or skew stays
/// with the command for the overlay, its clip taken onto the screen; so
/// does a mirror over text, whose glyphs a pen position cannot flip (an
/// image flips by its uv).
/// Command `c` moved by `(dx, dy)` with its clip where it was (an element
/// that moved inside what clips it); `None` for what does not move so.
/// Whether [`shift`] can move `c` (a layer or world command cannot).
fn shiftable(c: &DrawCmd) -> bool {
    match c {
        DrawCmd::StyledText { text, .. } => shiftable(text),
        DrawCmd::BeginLayer { .. } | DrawCmd::EndLayer { .. } | DrawCmd::World(_) => false,
        _ => true,
    }
}

/// Moves `c` by `(dx, dy)` in place, its clip where it was; `false`, with
/// nothing moved, for what [`shiftable`] refuses.
fn shift(c: &mut DrawCmd, dx: f32, dy: f32) -> bool {
    if !shiftable(c) {
        return false;
    }
    let mv = |r: &mut Rect| {
        r.x += dx;
        r.y += dy;
    };
    match c {
        DrawCmd::Glass { rect, .. }
        | DrawCmd::Quad { rect, .. }
        | DrawCmd::Image { rect, .. }
        | DrawCmd::Vector { rect, .. } => mv(rect),
        DrawCmd::Bar { center, .. } | DrawCmd::Half { center, .. } => {
            center.0 += dx;
            center.1 += dy;
        }
        DrawCmd::Text {
            x,
            baseline,
            bounds,
            ..
        } => {
            *x += dx;
            *baseline += dy;
            mv(bounds);
        }
        DrawCmd::StyledText { text, effects } => {
            shift(text, dx, dy);
            effects.fill_box[0] += dx;
            effects.fill_box[1] += dy;
        }
        DrawCmd::Shape { shape, .. } => {
            shape.center.x += dx;
            shape.center.y += dy;
        }
        DrawCmd::PushClip { center, .. } => {
            center.0 += dx;
            center.1 += dy;
        }
        DrawCmd::PopClip
        | DrawCmd::BeginLayer { .. }
        | DrawCmd::EndLayer { .. }
        | DrawCmd::World(_) => {}
    }
    true
}

pub(crate) fn bake(c: DrawCmd, m: Affine) -> (DrawCmd, Affine) {
    if m.is_identity() {
        return (c, m);
    }
    let r = |r: &Rect| m.map_rect(r);
    let mirrored_text =
        (m.a < 0.0 || m.d < 0.0) && matches!(c, DrawCmd::Text { .. } | DrawCmd::StyledText { .. });
    if !m.is_axis_aligned() || mirrored_text {
        let mut c = c;
        match &mut c {
            DrawCmd::Glass { clip, .. }
            | DrawCmd::Quad { clip, .. }
            | DrawCmd::Image { clip, .. }
            | DrawCmd::Bar { clip, .. }
            | DrawCmd::Half { clip, .. }
            | DrawCmd::Text { clip, .. } => *clip = r(clip),
            DrawCmd::StyledText { text, .. } => {
                if let Some(clip) = text.clip_mut() {
                    *clip = r(clip);
                }
            }
            // The UI pass's own commands keep the screen's clip and take
            // the whole map where they are emitted.
            _ => {}
        }
        return (c, m);
    }
    let (sx, sy) = (m.a.abs(), m.d.abs());
    let k = sx.min(sy);
    let p = |x: f32, y: f32| m.apply(x, y);
    let c = match c {
        DrawCmd::Glass {
            rect,
            radius,
            blur,
            saturation,
            tint,
            mut material,
            alpha,
            clip,
        } => DrawCmd::Glass {
            rect: r(&rect),
            radius: radius * k,
            blur,
            saturation,
            tint,
            material: {
                material.edge *= k;
                for sh in &mut material.shadows {
                    sh.offset *= k;
                    sh.blur *= k;
                    sh.spread *= k;
                }
                material
            },
            alpha,
            clip: r(&clip),
        },
        DrawCmd::Quad {
            rect,
            radius,
            color,
            clip,
        } => DrawCmd::Quad {
            rect: r(&rect),
            radius: radius * k,
            color,
            clip: r(&clip),
        },
        DrawCmd::Image {
            rect,
            texture,
            uv,
            color,
            clip,
        } => {
            // A mirroring scale flips the picture: its region swaps ends.
            let mut uv = uv;
            if m.a < 0.0 {
                uv.swap(0, 2);
            }
            if m.d < 0.0 {
                uv.swap(1, 3);
            }
            DrawCmd::Image {
                rect: r(&rect),
                texture,
                uv,
                color,
                clip: r(&clip),
            }
        }
        DrawCmd::Bar {
            center,
            w,
            h,
            angle,
            color,
            clip,
        } => DrawCmd::Bar {
            center: p(center.0, center.1),
            w: w * sx,
            h: h * sy,
            angle,
            color,
            clip: r(&clip),
        },
        DrawCmd::Half {
            center,
            radius,
            angle,
            color,
            clip,
        } => DrawCmd::Half {
            center: p(center.0, center.1),
            radius: radius * k,
            angle,
            color,
            clip: r(&clip),
        },
        DrawCmd::Text {
            x,
            baseline,
            size,
            text,
            color,
            font,
            bounds,
            clip,
            tracking,
            weight,
            glow,
            placements,
        } => {
            let (x, baseline) = p(x, baseline);
            DrawCmd::Text {
                x,
                baseline,
                size: size * sy,
                text,
                color,
                font,
                bounds: r(&bounds),
                clip: r(&clip),
                tracking,
                weight: weight * sy,
                glow: glow.map(|(c, g)| (c, g * sy)),
                placements,
            }
        }
        DrawCmd::StyledText { text, mut effects } => {
            let (text, _) = bake(*text, m);
            crate::draw::scale_effects(&mut effects, &m);
            DrawCmd::StyledText {
                text: Box::new(text),
                effects,
            }
        }
        // The UI pass's own commands take the whole map where they are
        // emitted (a non-uniform scale bends a corner exactly).
        other => return (other, m),
    };
    (c, Affine::IDENTITY)
}

/// What `Ui::paint_reuse` found, per node: judged this frame where
/// stamped, else as it painted last frame (same, clean, unmoved).
#[derive(Clone, Debug, Default)]
pub(crate) struct Reuse {
    stamp: u32,
    at: Vec<u32>,
    same: Vec<bool>,
    clean: Vec<bool>,
    delta: Vec<(f32, f32)>,
    /// The nodes painted anew (`None`: the whole frame is).
    pub repainted: Option<Vec<usize>>,
}

impl Reuse {
    fn begin(&mut self, n: usize) {
        self.stamp = self.stamp.wrapping_add(1);
        if self.stamp == 0 {
            self.at.fill(0);
            self.stamp = 1;
        }
        if self.at.len() < n {
            self.at.resize(n, 0);
            self.same.resize(n, false);
            self.clean.resize(n, false);
            self.delta.resize(n, (0.0, 0.0));
        }
    }

    fn set(&mut self, i: usize, same: bool, clean: bool, delta: (f32, f32)) {
        self.at[i] = self.stamp;
        (self.same[i], self.clean[i], self.delta[i]) = (same, clean, delta);
    }

    fn judged(&self, i: usize) -> bool {
        self.at[i] == self.stamp
    }

    fn same(&self, i: usize) -> bool {
        !self.judged(i) || self.same[i]
    }

    fn clean(&self, i: usize) -> bool {
        !self.judged(i) || self.clean[i]
    }

    fn delta(&self, i: usize) -> (f32, f32) {
        if self.judged(i) {
            self.delta[i]
        } else {
            (0.0, 0.0)
        }
    }

    /// Node `i` is not copied whole (as judged, or as last frame).
    fn unclean(&mut self, i: usize) {
        if !self.judged(i) {
            self.set(i, true, false, (0.0, 0.0));
        } else {
            self.clean[i] = false;
        }
    }
}

impl crate::ui::Tree {
    /// The value `get` gives of node `i` or its nearest ancestor that has
    /// one (an inherited CSS colour of a control's parts).
    pub(crate) fn nearest<T>(
        &self,
        mut i: usize,
        get: impl Fn(&crate::layout::Node) -> Option<T>,
    ) -> Option<T> {
        while i != crate::layout::NONE && i < self.nodes.len() {
            if let Some(v) = get(&self.nodes[i]) {
                return Some(v);
            }
            i = self.nodes[i].parent;
        }
        None
    }
}

/// Everything: no clip.
const INFINITE: Rect = Rect {
    x: -1e9,
    y: -1e9,
    width: 2e9,
    height: 2e9,
};

/// The box `transform-box` names: its border box or its content box.
fn reference_box(n: &crate::layout::Node, b: crate::transform3d::TransformBox) -> Rect {
    let r = n.placed.rect;
    match b {
        crate::transform3d::TransformBox::BorderBox => r,
        crate::transform3d::TransformBox::ContentBox => {
            let p = n.el.padding;
            Rect::new(
                r.x + p.left,
                r.y + p.top,
                (r.width - p.left - p.right).max(0.0),
                (r.height - p.top - p.bottom).max(0.0),
            )
        }
    }
}

/// The shared elements' ranges after a command was inserted at `at`.
fn shift_named(named: &mut [crate::view_transition::Named], at: usize) {
    for n in named.iter_mut().filter(|n| n.range.start >= at) {
        n.range = n.range.start + 1..n.range.end + 1;
    }
}

fn plane_begin() -> DrawCmd {
    DrawCmd::BeginLayer {
        material: og_draw::UiMaterial::Plain,
        params: [[0.0; 4]; og_draw::UI_MATERIAL_PARAMS],
        mask: None,
        expand: 0.0,
        rect: Rect::default(),
        filter: None,
        role: Default::default(),
    }
}

/// A 3D plane's composite, laid by its quad of the frame
/// ([`Ui::plane_quads`]) and cut by `clip` in the space it is laid onto.
fn plane_end(key: u64, clip: Rect) -> DrawCmd {
    DrawCmd::EndLayer {
        geometry: None,
        rect: Rect::default(),
        opacity: 1.0,
        blend: og_draw::UiBlend::Normal,
        clip,
        ground: Some(crate::draw::Ground::Plane(key)),
    }
}

impl crate::ui::Layout {
    /// This tick's quads of the CSS 3D planes (`world.grounds`): each
    /// root's layer laid through its map onto the space it stands in, with
    /// its context and depth there. One that shows a hidden back face has
    /// none (it is not drawn); a root gone from the tree leaves the set.
    pub(super) fn plane_quads(
        &mut self,
        world: &mut crate::world::WorldUi,
        tree: &crate::ui::Tree,
    ) {
        let keys: Vec<u64> = world.placement.planes.iter().copied().collect();
        for key in keys {
            let found = tree.index.find(key).filter(|&i| {
                tree.nodes[i].key == key
                    && tree.nodes[i].placed.plane.as_ref().is_some_and(|p| p.root)
            });
            let Some(i) = found else {
                world.placement.planes.remove(&key);
                continue;
            };
            let n = &tree.nodes[i];
            let p = n.placed.plane.as_deref().expect("a plane root");
            if p.hidden {
                continue;
            }
            let map = crate::transform3d::project(&p.m);
            let Some(inv) = map.invert() else { continue };
            // The screen (or the space it is laid onto) to its layer: rows
            // b, with a ray the drape meets at w = 1 (og_draw::UiGround).
            let mut b = inv.rows();
            let (cx, cy) = n.placed.rect.center();
            let (sx, sy) = map.apply(cx, cy);
            let w = b[2][0] * sx + b[2][1] * sy + b[2][2];
            if w < 0.0 {
                b = b.map(|r| r.map(|v| -v));
            }
            let r = n.placed.rect;
            let corner = |x: f32, y: f32| {
                let w = map.g * x + map.h * y + map.i;
                (w > 1e-6).then(|| {
                    let (px, py) = map.apply(x, y);
                    og_core::glam::Vec2::new(px, py)
                })
            };
            let corners = [
                corner(r.x, r.y),
                corner(r.right(), r.y),
                corner(r.right(), r.bottom()),
                corner(r.x, r.bottom()),
            ];
            let corners = corners
                .iter()
                .all(Option::is_some)
                .then(|| corners.map(|c| c.unwrap_or_default()));
            let (centre, _) = crate::transform3d::apply(&p.m, [cx, cy, 0.0]);
            world.placement.grounds.push(crate::GroundQuad {
                key,
                rect: r,
                corners,
                bounds: p.clip,
                drape: og_draw::UiDrape {
                    a: [[0.0; 3], [0.0; 3], [0.0, 0.0, -1.0]],
                    b,
                    scale: og_core::glam::Vec2::ONE,
                    reach: 0.0,
                },
                context: p.context,
                depth: centre[2],
            });
        }
    }
}
