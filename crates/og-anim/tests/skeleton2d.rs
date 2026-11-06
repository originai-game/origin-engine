//! 2D skeletons: the og.skeleton2d system, tick by tick.
//!
//! Ways this could fail: time wraps at the wrong length or a one-shot
//! wraps; an event fires twice in one pass, never, at the wrong wrap, or
//! across a seek; a new clip continues from the old time; an IK target is
//! read in the rig's pixels instead of the entity's frame (or ignores
//! flipX and the Transform); a snapshot replays events.

use og_anim::*;
use og_core::glam::{Vec2, Vec3};
use og_ecs::{Entity, World, WorldDesc};

const RIG: &str = r#"type rig2d
version 1
---
texture "leg.png"
size 16 16
unit 10
bone hip {
}
bone thigh {
  parent hip
  rotation -90
  length 10
}
bone shin {
  parent thigh
  at 10 0
  length 10
}
bone goal {
  parent hip
  at 5 -15
}
slot leg {
  bone thigh
  attachment leg
}
region leg {
  slot leg
  rect 0 0 16 16
}
ik foot {
  bones thigh shin
  target goal
  bend counterclockwise
}
clip walk {
  duration 0.5
  event 0 "start"
  event 0.25 "step"
}
clip once {
  duration 0.1
  loop false
  event 0.1 "end"
}
"#;

fn world() -> (World, Entity, Entity) {
    let mut w = World::new(&WorldDesc {
        tick_rate: 64,
        seed: 1,
    })
    .unwrap();
    let sys = install_skeleton2d(&mut w).unwrap();
    sys.add_rig(9, Rig2dAsset::parse(RIG).unwrap()).unwrap();
    let e = w
        .spawn((
            Transform {
                position: Vec3::new(3.0, 1.0, 0.0),
                ..Default::default()
            },
            Skeleton2D {
                rig: 9,
                ..Default::default()
            },
        ))
        .unwrap();
    let t = w
        .spawn((IkTarget2D {
            skeleton: e.to_bits(),
            // 1.5 world units below and 1 left of the hip: the rig's (-10, -15) px.
            target: Vec2::new(2.0, -0.5),
            ..Default::default()
        },))
        .unwrap();
    (w, e, t)
}

fn events(w: &World) -> Vec<String> {
    w.pending_events::<Skeleton2DEvent>()
        .unwrap()
        .iter()
        .map(|e| e.name.as_str().to_string())
        .collect()
}

#[test]
fn a_restored_snapshot_continues_without_replaying_events() {
    let (mut w, _, _) = world();
    for _ in 0..15 {
        w.step().unwrap();
    }
    let snap = w.snapshot();
    w.step().unwrap();
    let first = events(&w);
    assert_eq!(first, ["step"]);
    w.restore(&snap).unwrap();
    w.step().unwrap();
    assert_eq!(events(&w), first);
}

