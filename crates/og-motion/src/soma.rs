//! The generated skeleton for og-anim: SOMA's 30 model joints as a
//! [`SkeletonAsset`] (`soma` names, which [`og_anim::HumanoidMap`] reads),
//! generated poses as its local transforms, feet planted by contacts (and,
//! given the ground, put down on it and stepped), and a run of frames as a
//! [`ClipAsset`] that
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
/// A released foot on the ground ([`Planter::step`] given the ground)
/// steps to where the pose has it at this speed, m/s (at least over
/// [`RELEASE_FRAMES`]), lifted up to [`STEP_LIFT`] at the middle of the
/// step: a foot that has to move along the ground is picked up.
pub const STEP_SPEED: f32 = 1.4;
pub const STEP_LIFT: f32 = 0.07;
/// The share of a step at either end spent lifting and setting the foot
/// down, without moving it across.
const STEP_RISE: f32 = 0.15;
/// The share of a step after which where it lands is fixed.
const STEP_AIM: f32 = 0.6;
/// The most the hips are lowered for the planted feet, a share of the
/// leg's length, and how fast, leg lengths per second.
const MAX_DROP: f32 = 0.15;
const DROP_SPEED: f32 = 0.6;
/// The longest step, s.
const MAX_STEP: f32 = 0.35;
/// A planted foot this far (a share of its leg's length) across the
/// ground from where the pose has it steps there, one foot at a time: a
/// turn in place, or a body carried on faster than its feet.
pub const STRETCH: f32 = 0.22;
/// A free foot swinging over the ground faster than [`SWING_SPEED`] m/s
/// is held up to [`SWING_LIFT`] m over its rest height (full at twice that
/// speed): a swing clears the ground instead of skimming it, and comes down
/// as it slows to land.
pub const SWING_SPEED: f32 = 0.15;
pub const SWING_LIFT: f32 = 0.04;
/// The share of a leg's length over which its reach softens before full
/// extension (soft IK): a planted ankle moving that close to straight-leg
/// reach is followed more and more slowly, so the knee straightens
/// smoothly instead of snapping straight in the last millimetres.
pub const SOFT_REACH: f32 = 0.02;
/// A generated knee this far (a share of the thigh) off the hip-to-ankle
/// line names the leg's bend plane; a straighter leg keeps the last one.
const BEND_PLANE: f32 = 0.02;

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
    let mut planter = Planter::new(
        LEGS.iter()
            .map(|&(chain, contact)| Leg {
                chain: chain.map(|j| j as u32),
                contact,
                rest: 0.0,
            })
            .collect(),
    );
    for (pose, contact) in poses.iter_mut().zip(contacts) {
        planter.step(
            skeleton,
            pose,
            &Trs::IDENTITY,
            contact,
            None,
            1.0 / SAMPLE_RATE,
        );
    }
}

/// A leg [`Planter`] plants: its thigh, shin and ankle nodes, which of
/// the contacts is its heel (0 left, 2 right), and its ankle's height over
/// the ground at rest in model space (what a planted foot is put down to).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Leg {
    pub chain: [u32; 3],
    pub contact: usize,
    pub rest: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Plant {
    /// Where the ankle touched down, in the world.
    anchor: Option<GVec3>,
    /// Seconds since the foot was let go, and how long its step to the
    /// generated path takes.
    released: f32,
    step: f32,
    /// Where the ankle was last held, in the world.
    last: GVec3,
    /// Where the generated ankle was the frame before, in the world.
    was: Option<GVec3>,
    /// The way the knee bends off the hip-to-ankle line, in the world: the
    /// generated knee's, held while the generated leg is straight.
    bend: Option<GVec3>,
}

impl Plant {
    /// Whether the foot is partway through a step.
    fn stepping(&self) -> bool {
        self.released < self.step
    }

    /// Not planted, and long released: the ankle follows its generated path.
    const FREE: Plant = Plant {
        anchor: None,
        released: f32::INFINITY,
        step: 0.0,
        last: GVec3::ZERO,
        was: None,
        bend: None,
    };
}

/// Plants the feet of a character's frames one at a time, as
/// [`plant_feet`] does a run of them, on any skeleton whose legs it is
/// given, with the anchors held in the world: a character whose model
/// space moves under it (re-rooted onto a moving entity) keeps a planted
/// foot where it touched the ground.
#[derive(Clone, Debug, PartialEq)]
pub struct Planter {
    legs: Vec<(Leg, Plant)>,
    /// How far the hips are lowered so the planted feet can reach where
    /// they stand, in model space.
    drop: f32,
}

impl Planter {
    pub fn new(legs: Vec<Leg>) -> Planter {
        Planter {
            legs: legs.into_iter().map(|l| (l, Plant::FREE)).collect(),
            drop: 0.0,
        }
    }

