//! Origin Engine animation (docs/design.md §2.7): glTF skins and clips, a
//! blend-tree graph asset with layered state machines, root motion,
//! two-bone and FABRIK IK and blendshapes, evaluated by one deterministic
//! ECS system, `og.anim`.
//!
//! | item | what it is |
//! |---|---|
//! | [`install`] | registers the types, schedules `og.anim` and adds its state block; returns an [`AnimationSystem`] |
//! | [`Skeleton`], [`AnimationPlayer`], [`AnimationGraph`], [`Blendshapes`], [`IkChain`] | the components (`og.Skeleton`, …) |
//! | [`AnimationEvent`] | a clip marker passed in the last tick |
//! | [`SkeletonAsset`], [`ClipAsset`], [`GraphAsset`] | the assets (the cook's `<model>#anim` blob, an `animgraph` `.ogt`) |
//! | [`AnimationSystem`] | asset registration, the final pose, the per-entity [`AnimOutput`] (joint palette, blendshape weights) and [`PaletteSink`], the renderer contract |
//!
//! **Determinism.** f32 with `+ - * /` and `sqrt` only (no trigonometry:
//! rotations blend by normalised lerp, yaw is a swing-twist split), fixed
//! iteration orders (query order, then entity id), no clocks: the pose at a
//! tick is a function of the world, bit-identical on every host. Player
//! times, graph states, phases and cross-fades, and every entity's final
//! pose, live in the `og.anim` state block, so `snapshot`, `restore` and
//! `world_hash` cover them; a restored world continues exactly as the
//! uninterrupted run.

mod asset;
mod codec;
mod graph;
mod ik;
mod pose;
mod schema;
mod system;
mod xform;

pub use asset::{
    bind, Channel, ChannelPath, ClipAsset, ClipEvent, FramePos, SkeletonAsset, SkeletonNode, Skin,
    NONE,
};
pub use graph::{
    Condition, GraphAsset, Layer, LayerMode, Motion, MotionNode, Op, State, Transition,
};
pub use ik::{chain as ik_chain, fabrik, two_bone, IkResult};
pub use pose::{sample, Pose};
pub use schema::*;
pub use system::{install, AnimOutput, AnimationSystem, JointPalette, PaletteSink, SYSTEM_NAME};
pub use xform::{nlerp, rotation_arc, twist_y, unit, Trs};
