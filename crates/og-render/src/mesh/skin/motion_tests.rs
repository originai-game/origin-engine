use super::*;
use crate::scene::{MaterialHandle, MeshHandle};

#[test]
fn bakes_survive_playback_but_replacements_and_clip_switches_refresh_them() {
    let mut clips = vec![
        BakedClip {
            joints: 1,
            frames: vec![Mat4::IDENTITY, Mat4::from_translation(Vec3::Y)],
        },
        BakedClip {
            joints: 1,
            frames: vec![Mat4::from_translation(Vec3::X); 3],
        },
    ];
    let used = [0, 1].into();
    let mut cache = BakeCache::default();
    let bytes = cache.update(&clips, &used).unwrap();
    assert_eq!(bytes.len(), 5);
    assert_eq!(cache.offsets[&1], 2);
    // Playback selects different offsets into the same resident bytes.
    for at in [0.0, 0.25, 1.0, 0.0] {
        let (frame, _) = clips[0].at(at);
        assert_eq!(bytes[cache.offsets[&0] + frame], clips[0].frames[frame]);
        assert!(cache.update(&clips, &used).is_none());
    }
    clips[0].frames[1] = Mat4::from_translation(Vec3::Z);
    assert_eq!(cache.update(&clips, &used).unwrap()[1], clips[0].frames[1]);
    assert!(cache.update(&clips, &used).is_none());
    assert_eq!(cache.update(&clips, &[1].into()).unwrap(), clips[1].frames);
    assert_eq!(cache.offsets[&1], 0);
    assert!(cache.update(&clips, &[1].into()).is_none());
    assert!(
        cache
            .update(&clips, &Default::default())
            .unwrap()
            .is_empty()
    );
}

fn palette(m: &[Mat4]) -> JointPalette {
    JointPalette::from(m.to_vec())
}

/// The second section mirrors the first's layout: a palette with a
/// previous pose of its own length takes it as given (world space); one
/// without (a joint count that changed) repeats this frame's joints in
/// the world at its first instance's transform, which reads as no
/// motion; a pose that only moved with its instance still counts as
/// moved.
#[test]
fn previous_joints_follow_the_palettes_layout() {
    let t = Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0));
    let j = Mat4::from_translation(Vec3::Y);
    let instances = [
        SkinnedInstance::new(
            MeshHandle::from_index(0),
            MaterialHandle::from_index(0),
            t,
            1,
        ),
        SkinnedInstance::new(
            MeshHandle::from_index(0),
            MaterialHandle::from_index(0),
            Mat4::IDENTITY,
            0,
        ),
    ];
    let now = [palette(&[j]), palette(&[j, j])];
    let mut joints: Vec<_> = [j, j, j].iter().map(Mat4::to_cols_array_2d).collect();
    // Palette 0 unmoved (its world joints as now), palette 1 lost a joint.
    let before = [palette(&[j]), palette(&[j])];
    assert_eq!(
        append_previous(&now, &before, &instances, &[], &mut joints),
        Some(false)
    );
    assert_eq!(joints.len(), 6);
    assert_eq!(joints[4], (t * j).to_cols_array_2d());
    // The instance under palette 0 moved a metre: moved.
    let mut joints: Vec<_> = [j, j, j].iter().map(Mat4::to_cols_array_2d).collect();
    let before = [
        palette(&[Mat4::from_translation(Vec3::X) * j]),
        palette(&[t * j, t * j]),
    ];
    assert_eq!(
        append_previous(&now, &before, &instances, &[], &mut joints),
        Some(true)
    );
    assert_eq!(
        joints[3],
        (Mat4::from_translation(Vec3::X) * j).to_cols_array_2d()
    );
    // No previous palettes, or a count that does not match: no section.
    assert_eq!(
        append_previous(&now, &before[1..], &instances, &[], &mut joints),
        None
    );
    let mut joints = Vec::new();
    assert_eq!(
        append_previous(&now, &[], &instances, &[], &mut joints),
        None
    );
    assert!(joints.is_empty());
}

