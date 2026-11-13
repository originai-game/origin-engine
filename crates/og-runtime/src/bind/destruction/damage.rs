//! Damage: the tick's hits gathered, a prop broken by them, and the stress
//! solver breaking what can no longer hold itself up.

use super::*;

/// A hit that breaks pieces off drives them on: its prop, its point, the
/// striker's direction, impactRadius, the impulse past the threshold and
/// its closing speed.
pub(super) type Strike = (u64, Vec3, Vec3, f32, f32, f32);

/// The tick's hits: what each prop takes, the bursts they make, the
/// strikes and the explosions.
pub(super) struct Gathered {
    pub(super) hits: BTreeMap<u64, Hits>,
    pub(super) bursts: Vec<(u64, Vec3)>,
    pub(super) strikes: Vec<Strike>,
    pub(super) explosions: Vec<Explosion>,
}

/// The hits: impacts, then the calls in the order made.
pub(super) fn gather_hits(
    ctx: &SystemContext<'_>,
    l: &Ls,
    st: &mut State,
    assets: &Assets,
    settings: &BTreeMap<u64, Settings>,
) -> og_core::Result<Gathered> {
    let mut hits: BTreeMap<u64, Hits> = BTreeMap::new();
    let mut bursts: Vec<(u64, Vec3)> = Vec::new();
    let mut owner: BTreeMap<u64, u64> = BTreeMap::new();
    for (&k, p) in &st.props {
        if !p.placed {
            owner.insert(k, k);
        }
        for &e in p.entities.iter().filter(|&&e| e != 0) {
            owner.insert(e, k);
        }
    }
    let mut strikes: Vec<Strike> = Vec::new();
    for ev in ctx.events_bytes(l.collision.0)? {
        let v: record::CollisionStarted = row(&l.collision.1, &ev)?;
        let point = v.point;
        // The solver's impulse is that of the step that made the contact,
        // which for a fast body is a speculative one (rapier opens it
        // before they touch, and that step barely pushes): the hit is at
        // least its closing speed times the pair's reduced mass.
        let (ea, eb) = (v.a, v.b);
        let closing = v.relative_velocity.dot(v.normal).max(0.0);
        let (reduced_a, reduced_b) = (
            mass_of(ctx, l, &st.props, assets, settings, ea),
            mass_of(ctx, l, &st.props, assets, settings, eb),
        );
        let reduced = match (reduced_a, reduced_b) {
            (Mass::Finite(a), Mass::Finite(b)) => a * b / (a + b),
            (Mass::Finite(m), Mass::Infinite) | (Mass::Infinite, Mass::Finite(m)) => m,
            _ => 0.0,
        };
        let impulse = v.impulse.max(closing * reduced);
        let mut seen = None;
        for (side, of) in [("a", ea), ("b", eb)] {
            let Some(&k) = owner.get(&of) else {
                continue;
            };
            if seen == Some(k) {
                continue;
            }
            seen = Some(k);
            let s = &settings[&k];
            let amount = impulse - s.threshold;
            if amount > 0.0 && point.is_finite() {
                // The striker's motion through the prop: a's velocity
                // relative to b, turned round when the prop is a.
                // Only a prop that stood still gives the pieces it loses
                // the hit's push: a moving one's answer to the hit is
                // already in its velocity, which its pieces inherit.
                let me = if side == "a" {
                    (ea, reduced_a)
                } else {
                    (eb, reduced_b)
                };
                if matches!(me.1, Mass::Infinite) {
                    let rel = v.relative_velocity * if side == "b" { 1.0 } else { -1.0 };
                    let along = v.normal * if side == "b" { 1.0 } else { -1.0 };
                    strikes.push((k, point, rel.normalize_or(along), s.radius, amount, closing));
                }
                hits.entry(k).or_default().damage.push((
                    Damage {
                        point,
                        min_radius: s.radius / 3.0,
                        max_radius: s.radius,
                        amount,
                    },
                    CAUSE_IMPACT,
                ));
                if s.hit_fx != 0 {
                    bursts.push((s.hit_fx, point));
                }
            }
        }
    }
    let mut explosions: Vec<Explosion> = Vec::new();
    for op in std::mem::take(&mut st.ops) {
        match op {
            Op::Damage {
                entity,
                point,
                radius,
                inner,
                amount,
                impulse,
            } => {
                if let Some(s) = settings.get(&entity)
                    && st.props.contains_key(&entity)
                {
                    let point = Vec3::from(point);
                    let push = Vec3::from(impulse);
                    if push != Vec3::ZERO {
                        let dir = push.normalize();
                        strikes.push((entity, point, dir, radius, push.length(), f32::INFINITY));
                    }
                    hits.entry(entity).or_default().damage.push((
                        Damage {
                            point,
                            min_radius: inner,
                            max_radius: radius,
                            amount,
                        },
                        CAUSE_DAMAGE,
                    ));
                    if s.hit_fx != 0 {
                        bursts.push((s.hit_fx, point));
                    }
                }
            }
            Op::Explode {
                point,
                radius,
                inner,
                damage,
                impulse,
            } => {
                let point = Vec3::from(point);
                for &k in st.props.keys() {
                    hits.entry(k).or_default().damage.push((
                        Damage {
                            point,
                            min_radius: inner,
                            max_radius: radius,
                            amount: damage,
                        },
                        CAUSE_EXPLOSION,
                    ));
                }
                explosions.push((point, inner, radius, impulse));
            }
            Op::Fracture { entity } => {
                if st.props.contains_key(&entity) {
                    hits.entry(entity).or_default().fracture = true;
                }
            }
        }
    }
    Ok(Gathered {
        hits,
        bursts,
        strikes,
        explosions,
    })
}

