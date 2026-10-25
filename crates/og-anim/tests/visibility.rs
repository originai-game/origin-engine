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

mod support;

use og_anim::*;
use og_core::glam::{Quat, Vec3};
use og_ecs::{Entity, World, WorldDesc};

const RIG: u64 = 1;
const SWAP: u64 = 10;
const SHOW: u64 = 11;
const HIDE: u64 = 12;

const PROP: usize = 1;
const CAP: usize = 2;
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

/// Five frames at 4 Hz over a second.
fn clip(name: &str, channels: Vec<Channel>) -> ClipAsset {
    let c = ClipAsset {
        name: name.into(),
        duration: 1.0,
        rate: 4.0,
        source_rate: 4.0,
        frames: 5,
        channels,
        events: Vec::new(),
    };
    c.validate().unwrap();
    c
}

/// The prop shown on frames 0, 3 and 4, the left hand on 1 and 2.
fn swap() -> ClipAsset {
    clip(
        "swap",
        vec![
            vis(PROP, "prop", &[1.0, 0.0, 0.0, 1.0, 1.0]),
            vis(HAND_L, "hand.L", &[0.0, 1.0, 1.0, 0.0, 0.0]),
        ],
    )
}

fn sampled(skeleton: &SkeletonAsset, clip: &ClipAsset, t: f32) -> Vec<bool> {
    let mut p = Pose::rest(skeleton);
    sample(skeleton, clip, &bind(skeleton, clip), t, NONE, &mut p);
    p.visible
}

#[test]
fn a_visibility_channel_is_held_and_hides_what_is_under_its_node() {
    let sk = rig();
    let c = swap();
    assert_eq!(
        Pose::rest(&sk).visible,
        [true, true, true, true, false],
        "the rest pose hides the nodes the skeleton hides"
    );
    assert_eq!(sampled(&sk, &c, 0.0), [true, true, true, false, false]);
    assert_eq!(sampled(&sk, &c, 0.25), [true, false, true, true, false]);
    // 96 % of the way from frame 2 (hidden) to frame 3 (shown): held.
    assert!(
        !sampled(&sk, &c, 0.74)[PROP],
        "a fold is held to its frame's end"
    );
    assert!(sampled(&sk, &c, 0.75)[PROP]);
    // The cap is visible itself but under the hidden prop.
    let drawn = sk.drawn(&sampled(&sk, &c, 0.25));
    assert_eq!(drawn, [true, false, false, true, false]);

    // A visibility channel is STEP and holds 0 or 1.
    let mut linear = swap();
    linear.channels[0].step = false;
    assert!(linear.validate().is_err());
    let mut half = swap();
    half.channels[0].samples = vec![1.0, 0.5, 0.0, 1.0, 1.0].into();
    assert!(half.validate().is_err());
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

fn world() -> (World, AnimationSystem) {
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 3,
    })
    .unwrap();
    let anim = install(&mut w).unwrap();
    anim.add_skeleton(RIG, rig()).unwrap();
    anim.add_clip(SWAP, swap()).unwrap();
    let constant = |name: &str, prop: f32| {
        clip(
            name,
            vec![
                vis(PROP, "prop", &[prop; 5]),
                vis(HAND_L, "hand.L", &[1.0 - prop; 5]),
            ],
        )
    };
    anim.add_clip(SHOW, constant("show", 1.0)).unwrap();
    anim.add_clip(HIDE, constant("hide", 0.0)).unwrap();
    (w, anim)
}

fn skeleton() -> Skeleton {
    Skeleton {
        asset: RIG,
        ..Skeleton::default()
    }
}

