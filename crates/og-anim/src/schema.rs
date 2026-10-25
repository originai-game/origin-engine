//! The reflected types animation reads and writes. Their doc comments are
//! what the inspector, the probe, scene text and the TypeScript IDL show.
//!
//! Asset fields (`asset`, `clip`) are handles the host registered with
//! [`crate::AnimationSystem`]; 0 means none. Joint fields index the skin's
//! `joints` list (glTF `skin.joints`), so `0` is the skin's first joint.

// 3.1416 and 1.5708 are the IDL's angle bounds (og.LookAt, og.JointLimit), a
// hair above pi and pi/2 so the default ranges hold nothing; the derive
// writes them into generated code an item attribute does not reach.
#![allow(clippy::approx_constant)]

use og_core::glam::{Quat, Vec3};
use og_core::{Reflect, Str64};

/// Where an entity is, relative to its parent (or the world). The same
/// schema as the engine's `og.Transform`; a world that registered it from
/// the IDL (or from og-physics) adopts this type. Root motion moves it.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Transform")]
pub struct Transform {
    /// The position, as three.js Object3D.position.
    #[reflect(unit = "m")]
    pub position: Vec3,
    /// The rotation, as three.js Object3D.quaternion.
    #[reflect(default = [0, 0, 0, 1])]
    pub quaternion: Quat,
    /// The scale, as three.js Object3D.scale.
    #[reflect(default = [1, 1, 1])]
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            position: Vec3::ZERO,
            quaternion: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    /// An unrotated, unscaled transform at `position`.
    pub fn at(position: Vec3) -> Transform {
        Transform {
            position,
            ..Transform::default()
        }
    }
}

/// `Skeleton.rootMotion`: the root joint animates in place; the entity does not move.
pub const ROOT_MOTION_OFF: u32 = 0;
/// `Skeleton.rootMotion`: the root's horizontal (xz) travel and yaw move the entity's Transform.
pub const ROOT_MOTION_PLANAR: u32 = 1;
/// `Skeleton.rootMotion`: the root's whole translation and rotation move the entity's Transform.
pub const ROOT_MOTION_FULL: u32 = 2;

/// A posed skeleton: the entity is drawn skinned by the joint palette the
/// animation system writes each tick. The pose itself (every node's local
/// transform) is world state kept by the `og.anim` state block.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.Skeleton")]
pub struct Skeleton {
    /// The skeleton asset (the rig of a loaded glTF); 0 is none.
    pub asset: u64,
    /// Which of the asset's skins the joint palette is for (glTF skin index).
    #[reflect(min = 0)]
    pub skin: u32,
    /// Root motion extraction: off (animate in place), planar (xz travel and yaw move the Transform) or full.
    #[reflect(name = "rootMotion", variants = ["off", "planar", "full"])]
    pub root_motion: u32,
    /// The joint whose motion is extracted (an index into the skin's joints; 0 is its first joint, usually the hips).
    #[reflect(name = "rootJoint", min = 0)]
    pub root_joint: u32,
    /// Written: the number of joints in the palette (0 until the first tick).
    #[reflect(name = "jointCount")]
    pub joint_count: u32,
}

