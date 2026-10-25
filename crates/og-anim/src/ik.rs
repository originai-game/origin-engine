//! Inverse kinematics and constraints on a pose: the analytic two-bone
//! solver, FABRIK, the look-at (aim) constraint and joint limits. The IK
//! solvers find new model-space joint positions for a parent-linked chain
//! and then turn each joint (root to tip) by the shortest arc onto them;
//! only local rotations change, so bone lengths are kept exactly. A joint
//! with an [`JointLimit`] is clamped as it is turned.

use crate::asset::{NONE, SkeletonAsset};
use crate::schema::{JointLimit, LookAt};
use crate::xform::{Trs, nlerp, rotation_arc, unit};
use og_core::glam::{Quat, Vec3};
use og_core::math::{atan2, sin_cos};

/// What a solve did.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IkResult {
    /// Tip distance to the target after solving (model units).
    pub error: f32,
    /// The target was within the chain's reach.
    pub reachable: bool,
}

/// The chain of `bones` bones ending at `tip`, from its first joint to the
/// tip (`bones + 1` nodes), or `None` when the hierarchy is too short.
pub fn chain(skeleton: &SkeletonAsset, tip: u32, bones: u32) -> Option<Vec<u32>> {
    let mut nodes = vec![tip];
    let mut n = tip;
    for _ in 0..bones {
        let p = skeleton.nodes.get(n as usize)?.parent;
        if p == NONE {
            return None;
        }
        nodes.push(p);
        n = p;
    }
    nodes.reverse();
    Some(nodes)
}

/// The limit of chain position `i` (`limits` is aligned with the chain, or
/// empty for none).
fn limit_at(limits: &[Option<JointLimit>], i: usize) -> Option<&JointLimit> {
    limits.get(i).and_then(Option::as_ref)
}

impl JointLimit {
    /// `local`, a local rotation of the joint whose rest local rotation is
    /// `rest`, turned back inside the limit: its rotation from rest split
    /// into a swing of `axis` and a twist about it, the swing's angle held
    /// within `swing` and the twist within `twistMin..=twistMax`. Returned
    /// unchanged (bit for bit) when already inside.
    pub fn clamp(&self, rest: Quat, local: Quat) -> Quat {
        let len = self.axis.length();
        if len < 1e-12 || !len.is_finite() {
            return local;
        }
        let a = self.axis / len;
        let mut q = unit(rest.conjugate() * local);
        if q.w < 0.0 {
            q = -q;
        }
        let v = Vec3::new(q.x, q.y, q.z);
        let p = a * v.dot(a);
        let twist = Quat::from_xyzw(p.x, p.y, p.z, q.w);
        let twist = if twist.length_squared() > 1e-12 {
            unit(twist)
        } else {
            Quat::IDENTITY
        };
        let mut swing = unit(q * twist.conjugate());
        if swing.w < 0.0 {
            swing = -swing;
        }
        let theta = 2.0 * atan2(Vec3::new(twist.x, twist.y, twist.z).dot(a), twist.w);
        let sv = Vec3::new(swing.x, swing.y, swing.z);
        let s_len = sv.length();
        let phi = 2.0 * atan2(s_len, swing.w);
        let (lo, hi) = if self.twist_min <= self.twist_max {
            (self.twist_min, self.twist_max)
        } else {
            (self.twist_max, self.twist_min)
        };
        let theta_c = theta.clamp(lo, hi);
        let phi_c = phi.min(self.swing.max(0.0));
        if theta_c == theta && phi_c == phi {
            return local;
        }
        let twist = if theta_c == theta {
            twist
        } else {
            let (s, c) = sin_cos(theta_c * 0.5);
            Quat::from_xyzw(a.x * s, a.y * s, a.z * s, c)
        };
        let swing = if phi_c == phi || s_len < 1e-12 {
            swing
        } else {
            let (s, c) = sin_cos(phi_c * 0.5);
            let ax = sv / s_len;
            Quat::from_xyzw(ax.x * s, ax.y * s, ax.z * s, c)
        };
        unit(rest * swing * twist)
    }
}

