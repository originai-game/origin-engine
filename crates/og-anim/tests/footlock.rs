//! Foot locking (`og.FootLock`): a foot the playing clips plant stays where
//! it touched the ground, through root motion and a cross-fade to another
//! clip, until the clips lift it; then it rejoins its animation over
//! `release`. "Planted" is retargeting's notion (`FootPlants`: within 5 mm
//! of the foot's lowest in the clip).
//!
//! The rig's procedural walk plants a foot for a frame or two only, so the
//! scenes here play a stomp: half a cycle on a standing leg while root
//! motion carries the walker on.
//!
//! What would fail it: the lock taken in model space (root motion carries
//! the foot away), the lock dropped on a cross-fade, never released (the
//! foot drags behind the walker), the lock state outside the state block (a
//! restore mid-plant moves the foot).

use crate::support;

use og_anim::*;
use og_core::glam::Vec3;
use og_core::math;
use og_ecs::{Entity, World, WorldDesc};
use support::*;

const FOOT_L: u32 = 6;
const STOMP: u64 = 30;
const JOG: u64 = 31;

/// A stomp: the hips travel `stride` per cycle (root motion); the left leg
/// stands, knee bent, for the first half of the cycle (the foot at its lowest:
/// planted, while root motion carries the walker on) and lifts, the knee
/// bent further, in the second.
fn stomp(name: &str, duration: f32, stride: f32) -> ClipAsset {
    let half = move |t: f32| (t / duration * 2.0 - 1.0).max(0.0);
    let pi = std::f32::consts::PI;
    support::clip(
        name,
        duration,
        &[],
        vec![
            (
                1,
                "hips",
                ChannelPath::Translation,
                3,
                Box::new(move |t| vec![0.0, 1.0, stride * t / duration]),
            ),
            (
                4,
                "thigh.L",
                ChannelPath::Rotation,
                4,
                Box::new(move |t| {
                    let a = -0.5 - 0.4 * math::sin(pi * half(t));
                    math::quat_from_axis_angle(Vec3::X, a).to_array().to_vec()
                }),
            ),
            (
                5,
                "shin.L",
                ChannelPath::Rotation,
                4,
                Box::new(move |t| {
                    let a = 1.0 + 0.6 * math::sin(pi * half(t));
                    math::quat_from_axis_angle(Vec3::X, a).to_array().to_vec()
                }),
            ),
        ],
    )
}

fn world(lock: bool) -> (World, AnimationSystem, Entity, Option<Entity>) {
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 1,
    })
    .unwrap();
    let anim = install(&mut w).unwrap();
    register(&anim);
    anim.add_clip(STOMP, stomp("stomp", 1.0, 0.4)).unwrap();
    anim.add_clip(JOG, stomp("jog", 0.7, 0.5)).unwrap();
    let e = w
        .spawn((
            Transform::default(),
            Skeleton {
                asset: SKELETON,
                root_motion: ROOT_MOTION_PLANAR,
                ..Skeleton::default()
            },
            AnimationPlayer {
                fade: 0.3,
                ..AnimationPlayer::new(STOMP)
            },
        ))
        .unwrap();
    let l = lock.then(|| {
        w.spawn((FootLock {
            skeleton: e.to_bits(),
            foot: J_FOOT_L,
            ..FootLock::default()
        },))
            .unwrap()
    });
    (w, anim, e, l)
}

fn foot_world(w: &World, anim: &AnimationSystem, e: Entity) -> Vec3 {
    let t: Transform = w.get(e).unwrap();
    let p = node_position(&anim.pose(e).unwrap(), FOOT_L);
    t.position + t.quaternion * (t.scale * p)
}

#[test]
fn the_walks_plants_are_retargetings() {
    let walk = clips().into_iter().find(|c| c.0 == WALK).unwrap().1;
    let p = FootPlants::new(&skeleton(), &walk, FOOT_L);
    let planted = (0..walk.frames as usize).filter(|&i| p.planted(i)).count();
    assert!(
        planted >= 2 && planted < walk.frames as usize / 2,
        "{planted} of {} frames",
        walk.frames
    );
}