/// What the props' breaking made this tick.
#[derive(Default)]
pub(super) struct Broken {
    /// og.Fractured, encoded, sent at the end.
    pub(super) events: Vec<Vec<u8>>,
    /// Bodies made this run and their velocity, inserted at the end (after
    /// the explosions' push).
    pub(super) made: BTreeMap<u64, (Vec3, Vec3)>,
    pub(super) fresh_bodies: Vec<u64>,
    /// Fine pieces broken off this tick, and their mass.
    pub(super) debris: Vec<(Debris, f32)>,
    /// Joints broken this tick that shed grit, and the mass of the piece
    /// beside each (which the blast's push is shared by).
    pub(super) crumbs: Vec<(Crumb, f32)>,
    /// breakFx, hitFx and debris bursts: (emitter, where).
    pub(super) bursts: Vec<(u64, Vec3)>,
}

/// One prop's tick: its hits wear its bonds, stress breaks what can no
/// longer hold, broken joints shed grit, and what broke loose becomes
/// bodies (islands) or flies apart (shattered chunks).
#[allow(clippy::too_many_arguments)]
pub(super) fn break_prop(
    ctx: &mut SystemContext<'_>,
    m: &ModuleState,
    l: &Ls,
    t: &Clock,
    at: Breaking<'_>,
    p: &mut Prop,
    h: Hits,
    stress_budget: &mut u64,
    out: &mut Broken,
) -> og_core::Result<()> {
    let (a, s, k) = (at.a, at.s, at.key);
    let frame = pose_of_frame(&p.frame);
    let scale = uniform(frame.2);
    let before = p.inst.bonds.clone();
    let anchors_before = p.inst.anchors.clone();
    let shattered_before = p.inst.shattered.clone();
    let (mut cause, mut shatter) = wear(at, &frame, p, &h, scale);
    solve_stress(
        ctx,
        m,
        l,
        t,
        at,
        &frame,
        p,
        scale,
        stress_budget,
        &mut cause,
        &mut shatter,
    );
    break_masonry(t.tick, t.gravity, at, &frame, p, &before, &mut cause);
    let broken_now: Vec<bool> = (0..before.len())
        .map(|i| before[i] > 0.0 && p.inst.bonds[i] <= 0.0)
        .collect();
    if s.debris != 0 {
        shed_crumbs(t.tick, at, &frame, &broken_now, scale, &mut out.crumbs);
    }
    let changed = broken_now.iter().any(|&b| b)
        || (0..anchors_before.len()).any(|i| anchors_before[i] > 0.0 && p.inst.anchors[i] <= 0.0)
        || p.inst.shattered != shattered_before;
    if !changed {
        return Ok(());
    }
    let cause = cause.unwrap_or(if shatter.is_empty() {
        CAUSE_STRESS
    } else {
        CAUSE_SHATTER
    });
    // A moving prop breaks free: its chunks become entities moving with
    // it, and it loses its own body.
    let mut inherit = (Vec3::ZERO, Vec3::ZERO, frame.0);
    if !p.placed {
        let e = Entity::from_bits(k);
        if let Ok(vb) = ctx.get_bytes(e, l.velocity.0) {
            let v: record::Velocity = row(&l.velocity.1, &vb)?;
            inherit = (v.linear, v.angular, frame.0);
        }
        place(ctx, l, k, p, a)?;
    }
    p.fractured = true;
    free_islands(
        ctx,
        l,
        t,
        at,
        &frame,
        p,
        scale,
        inherit,
        &broken_now,
        cause,
        out,
    )?;
    shatter_chunks(ctx, l, t, at, p, scale, &shatter, out)
}