#[test]
fn bounding_boxes_follow_the_pose_in_the_entitys_frame_and_restore_with_snapshots() {
    // The leg's IK off (weight 0), so the thigh hangs as set up.
    let text = RIG.replace("bend counterclockwise", "bend counterclockwise\n  weight 0").replace(
        "clip walk {",
        "slot hurt {\n  bone thigh\n  attachment hurt\n}\nbox hurt {\n  slot hurt\n  polygon 0 -2 10 -2 10 2\n}\nclip walk {",
    );
    let rig = Rig2dAsset::parse(&text).unwrap();
    let mut w = World::new(&WorldDesc {
        tick_rate: 64,
        seed: 1,
    })
    .unwrap();
    let sys = install_skeleton2d(&mut w).unwrap();
    sys.add_rig(9, rig.clone()).unwrap();
    let place = Transform {
        position: Vec3::new(3.0, 1.0, 0.0),
        ..Default::default()
    };
    let e = w
        .spawn((
            place,
            Skeleton2D {
                rig: 9,
                ..Default::default()
            },
        ))
        .unwrap();
    assert_eq!(
        sys.boxes(e.to_bits()),
        None,
        "not posed before the first tick"
    );
    w.step().unwrap();
    let b = sys.boxes(e.to_bits()).unwrap();
    assert_eq!(
        (b.len(), b[0].slot.as_str(), b[0].name.as_str()),
        (1, "hurt", "hurt")
    );
    let near = |p: Vec2, x: f32, y: f32| (p.x - x).abs() < 1e-5 && (p.y - y).abs() < 1e-5;
    // The thigh's x axis points down (-90): (10, -2) in it is (-2, -10) px
    // from the hip, (-0.2, -1) units.
    assert!(near(b[0].points[1], 2.8, 0.0), "{b:?}");
    // Flipped about the entity's x.
    let s = w.get::<Skeleton2D>(e).unwrap();
    w.insert(e, Skeleton2D { flip_x: true, ..s }).unwrap();
    w.step().unwrap();
    let flipped = sys.boxes(e.to_bits()).unwrap();
    assert!(near(flipped[0].points[1], 3.2, 0.0), "{flipped:?}");
    // The boxes are state-block state: a restore answers what was saved.
    let snap = w.snapshot();
    w.insert(e, Skeleton2D { flip_x: false, ..s }).unwrap();
    w.step().unwrap();
    assert!(near(sys.boxes(e.to_bits()).unwrap()[0].points[1], 2.8, 0.0));
    w.restore(&snap).unwrap();
    assert_eq!(sys.boxes(e.to_bits()).unwrap(), flipped);
}

#[test]
fn an_ik_target_on_a_chain_of_three_bones_is_reached_and_its_error_is_the_tips_distance() {
    // A tail of three 10-pixel bones (unit 10: one world unit each).
    let tail = r#"type rig2d
version 1
---
texture "tail.png"
size 8 8
unit 10
bone root {
}
bone t1 {
  parent root
  length 10
}
bone t2 {
  parent t1
  at 10 0
  length 10
}
bone t3 {
  parent t2
  at 10 0
  length 10
}
ik tail {
  bones t1 t2 t3
  target root
  bend clockwise
}
clip still {
  duration 1
}
"#;
    let mut w = World::new(&WorldDesc {
        tick_rate: 64,
        seed: 1,
    })
    .unwrap();
    let sys = install_skeleton2d(&mut w).unwrap();
    sys.add_rig(3, Rig2dAsset::parse(tail).unwrap()).unwrap();
    let e = w
        .spawn((
            Transform {
                position: Vec3::new(1.0, 2.0, 0.0),
                ..Default::default()
            },
            Skeleton2D {
                rig: 3,
                ..Default::default()
            },
        ))
        .unwrap();
    let t = w
        .spawn((IkTarget2D {
            skeleton: e.to_bits(),
            target: Vec2::new(2.5, 3.5),
            ..Default::default()
        },))
        .unwrap();
    w.step().unwrap();
    let ik = w.get::<IkTarget2D>(t).unwrap();
    assert!(ik.reached && ik.error < 1e-4, "{ik:?}");
    // Out of reach (3 units of tail, 5 away): pointed at it, 2 short.
    w.insert(
        t,
        IkTarget2D {
            target: Vec2::new(4.0, 6.0),
            ..ik
        },
    )
    .unwrap();
    w.step().unwrap();
    let ik = w.get::<IkTarget2D>(t).unwrap();
    assert!(!ik.reached && (ik.error - 2.0).abs() < 1e-4, "{ik:?}");
}

const ARM: &str = r#"type rig2d
version 1
---
texture "arm.png"
size 8 8
unit 10
bone arm {
  length 10
}
clip low {
  duration 1
  bone arm {
    rotate 0 0
  }
  event 0.5 "tick"
}
clip high {
  duration 1
  bone arm {
    rotate 0 90
  }
}
clip slam {
  duration 0.5
  loop false
  bone arm {
    rotate 0 -45
  }
  event 0.1 "bang"
}
"#;

