use super::*;

/// What the rest rule makes of a dynamic body after the step.
enum Rest {
    /// Supported, slow before and after, and under no external force: damp
    /// residual micro-velocities without hard-resetting pose.
    Still,
    Moving,
    /// Asleep after the step: rapier left it where it was.
    Asleep,
}

/// One body read back after the step (by ranges of bodies on the lanes):
/// what the rest rule does to a dynamic one, whether it was slower than
/// og.Physics.sleepSpeed before and after the step, its component values
/// and its [`Tracked`].
type Read = (
    Option<Rest>,
    bool,
    Option<(Vec3, Quat, Option<Velocity>)>,
    Tracked,
);

/// The touching pairs, the rest rule and sleep after the step: what each
/// dynamic body's component values become. `had_force` marks body handle
/// indices that received an external force or impulse this tick (exempt
/// from rest velocity zeroing so gentle continuous forces are never glued).
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(super) fn after_step(
    settings: &Physics,
    scene: &Scene,
    sim: &mut Sim,
    handles: &Handles<'_>,
    had_force: &[bool],
    hits: &Hits<'_>,
    split: bool,
) -> (
    Vec<(Contact, CollisionStarted)>,
    Vec<Option<(Vec3, Quat, Option<Velocity>)>>,
) {
    let Sim {
        world: w,
        bodies: tracked,
        ext,
        ..
    } = sim;
    let grains = crate::lanes::now();
    // The pairs split only when the step is on the pool: below it, waking
    // the pool for them costs more than it saves.
    let pairs = if split { grains.pairs } else { usize::MAX };
    // The solid pairs: each one's contact (its entities in order, the
    // deepest point and normal, the hit), its deepest penetration and the
    // bodies it touches, by ranges of the contact graph's edges on the
    // lanes, joined in edge order: the narrow phase's pair order either way.
    let edges = w.narrow_phase.contact_graph().raw_graph().raw_edges();
    let (colliders, bodies) = (&w.colliders, &w.bodies);
    let read_pair = |pair: &rp::ContactPair| {
        let (c1, c2) = (&colliders[pair.collider1], &colliders[pair.collider2]);
        let (e1, e2) = (entity_of(c1).to_bits(), entity_of(c2).to_bits());
        let (mut point, mut normal) = (Vec3::ZERO, Vec3::ZERO);
        if let Some((manifold, contact)) = pair.find_deepest_contact() {
            point = unv(c1.position().transform_point(contact.local_p1));
            normal = unv(manifold.data.normal);
        }
        let (mut impulse, mut velocity) = hit(pair, colliders, bodies, hits.motions);
        if let Some(&(earlier, first)) = hits.carried.get(&pair_key(pair)) {
            impulse += earlier;
            velocity = first;
        }
        let (a, b, normal, velocity) = if e1 < e2 {
            (e1, e2, normal, velocity)
        } else {
            (e2, e1, -normal, -velocity)
        };
        let depth = pair
            .manifolds()
            .iter()
            .flat_map(|m| m.points.iter().map(|p| p.dist))
            .fold(f32::INFINITY, f32::min);
        let body = |c: &rp::Collider| c.parent().map(|p| p.into_raw_parts().0 as usize);
        (
            (
                Contact { a, b, sensor: 0 },
                CollisionStarted {
                    a,
                    b,
                    point,
                    normal,
                    impulse,
                    relative_velocity: velocity,
                },
            ),
            depth,
            [body(c1), body(c2)],
        )
    };
    let parts = crate::lanes::map_ranges(edges.len(), pairs, |r| {
        edges[r]
            .iter()
            .map(|e| &e.weight)
            .filter(|pair| pair.has_any_active_contact())
            .map(read_pair)
            .collect::<Vec<_>>()
    });
    // The deepest penetration of each body that has a solid contact, by
    // body handle index.
    let mut deepest: Vec<Option<f32>> = vec![None; w.bodies.len()];
    let mut touching = Vec::with_capacity(parts.iter().map(Vec::len).sum());
    for (contact, depth, parents) in parts.into_iter().flatten() {
        touching.push(contact);
        for i in parents.into_iter().flatten() {
            if i >= deepest.len() {
                deepest.resize(i + 1, None);
            }
            deepest[i] = Some(deepest[i].map_or(depth, |x| x.min(depth)));
        }
    }
    for (h1, h2, intersecting) in w.narrow_phase.intersection_pairs() {
        if !intersecting {
            continue;
        }
        let (c1, c2) = (&w.colliders[h1], &w.colliders[h2]);
        let (mut a, mut b) = (
            (entity_of(c1).to_bits(), c1.is_sensor()),
            (entity_of(c2).to_bits(), c2.is_sensor()),
        );
        if a.0 > b.0 {
            std::mem::swap(&mut a, &mut b);
        }
        let sensor = u32::from(a.1) | (u32::from(b.1) << 1);
        touching.push((
            Contact {
                a: a.0,
                b: b.0,
                sensor,
            },
            CollisionStarted::default(),
        ));
    }
    let mut touching = sorted(touching, pairs);
    // Global deduplication by entity pair (and solid vs sensor): handles
    // multi-collider entities (2D vehicles, ragdolls, chunked heightfields)
    // without emitting duplicate Contact/CollisionStarted events. Solid
    // impulses are summed and the hardest hit's point/normal/velocity kept;
    // sensor bitmasks are OR'd.
    touching.dedup_by(|later, kept| {
        let same_kind = (later.0.sensor == 0) == (kept.0.sensor == 0);
        if later.0.a == kept.0.a && later.0.b == kept.0.b && same_kind {
            if kept.0.sensor == 0 {
                let total = kept.1.impulse + later.1.impulse;
                if later.1.impulse > kept.1.impulse {
                    kept.1 = later.1;
                }
                kept.1.impulse = total;
            } else {
                kept.0.sensor |= later.0.sensor;
            }
            true
        } else {
            false
        }
    });

    // Each body read back by ranges of bodies on the lanes (rapier's
    // bodies read only), the rest rule decided there; then, in body order,
    // the resting bodies stilled in rapier and the tracked values kept.
    let (rest, calm) = (settings.rest_speed, settings.sleep_speed);
    let read: Vec<Read> = {
        let (bodies, deepest) = (&w.bodies, &deepest);
        by_bodies(scene.bodies.len(), |r| {
            scene.bodies[r.clone()]
                .iter()
                .zip(&handles.by_body[r])
                .map(|(b, &h)| {
                    let forced = had_force
                        .get(h.into_raw_parts().0 as usize)
                        .copied()
                        .unwrap_or(false);
                    read_body(b, h, bodies, deepest, (rest, calm), forced)
                })
                .collect()
        })
    };
    let calm: Vec<bool> = read.iter().map(|r| r.1).collect();
    let falling_asleep = sleep::decide(settings, scene, w, &mut ext.still, handles, &calm);
    let mut moved = Vec::with_capacity(scene.bodies.len());
    let mut kept = Vec::with_capacity(scene.bodies.len());
    for (k, (&h, (rest, _, mut out, mut t))) in handles.by_body.iter().zip(read).enumerate() {
        if falling_asleep[k] {
            // Where the step left it, as rapier keeps it asleep.
            let rb = &mut w.bodies[h];
            let (position, quaternion) = unpose(rb.position(), scene.bodies[k].position);
            rb.sleep();
            out = Some((position, quaternion, Some(Velocity::default())));
            (t.position, t.quaternion, t.velocity) =
                (position, quaternion, Some(Velocity::default()));
        } else if let Some(Rest::Still) = rest {
            // Damp residual micro-velocity without resetting pose (which would
            // wake sleeping contact partners and glue bodies against gentle forces).
            let rb = &mut w.bodies[h];
            rb.set_linvel(v(Vec3::ZERO), false);
            rb.set_angvel(angvel(Vec3::ZERO), false);
        }
        moved.push(out);
        kept.push(t);
    }
    keep_tracked(scene, tracked, kept);
    (touching, moved)
}

