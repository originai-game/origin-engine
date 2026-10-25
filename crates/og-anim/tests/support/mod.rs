//! A procedural walker rig shared by the og-anim tests (and the wasm probe):
//! a 13-node skeleton (hips, spine, head with a two-target mesh, two
//! three-joint legs, a three-joint left arm), five clips sampled at 30 Hz,
//! and a two-layer graph (locomotion state machine plus an additive wave
//! masked to the arm, in step with the gait while it moves).
#![allow(dead_code)]

pub mod walk;

use og_anim::*;
use og_core::glam::{Mat4, Quat, Vec3};
use og_core::math;

pub const SKELETON: u64 = 1;
pub const IDLE: u64 = 10;
pub const WALK: u64 = 11;
pub const RUN: u64 = 12;
pub const WAVE: u64 = 13;
pub const BLINK: u64 = 14;
/// The walk mirrored (`og_anim::mirror`).
pub const WALK_MIRRORED: u64 = 15;
pub const GRAPH: u64 = 20;

/// Joint indices (the skin's joints are nodes 1..=12).
pub const J_HIPS: u32 = 0;
pub const J_FOOT_L: u32 = 5;
pub const J_FOOT_R: u32 = 8;
pub const J_HAND_L: u32 = 11;
pub const J_HEAD: u32 = 2;
pub const J_SHIN_L: u32 = 4;
pub const J_SHIN_R: u32 = 7;
pub const J_ARM_L: u32 = 9;

pub fn skeleton() -> SkeletonAsset {
    let n = |name: &str, parent: u32, t: [f32; 3], mesh: u32| SkeletonNode {
        name: name.into(),
        parent,
        rest: Trs::new(Vec3::from(t), Quat::IDENTITY, Vec3::ONE),
        mesh,
    };
    let nodes = vec![
        n("Rig", NONE, [0.0, 0.0, 0.0], NONE),
        n("hips", 0, [0.0, 1.0, 0.0], NONE),
        n("spine", 1, [0.0, 0.3, 0.0], NONE),
        n("head", 2, [0.0, 0.4, 0.0], 0),
        n("thigh.L", 1, [0.15, 0.0, 0.0], NONE),
        n("shin.L", 4, [0.0, -0.45, 0.0], NONE),
        n("foot.L", 5, [0.0, -0.45, 0.0], NONE),
        n("thigh.R", 1, [-0.15, 0.0, 0.0], NONE),
        n("shin.R", 7, [0.0, -0.45, 0.0], NONE),
        n("foot.R", 8, [0.0, -0.45, 0.0], NONE),
        n("arm.L", 2, [0.25, 0.3, 0.0], NONE),
        n("forearm.L", 10, [0.3, 0.0, 0.0], NONE),
        n("hand.L", 11, [0.3, 0.0, 0.0], NONE),
    ];
    let mut s = SkeletonAsset {
        nodes,
        ..Default::default()
    };
    let model = s.model_matrices(&s.rest_pose());
    let joints: Vec<u32> = (1..13).collect();
    let inverse_bind = joints
        .iter()
        .map(|&j| model[j as usize].inverse())
        .collect();
    s.skins.push(Skin {
        name: "walker".into(),
        joints,
        inverse_bind,
    });
    s
}

fn rot_x(a: f32) -> Quat {
    math::quat_from_axis_angle(Vec3::X, a)
}

fn rot_z(a: f32) -> Quat {
    math::quat_from_axis_angle(Vec3::Z, a)
}

/// A channel as (node, node name, path, lanes, value at a time).
pub type ChannelFn<'a> = (u32, &'a str, ChannelPath, u32, Box<dyn Fn(f32) -> Vec<f32>>);

/// A clip from per-time functions of its channels.
pub fn clip(
    name: &str,
    duration: f32,
    events: &[(&str, f32)],
    channels: Vec<ChannelFn>,
) -> ClipAsset {
    let rate = 30.0;
    let frames = ClipAsset::frames_for(duration, rate);
    let channels = channels
        .into_iter()
        .map(|(node, node_name, path, lanes, f)| {
            let mut samples = Vec::new();
            for i in 0..frames {
                let t = (i as f32 / rate).min(duration);
                samples.extend(f(t));
            }
            Channel {
                node,
                node_name: node_name.into(),
                path,
                step: false,
                lanes,
                samples,
            }
        })
        .collect();
    ClipAsset {
        name: name.into(),
        duration,
        rate,
        source_rate: rate,
        frames,
        channels,
        events: events
            .iter()
            .map(|(n, t)| ClipEvent {
                name: n.to_string(),
                time: *t,
            })
            .collect(),
    }
}

fn q(v: Quat) -> Vec<f32> {
    v.to_array().to_vec()
}

const TAU: f32 = std::f32::consts::TAU;

