//! Layer masks and sync groups (the `animgraph` grammar's `mask` and
//! `sync` lines).
//!
//! What would fail it: a mask applied to the named node only (its children
//! left out), a mask weight ignored or applied to the blendshapes' nodes,
//! a mask on one layer leaking into another; a follower advancing its own
//! phase (its clip drifts from the leader's), the leader taken from the
//! wrong layer, a transition inside a group restarting at phase 0.

use crate::support;

use og_anim::*;
use og_core::glam::Quat;
use og_ecs::{Entity, World, WorldDesc};
use support::*;

const G: u64 = 40;
const HIPS: usize = 1;
const SPINE: usize = 2;
const HEAD: usize = 3;
const THIGH_L: usize = 4;
const ARM_L: usize = 10;
const FOREARM_L: usize = 11;

fn world(text: &str) -> (World, AnimationSystem, Entity) {
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 1,
    })
    .unwrap();
    let anim = install(&mut w).unwrap();
    register(&anim);
    anim.add_clip(50, spin()).unwrap();
    anim.add_graph(G, GraphAsset::parse(text, &resolve).unwrap())
        .unwrap();
    let e = w
        .spawn((
            Transform::default(),
            Skeleton {
                asset: SKELETON,
                ..Skeleton::default()
            },
            AnimationGraph::new(G),
        ))
        .unwrap();
    (w, anim, e)
}

fn pose_after(text: &str, ticks: u32) -> Vec<Trs> {
    let (mut w, anim, e) = world(text);
    for _ in 0..ticks {
        w.step().unwrap();
    }
    anim.pose(e).unwrap()
}

fn graph(layers: &str) -> String {
    format!(
        "type animgraph\nversion 1\n---\nparam p\nclip walk \"walk\"\nclip run \"run\"\nclip idle \"idle\"\nclip spin \"spin\"\n{layers}"
    )
}

const BASE: &str = "layer base {\n  state walk walk\n}\n";

/// A clip turning the spine and the left arm (so a mask can split them).
fn spin() -> ClipAsset {
    support::clip(
        "spin",
        1.0,
        &[],
        vec![
            (
                2,
                "spine",
                ChannelPath::Rotation,
                4,
                Box::new(|t| {
                    og_core::math::quat_from_axis_angle(og_core::glam::Vec3::Y, 0.8 * t)
                        .to_array()
                        .to_vec()
                }),
            ),
            (
                10,
                "arm.L",
                ChannelPath::Rotation,
                4,
                Box::new(|t| {
                    og_core::math::quat_from_axis_angle(og_core::glam::Vec3::Z, 1.0 * t)
                        .to_array()
                        .to_vec()
                }),
            ),
        ],
    )
}

fn resolve(name: &str) -> Option<u64> {
    match name {
        "spin" => Some(50),
        n => support::resolve(n),
    }
}