    /// Each leg's ankle height over the ground at rest (model space).
    pub fn rest(&self) -> [f32; 2] {
        let mut r = [0.0; 2];
        for ((leg, _), r) in self.legs.iter().zip(&mut r) {
            *r = leg.rest;
        }
        r
    }

    /// Each leg's ankle anchor while it is planted (not stepping), in the
    /// world: where a planted foot stands.
    pub fn planted(&self) -> impl Iterator<Item = (&Leg, GVec3)> {
        self.legs
            .iter()
            .filter(|(_, p)| !p.stepping())
            .filter_map(|(l, p)| p.anchor.map(|a| (l, a)))
    }

    /// Lowers the hips (the thighs' parent) as far as a planted foot
    /// needs to reach where it stands within soft IK's easy reach, at most
    /// [`MAX_DROP`] of the leg's length, easing there and back at
    /// [`DROP_SPEED`] leg lengths a second: a wide or turned stance bends
    /// the knees instead of leaving a foot hanging over the ground.
    fn lower_hips(&mut self, skeleton: &SkeletonAsset, pose: &mut [Trs], to_model: &Trs, dt: f32) {
        let Some(&(first, _)) = self.legs.first() else {
            return;
        };
        let pelvis = skeleton.nodes[first.chain[0] as usize].parent;
        if pelvis == NONE {
            return;
        }
        let model = skeleton.model_space(pose);
        let mut need = 0.0f32;
        let mut length = 0.0f32;
        for (leg, plant) in &self.legs {
            let [hip, knee, ankle] = leg.chain.map(|j| j as usize);
            let l = (model[knee].translation - model[hip].translation).length()
                + (model[ankle].translation - model[knee].translation).length();
            length = length.max(l);
            let Some(at) = plant.anchor.filter(|_| !plant.stepping()) else {
                continue;
            };
            let reach = l * (1.0 - SOFT_REACH);
            let to = to_model.transform_point(at) - model[hip].translation;
            let across = GVec3::new(to.x, 0.0, to.z).length_squared();
            // The drop d with |to + d·up| = reach.
            if to.length() > reach && across < reach * reach {
                need = need.max(-to.y - (reach * reach - across).sqrt());
            }
        }
        let goal = need.clamp(0.0, MAX_DROP * length);
        let rate = DROP_SPEED * length * dt;
        self.drop += (goal - self.drop).clamp(-rate, rate);
        if self.drop > 0.0 {
            let parent = skeleton.nodes[pelvis as usize].parent;
            let down = GVec3::new(0.0, -self.drop, 0.0);
            let local = if parent == NONE {
                down
            } else {
                let p = model[parent as usize];
                p.rotation.inverse() * down / p.scale
            };
            pose[pelvis as usize].translation += local;
        }
    }

    /// Lets go of every planted foot at once (a cut: the character was moved).
    /// The knees' bend planes are kept: a cut does not turn a knee.
    pub fn release(&mut self) {
        for (_, plant) in &mut self.legs {
            *plant = Plant {
                bend: plant.bend,
                ..Plant::FREE
            };
        }
    }

