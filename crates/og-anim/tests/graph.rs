//! The `animgraph` asset and its state machine: transitions fire in the tick
//! their condition holds and cross-fade over their `fade`, blend1d weights
//! follow the parameter, the additive layer adds its difference, and bad
//! text is refused with its line.

use crate::support;

use og_anim::*;
use og_ecs::{Entity, World, WorldDesc};
use std::collections::BTreeSet;
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

/// Three one-second clips that each pass a marker every half tick, so every
/// clip playing fires every tick, whatever its weight.
fn marker_world(mix: f32) -> (World, Entity) {
    const TEXT: &str = r#"type animgraph
version 1
---
param go
param mix
clip a1 "attack1"
clip a2 "attack2"
clip a3 "attack3"
blend1d pair mix {
  child a2 0
  child a3 1
}
layer base {
  state first a1
  state second pair
  entry first
  transition first second {
    when go gt 0.5
    fade 0.5
  }
}
layer upper {
  weight mix
  state only a3
  entry only
}
"#;
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 1,
    })
    .unwrap();
    let anim = install(&mut w).unwrap();
    anim.add_skeleton(SKELETON, skeleton()).unwrap();
    let marks: Vec<(String, f32)> = (0..240)
        .map(|i| (format!("m{i}"), i as f32 / 120.0))
        .collect();
    let marks: Vec<(&str, f32)> = marks.iter().map(|(n, t)| (n.as_str(), *t)).collect();
    for (h, name) in [(31, "attack1"), (32, "attack2"), (33, "attack3")] {
        let spine = Box::new(move |t: f32| {
            let q = og_core::math::quat_from_axis_angle(og_core::glam::Vec3::Z, 0.1 * h as f32 * t);
            q.to_array().to_vec()
        });
        let c = clip(
            name,
            2.0,
            &marks,
            vec![(2, "spine", ChannelPath::Rotation, 4, spine)],
        );
        anim.add_clip(h, c).unwrap();
    }
    let resolve = |n: &str| match n {
        "attack1" => Some(31),
        "attack2" => Some(32),
        "attack3" => Some(33),
        _ => None,
    };
    anim.add_graph(41, GraphAsset::parse(TEXT, &resolve).unwrap())
        .unwrap();
    let mut graph = AnimationGraph::new(41);
    graph.p1 = mix;
    let e = w
        .spawn((
            Transform::default(),
            Skeleton {
                asset: SKELETON,
                ..Skeleton::default()
            },
            graph,
        ))
        .unwrap();
    (w, e)
}

/// The markers fired in the last tick, each as (clip, state, layer, dominant, weight).
fn fired(w: &World) -> Vec<(u64, u32, u32, bool, f32)> {
    w.pending_events::<AnimationEvent>()
        .unwrap()
        .iter()
        .map(|e| (e.clip, e.state, e.layer, e.dominant, e.weight))
        .collect()
}

#[test]
fn cross_fade_markers_say_their_state_layer_and_which_clip_dominates() {
    let (mut w, e) = marker_world(0.0);
    for _ in 0..5 {
        w.step().unwrap();
    }
    set(&mut w, e, |g| g.p0 = 1.0);
    // The 0.5 s fade is 30 ticks: attack1 weighs more for its first half.
    let mut dominant = Vec::new();
    for tick in 0..40 {
        w.step().unwrap();
        let f = fired(&w);
        for &(clip, state, layer, _, _) in &f {
            assert_eq!(layer, 0, "tick {tick}: the upper layer weighs nothing");
            assert_eq!(state, if clip == 31 { 0 } else { 1 }, "tick {tick}");
        }
        let fading = tick < 29;
        assert_eq!(
            f.iter().any(|m| m.0 == 31),
            fading,
            "tick {tick}: attack1 fires while it fades out"
        );
        assert!(
            f.iter().any(|m| m.0 == 32),
            "tick {tick}: attack2 fires from the start"
        );
        assert!(
            !f.iter().any(|m| m.0 == 33),
            "tick {tick}: attack3 has no weight"
        );
        let mut on: Vec<u64> = f.iter().filter(|m| m.3).map(|m| m.0).collect();
        on.dedup();
        assert_eq!(on.len(), 1, "tick {tick}: one dominant clip, {f:?}");
        // The dominant clip is the heavier one (the weight its markers carry).
        let weight = |c: u64| f.iter().find(|m| m.0 == c).map_or(0.0, |m| m.4);
        let other = if on[0] == 31 { 32 } else { 31 };
        assert!(weight(on[0]) >= weight(other), "tick {tick}: {f:?}");
        dominant.push(on[0]);
    }
    // attack1 until the fade passes half (alpha = (tick + 1) / 30 reaches
    // 0.5 on tick 14), attack2 from then on.
    let switch = dominant.iter().position(|&c| c == 32).unwrap();
    assert!((14..=15).contains(&switch), "{dominant:?}");
    assert!(dominant[switch..].iter().all(|&c| c == 32), "{dominant:?}");
}

#[test]
fn a_three_way_blend_has_one_dominant_clip_per_layer() {
    // attack1 fading out into an even attack2 and attack3 blend; the upper
    // layer at weight 0.5 plays attack3 alone.
    let (mut w, e) = marker_world(0.5);
    w.step().unwrap();
    set(&mut w, e, |g| g.p0 = 1.0);
    let mut seen = BTreeSet::new();
    for tick in 0..40 {
        w.step().unwrap();
        let f = fired(&w);
        for layer in [0, 1] {
            let mut on: Vec<u64> = f
                .iter()
                .filter(|m| m.2 == layer && m.3)
                .map(|m| m.0)
                .collect();
            on.dedup();
            assert_eq!(on.len(), 1, "tick {tick} layer {layer}: {f:?}");
            if layer == 1 {
                assert_eq!(on[0], 33);
                continue;
            }
            // The heaviest base clip; attack2 and attack3 tie and the later one wins.
            let base: Vec<_> = f.iter().filter(|m| m.2 == 0).collect();
            let heaviest = base.iter().map(|m| m.4).fold(0.0f32, f32::max);
            assert!(
                base.iter().filter(|m| m.3).all(|m| m.4 == heaviest),
                "tick {tick}: {f:?}"
            );
            assert_ne!(on[0], 32, "tick {tick}: a tie goes to the later clip");
            seen.insert(on[0]);
        }
    }
    assert_eq!(seen, BTreeSet::from([31, 33]), "attack1, then attack3");
}