/// Turns the chain's joints so their model-space positions become
/// `targets` (one per chain node, the first unchanged), blending each new
/// local rotation with the old one by `weight`; a limited joint's new
/// rotation is clamped before the blend.
fn apply_positions(
    skeleton: &SkeletonAsset,
    pose: &mut [Trs],
    chain: &[u32],
    targets: &[Vec3],
    weight: f32,
    limits: &[Option<JointLimit>],
) {
    let mut model = skeleton.model_space(pose);
    for i in 0..chain.len() - 1 {
        let (j, c) = (chain[i] as usize, chain[i + 1] as usize);
        let from = model[c].translation - model[j].translation;
        let to = targets[i + 1] - model[j].translation;
        let delta = rotation_arc(from, to);
        let new_model_rot = delta * model[j].rotation;
        let parent_rot = match skeleton.nodes[j].parent {
            NONE => og_core::glam::Quat::IDENTITY,
            p => model[p as usize].rotation,
        };
        let mut solved = unit(parent_rot.conjugate() * new_model_rot);
        if let Some(l) = limit_at(limits, i) {
            solved = l.clamp(skeleton.nodes[j].rest.rotation, solved);
        }
        pose[j].rotation = nlerp(pose[j].rotation, solved, weight);
        // Re-derive the rest of the chain from the turned joint.
        let parent = match skeleton.nodes[j].parent {
            NONE => Trs::IDENTITY,
            p => model[p as usize],
        };
        model[j] = parent.mul(&pose[j]);
        for &k in &chain[i + 1..] {
            let k = k as usize;
            let p = skeleton.nodes[k].parent as usize;
            model[k] = model[p].mul(&pose[k]);
        }
    }
}

fn tip_error(skeleton: &SkeletonAsset, pose: &[Trs], tip: u32, target: Vec3) -> f32 {
    let model = skeleton.model_space(pose);
    (model[tip as usize].translation - target).length()
}

/// The analytic two-bone solve of `chain` (three nodes: upper, middle,
/// tip) towards `target`, bending towards `pole` (model space; `None` keeps
/// the current bend plane). An unreachable target straightens the chain
/// towards it. `limits` (aligned with `chain`, or empty) clamps each joint
/// as it turns, root first; a target they keep out of reach is missed.
pub fn two_bone(
    skeleton: &SkeletonAsset,
    pose: &mut [Trs],
    chain: &[u32],
    target: Vec3,
    pole: Option<Vec3>,
    weight: f32,
    limits: &[Option<JointLimit>],
) -> IkResult {
    debug_assert_eq!(chain.len(), 3);
    let model = skeleton.model_space(pose);
    let a = model[chain[0] as usize].translation;
    let b = model[chain[1] as usize].translation;
    let c = model[chain[2] as usize].translation;
    let (l1, l2) = ((b - a).length(), (c - b).length());
    let to_target = target - a;
    let dist = to_target.length();
    if l1 < 1e-9 || l2 < 1e-9 || dist < 1e-9 {
        return IkResult {
            error: (c - target).length(),
            reachable: false,
        };
    }
    let dir = to_target / dist;
    let reach_max = l1 + l2;
    let reach_min = (l1 - l2).abs();
    let reachable = dist <= reach_max && dist >= reach_min;
    let d = dist.clamp(reach_min + 1e-6, reach_max - 1e-6).max(1e-6);
    // The bend direction: towards the pole, else the current knee.
    let hint = pole.map_or(b - a, |p| p - a);
    let mut bend = hint - dir * hint.dot(dir);
    if bend.length_squared() < 1e-12 {
        let any = if dir.x.abs() < 0.9 { Vec3::X } else { Vec3::Y };
        bend = any - dir * any.dot(dir);
    }
    let bend = bend / bend.length();
    let x = (l1 * l1 - l2 * l2 + d * d) / (2.0 * d);
    let h = (l1 * l1 - x * x).max(0.0).sqrt();
    let knee = a + dir * x + bend * h;
    let tip = a + dir * d;
    apply_positions(skeleton, pose, chain, &[a, knee, tip], weight, limits);
    IkResult {
        error: tip_error(skeleton, pose, chain[2], target),
        reachable,
    }
}

