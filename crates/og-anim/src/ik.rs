//! Inverse kinematics on a pose: the analytic two-bone solver and FABRIK.
//! Both find new model-space joint positions for a parent-linked chain and
//! then turn each joint (root to tip) by the shortest arc onto them; only
//! local rotations change, so bone lengths are kept exactly.

use crate::asset::{SkeletonAsset, NONE};
use crate::xform::{nlerp, rotation_arc, Trs};
use og_core::glam::Vec3;

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

/// Turns the chain's joints so their model-space positions become
/// `targets` (one per chain node, the first unchanged), blending each new
/// local rotation with the old one by `weight`.
fn apply_positions(
    skeleton: &SkeletonAsset,
    pose: &mut [Trs],
    chain: &[u32],
    targets: &[Vec3],
    weight: f32,
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
        let solved = crate::xform::unit(parent_rot.conjugate() * new_model_rot);
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
/// towards it.
pub fn two_bone(
    skeleton: &SkeletonAsset,
    pose: &mut [Trs],
    chain: &[u32],
    target: Vec3,
    pole: Option<Vec3>,
    weight: f32,
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
    apply_positions(skeleton, pose, chain, &[a, knee, tip], weight);
    IkResult {
        error: tip_error(skeleton, pose, chain[2], target),
        reachable,
    }
}

/// FABRIK over `chain` (two or more nodes) towards `target`: at most
/// `iterations` backward-forward passes, stopping within `tolerance`.
pub fn fabrik(
    skeleton: &SkeletonAsset,
    pose: &mut [Trs],
    chain: &[u32],
    target: Vec3,
    iterations: u32,
    tolerance: f32,
    weight: f32,
) -> IkResult {
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
    apply_positions(skeleton, pose, chain, &p, weight);
    IkResult {
        error: tip_error(skeleton, pose, chain[last], target),
        reachable,
    }
}
