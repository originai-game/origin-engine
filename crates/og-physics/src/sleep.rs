//! Sleep (og.Physics.sleepTicks, G2-3): a dynamic body slower than
//! og.Physics.sleepSpeed before and after each of `sleepTicks` steps in a
//! row sleeps, but only with every dynamic body it touches (contacts and
//! joints, transitively): a group sleeps whole or not at all. Speed alone
//! counts, not the rest rule's support (a body wedged in a pile deeper
//! than its 2 mm, barely moving, would keep the pile awake for ever). Rapier then leaves it out of the solver
//! (`RigidBody::sleep`, its own sleeping off with `can_sleep(false)`).
//!
//! Rapier leaves a sleeping body out of contact solving too: an awake body
//! already resting on one would sink through it, and rapier wakes a
//! sleeping body only when a contact with it starts. So the invariant here
//! is that no sleeping body touches an awake one when the solver runs:
//! before each step, and again after it (rapier may have woken some), every
//! sleeping body joined by a contact pair (touching or within rapier's
//! prediction distance) or a joint to an awake dynamic body or a moving
//! kinematic one is woken, to a fixpoint. Groups sleeping
//! whole keep that from undoing itself.
//!
//! Only dynamic bodies are ever put to sleep or woken here. Rapier puts
//! still kinematic and fixed bodies to sleep by itself, as it did before
//! G2-3 (its activation is in the og.physics block): that is left alone,
//! so with sleep off a world steps byte for byte as it did
//! (`tests/unchanged.rs`, resting bodies).
//!
//! Nothing reads a clock or depends on an iteration order: the rest
//! counters are by entity, in the block's extension section; whether a body
//! sleeps is rapier's own flag, saved with its world; the wake closure and
//! the groups are fixpoints of the contact graph, whatever order they are
//! walked in; bodies are put to sleep in body order and the events are sent
//! in entity order.

use super::*;
use crate::SleepOut;

/// The entities whose bodies sleep, ascending (the tracked bodies are kept
/// in entity order).
pub(super) fn asleep(sim: &Sim) -> Vec<u64> {
    let w = &sim.world;
    sim.bodies
        .iter()
        .filter(|(_, t)| {
            w.bodies
                .get(t.handle)
                .is_some_and(|rb| rb.is_dynamic() && rb.is_sleeping())
        })
        .map(|(k, _)| *k)
        .collect()
}

/// A dynamic body that sleeps: the only kind this module puts to sleep and
/// wakes.
fn dozing(rb: &rp::RigidBody) -> bool {
    rb.is_dynamic() && rb.is_sleeping()
}

/// A body that keeps what it touches awake: an awake dynamic body, or a
/// kinematic one moving this tick.
fn active(rb: &rp::RigidBody) -> bool {
    if rb.is_sleeping() {
        return false;
    }
    rb.is_dynamic()
        || (rb.is_kinematic() && (rb.is_moving() || rb.next_position() != rb.position()))
}

/// The bodies of every solid pair in the contact graph and of every joint.
/// Touching or not: rapier wakes every contact-graph partner of a collider
/// whose pose changed (the rest rule resets poses), so a body near enough
/// to have a pair belongs to the group and to the wake.
fn links(w: &rp::PhysicsWorld) -> Vec<(rp::RigidBodyHandle, rp::RigidBodyHandle)> {
    let mut out = Vec::new();
    for e in w.narrow_phase.contact_graph().raw_graph().raw_edges() {
        let pair = &e.weight;
        let parent = |c| w.colliders.get(c).and_then(|c: &rp::Collider| c.parent());
        if let (Some(a), Some(b)) = (parent(pair.collider1), parent(pair.collider2)) {
            out.push((a, b));
        }
    }
    for (_, j) in w.impulse_joints.iter() {
        out.push((j.body1(), j.body2()));
    }
    out
}

/// Wakes every sleeping body linked to an active one, until none is.
fn wake_touching(w: &mut rp::PhysicsWorld) {
    loop {
        let mut wake = Vec::new();
        for (a, b) in links(w) {
            let (Some(ra), Some(rb)) = (w.bodies.get(a), w.bodies.get(b)) else {
                continue;
            };
            if dozing(ra) && active(rb) {
                wake.push(a);
            }
            if dozing(rb) && active(ra) {
                wake.push(b);
            }
        }
        if wake.is_empty() {
            return;
        }
        for h in wake {
            w.wake_up(h, true);
        }
    }
}

/// Before the step, after everything game code and the passes did to the
/// bodies: with sleep off every sleeping dynamic body wakes (one slept
/// while it was on); a body whose game code wrote
/// `sleeping` false or `canSleep` false wakes; then whatever touches an
/// awake body. Returns, per scene body, whether it sleeps through the step.
pub(super) fn before_step(
    settings: &Physics,
    scene: &Scene,
    sim: &mut Sim,
    handles: &Handles<'_>,
) -> Vec<bool> {
    let w = &mut sim.world;
    if !w.bodies.iter().any(|(_, rb)| dozing(rb)) {
        return Vec::new();
    }
    if settings.sleep_ticks == 0 {
        let all: Vec<_> = w
            .bodies
            .iter()
            .filter(|(_, rb)| dozing(rb))
            .map(|(h, _)| h)
            .collect();
        for h in all {
            w.wake_up(h, true);
        }
        return Vec::new();
    }
    for (b, &h) in scene.bodies.iter().zip(&handles.by_body) {
        if dozing(&w.bodies[h]) && (!b.body.sleeping || !b.body.can_sleep || !simulated(b)) {
            w.wake_up(h, true);
        }
    }
    wake_touching(w);
    handles
        .by_body
        .iter()
        .map(|&h| dozing(&w.bodies[h]))
        .collect()
}

