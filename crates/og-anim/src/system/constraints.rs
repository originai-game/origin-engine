//! Foot locks, joint limits, spring bones and IK evaluation.

use super::*;

/// One foot lock this tick: whether the clips plant the foot, the lock's
/// next state from `prev`, and the leg bent to hold the foot there.
pub(super) fn foot_lock(
    sh: &Shared,
    w: &mut Work,
    fl: &mut FootLock,
    prev: Lock,
    dt: f32,
    limits: &Limits,
) -> Result<Lock> {
    let foot = joint(w, fl.foot, "og.FootLock.foot")?;
    let Some(leg) = ik::chain(&w.asset, foot, 2) else {
        return Err(invalid(format!(
            "og.FootLock: joint {} has fewer than 2 bones above it",
            fl.foot
        )));
    };
    // Planted when the clips planting it hold at least half the stance.
    let (mut on, mut total) = (0.0f32, 0.0f32);
    for &(clip, t, weight) in &w.stance {
        if weight <= 0.0 {
            continue;
        }
        let Some(p) = sh.plants(w.skeleton.asset, clip, foot) else {
            continue;
        };
        let Some(c) = sh.clips.get(&clip.0) else {
            continue;
        };
        total += weight;
        if p.planted_at(c, t) {
            on += weight;
        }
    }
    let planted = total > 0.0 && on >= 0.5 * total;
    let frame = w.transform.map_or(Trs::IDENTITY, |t| {
        Trs::new(t.position, t.quaternion, t.scale)
    });
    let model = w.asset.model_space(&w.pose.nodes);
    let animated = frame.transform_point(model[foot as usize].translation);
    let release = fl.release.max(0.0);
    let lifted = |at: Vec3| {
        if release > 0.0 {
            Lock::Releasing(at, 0.0)
        } else {
            Lock::Free
        }
    };
    let lock = match prev {
        Lock::Free if planted => Lock::Held(animated),
        Lock::Free => Lock::Free,
        Lock::Held(at) if !planted || (at - animated).length() > fl.max_distance => lifted(at),
        Lock::Held(at) => Lock::Held(at),
        Lock::Releasing(from, t) => {
            let t = t + dt;
            if t >= release {
                Lock::Free
            } else {
                Lock::Releasing(from, t)
            }
        }
    };
    let target = match lock {
        Lock::Free => None,
        Lock::Held(at) => Some(at),
        Lock::Releasing(from, t) => Some(from + (animated - from) * (t / release)),
    };
    if let Some(target) = target {
        let to = frame.inverse().transform_point(target);
        let turned = model[foot as usize].rotation;
        let lim = chain_limits(w, &leg, limits);
        ik::two_bone(
            &w.asset,
            &mut w.pose.nodes,
            &leg,
            to,
            None,
            fl.weight.clamp(0.0, 1.0),
            &lim,
        );
        // The foot keeps its animated orientation.
        let parent = w.asset.nodes[foot as usize].parent as usize;
        let m = w.asset.model_space(&w.pose.nodes);
        w.pose.nodes[foot as usize].rotation = unit(m[parent].rotation.conjugate() * turned);
    }
    fl.planted = planted;
    fl.locked = matches!(lock, Lock::Held(_));
    Ok(lock)
}

/// Joint limits by (skeleton entity, node).
pub(super) type Limits = BTreeMap<(u64, u32), JointLimit>;

/// The node of joint `joint` of `w`'s skin, or an error naming `field`.
pub(super) fn joint(w: &Work, joint: u32, field: &str) -> Result<u32> {
    let skin = w.skeleton.skin;
    w.asset
        .joint_node(skin, joint)
        .ok_or_else(|| invalid(format!("{field} {joint} is not a joint of skin {skin}")))
}

/// The limits of `nodes` on `w`'s skeleton, aligned with them (empty when
/// none is limited, so the solvers take their unlimited path).
fn chain_limits(w: &Work, nodes: &[u32], limits: &Limits) -> Vec<Option<JointLimit>> {
    let key = w.entity.to_bits();
    let out: Vec<Option<JointLimit>> = nodes
        .iter()
        .map(|&n| limits.get(&(key, n)).copied())
        .collect();
    if out.iter().any(Option::is_some) {
        out
    } else {
        Vec::new()
    }
}

/// A point of `space` in the skeleton's model space (world: the inverse
/// of the entity's Transform).
fn model_point(space: u32, t: Option<Transform>, p: Vec3) -> Vec3 {
    match (space, t) {
        (IK_WORLD, Some(t)) => Trs::new(t.position, t.quaternion, t.scale)
            .inverse()
            .transform_point(p),
        _ => p,
    }
}

/// A direction of `space` in the skeleton's model space.
fn model_direction(space: u32, t: Option<Transform>, v: Vec3) -> Vec3 {
    match (space, t) {
        (IK_WORLD, Some(t)) => t.quaternion.conjugate() * v,
        _ => v,
    }
}

