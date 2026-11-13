use super::*;
use crate::scene::Projection;
use crate::skin::SkinData;

/// A two-joint clip: joint 1 lifted by `f` metres at frame `f`.
fn lift_clip(frames: usize) -> BakedClip {
    BakedClip {
        joints: 2,
        frames: (0..frames)
            .flat_map(|f| [Mat4::IDENTITY, Mat4::from_translation(Vec3::Y * f as f32)])
            .collect(),
    }
}

fn crowd_of(members: Vec<CrowdMember>, levels: usize) -> Crowd {
    Crowd {
        parts: vec![CrowdPart {
            levels: (0..levels).map(MeshHandle::from_index).collect(),
            material: MaterialHandle::from_index(0),
        }],
        errors: [0.0, 0.02, 0.08][..levels].to_vec(),
        center: Vec3::new(0.0, 1.0, 0.0),
        radius: 1.0,
        members,
        ..Default::default()
    }
}

/// Each mesh: a vertex at the origin on joint 0, one at (0, 1, 0) on joint 1.
fn joints() -> Vec<JointBox> {
    let skin = SkinData {
        joints: vec![[0, 0, 0, 0], [1, 0, 0, 0]],
        weights: vec![[1.0, 0.0, 0.0, 0.0]; 2],
    };
    crate::skin::joint_boxes(&[[0.0; 3], [0.0, 1.0, 0.0]], &skin)
}

fn frame_of(crowds: &[Crowd], clips: &[BakedClip], boxes: &[JointBox], motion: bool) -> CrowdFrame {
    build(
        crowds,
        clips,
        &|_| {
            Some(MeshInfo {
                skinned: true,
                index_count: 36,
                joints: boxes,
            })
        },
        1,
        &|_| false,
        &|c| 100 * c as usize + 7,
        motion,
        1.0,
    )
    .expect("a valid crowd")
}

#[test]
fn a_member_s_level_is_the_model_rule_at_its_distance() {
    let camera = Camera {
        position: Vec3::ZERO,
        projection: Projection::Perspective {
            fov_y_degrees: 50.0,
        },
        ..Camera::default()
    };
    let crowd = crowd_of(Vec::new(), 3);
    for d in [3.0f32, 12.0, 40.0, 160.0] {
        let at = Mat4::from_translation(Vec3::new(0.0, -1.0, -d));
        let per_unit = crate::lod::pixels_per_unit(&camera, Vec3::new(0.0, 0.0, -d), 1.0, 720.0);
        let want = crate::lod::select(&crowd.errors, per_unit, 1.0, None);
        assert_eq!(
            member_level(&crowd, &at, &camera, 720.0, 1.0),
            want,
            "at {d} m"
        );
    }
    // Near: the full mesh; far: the coarsest. A bias of 0 keeps the full mesh.
    let far = Mat4::from_translation(Vec3::new(0.0, 0.0, -400.0));
    assert_eq!(
        member_level(&crowd, &Mat4::IDENTITY, &camera, 720.0, 1.0),
        0
    );
    assert_eq!(member_level(&crowd, &far, &camera, 720.0, 1.0), 2);
    assert_eq!(member_level(&crowd, &far, &camera, 720.0, 0.0), 0);
    // Scale counts: a member twice the size keeps detail twice as far.
    let mid = Mat4::from_translation(Vec3::new(0.0, 0.0, -100.0));
    let big = mid * Mat4::from_scale(Vec3::splat(2.0));
    assert!(
        member_level(&crowd, &big, &camera, 720.0, 1.0)
            < member_level(&crowd, &mid, &camera, 720.0, 1.0)
    );
}

#[test]
fn cpu_records_group_members_by_level_and_pose_them_as_the_shader_blends() {
    let camera = Camera {
        position: Vec3::new(0.0, 1.0, 0.0),
        ..Camera::default()
    };
    // Members alternating near and far, at frames 0.25, 1.5, 2.75, 3.0.
    let members: Vec<CrowdMember> = (0..4)
        .map(|i| {
            let z = if i % 2 == 0 { -2.0 } else { -300.0 };
            let mut m = CrowdMember::new(
                Mat4::from_translation(Vec3::Z * z),
                1,
                0.25 + 1.25 * i as f32,
            );
            m.pose.previous_frame = 0.5;
            m
        })
        .collect();
    let crowds = vec![crowd_of(members, 3)];
    let clips = vec![lift_clip(2), lift_clip(4)];
    let boxes = joints();
    let f = frame_of(&crowds, &clips, &boxes, true);
    assert_eq!(f.draws.len(), 3);
    assert_eq!(f.previous.len(), 4);
    let CpuCrowds {
        runs,
        data,
        boxes: bx,
        ..
    } = cpu_records(&crowds, &f, &camera, 720.0, 1.0, Some(50));
    // Level 0 holds members 0 and 2 in order, level 2 members 1 and 3.
    assert_eq!(runs, vec![(0, 0, 2), (2, 2, 2)]);
    assert_eq!(data.len(), 4);
    // Clip 1 starts at slot 107 (the offset given), two joints a frame:
    // frame 0.25 is frame 0 a quarter on; 2.75 frame 2 three quarters
    // on; 3.0, the last frame, is the frame before it all the way.
    let slot = |r: &SkinnedInstanceGpu| (r.palette[0], -1.0 - r.palette[3]);
    assert_eq!(slot(&data[0]), (107.0, 0.25));
    assert_eq!(slot(&data[1]), (111.0, 0.75));
    assert_eq!(slot(&data[3]), (111.0, 1.0));
    assert_eq!(data[0].palette[1], 2.0);
    // Previous transforms at slot 50 + member, plus one; previous frame 0.5.
    let prev: Vec<f32> = data.iter().map(|r| r.morph1[0]).collect();
    assert_eq!(prev, vec![51.0, 53.0, 52.0, 54.0]);
    assert_eq!((data[0].morph1[1], data[0].morph1[2]), (107.0, 0.5));
    // Each box holds the member's lifted joint at every frame of the clip.
    for (r, (lo, hi)) in data.iter().zip(&bx) {
        let t = Vec3::from_array(r.transform[3]);
        assert!(lo.y <= t.y && hi.y >= t.y + 1.0 + 3.0, "{lo} {hi}");
    }
    // Without motion: no previous slots, records name none.
    let f = frame_of(&crowds, &clips, &boxes, false);
    let data = cpu_records(&crowds, &f, &camera, 720.0, 1.0, None).data;
    assert!(f.previous.is_empty() && data.iter().all(|r| r.morph1[0] == 0.0));
}

