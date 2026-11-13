use super::*;
use crate::scene::{MaterialHandle, MeshHandle};

/// A column of vertices from y 0 to 2: the lower ones on joint 0, the
/// upper ones on joint 1, the middle ones blended half and half.
fn column() -> (Vec<[f32; 3]>, SkinData) {
    let mut positions = Vec::new();
    let mut skin = SkinData::default();
    for i in 0..=20 {
        let y = i as f32 * 0.1;
        for x in [-0.2f32, 0.2] {
            positions.push([x, y, 0.1]);
            let w = ((y - 0.8) / 0.4).clamp(0.0, 1.0);
            skin.joints.push([0, 1, 0, 0]);
            skin.weights.push([1.0 - w, w, 0.0, 0.0]);
        }
    }
    (positions, skin)
}

fn posed(p: [f32; 3], skin: (&[u16; 4], &[f32; 4]), palette: &[Mat4], at: &Mat4) -> Vec3 {
    let p = Vec3::from(p);
    let v: Vec3 = (0..4)
        .map(|k| palette[skin.0[k] as usize].transform_point3(p) * skin.1[k])
        .sum();
    at.transform_point3(v)
}

fn inside((lo, hi): (Vec3, Vec3), p: Vec3) -> bool {
    p.cmpge(lo - 1e-4).all() && p.cmple(hi + 1e-4).all()
}

/// A pose that throws the upper half 6 m sideways and turns it: every
/// posed vertex, blended ones included, lies in the instance's box,
/// which the bind pose's sphere twice over does not reach (the old
/// bounds, which culled such a pose while it was in view).
#[test]
fn the_posed_box_holds_every_posed_vertex() {
    let (positions, skin) = column();
    let boxes = joint_boxes(&positions, &skin);
    let palette = [
        Mat4::IDENTITY,
        Mat4::from_translation(Vec3::new(6.0, 0.5, 0.0)) * Mat4::from_rotation_z(1.2),
    ];
    let at = Mat4::from_translation(Vec3::new(1.0, 0.0, -3.0)) * Mat4::from_rotation_y(0.7);
    let s = SkinnedInstance::new(
        MeshHandle::from_index(0),
        MaterialHandle::from_index(0),
        at,
        0,
    );
    let bind = (Vec3::new(0.0, 1.0, 0.1), Vec3::new(0.2, 1.0, 0.0));
    let b = skinned_box(&s, Some(bind), &boxes, &palette, None, false);
    let (c, r) = (at.transform_point3(bind.0), bind.1.length() * 2.0);
    let mut outside_sphere = 0;
    for (p, (j, w)) in positions.iter().zip(skin.joints.iter().zip(&skin.weights)) {
        let v = posed(*p, (j, w), &palette, &at);
        assert!(inside(b, v), "{v} outside {b:?}");
        outside_sphere += (v.distance(c) > r) as u32;
    }
    assert!(
        outside_sphere > 10,
        "{outside_sphere} vertices left the old sphere"
    );
    // And it is tight enough to cull: a little over the pose's reach.
    assert!((b.1 - b.0).max_element() < 9.0, "{b:?}");
    // Blendshapes move vertices off their joints' boxes: the sphere.
    let m = skinned_box(&s, Some(bind), &boxes, &palette, None, true);
    assert!((m.1 - m.0 - Vec3::splat(2.0 * r)).abs().max_element() < 1e-4);
}

/// A baked clip's box holds every frame's pose and the blends between
/// them, through the instance's transform.
#[test]
fn a_clips_box_holds_its_frames_and_their_blends() {
    let (positions, skin) = column();
    let boxes = joint_boxes(&positions, &skin);
    let frames: Vec<Mat4> = [0.0f32, 1.0, -0.8]
        .iter()
        .flat_map(|&a| {
            [
                Mat4::IDENTITY,
                Mat4::from_translation(Vec3::new(a * 3.0, 1.0, 0.0)) * Mat4::from_rotation_x(a),
            ]
        })
        .collect();
    let clip = BakedClip { joints: 2, frames };
    let at = Mat4::from_translation(Vec3::new(-4.0, 0.0, 2.0));
    let mut s = SkinnedInstance::new(
        MeshHandle::from_index(0),
        MaterialHandle::from_index(0),
        at,
        0,
    );
    s.baked = Some(BakedPose {
        clip: 0,
        frame: 0.0,
        previous_clip: 0,
        previous_frame: 0.0,
        previous_transform: at,
    });
    let b = skinned_box(&s, None, &boxes, &[], clip_box(&boxes, &clip), false);
    for f in [0.0, 0.3, 1.0, 1.5, 1.99] {
        let palette = clip.palette(f);
        for (p, (j, w)) in positions.iter().zip(skin.joints.iter().zip(&skin.weights)) {
            let v = posed(*p, (j, w), &palette, &at);
            assert!(inside(b, v), "frame {f}: {v} outside {b:?}");
        }
    }
}