/// Where joint `n`'s tail is in its own frame: the next joint of `chain`
/// after it, else its first child, else its parent bone continued at the
/// same length (the rest pose's).
fn tail_of(asset: &SkeletonAsset, pose: &[Trs], chain: &[u32], i: usize) -> Vec3 {
    let n = chain[i];
    let child = chain.get(i + 1).copied().or_else(|| {
        asset
            .nodes
            .iter()
            .position(|c| c.parent == n)
            .map(|c| c as u32)
    });
    match child {
        Some(c) => pose[c as usize].translation,
        None => {
            let rest = &asset.nodes[n as usize].rest;
            let s = rest.scale;
            let along = rest.rotation.conjugate() * rest.translation;
            if s.x != 0.0 && s.y != 0.0 && s.z != 0.0 {
                along / s
            } else {
                Vec3::ZERO
            }
        }
    }
}

/// A spring collider placed for one chain: in the spring's frame, a
/// segment (a point for a sphere) and the distance a tail keeps from it.
struct Placed {
    a: Vec3,
    b: Vec3,
    reach: f32,
}

/// The point of segment `a`…`b` nearest `x`.
fn nearest_on_segment(a: Vec3, b: Vec3, x: Vec3) -> Vec3 {
    let ab = b - a;
    let len2 = ab.dot(ab);
    if len2 <= 0.0 {
        return a;
    }
    a + ab * ((x - a).dot(ab) / len2).clamp(0.0, 1.0)
}

/// Where a tail at `x`, `length` from `head`, comes to rest outside a ball
/// of `reach` about `centre`: the point nearest `x` on the circle where the
/// bone's sphere about `head` meets the ball's surface. None when the bone
/// cannot clear the ball (its head too deep) or `x` lies on the axis.
fn push_out(head: Vec3, length: f32, centre: Vec3, reach: f32, x: Vec3) -> Option<Vec3> {
    let hc = centre - head;
    let d = hc.length();
    if d < 1e-9 || d + length < reach || d > length + reach {
        return None;
    }
    let n = hc / d;
    let a = (length * length - reach * reach + d * d) / (2.0 * d);
    let rho = (length * length - a * a).max(0.0).sqrt();
    let off = x - head;
    let side = off - n * off.dot(n);
    let l = side.length();
    (l > 1e-9).then(|| head + n * a + side * (rho / l))
}