#[test]
fn a_crowd_box_holds_every_frame_and_a_missing_clip_is_an_error() {
    let boxes = joints();
    let crowds = vec![crowd_of(vec![CrowdMember::new(Mat4::IDENTITY, 0, 0.0)], 1)];
    let f = frame_of(&crowds, &[lift_clip(5)], &boxes, false);
    let c = f.clips[0];
    assert_eq!((c.lo[1], c.hi[1]), (0.0, 5.0));
    assert_eq!(c.info, [7, 2, 5, 0]);
    let bad = vec![crowd_of(vec![CrowdMember::new(Mat4::IDENTITY, 3, 0.0)], 1)];
    let err = build(
        &bad,
        &[lift_clip(5)],
        &|_| None,
        1,
        &|_| false,
        &|_| 0,
        false,
        1.0,
    );
    assert!(err.is_err());
    // An empty crowd has no row.
    let f = frame_of(&[crowd_of(Vec::new(), 1)], &[lift_clip(2)], &boxes, false);
    assert!(f.crowds.is_empty() && f.draws.is_empty());
}

#[test]
fn an_impostor_is_the_last_level_and_keeps_its_clip() {
    let camera = Camera::default();
    let mut crowd = crowd_of(Vec::new(), 3);
    crowd.impostor = Some(CrowdImpostor {
        clip: 1,
        size: 16,
        ..Default::default()
    });
    // Three mesh levels and the card; its error a texel of the atlas
    // (2 m / 16 on a 1 m sphere), above the coarsest mesh's.
    assert_eq!((crowd.mesh_levels(), crowd.levels()), (3, 4));
    assert_eq!(crowd.level_errors(), vec![0.0, 0.02, 0.08, 0.125]);
    // A finer atlas (a texel 2 m / 64) takes the coarsest mesh's error:
    // the card replaces the mesh levels coarser than a texel (lod.rs's
    // rule for model impostors).
    let mut fine = crowd.clone();
    fine.impostor.as_mut().unwrap().size = 64;
    assert_eq!(fine.level_errors(), vec![0.0, 0.02, 0.08, 0.08]);
    // Far away: the card, for a member playing its clip; the coarsest
    // mesh for one playing another.
    let far = Mat4::from_translation(Vec3::new(0.0, 0.0, -2000.0));
    assert_eq!(member_level(&crowd, &far, &camera, 720.0, 1.0), 3);
    let on = CrowdMember::new(far, 1, 0.0);
    let off = CrowdMember::new(far, 0, 0.0);
    assert_eq!(drawn_level(&crowd, &on, 3), 3);
    assert_eq!(drawn_level(&crowd, &off, 3), 2);
    // Between the coarsest mesh and the card: still the mesh.
    // At 64 m (about 10 pixels a metre at 60 degrees) the coarsest mesh's
    // error is under a pixel and a card's texel over it.
    let mid = Mat4::from_translation(Vec3::new(0.0, -1.0, -64.0));
    assert_eq!(member_level(&crowd, &mid, &camera, 720.0, 1.0), 2);
    // Four mesh levels with an impostor: the last mesh level gives way.
    let mut four = crowd.clone();
    four.errors = vec![0.0, 0.01, 0.02, 0.04];
    assert_eq!((four.mesh_levels(), four.levels()), (3, 4));
    // A card's row: frame 15 of a 31-frame loop is halfway through 16
    // atlas frames; the last frame (the first again) wraps to 0.
    assert_eq!(card_row(15.0, 31, 16), 8.0);
    assert_eq!(card_row(30.0, 31, 16), 0.0);
    assert_eq!(card_row(f32::NAN, 31, 16), 0.0);
}
