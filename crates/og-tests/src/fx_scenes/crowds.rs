//! Crowds at scale: thousands of walkers playing one baked walk on the
//! GPU, for the crowd perf scenes (`crowd-5k`, `crowd-20k`, `crowd-50k`)
//! and the crowd frames.
//!
//! The walker is a block figure built here: eleven rigid parts (hips,
//! chest, head, two thighs, shins, upper arms and forearms), each a capped
//! cylinder bound to one joint, about 1,900 triangles at full detail, a
//! crowd member's budget in a commercial game's near level. Its walk is
//! baked at 30 frames a second over one second (31 frames, the last a copy
//! of the first).

use super::*;

/// The walker's joints, parents first: (parent, rest offset from it).
const WALKER_JOINTS: [(i32, [f32; 3]); 11] = [
    (-1, [0.0, 0.95, 0.0]),   // 0 hips
    (0, [0.0, 0.12, 0.0]),    // 1 chest
    (1, [0.0, 0.5, 0.0]),     // 2 head (neck base)
    (0, [-0.11, -0.02, 0.0]), // 3 thigh.L
    (3, [0.0, -0.44, 0.0]),   // 4 shin.L
    (0, [0.11, -0.02, 0.0]),  // 5 thigh.R
    (5, [0.0, -0.44, 0.0]),   // 6 shin.R
    (1, [-0.22, 0.44, 0.0]),  // 7 arm.L
    (7, [0.0, -0.3, 0.0]),    // 8 forearm.L
    (1, [0.22, 0.44, 0.0]),   // 9 arm.R
    (9, [0.0, -0.3, 0.0]),    // 10 forearm.R
];

/// Each part: (joint, length down from the joint (up for the chest and
/// head), radius, colour).
const WALKER_PARTS: [(u16, f32, f32, [f32; 4]); 11] = [
    (0, -0.16, 0.16, [0.25, 0.28, 0.42, 1.0]),
    (1, 0.46, 0.17, [0.72, 0.32, 0.22, 1.0]),
    (2, 0.24, 0.11, [0.86, 0.68, 0.55, 1.0]),
    (3, -0.44, 0.075, [0.25, 0.28, 0.42, 1.0]),
    (4, -0.46, 0.065, [0.25, 0.28, 0.42, 1.0]),
    (5, -0.44, 0.075, [0.25, 0.28, 0.42, 1.0]),
    (6, -0.46, 0.065, [0.25, 0.28, 0.42, 1.0]),
    (7, -0.3, 0.055, [0.72, 0.32, 0.22, 1.0]),
    (8, -0.28, 0.048, [0.86, 0.68, 0.55, 1.0]),
    (9, -0.3, 0.055, [0.72, 0.32, 0.22, 1.0]),
    (10, -0.28, 0.048, [0.86, 0.68, 0.55, 1.0]),
];

/// Each joint's rest position in the model (the sum of its offsets).
fn walker_rest() -> [Vec3; 11] {
    let mut at = [Vec3::ZERO; 11];
    for (j, &(parent, offset)) in WALKER_JOINTS.iter().enumerate() {
        let base = if parent < 0 {
            Vec3::ZERO
        } else {
            at[parent as usize]
        };
        at[j] = base + Vec3::from_array(offset);
    }
    at
}

