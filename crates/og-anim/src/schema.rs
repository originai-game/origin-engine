//! The reflected types animation reads and writes. Their doc comments are
//! what the inspector, the probe, scene text and the TypeScript IDL show.
//!
//! Asset fields (`asset`, `clip`) are handles the host registered with
//! [`crate::AnimationSystem`]; 0 means none. Joint fields index the skin's
//! `joints` list (glTF `skin.joints`), so `0` is the skin's first joint.

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
/// `IkChain.space`: `target` and `pole` are world positions (relative to the skeleton entity's Transform).
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
    /// Where the tip should be.
    #[reflect(unit = "m")]
    pub target: Vec3,
    /// A point the middle joint bends towards (two-bone); the zero vector keeps the animated bend.
    #[reflect(unit = "m")]
    pub pole: Vec3,
    /// What `target` and `pole` are relative to: world (the skeleton entity's Transform applies) or model.
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
