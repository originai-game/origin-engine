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

/// The first spot clear of `placed` for `b` (moved by `shift` mode), or
/// `None`: overlapping; `reach` bounds the move, and a moved box stays
/// inside `inside` (a contained tag's safe area: a crowd stacked off the
/// top of the screen hides its lowest-ranked instead).
fn declutter(
    mode: Declutter,
    b: Rect,
    placed: &[Rect],
    gap: f32,
    reach: f32,
    inside: Rect,
) -> Option<Vec2> {
    let clear = |r: &Rect| !placed.iter().any(|p| p.overlaps(r));
    if clear(&b) {
        return Some(Vec2::ZERO);
    }
    let within = |r: &Rect| {
        r.x >= inside.x - 0.5
            && r.y >= inside.y - 0.5
            && r.right() <= inside.right() + 0.5
            && r.bottom() <= inside.bottom() + 0.5
    };
    let moved = |d: Vec2| Rect::new(b.x + d.x, b.y + d.y, b.width, b.height);
    if mode == Declutter::Shift {
        // The shortest way clear of each box it meets, nearest first.
        let mut tries: Vec<Vec2> = Vec::new();
        for p in placed.iter().filter(|p| p.overlaps(&b)) {
            tries.push(Vec2::new(0.0, p.y - gap - b.bottom()));
            tries.push(Vec2::new(0.0, p.bottom() + gap - b.y));
            tries.push(Vec2::new(p.x - gap - b.right(), 0.0));
            tries.push(Vec2::new(p.right() + gap - b.x, 0.0));
        }
        og_core::sort::sort_by_key(&mut tries, |d| (d.length() * 16.0) as u32);
        if let Some(d) = tries
            .into_iter()
            .find(|&d| d.length() <= reach && clear(&moved(d)) && within(&moved(d)))
        {
            return Some(d);
        }
    }
    if matches!(mode, Declutter::Shift | Declutter::Stack) {
        // Up over what it meets, again and again; where that leaves the
        // area, down under it instead.
        for up in [true, false] {
            let mut d = Vec2::ZERO;
            for _ in 0..16 {
                let r = moved(d);
                let hit = placed.iter().filter(|p| p.overlaps(&r));
                let edge = if up {
                    hit.map(|p| p.y).reduce(f32::min)
                } else {
                    hit.map(|p| p.bottom()).reduce(f32::max)
                };
                let Some(edge) = edge else {
                    if within(&r) {
                        return Some(d);
                    }
                    break;
                };
                d.y = if up {
                    edge - gap - b.bottom()
                } else {
                    edge + gap - b.y
                };
                if d.y.abs() > reach {
                    break;
                }
            }
        }
    }
    None
}

impl Ui {
    pub(super) fn world_metas(&mut self) {
        let metas = std::mem::take(&mut self.world.metas);
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
            self.world.marks.insert(
                key,
                Marks {
                    enter: cmds,
                    leave: Vec::new(),
                },
            );
        }
        self.world.metas = metas;
    }
}

