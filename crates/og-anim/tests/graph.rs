//! The `animgraph` asset and its state machine: transitions fire in the tick
//! their condition holds and cross-fade over their `fade`, blend1d weights
//! follow the parameter, the additive layer adds its difference, and bad
//! text is refused with its line.

use crate::support;

use og_anim::*;
use og_ecs::{Entity, World, WorldDesc};
use support::*;

fn world() -> (World, AnimationSystem, Entity) {
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 1,
    })
    .unwrap();
    let anim = install(&mut w).unwrap();
    register(&anim);
    let e = w
        .spawn((
            Transform::default(),
            Skeleton {
                asset: SKELETON,
                ..Skeleton::default()
            },
            AnimationGraph::new(GRAPH),
        ))
        .unwrap();
    (w, anim, e)
}

fn set(w: &mut World, e: Entity, f: impl Fn(&mut AnimationGraph)) {
    let mut g: AnimationGraph = w.get(e).unwrap();
    f(&mut g);
    w.insert(e, g).unwrap();
}

#[test]
fn transitions_fire_when_their_condition_holds_and_cross_fade() {
    let (mut w, anim, e) = world();
    for _ in 0..10 {
        w.step().unwrap();
    }
    assert_eq!(w.get::<AnimationGraph>(e).unwrap().state, 0, "idle");
    set(&mut w, e, |g| g.p0 = 1.5); // walk speed
    w.step().unwrap();
    let g: AnimationGraph = w.get(e).unwrap();
    assert_eq!(
        (g.state, g.state_time),
        (1, 1.0 / 60.0),
        "move from this tick"
    );
    // Seven ticks into the 0.25 s (15-tick) fade the thigh is between idle's rest and walk's.
    let mut walk_only = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 1,
    })
    .unwrap();
    let a2 = install(&mut walk_only).unwrap();
    register(&a2);
    let e2 = walk_only
        .spawn((
            Transform::default(),
            Skeleton {
                asset: SKELETON,
                ..Skeleton::default()
            },
            AnimationPlayer::new(WALK),
        ))
        .unwrap();
    for _ in 0..7 {
        walk_only.step().unwrap();
    }
    for _ in 0..6 {
        w.step().unwrap();
    }
    let thigh = anim.pose(e).unwrap()[4].rotation;
    let walk_thigh = a2.pose(e2).unwrap()[4].rotation;
    assert!(thigh.dot(walk_thigh).abs() < 1.0 - 1e-6, "still fading");
    for _ in 0..60 {
        w.step().unwrap();
        walk_only.step().unwrap();
    }
    // After the fade, at speed 1.5 the blend is pure walk, in step with it.
    let a = anim.pose(e).unwrap();
    let b = a2.pose(e2).unwrap();
    for n in [4, 5, 7, 8] {
        assert!(
            a[n].rotation.dot(b[n].rotation).abs() > 1.0 - 1e-6,
            "node {n}"
        );
    }
    set(&mut w, e, |g| g.p0 = 0.0);
    w.step().unwrap();
    assert_eq!(w.get::<AnimationGraph>(e).unwrap().state, 0, "back to idle");
}

/// A one-state graph playing `line` (a `clip` line naming the walk).
fn mirror_world(line: &str) -> (World, AnimationSystem, Entity, Entity) {
    let text = format!(
        "type animgraph\nversion 1\n---\n{line}\nlayer base {{\n  state walk w\n  entry walk\n}}\n"
    );
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 1,
    })
    .unwrap();
    let anim = install(&mut w).unwrap();
    register(&anim);
    let resolve = |n: &str| match n {
        "walk mirrored" => Some(WALK_MIRRORED),
        n => resolve(n),
    };
    anim.add_graph(40, GraphAsset::parse(&text, &resolve).unwrap())
        .unwrap();
    let e = w
        .spawn((
            Transform::default(),
            Skeleton {
                asset: SKELETON,
                root_motion: ROOT_MOTION_PLANAR,
                ..Skeleton::default()
            },
            AnimationGraph::new(40),
        ))
        .unwrap();
    let l = w
        .spawn((FootLock {
            skeleton: e.to_bits(),
            foot: J_FOOT_L,
            ..FootLock::default()
        },))
        .unwrap();
    (w, anim, e, l)
}

#[test]
fn a_mirrored_graph_clip_plays_as_the_clip_og_anim_mirror_makes() {
    let (mut a, anim_a, ea, la) = mirror_world("clip w \"walk\" {\n  mirror true\n}");
    let (mut b, anim_b, eb, lb) = mirror_world("clip w \"walk mirrored\"");
    let (mut c, anim_c, ec, _) = mirror_world("clip w \"walk\"");
    let (mut differs, mut planted) = (false, 0);
    for _ in 0..120 {
        a.step().unwrap();
        b.step().unwrap();
        c.step().unwrap();
        assert_eq!(anim_a.pose(ea).unwrap(), anim_b.pose(eb).unwrap());
        assert_eq!(
            a.get::<Transform>(ea).unwrap(),
            b.get::<Transform>(eb).unwrap()
        );
        // The foot lock reads the mirrored clip's plants.
        let (fa, fb) = (
            a.get::<FootLock>(la).unwrap(),
            b.get::<FootLock>(lb).unwrap(),
        );
        assert_eq!((fa.planted, fa.locked), (fb.planted, fb.locked));
        planted += fa.planted as usize;
        // Markers keep their names and times, reported against the walk.
        let ev = |w: &World| -> Vec<(String, f32, u64)> {
            w.pending_events::<AnimationEvent>()
                .unwrap()
                .iter()
                .map(|e| (e.name.as_str().to_string(), e.time, e.clip))
                .collect()
        };
        let (va, vb) = (ev(&a), ev(&b));
        assert_eq!(
            va.iter().map(|e| (&e.0, e.1)).collect::<Vec<_>>(),
            vb.iter().map(|e| (&e.0, e.1)).collect::<Vec<_>>()
        );
        assert!(va.iter().all(|e| e.2 == WALK));
        differs |= anim_a.pose(ea).unwrap() != anim_c.pose(ec).unwrap();
    }
    assert!(differs, "the mirrored walk is not the walk");
    assert!(planted > 0, "the mirrored walk plants the left foot");
    let _ = anim_c;
    // `mirror` takes true or false.
    let e = GraphAsset::parse(
        "type animgraph\nversion 1\n---\nclip w \"walk\" {\n  mirror maybe\n}\nlayer base {\n  state walk w\n  entry walk\n}\n",
        &resolve,
    )
    .unwrap_err();
    assert!(e.reason().contains("line 5"), "{e}");
}