/// The walker's mesh and skin: each part a capped cylinder of `sides`
/// sides and `rings` rings, coloured by its vertex colours, weighted wholly
/// to its joint. `sides` 16 and `rings` 4: about 1,900 triangles.
pub fn walker_mesh(sides: u32, rings: u32) -> (MeshData, SkinData) {
    let rest = walker_rest();
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut joints = Vec::new();
    let mut weights = Vec::new();
    for &(joint, length, radius, color) in &WALKER_PARTS {
        let base = rest[joint as usize];
        let push = |vertices: &mut Vec<Vertex>, p: Vec3, n: Vec3, uv: [f32; 2]| {
            vertices.push(Vertex {
                tangent: [0.0; 4],
                position: p.to_array(),
                normal: n.to_array(),
                uv,
                color,
            });
        };
        let first = vertices.len() as u32;
        for r in 0..=rings {
            let f = r as f32 / rings as f32;
            // Rounded ends: the radius eases in at both ends.
            let round = (1.0 - (2.0 * f - 1.0).powi(4)).max(0.0).sqrt() * 0.35 + 0.65;
            let y = base.y + length * f;
            for s in 0..=sides {
                let a = s as f32 / sides as f32 * std::f32::consts::TAU;
                let n = Vec3::new(a.cos(), 0.0, a.sin());
                push(
                    &mut vertices,
                    Vec3::new(base.x, y, base.z) + n * radius * round,
                    n,
                    [s as f32 / sides as f32, f],
                );
            }
        }
        for r in 0..rings {
            for s in 0..sides {
                let a = first + r * (sides + 1) + s;
                let b = a + sides + 1;
                if length > 0.0 {
                    indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
                } else {
                    indices.extend_from_slice(&[a, a + 1, b, a + 1, b + 1, b]);
                }
            }
        }
        for (y, up) in [
            (base.y, -length.signum()),
            (base.y + length, length.signum()),
        ] {
            let center = vertices.len() as u32;
            push(
                &mut vertices,
                Vec3::new(base.x, y, base.z),
                Vec3::Y * up,
                [0.5, 0.5],
            );
            for s in 0..=sides {
                let a = s as f32 / sides as f32 * std::f32::consts::TAU;
                push(
                    &mut vertices,
                    Vec3::new(
                        base.x + a.cos() * radius * 0.65,
                        y,
                        base.z + a.sin() * radius * 0.65,
                    ),
                    Vec3::Y * up,
                    [0.5 + a.cos() * 0.5, 0.5 + a.sin() * 0.5],
                );
            }
            for s in 0..sides {
                let (p, q) = (center + 1 + s, center + 2 + s);
                if up > 0.0 {
                    indices.extend_from_slice(&[center, q, p]);
                } else {
                    indices.extend_from_slice(&[center, p, q]);
                }
            }
        }
        let added = vertices.len() - joints.len();
        joints.extend(std::iter::repeat_n([joint, 0, 0, 0], added));
        weights.extend(std::iter::repeat_n([1.0, 0.0, 0.0, 0.0], added));
    }
    (MeshData { vertices, indices }, SkinData { joints, weights })
}

/// The walker's palette at phase `p` (0 to 1) of its walk: legs swinging
/// from the hips, knees bending on the back swing, arms swinging against
/// the legs, the hips bobbing twice a stride.
pub fn walker_palette(p: f32) -> Vec<Mat4> {
    let rest = walker_rest();
    let w = p * std::f32::consts::TAU;
    let swing = w.sin() * 0.5;
    let knee = |phase: f32| ((phase.cos() + 1.0) * 0.5).powi(2) * 0.8;
    let local_rot = [
        Mat4::from_rotation_y(w.sin() * 0.08),
        Mat4::from_rotation_y(-w.sin() * 0.12),
        Mat4::IDENTITY,
        Mat4::from_rotation_x(swing),
        Mat4::from_rotation_x(knee(w)),
        Mat4::from_rotation_x(-swing),
        Mat4::from_rotation_x(knee(w + std::f32::consts::PI)),
        Mat4::from_rotation_x(-swing * 0.8) * Mat4::from_rotation_z(-0.12),
        Mat4::from_rotation_x(-0.35),
        Mat4::from_rotation_x(swing * 0.8) * Mat4::from_rotation_z(0.12),
        Mat4::from_rotation_x(-0.35),
    ];
    let bob = Mat4::from_translation(Vec3::Y * ((2.0 * w).cos() * 0.025));
    let mut global = [Mat4::IDENTITY; 11];
    let mut palette = Vec::with_capacity(11);
    for (j, &(parent, offset)) in WALKER_JOINTS.iter().enumerate() {
        let local = Mat4::from_translation(Vec3::from_array(offset)) * local_rot[j];
        global[j] = if parent < 0 {
            bob * local
        } else {
            global[parent as usize] * local
        };
        palette.push(global[j] * Mat4::from_translation(-rest[j]));
    }
    palette
}

/// The walk baked as a clip: `frames` frames over one stride, the last a
/// copy of the first.
pub fn walker_clip(frames: usize) -> og_render::BakedClip {
    let frames = frames.max(2);
    og_render::BakedClip {
        joints: WALKER_JOINTS.len() as u32,
        frames: (0..frames)
            .flat_map(|k| walker_palette(k as f32 / (frames - 1) as f32))
            .collect(),
    }
}

/// Frames of [`crowd_scene`]'s walk.
pub const CROWD_FRAMES: usize = 31;
/// Metres a walker covers a second.
const CROWD_SPEED: f32 = 1.3;

/// A deterministic hash of `i` to 0..1.
fn unit(i: u32, seed: u32) -> f32 {
    let mut h = i.wrapping_mul(0x9e37_79b9) ^ seed.wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 15)).wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    (h & 0xff_ffff) as f32 / 16_777_215.0
}

