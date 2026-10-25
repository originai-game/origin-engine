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
        (
            "param a\nlayer l {\n  state s nowhere\n}\n",
            "line 6: unknown motion 'nowhere'",
        ),
        (
            "clip c \"nope\"\nlayer l {\n  state s c\n}\n",
            "line 4: clip 'nope' is not loaded",
        ),
        (
            "clip c \"idle\"\nlayer l {\n  state s c\n  transition s s {\n    when speed gt 1\n  }\n}\n",
            "line 8: unknown parameter 'speed'",
        ),
        (
            "clip c \"idle\"\nblend1d b x {\n  child b 0\n}\n",
            "unknown parameter 'x'",
        ),
        (
            "param x\nblend1d b x {\n  child b 0\n}\nlayer l {\n  state s b\n}\n",
            "motion 'b' contains itself",
        ),
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

// ---- transition interruption -------------------------------------------------

const IG: u64 = 41;

/// idle → walk with `interrupt <mode>` over a 0.5 s (30-tick) fade; then
/// walk → run (the new state's own), idle → jump (the source's, `jump` the
/// wave clip) and any → idle (an `any` one) can each try to interrupt it.
fn interrupt_graph(mode: &str) -> String {
    format!(
        r#"type animgraph
version 1
---
param p
clip idle "idle"
clip walk "walk"
clip run "run"
clip jump "wave"
layer base {{
  state idle idle
  state walk walk
  state run run
  state jump jump
  transition idle walk {{
    when p ge 1
    fade 0.5
    interrupt {mode}
  }}
  transition walk run {{
    when p eq 2
    fade 0.1
  }}
  transition idle jump {{
    when p eq 3
    fade 0.1
  }}
  transition any idle {{
    when p eq 4
    fade 0.1
  }}
}}
"#
    )
}

fn interrupt_world(mode: &str) -> (World, Entity) {
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 1,
    })
    .unwrap();
    let anim = install(&mut w).unwrap();
    register(&anim);
    anim.add_graph(
        IG,
        GraphAsset::parse(&interrupt_graph(mode), &resolve).unwrap(),
    )
    .unwrap();
    let e = w
        .spawn((
            Transform::default(),
            Skeleton {
                asset: SKELETON,
                ..Skeleton::default()
            },
            AnimationGraph::new(IG),
        ))
        .unwrap();
    (w, e)
}

/// Starts idle → walk, then sets `p` to `then` the tick after: the base
/// state for the next 40 ticks.
fn interrupted(mode: &str, then: f32) -> Vec<u32> {
    let (mut w, e) = interrupt_world(mode);
    set(&mut w, e, |g| g.p0 = 1.0);
    w.step().unwrap();
    set(&mut w, e, |g| g.p0 = then);
    (0..40)
        .map(|_| {
            w.step().unwrap();
            w.get::<AnimationGraph>(e).unwrap().state
        })
        .collect()
}

#[test]
fn an_interrupt_rule_says_which_transitions_may_cut_a_fade() {
    let (walk, run, jump, idle) = (1, 2, 3, 0);
    let first = |states: &[u32], s: u32| states.iter().position(|&x| x == s);
    // any (the default): the new state's and `any` transitions cut in at once.
    assert_eq!(interrupted("any", 2.0)[0], run);
    assert_eq!(interrupted("any", 4.0)[0], idle);
    assert_eq!(first(&interrupted("any", 3.0), jump), None);
    // none: the 30-tick fade completes first (about 28 ticks after the
    // one that set p, as the f32 fade time sums), then the run fires.
    let none = interrupted("none", 2.0);
    let at = first(&none, run).expect("the run fires after the fade");
    assert!((28..=30).contains(&at), "{none:?}");
    assert!(none[..at].iter().all(|&s| s == walk), "{none:?}");
    assert!(interrupted("none", 4.0)[..28].iter().all(|&s| s == walk));
    // source: only the state being left (idle) may re-target the fade.
    assert_eq!(interrupted("source", 3.0)[0], jump);
    assert!(interrupted("source", 2.0)[..28].iter().all(|&s| s == walk));
    assert!(interrupted("source", 4.0)[..28].iter().all(|&s| s == walk));
    // next: the new state's own, not `any` ones.
    assert_eq!(interrupted("next", 2.0)[0], run);
    assert!(interrupted("next", 4.0)[..28].iter().all(|&s| s == walk));

    let e = GraphAsset::parse(&interrupt_graph("later"), &resolve).unwrap_err();
    assert!(
        e.reason().contains("line 17") && e.reason().contains("later"),
        "{e}"
    );
}

#[test]
fn a_guarded_fade_survives_a_snapshot() {
    let (mut a, e) = interrupt_world("none");
    set(&mut a, e, |g| g.p0 = 1.0);
    a.step().unwrap();
    set(&mut a, e, |g| g.p0 = 2.0);
    a.step().unwrap();
    let snap = a.snapshot();
    let mut want = Vec::new();
    for _ in 0..40 {
        a.step().unwrap();
        want.push((a.world_hash(), a.get::<AnimationGraph>(e).unwrap().state));
    }
    let (mut b, eb) = interrupt_world("none");
    b.restore(&snap).unwrap();
    for (h, s) in want {
        b.step().unwrap();
        assert_eq!(
            (b.world_hash(), b.get::<AnimationGraph>(eb).unwrap().state),
            (h, s)
        );
    }
}