    /// Plants one frame, `dt` s after the last: `pose` (local transforms of
    /// `skeleton`), placed in the world by `root` (model space to world,
    /// uniform scale), with the frame's four `contacts`. Given the height
    /// of the `ground` under the character, a foot is put down onto it (its
    /// ankle no higher than its leg's rest height over it), a foot let go
    /// on the ground steps to its generated path, lifted, and a planted
    /// foot left [`STRETCH`] behind its generated path steps there.
    pub fn step(
        &mut self,
        skeleton: &SkeletonAsset,
        pose: &mut [Trs],
        root: &Trs,
        contacts: &[f32; 4],
        ground: Option<f32>,
        dt: f32,
    ) {
        let to_model = root.inverse();
        if ground.is_some() {
            self.lower_hips(skeleton, pose, &to_model, dt);
        }
        let blend = (RELEASE_FRAMES + 1) as f32 / SAMPLE_RATE;
        let mut stepping = self.legs.iter().any(|(_, p)| p.stepping());
        for (leg, plant) in &mut self.legs {
            let [hip, knee, ankle] = leg.chain.map(|j| j as usize);
            let model = skeleton.model_space(pose);
            let generated = root.transform_point(model[ankle].translation);
            // The knee is a hinge: it bends in the generated knee's plane,
            // and a straight generated leg keeps the plane it last bent in
            // (never an arbitrary one, which bends the knee sideways or
            // backwards).
            let (a, b, c) = (
                model[hip].translation,
                model[knee].translation,
                model[ankle].translation,
            );
            let thigh = (b - a).length();
            let along = (c - a).normalize_or_zero();
            let off = (b - a) - along * (b - a).dot(along);
            if off.length() > BEND_PLANE * thigh {
                plant.bend = Some(root.rotation * off.normalize());
            }
            let length = thigh + (c - b).length();
            let scale = root.scale.x;
            let floor = ground.map(|g| g + leg.rest * scale);
            let down = contacts[leg.contact] > CONTACT;
            let put_down = |at: GVec3| match floor {
                Some(f) => GVec3::new(at.x, f, at.z),
                None => at,
            };
            let mut in_step = plant.stepping();
            // A planted foot lifts (its contact ends), or is left behind
            // its generated path and steps there unless the other foot is
            // stepping: either way it steps from where it stood.
            if let Some(at) = plant.anchor
                && !in_step
            {
                let behind = floor.is_some()
                    && !stepping
                    && GVec3::new(at.x - generated.x, 0.0, at.z - generated.z).length()
                        > STRETCH * length * scale;
                if !down || behind {
                    plant.anchor = None;
                    plant.last = at;
                    plant.released = 0.0;
                    plant.step = match floor {
                        Some(_) => (at - generated).length() / (STEP_SPEED * scale),
                        None => 0.0,
                    }
                    .clamp(blend, MAX_STEP);
                    in_step = true;
                    stepping = true;
                }
            }
            let was = plant.was.replace(generated);
            let target = if !in_step {
                if !down {
                    // A swing skimming the ground is lifted clear of it.
                    let (Some(f), Some(was)) = (floor, was) else {
                        continue;
                    };
                    let speed = GVec3::new(generated.x - was.x, 0.0, generated.z - was.z).length()
                        / (dt * scale);
                    let k = ((speed - SWING_SPEED) / SWING_SPEED).clamp(0.0, 1.0);
                    let clear = f + SWING_LIFT * scale * k;
                    if k <= 0.0 || generated.y >= clear {
                        continue;
                    }
                    GVec3::new(generated.x, clear, generated.z)
                } else {
                    let at = *plant.anchor.get_or_insert_with(|| put_down(generated));
                    plant.last = at;
                    at
                }
            } else {
                // A stepping foot heads for where the pose has it and,
                // from [`STEP_AIM`] of the way through, comes down where
                // the pose has it then: it is planted there once the step
                // is over.
                let to = if down {
                    if plant.released < STEP_AIM * plant.step {
                        plant.anchor = None;
                        put_down(generated)
                    } else {
                        *plant.anchor.get_or_insert_with(|| put_down(generated))
                    }
                } else {
                    plant.anchor = None;
                    generated
                };
                plant.released += dt;
                if plant.released >= plant.step - 1e-6 {
                    plant.released = f32::INFINITY;
                    if !down {
                        continue;
                    }
                    plant.last = to;
                    to
                } else {
                    stepped(plant.last, to, plant.released / plant.step, floor, scale)
                }
            };
            let foot = model[ankle].rotation;
            let target = soft_reach(
                a,
                to_model.transform_point(target),
                thigh + (c - b).length(),
            );
            let pole = plant.bend.map(|d| a + to_model.rotation * d);
            two_bone(skeleton, pose, &leg.chain, target, pole, 1.0, &[]);
            let shin = skeleton.model_space(pose)[leg.chain[1] as usize].rotation;
            pose[ankle].rotation = (shin.conjugate() * foot).normalize();
        }
    }
}

/// Where a foot stepping from `from` to `to` is `k` (0 to 1) of the way
/// through the step: on the `ground` (its rest height there) the foot
/// lifts, moves across while up and comes straight down; elsewhere it
/// blends across.
fn stepped(from: GVec3, to: GVec3, k: f32, ground: Option<f32>, scale: f32) -> GVec3 {
    let Some(f) = ground else {
        return from + (to - from) * k;
    };
    let across = ((k - STEP_RISE) / (1.0 - 2.0 * STEP_RISE)).clamp(0.0, 1.0);
    let mut at = from + (to - from) * (across * across * (3.0 - 2.0 * across));
    if to.y < f + STEP_LIFT * scale {
        let far = ((from - to).length() / (0.04 * scale)).min(1.0);
        let up = og_core::math::sin(std::f32::consts::PI * k);
        at.y = at.y.max(f) + STEP_LIFT * scale * far * up;
    }
    at
}

/// `target` drawn toward `hip` so a leg of `length` reaches it with soft IK
/// (Andy Nicholas, "Soft IK", 2009): within dₛ = `length` × (1 −
/// [`SOFT_REACH`]) it is unchanged; beyond, its distance d is eased to
/// dₛ + r·(1 − e^(−(d − dₛ)/r)), r = `length` − dₛ, which reaches `length`
/// only at infinity. The knee straightens smoothly as a planted ankle moves
/// out of reach, the ankle then short of it by at most r/e at full extension.
fn soft_reach(hip: GVec3, target: GVec3, length: f32) -> GVec3 {
    let room = SOFT_REACH * length;
    let start = length - room;
    let d = (target - hip).length();
    if d <= start || room <= 0.0 {
        return target;
    }
    let eased = start + room * (1.0 - og_core::math::exp(-(d - start) / room));
    hip + (target - hip) * (eased / d)
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
