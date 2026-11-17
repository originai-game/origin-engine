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

/// The top left and bottom right, in layout px, of what node `i` draws:
/// its box and every shown box under it, out-of-flow ones included, each
/// mapped by its own transform and those between it and `i` (a targeted
/// nameplate's frame scaled up), as it is drawn.
fn drawn_box(nodes: &[crate::layout::Node], i: usize) -> (Vec2, Vec2) {
    let r = nodes[i].placed.rect;
    let (mut lo, mut hi) = (Vec2::new(r.x, r.y), Vec2::new(r.right(), r.bottom()));
    let kids = |n: &crate::layout::Node, m: Affine| {
        n.children
            .iter()
            .chain(&n.out)
            .map(|&j| (j, m))
            .collect::<Vec<_>>()
    };
    let mut stack = kids(&nodes[i], Affine::IDENTITY);
    while let Some((j, m)) = stack.pop() {
        let b = nodes[j].placed.rect;
        let t = nodes[j].style.sampled.transform;
        let m = if t.is_identity() {
            m
        } else {
            m.mul(&t.matrix(&b))
        };
        if !b.is_empty() {
            let b = m.map_rect(&b);
            lo = lo.min(Vec2::new(b.x, b.y));
            hi = hi.max(Vec2::new(b.right(), b.bottom()));
        }
        stack.extend(kids(&nodes[j], m));
    }
    (lo, hi)
}

/// The shown nodes under ground `i` that show text (a badge, a label), the
/// outermost of each branch with what is under it: the ground's labels
/// rather than its marks on the ground.
fn ground_labels(nodes: &[crate::layout::Node], i: usize) -> Vec<usize> {
    let mut out = Vec::new();
    let mut stack: Vec<usize> = nodes[i]
        .children
        .iter()
        .chain(&nodes[i].out)
        .copied()
        .collect();
    while let Some(j) = stack.pop() {
        let n = &nodes[j];
        if n.hidden {
            continue;
        }
        if matches!(
            &*n.content,
            crate::layout::Content::Text(_) | crate::layout::Content::Rich(_)
        ) {
            out.push(j);
        } else {
            stack.extend(n.children.iter().chain(&n.out).copied());
        }
    }
    out
}