/// Plays one clip on a skeleton. Alone it poses the skeleton from its rest
/// pose; with an `og.AnimationGraph` it plays over the graph's result with
/// `weight`. Changing `clip` cross-fades from the previous one over `fade`.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.AnimationPlayer")]
pub struct AnimationPlayer {
    /// The clip asset; 0 plays nothing.
    pub clip: u64,
    /// The playback position in seconds; written every tick, set it to seek.
    #[reflect(unit = "s", min = 0)]
    pub time: f32,
    /// Playback rate (1 = as authored, negative plays backwards).
    #[reflect(min = -10, max = 10, default = 1, tunable)]
    pub speed: f32,
    /// Wrap at the end of the clip; otherwise hold the last frame and set `finished`.
    #[reflect(name = "loop", default = 1)]
    pub looping: bool,
    /// How much of the pose this player decides (0 = none, 1 = all).
    #[reflect(min = 0, max = 1, default = 1, tunable)]
    pub weight: f32,
    /// Cross-fade duration in seconds when `clip` changes.
    #[reflect(unit = "s", min = 0, max = 10, default = 0.2, tunable)]
    pub fade: f32,
    /// Stepped playback: the pose is held for 1/stepFps seconds, sampled at
    /// the clip time floored to that step (stop motion "on twos" at 12);
    /// 0 samples every tick. Time, markers and root motion stay continuous.
    #[reflect(name = "stepFps", min = 0, max = 60, tunable)]
    pub step_fps: f32,
    /// Written: a non-looping clip reached its end.
    pub finished: bool,
}

impl Default for AnimationPlayer {
    fn default() -> Self {
        AnimationPlayer {
            clip: 0,
            time: 0.0,
            speed: 1.0,
            looping: true,
            weight: 1.0,
            fade: 0.2,
            step_fps: 0.0,
            finished: false,
        }
    }
}

impl AnimationPlayer {
    /// A looping player of `clip`.
    pub fn new(clip: u64) -> AnimationPlayer {
        AnimationPlayer {
            clip,
            ..AnimationPlayer::default()
        }
    }
}

/// The number of graph parameters (`AnimationGraph.p0` … `p7`).
pub const GRAPH_PARAMS: usize = 8;

/// Runs a blend-tree asset (`.ogt` of type `animgraph`) on a skeleton: its
/// layers' state machines pick motions by the parameters `p0`…`p7`, which
/// the graph names (`param speed` is `p0`, the next `param` is `p1`, …).
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.AnimationGraph")]
pub struct AnimationGraph {
    /// The graph asset; 0 is none.
    pub asset: u64,
    /// How much of the pose the graph decides over the rest pose.
    #[reflect(min = 0, max = 1, default = 1, tunable)]
    pub weight: f32,
    /// A playback-rate multiplier for every layer.
    #[reflect(min = 0, max = 10, default = 1, tunable)]
    pub speed: f32,
    /// Parameter 0 (the graph's first `param`).
    #[reflect(tunable)]
    pub p0: f32,
    /// Parameter 1.
    #[reflect(tunable)]
    pub p1: f32,
    /// Parameter 2.
    #[reflect(tunable)]
    pub p2: f32,
    /// Parameter 3.
    #[reflect(tunable)]
    pub p3: f32,
    /// Parameter 4.
    #[reflect(tunable)]
    pub p4: f32,
    /// Parameter 5.
    #[reflect(tunable)]
    pub p5: f32,
    /// Parameter 6.
    #[reflect(tunable)]
    pub p6: f32,
    /// Parameter 7.
    #[reflect(tunable)]
    pub p7: f32,
    /// Written: the base layer's current state (its index in the graph file).
    pub state: u32,
    /// Written: seconds since the base layer entered `state`.
    #[reflect(name = "stateTime", unit = "s")]
    pub state_time: f32,
}

impl Default for AnimationGraph {
    fn default() -> Self {
        AnimationGraph {
            asset: 0,
            weight: 1.0,
            speed: 1.0,
            p0: 0.0,
            p1: 0.0,
            p2: 0.0,
            p3: 0.0,
            p4: 0.0,
            p5: 0.0,
            p6: 0.0,
            p7: 0.0,
            state: 0,
            state_time: 0.0,
        }
    }
}

impl AnimationGraph {
    /// A graph of `asset` with every parameter 0.
    pub fn new(asset: u64) -> AnimationGraph {
        AnimationGraph {
            asset,
            ..AnimationGraph::default()
        }
    }

    pub fn params(&self) -> [f32; GRAPH_PARAMS] {
        [
            self.p0, self.p1, self.p2, self.p3, self.p4, self.p5, self.p6, self.p7,
        ]
    }

