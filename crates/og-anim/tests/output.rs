//! What the renderer receives: the joint palette (model matrix × inverse
//! bind, identity at the bind pose, moving with the clip) and blendshape
//! weights that write through from the component or come from the clips,
//! both handed to a [`PaletteSink`]; and the refusals for missing assets.

use crate::support;

use og_anim::*;
use og_core::glam::{Mat4, Quat, Vec3};
use og_ecs::transform::Parent;
use og_ecs::{Entity, World, WorldDesc};
use support::*;

fn world() -> (World, AnimationSystem) {
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 1,
    })
    .unwrap();
    let anim = install(&mut w).unwrap();
    register(&anim);
    (w, anim)
}

fn skel() -> Skeleton {
    Skeleton {
        asset: SKELETON,
        ..Skeleton::default()
    }
}

#[test]
fn a_lantern_follows_the_posed_hand_and_a_non_joint_socket() {
    let (mut w, anim) = world();
    w.register::<Parent>().unwrap();
    w.register::<BoneAttachment>().unwrap();
    let mut asset = skeleton();
    asset.nodes.push(SkeletonNode {
        name: "socket".into(),
        parent: 12,
        rest: Trs::new(Vec3::new(0.2, 0.1, -0.3), Quat::IDENTITY, Vec3::ONE),
        mesh: NONE,
    });
    anim.add_skeleton(SKELETON, asset).unwrap();
    anim.add_clip(
        100,
        clip(
            "grip",
            1.0,
            &[],
            vec![(
                12,
                "hand.L",
                ChannelPath::Translation,
                3,
                Box::new(|_| vec![0.4, 0.2, -0.1]),
            )],
        ),
    )
    .unwrap();
    let rig = w
        .spawn((
            Transform {
                position: Vec3::new(4.0, 5.0, 6.0),
                quaternion: og_core::math::quat_from_axis_angle(
                    Vec3::Z,
                    std::f32::consts::FRAC_PI_2,
                ),
                scale: Vec3::new(2.0, 3.0, 4.0),
            },
            skel(),
            AnimationPlayer::new(100),
        ))
        .unwrap();
    let offset = Transform::at(Vec3::new(0.1, -0.2, 0.3));
    let lantern = w
        .spawn((
            offset,
            Parent {
                entity: rig.to_bits(),
            },
            BoneAttachment { node: 12 },
        ))
        .unwrap();
    let socket = w
        .spawn((
            Transform::default(),
            Parent {
                entity: rig.to_bits(),
            },
            BoneAttachment { node: 13 },
        ))
        .unwrap();
    let child = w
        .spawn((
            Transform::at(Vec3::new(0.2, 0.3, -0.4)),
            Parent {
                entity: lantern.to_bits(),
            },
        ))
        .unwrap();
    let place = |w: &World, e| w.matrix_world(e).unwrap().transform_point3(Vec3::ZERO);
    // No tick yet: an attachment uses the loaded rig's rest frame.
    assert!((place(&w, lantern) - Vec3::new(-0.2, 6.9, 7.2)).length() < 1e-4);
    w.step().unwrap();
    // Independent affine calculation: parent Rz(90) * scale(2,3,4),
    // posed hand (0.95,1.8,-0.1), then the authored grip offset.
    assert!(
        (place(&w, lantern) - Vec3::new(-0.8, 7.1, 6.8)).length() < 1e-4,
        "the lantern stayed at the skeleton root: {:?}",
        place(&w, lantern)
    );
    assert!((place(&w, socket) - Vec3::new(-1.7, 7.3, 4.4)).length() < 1e-4);
    assert!((place(&w, child) - Vec3::new(-1.7, 7.5, 5.2)).length() < 1e-4);
    assert_eq!(
        w.get::<og_ecs::transform::MatrixWorld>(child)
            .unwrap()
            .matrix,
        w.matrix_world(child).unwrap()
    );
    assert_eq!(w.get::<Transform>(lantern).unwrap(), offset);
    let saved = w.snapshot();
    w.remove::<BoneAttachment>(lantern).unwrap();
    assert!((place(&w, lantern) - Vec3::new(4.6, 5.2, 7.2)).length() < 1e-4);
    w.restore(&saved).unwrap();
    assert!((place(&w, lantern) - Vec3::new(-0.8, 7.1, 6.8)).length() < 1e-4);
}

#[derive(Default)]
struct Sink {
    palettes: Vec<(Entity, u32, Vec<Mat4>)>,
    weights: Vec<(Entity, u32, [f32; BLENDSHAPES])>,
}

impl PaletteSink for Sink {
    fn joint_palette(&mut self, entity: Entity, skin: u32, joints: &[Mat4]) {
        self.palettes.push((entity, skin, joints.to_vec()));
    }
    fn morph_weights(&mut self, entity: Entity, mesh: u32, weights: &[f32; BLENDSHAPES]) {
        self.weights.push((entity, mesh, *weights));
    }
}

#[test]
fn the_bind_pose_palette_is_identity_and_a_clip_moves_it() {
    let (mut w, anim) = world();
    let rest = w.spawn((Transform::default(), skel())).unwrap();
    let walking = w
        .spawn((Transform::default(), skel(), AnimationPlayer::new(WALK)))
        .unwrap();
    w.step().unwrap();
    let mut sink = Sink::default();
    anim.write_outputs(&mut sink);
    assert_eq!(sink.palettes.len(), 2);
    let (e, skin, p) = &sink.palettes[0];
    assert_eq!((*e, *skin, p.len()), (rest, 0, 12));
    assert!(p.iter().all(|m| near_identity(m, 1e-5)));
    assert_eq!(w.get::<Skeleton>(rest).unwrap().joint_count, 12);
    // Walking: the left foot joint's palette entry carries the foot's
    // bind-space origin to where the pose puts it.
    let (_, _, p) = &sink.palettes[1];
    let pose = anim.pose(walking).unwrap();
    let rest_foot = node_position(&skeleton().rest_pose(), 6);
    let posed_foot = node_position(&pose, 6);
    assert!((p[J_FOOT_L as usize].transform_point3(rest_foot) - posed_foot).length() < 1e-5);
    assert!(!near_identity(&p[J_FOOT_L as usize], 1e-3));
    assert!(sink.weights.is_empty(), "no og.Blendshapes, no weights");
}

#[test]
fn root_motion_moves_the_transform_and_pins_the_hips() {
    let (mut w, anim) = world();
    let e = w
        .spawn((
            Transform::at(Vec3::new(1.0, 0.0, 0.0)),
            Skeleton {
                root_motion: ROOT_MOTION_PLANAR,
                ..skel()
            },
            AnimationPlayer::new(WALK),
        ))
        .unwrap();
    for _ in 0..30 {
        w.step().unwrap();
    }
    let t: Transform = w.get(e).unwrap();
    // Half a walk cycle: 0.6 m forward (+z), the hip sway's yaw taken too.
    assert!(
        (t.position - Vec3::new(1.0, 0.0, 0.6)).length() < 0.05,
        "{t:?}"
    );
    let hips = anim.pose(e).unwrap()[1].translation;
    assert!(
        hips.x.abs() < 1e-5 && hips.z.abs() < 1e-5,
        "hips pinned: {hips}"
    );
    assert!((hips.y - 1.0).abs() < 0.05, "the bob stays in the pose");
    // A full cycle wraps without a jump: one stride per second.
    for _ in 0..90 {
        w.step().unwrap();
    }
    let t: Transform = w.get(e).unwrap();
    assert!((t.position.z - 2.4).abs() < 0.05, "{t:?}");
}