/// After the step, before the rest rule touches rapier (`calm`, per scene
/// body: slower than sleepSpeed before and after the step): wakes what touches an
/// awake body, counts each body's ticks at rest, and returns, per scene
/// body, whether it falls asleep now: its whole group (dynamic bodies
/// joined by contacts and joints) has rested `sleepTicks` or sleeps
/// already, and none of it touches a moving kinematic body.
///
/// A body falling asleep keeps the pose the step left it at (moved by
/// less than restSpeed × dt): the rest rule's reset of its pose would be a
/// change rapier answers at the next step by waking it and everything it
/// touches.
pub(super) fn decide(
    settings: &Physics,
    scene: &Scene,
    w: &mut rp::PhysicsWorld,
    counts: &mut IdMap<u64, u32>,
    handles: &Handles<'_>,
    calm: &[bool],
) -> Vec<bool> {
    let ticks = settings.sleep_ticks;
    let mut ready = vec![false; scene.bodies.len()];
    if ticks == 0 {
        counts.clear();
        return ready;
    }
    if w.bodies.iter().any(|(_, rb)| dozing(rb)) {
        wake_touching(w);
    }

    // The ticks each awake body has rested, by entity.
    let mut next: Vec<(u64, u32)> = Vec::new();
    for (k, (b, &h)) in scene.bodies.iter().zip(&handles.by_body).enumerate() {
        let key = b.entity.to_bits();
        if simulated(b) && b.body.can_sleep && calm[k] && !dozing(&w.bodies[h]) {
            let n = counts.get(&key).copied().unwrap_or(0).saturating_add(1);
            ready[k] = n >= ticks;
            next.push((key, n));
        }
    }
    next.sort_unstable_by_key(|e| e.0);
    *counts = next.into_iter().collect();
    if !ready.contains(&true) {
        return ready;
    }

    // The groups, by union-find over the links between dynamic bodies.
    let index = |h: rp::RigidBodyHandle| h.into_raw_parts().0 as usize;
    let n = w
        .bodies
        .iter()
        .map(|(h, _)| index(h) + 1)
        .max()
        .unwrap_or(0);
    let mut ok = vec![false; n];
    for (h, rb) in w.bodies.iter() {
        ok[index(h)] = dozing(rb);
    }
    for (k, &h) in handles.by_body.iter().enumerate() {
        ok[index(h)] |= ready[k];
    }
    let mut root: Vec<usize> = (0..n).collect();
    fn find(root: &mut [usize], mut i: usize) -> usize {
        while root[i] != i {
            root[i] = root[root[i]];
            i = root[i];
        }
        i
    }
    let mut spoiled = Vec::new();
    for (a, b) in links(w) {
        let (Some(ra), Some(rb)) = (w.bodies.get(a), w.bodies.get(b)) else {
            continue;
        };
        match (ra.is_dynamic(), rb.is_dynamic()) {
            (true, true) => {
                let (x, y) = (find(&mut root, index(a)), find(&mut root, index(b)));
                root[x.max(y)] = x.min(y);
            }
            (true, false) if active(rb) => spoiled.push(index(a)),
            (false, true) if active(ra) => spoiled.push(index(b)),
            _ => {}
        }
    }
    let mut group_ok = vec![true; n];
    for (i, &ok) in ok.iter().enumerate() {
        let r = find(&mut root, i);
        group_ok[r] &= ok;
    }
    for i in spoiled {
        let r = find(&mut root, i);
        group_ok[r] = false;
    }
    for (k, (b, &h)) in scene.bodies.iter().zip(&handles.by_body).enumerate() {
        ready[k] &= group_ok[find(&mut root, index(h))];
        if ready[k] {
            counts.remove(&b.entity.to_bits());
        }
    }
    ready
}

/// What sleep changed this tick, against `before` (the entities asleep at
/// its start): the bodies that fell asleep and woke, in entity order, and
/// the og.RigidBody values whose `sleeping` is no longer what the body is.
pub(super) fn report(
    scene: &Scene,
    w: &rp::PhysicsWorld,
    handles: &Handles<'_>,
    before: &[u64],
) -> SleepOut {
    let mut out = SleepOut::default();
    for (b, &h) in scene.bodies.iter().zip(&handles.by_body) {
        let key = b.entity.to_bits();
        // Only a simulated body sleeps (rapier may call a still kinematic
        // one sleeping; a controller's made-up og.RigidBody is not there).
        let now = simulated(b) && dozing(&w.bodies[h]);
        let was = before.binary_search(&key).is_ok();
        if now && !was {
            out.slept.push(key);
        } else if was && !now {
            out.woke.push(key);
        }
        if b.body.sleeping != now {
            out.writes.push((
                b.entity,
                RigidBody {
                    sleeping: now,
                    ..b.body
                },
            ));
        }
    }
    out.slept.sort_unstable();
    out.woke.sort_unstable();
    out
}