#[test]
fn a_cross_fade_swaps_at_its_middle() {
    let (mut w, anim) = world();
    let e = w
        .spawn((
            skeleton(),
            AnimationPlayer {
                fade: 0.5,
                ..AnimationPlayer::new(SHOW)
            },
        ))
        .unwrap();
    for _ in 0..5 {
        w.step().unwrap();
    }
    let mut p: AnimationPlayer = w.get(e).unwrap();
    p.clip = HIDE;
    w.insert(e, p).unwrap();
    let mut seen = Vec::new();
    for _ in 0..40 {
        w.step().unwrap();
        let v = anim.visible(e).unwrap();
        assert_ne!(
            v[PROP], v[HAND_L],
            "one of the two is drawn on every tick of the fade"
        );
        seen.push(v[PROP]);
    }
    let swapped = seen.iter().position(|&v| !v).unwrap();
    assert!(
        (13..=17).contains(&swapped),
        "the 30-tick fade swaps at its middle, not at tick {swapped}"
    );
    assert!(seen[swapped..].iter().all(|&v| !v), "and stays swapped");
    assert!(
        !anim.output(e).unwrap().drawn[CAP],
        "the cap goes with the prop"
    );
}

#[test]
fn mirroring_moves_a_visibility_channel_to_the_other_side() {
    let sk = rig();
    let c = swap();
    let m = mirror(&sk, &c).unwrap();
    let vis_of = |c: &ClipAsset| -> Vec<(u32, Vec<f32>)> {
        let mut v: Vec<_> = c
            .channels
            .iter()
            .filter(|ch| ch.path == ChannelPath::Visibility)
            .map(|ch| (ch.node, ch.decoded(c.frames)))
            .collect();
        v.sort_by_key(|x| x.0);
        v
    };
    assert_eq!(
        vis_of(&m),
        vec![
            (PROP as u32, vec![1.0, 0.0, 0.0, 1.0, 1.0]),
            (HAND_R as u32, vec![0.0, 1.0, 1.0, 0.0, 0.0]),
        ]
    );
    assert!(sampled(&sk, &m, 0.25)[HAND_R]);
    assert_eq!(vis_of(&mirror(&sk, &m).unwrap()), vis_of(&c));
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

/// Two skeletons, one playing the swap, one at rest.
fn populated() -> (World, AnimationSystem, [Entity; 2]) {
    let (mut w, anim) = world();
    let a = w.spawn((skeleton(), AnimationPlayer::new(SWAP))).unwrap();
    let b = w.spawn((skeleton(),)).unwrap();
    (w, anim, [a, b])
}

fn outputs(anim: &AnimationSystem, es: &[Entity; 2]) -> Vec<AnimOutput> {
    es.iter().map(|&e| anim.output(e).unwrap()).collect()
}

/// The hashes of ticks `from` to `to`, and the snapshot and outputs after
/// tick `at`.
fn run(
    w: &mut World,
    anim: &AnimationSystem,
    es: &[Entity; 2],
    (from, to, at): (u32, u32, u32),
) -> (Vec<u64>, Vec<u8>, Vec<AnimOutput>) {
    let mut hashes = Vec::new();
    let (mut snap, mut out) = (Vec::new(), Vec::new());
    for tick in from..to {
        w.step().unwrap();
        hashes.push(w.world_hash());
        if tick + 1 == at {
            (snap, out) = (w.snapshot(), outputs(anim, es));
        }
    }
    (hashes, snap, out)
}

#[test]
fn a_snapshot_keeps_the_flags_and_the_hashes() {
    let (mut a, anim_a, ea) = populated();
    let (ha, snap, at_20) = run(&mut a, &anim_a, &ea, (0, 60, 20));
    let (mut b, anim_b, eb) = populated();
    let (hb, _, _) = run(&mut b, &anim_b, &eb, (0, 60, 20));
    assert_eq!(ha, hb, "two runs give the same hashes");
    // Tick 20 is a third of a second in: frame 1, the prop folded away.
    assert_eq!(
        at_20[0].drawn,
        [true, false, false, true, false],
        "the player's frame"
    );
    assert_eq!(at_20[1].drawn, [true, true, true, true, false], "the rest");

    let (mut c, anim_c, ec) = populated();
    c.restore(&snap).unwrap();
    assert_eq!(c.world_hash(), ha[19], "the restored world is tick 20's");
    assert_eq!(
        outputs(&anim_c, &ec),
        at_20,
        "a restored world draws what the saved one drew"
    );
    let (hc, _, _) = run(&mut c, &anim_c, &ec, (20, 60, 0));
    assert_eq!(hc, ha[20..], "and continues to the same hashes");
}