/// Puts each body's [`Tracked`] (`kept`, in body order) in `tracked`,
/// which holds every body still in the rapier world and lacks those built
/// this tick: one walk of both in entity order, the new ones inserted
/// after it.
fn keep_tracked(scene: &Scene, tracked: &mut IdMap<u64, Tracked>, kept: Vec<Tracked>) {
    let keys: Vec<u64> = scene.bodies.iter().map(|b| b.entity.to_bits()).collect();
    let mut order: Vec<usize> = (0..keys.len()).collect();
    if !keys.is_sorted() {
        order.sort_unstable_by_key(|&i| keys[i]);
    }
    let mut kept: Vec<Option<Tracked>> = kept.into_iter().map(Some).collect();
    let mut order = order.into_iter().peekable();
    for (key, slot) in tracked.iter_mut() {
        // Bodies before `key` in entity order are new: inserted below.
        while let Some(&i) = order.peek() {
            if keys[i] >= *key {
                break;
            }
            order.next();
        }
        if let Some(i) = order.next_if(|&i| keys[i] == *key)
            && let Some(t) = kept[i].take()
        {
            *slot = t;
        }
    }
    for (key, t) in keys.into_iter().zip(kept) {
        if let Some(t) = t {
            tracked.insert(key, t);
        }
    }
}