    pub fn set_param(&mut self, slot: usize, value: f32) {
        match slot {
            0 => self.p0 = value,
            1 => self.p1 = value,
            2 => self.p2 = value,
            3 => self.p3 = value,
            4 => self.p4 = value,
            5 => self.p5 = value,
            6 => self.p6 = value,
            7 => self.p7 = value,
            _ => {}
        }
    }
}

/// The number of blendshape weights (`Blendshapes.w0` … `w7`).
pub const BLENDSHAPES: usize = 8;

/// Morph-target weights of the skeleton entity's mesh, in the order of the
/// glTF mesh's targets. With `source` clips the playing clips' weight
/// channels write them; with `component` game code does. The renderer reads
/// the result from the animation system's output every frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.Blendshapes")]
pub struct Blendshapes {
    /// Who decides the weights: component (game code writes w0…w7) or clips (the animation writes them).
    #[reflect(variants = ["component", "clips"])]
    pub source: u32,
    /// The glTF mesh index whose targets the weights drive (0 is the first mesh).
    #[reflect(min = 0)]
    pub mesh: u32,
    /// Weight of target 0.
    #[reflect(min = -1, max = 2, tunable)]
    pub w0: f32,
    /// Weight of target 1.
    #[reflect(min = -1, max = 2, tunable)]
    pub w1: f32,
    /// Weight of target 2.
    #[reflect(min = -1, max = 2, tunable)]
    pub w2: f32,
    /// Weight of target 3.
    #[reflect(min = -1, max = 2, tunable)]
    pub w3: f32,
    /// Weight of target 4.
    #[reflect(min = -1, max = 2, tunable)]
    pub w4: f32,
    /// Weight of target 5.
    #[reflect(min = -1, max = 2, tunable)]
    pub w5: f32,
    /// Weight of target 6.
    #[reflect(min = -1, max = 2, tunable)]
    pub w6: f32,
    /// Weight of target 7.
    #[reflect(min = -1, max = 2, tunable)]
    pub w7: f32,
}

/// `Blendshapes.source`: game code writes the weights.
pub const BLENDSHAPES_FROM_COMPONENT: u32 = 0;
/// `Blendshapes.source`: the playing clips' weight channels write them.
pub const BLENDSHAPES_FROM_CLIPS: u32 = 1;

impl Blendshapes {
    pub fn weights(&self) -> [f32; BLENDSHAPES] {
        [
            self.w0, self.w1, self.w2, self.w3, self.w4, self.w5, self.w6, self.w7,
        ]
    }

    pub fn set_weights(&mut self, w: [f32; BLENDSHAPES]) {
        [
            self.w0, self.w1, self.w2, self.w3, self.w4, self.w5, self.w6, self.w7,
        ] = w;
    }
}

/// `IkChain.solver`: analytic two-bone (thigh, shin, foot), bending towards `pole`.
pub const IK_TWO_BONE: u32 = 0;
/// `IkChain.solver`: FABRIK over `bones` bones.
pub const IK_FABRIK: u32 = 1;
/// `IkChain.space`: `target` and `pole` are in the frame the skeleton
/// entity's Transform is written in (its local transform is undone): true
/// world for a root entity, its parent's frame for a child.
pub const IK_WORLD: u32 = 0;
/// `IkChain.space`: `target` and `pole` are in the skeleton's model space.
pub const IK_MODEL: u32 = 1;