/// A gait: legs swinging by `swing` radians, hips travelling `stride` metres per cycle.
fn gait(name: &str, duration: f32, swing: f32, stride: f32, events: &[(&str, f32)]) -> ClipAsset {
    let phase = move |t: f32| t / duration * TAU;
    clip(
        name,
        duration,
        events,
        vec![
            (
                1,
                "hips",
                ChannelPath::Translation,
                3,
                Box::new(move |t| {
                    let bob = 0.03 * math::cos(2.0 * phase(t));
                    vec![0.0, 1.0 + bob, stride * t / duration]
                }),
            ),
            (
                1,
                "hips",
                ChannelPath::Rotation,
                4,
                Box::new(move |t| {
                    q(math::quat_from_axis_angle(
                        Vec3::Y,
                        0.1 * math::sin(phase(t)),
                    ))
                }),
            ),
            (
                4,
                "thigh.L",
                ChannelPath::Rotation,
                4,
                Box::new(move |t| q(rot_x(swing * math::sin(phase(t))))),
            ),
            (
                5,
                "shin.L",
                ChannelPath::Rotation,
                4,
                Box::new(move |t| q(rot_x(0.6 * swing * (1.0 + math::sin(phase(t) - 1.2))))),
            ),
            (
                7,
                "thigh.R",
                ChannelPath::Rotation,
                4,
                Box::new(move |t| q(rot_x(-swing * math::sin(phase(t))))),
            ),
            (
                8,
                "shin.R",
                ChannelPath::Rotation,
                4,
                Box::new(move |t| q(rot_x(0.6 * swing * (1.0 - math::sin(phase(t) - 1.2))))),
            ),
        ],
    )
}

pub fn clips() -> Vec<(u64, ClipAsset)> {
    vec![
        (
            IDLE,
            clip(
                "idle",
                2.0,
                &[],
                vec![
                    (
                        2,
                        "spine",
                        ChannelPath::Rotation,
                        4,
                        Box::new(|t| q(rot_z(0.05 * math::sin(t * TAU / 2.0)))),
                    ),
                    (
                        1,
                        "hips",
                        ChannelPath::Translation,
                        3,
                        Box::new(|t| vec![0.0, 1.0 - 0.01 * math::sin(t * TAU / 2.0), 0.0]),
                    ),
                ],
            ),
        ),
        (
            WALK,
            gait("walk", 1.0, 0.5, 1.2, &[("footL", 0.26), ("footR", 0.76)]),
        ),
        (
            RUN,
            gait("run", 0.6, 0.9, 1.5, &[("footL", 0.16), ("footR", 0.46)]),
        ),
        (
            WAVE,
            clip(
                "wave",
                1.0,
                &[("wave", 0.5)],
                vec![(
                    10,
                    "arm.L",
                    ChannelPath::Rotation,
                    4,
                    Box::new(|t| q(rot_z(1.2 * math::sin(t * TAU)))),
                )],
            ),
        ),
        (
            BLINK,
            clip(
                "blink",
                0.5,
                &[],
                vec![(
                    3,
                    "head",
                    ChannelPath::Weights,
                    2,
                    Box::new(|t| vec![t * 2.0, 1.0 - t * 2.0]),
                )],
            ),
        ),
    ]
}

pub const GRAPH_TEXT: &str = r#"type animgraph
version 1
---
param speed
param wave
clip idle "idle"
clip walk "walk"
clip run "run"
clip waving "wave"
blend1d move speed {
  child walk 1.5
  child run 4
}
layer base {
  state idle idle
  state move move {
    sync gait
  }
  entry idle
  transition idle move {
    when speed gt 0.2
    fade 0.25
    interrupt none
  }
  transition move idle {
    when speed le 0.2
    fade 0.3
  }
}
layer upper {
  mode additive
  weight wave
  mask "arm.L"
  state wave waving {
    sync gait
  }
  entry wave
}
"#;

pub fn resolve(name: &str) -> Option<u64> {
    match name {
        "idle" => Some(IDLE),
        "walk" => Some(WALK),
        "run" => Some(RUN),
        "wave" => Some(WAVE),
        "blink" => Some(BLINK),
        _ => None,
    }
}

/// Registers the rig's skeleton, clips and graph.
pub fn register(anim: &AnimationSystem) {
    anim.add_skeleton(SKELETON, skeleton()).unwrap();
    for (h, c) in clips() {
        if h == WALK {
            anim.add_clip(WALK_MIRRORED, mirror(&skeleton(), &c).unwrap())
                .unwrap();
        }
        anim.add_clip(h, c).unwrap();
    }
    anim.add_graph(GRAPH, GraphAsset::parse(GRAPH_TEXT, &resolve).unwrap())
        .unwrap();
}

/// Model-space position of a node in a pose.
pub fn node_position(pose: &[Trs], node: u32) -> Vec3 {
    let s = skeleton();
    s.model_space(pose)[node as usize].translation
}

/// Matrix close to identity, element-wise.
pub fn near_identity(m: &Mat4, eps: f32) -> bool {
    m.to_cols_array()
        .iter()
        .zip(Mat4::IDENTITY.to_cols_array())
        .all(|(a, b)| (a - b).abs() <= eps)
}

// ---- the tall rig retargeting reads (tests/retarget.rs, tools/wasm-determinism) ----