/// Walker `i`'s place, heading and walk phase at second `t`: on a jittered
/// grid `side` walkers wide, 1.3 m apart, walking a short loop about its
/// cell (a circle of 1.5 m), so the crowd stays put and every transform
/// changes every frame.
fn walker_at(i: u32, side: u32, t: f32) -> (Mat4, f32) {
    let (x, z) = ((i % side) as f32, (i / side) as f32);
    let half = side as f32 * 0.5;
    let cell = Vec3::new(
        (x - half + unit(i, 1) * 0.6) * 1.3,
        0.0,
        -(z + unit(i, 2) * 0.6) * 1.3,
    );
    let r = 1.5;
    let dir = if unit(i, 3) < 0.5 { 1.0 } else { -1.0 };
    let a = unit(i, 4) * std::f32::consts::TAU + dir * t * CROWD_SPEED / r;
    let at = cell + Vec3::new(a.cos(), 0.0, a.sin()) * r * 0.3;
    // Facing along the circle's tangent.
    let heading = -a - dir * std::f32::consts::FRAC_PI_2;
    let scale = 0.9 + unit(i, 5) * 0.2;
    let stride = 1.4 * scale;
    let phase = (unit(i, 6) + t * CROWD_SPEED / stride).rem_euclid(1.0);
    (
        Mat4::from_translation(at)
            * Mat4::from_rotation_y(heading)
            * Mat4::from_scale(Vec3::splat(scale)),
        phase,
    )
}

/// `count` walkers on a plaza, seen from 9 m up at its near edge looking
/// across it, a sun from the upper right casting their shadows, and four
/// buildings standing among them (occluders). With `taa`, the crowd's
/// motion vectors come from each member's previous frame and transform.
pub fn crowd_scene(count: u32) -> RenderScene {
    let mut scene = RenderScene::new();
    let side = (count as f32).sqrt().ceil().max(1.0) as u32;
    let depth = count.div_ceil(side) as f32 * 1.3;
    terrain(&mut scene, (side as f32 * 1.3).max(depth) * 2.0 + 40.0);
    // Three levels: 16 sides and 4 rings (about 1,900 triangles), 8 and
    // 2, 4 and 1. Each level's error is the chord error of its thickest
    // part (the chest, 0.17 m): r (1 - cos(pi / sides)), its rounding
    // ends a little more.
    let levels: Vec<og_render::MeshHandle> = [(16, 4), (8, 2), (4, 1)]
        .iter()
        .map(|&(sides, rings)| {
            let (mesh, skin) = walker_mesh(sides, rings);
            scene.add_skinned_mesh(mesh, skin)
        })
        .collect();
    let cloth = scene.add_material(Material {
        base_color: [1.0; 4],
        roughness: 0.7,
        ..Default::default()
    });
    scene.baked_clips.push(walker_clip(CROWD_FRAMES));
    let block = scene.add_mesh(shapes::cuboid(Vec3::new(12.0, 14.0, 12.0)));
    let stone = scene.add_material(Material {
        base_color: [0.62, 0.58, 0.52, 1.0],
        roughness: 0.85,
        ..Default::default()
    });
    for (x, z) in [(-0.3, 0.15), (0.25, 0.3), (-0.1, 0.55), (0.35, 0.75)] {
        scene.add_instance(
            block,
            stone,
            Mat4::from_translation(Vec3::new(x * side as f32 * 1.3, 7.0, -z * depth)),
        );
    }
    scene.crowds.push(og_render::Crowd {
        parts: vec![og_render::CrowdPart {
            levels,
            material: cloth,
        }],
        errors: vec![0.0, 0.02, 0.06],
        center: Vec3::new(0.0, 0.9, 0.0),
        radius: 0.95,
        members: (0..count)
            .map(|i| {
                let (at, phase) = walker_at(i, side, 0.0);
                og_render::CrowdMember::new(at, 0, phase * (CROWD_FRAMES - 1) as f32)
            })
            .collect(),
        ..Default::default()
    });
    scene.light.direction_to_light = Vec3::new(0.5, 0.8, 0.35);
    scene.light.cast_shadows = true;
    scene.camera = Camera {
        position: Vec3::new(0.0, 9.0, 6.0),
        target: Vec3::new(0.0, 0.0, -depth * 0.35),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    scene
}

/// Advances every walker of [`crowd_scene`] by a frame at 60 frames a
/// second: its place on its loop, its heading and its walk's frame, the
/// frame before kept for TAA.
pub fn walk_crowd(scene: &mut RenderScene, frame: u32) {
    let Some(crowd) = scene.crowds.first_mut() else {
        return;
    };
    let count = crowd.members.len() as u32;
    let side = (count as f32).sqrt().ceil().max(1.0) as u32;
    let t = (frame + 1) as f32 / 60.0;
    for (i, m) in crowd.members.iter_mut().enumerate() {
        let (at, phase) = walker_at(i as u32, side, t);
        let b = &mut m.pose;
        b.previous_frame = b.frame;
        b.previous_transform = m.transform;
        b.frame = phase * (CROWD_FRAMES - 1) as f32;
        m.transform = at;
    }
}