const ARM_GRAPH: &str = r#"type animgraph
version 1
---
param lift
clip low "low"
clip high "high"
clip hit "slam" {
  loop false
}
blend1d pose lift {
  child low 0
  child high 1
}
layer base {
  state hold pose
  state slam hit
  entry hold
  transition hold slam {
    when lift gt 1.5
    fade 0.25
  }
}
"#;

/// The arm bone's world angle in degrees, as og.skeleton2d's pose draws it.
fn arm_angle(w: &World, sys: &Skeleton2dSystem, e: Entity) -> f32 {
    let s = w.get::<Skeleton2D>(e).unwrap();
    let rig = sys.rig(9).unwrap();
    let frame = skeleton_frame(&rig, &Transform::default(), false);
    let m = skeleton_pose(&rig, &s, &frame, &[]).world[0];
    og_core::math::atan2(m.b, m.a).to_degrees()
}

/// An animation graph drives a 2D skeleton by clip names: a 1D blend is
/// the two clips around its parameter blended by weight, and a
/// transition cross-fades over its `fade` from the state it leaves, its
/// events firing; a restored world continues the same. Ways this could
/// fail: the heavier clip alone is drawn (no blend), the blend weight is
/// the lighter's complement, the turn blends the long way, the fade cuts,
/// the slam's event is lost, the machine is not in the snapshot.
#[test]
fn a_graph_blends_and_cross_fades_a_2d_skeleton() {
    let mut w = World::new(&WorldDesc {
        tick_rate: 64,
        seed: 1,
    })
    .unwrap();
    let sys = install_skeleton2d(&mut w).unwrap();
    sys.add_rig(9, Rig2dAsset::parse(ARM).unwrap()).unwrap();
    sys.add_graph(
        4,
        std::sync::Arc::new(SpriteGraph::parse(ARM_GRAPH).unwrap()),
    )
    .unwrap();
    let mut g = AnimationGraph::new(4);
    g.p0 = 0.25;
    let e = w
        .spawn((
            Transform::default(),
            Skeleton2D {
                rig: 9,
                ..Default::default()
            },
            g,
        ))
        .unwrap();
    for _ in 0..4 {
        w.step().unwrap();
    }
    let s = w.get::<Skeleton2D>(e).unwrap();
    assert_eq!((s.clip, s.blend_clip), (0, 1), "low, high over it");
    assert!((s.blend - 0.25).abs() < 1e-6, "{}", s.blend);
    assert!(
        (arm_angle(&w, &sys, e) - 22.5).abs() < 1e-3,
        "{}",
        arm_angle(&w, &sys, e)
    );
    // Lift past the threshold: hold (all high, +90) fades into slam (-45).
    let mut g = w.get::<AnimationGraph>(e).unwrap();
    g.p0 = 2.0;
    w.insert(e, g).unwrap();
    let snap = w.snapshot();
    let mut seen = Vec::new();
    let mut angles = Vec::new();
    for _ in 0..20 {
        w.step().unwrap();
        seen.extend(events(&w));
        angles.push(arm_angle(&w, &sys, e));
    }
    // After 8 ticks of 1/64 s the 0.25 s fade is half way: -45 + 135/2.
    assert!((angles[7] - 22.5).abs() < 1e-2, "{angles:?}");
    assert!((angles[3] - 56.25).abs() < 1e-2, "a quarter in: {angles:?}");
    assert!((angles[19] + 45.0).abs() < 1e-3, "slam alone: {angles:?}");
    let s = w.get::<Skeleton2D>(e).unwrap();
    assert_eq!((s.clip, s.blend), (2, 0.0));
    assert_eq!(w.get::<AnimationGraph>(e).unwrap().state, 1);
    assert_eq!(seen, ["bang"], "the slam's event at 0.1 s");
    w.restore(&snap).unwrap();
    let mut again = Vec::new();
    for _ in 0..20 {
        w.step().unwrap();
        again.push(arm_angle(&w, &sys, e));
    }
    assert_eq!(again, angles);
}
