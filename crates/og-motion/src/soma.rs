//! The generated skeleton for og-anim: SOMA's 30 model joints as a
//! [`SkeletonAsset`] (`soma` names, which [`og_anim::HumanoidMap`] reads),
//! generated poses as its local transforms, feet planted from the model's
//! contacts, and a run of frames as a [`ClipAsset`] that
//! [`og_anim::retarget`] puts on a game's rig.
//!
//! The rest pose is the bundle's `skeleton.*` tensors (og-motion-train's
//! `scripts/export.py`): each joint's parent among the 30 and its local rest
//! translation and rotation, the frames the model's rotations are in. A
//! generated pose is shown by its hips' position and every joint's rotation
//! on the rest bone lengths (the model's own joint positions agree with them
//! within its error, and a skeleton cannot stretch).

use og_anim::{Channel, ChannelPath, ClipAsset, NONE, SkeletonAsset, SkeletonNode, Trs, two_bone};
use og_core::glam::{Mat3 as GMat3, Quat, Vec3 as GVec3};

use crate::bundle::{Bundle, BundleError};
use crate::contract::{JOINT_NAMES, JOINTS, Mat3, Pose, SAMPLE_RATE};

/// A contact above this is the foot on the ground.
pub const CONTACT: f32 = 0.5;
/// Frames over which a planted foot rejoins its generated path when it lifts.
pub const RELEASE_FRAMES: usize = 4;

/// The legs: (thigh, shin, ankle) joints and the ankle's contact index.
const LEGS: [([usize; 3], usize); 2] = [([22, 23, 24], 0), ([26, 27, 28], 2)];

fn quat(m: &Mat3) -> Quat {
    let col = |c: usize| GVec3::new(m[0][c], m[1][c], m[2][c]);
    Quat::from_mat3(&GMat3::from_cols(col(0), col(1), col(2))).normalize()
}

/// SOMA's rest pose.
#[derive(Clone, Debug, PartialEq)]
pub struct Skeleton {
    /// Each joint's parent ([`NONE`] for the hips), parents first.
    pub parents: [u32; JOINTS],
    /// Local rest transforms, metres.
    pub rest: [Trs; JOINTS],
}

impl Skeleton {
    /// The rest pose a bundle carries.
    pub fn from_bundle(bundle: &Bundle) -> Result<Skeleton, BundleError> {
        let parents = bundle.get("skeleton.parents", &[JOINTS])?;
        let offsets = bundle.get("skeleton.offsets", &[JOINTS, 3])?;
        let rotations = bundle.get("skeleton.rotations", &[JOINTS, 3, 3])?;
        Ok(Skeleton {
            parents: std::array::from_fn(|j| {
                if parents[j] < 0.0 {
                    NONE
                } else {
                    parents[j] as u32
                }
            }),
            rest: std::array::from_fn(|j| {
                let m: Mat3 =
                    std::array::from_fn(|r| std::array::from_fn(|c| rotations[j * 9 + r * 3 + c]));
                Trs::new(
                    GVec3::from_slice(&offsets[j * 3..j * 3 + 3]),
                    quat(&m),
                    GVec3::ONE,
                )
            }),
        })
    }

    /// The skeleton as og-anim's asset: one node per joint, named as
    /// [`JOINT_NAMES`], no mesh.
    pub fn asset(&self) -> SkeletonAsset {
        SkeletonAsset {
            nodes: (0..JOINTS)
                .map(|j| SkeletonNode {
                    name: JOINT_NAMES[j].into(),
                    parent: self.parents[j],
                    rest: self.rest[j],
                    mesh: NONE,
                })
                .collect(),
            ..Default::default()
        }
    }

    /// A generated world pose as local transforms: the hips where the pose
    /// has them, every joint turned as the pose turns it, on the rest
    /// bone lengths.
    pub fn local(&self, pose: &Pose) -> Vec<Trs> {
        let world: Vec<Quat> = pose.rotations.iter().map(quat).collect();
        (0..JOINTS)
            .map(|j| match self.parents[j] {
                NONE => Trs::new(GVec3::from_array(pose.positions[j]), world[j], GVec3::ONE),
                p => Trs::new(
                    self.rest[j].translation,
                    (world[p as usize].conjugate() * world[j]).normalize(),
                    GVec3::ONE,
                ),
            })
            .collect()
    }
}

/// Plants the feet of a run of local poses (consecutive frames at 30 Hz of
/// one character): while the model reports a heel on the ground its ankle
/// stays where it touched down, the leg solved by two-bone IK and the
/// foot keeping its generated orientation; when the heel lifts, the ankle
/// rejoins its generated path over [`RELEASE_FRAMES`]. `contacts` holds the
/// model's four per frame (left heel, left toe, right heel, right toe).
pub fn plant_feet(skeleton: &SkeletonAsset, poses: &mut [Vec<Trs>], contacts: &[[f32; 4]]) {
    debug_assert_eq!(poses.len(), contacts.len());
    for (chain, contact) in LEGS {
        let ankle = chain[2];
        let generated: Vec<GVec3> = poses
            .iter()
            .map(|p| skeleton.model_space(p)[ankle].translation)
            .collect();
        let mut anchor: Option<GVec3> = None;
        let mut released = RELEASE_FRAMES;
        let mut last = GVec3::ZERO;
        for (f, pose) in poses.iter_mut().enumerate() {
            let down = contacts[f][contact] > CONTACT;
            let target = if down {
                let at = *anchor.get_or_insert(generated[f]);
                released = 0;
                last = at;
                at
            } else {
                anchor = None;
                if released >= RELEASE_FRAMES {
                    continue;
                }
                released += 1;
                let k = released as f32 / (RELEASE_FRAMES + 1) as f32;
                last + (generated[f] - last) * k
            };
            let model = skeleton.model_space(pose);
            let foot = model[ankle].rotation;
            let joints = chain.map(|j| j as u32);
            two_bone(skeleton, pose, &joints, target, None, 1.0, &[]);
            let shin = skeleton.model_space(pose)[chain[1]].rotation;
            pose[ankle].rotation = (shin.conjugate() * foot).normalize();
        }
    }
}

/// A run of local poses (consecutive frames at 30 Hz) as a clip of the
/// skeleton: the hips' translation and every joint's rotation.
pub fn clip(name: &str, skeleton: &SkeletonAsset, poses: &[Vec<Trs>]) -> ClipAsset {
    assert!(!poses.is_empty(), "a clip needs a frame");
    let duration = (poses.len() - 1) as f32 / SAMPLE_RATE;
    let frames = ClipAsset::frames_for(duration, SAMPLE_RATE);
    let at = |i: u32| &poses[(i as usize).min(poses.len() - 1)];
    let channel = |node: usize, path: ChannelPath| {
        let lanes = if path == ChannelPath::Rotation { 4 } else { 3 };
        let samples: Vec<f32> = (0..frames)
            .flat_map(|i| match path {
                ChannelPath::Rotation => at(i)[node].rotation.to_array().to_vec(),
                _ => at(i)[node].translation.to_array().to_vec(),
            })
            .collect();
        Channel {
            node: node as u32,
            node_name: skeleton.nodes[node].name.clone(),
            path,
            step: false,
            lanes,
            samples: samples.into(),
        }
    };
    let mut channels = vec![channel(0, ChannelPath::Translation)];
    channels.extend((0..JOINTS).map(|j| channel(j, ChannelPath::Rotation)));
    ClipAsset {
        name: name.into(),
        duration,
        rate: SAMPLE_RATE,
        source_rate: SAMPLE_RATE,
        frames,
        channels,
        events: Vec::new(),
    }
}