/// An inverse-kinematics constraint on a skeleton, applied after its clips
/// and graph: the chain of `bones` bones ending at joint `tip` turns so the
/// tip reaches `target`. Put it on its own entity (a skeleton can have many
/// chains); chains apply in query order.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.IkChain")]
pub struct IkChain {
    /// The entity with the og.Skeleton this chain bends.
    pub skeleton: u64,
    /// The end joint (an index into the skin's joints): the foot or the hand.
    #[reflect(min = 0)]
    pub tip: u32,
    /// How many bones above the tip turn (two-bone always uses 2).
    #[reflect(min = 1, max = 32, default = 2)]
    pub bones: u32,
    /// twoBone (analytic, with a pole) or fabrik (iterative, any length).
    #[reflect(variants = ["twoBone", "fabrik"])]
    pub solver: u32,
    /// Where the tip should be, in `space`: for `world`, the frame of the skeleton entity's og.Transform (its parent's frame when it has a parent, not true world).
    #[reflect(unit = "m")]
    pub target: Vec3,
    /// A point the middle joint bends towards (two-bone); the zero vector keeps the animated bend.
    #[reflect(unit = "m")]
    pub pole: Vec3,
    /// What `target` and `pole` are relative to: world (the frame the skeleton entity's og.Transform is in, so its parent's frame for a child entity, not true world) or model.
    #[reflect(variants = ["world", "model"])]
    pub space: u32,
    /// How much of the solved rotation is applied (0 = the animated pose, 1 = fully solved).
    #[reflect(min = 0, max = 1, default = 1, tunable)]
    pub weight: f32,
    /// FABRIK iterations at most.
    #[reflect(min = 1, max = 64, default = 10)]
    pub iterations: u32,
    /// The tip distance (model units) at which the target counts as reached.
    #[reflect(min = 0, default = 0.001, unit = "m")]
    pub tolerance: f32,
    /// Written: the tip is within `tolerance` of the target after solving.
    pub reached: bool,
    /// Written: the tip's distance from the target after solving, in model units.
    #[reflect(unit = "m")]
    pub error: f32,
}

impl Default for IkChain {
    fn default() -> Self {
        IkChain {
            skeleton: 0,
            tip: 0,
            bones: 2,
            solver: IK_TWO_BONE,
            target: Vec3::ZERO,
            pole: Vec3::ZERO,
            space: IK_WORLD,
            weight: 1.0,
            iterations: 10,
            tolerance: 0.001,
            reached: false,
            error: 0.0,
        }
    }
}

/// An aim constraint on a skeleton, applied after its clips, graph and IK
/// chains: `joint` turns so its `axis` points at `target` (three.js
/// Object3D.lookAt: +Z at the target, +Y towards `up`). Put it on its own
/// entity (a head, the eyes, a turret's barrel); constraints apply in query
/// order.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.LookAt")]
pub struct LookAt {
    /// The entity with the og.Skeleton this constraint turns.
    pub skeleton: u64,
    /// The joint that aims (an index into the skin's joints): the head, an eye.
    #[reflect(min = 0)]
    pub joint: u32,
    /// How many joints share the turn: `joint` and its parents (3 spreads a look over neck, chest and spine), each parent taking an even share and `joint` the rest.
    #[reflect(min = 1, max = 8, default = 1)]
    pub bones: u32,
    /// The point to look at, in `space`.
    #[reflect(unit = "m")]
    pub target: Vec3,
    /// What `target` and `up` are relative to: world (the frame the skeleton entity's og.Transform is in) or model.
    #[reflect(variants = ["world", "model"])]
    pub space: u32,
    /// The joint's forward, in its own frame: the axis turned onto the target.
    #[reflect(default = [0, 0, 1])]
    pub axis: Vec3,
    /// The joint's up, in its own frame: yaw turns about it, pitch towards it, and it rolls towards `up`.
    #[reflect(name = "upAxis", default = [0, 1, 0])]
    pub up_axis: Vec3,
    /// The up hint, a direction in `space` the joint's `upAxis` leans to (three.js Object3D.up); the zero vector keeps the animated roll.
    #[reflect(default = [0, 1, 0])]
    pub up: Vec3,
    /// The largest turn about `upAxis`, left or right of the animated forward.
    #[reflect(
        name = "maxYaw",
        unit = "rad",
        min = 0,
        max = 3.1416,
        default = 3.1416,
        tunable
    )]
    pub max_yaw: f32,
    /// The largest turn towards or away from `upAxis`, from the animated forward.
    #[reflect(
        name = "maxPitch",
        unit = "rad",
        min = 0,
        max = 1.5708,
        default = 1.5708,
        tunable
    )]
    pub max_pitch: f32,
    /// How much of the turn is applied (0 = the animated pose, 1 = aimed).
    #[reflect(min = 0, max = 1, default = 1, tunable)]
    pub weight: f32,
    /// Written: the target lay outside maxYaw or maxPitch, so the joint aims at the nearest direction inside them.
    pub clamped: bool,
}