/// One spring chain this tick: each joint's tail, top first, stepped
/// towards where the pose (with the joints above already swung) puts it,
/// held at the bone's length, pushed out of the colliders it meets (and
/// held at the length again), and the joint turned to point at it.
pub(super) fn spring(
    w: &mut Work,
    sb: &mut SpringBone,
    prev: Option<&SpringState>,
    colliders: &[(u32, SpringCollider)],
    dt: f32,
) -> Result<SpringState> {
    let node = joint(w, sb.joint, "og.SpringBone.joint")?;
    let bones = sb.bones.clamp(1, 16);
    let Some(chain) = ik::chain(&w.asset, node, bones - 1) else {
        return Err(invalid(format!(
            "og.SpringBone: joint {} has fewer than {} joints above it",
            sb.joint,
            bones - 1
        )));
    };
    let frame = match (sb.space, w.transform) {
        (SPRING_WORLD, Some(t)) => Trs::new(t.position, t.quaternion, t.scale),
        _ => Trs::IDENTITY,
    };
    let to_model = frame.inverse();
    let prev = prev.filter(|p| p.tails.len() == chain.len());
    // Implicit Euler on x'' = ω²(target − x) − 2ζω x' + g: stable at any
    // frequency and tick.
    // Per joint, top to tip, frequency and damping move linearly from
    // theirs to tipFrequency and tipDamping times theirs (1 and 1: exactly
    // the chain's own, for every joint).
    let (tip_f, tip_d) = (sb.tip_frequency.max(0.0), sb.tip_damping.max(0.0));
    let along = |i: usize| match chain.len() {
        1 => 0.0,
        n => i as f32 / (n - 1) as f32,
    };
    let coefficients = |i: usize| {
        let t = along(i);
        let omega = std::f32::consts::TAU * sb.frequency.max(0.0) * (1.0 + (tip_f - 1.0) * t);
        let zeta = sb.damping.max(0.0) * (1.0 + (tip_d - 1.0) * t);
        (omega * omega, 2.0 * zeta * omega)
    };
    let weight = sb.weight.clamp(0.0, 1.0);
    let model = w.asset.model_space(&w.pose.nodes);
    // The colliders of this skeleton the chain is kept out of, placed in
    // the spring's frame from the pose as it is when the chain starts.
    let scale = frame.scale.abs().max_element();
    let radius = sb.radius.max(0.0);
    let placed: Vec<Placed> = colliders
        .iter()
        .filter(|(_, c)| c.skeleton == sb.skeleton && c.layers & sb.colliders != 0)
        .map(|&(node, c)| {
            let m = frame.mul(&model[node as usize]);
            let a = m.transform_point(c.offset);
            let b = if c.shape == SPRING_CAPSULE {
                m.transform_point(c.tail)
            } else {
                a
            };
            Placed {
                a,
                b,
                reach: (c.radius.max(0.0) + radius) * scale,
            }
        })
        .collect();
    let mut parent = match w.asset.nodes[chain[0] as usize].parent {
        NONE => Trs::IDENTITY,
        p => model[p as usize],
    };
    let mut tails = Vec::with_capacity(chain.len());
    for (i, &n) in chain.iter().enumerate() {
        let tail = tail_of(&w.asset, &w.pose.nodes, &chain, i);
        let local = w.pose.nodes[n as usize];
        let m = parent.mul(&local);
        let head = frame.transform_point(m.translation);
        let target = frame.transform_point(m.transform_point(tail));
        let length = (target - head).length();
        let (mut x, mut v) = prev.map_or((target, Vec3::ZERO), |p| p.tails[i]);
        if length < 1e-6 {
            // No bone to swing.
            tails.push((target, Vec3::ZERO));
            parent = m;
            continue;
        }
        let x0 = x;
        if dt > 0.0 {
            let (k, c) = coefficients(i);
            v = (v + ((target - x) * k + sb.gravity) * dt) / (1.0 + dt * c + dt * dt * k);
            x += v * dt;
        }
        // Held at the bone's length from the head.
        let hold = |x: Vec3| {
            let d = x - head;
            let l = d.length();
            if l > 1e-9 {
                head + d * (length / l)
            } else {
                target
            }
        };
        x = hold(x);
        // Out of every collider it meets, in order, still at the length.
        for p in &placed {
            let near = nearest_on_segment(p.a, p.b, x);
            if (x - near).length() < p.reach {
                x = push_out(head, length, near, p.reach, x).unwrap_or_else(|| {
                    // The bone cannot clear it (its head is inside): out
                    // from the centre, then back at the length.
                    let d = x - near;
                    hold(near + d.normalize_or((x - head).normalize_or(Vec3::Y)) * p.reach)
                });
            }
        }
        if dt > 0.0 {
            v = (x - x0) / dt;
        }
        sb.lag = sb.lag.max((x - target).length());
        let from = m.rotation * (m.scale * tail);
        let to = to_model.transform_point(x) - m.translation;
        let turned = unit(rotation_arc(from, to) * m.rotation);
        let swung = unit(parent.rotation.conjugate() * turned);
        w.pose.nodes[n as usize].rotation = nlerp(local.rotation, swung, weight);
        parent = parent.mul(&w.pose.nodes[n as usize]);
        tails.push((x, v));
    }
    Ok(SpringState {
        skeleton: sb.skeleton,
        space: sb.space,
        tails,
    })
}

pub(super) fn aim(w: &mut Work, look: &mut LookAt, limits: &Limits) -> Result<()> {
    let node = joint(w, look.joint, "og.LookAt.joint")?;
    let bones = look.bones.clamp(1, 8);
    let Some(nodes) = ik::chain(&w.asset, node, bones - 1) else {
        return Err(invalid(format!(
            "og.LookAt: joint {} has fewer than {} joints above it",
            look.joint,
            bones - 1
        )));
    };
    let target = model_point(look.space, w.transform, look.target);
    let up = (look.up != Vec3::ZERO).then(|| model_direction(look.space, w.transform, look.up));
    let lim = chain_limits(w, &nodes, limits);
    look.clamped = ik::look_at(&w.asset, &mut w.pose.nodes, &nodes, target, up, look, &lim);
    Ok(())
}

pub(super) fn solve(w: &mut Work, chain: &mut IkChain, limits: &Limits) -> Result<()> {
    let tip = joint(w, chain.tip, "og.IkChain.tip")?;
    let bones = if chain.solver == IK_TWO_BONE {
        2
    } else {
        chain.bones.max(1)
    };
    let Some(nodes) = ik::chain(&w.asset, tip, bones) else {
        return Err(invalid(format!(
            "og.IkChain: joint {} has fewer than {bones} bones above it",
            chain.tip
        )));
    };
    let to_model = |p: Vec3| model_point(chain.space, w.transform, p);
    let target = to_model(chain.target);
    let weight = chain.weight.clamp(0.0, 1.0);
    let lim = chain_limits(w, &nodes, limits);
    let result = if chain.solver == IK_TWO_BONE {
        let pole = (chain.pole != Vec3::ZERO).then(|| to_model(chain.pole));
        ik::two_bone(
            &w.asset,
            &mut w.pose.nodes,
            &nodes,
            target,
            pole,
            weight,
            &lim,
        )
    } else {
        ik::fabrik(
            &w.asset,
            &mut w.pose.nodes,
            &nodes,
            target,
            chain.iterations,
            chain.tolerance,
            weight,
            &lim,
        )
    };
    chain.error = result.error;
    chain.reached = result.reachable && result.error <= chain.tolerance;
    Ok(())
}
