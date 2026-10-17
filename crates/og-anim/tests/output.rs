//! What the renderer receives: the joint palette (model matrix × inverse
//! bind, identity at the bind pose, moving with the clip) and blendshape
//! weights that write through from the component or come from the clips,
//! both handed to a [`PaletteSink`]; and the refusals for missing assets.

mod support;

use og_anim::*;
use og_core::glam::{Mat4, Vec3};
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

#[derive(Default)]
struct Sink {
    palettes: Vec<(Entity, u32, Vec<Mat4>)>,
    weights: Vec<(Entity, u32, [f32; 8])>,
}

impl PaletteSink for Sink {
    fn joint_palette(&mut self, entity: Entity, skin: u32, joints: &[Mat4]) {
        self.palettes.push((entity, skin, joints.to_vec()));
    }
    fn morph_weights(&mut self, entity: Entity, mesh: u32, weights: &[f32; 8]) {
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
fn component_weights_write_through() {
    let (mut w, anim) = world();
    let want = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
    let mut b = Blendshapes {
        mesh: 0,
        ..Blendshapes::default()
    };
    b.set_weights(want);
    let e = w
        .spawn((Transform::default(), skel(), b, AnimationPlayer::new(BLINK)))
        .unwrap();
    w.step().unwrap();
    assert_eq!(
        anim.output(e).unwrap().weights,
        want,
        "the clip does not override"
    );
    let mut sink = Sink::default();
    anim.write_outputs(&mut sink);
    assert_eq!(sink.weights, vec![(e, 0, want)]);
    b.w3 = -0.5;
    w.insert(e, b).unwrap();
    w.step().unwrap();
    assert_eq!(anim.output(e).unwrap().weights[3], -0.5);
}

#[test]
fn clip_weights_drive_the_component() {
    let (mut w, anim) = world();
    let e = w
        .spawn((
            Transform::default(),
            skel(),
            Blendshapes {
                source: BLENDSHAPES_FROM_CLIPS,
                ..Blendshapes::default()
            },
            AnimationPlayer::new(BLINK),
        ))
        .unwrap();
    for k in 1..=12u32 {
        w.step().unwrap();
        let t = k as f32 / 60.0;
        let b: Blendshapes = w.get(e).unwrap();
        assert!(
            (b.w0 - 2.0 * t).abs() < 1e-5 && (b.w1 - (1.0 - 2.0 * t)).abs() < 1e-5,
            "{b:?} at {t}"
        );
        assert_eq!(anim.output(e).unwrap().weights, b.weights());
    }
    // Another mesh index takes nothing from the clip (head draws mesh 0).
    let mut b: Blendshapes = w.get(e).unwrap();
    b.mesh = 1;
    b.set_weights([0.0; 8]);
    w.insert(e, b).unwrap();
    w.step().unwrap();
    assert_eq!(w.get::<Blendshapes>(e).unwrap().weights(), [0.0; 8]);
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

#[test]
fn unregistered_assets_are_refused_with_the_entity() {
    let (mut w, anim) = world();
    let e = w
        .spawn((Skeleton {
            asset: 99,
            ..Skeleton::default()
        },))
        .unwrap();
    let err = w.step().unwrap_err().to_string();
    assert!(
        err.contains("og.Skeleton.asset 99") && err.contains(&e.to_bits().to_string()),
        "{err}"
    );
    w.despawn(e).unwrap();
    w.spawn((skel(), AnimationPlayer::new(77))).unwrap();
    let err = w.step().unwrap_err().to_string();
    assert!(err.contains("og.AnimationPlayer.clip 77"), "{err}");
    assert!(anim.add_clip(0, clips().remove(0).1).is_err());
    let mut bad = clips().remove(0).1;
    bad.frames += 1;
    assert!(anim.add_clip(5, bad).is_err());
}