/// The touching pairs sorted by their contact, ties (a ragdoll's bones on
/// one floor) in their order: what a stable sort leaves, made by sorting
/// each pair's key with its index (unique, so on the lanes or not the one
/// order) and gathering the pairs in it.
fn sorted(
    touching: Vec<(Contact, CollisionStarted)>,
    grain: usize,
) -> Vec<(Contact, CollisionStarted)> {
    let mut keys: Vec<(Contact, u32)> = touching
        .iter()
        .enumerate()
        .map(|(i, t)| (t.0, i as u32))
        .collect();
    if keys.is_sorted() {
        return touching;
    }
    og_lanes::sort(og_lanes::Site::Physics, &mut keys, grain);
    keys.into_iter()
        .map(|(_, i)| touching[i as usize])
        .collect()
}

/// A body after the step: checks post-step sleep state directly (so bodies
/// woken mid-step by impacts are written back on the same tick), applies
/// the rest rule for unsupported/unforced slow bodies, and builds [`Tracked`].
fn read_body(
    b: &Body,
    h: rp::RigidBodyHandle,
    bodies: &rp::RigidBodySet,
    deepest: &[Option<f32>],
    (rest, calm_speed): (f32, f32),
    forced: bool,
) -> Read {
    let mut calm = false;
    let rb = &bodies[h];
    let (rest_rule, out) = if simulated(b) && rb.is_sleeping() {
        // Still asleep after the step: where it was, its components as they are.
        (Some(Rest::Asleep), None)
    } else if simulated(b) {
        let velocity = unlocked(
            Velocity {
                linear: unv(rb.linvel()),
                angular: unangvel(rb.angvel()),
            },
            b.body.lock,
        );
        let prior = unlocked(b.velocity.unwrap_or_default(), b.body.lock);
        let supported = deepest
            .get(h.into_raw_parts().0 as usize)
            .copied()
            .flatten()
            .is_some_and(|d| d >= -REST_PENETRATION);
        calm = slow(&prior, calm_speed) && slow(&velocity, calm_speed);
        let (position, quaternion) = unpose(rb.position(), b.position);
        if !forced && rest > 0.0 && supported && slow(&prior, rest) && slow(&velocity, rest) {
            (
                Some(Rest::Still),
                Some((position, quaternion, Some(Velocity::default()))),
            )
        } else {
            (
                Some(Rest::Moving),
                Some((position, quaternion, Some(velocity))),
            )
        }
    } else if moves_itself(b) {
        // A kinematic body moving by its velocity: where the step took it.
        let (position, quaternion) = unpose(rb.position(), b.position);
        (None, Some((position, quaternion, None)))
    } else {
        (None, None)
    };
    let (position, quaternion, velocity) = match out {
        Some((p, q, vel)) => (p, q, vel),
        None => (b.position, b.quaternion, b.velocity),
    };
    let tracked = Tracked {
        handle: h,
        body: b.body,
        collider: b.collider,
        scale: b.scale,
        position,
        quaternion,
        velocity,
    };
    (rest_rule, calm, out, tracked)
}
