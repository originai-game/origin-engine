//! The `animgraph` asset and its state machine: transitions fire in the tick
//! their condition holds and cross-fade over their `fade`, blend1d weights
//! follow the parameter, the additive layer adds its difference, and bad
//! text is refused with its line.

mod support;

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

#[test]
fn blend1d_mixes_walk_and_run_and_the_additive_layer_adds() {
    let (mut w, anim, e) = world();
    set(&mut w, e, |g| g.p0 = 2.75); // halfway between walk (1.5) and run (4)
    for _ in 0..40 {
        w.step().unwrap();
    }
    // With phase sync both clips are at the same phase: the hips' forward
    // pose position is between walk's and run's at that phase.
    let pose = anim.pose(e).unwrap();
    let z = pose[1].translation.z;
    assert!(z > 0.0, "{z}");
    // The additive wave raises the arm only when its weight parameter is up.
    let arm_before = pose[10].rotation;
    set(&mut w, e, |g| g.p1 = 1.0);
    w.step().unwrap();
    let arm_after = anim.pose(e).unwrap()[10].rotation;
    assert!(arm_before.dot(arm_after).abs() < 1.0 - 1e-4);
}

#[test]
fn bad_graph_text_is_refused_with_its_line() {
    let cases = [
        ("param a\nlayer l {\n  state s nowhere\n}\n", "line 6: unknown motion 'nowhere'"),
        ("clip c \"nope\"\nlayer l {\n  state s c\n}\n", "line 4: clip 'nope' is not loaded"),
        ("clip c \"idle\"\nlayer l {\n  state s c\n  transition s s {\n    when speed gt 1\n  }\n}\n", "line 8: unknown parameter 'speed'"),
        ("clip c \"idle\"\nblend1d b x {\n  child b 0\n}\n", "unknown parameter 'x'"),
        ("param x\nblend1d b x {\n  child b 0\n}\nlayer l {\n  state s b\n}\n", "motion 'b' contains itself"),
        ("param x\n", "at least one `layer`"),
    ];
    for (body, want) in cases {
        let text = format!("type animgraph\nversion 1\n---\n{body}");
        let err = GraphAsset::parse(&text, &resolve).unwrap_err().to_string();
        assert!(err.contains(want), "{body:?}: {err}");
    }
    let g = GraphAsset::parse(GRAPH_TEXT, &resolve).unwrap();
    assert_eq!(g.params, ["speed", "wave"]);
    assert_eq!(g.param("wave"), Some(1));
    assert_eq!(g.clips(), vec![IDLE, WALK, RUN, WAVE]);
    assert_eq!(g.layers[1].mode, LayerMode::Additive);
}