#[test]
fn a_mask_limits_an_override_layer_to_its_branch_by_weight() {
    let ticks = 21;
    let walk = pose_after(&graph(BASE), ticks);
    let spin_all = pose_after(&graph("layer base {\n  state spin spin\n}\n"), ticks);
    // The arm's branch fully, the spine at half (its head and arm under it
    // are named again or inherit), nothing else.
    let masked = pose_after(
        &graph(&format!(
            "{BASE}layer upper {{\n  mask \"spine\" 0.5\n  mask \"arm.L\"\n  state spin spin\n}}\n"
        )),
        ticks,
    );
    let rest = skeleton().rest_pose();
    // Outside the mask the walk is untouched, bit for bit.
    assert_eq!(masked[HIPS], walk[HIPS]);
    assert_eq!(masked[THIGH_L], walk[THIGH_L]);
    assert_ne!(walk[THIGH_L], rest[THIGH_L]);
    // The arm's branch is the spin's, the forearm under it too.
    assert_eq!(masked[ARM_L], spin_all[ARM_L]);
    assert_eq!(masked[FOREARM_L], spin_all[FOREARM_L]);
    // The spine and the head under it are halfway from the walk to the spin.
    for n in [SPINE, HEAD] {
        let half = walk[n].lerp(&spin_all[n], 0.5);
        assert!(
            (masked[n].rotation.dot(half.rotation).abs() - 1.0).abs() < 1e-6,
            "node {n}"
        );
    }
    assert_ne!(masked[SPINE].rotation, walk[SPINE].rotation);
    assert_ne!(masked[SPINE].rotation, spin_all[SPINE].rotation);

    // An additive layer's mask: only the arm gets the spin's difference.
    let additive = pose_after(
        &graph(&format!(
            "{BASE}layer upper {{\n  mode additive\n  mask \"arm.L\"\n  state spin spin\n}}\n"
        )),
        ticks,
    );
    assert_eq!(additive[SPINE], walk[SPINE]);
    assert_ne!(additive[ARM_L].rotation, walk[ARM_L].rotation);

    // Mistakes name their line.
    let e = GraphAsset::parse(
        &graph(&format!(
            "{BASE}layer upper {{\n  mask \"arm.L\" 2\n  state spin spin\n}}\n"
        )),
        &resolve,
    )
    .unwrap_err();
    assert!(e.reason().contains("line 13"), "{e}");
}

/// The thigh's rotation of `clip` at clip time `t`.
fn thigh_at(clip: u64, t: f32) -> Quat {
    let s = skeleton();
    let c = clips().into_iter().find(|c| c.0 == clip).unwrap().1;
    let mut p = Pose::rest(&s);
    sample(&s, &c, &bind(&s, &c), t, NONE, &mut p);
    p.nodes[THIGH_L].rotation
}

fn near(a: Quat, b: Quat) -> bool {
    (a.dot(b).abs() - 1.0).abs() < 1e-6
}

#[test]
fn a_synced_layer_plays_at_its_leaders_phase() {
    // The base walks (1.0 s); the upper layer overrides with the run
    // (0.6 s). Synced, the run is sampled at the walk's phase.
    let ticks = 40;
    let phase = ticks as f32 / 60.0; // the walk's phase after 40 ticks (< 1)
    let layers = |sync: &str| {
        graph(&format!(
            "layer base {{\n  state walk walk {{\n    sync {sync}\n  }}\n}}\nlayer upper {{\n  state run run {{\n    sync gait\n  }}\n}}\n"
        ))
    };
    let synced = pose_after(&layers("gait"), ticks);
    assert!(
        near(synced[THIGH_L].rotation, thigh_at(RUN, phase * 0.6)),
        "the run at the walk's phase"
    );
    // In another group each keeps its own phase.
    let free = pose_after(&layers("other"), ticks);
    let own = (ticks as f32 / 60.0 / 0.6).fract() * 0.6;
    assert!(near(free[THIGH_L].rotation, thigh_at(RUN, own)));
    assert!(!near(free[THIGH_L].rotation, synced[THIGH_L].rotation));
}

#[test]
fn a_transition_inside_a_group_keeps_the_phase() {
    let text = graph(
        "layer base {\n  state walk walk {\n    sync gait\n  }\n  state run run {\n    sync gait\n  }\n  transition walk run {\n    when p gt 0.5\n    fade 0\n  }\n}\n",
    );
    let (mut w, anim, e) = world(&text);
    for _ in 0..30 {
        w.step().unwrap();
    }
    let mut g: AnimationGraph = w.get(e).unwrap();
    g.p0 = 1.0;
    w.insert(e, g).unwrap();
    w.step().unwrap();
    // The walk was at phase 0.5; the run continues from it, one tick on.
    let phase = 0.5 + (1.0 / 60.0) / 0.6;
    let pose = anim.pose(e).unwrap();
    assert!(
        near(pose[THIGH_L].rotation, thigh_at(RUN, phase * 0.6)),
        "the run from the walk's phase"
    );
    assert!(!near(pose[THIGH_L].rotation, thigh_at(RUN, 1.0 / 60.0)));
}
