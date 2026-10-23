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
//! | [`Flipbook`], [`FlipbookEvent`], [`FlipbookSheet`], [`install_flipbook`] | sprite animation: sheets of frames, clips with fps, loop / ping-pong / once and frame events, played by `og.flipbook` |
//! | [`SkeletonAsset`], [`ClipAsset`], [`GraphAsset`] | the assets (the cook's `<model>#anim` blob, an `animgraph` `.ogt`) |
//! | [`HumanoidMap`], [`RigAsset`], [`retarget`] | humanoid bone maps (detected from Mixamo, VRM, Unity, Rigify, Unreal or Blender names, or a `type rig` .ogt) and clips rewritten from one rig to another (rest correction, bone lengths, scaled root travel, foot IK) |
//! | [`Rig2dAsset`], [`Skeleton2D`], [`Skeleton2DEvent`], [`IkTarget2D`], [`install_skeleton2d`] | 2D skeletal animation: the `type rig2d` asset (bones, slots, region and weighted-mesh attachments, IK, clips with bone, slot and deform keys), played by `og.skeleton2d` and drawn as triangles |
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
mod flipbook;
mod graph;
mod ik;
mod pose;
mod retarget;
mod rig2d;
mod schema;
mod skeleton2d;
mod system;
mod xform;

pub use asset::{
    Channel, ChannelPath, ClipAsset, ClipEvent, FramePos, NONE, SkeletonAsset, SkeletonNode, Skin,
    bind,
};
pub use flipbook::{
    FLIPBOOK_SYSTEM, FlipClip, FlipMode, Flipbook, FlipbookEvent, FlipbookSheet, FlipbookSystem,
    SheetFilter, SheetFrame, install_flipbook,
};
pub use graph::{
    Condition, GraphAsset, Layer, LayerMode, Motion, MotionNode, Op, State, Transition,
};
pub use ik::{IkResult, chain as ik_chain, fabrik, two_bone};
pub use pose::{Pose, sample, stepped};
pub use retarget::{
    HUMAN_BONE_COUNT, HUMAN_BONES, HumanoidMap, RetargetOptions, Retargeted, RigAsset, SCHEMES,
    human_bone, retarget,
};
pub use rig2d::{
    Affine2, Attachment2d, AttachmentKind, Bone2d, BoneTimeline, Clip2d, DeformTimeline, Ik2d,
    Influence, Key, MeshVertex2d, Pose2d, RIG2D_TYPE, Rig2dAsset, RigVertex, Slot2d, SlotTimeline,
};
pub use schema::*;
pub use skeleton2d::{
    IkTarget2D, SKELETON2D_SYSTEM, Skeleton2D, Skeleton2DEvent, Skeleton2dSystem,
    install_skeleton2d, skeleton_frame, skeleton_pose,
};
pub use system::{AnimOutput, AnimationSystem, JointPalette, PaletteSink, SYSTEM_NAME, install};
pub use xform::{Trs, nlerp, rotation_arc, twist_y, unit};
