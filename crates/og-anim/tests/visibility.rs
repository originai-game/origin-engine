//! Visibility channels: a node's visible flag through sampling (held, never
//! interpolated), the hierarchy (a hidden node hides what is under it),
//! blending (the heavier pose's flag, so a cross-fade swaps at its middle),
//! additive and masked layers, mirroring, retargeting, and the state block
//! (a snapshot restores the flags, so the outputs and hashes match).
//!
//! Ways this could fail: a visibility sample interpolated (a fold shows a
//! tick early), a blend that ORs the flags (both frames of a cross-fade
//! drawn) or takes the newcomer's at once, a child drawn under a hidden
//! parent, a mirrored or retargeted clip that drops the channel, or flags
//! the state block does not keep (a restored world draws every node).

use crate::support;

use og_anim::*;
use og_core::glam::{Quat, Vec3};

const PROP: usize = 1;
const HAND_L: usize = 3;
const HAND_R: usize = 4;

/// A root with a prop (a cap under it) and two hands; the right hand is
/// hidden at rest.
fn rig() -> SkeletonAsset {
    let n = |name: &str, parent: u32, x: f32, mesh: u32| SkeletonNode {
        name: name.into(),
        parent,
        rest: Trs::new(Vec3::new(x, 0.0, 0.0), Quat::IDENTITY, Vec3::ONE),
        mesh,
    };
    SkeletonAsset {
        nodes: vec![
            n("root", NONE, 0.0, NONE),
            n("prop", 0, 0.0, 0),
            n("cap", 1, 0.0, 1),
            n("hand.L", 0, 0.3, 2),
            n("hand.R", 0, -0.3, 3),
        ],
        hidden: vec![HAND_R as u32],
        ..Default::default()
    }
}

fn vis(node: usize, name: &str, samples: &[f32]) -> Channel {
    Channel {
        node: node as u32,
        node_name: name.into(),
        path: ChannelPath::Visibility,
        step: true,
        lanes: 1,
        samples: samples.to_vec().into(),
    }
}

fn sampled(skeleton: &SkeletonAsset, clip: &ClipAsset, t: f32) -> Vec<bool> {
    let mut p = Pose::rest(skeleton);
    sample(skeleton, clip, &bind(skeleton, clip), t, NONE, &mut p);
    p.visible
}

#[test]
fn blends_take_the_heavier_poses_flags() {
    let sk = rig();
    let shown = Pose::rest(&sk);
    let mut hidden = Pose::rest(&sk);
    hidden.visible[PROP] = false;
    hidden.visible[HAND_R] = true;
    for (t, want) in [(0.4, true), (0.5, false), (0.9, false)] {
        let mut p = shown.clone();
        p.lerp_to(&hidden, t);
        assert_eq!(p.visible[PROP], want, "lerp_to by {t}");
        assert_eq!(p.visible[HAND_R], !want, "lerp_to by {t}");
    }
    // A masked layer at 0.8: the prop's branch at half (0.4) keeps the
    // base's flag, the right hand at full (0.8) takes the layer's.
    let mask = [0.0, 0.5, 0.5, 0.0, 1.0];
    let mut p = shown.clone();
    p.lerp_to_masked(&hidden, 0.8, &mask);
    assert!(p.visible[PROP]);
    assert!(p.visible[HAND_R]);
    // An additive layer sets the flags its clip toggles from its
    // reference frame, from a weight of one half.
    let mut base = shown.clone();
    base.visible[HAND_L] = false;
    for (w, want) in [(0.3, true), (0.6, false)] {
        let mut p = base.clone();
        p.add(&hidden, &shown, w);
        assert_eq!(p.visible[PROP], want, "additive at {w}");
        assert!(!p.visible[HAND_L], "not toggled by the layer: the base's");
    }
}

#[test]
fn retargeting_carries_visibility_by_bone_and_by_name() {
    // The walker and the tall Mixamo rig, each holding a sword.
    let with_sword = |mut s: SkeletonAsset| {
        s.nodes.push(SkeletonNode {
            name: "sword".into(),
            parent: 0,
            rest: Trs::IDENTITY,
            mesh: 7,
        });
        s
    };
    let source = with_sword(support::skeleton());
    let target = with_sword(support::tall());
    let (sm, tm) = (HumanoidMap::detect(&source), HumanoidMap::detect(&target));
    let arm = human_bone("leftUpperArm").unwrap();
    let (s_arm, t_arm) = (sm.nodes[arm], tm.nodes[arm]);
    assert!(s_arm != NONE && t_arm != NONE);
    let (_, walk) = support::clips()
        .into_iter()
        .find(|(h, _)| *h == support::WALK)
        .unwrap();
    let mut c = walk.clone();
    let frames = c.frames as usize;
    let blink: Vec<f32> = (0..frames).map(|i| (i % 2) as f32).collect();
    let flash: Vec<f32> = (0..frames)
        .map(|i| u8::from(i < frames / 2) as f32)
        .collect();
    let s_sword = source.node("sword").unwrap() as usize;
    c.channels.push(vis(
        s_arm as usize,
        &source.nodes[s_arm as usize].name,
        &blink,
    ));
    c.channels.push(vis(s_sword, "sword", &flash));
    c.validate().unwrap();
    let r = retarget(&source, &sm, &target, &tm, &c, &RetargetOptions::default())
        .unwrap()
        .clip;
    let t_sword = target.node("sword").unwrap();
    let got: Vec<(u32, Vec<f32>)> = r
        .channels
        .iter()
        .filter(|ch| ch.path == ChannelPath::Visibility)
        .map(|ch| (ch.node, ch.decoded(r.frames)))
        .collect();
    assert_eq!(
        got,
        vec![(t_arm, blink.to_vec()), (t_sword, flash.to_vec())]
    );
    // And it plays: the retargeted clip hides the target's arm on odd frames.
    let at = |i: usize| sampled(&target, &r, i as f32 / r.rate)[t_arm as usize];
    assert_eq!((at(0), at(1), at(2)), (false, true, false));
}