/// What the grounds' text covers on the screen, og-ui px: a path's badge,
/// a placement's label. A ground lies pinned on the scene, so its text
/// is in place before any tag is decluttered, and the tags keep clear of
/// it (a prompt drawn over "Out of reach" hid it).
fn ground_text(
    world: &crate::world::WorldUi,
    nodes: &[crate::layout::Node],
    vp: Vec2,
) -> Vec<Rect> {
    let Some(view) = world.declared.view else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for p in world.declared.panels.iter().filter(|p| p.desc.projected) {
        let Some((t, rotation, size)) = p.desc.quad else {
            continue;
        };
        let Some((center, q)) = world.pose_of(t) else {
            continue;
        };
        let q = q * rotation;
        let r = nodes[p.node].placed.rect;
        // A layout point's screen point, through the plane (as drawn).
        let seen = |x: f32, y: f32| {
            let on = Vec3::new(
                (x - r.x) / r.width.max(1e-6) * size.x - size.x / 2.0,
                size.y / 2.0 - (y - r.y) / r.height.max(1e-6) * size.y,
                0.0,
            );
            let (sx, sy, _) = view.project(center + q * on, vp.x, vp.y)?;
            Some(Vec2::new(sx, sy))
        };
        let mut stack: Vec<usize> = nodes[p.node]
            .children
            .iter()
            .chain(&nodes[p.node].out)
            .copied()
            .collect();
        while let Some(j) = stack.pop() {
            let n = &nodes[j];
            stack.extend(n.children.iter().chain(&n.out).copied());
            let b = n.placed.rect;
            if n.probe_text.trim().is_empty() || b.is_empty() {
                continue;
            }
            let corners = [
                (b.x, b.y),
                (b.right(), b.y),
                (b.right(), b.bottom()),
                (b.x, b.bottom()),
            ]
            .map(|(x, y)| seen(x, y));
            if corners.iter().any(Option::is_none) {
                continue;
            }
            let c = corners.map(Option::unwrap);
            let (lo, hi) = c[1..]
                .iter()
                .fold((c[0], c[0]), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
            out.push(Rect::new(lo.x, lo.y, hi.x - lo.x, hi.y - lo.y));
        }
    }
    out
}

/// The spot nearest `b`'s own that is clear of `placed`, as the move
/// there, or `None`: nothing fits and it is hidden. As map labels are
/// placed (MapLibre's collision index): by rank, each in the first of its
/// candidate spots clear of those placed before it and inside the screen.
/// A spot is clear only against a side of a box near it, so the
/// candidates are `b` a `gap` off the sides of the boxes within `reach`,
/// on either axis or both. `Stack` first takes the nearest straight above,
/// then straight below (nameplates over a crowd); `Shift` and a stack with
/// no room either way take the nearest of all. A moved box stays inside
/// `inside` (the screen) and within `reach` of its spot.
fn declutter(
    mode: Declutter,
    b: Rect,
    placed: &[Rect],
    gap: f32,
    reach: f32,
    inside: Rect,
) -> Option<Vec2> {
    // Clear: a gap from every placed box (touching is not clear).
    let clear = |r: &Rect| {
        let r = r.outset(gap - 0.01);
        !placed.iter().any(|p| p.overlaps(&r))
    };
    if clear(&b) {
        return Some(Vec2::ZERO);
    }
    if !matches!(mode, Declutter::Shift | Declutter::Stack) {
        return None;
    }
    let moved = |d: Vec2| Rect::new(b.x + d.x, b.y + d.y, b.width, b.height);
    let (mut xs, mut ys) = (vec![0.0], vec![0.0]);
    let nearby = b.outset(reach);
    for p in placed.iter().filter(|p| p.overlaps(&nearby)) {
        xs.extend([p.x - gap - b.right(), p.right() + gap - b.x]);
        ys.extend([p.y - gap - b.bottom(), p.bottom() + gap - b.y]);
    }
    // Keep the same coordinates, but sweep their free y intervals instead
    // of materialising and sorting the Cartesian product. Each obstacle
    // excludes one contiguous interval of sorted candidate ordinates.
    xs.retain(|&x| {
        Vec2::new(x, 0.0).length() <= reach
            && b.x + x >= inside.x - 0.5
            && b.x + x + b.width <= inside.right() + 0.5
    });
    ys.retain(|&y| {
        Vec2::new(0.0, y).length() <= reach
            && b.y + y >= inside.y - 0.5
            && b.y + y + b.height <= inside.bottom() + 0.5
    });
    // Coordinates are deduplicated, and equal-start intervals commute:
    // these sorts have no meaningful tie order to preserve.
    ys.sort_unstable_by(f32::total_cmp);
    ys.dedup();
    if xs.is_empty() || ys.is_empty() {
        return None;
    }
    let zero = ys.partition_point(|&y| y < 0.0);
    if mode == Declutter::Stack && xs.contains(&0.0) {
        // Stacks usually have room on their own vertical. Test it before
        // building the two-dimensional collision index. A collision rules
        // out a whole run of ordinates, not just the one tested.
        let free = |at: usize| clear(&moved(Vec2::new(0.0, ys[at])));
        let mut end = zero;
        while end > 0 {
            let at = end - 1;
            let r = moved(Vec2::new(0.0, ys[at])).outset(gap - 0.01);
            if let Some(p) = placed.iter().find(|p| p.overlaps(&r)) {
                end = ys[..at].partition_point(|&y| {
                    moved(Vec2::new(0.0, y)).outset(gap - 0.01).bottom() <= p.y
                });
            } else {
                let length = Vec2::new(0.0, ys[at]).length();
                let lo = ys[..=at].partition_point(|&y| Vec2::new(0.0, y).length() > length);
                let at = (lo..=at).find(|&at| free(at)).unwrap();
                return Some(Vec2::new(0.0, ys[at]));
            }
        }
        let mut at = ys.partition_point(|&y| y <= 0.0);
        while at < ys.len() {
            let r = moved(Vec2::new(0.0, ys[at])).outset(gap - 0.01);
            if let Some(p) = placed.iter().find(|p| p.overlaps(&r)) {
                at += 1 + ys[at + 1..].partition_point(|&y| {
                    moved(Vec2::new(0.0, y)).outset(gap - 0.01).y < p.bottom()
                });
            } else {
                return Some(Vec2::new(0.0, ys[at]));
            }
        }
    }
    xs.sort_unstable_by(f32::total_cmp);
    xs.dedup();
    let leftmost = moved(Vec2::new(xs[0], 0.0)).outset(gap - 0.01).x;
    let rightmost = moved(Vec2::new(xs[xs.len() - 1], 0.0))
        .outset(gap - 0.01)
        .right();
    let edges: Vec<_> = ys
        .iter()
        .map(|&y| {
            let r = moved(Vec2::new(0.0, y)).outset(gap - 0.01);
            (r.y, r.bottom())
        })
        .collect();
    let topmost = edges[0].0;
    let bottommost = edges[edges.len() - 1].1;
    let mut blocked = Vec::new();
    for p in placed {
        if p.x >= rightmost || p.right() <= leftmost || p.y >= bottommost || p.bottom() <= topmost {
            continue;
        }
        let top = edges.partition_point(|&(_, bottom)| bottom <= p.y);
        let bottom = edges.partition_point(|&(top, _)| top < p.bottom());
        if top < bottom {
            blocked.push((p.x, p.right(), top, bottom));
        }
    }
    blocked.sort_unstable_by_key(|(_, _, top, _)| *top);
    xs.sort_unstable_by(|a, c| a.abs().total_cmp(&c.abs()).then(a.total_cmp(c)));
    let order = |d: Vec2| (d.length(), d.x.abs(), d.y, d.x);
    let mut best: Option<Vec2> = None;
    for x in xs {
        // No y can make this or a farther x nearer than the best spot.
        if best.is_some_and(|d| Vec2::new(x, 0.0).length() > d.length()) {
            break;
        }
        let mut free = |lo: usize, hi: usize| {
            if lo >= hi {
                return;
            }
            for at in [
                zero.clamp(lo, hi - 1),
                zero.saturating_sub(1).clamp(lo, hi - 1),
            ] {
                let length = Vec2::new(x, ys[at]).length();
                // Preserve the old up-first tie when lengths round equal.
                let at = if ys[at] < 0.0 {
                    lo + ys[lo..=at].partition_point(|&y| Vec2::new(x, y).length() > length)
                } else {
                    at
                };
                let d = Vec2::new(x, ys[at]);
                if d.length() <= reach && best.is_none_or(|old| order(d) < order(old)) {
                    best = Some(d);
                }
            }
        };
        let r = moved(Vec2::new(x, 0.0)).outset(gap - 0.01);
        let (left, right) = (r.x, r.right());
        let mut start = 0;
        for &(obstacle_left, obstacle_right, lo, hi) in &blocked {
            if hi <= start || obstacle_left >= right || obstacle_right <= left {
                continue;
            }
            free(start, lo);
            start = start.max(hi);
            if start >= ys.len() {
                break;
            }
        }
        free(start, ys.len());
    }
    best
}

/// The most numbers a template lays apart on one tick; the rest stand on
/// their points.
const SPAWNED_PLACED: usize = 64;

impl crate::world::WorldUi {
    /// Lays the numbers and texts spawned in the world this tick (damage
    /// numbers) apart, as games stack a target's hits: each clear of the
    /// tags placed (`taken`, the screen boxes `world_anchors` returns), of
    /// those still rising and of those spawned before it (each over the
    /// path it rises along while it shows), at the nearest
    /// spot a stack takes (above, then below, then any way), moved once
    /// by its offset (`aux`) for its whole life. One with no room in reach
    /// is dropped rather than drawn over another, where neither reads.
    pub(super) fn place_spawned(&mut self, env: &crate::ui::Env, tick: u32, mut taken: Vec<Rect>) {
        use og_draw::UiMassKind;
        let Some(view) = self.declared.view else {
            return;
        };
        let vp = env.viewport;
        let unit = env.unit.max(1e-6);
        let safe = Rect::new(
            vp.safe_area.left,
            vp.safe_area.top,
            vp.width - vp.safe_area.horizontal(),
            vp.height - vp.safe_area.vertical(),
        );
        let gap = env.theme.space.xs;
        let metrics = &env.fonts.body.metrics;
        for (k, (_, t, space)) in self.declared.templates.iter().enumerate() {
            let words = matches!(t.kind, UiMassKind::Number { .. } | UiMassKind::Text { .. });
            if *space != UiMassSpace::World || t.lifetime == 0 || !words {
                continue;
            }
            let Some(pool) = self.state.pools.get_mut(&(k as u32)) else {
                continue;
            };
            // Its box on the screen at `age` (a fraction of its life), as
            // the shader draws it at rest size: projected, offset by its
            // key and its own move, the pivot on the point.
            let boxed = |i: &UiMassInstance, age: f32| -> Option<Rect> {
                let text = match &t.kind {
                    UiMassKind::Number {
                        decimals,
                        plus,
                        compact,
                    } => og_draw::format_number(i.value, *decimals, *plus, *compact),
                    UiMassKind::Text { strings } => strings.get(i.value.max(0.0) as usize)?.clone(),
                    _ => return None,
                };
                let (x, y, depth) = view.project(Vec3::from(i.position), vp.width, vp.height)?;
                let d = if t.distance.x > 0.0 {
                    (t.distance.x / depth.max(1e-4)).clamp(t.distance.y, t.distance.z)
                } else {
                    1.0
                };
                let (off, scale, _) = t.sample(age);
                let em = t.size.x * i.scale * scale.min(1.0) * d;
                let w = metrics.measure(&text, em);
                let at = Vec2::new(x, y) + (off + Vec2::new(i.aux[0], i.aux[1]) / unit) * d;
                Some(Rect::new(
                    at.x - t.pivot.x * w,
                    at.y - t.pivot.y * em,
                    w,
                    em,
                ))
            };
            // The path it rises along from `age` while it still shows
            // (at its keys and each eighth of its life, while half visible or
            // more): what it
            // covers until it fades, not only where it starts.
            let stops: Vec<f32> = t
                .keys_drawn()
                .iter()
                .map(|k| k.t)
                .chain((1..=8).map(|k| k as f32 / 8.0))
                .collect();
            let swept = |i: &UiMassInstance, age: f32| -> Option<Rect> {
                let mut r = boxed(i, age)?;
                for &at in stops
                    .iter()
                    .filter(|&&at| at > age && t.sample(at).2 >= 0.5)
                {
                    if let Some(b) = boxed(i, at) {
                        let (x, y) = (r.x.min(b.x), r.y.min(b.y));
                        r = Rect::new(
                            x,
                            y,
                            r.right().max(b.right()) - x,
                            r.bottom().max(b.bottom()) - y,
                        );
                    }
                }
                Some(r)
            };
            // Those of earlier ticks over the rest of their rise.
            for i in pool.iter().filter(|i| i.start != tick && i.color[3] > 0.0) {
                let age = tick.wrapping_sub(i.start) as f32 / t.lifetime as f32;
                taken.extend(swept(i, age));
            }
            for i in pool
                .iter_mut()
                .filter(|i| i.start == tick)
                .take(SPAWNED_PLACED)
            {
                let Some(b) = swept(i, 0.0) else {
                    continue;
                };
                let reach = 3.0 * b.height.max(b.width.min(160.0));
                let (x, y) = (safe.x.min(b.x), safe.y.min(b.y));
                let inside = Rect::new(
                    x,
                    y,
                    safe.right().max(b.right()) - x,
                    safe.bottom().max(b.bottom()) - y,
                );
                match declutter(Declutter::Stack, b, &taken, gap, reach, inside) {
                    Some(d) => {
                        i.aux[0] += d.x * unit;
                        i.aux[1] += d.y * unit;
                        taken.push(Rect::new(b.x + d.x, b.y + d.y, b.width, b.height));
                    }
                    None => i.color[3] = 0.0,
                }
            }
        }
    }
}

impl Ui {
    pub(super) fn world_metas(&mut self) {
        let metas = std::mem::take(&mut self.world.declared.metas);
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
            self.world.placement.marks.insert(
                key,
                Marks {
                    enter: cmds,
                    leave: Vec::new(),
                },
            );
        }
        self.world.declared.metas = metas;
    }
}