impl Default for LookAt {
    fn default() -> Self {
        LookAt {
            skeleton: 0,
            joint: 0,
            bones: 1,
            target: Vec3::ZERO,
            space: IK_WORLD,
            axis: Vec3::Z,
            up_axis: Vec3::Y,
            up: Vec3::Y,
            max_yaw: 3.1416,
            max_pitch: 1.5708,
            weight: 1.0,
            clamped: false,
        }
    }
}

/// How far a joint may turn from its rest rotation when og.anim's solvers
/// (og.IkChain, og.LookAt, og.FootLock) turn it: a swing of the `axis`
/// within a cone and a twist about it within a range. A hinge (a knee, an
/// elbow) is `axis` along the hinge, `swing` 0 and the bend range as
/// `twistMin`…`twistMax`. Put it on its own entity, one per joint; the
/// animated pose itself is never clamped.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.JointLimit")]
pub struct JointLimit {
    /// The entity with the og.Skeleton whose joint this limits.
    pub skeleton: u64,
    /// The limited joint (an index into the skin's joints).
    #[reflect(min = 0)]
    pub joint: u32,
    /// The twist axis in the joint's rest frame: along the bone for a ball joint, along the hinge for a hinge.
    #[reflect(default = [0, 1, 0])]
    pub axis: Vec3,
    /// The cone half-angle the axis may swing away from its rest direction (0 locks it: a hinge; pi is free).
    #[reflect(unit = "rad", min = 0, max = 3.1416, default = 3.1416, tunable)]
    pub swing: f32,
    /// The least twist about `axis` from rest (a hinge's bend one way).
    #[reflect(name = "twistMin", unit = "rad", min = -3.1416, max = 3.1416, default = -3.1416, tunable)]
    pub twist_min: f32,
    /// The most twist about `axis` from rest (a hinge's bend the other way).
    #[reflect(name = "twistMax", unit = "rad", min = -3.1416, max = 3.1416, default = 3.1416, tunable)]
    pub twist_max: f32,
}

impl Default for JointLimit {
    fn default() -> Self {
        JointLimit {
            skeleton: 0,
            joint: 0,
            axis: Vec3::Y,
            swing: 3.1416,
            twist_min: -3.1416,
            twist_max: 3.1416,
        }
    }
}

/// Foot locking: while the playing clips plant `foot` (it is within 5 mm of
/// its lowest in each clip, weighed across blends and cross-fades), the foot
/// stays where it touched the ground, through clip blends and root motion,
/// its leg bent by two-bone IK; when the clips lift it the foot rejoins its
/// animation over `release`. Put it on its own entity, one per foot; locks
/// apply after root motion and before og.IkChain, in query order.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.FootLock")]
pub struct FootLock {
    /// The entity with the og.Skeleton whose foot this holds.
    pub skeleton: u64,
    /// The foot joint (an index into the skin's joints); the two bones above it are the leg that bends.
    #[reflect(min = 0)]
    pub foot: u32,
    /// How much of the lock is applied (0 = the animated foot, 1 = held).
    #[reflect(min = 0, max = 1, default = 1, tunable)]
    pub weight: f32,
    /// Seconds the foot takes to rejoin its animation once lifted (0: at once).
    #[reflect(unit = "s", min = 0, max = 2, default = 0.15, tunable)]
    pub release: f32,
    /// Farther than this from where the animation puts the foot, the lock lets go (a leg cannot stretch).
    #[reflect(name = "maxDistance", unit = "m", min = 0, default = 0.5, tunable)]
    pub max_distance: f32,
    /// Written: the playing clips plant the foot this tick.
    pub planted: bool,
    /// Written: the foot is held at its lock this tick.
    pub locked: bool,
}

