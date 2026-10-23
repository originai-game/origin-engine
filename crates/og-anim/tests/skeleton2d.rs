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
