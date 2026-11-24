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