impl Default for FootLock {
    fn default() -> Self {
        FootLock {
            skeleton: 0,
            foot: 0,
            weight: 1.0,
            release: 0.15,
            max_distance: 0.5,
            planted: false,
            locked: false,
        }
    }
}

/// `SpringBone.space`: the springs move in the frame the skeleton entity's
/// Transform is written in, so the chain lags behind the entity's motion.
pub const SPRING_WORLD: u32 = 0;
/// `SpringBone.space`: the springs move in the skeleton's model space, so
/// only the animation moves them (a teleport or a vehicle's motion does not).
pub const SPRING_MODEL: u32 = 1;

/// A spring (jiggle) bone chain (three-vrm's VRMSpringBone): `joint` and
/// its `bones` − 1 parents swing after their parents' motion. Each joint's
/// tail (the next joint of the chain, the last joint's first child, or, for
/// a joint without one, its parent bone continued at the same length) is a
/// point mass pulled towards where the animated pose puts it by a damped
/// spring, under `gravity`, held at the bone's length; the joint turns to
/// point at it. Applied after og.IkChain and og.LookAt, top joint first.
/// Put it on its own entity (a tail, an ear, a strand of hair, a single
/// jiggling node); chains apply in query order. The tails' positions and
/// velocities are world state (the `og.anim` state block).
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.SpringBone")]
pub struct SpringBone {
    /// The entity with the og.Skeleton whose joints this swings.
    pub skeleton: u64,
    /// The chain's last joint (an index into the skin's joints): the tip of a tail, an ear.
    #[reflect(min = 0)]
    pub joint: u32,
    /// How many joints swing: `joint` and its parents (1 is the joint alone).
    #[reflect(min = 1, max = 16, default = 1)]
    pub bones: u32,
    /// How fast a tail springs back to its animated place: the spring's natural frequency (0: no spring, it only hangs and swings).
    #[reflect(unit = "hz", min = 0, max = 30, default = 3, tunable)]
    pub frequency: f32,
    /// The damping ratio: 0 swings on, 1 settles without overshooting (critical damping).
    #[reflect(min = 0, max = 2, default = 0.3, tunable)]
    pub damping: f32,
    /// The last joint's frequency as a multiple of `frequency`, the joints between in proportion along the chain (1: every joint alike; 0.25: the tip springs back four times slower than the top, a soft strand on a stiff root).
    #[reflect(name = "tipFrequency", min = 0, max = 4, default = 1, tunable)]
    pub tip_frequency: f32,
    /// The last joint's damping ratio as a multiple of `damping`, the joints between in proportion along the chain (1: every joint alike).
    #[reflect(name = "tipDamping", min = 0, max = 4, default = 1, tunable)]
    pub tip_damping: f32,
    /// An acceleration on every tail, in `space` (three-vrm's gravityDir times gravityPower): (0, -9.8, 0) makes a chain droop.
    #[reflect(unit = "m/s2")]
    pub gravity: Vec3,
    /// Where the tails move: world (the frame the skeleton entity's og.Transform is in, so the chain lags behind the entity's motion) or model (only the animation moves it).
    #[reflect(variants = ["world", "model"])]
    pub space: u32,
    /// How much of the swing is applied (0 = the animated pose, 1 = the spring's).
    #[reflect(min = 0, max = 1, default = 1, tunable)]
    pub weight: f32,
    /// The tails' radius against og.SpringCollider (three-vrm's hitRadius): a tail stays this far outside every collider it meets (0: a point).
    #[reflect(unit = "m", min = 0, tunable)]
    pub radius: f32,
    /// The og.SpringCollider layers the tails are pushed out of, as bits: a collider on the same skeleton counts when its `layers` meet these.
    #[reflect(default = 4294967295)]
    pub colliders: u32,
    /// Written: the farthest a tail is from its animated place this tick.
    #[reflect(unit = "m")]
    pub lag: f32,
}

