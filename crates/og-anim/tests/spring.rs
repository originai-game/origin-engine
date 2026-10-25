//! Spring bones (`og.SpringBone`): a tail of three joints behind a body
//! that the game moves. Each joint's tail is a damped spring after where the
//! pose puts it, held at the bone's length.
//!
//! What would fail it: the springs taken in model space by default (the
//! tail would not lag behind the moving body), no damping (it would never
//! settle), a stretched bone, gravity ignored or not scaled by stiffness,
//! the tails outside the state block (a restore mid-swing would snap them
//! back), a spring turning joints outside its chain.

use og_anim::*;
use og_core::glam::{Quat, Vec3};
use og_ecs::{Entity, World, WorldDesc};

const SKELETON: u64 = 1;
/// Joints (skin order = nodes 1..): the body, then the tail's three bones.
const BODY: u32 = 0;
const TAIL1: u32 = 1;
const TAIL3: u32 = 3;
/// Node of the last tail joint.
const TIP_NODE: usize = 4;

fn skeleton() -> SkeletonAsset {
    let n = |name: &str, parent: u32, t: [f32; 3]| SkeletonNode {
        name: name.into(),
        parent,
        rest: Trs::new(Vec3::from(t), Quat::IDENTITY, Vec3::ONE),
        mesh: NONE,
    };
    let mut s = SkeletonAsset {
        nodes: vec![
            n("Rig", NONE, [0.0, 0.0, 0.0]),
            n("body", 0, [0.0, 1.0, 0.0]),
            n("tail1", 1, [0.0, 0.0, -0.2]),
            n("tail2", 2, [0.0, 0.0, -0.2]),
            n("tail3", 3, [0.0, 0.0, -0.2]),
        ],
        skins: Vec::new(),
    };
    let model = s.model_matrices(&s.rest_pose());
    let joints: Vec<u32> = (1..5).collect();
    let inverse_bind = joints
        .iter()
        .map(|&j| model[j as usize].inverse())
        .collect();
    s.skins.push(Skin {
        name: "critter".into(),
        joints,
        inverse_bind,
    });
    s
}

fn world(spring: Option<SpringBone>) -> (World, AnimationSystem, Entity, Option<Entity>) {
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 1,
    })
    .unwrap();
    let anim = install(&mut w).unwrap();
    anim.add_skeleton(SKELETON, skeleton()).unwrap();
    let e = w
        .spawn((
            Transform::default(),
            Skeleton {
                asset: SKELETON,
                ..Skeleton::default()
            },
        ))
        .unwrap();
    let s = spring.map(|s| {
        w.spawn((SpringBone {
            skeleton: e.to_bits(),
            ..s
        },))
            .unwrap()
    });
    (w, anim, e, s)
}

fn tail(bones: u32) -> SpringBone {
    SpringBone {
        joint: TAIL3,
        bones,
        ..SpringBone::default()
    }
}

/// Moves the body along +x at `speed` m/s for `moving` ticks, then holds
/// it, `ticks` in all; per tick: the tip's position relative to the body
/// and the spring's `lag`.
fn run(
    w: &mut World,
    anim: &AnimationSystem,
    e: Entity,
    s: Option<Entity>,
    speed: f32,
    moving: u32,
    ticks: u32,
) -> Vec<(Vec3, f32)> {
    let mut out = Vec::new();
    for tick in 0..ticks {
        let mut t: Transform = w.get(e).unwrap();
        if tick < moving {
            t.position.x += speed / 60.0;
        }
        w.insert(e, t).unwrap();
        w.step().unwrap();
        let model = skeleton().model_space(&anim.pose(e).unwrap());
        let lag = s.map_or(0.0, |s| w.get::<SpringBone>(s).unwrap().lag);
        out.push((model[TIP_NODE].translation, lag));
    }
    out
}

const REST_TIP: Vec3 = Vec3::new(0.0, 1.0, -0.6);

