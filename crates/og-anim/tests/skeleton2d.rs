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

fn rig() -> Rig2dAsset {
    Rig2dAsset {
        texture: "leg.png".into(),
        size: [16, 16],
        unit: 10.0,
        filter: SheetFilter::Linear,
        bones: [
            ("hip", None, [0.0, 0.0], 0.0, 0.0),
            ("thigh", Some(0), [0.0, 0.0], -90.0, 10.0),
            ("shin", Some(1), [10.0, 0.0], 0.0, 10.0),
            ("goal", Some(0), [5.0, -15.0], 0.0, 0.0),
        ]
        .into_iter()
        .map(|(name, parent, [x, y], rotation, length)| Bone2d {
            name: name.into(),
            parent,
            x,
            y,
            rotation,
            length,
            skew: 0.0,
            scale: [1.0; 2],
        })
        .collect(),
        slots: vec![Slot2d {
            name: "leg".into(),
            bone: 1,
            attachment: Some(0),
            color: [1.0; 4],
            offset: [0.0; 4],
        }],
        attachments: vec![Attachment2d {
            name: "leg".into(),
            slot: 0,
            kind: AttachmentKind::Region {
                rect: [0.0, 0.0, 16.0, 16.0],
                at: [0.0; 2],
                rotation: 0.0,
                skew: 0.0,
                size: [16.0; 2],
            },
        }],
        iks: vec![Ik2d {
            name: "foot".into(),
            bones: vec![1, 2],
            target: 3,
            counterclockwise: true,
            weight: 1.0,
        }],
        clips: [
            (
                "walk",
                0.5,
                true,
                vec![(0.0, "start".into()), (0.25, "step".into())],
            ),
            ("once", 0.1, false, vec![(0.1, "end".into())]),
        ]
        .into_iter()
        .map(|(name, duration, looping, events)| Clip2d {
            name: name.into(),
            duration,
            looping,
            events,
            bones: vec![],
            slots: vec![],
            deforms: vec![],
            order: vec![],
            iks: vec![],
        })
        .collect(),
        transforms: vec![],
        paths: vec![],
        skins: vec![],
        armatures: vec![],
    }
}

fn world() -> (World, Entity, Entity) {
    let mut w = World::new(&WorldDesc {
        tick_rate: 64,
        seed: 1,
    })
    .unwrap();
    let sys = install_skeleton2d(&mut w).unwrap();
    let rig = rig();
    let bytes = rig.to_bytes().unwrap();
    let decoded = Rig2dAsset::from_bytes(&bytes).unwrap();
    assert_eq!(decoded, rig);
    for end in 0..bytes.len() {
        assert!(Rig2dAsset::from_bytes(&bytes[..end]).is_err());
    }
    let mut trailing = bytes;
    trailing.push(0);
    assert!(Rig2dAsset::from_bytes(&trailing).is_err());
    sys.add_rig(9, decoded).unwrap();
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