impl Default for SpringBone {
    fn default() -> Self {
        SpringBone {
            skeleton: 0,
            joint: 0,
            bones: 1,
            frequency: 3.0,
            damping: 0.3,
            tip_frequency: 1.0,
            tip_damping: 1.0,
            gravity: Vec3::ZERO,
            space: SPRING_WORLD,
            weight: 1.0,
            radius: 0.0,
            colliders: u32::MAX,
            lag: 0.0,
        }
    }
}

/// `SpringCollider.shape`: a ball of `radius` about `offset`.
pub const SPRING_SPHERE: u32 = 0;
/// `SpringCollider.shape`: a capsule of `radius` about the segment from
/// `offset` to `tail`.
pub const SPRING_CAPSULE: u32 = 1;

/// A collider spring bones are kept out of (three-vrm's
/// VRMSpringBoneCollider): a sphere or a capsule riding on a joint of a
/// skeleton (a head, a chest, an upper arm), so hair, ears and tails do not
/// swing through the body. Each tick, after a tail of an og.SpringBone of
/// the same skeleton has stepped and before its joint turns, the tail is
/// pushed out of every collider whose `layers` meet the spring's
/// `colliders` (in query order) by `radius` plus the spring's `radius`, and
/// put back at its bone's length. Put it on its own entity, one per shape;
/// colliders follow the pose at the moment a chain is swung, so a collider
/// on a joint an earlier chain swung follows that swing.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.SpringCollider")]
pub struct SpringCollider {
    /// The entity with the og.Skeleton whose spring bones this keeps out.
    pub skeleton: u64,
    /// The joint the collider rides on (an index into the skin's joints).
    #[reflect(min = 0)]
    pub joint: u32,
    /// A sphere about `offset`, or a capsule from `offset` to `tail`.
    #[reflect(variants = ["sphere", "capsule"])]
    pub shape: u32,
    /// The sphere's centre, or the capsule's first end, in the joint's frame.
    #[reflect(unit = "m")]
    pub offset: Vec3,
    /// The capsule's other end, in the joint's frame (a sphere ignores it).
    #[reflect(unit = "m")]
    pub tail: Vec3,
    /// The radius, in metres of the skeleton's model space (the joint's own scale does not apply).
    #[reflect(unit = "m", min = 0, default = 0.1, tunable)]
    pub radius: f32,
    /// The layers this collider is on, as bits (a spring bone's `colliders` picks them).
    #[reflect(default = 1)]
    pub layers: u32,
}

impl Default for SpringCollider {
    fn default() -> Self {
        SpringCollider {
            skeleton: 0,
            joint: 0,
            shape: SPRING_SPHERE,
            offset: Vec3::ZERO,
            tail: Vec3::ZERO,
            radius: 0.1,
            layers: 1,
        }
    }
}

/// A named marker of a clip that playback passed in the last tick (read in
/// the tick after it fired, like every event).
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.AnimationEvent")]
pub struct AnimationEvent {
    /// The skeleton entity.
    pub entity: u64,
    /// The clip asset the marker belongs to.
    pub clip: u64,
    /// The marker's name.
    pub name: Str64,
    /// The marker's time in the clip, in seconds.
    #[reflect(unit = "s")]
    pub time: f32,
    /// The clip's blend weight when it fired (cross-fades and blends fire every contributing clip's markers).
    pub weight: f32,
}