/// FABRIK over `chain` (two or more nodes) towards `target`: at most
/// `iterations` backward-forward passes, stopping within `tolerance`. With
/// `limits` (aligned with `chain`), every pass turns the joints, clamping
/// each, and the next pass starts from the clamped chain.
#[allow(clippy::too_many_arguments)]
pub fn fabrik(
    skeleton: &SkeletonAsset,
    pose: &mut [Trs],
    chain: &[u32],
    target: Vec3,
    iterations: u32,
    tolerance: f32,
    weight: f32,
    limits: &[Option<JointLimit>],
) -> IkResult {
    if limits.iter().any(Option::is_some) {
        return fabrik_limited(
            skeleton, pose, chain, target, iterations, tolerance, weight, limits,
        );
    }
    let model = skeleton.model_space(pose);
    let mut p: Vec<Vec3> = chain
        .iter()
        .map(|&n| model[n as usize].translation)
        .collect();
    let lengths: Vec<f32> = p.windows(2).map(|w| (w[1] - w[0]).length()).collect();
    let total: f32 = lengths.iter().sum();
    let root = p[0];
    let reachable = (target - root).length() <= total;
    let last = p.len() - 1;
    if !reachable {
        let dir = target - root;
        let dir = if dir.length_squared() > 0.0 {
            dir / dir.length()
        } else {
            Vec3::Y
        };
        for i in 1..=last {
            p[i] = p[i - 1] + dir * lengths[i - 1];
        }
    } else {
        for _ in 0..iterations.max(1) {
            if (p[last] - target).length() <= tolerance {
                break;
            }
            p[last] = target;
            for i in (0..last).rev() {
                let d = p[i] - p[i + 1];
                let len = d.length();
                if len > 1e-12 {
                    p[i] = p[i + 1] + d * (lengths[i] / len);
                }
            }
            p[0] = root;
            for i in 1..=last {
                let d = p[i] - p[i - 1];
                let len = d.length();
                if len > 1e-12 {
                    p[i] = p[i - 1] + d * (lengths[i - 1] / len);
                }
            }
        }
    }
    apply_positions(skeleton, pose, chain, &p, weight, &[]);
    IkResult {
        error: tip_error(skeleton, pose, chain[last], target),
        reachable,
    }
}

/// One backward-forward FABRIK pass over positions `p` from `root`.
fn fabrik_pass(p: &mut [Vec3], lengths: &[f32], root: Vec3, target: Vec3) {
    let last = p.len() - 1;
    p[last] = target;
    for i in (0..last).rev() {
        let d = p[i] - p[i + 1];
        let len = d.length();
        if len > 1e-12 {
            p[i] = p[i + 1] + d * (lengths[i] / len);
        }
    }
    p[0] = root;
    for i in 1..=last {
        let d = p[i] - p[i - 1];
        let len = d.length();
        if len > 1e-12 {
            p[i] = p[i - 1] + d * (lengths[i - 1] / len);
        }
    }
}

/// FABRIK with joint limits: each pass's positions are turned into
/// clamped rotations at once, and the next pass starts from them.
#[allow(clippy::too_many_arguments)]
fn fabrik_limited(
    skeleton: &SkeletonAsset,
    pose: &mut [Trs],
    chain: &[u32],
    target: Vec3,
    iterations: u32,
    tolerance: f32,
    weight: f32,
    limits: &[Option<JointLimit>],
) -> IkResult {
    let animated: Vec<Quat> = chain.iter().map(|&n| pose[n as usize].rotation).collect();
    let positions = |pose: &[Trs]| -> Vec<Vec3> {
        let model = skeleton.model_space(pose);
        chain
            .iter()
            .map(|&n| model[n as usize].translation)
            .collect()
    };
    let mut p = positions(pose);
    let lengths: Vec<f32> = p.windows(2).map(|w| (w[1] - w[0]).length()).collect();
    let total: f32 = lengths.iter().sum();
    let root = p[0];
    let reachable = (target - root).length() <= total;
    let last = p.len() - 1;
    if !reachable {
        let dir = target - root;
        let dir = if dir.length_squared() > 0.0 {
            dir / dir.length()
        } else {
            Vec3::Y
        };
        for i in 1..=last {
            p[i] = p[i - 1] + dir * lengths[i - 1];
        }
        apply_positions(skeleton, pose, chain, &p, 1.0, limits);
    } else {
        for _ in 0..iterations.max(1) {
            if (p[last] - target).length() <= tolerance {
                break;
            }
            fabrik_pass(&mut p, &lengths, root, target);
            apply_positions(skeleton, pose, chain, &p, 1.0, limits);
            p = positions(pose);
        }
    }
    if weight < 1.0 {
        for (k, &n) in chain[..last].iter().enumerate() {
            let n = n as usize;
            pose[n].rotation = nlerp(animated[k], pose[n].rotation, weight);
        }
    }
    IkResult {
        error: tip_error(skeleton, pose, chain[last], target),
        reachable,
    }
}

/// `v` normalised, or `None` when it has no direction.
fn direction(v: Vec3) -> Option<Vec3> {
    let len = v.length();
    (len > 1e-12 && len.is_finite()).then(|| v / len)
}

/// The rotation about `axis` (a unit vector) taking `a` onto `b`, both
/// perpendicular to it.
fn arc_about(axis: Vec3, a: Vec3, b: Vec3) -> Quat {
    match (direction(a), direction(b)) {
        (Some(a), Some(b)) if a.dot(b) < -0.999_999 => Quat::from_xyzw(axis.x, axis.y, axis.z, 0.0),
        (Some(a), Some(b)) => rotation_arc(a, b),
        _ => Quat::IDENTITY,
    }
}