/// Two vertices a slot, the shader's reading (`morphed`'s `deformed`):
/// vertex `v` at slot `v / 2`, its position in column `(v % 2) * 2` and
/// its normal in the next; an odd count leaves the last slot half empty.
#[test]
fn a_baked_clip_blends_its_frames_as_the_shader_does() {
    // Two joints, three frames: joint 0 moves up 1 a frame, joint 1
    // stays.
    let up = |y: f32| Mat4::from_translation(Vec3::new(0.0, y, 0.0));
    let clip = BakedClip {
        joints: 2,
        frames: vec![
            up(0.0),
            Mat4::IDENTITY,
            up(1.0),
            Mat4::IDENTITY,
            up(2.0),
            Mat4::IDENTITY,
        ],
    };
    assert_eq!(clip.frame_count(), 3);
    // The frame before and how far on: the last frame is reached
    // from the one before it, never past it.
    assert_eq!(clip.at(0.25), (0, 0.25));
    assert_eq!(clip.at(1.5), (1, 0.5));
    assert_eq!(clip.at(2.0), (1, 1.0));
    assert_eq!(clip.at(7.0), (1, 1.0));
    assert_eq!(clip.at(-1.0), (0, 0.0));
    assert_eq!(clip.at(f32::NAN), (0, 0.0));
    let p = clip.palette(1.25);
    assert!(p[0].abs_diff_eq(up(1.25), 1e-6), "{:?}", p[0]);
    assert_eq!(p[1], Mat4::IDENTITY);
}

#[test]
fn deformed_vertices_pack_two_a_slot() {
    let d = DeformedVertices {
        positions: (0..3)
            .map(|v| Vec3::new(v as f32, 10.0 + v as f32, -1.0))
            .collect(),
        normals: (0..3)
            .map(|v| Vec3::new(0.0, 0.0, v as f32 + 0.5))
            .collect(),
        garment: None,
    };
    let slots = deformed_slots(&d);
    assert_eq!(slots.len(), 2);
    assert_eq!(slots[0][2], [1.0, 11.0, -1.0, 1.0]);
    assert_eq!(slots[0][3], [0.0, 0.0, 1.5, 0.0]);
    assert_eq!(slots[1][0], [2.0, 12.0, -1.0, 1.0]);
    assert_eq!(slots[1][1], [0.0, 0.0, 2.5, 0.0]);
    assert_eq!(slots[1][2], [0.0; 4]);
}

/// The previous section carries the previous frame's weight slots after
/// its joints, at the same offset as the current ones; a change of
/// weight alone counts as motion; mismatched slots give no section.
#[test]
fn previous_weights_follow_the_joints() {
    let j = Mat4::IDENTITY;
    let inst = [SkinnedInstance::new(
        MeshHandle::from_index(0),
        MaterialHandle::from_index(0),
        Mat4::IDENTITY,
        0,
    )];
    let now = [palette(&[j])];
    let w_now = weight_slots(&[0.5; 20], 20);
    let w_before = weight_slots(&[0.25; 20], 20);
    let mut slots = vec![j.to_cols_array_2d()];
    slots.extend_from_slice(&w_now);
    assert_eq!(
        append_previous(&now, &now, &inst, &w_before, &mut slots),
        Some(true)
    );
    assert_eq!(slots.len(), 6);
    assert_eq!(slots[3..4], [j.to_cols_array_2d()]);
    assert_eq!(slots[4..], w_before[..]);
    // The same weights and joints: nothing moved.
    let mut slots = vec![j.to_cols_array_2d()];
    slots.extend_from_slice(&w_now);
    assert_eq!(
        append_previous(&now, &now, &inst, &w_now, &mut slots),
        Some(false)
    );
    // Weight slots that do not match the current section's: none.
    let mut slots = vec![j.to_cols_array_2d()];
    slots.extend_from_slice(&w_now);
    assert_eq!(append_previous(&now, &now, &inst, &[], &mut slots), None);
}