impl crate::ui::Layout {
    pub(super) fn world_anchors(
        &mut self,
        world: &mut crate::world::WorldUi,
        env: &crate::ui::Env,
        tree: &mut crate::ui::Tree,
    ) -> Vec<Rect> {
        let vp = env.viewport;
        let screen = Rect::new(0.0, 0.0, vp.width, vp.height);
        let safe = Rect::new(
            vp.safe_area.left,
            vp.safe_area.top,
            vp.width - vp.safe_area.horizontal(),
            vp.height - vp.safe_area.vertical(),
        );
        let anchors = std::mem::take(&mut world.declared.anchors);
        world.placement.placed.clear();
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
            /// What it draws on the screen (`drawn_box`), which declutters.
            drawn: Rect,
            arrow: Option<(f32, Srgba, f32)>,
        }
        let mut placed: Vec<Placed> = Vec::new();
        let view = world.declared.view;
        for (e, a) in anchors.iter().enumerate() {
            let n = &tree.nodes[a.node];
            let key = n.key;
            let r = n.placed.rect;
            let d = &a.desc;
            let mut info = AnchorInfo {
                scale: 1.0,
                ..AnchorInfo::default()
            };
            let (Some(view), Some((p, _))) = (view, world.pose_of(d.target)) else {
                placed.push(Placed {
                    entry: e,
                    key,
                    at: Vec2::ZERO,
                    pivot: d.pivot.fractions(),
                    scale: 1.0,
                    alpha: 0.0,
                    depth: 0.0,
                    info,
                    drawn: Rect::default(),
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
                let g = env.theme.space.sm;
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
            let (lo, hi) = drawn_box(&tree.nodes, a.node);
            let (px, py) = (r.x + pivot.0 * r.width, r.y + pivot.1 * r.height);
            let drawn = Rect::new(
                at.x + (lo.x - px) * scale,
                at.y + (lo.y - py) * scale,
                (hi.x - lo.x) * scale,
                (hi.y - lo.y) * scale,
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
                drawn,
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
        let gap = env.theme.space.xs;
        // The grounds' text first: pinned, it never moves for a tag.
        let mut taken = ground_text(world, &tree.nodes, Vec2::new(vp.width, vp.height));
        let mut moves = vec![(Vec2::ZERO, 1.0f32); placed.len()];
        for &k in &by_rank {
            let pl = &placed[k];
            if pl.alpha <= 0.0 || pl.drawn.is_empty() {
                continue;
            }
            let mode = anchors[pl.entry].desc.declutter;
            if mode == Declutter::None {
                taken.push(pl.drawn);
                continue;
            }
            let reach = 3.0 * pl.drawn.height.max(pl.drawn.width.min(160.0));
            let a = &anchors[pl.entry].desc;
            // A move never takes it off the safe area, nor further off
            // than its own spot is (a tag at the edge stays where it is).
            let b = pl.drawn;
            let inside = if a.contain && a.clamp.is_none() {
                safe
            } else {
                let (x, y) = (safe.x.min(b.x), safe.y.min(b.y));
                Rect::new(
                    x,
                    y,
                    safe.right().max(b.right()) - x,
                    safe.bottom().max(b.bottom()) - y,
                )
            };
            match declutter(mode, b, &taken, gap, reach, inside) {
                Some(d) => {
                    let r = pl.drawn;
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
            let fresh = !world.state.smooth.contains_key(&pl.key);
            let s = world.state.smooth.entry(pl.key).or_default();
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
            let r = tree.nodes[node].placed.rect;
            let px = r.x + pl.pivot.0 * r.width;
            let py = r.y + pl.pivot.1 * r.height;
            let at = pl.at + Vec2::new(s.dx, s.dy);
            let n = &mut tree.nodes[node];
            n.style.sampled.transform = Transform {
                x: at.x - px,
                y: at.y - py,
                scale_x: pl.scale,
                scale_y: pl.scale,
                pivot_x: pl.pivot.0,
                pivot_y: pl.pivot.1,
                ..n.style.sampled.transform
            };
            n.style.sampled.opacity *= s.alpha;
            if s.alpha < 0.05 {
                n.style.sampled.inert = true;
            }
            let mut info = pl.info;
            info.x = at.x;
            info.y = at.y;
            info.alpha = s.alpha;
            world.placement.last.insert(pl.key, info);
            // Bracketed by its key, so a frame drawn between ticks moves it
            // with its point (`UiFrame::emit_shifted`).
            let mut marks = Marks::default();
            marks
                .enter
                .push(DrawCmd::World(Box::new(WorldCmd::Anchor { key: pl.key })));
            world.placement.placed.push(super::AnchorPoint {
                key: pl.key,
                target: d.target,
                offset: d.offset,
                clamped: pl.info.clamped,
            });
            if let Some(view) = world.declared.view
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
            world.placement.marks.insert(pl.key, marks);
        }
        world.state.smooth.retain(|k, _| seen.contains(k));
        world.placement.last.retain(|k, _| seen.contains(k));
        world.declared.anchors = anchors;
        taken
    }

    pub(super) fn world_panels(
        &mut self,
        ui_world: &mut crate::world::WorldUi,
        env: &crate::ui::Env,
        tree: &mut crate::ui::Tree,
    ) {
        let mut hits = Vec::new();
        let mut grounds = Vec::new();
        for p in &ui_world.declared.panels {
            let key = tree.nodes[p.node].key;
            if p.desc.projected {
                // Drawn by the UI onto where its quad is seen this tick: the
                // brackets are the same every tick (the quad goes in the
                // frame, `UiFrame::grounds`), so a moving camera relevels
                // and rebuilds nothing. Inert: no ray, no focus.
                tree.nodes[p.node].style.sampled.inert = true;
                let all = Rect::new(-1e9, -1e9, 2e9, 2e9);
                let begin = || DrawCmd::BeginLayer {
                    material: og_draw::UiMaterial::Plain,
                    params: [[0.0; 4]; og_draw::UI_MATERIAL_PARAMS],
                    mask: None,
                    expand: 0.0,
                    rect: Rect::default(),
                    filter: None,
                    role: Default::default(),
                };
                let end = |rect: Rect, ground| DrawCmd::EndLayer {
                    geometry: None,
                    rect,
                    opacity: 1.0,
                    blend: og_draw::UiBlend::Normal,
                    clip: all,
                    ground: Some(ground),
                };
                ui_world.placement.marks.insert(
                    key,
                    Marks {
                        enter: vec![begin()],
                        leave: vec![end(
                            tree.nodes[p.node].placed.rect,
                            crate::draw::Ground::Panel(key),
                        )],
                    },
                );
                // Its texts and badges: a layer each, laid with it over
                // every ground's other drawing (`Ground::Label`).
                for j in ground_labels(&tree.nodes, p.node) {
                    let m = ui_world
                        .placement
                        .marks
                        .entry(tree.nodes[j].key)
                        .or_default();
                    m.enter.insert(0, begin());
                    m.leave.push(end(
                        tree.nodes[j].placed.rect,
                        crate::draw::Ground::Label(key),
                    ));
                }
                if let (Some(view), Some((t, rotation, size))) =
                    (ui_world.declared.view, p.desc.quad)
                    && let Some((center, q)) = ui_world.pose_of(t)
                {
                    let (vp, unit) = (env.viewport, env.unit.max(1e-6));
                    let q = q * rotation;
                    let (hx, hy) = (size.x / 2.0, size.y / 2.0);
                    let r = tree.nodes[p.node].placed.rect;
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
                    // What it draws (its box and what overflows it, such as
                    // a badge past a path's end), on its plane.
                    let (lo, hi) = drawn_box(&tree.nodes, p.node);
                    let on_plane = |x: f32, y: f32| {
                        (
                            (x - r.x) / r.width.max(1e-6) * size.x - hx,
                            hy - (y - r.y) / r.height.max(1e-6) * size.y,
                        )
                    };
                    let drawn = [(lo.x, lo.y), (hi.x, lo.y), (hi.x, hi.y), (lo.x, hi.y)]
                        .map(|(x, y)| on_plane(x, y));
                    // What that covers on the screen, `reach` either side
                    // of the plane; all of it while a corner is behind the eye.
                    let boxed: Vec<Option<Vec2>> = drawn
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
                ui_world.placement.marks.insert(
                    key,
                    Marks {
                        enter: vec![DrawCmd::World(Box::new(WorldCmd::BeginPanel {
                            target,
                            rect: tree.nodes[p.node].placed.rect,
                        }))],
                        leave: vec![DrawCmd::World(Box::new(WorldCmd::EndPanel))],
                    },
                );
            }
            if let Some((t, rotation, size)) = p.desc.quad
                && let Some((center, q)) = ui_world.pose_of(t)
            {
                hits.push(PanelHit {
                    key,
                    rect: tree.nodes[p.node].placed.rect,
                    center,
                    rotation: q * rotation,
                    size,
                });
            }
        }
        ui_world.placement.last_panels = hits;
        ui_world.placement.grounds = grounds;
    }

    /// The order with world anchors and meta layers drawn (and hit) under
    /// the screen's own elements: right after the root, before its children.
    pub(crate) fn world_order(
        &self,
        order: &mut Vec<usize>,
        world: &crate::world::WorldUi,
        tree: &crate::ui::Tree,
    ) {
        if order.is_empty() {
            return;
        }
        let under = |i: usize| -> bool {
            let n = &tree.nodes[i];
            n.float == Float::World && !world.is_panel_node(i)
        };
        // Subtrees of world layers below the root (each a run of the order).
        let mut moved = Vec::new();
        let mut rest = Vec::with_capacity(order.len());
        let mut k = 0;
        while k < order.len() {
            let i = order[k];
            if under(i) {
                let end = crate::layout::subtree_ends(&tree.nodes, &order[k..])[0] + k;
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

impl crate::world::WorldUi {
    /// What a node's commands are wrapped in (an occlusion test, a panel's
    /// target, an edge arrow, a meta layer's paint, particles).
    pub(crate) fn world_marks(&self, key: u64) -> Option<&Marks> {
        self.placement.marks.get(&key)
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
        // Touching a's bottom is not clear: it keeps a gap (a bubble's
        // tail on a cast bar).
        let t = Rect::new(0.0, 20.0, 100.0, 20.0);
        assert_eq!(
            declutter(Declutter::Shift, t, &[a], 2.0, 100.0, ANY),
            Some(Vec2::new(0.0, 2.0))
        );
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
        // With no room above or below but room beside, it goes beside.
        let wide = Rect::new(0.0, 0.0, 300.0, 40.0);
        assert_eq!(
            declutter(Declutter::Stack, b, &[a], 2.0, 120.0, wide),
            Some(Vec2::new(102.0, 0.0))
        );
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
