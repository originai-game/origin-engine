use super::*;

impl Ui {
    /// Staggered entrances: under a node with a stagger, the k-th drawn
    /// leaf (a label, a control, an icon) enters `k` steps after the first;
    /// a long list shortens the step so the cascade ends within one more
    /// entrance. The cascade is only seen: a control takes input as soon as
    /// the panel's own entrance (inert while it moves) has played, where it
    /// is drawn, not when its place in the cascade has.
    pub(super) fn apply_stagger(&mut self, order: &[usize]) {
        let now = self.state.tick as f32;
        let (reduced, distance) = (self.theme.reduced_motion, self.theme.space.lg);
        for &i in order {
            let Some(st) = self.nodes[i].stagger else {
                continue;
            };
            let leaves: Vec<usize> = preorder(&self.nodes, i)
                .into_iter()
                .skip(1)
                .filter(|&j| {
                    let n = &self.nodes[j];
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
                if p >= 1.0 || self.nodes[j].fx.is_some() {
                    continue;
                }
                self.nodes[j].fx = Some(crate::motion::fx(
                    st.kind,
                    self.theme
                        .tokens
                        .ease(crate::tokens::EaseRole::WidgetEnter)
                        .apply(p),
                    distance,
                    reduced,
                ));
            }
        }
    }

    /// The tick `key` was first seen on (kept while it is asked every tick).
    pub(crate) fn since(&mut self, key: u64) -> u64 {
        let fresh = !self.has_anim(key);
        let mut a = self.anim(key);
        if fresh {
            a.t = self.state.tick;
        }
        self.set_anim(key, a);
        a.t
    }

    /// Each element's map onto the screen (docs/ui.md §2): its layout
    /// animation, its drag and pinch, its transition's movement and its own
    /// transform, about its box, composed with its parent's; its opacity
    /// multiplied down the tree; inert where a transition moves it, a state
    /// or timeline holds it, or its parent is. Clips made by a transformed
    /// scroll container move with it.
    pub(super) fn apply_world(&mut self, order: &[usize]) {
        for &i in order {
            let parent = self.nodes[i].parent;
            let (alpha, inert, pw) = if parent == NONE {
                (1.0, false, Affine::IDENTITY)
            } else {
                let p = &self.nodes[parent];
                (p.alpha, p.inert, p.world)
            };
            let n = &mut self.nodes[i];
            n.alpha = alpha * n.sampled.opacity.clamp(0.0, 1.0);
            n.scale = 1.0;
            n.inert = inert || n.sampled.inert;
            let r = n.rect;
            let (cx, cy) = r.center();
            let mut local = Affine::IDENTITY;
            let [fx, fy, fsx, fsy] = n.flip;
            if fx != 0.0 || fy != 0.0 || fsx != 1.0 || fsy != 1.0 {
                local = Affine::translate(fx, fy).mul(&Affine::scale(fsx, fsy).about(r.x, r.y));
            }
            let [hx, hy, hs, ho] = n.custom;
            n.alpha *= ho.clamp(0.0, 1.0);
            if hx != 0.0 || hy != 0.0 || hs != 1.0 {
                local = local
                    .mul(&Affine::translate(hx, hy))
                    .mul(&Affine::scale(hs, hs).about(cx, cy));
            }
            let [gx, gy, gs, gr] = n.gesture;
            if gx != 0.0 || gy != 0.0 || gs != 1.0 || gr != 0.0 {
                let turn = Affine::rotate(gr).mul(&Affine::scale(gs, gs)).about(cx, cy);
                local = local.mul(&Affine::translate(gx, gy)).mul(&turn);
            }
            if let Some(f) = n.fx {
                n.alpha *= f.alpha;
                n.inert |= f.inert;
                if f.dx != 0.0 || f.dy != 0.0 || f.scale != 1.0 {
                    local = local
                        .mul(&Affine::translate(f.dx, f.dy))
                        .mul(&Affine::scale(f.scale, f.scale).about(cx, cy));
                }
            }
            if !n.sampled.transform.is_identity() {
                local = local.mul(&n.sampled.transform.matrix(&r));
            }
            n.world = if local.is_identity() {
                pw
            } else {
                pw.mul(&local)
            };
            n.screen = n.world.map_rect(&r);
        }
        // A transformed scroll container's clip moves with it: made from
        // the clip layout gave (mirrored right to left), so a node layout
        // left out is not moved twice.
        let mirror = (self.direction == og_i18n::Direction::Rtl).then_some(self.viewport.width);
        for &i in order {
            let src = self.nodes[i].clip_src;
            if src != NONE && !self.nodes[src].world.is_identity() {
                let w = self.nodes[src].world;
                let mut c = self.nodes[i].laid_clip;
                if let Some(width) = mirror {
                    c.x = width - c.x - c.width;
                }
                self.nodes[i].clip = w.map_rect(&c);
            }
        }
    }

    /// The frame's draw commands with their maps (docs/ui.md §2): each
    /// element's in its layout box, mapped by its world (an axis-aligned
    /// map is baked into the command, a turn or skew is left for the
    /// overlay); a scroll container's bars after its content; leaving
    /// elements (ghosts) after their parent's subtree, or last; and the
    /// drawing of every element with an exit or a match key frozen for
    /// next tick.
    pub(super) fn commands(&mut self, order: &[usize]) -> Vec<(DrawCmd, Affine)> {
        let dpr = self.viewport.dpr();
        let snap = |v: f32| (v * dpr).round() / dpr;
        let ends = crate::layout::subtree_ends(&self.nodes, order);
        let mut cmds: Vec<(DrawCmd, Affine)> = Vec::new();
        let ghosts = std::mem::take(&mut self.state.motion.ghosts);
        let present: crate::id::KeySet = order.iter().map(|&i| self.nodes[i].key).collect();
        let mut drawn = vec![false; ghosts.len()];
        let mut frozen: IdMap<u64, crate::motion::state::Frozen> = IdMap::new();
        // Open subtrees: (node, its first command, the order position past it).
        let mut open: Vec<(usize, usize, usize)> = Vec::new();
        // Focus rings waiting for the subtree they draw over (`ring_over`).
        let mut rings: Vec<(usize, Vec<(DrawCmd, Affine)>)> = Vec::new();
        // Elements whose commands and ancestors' fills are as last frame
        // (docs/ui.md §2): their commands are last frame's.
        let mut same = vec![false; self.nodes.len()];
        let mut next = crate::id::KeyMap::<(usize, Vec<DrawCmd>, crate::layout::DrawSig)>::with_capacity_and_hasher(
            order.len(),
            Default::default(),
        );
        let prev_kinds = std::mem::take(&mut self.prev_kinds);
        self.kept_spans.clear();
        let mut written = vec![false; self.nodes.len()];
        for &i in &self.emitted {
            written[i] = true;
        }
        let theme = self.theme;
        // An element's draw (crate::paint) in its layout box, at its
        // opacity; its clip is the screen's (bake leaves draw commands be).
        let press_scale = theme.press_scale;
        let marked = |n: &crate::layout::Node| {
            n.el.part
                .is_some_and(|p| p.has("selected") || p.has("focused"))
        };
        let placed = |n: &crate::layout::Node| crate::paint::Placed {
            rect: part_rect(n, press_scale),
            radius: n.sampled.radius,
            clip: n.clip,
            alpha: n.alpha,
        };
        for (k, &i) in order.iter().enumerate() {
            let n = &self.nodes[i];
            let world = n.world;
            let start = cmds.len();
            let game_draw = self.draws.contains_key(&i);
            let draw = self
                .draws
                .get(&i)
                .or_else(|| self.part_draws.get(&i))
                .cloned();
            let n = &self.nodes[i];
            if let Some(m) = self.world_marks(n.key) {
                cmds.extend(m.enter.iter().map(|c| (c.clone(), Affine::IDENTITY)));
            }
            // One order for game and style draws, over the element's
            // background (`set_draw`): ground, rim, content, then children
            // and raised badges; a selected or focused part's rim (its
            // mark) after its children (`marked`), so the item's art never
            // covers it.
            let over_background = |ui: &Self| -> Vec<(DrawCmd, Affine)> {
                let Some(d) = &draw else { return Vec::new() };
                let n = &ui.nodes[i];
                let mut out = d.enter(&theme, &placed(n));
                if !marked(n) {
                    out.extend(d.strokes(&theme, &placed(n)));
                }
                out.into_iter().map(|c| (c, world)).collect()
            };
            // Rich text draws from the tick (its moving spans), focus and
            // the hot link as well: it is always made again.
            let sig = crate::layout::DrawSig::of(n);
            same[i] = !self.kept.fresh
                && !matches!(n.content, Content::Rich(_))
                && match self.kept.prev.get(&i) {
                    Some(b) => {
                        crate::layout::draw_eq(n, b) && Some(&self.kinds[i]) == prev_kinds.get(&i)
                    }
                    // A retained node not written again: its own inputs are
                    // as they were; what the frame moves is in its signature.
                    None => {
                        !written[i]
                            && self
                                .kept
                                .cmds
                                .get(&n.key)
                                .is_some_and(|(_, _, old)| *old == sig)
                    }
                }
                && (n.parent == NONE || same[n.parent])
                && !game_draw;
            if i != 0 || n.sampled.background.is_some() {
                let mut local = Vec::new();
                let mut background = 0;
                if same[i]
                    && let Some((b, c, _)) = self.kept.cmds.get(&n.key)
                {
                    (background, local) = (*b, c.clone());
                    self.stats.kept_commands += 1;
                } else if world.is_identity() {
                    self.stats.painted += 1;
                    background = self.node_commands(i, &mut local, &snap);
                } else if let Some(inv) = world.invert() {
                    self.stats.painted += 1;
                    // Drawn in its layout box: its screen clip taken there.
                    let screen_clip = n.clip;
                    self.nodes[i].clip = inv.map_rect(&screen_clip);
                    self.drawing = world;
                    background = self.node_commands(i, &mut local, &snap);
                    self.drawing = Affine::IDENTITY;
                    self.nodes[i].clip = screen_clip;
                }
                if let Some(d) = &draw {
                    let grid = match &self.nodes[i].content {
                        Content::Text(t) => t.metrics(&self.fonts).pixel_grid(),
                        _ => 0,
                    };
                    d.style_text(&theme, &mut local, grid);
                }
                next.insert(self.nodes[i].key, (background, local.clone(), sig));
                let kept_at = same[i].then_some(local.len());
                let mut local = local.into_iter().map(|c| (c, world));
                let from = cmds.len();
                cmds.extend(local.by_ref().take(background));
                let behind = cmds.len();
                cmds.extend(over_background(self));
                let over = cmds.len();
                cmds.extend(local);
                // Kept commands were read for missing glyphs when painted.
                if kept_at.is_some() {
                    self.kept_spans.push(from..behind);
                    self.kept_spans.push(over..cmds.len());
                }
            } else {
                cmds.extend(over_background(self));
            }
            let mut ring = Vec::new();
            if world.is_identity() {
                self.focus_ring_commands(i, &mut ring);
            } else if let Some(inv) = world.invert() {
                let screen_clip = self.nodes[i].clip;
                self.nodes[i].clip = inv.map_rect(&screen_clip);
                self.drawing = world;
                self.focus_ring_commands(i, &mut ring);
                self.drawing = Affine::IDENTITY;
                self.nodes[i].clip = screen_clip;
            }
            if !ring.is_empty() {
                rings.push((
                    self.ring_over(i),
                    ring.into_iter().map(|c| (c, world)).collect(),
                ));
            }
            if let Some(d) = &draw {
                let at = placed(&self.nodes[i]);
                cmds.extend(d.children(&at).into_iter().map(|c| (c, world)));
            }
            open.push((i, start, ends[k]));
            while let Some(&(j, first, end)) = open.last() {
                if end != k + 1 {
                    break;
                }
                open.pop();
                let key = self.nodes[j].key;
                for (g, ghost) in ghosts.iter().enumerate() {
                    if !drawn[g] && ghost.frozen.parent == key {
                        drawn[g] = true;
                        self.ghost_commands(ghost, &mut cmds);
                    }
                }
                if self.nodes[j].scrolls() && self.nodes[j].el.fade > 0.0 {
                    let mut fades = Vec::new();
                    self.scroll_fade_commands(j, &mut fades);
                    let w = self.nodes[j].world;
                    cmds.extend(fades.into_iter().map(|c| (c, w)));
                }
                if self.nodes[j].scrolls() && !self.nodes[j].el.hide_bars {
                    let mut bars = Vec::new();
                    self.scroll_bar_commands(j, &mut bars);
                    let w = self.nodes[j].world;
                    cmds.extend(bars.into_iter().map(|c| (c, w)));
                }
                let w = self.nodes[j].world;
                if let Some(d) = self.draws.get(&j).or_else(|| self.part_draws.get(&j)) {
                    let at = placed(&self.nodes[j]);
                    let leave = if marked(&self.nodes[j]) {
                        d.leave(&theme, &at)
                    } else {
                        d.leave_stroked(&at)
                    };
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
                if let Some(m) = self.world_marks(key) {
                    cmds.extend(m.leave.iter().map(|c| (c.clone(), Affine::IDENTITY)));
                }
                let nj = &self.nodes[j];
                if nj.props.exit.is_some() || !nj.props.match_key.is_empty() {
                    let inv = nj.world.invert().unwrap_or(Affine::IDENTITY);
                    let parent = if nj.parent == NONE {
                        0
                    } else {
                        self.nodes[nj.parent].key
                    };
                    frozen.insert(
                        key,
                        crate::motion::state::Frozen {
                            cmds: cmds[first..]
                                .iter()
                                .filter(|(c, _)| !matches!(c, DrawCmd::World(_)))
                                .map(|(c, m)| (c.clone(), inv.mul(m)))
                                .collect(),
                            rect: nj.rect,
                            world: nj.world,
                            alpha: nj.alpha,
                            parent: if present.contains(&parent) { parent } else { 0 },
                            path: self.path_of(j),
                            exit: nj.props.exit.clone(),
                            match_key: if nj.props.match_key.is_empty() {
                                0
                            } else {
                                crate::motion::state::name_hash(&nj.props.match_key)
                            },
                            radius: nj.sampled.radius,
                        },
                    );
                }
                // A glitching subtree draws through the glitch material
                // (TransitionKind::Glitch): a layer from its first command
                // to its last.
                if let Some(g) = self.nodes[j].fx.map(|f| f.glitch).filter(|&g| g > 0.0) {
                    let nj = &self.nodes[j];
                    let strength = match theme.art.motion.glitch {
                        s if s > 0.0 => s,
                        _ => 1.0,
                    };
                    let (begin, end) = glitch_layer(
                        key,
                        g * strength,
                        nj.screen.height,
                        self.ticks_per_second,
                        nj.rect,
                        nj.clip,
                    );
                    // Every later command moves one place: no span is kept.
                    self.kept_spans.retain(|r| r.end <= first);
                    cmds.insert(first, (begin, nj.world));
                    cmds.push((end, nj.world));
                }
            }
        }
        for (g, ghost) in ghosts.iter().enumerate() {
            if !drawn[g] {
                self.ghost_commands(ghost, &mut cmds);
            }
        }
        self.state.motion.ghosts = ghosts;
        self.state.motion.frozen = frozen;
        self.kept.cmds = next;
        cmds
    }

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
                .map(|n| n.screen);
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
        for (c, rel) in &f.cmds {
            let mut c = c.clone();
            fade_cmd(&mut c, fade);
            cmds.push((c, world.mul(rel)));
        }
    }

    /// A scroll container's bars, over its content.
    /// A scroll container's clip in its own space.
    /// The subtree node `i`'s focus ring draws after: its screen's (the
    /// root's child it is in), or the world-placed element's it is in, so
    /// a ring stays in that element's space.
    fn ring_over(&self, i: usize) -> usize {
        let mut j = i;
        loop {
            let parent = self.nodes[j].parent;
            if parent == NONE || parent == 0 || self.world_marks(self.nodes[j].key).is_some() {
                return j;
            }
            j = parent;
        }
    }

    fn scroll_clip(&self, s: usize) -> Rect {
        let sn = &self.nodes[s];
        if sn.world.is_identity() {
            sn.clip.intersect(&sn.rect)
        } else {
            sn.world
                .invert()
                .map_or(sn.clip, |inv| inv.map_rect(&sn.clip))
                .intersect(&sn.rect)
        }
    }

    /// The fade over each edge of a scroll container where more of its
    /// content lies beyond, as strong as the content past that edge is
    /// over the fade's reach: none at the top of a list, full a fade's
    /// length into it, so a cut row reads as "there is more".
    fn scroll_fade_commands(&self, s: usize, cmds: &mut Vec<DrawCmd>) {
        use crate::paint::{ColorRef, ElementDraw, Gradient, Paint, Placed, Stop};
        let sn = &self.nodes[s];
        let reach = sn.el.fade;
        let into = sn.el.fade_into.unwrap_or(self.theme.palette.panel);
        let clip = self.scroll_clip(s);
        let r = sn.rect;
        let max_y = sn.max_scroll();
        let off_y = self
            .state
            .scroll
            .get(&sn.key)
            .copied()
            .unwrap_or(0.0)
            .clamp(0.0, max_y);
        let max_x = sn.max_scroll_x();
        let off_x = self
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
                alpha: sn.alpha * k,
            };
            cmds.extend(draw.enter(&self.theme, &at));
        }
    }

    fn scroll_bar_commands(&self, s: usize, cmds: &mut Vec<DrawCmd>) {
        let t = &self.theme;
        let p = &t.palette;
        let dpr = self.viewport.dpr();
        let sn = &self.nodes[s];
        let clip = self.scroll_clip(s);
        let bar = |axis, rect: Rect, cmds: &mut Vec<DrawCmd>| {
            let part = crate::dress::PartRef::of("Scroll", "bar", None).attr("axis", axis);
            let dressing = self.dress.resolve(&part);
            if dressing.matched == 0 {
                cmds.push(DrawCmd::Quad {
                    rect: rect.snapped(dpr),
                    radius: t.radius.bar,
                    color: p.control_strong.linear(),
                    clip,
                });
            } else {
                self.part_commands_at(
                    &part,
                    dressing.draw.subpart_rect(rect, sn.rect).snapped(dpr),
                    Some(dressing.draw.subpart_color(p.control_strong, t)),
                    t.radius.bar,
                    1.0,
                    clip,
                    cmds,
                );
            }
        };
        let max_x = sn.max_scroll_x();
        if max_x > 0.0 {
            let inner_w = sn.w - sn.el.padding.horizontal();
            let track = sn.rect;
            let bar_w = crate::scroll::bar_len(track.width, inner_w, sn.content_w, t.space.xl);
            let off = self.state.scroll_x.get(&sn.key).copied().unwrap_or(0.0);
            let x = crate::scroll::bar_start(track.x, track.width, bar_w, off, max_x);
            let h = t.space.xs;
            bar(
                "horizontal",
                Rect::new(x, track.bottom() - h - t.space.xxs, bar_w, h),
                cmds,
            );
        }
        let max = sn.max_scroll();
        if max > 0.0 {
            let inner_h = sn.h - sn.el.padding.vertical();
            let track = sn.rect;
            let bar_h = (track.height * inner_h / sn.content_h).max(t.space.xl);
            let off = self
                .state
                .scroll
                .get(&sn.key)
                .copied()
                .unwrap_or(0.0)
                .clamp(0.0, max);
            let y = track.y + (track.height - bar_h) * off / max;
            let w = t.space.xs;
            bar(
                "vertical",
                Rect::new(track.right() - w - t.space.xxs, y, w, bar_h),
                cmds,
            );
        }
    }
}

/// A command's colours faded by `k` (a leaving element's opacity).
fn fade_cmd(c: &mut DrawCmd, k: f32) {
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
    let mut params = [[0.0; 4]; og_render::UI_MATERIAL_PARAMS];
    params[0] = [
        g.clamp(0.0, 1.0),
        slice,
        split,
        ticks_per_second as f32 / 2.0,
    ];
    let begin = DrawCmd::BeginLayer {
        material: og_render::UiMaterial::Builtin(og_render::UiBuiltin::Glitch),
        params,
        mask: None,
        expand: split + 2.0,
        rect: Rect::default(),
    };
    let end = DrawCmd::EndLayer {
        geometry: None,
        rect,
        opacity: 1.0,
        blend: og_render::UiBlend::Normal,
        clip,
    };
    (begin, end)
}

/// A command with its map applied where the map keeps rectangles upright
/// (translation and scale): the identity left over. A turn or skew stays
/// with the command for the overlay, its clip taken onto the screen; so
/// does a mirror over text, whose glyphs a pen position cannot flip (an
/// image flips by its uv).
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
