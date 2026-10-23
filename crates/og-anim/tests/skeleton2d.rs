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
fn a_looping_clip_wraps_and_fires_each_event_once_per_pass() {
    let (mut w, e, _) = world();
    let mut log = Vec::new();
    for tick in 1..=65 {
        w.step().unwrap();
        for n in events(&w) {
            log.push((tick, n));
        }
    }
    // 64 ticks a second (exact in binary), 32 per pass: "start" at 0 on
    // the first tick and on every wrap, "step" at 0.25 s (tick 16 of each).
    let names: Vec<_> = log.iter().map(|(t, n)| format!("{n}@{t}")).collect();
    assert_eq!(
        names,
        ["start@1", "step@16", "start@32", "step@48", "start@64"]
    );
    let s = w.get::<Skeleton2D>(e).unwrap();
    assert_eq!(s.time, 1.0 / 64.0);
    assert!(!s.finished);
}

#[test]
fn a_one_shot_holds_its_end_and_a_clip_change_restarts_at_zero() {
    let (mut w, e, _) = world();
    for _ in 0..10 {
        w.step().unwrap();
    }
    w.insert(
        e,
        Skeleton2D {
            clip: 1,
            looping: false,
            ..w.get::<Skeleton2D>(e).unwrap()
        },
    )
    .unwrap();
    let mut ended = 0;
    for _ in 0..12 {
        w.step().unwrap();
        ended += events(&w).iter().filter(|n| *n == "end").count();
    }
    let s = w.get::<Skeleton2D>(e).unwrap();
    assert_eq!((s.time, s.finished, ended), (0.1, true, 1));
    // A seek fires nothing; the clip runs again from there.
    w.insert(e, Skeleton2D { time: 0.05, ..s }).unwrap();
    w.step().unwrap();
    assert!(events(&w).is_empty());
    assert!(!w.get::<Skeleton2D>(e).unwrap().finished);
}

#[test]
fn an_ik_target_in_the_entitys_frame_is_reached_and_flip_mirrors_it() {
    let (mut w, e, t) = world();
    w.step().unwrap();
    let ik = w.get::<IkTarget2D>(t).unwrap();
    assert!(ik.reached && ik.error < 1e-3, "{ik:?}");
    // The foot's tip is at the target, in world units.
    let rig = Rig2dAsset::parse(RIG).unwrap();
    let s = w.get::<Skeleton2D>(e).unwrap();
    let frame = skeleton_frame(&rig, &w.get::<Transform>(e).unwrap(), false);
    let pose = skeleton_pose(&rig, &s, &frame, &[(0, Vec2::new(2.0, -0.5), 1.0)]);
    let tip = frame.apply_to(pose.world[2].apply(10.0, 0.0));
    assert!(
        (tip[0] - 2.0).abs() < 1e-3 && (tip[1] + 0.5).abs() < 1e-3,
        "{tip:?}"
    );
    // Out of reach (2 units of leg): not reached, the error in world units.
    w.insert(
        t,
        IkTarget2D {
            target: Vec2::new(3.0, -3.0),
            ..ik
        },
    )
    .unwrap();
    w.step().unwrap();
    let ik = w.get::<IkTarget2D>(t).unwrap();
    assert!(!ik.reached && (ik.error - 2.0).abs() < 1e-3, "{ik:?}");
    // Flipped, a target to the right of the hip is the rig's left.
    w.insert(e, Skeleton2D { flip_x: true, ..s }).unwrap();
    w.insert(
        t,
        IkTarget2D {
            target: Vec2::new(4.0, -0.5),
            ..ik
        },
    )
    .unwrap();
    w.step().unwrap();
    assert!(w.get::<IkTarget2D>(t).unwrap().reached);
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

/// og.Skeleton2D.skin picks what its slots show: here a bigger hurt box.
#[test]
fn the_skin_a_skeleton_wears_is_what_its_pose_shows() {
    let rig = r#"type rig2d
version 1
---
texture "t.png"
size 16 16
unit 10
bone root {
}
slot hurt {
  bone root
  attachment small
}
box small {
  slot hurt
  polygon 0 0 10 0 10 10
}
box big {
  slot hurt
  polygon 0 0 20 0 20 20
}
skin giant {
  swap small big
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
    sys.add_rig(4, Rig2dAsset::parse(rig).unwrap()).unwrap();
    let e = w
        .spawn((
            Transform::default(),
            Skeleton2D {
                rig: 4,
                ..Default::default()
            },
        ))
        .unwrap();
    w.step().unwrap();
    let shown = |sys: &Skeleton2dSystem| sys.boxes(e.to_bits()).unwrap()[0].clone();
    // 10 pixels to a unit: the default's box reaches (1, 1).
    assert_eq!(
        (shown(&sys).name.as_str(), shown(&sys).points[2]),
        ("small", Vec2::new(1.0, 1.0))
    );
    let s = w.get::<Skeleton2D>(e).unwrap();
    w.insert(e, Skeleton2D { skin: 1, ..s }).unwrap();
    w.step().unwrap();
    assert_eq!(
        (shown(&sys).name.as_str(), shown(&sys).points[2]),
        ("big", Vec2::new(2.0, 2.0))
    );
}