/// (name, parent, model position in metres, model rotation)
pub type NodeDef = (&'static str, u32, Vec3, Quat);

pub fn rz(deg: f32) -> Quat {
    math::quat_from_axis_angle(Vec3::Z, deg.to_radians())
}

/// A skeleton from model-space rest transforms under a root scaled by `scale`.
pub fn skeleton_from_model(defs: &[NodeDef], scale: f32) -> SkeletonAsset {
    let mut model: Vec<Trs> = Vec::new();
    let mut nodes = Vec::new();
    for (i, &(name, parent, p, r)) in defs.iter().enumerate() {
        let m = if i == 0 {
            Trs::new(Vec3::ZERO, Quat::IDENTITY, Vec3::splat(scale))
        } else {
            Trs::new(p, r, Vec3::splat(scale))
        };
        let local = if parent == NONE {
            m
        } else {
            let pm = model[parent as usize];
            let l = pm.inverse().mul(&m);
            Trs::new(l.translation, l.rotation, Vec3::ONE)
        };
        model.push(m);
        nodes.push(SkeletonNode {
            name: name.into(),
            parent,
            rest: local,
            mesh: NONE,
        });
    }
    let mut s = SkeletonAsset {
        nodes,
        ..Default::default()
    };
    let mm = s.model_matrices(&s.rest_pose());
    let joints: Vec<u32> = (1..s.nodes.len() as u32).collect();
    let inverse_bind = joints.iter().map(|&j| mm[j as usize].inverse()).collect();
    s.skins.push(Skin {
        name: "tall".into(),
        joints,
        inverse_bind,
    });
    s
}

/// The retargeting target: Mixamo names, an armature scaled by 0.01,
/// Mixamo-style rest rotations (+Y along each bone), A-posed arms, and
/// thigh 0.70, shin 0.44, hips at 1.30: a leg 1.14 m against the walker's 0.90.
pub fn tall() -> SkeletonAsset {
    let down = rz(180.0);
    let toe = rotation_to(Vec3::new(0.0, -0.12, 0.12));
    let defs: Vec<NodeDef> = vec![
        ("Armature", NONE, Vec3::ZERO, Quat::IDENTITY),
        (
            "mixamorig:Hips",
            0,
            Vec3::new(0.0, 1.3, 0.0),
            Quat::IDENTITY,
        ),
        (
            "mixamorig:Spine",
            1,
            Vec3::new(0.0, 1.5, 0.0),
            Quat::IDENTITY,
        ),
        (
            "mixamorig:Spine1",
            2,
            Vec3::new(0.0, 1.65, 0.0),
            Quat::IDENTITY,
        ),
        (
            "mixamorig:Neck",
            3,
            Vec3::new(0.0, 1.85, 0.0),
            Quat::IDENTITY,
        ),
        (
            "mixamorig:Head",
            4,
            Vec3::new(0.0, 1.97, 0.0),
            Quat::IDENTITY,
        ),
        (
            "mixamorig:LeftShoulder",
            3,
            Vec3::new(0.08, 1.8, 0.0),
            rz(-90.0),
        ),
        ("mixamorig:LeftArm", 6, Vec3::new(0.2, 1.8, 0.0), rz(-135.0)),
        (
            "mixamorig:LeftForeArm",
            7,
            Vec3::new(
                0.2 + 0.3 * std::f32::consts::FRAC_1_SQRT_2,
                1.8 - 0.3 * std::f32::consts::FRAC_1_SQRT_2,
                0.0,
            ),
            rz(-135.0),
        ),
        (
            "mixamorig:LeftHand",
            8,
            Vec3::new(
                0.2 + 0.55 * std::f32::consts::FRAC_1_SQRT_2,
                1.8 - 0.55 * std::f32::consts::FRAC_1_SQRT_2,
                0.0,
            ),
            rz(-135.0),
        ),
        ("mixamorig:LeftUpLeg", 1, Vec3::new(0.12, 1.3, 0.0), down),
        ("mixamorig:LeftLeg", 10, Vec3::new(0.12, 0.6, 0.0), down),
        ("mixamorig:LeftFoot", 11, Vec3::new(0.12, 0.16, 0.0), toe),
        (
            "mixamorig:LeftToeBase",
            12,
            Vec3::new(0.12, 0.04, 0.12),
            toe,
        ),
        ("mixamorig:RightUpLeg", 1, Vec3::new(-0.12, 1.3, 0.0), down),
        ("mixamorig:RightLeg", 14, Vec3::new(-0.12, 0.6, 0.0), down),
        ("mixamorig:RightFoot", 15, Vec3::new(-0.12, 0.16, 0.0), toe),
        (
            "mixamorig:RightToeBase",
            16,
            Vec3::new(-0.12, 0.04, 0.12),
            toe,
        ),
    ];
    skeleton_from_model(&defs, 0.01)
}

/// The rotation taking +Y to `dir`.
pub fn rotation_to(dir: Vec3) -> Quat {
    og_anim::rotation_arc(Vec3::Y, dir)
}