/// The stress solver on what stands (when something changed) and, every
/// REST_EVERY ticks, on each loose island at rest on what it rests on.
#[allow(clippy::too_many_arguments)]
pub(super) fn solve_stress(
    ctx: &SystemContext<'_>,
    m: &ModuleState,
    l: &Ls,
    t: &Clock,
    at: Breaking<'_>,
    frame: &Pose,
    p: &mut Prop,
    scale: f32,
    stress_budget: &mut u64,
    cause: &mut Option<u32>,
    shatter: &mut Vec<u32>,
) {
    let (a, s, pose) = (at.a, at.s, at.pose);
    let (tick, gravity) = (t.tick, t.gravity);
    let iterations = t.res.stress_iterations;
    let anchors = p.anchors;
    if anchors && p.inst.dirty {
        let islands = islands_of(&p.inst, a, anchors);
        let down = frame.1.inverse() * gravity;
        let n = p.inst.stress(
            a,
            &islands,
            &[],
            down,
            scale,
            &s.mat,
            iterations,
            stress_budget,
        );
        if n > 0 && cause.is_none() {
            *cause = Some(CAUSE_STRESS);
        }
    } else {
        p.inst.dirty = false;
    }
    // A loose island at rest is solved on what it rests on: what spans
    // a hole between two piers bends, and breaks over it, where a
    // rigid compound would lie across them as a lintel. One at rest
    // with nothing under it is wedged by its sides (blocks jammed
    // between piers into a flat arch, which loose masonry never holds:
    // its corners crush): it crumbles, into its pieces.
    if tick.is_multiple_of(REST_EVERY)
        && let Some(q) = physics(m).and_then(|ph| ph.tick_query().ok())
    {
        let held = p.inst.dirty;
        for isl in islands_of(&p.inst, a, anchors)
            .into_iter()
            .filter(|i| !i.anchored)
        {
            let body = p.body[isl.chunks[0] as usize];
            if body == 0 || isl.chunks.iter().any(|&c| p.body[c as usize] != body) {
                continue;
            }
            let speed = ctx
                .get_bytes(Entity::from_bits(body), l.velocity.0)
                .ok()
                .and_then(|vb| row::<record::Velocity>(&l.velocity.1, &vb).ok())
                .map_or(0.0, |v| v.linear.length());
            // Moving or not, what rests on something is solved on it: a
            // long run of loose courses pivoting off a pier is a
            // cantilever, and cracks at the pivot as it swings rather
            // than turning away whole like a plank.
            if speed > REST_SPEED && isl.chunks.len() < 2 {
                continue;
            }
            let supports = supports_of(&q, a, pose, &isl.chunks, body, gravity);
            if supports.is_empty() {
                if speed > REST_SPEED {
                    continue;
                }
                let freed = isl.chunks.iter().map(|&c| p.loose[c as usize]).min();
                if freed.is_some_and(|t| tick.saturating_sub(t) >= WEDGED_AFTER) {
                    crumble(a, &mut p.inst, &isl.chunks, shatter);
                    cause.get_or_insert(CAUSE_STRESS);
                }
                continue;
            }
            if isl.chunks.len() < 2 {
                continue;
            }
            let Some(root) = pose[isl.chunks[0] as usize] else {
                continue;
            };
            let n = p.inst.stress(
                a,
                std::slice::from_ref(&isl),
                &supports,
                root.1.inverse() * gravity,
                scale,
                &s.mat,
                iterations,
                stress_budget,
            );
            if n > 0 && cause.is_none() {
                *cause = Some(CAUSE_STRESS);
            }
        }
        p.inst.dirty |= held;
    }
}