impl Ui {
    pub(super) fn world_anchors(&mut self) {
        let vp = self.env.viewport;
        let screen = Rect::new(0.0, 0.0, vp.width, vp.height);
        let safe = Rect::new(
            vp.safe_area.left,
            vp.safe_area.top,
            vp.width - vp.safe_area.horizontal(),
            vp.height - vp.safe_area.vertical(),
        );
        let anchors = std::mem::take(&mut self.world.anchors);
        self.world.placed.clear();
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
            rect: Rect,
            arrow: Option<(f32, Srgba, f32)>,
        }
        let mut placed: Vec<Placed> = Vec::new();
        let view = self.world.view;
        for (e, a) in anchors.iter().enumerate() {
            let n = &self.tree.nodes[a.node];
            let key = n.key;
            let r = n.rect;
            let d = &a.desc;
            let mut info = AnchorInfo {
                scale: 1.0,
                ..AnchorInfo::default()
            };
            let (Some(view), Some((p, _))) = (view, self.world.pose_of(d.target)) else {
                placed.push(Placed {
                    entry: e,
                    key,
                    at: Vec2::ZERO,
                    pivot: d.pivot.fractions(),
                    scale: 1.0,
                    alpha: 0.0,
                    depth: 0.0,
                    info,
                    rect: Rect::default(),
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
                alpha *= k * k * (3.0 - 2.0 * k);
                let g = self.env.theme.space.sm;
                let safe = Rect::new(
                    safe.x + g,
                    safe.y + g,
                    (safe.width - 2.0 * g).max(0.0),
                    (safe.height - 2.0 * g).max(0.0),
                );
                let b = Rect::new(
                    at.x - pivot.0 * half.x * 2.0,
                    at.y - pivot.1 * half.y * 2.0,
                    half.x * 2.0,
                    half.y * 2.0,
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
            let rect = Rect::new(
                at.x - pivot.0 * half.x * 2.0,
                at.y - pivot.1 * half.y * 2.0,
                half.x * 2.0,
                half.y * 2.0,
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
                rect,
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
        let gap = self.env.theme.space.xs;
        let mut taken: Vec<Rect> = Vec::new();
        let mut moves = vec![(Vec2::ZERO, 1.0f32); placed.len()];
        for &k in &by_rank {
            let pl = &placed[k];
            if pl.alpha <= 0.0 || pl.rect.is_empty() {
                continue;
            }
            let mode = anchors[pl.entry].desc.declutter;
            if mode == Declutter::None {
                taken.push(pl.rect);
                continue;
            }
            let reach = 3.0 * pl.rect.height.max(pl.rect.width.min(160.0));
            let a = &anchors[pl.entry].desc;
            let inside = if a.contain && a.clamp.is_none() {
                safe
            } else {
                Rect::new(-1e6, -1e6, 2e6, 2e6)
            };
            match declutter(mode, pl.rect, &taken, gap, reach, inside) {
                Some(d) => {
                    let r = pl.rect;
                    taken.push(Rect::new(r.x + d.x, r.y + d.y, r.width, r.height));
                    moves[k] = (d, 1.0);
                }
                None => moves[k] = (Vec2::ZERO, 0.0),
            }
        }
        // Eased toward this tick's values (world state), applied.
        let mut seen = Vec::new();
        for (k, pl) in placed.iter().enumerate() {
            let d = &anchors[pl.entry].desc;
            let (shift, keep) = moves[k];
            let target_alpha = pl.alpha * keep;
            let fresh = !self.world.state.smooth.contains_key(&pl.key);
            let s = self.world.state.smooth.entry(pl.key).or_default();
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
            let r = self.tree.nodes[node].rect;
            let px = r.x + pl.pivot.0 * r.width;
            let py = r.y + pl.pivot.1 * r.height;
            let at = pl.at + Vec2::new(s.dx, s.dy);
            let n = &mut self.tree.nodes[node];
            n.sampled.transform = Transform {
                x: at.x - px,
                y: at.y - py,
                scale_x: pl.scale,
                scale_y: pl.scale,
                pivot_x: pl.pivot.0,
                pivot_y: pl.pivot.1,
                ..n.sampled.transform
            };
            n.sampled.opacity *= s.alpha;
            if s.alpha < 0.05 {
                n.sampled.inert = true;
            }
            let mut info = pl.info;
            info.x = at.x;
            info.y = at.y;
            info.alpha = s.alpha;
            self.world.last.insert(pl.key, info);
            // Bracketed by its key, so a frame drawn between ticks moves it
            // with its point (`UiFrame::emit_shifted`).
            let mut marks = Marks::default();
            marks
                .enter
                .push(DrawCmd::World(Box::new(WorldCmd::Anchor { key: pl.key })));
            self.world.placed.push(super::AnchorPoint {
                key: pl.key,
                target: d.target,
                offset: d.offset,
                clamped: pl.info.clamped,
            });
            if let Some(view) = self.world.view
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
            self.world.marks.insert(pl.key, marks);
        }
        self.world.state.smooth.retain(|k, _| seen.contains(k));
        self.world.last.retain(|k, _| seen.contains(k));
        self.world.anchors = anchors;
    }

    pub(super) fn world_panels(&mut self) {
        let mut hits = Vec::new();
        let mut grounds = Vec::new();
        for p in &self.world.panels {
            let key = self.tree.nodes[p.node].key;
            if p.desc.projected {
                // Drawn by the UI onto where its quad is seen this tick: the
                // brackets are the same every tick (the quad goes in the
                // frame, `UiFrame::grounds`), so a moving camera relevels
                // and rebuilds nothing. Inert: no ray, no focus.
                self.tree.nodes[p.node].sampled.inert = true;
                let all = Rect::new(-1e9, -1e9, 2e9, 2e9);
                self.world.marks.insert(
                    key,
                    Marks {
                        enter: vec![DrawCmd::BeginLayer {
                            material: og_draw::UiMaterial::Plain,
                            params: [[0.0; 4]; og_draw::UI_MATERIAL_PARAMS],
                            mask: None,
                            expand: 0.0,
                            rect: Rect::default(),
                            filter: None,
                            role: Default::default(),
                        }],
                        leave: vec![DrawCmd::EndLayer {
                            geometry: None,
                            rect: self.tree.nodes[p.node].rect,
                            opacity: 1.0,
                            blend: og_draw::UiBlend::Normal,
                            clip: all,
                            ground: Some(crate::draw::Ground::Panel(key)),
                        }],
                    },
                );
                if let (Some(view), Some((t, rotation, size))) = (self.world.view, p.desc.quad)
                    && let Some((center, q)) = self.world.pose_of(t)
                {
                    let (vp, unit) = (self.env.viewport, self.env.unit.max(1e-6));
                    let q = q * rotation;
                    let (hx, hy) = (size.x / 2.0, size.y / 2.0);
                    let r = self.tree.nodes[p.node].rect;
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
                    // What its box (the rect, `reach` either side) covers on
                    // the screen; all of it while a corner is behind the eye.
                    let boxed: Vec<Option<Vec2>> = quad
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
                self.world.marks.insert(
                    key,
                    Marks {
                        enter: vec![DrawCmd::World(Box::new(WorldCmd::BeginPanel {
                            target,
                            rect: self.tree.nodes[p.node].rect,
                        }))],
                        leave: vec![DrawCmd::World(Box::new(WorldCmd::EndPanel))],
                    },
                );
            }
            if let Some((t, rotation, size)) = p.desc.quad
                && let Some((center, q)) = self.world.pose_of(t)
            {
                hits.push(PanelHit {
                    key,
                    rect: self.tree.nodes[p.node].rect,
                    center,
                    rotation: q * rotation,
                    size,
                });
            }
        }
        self.world.last_panels = hits;
        self.world.grounds = grounds;
    }

    /// The order with world anchors and meta layers drawn (and hit) under
    /// the screen's own elements: right after the root, before its children.
    pub(crate) fn world_order(&self, order: &mut Vec<usize>) {
        if order.is_empty() {
            return;
        }
        let under = |i: usize| -> bool {
            let n = &self.tree.nodes[i];
            n.float == Float::World && !self.world.is_panel_node(i)
        };
        // Subtrees of world layers below the root (each a run of the order).
        let mut moved = Vec::new();
        let mut rest = Vec::with_capacity(order.len());
        let mut k = 0;
        while k < order.len() {
            let i = order[k];
            if under(i) {
                let end = crate::layout::subtree_ends(&self.tree.nodes, &order[k..])[0] + k;
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

impl Ui {
    /// What a node's commands are wrapped in (an occlusion test, a panel's
    /// target, an edge arrow, a meta layer's paint, particles).
    pub(crate) fn world_marks(&self, key: u64) -> Option<&Marks> {
        self.world.marks.get(&key)
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
            declutter(Declutter::Shift, b, &[a], 2.0, 100.0, ANY),
            Some(Vec2::new(0.0, 6.0))
        );
        // Stack: always above.
        assert_eq!(
            declutter(Declutter::Stack, b, &[a], 2.0, 100.0, ANY),
            Some(Vec2::new(0.0, -38.0))
        );
        assert_eq!(declutter(Declutter::Hide, b, &[a], 2.0, 100.0, ANY), None);
        // Clear: no move.
        let c = Rect::new(0.0, 40.0, 100.0, 20.0);
        assert_eq!(
            declutter(Declutter::Hide, c, &[a], 2.0, 100.0, ANY),
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
                Rect::new(0.0, 0.0, 100.0, 100.0)
            ),
            Some(Vec2::new(0.0, 16.0))
        );
        assert_eq!(declutter(Declutter::Stack, b, &[a], 2.0, 100.0, top), None);
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