/// Runs `ticks`, switching to the jog at `switch`; per tick: (locked,
/// planted, foot in world).
fn run(
    w: &mut World,
    anim: &AnimationSystem,
    e: Entity,
    l: Option<Entity>,
    ticks: u64,
    switch: u64,
) -> Vec<(bool, bool, Vec3)> {
    let mut out = Vec::new();
    for tick in 0..ticks {
        if tick == switch {
            let mut p: AnimationPlayer = w.get(e).unwrap();
            p.clip = JOG;
            w.insert(e, p).unwrap();
        }
        w.step().unwrap();
        let (locked, planted) = l.map_or((false, false), |l| {
            let f: FootLock = w.get(l).unwrap();
            (f.locked, f.planted)
        });
        out.push((locked, planted, foot_world(w, anim, e)));
    }
    out
}

#[test]
fn a_planted_foot_stays_put_through_root_motion_and_a_cross_fade() {
    // The jog starts ten ticks into the second plant, so the lock must
    // hold while the stomp fades into it.
    let (mut w, anim, e, l) = world(true);
    let probe = run(&mut w, &anim, e, l, 240, u64::MAX);
    let first = (1..probe.len())
        .filter(|&i| probe[i].0 && !probe[i - 1].0)
        .nth(1)
        .expect("the walk plants twice");
    let switch = first as u64 + 10;
    let (mut w, anim, e, l) = world(true);
    let locked = run(&mut w, &anim, e, l, 240, switch);
    let (mut fw, fanim, fe, _) = world(false);
    let free = run(&mut fw, &fanim, fe, None, 240, switch);

    let (mut worst, mut worst_free, mut held, mut spans) = (0f32, 0f32, 0usize, 0usize);
    let mut across_fade = false;
    let mut start: Option<(usize, Vec3)> = None;
    for (i, &(lk, _, p)) in locked.iter().enumerate() {
        match (lk, start) {
            (true, None) => {
                start = Some((i, p));
                spans += 1;
            }
            (true, Some((s, at))) => {
                held += 1;
                worst = worst.max((p - at).length());
                worst_free = worst_free.max((free[i].2 - free[s].2).length());
                across_fade |= s < switch as usize && i > switch as usize;
            }
            (false, _) => start = None,
        }
    }
    eprintln!("{spans} plants, {held} held ticks, locked drift {worst}, free drift {worst_free}");
    assert!(spans >= 3 && held >= 60, "{spans} plants, {held} ticks");
    assert!(worst < 2e-3, "a locked foot moves {worst} m");
    assert!(
        worst_free > 0.02,
        "without the lock it slides {worst_free} m"
    );
    assert!(across_fade, "a plant held across the cross-fade to the jog");
    // Lifted, it rejoins its animation within `release` (0.15 s, 9 ticks):
    // the lock changes no clip or root motion, so the free run's foot is
    // where the animation puts it.
    let mut releases = 0;
    for i in 1..locked.len() - 10 {
        if locked[i - 1].0 && !locked[i].0 && !locked[i + 10].0 && !locked[i + 10].1 {
            releases += 1;
            let d = (locked[i + 10].2 - free[i + 10].2).length();
            assert!(
                d < 1e-4,
                "tick {i}: the released foot is {d} m from its animation"
            );
        }
    }
    assert!(releases >= 2, "{releases} releases checked");
}

#[test]
fn a_restore_mid_plant_keeps_the_foot_where_it_was() {
    let (mut a, anim_a, e, l) = world(true);
    let mut snap = None;
    for _ in 0..300 {
        a.step().unwrap();
        let f: FootLock = a.get(l.unwrap()).unwrap();
        if f.locked && snap.is_none() {
            // A tick into the plant, so the lock was taken before.
            a.step().unwrap();
            snap = Some(a.snapshot());
            break;
        }
    }
    let snap = snap.expect("the walk plants the foot");
    let mut hashes = Vec::new();
    for _ in 0..30 {
        a.step().unwrap();
        hashes.push((a.world_hash(), foot_world(&a, &anim_a, e)));
    }
    let (mut b, anim_b, eb, _) = world(true);
    b.restore(&snap).unwrap();
    for (h, p) in hashes {
        b.step().unwrap();
        assert_eq!(b.world_hash(), h);
        assert_eq!(foot_world(&b, &anim_b, eb), p);
    }
}
