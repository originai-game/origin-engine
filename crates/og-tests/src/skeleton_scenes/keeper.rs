//! Native reference-scene data; no author parser is needed to draw a frame.
use og_anim::*;

pub(super) fn rig() -> Rig2dAsset {
    let bones = [
        ("root", None, [0.0, 0.0], 0.0, 0.0),
        ("hip", Some(0), [0.0, 22.0], 90.0, 16.0),
        ("head", Some(1), [16.0, 0.0], -90.0, 0.0),
        ("arm", Some(1), [14.0, -5.0], 180.0, 13.0),
        ("thigh-b", Some(1), [0.0, 2.0], 180.0, 11.0),
        ("shin-b", Some(4), [11.0, 0.0], 0.0, 11.0),
        ("thigh-f", Some(1), [0.0, -2.0], 180.0, 11.0),
        ("shin-f", Some(6), [11.0, 0.0], 0.0, 11.0),
        ("cape-1", Some(1), [15.0, 4.0], 180.0, 12.0),
        ("cape-2", Some(8), [12.0, 0.0], 0.0, 12.0),
        ("foot-b", Some(0), [-3.0, 0.0], 0.0, 0.0),
        ("foot-f", Some(0), [3.0, 0.0], 0.0, 0.0),
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
    .collect();
    let slots = [
        ("cape", 1),
        ("thigh-b", 4),
        ("shin-b", 5),
        ("boot-b", 5),
        ("torso", 1),
        ("thigh-f", 6),
        ("shin-f", 7),
        ("boot-f", 7),
        ("head", 2),
        ("arm", 3),
        ("lantern", 3),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (name, bone))| Slot2d {
        name: name.into(),
        bone,
        attachment: Some(i),
        color: if (1..=3).contains(&i) {
            [0.72, 0.72, 0.8, 1.0]
        } else {
            [1.0; 4]
        },
        offset: [0.0; 4],
    })
    .collect();
    let mut vertices = Vec::new();
    for row in 0..4 {
        for col in 0..3 {
            let c = col as f32;
            let influences = match row {
                0 => vec![Influence {
                    bone: 1,
                    x: 16.0,
                    y: 16.0 - c * 7.0,
                    weight: 1.0,
                }],
                1 => vec![
                    Influence {
                        bone: 1,
                        x: 7.0,
                        y: 16.0 - c * 7.0,
                        weight: 0.4,
                    },
                    Influence {
                        bone: 8,
                        x: 8.0,
                        y: -12.0 + c * 7.0,
                        weight: 0.6,
                    },
                ],
                2 => vec![
                    Influence {
                        bone: 8,
                        x: 17.0,
                        y: -12.0 + c * 7.0,
                        weight: 0.5,
                    },
                    Influence {
                        bone: 9,
                        x: 5.0,
                        y: -12.0 + c * 7.0,
                        weight: 0.5,
                    },
                ],
                _ => vec![Influence {
                    bone: 9,
                    x: 14.0,
                    y: -12.0 + c * 7.0,
                    weight: 1.0,
                }],
            };
            vertices.push(MeshVertex2d {
                uv: [c * 6.0, 32.0 + row as f32 * 8.0],
                influences,
            });
        }
    }
    let mut attachments = vec![Attachment2d {
        name: "cape".into(),
        slot: 0,
        kind: AttachmentKind::Mesh {
            vertices,
            triangles: vec![
                0, 3, 1, 1, 3, 4, 1, 4, 2, 2, 4, 5, 3, 6, 4, 4, 6, 7, 4, 7, 5, 5, 7, 8, 6, 9, 7, 7,
                9, 10, 7, 10, 8, 8, 10, 11,
            ],
        },
    }];
    for (slot, name, rect, at, rotation) in [
        (1, "thigh-b", [32.0, 0.0, 5.0, 12.0], [5.5, 0.0], 90.0),
        (2, "shin-b", [40.0, 0.0, 5.0, 12.0], [5.5, 0.0], 90.0),
        (3, "boot-b", [48.0, 0.0, 10.0, 5.0], [10.5, 2.0], 90.0),
        (4, "torso", [0.0, 0.0, 12.0, 20.0], [8.0, 0.0], -90.0),
        (5, "thigh-f", [32.0, 0.0, 5.0, 12.0], [5.5, 0.0], 90.0),
        (6, "shin-f", [40.0, 0.0, 5.0, 12.0], [5.5, 0.0], 90.0),
        (7, "boot-f", [48.0, 0.0, 10.0, 5.0], [10.5, 2.0], 90.0),
        (8, "head", [15.0, 0.0, 14.0, 12.0], [0.0, 5.0], 0.0),
        (9, "arm", [32.0, 16.0, 4.0, 14.0], [6.5, 0.0], 90.0),
        (10, "lantern", [52.0, 16.0, 8.0, 10.0], [15.0, 0.0], 90.0),
    ] {
        attachments.push(Attachment2d {
            name: name.into(),
            slot,
            kind: AttachmentKind::Region {
                rect,
                at,
                rotation,
                skew: 0.0,
                size: [rect[2], rect[3]],
            },
        });
    }
    let mut idle = clip("idle", 2.0);
    idle.bones.push(BoneTimeline {
        bone: 1,
        translate: vec![
            key(0.0, [0.0, 0.0]),
            key(1.0, [0.0, -1.5]),
            key(2.0, [0.0, 0.0]),
        ],
        ..timeline(1)
    });
    for (bone, values) in [
        (2, [0.0, 4.0, 0.0]),
        (3, [4.0, -8.0, 4.0]),
        (8, [-4.0, 8.0, -4.0]),
        (9, [-6.0, 12.0, -6.0]),
    ] {
        let mut t = timeline(bone);
        t.rotate = values
            .into_iter()
            .enumerate()
            .map(|(i, v)| {
                key(
                    i as f32,
                    Rotation {
                        degrees: v,
                        turn: Turn::Shorter,
                    },
                )
            })
            .collect();
        idle.bones.push(t);
    }
    idle.deforms.push(DeformTimeline {
        attachment: 0,
        keys: [
            (0.0, [0.0; 4]),
            (0.5, [0.0, -1.0, -3.0, -5.0]),
            (1.0, [0.0, 1.0, 2.0, 3.0]),
            (1.5, [0.0, -1.0, -3.5, -6.0]),
            (2.0, [0.0; 4]),
        ]
        .into_iter()
        .map(|(time, ys)| {
            let mut value = Vec::new();
            for (row, y) in ys.into_iter().enumerate() {
                for _ in 0..if row == 1 || row == 2 { 6 } else { 3 } {
                    value.extend([0.0, y]);
                }
            }
            key(time, value)
        })
        .collect(),
    });
    idle.events.push((0.0, "inhale".into()));
    let mut walk = clip("walk", 0.8);
    let mut hip = timeline(1);
    hip.translate = [
        (0.0, -1.0),
        (0.2, 0.6),
        (0.4, -1.0),
        (0.6, 0.6),
        (0.8, -1.0),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (t, y))| {
        let mut k = key(t, [0.0, y]);
        if i < 4 {
            k.ease = Ease::Bezier(if i % 2 == 0 {
                vec![0.3, 0.0, 0.6, 1.0]
            } else {
                vec![0.4, 0.0, 0.7, 1.0]
            });
        }
        k
    })
    .collect();
    walk.bones.push(hip);
    for (bone, times, values) in [
        (2, vec![0.0, 0.4, 0.8], vec![-2.0, 2.0, -2.0]),
        (3, vec![0.0, 0.4, 0.8], vec![16.0, -12.0, 16.0]),
        (
            8,
            vec![0.0, 0.2, 0.4, 0.6, 0.8],
            vec![-10.0, -14.0, -10.0, -14.0, -10.0],
        ),
        (
            9,
            vec![0.0, 0.3, 0.5, 0.7, 0.8],
            vec![-12.0, -20.0, -12.0, -20.0, -12.0],
        ),
    ] {
        let mut t = timeline(bone);
        let end = times.len() - 1;
        t.rotate = times
            .into_iter()
            .zip(values)
            .enumerate()
            .map(|(i, (t, v))| {
                let mut k = key(
                    t,
                    Rotation {
                        degrees: v,
                        turn: Turn::Shorter,
                    },
                );
                if bone == 3 && i < end {
                    k.ease = Ease::Bezier(vec![0.42, 0.0, 0.58, 1.0]);
                }
                k
            })
            .collect();
        walk.bones.push(t);
    }
    walk.events = vec![(0.0, "step".into()), (0.4, "step".into())];
    Rig2dAsset {
        texture: "keeper.png".into(),
        size: [64, 64],
        filter: SheetFilter::Nearest,
        unit: 20.0,
        bones,
        slots,
        attachments,
        clips: vec![idle, walk],
        iks: [("leg-b", vec![4, 5], 10), ("leg-f", vec![6, 7], 11)]
            .into_iter()
            .map(|(name, bones, target)| Ik2d {
                name: name.into(),
                bones,
                target,
                counterclockwise: false,
                weight: 1.0,
            })
            .collect(),
        transforms: vec![],
        paths: vec![],
        skins: vec![],
        armatures: vec![],
    }
}

fn key<T>(time: f32, value: T) -> Key<T> {
    Key {
        time,
        value,
        ease: Ease::Linear,
    }
}
fn timeline(bone: usize) -> BoneTimeline {
    BoneTimeline {
        bone,
        rotate: vec![],
        skew: vec![],
        translate: vec![],
        scale: vec![],
    }
}
fn clip(name: &str, duration: f32) -> Clip2d {
    Clip2d {
        name: name.into(),
        duration,
        looping: true,
        bones: vec![],
        slots: vec![],
        deforms: vec![],
        order: vec![],
        iks: vec![],
        events: vec![],
    }
}