#[test]
fn a_tail_lags_behind_the_moving_body_then_swings_back_and_settles() {
    let (mut w, anim, e, s) = world(Some(tail(3)));
    let probe = run(&mut w, &anim, e, s, 2.0, 30, 300);
    // While the body moves, the tip trails it (towards -x of the rest tip).
    let trailing = probe[10..30].iter().map(|p| p.0.x).fold(f32::MAX, f32::min);
    assert!(trailing < -0.1, "the moving tip trails by {trailing} m");
    assert!(probe[20].1 > 0.02, "lag {}", probe[20].1);
    // Stopped, it swings past the rest place (damping 0.3 overshoots)...
    let past = probe[30..90].iter().map(|p| p.0.x).fold(f32::MIN, f32::max);
    assert!(past > 0.03, "the tip swings past rest by {past} m");
    // ...and settles back onto the animated pose.
    let (end, lag) = probe[299];
    assert!((end - REST_TIP).length() < 1e-3, "settled at {end}");
    assert!(lag < 1e-3, "lag {lag}");
    // Bones keep their lengths throughout.
    let pose = anim.pose(e).unwrap();
    for bone in &pose[2..=4] {
        assert!((bone.translation.length() - 0.2).abs() < 1e-6);
    }
    // Without a spring, the tip rides rigidly with the body.
    let (mut w, anim, e, _) = world(None);
    for (tip, _) in run(&mut w, &anim, e, None, 2.0, 30, 60) {
        assert_eq!(tip, REST_TIP);
    }
}

#[test]
fn a_model_space_spring_ignores_the_entitys_motion() {
    let (mut w, anim, e, s) = world(Some(SpringBone {
        space: SPRING_MODEL,
        ..tail(3)
    }));
    for (tip, lag) in run(&mut w, &anim, e, s, 5.0, 60, 90) {
        assert!((tip - REST_TIP).length() < 1e-5, "{tip}");
        assert!(lag < 1e-5, "lag {lag}");
    }
}

#[test]
fn gravity_droops_a_soft_tail_more_than_a_stiff_one() {
    let droop = |frequency: f32| {
        let (mut w, anim, e, s) = world(Some(SpringBone {
            frequency,
            damping: 1.0,
            gravity: Vec3::new(0.0, -9.8, 0.0),
            ..tail(3)
        }));
        let probe = run(&mut w, &anim, e, s, 0.0, 0, 600);
        let tip = probe[599].0;
        // Settled: the last second moves it by less than a tenth of a millimetre.
        assert!((tip - probe[540].0).length() < 1e-4);
        REST_TIP.y - tip.y
    };
    let (soft, stiff) = (droop(1.0), droop(4.0));
    eprintln!("droop: soft {soft} m, stiff {stiff} m");
    assert!(stiff > 0.02, "a stiff tail droops {stiff} m");
    assert!(soft > 3.0 * stiff, "soft {soft} m, stiff {stiff} m");
}

#[test]
fn a_single_joint_spring_turns_that_joint_only() {
    let (mut w, anim, e, s) = world(Some(SpringBone {
        joint: TAIL1,
        ..tail(1)
    }));
    let probe = run(&mut w, &anim, e, s, 3.0, 20, 20);
    assert!(probe[19].1 > 0.01, "lag {}", probe[19].1);
    let pose = anim.pose(e).unwrap();
    let rest = skeleton().rest_pose();
    assert_ne!(pose[2].rotation, rest[2].rotation, "tail1 swings");
    for n in [1usize, 3, 4] {
        assert_eq!(pose[n], rest[n], "node {n} keeps its pose");
    }
    // A joint id past the skin is refused with the field.
    let (mut w, _, _, _) = world(Some(SpringBone {
        joint: 9,
        ..tail(1)
    }));
    let err = w.step().unwrap_err();
    assert!(err.reason().contains("og.SpringBone.joint"), "{err}");
    let _ = BODY;
}

#[test]
fn a_restore_mid_swing_continues_the_same() {
    let steps = |w: &mut World, e: Entity, n: u32| {
        let mut hashes = Vec::new();
        for i in 0..n {
            let mut t: Transform = w.get(e).unwrap();
            t.position.x += if i % 40 < 20 { 0.05 } else { -0.05 };
            w.insert(e, t).unwrap();
            w.step().unwrap();
            hashes.push(w.world_hash());
        }
        hashes
    };
    let spring = SpringBone {
        gravity: Vec3::new(0.0, -3.0, 0.0),
        ..tail(3)
    };
    let (mut a, anim_a, ea, _) = world(Some(spring));
    steps(&mut a, ea, 50);
    let snap = a.snapshot();
    let after = steps(&mut a, ea, 60);
    let pose = anim_a.pose(ea).unwrap();
    // The same run again gives the same hashes.
    let (mut b, _, eb, _) = world(Some(spring));
    steps(&mut b, eb, 50);
    assert_eq!(steps(&mut b, eb, 60), after);
    // And a fresh world restored mid-swing continues exactly.
    let (mut c, anim_c, ec, _) = world(Some(spring));
    c.restore(&snap).unwrap();
    assert_eq!(steps(&mut c, ec, 60), after);
    assert_eq!(anim_c.pose(ec).unwrap(), pose);
}