/// The model rotation of a joint at model rotation `current` whose local
/// `forward` points along `aim` (unit) and whose local `up` leans towards
/// `up_hint` (none: the shortest turn, keeping the current roll).
fn aimed(current: Quat, forward: Vec3, up: Vec3, aim: Vec3, up_hint: Option<Vec3>) -> Quat {
    let keep_roll = || unit(rotation_arc(current * forward, aim) * current);
    let Some(hint) = up_hint else {
        return keep_roll();
    };
    let want = hint - aim * hint.dot(aim);
    if want.length_squared() < 1e-12 {
        return keep_roll();
    }
    let q1 = rotation_arc(forward, aim);
    let u1 = q1 * up;
    let q2 = arc_about(aim, u1 - aim * u1.dot(aim), want);
    unit(q2 * q1)
}

/// The look-at (aim) constraint: turns `chain` (top first, the aiming joint
/// last) so the last joint's `look.axis` points at `target` (model space)
/// and its `look.upAxis` leans towards `up` (a model-space direction; none
/// keeps the animated roll). The direction is first held within
/// `look.maxYaw` and `look.maxPitch` of the animated forward (yaw about
/// the animated up axis, pitch towards it); each joint above the last takes
/// an even share of the remaining turn and the last joint the rest, each
/// clamped by `limits` (aligned with `chain`, or empty); the result is
/// blended with the animated rotations by `look.weight`. Returns whether the
/// direction was clamped.
pub fn look_at(
    skeleton: &SkeletonAsset,
    pose: &mut [Trs],
    chain: &[u32],
    target: Vec3,
    up: Option<Vec3>,
    look: &LookAt,
    limits: &[Option<JointLimit>],
) -> bool {
    let Some(&tip) = chain.last() else {
        return false;
    };
    let j = tip as usize;
    let Some(forward) = direction(look.axis) else {
        return false;
    };
    let up_axis =
        direction(look.up_axis - forward * look.up_axis.dot(forward)).unwrap_or_else(|| {
            let any = if forward.x.abs() < 0.9 {
                Vec3::X
            } else {
                Vec3::Y
            };
            (any - forward * any.dot(forward)).normalize()
        });
    let right = up_axis.cross(forward);
    let animated: Vec<Quat> = chain.iter().map(|&n| pose[n as usize].rotation).collect();
    let m = skeleton.model_space(pose)[j];
    let d = target - m.translation;
    if direction(d).is_none() {
        return false;
    }
    // The direction in the animated joint's frame, held within the angles.
    let dl = m.rotation.conjugate() * d;
    let (x, y, z) = (dl.dot(right), dl.dot(up_axis), dl.dot(forward));
    let yaw = atan2(x, z);
    let pitch = atan2(y, (x * x + z * z).sqrt());
    let max_yaw = look.max_yaw.max(0.0);
    let max_pitch = look.max_pitch.max(0.0);
    let (yc, pc) = (
        yaw.clamp(-max_yaw, max_yaw),
        pitch.clamp(-max_pitch, max_pitch),
    );
    let clamped = yc != yaw || pc != pitch;
    // Clamped, the joint aims along the held direction wherever its
    // parents carry it; otherwise at the target from where it ends up.
    let held = clamped.then(|| {
        let (sy, cy) = sin_cos(yc);
        let (sp, cp) = sin_cos(pc);
        m.rotation * ((right * sy + forward * cy) * cp + up_axis * sp)
    });
    let n = chain.len();
    for (i, &k) in chain.iter().enumerate() {
        let model = skeleton.model_space(pose);
        let Some(aim) = held.or_else(|| direction(target - model[j].translation)) else {
            break;
        };
        let want = aimed(model[j].rotation, forward, up_axis, aim, up);
        let delta = unit(want * model[j].rotation.conjugate());
        let k = k as usize;
        let share = if i + 1 == n {
            delta
        } else {
            nlerp(Quat::IDENTITY, delta, 1.0 / (n - i) as f32)
        };
        let parent_rot = match skeleton.nodes[k].parent {
            NONE => Quat::IDENTITY,
            p => model[p as usize].rotation,
        };
        let mut local = unit(parent_rot.conjugate() * unit(share * model[k].rotation));
        if let Some(l) = limit_at(limits, i) {
            local = l.clamp(skeleton.nodes[k].rest.rotation, local);
        }
        pose[k].rotation = local;
    }
    let weight = look.weight.clamp(0.0, 1.0);
    if weight < 1.0 {
        for (i, &k) in chain.iter().enumerate() {
            let k = k as usize;
            pose[k].rotation = nlerp(animated[i], pose[k].rotation, weight);
        }
    }
    clamped
}
